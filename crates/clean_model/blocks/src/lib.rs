//! The blocks model, built from its behaviour spec alone: blocks that break and come back, the
//! scroll cells that change which screens the view may show, and the stand-ins a room places.
//!
//! A reacting cell's mark names one of the room's imported reactions (`Reaction`, game content);
//! the rules (`Rules`) say, by the reaction's family, what her touches and the hits ask of its
//! cell, and by the family a cell marked `Nothing` is met as, what they ask of that. A started
//! reaction takes one of 40 places (`Places`), and the run steps its script each frame
//! (`TheScriptsRun`).
//!
//! A caller holds the `State`. When a room starts it calls `State::start_room` with the room's
//! placements; each frame it calls `State::step` with `Rules::reference()` (or adjusted rules),
//! the requests in the frame's order (each cell met as `Request::Meets`, with its `Meeting`: she
//! inside it, her touch, or a hit), how she moves and whether the run runs or is paused, and
//! reads the `Report` and the state's screens back.

mod catalogue;
mod cells;
mod inputs;
mod places;
mod reactions;
mod report;
mod rules;
mod state;

pub use catalogue::{Behaviour, BehaviourId, CATALOGUE};
pub use cells::{
    Answer, CellKind, Grid, Inside, Mark, OtherMetAs, ReactingBody, StandIn, Tile, Tilemap,
};
pub use inputs::{Meeting, OutsideTheRoom, Placement, Request, Run, ScreenChange};
pub use places::{PLACE_COUNT, Place, Places, Running, ScrollReaction, Touched};
pub use reactions::{
    Becomes, Frames, MOST_FRAMES, PatternCell, Reaction, ReactionName, SoundRequest, Step,
};
pub use report::Report;
pub use rules::{
    Bodies, Change, FirstStep, HitRule, HoldUntilRun, Row, Rules, Start, Starter, TouchRule,
};
pub use state::State;
