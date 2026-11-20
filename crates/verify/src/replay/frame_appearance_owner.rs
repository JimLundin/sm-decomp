//! Current appearance joins and whole-owner navigation at the verification edge.

use super::frame_origin::profiles::{ProfileName, Templates};
use game::production::frame::appearance_caller::{PhysicalContract, Requirement};

/// Incomplete current producers or an unavailable qualified owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// The independently gated current predicate projection differs.
    Source,
    /// Missing or ambiguous complete source envelope.
    Envelope,
}

pub(super) fn pre_wait_template(
    templates: &Templates,
    owner: &str,
) -> Result<data::timing::Template, Fault> {
    let mut profiles = templates
        .profiles
        .iter()
        .filter(|profile| profile.name == ProfileName::PreWait);
    let profile = profiles.next().ok_or(Fault::Envelope)?;
    if profiles.next().is_some() {
        return Err(Fault::Envelope);
    }
    let mut portions = templates
        .portions
        .iter()
        .filter(|portion| portion.semantic_owner == owner);
    let first = portions.next().ok_or(Fault::Envelope)?;
    let mut end = first.end_exposure;
    for portion in portions {
        if portion.first_exposure != end {
            return Err(Fault::Envelope);
        }
        end = portion.end_exposure;
    }
    profile
        .slice(first.first_exposure, end)
        .map_err(|_| Fault::Envelope)?
        .template()
        .map_err(|_| Fault::Envelope)
}

/// Qualified whole ordinary animation and the appearance part of shared `PoseFit`.
pub struct PhysicalOwners {
    /// Bill the whole ordinary animation once after actual-current qualification.
    pub animation: PhysicalContract,
    /// Conjoin with movement `PoseFit` qualification and bill the shared owner once.
    pub selected_clip: PhysicalContract,
}

#[derive(serde::Deserialize)]
struct Source {
    format: String,
    #[serde(flatten)]
    inputs: InputProvenance,
    projection_only_of_accepted_sources: bool,
    animation_current: toml::Table,
    pose_fit_current: toml::Table,
    physical_portion: Vec<Part>,
    operation: Vec<Operation>,
}

#[derive(serde::Deserialize)]
struct InputProvenance {
    native_terminal_values_exported: bool,
    source_callback_at_runtime: bool,
    expected_timing_or_result_selectors: bool,
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
    #[serde(flatten)]
    billing: EnvelopeBilling,
    per_local_physical_rule_invented: bool,
    family_only_admission_allowed: bool,
}

#[derive(serde::Deserialize)]
struct EnvelopeBilling {
    whole_envelope_required: bool,
    all_portions_once: bool,
}

/// Import the independently gated projection as actual-current predicates only.
/// Its mapped current step is an equality guard, never an animation initializer.
/// # Errors
/// Changed projection, invalid semantic predicates or incomplete whole envelopes.
pub fn physical_owners(templates: &Templates, text: &str) -> Result<PhysicalOwners, Fault> {
    use crate::lock::Sha256;
    if Sha256::of(text.as_bytes())
        != Sha256::try_from("0b57c1ef79d6497f2a8abe2e9aed13029f59bdd6e16dd657e06e1ee43e62f251")
            .map_err(|_| Fault::Source)?
    {
        return Err(Fault::Source);
    }
    let source: Source = toml::from_str(text).map_err(|_| Fault::Source)?;
    if source.format != "portable-selected-stage-current-predicates-v1"
        || source.inputs.native_terminal_values_exported
        || source.inputs.source_callback_at_runtime
        || source.inputs.expected_timing_or_result_selectors
        || !source.projection_only_of_accepted_sources
    {
        return Err(Fault::Source);
    }
    let step = animation_step(&source.animation_current)?;
    validate_pose_fit(&source.pose_fit_current)?;
    Ok(PhysicalOwners {
        animation: PhysicalContract::new(
            Requirement::OrdinaryAnimation {
                step: samus_appearance::Step(step),
            },
            whole(templates, &source, "S5.Animation")?,
        ),
        selected_clip: PhysicalContract::new(
            Requirement::RunningInitialization,
            whole(templates, &source, "S3.PoseFit")?,
        ),
    })
}

