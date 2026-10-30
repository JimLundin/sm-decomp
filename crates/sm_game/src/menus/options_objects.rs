//! Options sprites: allocation, script execution and drawing in their original order.
#![forbid(unsafe_code)]
use super::menu_tilemap::native_record;
use super::prelude::*;
use crate::ram::word;

/// Game options menu object definitions in bank $82 ($F4B8): a setup, a
/// pre-instruction and an instruction list each, with the bank log's names.
pub mod definition {
    pub const MENU_SELECTION_MISSILE: u16 = 0xf4b8;
    pub const OPTION_MODE_BORDER: u16 = 0xf4c4;
    pub const CONTROLLER_SETTING_MODE_BORDER: u16 = 0xf4ca;
    pub const SPECIAL_SETTING_MODE_BORDER: u16 = 0xf4d0;
    pub const SAMUS_DATA_BORDER: u16 = 0xf4d6;
}

/// The definitions' setups.
mod init {
    pub const MENU_SELECTION_MISSILE: u16 = 0xf296;
    /// The unused file select menu Samus helmet ($F4BE).
    pub const FILE_SELECT_SAMUS_HELMET: u16 = 0xf419;
    pub const OPTION_MODE_BORDER: u16 = 0xf34b;
    pub const CONTROLLER_SETTING_MODE_BORDER: u16 = 0xf353;
    pub const SPECIAL_SETTING_MODE_BORDER: u16 = 0xf35b;
    pub const SAMUS_DATA_BORDER: u16 = 0xf363;
}

/// Pre-instructions, with the bank log's names.
mod preinstruction {
    /// $82:8C10: an RTS.
    pub const NONE: u16 = 0x8c10;
    /// $82:8C81: the target of "clear pre-instruction".
    pub const CLEARED: u16 = 0x8c81;
    pub const MENU_SELECTION_MISSILE: u16 = 0xf2a9;
    pub const OPTION_MODE_BORDER: u16 = 0xf376;
    pub const CONTROLLER_SETTING_MODE_BORDER: u16 = 0xf3a0;
    pub const SPECIAL_SETTING_MODE_BORDER: u16 = 0xf3e2;
    pub const FILE_SELECT_SAMUS_HELMET: u16 = 0xf42c;
}

/// Instructions, with the bank log's names.
mod instruction {
    pub const DELETE: u16 = 0x8c5a;
    pub const SLEEP: u16 = 0x8c64;
    pub const SET_PREINSTRUCTION: u16 = 0x8c6e;
    pub const CLEAR_PREINSTRUCTION: u16 = 0x8c79;
    pub const GOTO: u16 = 0x8c82;
    pub const DECREMENT_TIMER_AND_GOTO: u16 = 0x8c89;
    pub const SET_TIMER: u16 = 0x8c93;
}

/// The objects' bank; definitions, scripts and records all live in $82.
const BANK: u32 = 0x82_0000;
/// $82:F4B6: an instruction list that deletes the object.
const DELETE_LIST: u16 = 0xf4b6;
/// The objects' sprite palette index: palette 7, a multiple of $200.
const OBJECT_PALETTE: u16 = 7 * 0x200;

const _: () = {
    use crate::constants::bank_82::*;
    assert!(definition::MENU_SELECTION_MISSILE == ADDR_STRU_82F4B8 as u16);
    assert!(definition::OPTION_MODE_BORDER == ADDR_STRU_82F4C4 as u16);
    assert!(definition::CONTROLLER_SETTING_MODE_BORDER == ADDR_STRU_82F4CA as u16);
    assert!(definition::SPECIAL_SETTING_MODE_BORDER == ADDR_STRU_82F4D0 as u16);
    assert!(definition::SAMUS_DATA_BORDER == ADDR_STRU_82F4D6 as u16);
    assert!(DELETE_LIST == ADDR_OFF_82F4B6 as u16);
    assert!(OBJECT_PALETTE == 3584);
};

pub fn clear<H: Host + ?Sized>(game: &mut H) {
    crate::word_transfers!(game, descending, 8;
        le(OPTIONS_OBJECT_SPRITEMAPS, 7) <- value(0),
        le(OPTIONS_OBJECT_INSTR_LISTS, 7) <- value(0),
    );
}

