//! Sealed early Inside capability over actual immutable room blocks.

use super::{movement::Movement, movement_stage};
use crate::production::{
    MovementContent,
    timing::WorkId,
    world::{Boundary, WorldFault},
};
use core::cell::Cell as FaultCell;
use samus_movement::suspended::{MovementDecision, SuspendedMovement};
use types::{Block, Cell, Hitbox, Way, World};

#[cfg(test)]
mod tests;

/// Current immutable room geometry usable only by the first Inside operation.
/// Actor membership and reaction permissions are unavailable at this opportunity.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct InsideWorld {
    identity: Option<RoomIdentity>,
    width: u16,
    cells: Vec<Block>,
    boundary: Boundary,
}

/// Checkpoint representation of a checked loaded-content room ID, never a native pointer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
struct RoomIdentity(u16);

/// Canonical owner of complete current immutable room blocks before actor updating.
pub struct InsideWorldPin;
impl canonical::model::Pinned for InsideWorldPin {
    type State = InsideWorld;
    const NAME: &'static str = "production-frame-inside-world";
    const SHAPE: u64 = 0x616C_8A27_AD5E_116C;
}

#[cfg(test)]
#[test]
fn current_inside_world_shape_remains_pinned() {
    assert_eq!(canonical::model::pin_holds::<InsideWorldPin>(), Ok(()));
}

/// An Inside capability cannot service a later movement opportunity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// The actual continuation is not the first Inside request or its retained answer.
    Opportunity,
    /// The actual selected physical operation rejects its sequence or completion.
    Stage(movement_stage::Fault),
    /// A candidate attempted an unavailable reaction-permission query.
    Reaction,
    /// A candidate attempted an unavailable current actor-roster query.
    Actors,
}

impl InsideWorld {
    /// Admit current complete room blocks and explicit outside geometry.
    /// # Errors
    /// Invalid dimensions, stand-in loops or stand-ins outside this room.
    pub fn new(width: u16, cells: Vec<Block>, boundary: Boundary) -> Result<Self, WorldFault> {
        if width == 0
            || cells.is_empty()
            || !cells.len().is_multiple_of(usize::from(width))
            || cells
                .len()
                .checked_div(usize::from(width))
                .is_none_or(|height| height > usize::from(u16::MAX))
        {
            return Err(WorldFault::Shape);
        }
        let world = Self {
            identity: None,
            width,
            cells,
            boundary,
        };
        for (index, _) in world.cells.iter().enumerate() {
            let start = Cell {
                x: u16::try_from(
                    index
                        .checked_rem(usize::from(width))
                        .ok_or(WorldFault::Shape)?,
                )
                .map_err(|_| WorldFault::Shape)?,
                y: u16::try_from(
                    index
                        .checked_div(usize::from(width))
                        .ok_or(WorldFault::Shape)?,
                )
                .map_err(|_| WorldFault::Shape)?,
            };
            let mut at = start;
            for visited in 0..=world.cells.len() {
                let index = world.index(at).ok_or(WorldFault::StandInOutside(at))?;
                if visited == world.cells.len() {
                    return Err(WorldFault::StandInLoop(start));
                }
                match world
                    .cells
                    .get(index)
                    .copied()
                    .ok_or(WorldFault::StandInOutside(at))?
                {
                    Block::StandsFor(Some(next)) => at = next,
                    Block::Air
                    | Block::Solid
                    | Block::Answered
                    | Block::Slope(_)
                    | Block::Stepped(_)
                    | Block::StandsFor(None) => break,
                }
            }
        }
        Ok(world)
    }

    /// Admit actual room geometry under its checked loaded-content identity.
    /// The importer supplies the ID associated with the same decoded room header.
    /// # Errors
    /// Invalid dimensions or stand-in relations.
    pub fn for_room(
        identity: data::id::RoomId,
        width: u16,
        cells: Vec<Block>,
        boundary: Boundary,
    ) -> Result<Self, WorldFault> {
        let mut world = Self::new(width, cells, boundary)?;
        world.identity = Some(RoomIdentity(identity.0));
        Ok(world)
    }

    /// Actual loaded-content room backing these complete current blocks.
    #[must_use]
    pub const fn room_identity(&self) -> Option<data::id::RoomId> {
        match self.identity {
            Some(identity) => Some(data::id::RoomId(identity.0)),
            None => None,
        }
    }

    /// Join this actual immutable block owner to its checked movement-room content.
    /// Unbound geometry cannot claim integrated room ownership.
    #[must_use]
    pub fn matches_room(&self, room: &super::movement_caller::Room) -> bool {
        self.room_identity().is_some()
            && self.room_identity() == room.identity
            && self.width == room.width
            && self.cells == room.cells
            && self.boundary == room.boundary
    }

    /// Recheck complete canonical room geometry after restoration.
    /// # Errors
    /// Invalid dimensions or stand-in relations; no external producer is queried.
    pub fn validate(&self) -> Result<(), WorldFault> {
        Self::new(self.width, self.cells.clone(), self.boundary).map(|_| ())
    }

