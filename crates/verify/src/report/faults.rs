//! Current catalogue assignments and the detections credited by a saved fault report.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::Path;

use serde::Deserialize;

use super::{Measured, read, replay::cell};
use crate::lock::Sha256;

const MANIFEST: &str = "tools/mutants.toml";

#[derive(Deserialize)]
struct Manifest {
    fault: Vec<Fault>,
}

#[derive(Deserialize)]
struct Fault {
    name: String,
    rule: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Evidence {
    format: u16,
    manifest_sha256: String,
    detected: Vec<String>,
}

fn detections(path: &Path, manifest: &str) -> Result<BTreeSet<String>, String> {
    let evidence: Evidence =
        toml::from_str(&read(path)?).map_err(|error| format!("{}: {error}", path.display()))?;
    if evidence.format != 1
        || evidence.manifest_sha256 != Sha256::of(manifest.as_bytes()).to_string()
    {
        return Err(format!(
            "{}: incompatible fault evidence or changed roster",
            path.display()
        ));
    }
    let count = evidence.detected.len();
    let names: BTreeSet<_> = evidence.detected.into_iter().collect();
    if names.len() != count {
        return Err(format!("{}: repeated fault detection", path.display()));
    }
    Ok(names)
}

pub(super) fn section(
    out: &mut String,
    root: &Path,
    domains: &[Measured],
    evidence: Option<&Path>,
) -> Result<(), String> {
    let source = read(&root.join(MANIFEST))?;
    let manifest: Manifest =
        toml::from_str(&source).map_err(|error| format!("{MANIFEST}: {error}"))?;
    let mut detected = evidence
        .map(|path| detections(path, &source))
        .transpose()?
        .unwrap_or_default();
    let model_rules = domains.iter().flat_map(|model| {
        model
            .count
            .blocks
            .iter()
            .filter(|block| block.entry)
            .map(|block| format!("{}/{}", model.domain.name, block.heading))
    });
    let rooms = std::iter::once(game::production::played::intro_rooms::RULE)
        .chain(game::production::shaft_entry::CATALOGUE)
        .map(str::to_owned);
    let mut rows: BTreeMap<_, (Vec<String>, Vec<String>)> = model_rules
        .chain(rooms)
        .map(|rule| (rule, (Vec::new(), Vec::new())))
        .collect();

    let mut names = BTreeSet::new();
    let mut unassigned = 0_usize;
    for fault in manifest.fault {
        if fault.name.is_empty() || !names.insert(fault.name.clone()) {
            return Err(format!("{MANIFEST}: empty or duplicate fault name"));
        }
        if let Some(rule) = fault.rule {
            let (assigned, confirmed) = rows
                .get_mut(&rule)
                .ok_or_else(|| format!("{MANIFEST}: unknown catalogue rule {rule}"))?;
            let name = cell(&fault.name);
            if detected.remove(&fault.name) {
                confirmed.push(name.clone());
            }
            assigned.push(name);
        } else {
            unassigned = unassigned.saturating_add(1);
        }
    }
    if let Some(name) = detected.first() {
        return Err(format!(
            "fault evidence names an unassigned or absent fault {name}"
        ));
    }
    out.push_str("\n## Planted fault coverage\n\n");
    let _ = writeln!(out, "{unassigned} faults have no rule assignment.\n");
    if evidence.is_none() {
        out.push_str("Supply a saved receipt with `--fault-evidence` to see which assigned faults the run detected.\n\n");
    }
    out.push_str("| Catalogue rule | Assigned faults | Detected faults |\n| --- | --- | --- |\n");
    for (rule, (assigned, confirmed)) in rows {
        let confirmed = if confirmed.is_empty() {
            "unknown".to_owned()
        } else {
            confirmed.join(", ")
        };
        let _ = writeln!(
            out,
            "| {} | {} | {confirmed} |",
            cell(&rule),
            assigned.join(", ")
        );
    }
    Ok(())
}