pub fn spawn<H: Host + ?Sized>(parameter: u16, address: u16, game: &mut H) -> u8 {
    word!(game.OPTIONS_OBJECT_INIT_PARAM = parameter);
    let mut index = 14u16;
    while word!(game.OPTIONS_OBJECT_INSTR_LISTS[index >> 1]) != 0 {
        index = index.wrapping_sub(2);
        if index & 0x8000 != 0 {
            return 1;
        }
    }
    let record = native_record(BANK | u32::from(address), game);
    let slot = index >> 1;
    word! { game; OPTIONS_OBJECT_PREINSTRS[slot] = record.word(2);
    OPTIONS_OBJECT_INSTR_LISTS[slot] = record.word(4); OPTIONS_OBJECT_INSTR_TIMERS[slot] = 1; }
    word! { game, 0;
        OPTIONS_OBJECT_SPRITEMAPS[slot], OPTIONS_OBJECT_TIMERS[slot], OPTIONS_OBJECT_UNUSED_A[slot], OPTIONS_OBJECT_UNUSED_B[slot],
    }
    let entry = record.word(0);
    match entry {
        init::MENU_SELECTION_MISSILE | init::FILE_SELECT_SAMUS_HELMET => {
            let left = entry == init::MENU_SELECTION_MISSILE;
            word! { game; OPTIONS_OBJECT_X_POS[slot] = if left { 24 } else { 216 };
            OPTIONS_OBJECT_Y_POS[slot] = if left { 56 } else { 16 };
            OPTIONS_OBJECT_PALETTES[slot] = OBJECT_PALETTE; }
        }
        init::OPTION_MODE_BORDER
        | init::CONTROLLER_SETTING_MODE_BORDER
        | init::SPECIAL_SETTING_MODE_BORDER
        | init::SAMUS_DATA_BORDER => {
            let x: u16 = match entry {
                init::OPTION_MODE_BORDER => 124,
                init::CONTROLLER_SETTING_MODE_BORDER => 132,
                _ => 128,
            };
            let at = crate::ram::word_arrays::OPTIONS_OBJECT_X_POS + 2 * usize::from(slot);
            game.ram().copy_in(at, &x.to_ne_bytes());
            word! { game; OPTIONS_OBJECT_Y_POS[slot] = 16; OPTIONS_OBJECT_PALETTES[slot] = OBJECT_PALETTE; }
        }
        _ => {
            game.unreachable_native();
        }
    }
    0
}

pub fn advance_and_draw<H: Host + ?Sized>(game: &mut H) {
    let mut index = 14i32;
    while index >= 0 {
        word!(game.OPTIONS_OBJECT_INDEX = index as u16);
        if word!(game.OPTIONS_OBJECT_INSTR_LISTS[index >> 1]) != 0 {
            advance(index as u16, game);
            index = i32::from(word!(game.OPTIONS_OBJECT_INDEX));
        }
        index -= 2;
    }
    for slot in (0..8).rev() {
        if word!(game.OPTIONS_OBJECT_SPRITEMAPS[slot]) != 0 {
            crate::sprite_output::paletted(
                0x82,
                word!(game.OPTIONS_OBJECT_SPRITEMAPS[slot]),
                (
                    word!(game.OPTIONS_OBJECT_X_POS[slot]),
                    word!(game.OPTIONS_OBJECT_Y_POS[slot]),
                ),
                word!(game.OPTIONS_OBJECT_PALETTES[slot]),
                crate::sprite_drawing::Clip::Bounded,
                game,
            );
        }
    }
}

fn end_script<H: Host + ?Sized>(slot: u16, game: &mut H) {
    word! { game; OPTIONS_OBJECT_INSTR_TIMERS[slot] = 1; OPTIONS_OBJECT_INSTR_LISTS[slot] = DELETE_LIST; }
}

