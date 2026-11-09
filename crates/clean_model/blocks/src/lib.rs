//! The blocks model, built from its behaviour spec alone: blocks that break and come back, the
//! scroll cells that change which screens the view may show, and the stand-ins a room places.
//!
//! A reacting cell's mark names an imported reaction (`Reaction`, game content); the family
//! rules (`Rules`) say what starts it and how its start changes its cell; the started reaction
//! takes one of 40 places (`Places`) and one interpreter runs its script's steps each frame
//! (`TheScriptsRun`). A caller holds the `State` and each frame calls `Blocks::step` with the
//! room's placements if it starts, the requests in the frame's order, how she moves and whether
//! the run is paused; it reads the `Report` and the state's screens back.

mod catalogue;
mod cells;
mod frame;
mod places;
mod reactions;
mod report;
mod room_start;
mod rules;
mod run;
mod state;

pub use catalogue::{Behaviour, BehaviourId, CATALOGUE};
pub use cells::{Answer, CellKind, Grid, Inside, Mark, Reacting, Tile, Tilemap};
pub use frame::{Blocks, Request, Run};
pub use places::{PLACE_COUNT, Place, Places, Running, ScreenChange, ScrollReaction, Touched};
pub use reactions::{
    Becomes, Frames, MOST_FRAMES, PatternCell, Reaction, ReactionName, Reactions, SoundRequest,
    Step,
};
pub use report::Report;
pub use room_start::{Placement, StandIn};
pub use rules::{
    Change, FamilyRules, FirstStep, HitRule, NothingRules, NothingTouch, Rules, Start, Starter,
    TouchRule,
};
pub use state::State;
