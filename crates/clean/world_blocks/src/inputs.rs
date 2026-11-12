//! What the frame and the room's start give the model (spec §1.3).

use types::{Cell, HitBy, Point, Screen, ScreenKind, Way};

use crate::cells::StandIn;

/// One of the frame's requests (spec §1.3), answered in the frame's order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Request {
    /// A cell of the room, and how it is met. It is the cell a stand-in stands for, already
    /// resolved, and always a cell of the room.
    Meets {
        /// The cell.
        cell: Cell,
        /// How it is met.
        meeting: Meeting,
    },
    /// Another part's start, which takes a place its part holds.
    OtherPart,
}

/// How a request meets its cell (spec §1.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Meeting {
    /// She is inside it, at this point of her.
    Inside(Point),
    /// Her move or look reaches it, this way; answered in the report (`AnswersToHerMotion`).
    Touch(Way),
    /// One of her weapons or the grapple reaches it.
    Hit(HitBy),
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

/// A screen a scroll reaction changes, and the kind it takes (`ScrollCells`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScreenChange {
    /// The screen.
    pub screen: Screen,
    /// The kind it takes.
    pub becomes: ScreenKind,
}

/// A request or a placement that names a cell past the room's edge, which the spec rules out.
///
/// What her motion and her weapons meet there is theirs (§1.3, §8), and the room's loading places
/// nothing there (R2). The model refuses it as it enters, and goes no further.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OutsideTheRoom(pub Cell);
