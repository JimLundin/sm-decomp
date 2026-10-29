//! Finite compatibility seam only; normal domain calls use owned collections.
#![forbid(unsafe_code)]
use crate::game::session_access::SessionAccess;
pub(crate) use sm_game::progress::{BossWordOrder, FlagAliases, FlagColumn as LegacyFlagColumn};
pub(crate) trait Host: SessionAccess + sm_game::progress::FlagHost {}
