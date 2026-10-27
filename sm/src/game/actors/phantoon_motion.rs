//! Figure-eight motion and animation timing over the authoritative actor bytes.
#![forbid(unsafe_code)]
use crate::{
    game::{enemy_layouts::EnemyLayout8 as Phantoon, session_runtime::native_rom_diagnostic},
    game_state::Game,
    ram_map::slot,
    rom::{RomRecord, lorom_offset},
    rtl::rom_table_word,
};

const MOVEMENT_DATA: u16 = 0xe3d2;
const ANIMATION_TABLE: u32 = 0xa7ccfd;
const ANIMATION_INSTRUCTION: u16 = 0xcceb;

fn native_record(address: u16, game: &mut Game) -> RomRecord {
    let address = 0xa70000 | u32::from(address);
    if address & 0x8000 == 0 {
        native_rom_diagnostic(address, game);
    }
    RomRecord {
        image: game.capture_native_rom(),
        base: lorom_offset(address) as usize,
    }
}

fn adjust_speed(right: bool, game: &mut Game) {
    let phase = slot!(game, Phantoon[0].var_D);
    // The bounds and restart values differ between the two clockwise halves.
    let (acceleration, bound, clamp, lower, restart): (i32, u16, u16, bool, bool) =
        match (right, phase) {
            (false, 0) => (0x600, 2, 1, false, false),
            (false, p) if p & 1 != 0 => (0x1000, 7, 7, false, false),
            (false, _) => (-0x1000, 0, 1, true, true),
            (true, 0) => (-0x600, 0xfffe, 0, true, false),
            (true, p) if p & 1 != 0 => (-0x1000, 0xfff9, 0xfffa, true, false),
            (true, _) => (0x1000, 0, 0, false, true),
        };
    slot!(game, Phantoon[0](var_C, var_B) += acceleration as u32);
    let reached = if lower {
        slot!(game, Phantoon[0].var_C) == bound
            || (slot!(game, Phantoon[0].var_C) as i16).wrapping_sub(bound as i16) < 0
    } else {
        (slot!(game, Phantoon[0].var_C) as i16).wrapping_sub(bound as i16) >= 0
    };
    if reached {
        slot!(game, Phantoon[0].var_C = clamp);
        slot!(game, Phantoon[0].var_B = 0);
        slot!(
            game,
            Phantoon[0].var_D = if restart {
                0
            } else {
                slot!(game, Phantoon[0].var_D).wrapping_add(1)
            }
        );
    }
}

fn offset_position(position: u16, byte: u8, right: bool) -> u16 {
    let delta = (byte as i8) as u16;
    if right {
        position.wrapping_sub(delta)
    } else {
        position.wrapping_add(delta)
    }
}

fn move_figure_eight(right: bool, game: &mut Game) {
    let speed = slot!(game, Phantoon[0].var_C);
    let mut remaining = i32::from(if right {
        (-i32::from(speed)) as u16
    } else {
        speed
    });
    while remaining != 0 {
        let record = native_record(
            (2 * i32::from(slot!(game, Phantoon[0].var_A)) + i32::from(MOVEMENT_DATA)) as u16,
            game,
        );
        // Each position read precedes its lazy ROM byte; X is stored before Y.
        slot!(
            game,
            Phantoon[0].base.x_pos =
                offset_position(slot!(game, Phantoon[0].base.x_pos), record.byte(0), right)
        );
        slot!(
            game,
            Phantoon[0].base.y_pos =
                offset_position(slot!(game, Phantoon[0].base.y_pos), record.byte(1), right)
        );
        slot!(
            game,
            Phantoon[0].var_A = if right {
                slot!(game, Phantoon[0].var_A).wrapping_sub(1)
            } else {
                slot!(game, Phantoon[0].var_A).wrapping_add(1)
            }
        );
        if right {
            if slot!(game, Phantoon[0].var_A) & 0x8000 != 0 {
                slot!(game, Phantoon[0].var_A = 533);
            }
        } else if (i32::from(slot!(game, Phantoon[0].var_A)) - 534) & 0x8000 == 0 {
            slot!(game, Phantoon[0].var_A = 0);
        }
        remaining -= 1;
    }
}

fn advance_animation(object: u16, game: &mut Game) {
    let part = i32::from(object) + 192;
    let expired = slot!(game, Phantoon[part].var_B) == 1;
    slot!(
        game,
        Phantoon[part].var_B = slot!(game, Phantoon[part].var_B).wrapping_sub(1)
    );
    let underflow = slot!(game, Phantoon[part].var_B) & 0x8000 != 0;
    if expired || underflow {
        if slot!(game, Phantoon[part].var_C) & 0x8000 == 0 {
            let expired = slot!(game, Phantoon[part].var_C) == 1;
            slot!(
                game,
                Phantoon[part].var_C = slot!(game, Phantoon[part].var_C).wrapping_sub(1)
            );
            let underflow = slot!(game, Phantoon[part].var_C) & 0x8000 != 0;
            let address = if expired || underflow {
                slot!(game, Phantoon[part].var_C = u16::MAX);
                rom_table_word!(
                    game,
                    ANIMATION_TABLE,
                    slot!(game, Phantoon[part].var_A) as isize
                )
            } else {
                // The live C read precedes the loaded-image capture and live A read.
                (2 * i32::from(slot!(game, Phantoon[part].var_C))
                    + i32::from(rom_table_word!(
                        game,
                        ANIMATION_TABLE,
                        slot!(game, Phantoon[part].var_A) as isize
                    ))) as u16
            };
            slot!(
                game,
                Phantoon[part].var_B = native_record(address, game).word(2)
            );
            slot!(game, Phantoon[part].base; instruction_timer = 1, current_instruction = ANIMATION_INSTRUCTION);
        } else {
            let pattern = game.next_random() & 3;
            slot!(game, Phantoon[part].var_A = pattern);
            let address = rom_table_word!(game, ANIMATION_TABLE, pattern as isize);
            let count = native_record(address, game).word(0);
            slot!(game, Phantoon[part].var_C = count);
            slot!(
                game,
                Phantoon[part].var_B =
                    native_record((2 * i32::from(count) + i32::from(address)) as u16, game).word(2)
            );
        }
    }
}

pub(in crate::game) fn advance_figure_eight(object: u16, game: &mut Game) {
    let right = slot!(game, Phantoon[0x40].var_C) != 0;
    adjust_speed(right, game);
    move_figure_eight(right, game);
    advance_animation(object, game);
}
