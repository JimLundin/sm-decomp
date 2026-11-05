//! Atomic runs in `sm_game::actors::atomic` on the scheduler's lease; its
//! unreachable diagnostic runs on that lease.
#![forbid(unsafe_code)]
use crate::game::session_runtime::Runtime;
pub(crate) use sm_game::actors::atomic::*;

impl Host for Runtime<'_> {
    fn unreachable(&mut self) {
        crate::rtl::unreachable()
    }
}
