//! Frozen ground-following identity mapping and whole physical envelope.

use super::frame_origin::profiles::{ProfileName, Templates};
use game::production::frame::movement_ground_contract::PhysicalContract;

/// Missing frozen authority or incompatible complete physical owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// The independently accepted source bytes differ.
    Source,
    /// Whole physical portions or effects differ.
    Profile,
}

#[derive(serde::Deserialize)]
struct Source {
    physical_portion: Vec<Part>,
    operation: Vec<Operation>,
}
#[derive(serde::Deserialize)]
struct Part {
    profile: ProfileName,
    first_exposure: u64,
    end_exposure: u64,
    semantic_owner: String,
    receipt_contract: String,
    completed_producer_receipts: Vec<String>,
}
#[derive(serde::Deserialize)]
struct Operation {
    owner: String,
    profile: ProfileName,
    first_exposure: u64,
    end_exposure_exclusive: u64,
    whole_envelope_required: bool,
    all_portions_once: bool,
    per_local_physical_rule_invented: bool,
}

/// Import the exact accepted projection and its conditional identity authority.
/// Both pins are required before any complete group can be admitted.
/// # Errors
/// Different source bytes, omitted/aliased portions or physical device effects.
pub fn contract(
    templates: &Templates,
    text: &str,
    grouping: &str,
) -> Result<PhysicalContract, Fault> {
    use crate::lock::Sha256;
    for (text, pin) in [
        (
            text,
            "0b57c1ef79d6497f2a8abe2e9aed13029f59bdd6e16dd657e06e1ee43e62f251",
        ),
        (
            grouping,
            "fb562f8775c3222e8fc020136bbad8bdc2d1c9b1e8bd94ef58ec11a452542cd8",
        ),
    ] {
        if Sha256::of(text.as_bytes()) != Sha256::try_from(pin).map_err(|_| Fault::Source)? {
            return Err(Fault::Source);
        }
    }
    let source: Source = toml::from_str(text).map_err(|_| Fault::Source)?;
    let template = whole(templates, &source)?;
    if template
        .runs
        .iter()
        .any(|run| !matches!(run.effect, data::timing::TemplateEffect::None))
    {
        return Err(Fault::Profile);
    }
    Ok(PhysicalContract::new(template))
}

fn whole(templates: &Templates, source: &Source) -> Result<data::timing::Template, Fault> {
    let mut operations = source
        .operation
        .iter()
        .filter(|operation| operation.owner == "S3.GroundPull");
    let operation = operations.next().ok_or(Fault::Profile)?;
    if operations.next().is_some()
        || operation.profile != ProfileName::PreWait
        || !operation.whole_envelope_required
        || !operation.all_portions_once
        || operation.per_local_physical_rule_invented
    {
        return Err(Fault::Profile);
    }
    let required = source
        .physical_portion
        .iter()
        .filter(|part| part.semantic_owner == "S3.GroundPull")
        .collect::<Vec<_>>();
    let actual = templates
        .portions
        .iter()
        .filter(|part| part.semantic_owner == "S3.GroundPull")
        .collect::<Vec<_>>();
    if required.is_empty() || actual.len() != required.len() {
        return Err(Fault::Profile);
    }
    let mut end = operation.first_exposure;
    for (actual, required) in actual.iter().zip(required) {
        if actual.profile != operation.profile
            || actual.first_exposure != end
            || actual.end_exposure <= end
            || actual.profile != required.profile
            || actual.first_exposure != required.first_exposure
            || actual.end_exposure != required.end_exposure
            || actual.receipt_contract != required.receipt_contract
            || actual.completed_producer_receipts != required.completed_producer_receipts
        {
            return Err(Fault::Profile);
        }
        end = actual.end_exposure;
    }
    if end != operation.end_exposure_exclusive {
        return Err(Fault::Profile);
    }
    let mut profiles = templates
        .profiles
        .iter()
        .filter(|profile| profile.name == operation.profile);
    let profile = profiles.next().ok_or(Fault::Profile)?;
    if profiles.next().is_some() {
        return Err(Fault::Profile);
    }
    profile
        .slice(operation.first_exposure, end)
        .map_err(|_| Fault::Profile)?
        .template()
        .map_err(|_| Fault::Profile)
}
