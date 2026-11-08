//! The progress model, built from its behaviour spec alone: what the game remembers of her run
//! besides where she is.
//!
//! A `Progress` holds what she has (`Inventory`) and what has happened in the world
//! (`WorldMemory`). Domains that act tell it what happened with a `Report`, applied by
//! `Progress::apply`; domains that depend on it ask the `WorldMemory` (has an event happened, is a
//! boss defeated, is a place emptied, is a door open, is a map known) or the `Inventory` (what is
//! switched on, the item percentage). A room picks the version it loads with `RoomStates::choose`.
#![expect(
    clippy::doc_markdown,
    reason = "spec entries such as EventsHappen are named plainly, as Spec: Name, not as Rust items"
)]

mod catalogue;
mod count;
mod inventory;
mod per_area;
mod progress;
mod report;
mod room_states;
mod rules;
mod supplies;
mod world;

pub use catalogue::{Behaviour, BehaviourId, CATALOGUE};
pub use count::{Count, Divisor};
pub use inventory::{Collection, Inventory, Pickup, ReserveMode};
pub use per_area::PerArea;
pub use progress::Progress;
pub use report::Report;
pub use room_states::{Condition, RoomEntry, RoomStates};
pub use rules::{BeamSwitchesOff, Rules};
pub use supplies::{PerSupply, Supply};
pub use world::{AreaMap, WorldMemory};
