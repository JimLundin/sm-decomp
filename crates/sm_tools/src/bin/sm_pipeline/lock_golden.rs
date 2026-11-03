//! `lock-check` against `tools/scenario.py lock --check`'s verdicts over the
//! planner fixture and broken copies of it (`fixture/lock_check.rs`, written
//! by the Python at e85282582): a stale entry, a missing pin, a lockable row
//! neither locked nor unlocked, one both locked and unlocked, and a lock entry
//! for no scenario must all be rejected, naming the same problems.

use std::path::Path;

use crate::plan_golden::fixture;
use crate::scenario;

struct LockCase {
    label: &'static str,
    /// Text replacements in a copy of the fixture: file, old text, new text.
    edits: &'static [(&'static str, &'static str, &'static str)],
    passes: bool,
    /// What the check printed or exited with, ending in a newline; None where
    /// scenario.py crashed instead of naming the problem.
    message: Option<&'static str>,
}

include!("fixture/lock_check.rs");

fn copy(from: &Path, to: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

#[test]
fn checks_the_lock_as_scenario_py_did() {
    let mut differ = Vec::new();
    for case in CASES {
        let root = std::env::temp_dir().join(format!(
            "sm_pipeline-lock-{}-{}",
            std::process::id(),
            case.label
        ));
        copy(&fixture(), &root).expect("a copy of the fixture");
        for &(file, old, new) in case.edits {
            let path = root.join(file);
            let text = std::fs::read_to_string(&path).expect("a fixture file");
            assert_eq!(text.matches(old).count(), 1, "{}: {file}", case.label);
            std::fs::write(&path, text.replace(old, new)).expect("an edited fixture file");
        }
        let checked = scenario::lock_check(&root);
        let _ = std::fs::remove_dir_all(&root);
        let (passes, message) = match &checked {
            Ok(line) => (true, line),
            Err(problems) => (false, problems),
        };
        if passes != case.passes
            || case
                .message
                .is_some_and(|golden| golden != format!("{message}\n"))
        {
            differ.push(format!(
                "{}: {checked:?}, golden passes={} {:?}",
                case.label, case.passes, case.message
            ));
        }
    }
    assert!(differ.is_empty(), "{}", differ.join("\n"));
}
