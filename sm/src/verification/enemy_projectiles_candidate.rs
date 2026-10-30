//! Candidate adapter for the frozen actual-old 354-case observations.
//! Declare as a private child of enemy_projectiles to exercise real production paths.
//! This first tranche does not cover all setup/pre-action/contact/drawing families.
use super::{
    ClearEprojs, DisableEprojs, EnableEprojs, EprojRunAll, spawn_with_enemy_gfx,
    spawn_with_room_gfx,
};
use sm_game::enemy_script::EnemyScript;

// Candidate-only adapters invoke the maintained implementation and publish the
// exclusive owner on normal return or unwind before the unchanged observations.
fn CallEprojInstr(ea: u32, k: u16, script: &EnemyScript, game: &mut Game) -> Option<usize> {
    let EnemyScript::At { image, .. } = script else {
        unreachable!()
    };
    super::with_execution(game, |ctx| super::commands::dispatch(ea, k, script, ctx))
        .resolve(image, 0x28000)
        .map(|next| {
            let EnemyScript::At { position, .. } = next else {
                unreachable!()
            };
            position
        })
}
fn move_with_velocity(k: u16, game: &mut Game) -> u16 {
    super::with_execution(game, |ctx| super::shared::move_with_velocity(k, ctx))
}
fn move_with_x_velocity(k: u16, game: &mut Game) -> u16 {
    super::with_execution(game, |ctx| super::shared::move_with_x_velocity(k, ctx))
}
fn move_with_y_velocity(k: u16, game: &mut Game) -> u16 {
    super::with_execution(game, |ctx| super::shared::move_with_y_velocity(k, ctx))
}
use crate::game_state::Game;
use crate::ram_map::{word_arrays as a, words as w};
use crate::rom::RomImage;
use crate::types::{EnemyData, Point16U, Rect16U};
use std::panic::{AssertUnwindSafe, catch_unwind};

// Explicitly selected field/control commands: no unchecked native operand loads.
const COMMANDS: &[u32] = &[
    8814932, 8814937, 8814945, 8814954, 8815061, 8815046, 8815054, 8815019, 8815024, 8815070,
    8815152, 8815164, 8815176, 8815186, 8815196, 8815206, 8815216, 8815226, 8815236, 8815246,
    8815256, 8815265, 8826770, 8826678, 8827958, 8823742, 8822864, 8828917, 8828993, 8833232,
    8831412, 8835527, 8838234, 8840499, 8831347, 8836594, 8836650,
];
const QUEUES: &[u32] = &[
    8815357, 8815369, 8815378, 8815387, 8815396, 8815405, 8815414, 8815423, 8815432, 8815441,
    8815450, 8815459, 8815468, 8815477, 8815486, 8815495,
];
const CORE: u32 = 24;
const QUEUE_START: u32 = CORE + COMMANDS.len() as u32 * 6;
const MOTION_START: u32 = QUEUE_START + QUEUES.len() as u32 * 4;
const SHORT_START: u32 = MOTION_START + 36;
const ORIGINAL_CASES: u32 = SHORT_START + 8;
#[path = "candidate_attachment_cases.rs"]
mod attachment_cases;
#[path = "candidate_collision_cases.rs"]
mod collision_cases;
#[path = "candidate_drop_contact_cases.rs"]
mod drop_contact_cases;
#[path = "candidate_family_cases.rs"]
mod family_cases;
#[path = "candidate_pre_family_cases.rs"]
mod pre_family_cases;
#[path = "candidate_respawn_cases.rs"]
mod respawn_cases;
#[path = "candidate_setup_family_cases.rs"]
mod setup_family_cases;
#[path = "candidate_waffle_cases.rs"]
mod waffle_cases;
pub(crate) const CASES: u32 = ORIGINAL_CASES
    + family_cases::CASES
    + attachment_cases::CASES
    + drop_contact_cases::CASES
    + waffle_cases::CASES
    + respawn_cases::CASES
    + pre_family_cases::CASES
    + collision_cases::CASES
    + setup_family_cases::CASES;
pub(crate) const OUTPUT_BYTES: usize = 131099;

