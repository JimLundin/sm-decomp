//! What the frame and the room's start give the model (spec §1.3).

use clean_types::{Cell, HitBy, Point, Touch};

use crate::cells::StandIn;
use crate::places::ScreenChange;

/// One of the frame's requests (spec §1.3). A cell is the one a stand-in stands for, already
/// resolved, and always a cell of the room.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Request {
    /// A cell she is inside, and the point of her that is inside it.
    Inside {
        /// The cell.
        cell: Cell,
        /// Her point inside it.
        point: Point,
    },
    /// A cell her move or look reaches, and its way; answered in the report.
    Touch(Touch),
    /// A cell, and what reaches it.
    Hit {
        /// The cell.
        cell: Cell,
        /// What reaches it.
        by: HitBy,
    },
    /// Another part's start, which takes a place its part holds.
    OtherPart,
}

/// Whether the run comes this frame (`TheScriptsRun`), as the room's loading and the doors say.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Run {
    /// The run comes after the requests.
    Runs,
    /// Paused: nothing runs, and every wait, hold and touch stays to the next run.
    Paused,
}

/// One of the room's placements, in the room's order (spec §1.3, `RoomStart`).
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

/// A request or a placement that names a cell past the room's edge, which the spec rules out.
///
/// What her motion and her weapons meet there is theirs (§1.3, §8), and the room's loading places
/// nothing there (R2). The model refuses it as it enters, and goes no further.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OutsideTheRoom(pub Cell);
