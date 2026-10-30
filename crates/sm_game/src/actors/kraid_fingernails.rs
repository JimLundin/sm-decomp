//! Kraid's fingernails, the good and bad claws Kraid flicks across the room
//! (bank $A7:BCEF-$A7:BE8E): the initialisation they share.
#![forbid(unsafe_code)]
use crate::actors::host::ActorHost;
use crate::actors::layouts::{KraidLayout as Kraid, part_view};
use crate::cartridge::native_record;
use crate::constants::enemy_properties::kEnemyProps_Invisible;
use crate::ram::RawRam as _;
use crate::ram::slot;

part_view! {
    /// A fingernail's view of its slot.
    Fingernail: KraidLayout {
        words: [
            function, x_subvelocity, kraid_var_C, kraid_var_D, kraid_var_E, function_timer,
            kraid_parameter_1, kraid_parameter_2,
        ],
        pad1: [u8; 26696],
        words: [
            next_function, field_2, field_4, kraid_thinking, kraid_min_y_pos_eject,
            kraid_mouth_flags,
        ],
        kraid_healths_8ths: [u16; 8],
        words: [field_1C, kraid_target_x],
        kraid_healths_4ths: [u16; 4],
        words: [field_28, kraid_hurt_frame, kraid_hurt_frame_timer],
        field_2E: [u16; 9],
        words: [],
    }
}

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
