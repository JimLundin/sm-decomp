//! Summarise a `cargo test` log per test binary and require every expected one
//! (ported from the deleted `.github/scripts/test_summary.py`).
//!
//! Each package directory (e.g. crates/sm_game) must have run its library unit
//! tests, every tests/*.rs integration test and its doc-tests. A test file that
//! is not built, or a binary that stops before its result line, fails the step
//! instead of passing silently.

use std::path::Path;

/// A package whose test binaries must all run: its directory name and the
/// file names of its tests/*.rs integration tests.
pub struct Package {
    pub name: String,
    pub tests: Vec<String>,
}

impl Package {
    /// The package in a directory, as its tests/ directory lists it.
    pub fn read(directory: &Path) -> Result<Self, String> {
        let name = directory
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .ok_or_else(|| format!("{} names no package", directory.display()))?;
        let mut tests: Vec<String> = match std::fs::read_dir(directory.join("tests")) {
            Ok(entries) => entries
                .filter_map(Result::ok)
                .filter_map(|entry| entry.file_name().into_string().ok())
                .filter(|file| file.ends_with(".rs"))
                .collect(),
            Err(_) => Vec::new(),
        };
        tests.sort();
        Ok(Self { name, tests })
    }

    /// The binaries it must run, in the order the table lists missing ones.
    fn expected(&self) -> impl Iterator<Item = String> + '_ {
        [
            format!("{} (unit)", self.name),
            format!("{} (doc)", self.name),
        ]
        .into_iter()
        .chain(self.tests.iter().map(|test| format!("tests/{test}")))
    }
}

/// One `test result:` line: status, then passed, failed, ignored, measured and
/// filtered-out counts, as the log wrote them.
type Outcome = (String, [String; 5]);

/// The summary of one log.
pub struct Summary {
    /// The Markdown table, ending in a newline.
    pub report: String,
    /// Expected binaries that did not run.
    pub missing: Vec<String>,
    /// Binaries that failed or gave no result.
    pub failed: Vec<String>,
}

/// Python's `str.isspace`, which `\s` and `\S` use.
fn space(character: char) -> bool {
    character.is_whitespace() || ('\x1c'..='\x1f').contains(&character)
}

/// The leading run of non-space characters, when it is not empty (`\S+`).
fn word(text: &str) -> Option<(&str, &str)> {
    let end = text.find(space).unwrap_or(text.len());
    (end > 0).then(|| text.split_at(end))
}

/// The leading run of characters `keep` accepts, when it is not empty.
fn run_of(text: &str, keep: impl Fn(char) -> bool) -> Option<(&str, &str)> {
    let end = text
        .find(|character| !keep(character))
        .unwrap_or(text.len());
    (end > 0).then(|| text.split_at(end))
}

/// The crate of a test executable path: `(?:.*[\\/])?deps[\\/](crate)-hex(?:\.exe)?\)`,
/// the last such `deps` directory first, as the greedy prefix finds it.
fn deps_crate(text: &str) -> Option<&str> {
    let at = |start: usize| -> Option<&str> {
        let rest = text[start..].strip_prefix("deps")?;
        let rest = rest.strip_prefix(['/', '\\'])?;
        let (name, rest) = run_of(rest, |c| c.is_ascii_alphanumeric() || c == '_')?;
        let (_, rest) = run_of(
            rest.strip_prefix('-')?,
            |c| matches!(c, '0'..='9' | 'a'..='f'),
        )?;
        (rest.starts_with(".exe)") || rest.starts_with(')')).then_some(name)
    };
    let separators = text
        .char_indices()
        .filter(|&(_, character)| matches!(character, '/' | '\\'))
        .map(|(index, _)| index + 1);
    let starts: Vec<usize> = separators.collect();
    starts.into_iter().rev().chain([0]).find_map(at)
}

/// A `Running [unittests ]SOURCE (…deps/CRATE-HASH)` line: whether it runs unit
/// tests, its source and its crate.
fn running(line: &str) -> Option<(bool, &str, &str)> {
    let rest = line.trim_start_matches(space).strip_prefix("Running ")?;
    fn binary(rest: &str) -> Option<(&str, &str)> {
        let (source, rest) = word(rest)?;
        Some((source, deps_crate(rest.strip_prefix(" (")?)?))
    }
    rest.strip_prefix("unittests ")
        .and_then(|rest| binary(rest).map(|(source, name)| (true, source, name)))
        .or_else(|| binary(rest).map(|(source, name)| (false, source, name)))
}

