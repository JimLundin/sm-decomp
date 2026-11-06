//! `tools gate LOCK BUNDLE ROM GREEN [--update] ROW...`: the free-run gate (D22) over the
//! named rows. Each row starts from its seed's record in the relock bundle's directory
//! `BUNDLE` (`seeds/<capture>/NNNN.record`), with `BUNDLE/pads/ROW.pads` checked against the
//! row's pin, and runs on the content read from `ROM`. It prints every (row, stream) outcome
//! and the row's green status, fails when a pair the ratchet file `GREEN` lists isn't green,
//! and with `--update` adds the new green pairs to it.

use std::fmt::Write as _;
use std::path::Path;
use std::sync::Arc;

use verify::bundle::{RowInputs, content};
use verify::gate::{Ratchet, free_run};
use verify::lock::{Lock, Outcome, RowId, Side};
use verify::lockstep::{slice_zero_plms, slice_zero_rooms};

pub const USAGE: &str = "tools gate LOCK BUNDLE ROM GREEN [--update] ROW...";

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
    let [lock_path, bundle, rom, green, rest @ ..] = args else {
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
    let image = std::fs::read(rom).map_err(|error| format!("{rom}: {error}"))?;
    let plms = slice_zero_plms(&image).map_err(|error| format!("{rom}: {error:?}"))?;
    let content = Arc::new(content(image).map_err(|error| format!("{rom}: {error}"))?);
    let rooms = slice_zero_rooms()
        .with_programs(content.enemy_programs.clone())
        .with_samus_tiles(content.samus_tiles.clone())
        .with_plms(plms);
    let mut out = String::new();
    let mut reports = Vec::new();
    for id in rows {
        let row = lock
            .row(&RowId(id.clone()))
            .ok_or_else(|| format!("{id}: not in the lock"))?;
        let inputs =
            RowInputs::read(Path::new(bundle), row, &rooms).map_err(|error| error.to_string())?;
        let run = free_run(row, inputs.seed, &inputs.pads, &content);
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
