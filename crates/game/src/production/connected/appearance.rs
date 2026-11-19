//! Shared semantic reads and retained choices for actual appearance owners.

use super::{Fault, inputs::Inputs};
use crate::production::{AppearanceContent, appearance::Work, movement_work::Movement};
use samus_appearance::{AnimationDecision, AppearanceRead, Figure, MoveEvent, WorkStep};
use samus_movement::suspended::{LocalRule, MovementDecision};
use types::{Action, Facing, Set, Sprite};

pub(super) fn running(figure: Figure, content: &AppearanceContent) -> Result<(), Fault> {
    if figure.pose.action != Action::Running
        || !matches!(figure.pose.facing, Facing::Left | Facing::Right)
        || figure.sprite != Sprite::Plain
        || content.get(figure).is_none()
    {
        return Err(Fault::Figure);
    }
    Ok(())
}

/// Read only the currently playing owner; no future accepted pose supplies Ahead.
pub(super) fn ahead(
    work: &Work,
    playing: Figure,
    content: &AppearanceContent,
) -> Result<samus_appearance::Ahead, Fault> {
    running(playing, content)?;
    Ok(crate::production::appearance::ahead(
        &work.model.0.state,
        content.get(playing).ok_or(Fault::Figure)?,
    ))
}

pub(super) fn contact(
    work: &mut Work,
    figure: Figure,
    inputs: &Inputs,
    content: &AppearanceContent,
) -> Result<(), Fault> {
    running(figure, content)?;
    let read = AppearanceRead::Contact {
        figure,
        playhead: work.model.0.state.playhead,
        stage: work.model.0.state.stage,
        clip: content.get(figure).ok_or(Fault::Figure)?.clone(),
        suppressors: inputs.contact,
    };
    work.model.0.steps.push(WorkStep::Appearance(read));
    if !work.complete() {
        return Err(Fault::Appearance);
    }
    Ok(())
}

pub(super) fn animation_inputs(
    movement: &Movement,
    previous: &samus_movement::Samus,
    playing: Figure,
    accepted: Figure,
    inputs: &Inputs,
) -> Result<samus_appearance::Inputs, Fault> {
    let outputs = movement.outputs().ok_or(Fault::Movement)?;
    if previous.run.dashing != movement.state().run.dashing
        || outputs.dashing_after_moves != previous.run.dashing
        || movement.state().pose.action != Action::Running
        || outputs.wall_test != samus_movement::WallTest::Neither
    {
        return Err(Fault::Family);
    }
    let mut moves = Set::default();
    moves[MoveEvent::StandingRestart] = false;
    moves[MoveEvent::WallJumpReadied] = false;
    moves[MoveEvent::DashLost] = previous.run.dashing == samus_movement::Dashing::On
        && movement.state().run.dashing == samus_movement::Dashing::Off;
    moves[MoveEvent::HandedOver] = movement
        .decisions()
        .iter()
        .rev()
        .find_map(|decision| {
            if let MovementDecision::Local {
                rule: LocalRule::PoseChoice { choice, .. },
                ..
            } = decision
            {
                Some(choice.handed_over)
            } else {
                None
            }
        })
        .unwrap_or(false);
    Ok(samus_appearance::Inputs {
        figure: playing,
        figure_after: accepted,
        moves,
        dash: match outputs.dashing_after_moves {
            samus_movement::Dashing::Off => samus_appearance::Dash::Off,
            samus_movement::Dashing::On => samus_appearance::Dash::On,
        },
        run_held: inputs.held.0 & types::button::RUN.0 != 0,
        heading_down: outputs.heading_down,
        vertical_speed: outputs.vertical_speed_after_moves,
        energy: inputs.energy,
        items: inputs.items,
        liquid: inputs.liquid,
        sounds_waiting: inputs.sounds_waiting,
    })
}

pub(super) fn begin_animation(
    work: &mut Work,
    data: samus_appearance::AnimationInputs,
) -> Result<(), Fault> {
    if work.model.0.pending.is_some() || work.model.0.consumed != work.model.0.steps.len() {
        return Err(Fault::Appearance);
    }
    work.model.0.animation = Some(data);
    work.model
        .0
        .steps
        .extend(samus_appearance::SuspendedAppearance::animation_steps());
    Ok(())
}

/// Resolve actual consumed initialization, without running later Finish for its sprite.
pub(super) fn publication(
    movement: &Movement,
    playing: Figure,
    content: &AppearanceContent,
) -> Result<(Figure, Figure), Fault> {
    let Some(MovementDecision::Local {
        rule: LocalRule::PoseInitialization { accepted },
        before,
        answer,
        ..
    }) = movement.decisions().last()
    else {
        return Err(Fault::Publication);
    };
    if answer != movement.state() || answer.pose != accepted.pose {
        return Err(Fault::Publication);
    }
    let from = Figure {
        pose: before.pose,
        sprite: playing.sprite,
    };
    let selected = Figure {
        pose: accepted.pose,
        sprite: playing.sprite,
    };
    running(from, content)?;
    running(selected, content)?;
    Ok((from, selected))
}

pub(super) fn selection(
    work: &mut Work,
    from: Figure,
    selected: Figure,
    inputs: &Inputs,
) -> Result<(), Fault> {
    if work.model.0.pending.is_some()
        || !matches!(
            work.model.0.steps.get(work.model.0.consumed),
            Some(WorkStep::Animation(AnimationDecision::SelectedClip))
        )
    {
        return Err(Fault::Appearance);
    }
    let read = WorkStep::Appearance(AppearanceRead::Selection {
        from,
        selected,
        items: inputs.items,
        submerged: inputs.liquid.submerged,
        cinematic: inputs.cinematic,
    });
    let at = work.model.0.consumed;
    work.model.0.steps = work
        .model
        .0
        .steps
        .iter()
        .take(at)
        .cloned()
        .chain(core::iter::once(read))
        .chain(work.model.0.steps.iter().skip(at).cloned())
        .collect();
    if !work.complete() {
        return Err(Fault::Appearance);
    }
    Ok(())
}
