//! Exact whole `SoundOffer` and Poststep envelopes from accepted finite portions.

use super::frame_origin::profiles::{ProfileName, Templates};
use game::production::frame::post_down_contract::Physical;

/// Missing, duplicated or different complete source portion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fault;

/// Import the two distinct owners, excluding the subsequent `HUDMap` call.
/// # Errors
/// Different source ownership, dependencies or complete physical boundaries.
pub fn physical(templates: &Templates) -> Result<Physical, Fault> {
    Ok(Physical {
        sound_offer: whole(templates, "S4.SoundOffer", 11200, 11327)?,
        recurring_footstep: None,
        poststep: whole(templates, "S3.Poststep", 11327, 11369)?,
    })
}

fn whole(
    templates: &Templates,
    owner: &str,
    first: u64,
    end: u64,
) -> Result<data::timing::Template, Fault> {
    let mut portions = templates
        .portions
        .iter()
        .filter(|part| part.semantic_owner == owner);
    let part = portions.next().ok_or(Fault)?;
    if portions.next().is_some()
        || part.profile != ProfileName::PreWait
        || part.first_exposure != first
        || part.end_exposure != end
        || part.receipt_contract != owner
        || !part.completed_producer_receipts.is_empty()
    {
        return Err(Fault);
    }
    let mut profiles = templates
        .profiles
        .iter()
        .filter(|profile| profile.name == ProfileName::PreWait);
    let profile = profiles.next().ok_or(Fault)?;
    if profiles.next().is_some() {
        return Err(Fault);
    }
    profile
        .slice(first, end)
        .map_err(|_| Fault)?
        .template()
        .map_err(|_| Fault)
}
