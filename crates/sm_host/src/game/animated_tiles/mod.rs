//! Animated tiles compose on the shared Session at each legacy entry.
use crate::{game::session_runtime::Runtime, game_state::Game};
pub(crate) use sm_game::animated_tiles::execution;
#[cfg(not(sm_host))]
#[path = "../../../../../sm/src/verification/animated_entries.rs"]
pub(crate) mod checks;
