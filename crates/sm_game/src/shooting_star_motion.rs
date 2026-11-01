//! Shooting-star motion and drawing for the ending's star field.
use crate::imported::word as table_word;
use crate::menus::host::Host;
use crate::ram::words::OAM_NEXT_PTR;
use sm_data::shooting_stars::TILE_ADDRESS as STAR_TILE_ADDRESS;

/// The 40 16-byte star records: state (index, frame high, bit 15 waiting), X
/// and Y positions with fractions, timer, and speeds; OAM and reset values.
const RECORDS: core::ops::Range<usize> = 0xe0c..0x108c;
const STATE: usize = 0;
const X: usize = 2;
const X_FRACTION: usize = 4;
const Y: usize = 6;
const Y_FRACTION: usize = 8;
const TIMER: usize = 10;
const X_SPEED: usize = 12;
const Y_SPEED: usize = 14;
const OAM: usize = 0x370;
const CENTRE: u16 = 128;
const RESET_WAIT: u16 = 32;
const CENTRED: [(usize, u16); 4] = [(X, CENTRE), (Y, CENTRE), (X_FRACTION, 0), (Y_FRACTION, 0)];
const STILL: [(usize, u16); 2] = [(X_SPEED, 0), (Y_SPEED, 0)];

/// Store each (field, value) word of the record at `at`, in order.
fn store_fields<H: Host + ?Sized>(game: &mut H, at: usize, fields: &[(usize, u16)]) {
    for &(field, value) in fields {
        game.write_slice_native(at + field, value);
    }
}

/// Clear a star's frame and wait bit, keeping its index.
fn rewind<H: Host + ?Sized>(game: &mut H, at: usize) {
    game.write_slice_native(at + STATE, u16::from(game.byte(at + STATE)));
}

/// Reset every star record to the centre with its start delay.
pub fn initialize_shooting_stars<H: Host + ?Sized>(game: &mut H) {
    for index in 0..40 {
        let at = RECORDS.start + 16 * index;
        game.write_slice_native(at + STATE, index as u16);
        store_fields(game, at, &STILL);
        let delay = crate::shooting_stars::star(game, index).start_delay;
        game.write_slice_native(at + TIMER, if delay == 0 { RESET_WAIT } else { delay });
        if delay != 0 {
            game.write_word_le(game.word(at + STATE) | 0x8000, at + STATE);
        }
        store_fields(game, at, &CENTRED);
    }
}

/// Move every star, then draw the visible ones into OAM, while `enabled`
/// (`shooting_stars_enabled`) is set.
pub fn advance_shooting_stars<H: Host + ?Sized>(enabled: u16, game: &mut H) {
    if enabled == 0 {
        return;
    }
    for at in RECORDS.step_by(16) {
        let identity = game.word(at + STATE);
        if (identity as i16) < 0 {
            let timer = game.word(at + TIMER).wrapping_sub(1);
            game.write_slice_native(at + TIMER, timer);
            if (timer as i16) < 0 {
                game.write_slice_native(at + TIMER, RESET_WAIT);
                rewind(game, at);
            }
            continue;
        }
        let star = crate::shooting_stars::star(game, usize::from(identity as u8));
        let fast = ((i32::from(identity & 0xff00) - 1024) & 0x8000) == 0;
        let x_step = star.x_acceleration.wrapping_mul(1 + u16::from(fast));
        let x_speed = game.word(at + X_SPEED).wrapping_add(x_step);
        game.write_word_le(x_speed, at + X_SPEED);
        let boost = if fast { star.y_acceleration } else { 0 };
        let y_speed = game.word(at + Y_SPEED).wrapping_add(boost);
        game.write_slice_native(at + Y_SPEED, star.y_acceleration.wrapping_add(y_speed));
        for (position, fraction, velocity) in [(X, X_FRACTION, X_SPEED), (Y, Y_FRACTION, Y_SPEED)] {
            let speed = game.word(at + velocity);
            let whole = speed.to_ne_bytes()[1] as i8 as u16;
            let (fractional, carry) = (speed << 8).overflowing_add(game.word(at + fraction));
            game.write_slice_native(at + fraction, fractional);
            let step = whole.wrapping_add(u16::from(carry));
            let moved = game.word(at + position).wrapping_add(step);
            game.write_word_le(moved, at + position);
        }
    }
    let mut output = game.word(OAM_NEXT_PTR);
    for at in RECORDS.step_by(16) {
        if (game.word(at + STATE) as i16) < 0 {
            continue;
        }
        let x = game.word(at + X) as i16;
        let oam = OAM + usize::from(output);
        let y: i16;
        if game.byte(at + X + 1) != 0
            || ((i32::from(x) - 4) & 0xff00) != 0
            || {
                // This overlapping word write precedes even the Y rejection.
                game.write_slice_native(oam, (x as u16).wrapping_sub(4));
                y = game.word(at + Y) as i16;
                game.byte(at + Y + 1) != 0
            }
            || ((i32::from(y) - 4) & 0xff00) != 0
        {
            game.write_slice_native(at + TIMER, RESET_WAIT);
            store_fields(game, at, &CENTRED);
            store_fields(game, at, &STILL);
            rewind(game, at);
            continue;
        }
        game.write_slice_native(oam + 1, (y as u16).wrapping_sub(4));
        let timer = game.word(at + TIMER).wrapping_sub(1);
        game.write_slice_native(at + TIMER, timer);
        if timer as i16 <= 0 {
            let identity = game.word(at + STATE);
            let star = usize::from(identity as u8);
            let frame_delay = crate::shooting_stars::star(game, star).frame_delay;
            game.write_slice_native(at + TIMER, frame_delay);
            game.write_slice_native(at + STATE, identity.wrapping_add(512));
        }
        if game.byte(at + STATE + 1) != 0 {
            let frame = ((game.word(at + STATE) & 0xff00) >> 9) as isize;
            let tile = table_word(
                &game.visual_tables().star_tiles,
                game,
                STAR_TILE_ADDRESS,
                frame,
            );
            game.write_slice_native(oam + 2, tile);
            output = output.wrapping_add(4);
        }
    }
    game.write_word_le(output, OAM_NEXT_PTR);
}
