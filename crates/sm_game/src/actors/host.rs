//! What bank-derived actor code needs from every host: named and raw work
//! RAM, enemy slots, native and loaded cartridge reads, and the RNG.
#![forbid(unsafe_code)]
use crate::cartridge::{LoadedImage, NativeDiagnostics};
use crate::ram::{CompatibilityRam, WorkRamAccess};

/// The game's random number generator.
pub trait Random {
    fn next_random(&mut self) -> u16;
}

pub trait ActorHost:
    WorkRamAccess + CompatibilityRam + NativeDiagnostics + LoadedImage + Random
{
}

impl<T> ActorHost for T where
    T: WorkRamAccess + CompatibilityRam + NativeDiagnostics + LoadedImage + Random + ?Sized
{
}

/// An axis between an actor and Samus.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
}

/// How horizontal movement treats slopes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slopes {
    Ignore,
    Solid,
    Follow,
}

/// Whether a reaction finalizes the death it causes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Death {
    Finalize,
    LeavePending,
}

/// How a shot reaction ends.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shot {
    ReactionOnly,
    Flash,
    Finalize,
}

/// The shared actor operations the compatibility runtime still implements:
/// queries against Samus and the screen, movement, the common reactions,
/// enemy projectile spawns and the enemy speed tables.
pub trait ActorRuntime {
    /// Samus's offset from actor `k` on `axis`.
    fn player_delta(&mut self, k: u16, axis: Axis) -> u16;
    /// Whether Samus is closer than `distance` to actor `k` on `axis`.
    fn player_within(&mut self, k: u16, axis: Axis, distance: u16) -> bool;
    /// Whether the current actor is outside the screen.
    fn current_outside_screen(&mut self) -> bool;
    /// Move actor `k` by `amount` (16.16) horizontally; true when blocked.
    fn move_horizontal(&mut self, k: u16, amount: i32, slopes: Slopes) -> bool;
    /// Move actor `k` by `amount` (16.16) vertically; true when blocked.
    fn move_vertical(&mut self, k: u16, amount: i32) -> bool;
    fn snap_to_slopes(&mut self, k: u16);
    fn contact_reaction(&mut self, death: Death);
    fn shot_reaction(&mut self, shot: Shot);
    fn power_bomb_reaction(&mut self, death: Death);
    /// Spawn projectile `definition` from actor `k` with `parameter`; its slot.
    fn spawn_projectile(&mut self, parameter: u16, k: u16, definition: u16) -> u16;
    /// Spawn projectile `definition` with room graphics; its slot.
    fn spawn_room_projectile(&mut self, definition: u16, parameter: u16) -> u16;
    /// The common enemy speed table word at signed word `index`.
    fn enemy_speed(&self, table: sm_data::enemy_speeds::SpeedTable, index: isize) -> u16;
}
