//! A boss's per-frame lease: the state steps of its main AI run on the owned
//! Session, with the shared reactions and spawns it needs, and `foreign` runs
//! the rest on the game.
//!
//! # Design
//!
//! A species module writes its state steps against [`BossState`]: slot words
//! through `session_slot!`, named RAM through `word!`, sounds, the cartridge
//! and records, Samus's position, the shared actor operations, plus the few
//! effects every boss's frame uses. The game
//! implements it over the RAM image, delegating to its existing effects, and
//! the lease implements it on the Session, running those effects on the same
//! lease (they were leases of their own on the game). So a step behaves the same
//! called from either, and a main AI that opens one lease per frame keeps its
//! steps there, grouping what still needs the game into `foreign` calls.
#![forbid(unsafe_code)]
use crate::actors::host::{Death, Shot};
use crate::actors::session_slots::SlotAccess;

/// What a boss's state steps read, write and call.
pub trait BossState:
    SlotAccess
    + crate::ram::CompatibilityRam
    + crate::effects::SoundEffects
    + crate::cartridge::LoadedImage
    + crate::cartridge::NativeDiagnostics
    + crate::records::Records
    + crate::player_position::PlayerPosition
    + crate::actors::host::ActorRuntime
    + crate::word_transfers::LoadedRom
{
    /// The shared shot reaction for the current actor.
    fn react_to_shot(&mut self, shot: Shot);
    /// The shared contact reaction for the current actor.
    fn react_to_contact(&mut self, death: Death);
    /// Where the next enemy projectile spawns.
    fn place_projectile_spawn(&mut self, x: u16, y: u16);
    /// Spawn the room-graphics enemy projectile `definition` with `parameter`.
    fn spawn_room_eproj(&mut self, definition: u16, parameter: u16) -> u16;
    /// Spawn enemy projectile `definition` from actor `k` with `parameter`.
    fn spawn_actor_eproj(&mut self, parameter: u16, k: u16, definition: u16) -> u16;
    /// The next random number.
    fn roll_random(&mut self) -> u16;
    /// Decompress the cartridge data at `source` into RAM at `destination`.
    fn decompress_into_ram(&mut self, source: u32, destination: usize);
    /// Which way Samus is from the current actor.
    fn player_direction(&mut self) -> crate::actors::api::PlayerDirection;
    /// The room camera.
    fn camera(&mut self) -> crate::camera::Camera;
    /// The high word of `amplitude` times the cosine of `phase`.
    fn scaled_cosine(&mut self, phase: u16, amplitude: u16) -> u16;
    /// The high word of `amplitude` times the sine of `phase`.
    fn scaled_sine(&mut self, phase: u16, amplitude: u16) -> u16;
    /// The (x, y) step of `amplitude` along direction `phase`.
    fn direction_step(&mut self, phase: u16, amplitude: u16) -> crate::geometry::Vector32;
    /// Move the current actor by `vector` along `phase`.
    fn step_along(&mut self, phase: u16, vector: crate::geometry::Vector32);
    /// The enemy scratch word the vector movement reads.
    fn set_step_scratch(&mut self, value: u16);
    /// The actor content record at `bank:address`, as the original actor read.
    fn actor_record(&mut self, bank: u8, address: u16) -> sm_data::cartridge::RomRecord;
}

/// A lease on a boss's game. The game is its own lease, whose `foreign` runs
/// the action on itself, so steps written against a lease also run unleased.
pub trait BossLease: BossState {
    type Game: ?Sized;
    fn foreign<T>(&mut self, action: impl FnOnce(&mut Self::Game) -> T) -> T;
}
