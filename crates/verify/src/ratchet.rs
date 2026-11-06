//! The lockstep ratchet: a row's lockstep run reduced to the numbers that must not get worse,
//! and the plain-text file that holds them (`scenarios/locks/lockstep.ratchet.txt`).
//!
//! A summary is the Ticks compared, the Ticks fully equal, and per top-level field group the
//! Ticks it differs on and the first of them. The file holds two kinds of line per row:
//!
//! ```text
//! row area-brinstar ticks 476 equal 90
//! diff area-brinstar video 74 7
//! ```
//!
//! `diff ROW GROUP COUNT FIRST_FRAME`. A group with no line is equal on every Tick. The
//! first frame is information, the check compares the counts only.

use std::collections::BTreeMap;
use std::fmt;

use canonical::Segment;

use crate::lockstep::Report;

/// One group's differences over a run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Group {
    /// The Ticks on which the group has a differing field.
    pub differed: u64,
    /// The frame k+1 of the first of them.
    pub first: u64,
}

/// A row's lockstep numbers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Summary {
    /// The row's id.
    pub row: String,
    /// The Ticks compared.
    pub ticks: u64,
    /// The Ticks whose compared fields all agree.
    pub equal: u64,
    /// The groups that differ on at least one Tick, by name.
    pub groups: BTreeMap<String, Group>,
}

/// The top-level field a step's difference lies in: `video` for `video.memory.vram`.
fn group_of(difference: &canonical::Difference) -> Option<&'static str> {
    difference.path().iter().find_map(|segment| match segment {
        Segment::Field(name) => Some(*name),
        Segment::Variant(_) | Segment::Index(_) | Segment::Len => None,
    })
}

fn count(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

impl Summary {
    /// The numbers of `report`, run over `row`.
    #[must_use]
    pub fn of(row: &str, report: &Report) -> Self {
        let mut groups: BTreeMap<String, Group> = BTreeMap::new();
        for step in &report.steps {
            let mut seen: Vec<&'static str> =
                step.differences.iter().filter_map(group_of).collect();
            seen.sort_unstable();
            seen.dedup();
            for name in seen {
                groups
                    .entry(name.to_owned())
                    .and_modify(|group| group.differed = group.differed.saturating_add(1))
                    .or_insert(Group {
                        differed: 1,
                        first: step.frame.0,
                    });
            }
        }
        Self {
            row: row.to_owned(),
            ticks: count(report.compared()),
            equal: count(report.equal()),
            groups,
        }
    }

    /// How `self`, a fresh run, stands against `baseline`, the committed numbers.
    #[must_use]
    pub fn against(&self, baseline: &Self) -> Verdict {
        let mut regressions = Vec::new();
        let mut improved = false;
        if self.equal < baseline.equal {
            regressions.push(format!(
                "{}: Ticks fully equal {} -> {}",
                self.row, baseline.equal, self.equal
            ));
        }
        improved |= self.equal > baseline.equal;
        for (name, now) in &self.groups {
            let before = baseline.groups.get(name).map_or(0, |group| group.differed);
            if now.differed > before {
                regressions.push(format!(
                    "{}: {name} differs on {} Ticks (was {before})",
                    self.row, now.differed
                ));
            }
            improved |= now.differed < before;
        }
        improved |= baseline
            .groups
            .keys()
            .any(|name| !self.groups.contains_key(name));
        Verdict {
            regressions,
            improved,
        }
    }
}

impl fmt::Display for Summary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "row {} ticks {} equal {}",
            self.row, self.ticks, self.equal
        )?;
        for (name, group) in &self.groups {
            writeln!(
                f,
                "diff {} {name} {} {}",
                self.row, group.differed, group.first
            )?;
        }
        Ok(())
    }
}

/// The outcome of checking a run against the ratchet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Verdict {
    /// What got worse; empty when the run holds the ratchet.
    pub regressions: Vec<String>,
    /// Whether some count got better, so the file should be updated.
    pub improved: bool,
}

/// Why the ratchet file doesn't read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError {
    /// The 1-based line.
    pub line: usize,
    /// What is wrong with it.
    pub message: &'static str,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for ParseError {}

/// Reads the ratchet file: the rows in file order.
///
/// # Errors
///
/// A [`ParseError`] for a line that isn't a comment, blank, `row` or `diff` line, or a `diff`
/// line for a row that has no `row` line before it.
pub fn parse(text: &str) -> Result<Vec<Summary>, ParseError> {
    let mut rows: Vec<Summary> = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let line_number = index.saturating_add(1);
        let fail = |message| ParseError {
            line: line_number,
            message,
        };
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let number = |word: &str| word.parse::<u64>().map_err(|_| fail("not a number"));
        let words: Vec<&str> = line.split_whitespace().collect();
        match words.as_slice() {
            ["row", id, "ticks", ticks, "equal", equal] => rows.push(Summary {
                row: (*id).to_owned(),
                ticks: number(ticks)?,
                equal: number(equal)?,
                groups: BTreeMap::new(),
            }),
            ["diff", id, name, differed, first] => {
                let row = rows
                    .iter_mut()
                    .find(|row| row.row == *id)
                    .ok_or_else(|| fail("diff line before its row line"))?;
                row.groups.insert(
                    (*name).to_owned(),
                    Group {
                        differed: number(differed)?,
                        first: number(first)?,
                    },
                );
            }
            _ => return Err(fail("expected `row` or `diff` line")),
        }
    }
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::{Group, Summary, parse};
    use std::collections::BTreeMap;

    fn summary(equal: u64, video: Option<u64>) -> Summary {
        Summary {
            row: "r".to_owned(),
            ticks: 10,
            equal,
            groups: video
                .map(|differed| ("video".to_owned(), Group { differed, first: 3 }))
                .into_iter()
                .collect::<BTreeMap<_, _>>(),
        }
    }

    #[test]
    fn the_file_reads_back_what_a_summary_prints() {
        let original = summary(7, Some(2));
        assert_eq!(parse(&original.to_string()), Ok(vec![original]));
    }

    #[test]
    fn worse_counts_fail_and_better_counts_are_reported() {
        let baseline = summary(7, Some(2));
        assert_eq!(summary(7, Some(2)).against(&baseline).regressions.len(), 0);
        assert_eq!(summary(6, Some(2)).against(&baseline).regressions.len(), 1);
        assert_eq!(summary(7, Some(3)).against(&baseline).regressions.len(), 1);
        let better = summary(8, None).against(&baseline);
        assert_eq!(better.regressions.len(), 0);
        assert!(better.improved);
    }
}
