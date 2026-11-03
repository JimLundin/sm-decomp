//! A boss's per-frame lease: the state steps of its main AI run on the owned
//! Session, with the shared reactions and spawns it needs, and `foreign` runs
//! the rest on the game.
//!
//! # Design
//!
//! A species module writes its state steps against [`BossState`]: slot words
//! through its slot view, named RAM through `word!`, sounds, the cartridge
//! and records, Samus's position, the shared actor operations, plus the few
//! effects every boss's frame uses. The game
//! implements it over the RAM image, delegating to its existing effects, and
//! the lease implements it on the Session, running those effects on the same
//! lease (they were leases of their own on the game). So a step behaves the same
//! called from either, and a main AI that opens one lease per frame keeps its
//! steps there, grouping what still needs the game into `foreign` calls.
use crate::actors::host::{Death, Shot};
use crate::legacy_overflow::SlotBytes;

/// What a boss's state steps read, write and call.
pub trait BossState:
    crate::fx_domain::Sources
    + crate::presentation_workspace::PaletteAccess
    + crate::display::preparation::Access
    + SlotBytes
    + crate::ram::CompatibilityRam
    + crate::effects::SoundEffects
    + crate::cartridge::LoadedImage
    + crate::cartridge::NativeDiagnostics
    + crate::records::Records
    + crate::player_position::PlayerPosition
    + crate::actors::host::ActorRuntime
    + crate::word_transfers::LoadedRom
    + crate::display::transfer::TilemapQueue
    + crate::sprite_drawing::DisplayObjects
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
    /// The current actor's slot offset, as the scheduler selected it.
    fn current_enemy(&self) -> u16;
    /// The current room's area.
    fn area(&self) -> u16;
    /// The NMI frame counter.
    fn frame(&self) -> u16;
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
    /// The angle from actor `k` to Samus.
    fn player_angle(&mut self, k: u16) -> u16;
    /// The direction of the projectile the current actor collided with.
    fn shot_direction(&self) -> u16;
    /// The random number, as last rolled.
    fn current_random(&self) -> u16;
    /// Make actor `k` the current actor.
    fn select_enemy(&mut self, k: u16);
    /// Kill the current actor with death `kind`.
    fn kill_current(&mut self, kind: crate::actors::api::DeathKind);
    /// The current area's boss flags.
    fn area_boss_byte(&mut self) -> u8;
}

/// A lease on a boss's game. The game is its own lease, whose `foreign` runs
/// the action on itself, so steps written against a lease also run unleased.
pub trait BossLease: BossState {
    type Game: ?Sized;
    fn foreign<T>(&mut self, action: impl FnOnce(&mut Self::Game) -> T) -> T;
    /// Run `action` on the lease itself where a `foreign` call ran it on the
    /// game: the external observations are refreshed on entry, as the game's
    /// RAM was current there, and on return, as `foreign` re-imported them.
    fn owned_step<T>(&mut self, action: impl FnOnce(&mut Self) -> T) -> T;
}

/// The BG2 tilemap staging RAM ($7E:2000..$7E:2FFF) by word index: the
/// Session's `actors.extended` last two blocks, which Ridley keeps its tail in.
///
/// Stores refresh the host's observations, as the projected byte store did
/// after each byte; nothing reads them between the two bytes, so once per
/// word (or per fill) leaves the same result.
pub mod bg2_staging {
    use crate::session::Session;
    use crate::session_access::SessionAccess;
    use sm_data::table::check_index;

    /// The staging words.
    pub const WORDS: usize = 2048;
    const BLOCK: usize = 1024;

    fn word(session: &Session, index: usize) -> &u16 {
        check_index(index, WORDS);
        &session.actors.extended.blocks[4 + index / BLOCK][index % BLOCK]
    }
    fn word_mut(session: &mut Session, index: usize) -> &mut u16 {
        check_index(index, WORDS);
        &mut session.actors.extended.blocks[4 + index / BLOCK][index % BLOCK]
    }
    /// Word `index`.
    pub fn get<H: SessionAccess + ?Sized>(h: &H, index: usize) -> u16 {
        *word(h.session(), index)
    }
    /// Store `value` to word `index`.
    pub fn set<H: SessionAccess + ?Sized>(h: &mut H, index: usize, value: u16) {
        *word_mut(h.session_mut(), index) = value;
        h.words_stored();
    }
    /// Store `value` to word `index`, where `index` may pass the staging RAM's
    /// end: past it, the store reaches the RAM after $7E:2FFF, low byte then
    /// high, as the original's indexed store did.
    pub fn set_unbounded<H>(h: &mut H, index: usize, value: u16)
    where
        H: SessionAccess + crate::legacy_overflow::SlotBytes + ?Sized,
    {
        if index < WORDS {
            set(h, index, value);
        } else {
            // LEGACY(original-overflow): Crocomire's melt cursor and tilemap
            // index run past the 2048 staging words ($7E:2000..$2FFF).
            crate::legacy_overflow::set_slot_word(h, 0x2000 + 2 * index, value);
        }
    }
    /// Store `value` to each word index in `words`, in its order.
    pub fn fill<H: SessionAccess + ?Sized>(
        h: &mut H,
        words: impl Iterator<Item = usize>,
        value: u16,
    ) {
        let session = h.session_mut();
        let mut stored = false;
        for index in words {
            *word_mut(session, index) = value;
            stored = true;
        }
        if stored {
            h.words_stored();
        }
    }
}
