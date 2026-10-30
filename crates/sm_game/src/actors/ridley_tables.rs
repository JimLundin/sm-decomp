//! Ridley's pogo speeds and tail data copies from bank $A6 ($A6:B90F, $A6:D3B4).
#![forbid(unsafe_code)]
use crate::actors::host::ActorHost;
use crate::actors::ridley::Ridley;
use crate::cartridge::native_record;
use crate::cartridge::rom_table_word;
use crate::ram::RawRam as _;
use crate::ram::{slot, word};

const BANK: u8 = 0xa6;
const RIDLEY: i32 = 0;
/// $A6:B965 and $A6:B96D: four pointers each to rows of X and Y velocities,
/// picked at random.
const X_VELOCITY_ROWS: u32 = 0xa6_b965;
const Y_VELOCITY_ROWS: u32 = 0xa6_b96d;
const ROW_CHOICES: u16 = 3;
/// $A6:B94D and $A6:B959: the pogo's upwards and downwards Y accelerations,
/// by acceleration index.
const UPWARDS_ACCELERATIONS: u32 = 0xa6_b94d;
const DOWNWARDS_ACCELERATIONS: u32 = 0xa6_b959;
/// $7E:200E and $7E:200C: the pogo's upwards and downwards Y accelerations.
const UPWARDS_ACCELERATION: usize = 7;
const DOWNWARDS_ACCELERATION: usize = 6;
/// Ridley's tail has seven segments, 20 bytes apart in tail RAM.
const TAIL_SEGMENTS: usize = 7;
const TAIL_SEGMENT_BYTES: usize = 20;
const SIGN: u16 = 0x8000;

/// Set Ridley pogo speeds ($A6:B90F): a random row of velocities, the
/// accelerations for Ridley's acceleration index, then the Y velocity from
/// its row and the X velocity, keeping its sign, from the other, each through
/// its own native fetch. Was `Ridley_Func_29`.
pub fn set_pogo_speeds(game: &mut (impl ActorHost + ?Sized)) {
    let pick = (word!(game.random_number) & ROW_CHOICES) as isize;
    let x_row = rom_table_word!(game, X_VELOCITY_ROWS, pick);
    let y_row = rom_table_word!(game, Y_VELOCITY_ROWS, pick);
    let offset = (2 * (i32::from(slot!(game, Ridley[RIDLEY].acceleration_index)) + 2)) as u16;
    let step = (offset >> 1) as isize;
    word!(
        game.tilemap_stuff[UPWARDS_ACCELERATION] =
            rom_table_word!(game, UPWARDS_ACCELERATIONS, step)
    );
    word!(
        game.tilemap_stuff[DOWNWARDS_ACCELERATION] =
            rom_table_word!(game, DOWNWARDS_ACCELERATIONS, step)
    );
    slot!(
        game,
        Ridley[RIDLEY].y_velocity = native_record(BANK, y_row, game).word(usize::from(offset))
    );
    let speed = native_record(BANK, x_row, game).word(usize::from(offset));
    slot!(
        game,
        Ridley[RIDLEY].x_velocity = (if slot!(game, Ridley[RIDLEY].x_velocity) & SIGN != 0 {
            -i32::from(speed)
        } else {
            i32::from(speed)
        }) as u16
    );
}

/// Set Ridley tail segments variable from list ($A6:D3B4): copy the seven
/// words of the tail data list at `source` to the tail segments' variable at
/// `destination`. Was `Ridley_Func_93`.
pub fn set_tail_segments_from_list(
    destination: u16,
    source: u16,
    game: &mut (impl ActorHost + ?Sized),
) {
    let words = native_record(BANK, source, game);
    for i in 0..TAIL_SEGMENTS {
        let bytes = words.word(2 * i).to_ne_bytes();
        let at = usize::from(destination) + TAIL_SEGMENT_BYTES * i;
        game.ram().write_byte(at, bytes[0]);
        game.ram().write_byte(at + 1, bytes[1]);
    }
}
