//! The command line over the engine, the oracle and the comparison: replay, scenarios,
//! locking and the Map Rando and sm-json-data importers.
//!
//! It has no comparison logic of its own; that lives in `verify` (ARCHITECTURE §4.1).
//!
//! `tools lockstep LOCK CACHE ROM ROW [FROM [TICKS]]`: lockstep over a row's record cache
//! (`verify::lockstep`) on the content read from `ROM`, printing the report.
//!
//! `tools lockstep --check-ratchet LOCK CACHE ROM RATCHET`: every ratchet row, held to its
//! numbers.
//!
//! `tools gate LOCK BUNDLE ROM GREEN [--update] ROW...`: the free-run gate ([`gate`]).

mod gate;
mod ratchet;

use std::io::Write;
use std::process::ExitCode;

use verify::ratchet::Summary;

const LOCKSTEP_USAGE: &str = "tools lockstep LOCK CACHE ROM ROW [FROM [TICKS]]";

fn usage() -> String {
    format!("usage: {LOCKSTEP_USAGE}\n       {}", gate::USAGE)
}

fn run(args: &[String]) -> Result<String, String> {
    match args {
        [command, rest @ ..] if command == "lockstep" => run_lockstep(rest),
        [command, rest @ ..] if command == "gate" => gate::run(rest),
        _ => Err(usage()),
    }
}

fn run_lockstep(args: &[String]) -> Result<String, String> {
    if let [flag, rest @ ..] = args
        && flag == "--check-ratchet"
    {
        return ratchet::check(rest);
    }
    let [lock, cache, rom, row, window @ ..] = args else {
        return Err(usage());
    };
    let number = |text: &String| {
        text.parse::<u64>()
            .map_err(|error| format!("{text}: {error}"))
    };
    let (from, ticks) = match window {
        [] => (0, u64::MAX),
        [from] => (number(from)?, u64::MAX),
        [from, ticks] => (number(from)?, number(ticks)?),
        [_, _, ..] => return Err(usage()),
    };
    let report = ratchet::run_row(lock, cache, rom, row, from..from.saturating_add(ticks))?;
    Ok(format!(
        "{report}ratchet lines:\n{}",
        Summary::of(row, &report)
    ))
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(report) => {
            let written = std::io::stdout().lock().write_all(report.as_bytes());
            if written.is_ok() {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(message) => {
            // Nothing to report the failure to if stderr itself fails.
            let _ = writeln!(std::io::stderr().lock(), "tools: {message}");
            ExitCode::FAILURE
        }
    }
}
