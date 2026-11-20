//! Retained room and no-upload preparation before actual game-time maintenance.

use super::{Content, Dependency, State, suffix_caller};
use crate::production::{ProductionFault, Unsupported, timing};
use canonical::Canonical;
use data::timing::Profile;
use types::Parts;

pub(super) const BLOCKS: timing::WorkId = timing::WorkId(1422);
pub(super) const ROOM: timing::WorkId = timing::WorkId(1423);

/// Physical envelopes for the reached room preparation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Contract {
    /// Complete actual block preparation and no-upload work.
    pub blocks: Profile,
    /// Actual current same-room `NoAction` envelope.
    pub room: Profile,
}

/// Origin-backed current room and carried layer facts.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Admission {
    /// Actual room qualified for the dormant room-main call.
    pub room: data::id::RoomId,
    /// Live carried comparison blocks for current layer one.
    pub previous_layer_one: [u16; 2],
    /// Separate reached external layer-two ownership dispositions.
    pub external_layer_two: [bool; 2],
}

/// Eight actual prepared/current block coordinates, in caller order.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Blocks([u16; 8]);

/// Canonical room work at the precise pre-clock boundaries.
#[derive(Canonical, Clone, Debug, Default, PartialEq, Eq)]
pub enum Stage {
    /// Block preparation is not yet selected.
    #[default]
    Pending,
    /// Actual block coordinates are pending physical publication.
    Preparing(Blocks),
    /// Actual block coordinates returned.
    RoomReady(Blocks),
    /// Current room has offered its actual quiet action.
    RoomCharging(Blocks),
    /// Room completed; game-time handling is the next actual caller.
    Completed(Blocks),
}

const fn missing() -> ProductionFault {
    ProductionFault::Unsupported(Unsupported::Frame(Dependency::RoomPreparation))
}

impl State {
    fn current_blocks(&self, content: &Content) -> Result<Blocks, ProductionFault> {
        let history = self.room_tail_origin.as_ref().ok_or_else(missing)?;
        let suffix_caller::BackgroundScroll::Complete(scroll) = &self.background else {
            return Err(missing());
        };
        let initial = &content.return_work.as_ref().ok_or_else(missing)?.initial;
        if self.validate_room_owner(content).map_err(|_| missing())? != history.room
            || !history
                .external_layer_two
                .into_iter()
                .all(|external| external)
        {
            return Err(missing());
        }
        let view = self.camera.0.position();
        let current = [view.x.pixel().0 >> 4, view.y.pixel().0 >> 4];
        if current != history.previous_layer_one {
            return Err(missing());
        }
        Ok(Blocks([
            scroll.one_horizontal >> 4,
            initial.background_two_horizontal >> 4,
            current[0],
            initial.background_two_horizontal >> 4,
            scroll.one_vertical >> 4,
            scroll.two_vertical >> 4,
            current[1],
            scroll.layer_two_vertical >> 4,
        ]))
    }

    pub(super) fn advance_room_tail(&mut self, content: &Content) -> Result<(), ProductionFault> {
        let contract = content.room_tail.as_ref().ok_or_else(missing)?;
        let actual = self.current_blocks(content)?;
        let (work, profile, next) = match self.room_tail {
            Stage::Pending => (BLOCKS, &contract.blocks, Stage::Preparing(actual)),
            Stage::RoomReady(before) if before == actual => {
                (ROOM, &contract.room, Stage::RoomCharging(actual))
            }
            Stage::Preparing(before) | Stage::RoomCharging(before) if before == actual => {
                let (work, profile) = if matches!(self.room_tail, Stage::Preparing(_)) {
                    (BLOCKS, &contract.blocks)
                } else {
                    (ROOM, &contract.room)
                };
                if self.timing.work.iter().any(|candidate| {
                    candidate.kind == timing::WorkKind::Semantic(work)
                        && &candidate.profile == profile
                }) {
                    return Ok(());
                }
                return Err(missing());
            }
            Stage::Preparing(_)
            | Stage::RoomReady(_)
            | Stage::RoomCharging(_)
            | Stage::Completed(_) => return Err(missing()),
        };
        self.timing
            .begin(work, profile.clone())
            .map_err(ProductionFault::FrameHardware)?;
        self.room_tail = next;
        Ok(())
    }

    pub(super) fn complete_room_tail(
        &mut self,
        work: timing::WorkId,
        content: &Content,
    ) -> Result<(), ProductionFault> {
        let actual = self.current_blocks(content)?;
        self.room_tail = match (work, &self.room_tail) {
            (BLOCKS, Stage::Preparing(held)) if *held == actual => Stage::RoomReady(actual),
            (ROOM, Stage::RoomCharging(held)) if *held == actual => Stage::Completed(actual),
            _ => return Err(missing()),
        };
        Ok(())
    }
}
