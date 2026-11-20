//! Compound support publication through its sole actual physical owner.

use super::movement_caller::Current;
use super::movement_support_contract::{Inputs, Stage};
use super::{Caller, Content, Dependency, Rules, State};
use crate::production::{ProductionFault, Unsupported, timing::WorkId};

const fn missing() -> ProductionFault {
    ProductionFault::Unsupported(Unsupported::Frame(
        Dependency::PlayerContinuationPreparation,
    ))
}

impl State {
    pub(super) fn begin_support(
        &mut self,
        content: &Content,
        rules: &Rules,
    ) -> Result<(), ProductionFault> {
        self.validate_room_owner(content)?;
        let inputs = self.ordinary_inputs()?;
        let Caller::PlayerContinuation {
            remaining,
            ordinary: Some(ordinary),
            support: Stage::Idle,
            charging: false,
            ..
        } = &self.caller
        else {
            return Err(missing());
        };
        if !ordinary.is_complete()
            || !ordinary.actors_hold(remaining.0.actors.as_ref().ok_or_else(missing)?)
        {
            return Err(missing());
        }
        let current = Current {
            appearance: &self.appearance,
            figure: self.playing_figure.0,
            actors: remaining.0.actors.as_ref().ok_or_else(missing)?,
            inputs: &inputs,
        };
        let room = content.movement_room.as_ref().ok_or_else(missing)?;
        let contract = content.compound_support.as_ref().ok_or_else(missing)?;
        let mut stage = Stage::Idle;
        stage
            .choose(
                &self.movement,
                &current,
                room,
                &content.movement,
                &rules.movement,
            )
            .map_err(|_| missing())?;
        let (profile, requests) = contract
            .physical(
                &stage,
                &self.movement,
                &Inputs {
                    current: &current,
                    room,
                    content: &content.movement,
                    rules: &rules.movement,
                },
            )
            .map_err(|_| missing())?;
        self.timing
            .begin(WorkId(1001), profile)
            .map_err(ProductionFault::FrameHardware)?;
        stage.bind(WorkId(1001), requests).map_err(|_| missing())?;
        let Caller::PlayerContinuation {
            support, charging, ..
        } = &mut self.caller
        else {
            return Err(ProductionFault::WrongScope);
        };
        *support = stage;
        *charging = true;
        Ok(())
    }

    pub(super) fn validate_support(
        &self,
        content: &Content,
        rules: &Rules,
    ) -> Result<(), ProductionFault> {
        self.validate_room_owner(content)?;
        let inputs = self.ordinary_inputs()?;
        let Caller::PlayerContinuation {
            remaining,
            ordinary: Some(ordinary),
            support: stage @ Stage::Charging { .. },
            charging: true,
            ..
        } = &self.caller
        else {
            return Err(missing());
        };
        if !ordinary.is_complete()
            || !ordinary.actors_hold(remaining.0.actors.as_ref().ok_or_else(missing)?)
        {
            return Err(missing());
        }
        let current = Current {
            appearance: &self.appearance,
            figure: self.playing_figure.0,
            actors: remaining.0.actors.as_ref().ok_or_else(missing)?,
            inputs: &inputs,
        };
        content
            .compound_support
            .as_ref()
            .ok_or_else(missing)?
            .physical(
                stage,
                &self.movement,
                &Inputs {
                    current: &current,
                    room: content.movement_room.as_ref().ok_or_else(missing)?,
                    content: &content.movement,
                    rules: &rules.movement,
                },
            )
            .map_err(|_| missing())?;
        Ok(())
    }

    pub(super) fn complete_support(
        &mut self,
        identity: WorkId,
        content: &Content,
        rules: &Rules,
    ) -> Result<(), ProductionFault> {
        self.validate_support(content, rules)?;
        let inputs = self.ordinary_inputs()?;
        let Caller::PlayerContinuation {
            remaining, support, ..
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
        let mut stage = support.clone();
        let completion = stage
            .complete(
                identity,
                &mut self.movement,
                &Inputs {
                    current: &current,
                    room: content.movement_room.as_ref().ok_or_else(missing)?,
                    content: &content.movement,
                    rules: &rules.movement,
                },
                content.compound_support.as_ref().ok_or_else(missing)?,
            )
            .map_err(|_| missing())?;
        self.world = Some(completion.receipt.world);
        let Caller::PlayerContinuation {
            support, charging, ..
        } = &mut self.caller
        else {
            return Err(ProductionFault::WrongScope);
        };
        *support = stage;
        *charging = false;
        Ok(())
    }
}
