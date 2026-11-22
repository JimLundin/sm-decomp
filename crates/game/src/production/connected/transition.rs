//! Actual semantic transition ordering for the retained connected owners.
use super::{
    AnimationDecision, Content, Fault, Figure, LocalRule, Model, MovementDecision, Phase, Rules,
    State, WorkStep, appearance,
};

impl State {
    pub(super) fn movement_phase(
        &mut self,
        phase: Phase,
        content: &Content,
        rules: &Rules,
    ) -> Result<Phase, Fault> {
        Ok(match phase {
            Phase::Motion(ctx) => {
                self.movement_operation(&ctx, content, rules, false)?;
                Phase::MotionComplete(ctx)
            }
            Phase::MotionComplete(ctx) => {
                let decision = self.movement_operation(&ctx, content, rules, true)?;
                if matches!(
                    decision,
                    MovementDecision::Local {
                        rule: LocalRule::VerticalSpeedGrowth,
                        ..
                    }
                ) {
                    Phase::Contact(ctx)
                } else {
                    Phase::Motion(ctx)
                }
            }
            Phase::Pose(ctx) => {
                self.movement_operation(&ctx, content, rules, false)?;
                Phase::PoseComplete(ctx)
            }
            Phase::PoseComplete(ctx) => {
                let decision = self.movement_operation(&ctx, content, rules, true)?;
                if matches!(
                    decision,
                    MovementDecision::Local {
                        rule: LocalRule::PoseInitialization { .. },
                        ..
                    }
                ) {
                    Phase::Selection(ctx)
                } else {
                    Phase::Pose(ctx)
                }
            }
            Phase::RemainingMotion(ctx) => {
                if self.movement.finished().is_some() {
                    Phase::DashLoss(ctx)
                } else {
                    self.movement_operation(&ctx, content, rules, false)?;
                    Phase::RemainingMotionComplete(ctx)
                }
            }
            Phase::RemainingMotionComplete(ctx) => {
                self.movement_operation(&ctx, content, rules, true)?;
                Phase::RemainingMotion(ctx)
            }
            Phase::Ready
            | Phase::EarlyMap(_)
            | Phase::Contact(_)
            | Phase::Animation(_)
            | Phase::AnimationComplete(_)
            | Phase::Selection(_)
            | Phase::SelectedClip(_)
            | Phase::SelectedClipComplete(_)
            | Phase::DashLoss(_)
            | Phase::DashLossComplete(_)
            | Phase::Camera(_) => return Err(Fault::Order),
        })
    }
    pub(super) fn appearance_phase(
        &mut self,
        phase: Phase,
        content: &Content,
        rules: &Rules,
    ) -> Result<Phase, Fault> {
        Ok(match phase {
            Phase::Contact(ctx) => {
                let figure = Figure {
                    pose: self.movement.state().pose,
                    sprite: ctx.playing.0.sprite,
                };
                appearance::contact(
                    &mut self.appearance,
                    figure,
                    &ctx.inputs.0,
                    &content.appearance,
                )?;
                if self.early_map.is_some() {
                    Phase::EarlyMap(ctx)
                } else {
                    self.begin_animation(ctx, content, rules)?
                }
            }
            Phase::EarlyMap(ctx) => self.begin_animation(ctx, content, rules)?,
            Phase::Animation(ctx) => {
                if matches!(
                    self.appearance
                        .model
                        .0
                        .steps
                        .get(self.appearance.model.0.consumed),
                    Some(WorkStep::Animation(AnimationDecision::SelectedClip))
                ) {
                    Phase::Pose(ctx)
                } else {
                    if self.appearance.offer().is_none() {
                        return Err(Fault::Appearance);
                    }
                    Phase::AnimationComplete(ctx)
                }
            }
            Phase::AnimationComplete(ctx) => {
                if !self.appearance.complete() {
                    return Err(Fault::Appearance);
                }
                Phase::Animation(ctx)
            }
            Phase::Selection(ctx) => {
                let (from, selected) =
                    appearance::publication(&self.movement, ctx.playing.0, &content.appearance)?;
                appearance::selection(&mut self.appearance, from, selected, &ctx.inputs.0)?;
                self.appearance.model.0.animation =
                    Some(self.animation_data(&ctx, from, selected, content, rules)?);
                Phase::SelectedClip(ctx)
            }
            Phase::SelectedClip(ctx) => {
                if self.appearance.offer().is_none() {
                    return Err(Fault::Appearance);
                }
                Phase::SelectedClipComplete(ctx)
            }
            Phase::SelectedClipComplete(ctx) => {
                let offered = self
                    .appearance
                    .model
                    .0
                    .pending
                    .clone()
                    .ok_or(Fault::Appearance)?;
                if offered.rule != AnimationDecision::SelectedClip || !self.appearance.complete() {
                    return Err(Fault::Appearance);
                }
                self.playing = Model(offered.inputs.figure_after);
                Phase::RemainingMotion(ctx)
            }
            Phase::DashLoss(_)
            | Phase::DashLossComplete(_)
            | Phase::Ready
            | Phase::Motion(_)
            | Phase::MotionComplete(_)
            | Phase::Pose(_)
            | Phase::PoseComplete(_)
            | Phase::RemainingMotion(_)
            | Phase::RemainingMotionComplete(_)
            | Phase::Camera(_) => return Err(Fault::Order),
        })
    }
    fn begin_animation(
        &mut self,
        ctx: Box<super::Context>,
        content: &Content,
        rules: &Rules,
    ) -> Result<Phase, Fault> {
        let figure = Figure {
            pose: self.movement.state().pose,
            sprite: ctx.playing.0.sprite,
        };
        let data = self.animation_data(&ctx, figure, figure, content, rules)?;
        appearance::begin_animation(&mut self.appearance, data)?;
        Ok(Phase::Animation(ctx))
    }

