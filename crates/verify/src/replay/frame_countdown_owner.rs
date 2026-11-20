//! Independent origin qualification and whole countdown bill at the verification edge.

use super::frame_origin::profiles::{Effect, ProfileName, Templates};
use game::production::frame::countdown_caller::Physical;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Projection {
    format: String,
    authority: String,
    countdown: String,
    private_wram_offset: usize,
    private_dormant_encoding: u16,
    quantity_width: usize,
    read_indices: Vec<usize>,
    executing_read_bytes: usize,
    writer_count: usize,
    origin_equal_at_both_uses: bool,
    origin_sha256: String,
}

/// Invalid source qualification, incomplete origin or effectful physical ownership.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// Required current origin and exact source coverage were not admitted.
    Source,
}

/// Admit only authenticated independent complete countdown input and whole Timer work.
/// # Errors
/// Missing, stale, shortened, overlapping or device-effectful source ownership.
pub fn physical(bytes: &[u8], text: &str, templates: &Templates) -> Result<Physical, Fault> {
    let projection: Projection = toml::from_str(text).map_err(|_| Fault::Source)?;
    if projection.format != "countdown-origin-qualification-v1"
        || projection.authority != "IndependentCompleteFrameOriginAndActualTimerStatusReads"
        || projection.countdown != "Dormant"
        || projection.quantity_width != 2
        || projection.private_dormant_encoding != 0
        || projection.read_indices != [3568, 3580]
        || projection.executing_read_bytes != 4
        || projection.writer_count != 0
        || !projection.origin_equal_at_both_uses
        || crate::lock::Sha256::try_from(projection.origin_sha256.as_str())
            .map_err(|_| Fault::Source)?
            != crate::lock::Sha256::of(bytes)
        || bytes.len() != 131_072
    {
        return Err(Fault::Source);
    }
    let countdown = super::hud::countdown_origin(bytes).map_err(|_| Fault::Source)?;
    let mut changed = bytes.to_vec();
    changed
        .get_mut(projection.private_wram_offset..projection.private_wram_offset.saturating_add(2))
        .ok_or(Fault::Source)?
        .copy_from_slice(&1_u16.to_le_bytes());
    if super::hud::countdown_origin(&changed).is_ok()
        || countdown != interface_hud::Countdown::Dormant
    {
        return Err(Fault::Source);
    }
    let mut portions = templates
        .portions
        .iter()
        .filter(|part| part.semantic_owner == "S5.Timer");
    let portion = portions.next().ok_or(Fault::Source)?;
    if portions.next().is_some()
        || portion.profile != ProfileName::PreWait
        || portion.receipt_contract != "S5.Timer"
        || portion.first_exposure != 13083
        || portion.end_exposure != 13191
        || !portion.completed_producer_receipts.is_empty()
        || templates
            .portions
            .iter()
            .filter(|part| {
                part.profile == ProfileName::PreWait
                    && part.first_exposure < 13191
                    && part.end_exposure > 13083
            })
            .count()
            != 1
    {
        return Err(Fault::Source);
    }
    let mut profiles = templates
        .profiles
        .iter()
        .filter(|profile| profile.name == ProfileName::PreWait);
    let profile = profiles.next().ok_or(Fault::Source)?;
    if profiles.next().is_some() {
        return Err(Fault::Source);
    }
    let slice = profile.slice(13083, 13191).map_err(|_| Fault::Source)?;
    if slice.runs.iter().any(|run| run.effect != Effect::None) {
        return Err(Fault::Source);
    }
    Ok(Physical {
        template: slice.template().map_err(|_| Fault::Source)?,
    })
}
