//! Phantoon's figure-8 flight and its casual flames (bank $A7:CFCA-$A7:D2D0).
#![forbid(unsafe_code)]
use crate::actors::boss_lease::BossState;
use crate::actors::layouts::part_view;
use crate::actors::phantoon_actions::Mouth;
use crate::actors::session_slots::session_slot;
use crate::cartridge::rom_table_word;
use sm_data::cartridge::RomRecord;

part_view! {
    /// The body's view of its slot.
    Body: EnemyLayout8 {
        words: [
            movement_index, speed_subpixel, speed, movement_stage, var_E, var_F, parameter_1,
            parameter_2,
        ],
    }
}

part_view! {
    /// The eye's view of its slot.
    Eye: EnemyLayout8 {
        words: [var_A, var_B, movement_reversed, var_D, var_E, var_F, parameter_1, parameter_2],
    }
}

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

fn native_record(address: u16, game: &mut (impl BossState + ?Sized)) -> RomRecord {
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
fn update_speed(reversed: bool, game: &mut (impl BossState + ?Sized)) {
    let Stage {
        acceleration,
        bound,
        clamp,
        lower,
        restart,
    } = stage(reversed, session_slot!(game, Body[BODY].movement_stage));
    session_slot!(
        game,
        Body[BODY](speed, speed_subpixel) += acceleration as u32
    );
    let reached = if lower {
        session_slot!(game, Body[BODY].speed) == bound
            || (session_slot!(game, Body[BODY].speed) as i16).wrapping_sub(bound as i16) < 0
    } else {
        (session_slot!(game, Body[BODY].speed) as i16).wrapping_sub(bound as i16) >= 0
    };
    if reached {
        session_slot!(game, Body[BODY].speed = clamp);
        session_slot!(game, Body[BODY].speed_subpixel = 0);
        session_slot!(
            game,
            Body[BODY].movement_stage = if restart {
                0
            } else {
                session_slot!(game, Body[BODY].movement_stage).wrapping_add(1)
            }
        );
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
fn move_in_figure_eight(reversed: bool, game: &mut (impl BossState + ?Sized)) {
    let speed = session_slot!(game, Body[BODY].speed);
    let mut remaining = i32::from(if reversed {
        (-i32::from(speed)) as u16
    } else {
        speed
    });
    while remaining != 0 {
        let record = native_record(
            (2 * i32::from(session_slot!(game, Body[BODY].movement_index))
                + i32::from(MOVEMENT_DATA)) as u16,
            game,
        );
        // Each position read precedes its lazy ROM byte; X is stored before Y.
        session_slot!(
            game,
            Body[BODY].base.x_pos = offset_position(
                session_slot!(game, Body[BODY].base.x_pos),
                record.byte(0),
                reversed
            )
        );
        session_slot!(
            game,
            Body[BODY].base.y_pos = offset_position(
                session_slot!(game, Body[BODY].base.y_pos),
                record.byte(1),
                reversed
            )
        );
        session_slot!(
            game,
            Body[BODY].movement_index = if reversed {
                session_slot!(game, Body[BODY].movement_index).wrapping_sub(1)
            } else {
                session_slot!(game, Body[BODY].movement_index).wrapping_add(1)
            }
        );
        if reversed {
            if session_slot!(game, Body[BODY].movement_index) & SIGN != 0 {
                session_slot!(game, Body[BODY].movement_index = (STEPS - 1) as u16);
            }
        } else if (i32::from(session_slot!(game, Body[BODY].movement_index)) - STEPS) & SIGN as i32
            == 0
        {
            session_slot!(game, Body[BODY].movement_index = 0);
        }
        remaining -= 1;
    }
}

/// Handle casual flames ($A7:CFCA): count down the flame timer; at zero, step
/// through the pattern's timers, setting the mouth to spawn a flame, or pick a
/// new pattern once it is spent.
fn handle_casual_flames(object: u16, game: &mut (impl BossState + ?Sized)) {
    let mouth = i32::from(object) + MOUTH_OFFSET;
    let expired = session_slot!(game, Mouth[mouth].casual_flame_timer) == 1;
    session_slot!(
        game,
        Mouth[mouth].casual_flame_timer =
            session_slot!(game, Mouth[mouth].casual_flame_timer).wrapping_sub(1)
    );
    let underflow = session_slot!(game, Mouth[mouth].casual_flame_timer) & SIGN != 0;
    if expired || underflow {
        if session_slot!(game, Mouth[mouth].casual_flame_pattern_index) & SIGN == 0 {
            let expired = session_slot!(game, Mouth[mouth].casual_flame_pattern_index) == 1;
            session_slot!(
                game,
                Mouth[mouth].casual_flame_pattern_index =
                    session_slot!(game, Mouth[mouth].casual_flame_pattern_index).wrapping_sub(1)
            );
            let underflow =
                session_slot!(game, Mouth[mouth].casual_flame_pattern_index) & SIGN != 0;
            let address = if expired || underflow {
                session_slot!(game, Mouth[mouth].casual_flame_pattern_index = NEW_PATTERN);
                rom_table_word!(
                    game,
                    CASUAL_FLAME_TIMERS,
                    session_slot!(game, Mouth[mouth].casual_flame_pattern) as isize
                )
            } else {
                // The live index read precedes the loaded-image capture and live pattern read.
                (2 * i32::from(session_slot!(game, Mouth[mouth].casual_flame_pattern_index))
                    + i32::from(rom_table_word!(
                        game,
                        CASUAL_FLAME_TIMERS,
                        session_slot!(game, Mouth[mouth].casual_flame_pattern) as isize
                    ))) as u16
            };
            session_slot!(
                game,
                Mouth[mouth].casual_flame_timer = native_record(address, game).word(2)
            );
            session_slot!(game, Mouth[mouth].base; instruction_timer = 1, current_instruction = SPAWN_FLAME);
        } else {
            let pattern = game.roll_random() & CASUAL_FLAME_PATTERN_BITS;
            session_slot!(game, Mouth[mouth].casual_flame_pattern = pattern);
            let address = rom_table_word!(game, CASUAL_FLAME_TIMERS, pattern as isize);
            let count = native_record(address, game).word(0);
            session_slot!(game, Mouth[mouth].casual_flame_pattern_index = count);
            session_slot!(
                game,
                Mouth[mouth].casual_flame_timer =
                    native_record((2 * i32::from(count) + i32::from(address)) as u16, game).word(2)
            );
        }
    }
}

/// Adjust speed and move Phantoon in figure-8 ($A7:D0F1), then handle its
/// casual flames. Was `advance_figure_eight`.
pub fn fly_figure_eight(object: u16, game: &mut (impl BossState + ?Sized)) {
    let reversed = session_slot!(game, Eye[EYE].movement_reversed) != 0;
    update_speed(reversed, game);
    move_in_figure_eight(reversed, game);
    handle_casual_flames(object, game);
}