    pub(super) fn dash_phase(&mut self, phase: Phase) -> Result<Phase, Fault> {
        Ok(match phase {
            Phase::DashLoss(ctx) => {
                let (_, out) = self.movement.finished().ok_or(Fault::Movement)?;
                if out.dash_began
                    || out.dash_lost
                    || out.wall_test != samus_movement::WallTest::Neither
                    || out.standing_animation == samus_movement::StandingAnimation::Restarts
                {
                    return Err(Fault::Family);
                }
                if self.appearance.offer().is_some() {
                    Phase::DashLossComplete(ctx)
                } else {
                    if self.appearance.model.0.consumed != self.appearance.model.0.steps.len() {
                        return Err(Fault::Appearance);
                    }
                    Phase::Camera(ctx)
                }
            }
            Phase::DashLossComplete(ctx) => {
                if !self.appearance.complete()
                    || self.appearance.model.0.consumed != self.appearance.model.0.steps.len()
                {
                    return Err(Fault::Appearance);
                }
                Phase::Camera(ctx)
            }
            Phase::Ready
            | Phase::Motion(_)
            | Phase::MotionComplete(_)
            | Phase::EarlyMap(_)
            | Phase::Contact(_)
            | Phase::Animation(_)
            | Phase::AnimationComplete(_)
            | Phase::Pose(_)
            | Phase::PoseComplete(_)
            | Phase::Selection(_)
            | Phase::SelectedClip(_)
            | Phase::SelectedClipComplete(_)
            | Phase::RemainingMotion(_)
            | Phase::RemainingMotionComplete(_)
            | Phase::Camera(_) => return Err(Fault::Order),
        })
    }
    pub(super) fn camera_phase(&mut self, phase: Phase, rules: &Rules) -> Result<Phase, Fault> {
        let Phase::Camera(ctx) = phase else {
            return Err(Fault::Order);
        };
        super::super::camera_semantic::follow(
            &mut self.camera.0,
            &ctx.previous.0,
            &self.movement,
            &ctx.inputs.0.camera,
            rules.camera.get(),
        )?;
        self.completed = self.completed.checked_add(1).ok_or(Fault::Order)?;
        Ok(Phase::Ready)
    }
}
