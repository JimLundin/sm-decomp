//! The Ceres and Zebes escape typewriter text, read word by word from bank $A6.
#![forbid(unsafe_code)]
use crate::game::constants::bank_a9::ADDR_LOC_A9C49C;
use crate::game::music::{queue_sfx2_max3, queue_sfx3_max3};
use crate::game::session_runtime::native_record;
use crate::game_state::Game;
use crate::ram_map::{slot, word};
use sm_game::actors::mother_brain::MotherBrainBody;

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
    word!(game.PALETTE_BUFFER[157] = word!(game.PALETTE_BUFFER[125]));
    word!(game.PALETTE_BUFFER[158] = word!(game.PALETTE_BUFFER[126]));
    slot!(
        game,
        MotherBrainBody[0].beam_left_edge = ADDR_LOC_A9C49C as u16
    );
    slot!(game, MotherBrainBody[0]; beam_left_origin_x = 0, beam_left_origin_y = 0, beam_right_origin_x = 0, beam_right_origin_y = 0);
    word! { game, 0; REG_BG2HOFS, REG_BG2VOFS }
}

/// Type one character per elapsed delay; 1 once the text's end word is read.
/// Control words compare in full; characters use their low byte.
/// Was HandleTypewriterText_Ext.
pub(crate) fn type_escape_text(tiles: u16, game: &mut Game) -> u8 {
    let delay = slot!(game, MotherBrainBody[0].beam_left_origin_y);
    if delay != 0 {
        slot!(
            game,
            MotherBrainBody[0].beam_left_origin_y = delay.wrapping_sub(1)
        );
        return 0;
    }
    slot!(
        game,
        MotherBrainBody[0].beam_left_origin_y = slot!(game, MotherBrainBody[0].beam_right_origin_x)
    );
    let mut cursor = i32::from(slot!(game, MotherBrainBody[0].beam_left_edge));
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
                MotherBrainBody[0].beam_right_origin_x = text_word(i32::from(value), game)
            );
            cursor = i32::from(value) + 2;
        }
        if code != SET_POSITION {
            break;
        }
        slot!(
            game,
            MotherBrainBody[0].beam_left_origin_x = text_word(cursor + 2, game)
        );
        cursor += 4;
    }
    let mut character = code as u8 as i16;
    if character as u8 == b' ' {
        slot!(
            game,
            MotherBrainBody[0].beam_left_origin_x =
                slot!(game, MotherBrainBody[0].beam_left_origin_x).wrapping_add(1)
        );
        slot!(
            game,
            MotherBrainBody[0].beam_left_edge = (cursor + 1) as u16
        );
        return 0;
    }
    if character as u8 == b'!' {
        character = 91;
    }
    slot!(
        game,
        MotherBrainBody[0].beam_left_edge = (cursor + 1) as u16
    );
    // One two-byte tile from $7E:8034; the source word overlaps the bank word.
    let tail = word!(game.VRAM_WRITE_QUEUE_TAIL);
    let at = QUEUE + usize::from(tail);
    store_word(at, 2, game);
    store_word(at + 3, 0x7e00, game);
    slot!(
        game,
        MotherBrainBody[0].beam_right_edge = (i32::from(tiles) + i32::from(character) - 65) as u16
    );
    store_word(at + 2, 0x8034, game);
    let position = slot!(game, MotherBrainBody[0].beam_left_origin_x);
    store_word(at + 5, position, game);
    slot!(
        game,
        MotherBrainBody[0].beam_left_origin_x = position.wrapping_add(1)
    );
    word!(game.VRAM_WRITE_QUEUE_TAIL = tail.wrapping_add(7));
    let ticks = slot!(game, MotherBrainBody[0].beam_right_origin_y).wrapping_add(1);
    slot!(game, MotherBrainBody[0].beam_right_origin_y = ticks);
    if (ticks as i32 - 2) & 0x8000 == 0 {
        slot!(game, MotherBrainBody[0].beam_right_origin_y = 0);
        if word!(game.AREA_INDEX) == 6 {
            queue_sfx2_max3(0x45, game);
        } else {
            queue_sfx3_max3(0xd, game);
        }
    }
    0
}
