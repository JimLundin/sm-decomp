//! Shared physical pose fitting, movement return, and selected-clip publication.

use super::{Caller, Content, Dependency, Rules, State, appearance_caller, late_movement};
use crate::production::{ProductionFault, Unsupported, timing::WorkId};
use canonical::Canonical;
use samus_movement::suspended::{LocalRule, MovementDecision};
use types::{BodyFitResult, Facing};

pub(super) const WORK: WorkId = appearance_caller::Kind::SelectedClip.identity();

/// Actual movement choices through initialization and the retained selected appearance answer.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct FitWork {
    before: super::movement::Movement,
    initialized: super::movement::Movement,
    appearance: appearance_caller::Stage,
}

const fn missing() -> ProductionFault {
    ProductionFault::Unsupported(Unsupported::Frame(Dependency::PoseFitPreparation))
}

impl State {
    pub(super) fn choose_pose_fit(
        &self,
        content: &Content,
        rules: &Rules,
    ) -> Result<(FitWork, data::timing::Profile), ProductionFault> {
        let fit = content.pose_fit.as_ref().ok_or_else(missing)?;
        let inputs = self.late_movement_inputs()?;
        let world = self.world.as_ref().ok_or_else(missing)?;
        let radius = content
            .movement
            .radius(self.movement.state().pose)
            .ok_or_else(missing)?;
        let body = |_| radius;
        let cannon = |pose| content.movement.cannon_heading(pose);
        let frame = inputs.frame(&body, &cannon);
        let mut candidate = self.clone();
        let movement = &mut candidate.movement;
        let Some(MovementDecision::Local {
            rule:
                LocalRule::BodyFit {
                    old_radius,
                    selected_radius,
                    selected_pose,
                    result,
                },
            before,
            answer,
            ..
        }) = movement
            .offer(&content.movement, &rules.movement, &frame, &world.0)
            .map_err(|_| missing())?
        else {
            return Err(missing());
        };
        if selected_pose == before.pose
            || selected_pose.facing == Facing::Screen
            || before.pose.facing == Facing::Screen
            || old_radius.y.0 < selected_radius.y.0
            || content.movement.radius(before.pose) != Some(old_radius)
            || content.movement.radius(selected_pose) != Some(selected_radius)
            || result != BodyFitResult::NoGrowth
            || before != answer
        {
            return Err(missing());
        }
        movement
            .complete(&content.movement, &rules.movement, &frame, &world.0)
            .map_err(|_| missing())?;
        let Some(MovementDecision::Local {
            rule: LocalRule::PoseInitialization { accepted },
            ..
        }) = movement
            .offer(&content.movement, &rules.movement, &frame, &world.0)
            .map_err(|_| missing())?
        else {
            return Err(missing());
        };
        if accepted.pose != selected_pose
            || accepted.lower.is_some()
            || !accepted.effects.is_empty()
        {
            return Err(missing());
        }
        movement
            .complete(&content.movement, &rules.movement, &frame, &world.0)
            .map_err(|_| missing())?;
        let (from, selected) = movement
            .selected_running_figures(self.playing_figure.0, &content.appearance)
            .ok_or_else(missing)?;
        let mut animation = self.fresh_animation_inputs(content)?;
        animation.figure_after = selected;
        let cinematic = if inputs.cinematic_active {
            samus_appearance::CinematicActivity::Active
        } else {
            samus_appearance::CinematicActivity::Inactive
        };
        let mut appearance = appearance_caller::Stage::Idle;
        appearance
            .selected_clip(
                &mut candidate,
                &content.appearance,
                animation,
                from,
                rules.appearance.clone(),
                cinematic,
            )
            .map_err(|_| missing())?;
        let profile = fit
            .qualify(
                &appearance,
                &candidate,
                &content.movement,
                &content.appearance,
            )
            .map_err(|_| missing())?;
        appearance.bind(WORK).map_err(|_| missing())?;
        Ok((
            FitWork {
                before: self.movement.clone(),
                initialized: candidate.movement,
                appearance,
            },
            profile,
        ))
    }

