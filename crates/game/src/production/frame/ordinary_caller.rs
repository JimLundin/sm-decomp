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

    /// Actual request and accepted answer after arithmetic and travel complete.
    #[must_use]
    pub const fn completion(&self) -> Option<&super::movement_caller::PairReceipt> {
        match &*self.stage {
            Stage::Complete(receipt) => Some(receipt),
            Stage::Offered { .. } | Stage::Charging { .. } => None,
        }
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

/// Physical work selected after the ordinary Run returns and before downward travel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TravelWork {
    /// Ordinary rightward movement.
    OrdinaryRight,
    /// Both support checks in one physical request.
    Support,
    /// The ground-following pull.
    Ground,
}

impl TravelWork {
    /// The timing owner retains this identity until the whole request returns.
    #[must_use]
    pub const fn identity(self) -> WorkId {
        match self {
            Self::OrdinaryRight => PairOwner::OrdinaryRight.identity(),
            Self::Support => WorkId(1001),
            Self::Ground => WorkId(1003),
        }
    }
}

impl Caller {
    pub(super) fn charging_travel(&self) -> Option<TravelWork> {
        let Self::PlayerContinuation {
            ordinary,
            support,
            ground,
            charging: true,
            ..
        } = self
        else {
            return None;
        };
        if matches!(ground, super::movement_ground_contract::Stage::Charging(_)) {
            Some(TravelWork::Ground)
        } else if matches!(
            support,
            super::movement_support_contract::Stage::Charging { .. }
        ) {
            Some(TravelWork::Support)
        } else {
            ordinary
                .as_ref()
                .is_some_and(OrdinaryWork::is_charging)
                .then_some(TravelWork::OrdinaryRight)
        }
    }
}

impl State {
    pub(super) fn validate_charging_travel(
        &self,
        content: &Content,
        rules: &Rules,
    ) -> Result<bool, ProductionFault> {
        let Some(work) = self.caller.charging_travel() else {
            return Ok(false);
        };
        let profile = match work {
            TravelWork::OrdinaryRight => self.ordinary_profile(content, rules)?,
            TravelWork::Support => self.support_profile(content, rules)?,
            TravelWork::Ground => self.ground_profile(content, rules)?,
        };
        if !self.has_physical_bill(work.identity(), &profile) {
            return Err(missing());
        }
        Ok(true)
    }

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
        let work = TravelWork::OrdinaryRight.identity();
        self.timing
            .begin(work, profile)
            .map_err(ProductionFault::FrameHardware)?;
        stage.bind(work, requests).map_err(|_| missing())?;
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

    fn ordinary_profile(
        &self,
        content: &Content,
        rules: &Rules,
    ) -> Result<data::timing::Profile, ProductionFault> {
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
        let (profile, _) = content
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
        Ok(profile)
    }

    pub(super) fn complete_ordinary(
        &mut self,
        identity: WorkId,
        content: &Content,
        rules: &Rules,
    ) -> Result<(), ProductionFault> {
        self.ordinary_profile(content, rules)?;
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
