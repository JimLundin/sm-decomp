//! Shooting-star motion and drawing for the ending's star field.
use crate::imported::word as table_word;
use crate::ram::RawRam as _;
use crate::ram::{checked_word, word};
use sm_data::shooting_stars::TILE_ADDRESS as STAR_TILE_ADDRESS;

// Shooting-star scratch records stay in RAM. Native scalar loads use from_le
// on checked LE reads; only the original compound updates use LE arithmetic.
pub fn initialize_shooting_stars<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    for index in 0..40 {
        let at = 0xe0c + 16 * index;
        crate::word_transfers!(game, ascending, 1;
            native(at) <- value(index as u16),
            native(at + 12) <- value(0),
            native(at + 14) <- value(0),
        );
        let delay = crate::shooting_stars::star(game, index).start_delay;
        crate::word_transfers!(game, ascending, 1; native(at + 10) <- value(if delay == 0 { 32 } else { delay }));
        if delay != 0 {
            checked_word!(game, at; = checked_word!(game, at;) | 0x8000);
        }
        for (offset, value) in [(2, 128), (6, 128), (4, 0), (8, 0)] {
            crate::word_transfers!(game, ascending, 1; native(at + offset) <- value(value));
        }
    }
}

pub fn advance_shooting_stars<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    if word!(game.shooting_stars_enabled) == 0 {
        return;
    }
    for at in (0xe0c..0x108c).step_by(16) {
        let identity = u16::from_le(checked_word!(game, at;));
        if (identity as i16) < 0 {
            let timer = u16::from_le(checked_word!(game, at + 10;)).wrapping_sub(1);
            crate::word_transfers!(game, ascending, 1; native(at + 10) <- value(timer));
            if (timer as i16) < 0 {
                crate::word_transfers!(game, ascending, 1; native(at + 10) <- value(32));
                crate::word_transfers!(game, ascending, 1; native(at) <- value(u16::from(game.ram().read_byte(at))));
            }
            continue;
        }
        let star = crate::shooting_stars::star(game, usize::from(identity as u8));
        let fast = ((i32::from(identity & 0xff00) - 1024) & 0x8000) == 0;
        checked_word!(game, at + 12; += if fast {
            star.x_acceleration.wrapping_add(star.x_acceleration)
        } else {
            star.x_acceleration
        });
        let y_speed = if fast {
            star.y_acceleration
                .wrapping_add(u16::from_le(checked_word!(game, at + 14;)))
        } else {
            u16::from_le(checked_word!(game, at + 14;))
        };
        crate::word_transfers!(game, ascending, 1; native(at + 14) <- value(
            star.y_acceleration.wrapping_add(y_speed)
        ));
        for (position, fraction, velocity) in [(2, 4, 12), (6, 8, 14)] {
            let speed = u16::from_le(checked_word!(game, at + velocity;));
            let whole = speed.to_ne_bytes()[1] as i8 as u16;
            let (fractional, carry) =
                (speed << 8).overflowing_add(u16::from_le(checked_word!(game, at + fraction;)));
            crate::word_transfers!(game, ascending, 1; native(at + fraction) <- value(fractional));
            checked_word!(game, at + position; += whole.wrapping_add(u16::from(carry)));
        }
    }
    let mut output = word!(game.oam_next_ptr);
    for at in (0xe0c..0x108c).step_by(16) {
        if (u16::from_le(checked_word!(game, at;)) as i16) < 0 {
            continue;
        }
        let x = u16::from_le(checked_word!(game, at + 2;)) as i16;
        let oam = 0x370 + usize::from(output);
        let y: i16;
        if game.ram().read_byte(at + 3) != 0
            || (i32::from(x) - 4) & 0xff00 != 0
            || {
                // This overlapping word write precedes even the Y rejection.
                crate::word_transfers!(game, ascending, 1; native(oam) <- value((x as u16).wrapping_sub(4)));
                y = u16::from_le(checked_word!(game, at + 6;)) as i16;
                game.ram().read_byte(at + 7) != 0
            }
            || (i32::from(y) - 4) & 0xff00 != 0
        {
            for (offset, value) in [
                (10, 32),
                (2, 128),
                (6, 128),
                (4, 0),
                (8, 0),
                (12, 0),
                (14, 0),
            ] {
                crate::word_transfers!(game, ascending, 1; native(at + offset) <- value(value));
            }
            crate::word_transfers!(game, ascending, 1; native(at) <- value(u16::from(game.ram().read_byte(at))));
            continue;
        }
        crate::word_transfers!(game, ascending, 1; native(oam + 1) <- value((y as u16).wrapping_sub(4)));
        let timer = u16::from_le(checked_word!(game, at + 10;)).wrapping_sub(1);
        crate::word_transfers!(game, ascending, 1; native(at + 10) <- value(timer));
        if timer as i16 <= 0 {
            let identity = u16::from_le(checked_word!(game, at;));
            let star = usize::from(u16::from_le(checked_word!(game, at;)) as u8);
            let frame_delay = crate::shooting_stars::star(game, star).frame_delay;
            crate::word_transfers!(game, ascending, 1; native(at + 10) <- value(frame_delay));
            crate::word_transfers!(game, ascending, 1; native(at) <- value(identity.wrapping_add(512)));
        }
        if game.ram().read_byte(at + 1) != 0 {
            crate::word_transfers!(game, ascending, 1; native(oam + 2) <- value(
                table_word(&game.visual_tables().star_tiles, game, STAR_TILE_ADDRESS,
                    ((u16::from_le(checked_word!(game, at;)) & 0xff00) >> 9) as isize)
            ));
            output = output.wrapping_add(4);
        }
    }
    word!(game.oam_next_ptr = output);
}
