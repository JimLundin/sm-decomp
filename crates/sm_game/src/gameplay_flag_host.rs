//! Finite compatibility seam only; normal domain calls use owned collections.
pub use crate::progress::{BossWordOrder, FlagAliases, FlagColumn as LegacyFlagColumn};
use crate::session_access::SessionAccess;
pub trait Host: SessionAccess + crate::progress::FlagHost {}
