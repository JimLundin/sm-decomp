//! Phantoon's figure-8 flight and its casual flames (bank $A7:CFCA-$A7:D2D0).
use crate::actors::phantoon::{State, part};

use crate::cartridge::rom_table_word;
use sm_data::cartridge::RomRecord;

const BANK: u8 = 0xa7;
const BODY: i32 = 0;
const EYE: i32 = 0x40;
/// The mouth's slot, from the figure-8's `object`.
const MOUTH_OFFSET: i32 = 0xc0;

/// $A7:E3D2: the figure-8's 216h steps of signed X and Y bytes.
const MOVEMENT_DATA: u16 = 0xe3d2;
const STEPS: i32 = 0x216;
/// $A7:CCFD: the four casual flame patterns, each a count then flame timers.
const CASUAL_FLAME_TIMERS: u32 = 0xa7_ccfd;
const CASUAL_FLAME_PATTERN_BITS: u16 = 3;
/// $A7:CCEB: the mouth's spawn flame instruction list.
const SPAWN_FLAME: u16 = 0xcceb;
/// A pattern index of FFFFh picks a new pattern.
const NEW_PATTERN: u16 = u16::MAX;
const SIGN: u16 = 0x8000;

fn native_record(address: u16, game: &mut (impl State + ?Sized)) -> RomRecord {
    crate::cartridge::native_record(BANK, address, game)
}

/// One movement stage: its speed change (16.16), the bound that ends it and
/// the speed it is clamped to there, whether it slows to a lower bound, and
/// whether it restarts at stage 0.
struct Stage {
    acceleration: i32,
    bound: u16,
    clamp: u16,
    lower: bool,
    restart: bool,
}

/// The stage for `stage` from the accelerations at $A7:CD73 and caps at
/// $A7:CD7B, or for the reverse figure-8 at $A7:CD81 and $A7:CD89.
fn stage(reversed: bool, stage: u16) -> Stage {
    let (acceleration, bound, clamp, lower, restart) = match (reversed, stage) {
        (false, 0) => (0x600, 2, 1, false, false),
        (false, p) if p & 1 != 0 => (0x1000, 7, 7, false, false),
        (false, _) => (-0x1000, 0, 1, true, true),
        (true, 0) => (-0x600, 0xfffe, 0, true, false),
        (true, p) if p & 1 != 0 => (-0x1000, 0xfff9, 0xfffa, true, false),
        (true, _) => (0x1000, 0, 0, false, true),
    };
    Stage {
        acceleration,
        bound,
        clamp,
        lower,
        restart,
    }
}

/// Update figure-8 Phantoon speed ($A7:D114), or its reverse ($A7:D193):
/// accelerate, then slow, through the movement stages.
fn update_speed(reversed: bool, game: &mut (impl State + ?Sized)) {
    let Stage {
        acceleration,
        bound,
        clamp,
        lower,
        restart,
    } = stage(reversed, part!(game, Body[BODY].movement_stage));
    part!(
        game,
        Body[BODY](speed, speed_subpixel) += acceleration as u32
    );
    let reached = if lower {
        part!(game, Body[BODY].speed) == bound
            || (part!(game, Body[BODY].speed) as i16).wrapping_sub(bound as i16) < 0
    } else {
        (part!(game, Body[BODY].speed) as i16).wrapping_sub(bound as i16) >= 0
    };
    if reached {
        part!(game, Body[BODY]; speed = clamp, speed_subpixel = 0);
        part!(game, Body[BODY].movement_stage, |old| if restart {
            0
        } else {
            old.wrapping_add(1)
        });
    }
}

fn offset_position(position: u16, byte: u8, reversed: bool) -> u16 {
    let delta = (byte as i8) as u16;
    if reversed {
        position.wrapping_sub(delta)
    } else {
        position.wrapping_add(delta)
    }
}

