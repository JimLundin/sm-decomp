//! Qualified native origin for the three current appearance contact suppressors.

use samus_appearance::ContactSuppressor;
use serde::Deserialize;
use types::Set;

/// An incomplete source image or incompatible appearance provenance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppearanceInputFault {
    /// Complete physical WRAM is required at the independent source edge.
    Length,
    /// Required scalar memory is unavailable.
    Field,
    /// The complete source descriptor contract is absent or incompatible.
    Provenance,
}

#[derive(Deserialize)]
#[serde(transparent)]
struct Flag(bool);

#[derive(Deserialize)]
struct SourceProof {
    format: String,
    actual_appearance_playhead_boost_and_figure_are_required: Flag,
    source_pose_playhead_substitution_allowed: Flag,
    sound_queue_authority: String,
    source_queue_indices_are_model_input: Flag,
    liquid_history_air_authority: String,
    current_dry_surfaces_are_history_authority: Flag,
    current_appearance_neighbor: Vec<Descriptor>,
}

#[derive(Deserialize)]
struct Descriptor {
    name: String,
    private_wram_offset: usize,
    quantity_width: usize,
    opportunity_source_index: usize,
    private_current_encoding: u16,
    origin_equal: bool,
    preceding_writer_indices: Vec<usize>,
    authority: String,
    physical_read_at_projection_claimed: bool,
}

const NEIGHBORS: [(&str, usize); 8] = [
    ("SAMUS_HEALTH", 0x09c2),
    ("EQUIPPED_ITEMS", 0x09a2),
    ("JOYPAD1_LASTKEYS", 0x008b),
    ("JOYPAD1_NEWKEYS", 0x008f),
    ("BINDING_RUN", 0x09b6),
    ("CINEMATIC_ACTIVE", 0x1f51),
    ("CurrentBossActivity", 0x179c),
    ("CurrentSpecialColourTimer", 0x0a68),
];
const OPPORTUNITIES: [usize; 4] = [3054, 3584, 3984, 4141];

fn word(bytes: &[u8], at: usize) -> Result<u16, AppearanceInputFault> {
    let bytes = bytes
        .get(at..at.saturating_add(2))
        .ok_or(AppearanceInputFault::Field)?;
    Ok(u16::from_le_bytes(
        bytes.try_into().map_err(|_| AppearanceInputFault::Field)?,
    ))
}

fn descriptors(bytes: &[u8], fields: &[Descriptor]) -> Result<(), AppearanceInputFault> {
    if fields.len() != 32 {
        return Err(AppearanceInputFault::Provenance);
    }
    for (name, offset) in NEIGHBORS {
        for opportunity in OPPORTUNITIES {
            let mut matches = fields.iter().filter(|field| {
                field.name == name && field.opportunity_source_index == opportunity
            });
            let field = matches.next().ok_or(AppearanceInputFault::Provenance)?;
            if matches.next().is_some()
                || field.private_wram_offset != offset
                || field.quantity_width != 2
                || !field.origin_equal
                || !field.preceding_writer_indices.is_empty()
                || field.authority != "ActualCurrentNeighborOwnerRevalidatedAtOpportunity"
                || field.physical_read_at_projection_claimed
                || field.private_current_encoding != word(bytes, offset)?
            {
                return Err(AppearanceInputFault::Provenance);
            }
        }
    }
    Ok(())
}

/// Forward only actual current Contact suppressors from the accepted neighbour origin.
/// Other appearance producers are supplied at their own later opportunities.
/// # Errors
/// Missing origin, source descriptor or current neighbour field.
pub fn qualified_contact(
    bytes: &[u8],
    proof: &str,
) -> Result<Set<ContactSuppressor>, AppearanceInputFault> {
    qualify_source(bytes, proof)?;
    let mut suppressors = Set::default();
    suppressors[ContactSuppressor::Cinematic] = word(bytes, 0x1f51)? != 0;
    suppressors[ContactSuppressor::Boss] = word(bytes, 0x179c)? != 0;
    suppressors[ContactSuppressor::SpecialColours] = word(bytes, 0x0a68)? != 0;
    Ok(suppressors)
}

fn qualify_source(bytes: &[u8], proof: &str) -> Result<(), AppearanceInputFault> {
    if bytes.len() != 131_072 {
        return Err(AppearanceInputFault::Length);
    }
    let proof: SourceProof = toml::from_str(proof).map_err(|_| AppearanceInputFault::Provenance)?;
    if proof.format != "bounded-current-appearance-neighbors-v1"
        || !proof
            .actual_appearance_playhead_boost_and_figure_are_required
            .0
        || proof.source_pose_playhead_substitution_allowed.0
        || proof.sound_queue_authority != "ActualCompletedSenderOfferAndCurrentSenderState"
        || proof.source_queue_indices_are_model_input.0
        || proof.liquid_history_air_authority
            != "AcceptedIndependentS5AnimationRetainedPhysicsDecodeAt3615"
        || proof.current_dry_surfaces_are_history_authority.0
    {
        return Err(AppearanceInputFault::Provenance);
    }
    descriptors(bytes, &proof.current_appearance_neighbor)
}

#[cfg(test)]
#[path = "frame_appearance_inputs/tests.rs"]
mod tests;
