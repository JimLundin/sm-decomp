//! Borrowed source operations belong to the input adapters that consume them.

use super::index::{Borrowed, Case, InputProvider, Start};
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
enum Selection {
    #[default]
    All,
    NativeOffer,
    ConstructedOffer,
    Ribs,
    OrdinaryRoom,
}

impl Selection {
    fn applies(&self, case: &Case) -> bool {
        match self {
            Self::All => true,
            Self::NativeOffer => matches!(case.start, Start::NativeSenderOffer { .. }),
            Self::ConstructedOffer => matches!(case.start, Start::ConstructedSenderOffer { .. }),
            Self::Ribs => super::played::ridley_ribs::is_case(case),
            Self::OrdinaryRoom => !super::played::ridley_ribs::is_case(case),
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged, deny_unknown_fields)]
enum Provider {
    Fixed(String),
    Selected { packet: Packet },
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Packet {
    SenderInputs,
    ConnectedDirectory,
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

/// Keep the expanded source visible in replay map, including for published format-1 indices.
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

impl Operation {
    fn describes(&self, input: &Borrowed) -> bool {
        self.dependency == input.dependency
            && self.phase == input.phase
            && self.fields.len() == input.fields.len()
            && self.fields.iter().all(|field| input.fields.contains(field))
    }

    fn admits(&self, case: &Case, input: &Borrowed) -> bool {
        self.describes(input)
            && match &self.provider {
                Provider::Fixed(provider) => *provider == input.provider,
                Provider::Selected {
                    packet: Packet::SenderInputs,
                } => case
                    .sender_packet()
                    .is_ok_and(|packet| packet.input_path() == input.provider),
                Provider::Selected {
                    packet: Packet::ConnectedDirectory,
                } => case.connected_packet().is_some_and(|packet| {
                    input.provider.strip_prefix("authenticated-") == Some(packet.directory.as_str())
                }),
            }
    }
}

pub(super) fn valid(case: &Case) -> bool {
    let Some(operations) = OPERATIONS.as_ref() else {
        return false;
    };
    let mut supported = operations
        .values()
        .filter(|op| op.readers.contains(&case.inputs))
        .peekable();
    if supported.peek().is_none() {
        return !case
            .borrowed
            .iter()
            .any(|input| operations.values().any(|op| op.describes(input)));
    }
    let required: Vec<_> = supported.filter(|op| op.selection.applies(case)).collect();
    case.borrowed.len() == required.len()
        && required
            .iter()
            .all(|op| case.borrowed.iter().any(|input| op.admits(case, input)))
}