fn put(game: &mut Game, at: usize, value: u16) {
    game.ram().copy_in(at, &value.to_le_bytes());
}
fn slot(game: &mut Game, base: usize, index: u16, value: u16) {
    put(game, base + usize::from(index & !1), value);
}
fn bytes(image: &RomImage, at: usize, values: &[u8]) {
    for (offset, value) in values.iter().copied().enumerate() {
        if at + offset < image.len() {
            image.write(at + offset, value);
        }
    }
}
fn script_word(image: &RomImage, at: u16, value: u16) {
    bytes(image, 0x28000 + usize::from(at), &value.to_ne_bytes());
}
fn definition(image: &RomImage, at: u16, setup: u16) {
    let base = crate::rtl::lorom_offset(0x860000 | u32::from(at)) as usize;
    for (offset, value) in [setup, 0x8170, 0x9000, 0x0705, 0xa345, 0, 0]
        .into_iter()
        .enumerate()
    {
        bytes(image, base + offset * 2, &value.to_ne_bytes());
    }
}
fn seed(game: &mut Game, image: &RomImage) {
    for at in 0..game.ram().len() {
        game.ram()
            .write_byte(at, (at.wrapping_mul(37) ^ (at >> 8)) as u8);
    }
    game.eproj_spawn_rect = Rect16U {
        x: 0xfff0,
        y: 0x1234,
        w: 0x30,
        h: 0xfedc,
    };
    game.eproj_spawn_r22 = 0x9246;
    game.eproj_spawn_pt = Point16U {
        x: 0x4567,
        y: 0x89ab,
    };
    game.eproj_spawn_varE24 = 0xcdef;
    for (at, value) in [
        (w::CUR_ENEMY_INDEX, 0),
        (w::EPROJ_ENABLE_FLAG, 0x9234),
        (w::EPROJ_INDEX, 34),
        (w::EPROJ_INIT_PARAM_1, 0x5678),
        (w::EPROJ_INIT_PARAM_2, 0x9876),
        (w::GAME_STATE, 8),
        (w::DEBUG_DISABLE_SOUNDS, 0),
        (w::POWER_BOMB_EXPLOSION_STATUS, 0),
        (w::MUSIC_QUEUE_WRITE_POS, 0),
        (w::MUSIC_QUEUE_READ_POS, 0),
    ] {
        put(game, at, value);
    }
    for offset in 0..3 {
        game.ram().write_byte(0x643 + offset, 0);
        game.ram().write_byte(0x646 + offset, 0);
    }
    for (field, value) in [
        (core::mem::offset_of!(EnemyData, x_pos), 0x1234),
        (core::mem::offset_of!(EnemyData, y_pos), 0x5678),
        (core::mem::offset_of!(EnemyData, vram_tiles_index), 0x135),
        (core::mem::offset_of!(EnemyData, palette_index), 0xa00),
    ] {
        put(game, 0xf78 + field, value);
    }
    for index in (0..36).step_by(2) {
        slot(game, a::EPROJ_ID, index, 0x8100 + index);
        slot(game, a::EPROJ_PRE_INSTR, index, 0x8170);
        slot(game, a::EPROJ_INSTR_LIST_PTR, index, 0x9000);
        slot(game, a::EPROJ_INSTR_TIMERS, index, 1);
    }
    definition(image, 0x8000, 8826704u32 as u16);
    definition(image, 0x2000, 8826704u32 as u16);
    script_word(image, 0x9000, 3);
    script_word(image, 0x9002, 0x1234);
}

unsafe fn core_case(id: u32, game: &mut Game, image: &RomImage) -> Option<usize> {
    {
        match id {
            0 => EnableEprojs(game),
            1 => DisableEprojs(game),
            2 => ClearEprojs(game),
            3 => return Some(spawn_with_room_gfx(0xffff, 0x4321, game) as usize),
            4 => return Some(spawn_with_enemy_gfx(0x4321, 0, 0xffff, game) as usize),
            5..=12 | 23 => {
                if id != 9 {
                    slot(game, a::EPROJ_ID, if id == 6 { 0 } else { 34 }, 0);
                }
                if id == 7 {
                    slot(game, a::EPROJ_ID, 34, 0x8122);
                    slot(game, a::EPROJ_ID, 4, 0);
                    slot(game, a::EPROJ_ID, 28, 0);
                }
                if id == 9 || id == 10 {
                    game.loaded_rom = None;
                }
                if id == 23 {
                    definition(image, 0x8000, 8822936u32 as u16);
                }
                return Some(if id == 6 {
                    spawn_with_enemy_gfx(0x4321, 0, 0x8000, game)
                } else {
                    spawn_with_room_gfx(if id == 8 { 0x2000 } else { 0x8000 }, 0x4321, game)
                } as usize);
            }
            13..=22 => {
                for index in (0..36).step_by(2) {
                    slot(game, a::EPROJ_ID, index, 0);
                }
                if id != 14 {
                    slot(game, a::EPROJ_ID, 34, 0x8000);
                }
                if id <= 16 {
                    game.loaded_rom = None;
                }
                if id == 13 {
                    put(game, w::EPROJ_ENABLE_FLAG, 0x1234);
                }
                if id == 15 {
                    slot(game, a::EPROJ_INSTR_TIMERS, 34, 0);
                }
                if id == 18 {
                    script_word(image, 0x9000, 8814932u32 as u16);
                }
                if id == 19 {
                    script_word(image, 0x9000, 8814937u32 as u16);
                }
                if id == 20 {
                    script_word(image, 0x9000, 8815019u32 as u16);
                    script_word(image, 0x9002, 0xfffe);
                    script_word(image, 0xfffe, 4);
                    bytes(image, 0x38000, &0x1234u16.to_ne_bytes());
                }
                if id == 21 {
                    put(
                        game,
                        w::MUSIC_QUEUE_WRITE_POS,
                        (w::EPROJ_INDEX - a::MUSIC_QUEUE_TRACK) as u16,
                    );
                    script_word(image, 0x9000, 8815357u32 as u16);
                    bytes(image, 0x31002, &[0]);
                    script_word(image, 0x9003, 3);
                    script_word(image, 0x9005, 0x1234);
                }
                if id == 22 {
                    script_word(image, 0x9000, 8815024u32 as u16);
                    bytes(image, 0x31002, &[2]);
                    script_word(image, 0x9004, 3);
                    script_word(image, 0x9006, 0x1234);
                }
                EprojRunAll(game);
            }
            _ => unreachable!(),
        }
        None
    }
}