    pub(super) fn validate_fit(
        &self,
        fit: &FitWork,
        content: &Content,
    ) -> Result<data::timing::Profile, ProductionFault> {
        if self.movement != fit.before
            || fit
                .initialized
                .decisions()
                .strip_prefix(fit.before.decisions())
                .is_none_or(|suffix| {
                    !matches!(
                        suffix,
                        [
                            MovementDecision::Local {
                                rule: LocalRule::BodyFit { .. },
                                ..
                            },
                            MovementDecision::Local {
                                rule: LocalRule::PoseInitialization { .. },
                                ..
                            }
                        ]
                    )
                })
        {
            return Err(missing());
        }
        let candidate = self.fit_candidate(fit)?;
        content
            .pose_fit
            .as_ref()
            .ok_or_else(missing)?
            .qualify(
                &fit.appearance,
                &candidate,
                &content.movement,
                &content.appearance,
            )
            .map_err(|_| missing())
    }

    fn fit_candidate(&self, fit: &FitWork) -> Result<Self, ProductionFault> {
        let receipt = fit.appearance.receipt().ok_or_else(missing)?;
        if self.appearance != receipt.before || self.playing_figure != receipt.playing {
            return Err(missing());
        }
        let mut candidate = self.clone();
        candidate.movement = fit.initialized.clone();
        candidate.appearance = receipt.chosen.clone();
        Ok(candidate)
    }

    pub(super) fn finish_pose_fit(
        &mut self,
        content: &Content,
        rules: &Rules,
    ) -> Result<(), ProductionFault> {
        let late_movement::Stage::Fit(fit) = &self.late_movement else {
            return Err(missing());
        };
        let _profile = self.validate_fit(fit, content)?;
        let mut appearance = fit.appearance.clone();
        let mut candidate = self.fit_candidate(fit)?;
        let (completed, outputs) = appearance
            .complete(&mut candidate, WORK)
            .map_err(|_| missing())?;
        if !outputs.sounds.is_empty() || !outputs.graphics.is_empty() {
            return Err(missing());
        }
        let inputs = self.late_movement_inputs()?;
        let Caller::PlayerContinuation { remaining, .. } = &self.caller else {
            return Err(missing());
        };
        let world = content
            .movement_room
            .as_ref()
            .ok_or_else(missing)?
            .world(remaining.0.actors.as_ref().ok_or_else(missing)?)
            .map_err(|_| missing())?;
        let radius = content
            .movement
            .radius(candidate.movement.state().pose)
            .ok_or_else(missing)?;
        let body = |_| radius;
        let cannon = |pose| content.movement.cannon_heading(pose);
        let frame = inputs.frame(&body, &cannon);
        for required_finish in [false, true] {
            let offered = candidate
                .movement
                .offer(&content.movement, &rules.movement, &frame, &world)
                .map_err(|_| missing())?;
            if !matches!(
                (required_finish, offered),
                (
                    false,
                    Some(MovementDecision::Local {
                        rule: LocalRule::FloorResponse { contact: None },
                        ..
                    })
                ) | (
                    true,
                    Some(MovementDecision::Local {
                        rule: LocalRule::Finish,
                        ..
                    })
                )
            ) {
                return Err(missing());
            }
            candidate
                .movement
                .complete(&content.movement, &rules.movement, &frame, &world)
                .map_err(|_| missing())?;
        }
        let (_, outputs) = candidate.movement.finished().ok_or_else(missing)?;
        if outputs.floor_response.is_some()
            || !outputs.touchdown_requests.is_empty()
            || outputs.touchdown_graphics
            || outputs.sprite != candidate.playing_figure.0.sprite
        {
            return Err(missing());
        }
        self.movement = candidate.movement;
        self.appearance = candidate.appearance;
        self.playing_figure = candidate.playing_figure;
        self.late_movement = late_movement::Stage::Complete(Box::new(completed));
        Ok(())
    }
}