/// A `Doc-tests CRATE` line's crate.
fn doctests(line: &str) -> Option<&str> {
    let rest = line.trim_start_matches(space).strip_prefix("Doc-tests ")?;
    word(rest).map(|(name, _)| name)
}

/// A `test result: STATUS. N passed; N failed; N ignored; N measured; N filtered out` line.
fn result(line: &str) -> Option<Outcome> {
    let rest = line.strip_prefix("test result: ")?;
    let (status, rest) = run_of(rest, |c| c.is_alphanumeric() || c == '_')?;
    let mut rest = rest.strip_prefix(". ")?;
    let mut counts: [String; 5] = Default::default();
    for (count, label) in counts.iter_mut().zip([
        " passed; ",
        " failed; ",
        " ignored; ",
        " measured; ",
        " filtered out",
    ]) {
        let (digits, after) = run_of(rest, |c| c.is_ascii_digit())?;
        *count = digits.to_owned();
        rest = after.strip_prefix(label)?;
    }
    Some((status.to_owned(), counts))
}

/// The log's lines as Python's text mode and `splitlines` split them.
fn lines(log: &str) -> impl Iterator<Item = &str> {
    let mut rest = log;
    std::iter::from_fn(move || {
        if rest.is_empty() {
            return None;
        }
        let breaks = |c: char| {
            matches!(
                c,
                '\n' | '\r'
                    | '\x0b'
                    | '\x0c'
                    | '\x1c'
                    | '\x1d'
                    | '\x1e'
                    | '\u{85}'
                    | '\u{2028}'
                    | '\u{2029}'
            )
        };
        let end = rest.find(breaks).unwrap_or(rest.len());
        let line = &rest[..end];
        let after = &rest[end..];
        let width = if after.starts_with("\r\n") {
            2
        } else {
            after.chars().next().map_or(0, char::len_utf8)
        };
        rest = &after[width..];
        Some(line)
    })
}

/// Each test binary in log order, with its result, or None when it gave none.
fn binaries(log: &str) -> Vec<(String, Option<Outcome>)> {
    let mut current: Option<String> = None;
    let mut found = Vec::new();
    for line in lines(log) {
        let started = running(line)
            .map(|(unit, source, name)| {
                if unit {
                    format!("{name} (unit)")
                } else {
                    source.replace('\\', "/")
                }
            })
            .or_else(|| doctests(line).map(|name| format!("{name} (doc)")));
        if let Some(name) = started {
            if let Some(unfinished) = current.replace(name) {
                found.push((unfinished, None));
            }
            continue;
        }
        if let Some(outcome) = result(line)
            && let Some(name) = current.take()
        {
            found.push((name, Some(outcome)));
        }
    }
    found.extend(current.map(|name| (name, None)));
    found
}

/// Summarise a log against the packages whose binaries must all run.
pub fn summarise(log: &str, packages: &[Package]) -> Summary {
    let found = binaries(log);
    let mut rows = vec![
        "| Test binary | Result | Passed | Failed | Ignored | Filtered |".to_owned(),
        "| --- | --- | --- | --- | --- | --- |".to_owned(),
    ];
    let mut failed = Vec::new();
    for (name, outcome) in &found {
        match outcome {
            None => {
                rows.push(format!("| `{name}` | **no result** | | | | |"));
                failed.push(name.clone());
            }
            Some((status, [passed, fails, ignored, _, filtered])) => {
                let shown = if status == "ok" {
                    status.clone()
                } else {
                    format!("**{status}**")
                };
                rows.push(format!(
                    "| `{name}` | {shown} | {passed} | {fails} | {ignored} | {filtered} |"
                ));
                if status != "ok" {
                    failed.push(name.clone());
                }
            }
        }
    }
    let missing: Vec<String> = packages
        .iter()
        .flat_map(Package::expected)
        .filter(|name| !found.iter().any(|(ran, _)| ran == name))
        .collect();
    rows.extend(
        missing
            .iter()
            .map(|name| format!("| `{name}` | **not run** | | | | |")),
    );
    Summary {
        report: format!("### Target package tests\n\n{}\n", rows.join("\n")),
        missing,
        failed,
    }
}

