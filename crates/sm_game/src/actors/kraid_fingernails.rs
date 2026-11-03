//! Kraid's fingernails, the good and bad claws Kraid flicks across the room
//! (bank $A7:BCEF-$A7:BE8E): the initialisation they share.
use crate::actors::boss_lease::BossState;
use crate::actors::kraid::{Body, FingernailEdit};
use crate::actors::slot_view::SlotState;
use crate::cartridge::native_record;
use crate::constants::enemy_properties::K_ENEMY_PROPS_INVISIBLE;

/// Bank $A7 instruction lists.
pub mod instruction_list {
    /// $A7:8B0A, the fingernail; its first entry's spritemap is word 2.
    pub const FINGERNAIL: u16 = 0x8b0a;
}

/// Bank $A7 Kraid enemy functions, stored in `function` and `next_function`.
pub mod function {
    /// $A7:B92D: count down `function_timer`, then run `next_function`.
    pub const HANDLE_FUNCTION_TIMER: u16 = 0xb92d;
    /// $A7:BD60: initialise fingernail.
    pub const INITIALISE_FINGERNAIL: u16 = 0xbd60;
}

const BANK: u8 = 0xa7;
const X_SUBVELOCITY: u16 = 0x28;
/// The instruction timer that holds the list on its first entry.
const HOLD: u16 = 0x7fff;
/// Frames before the fingernail is initialised.
const START_DELAY: u16 = 0x40;

/// Kraid's fingernails initialisation AI ($A7:BCF2), for good and bad
/// fingernails. Was `KraidFingernailInit`.
pub fn initialize(k: u16, game: &mut (impl BossState + SlotState + ?Sized)) {
    let palette_index = Body::of(game, 0).actor().palette;
    FingernailEdit::of(game, k).actor().palette = palette_index;
    *FingernailEdit::of(game, k).x_subspeed() = X_SUBVELOCITY;
    FingernailEdit::of(game, k).actor().properties |= K_ENEMY_PROPS_INVISIBLE as u16;
    FingernailEdit::of(game, k).actor().instruction_delay = HOLD;
    FingernailEdit::of(game, k).actor().instruction = instruction_list::FINGERNAIL;
    let spritemap = native_record(BANK, instruction_list::FINGERNAIL, game).word(2);
    FingernailEdit::of(game, k).actor().spritemap = spritemap;
    *FingernailEdit::of(game, k).next_function() = function::INITIALISE_FINGERNAIL;
    *FingernailEdit::of(game, k).function() = function::HANDLE_FUNCTION_TIMER;
    *FingernailEdit::of(game, k).function_timer() = START_DELAY;
}
