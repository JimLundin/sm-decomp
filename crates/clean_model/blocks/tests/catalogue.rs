//! The catalogue checked against the spec (`docs/behaviours/blocks.md`): one entry per spec entry,
//! in the spec's order, each with the spec's kind and switch.

#![cfg(test)]

use clean_model_blocks::CATALOGUE;
use clean_types::{Kind, Switch};

/// The spec, whose entries the catalogue lists.
const SPEC: &str = include_str!("../../../../docs/behaviours/blocks.md");

/// A spec entry as the spec states it: its name, its kind and its switch.
struct SpecEntry {
    name: String,
    kind: String,
    switch: String,
}

/// The value of a `- key: value.` line, without its full stop.
fn value<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    line.strip_prefix("- ")?
        .strip_prefix(key)?
        .strip_prefix(": ")
        .map(|value| value.trim_end_matches('.'))
}

/// The spec's entries: each `### Name` heading that states a `- kind:`, with its `- switch:`.
fn spec_entries() -> Vec<SpecEntry> {
    let mut entries: Vec<SpecEntry> = Vec::new();
    for line in SPEC.lines() {
        if let Some(name) = line.strip_prefix("### ") {
            entries.push(SpecEntry {
                name: name.trim().to_owned(),
                kind: String::new(),
                switch: String::new(),
            });
        }
        let Some(entry) = entries.last_mut() else {
            continue;
        };
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
