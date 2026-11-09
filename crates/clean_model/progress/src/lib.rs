//! The progress model, built from its behaviour spec alone: what the game remembers of her run
//! besides where she is.
//!
//! A `Progress` holds what she has (its `Inventory`), what has happened in the world and what
//! its places remember, each a public field other domains read as it is now: `events.contains`,
//! `defeated[area].intersects`, `emptied.contains`, `opened.contains`, `maps[area].is_known`,
//! `zebetites`, `inventory.equipment()`, `inventory.item_percentage`. Domains that act tell it
//! what happened with a `Report`, which `Progress::apply` records. A room picks the version it
//! loads with `RoomStates::choose`.
#![expect(
    clippy::doc_markdown,
    reason = "spec entries such as EventsHappen are named plainly, as Spec: Name, not as Rust items"
)]

mod sets;

mod catalogue;
mod count;
mod inventory;
mod per_area;
mod percentage;
mod places;
mod progress;
mod report;
mod room_states;
mod rules;
mod supplies;
mod table;
mod world;

pub use catalogue::{Behaviour, BehaviourId, CATALOGUE};
pub use count::Count;
pub use inventory::{Beams, Inventory, Items, Pickup, ReserveMode};
pub use per_area::PerArea;
pub use places::{MapStatus, Remembered};
pub use progress::Progress;
pub use report::Report;
pub use room_states::{Condition, RoomEntry, RoomStates};
pub use rules::{BeamSwitchesOff, Rules};
pub use supplies::{PerSupply, Supply};
pub use table::{Key, Table};
pub use world::{BossKinds, Events};
