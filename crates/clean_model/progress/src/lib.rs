//! The progress model, built from its behaviour spec alone: what the game remembers of her run
//! besides where she is.
//!
//! A `Progress` holds what she has (its `Inventory`), what has happened in the world and what
//! its places remember, each a public field other domains read as it is now: `events[event]`,
//! `defeated[area].intersects`, `emptied.contains`, `opened.contains`, `maps[area].is_known`,
//! `zebetites`, `inventory.equipment()`, `inventory.item_percentage`. Its default is a new file.
//! Domains that act tell it what happened with a `Report`, which `Progress::apply` records. A
//! room picks the version it loads with `RoomStates::choose`.
#![expect(
    clippy::doc_markdown,
    reason = "spec entries such as EventsHappen are named plainly, as Spec: Name, not as Rust items"
)]

mod catalogue;
mod inventory;
mod percentage;
mod places;
mod progress;
mod report;
mod room_states;
mod rules;

pub use catalogue::{Behaviour, BehaviourId, CATALOGUE};
pub use inventory::{
    Beams, Count, Inventory, Items, PerBeam, PerSupply, Pickup, ReserveMode, Supply,
};
pub use places::{MapStatus, Remembered};
pub use progress::{BossKinds, Events, PerArea, Progress};
pub use report::Report;
pub use room_states::{Condition, EnteredRoom, RoomStates, Version};
pub use rules::Rules;
