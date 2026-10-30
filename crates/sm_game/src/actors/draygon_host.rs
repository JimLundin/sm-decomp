//! What the Draygon family needs beyond the shared actor operations.
#![forbid(unsafe_code)]
use crate::actors::host::{ActorHost, ActorRuntime};
use crate::effects::SoundEffects;
use crate::word_transfers::LoadedRom;

pub trait Host: ActorHost + ActorRuntime + SoundEffects + LoadedRom {
    /// Create a sprite object of `kind` at (`x`, `y`); its handle.
    fn create_sprite(&mut self, x: u16, y: u16, kind: u16, appearance: u16) -> u16;
    /// Every 128 frames, a bubble from Draygon's body.
    fn draygon_body_bubble(&mut self);
    fn draygon_spawn_trails(&mut self);
    fn draygon_trajectory(&mut self);
    /// Where the next enemy projectile spawns.
    fn set_projectile_spawn_point(&mut self, x: u16, y: u16);
    /// Store each (at, value) word natively, in order, through one session lease.
    fn write_native_words(&mut self, writes: &[(usize, u16)]);
    fn disable_minimap_for_boss(&mut self);
    /// The high word of `amplitude` times the cosine of `phase`.
    fn actor_cosine(&mut self, phase: u16, amplitude: u16) -> u16;
    /// The original unknown-address failure.
    fn unreachable(&mut self);
}

pub fn abs16(value: u16) -> u16 {
    (value as i16).unsigned_abs()
}
