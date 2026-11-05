//! Owned room arrival, installation and transition rules.
pub(crate) use sm_game::room_entry::api;
mod compatibility;
pub(crate) use sm_game::room_entry::content;
mod content_runtime;
pub(crate) use api::{ArrivalRequest, ContactSite, EntryPoll, EntrySignal, StartSite};
pub use prepared_assets::{PreparedRoomAssets, PreparedRoomError};
pub(crate) use selection::begin;
pub(crate) use sm_game::room_entry::effects;
pub(crate) use sm_game::room_entry::installation;
pub(super) use sm_game::room_entry::prepared_assets;
pub(crate) use sm_game::room_entry::protocol;
pub(crate) use sm_game::room_entry::selection;
pub use sm_game::room_entry::world;
