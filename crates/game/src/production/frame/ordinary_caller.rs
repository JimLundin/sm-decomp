//! Actual ordinary travel with full current-owner and live-product guards.

use super::movement_caller::{Current, Pair, PairOwner, Run};
use super::movement_pair_contract::{CompletionInputs, Stage};
use super::movement_support_contract::ActorCompletionPin;
use super::{Caller, Content, Dependency, Rules, State};
use crate::production::{ProductionFault, Unsupported, timing::WorkId};
use canonical::{Canonical, model::Model};

/// One actual ordinary operation and its immutable completed actor dependency.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct OrdinaryWork {
    stage: Box<Stage>,
    actors: Model<ActorCompletionPin>,
}

impl OrdinaryWork {
    pub(super) fn actors_hold(
        &self,
        actors: &super::borrowed_player_preparation::ActorUpdateCompletion,
    ) -> bool {
        &self.actors.0 == actors
    }

    /// Its complete physical work remains active.
    #[must_use]
    pub const fn is_charging(&self) -> bool {
        matches!(*self.stage, Stage::Charging { .. })
    }

    /// Both actual product bytes and the retained answer were consumed once.
    #[must_use]
    pub const fn is_complete(&self) -> bool {
        matches!(*self.stage, Stage::Complete(_))
    }

    pub(super) fn sample(
        &mut self,
        work: WorkId,
        sample: data::timing::Sample,
        byte: u8,
    ) -> Result<(), ProductionFault> {
        self.stage.sample(work, sample, byte).map_err(|_| missing())
    }
}

const fn missing() -> ProductionFault {
    ProductionFault::Unsupported(Unsupported::Frame(
        Dependency::PlayerContinuationPreparation,
    ))
}

impl State {
    pub(super) fn ordinary_inputs(&self) -> Result<super::inputs::MovementInputs, ProductionFault> {
        if self.remaining_history()? != super::player_continuation::Stage::Run {
            return Err(missing());
        }
        let Caller::PlayerContinuation {
            run: Run::Complete(completion),
            ..
        } = &self.caller
        else {
            return Err(missing());
        };
        let mut current = self.run_inputs()?;
        current.animation_ends = completion.ahead.inputs.0.animation_ends;
        current.spin_turned_far = completion.ahead.inputs.0.spin_turned_far;
        if current != completion.ahead.inputs.0
            || self.movement_inputs.as_ref() != Some(&completion.ahead.inputs)
            || self.appearance != completion.ahead.appearance
            || self.playing_figure.0 != completion.ahead.figure.0
        {
            return Err(missing());
        }
        Ok(current)
    }

    pub(super) fn begin_ordinary(
        &mut self,
        content: &Content,
        rules: &Rules,
    ) -> Result<(), ProductionFault> {
        self.validate_room_owner(content)?;
        let inputs = self.ordinary_inputs()?;
        let room = content.movement_room.as_ref().ok_or_else(missing)?;
        let contract = content.ordinary_right.as_ref().ok_or_else(missing)?;
        let Caller::PlayerContinuation {
            remaining,
            ordinary: None,
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
        let mut pair = Pair::Idle;
        pair.choose(
            PairOwner::OrdinaryRight,
            &mut self.movement,
            &current,
            room,
            &content.movement,
            &rules.movement,
        )
        .map_err(|_| missing())?;
        let (profile, requests) = contract
            .physical(
                pair.receipt().ok_or_else(missing)?,
                &self.movement,
                room,
                current.actors,
                &content.movement,
            )
            .map_err(|_| missing())?;
        let actors = Model(current.actors.clone());
        let mut stage = Stage::offered(pair, &rules.movement).map_err(|_| missing())?;
        self.timing
            .begin(WorkId(1000), profile)
            .map_err(ProductionFault::FrameHardware)?;
        stage.bind(WorkId(1000), requests).map_err(|_| missing())?;
        let Caller::PlayerContinuation {
            ordinary, charging, ..
        } = &mut self.caller
        else {
            return Err(ProductionFault::WrongScope);
        };
        *ordinary = Some(OrdinaryWork {
            stage: Box::new(stage),
            actors,
        });
        *charging = true;
        Ok(())
    }

    pub(super) fn validate_ordinary(
        &self,
        content: &Content,
        rules: &Rules,
    ) -> Result<(), ProductionFault> {
        self.validate_room_owner(content)?;
        let inputs = self.ordinary_inputs()?;
        let Caller::PlayerContinuation {
            remaining,
            ordinary: Some(ordinary),
            charging: true,
            ..
        } = &self.caller
        else {
            return Err(missing());
        };
        let actors = remaining.0.actors.as_ref().ok_or_else(missing)?;
        if !ordinary.is_charging()
            || actors != &ordinary.actors.0
            || ordinary
                .stage
                .receipt()
                .is_none_or(|receipt| receipt.inputs.0 != inputs)
        {
            return Err(missing());
        }
        ordinary
            .stage
            .validate_rules(&rules.movement)
            .map_err(|_| missing())?;
        content
            .ordinary_right
            .as_ref()
            .ok_or_else(missing)?
            .physical(
                ordinary.stage.receipt().ok_or_else(missing)?,
                &self.movement,
                content.movement_room.as_ref().ok_or_else(missing)?,
                actors,
                &content.movement,
            )
            .map_err(|_| missing())?;
        Ok(())
    }

    pub(super) fn complete_ordinary(
        &mut self,
        identity: WorkId,
        content: &Content,
        rules: &Rules,
    ) -> Result<(), ProductionFault> {
        self.validate_ordinary(content, rules)?;
        let inputs = self.ordinary_inputs()?;
        let Caller::PlayerContinuation {
            remaining,
            ordinary: Some(ordinary),
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
        let mut stage = ordinary.stage.clone();
        let receipt = stage
            .complete(
                identity,
                &mut self.movement,
                &CompletionInputs {
                    current: &current,
                    room: content.movement_room.as_ref().ok_or_else(missing)?,
                    content: &content.movement,
                    rules: &rules.movement,
                    contract: content.ordinary_right.as_ref().ok_or_else(missing)?,
                },
            )
            .map_err(|_| missing())?;
        self.world = Some(receipt.world);
        let Caller::PlayerContinuation {
            ordinary: Some(ordinary),
            charging,
            ..
        } = &mut self.caller
        else {
            return Err(ProductionFault::WrongScope);
        };
        ordinary.stage = stage;
        *charging = false;
        Ok(())
    }
}
