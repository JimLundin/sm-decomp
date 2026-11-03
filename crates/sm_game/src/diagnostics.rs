//! The original's fatal diagnostics, which end the run.

/// The original's unreachable-state failure. It reads no game state, so a
/// Session lease reports it without publishing its Session first.
pub trait Fatal {
    fn unreachable(&mut self) -> !;
}

/// A boss host fails through its own unknown-address failure.
impl<T: crate::actors::host::BossEffects + ?Sized> Fatal for T {
    fn unreachable(&mut self) -> ! {
        crate::actors::host::BossEffects::unreachable(self)
    }
}
