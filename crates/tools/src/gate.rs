//! `tools gate LOCK CACHE PADS GREEN [--update] ROW...`: the free-run gate (D22) over the
//! named rows, each from its cache's first record (the seed's first frame), with
//! `PADS/ROW.pads` checked against the row's pin. It prints every (row, stream) outcome and
//! the row's green status, fails when a pair the ratchet file `GREEN` lists isn't green, and
//! with `--update` adds the new green pairs to it.

use std::fmt::Write as _;
use std::path::Path;
use std::sync::Arc;

use verify::cache::Cache;
use verify::gate::{FreeRun, Ratchet, Seed, buttons, free_run};
use verify::lock::{Lock, Outcome, Row, RowId, Side};
use verify::lockstep::slice_zero_rooms;

pub const USAGE: &str = "tools gate LOCK CACHE PADS GREEN [--update] ROW...";

/// The run of one row: its seed decoded from the cache and its pads from the pad file.
fn run_row(row: &Row, cache: &Path, pads: &Path) -> Result<FreeRun, String> {
    let id = &row.id;
    let pad_file = pads.join(format!("{id}.pads"));
    let pin = oracle::relock::file_sha256(&pad_file);
    if pin.as_deref() != Some(row.pads.to_string().as_str()) {
        return Err(format!("{}: not the row's pads", pad_file.display()));
    }
    let pads: Vec<_> = oracle::inputs::read(&pad_file)?
        .into_iter()
        .map(buttons)
        .collect();
    let cache = Cache::open(cache, row).map_err(|error| format!("{id}: {error}"))?;
    let first = cache
        .entries()
        .first()
        .ok_or_else(|| format!("{id}: the cache holds no frames"))?;
    let record = cache
        .record(first)
        .map_err(|error| format!("{id}: {error}"))?;
    let seed = Seed::decode(&record, &slice_zero_rooms())
        .map_err(|error| format!("{id}: the seed doesn't decode: {error:?}"))?;
    Ok(free_run(
        row,
        seed,
        &pads,
        // open: the tool reads no ROM yet, so Samus's pose rules run on empty tables.
        &Arc::default(),
    ))
}

fn outcome(outcome: Outcome) -> String {
    match outcome {
        Outcome::Equal => "equal".to_owned(),
        Outcome::Differs { span } => format!("differs from span {}", span.0),
        Outcome::Absent(Side::Lock) => "absent from the lock".to_owned(),
        Outcome::Absent(Side::Engine) => "absent from the engine".to_owned(),
        Outcome::NotLocked => "not locked".to_owned(),
    }
}

/// Runs the gate; the report, or why it failed.
pub fn run(args: &[String]) -> Result<String, String> {
    let [lock_path, cache, pads, green, rest @ ..] = args else {
        return Err(USAGE.to_owned());
    };
    let (update, rows) = match rest {
        [flag, rows @ ..] if flag == "--update" => (true, rows),
        rows => (false, rows),
    };
    if rows.is_empty() {
        return Err(USAGE.to_owned());
    }
    let text =
        std::fs::read_to_string(lock_path).map_err(|error| format!("{lock_path}: {error}"))?;
    let lock = Lock::parse(&text).map_err(|error| format!("{lock_path}: {error:?}"))?;
    let ratchet = match std::fs::read_to_string(green) {
        Ok(text) => Ratchet::parse(&text).map_err(|error| format!("{green}: {error}"))?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ratchet::default(),
        Err(error) => return Err(format!("{green}: {error}")),
    };
    let mut out = String::new();
    let mut reports = Vec::new();
    for id in rows {
        let row = lock
            .row(&RowId(id.clone()))
            .ok_or_else(|| format!("{id}: not in the lock"))?;
        let run = run_row(row, Path::new(cache), Path::new(pads))?;
        let green = if run.report.green() { "yes" } else { "no" };
        let _ = writeln!(out, "{id}: {} frames, green {green}", run.frames.0);
        if let Some(stop) = run.stop {
            let _ = writeln!(out, "  stopped: {stop}");
        }
        for report in &run.report.streams {
            let _ = writeln!(out, "  {} {}", report.stream, outcome(report.outcome));
        }
        reports.push(run.report);
    }
    let regressions = ratchet.regressions(&reports);
    let grown = ratchet.grown(&reports);
    for (row, stream) in ratchet.new_pairs(&reports) {
        let _ = writeln!(out, "new green pair: {row} {stream}");
    }
    if !regressions.is_empty() {
        let pairs: Vec<String> = regressions
            .iter()
            .map(|(row, stream)| format!("{row} {stream}"))
            .collect();
        return Err(format!("{out}green pairs turned red: {}", pairs.join(", ")));
    }
    if update && grown != ratchet {
        std::fs::write(green, grown.to_string()).map_err(|error| format!("{green}: {error}"))?;
        let _ = writeln!(out, "{green}: {} green pairs", grown.pairs.len());
    }
    Ok(out)
}
