//! The credits roll: script commands, text uploads and the ending handoff.
use crate::cinematic::{cinematic_native_record, queue_cinematic_tilemap};
use crate::ram::RawRam as _;
use crate::ram::word;
use crate::visual_state::CinematicTilemap;

// The credits stream shares the background pre-instructions and text tilemap.
// RAM remains authoritative across script callbacks and the ending handoff.
pub fn prepare_credit_roll<H: crate::menus::host::Host + ?Sized>(
    definition: u16,
    game: &mut H,
) -> u32 {
    crate::word_transfers!(game, descending, 1024; native(0x37fe) <- value(0x7f));
    word! { game, 0;
        CREDITS_TILEMAP_ROW, CREDITS_INSTR_LIST, CREDITS_INSTRUCTION_TIMER, CREDITS_TIMER, CREDITS_PRE_INSTRUCTION,
    }
    word!(game.CINEMATIC_BG_VRAM_ADDRESS = 18432);
    word! { game, 0;
        LAST_CREDITS_UPDATE_Y, CINEMATIC_X_SUB, CINEMATIC_X, CINEMATIC_Y_SUB, CINEMATIC_Y,
    }
    word!(game.CREDITS_ENABLED |= 0x8000);
    word! { game, 3; SCREEN_FADE_DELAY, SCREEN_FADE_COUNTER }
    let record = cinematic_native_record::<0x8b>(definition, game);
    word! {game; CREDITS_PRE_INSTRUCTION = record.word(2); CREDITS_INSTR_LIST = record.word(4);
    CREDITS_INSTRUCTION_TIMER = 1; CREDITS_TIMER = 0; }
    0x8b0000 | u32::from(record.word(0))
}

// False leaves the legacy invalid-opcode report to the bank, before its upload.
pub fn advance_credit_roll<H: crate::menus::host::Host + ?Sized>(game: &mut H) -> bool {
    // The original pair reads native low then high and writes low before high.
    let (fraction, carry) = u16::from_le(word!(game.CINEMATIC_Y_SUB)).overflowing_add(0x8000);
    let whole = u16::from_le(word!(game.CINEMATIC_Y)).wrapping_add(u16::from(carry));
    word!(game.CINEMATIC_Y_SUB = fraction.to_le());
    word!(game.CINEMATIC_Y = whole.to_le());
    if (i32::from(word!(game.CINEMATIC_Y)) - i32::from(word!(game.LAST_CREDITS_UPDATE_Y)) - 8)
        & 0x8000
        != 0
    {
        return true;
    }
    word!(game.LAST_CREDITS_UPDATE_Y = word!(game.CINEMATIC_Y));
    let mut cursor = word!(game.CREDITS_INSTR_LIST);
    loop {
        let record = cinematic_native_record::<0x8c>(cursor, game);
        let command = record.word(0);
        if record.word(0) & 0x8000 == 0 {
            break;
        }
        cursor = cursor.wrapping_add(2);
        match 0x8b0000 | u32::from(command) {
            0x8b99fe => {
                word!(game.CREDITS_INSTR_LIST = 0);
                return true;
            }
            0x8b9a06 => cursor = cinematic_native_record::<0x8c>(cursor, game).word(0),
            0x8b9a0d => {
                cursor = if word!(game.CREDITS_TIMER -= 1, reread) != 0 {
                    cinematic_native_record::<0x8c>(cursor, game).word(0)
                } else {
                    cursor.wrapping_add(2)
                };
            }
            0x8b9a17 => {
                word!(game.CREDITS_TIMER = cinematic_native_record::<0x8c>(cursor, game).word(0));
                cursor = cursor.wrapping_add(2);
            }
            0x8bf6fe => {
                word!(game.CREDITS_ENABLED &= !0x8000);
                crate::word_transfers!(game, ascending, 252; le(PALETTE_BUFFER, 4) <- rom(0x8ce7e9, 4));
                game.ram().write_byte(0x69, 0); // reg_TM
                game.ram().write_byte(0x51, 0x80); // reg_INIDISP
                word! { game, 0;
                    SCREEN_FADE_DELAY, SCREEN_FADE_COUNTER, CINEMATIC_Y_SUB, CINEMATIC_Y,
                }
                word!(game.SHOOTING_STARS_ENABLED = 1);
                crate::flow::set(game, crate::flow::Word::CinematicFunction, 0xe0f4);
                word!(game.CINEMATIC_FUNCTION_TIMER = 60);
            }
            _ => return false,
        }
        if cursor == 0 {
            return true;
        }
    }
    let destination = u16::from(word!(game.CREDITS_TILEMAP_ROW) as u8) * 0x40;
    let source = 0x10000 + u32::from(cinematic_native_record::<0x8c>(cursor, game).word(2));
    crate::word_transfers!(game, ascending, 32; native(0x3000 + destination) <- native(source));
    word!(game.CREDITS_TILEMAP_ROW = word!(game.CREDITS_TILEMAP_ROW).wrapping_add(1) & 0x1f);
    word!(game.CREDITS_INSTR_LIST = cursor.wrapping_add(4));
    true
}

// Credits retain native pre-instruction and failure order before their tilemap.
pub fn advance_credits<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    if word!(game.CREDITS_ENABLED) & 0x8000 != 0 {
        if word!(game.CREDITS_INSTR_LIST) != 0 {
            let command = 0x8b0000 | u32::from(word!(game.CREDITS_PRE_INSTRUCTION));
            crate::cinematic_objects::advance_cinematic_background_pre(command, 0, game);
            if !advance_credit_roll(game) {
                game.rtl_unreachable();
            }
        }
        // Delete and disable still publish this frame's tilemap.
        queue_cinematic_tilemap(CinematicTilemap::FullText, game);
    }
}
