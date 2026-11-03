//! Until the workflow switches and the Python planner is deleted, the Rust
//! plan must equal `.github/scripts/plan_verify.py`'s byte for byte: the
//! GITHUB_OUTPUT matrix and the step summary, for every scope over the
//! committed scenarios and lock. CI's Quality job runs it with `cargo test`;
//! it needs `python3` (or PYTHON) on the path.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::plan::{self, Scope, Settings};
use crate::scenario;

fn root() -> PathBuf {
    Path::new(option_env!("CARGO_MANIFEST_DIR").unwrap_or("crates/sm_tools")).join("../..")
}

struct Case {
    label: String,
    settings: Settings,
    /// Whether both planners must succeed; otherwise both must fail.
    succeeds: bool,
}

fn case(
    label: &str,
    scope: Scope,
    selected: Vec<String>,
    oracle_sample: bool,
    cover_routes: bool,
    slot: u64,
) -> Case {
    let succeeds = !label.ends_with("-fails");
    Case {
        label: label.to_owned(),
        settings: Settings {
            scope,
            selected,
            oracle_sample,
            cover_routes,
            slot,
        },
        succeeds,
    }
}

/// The Python planner's GITHUB_OUTPUT and summary, or None when it failed.
fn python(root: &Path, case: &Case) -> Option<(String, String)> {
    let directory =
        std::env::temp_dir().join(format!("sm_pipeline-{}-{}", std::process::id(), case.label));
    std::fs::create_dir_all(&directory).expect("a temporary directory");
    let (output, summary) = (directory.join("output"), directory.join("summary"));
    for path in [&output, &summary] {
        std::fs::write(path, "").expect("an empty output file");
    }
    let settings = &case.settings;
    let flag = |on: bool| if on { "1" } else { "" };
    let scope = match settings.scope {
        Scope::Gate => "gate",
        Scope::Full => "full",
        Scope::Rows => "rows",
        Scope::Coverage => "coverage",
    };
    let python = std::env::var("PYTHON").unwrap_or_else(|_| "python3".into());
    let status = Command::new(&python)
        .arg("-B")
        .arg(root.join(".github/scripts/plan_verify.py"))
        .current_dir(root)
        .env_remove("FULL_ROUTE")
        .env("SCOPE", scope)
        .env("SELECTED", settings.selected.join(" "))
        .env("ORACLE_SAMPLE", flag(settings.oracle_sample))
        .env("COVER_ROUTES", flag(settings.cover_routes))
        .env("ORACLE_SLOT", settings.slot.to_string())
        .env("GITHUB_OUTPUT", &output)
        .env("GITHUB_STEP_SUMMARY", &summary)
        .stdout(std::process::Stdio::null())
        .status()
        .unwrap_or_else(|error| panic!("{python} runs the Python planner: {error}"));
    let read = |path: &Path| std::fs::read_to_string(path).expect("the planner's output");
    let result = status.success().then(|| (read(&output), read(&summary)));
    let _ = std::fs::remove_dir_all(&directory);
    result
}

/// Where two texts first differ, with some context, or None when equal.
fn difference(rust: &str, python: &str) -> Option<String> {
    let at = rust
        .bytes()
        .zip(python.bytes())
        .position(|(a, b)| a != b)
        .or_else(|| (rust.len() != python.len()).then(|| rust.len().min(python.len())))?;
    let window = |text: &str| {
        let start = text.floor_char_boundary(at.saturating_sub(160));
        let end = text.ceil_char_boundary((at + 160).min(text.len()));
        text[start..end].to_owned()
    };
    Some(format!(
        "first difference at byte {at}:\n  rust:   {}\n  python: {}",
        window(rust),
        window(python)
    ))
}

