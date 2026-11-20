//! Actual ordinary Animation after the returned Countdown owner.

use super::{Caller, Content, Dependency, Rules, State, appearance_caller};
use crate::production::{ProductionFault, Unsupported, timing};
use canonical::Canonical;
use samus_appearance::{Dash, MoveEvent, SoundsWaiting};
use samus_movement::{Dashing, StandingAnimation, WallTest};
use types::Set;

/// Actual chosen and consumed Animation, independently from Countdown progress.
#[derive(Canonical, Clone, Debug, Default, PartialEq, Eq)]
pub enum Stage {
    /// The actual opportunity has not been selected.
    #[default]
    Pending,
    /// Complete retained answer awaits its physical return.
    Charging(appearance_caller::Stage),
    /// Actual returned answer has been consumed exactly once.
    Complete(appearance_caller::Completion),
}

const fn missing(dependency: Dependency) -> ProductionFault {
    ProductionFault::Unsupported(Unsupported::Frame(dependency))
}

struct AnimationNeighbors {
    run_held: bool,
    items: Set<types::Item>,
    energy: u16,
    liquid: samus_appearance::Liquid,
    sounds_waiting: SoundsWaiting,
}

impl AnimationNeighbors {
    fn with_movement(
        self,
        playing: samus_appearance::Figure,
        outcome: &samus_movement::Outcome,
    ) -> samus_appearance::Inputs {
        let mut moves = Set::default();
        moves[MoveEvent::StandingRestart] =
            outcome.standing_animation == StandingAnimation::Restarts;
        moves[MoveEvent::WallJumpReadied] = outcome.wall_test == WallTest::Readied;
        moves[MoveEvent::HandedOver] = outcome.handed_over;
        moves[MoveEvent::DashLost] = outcome.dash_lost;
        let dash = if outcome.dash_began {
            Dash::Began
        } else {
            match outcome.dashing_after_moves {
                Dashing::Off => Dash::Off,
                Dashing::On => Dash::On,
            }
        };
        samus_appearance::Inputs {
            figure: playing,
            figure_after: playing,
            moves,
            dash,
            run_held: self.run_held,
            heading_down: outcome.heading_down,
            vertical_speed: outcome.vertical_speed_after_moves,
            energy: self.energy,
            items: self.items,
            liquid: self.liquid,
            sounds_waiting: self.sounds_waiting,
        }
    }
}

impl State {
    /// Validate Contact as history after an actual later Appearance operation.
    pub(super) fn animation_prefix_holds(&self, prefix: &super::post_down_caller::Stage) -> bool {
        let receipt = match &self.animation {
            Stage::Pending => return prefix.owners_hold(self),
            Stage::Charging(stage) => {
                if stage.validate(self).is_err() {
                    return false;
                }
                let Some(receipt) = stage.receipt() else {
                    return false;
                };
                receipt
            }
            Stage::Complete(completion) => {
                if completion.work != appearance_caller::Kind::Animation.identity()
                    || completion.receipt.validate().is_err()
                    || !self.animation_result_holds(completion)
                {
                    return false;
                }
                &completion.receipt
            }
        };
        let mut history = self.clone();
        if let super::interruption_caller::Stage::Complete(before) = &self.interruption {
            history.movement = before.as_ref().clone();
        }
        if let Some(sender) = &self.sender_history {
            sender.use_in_history(&mut history);
        }
        history.appearance = receipt.before.clone();
        history.playing_figure = receipt.playing;
        prefix.owners_hold(&history)
    }
    fn animation_result_holds(&self, completion: &appearance_caller::Completion) -> bool {
        if let super::late_movement::Stage::Complete(selected) = &self.late_movement {
            selected.work == super::pose_fit_caller::WORK
                && selected.receipt.validate().is_ok()
                && selected.receipt.before == completion.receipt.result
                && selected.receipt.playing == completion.receipt.playing_after
                && self.s6_drawing.selected_result_holds(selected, self)
                && self.movement.finished().is_some()
        } else {
            self.appearance == completion.receipt.result
                && self.playing_figure == completion.receipt.playing_after
        }
    }

