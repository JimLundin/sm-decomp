//! The credits roll: script commands, text uploads and the ending handoff.
use crate::cinematic::{cinematic_native_record, queue_cinematic_tilemap};
use crate::cinematic_state::cinematic;
use crate::ram::RawRam as _;
use crate::visual_state::CinematicTilemap;

// The credits stream shares the background pre-instructions and text tilemap.
// RAM remains authoritative across script callbacks and the ending handoff.
pub fn prepare_credit_roll<H: crate::menus::host::Host + ?Sized>(
    definition: u16,
    game: &mut H,
) -> u32 {
    crate::word_transfers!(game, descending, 1024; native(0x37fe) <- value(0x7f));
    cinematic! { game, 0;
        CreditsTilemapRow, CreditsInstructionList, CreditsInstructionTimer, CreditsTimer, CreditsPreInstruction,
    }
    cinematic!(game.BgVramAddress = 18432);
    cinematic! { game, 0;
        LastCreditsUpdateY, XSub, X, YSub, Y,
    }
    cinematic!(game.CreditsEnabled |= 0x8000);
    crate::screen_fade::set_both(game, 3, 3);
    let record = cinematic_native_record::<0x8b>(definition, game);
    cinematic! {game; CreditsPreInstruction = record.word(2); CreditsInstructionList = record.word(4);
    CreditsInstructionTimer = 1; CreditsTimer = 0; }
    0x8b0000 | u32::from(record.word(0))
}

// False leaves the legacy invalid-opcode report to the bank, before its upload.
pub fn advance_credit_roll<H: crate::menus::host::Host + ?Sized>(game: &mut H) -> bool {
    // The original pair reads native low then high and writes low before high.
    let (fraction, carry) = u16::from_le(cinematic!(game.YSub)).overflowing_add(0x8000);
    let whole = u16::from_le(cinematic!(game.Y)).wrapping_add(u16::from(carry));
    cinematic!(game.YSub = fraction.to_le());
    cinematic!(game.Y = whole.to_le());
    if (i32::from(cinematic!(game.Y)) - i32::from(cinematic!(game.LastCreditsUpdateY)) - 8) & 0x8000
        != 0
    {
        return true;
    }
    cinematic!(game.LastCreditsUpdateY = cinematic!(game.Y));
    let mut cursor = cinematic!(game.CreditsInstructionList);
    loop {
        let record = cinematic_native_record::<0x8c>(cursor, game);
        let command = record.word(0);
        if record.word(0) & 0x8000 == 0 {
            break;
        }
        cursor = cursor.wrapping_add(2);
        match 0x8b0000 | u32::from(command) {
            0x8b99fe => {
                cinematic!(game.CreditsInstructionList = 0);
                return true;
            }
            0x8b9a06 => cursor = cinematic_native_record::<0x8c>(cursor, game).word(0),
            0x8b9a0d => {
                cursor = if cinematic!(game.CreditsTimer -= 1, reread) != 0 {
                    cinematic_native_record::<0x8c>(cursor, game).word(0)
                } else {
                    cursor.wrapping_add(2)
                };
            }
            0x8b9a17 => {
                cinematic!(
                    game.CreditsTimer = cinematic_native_record::<0x8c>(cursor, game).word(0)
                );
                cursor = cursor.wrapping_add(2);
            }
            0x8bf6fe => {
                cinematic!(game.CreditsEnabled &= !0x8000);
                crate::word_transfers!(game, ascending, 252; le(PALETTE_BUFFER, 4) <- rom(0x8ce7e9, 4));
                game.ram().write_byte(0x69, 0); // reg_TM
                game.ram().write_byte(0x51, 0x80); // reg_INIDISP
                crate::screen_fade::set_both(game, 0, 0);
                cinematic! { game, 0; YSub, Y }
                cinematic!(game.ShootingStarsEnabled = 1);
                crate::flow::set(game, crate::flow::Word::CinematicFunction, 0xe0f4);
                cinematic!(game.FunctionTimer = 60);
            }
            _ => return false,
        }
        if cursor == 0 {
            return true;
        }
    }
    let destination = u16::from(cinematic!(game.CreditsTilemapRow) as u8) * 0x40;
    let source = 0x10000 + u32::from(cinematic_native_record::<0x8c>(cursor, game).word(2));
    crate::word_transfers!(game, ascending, 32; native(0x3000 + destination) <- native(source));
    cinematic!(game.CreditsTilemapRow = cinematic!(game.CreditsTilemapRow).wrapping_add(1) & 0x1f);
    cinematic!(game.CreditsInstructionList = cursor.wrapping_add(4));
    true
}

// Credits retain native pre-instruction and failure order before their tilemap.
pub fn advance_credits<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    if cinematic!(game.CreditsEnabled) & 0x8000 != 0 {
        if cinematic!(game.CreditsInstructionList) != 0 {
            let command = 0x8b0000 | u32::from(cinematic!(game.CreditsPreInstruction));
            crate::cinematic_objects::advance_cinematic_background_pre(command, 0, game);
            if !advance_credit_roll(game) {
                game.rtl_unreachable();
            }
        }
        // Delete and disable still publish this frame's tilemap.
        queue_cinematic_tilemap(CinematicTilemap::FullText, game);
    }
}
