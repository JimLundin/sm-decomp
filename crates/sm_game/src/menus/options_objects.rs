//! Options sprites: allocation, script execution and drawing in their original order.
use super::menu_tilemap::native_record;
use super::prelude::*;
use crate::ram::word;
use objects::{Array, Word};

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

/// The options objects' words ($1A8F..$1B4C), read and stored through the
/// host's projected bytes, low byte then high. They alias the enemy projectile
/// table, so the Session's enemy projectiles own and encode their bytes.
pub mod objects {
    use crate::ram::CompatibilityRam;

    /// A scalar word, by its RAM address.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Word {
        Index = 0x1a8f,
        InitParameter = 0x1a93,
    }

    /// A word per object (or per binding), by its first word's RAM address.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Array {
        Spritemaps = 0x1a9d,
        XPositions = 0x1aad,
        YPositions = 0x1abd,
        Palettes = 0x1acd,
        UnusedA = 0x1add,
        UnusedB = 0x1aed,
        InstructionLists = 0x1afd,
        PreInstructions = 0x1b0d,
        InstructionTimers = 0x1b1d,
        Timers = 0x1b2d,
        /// The controller settings' binding choices: shoot, jump, run, item
        /// cancel, item switch, aim down, aim up.
        ControllerBindings = 0x1b3d,
    }

    const _: () = {
        use crate::ram::{word_arrays as array, words as word};
        assert!(word::OPTIONS_OBJECT_INDEX == Word::Index as usize);
        assert!(word::OPTIONS_OBJECT_INIT_PARAM == Word::InitParameter as usize);
        assert!(array::OPTIONS_OBJECT_SPRITEMAPS == Array::Spritemaps as usize);
        assert!(array::OPTIONS_OBJECT_X_POS == Array::XPositions as usize);
        assert!(array::OPTIONS_OBJECT_Y_POS == Array::YPositions as usize);
        assert!(array::OPTIONS_OBJECT_PALETTES == Array::Palettes as usize);
        assert!(array::OPTIONS_OBJECT_UNUSED_A == Array::UnusedA as usize);
        assert!(array::OPTIONS_OBJECT_UNUSED_B == Array::UnusedB as usize);
        assert!(array::OPTIONS_OBJECT_INSTR_LISTS == Array::InstructionLists as usize);
        assert!(array::OPTIONS_OBJECT_PREINSTRS == Array::PreInstructions as usize);
        assert!(array::OPTIONS_OBJECT_INSTR_TIMERS == Array::InstructionTimers as usize);
        assert!(array::OPTIONS_OBJECT_TIMERS == Array::Timers as usize);
        assert!(array::OPTIONS_CONTROLLER_BINDINGS == Array::ControllerBindings as usize);
    };

    pub fn get<H: CompatibilityRam + ?Sized>(h: &H, word: Word) -> u16 {
        crate::projection::word(h, word as usize)
    }
    pub fn set<H: CompatibilityRam + ?Sized>(h: &mut H, word: Word, value: u16) {
        crate::projection::set_word(h, word as usize, value);
    }
    fn at<I: TryInto<usize, Error: core::fmt::Debug>>(array: Array, index: I) -> usize {
        array as usize + 2 * index.try_into().expect("options object index")
    }
    /// Word `index` of `array`.
    pub fn get_at<H, I>(h: &H, array: Array, index: I) -> u16
    where
        H: CompatibilityRam + ?Sized,
        I: TryInto<usize, Error: core::fmt::Debug>,
    {
        crate::projection::word(h, at(array, index))
    }
    /// Store `value` to word `index` of `array`.
    pub fn set_at<H, I>(h: &mut H, array: Array, index: I, value: u16)
    where
        H: CompatibilityRam + ?Sized,
        I: TryInto<usize, Error: core::fmt::Debug>,
    {
        crate::projection::set_word(h, at(array, index), value);
    }
}

pub fn clear<H: Host + ?Sized>(game: &mut H) {
    for slot in (0..8).rev() {
        objects::set_at(game, Array::Spritemaps, slot, 0);
        objects::set_at(game, Array::InstructionLists, slot, 0);
    }
}

