//! Owned room arrival, installation and transition rules.
pub(crate) mod api;
pub(crate) mod background;
mod compatibility;
pub(crate) mod content;
mod content_background;
mod content_runtime;
mod content_state;
pub(crate) mod effects;
pub(crate) mod installation;
pub(super) mod prepared_assets;
pub(super) mod prepared_tables;
pub use prepared_assets::{PreparedRoomAssets, PreparedRoomError};
pub(crate) mod protocol;
pub(crate) mod selection;
pub(crate) mod state;
pub mod world;
pub(crate) use api::{ArrivalRequest, ContactSite, EntryPoll, EntrySignal, EntryWait, StartSite};
pub(crate) use selection::begin;
