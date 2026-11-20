//! Complete early HUD physical envelope imported only for the qualified quiet owner.

use super::frame_origin::profiles::{Effect, ProfileName, Templates};
use data::timing::Template;

/// Missing whole source scope, wrong owner namespace or unexpected device effects.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// The sole complete owner portion or its profile is missing or incompatible.
    Scope,
    /// The quiet whole owner contains a device publication or sample.
    Effects,
}

/// Import every exposure of the sole qualified early controls/weapons owner.
/// Source coordinates and names prove coverage only; actual current HUD answers choose the arm.
/// # Errors
/// Rejects missing, extra, overlapping, shortened or differently named owner parts and device effects.
pub fn whole_template(templates: &Templates) -> Result<Template, Fault> {
    let mut parts = templates
        .portions
        .iter()
        .filter(|part| part.semantic_owner == "S2.HudWeapons");
    let part = parts.next().ok_or(Fault::Scope)?;
    if parts.next().is_some()
        || part.profile != ProfileName::PreWait
        || part.receipt_contract != "S2.HudWeapons"
        || part.first_exposure != 4615
        || part.end_exposure != 4905
        || !part.completed_producer_receipts.is_empty()
    {
        return Err(Fault::Scope);
    }
    if templates
        .portions
        .iter()
        .filter(|part| {
            part.profile == ProfileName::PreWait
                && part.first_exposure < 4905
                && part.end_exposure > 4615
        })
        .count()
        != 1
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
    let whole = profile.slice(4615, 4905).map_err(|_| Fault::Scope)?;
    if whole.runs.iter().any(|run| run.effect != Effect::None) {
        return Err(Fault::Effects);
    }
    whole.template().map_err(|_| Fault::Scope)
}

#[cfg(test)]
#[path = "frame_hud_controls_owner/tests.rs"]
mod tests;
