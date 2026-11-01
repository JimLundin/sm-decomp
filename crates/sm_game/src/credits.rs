//! The credits roll: script commands, text uploads and the ending handoff.
use crate::cinematic::{cinematic_native_record, queue_cinematic_tilemap};
use crate::cinematic_state::Cinematic;
use crate::ram::RawRam as _;
use crate::visual_state::CinematicTilemap;

// The credits stream shares the background pre-instructions and text tilemap.
// RAM remains authoritative across script callbacks and the ending handoff.
pub fn prepare_credit_roll<H: crate::menus::host::Host + ?Sized>(
    definition: u16,
    cine: &mut Cinematic,
    game: &mut H,
) -> u32 {
    crate::word_transfers!(game, descending, 1024; native(0x37fe) <- value(0x7f));
    *cine.credits_tilemap_row() = 0;
    *cine.credits_instruction_list() = 0;
    *cine.credits_instruction_timer() = 0;
    *cine.credits_timer() = 0;
    *cine.credits_pre_instruction() = 0;
    *cine.bg_vram_address() = 18432;
    *cine.last_credits_update_y() = 0;
    *cine.x_sub() = 0;
    *cine.x() = 0;
    *cine.y_sub() = 0;
    *cine.y() = 0;
    *cine.credits_enabled() |= 0x8000;
    crate::screen_fade::store(game, 3, 3);
    let record = cinematic_native_record::<0x8b>(definition, game);
    *cine.credits_pre_instruction() = record.word(2);
    *cine.credits_instruction_list() = record.word(4);
    *cine.credits_instruction_timer() = 1;
    *cine.credits_timer() = 0;
    0x8b0000 | u32::from(record.word(0))
}

// False leaves the legacy invalid-opcode report to the bank, before its upload.
pub fn advance_credit_roll<H: crate::menus::host::Host + ?Sized>(
    cine: &mut Cinematic,
    game: &mut H,
) -> bool {
    // The original pair reads native low then high and writes low before high.
    let (fraction, carry) = u16::from_le(*cine.y_sub()).overflowing_add(0x8000);
    let whole = u16::from_le(*cine.y()).wrapping_add(u16::from(carry));
    *cine.y_sub() = fraction.to_le();
    *cine.y() = whole.to_le();
    if (i32::from(*cine.y()) - i32::from(*cine.last_credits_update_y()) - 8) & 0x8000 != 0 {
        return true;
    }
    *cine.last_credits_update_y() = *cine.y();
    let mut cursor = *cine.credits_instruction_list();
    loop {
        let record = cinematic_native_record::<0x8c>(cursor, game);
        let command = record.word(0);
        if record.word(0) & 0x8000 == 0 {
            break;
        }
        cursor = cursor.wrapping_add(2);
        match 0x8b0000 | u32::from(command) {
            0x8b99fe => {
                *cine.credits_instruction_list() = 0;
                return true;
            }
            0x8b9a06 => cursor = cinematic_native_record::<0x8c>(cursor, game).word(0),
            0x8b9a0d => {
                cursor = if {
                    *cine.credits_timer() = cine.credits_timer().wrapping_sub(1);
                    *cine.credits_timer()
                } != 0
                {
                    cinematic_native_record::<0x8c>(cursor, game).word(0)
                } else {
                    cursor.wrapping_add(2)
                };
            }
            0x8b9a17 => {
                *cine.credits_timer() = cinematic_native_record::<0x8c>(cursor, game).word(0);
                cursor = cursor.wrapping_add(2);
            }
            0x8bf6fe => {
                *cine.credits_enabled() &= !0x8000;
                crate::word_transfers!(game, ascending, 252; le(PALETTE_BUFFER, 4) <- rom(0x8ce7e9, 4));
                game.ram().write_byte(0x69, 0); // reg_TM
                game.ram().write_byte(0x51, 0x80); // reg_INIDISP
                crate::screen_fade::store(game, 0, 0);
                *cine.y_sub() = 0;
                *cine.y() = 0;
                *cine.shooting_stars_enabled() = 1;
                crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
                    flow.cinematic_function = 0xe0f4
                });
                *cine.function_timer() = 60;
            }
            _ => return false,
        }
        if cursor == 0 {
            return true;
        }
    }
    let destination = u16::from(*cine.credits_tilemap_row() as u8) * 0x40;
    let source = 0x10000 + u32::from(cinematic_native_record::<0x8c>(cursor, game).word(2));
    crate::word_transfers!(game, ascending, 32; native(0x3000 + destination) <- native(source));
    *cine.credits_tilemap_row() = (*cine.credits_tilemap_row()).wrapping_add(1) & 0x1f;
    *cine.credits_instruction_list() = cursor.wrapping_add(4);
    true
}

// Credits retain native pre-instruction and failure order before their tilemap.
pub fn advance_credits<H: crate::menus::host::Host + ?Sized>(cine: &mut Cinematic, game: &mut H) {
    if *cine.credits_enabled() & 0x8000 != 0 {
        if *cine.credits_instruction_list() != 0 {
            let command = 0x8b0000 | u32::from(*cine.credits_pre_instruction());
            crate::cinematic_objects::advance_cinematic_background_pre(command, 0, cine, game);
            if !advance_credit_roll(cine, game) {
                game.rtl_unreachable();
            }
        }
        // Delete and disable still publish this frame's tilemap.
        queue_cinematic_tilemap(CinematicTilemap::FullText, cine, game);
    }
}
