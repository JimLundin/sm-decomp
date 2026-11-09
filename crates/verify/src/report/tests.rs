use std::path::Path;

use super::lines::code_lines;
use super::{Coverage, baseline, check, coverage, current, fit, floors, generate, grouped, tables};

/// The repository root, from this crate's manifest.
fn root() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
}

#[test]
fn committed_report_is_current() {
    assert_eq!(check(root()), Ok(()));
}

#[test]
fn a_changed_report_is_stale() {
    let fresh = "# Rebuild report\n\n| a | 1 |\n";
    assert_eq!(current(fresh, fresh), Ok(()));
    assert_eq!(current(&fresh.replace('\n', "\r\n"), fresh), Ok(()));
    assert!(current("# Rebuild report\n\n| a | 2 |\n", fresh).is_err());
    assert!(current("", fresh).is_err());
}

#[test]
fn the_report_is_deterministic() {
    let first = generate(root());
    assert!(first.is_ok());
    assert_eq!(first, generate(root()));
}

#[test]
fn blank_and_comment_lines_dont_count() {
    let src = "//! Crate doc.\n\n/// Item doc.\nfn a() {\n    // a comment\n    let x = 1; // trailing\n}\n";
    assert_eq!(code_lines(src), 3);
}

#[test]
fn block_comments_nest() {
    let src = "/* one\n /* two */\n still */\nfn a() {}\n/** doc */\n";
    assert_eq!(code_lines(src), 1);
}

#[test]
fn literals_are_code_and_hide_comment_marks() {
    let src = "let a = \"// not a comment\";\nlet b = \"line\n\n/* inside */\";\nlet c = r#\"x \" // y\"#;\n";
    assert_eq!(code_lines(src), 4);
}

#[test]
fn test_items_dont_count() {
    let src = "fn a() {}\n#[cfg(test)]\nmod tests {\n    fn b() { let c = '{'; }\n}\n#[cfg(test)]\nuse std::fmt;\nfn d<'a>(x: &'a str) -> char { '}' }\n#[cfg(test)]\nmod tests;\n";
    assert_eq!(code_lines(src), 2);
}

#[test]
fn figures_read_as_an_observer_would() {
    assert_eq!(grouped(0), "0");
    assert_eq!(grouped(1_234_567), "1,234,567");
    assert_eq!(fit(8 * 104, Some(100)), "1.04");
    assert_eq!(fit(8, None), "-");
}

#[test]
fn the_baseline_commit_is_the_last_the_budget_paragraph_names() {
    let section =
        "**Budget.** at spine `bc6f74f9c`, over `x + 1` at `c7f927ab6`.\n\nLater `0123456789`.\n";
    assert_eq!(baseline(section).as_deref(), Some("c7f927ab6"));
    // A baseline paragraph names it first, wherever it stands.
    let section = "**Budget.** k = 36, at `c7f927ab6`.\n\n**Baseline.** at spine `387640758`.\n";
    assert_eq!(baseline(section).as_deref(), Some("387640758"));
}

#[test]
fn floors_and_tables_parse() {
    let parsed = floors("# floors\ncamera r1 10\ncamera r2 5\n");
    assert_eq!(parsed.map(|floors| floors.values().sum::<usize>()), Ok(15));
    assert!(floors("camera r1\n").is_err());
    assert_eq!(
        tables("[[vector]]\nid = 1\n[[pose_step]]\n[[vector]]\n", "vector"),
        2
    );
}

#[test]
fn coverage_parses() {
    let parsed = coverage("# coverage\ncamera frames 10 8 1 5\ncamera entry Follow 3 4\n");
    let camera = Coverage {
        frames: Some([10, 8, 1, 5]),
        entries: vec![("Follow".to_owned(), 3, 4)],
    };
    assert_eq!(
        parsed.map(|domains| domains.get("camera") == Some(&camera)),
        Ok(true)
    );
    assert!(coverage("camera frames 10 8 1\n").is_err());
    assert!(coverage("camera entry Follow x 4\n").is_err());
}
