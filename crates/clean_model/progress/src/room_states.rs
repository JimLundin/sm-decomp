//! Which version of a room loads, given the run's progress. Spec: RoomStates.

use std::collections::BTreeSet;

use clean_types::{Area, BossKind, Entrance, Event, Item};

use crate::progress::Progress;
use crate::supplies::Supply;

/// The condition a version of a room loads on. Spec: RoomStates.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Condition {
    /// Always holds.
    Always,
    /// Holds once the event has happened.
    Happened(Event),
    /// Holds when a boss of any of these kinds is defeated in the room's own area.
    Defeated(BTreeSet<BossKind>),
    /// Holds when the item is hers, switched on or not.
    Has(Item),
    /// Holds when the morph ball is hers and her missile capacity isn't 0, however many
    /// missiles she holds.
    MorphBallAndMissiles,
    /// Holds when her power bomb capacity isn't 0, however many power bombs she holds.
    PowerBombs,
    /// Holds when she came into the room by this entrance.
    EnteredBy(Entrance),
}

/// The room she loads into: its area, and the entrance she came in by, if any.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoomEntry {
    /// The room's own area.
    pub area: Area,
    /// The way she came into the room, if she came in by one.
    pub entered_by: Option<Entrance>,
}

impl Condition {
    /// Whether the condition holds for `progress`, for the room she loads into.
    #[must_use]
    fn holds(&self, progress: &Progress, room: RoomEntry) -> bool {
        let inventory = &progress.inventory;
        match self {
            Self::Always => true,
            Self::Happened(event) => progress.world.has_happened(*event),
            Self::Defeated(kinds) => progress.world.is_any_defeated(room.area, kinds),
            Self::Has(item) => inventory.items.is_hers(*item),
            Self::MorphBallAndMissiles => {
                inventory.items.is_hers(Item::MorphBall)
                    && !inventory.capacity.get(Supply::Missiles).is_zero()
            }
            Self::PowerBombs => !inventory.capacity.get(Supply::PowerBombs).is_zero(),
            Self::EnteredBy(entrance) => room.entered_by == Some(*entrance),
        }
    }
}

/// A room's versions in its order, each with its condition; the last always holds.
/// Spec: RoomStates.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoomStates {
    /// Every version but the last, in order.
    conditional: Vec<Condition>,
}

impl RoomStates {
    /// A room's versions, in order, or nothing unless the last is `Condition::Always`.
    #[must_use]
    pub fn new(mut states: Vec<Condition>) -> Option<Self> {
        match states.pop() {
            Some(Condition::Always) => Some(Self {
                conditional: states,
            }),
            Some(_) | None => None,
        }
    }

    /// The index, from 0, of the version that loads: the first whose condition holds, or the
    /// last when none before it does. Spec: RoomStates.
    #[must_use]
    pub fn choose(&self, progress: &Progress, room: RoomEntry) -> usize {
        self.conditional
            .iter()
            .position(|condition| condition.holds(progress, room))
            .unwrap_or(self.conditional.len())
    }
}
