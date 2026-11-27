use super::{Case, OPERATIONS, Operation, resolve};
use crate::replay::index::Qualification;
use crate::replay::intro_sources::{Join, RecordCut};
use serde::Deserialize;
use std::sync::LazyLock;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Published {
    from: Operation,
    operations: Vec<String>,
    handoff: Option<RecordCut>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Replacements {
    source: Vec<Published>,
}

static PUBLISHED: LazyLock<Option<Replacements>> = LazyLock::new(|| {
    toml::from_str(include_str!(
        "../../../../../scenarios/replay/published-intro-inputs.toml"
    ))
    .ok()
});

pub(in crate::replay) fn normalize(case: &mut Case) -> Result<(), &'static str> {
    let published = PUBLISHED.as_ref().ok_or("published intro sources")?;
    let sources: Vec<_> = published
        .source
        .iter()
        .filter(|entry| {
            entry.from.readers.contains(&case.inputs) && entry.from.selection.applies(case, None)
        })
        .collect();
    let count = |source: &Published| {
        case.borrowed
            .iter()
            .filter(|b| source.from.admits(case, b))
            .count()
    };
    if !sources.iter().any(|source| count(source) > 0) {
        return Ok(());
    }
    if sources.iter().any(|source| count(source) != 1) {
        return Err("incomplete or duplicate published intro sources");
    }
    let operations = OPERATIONS.as_ref().ok_or("intro source operations")?;
    let mut inputs = Vec::new();
    let mut provenance = Vec::new();
    for input in &case.borrowed {
        if let Some(source) = sources.iter().find(|s| s.from.admits(case, input)) {
            if source.handoff.as_ref().is_some_and(|pin| {
                !matches!(
                    &case.intro_join, Some(Join::Flashback { handoff, .. }) if handoff == pin
                )
            }) {
                return Err("published flashback handoff");
            }
            provenance.push(Qualification::Published(input.clone()));
            for name in &source.operations {
                let operation = operations
                    .get(name)
                    .ok_or("unknown intro source operation")?;
                if operation.selection.applies(case, None) {
                    inputs.push(
                        resolve(name, None, String::new())
                            .ok_or("invalid published intro source")?,
                    );
                }
            }
        } else {
            inputs.push(input.clone());
        }
    }
    case.borrowed = inputs;
    case.qualifications.extend(provenance);
    Ok(())
}
