//! Current facing and all selected motion-reset bindings at the independent source edge.

use super::frame_origin::profiles::{Effect, ProfileName, Templates};
use data::pose::{MovementType, PoseFacing};
use data::timing::Template;
use game::production::frame::motion_reset_contract::CurrentFacing;
use serde::Deserialize;

/// Incompatible current source, selected terminal metadata or whole profile.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// Current independent source image or metadata is incomplete.
    Source,
    /// Current facing/action is outside the admitted ordinary running meaning.
    Facing,
    /// Selected bindings are incomplete or expose native terminal values.
    Selected,
    /// Whole operation scope is missing or contains a device effect.
    Profile,
}

#[derive(Deserialize)]
struct FacingContract {
    format: String,
    source_coordinates_role: String,
    selected_model_terminals_exported: bool,
    named_imported_content_guard_required: bool,
    current_field: Vec<CurrentField>,
    borrowed_terminal_field: Vec<toml::Value>,
    selected_output_binding: Vec<Selected>,
}

#[derive(Deserialize)]
struct CurrentField {
    owner: String,
    section: String,
    name: String,
    private_wram_offset: usize,
    quantity_width: usize,
    required_current_encoding: u32,
    ownership: String,
    revalidate_before_consumption: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Selected {
    owner: String,
    section: String,
    name: String,
    private_wram_offset: usize,
    quantity_width: usize,
    source_writer_index: usize,
    source_comment: String,
    ownership: String,
    source_terminal_not_runtime_input: bool,
}

const SELECTED: [(&str, usize, usize); 11] = [
    ("SAMUS_LEFT_TRAVEL", 0x0daa, 1513),
    ("SAMUS_LEFT_TRAVEL_FRACTION", 0x0dac, 1514),
    ("SAMUS_RIGHT_TRAVEL", 0x0dae, 1515),
    ("SAMUS_RIGHT_TRAVEL_FRACTION", 0x0db0, 1516),
    ("SAMUS_UP_TRAVEL", 0x0db2, 1517),
    ("SAMUS_UP_TRAVEL_FRACTION", 0x0db4, 1518),
    ("SAMUS_DOWN_TRAVEL", 0x0db6, 1519),
    ("SAMUS_DOWN_TRAVEL_FRACTION", 0x0db8, 1520),
    ("NewPoseAnimationFrame", 0x0a9a, 1521),
    ("PENDING_FIRING_AIM", 0x0b5e, 1522),
    ("PreviousPoseFacingAndPreparedAction", 0x0a10, 1528),
];

fn selected(fields: &[Selected]) -> Result<(), Fault> {
    if fields.len() != 11 {
        return Err(Fault::Selected);
    }
    for (name, offset, writer) in SELECTED {
        let mut matches = fields.iter().filter(|field| field.name == name);
        let field = matches.next().ok_or(Fault::Selected)?;
        if matches.next().is_some()
            || field.owner != "MotionReset"
            || field.section != "MotionResetTerminal"
            || field.private_wram_offset != offset
            || field.quantity_width != 2
            || field.source_writer_index != writer
            || !field.source_comment.is_empty()
            || field.ownership != "ActualSelectedModelOrParentOwnedResult"
            || !field.source_terminal_not_runtime_input
        {
            return Err(Fault::Selected);
        }
    }
    Ok(())
}

/// Validate all eleven selected bindings and decode only the actual current facing/action.
/// The loader pins accepted source and origin; native selected terminals are never read.
/// # Errors
/// Rejects incomplete/aliased metadata, selected terminal encodings or invalid current facing/action.
pub fn contract(bytes: &[u8], text: &str, labels: &str) -> Result<CurrentFacing, Fault> {
    if bytes.len() != 131_072 {
        return Err(Fault::Source);
    }
    let source: FacingContract = toml::from_str(text).map_err(|_| Fault::Source)?;
    if source.format != "player-entry-named-current-terminal-bindings-v2"
        || source.source_coordinates_role != "ProvenanceOnly"
        || source.selected_model_terminals_exported
        || !source.named_imported_content_guard_required
        || source.current_field.len() != 17
        || source.borrowed_terminal_field.len() != 12
    {
        return Err(Fault::Source);
    }
    selected(&source.selected_output_binding)?;
    super::frame_player_preparation_receipts::current_pose(bytes, text)
        .map_err(|_| Fault::Source)?;
    super::frame_player_preparation_receipts::current_suit(bytes, text)
        .map_err(|_| Fault::Source)?;
    super::frame_player_preparation_receipts::current_environment(bytes, text, labels)
        .map_err(|_| Fault::Source)?;
    let mut fields = source
        .current_field
        .iter()
        .filter(|field| field.owner == "MotionReset");
    let field = fields.next().ok_or(Fault::Source)?;
    if fields.next().is_some()
        || field.name != "SAMUS_POSE_X_DIR"
        || field.section != "MotionResetOrigin"
        || field.private_wram_offset != 0x0a1e
        || field.quantity_width != 2
        || field.ownership != "ActualCurrentOwnerAtThisOpportunity"
        || !field.revalidate_before_consumption
    {
        return Err(Fault::Source);
    }
    let bytes = bytes.get(0x0a1e..0x0a20).ok_or(Fault::Source)?;
    let bytes: [u8; 2] = bytes.try_into().map_err(|_| Fault::Source)?;
    if u32::from(u16::from_le_bytes(bytes)) != field.required_current_encoding {
        return Err(Fault::Source);
    }
    current_facing(bytes)
}

fn current_facing([facing, action]: [u8; 2]) -> Result<CurrentFacing, Fault> {
    if MovementType::try_from(action).map_err(|_| Fault::Facing)? != MovementType::Running {
        return Err(Fault::Facing);
    }
    match PoseFacing::from(facing) {
        PoseFacing::Left => Ok(CurrentFacing::Left),
        PoseFacing::Right => Ok(CurrentFacing::Right),
        PoseFacing::Forward | PoseFacing::Unknown(_) => Err(Fault::Facing),
    }
}

/// Import every exposure of the sole qualified motion-reset owner, retaining polls.
/// # Errors
/// Rejects missing/shortened/overlapping/wrong owner scope or any device effect.
pub fn whole_template(templates: &Templates) -> Result<Template, Fault> {
    let mut parts = templates
        .portions
        .iter()
        .filter(|part| part.semantic_owner == "S2.MotionReset");
    let part = parts.next().ok_or(Fault::Profile)?;
    if parts.next().is_some()
        || part.profile != ProfileName::PreWait
        || part.receipt_contract != "S2.MotionReset"
        || part.first_exposure != 5163
        || part.end_exposure != 5275
        || !part.completed_producer_receipts.is_empty()
        || templates
            .portions
            .iter()
            .filter(|part| {
                part.profile == ProfileName::PreWait
                    && part.first_exposure < 5275
                    && part.end_exposure > 5163
            })
            .count()
            != 1
    {
        return Err(Fault::Profile);
    }
    let mut profiles = templates
        .profiles
        .iter()
        .filter(|profile| profile.name == ProfileName::PreWait);
    let profile = profiles.next().ok_or(Fault::Profile)?;
    if profiles.next().is_some() {
        return Err(Fault::Profile);
    }
    let whole = profile.slice(5163, 5275).map_err(|_| Fault::Profile)?;
    if whole.runs.iter().any(|run| run.effect != Effect::None) {
        return Err(Fault::Profile);
    }
    whole.template().map_err(|_| Fault::Profile)
}

#[cfg(test)]
#[path = "frame_motion_reset_contract/tests.rs"]
mod tests;
