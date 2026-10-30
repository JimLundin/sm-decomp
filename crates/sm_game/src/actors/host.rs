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

/// What the bosses share beyond the common actor operations: sprite objects,
/// projectile placement, vector movement, Samus control and damage, boss
/// flags, music, PLMs and the original failure path.
pub trait BossEffects:
    ActorHost + ActorRuntime + crate::effects::SoundEffects + crate::word_transfers::LoadedRom
{
    /// Create a sprite object of `kind` at (`x`, `y`); its handle.
    fn create_sprite(&mut self, x: u16, y: u16, kind: u16, appearance: u16) -> u16;
    /// Where the next enemy projectile spawns.
    fn set_projectile_spawn_point(&mut self, x: u16, y: u16);
    /// Store each (at, value) word natively, in order, through one session lease.
    fn write_native_words(&mut self, writes: &[(usize, u16)]);
    fn disable_minimap_for_boss(&mut self);
    /// The high word of `amplitude` times the cosine of `phase`.
    fn actor_cosine(&mut self, phase: u16, amplitude: u16) -> u16;
    /// The high word of `amplitude` times the sine of `phase`.
    fn actor_sine(&mut self, phase: u16, amplitude: u16) -> u16;
    /// The (x, y) step of `amplitude` along direction `phase`.
    fn actor_direction_vector(&mut self, phase: u16, amplitude: u16) -> crate::geometry::Vector32;
    /// Move the current actor by `vector` along `phase`.
    fn apply_vector(&mut self, phase: u16, vector: crate::geometry::Vector32);
    /// The enemy scratch word the vector movement reads.
    fn set_enemy_scratch(&mut self, value: u16);
    /// The actor content record at `bank:address`, as the original actor read.
    fn actor_legacy_record(&mut self, bank: u8, address: u16) -> sm_data::cartridge::RomRecord;
    /// The native record at `bank:address`, with the original diagnostic.
    fn legacy_native_record(&mut self, bank: u8, address: u16) -> sm_data::cartridge::RomRecord;
    /// Run Samus command `command`; its result.
    fn command_player(&mut self, command: u16) -> u16;
    fn correct_previous_position(&mut self);
    /// Contact damage `amount` after Samus's suit.
    fn suit_damage(&mut self, amount: u16) -> u16;
    fn damage_player(&mut self, amount: u16);
    fn retire_all_sprites(&mut self);
    /// Queue music `track` with the original eight-frame delay.
    fn queue_music_delayed(&mut self, track: u16);
    /// The boss word for `area`, little-endian.
    fn boss_word(&mut self, area: u16) -> u16;
    fn set_boss_word(&mut self, area: u16, value: u16);
    /// The original unknown-address failure.
    fn unreachable(&mut self);
    /// Spawn the PLM with `header` at block (`block_x`, `block_y`).
    fn spawn_plm(&mut self, block_x: u8, block_y: u8, header: u16);
}

pub fn abs16(value: u16) -> u16 {
    (value as i16).unsigned_abs()
}