pub fn spawn<H: Host + ?Sized>(parameter: u16, address: u16, game: &mut H) -> u8 {
    objects::set(game, Word::InitParameter, parameter);
    let mut index = 14u16;
    while objects::get_at(game, Array::InstructionLists, index >> 1) != 0 {
        index = index.wrapping_sub(2);
        if index & 0x8000 != 0 {
            return 1;
        }
    }
    let record = native_record(BANK | u32::from(address), game);
    let slot = index >> 1;
    objects::set_at(game, Array::PreInstructions, slot, record.word(2));
    objects::set_at(game, Array::InstructionLists, slot, record.word(4));
    objects::set_at(game, Array::InstructionTimers, slot, 1);
    for array in [
        Array::Spritemaps,
        Array::Timers,
        Array::UnusedA,
        Array::UnusedB,
    ] {
        objects::set_at(game, array, slot, 0);
    }
    let entry = record.word(0);
    match entry {
        init::MENU_SELECTION_MISSILE | init::FILE_SELECT_SAMUS_HELMET => {
            let left = entry == init::MENU_SELECTION_MISSILE;
            objects::set_at(game, Array::XPositions, slot, if left { 24 } else { 216 });
            objects::set_at(game, Array::YPositions, slot, if left { 56 } else { 16 });
            objects::set_at(game, Array::Palettes, slot, OBJECT_PALETTE);
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
            let at = Array::XPositions as usize + 2 * usize::from(slot);
            game.ram().copy_in(at, &x.to_ne_bytes());
            objects::set_at(game, Array::YPositions, slot, 16);
            objects::set_at(game, Array::Palettes, slot, OBJECT_PALETTE);
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
        objects::set(game, Word::Index, index as u16);
        if objects::get_at(game, Array::InstructionLists, index >> 1) != 0 {
            advance(index as u16, game);
            index = i32::from(objects::get(game, Word::Index));
        }
        index -= 2;
    }
    for slot in (0..8).rev() {
        if objects::get_at(game, Array::Spritemaps, slot) != 0 {
            crate::sprite_output::paletted(
                0x82,
                objects::get_at(game, Array::Spritemaps, slot),
                (
                    objects::get_at(game, Array::XPositions, slot),
                    objects::get_at(game, Array::YPositions, slot),
                ),
                objects::get_at(game, Array::Palettes, slot),
                crate::sprite_drawing::Clip::Bounded,
                game,
            );
        }
    }
}

fn end_script<H: Host + ?Sized>(slot: u16, game: &mut H) {
    objects::set_at(game, Array::InstructionTimers, slot, 1);
    objects::set_at(game, Array::InstructionLists, slot, DELETE_LIST);
}

fn before_script<H: Host + ?Sized>(slot: u16, game: &mut H) {
    match objects::get_at(game, Array::PreInstructions, slot) {
        preinstruction::MENU_SELECTION_MISSILE => {
            if game.observed_inputs().transitions.game_state == 2 {
                let screen = word!(game.GAME_OPTIONS_SCREEN_INDEX);
                let table = crate::menu_lists::options_screen_objects(game, screen as isize);
                if table != 0 {
                    let address = table.wrapping_add(word!(game.MENU_OPTION_INDEX).wrapping_mul(4));
                    let record = native_record(BANK | u32::from(address), game);
                    objects::set_at(game, Array::XPositions, slot, record.word(0));
                    objects::set_at(game, Array::YPositions, slot, record.word(2));
                } else {
                    objects::set_at(game, Array::XPositions, slot, 384);
                    objects::set_at(game, Array::YPositions, slot, 16);
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
                    let y = objects::get_at(game, Array::YPositions, slot);
                    objects::set_at(game, Array::YPositions, slot, y.wrapping_sub(2));
                }
                10 => {
                    let y = objects::get_at(game, Array::YPositions, slot);
                    objects::set_at(game, Array::YPositions, slot, y.wrapping_add(2));
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
            objects::set_at(game, Array::Spritemaps, slot, 0);
            objects::set_at(game, Array::InstructionLists, slot, 0);
            0
        }
        instruction::SLEEP => {
            objects::set_at(game, Array::InstructionLists, slot, address.wrapping_sub(2));
            0
        }
        instruction::SET_PREINSTRUCTION | instruction::SET_TIMER => {
            let value = native_record(BANK | u32::from(address), game).word(0);
            if opcode == instruction::SET_PREINSTRUCTION {
                objects::set_at(game, Array::PreInstructions, slot, value);
            } else {
                objects::set_at(game, Array::Timers, slot, value);
            }
            address.wrapping_add(2)
        }
        instruction::CLEAR_PREINSTRUCTION => {
            objects::set_at(game, Array::PreInstructions, slot, preinstruction::CLEARED);
            address
        }
        instruction::GOTO => native_record(BANK | u32::from(address), game).word(0),
        instruction::DECREMENT_TIMER_AND_GOTO => {
            let timer = objects::get_at(game, Array::Timers, slot);
            objects::set_at(game, Array::Timers, slot, timer.wrapping_sub(1));
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
    let index = objects::get(game, Word::Index);
    let slot = objects::get(game, Word::Index) >> 1;
    let timer = objects::get_at(game, Array::InstructionTimers, slot);
    objects::set_at(game, Array::InstructionTimers, slot, timer.wrapping_sub(1));
    if timer == 1 {
        let mut address = objects::get_at(game, Array::InstructionLists, slot);
        loop {
            let record = native_record(BANK | u32::from(address), game);
            if record.word(0) & 0x8000 == 0 {
                let slot = index >> 1;
                objects::set_at(game, Array::InstructionTimers, slot, record.word(0));
                objects::set_at(game, Array::Spritemaps, slot, record.word(2));
                objects::set_at(game, Array::InstructionLists, slot, address.wrapping_add(4));
                return;
            }
            address = instruction(record.word(0), index >> 1, address.wrapping_add(2), game);
            if address == 0 {
                return;
            }
        }
    }
}
