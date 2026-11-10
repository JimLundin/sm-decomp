//! The catalogue checked against the spec (`docs/behaviours/blocks.md`): one entry per spec entry,
//! in the spec's order, each with the spec's summary, kind and switch.

#![cfg(test)]

use clean_model_blocks::CATALOGUE;
use clean_types::{Kind, Switch};

/// The spec, whose entries the catalogue lists.
const SPEC: &str = include_str!("../../../../docs/behaviours/blocks.md");

/// How the spec indents a line that carries on the bullet above it.
const CARRIES_ON: &str = "  ";

/// A spec entry as the spec states it: its name, its summary, its kind and its switch.
struct SpecEntry {
    name: String,
    summary: String,
    kind: String,
    switch: String,
}

/// Text with every run of whitespace made one space, as a wrapped line reads.
fn spaced(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The value of a `- key: value.` line, without its full stop.
fn value<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    line.strip_prefix("- ")?
        .strip_prefix(key)?
        .strip_prefix(": ")
        .map(|value| value.trim_end_matches('.'))
}

/// The spec's entries: each `### Name` heading that states a `- kind:`, with its `- summary:`
/// (and the lines that carry it on) and its `- switch:`.
fn spec_entries() -> Vec<SpecEntry> {
    let mut entries: Vec<SpecEntry> = Vec::new();
    let mut in_summary = false;
    for line in SPEC.lines() {
        if let Some(name) = line.strip_prefix("### ") {
            entries.push(SpecEntry {
                name: name.trim().to_owned(),
                summary: String::new(),
                kind: String::new(),
                switch: String::new(),
            });
        }
        let Some(entry) = entries.last_mut() else {
            continue;
        };
        if in_summary && let Some(more) = line.strip_prefix(CARRIES_ON) {
            entry.summary = spaced(&format!("{} {more}", entry.summary));
            continue;
        }
        in_summary = false;
        if let Some(summary) = line.strip_prefix("- summary: ") {
            entry.summary = spaced(summary);
            in_summary = true;
        }
        if let Some(kind) = value(line, "kind") {
            kind.clone_into(&mut entry.kind);
        }
        if let Some(switch) = value(line, "switch") {
            switch.clone_into(&mut entry.switch);
        }
    }
    entries.retain(|entry| !entry.kind.is_empty());
    entries
}

/// A kind as the spec writes it.
const fn kind_name(kind: &Kind) -> &'static str {
    match kind {
        Kind::Mechanic => "mechanic",
        Kind::Quirk => "quirk",
        Kind::Timing => "timing",
        Kind::Output => "output",
        Kind::Encoding => "encoding",
    }
}

/// A switch as the spec writes it: `none` for an entry that is always on.
const fn switch_name(switch: &Switch) -> &'static str {
    match switch {
        Switch::Fixed => "none",
        Switch::Toggle => "toggle",
        Switch::Parameter => "parameter",
    }
}

#[test]
fn the_catalogue_matches_the_spec_one_for_one() {
    let entries = spec_entries();
    let names: Vec<&str> = entries.iter().map(|entry| entry.name.as_str()).collect();
    let catalogued: Vec<String> = CATALOGUE
        .iter()
        .map(|behaviour| format!("{:?}", behaviour.id))
        .collect();
    assert_eq!(catalogued, names, "the catalogue lists the spec's entries");
    for (behaviour, entry) in CATALOGUE.iter().zip(&entries) {
        assert_eq!(
            spaced(behaviour.summary),
            entry.summary,
            "{}: summary",
            entry.name
        );
        assert_eq!(
            kind_name(&behaviour.kind),
            entry.kind,
            "{}: kind",
            entry.name
        );
        let switch = entry.switch.split_whitespace().next().unwrap_or_default();
        assert_eq!(
            switch_name(&behaviour.switch),
            switch,
            "{}: switch",
            entry.name
        );
    }
}
