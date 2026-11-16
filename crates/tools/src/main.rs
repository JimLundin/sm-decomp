//! Commands for indexed production replay, record caches, and project reports.
//! Comparison and execution live in `verify::replay`.

mod cache_lean;
mod replay;

use std::io::Write;
use std::process::ExitCode;

fn usage() -> String {
    format!(
        "usage: {}\n       {}\n       {}\n       {}\n       {}",
        replay::USAGE,
        verify::spec_units::USAGE,
        cache_lean::USAGE,
        verify::size::USAGE,
        verify::report::USAGE
    )
}

fn run(args: &[String]) -> Result<String, String> {
    match args {
        [command, rest @ ..] if command == "spec-units" => verify::spec_units::run(rest),
        [command, rest @ ..] if command == "cache-lean" => cache_lean::run(rest),
        [command, rest @ ..] if command == "size" => verify::size::run(rest),
        [command, rest @ ..] if command == "report" => verify::report::run(rest),
        [command, rest @ ..] if command == "replay" => replay::run(rest),
        _ => Err(usage()),
    }
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
