//! Complete compound support envelope and both actual arithmetic tasks at the edge.

use super::frame_origin::profiles::{Effect, Profile, ProfileName, Templates};
use game::production::frame::movement_pair_contract::SourceAdmission;

/// An omitted, reordered or aliased support physical owner or arithmetic task.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// The independently admitted current source or room provenance differs.
    Source,
    /// The whole compound owner, ordered portions or live arithmetic bindings differ.
    Profile,
}

#[derive(serde::Deserialize)]
struct Source {
    physical_owner: Vec<MovementEnvelope>,
    constituent_portion: Vec<Part>,
    arithmetic_binding: Vec<Binding>,
    support_composition: Composition,
}
#[derive(serde::Deserialize)]
struct MovementEnvelope {
    owner: String,
    profile: ProfileName,
    first_exposure: u64,
    end_exposure: u64,
    source_matches_frozen_profile: bool,
}
#[derive(serde::Deserialize)]
struct Part {
    owner: String,
    profile: ProfileName,
    first_exposure: u64,
    end_exposure: u64,
    semantic_owner: String,
    receipt_contract: String,
    completed_producer_receipts: Vec<String>,
}
#[derive(serde::Deserialize)]
struct Binding {
    owner: String,
    kind: String,
    profile: ProfileName,
    exposure: u64,
    device: String,
    producer: Option<String>,
    task_ordinal: u32,
    actual_request_quantity_or_actual_hardware_sample_required: bool,
}
#[derive(serde::Deserialize)]
struct Composition {
    named_floor_end_is_actual_completion: bool,
    irq_is_separate_hardware_owner: bool,
    both_actual_support_answers_required_before_pair_complete: bool,
}

/// Import actual current geometry and the single complete support contract.
/// # Errors
/// Current source provenance, complete physical envelope or ordered tasks differ.
pub fn contract(
    origin: &[u8],
    native: &super::movement::Content,
    templates: &Templates,
    text: &str,
) -> Result<
    (
        game::production::frame::movement_caller::Room,
        game::production::frame::movement_support_contract::PhysicalContract,
    ),
    Fault,
> {
    let (admission, template) = envelope(origin, native, templates, text)?;
    Ok((
        admission.room.clone(),
        game::production::frame::movement_support_contract::PhysicalContract::new(
            admission, template,
        ),
    ))
}

/// Import one whole support routine, with shared admission/return and both tasks once.
/// The source query rows and products never initialize a model or choose this envelope.
/// # Errors
/// Different current source, missing owner portions, an early floor completion or aliases.
pub fn envelope(
    origin: &[u8],
    native: &super::movement::Content,
    templates: &Templates,
    text: &str,
) -> Result<(SourceAdmission, data::timing::Template), Fault> {
    let admission =
        super::frame_movement_pair_owner::source_admission(origin, native, templates, text)
            .map_err(|_| Fault::Source)?;
    let source: Source = toml::from_str(text).map_err(|_| Fault::Source)?;
    if source
        .support_composition
        .named_floor_end_is_actual_completion
        || !source.support_composition.irq_is_separate_hardware_owner
        || !source
            .support_composition
            .both_actual_support_answers_required_before_pair_complete
    {
        return Err(Fault::Source);
    }
    let mut owners = source
        .physical_owner
        .iter()
        .filter(|owner| owner.owner == "SupportPair");
    let owner = owners.next().ok_or(Fault::Profile)?;
    if owners.next().is_some()
        || !owner.source_matches_frozen_profile
        || owner.profile != ProfileName::PreWait
    {
        return Err(Fault::Profile);
    }
    let mut end = owner.first_exposure;
    let parts = source
        .constituent_portion
        .iter()
        .filter(|part| part.owner == "SupportPair")
        .collect::<Vec<_>>();
    if parts.is_empty() {
        return Err(Fault::Profile);
    }
    for part in parts {
        let mut matching = templates.portions.iter().filter(|actual| {
            actual.profile == part.profile && actual.first_exposure == part.first_exposure
        });
        let actual = matching.next().ok_or(Fault::Profile)?;
        if matching.next().is_some()
            || part.profile != owner.profile
            || part.first_exposure != end
            || part.end_exposure <= end
            || actual.end_exposure != part.end_exposure
            || actual.semantic_owner != part.semantic_owner
            || actual.receipt_contract != part.receipt_contract
            || actual.completed_producer_receipts != part.completed_producer_receipts
        {
            return Err(Fault::Profile);
        }
        end = part.end_exposure;
    }
    if end != owner.end_exposure {
        return Err(Fault::Profile);
    }
    let mut profiles = templates
        .profiles
        .iter()
        .filter(|profile| profile.name == owner.profile);
    let profile = profiles.next().ok_or(Fault::Profile)?;
    if profiles.next().is_some() {
        return Err(Fault::Profile);
    }
    let whole = profile
        .slice(owner.first_exposure, end)
        .map_err(|_| Fault::Profile)?;
    check_arithmetic(&whole, owner, &source)?;
    Ok((admission, whole.template().map_err(|_| Fault::Profile)?))
}

fn check_arithmetic(
    whole: &Profile,
    owner: &MovementEnvelope,
    source: &Source,
) -> Result<(), Fault> {
    let expected = source
        .arithmetic_binding
        .iter()
        .filter(|binding| binding.owner == "SupportPair")
        .collect::<Vec<_>>();
    if expected.len() != 10 {
        return Err(Fault::Profile);
    }
    let mut expected = expected.into_iter().enumerate();
    let mut exposure = owner.first_exposure;
    for run in &whole.runs {
        for _ in 0..run.count {
            if !matches!(run.effect, Effect::None) {
                let (ordinal, binding) = expected.next().ok_or(Fault::Profile)?;
                let (task_ordinal, name) = if ordinal < 5 {
                    (6, "FloorSupport")
                } else {
                    (7, "CeilingSupport")
                };
                if binding.profile != owner.profile
                    || binding.exposure != exposure
                    || binding.task_ordinal != task_ordinal
                    || !binding.actual_request_quantity_or_actual_hardware_sample_required
                {
                    return Err(Fault::Profile);
                }
                let (device, task) = match &run.effect {
                    Effect::Publication(publication)
                        if binding.kind == "publication"
                            && binding.producer.as_ref() == Some(&publication.producer) =>
                    {
                        (publication.device.as_str(), publication.arithmetic.as_ref())
                    }
                    Effect::Sample(data::timing::Sample::ProductLow, task)
                        if binding.kind == "sample" =>
                    {
                        ("ProductLow", task.as_ref())
                    }
                    Effect::Sample(data::timing::Sample::ProductHigh, task)
                        if binding.kind == "sample" =>
                    {
                        ("ProductHigh", task.as_ref())
                    }
                    Effect::None | Effect::Publication(_) | Effect::Sample(_, _) => {
                        return Err(Fault::Profile);
                    }
                };
                let task = task.ok_or(Fault::Profile)?;
                if device != binding.device
                    || task.name != name
                    || task.quantities != ["RoomWidth", "CellRow", "Zero"]
                {
                    return Err(Fault::Profile);
                }
            }
            exposure = exposure.checked_add(1).ok_or(Fault::Profile)?;
        }
    }
    if expected.next().is_some() {
        return Err(Fault::Profile);
    }
    Ok(())
}
