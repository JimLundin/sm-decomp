//! Which version of a room loads, given the run's progress. Spec: RoomStates.

use clean_types::{Area, Entrance, Event, Item};

use crate::progress::Progress;
use crate::supplies::Supply;
use crate::world::BossKinds;

/// The condition a version of a room loads on. Spec: RoomStates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Condition {
    /// Always holds.
    Always,
    /// Holds once the event has happened.
    Happened(Event),
    /// Holds when a boss of any of these kinds is defeated in the room's own area.
    Defeated(BossKinds),
    /// Holds when the item is hers, equipped or not.
    Has(Item),
    /// Holds when the morph ball is hers and her missile capacity isn't 0, however many
    /// missiles she holds.
    MorphBallAndMissiles,
    /// Holds when her power bomb capacity isn't 0, however many power bombs she holds.
    PowerBombs,
    /// Holds when she came into the room by this entrance.
    EnteredBy(Entrance),
}

/// The room she loads into: its area, and the entrance she came in by, if any. The area is the
/// new room's, known before its state is chosen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoomEntry {
    /// The room's own area.
    pub area: Area,
    /// The way she came into the room, if she came in by one.
    pub entered_by: Option<Entrance>,
}

impl Condition {
    /// Whether the condition holds for `progress`, for the room she loads into. The missiles'
    /// and power bombs' conditions read capacity, not what she holds, so a room changes for a
    /// tank taken even with every one spent.
    #[must_use]
    fn holds(self, progress: &Progress, room: RoomEntry) -> bool {
        let inventory = &progress.inventory;
        let has_any = |supply| inventory.capacity[supply].0 != 0;
        match self {
            Self::Always => true,
            Self::Happened(event) => progress.events.contains(event),
            Self::Defeated(kinds) => progress.defeated[room.area].intersects(kinds),
            Self::Has(item) => inventory.items.contains(item),
            Self::MorphBallAndMissiles => {
                inventory.items.contains(Item::MorphBall) && has_any(Supply::Missiles)
            }
            Self::PowerBombs => has_any(Supply::PowerBombs),
            Self::EnteredBy(entrance) => room.entered_by == Some(entrance),
        }
    }
}

/// A room's versions in its order, each with its condition; the last always holds.
/// Spec: RoomStates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoomStates<'a> {
    /// Every version but the last, in order.
    conditional: &'a [Condition],
}

impl<'a> RoomStates<'a> {
    /// A room's versions, in order, or nothing unless the last is `Condition::Always`.
    #[must_use]
    pub const fn new(states: &'a [Condition]) -> Option<Self> {
        match states.split_last() {
            Some((Condition::Always, conditional)) => Some(Self { conditional }),
            Some(_) | None => None,
        }
    }

    /// The index, from 0, of the version that loads: the first whose condition holds, or the
    /// last when none before it does. Spec: RoomStates.
    #[must_use]
    pub fn choose(&self, progress: &Progress, room: RoomEntry) -> usize {
        self.conditional
            .iter()
            .position(|&condition| condition.holds(progress, room))
            .unwrap_or(self.conditional.len())
    }
}
