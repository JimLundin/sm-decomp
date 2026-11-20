//! Retained downward travel through its complete physical owner.

use super::movement_caller::Current;
use super::movement_down_contract::{Inputs, Stage};
use super::{Caller, Content, Dependency, Rules, State};
use crate::production::{ProductionFault, Unsupported, timing::WorkId};

const fn missing() -> ProductionFault {
    ProductionFault::Unsupported(Unsupported::Frame(
        Dependency::PlayerContinuationPreparation,
    ))
}

impl State {
    pub(super) fn begin_down(
        &mut self,
        content: &Content,
        rules: &Rules,
    ) -> Result<(), ProductionFault> {
        let inside = self.validate_inside_provenance(content)?.clone();
        let inputs = self.ordinary_inputs()?;
        let Caller::PlayerContinuation {
            remaining,
            ground: super::movement_ground_contract::Stage::Complete(ground),
            down: Stage::Idle,
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
            ground,
        };
        let contract = content.ground_following_down.as_ref().ok_or_else(missing)?;
        let mut stage = Stage::Idle;
        stage
            .choose(&self.movement, &inputs)
            .map_err(|_| missing())?;
        let (profile, requests) = contract
            .physical(&stage, &self.movement, &inputs)
            .map_err(|_| missing())?;
        self.timing
            .begin(WorkId(1004), profile)
            .map_err(ProductionFault::FrameHardware)?;
        stage.bind(WorkId(1004), requests).map_err(|_| missing())?;
        let Caller::PlayerContinuation { down, charging, .. } = &mut self.caller else {
            return Err(ProductionFault::WrongScope);
        };
        *down = stage;
        *charging = true;
        Ok(())
    }

    pub(super) fn validate_down(
        &self,
        content: &Content,
        rules: &Rules,
    ) -> Result<(), ProductionFault> {
        let inside = self.validate_inside_provenance(content)?.clone();
        let inputs = self.ordinary_inputs()?;
        let Caller::PlayerContinuation {
            remaining,
            ground: super::movement_ground_contract::Stage::Complete(ground),
            down: down @ Stage::Charging { .. },
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
            .ground_following_down
            .as_ref()
            .ok_or_else(missing)?
            .physical(
                down,
                &self.movement,
                &Inputs {
                    inside: &inside,
                    current: &current,
                    room: content.movement_room.as_ref().ok_or_else(missing)?,
                    content: &content.movement,
                    rules: &rules.movement,
                    ground,
                },
            )
            .map_err(|_| missing())?;
        Ok(())
    }

    pub(super) fn complete_down(
        &mut self,
        identity: WorkId,
        content: &Content,
        rules: &Rules,
    ) -> Result<(), ProductionFault> {
        self.validate_down(content, rules)?;
        let inside = self.validate_inside_provenance(content)?.clone();
        let inputs = self.ordinary_inputs()?;
        let Caller::PlayerContinuation {
            remaining,
            ground: super::movement_ground_contract::Stage::Complete(ground),
            down,
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
        let mut stage = down.clone();
        let completion = stage
            .complete(
                identity,
                &mut self.movement,
                &Inputs {
                    inside: &inside,
                    current: &current,
                    room: content.movement_room.as_ref().ok_or_else(missing)?,
                    content: &content.movement,
                    rules: &rules.movement,
                    ground,
                },
                content.ground_following_down.as_ref().ok_or_else(missing)?,
            )
            .map_err(|_| missing())?;
        self.world = Some(completion.receipt.world);
        let Caller::PlayerContinuation { down, charging, .. } = &mut self.caller else {
            return Err(ProductionFault::WrongScope);
        };
        *down = stage;
        *charging = false;
        Ok(())
    }
}
