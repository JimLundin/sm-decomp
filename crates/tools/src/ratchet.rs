//! `tools lockstep --check-ratchet LOCK CACHE ROM RATCHET`: lockstep over every row the ratchet
//! file names, held against its numbers ([`verify::ratchet`]).

use std::fmt::Write as _;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use verify::bundle::{content, rooms};
use verify::cache::Cache;
use verify::lock::{Lock, RowId};
use verify::lockstep::{Report, lockstep};
use verify::ratchet::{Summary, parse};

pub const USAGE: &str = "tools lockstep --check-ratchet LOCK CACHE ROM RATCHET";

/// Lockstep over one row's cache from `from` for `ticks` frames.
pub fn run_row(
    lock: &str,
    cache: &str,
    rom: &str,
    row: &str,
    window: std::ops::Range<u64>,
) -> Result<Report, String> {
    let text = std::fs::read_to_string(lock).map_err(|error| format!("{lock}: {error}"))?;
    let lock = Lock::parse(&text).map_err(|error| format!("{lock}: {error:?}"))?;
    let row = lock
        .row(&RowId(row.to_owned()))
        .ok_or_else(|| format!("{row}: not in the lock"))?;
    let cache = Cache::open(Path::new(cache), row).map_err(|error| error.to_string())?;
    let image = std::fs::read(rom).map_err(|error| format!("{rom}: {error}"))?;
    let content = Arc::new(content(image.clone()).map_err(|error| format!("{rom}: {error}"))?);
    let rooms = rooms(&content, &image).map_err(|error| format!("{rom}: {error:?}"))?;
    lockstep(&cache, &rooms, &content, window).map_err(|error| error.to_string())
}

/// Runs the check; the report when every row holds its numbers, the regressions otherwise.
pub fn check(args: &[String]) -> Result<String, String> {
    let [lock, cache, rom, ratchet] = args else {
        return Err(USAGE.to_owned());
    };
    let text = std::fs::read_to_string(ratchet).map_err(|error| format!("{ratchet}: {error}"))?;
    let baselines = parse(&text).map_err(|error| format!("{ratchet}: {error}"))?;
    let mut out = String::new();
    let mut regressions = Vec::new();
    let mut improved = String::new();
    let (mut compared, mut seconds) = (0_u64, 0.0_f64);
    for baseline in &baselines {
        let started = Instant::now();
        let report = run_row(lock, cache, rom, &baseline.row, 0..u64::MAX)?;
        seconds += started.elapsed().as_secs_f64();
        let now = Summary::of(&baseline.row, &report);
        compared = compared.saturating_add(now.ticks);
        let verdict = now.against(baseline);
        regressions.extend(verdict.regressions);
        if verdict.improved {
            let _ = write!(improved, "{now}");
        }
        let _ = write!(out, "{now}");
    }
    if !regressions.is_empty() {
        return Err(format!(
            "lockstep ratchet broken:\n{}",
            regressions.join("\n")
        ));
    }
    if !improved.is_empty() {
        let _ = write!(
            out,
            "improved; replace these rows in {ratchet}:\n{improved}"
        );
    }
    let rate = f64::from(u32::try_from(compared).unwrap_or(u32::MAX)) / seconds.max(f64::EPSILON);
    let _ = writeln!(
        out,
        "ratchet holds: {compared} Ticks in {seconds:.2} s ({rate:.0} Ticks/s)"
    );
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::check;

    /// The committed ratchet holds against the record cache in `SM_CACHE` and the ROM in
    /// `SM_ROM`; skipped where either isn't present.
    #[test]
    fn committed_ratchet_holds_against_the_cache() {
        let (Some(cache), Some(rom)) = (std::env::var_os("SM_CACHE"), std::env::var_os("SM_ROM"))
        else {
            return;
        };
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scenarios");
        let path = |p: std::path::PathBuf| p.to_string_lossy().into_owned();
        let args = [
            path(root.join("locks/lsnes.toml")),
            path(cache.into()),
            path(rom.into()),
            path(root.join("locks/lockstep.ratchet.txt")),
        ];
        assert!(check(&args).is_ok());
    }

    #[test]
    fn a_broken_ratchet_is_an_error() {
        let args = [
            "a".to_owned(),
            "b".to_owned(),
            "c".to_owned(),
            "d".to_owned(),
        ];
        assert!(check(&args).is_err());
    }
}
