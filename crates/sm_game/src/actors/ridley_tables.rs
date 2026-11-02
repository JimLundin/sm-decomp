//! Ridley's pogo speeds and tail data copies from bank $A6 ($A6:B90F, $A6:D3B4).
use crate::actors::ridley::{RIDLEY, Ridley, RidleyEdit, set_tail_word};
use crate::actors::slot_view::SlotState;
use crate::cartridge::{NativeDiagnostics, native_record};
use crate::ram::{CompatibilityRam, word};
use crate::records::Records;

const BANK: u8 = 0xa6;
/// The random pick among the four rows of X and Y velocity pointers.
const ROW_CHOICES: u16 = 3;
/// $7E:200E and $7E:200C: the pogo's upwards and downwards Y accelerations.
const UPWARDS_ACCELERATION: usize = 7;
const DOWNWARDS_ACCELERATION: usize = 6;
/// Ridley's tail has seven segments, ten words apart in tail RAM.
const TAIL_SEGMENTS: usize = 7;
const TAIL_SEGMENT_WORDS: usize = 10;
const SIGN: u16 = 0x8000;

/// Set Ridley pogo speeds ($A6:B90F): a random row of velocities, the
/// accelerations for Ridley's acceleration index, then the Y velocity from
/// its row and the X velocity, keeping its sign, from the other, each through
/// its own native fetch. Was `Ridley_Func_29`.
pub fn set_pogo_speeds(
    game: &mut (impl CompatibilityRam + NativeDiagnostics + Records + SlotState + ?Sized),
) {
    let pick = (word!(game.RANDOM_NUMBER) & ROW_CHOICES) as isize;
    let x_row = crate::actor_tables::word(game, |t| &t.ridley_x_velocities, pick);
    let y_row = crate::actor_tables::word(game, |t| &t.ridley_y_velocities, pick);
    let offset = (2 * (i32::from(Ridley::of(game, RIDLEY).acceleration_index()) + 2)) as u16;
    let step = (offset >> 1) as isize;
    word!(
        game.TILEMAP_STUFF[UPWARDS_ACCELERATION] =
            crate::actor_tables::word(game, |t| &t.ridley_upwards, step)
    );
    word!(
        game.TILEMAP_STUFF[DOWNWARDS_ACCELERATION] =
            crate::actor_tables::word(game, |t| &t.ridley_downwards, step)
    );
    *RidleyEdit::of(game, RIDLEY).y_velocity() =
        native_record(BANK, y_row, game).word(usize::from(offset));
    let speed = native_record(BANK, x_row, game).word(usize::from(offset));
    *RidleyEdit::of(game, RIDLEY).x_velocity() =
        (if Ridley::of(game, RIDLEY).x_velocity() & SIGN != 0 {
            -i32::from(speed)
        } else {
            i32::from(speed)
        }) as u16;
}

/// Set Ridley tail segments variable from list ($A6:D3B4): copy the seven
/// words of the tail data list at `source` to the tail segments' variable,
/// segment 0's at tail word `destination`. Was `Ridley_Func_93`.
pub fn set_tail_segments_from_list(
    destination: usize,
    source: u16,
    game: &mut (impl NativeDiagnostics + SlotState + ?Sized),
) {
    let words = native_record(BANK, source, game);
    for segment in 0..TAIL_SEGMENTS {
        let value = words.word(2 * segment);
        set_tail_word(game, destination + TAIL_SEGMENT_WORDS * segment, value);
    }
}
