//! Commands for indexed production replay, record caches, and project reports.
//! Comparison and execution live in `verify::replay`.

mod cache_check;
mod engine_size;
mod replay;

use std::io::Write;
use std::process::ExitCode;

fn usage() -> String {
    format!(
        "usage: {}\n       {}\n       {}\n       {}",
        replay::USAGE,
        cache_check::USAGE,
        engine_size::USAGE,
        verify::report::USAGE
    )
}

fn run(args: &[String]) -> Result<String, String> {
    match args {
        [command, rest @ ..] if command == "cache-check" => cache_check::run(rest),
        [command, rest @ ..] if command == "engine-size" => engine_size::run(rest),
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
