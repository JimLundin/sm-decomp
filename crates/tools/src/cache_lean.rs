//! `tools cache-lean LOCK FROM TO ROW...`: each row's record cache under `FROM`, checked
//! against the lock at `LOCK`, written again as the lean tier under `TO`
//! (`verify::cache::Cache::write_lean`), with each row's bytes in both tiers. A cache taken
//! before the lean tier existed becomes one without playing its rows again. Existing lean
//! destinations are refused without changing them; full tiers can be converted in place.

use std::fmt::Write as _;
use std::path::Path;

use verify::cache::Cache;
use verify::lock::{Lock, RowId};

pub const USAGE: &str = "tools cache-lean LOCK FROM TO ROW...";

/// Writes the rows' lean tiers; the report, or why a row failed.
pub fn run(args: &[String]) -> Result<String, String> {
    let [lock_path, from, to, rows @ ..] = args else {
        return Err(USAGE.to_owned());
    };
    if rows.is_empty() {
        return Err(USAGE.to_owned());
    }
    let text =
        std::fs::read_to_string(lock_path).map_err(|error| format!("{lock_path}: {error}"))?;
    let lock = Lock::parse(&text).map_err(|error| format!("{lock_path}: {error:?}"))?;
    let mut out = String::new();
    let (mut before, mut after) = (0_usize, 0_usize);
    for id in rows {
        let row = lock
            .row(&RowId(id.clone()))
            .ok_or_else(|| format!("{id}: not in the lock"))?;
        let cache = Cache::open(Path::new(from), row).map_err(|error| format!("{id}: {error}"))?;
        let lean = cache
            .write_lean(row, Path::new(to))
            .map_err(|error| format!("{id}: {error}"))?;
        let _ = writeln!(out, "{id}: {} bytes, lean {lean} bytes", cache.bytes());
        before = before.saturating_add(cache.bytes());
        after = after.saturating_add(lean);
    }
    let _ = writeln!(out, "all rows: {before} bytes, lean {after} bytes");
    Ok(out)
}
