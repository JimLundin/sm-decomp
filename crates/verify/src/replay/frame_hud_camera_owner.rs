//! Exact HUD producer/return source coverage at the independent verification edge.

use super::frame_origin::profiles::{Effect, ProfileName, Templates};
use game::production::frame::hud_camera_caller::RefreshPhysical;

/// Missing, changed or effectful actual full HUD source ownership.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// The complete producer, ready return or materialization boundary is incompatible.
    Scope,
    /// A no-device-effects HUD owner contains a sample or publication.
    Effects,
}

/// Import the complete HUD bill split at the independently admitted Ready boundary.
/// Coordinates prove coverage; production admits fresh actual owners and chooses the answer.
/// # Errors
/// Rejects extra, shortened, overlapping or renamed ownership and device effects.
pub fn refresh_physical(templates: &Templates) -> Result<RefreshPhysical, Fault> {
    readiness(templates)?;
    let mut parts = templates
        .portions
        .iter()
        .filter(|part| part.semantic_owner == "S5.HudMap");
    let producer = parts.next().ok_or(Fault::Scope)?;
    let tail = parts.next().ok_or(Fault::Scope)?;
    if parts.next().is_some()
        || producer.profile != ProfileName::PreWait
        || producer.receipt_contract != "S5.HudMap"
        || producer.first_exposure != 11369
        || producer.end_exposure != 13082
        || !producer.completed_producer_receipts.is_empty()
        || tail.profile != ProfileName::PreWait
        || tail.receipt_contract != "HudTilemapReady"
        || tail.first_exposure != 13082
        || tail.end_exposure != 13083
        || tail.completed_producer_receipts != ["HudTilemapReady"]
        || templates
            .portions
            .iter()
            .filter(|part| {
                part.profile == ProfileName::PreWait
                    && part.first_exposure < 13083
                    && part.end_exposure > 11369
            })
            .count()
            != 2
    {
        return Err(Fault::Scope);
    }
    let mut profiles = templates
        .profiles
        .iter()
        .filter(|profile| profile.name == ProfileName::PreWait);
    let profile = profiles.next().ok_or(Fault::Scope)?;
    if profiles.next().is_some() {
        return Err(Fault::Scope);
    }
    let slice = |first, end| {
        let part = profile.slice(first, end).map_err(|_| Fault::Scope)?;
        if part.runs.iter().any(|run| run.effect != Effect::None) {
            return Err(Fault::Effects);
        }
        part.template().map_err(|_| Fault::Scope)
    };
    Ok(RefreshPhysical {
        producer: slice(11369, 13082)?,
        return_tail: slice(13082, 13083)?,
    })
}

fn readiness(templates: &Templates) -> Result<(), Fault> {
    let mut bindings = templates
        .materializations
        .iter()
        .filter(|binding| binding.name == "HudTilemap");
    let binding = bindings.next().ok_or(Fault::Scope)?;
    if bindings.next().is_some()
        || binding.initial_seed
        || binding.opportunity != "HudTilemapReady"
        || binding.profile != Some(ProfileName::PreWait)
        || binding.after_exposure != Some(13082)
        || binding.length != 192
    {
        return Err(Fault::Scope);
    }
    Ok(())
}