    /// Actual current room width in 16-pixel blocks.
    #[must_use]
    pub const fn width(&self) -> u16 {
        self.width
    }

    /// Choose only the genuine first Inside request and its actual retained block answer.
    /// The private world is never exposed for motion, support or actor queries.
    /// # Errors
    /// Later movement work, absent current work or invalid physical scheduling.
    pub fn choose<'a>(
        &self,
        stage: &'a mut movement_stage::Stage,
        movement: &mut Movement,
        content: &MovementContent,
        rules: &samus_movement::Rules,
        frame: &samus_movement::Frame<'_>,
    ) -> Result<&'a movement_stage::Receipt, Fault> {
        let Movement::Work(work) = movement else {
            return Err(Fault::Opportunity);
        };
        if !work.completed.0.is_empty() {
            return Err(Fault::Opportunity);
        }
        match work.model.0.offered.as_ref() {
            Some(MovementDecision::InsideRequest(_)) => {}
            None if SuspendedMovement::new(work.model.0.start)
                .is_ok_and(|fresh| fresh == *work.model.0) => {}
            None | Some(_) => return Err(Fault::Opportunity),
        }
        self.transaction(stage, movement, |stage, movement, blocks| {
            stage
                .choose(
                    movement_stage::Group::Inside,
                    movement,
                    &movement_stage::Inputs {
                        content,
                        rules,
                        frame,
                        world: blocks,
                    },
                )
                .map(|_| ())
                .map_err(Fault::Stage)
        })?;
        match stage {
            movement_stage::Stage::Offered(receipt) => Ok(receipt),
            movement_stage::Stage::Idle | movement_stage::Stage::Charging { .. } => {
                Err(Fault::Opportunity)
            }
        }
    }

    /// Consume only the retained Inside answer at its matching whole physical completion.
    /// # Errors
    /// Any motion/support/actor opportunity or changed physical identity/retained answer.
    pub fn complete(
        &self,
        stage: &mut movement_stage::Stage,
        completed: WorkId,
        movement: &mut Movement,
        content: &MovementContent,
        rules: &samus_movement::Rules,
        frame: &samus_movement::Frame<'_>,
    ) -> Result<movement_stage::Completion, Fault> {
        let movement_stage::Stage::Charging { receipt, .. } = stage else {
            return Err(Fault::Opportunity);
        };
        let Movement::Work(work) = movement else {
            return Err(Fault::Opportunity);
        };
        if receipt.group != movement_stage::Group::Inside
            || !matches!(
                work.completed.0.as_slice(),
                [MovementDecision::InsideRequest(_)]
            )
            || !matches!(
                work.model.0.offered,
                Some(MovementDecision::InsideAnswer(_))
            )
        {
            return Err(Fault::Opportunity);
        }
        self.transaction(stage, movement, |stage, movement, blocks| {
            stage
                .complete(
                    completed,
                    movement,
                    &movement_stage::Inputs {
                        content,
                        rules,
                        frame,
                        world: blocks,
                    },
                )
                .map_err(Fault::Stage)
        })
    }

    // Every candidate path checks unavailable queries before publishing either owner.
    fn transaction<R>(
        &self,
        stage: &mut movement_stage::Stage,
        movement: &mut Movement,
        operation: impl FnOnce(
            &mut movement_stage::Stage,
            &mut Movement,
            &Blocks<'_>,
        ) -> Result<R, Fault>,
    ) -> Result<R, Fault> {
        let mut next_stage = stage.clone();
        let mut next_movement = movement.clone();
        let blocks = Blocks {
            room: self,
            fault: FaultCell::new(None),
        };
        let result = operation(&mut next_stage, &mut next_movement, &blocks);
        if let Some(fault) = blocks.fault.get() {
            return Err(fault);
        }
        let result = result?;
        *stage = next_stage;
        *movement = next_movement;
        Ok(result)
    }

    pub(super) fn block_type(&self, at: Cell) -> Block {
        self.index(at)
            .and_then(|index| self.cells.get(index))
            .copied()
            .unwrap_or(match self.boundary {
                Boundary::Air => Block::Air,
                Boundary::Solid => Block::Solid,
            })
    }

    fn index(&self, at: Cell) -> Option<usize> {
        if at.x >= self.width {
            return None;
        }
        let index = usize::from(at.y)
            .checked_mul(usize::from(self.width))?
            .checked_add(usize::from(at.x))?;
        (index < self.cells.len()).then_some(index)
    }
}

// Only fixed Inside choice/completion can borrow this implementation.
struct Blocks<'a> {
    room: &'a InsideWorld,
    fault: FaultCell<Option<Fault>>,
}
impl World for Blocks<'_> {
    fn block(&self, at: Cell) -> Block {
        self.room.block_type(at)
    }
    fn lets_through(&self, _: Cell, _: Way) -> bool {
        self.fault.set(self.fault.get().or(Some(Fault::Reaction)));
        false
    }
    fn solid_enemies(&self) -> &[Hitbox] {
        self.fault.set(self.fault.get().or(Some(Fault::Actors)));
        &[]
    }
}