unsafe fn execute(id: u32, game: &mut Game, image: &RomImage) -> Option<usize> {
    if id >= ORIGINAL_CASES {
        let id = id - ORIGINAL_CASES;
        unsafe {
            if id < family_cases::CASES {
                return family_cases::execute(id, game, image);
            }
            let id = id - family_cases::CASES;
            if id < attachment_cases::CASES {
                return attachment_cases::execute(id, game);
            }
            let id = id - attachment_cases::CASES;
            if id < drop_contact_cases::CASES {
                return drop_contact_cases::execute(id, game, image);
            }
            let id = id - drop_contact_cases::CASES;
            if id < waffle_cases::CASES {
                return waffle_cases::execute(id, game, image);
            }
            let id = id - waffle_cases::CASES;
            if id < respawn_cases::CASES {
                return respawn_cases::execute(id, game, image);
            }
            let id = id - respawn_cases::CASES;
            if id < pre_family_cases::CASES {
                return pre_family_cases::execute(id, game);
            }
            let id = id - pre_family_cases::CASES;
            if id < collision_cases::CASES {
                return collision_cases::execute(id, game, image);
            }
            return setup_family_cases::execute(id - collision_cases::CASES, game, image);
        }
    }
    unsafe { original_execute(id, game, image) }
}
unsafe fn original_execute(id: u32, game: &mut Game, image: &RomImage) -> Option<usize> {
    unsafe {
        if id < CORE {
            return core_case(id, game, image);
        }
        if id < QUEUE_START {
            let n = id - CORE;
            let variant = (n % 6) as usize;
            let index: u16 = [0, 1, 34, 0xffff, 2, 4][variant];
            let cursor: u16 = [0x9000, 0x9001, 0xfffe, 0, 0x9000, 0x9000][variant];
            let operand: u16 = [0, 0x9234, 0xffff, 1, 0x7fff, 0xff80][variant];
            for offset in 0..6 {
                bytes(
                    image,
                    0x28000 + usize::from(cursor) + offset * 2,
                    &operand.wrapping_add(offset as u16).to_ne_bytes(),
                );
            }
            slot(
                game,
                a::EPROJ_TIMERS,
                index,
                [0, 1, 0xffff, 2, 1, 2][variant],
            );
            slot(game, a::EPROJ_E, index, (variant * 2) as u16);
            put(game, w::EPROJ_INDEX, if variant == 4 { 34 } else { 0 });
            put(game, w::NMI_FRAME_COUNTER_WORD, (variant & 1) as u16);
            return CallEprojInstr(
                COMMANDS[(n / 6) as usize],
                index,
                &EnemyScript::at(image.clone(), 0x28000 + usize::from(cursor as u16)),
                game,
            );
        }
        if id < MOTION_START {
            let n = id - QUEUE_START;
            let variant = n % 4;
            let cursor: u16 = if variant == 3 { 0xffff } else { 0x9000 };
            bytes(image, 0x28000 + usize::from(cursor), &[0x23]);
            if variant == 1 {
                put(game, w::GAME_STATE, 40);
            }
            if variant == 2 {
                for channel in 0..3 {
                    game.ram().write_byte(0x646 + channel, 15);
                }
                put(
                    game,
                    w::MUSIC_QUEUE_WRITE_POS,
                    (w::EPROJ_INDEX - a::MUSIC_QUEUE_TRACK) as u16,
                );
            }
            if variant == 3 {
                put(game, w::POWER_BOMB_EXPLOSION_STATUS, 0x8000);
            }
            return CallEprojInstr(
                QUEUES[(n / 4) as usize],
                34,
                &EnemyScript::at(image.clone(), 0x28000 + usize::from(cursor as u16)),
                game,
            );
        }
        if id < SHORT_START {
            let n = id - MOTION_START;
            let index = [0, 1, 34, 0xffff][(n / 9) as usize];
            let velocity: u16 = [0x017f, 0x80ff, 0xffff][((n / 3) % 3) as usize];
            slot(game, a::EPROJ_X_VEL, index, velocity);
            slot(game, a::EPROJ_Y_VEL, index, velocity.wrapping_neg());
            slot(game, a::EPROJ_X_POS, index, 0xffff);
            slot(game, a::EPROJ_Y_POS, index, 0);
            game.ram()
                .write_byte(a::EPROJ_X_SUBPOS + usize::from(index) + 1, 0xf0);
            game.ram()
                .write_byte(a::EPROJ_Y_SUBPOS + usize::from(index) + 1, 0x80);
            return Some(match n % 3 {
                0 => move_with_velocity(index, game),
                1 => move_with_x_velocity(index, game),
                _ => move_with_y_velocity(index, game),
            } as usize);
        }
        let n = id - SHORT_START;
        let command = [
            8815046, 8815046, 8815061, 8815019, 8815024, 8814932, 8814937, 8815357,
        ][n as usize];
        slot(game, a::EPROJ_TIMERS, 1, if n == 0 { 1 } else { 0 });
        let short = RomImage::zeroed(0x31000 + if n == 4 || n == 7 { 1 } else { 0 });
        if short.len() > 0x31000 {
            short.write(0x31000, 0xff);
        }
        // The executor captures its instruction image separately from Game.
        CallEprojInstr(
            command,
            1,
            &EnemyScript::at(short, 0x28000 + usize::from(0x9000 as u16)),
            game,
        )
    }
}

