//! Options sprites: allocation, script execution and drawing in their original order.
#![forbid(unsafe_code)]
use super::menu_tilemap::native_record;
use super::prelude::*;
use crate::ram::word;

/// Game options menu object definitions in bank $82: an initialisation
/// function, a pre-instruction and an instruction list, named for the object.
pub mod definition {
    pub const MENU_CURSOR: u16 = 0xf4b8;
    pub const OPTIONS_TITLE: u16 = 0xf4c4;
    pub const CONTROLLER_TITLE: u16 = 0xf4ca;
    pub const SPECIAL_TITLE: u16 = 0xf4d0;
    pub const FADE_IN_TITLE: u16 = 0xf4d6;
}

/// The definitions' initialisation functions.
mod init {
    /// The menu cursor, and an unspawned cursor at the screen's right.
    pub const MENU_CURSOR: u16 = 0xf296;
    pub const RIGHT_CURSOR: u16 = 0xf419;
    /// The screen titles, one per definition above.
    pub const OPTIONS_TITLE: u16 = 0xf34b;
    pub const CONTROLLER_TITLE: u16 = 0xf353;
    pub const SPECIAL_TITLE: u16 = 0xf35b;
    pub const FADE_IN_TITLE: u16 = 0xf363;
}

/// Pre-instructions, named for what they do.
mod preinstruction {
    /// $82:8C10: an RTS; nothing happens.
    pub const NONE: u16 = 0x8c10;
    /// $82:8C81: the target of "clear pre-instruction".
    pub const CLEARED: u16 = 0x8c81;
    /// Place the cursor at the current menu option.
    pub const FOLLOW_MENU_OPTION: u16 = 0xf2a9;
    /// End outside the options state, or on the blanked screen 6.
    pub const END_OUTSIDE_OPTIONS_OR_BLANK: u16 = 0xf376;
    /// Scroll with screens 9 and 10; end on the blanked screen 6.
    pub const SCROLL_WITH_SCREEN: u16 = 0xf3a0;
    /// End on the blanked screen 6.
    pub const END_ON_BLANK: u16 = 0xf3e2;
    /// End outside the options state.
    pub const END_OUTSIDE_OPTIONS: u16 = 0xf42c;
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
/// The titles and cursors' sprite palette index: palette 7, a multiple of $200.
const OBJECT_PALETTE: u16 = 7 * 0x200;

const _: () = {
    use crate::constants::bank_82::*;
    assert!(definition::MENU_CURSOR == addr_stru_82F4B8 as u16);
    assert!(definition::OPTIONS_TITLE == addr_stru_82F4C4 as u16);
    assert!(definition::CONTROLLER_TITLE == addr_stru_82F4CA as u16);
    assert!(definition::SPECIAL_TITLE == addr_stru_82F4D0 as u16);
    assert!(definition::FADE_IN_TITLE == addr_stru_82F4D6 as u16);
    assert!(DELETE_LIST == addr_off_82F4B6 as u16);
    assert!(OBJECT_PALETTE == 3584);
};

pub fn clear<H: Host + ?Sized>(game: &mut H) {
    crate::word_transfers!(game, descending, 8;
        le(options_object_spritemaps, 7) <- value(0),
        le(options_object_instr_lists, 7) <- value(0),
    );
}

pub fn spawn<H: Host + ?Sized>(parameter: u16, address: u16, game: &mut H) -> u8 {
    word!(game.options_object_init_param = parameter);
    let mut index = 14u16;
    while word!(game.options_object_instr_lists[index >> 1]) != 0 {
        index = index.wrapping_sub(2);
        if index & 0x8000 != 0 {
            return 1;
        }
    }
    let record = native_record(BANK | u32::from(address), game);
    let slot = index >> 1;
    word! { game; options_object_preinstrs[slot] = record.word(2);
    options_object_instr_lists[slot] = record.word(4); options_object_instr_timers[slot] = 1; }
    word! { game, 0;
        options_object_spritemaps[slot], options_object_timers[slot], options_object_unused_a[slot], options_object_unused_b[slot],
    }
    let entry = record.word(0);
    match entry {
        init::MENU_CURSOR | init::RIGHT_CURSOR => {
            let left = entry == init::MENU_CURSOR;
            word! { game; options_object_x_pos[slot] = if left { 24 } else { 216 };
            options_object_y_pos[slot] = if left { 56 } else { 16 };
            options_object_palettes[slot] = OBJECT_PALETTE; }
        }
        init::OPTIONS_TITLE
        | init::CONTROLLER_TITLE
        | init::SPECIAL_TITLE
        | init::FADE_IN_TITLE => {
            let x: u16 = match entry {
                init::OPTIONS_TITLE => 124,
                init::CONTROLLER_TITLE => 132,
                _ => 128,
            };
            let at = crate::ram::word_arrays::options_object_x_pos + 2 * usize::from(slot);
            game.ram().copy_in(at, &x.to_ne_bytes());
            word! { game; options_object_y_pos[slot] = 16; options_object_palettes[slot] = OBJECT_PALETTE; }
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
        word!(game.options_object_index = index as u16);
        if word!(game.options_object_instr_lists[index >> 1]) != 0 {
            advance(index as u16, game);
            index = i32::from(word!(game.options_object_index));
        }
        index -= 2;
    }
    for slot in (0..8).rev() {
        if word!(game.options_object_spritemaps[slot]) != 0 {
            crate::sprite_output::paletted(
                0x82,
                word!(game.options_object_spritemaps[slot]),
                (
                    word!(game.options_object_x_pos[slot]),
                    word!(game.options_object_y_pos[slot]),
                ),
                word!(game.options_object_palettes[slot]),
                crate::sprite_drawing::Clip::Bounded,
                game,
            );
        }
    }
}

fn end_script<H: Host + ?Sized>(slot: u16, game: &mut H) {
    word! { game; options_object_instr_timers[slot] = 1; options_object_instr_lists[slot] = DELETE_LIST; }
}

fn before_script<H: Host + ?Sized>(slot: u16, game: &mut H) {
    match word!(game.options_object_preinstrs[slot]) {
        preinstruction::FOLLOW_MENU_OPTION => {
            if word!(game.game_state) == 2 {
                let screen = word!(game.game_options_screen_index);
                let table = crate::menu_lists::options_screen_objects(game, screen as isize);
                if table != 0 {
                    let address = table.wrapping_add(word!(game.menu_option_index).wrapping_mul(4));
                    let record = native_record(BANK | u32::from(address), game);
                    word! { game; options_object_x_pos[slot] = record.word(0);
                    options_object_y_pos[slot] = record.word(2); }
                } else {
                    word! { game; options_object_x_pos[slot] = 384; options_object_y_pos[slot] = 16; }
                }
            } else {
                end_script(slot, game);
            }
        }
        preinstruction::END_OUTSIDE_OPTIONS_OR_BLANK => {
            if word!(game.game_state) != 2
                || word!(game.game_options_screen_index) == 6
                    && game.ram().read_byte(crate::ram::mirror::INIDISP) == 0x80
            {
                end_script(slot, game);
            }
        }
        preinstruction::SCROLL_WITH_SCREEN => match word!(game.game_options_screen_index) {
            6 if game.ram().read_byte(crate::ram::mirror::INIDISP) == 0x80 => {
                end_script(slot, game)
            }
            9 => {
                word!(game.options_object_y_pos[slot] -= 2);
            }
            10 => {
                word!(game.options_object_y_pos[slot] += 2);
            }
            _ => {}
        },
        preinstruction::END_ON_BLANK => {
            if word!(game.game_options_screen_index) == 6
                && game.ram().read_byte(crate::ram::mirror::INIDISP) == 0x80
            {
                end_script(slot, game);
            }
        }
        preinstruction::END_OUTSIDE_OPTIONS => {
            if word!(game.game_state) != 2 {
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
            word! { game, 0; options_object_spritemaps[slot], options_object_instr_lists[slot] }
            0
        }
        instruction::SLEEP => {
            word!(game.options_object_instr_lists[slot] = address.wrapping_sub(2));
            0
        }
        instruction::SET_PREINSTRUCTION | instruction::SET_TIMER => {
            let value = native_record(BANK | u32::from(address), game).word(0);
            if opcode == instruction::SET_PREINSTRUCTION {
                word!(game.options_object_preinstrs[slot] = value);
            } else {
                word!(game.options_object_timers[slot] = value);
            }
            address.wrapping_add(2)
        }
        instruction::CLEAR_PREINSTRUCTION => {
            word!(game.options_object_preinstrs[slot] = preinstruction::CLEARED);
            address
        }
        instruction::GOTO => native_record(BANK | u32::from(address), game).word(0),
        instruction::DECREMENT_TIMER_AND_GOTO => {
            let timer = word!(game.options_object_timers[slot]);
            word!(game.options_object_timers[slot] -= 1);
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
    let index = word!(game.options_object_index);
    let slot = word!(game.options_object_index) >> 1;
    let timer = word!(game.options_object_instr_timers[slot]);
    word!(game.options_object_instr_timers[slot] -= 1);
    if timer == 1 {
        let mut address = word!(game.options_object_instr_lists[slot]);
        loop {
            let record = native_record(BANK | u32::from(address), game);
            if record.word(0) & 0x8000 == 0 {
                let slot = index >> 1;
                word! { game; options_object_instr_timers[slot] = record.word(0);
                options_object_spritemaps[slot] = record.word(2);
                options_object_instr_lists[slot] = address.wrapping_add(4); }
                return;
            }
            address = instruction(record.word(0), index >> 1, address.wrapping_add(2), game);
            if address == 0 {
                return;
            }
        }
    }
}