fn animation_step(animation: &toml::Table) -> Result<u8, Fault> {
    require_strings(
        animation,
        &[
            ("figure", "Running"),
            ("current", "Show"),
            ("next", "Show"),
            ("time_left", "Last"),
            ("dash", "On"),
            ("booster", "Absent"),
            ("slowdown", "None"),
            ("retained_physics", "Air"),
            ("current_effect", "NoLiquid"),
            ("stage", "Stage0"),
            (
                "current_step_role",
                "CheckedClipMappedCurrentStepEqualityOnly",
            ),
            (
                "playing_result_recipe",
                "ActualCurrentStepNextAndActualCheckedRunPaceHold",
            ),
        ],
    )?;
    require_flags(
        animation,
        &[
            ("content_verified", true),
            ("fresh", true),
            ("step_valid", true),
            ("pace_valid", true),
            ("current_step_is_runtime_initializer", false),
            ("selected_clip_initialization_included", false),
            ("native_playhead_terminal_is_runtime_input", false),
        ],
    )?;
    let sequence = animation
        .get("complete_sequence")
        .and_then(toml::Value::as_array)
        .ok_or(Fault::Source)?;
    let expected = [
        "DashBegan",
        "StandingRestart",
        "WallJumpReady",
        "FastFall",
        "RisingHold",
        "ClipPlaying",
    ];
    if !sequence
        .iter()
        .map(toml::Value::as_str)
        .eq(expected.map(Some))
    {
        return Err(Fault::Source);
    }
    let step = animation
        .get("current_step")
        .and_then(toml::Value::as_integer)
        .and_then(|step| u8::try_from(step).ok())
        .ok_or(Fault::Source)?;
    Ok(step)
}

fn validate_pose_fit(pose: &toml::Table) -> Result<(), Fault> {
    require_strings(
        pose,
        &[
            ("fit", "DistinctNonFrontNoGrowth"),
            ("initialization", "RunningToRunning"),
            ("retained", "ActualSelected"),
            ("playhead", "Keep"),
            ("jump", "None"),
            ("effects", "None"),
        ],
    )?;
    require_flags(
        pose,
        &[
            ("fresh", true),
            ("old_body_height_at_least_actual_selected_body", true),
            ("actual_checked_current_and_selected_bodies_required", true),
            (
                "actual_selected_figure_and_completed_playing_state_required",
                true,
            ),
            ("clip_start_transient_reset_is_not_playhead_state", true),
            (
                "native_prospective_pose_or_playhead_terminal_is_runtime_input",
                false,
            ),
        ],
    )?;
    Ok(())
}

fn require_strings(table: &toml::Table, fields: &[(&str, &str)]) -> Result<(), Fault> {
    if fields
        .iter()
        .any(|(key, expected)| table.get(*key).and_then(toml::Value::as_str) != Some(*expected))
    {
        return Err(Fault::Source);
    }
    Ok(())
}

fn require_flags(table: &toml::Table, fields: &[(&str, bool)]) -> Result<(), Fault> {
    if fields
        .iter()
        .any(|(key, expected)| table.get(*key).and_then(toml::Value::as_bool) != Some(*expected))
    {
        return Err(Fault::Source);
    }
    Ok(())
}

fn whole(
    templates: &Templates,
    source: &Source,
    owner: &str,
) -> Result<data::timing::Template, Fault> {
    let mut operations = source
        .operation
        .iter()
        .filter(|operation| operation.owner == owner);
    let operation = operations.next().ok_or(Fault::Envelope)?;
    if operations.next().is_some()
        || operation.profile != ProfileName::PreWait
        || !operation.billing.whole_envelope_required
        || !operation.billing.all_portions_once
        || operation.per_local_physical_rule_invented
        || operation.family_only_admission_allowed
    {
        return Err(Fault::Envelope);
    }
    let required = source
        .physical_portion
        .iter()
        .filter(|part| part.semantic_owner == owner)
        .collect::<Vec<_>>();
    let actual = templates
        .portions
        .iter()
        .filter(|part| part.semantic_owner == owner)
        .collect::<Vec<_>>();
    if required.is_empty() || required.len() != actual.len() {
        return Err(Fault::Envelope);
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
            return Err(Fault::Envelope);
        }
        end = actual.end_exposure;
    }
    if end != operation.end_exposure_exclusive {
        return Err(Fault::Envelope);
    }
    let mut profiles = templates
        .profiles
        .iter()
        .filter(|profile| profile.name == operation.profile);
    let profile = profiles.next().ok_or(Fault::Envelope)?;
    if profiles.next().is_some() {
        return Err(Fault::Envelope);
    }
    profile
        .slice(operation.first_exposure, end)
        .map_err(|_| Fault::Envelope)?
        .template()
        .map_err(|_| Fault::Envelope)
}
