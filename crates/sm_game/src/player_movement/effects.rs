//! Finite observations and exceptional projection for normal locomotion.
#![forbid(unsafe_code)]
use super::content::Content;
use crate::player_transitions::effects::Host as TransitionHost;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EchoAxis {
    X,
    Y,
}
pub trait Effects {
    /// Read the current actor-owned boss selector without exposing actor storage.
    fn boss_active(&self) -> bool;
    /// The captured crash-echo projectile initializer, retaining its own Runtime
    /// interval and ordered launch-record reads before the caller resumes.
    fn initialize_shinespark_echo(&mut self, slot: u16);
    /// Only word indices outside the four owned echo slots. Preserve live aliases
    /// and prefix-LE publication; x publication precedes the caller's fresh y read.
    fn write_echo_coordinate(&mut self, axis: EchoAxis, index: usize, value: u16);
}
pub trait Host: TransitionHost + Content + Effects {}
impl<T: TransitionHost + Content + Effects> Host for T {}
