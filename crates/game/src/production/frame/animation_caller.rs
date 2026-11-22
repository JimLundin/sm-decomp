//! Actual ordinary Animation after the returned Countdown owner.

use super::{Caller, Content, Dependency, Rules, State, appearance_caller};
use crate::production::{ProductionFault, Unsupported, timing};
use canonical::Canonical;
use samus_appearance::{Dash, MoveEvent, SoundsWaiting};
use samus_movement::{Dashing, StandingAnimation, WallTest};
use types::{Parts, Set};

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

/// Admitted room-dependent conditions for the adjacent ordinary Animation hold.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Adjacent {
    /// Actual origin room independently shown unchanged through the hold.
    pub room: data::id::RoomId,
    /// Complete independently admitted original image identity.
    pub origin: [u8; 32],
    /// Whether the retained FX dispatch takes its quiet branch.
    pub quiet_fx: bool,
    /// Independently observed horizontal slowdown divisor.
    pub x_divisor: u16,
    /// Effect-free CPU envelope for the hold.
    pub physical: appearance_caller::PhysicalContract,
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

    /// Read the live Movement, Health and Appearance neighbours for the selected figure.
    /// # Errors
    /// The current Run, actor, interrupt and input lifetimes must all be qualified.
    pub fn fresh_animation_inputs(
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
        let actors = remaining
            .0
            .actors
            .as_ref()
            .ok_or_else(|| missing(Dependency::HealthCurrentInputs))?;
        let energy = self.energy_for_animation(content, actors)?;
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

    fn animation_contract<'a>(
        &self,
        content: &'a Content,
        time_left: samus_appearance::Frames,
    ) -> Result<&'a appearance_caller::PhysicalContract, ProductionFault> {
        let unavailable = || missing(Dependency::AnimationPreparation);
        if self.publications.original_interval() {
            return content.animation.as_ref().ok_or_else(unavailable);
        }
        let adjacent = content
            .adjacent_animation
            .as_ref()
            .ok_or_else(unavailable)?;
        let origin = content.liquid_history.as_ref().ok_or_else(unavailable)?;
        let samus = self.movement.state();
        let centre = samus.position.y.pixel().0;
        let height = content
            .movement
            .radius(self.playing_figure.0.pose)
            .ok_or_else(unavailable)?
            .y
            .0;
        // The reached dry branch still reads the current figure's body edges.
        let ordinary_edges = height
            .checked_sub(1)
            .and_then(|below| centre.checked_add(below))
            .is_some()
            && centre.checked_sub(height).is_some();
        if adjacent.origin != origin.origin
            || !ordinary_edges
            || self.validate_room_owner(content) != Ok(adjacent.room)
            || !adjacent.quiet_fx
            || adjacent.x_divisor != 0
        {
            return Err(unavailable());
        }
        if time_left == samus_appearance::Frames::LAST
            && let Some(advance) = &content.recurring_animation_advance
        {
            return Ok(advance);
        }
        Ok(&adjacent.physical)
    }

    fn animation_profile(
        &self,
        content: &Content,
        selection: &appearance_caller::Stage,
    ) -> Result<data::timing::Profile, ProductionFault> {
        if !self.animation_energy_matches(selection) {
            return Err(missing(Dependency::HealthCurrentInputs));
        }
        let before = selection
            .receipt()
            .ok_or_else(|| missing(Dependency::AnimationPreparation))?;
        self.animation_contract(content, before.before.model.0.state.playhead.time_left)?
            .qualify(selection, self, &content.movement, &content.appearance)
            .map_err(|_| missing(Dependency::AnimationPreparation))
    }

    fn animation_energy_matches(&self, selection: &appearance_caller::Stage) -> bool {
        let Some(health) = &self.health else {
            return false;
        };
        selection
            .receipt()
            .and_then(|receipt| receipt.chosen.model.0.pending.as_ref())
            .is_some_and(|answer| answer.inputs.energy == health.state().energy.whole.get())
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
                if !self.publications.original_interval() && content.adjacent_animation.is_some() {
                    self.animation_contract(
                        content,
                        self.appearance.model.0.state.playhead.time_left,
                    )?;
                }
                self.countdown_scene_offer(content, rules)?;
                let inputs = self.fresh_animation_inputs(content)?;
                let mut stage = appearance_caller::Stage::Idle;
                stage
                    .animation(self, &content.appearance, inputs, rules.appearance.clone())
                    .map_err(|_| missing(Dependency::AnimationPreparation))?;
                let profile = self.animation_profile(content, &stage)?;
                let work = appearance_caller::Kind::Animation.identity();
                stage
                    .bind(work)
                    .map_err(|_| missing(Dependency::AnimationPreparation))?;
                self.timing
                    .begin(work, profile)
                    .map_err(ProductionFault::FrameHardware)?;
                self.animation = Stage::Charging(stage);
            }
            Stage::Charging(stage) => self.validate_animation_charge(content, rules, stage)?,
            Stage::Complete(_) => return Err(missing(Dependency::InterruptionPreparation)),
        }
        Ok(())
    }

    fn validate_animation_charge(
        &self,
        content: &Content,
        rules: &Rules,
        selection: &appearance_caller::Stage,
    ) -> Result<(), ProductionFault> {
        let identity = appearance_caller::Kind::Animation.identity();
        let unexpected = || missing(Dependency::AnimationPreparation);
        if !matches!(selection, appearance_caller::Stage::Charging { work, .. } if *work == identity)
        {
            return Err(unexpected());
        }
        let inputs = self.fresh_animation_inputs(content)?;
        let expected = self.animation_profile(content, selection)?;
        if !self.publications.original_interval() {
            self.countdown_scene_offer(content, rules)?;
            if !Self::retained_inputs_match(
                selection.receipt().ok_or_else(unexpected)?,
                inputs,
                rules,
            ) {
                return Err(unexpected());
            }
        }
        let mut charges = self
            .timing
            .work
            .iter()
            .filter(|work| work.kind == timing::WorkKind::Semantic(identity));
        if charges.next().is_none_or(|work| work.profile != expected) || charges.next().is_some() {
            return Err(unexpected());
        }
        Ok(())
    }

    fn retained_inputs_match(
        receipt: &appearance_caller::Receipt,
        inputs: samus_appearance::Inputs,
        rules: &Rules,
    ) -> bool {
        receipt
            .chosen
            .model
            .0
            .animation
            .as_ref()
            .is_some_and(|data| data.inputs == inputs && data.rules == rules.appearance)
    }

    pub(crate) fn validate_animation_checkpoint(
        &self,
        content: &Content,
        rules: &Rules,
    ) -> Result<(), super::Fault> {
        use super::Fault::InvalidPlayerEntry;
        let work = appearance_caller::Kind::Animation.identity();
        let charging = self
            .timing
            .work
            .iter()
            .any(|item| item.kind == timing::WorkKind::Semantic(work));
        if !matches!(self.animation, Stage::Pending)
            && !matches!(
                self.countdown,
                super::countdown_caller::Stage::Complete { .. }
            )
        {
            return Err(InvalidPlayerEntry);
        }
        match &self.animation {
            Stage::Pending => {
                if charging {
                    return Err(InvalidPlayerEntry);
                }
            }
            Stage::Charging(selection) => self
                .validate_animation_charge(content, rules, selection)
                .map_err(|_| InvalidPlayerEntry)?,
            Stage::Complete(done) => {
                if charging || done.work != work {
                    return Err(InvalidPlayerEntry);
                }
                if self.publications.original_interval()
                    && !done
                        .receipt
                        .chosen
                        .model
                        .0
                        .pending
                        .as_ref()
                        .is_some_and(|answer| {
                            Some(answer.inputs.energy) == self.original_animation_energy(content)
                        })
                {
                    return Err(InvalidPlayerEntry);
                }
                if !self.publications.original_interval() {
                    let (_, in_nmi) = self
                        .gameplay_publications()
                        .map_err(|_| InvalidPlayerEntry)?;
                    self.validate_animation_countdown(content, rules, in_nmi)
                        .map_err(|_| InvalidPlayerEntry)?;
                    let mut at_animation = self.clone();
                    if let super::interruption_caller::Stage::Complete(before) = &self.interruption
                    {
                        at_animation.movement = before.as_ref().clone();
                        at_animation.playing_figure = done.receipt.playing;
                        if let Some(sender) = &self.sender_history {
                            sender.use_in_history(&mut at_animation);
                        }
                    }
                    let inputs = at_animation
                        .fresh_animation_inputs(content)
                        .map_err(|_| InvalidPlayerEntry)?;
                    if !Self::retained_inputs_match(&done.receipt, inputs, rules) {
                        return Err(InvalidPlayerEntry);
                    }
                }
                // Appearance has consumed the answer. Its retained receipt still identifies
                // the physical family that could have completed at this opportunity.
                self.animation_contract(
                    content,
                    done.receipt.before.model.0.state.playhead.time_left,
                )
                .map_err(|_| InvalidPlayerEntry)?
                .profile_for_receipt(&done.receipt, self, &content.movement, &content.appearance)
                .map_err(|_| InvalidPlayerEntry)?;
            }
        }
        Ok(())
    }

    fn validate_animation_countdown(
        &self,
        content: &Content,
        rules: &Rules,
        in_nmi: bool,
    ) -> Result<(), ProductionFault> {
        if in_nmi {
            return Ok(());
        }
        if matches!(self.late_hud, super::late_hud::Stage::Completed(_)) {
            // Late status now owns the tilemap. Revalidate the actual Countdown offer
            // from its retained scene and IRQ history.
            let super::countdown_caller::Stage::Complete { offer, .. } = self.countdown else {
                return Err(missing(Dependency::CountdownPreparation));
            };
            self.revalidate_countdown_scene_at_ceres(content, offer)
        } else {
            self.countdown_scene_offer(content, rules).map(|_| ())
        }
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
