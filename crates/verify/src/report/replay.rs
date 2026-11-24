//! Reporting consumes the acceptance schema; it never reinterprets a replay as gameplay.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

use crate::replay::baseline::{Baseline, Floor};
use crate::replay::index::{Case, CaseKind, IndexSnapshot};
use crate::replay::result::Outcome;

const INDEX: &str = "scenarios/replay/index.toml";
const BASELINE: &str = "scenarios/replay/baseline.toml";

pub(super) struct Snapshot {
    rows: Vec<(Case, Floor)>,
}

impl Snapshot {
    pub(super) fn read(root: &Path) -> Result<Self, String> {
        let snapshot = IndexSnapshot::read(&root.join(INDEX))
            .map_err(|fault| format!("{INDEX}: {fault:?}"))?;
        let baseline = Baseline::read(&root.join(BASELINE))
            .map_err(|fault| format!("{BASELINE}: {fault:?}"))?;
        baseline
            .validate_index(&snapshot)
            .map_err(|fault| format!("{BASELINE}: {fault:?}"))?;
        let mut floors: BTreeMap<_, _> = baseline
            .cases
            .into_iter()
            .map(|floor| (floor.id.clone(), floor))
            .collect();
        let rows = snapshot
            .into_index()
            .cases
            .into_iter()
            .map(|case| {
                floors
                    .remove(&case.id)
                    .map(|floor| (case, floor))
                    .ok_or_else(|| format!("{BASELINE}: missing case"))
            })
            .collect::<Result<_, _>>()?;
        Ok(Self { rows })
    }

    fn rows(&self) -> impl Iterator<Item = (&Case, &Floor)> {
        self.rows.iter().map(|(case, floor)| (case, floor))
    }

    pub(super) fn section(&self, out: &mut String) {
        out.push_str("\n## Replay evidence\n\nThese are committed acceptance requirements, not a fresh replay run. \
            One runner executes the selected production systems and compares independent observations. \
            Each row keeps its own scope; overlapping cases are never added into whole-game progress.\n\n");
        for kind in [CaseKind::Scoped, CaseKind::Integrated, CaseKind::Movie] {
            let _ = writeln!(out, "### {}\n", kind_label(kind));
            let rows: Vec<_> = self.rows().filter(|(case, _)| case.kind == kind).collect();
            if rows.is_empty() {
                out.push_str("No indexed case. No acceptance credit.\n\n");
                continue;
            }
            out.push_str("| Case | Required outcome | Compared cuts | Null cuts | Gain over null | Fields | Outputs | Restorations |\n\
                | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |\n");
            for (case, floor) in &rows {
                let counts = &floor.counts;
                let gain = match floor.expected {
                    Outcome::Pass => counts.cuts.saturating_sub(counts.null_cuts).to_string(),
                    Outcome::Difference { .. }
                    | Outcome::Unsupported { .. }
                    | Outcome::DataFailure { .. }
                    | Outcome::CheckpointFailure { .. } => "unaccepted prefix".to_owned(),
                };
                let _ = writeln!(
                    out,
                    "| {} | {} | {} | {} | {} | {} | {} | {} |",
                    cell(&case.id.0),
                    cell(&outcome(&floor.expected)),
                    counts.cuts,
                    counts.null_cuts,
                    gain,
                    counts.fields,
                    counts.outputs,
                    counts.restorations
                );
            }
            out.push('\n');
            for (case, _) in rows {
                let witness = if case.rules.is_empty() {
                    "declares no rule witness".to_owned()
                } else {
                    format!(
                        "witnesses {}",
                        case.rules
                            .iter()
                            .map(|rule| cell(rule))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                };
                let _ = writeln!(
                    out,
                    "- **{}** {}. Limits: {}.",
                    cell(&case.id.0),
                    witness,
                    case.limits
                        .iter()
                        .map(|limit| cell(limit))
                        .collect::<Vec<_>>()
                        .join("; ")
                );
            }
            out.push('\n');
        }
        out.push_str(
            "A held Unsupported boundary remains Unsupported and proves no completed case. \
            Earlier claim snapshots are retained in `scenarios/replay/prior-claims.toml`; \
            `scenarios/replay/transition.toml` accounts for their replacement or retirement. \
            Rule names identify intended witnesses, not a count of proven rules.\n",
        );
    }
}

const fn kind_label(kind: CaseKind) -> &'static str {
    match kind {
        CaseKind::Scoped => "Scoped cases",
        CaseKind::Integrated => "Integrated cases",
        CaseKind::Movie => "Whole-movie cases",
    }
}

fn outcome(outcome: &Outcome) -> String {
    match outcome {
        Outcome::Pass => "Pass".to_owned(),
        Outcome::Unsupported { cut, dependency } => format!("Unsupported at {cut:?}: {dependency}"),
        Outcome::Difference { cut, field } => format!("Difference at {cut}: {field}"),
        Outcome::DataFailure { cut, source } => format!("Data failure at {cut:?}: {source}"),
        Outcome::CheckpointFailure { cut } => format!("Checkpoint failure at {cut}"),
    }
}

fn cell(text: &str) -> String {
    text.replace('|', "&#124;").replace(['\r', '\n'], " ")
}
