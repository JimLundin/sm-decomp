//! Actual selected preparation required before charging the whole motion-reset operation.

use super::hud::Hud;
use super::hud_controls_inputs::{Bindings, CurrentControls};
use super::hud_stage::ControlsCompletion;
use super::movement::Movement;
use super::movement_stage::{Completion, Group};
use crate::production::timing::WorkId;
use canonical::Canonical;
use data::timing::{Profile, Template, TemplateEffect};
use interface_hud::{Observation, Selection};
use samus_movement::suspended::MovementDecision;
use types::{Action, Facing, TravelDistance};

/// Current facing decoded independently at this preparation opportunity.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum CurrentFacing {
    /// Current left-facing running pose.
    Left,
    /// Current right-facing running pose.
    Right,
}

/// Completed discharge of this frame's transient clip-start preparation.
/// This receipt marker neither owns nor initializes an animation playhead.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClipStartPreparation {
    /// Transient preparation was cleared by the completed physical operation.
    Cleared,
}

/// Actual current owners and completed prerequisites for motion reset.
#[derive(Clone, Copy)]
pub struct Current<'a> {
    /// Sole current movement authority and its actual publications.
    pub movement: &'a Movement,
    /// Actual complete Inside operation at physical owner 15.
    pub inside: &'a Completion,
    /// Sole HUD owner retaining its completed observation.
    pub hud: &'a Hud,
    /// Full actual HUD controls completion at physical owner 16.
    pub controls: &'a ControlsCompletion,
    /// Independently qualified configured item controls.
    pub bindings: &'a Bindings,
    /// Current gameplay, completed `PoseInput` and actual current inventory.
    pub control_inputs: CurrentControls<'a>,
}

/// Missing actual completion, changed current facing or unsupported physical work.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// Completed owners or current physical controls are incompatible.
    Dependency,
    /// Actual movement publications are not the required fresh zero/no-aim values.
    Outputs,
    /// Current decoded facing/action differs from the actual movement pose.
    Facing,
    /// Whole physical work is invalid or contains an unexpected device effect.
    Profile,
}

/// Qualify actual selected publications and current owners before returning physical work.
/// Completion captures current pose history and the cleared transient marker separately.
/// # Errors
/// Rejects stale/incorrect completions, nonzero actual output, changed facing or device effects.
pub fn qualify(
    current: Current<'_>,
    facing: CurrentFacing,
    whole: &Template,
) -> Result<Profile, Fault> {
    if current.inside.work != WorkId(15)
        || current.controls.work != WorkId(16)
        || current.inside.receipt.group != Group::Inside
    {
        return Err(Fault::Dependency);
    }
    let inside = current.inside.receipt.decisions.0.as_slice();
    if !matches!(
        inside,
        [
            MovementDecision::InsideRequest(_),
            MovementDecision::InsideAnswer(_)
        ]
    ) {
        return Err(Fault::Dependency);
    }
    let Movement::Work(work) = current.movement else {
        return Err(Fault::Dependency);
    };
    if work.completed.0.as_slice() != inside {
        return Err(Fault::Dependency);
    }
    let Some(Observation::Completed(answer)) = current.hud.controls() else {
        return Err(Fault::Dependency);
    };
    let inputs = current
        .bindings
        .inputs(current.control_inputs)
        .map_err(|_| Fault::Dependency)?;
    if answer != &current.controls.answer.0
        || answer.inputs != inputs
        || answer.selection != Selection::None
        || answer.switched.is_some()
        || inputs.select != interface_hud::SelectButton::NotPressed
        || inputs.cancel != interface_hud::CancelButton::NotPressed
        || !inputs.can_switch
        || current.control_inputs.pose != current.movement.state().pose
    {
        return Err(Fault::Dependency);
    }
    let outputs = current.movement.outputs().ok_or(Fault::Outputs)?;
    if outputs.pending_aim.is_some()
        || outputs
            .travelled
            .iter()
            .any(|(_, travel)| *travel != TravelDistance::ZERO)
    {
        return Err(Fault::Outputs);
    }
    let pose = current.movement.state().pose;
    let expected = match facing {
        CurrentFacing::Left => Facing::Left,
        CurrentFacing::Right => Facing::Right,
    };
    if pose.action != Action::Running || pose.facing != expected {
        return Err(Fault::Facing);
    }
    if whole
        .runs
        .iter()
        .any(|run| run.effect != TemplateEffect::None)
    {
        return Err(Fault::Profile);
    }
    whole.bind(&[]).map_err(|_| Fault::Profile)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleared_transient_marker_restores_without_an_animation_state() {
        let marker = ClipStartPreparation::Cleared;
        let bytes = canonical::to_bytes(&marker);
        assert_eq!(
            canonical::from_bytes::<ClipStartPreparation>(&bytes).unwrap(),
            marker
        );
    }
}
