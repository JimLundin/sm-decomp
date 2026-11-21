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
    pub(crate) fn validate_grounded_checkpoint(
        &self,
        content: &Content,
        rules: &Rules,
    ) -> Result<(), super::Fault> {
        use super::post_down_caller::Stage as Post;
        let Caller::PlayerContinuation {
            down, post_down, ..
        } = &self.caller
        else {
            return Ok(());
        };
        let result = if matches!(down, Stage::Charging { .. }) {
            self.validate_down(content, rules)
        } else if matches!(down, Stage::Complete(_)) {
            self.validate_completed_down(content, rules).and_then(|()| {
                if matches!(post_down, Post::Contact { .. }) {
                    self.validate_stride_contact(content)
                } else if matches!(post_down, Post::Poststep { .. }) {
                    self.validate_poststep(content)
                } else {
                    Ok(())
                }
            })
        } else {
            Ok(())
        };
        result.map_err(|_| super::Fault::InvalidPlayerEntry)
    }

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
            down: down @ Stage::Charging { pending, .. },
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
        let (profile, requests) = content
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
        if pending.requests() != requests || !self.has_physical_bill(WorkId(1004), &profile) {
            return Err(missing());
        }
        Ok(())
    }

    pub(super) fn validate_completed_down(
        &self,
        content: &Content,
        rules: &Rules,
    ) -> Result<(), ProductionFault> {
        let inside = self.validate_inside_provenance(content)?;
        let Caller::PlayerContinuation {
            remaining,
            ground: super::movement_ground_contract::Stage::Complete(ground),
            down: Stage::Complete(completed),
            run: super::movement_caller::Run::Complete(run),
            post_down,
            ..
        } = &self.caller
        else {
            return Err(missing());
        };
        let (terminal, appearance, playing) = post_down.down_history()?.map_or(
            (&self.movement, &self.appearance, self.playing_figure),
            |(movement, contact)| (movement, &contact.before, contact.playing),
        );
        if appearance != &run.ahead.appearance
            || playing != run.ahead.figure
            || self.movement_inputs.as_ref() != Some(&run.ahead.inputs)
        {
            return Err(missing());
        }
        let current = Current {
            appearance,
            figure: playing.0,
            actors: remaining.0.actors.as_ref().ok_or_else(missing)?,
            inputs: &run.ahead.inputs.0,
        };
        completed
            .validate(
                terminal,
                &Inputs {
                    inside,
                    current: &current,
                    room: content.movement_room.as_ref().ok_or_else(missing)?,
                    content: &content.movement,
                    rules: &rules.movement,
                    ground,
                },
                content.ground_following_down.as_ref().ok_or_else(missing)?,
            )
            .map_err(|_| missing())
    }

    pub(super) fn complete_down(
        &mut self,
        identity: WorkId,
        content: &Content,
        rules: &Rules,
    ) -> Result<(), ProductionFault> {
        let inside = self.validate_inside_provenance(content)?.clone();
        let inputs = self.ordinary_inputs()?;
        let Caller::PlayerContinuation {
            remaining,
            ground: super::movement_ground_contract::Stage::Complete(ground),
            down,
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
