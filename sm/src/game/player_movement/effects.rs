//! Finite observations and exceptional projection for normal locomotion.
#![forbid(unsafe_code)]
use super::content::Content;
use crate::game::player_transitions::effects::Host as TransitionHost;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EchoAxis {
    X,
    Y,
}
pub(crate) trait Effects {
    /// Read the current actor-owned boss selector without exposing actor storage.
    fn boss_active(&self) -> bool;
    /// Only word indices outside the four owned echo slots. Preserve live aliases
    /// and prefix-LE publication; x publication precedes the caller's fresh y read.
    fn write_echo_coordinate(&mut self, axis: EchoAxis, index: usize, value: u16);
}
pub(crate) trait Host: TransitionHost + Content + Effects {}
impl<T: TransitionHost + Content + Effects> Host for T {}
