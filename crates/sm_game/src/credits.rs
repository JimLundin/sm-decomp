//! The credits roll: script commands, text uploads and the ending handoff.
// The original's names and assignment shapes are kept.
#![allow(non_snake_case, non_upper_case_globals, unused_mut, unused_assignments)]
use crate::cinematic::{cinematic_native_record, queue_cinematic_tilemap};
#[allow(unused_imports)]
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
        cinematic_var26, cinematic_var21, cinematic_var22, cinematic_var23, cinematic_var24,
    }
    word!(game.cinematicbg_var3 = 18432);
    word! { game, 0;
        eproj_pre_instr[0], cinematic_var7, cinematic_var8, cinematic_var9, cinematic_var10,
    }
    word!(game.cinematic_var25 |= 0x8000);
    word! { game, 3; screen_fade_delay, screen_fade_counter }
    let record = cinematic_native_record::<0x8b>(definition, game);
    word! {game; cinematic_var24 = record.word(2); cinematic_var21 = record.word(4);
    cinematic_var22 = 1; cinematic_var23 = 0; }
    0x8b0000 | u32::from(record.word(0))
}

// False leaves the legacy invalid-opcode report to the bank, before its upload.
pub fn advance_credit_roll<H: crate::menus::host::Host + ?Sized>(game: &mut H) -> bool {
    // The original pair reads native low then high and writes low before high.
    let (fraction, carry) = u16::from_le(word!(game.cinematic_var9)).overflowing_add(0x8000);
    let whole = u16::from_le(word!(game.cinematic_var10)).wrapping_add(u16::from(carry));
    word!(game.cinematic_var9 = fraction.to_le());
    word!(game.cinematic_var10 = whole.to_le());
    if (i32::from(word!(game.cinematic_var10)) - i32::from(word!(game.eproj_pre_instr[0])) - 8)
        & 0x8000
        != 0
    {
        return true;
    }
    word!(game.eproj_pre_instr[0] = word!(game.cinematic_var10));
    let mut cursor = word!(game.cinematic_var21);
    loop {
        let record = cinematic_native_record::<0x8c>(cursor, game);
        let command = record.word(0);
        if record.word(0) & 0x8000 == 0 {
            break;
        }
        cursor = cursor.wrapping_add(2);
        match 0x8b0000 | u32::from(command) {
            0x8b99fe => {
                word!(game.cinematic_var21 = 0);
                return true;
            }
            0x8b9a06 => cursor = cinematic_native_record::<0x8c>(cursor, game).word(0),
            0x8b9a0d => {
                cursor = if word!(game.cinematic_var23 -= 1, reread) != 0 {
                    cinematic_native_record::<0x8c>(cursor, game).word(0)
                } else {
                    cursor.wrapping_add(2)
                };
            }
            0x8b9a17 => {
                word!(game.cinematic_var23 = cinematic_native_record::<0x8c>(cursor, game).word(0));
                cursor = cursor.wrapping_add(2);
            }
            0x8bf6fe => {
                word!(game.cinematic_var25 &= !0x8000);
                crate::word_transfers!(game, ascending, 252; le(palette_buffer, 4) <- rom(0x8ce7e9, 4));
                game.ram().write_byte(0x69, 0); // reg_TM
                game.ram().write_byte(0x51, 0x80); // reg_INIDISP
                word! { game, 0;
                    screen_fade_delay, screen_fade_counter, cinematic_var9, cinematic_var10,
                }
                word!(game.eproj_pre_instr[1] = 1);
                crate::flow::set(game, crate::flow::Word::CinematicFunction, 0xe0f4);
                word!(game.cinematic_var4 = 60);
            }
            _ => return false,
        }
        if cursor == 0 {
            return true;
        }
    }
    let destination = u16::from(word!(game.cinematic_var26) as u8) * 0x40;
    let source = 0x10000 + u32::from(cinematic_native_record::<0x8c>(cursor, game).word(2));
    crate::word_transfers!(game, ascending, 32; native(0x3000 + destination) <- native(source));
    word!(game.cinematic_var26 = word!(game.cinematic_var26).wrapping_add(1) & 0x1f);
    word!(game.cinematic_var21 = cursor.wrapping_add(4));
    true
}

// Credits retain native pre-instruction and failure order before their tilemap.
#[forbid(unsafe_code)]
pub fn advance_credits<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    if word!(game.cinematic_var25) & 0x8000 != 0 {
        if word!(game.cinematic_var21) != 0 {
            let command = 0x8b0000 | u32::from(word!(game.cinematic_var24));
            crate::cinematic_objects::advance_cinematic_background_pre(command, 0, game);
            if !advance_credit_roll(game) {
                game.rtl_unreachable();
            }
        }
        // Delete and disable still publish this frame's tilemap.
        queue_cinematic_tilemap(CinematicTilemap::FullText, game);
    }
}