/// Move Phantoon in figure-8 ($A7:D215), or in reverse ($A7:D271): one step
/// of the movement data for each pixel of speed.
fn move_in_figure_eight(reversed: bool, game: &mut (impl State + ?Sized)) {
    let speed = part!(game, Body[BODY].speed);
    let mut remaining = i32::from(if reversed {
        (-i32::from(speed)) as u16
    } else {
        speed
    });
    while remaining != 0 {
        let record = native_record(
            (2 * i32::from(part!(game, Body[BODY].movement_index)) + i32::from(MOVEMENT_DATA))
                as u16,
            game,
        );
        // Each position read precedes its lazy ROM byte; X is stored before Y.
        part!(game, Body[BODY].base.x_pos, |old| offset_position(
            old,
            record.byte(0),
            reversed
        ));
        part!(game, Body[BODY].base.y_pos, |old| offset_position(
            old,
            record.byte(1),
            reversed
        ));
        part!(
            game,
            Body[BODY].movement_index = if reversed {
                part!(game, Body[BODY].movement_index).wrapping_sub(1)
            } else {
                part!(game, Body[BODY].movement_index).wrapping_add(1)
            }
        );
        if reversed {
            if part!(game, Body[BODY].movement_index) & SIGN != 0 {
                part!(game, Body[BODY].movement_index = (STEPS - 1) as u16);
            }
        } else if (i32::from(part!(game, Body[BODY].movement_index)) - STEPS) & SIGN as i32 == 0 {
            part!(game, Body[BODY].movement_index = 0);
        }
        remaining -= 1;
    }
}

/// Handle casual flames ($A7:CFCA): count down the flame timer; at zero, step
/// through the pattern's timers, setting the mouth to spawn a flame, or pick a
/// new pattern once it is spent.
fn handle_casual_flames(object: u16, game: &mut (impl State + ?Sized)) {
    let mouth = i32::from(object) + MOUTH_OFFSET;
    let expired = part!(game, Mouth[mouth].casual_flame_timer) == 1;
    part!(game, Mouth[mouth].casual_flame_timer, |old| old
        .wrapping_sub(1));
    let underflow = part!(game, Mouth[mouth].casual_flame_timer) & SIGN != 0;
    if expired || underflow {
        if part!(game, Mouth[mouth].casual_flame_pattern_index) & SIGN == 0 {
            let expired = part!(game, Mouth[mouth].casual_flame_pattern_index) == 1;
            part!(game, Mouth[mouth].casual_flame_pattern_index, |old| old
                .wrapping_sub(1));
            let underflow = part!(game, Mouth[mouth].casual_flame_pattern_index) & SIGN != 0;
            let address = if expired || underflow {
                part!(game, Mouth[mouth].casual_flame_pattern_index = NEW_PATTERN);
                rom_table_word!(
                    game,
                    CASUAL_FLAME_TIMERS,
                    part!(game, Mouth[mouth].casual_flame_pattern) as isize
                )
            } else {
                // The live index read precedes the loaded-image capture and live pattern read.
                (2 * i32::from(part!(game, Mouth[mouth].casual_flame_pattern_index))
                    + i32::from(rom_table_word!(
                        game,
                        CASUAL_FLAME_TIMERS,
                        part!(game, Mouth[mouth].casual_flame_pattern) as isize
                    ))) as u16
            };
            part!(
                game,
                Mouth[mouth].casual_flame_timer = native_record(address, game).word(2)
            );
            part!(game, Mouth[mouth].base; instruction_timer = 1, current_instruction = SPAWN_FLAME);
        } else {
            let pattern = game.phantoon_random() & CASUAL_FLAME_PATTERN_BITS;
            part!(game, Mouth[mouth].casual_flame_pattern = pattern);
            let address = rom_table_word!(game, CASUAL_FLAME_TIMERS, pattern as isize);
            let count = native_record(address, game).word(0);
            part!(game, Mouth[mouth].casual_flame_pattern_index = count);
            part!(
                game,
                Mouth[mouth].casual_flame_timer =
                    native_record((2 * i32::from(count) + i32::from(address)) as u16, game).word(2)
            );
        }
    }
}

/// Adjust speed and move Phantoon in figure-8 ($A7:D0F1), then handle its
/// casual flames. Was `advance_figure_eight`.
pub fn fly_figure_eight(object: u16, game: &mut (impl State + ?Sized)) {
    let reversed = part!(game, Eye[EYE].movement_reversed) != 0;
    update_speed(reversed, game);
    move_in_figure_eight(reversed, game);
    handle_casual_flames(object, game);
}
