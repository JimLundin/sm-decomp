//! The verification pipeline around the verifier (docs/VERIFY_PIPELINE.md).
//!
//!   sm_pipeline plan [--root DIR]
//!       plan one ci.yml Verify run as its job matrix, from the environment
//!       verify-plan sets: SCOPE (gate, full, rows or coverage), SELECTED,
//!       ORACLE_SAMPLE=1, COVER_ROUTES=1 and, to pin the oracle sample's
//!       three-hour slot, ORACLE_SLOT. It appends the matrix to GITHUB_OUTPUT
//!       and its summary to GITHUB_STEP_SUMMARY (stdout when unset).
//!
//! It needs only the standard library, so CI builds it in seconds with
//! `rustc --edition=2024 -O crates/sm_tools/src/bin/sm_pipeline/main.rs`.
#![forbid(unsafe_code)]

mod corpus;
mod json;
mod pack;
mod plan;
mod route;
mod scenario;
mod toml;

#[cfg(test)]
mod plan_golden;

use std::io::Write as _;
use std::path::PathBuf;
use std::process::ExitCode;

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

/// Append to the file a GitHub variable names, or print when it is unset.
fn append(variable_name: &str, text: &str) -> Result<(), String> {
    match std::env::var_os(variable_name) {
        Some(path) => std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .and_then(|mut file| file.write_all(text.as_bytes()))
            .map_err(|error| format!("{variable_name}: {error}")),
        None => {
            print!("{text}");
            Ok(())
        }
    }
}

fn run() -> Result<(), String> {
    let mut arguments = std::env::args().skip(1);
    let command = arguments.next().unwrap_or_default();
    let mut root = PathBuf::from(".");
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--root" => root = arguments.next().ok_or("--root needs a directory")?.into(),
            _ => return Err(format!("unknown argument {argument}")),
        }
    }
    match command.as_str() {
        "plan" => {
            let planned = plan::plan(&root, &settings()?)?;
            append("GITHUB_OUTPUT", &planned.output)?;
            append("GITHUB_STEP_SUMMARY", &planned.summary)?;
            print!("{}", planned.log);
            Ok(())
        }
        _ => Err("usage: sm_pipeline plan [--root DIR]".into()),
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
