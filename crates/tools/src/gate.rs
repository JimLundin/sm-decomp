//! `tools gate LOCK BUNDLE ROM GREEN [--update] ROW...`: the free-run gate (D22) over the
//! named rows. Each row starts from its seed's record in the relock bundle's directory
//! `BUNDLE` (`seeds/<capture>/NNNN.record`), with `BUNDLE/pads/ROW.pads` checked against the
//! row's pin, and runs on the content read from `ROM`. It prints every (row, stream) outcome
//! and the row's green status. A row the engine can't start (a seed in a room, with a species or
//! in a game state it doesn't know, or a start the gate doesn't run from) prints why and the
//! gate goes on to the next. It fails when a row the ratchet file `GREEN` holds as starting
//! doesn't start or a pair it lists isn't green, never for a row that never started, and with
//! `--update` adds the new rows that start and the new green pairs to it.

use std::fmt::Write as _;
use std::path::Path;
use std::sync::Arc;

use verify::bundle::{RowInputs, content, rooms};
use verify::gate::{Ratchet, Stop, free_run};
use verify::lock::{Lock, Outcome, RowId, Side};

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
    let content = Arc::new(content(image.clone()).map_err(|error| format!("{rom}: {error}"))?);
    let rooms = rooms(&content, &image).map_err(|error| format!("{rom}: {error:?}"))?;
    let mut out = String::new();
    let mut reports = Vec::new();
    let mut not_started = Vec::new();
    for id in rows {
        let row = lock
            .row(&RowId(id.clone()))
            .ok_or_else(|| format!("{id}: not in the lock"))?;
        let inputs = match RowInputs::read(Path::new(bundle), row, &rooms) {
            Ok(inputs) => inputs,
            Err(error) => {
                let why = error
                    .why_it_cannot_start()
                    .ok_or_else(|| error.to_string())?;
                let _ = writeln!(out, "{id}: doesn't start: {why}");
                not_started.push(row.id.clone());
                continue;
            }
        };
        let run = free_run(row, inputs.seed, &inputs.pads, &content);
        if let Some(Stop::Open(error)) = run.stop {
            let _ = writeln!(out, "{id}: doesn't start: {error:?}");
            not_started.push(row.id.clone());
            continue;
        }
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
    let grown = ratchet.grown(&reports);
    for row in ratchet.new_starts(&reports) {
        let _ = writeln!(out, "new row that starts: {row}");
    }
    for (row, stream) in ratchet.new_pairs(&reports) {
        let _ = writeln!(out, "new green pair: {row} {stream}");
    }
    let failures: Vec<String> = ratchet
        .lost_starts(&not_started)
        .iter()
        .map(|row| format!("{row} doesn't start"))
        .chain(
            ratchet
                .regressions(&reports)
                .iter()
                .map(|(row, stream)| format!("{row} {stream}")),
        )
        .collect();
    if !failures.is_empty() {
        return Err(format!(
            "{out}the ratchet's rows and green pairs lost: {}",
            failures.join(", ")
        ));
    }
    if update && grown != ratchet {
        std::fs::write(green, grown.to_string()).map_err(|error| format!("{green}: {error}"))?;
        let _ = writeln!(
            out,
            "{green}: {} rows that start, {} green pairs",
            grown.started.len(),
            grown.pairs.len()
        );
    }
    Ok(out)
}