impl Summary {
    /// What the step prints to stderr, one line per problem.
    pub fn diagnostics(&self) -> String {
        let mut text = String::new();
        if !self.missing.is_empty() {
            text += &format!(
                "expected test binaries that did not run: {}\n",
                self.missing.join(", ")
            );
        }
        if !self.failed.is_empty() {
            text += &format!(
                "test binaries that failed or gave no result: {}\n",
                self.failed.join(", ")
            );
        }
        text
    }

    pub fn passed(&self) -> bool {
        self.missing.is_empty() && self.failed.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// How a case's log derives from a stored one.
    enum Log {
        Unchanged,
        /// Only its first lines, as if the binary after them never finished.
        Lines(usize),
        /// Windows test executables (`C:\…\deps\crate-hash.exe`).
        WindowsPaths,
        Crlf,
    }

    struct Golden {
        label: &'static str,
        log: &'static str,
        derive: Log,
        packages: &'static [(&'static str, &'static [&'static str])],
        /// test_summary.py's stdout (the table and print's newline).
        report: &'static str,
        diagnostics: &'static str,
        fails: bool,
    }

    include!("fixture/test_summary.rs");

    fn derive(log: &str, how: &Log) -> String {
        match how {
            Log::Unchanged => log.to_owned(),
            Log::Lines(count) => {
                let kept: Vec<&str> = log.split('\n').take(*count).collect();
                format!("{}\n", kept.join("\n"))
            }
            Log::WindowsPaths => log
                .split('\n')
                .map(|line| match line.find("(target/debug/deps/") {
                    Some(at) if line.ends_with(')') => {
                        let binary = &line[at + "(target/debug/deps/".len()..line.len() - 1];
                        format!("{}(C:\\a\\target\\debug\\deps\\{binary}.exe)", &line[..at])
                    }
                    _ => line.to_owned(),
                })
                .collect::<Vec<_>>()
                .join("\n"),
            Log::Crlf => log.replace('\n', "\r\n"),
        }
    }

    #[test]
    fn summarises_as_test_summary_py_did() {
        let mut differ = Vec::new();
        for case in CASES {
            let packages: Vec<Package> = case
                .packages
                .iter()
                .map(|&(name, tests)| Package {
                    name: name.to_owned(),
                    tests: tests.iter().map(|&test| test.to_owned()).collect(),
                })
                .collect();
            let summary = summarise(&derive(case.log, &case.derive), &packages);
            if format!("{}\n", summary.report) != case.report {
                differ.push(format!(
                    "{}: report\n{}\n---- golden\n{}",
                    case.label, summary.report, case.report
                ));
            }
            if summary.diagnostics() != case.diagnostics || summary.passed() == case.fails {
                differ.push(format!(
                    "{}: diagnostics {:?}, golden {:?}",
                    case.label,
                    summary.diagnostics(),
                    case.diagnostics
                ));
            }
        }
        assert!(differ.is_empty(), "{}", differ.join("\n"));
    }

    #[test]
    fn reads_the_lines_cargo_writes() {
        assert_eq!(
            running(
                "     Running unittests src/lib.rs (target/debug/deps/sm_data-04bafd2c9d559eeb)"
            ),
            Some((true, "src/lib.rs", "sm_data"))
        );
        assert_eq!(
            running("     Running tests/a.rs (C:\\t\\deps\\x\\deps\\a_b-0f.exe)"),
            Some((false, "tests/a.rs", "a_b"))
        );
        // The last deps directory that fits, then the first.
        assert_eq!(
            running("Running x (deps/a-1/deps/b-2)"),
            Some((false, "x", "b"))
        );
        assert_eq!(
            running("Running x (deps/a-1) (q/deps/z)"),
            Some((false, "x", "a"))
        );
        assert_eq!(
            running("Running unittests (deps/u-1)"),
            Some((false, "unittests", "u"))
        );
        assert_eq!(running("Running x (deps/A-F1)"), None);
        assert_eq!(doctests("   Doc-tests sm_game"), Some("sm_game"));
        assert_eq!(
            result(
                "test result: FAILED. 31 passed; 1 failed; 0 ignored; 0 measured; 2 filtered out; finished"
            ),
            Some((
                "FAILED".to_owned(),
                ["31", "1", "0", "0", "2"].map(str::to_owned)
            ))
        );
        assert_eq!(
            result(" test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out"),
            None
        );
        assert_eq!(
            lines("a\r\nb\rc\n\nd").collect::<Vec<_>>(),
            ["a", "b", "c", "", "d"]
        );
    }
}