pub(crate) fn capture_case(id: u32) -> Option<Vec<u8>> {
    if id >= CASES {
        return None;
    }
    let image = RomImage::zeroed(match id {
        11 => 0x30005,
        12 => 0x30009,
        _ if id >= ORIGINAL_CASES => family_cases::IMAGE_BYTES,
        _ => 0x80000,
    });
    let mut game = Game::new();
    seed(&mut game, &image);
    game.loaded_rom = Some(image.clone());
    game.register_native_rom(image.clone());
    let result = catch_unwind(AssertUnwindSafe(|| unsafe {
        execute(id, &mut game, &image)
    }));
    let expected_panic =
        matches!(id, 10 | 11 | 12 | 16) || (SHORT_START + 1..=SHORT_START + 3).contains(&id);
    assert_eq!(
        result.is_err(),
        expected_panic,
        "enemy-projectile core case {id}"
    );
    assert_eq!(
        game.g_fail,
        id == 8,
        "enemy-projectile diagnostic case {id}"
    );
    let mut output = (0..game.ram().len())
        .map(|at| game.ram().read_byte(at))
        .collect::<Vec<_>>();
    for value in [
        game.eproj_spawn_rect.x,
        game.eproj_spawn_rect.y,
        game.eproj_spawn_rect.w,
        game.eproj_spawn_rect.h,
        game.eproj_spawn_r22,
        game.eproj_spawn_pt.x,
        game.eproj_spawn_pt.y,
        game.eproj_spawn_varE24,
    ] {
        output.extend_from_slice(&value.to_le_bytes());
    }
    let returned = result.unwrap_or(None);
    output.push(u8::from(returned.is_some()));
    output.extend_from_slice(&(returned.unwrap_or(0) as u64).to_le_bytes());
    output.push(u8::from(expected_panic));
    output.push(u8::from(game.g_fail));
    assert_eq!(output.len(), OUTPUT_BYTES);
    Some(output)
}

#[path = "qualified_checks.rs"]
pub(crate) mod qualified_checks;
#[path = "qualified_lavaquake_checks.rs"]
pub(crate) mod qualified_lavaquake_checks;
#[path = "qualified_motion_checks.rs"]
pub(crate) mod qualified_motion_checks;
#[path = "qualified_turret_checks.rs"]
pub(crate) mod qualified_turret_checks;
#[path = "candidate_supplemental_cases.rs"]
pub(crate) mod supplemental_cases;
