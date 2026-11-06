//! The address map: where in the ROM each decoded content record came from (D15).
//!
//! The decoder emits it beside the content, for verification and tools only. Engine code
//! refers to content by ID and never reads it (K1).

use crate::id::RoomId;
use crate::rom::RomAddress;
use crate::room::StateIndex;

/// A record of a room state.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StatePart {
    /// The 26-byte state header.
    Header,
    /// The compressed level data.
    LevelData,
    /// The FX list in bank `$83`.
    Fx,
    /// The enemy population in bank `$A1`.
    EnemyPopulation,
    /// The enemy graphics set in bank `$B4`.
    EnemySet,
    /// The scroll table in bank `$8F`.
    Scroll,
    /// The PLM population in bank `$8F`.
    PlmPopulation,
    /// The library background in bank `$8F`.
    LibraryBackground,
}

/// A named content location.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Location {
    /// A room's header, its stable key.
    RoomHeader(RoomId),
    /// A room's door list.
    DoorList(RoomId),
    /// A record of one of a room's states.
    State(RoomId, StateIndex, StatePart),
}

/// Every decoded location, in decode order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AddressMap {
    entries: Vec<(Location, RomAddress)>,
}

impl AddressMap {
    /// Record that `location` was decoded from `address`.
    pub fn insert(&mut self, location: Location, address: RomAddress) {
        self.entries.push((location, address));
    }

    /// Where `location` was decoded from.
    #[must_use]
    pub fn get(&self, location: Location) -> Option<RomAddress> {
        self.entries
            .iter()
            .find_map(|&(known, address)| (known == location).then_some(address))
    }

    /// The locations and their addresses, in decode order.
    pub fn iter(&self) -> impl Iterator<Item = (Location, RomAddress)> + '_ {
        self.entries.iter().copied()
    }
}
