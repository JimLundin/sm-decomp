//! `tools localise LOCK BUNDLE CACHE ROM ROW...`: the free-run localiser
//! (`verify::localise`) over the named rows. Each row starts from its seed's record in the
//! relock bundle directory `BUNDLE`, as `tools gate` runs it, and every frame is compared with
//! the row's record cache under `CACHE`; it prints each row's first differing frame per group
//! and per field.

use std::fmt::Write as _;
use std::path::Path;

pub const USAGE: &str = "tools localise LOCK BUNDLE CACHE ROM ROW...";

/// Runs the localiser; the tables, or why a row didn't run.
pub fn run(args: &[String]) -> Result<String, String> {
    let [lock, bundle, cache, rom, rows @ ..] = args else {
        return Err(USAGE.to_owned());
    };
    if rows.is_empty() {
        return Err(USAGE.to_owned());
    }
    let mut out = String::new();
    for row in rows {
        let localised = verify::localise::run_row(
            Path::new(lock),
            Path::new(bundle),
            Path::new(cache),
            Path::new(rom),
            row,
        )?;
        let _ = write!(out, "{localised}");
    }
    Ok(out)
}
