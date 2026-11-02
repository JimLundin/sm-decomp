//! The zebetites: the regenerating barriers in Mother Brain's room. Each one
//! stands alone or as a pair sharing health; the room counts how many have
//! been destroyed in three event flags and places the next one from it.
//!
//! Was `Zebetites` in the original decompilation (bank $A6:FB72-$A6:FD54). Its
//! view names the slot words over the same RAM bytes as `EnemyLayout8`.
use crate::actors::boss_lease::BossLease;
use crate::actors::deaths::Deaths;
use crate::actors::host::BossEffects;
use crate::actors::layouts::{EnemyBase, EnemyLayout8};
use crate::palette_copies::copy_rom_colors_to_palette;
use crate::session_slot;

/// The lease the zebetites run on, with the room's event flags and spawning the
/// next zebetites; dying and the shared boss effects are the game's.
pub trait Lease: BossLease<Game: BossEffects + Deaths> {
    fn event_happened(&mut self, event: u16) -> u16;
    fn set_event(&mut self, event: u16);
    fn clear_event(&mut self, event: u16);
    /// Spawn the bank $A6 enemy population at `population`; the first slot.
    fn spawn_population(&mut self, bank: u8, population: u16) -> u16;
}

/// A zebetite's view of its `EnemyLayout8` slot.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Zebetite {
    pub base: EnemyBase,
    /// Bank $A6 state function, one of [`state`].
    pub state: u16,
    pub var_b: u16,
    /// The glow palette frame, 0 to 7. Only the first enemy slot's is used.
    pub glow_frame: u16,
    /// How many zebetites have been destroyed, from the event flags.
    pub destroyed: u16,
    pub var_e: u16,
    /// Bit 15 set when this placement is a pair of small zebetites.
    pub pair: u16,
    /// Nonzero for the second zebetite of a pair.
    pub second_of_pair: u16,
    /// The other zebetite of the pair.
    pub partner: u16,
}

const _: () = {
    use core::mem::{offset_of, size_of};
    assert!(offset_of!(Zebetite, state) == offset_of!(EnemyLayout8, var_a));
    assert!(offset_of!(Zebetite, glow_frame) == offset_of!(EnemyLayout8, var_c));
    assert!(offset_of!(Zebetite, pair) == offset_of!(EnemyLayout8, var_f));
    assert!(offset_of!(Zebetite, partner) == offset_of!(EnemyLayout8, parameter_2));
    assert!(size_of::<Zebetite>() == size_of::<EnemyLayout8>());
};

/// Bank $A6 state functions, stored in the `state` word.
pub mod state {
    pub const SPAWN_PARTNER: u16 = 0xfc41;
    pub const WAIT_FOR_DOOR: u16 = 0xfc5b;
    pub const ACTIVE: u16 = 0xfc67;
}

/// Bank $A6 enemy populations: the next zebetites, and a pair's partner.
const NEXT_ZEBETITES: u16 = 0xfce1;
const PARTNER: u16 = 0xfcf9;
/// The events whose bits count destroyed zebetites, lowest bit first.
const DESTROYED_EVENTS: [u16; 3] = [3, 4, 5];
/// Four zebetites stand in all.
const ZEBETITES: u16 = 4;
/// Health regrows one point a frame up to this.
const MAX_HEALTH: i16 = 1000;
/// The glow colours: two per frame from $A6:FD87, into palette word 0x158.
const GLOW_COLORS: u16 = 0xfd87;
const GLOW_PALETTE_WORD: u16 = 0x158;
/// The sound of a shot striking a zebetite.
const HIT_SOUND: u8 = 9;

/// Places the next zebetite from the destroyed count. Was `Zebetites_Init`.
pub fn initialize<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| owned::initialize(&mut crate::runtime_access::RuntimeAccess::runtime(game)))
}

/// Stills the earthquake shake and runs the current state. Was `Zebetites_Main`.
pub fn main_ai<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| owned::main_ai(&mut crate::runtime_access::RuntimeAccess::runtime(game)))
}

/// A pair's first zebetite spawns the second and links them. Was `Zebetites_Func_1`.

/// Waits for the door transition to finish. Was `Zebetites_Func_2`.

/// Glows, shows its health stage and regrows; once destroyed, counts itself
/// and makes way for the next zebetite. Was `Zebetites_Func_3`.

/// Adds this zebetite to the destroyed count in the event flags. Was
/// `Zebetites_Func_6`/`Zebetites_Func_7`.

/// Shows the animation for the current health: five stages from full to
/// almost destroyed. Was `Zebetites_Func_5`.

/// Advances the shared glow one frame, when no palette change is pending and
/// this is a pair's first zebetite. Was `Zebetites_Func_4`.
pub fn glow<L: Lease + ?Sized>(game: &mut L) {
    if crate::sequence_word!(game.PALETTE_CHANGE_NUM) == 0
        && session_slot!(
            game,
            Zebetite[i32::from(game.current_enemy())].second_of_pair
        ) == 0
    {
        let first: i32 = 0;
        let frame: u16 = ((session_slot!(game, Zebetite[first].glow_frame) as i32 + 1) & 7) as u16;
        session_slot!(game, Zebetite[first].glow_frame = frame);
        copy_rom_colors_to_palette(
            GLOW_PALETTE_WORD,
            0xa6,
            GLOW_COLORS.wrapping_add(4 * frame),
            2,
            game,
        );
    }
}

/// Touched by Samus. Was `Zebetites_Touch`.
pub fn touch<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| owned::touch(&mut crate::runtime_access::RuntimeAccess::runtime(game)))
}

/// Hit by a shot: a pair shares its health and flash. Was `Zebetites_Shot`.
pub fn shot<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| owned::shot(&mut crate::runtime_access::RuntimeAccess::runtime(game)))
}

#[path = "zebetites_owned.rs"]
pub mod owned;