    pub(super) fn fresh_animation_inputs(
        &self,
        content: &Content,
    ) -> Result<samus_appearance::Inputs, ProductionFault> {
        let Caller::PlayerContinuation {
            completed,
            remaining,
            run: super::movement_caller::Run::Complete(run),
            ..
        } = &self.caller
        else {
            return Err(missing(Dependency::AnimationPreparation));
        };
        let liquid = self
            .liquid_history
            .as_ref()
            .ok_or_else(|| missing(Dependency::AnimationLiquidHistory))?
            .at_animation(
                content
                    .liquid_history
                    .as_ref()
                    .ok_or_else(|| missing(Dependency::AnimationLiquidHistory))?,
                completed,
                run,
            )
            .map_err(|_| missing(Dependency::AnimationLiquidHistory))?;
        let energy = self
            .health
            .as_ref()
            .ok_or_else(|| missing(Dependency::HealthCurrentInputs))?
            .at_animation(
                content
                    .health
                    .as_ref()
                    .ok_or_else(|| missing(Dependency::HealthCurrentInputs))?,
                remaining
                    .0
                    .actors
                    .as_ref()
                    .ok_or_else(|| missing(Dependency::HealthCurrentInputs))?,
            )
            .map_err(|_| missing(Dependency::HealthCurrentInputs))?;
        self.senders
            .validate()
            .map_err(|_| missing(Dependency::AnimationPreparation))?;
        self.sender_stage
            .validate(&self.senders)
            .map_err(|_| missing(Dependency::AnimationPreparation))?;
        let queue = u8::try_from(self.senders.contact.0.queue.len())
            .map_err(|_| missing(Dependency::AnimationPreparation))?;
        let sounds_waiting =
            SoundsWaiting::new(queue).ok_or_else(|| missing(Dependency::AnimationPreparation))?;
        let pose = completed
            .0
            .pose_input
            .as_ref()
            .ok_or_else(|| missing(Dependency::AnimationPreparation))?;
        let environment = completed
            .0
            .environment
            .as_ref()
            .ok_or_else(|| missing(Dependency::AnimationPreparation))?;
        let outcome = self
            .movement
            .outputs()
            .ok_or_else(|| missing(Dependency::AnimationPreparation))?;
        let neighbors = AnimationNeighbors {
            run_held: pose.terminal.held_actions.0 & types::button::RUN.0 != 0,
            items: environment.dependency.equipment,
            energy,
            liquid,
            sounds_waiting,
        };
        Ok(neighbors.with_movement(self.playing_figure.0, outcome))
    }

    pub(super) fn advance_animation(
        &mut self,
        content: &Content,
        rules: &Rules,
    ) -> Result<(), ProductionFault> {
        if !self.countdown.holds(self) {
            return Err(missing(Dependency::CountdownPreparation));
        }
        let Caller::PlayerContinuation { post_down, .. } = &self.caller else {
            return Err(missing(Dependency::AnimationPreparation));
        };
        if !self.animation_prefix_holds(post_down) {
            return Err(missing(Dependency::AnimationPreparation));
        }
        if let Some(origin) = &content.liquid_history {
            self.liquid_history
                .as_ref()
                .ok_or_else(|| missing(Dependency::AnimationLiquidHistory))?
                .validate(origin)
                .map_err(|_| missing(Dependency::AnimationLiquidHistory))?;
        } else if self.liquid_history.is_some() {
            return Err(missing(Dependency::AnimationLiquidHistory));
        }
        match &self.animation {
            Stage::Pending => {
                if content.animation.is_none() && content.liquid_history.is_none() {
                    return Err(missing(Dependency::AnimationPreparation));
                }
                self.countdown_scene_offer(content, rules)?;
                let inputs = self.fresh_animation_inputs(content)?;
                let mut stage = appearance_caller::Stage::Idle;
                stage
                    .animation(self, &content.appearance, inputs, rules.appearance.clone())
                    .map_err(|_| missing(Dependency::AnimationPreparation))?;
                let profile = content
                    .animation
                    .as_ref()
                    .ok_or_else(|| missing(Dependency::AnimationPreparation))?
                    .qualify(&stage, self, &content.movement, &content.appearance)
                    .map_err(|_| missing(Dependency::AnimationPreparation))?;
                let work = appearance_caller::Kind::Animation.identity();
                stage
                    .bind(work)
                    .map_err(|_| missing(Dependency::AnimationPreparation))?;
                self.timing
                    .begin(work, profile)
                    .map_err(ProductionFault::FrameHardware)?;
                self.animation = Stage::Charging(stage);
            }
            Stage::Charging(stage) => {
                stage
                    .validate(self)
                    .map_err(|_| missing(Dependency::AnimationPreparation))?;
                self.fresh_animation_inputs(content)?;
                let profile = content
                    .animation
                    .as_ref()
                    .ok_or_else(|| missing(Dependency::AnimationPreparation))?
                    .qualify(stage, self, &content.movement, &content.appearance)
                    .map_err(|_| missing(Dependency::AnimationPreparation))?;
                if self
                    .timing
                    .work
                    .iter()
                    .filter(|work| {
                        work.kind
                            == timing::WorkKind::Semantic(
                                appearance_caller::Kind::Animation.identity(),
                            )
                    })
                    .count()
                    != 1
                    || self
                        .timing
                        .work
                        .iter()
                        .find(|work| {
                            work.kind
                                == timing::WorkKind::Semantic(
                                    appearance_caller::Kind::Animation.identity(),
                                )
                        })
                        .is_none_or(|work| work.profile != profile)
                {
                    return Err(missing(Dependency::AnimationPreparation));
                }
            }
            Stage::Complete(_) => return Err(missing(Dependency::InterruptionPreparation)),
        }
        Ok(())
    }

    pub(super) fn complete_animation(
        &mut self,
        returned: timing::WorkId,
    ) -> Result<(), ProductionFault> {
        let Stage::Charging(stage) = &self.animation else {
            return Err(missing(Dependency::AnimationPreparation));
        };
        let mut stage = stage.clone();
        let (completion, outputs) = stage
            .complete(self, returned)
            .map_err(|_| missing(Dependency::AnimationPreparation))?;
        if !outputs.sounds.is_empty() || !outputs.graphics.is_empty() {
            return Err(missing(Dependency::AnimationPreparation));
        }
        self.animation = Stage::Complete(completion);
        Ok(())
    }
}

#[cfg(test)]
mod tests;
