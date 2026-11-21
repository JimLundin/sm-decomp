//! Actual local Run query, full physical hold and completion.

use super::borrowed_player_preparation::{CompletedInsideAndHudWeapons, CompletedPlayerSelection};
use super::movement_caller::{Current, Run, RunDependencies};
use super::{Caller, Content, Dependency, Rules, State};
use crate::production::{ProductionFault, Unsupported, timing::WorkId};

const fn missing() -> ProductionFault {
    ProductionFault::Unsupported(Unsupported::Frame(
        Dependency::PlayerContinuationPreparation,
    ))
}

impl State {
    pub(super) fn run_inputs(&self) -> Result<super::inputs::MovementInputs, ProductionFault> {
        let Caller::PlayerContinuation { completed, .. } = &self.caller else {
            return Err(ProductionFault::WrongScope);
        };
        let pose = completed.0.pose_input.as_ref().ok_or_else(missing)?;
        let environment = completed.0.environment.as_ref().ok_or_else(missing)?;
        let mut inputs = self.inside_inputs.as_ref().ok_or_else(missing)?.0;
        inputs.pad = pose.terminal.held_actions;
        inputs.pressed = pose.terminal.pressed_actions;
        inputs.items = environment.dependency.equipment;
        inputs.sprite_before_movement = self.playing_figure.0.sprite;
        Ok(inputs)
    }

    fn run_dependencies(&self) -> Result<RunDependencies<'_>, ProductionFault> {
        let Caller::PlayerContinuation {
            completed,
            controls,
            remaining,
            motion_reset,
            ..
        } = &self.caller
        else {
            return Err(ProductionFault::WrongScope);
        };
        Ok(RunDependencies {
            recurring: !self.publications.original_interval(),
            environment: completed.0.environment.as_ref().ok_or_else(missing)?,
            selection: CompletedPlayerSelection {
                inside_hud: CompletedInsideAndHudWeapons {
                    inside: self.inside_completed.as_ref().ok_or_else(missing)?,
                    controls,
                },
                motion_reset: motion_reset.as_ref().ok_or_else(missing)?,
            },
            clocks: &remaining
                .0
                .projectiles
                .as_ref()
                .ok_or_else(missing)?
                .dependency
                .clocks,
        })
    }

    pub(super) fn validate_run(
        &self,
        content: &Content,
        rules: &Rules,
    ) -> Result<(), ProductionFault> {
        if self.remaining_history()? != super::player_continuation::Stage::Run {
            return Err(missing());
        }
        let Caller::PlayerContinuation { run, .. } = &self.caller else {
            return Err(ProductionFault::WrongScope);
        };
        let query = run.query().ok_or_else(missing)?;
        if run.receipt().is_none_or(|receipt| {
            receipt.decisions.0.iter().any(|decision| {
                !matches!(decision, samus_movement::suspended::MovementDecision::Local { inputs, .. }
                    if inputs.rules == rules.movement)
            })
        }) {
            return Err(missing());
        }
        let mut inputs = self.run_inputs()?;
        inputs.animation_ends = query.inputs.0.animation_ends;
        inputs.spin_turned_far = query.inputs.0.spin_turned_far;
        let playing = self.playing_figure.0;
        if query.figure.0 != playing
            || query.appearance != self.appearance
            || inputs != query.inputs.0
        {
            return Err(missing());
        }
        content
            .run
            .as_ref()
            .ok_or_else(missing)?
            .physical(run, &self.movement, &self.run_dependencies()?)
            .map_err(|_| missing())?;
        Ok(())
    }

    pub(super) fn begin_run(
        &mut self,
        content: &Content,
        rules: &Rules,
    ) -> Result<(), ProductionFault> {
        if self.remaining_history()? != super::player_continuation::Stage::Run {
            return Err(missing());
        }
        let inputs = self.run_inputs()?;
        let Caller::PlayerContinuation {
            remaining,
            run: Run::Idle,
            charging: false,
            ..
        } = &self.caller
        else {
            return Err(missing());
        };
        let current = Current {
            appearance: &self.appearance,
            figure: self.playing_figure.0,
            actors: remaining.0.actors.as_ref().ok_or_else(missing)?,
            inputs: &inputs,
        };
        let mut run = Run::Idle;
        run.choose(
            &mut self.movement,
            &current,
            &content.appearance,
            &content.movement,
            &rules.movement,
        )
        .map_err(|_| missing())?;
        let profile = content
            .run
            .as_ref()
            .ok_or_else(missing)?
            .physical(&run, &self.movement, &self.run_dependencies()?)
            .map_err(|_| missing())?
            .bind(&[])
            .map_err(|_| missing())?;
        self.timing
            .begin(WorkId(23), profile)
            .map_err(ProductionFault::FrameHardware)?;
        run.bind(WorkId(23)).map_err(|_| missing())?;
        let Caller::PlayerContinuation {
            run: saved,
            charging,
            ..
        } = &mut self.caller
        else {
            return Err(ProductionFault::WrongScope);
        };
        *saved = run;
        *charging = true;
        Ok(())
    }

    pub(super) fn complete_run(
        &mut self,
        identity: WorkId,
        content: &Content,
        rules: &Rules,
    ) -> Result<(), ProductionFault> {
        self.validate_run(content, rules)?;
        let inputs = self.run_inputs()?;
        let Caller::PlayerContinuation {
            remaining,
            run,
            charging: true,
            ..
        } = &self.caller
        else {
            return Err(missing());
        };
        let current = Current {
            appearance: &self.appearance,
            figure: self.playing_figure.0,
            actors: remaining.0.actors.as_ref().ok_or_else(missing)?,
            inputs: &inputs,
        };
        let mut run = run.clone();
        let completion = run
            .complete(
                identity,
                &mut self.movement,
                &current,
                &content.movement,
                &rules.movement,
            )
            .map_err(|_| missing())?;
        if let Some(history) = &mut self.liquid_history {
            history.run(&completion).map_err(|_| {
                ProductionFault::Unsupported(Unsupported::Frame(Dependency::AnimationLiquidHistory))
            })?;
        }
        self.movement_inputs = Some(completion.ahead.inputs);
        let Caller::PlayerContinuation {
            run: saved,
            charging,
            ..
        } = &mut self.caller
        else {
            return Err(ProductionFault::WrongScope);
        };
        *saved = run;
        *charging = false;
        Ok(())
    }
}
