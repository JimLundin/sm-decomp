//! The planner's golden plans. `fixture/` is a frozen root cut from the
//! committed scenarios (56 rows of the corpus, coverage and both routes, with
//! their lock entries, measured seconds and capture indexes), and
//! `fixture/cases.rs` holds the GITHUB_OUTPUT and step summary that
//! `.github/scripts/plan_verify.py` wrote over it for every scope before it was
//! deleted (generated at e85282582). The Rust plan must equal them byte for
//! byte, and the cases the Python planner refused must fail.

use std::path::{Path, PathBuf};

use crate::plan::{self, Scope, Settings};

struct Planned {
    output: &'static str,
    summary: &'static str,
}

struct Golden {
    label: &'static str,
    scope: Scope,
    selected: &'static [&'static str],
    oracle_sample: bool,
    cover_routes: bool,
    slot: u64,
    /// None where the planner must refuse the case.
    planned: Option<Planned>,
}

const CASES: &[Golden] = include!("fixture/cases.rs");

pub fn fixture() -> PathBuf {
    Path::new(option_env!("CARGO_MANIFEST_DIR").unwrap_or("crates/sm_tools"))
        .join("src/bin/sm_pipeline/fixture")
}

/// Where two texts first differ, with some context, or None when equal.
fn difference(rust: &str, golden: &str) -> Option<String> {
    let at = rust
        .bytes()
        .zip(golden.bytes())
        .position(|(a, b)| a != b)
        .or_else(|| (rust.len() != golden.len()).then(|| rust.len().min(golden.len())))?;
    let window = |text: &str| {
        let start = text.floor_char_boundary(at.saturating_sub(160));
        let end = text.ceil_char_boundary((at + 160).min(text.len()));
        text[start..end].to_owned()
    };
    Some(format!(
        "first difference at byte {at}:\n  rust:   {}\n  golden: {}",
        window(rust),
        window(golden)
    ))
}

#[test]
fn plans_every_scope_as_the_golden_plans() {
    let root = fixture();
    let failures: Vec<String> = CASES
        .iter()
        .filter_map(|case| {
            let settings = Settings {
                scope: case.scope,
                selected: case.selected.iter().map(|&id| id.to_owned()).collect(),
                oracle_sample: case.oracle_sample,
                cover_routes: case.cover_routes,
                slot: case.slot,
            };
            match (plan::plan(&root, &settings), &case.planned) {
                (Err(_), None) => None,
                (Ok(_), None) => Some(format!("{}: the planner did not refuse it", case.label)),
                (Err(error), Some(_)) => Some(format!("{}: {error}", case.label)),
                (Ok(planned), Some(golden)) => {
                    let output = difference(&planned.output, golden.output)
                        .map(|at| format!("{} output {at}", case.label));
                    let summary = difference(&planned.summary, golden.summary)
                        .map(|at| format!("{} summary {at}", case.label));
                    output
                        .into_iter()
                        .chain(summary)
                        .reduce(|a, b| format!("{a}\n{b}"))
                }
            }
        })
        .collect();
    assert!(
        failures.is_empty(),
        "plans differ from the golden plans:\n{}",
        failures.join("\n")
    );
}