fn before_script<H: Host + ?Sized>(slot: u16, game: &mut H) {
    match word!(game.OPTIONS_OBJECT_PREINSTRS[slot]) {
        preinstruction::MENU_SELECTION_MISSILE => {
            if game.observed_inputs().transitions.game_state == 2 {
                let screen = word!(game.GAME_OPTIONS_SCREEN_INDEX);
                let table = crate::menu_lists::options_screen_objects(game, screen as isize);
                if table != 0 {
                    let address = table.wrapping_add(word!(game.MENU_OPTION_INDEX).wrapping_mul(4));
                    let record = native_record(BANK | u32::from(address), game);
                    word! { game; OPTIONS_OBJECT_X_POS[slot] = record.word(0);
                    OPTIONS_OBJECT_Y_POS[slot] = record.word(2); }
                } else {
                    word! { game; OPTIONS_OBJECT_X_POS[slot] = 384; OPTIONS_OBJECT_Y_POS[slot] = 16; }
                }
            } else {
                end_script(slot, game);
            }
        }
        preinstruction::OPTION_MODE_BORDER => {
            if game.observed_inputs().transitions.game_state != 2
                || word!(game.GAME_OPTIONS_SCREEN_INDEX) == 6
                    && game.ram().read_byte(crate::ram::mirror::INIDISP) == 0x80
            {
                end_script(slot, game);
            }
        }
        preinstruction::CONTROLLER_SETTING_MODE_BORDER => {
            match word!(game.GAME_OPTIONS_SCREEN_INDEX) {
                6 if game.ram().read_byte(crate::ram::mirror::INIDISP) == 0x80 => {
                    end_script(slot, game)
                }
                9 => {
                    word!(game.OPTIONS_OBJECT_Y_POS[slot] -= 2);
                }
                10 => {
                    word!(game.OPTIONS_OBJECT_Y_POS[slot] += 2);
                }
                _ => {}
            }
        }
        preinstruction::SPECIAL_SETTING_MODE_BORDER => {
            if word!(game.GAME_OPTIONS_SCREEN_INDEX) == 6
                && game.ram().read_byte(crate::ram::mirror::INIDISP) == 0x80
            {
                end_script(slot, game);
            }
        }
        preinstruction::FILE_SELECT_SAMUS_HELMET => {
            if game.observed_inputs().transitions.game_state != 2 {
                end_script(slot, game);
            }
        }
        preinstruction::NONE => {}
        _ => {
            game.unreachable_native();
        }
    }
}

fn instruction<H: Host + ?Sized>(opcode: u16, slot: u16, address: u16, game: &mut H) -> u16 {
    match opcode {
        instruction::DELETE => {
            word! { game, 0; OPTIONS_OBJECT_SPRITEMAPS[slot], OPTIONS_OBJECT_INSTR_LISTS[slot] }
            0
        }
        instruction::SLEEP => {
            word!(game.OPTIONS_OBJECT_INSTR_LISTS[slot] = address.wrapping_sub(2));
            0
        }
        instruction::SET_PREINSTRUCTION | instruction::SET_TIMER => {
            let value = native_record(BANK | u32::from(address), game).word(0);
            if opcode == instruction::SET_PREINSTRUCTION {
                word!(game.OPTIONS_OBJECT_PREINSTRS[slot] = value);
            } else {
                word!(game.OPTIONS_OBJECT_TIMERS[slot] = value);
            }
            address.wrapping_add(2)
        }
        instruction::CLEAR_PREINSTRUCTION => {
            word!(game.OPTIONS_OBJECT_PREINSTRS[slot] = preinstruction::CLEARED);
            address
        }
        instruction::GOTO => native_record(BANK | u32::from(address), game).word(0),
        instruction::DECREMENT_TIMER_AND_GOTO => {
            let timer = word!(game.OPTIONS_OBJECT_TIMERS[slot]);
            word!(game.OPTIONS_OBJECT_TIMERS[slot] -= 1);
            if timer == 1 {
                address.wrapping_add(2)
            } else {
                native_record(BANK | u32::from(address), game).word(0)
            }
        }
        _ => game.unreachable_native(),
    }
}

fn advance<H: Host + ?Sized>(index: u16, game: &mut H) {
    before_script(index >> 1, game);
    let index = word!(game.OPTIONS_OBJECT_INDEX);
    let slot = word!(game.OPTIONS_OBJECT_INDEX) >> 1;
    let timer = word!(game.OPTIONS_OBJECT_INSTR_TIMERS[slot]);
    word!(game.OPTIONS_OBJECT_INSTR_TIMERS[slot] -= 1);
    if timer == 1 {
        let mut address = word!(game.OPTIONS_OBJECT_INSTR_LISTS[slot]);
        loop {
            let record = native_record(BANK | u32::from(address), game);
            if record.word(0) & 0x8000 == 0 {
                let slot = index >> 1;
                word! { game; OPTIONS_OBJECT_INSTR_TIMERS[slot] = record.word(0);
                OPTIONS_OBJECT_SPRITEMAPS[slot] = record.word(2);
                OPTIONS_OBJECT_INSTR_LISTS[slot] = address.wrapping_add(4); }
                return;
            }
            address = instruction(record.word(0), index >> 1, address.wrapping_add(2), game);
            if address == 0 {
                return;
            }
        }
    }
}
