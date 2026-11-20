//! Qualify actual quiet early HUD controls before offering their whole physical profile.

use super::hud::Hud;
use super::hud_controls_inputs::{Bindings, CurrentControls};
use super::hud_stage::{self, StageFault};
use data::timing::{Profile, Template, TemplateEffect};
use interface_hud::{CancelButton, Observation, SelectButton, Selection};

/// Missing current owner, quiet controls or complete no-device-effect profile.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// Current completed input, gameplay or inventory owners are incompatible.
    Inputs(super::hud_controls_inputs::Fault),
    /// Current controls or the actual HUD answer are outside the quiet profile.
    Quiet,
    /// The actual HUD cannot choose another controls observation.
    Stage(StageFault),
    /// The whole template is invalid or contains a device effect.
    Profile,
}

/// Retain actual controls only after their current inputs and full answer qualify atomically.
/// The caller charges the entire returned operation and publishes its completion receipt once.
/// # Errors
/// Changed current owners, active item controls, repeated choice or unsupported device effects
/// leave the sole HUD owner unchanged and return no physical profile.
pub fn choose(
    hud: &mut Hud,
    bindings: &Bindings,
    current: CurrentControls<'_>,
    whole_source: &Template,
) -> Result<Profile, Fault> {
    let inputs = bindings.inputs(current).map_err(Fault::Inputs)?;
    if inputs.select != SelectButton::NotPressed
        || inputs.cancel != CancelButton::NotPressed
        || !inputs.can_switch
        || hud.state().selection != Selection::None
    {
        return Err(Fault::Quiet);
    }
    if whole_source
        .runs
        .iter()
        .any(|run| run.effect != TemplateEffect::None)
    {
        return Err(Fault::Profile);
    }
    let profile = whole_source.bind(&[]).map_err(|_| Fault::Profile)?;
    let mut candidate = hud.clone();
    hud_stage::choose_controls(&mut candidate, inputs).map_err(Fault::Stage)?;
    let Some(Observation::Chosen(answer)) = candidate.controls() else {
        return Err(Fault::Quiet);
    };
    if answer.inputs != inputs || answer.selection != Selection::None || answer.switched.is_some() {
        return Err(Fault::Quiet);
    }
    *hud = candidate;
    Ok(profile)
}

#[cfg(test)]
#[path = "hud_controls_owner/tests.rs"]
mod tests;
