//! Whether a pull request's changed files can affect a replay (ported from the
//! deleted `.github/scripts/replay_relevant.py`).
//!
//! verify.yml's verify-plan pipes `git diff --name-only` against the pull
//! request's base, and then against each recent head that passed Verify on its
//! branch, into `sm_pipeline relevant`. When every file of one comparison is
//! irrelevant the replays are skipped; anything else, including a file this
//! table cannot classify or a comparison the shell cannot fetch, runs the gate.

/// Files that cannot change what a replay executes or compares, as `fnmatch`
/// patterns: `*` matches any run of characters, `/` included, and `?` one.
pub const IRRELEVANT: &[&str] = &[
    "*.md",
    "docs/*",
    ".agents/*",
    "archive/*",
    "tools/lane/*",
    "tools/ram_ratchet.py",
    "tools/cycle_report.py",
    "tools/progress.sh",
    "tools/progress_accounting.py",
    "crates/sm_tools/src/bin/sm_pipeline/relevant.rs",
    // Measured row seconds only decide how rows pack into jobs, not what any row replays.
    ".github/scripts/row_seconds.*",
    // Tests the Quality jobs already run: target-package integration tests (cargo
    // test) and the mutation lists, whose anchors Quality checks.
    // sm/src/verification/*.rs is the verifier itself and stays relevant.
    // The workflows are relevant: verify.yml defines the Verify jobs.
    "crates/*/tests/*",
    "sm/src/verification/*mutations.py",
    "crates/sm_tools/src/bin/sm_pipeline/test_summary.rs",
];

/// Whether `text` matches the whole of `pattern`, case-sensitively, as
/// Python's `fnmatch.fnmatchcase` does for patterns without `[` sets.
fn matches(pattern: &str, text: &str) -> bool {
    let (pattern, text): (Vec<char>, Vec<char>) =
        (pattern.chars().collect(), text.chars().collect());
    let (mut at, mut from) = (0, 0);
    // The last `*` seen, and where in the text its match ends so far.
    let mut star: Option<(usize, usize)> = None;
    while from < text.len() {
        match pattern.get(at) {
            Some('*') => {
                star = Some((at, from));
                at += 1;
            }
            Some(&wanted) if wanted == '?' || wanted == text[from] => {
                at += 1;
                from += 1;
            }
            _ => match star {
                // Let the last `*` take one more character and try again.
                Some((star_at, star_from)) => {
                    star = Some((star_at, star_from + 1));
                    at = star_at + 1;
                    from = star_from + 1;
                }
                None => return false,
            },
        }
    }
    pattern[at..].iter().all(|&rest| rest == '*')
}

/// Whether a changed file cannot affect any replay.
pub fn irrelevant(path: &str) -> bool {
    IRRELEVANT.iter().any(|pattern| matches(pattern, path))
}

/// The files that can affect a replay, of a `git diff --name-only` listing.
pub fn relevant(listing: &str) -> Vec<&str> {
    listing
        .lines()
        .map(|line| line.trim_end_matches('\r'))
        .filter(|path| !path.is_empty() && !irrelevant(path))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    include!("fixture/relevant.rs");

    #[test]
    fn classifies_every_file_as_replay_relevant_py_did() {
        let differ: Vec<String> = PATHS
            .iter()
            .filter(|&&(path, skip)| irrelevant(path) != skip)
            .map(|&(path, skip)| format!("{path}: the Python table said irrelevant={skip}"))
            .collect();
        assert!(differ.is_empty(), "{}", differ.join("\n"));
        let verdicts: Vec<String> = PULL_REQUESTS
            .iter()
            .filter(|&&(_, skip, files)| {
                let listing: String = files
                    .iter()
                    .map(|&index| format!("{}\n", PATHS[index].0))
                    .collect();
                relevant(&listing).is_empty() != skip
            })
            .map(|&(number, skip, _)| format!("#{number}: the Python verdict was skip={skip}"))
            .collect();
        assert!(verdicts.is_empty(), "{}", verdicts.join("\n"));
    }

    #[test]
    fn matches_as_fnmatchcase() {
        assert!(matches("*", ""));
        assert!(matches("a*b*c", "a/x/b/y/c"));
        assert!(!matches("a*b*c", "a/x/b/y/cd"));
        assert!(matches("?.md", "x.md"));
        assert!(!matches("?.md", "xy.md"));
        assert!(matches("**x", "x"));
        assert!(!matches("docs/*", "docs"));
        assert!(!matches("*.md", "a.MD"));
        // The table uses no `[` sets, which this matcher would read literally.
        assert!(IRRELEVANT.iter().all(|pattern| !pattern.contains('[')));
    }

    #[test]
    fn an_empty_listing_has_nothing_relevant() {
        // replay_relevant.py skipped a comparison with no changed files.
        assert!(relevant("").is_empty());
        assert!(relevant("\n").is_empty());
        assert_eq!(relevant("docs/a.md\nsm/src/lib.rs\r\n"), ["sm/src/lib.rs"]);
    }
}