#[test]
fn plans_as_plan_verify_py_does() {
    let root = root();
    let lock = scenario::read_lock(&root).expect("the lock");
    let rows = scenario::rows(&root, &lock).expect("the scenarios");
    let ids: Vec<String> = rows.iter().map(|row| row.id.clone()).collect();
    let every = |step: usize, offset: usize| {
        ids.iter()
            .skip(offset)
            .step_by(step)
            .cloned()
            .collect::<Vec<_>>()
    };
    let first_where = |keep: &dyn Fn(&scenario::Row) -> bool| {
        rows.iter().find(|row| keep(row)).map(|row| row.id.clone())
    };
    let mut mixed = every(11, 3);
    mixed.extend(first_where(&|row| row.start_frame > 20_000));
    mixed.extend(first_where(&|row| row.checkpoint.is_some()));
    mixed.extend(first_where(&|row| row.chain.as_deref() == Some("any")));
    let corpus_one: Vec<String> = first_where(&|row| row.chain.is_none() && row.pending.is_none())
        .into_iter()
        .collect();
    let room_one: Vec<String> = first_where(&|row| row.chain.as_deref() == Some("100"))
        .into_iter()
        .collect();
    // Slots around 2026-11-03 and two neighbours, so the sample rotates.
    let slot = 165_000;
    let cases = [
        case("gate", Scope::Gate, Vec::new(), false, false, slot),
        case("full", Scope::Full, Vec::new(), false, false, slot),
        case("full-sample", Scope::Full, Vec::new(), true, false, slot),
        case(
            "full-sample-next",
            Scope::Full,
            Vec::new(),
            true,
            false,
            slot + 1,
        ),
        case("full-sample-zero", Scope::Full, Vec::new(), true, false, 0),
        case("rows-every-5", Scope::Rows, every(5, 0), false, false, slot),
        case("rows-mixed", Scope::Rows, mixed, false, false, slot),
        case(
            "rows-corpus-one",
            Scope::Rows,
            corpus_one,
            false,
            false,
            slot,
        ),
        case("rows-room-one", Scope::Rows, room_one, false, false, slot),
        case(
            "rows-unknown-fails",
            Scope::Rows,
            vec!["no-such-row".into()],
            false,
            false,
            slot,
        ),
        case(
            "rows-empty-fails",
            Scope::Rows,
            Vec::new(),
            false,
            false,
            slot,
        ),
        case("coverage", Scope::Coverage, Vec::new(), false, false, slot),
        case(
            "coverage-routes",
            Scope::Coverage,
            Vec::new(),
            false,
            true,
            slot,
        ),
    ];
    let failures: Vec<String> = std::thread::scope(|threads| {
        let running: Vec<_> = cases
            .iter()
            .map(|case| {
                let root = &root;
                threads.spawn(move || {
                    let rust = plan::plan(root, &case.settings)
                        .ok()
                        .map(|planned| (planned.output, planned.summary));
                    let python = python(root, case);
                    match (rust, python) {
                        (None, None) if !case.succeeds => None,
                        (None, None) => Some(format!("{}: both planners failed", case.label)),
                        (Some(_), _) if !case.succeeds => {
                            Some(format!("{}: the Rust planner succeeded", case.label))
                        }
                        (Some(_), None) => Some(format!(
                            "{}: the Python planner failed and the Rust one did not",
                            case.label
                        )),
                        (None, Some(_)) => Some(format!(
                            "{}: the Rust planner failed and the Python one did not",
                            case.label
                        )),
                        (Some((output, summary)), Some((python_output, python_summary))) => {
                            let output = difference(&output, &python_output)
                                .map(|at| format!("{} output {at}", case.label));
                            let summary = difference(&summary, &python_summary)
                                .map(|at| format!("{} summary {at}", case.label));
                            output
                                .into_iter()
                                .chain(summary)
                                .reduce(|a, b| format!("{a}\n{b}"))
                        }
                    }
                })
            })
            .collect();
        running
            .into_iter()
            .filter_map(|thread| thread.join().expect("a planner thread"))
            .collect()
    });
    assert!(
        failures.is_empty(),
        "the Rust and Python plans differ:\n{}",
        failures.join("\n")
    );
    println!("{} scopes planned identically", cases.len());
}
