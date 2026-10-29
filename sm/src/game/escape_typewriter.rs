//! The Ceres and Zebes escape typewriter text, read word by word from bank $A6.
#![forbid(unsafe_code)]
use crate::game::constants::bank_a9::addr_loc_A9C49C;
use crate::game::music::{QueueSfx2_Max3, QueueSfx3_Max3};
use crate::game::session_runtime::native_record;
use crate::game_state::Game;
use crate::ram_map::{slot, word};
use crate::types::Enemy_MotherBrain;

const QUEUE: usize = 0xd0;
const TEXT_END: i16 = 0;
const SET_DELAY: i16 = 1;
const SET_POSITION: i16 = 13;

fn text_word(cursor: i32, game: &mut Game) -> u16 {
    native_record(0xa6, cursor as u16, game).word(0)
}

fn store_word(at: usize, value: u16, game: &mut Game) {
    let bytes = value.to_ne_bytes();
    game.ram().write_byte(at, bytes[0]);
    game.ram().write_byte(at + 1, bytes[1]);
}

/// Was SetupZebesEscapeTypewriter.
pub(crate) fn start_zebes_escape_typewriter(game: &mut Game) {
    word!(game.palette_buffer[157] = word!(game.palette_buffer[125]));
    word!(game.palette_buffer[158] = word!(game.palette_buffer[126]));
    slot!(game, Enemy_MotherBrain[0].var_3B = addr_loc_A9C49C as u16);
    slot!(game, Enemy_MotherBrain[0]; var_3C = 0, var_3D = 0, var_3E = 0, var_3F = 0);
    word! { game, 0; reg_BG2HOFS, reg_BG2VOFS }
}

/// Type one character per elapsed delay; 1 once the text's end word is read.
/// Control words compare in full; characters use their low byte.
/// Was HandleTypewriterText_Ext.
pub(crate) fn type_escape_text(tiles: u16, game: &mut Game) -> u8 {
    let delay = slot!(game, Enemy_MotherBrain[0].var_3D);
    if delay != 0 {
        slot!(game, Enemy_MotherBrain[0].var_3D = delay.wrapping_sub(1));
        return 0;
    }
    slot!(
        game,
        Enemy_MotherBrain[0].var_3D = slot!(game, Enemy_MotherBrain[0].var_3E)
    );
    let mut cursor = i32::from(slot!(game, Enemy_MotherBrain[0].var_3B));
    let mut code;
    loop {
        loop {
            code = text_word(cursor, game) as i16;
            if code == TEXT_END {
                return 1;
            }
            if code != SET_DELAY {
                break;
            }
            let value = (cursor + 2) as u16;
            slot!(
                game,
                Enemy_MotherBrain[0].var_3E = text_word(i32::from(value), game)
            );
            cursor = i32::from(value) + 2;
        }
        if code != SET_POSITION {
            break;
        }
        slot!(
            game,
            Enemy_MotherBrain[0].var_3C = text_word(cursor + 2, game)
        );
        cursor += 4;
    }
    let mut character = code as u8 as i16;
    if character as u8 == b' ' {
        slot!(
            game,
            Enemy_MotherBrain[0].var_3C = slot!(game, Enemy_MotherBrain[0].var_3C).wrapping_add(1)
        );
        slot!(game, Enemy_MotherBrain[0].var_3B = (cursor + 1) as u16);
        return 0;
    }
    if character as u8 == b'!' {
        character = 91;
    }
    slot!(game, Enemy_MotherBrain[0].var_3B = (cursor + 1) as u16);
    // One two-byte tile from $7E:8034; the source word overlaps the bank word.
    let tail = word!(game.vram_write_queue_tail);
    let at = QUEUE + usize::from(tail);
    store_word(at, 2, game);
    store_word(at + 3, 0x7e00, game);
    slot!(
        game,
        Enemy_MotherBrain[0].var_3A = (i32::from(tiles) + i32::from(character) - 65) as u16
    );
    store_word(at + 2, 0x8034, game);
    let position = slot!(game, Enemy_MotherBrain[0].var_3C);
    store_word(at + 5, position, game);
    slot!(game, Enemy_MotherBrain[0].var_3C = position.wrapping_add(1));
    word!(game.vram_write_queue_tail = tail.wrapping_add(7));
    let ticks = slot!(game, Enemy_MotherBrain[0].var_3F).wrapping_add(1);
    slot!(game, Enemy_MotherBrain[0].var_3F = ticks);
    if (ticks as i32 - 2) & 0x8000 == 0 {
        slot!(game, Enemy_MotherBrain[0].var_3F = 0);
        if word!(game.area_index) == 6 {
            QueueSfx2_Max3(0x45, game);
        } else {
            QueueSfx3_Max3(0xd, game);
        }
    }
    0
}
