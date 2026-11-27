//! Borrowed source operations belong to the input adapters that consume them.

pub(super) mod escape;

use super::index::{Borrowed, Case, InputProvider, Responsibility, Start};
use super::intro_sources::{Join, Room};
use super::save_files::SaveSource;
use game::production::played::intro_rooms::Start as Scene;
use serde::{Deserialize, Deserializer};
use std::{collections::BTreeMap, sync::LazyLock};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Operation {
    readers: Vec<InputProvider>,
    #[serde(default)]
    selection: Selection,
    dependency: String,
    provider: Provider,
    fields: Vec<String>,
    phase: String,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum Selection {
    #[default]
    All,
    Movement,
    Escape(Vec<escape::Origin>),
    NativeOffer,
    ConstructedOffer,
    IntroRoom(#[serde(with = "Room")] Scene),
    Save(SaveSource),
}

impl Selection {
    pub(super) fn applies(&self, case: &Case, origin: Option<escape::Origin>) -> bool {
        match self {
            Self::All => true,
            Self::Movement => case.responsibilities == [Responsibility::Movement],
            Self::Escape(origins) => origin.is_some_and(|actual| origins.contains(&actual)),
            Self::NativeOffer => matches!(case.start, Start::NativeSenderOffer { .. }),
            Self::ConstructedOffer => matches!(case.start, Start::ConstructedSenderOffer { .. }),
            Self::IntroRoom(scene) => matches!(
                &case.intro_join, Some(Join::Room { scene: selected, .. }) if selected == scene
            ),
            Self::Save(source) => case
                .save_steps
                .as_ref()
                .is_some_and(|steps| steps.uses(*source)),
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged, deny_unknown_fields)]
enum Provider {
    Fixed(String),
    Selected {
        packet: Packet,
        #[serde(default)]
        prefix: String,
        #[serde(default)]
        suffix: String,
    },
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Packet {
    SenderInputs,
    ConnectedDirectory,
    FixtureInputs,
    FixtureDirectory,
}

impl Packet {
    fn selected<'a>(&self, case: &'a Case) -> Option<&'a str> {
        match self {
            Self::SenderInputs => Some(case.sender_packet().ok()?.input_path()),
            Self::ConnectedDirectory => Some(&case.connected_packet.as_ref()?.directory),
            Self::FixtureInputs => Some(&case.fixture_pair().ok()?.inputs.path),
            Self::FixtureDirectory => {
                Some(case.fixture_pair().ok()?.inputs.path.split_once('/')?.0)
            }
        }
    }
}

static OPERATIONS: LazyLock<Option<BTreeMap<String, Operation>>> = LazyLock::new(|| {
    toml::from_str(include_str!(
        "../../../../scenarios/replay/borrowed-inputs.toml"
    ))
    .ok()
});

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Reference {
    operation: String,
    provider: Option<String>,
    #[serde(default)]
    claim: String,
}

/// Expand the selected adapter operation into its public source declaration.
pub(super) fn resolve(
    operation: &str,
    provider: Option<String>,
    claim: String,
) -> Option<Borrowed> {
    let source = OPERATIONS.as_ref()?.get(operation)?;
    let provider = match (&source.provider, provider) {
        (Provider::Fixed(fixed), None) => fixed.clone(),
        (Provider::Selected { .. }, Some(packet)) => packet,
        _ => return None,
    };
    Some(Borrowed {
        dependency: source.dependency.clone(),
        fields: source.fields.clone(),
        phase: source.phase.clone(),
        provider,
        claim,
    })
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Declaration {
    Operation(Reference),
    Published(Borrowed),
}

/// Keep the expanded source visible in replay map.
pub(super) fn read<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<Borrowed>, D::Error> {
    Vec::<Declaration>::deserialize(deserializer)?
        .into_iter()
        .map(|entry| match entry {
            Declaration::Published(input) => Ok(input),
            Declaration::Operation(binding) => {
                resolve(&binding.operation, binding.provider, binding.claim)
                    .ok_or_else(|| serde::de::Error::custom("invalid borrowed input operation"))
            }
        })
        .collect()
}

impl Borrowed {
    pub(super) fn describes(&self, dependency: &str, fields: &[String], phase: &str) -> bool {
        self.dependency == dependency
            && self.phase == phase
            && fields.len() == self.fields.len()
            && fields.iter().all(|field| self.fields.contains(field))
    }
}

impl Operation {
    fn describes(&self, input: &Borrowed) -> bool {
        input.describes(&self.dependency, &self.fields, &self.phase)
    }

    fn admits(&self, case: &Case, input: &Borrowed) -> bool {
        self.describes(input)
            && match &self.provider {
                Provider::Fixed(provider) => *provider == input.provider,
                Provider::Selected {
                    packet,
                    prefix,
                    suffix,
                } => input
                    .provider
                    .strip_prefix(prefix)
                    .and_then(|path| path.strip_suffix(suffix))
                    .is_some_and(|path| packet.selected(case) == Some(path)),
            }
    }
}

pub(super) fn valid(case: &Case) -> bool {
    let origins = escape::Origin::candidates(case);
    if origins.is_empty() {
        valid_for(case, None)
    } else {
        origins.iter().any(|&origin| valid_for(case, Some(origin)))
    }
}

fn valid_for(case: &Case, origin: Option<escape::Origin>) -> bool {
    let Some(operations) = OPERATIONS.as_ref() else {
        return false;
    };
    let required: Vec<_> = operations
        .values()
        .filter(|op| op.readers.contains(&case.inputs) && op.selection.applies(case, origin))
        .collect();
    case.borrowed.len() == required.len()
        && required
            .iter()
            .all(|op| case.borrowed.iter().any(|input| op.admits(case, input)))
}

#[cfg(test)]
mod tests {
    use super::super::index::{Index, InputProvider};

    #[test]
    fn qualifications_can_change_without_supplying_an_input() {
        let index = Index::parse(include_str!("../../../../scenarios/replay/index.toml")).unwrap();
        let mut case = index
            .cases
            .into_iter()
            .find(|case| case.inputs == InputProvider::NativeSearchInputs)
            .unwrap();
        case.qualifications.clear();
        case.borrowed[0].claim.clear();
        assert!(case.validate().is_ok());
        case.qualifications.push("moved".to_owned());
        case.borrowed.pop();
        assert!(case.validate().is_err());
    }
}
