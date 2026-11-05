//! Runs deliberate one-line bugs against the checks that must catch them.
//!
//! `sm_mutants list`                      the catalogue
//! `sm_mutants anchors`                   every anchor occurs exactly once (no build)
//! `sm_mutants build [SELECT]`            every selected mutant compiles (no ROM)
//! `sm_mutants run [SELECT]`              build and prove the selected mutants
//!
//! SELECT: `--slot I/N` (the I-th of N rotating parts; `--slot today/N` picks
//! by the day), `--only NAME`, `--rom-free` (skip mutants that need the ROM),
//! `--rom` (only those).
//!
//! Run from the repository root. Each mutant is applied to the checkout and
//! restored afterwards, also when a step fails. Every verdict is one of killed,
//! survived or broken; a broken mutant (stale anchor, compile error, failing
//! baseline) is plumbing, never a kill, and the run continues past it.
//! Exit status: 0 all killed, 1 a survivor, 2 a broken mutant.
mod catalogue;

use catalogue::{Edit, MUTANTS, Mutant, Proof, Tests};
use std::path::Path;
use std::process::{Command, Output};

/// A mutant's result.
enum Verdict {
    Killed(String),
    Survived(String),
    Broken(String),
}

/// The checkout's files as they were before a mutant's edits; restores them
/// when dropped.
struct Applied(Vec<(&'static str, Vec<u8>)>);

impl Drop for Applied {
    fn drop(&mut self) {
        for (file, original) in &self.0 {
            if let Err(error) = std::fs::write(file, original) {
                eprintln!("sm_mutants: could not restore {file}: {error}");
            }
        }
    }
}

fn anchor_count(edit: &Edit) -> Result<usize, String> {
    let text =
        std::fs::read_to_string(edit.file).map_err(|error| format!("{}: {error}", edit.file))?;
    Ok(text.matches(edit.anchor).count())
}

fn apply(mutant: &Mutant) -> Result<Applied, String> {
    let mut applied = Applied(Vec::new());
    for edit in mutant.edits {
        let original =
            std::fs::read(edit.file).map_err(|error| format!("{}: {error}", edit.file))?;
        let text =
            String::from_utf8(original.clone()).map_err(|_| format!("{}: not UTF-8", edit.file))?;
        if text.matches(edit.anchor).count() != 1 {
            return Err(format!("{}: anchor must occur exactly once", edit.file));
        }
        // Keep the first copy of a file edited twice.
        if !applied.0.iter().any(|(file, _)| *file == edit.file) {
            applied.0.push((edit.file, original));
        }
        let mutated = text.replacen(edit.anchor, edit.replacement, 1);
        std::fs::write(edit.file, mutated).map_err(|error| format!("{}: {error}", edit.file))?;
    }
    Ok(applied)
}

/// Cargo with lints capped at warnings, so a mutant that leaves an item
/// unused is judged by its tests, not by `deny(warnings)`. The baseline
/// builds the same way.
fn cargo(arguments: &[&str]) -> Result<Output, String> {
    let mut flags = std::env::var("RUSTFLAGS").unwrap_or_default();
    flags.push_str(" --cap-lints=warn");
    Command::new("cargo")
        .args(arguments)
        .env("RUSTFLAGS", flags.trim())
        .output()
        .map_err(|error| format!("cargo: {error}"))
}

fn test_target(tests: Tests) -> Vec<&'static str> {
    match tests {
        Tests::Lib => vec!["--lib"],
        Tests::Integration(name) => vec!["--test", name],
    }
}

/// Whether the named test passed, failed or did not run, from libtest's lines.
fn test_result(output: &Output, name: &str) -> Option<bool> {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = |verdict: &str| {
        stdout
            .lines()
            .any(|line| line.trim() == format!("test {name} ... {verdict}"))
    };
    if line("ok") {
        Some(true)
    } else if line("FAILED") {
        Some(false)
    } else {
        None
    }
}

fn run_test(package: &str, tests: Tests, name: &str) -> Result<Output, String> {
    let mut arguments = vec!["test", "-p", package];
    arguments.extend(test_target(tests));
    arguments.extend(["--", "--exact", name]);
    cargo(&arguments)
}

fn build(proof: &Proof) -> Result<(), String> {
    let output = match proof {
        Proof::Unit { package, tests, .. } => {
            let mut arguments = vec!["test", "-p", package, "--no-run"];
            arguments.extend(test_target(*tests));
            cargo(&arguments)?
        }
        Proof::Lock { package, .. } => cargo(&["check", "-p", package, "--lib"])?,
    };
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let errors: Vec<&str> = stderr
        .lines()
        .filter(|line| line.starts_with("error"))
        .take(3)
        .collect();
    Err(format!("compile error: {}", errors.join(" | ")))
}

fn unit(mutant: &Mutant, package: &str, tests: Tests, names: &[&str]) -> Verdict {
    for name in names {
        match run_test(package, tests, name).map(|output| test_result(&output, name)) {
            Ok(Some(true)) => {}
            Ok(_) => return Verdict::Broken(format!("baseline: {name} did not pass")),
            Err(error) => return Verdict::Broken(error),
        }
    }
    let _applied = match apply(mutant) {
        Ok(applied) => applied,
        Err(error) => return Verdict::Broken(error),
    };
    if let Err(error) = build(&mutant.proof) {
        return Verdict::Broken(error);
    }
    let mut passed = Vec::new();
    for name in names {
        match run_test(package, tests, name).map(|output| test_result(&output, name)) {
            Ok(Some(false)) => {}
            Ok(Some(true)) => passed.push(*name),
            Ok(None) => return Verdict::Broken(format!("{name} did not run")),
            Err(error) => return Verdict::Broken(error),
        }
    }
    if passed.is_empty() {
        Verdict::Killed(format!("by {}", names.join(", ")))
    } else {
        Verdict::Survived(format!("{} still passed", passed.join(", ")))
    }
}

/// Engine-only replay of `rows` against the committed lock. The verifier
/// builds itself with the mutant applied; the oracle is not involved.
fn lock(mutant: &Mutant, rows: &[&str]) -> Verdict {
    if !Path::new("rom/sm.smc").is_file() {
        return Verdict::Broken("needs rom/sm.smc".into());
    }
    let _applied = match apply(mutant) {
        Ok(applied) => applied,
        Err(error) => return Verdict::Broken(error),
    };
    if let Err(error) = build(&mutant.proof) {
        return Verdict::Broken(error);
    }
    let mut command = Command::new(if cfg!(windows) { "python" } else { "python3" });
    command.args([
        "tools/verify_dll.py",
        "rom/sm.smc",
        "--corpus",
        "scenarios",
        "--direct",
        "--mode",
        "check-lock",
        "--lock",
        "scenarios/lock.toml",
    ]);
    for row in rows {
        command.args(["--segment", row]);
    }
    let output = match command.output() {
        Ok(output) => output,
        Err(error) => return Verdict::Broken(format!("verify_dll: {error}")),
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut caught = Vec::new();
    let mut matched = Vec::new();
    for row in rows {
        if let Some(line) = stdout
            .lines()
            .find(|line| line.starts_with(&format!("{row}: lock mismatch in")))
        {
            caught.push(line.to_owned());
        } else if stdout
            .lines()
            .any(|line| line.starts_with(&format!("{row}: lock matches")))
        {
            matched.push(*row);
        }
    }
    match (caught.is_empty(), matched.len() == rows.len()) {
        (false, _) => Verdict::Killed(caught.join(" | ")),
        (true, true) => Verdict::Survived(format!("lock matches on {}", rows.join(", "))),
        (true, false) => Verdict::Broken(format!(
            "verifier exit {:?}; not every row reported",
            output.status.code()
        )),
    }
}

fn prove(mutant: &Mutant) -> Verdict {
    match &mutant.proof {
        Proof::Unit {
            package,
            tests,
            names,
        } => unit(mutant, package, *tests, names),
        Proof::Lock { rows, .. } => lock(mutant, rows),
    }
}

/// The selected mutants, in catalogue order.
fn select(arguments: &[String]) -> Result<Vec<&'static Mutant>, String> {
    let mut selected: Vec<&Mutant> = MUTANTS.iter().collect();
    let mut rest = arguments.iter();
    while let Some(argument) = rest.next() {
        match argument.as_str() {
            "--slot" => {
                let value = rest.next().ok_or("--slot needs I/N")?;
                let (index, count) = value.split_once('/').ok_or("--slot needs I/N")?;
                let count: usize = count.parse().map_err(|_| "bad slot count")?;
                let index: usize = if index == "today" {
                    let days = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map_err(|_| "clock before 1970")?
                        .as_secs()
                        / 86_400;
                    usize::try_from(days).map_err(|_| "day")? % count + 1
                } else {
                    index.parse().map_err(|_| "bad slot index")?
                };
                if count == 0 || !(1..=count).contains(&index) {
                    return Err("slot index must be 1..=N".into());
                }
                println!("slot {index}/{count}");
                selected = selected
                    .into_iter()
                    .enumerate()
                    .filter(|(at, _)| at % count == index - 1)
                    .map(|(_, mutant)| mutant)
                    .collect();
            }
            "--only" => {
                let name = rest.next().ok_or("--only needs a name")?;
                selected.retain(|mutant| mutant.name == name);
            }
            "--rom-free" => selected.retain(|mutant| !mutant.proof.needs_rom()),
            "--rom" => selected.retain(|mutant| mutant.proof.needs_rom()),
            other => return Err(format!("unknown argument {other}")),
        }
    }
    Ok(selected)
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let Some((command, rest)) = arguments.split_first() else {
        eprintln!("usage: sm_mutants list | anchors | build [SELECT] | run [SELECT]");
        std::process::exit(2);
    };
    let selected = match select(rest) {
        Ok(selected) => selected,
        Err(error) => {
            eprintln!("sm_mutants: {error}");
            std::process::exit(2);
        }
    };
    let (mut killed, mut survived, mut broken) = (0, 0, 0);
    for mutant in &selected {
        let verdict = match command.as_str() {
            "list" => {
                let files: Vec<&str> = mutant.edits.iter().map(|edit| edit.file).collect();
                let proof = if mutant.proof.needs_rom() {
                    "lock"
                } else {
                    "unit"
                };
                println!(
                    "{}: {:?}; {proof} in {}; {}",
                    mutant.name,
                    mutant.target,
                    mutant.proof.package(),
                    files.join(", ")
                );
                continue;
            }
            "anchors" => match mutant
                .edits
                .iter()
                .map(anchor_count)
                .collect::<Result<Vec<_>, _>>()
            {
                Ok(counts) if counts.iter().all(|&count| count == 1) => {
                    Verdict::Killed("anchors present".into())
                }
                Ok(counts) => Verdict::Broken(format!("anchor counts {counts:?}")),
                Err(error) => Verdict::Broken(error),
            },
            "build" => match apply(mutant).and_then(|_applied| build(&mutant.proof)) {
                Ok(()) => Verdict::Killed("compiles".into()),
                Err(error) => Verdict::Broken(error),
            },
            "run" => prove(mutant),
            other => {
                eprintln!("sm_mutants: unknown command {other}");
                std::process::exit(2);
            }
        };
        let (word, detail) = match verdict {
            Verdict::Killed(detail) => {
                killed += 1;
                (if command == "run" { "KILLED" } else { "OK" }, detail)
            }
            Verdict::Survived(detail) => {
                survived += 1;
                ("SURVIVED", detail)
            }
            Verdict::Broken(detail) => {
                broken += 1;
                ("BROKEN", detail)
            }
        };
        println!("{}: {word}: {detail}", mutant.name);
    }
    if command != "list" {
        println!(
            "sm_mutants {command}: {} selected, {killed} ok, {survived} survived, {broken} broken",
            selected.len()
        );
    }
    std::process::exit(if broken > 0 {
        2
    } else {
        i32::from(survived > 0)
    });
}
