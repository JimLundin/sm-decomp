//! Actual room schema and capture implementation used by the composed cores.
//! Runtime/device effects remain in the production host, outside this carrier.
#[path = "../../game/room_entry/api.rs"]
pub(crate) mod api;
#[path = "../../game/room_entry/content.rs"]
pub(crate) mod content;
#[path = "../../game/room_entry/content_background.rs"]
mod content_background;
#[path = "../../game/room_entry/content_state.rs"]
mod content_state;
#[path = "../../game/room_entry/effects.rs"]
pub(crate) mod effects;
#[path = "../../game/room_entry/state.rs"]
pub(crate) mod state;
#[path = "../../game/room_entry/world.rs"]
pub(crate) mod world;
pub(crate) use api::{ArrivalRequest, ContactSite, EntryPoll, EntrySignal, EntryWait, StartSite};
#[path = "../../game/room_entry/prepared_tables.rs"]
pub(crate) mod prepared_tables;
