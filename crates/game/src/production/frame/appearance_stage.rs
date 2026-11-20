//! Current appearance reads and retained animation effects at physical completion.

use super::State;
use crate::production::appearance::{Outputs, Work, WorkFault};
use samus_appearance::{
    AnimationDecision, AnimationInputs, AppearanceRead, CinematicActivity, ContactSuppressor,
    OfferedDecision, WorkStep,
};
use types::{Item, Set};

/// An appearance opportunity cannot replace unfinished work or repeat completion.
#[derive(Debug, PartialEq, Eq)]
pub enum StageFault {
    /// A prior opportunity remains unconsumed, or no opportunity was chosen.
    Order,
    /// The complete retained work has invalid consumer or decision progress.
    Work(WorkFault),
    /// Current selected owners or checked clip content are unavailable.
    Owner(super::Fault),
}

fn ready(work: &Work) -> Result<(), StageFault> {
    work.validate().map_err(StageFault::Work)?;
    if work.model.0.pending.is_some() || work.model.0.consumed != work.model.0.steps.len() {
        return Err(StageFault::Order);
    }
    Ok(())
}

/// Retain one actual animation answer without applying its effects.
/// The physical caller groups decisions according to its qualified work profile.
///
/// # Errors
/// Invalid retained work or an unfinished previous opportunity rejects atomically.
pub fn choose_animation(
    work: &mut Work,
    inputs: AnimationInputs,
    rule: AnimationDecision,
) -> Result<Option<OfferedDecision>, StageFault> {
    ready(work)?;
    let mut candidate = work.clone();
    candidate.model.0.animation = Some(inputs);
    candidate.model.0.steps.push(WorkStep::Animation(rule));
    let answer = candidate.offer().cloned();
    *work = candidate;
    Ok(answer)
}

/// Capture one read at its actual opportunity, without publishing any effects.
/// Previously completed receipts and external delivery positions remain owned.
///
/// # Errors
/// Invalid retained work or an unfinished previous opportunity.
pub fn choose_read(work: &mut Work, input: AppearanceRead) -> Result<(), StageFault> {
    ready(work)?;
    work.model.0.steps.push(WorkStep::Appearance(input));
    Ok(())
}

/// Capture contact from the actual movement figure before animation advances.
/// Suppressors belong to their current external owners and affect sound alone.
///
/// # Errors
/// Unfinished work or missing checked current-figure content.
pub fn choose_contact(
    state: &mut State,
    content: &crate::production::AppearanceContent,
    figure: samus_appearance::Figure,
    suppressors: Set<ContactSuppressor>,
) -> Result<(), StageFault> {
    if figure.pose != state.movement.state().pose {
        return Err(StageFault::Order);
    }
    let input = AppearanceRead::Contact {
        figure,
        playhead: state.appearance.model.0.state.playhead,
        stage: state.appearance.model.0.state.stage,
        clip: state
            .current_clip(content)
            .map_err(StageFault::Owner)?
            .clone(),
        suppressors,
    };
    choose_read(&mut state.appearance, input)
}

/// Capture the completed movement figure at its actual selection opportunity.
/// Capturing the read does not publish a new playing figure before its completion.
///
/// # Errors
/// Unfinished movement, missing clip content or unfinished appearance work.
pub fn choose_selection(
    state: &mut State,
    content: &crate::production::AppearanceContent,
    items: Set<Item>,
    submerged: bool,
    cinematic: CinematicActivity,
) -> Result<(), StageFault> {
    let input = AppearanceRead::Selection {
        from: state.playing_figure.0,
        selected: state.accepted_figure(content).map_err(StageFault::Owner)?,
        items,
        submerged,
        cinematic,
    };
    choose_read(&mut state.appearance, input)
}

/// Capture drawing from the independently prepared action and current movement.
///
/// # Errors
/// Missing preparation or unfinished appearance work.
pub fn choose_drawing(state: &mut State) -> Result<(), StageFault> {
    let input = state.drawing_input().map_err(StageFault::Owner)?;
    choose_read(&mut state.appearance, input)
}

/// Complete selected-clip initialization and publish its retained actual figure together.
/// A later movement selection cannot replace the figure that chose this answer.
///
/// # Errors
/// Missing or different pending decision, or invalid retained work.
pub fn complete_selected_clip(state: &mut State) -> Result<Outputs, StageFault> {
    let decision = state
        .appearance
        .model
        .0
        .pending
        .as_ref()
        .ok_or(StageFault::Order)?;
    if decision.rule != AnimationDecision::SelectedClip {
        return Err(StageFault::Order);
    }
    let selected = decision.inputs.figure_after;
    let outputs = complete(&mut state.appearance)?;
    state.playing_figure.0 = selected;
    Ok(outputs)
}

/// Apply one retained effect and return its once-only downstream output delta.
///
/// Call only after the physical caller admits completion, on its private candidate;
/// commit the candidate together with the actual sound and graphical consumers.
///
/// # Errors
/// Invalid work or absent/already consumed opportunity rejects atomically.
pub fn complete(work: &mut Work) -> Result<Outputs, StageFault> {
    work.validate().map_err(StageFault::Work)?;
    if work.model.0.consumed == work.model.0.steps.len() {
        return Err(StageFault::Order);
    }
    if matches!(
        work.model.0.steps.get(work.model.0.consumed),
        Some(WorkStep::Animation(_))
    ) && work.model.0.pending.is_none()
    {
        return Err(StageFault::Order);
    }
    let mut candidate = work.clone();
    if !candidate.complete() {
        return Err(StageFault::Order);
    }
    let outputs = candidate.deliver().map_err(StageFault::Work)?;
    *work = candidate;
    Ok(outputs)
}

#[cfg(test)]
mod tests;
