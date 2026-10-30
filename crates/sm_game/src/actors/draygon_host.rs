//! What the Draygon family needs beyond the shared boss effects.
#![forbid(unsafe_code)]
use crate::actors::host::BossEffects;
pub use crate::actors::host::abs16;

pub trait Host: BossEffects {
    /// Every 128 frames, a bubble from Draygon's body.
    fn draygon_body_bubble(&mut self);
    fn draygon_spawn_trails(&mut self);
    fn draygon_trajectory(&mut self);
    /// Draygon grabs Samus, facing `direction`.
    fn grab_samus(&mut self, direction: u16);
    fn release_samus(&mut self);
    fn draygon_spawn_ring(&mut self);
    fn draygon_death_bubble(&mut self);
    /// Draygon's sprite motion with `horizontal_bias`.
    fn draygon_motion(&mut self, horizontal_bias: u16);
    /// Draygon's death drop for `area`.
    fn scatter_draygon_drop(&mut self, area: u16);
}
