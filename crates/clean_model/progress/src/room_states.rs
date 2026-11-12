//! Which version of a room loads, given the run's progress (spec §5). Spec: RoomStates.

use clean_types::{Area, BossKind, Entrance, Event, Item, Members, Set};

use crate::inventory::Supply;
use crate::progress::Progress;

/// The condition a version of a room loads on. Spec: RoomStates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Condition {
    /// Holds once the event has happened.
    Happened(Event),
    /// Holds when a boss of any of these kinds is defeated in the room's own area.
    Defeated(Set<BossKind>),
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

/// The room she enters: its area, and the entrance she came in by, if any. The area is the
/// room's own, known before its state is chosen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EnteredRoom {
    /// The room's own area.
    pub area: Area,
    /// The way she came into the room, if she came in by one.
    pub entered_by: Option<Entrance>,
}

impl Condition {
    /// Whether the condition holds for `progress`, in the room she enters. The missiles' and
    /// power bombs' conditions read capacity, not what she holds, so a room changes for a tank
    /// taken even with every one spent.
    fn holds(self, progress: &Progress, room: EnteredRoom) -> bool {
        let inventory = &progress.inventory;
        let has_any = |supply| inventory.capacity.0[supply].0 != 0;
        match self {
            Self::Happened(event) => progress.events[event],
            Self::Defeated(kinds) => progress.defeated[room.area].intersects(&kinds),
            Self::Has(item) => inventory.items[item],
            Self::MorphBallAndMissiles => {
                inventory.items[Item::MorphBall] && has_any(Supply::Missiles)
            }
            Self::PowerBombs => has_any(Supply::PowerBombs),
            Self::EnteredBy(entrance) => room.entered_by == Some(entrance),
        }
    }
}

/// A room's versions: one for each of its conditions, in its order, then the default version,
/// which loads when none of them holds. Spec: RoomStates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoomStates<'a> {
    /// The conditions of every version but the default, in the room's order.
    pub conditional: &'a [Condition],
}

/// The version of a room that loads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Version {
    /// The version of the condition at this index, from 0, of `RoomStates::conditional`.
    Conditional(usize),
    /// The default version, which loads when no condition holds.
    Default,
}

impl RoomStates<'_> {
    /// The version that loads: the first whose condition holds, or the default when none does.
    /// So a version whose condition holds wins over every later one, whether or not theirs hold
    /// too. Spec: RoomStates.
    #[must_use]
    pub fn choose(&self, progress: &Progress, room: EnteredRoom) -> Version {
        self.conditional
            .iter()
            .position(|&condition| condition.holds(progress, room))
            .map_or(Version::Default, Version::Conditional)
    }
}
