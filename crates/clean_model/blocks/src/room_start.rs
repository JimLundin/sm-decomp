//! The room's start (`RoomStart`): its placements take places in the room's order.

use clean_types::Cell;

use crate::cells::{CellKind, Mark};
use crate::places::{Place, ScreenChange, ScrollReaction, Touched};
use crate::state::State;

/// One of the room's placements, in the room's order (spec §1.3).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Placement {
    /// A scroll cell, with the screens it changes.
    Scroll {
        /// Its cell.
        cell: Cell,
        /// The screens it changes, in order.
        changes: Vec<ScreenChange>,
    },
    /// A stand-in, with its way and its reach.
    StandIn {
        /// Its cell.
        cell: Cell,
        /// Along its row or its column.
        way: StandIn,
        /// How far the cell it stands for is, in cells.
        reach: i16,
    },
    /// Another part's placement, which holds its place.
    OtherPart,
}

/// The way a stand-in stands for another cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StandIn {
    /// Along its row.
    Across,
    /// Along its column.
    Down,
}

impl StandIn {
    /// The cell kind of a stand-in this way.
    #[must_use]
    pub const fn kind(self) -> CellKind {
        match self {
            Self::Across => CellKind::StandInAcross,
            Self::Down => CellKind::StandInDown,
        }
    }
}

impl State {
    /// `RoomStart`: each placement takes a place as any start does (`Places`). A scroll cell's
    /// reaction keeps its place and its cell becomes scroll air, keeping its look; a stand-in's
    /// cell becomes the stand-in and its place is free again at once; another part's holds its
    /// place. With every place taken a placement doesn't start, and its cell is left as it was.
    /// `Blocks::step` runs it first in the frame.
    pub fn start_room(&mut self, placements: &[Placement]) {
        for placement in placements {
            match placement {
                Placement::Scroll { cell, changes } => self.place_scroll(*cell, changes),
                Placement::StandIn { cell, way, reach } => {
                    self.place_stand_in(*cell, way.kind(), *reach);
                }
                Placement::OtherPart => {
                    self.places.take(Place::OtherPart);
                }
            }
        }
    }

    /// `RoomStart`: a scroll cell placed.
    fn place_scroll(&mut self, cell: Cell, changes: &[ScreenChange]) {
        let reaction = ScrollReaction {
            cell,
            changes: changes.to_vec(),
            touched: Touched::No,
        };
        if !self.places.take(Place::Scroll(reaction)) {
            return;
        }
        if let Some(tile) = self.tiles.get_mut(cell) {
            tile.kind = CellKind::ReactingAir;
            tile.mark = Mark::Scroll;
        }
    }

    /// `RoomStart`: a stand-in placed; its place is free again at once, so it needs only that one
    /// is free.
    fn place_stand_in(&mut self, cell: Cell, kind: CellKind, reach: i16) {
        if self.places.full() {
            return;
        }
        if let Some(tile) = self.tiles.get_mut(cell) {
            tile.kind = kind;
            tile.mark = Mark::Reach(reach);
        }
    }
}
