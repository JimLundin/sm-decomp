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
//!
//! `tools spec-units SPEC VECTORS [--notes FILE] [--areas FILE] [--against OLD]`:
//! a behaviour spec's units and the size budget they set, by area (`verify::spec_units`).
//!
//! `tools localise LOCK BUNDLE CACHE ROM ROW...`: the free run's first differing frame per
//! field against the record cache ([`localise`]).
//!
//! `tools harness DOMAIN LOCK CACHE ROW...`: a domain's model against the oracle on every cached
//! frame of each row (`verify::harness`); `DOMAIN` is `game-time`.
//!
//! `tools cache-lean LOCK FROM TO ROW...`: each row's record cache under `FROM` written again
//! as the lean tier under `TO` ([`cache_lean`]).
//!
//! `tools size PATH...`: code volume in logical tokens, per file and in total (`verify::size`).
//!
//! `tools report [--write | --check]`: the rebuild's size and progress report, `docs/REPORT.md`
//! (`verify::report`), printed, or written or checked with the README's status.

mod cache_lean;
mod gate;
mod localise;
mod ratchet;

use std::fmt::Write as _;
use std::io::Write;
use std::process::ExitCode;

use std::path::Path;

use verify::harness::{self, game_time::Clock};
use verify::ratchet::Summary;

const LOCKSTEP_USAGE: &str = "tools lockstep LOCK CACHE ROM ROW [FROM [TICKS]]";
const HARNESS_USAGE: &str = "tools harness game-time LOCK CACHE ROW...";

fn usage() -> String {
    format!(
        "usage: {LOCKSTEP_USAGE}\n       {HARNESS_USAGE}\n       {}\n       {}\n       {}\n       {}\n       {}\n       {}",
        verify::spec_units::USAGE,
        gate::USAGE,
        localise::USAGE,
        cache_lean::USAGE,
        verify::size::USAGE,
        verify::report::USAGE
    )
}

fn run(args: &[String]) -> Result<String, String> {
    match args {
        [command, rest @ ..] if command == "spec-units" => verify::spec_units::run(rest),
        [command, rest @ ..] if command == "lockstep" => run_lockstep(rest),
        [command, rest @ ..] if command == "gate" => gate::run(rest),
        [command, rest @ ..] if command == "localise" => localise::run(rest),
        [command, rest @ ..] if command == "harness" => run_harness(rest),
        [command, rest @ ..] if command == "cache-lean" => cache_lean::run(rest),
        [command, rest @ ..] if command == "size" => verify::size::run(rest),
        [command, rest @ ..] if command == "report" => verify::report::run(rest),
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

fn run_harness(args: &[String]) -> Result<String, String> {
    let [domain, lock, cache, rows @ ..] = args else {
        return Err(usage());
    };
    if domain != "game-time" || rows.is_empty() {
        return Err(usage());
    }
    let mut out = String::new();
    for row in rows {
        let report = harness::run_cached(&Clock, Path::new(lock), Path::new(cache), row)
            .map_err(|error| format!("{row}: {error}"))?;
        let _ = write!(out, "{row}: {report}");
    }
    Ok(out)
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
