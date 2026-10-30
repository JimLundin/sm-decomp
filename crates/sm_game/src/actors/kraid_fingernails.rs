//! Kraid's fingernails, the good and bad claws Kraid flicks across the room
//! (bank $A7:BCEF-$A7:BE8E): the initialisation they share.
#![forbid(unsafe_code)]
use crate::actors::host::ActorHost;
use crate::actors::layouts::{EnemyBase, KraidLayout as Kraid, same_words};
use crate::cartridge::native_record;
use crate::constants::enemy_properties::kEnemyProps_Invisible;
use crate::ram::RawRam as _;
use crate::ram::slot;

/// A fingernail's view of its slot.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Fingernail {
    pub base: EnemyBase,
    pub function: u16,
    pub x_subvelocity: u16,
    pad_0fac: [u16; 3],
    pub function_timer: u16,
    pad_0fb4: [u8; 26700],
    pub next_function: u16,
    pub unread_7802: u16,
    pub unread_7804: u16,
    pub kraid_thinking: u16,
    pub kraid_min_y_pos_eject: u16,
    pub kraid_mouth_flags: u16,
    pub health_eighths: [u16; 8],
    pad_781c: [u16; 1],
    pub kraid_target_x: u16,
    pub health_quarters: [u16; 4],
    pad_7828: [u16; 1],
    pub kraid_hurt_frame: u16,
    pub kraid_hurt_frame_timer: u16,
    pub unread_782e: [u16; 9],
}

same_words!(Fingernail: KraidLayout {
    function = kraid_var_A, x_subvelocity = kraid_var_B, function_timer = kraid_var_F,
    next_function = kraid_next, unread_7802 = field_2, unread_7804 = field_4,
    kraid_thinking = kraid_thinking, kraid_min_y_pos_eject = kraid_min_y_pos_eject,
    kraid_mouth_flags = kraid_mouth_flags, health_eighths = kraid_healths_8ths,
    kraid_target_x = kraid_target_x, health_quarters = kraid_healths_4ths,
    kraid_hurt_frame = kraid_hurt_frame, kraid_hurt_frame_timer = kraid_hurt_frame_timer,
    unread_782e = field_2E,
});

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
pub fn initialize(k: u16, game: &mut (impl ActorHost + ?Sized)) {
    let palette_index = slot!(game, Kraid[0].base.palette_index);
    let nail = i32::from(k);
    slot!(game, Fingernail[nail].base.palette_index = palette_index);
    slot!(game, Fingernail[nail].x_subvelocity = X_SUBVELOCITY);
    slot!(
        game,
        Fingernail[nail].base.properties =
            slot!(game, Fingernail[nail].base.properties) | kEnemyProps_Invisible as u16
    );
    slot!(game, Fingernail[nail].base;
        instruction_timer = HOLD, current_instruction = instruction_list::FINGERNAIL);
    let spritemap = native_record(BANK, instruction_list::FINGERNAIL, game).word(2);
    slot!(game, Fingernail[nail].base.spritemap_pointer = spritemap);
    slot!(
        game,
        Fingernail[nail].next_function = function::INITIALISE_FINGERNAIL
    );
    slot!(
        game,
        Fingernail[nail].function = function::HANDLE_FUNCTION_TIMER
    );
    slot!(game, Fingernail[nail].function_timer = START_DELAY);
}
