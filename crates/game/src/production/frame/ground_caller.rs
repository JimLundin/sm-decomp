//! Retained ground pull through its complete physical owner.

use super::movement_caller::{Current, Run};
use super::movement_ground_contract::{Inputs, Stage};
use super::{Caller, Content, Dependency, Rules, State};
use crate::production::{ProductionFault, Unsupported, timing::WorkId};

const fn missing() -> ProductionFault {
    ProductionFault::Unsupported(Unsupported::Frame(
        Dependency::PlayerContinuationPreparation,
    ))
}

impl State {
    pub(super) fn begin_ground(
        &mut self,
        content: &Content,
        rules: &Rules,
    ) -> Result<(), ProductionFault> {
        let inside = self.validate_inside_provenance(content)?.clone();
        let inputs = self.ordinary_inputs()?;
        let Caller::PlayerContinuation {
            remaining,
            run: Run::Complete(run),
            support: super::movement_support_contract::Stage::Complete(support),
            ground: Stage::Idle,
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
        let inputs = Inputs {
            inside: &inside,
            current: &current,
            room: content.movement_room.as_ref().ok_or_else(missing)?,
            content: &content.movement,
            rules: &rules.movement,
            run,
            support,
        };
        let contract = content.ground_pull.as_ref().ok_or_else(missing)?;
        let mut stage = Stage::Idle;
        stage
            .choose(&self.movement, &inputs)
            .map_err(|_| missing())?;
        let profile = contract
            .physical(&stage, &self.movement, &inputs)
            .map_err(|_| missing())?;
        let work = super::TravelWork::Ground.identity();
        self.timing
            .begin(work, profile)
            .map_err(ProductionFault::FrameHardware)?;
        stage.bind(work).map_err(|_| missing())?;
        let Caller::PlayerContinuation {
            ground, charging, ..
        } = &mut self.caller
        else {
            return Err(ProductionFault::WrongScope);
        };
        *ground = stage;
        *charging = true;
        Ok(())
    }

    pub(super) fn ground_profile(
        &self,
        content: &Content,
        rules: &Rules,
    ) -> Result<data::timing::Profile, ProductionFault> {
        let inside = self.validate_inside_provenance(content)?.clone();
        let inputs = self.ordinary_inputs()?;
        let Caller::PlayerContinuation {
            remaining,
            run: Run::Complete(run),
            support: super::movement_support_contract::Stage::Complete(support),
            ground: ground @ Stage::Charging(_),
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
        content
            .ground_pull
            .as_ref()
            .ok_or_else(missing)?
            .physical(
                ground,
                &self.movement,
                &Inputs {
                    inside: &inside,
                    current: &current,
                    room: content.movement_room.as_ref().ok_or_else(missing)?,
                    content: &content.movement,
                    rules: &rules.movement,
                    run,
                    support,
                },
            )
            .map_err(|_| missing())
    }

    pub(super) fn complete_ground(
        &mut self,
        identity: WorkId,
        content: &Content,
        rules: &Rules,
    ) -> Result<(), ProductionFault> {
        self.ground_profile(content, rules)?;
        let inside = self.validate_inside_provenance(content)?.clone();
        let inputs = self.ordinary_inputs()?;
        let Caller::PlayerContinuation {
            remaining,
            run: Run::Complete(run),
            support: super::movement_support_contract::Stage::Complete(support),
            ground,
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
        let mut stage = ground.clone();
        stage
            .complete(
                identity,
                &mut self.movement,
                &Inputs {
                    inside: &inside,
                    current: &current,
                    room: content.movement_room.as_ref().ok_or_else(missing)?,
                    content: &content.movement,
                    rules: &rules.movement,
                    run,
                    support,
                },
                content.ground_pull.as_ref().ok_or_else(missing)?,
            )
            .map_err(|_| missing())?;
        let Caller::PlayerContinuation {
            ground, charging, ..
        } = &mut self.caller
        else {
            return Err(ProductionFault::WrongScope);
        };
        *ground = stage;
        *charging = false;
        Ok(())
    }
}
