//! Owned room arrival, installation and transition rules.
pub(crate) use sm_game::room_entry::api;
pub(crate) use sm_game::room_entry::background;
mod compatibility;
pub(crate) use sm_game::room_entry::content;
mod content_runtime;
pub use prepared_assets::{PreparedRoomAssets, PreparedRoomError};
pub(crate) use sm_game::room_entry::effects;
pub(crate) use sm_game::room_entry::installation;
pub(super) use sm_game::room_entry::prepared_assets;
pub(crate) mod protocol;
pub(crate) use api::{ArrivalRequest, ContactSite, EntryPoll, EntrySignal, EntryWait, StartSite};
pub(crate) use selection::begin;
pub(crate) use sm_game::room_entry::selection;
pub use sm_game::room_entry::world;
