//! The verification pipeline around the verifier (docs/VERIFY_PIPELINE.md).
//!
//!   sm_pipeline plan [--root DIR]
//!       plan one ci.yml Verify run as its job matrix, from the environment
//!       verify-plan sets: SCOPE (gate, full, rows or coverage), SELECTED,
//!       ORACLE_SAMPLE=1, COVER_ROUTES=1 and, to pin the oracle sample's
//!       three-hour slot, ORACLE_SLOT. It appends the matrix to GITHUB_OUTPUT
//!       and its summary to GITHUB_STEP_SUMMARY (stdout when unset).
//!   sm_pipeline lock-check [--root DIR]
//!       check the scenarios and their lock as `tools/scenario.py lock --check`
//!       does: print the counts, or exit 1 naming every problem (a stale or
//!       orphaned entry, a missing pin, a lockable row neither locked nor
//!       `unlocked`).
//!   sm_pipeline relevant
//!       read a `git diff --name-only` listing on stdin, print the files that
//!       can affect a replay, and exit 0 only when there are none.
//!   sm_pipeline test-summary LOG PACKAGE_DIR...
//!       print the per-binary table of a `cargo test` log (and append it to
//!       GITHUB_STEP_SUMMARY when set), and exit 1 when a binary failed, gave
//!       no result, or was expected from a package's sources and did not run.
//!   sm_pipeline volume [--root DIR] [--baseline FILE] [--file PATH...]
//!       print the code volume (volume.rs): logic and data tokens, comment
//!       lines and structure counts per engine package and sm_game module, the
//!       oracle and verification apart, then `tools/ram_ratchet.py`'s measures.
//!       --baseline compares those measures with a file's `"name": value`
//!       entries and exits 1 when one rose; --file measures the given files.
//!
//! It needs only the standard library, so CI builds it in seconds with
//! `rustc --edition=2024 -O crates/sm_tools/src/bin/sm_pipeline/main.rs`.
#![forbid(unsafe_code)]

mod corpus;
mod json;
mod pack;
mod plan;
mod relevant;
mod route;
mod scenario;
mod test_summary;
mod toml;
mod volume;

#[cfg(test)]
mod lock_golden;
#[cfg(test)]
mod plan_golden;

use std::io::{Read as _, Write as _};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "usage: sm_pipeline plan [--root DIR]
       sm_pipeline lock-check [--root DIR]
       sm_pipeline relevant < LISTING
       sm_pipeline test-summary LOG PACKAGE_DIR...
       sm_pipeline volume [--root DIR] [--baseline FILE] [--file PATH...]";

/// An environment variable, empty when unset.
fn variable(name: &str) -> String {
    std::env::var(name).unwrap_or_default()
}

fn settings() -> Result<plan::Settings, String> {
    let slot = match variable("ORACLE_SLOT") {
        slot if slot.is_empty() => corpus::oracle_slot(std::time::SystemTime::now()),
        slot => slot
            .parse()
            .map_err(|_| format!("ORACLE_SLOT {slot} is not a slot number"))?,
    };
    Ok(plan::Settings {
        scope: plan::Scope::parse(&variable("SCOPE"))?,
        selected: variable("SELECTED")
            .split_whitespace()
            .map(str::to_owned)
            .collect(),
        oracle_sample: variable("ORACLE_SAMPLE") == "1",
        cover_routes: variable("COVER_ROUTES") == "1",
        slot,
    })
}

/// Append to a file.
fn append_to(path: &Path, text: &str) -> Result<(), String> {
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .and_then(|mut file| file.write_all(text.as_bytes()))
        .map_err(|error| format!("{}: {error}", path.display()))
}

/// Append to the file a GitHub variable names, or print when it is unset.
fn append(variable_name: &str, text: &str) -> Result<(), String> {
    match std::env::var_os(variable_name) {
        Some(path) => append_to(Path::new(&path), text),
        None => {
            print!("{text}");
            Ok(())
        }
    }
}

/// The repository root an optional `--root DIR` names, `.` by default.
fn root(arguments: &[String]) -> Result<PathBuf, String> {
    match arguments {
        [] => Ok(PathBuf::from(".")),
        [flag, directory] if flag == "--root" => Ok(directory.into()),
        _ => Err(USAGE.into()),
    }
}

fn plan_command(arguments: &[String]) -> Result<ExitCode, String> {
    let planned = plan::plan(&root(arguments)?, &settings()?)?;
    append("GITHUB_OUTPUT", &planned.output)?;
    append("GITHUB_STEP_SUMMARY", &planned.summary)?;
    print!("{}", planned.log);
    Ok(ExitCode::SUCCESS)
}

fn relevant_command() -> Result<ExitCode, String> {
    let mut listing = String::new();
    std::io::stdin()
        .read_to_string(&mut listing)
        .map_err(|error| format!("stdin: {error}"))?;
    let relevant = relevant::relevant(&listing);
    for path in &relevant {
        println!("replay-relevant: {path}");
    }
    Ok(if relevant.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}

fn test_summary_command(arguments: &[String]) -> Result<ExitCode, String> {
    let (log, directories) = arguments.split_first().ok_or(USAGE)?;
    let bytes = std::fs::read(log).map_err(|error| format!("{log}: {error}"))?;
    let packages = directories
        .iter()
        .map(|directory| test_summary::Package::read(Path::new(directory)))
        .collect::<Result<Vec<_>, _>>()?;
    let summary = test_summary::summarise(&String::from_utf8_lossy(&bytes), &packages);
    println!("{}", summary.report);
    let step_summary = variable("GITHUB_STEP_SUMMARY");
    if !step_summary.is_empty() {
        append_to(Path::new(&step_summary), &format!("{}\n", summary.report))?;
    }
    eprint!("{}", summary.diagnostics());
    Ok(if summary.passed() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}

fn volume_command(arguments: &[String]) -> Result<ExitCode, String> {
    let (mut root, mut baseline, mut files) = (PathBuf::from("."), None, Vec::new());
    let mut arguments = arguments.iter();
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--root" => root = arguments.next().ok_or(USAGE)?.into(),
            "--baseline" => baseline = Some(arguments.next().ok_or(USAGE)?),
            "--file" => files.extend(arguments.by_ref().cloned()),
            _ => return Err(USAGE.into()),
        }
    }
    if !files.is_empty() {
        for file in &files {
            let text = std::fs::read_to_string(file).map_err(|error| format!("{file}: {error}"))?;
            println!("{file}: {:?}", volume::measure(&text));
        }
        return Ok(ExitCode::SUCCESS);
    }
    let rows = volume::survey(&root)?;
    print!("{}", volume::report(&rows));
    let Some(baseline) = baseline else {
        return Ok(ExitCode::SUCCESS);
    };
    let text = std::fs::read_to_string(baseline).map_err(|error| format!("{baseline}: {error}"))?;
    let (comparison, held) = volume::compare(&rows, &volume::baseline(&text));
    print!("{comparison}");
    Ok(if held {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}

fn run() -> Result<ExitCode, String> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let (command, rest) = arguments.split_first().ok_or(USAGE)?;
    match command.as_str() {
        "plan" => plan_command(rest),
        "lock-check" => {
            println!("{}", scenario::lock_check(&root(rest)?)?);
            Ok(ExitCode::SUCCESS)
        }
        "relevant" if rest.is_empty() => relevant_command(),
        "test-summary" => test_summary_command(rest),
        "volume" => volume_command(rest),
        _ => Err(USAGE.into()),
    }
}

fn main() -> ExitCode {
    run().unwrap_or_else(|error| {
        eprintln!("{error}");
        ExitCode::FAILURE
    })
}
