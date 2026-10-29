//! Actual room schema and capture implementation used by the composed cores.
//! Runtime/device effects remain in the production host, outside this carrier.
pub(crate) use api::EntryPoll;
pub(crate) use sm_game::room_entry::api;
#[allow(
    unused_imports,
    reason = "Detached room carrier omits the SpawnObject runtime facade consumer."
)]
pub(crate) use sm_game::room_entry::content;
pub(crate) use sm_game::room_entry::effects;
#[allow(
    unused_imports,
    reason = "Detached room carrier retains world schema reexports but omits runtime facade consumers."
)]
pub(crate) use sm_game::room_entry::world;
