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
    /// Draygon grabs Samus, facing `direction`.
    fn grab_samus(&mut self, direction: u16);
    fn release_samus(&mut self);
    /// Contact damage `amount` after Samus's suit.
    fn suit_damage(&mut self, amount: u16) -> u16;
    fn damage_player(&mut self, amount: u16);
    /// The original unknown-address failure.
    fn unreachable(&mut self);
}

pub fn abs16(value: u16) -> u16 {
    (value as i16).unsigned_abs()
}
