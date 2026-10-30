//! Norfair Lava Man's three cooperating parts, lava motion and palette cycle.
//! Native actor/spawn words and little-endian queues remain authoritative.
#![forbid(unsafe_code)]
use super::access::ActorKey as Key;
use super::{api, combat};
use crate::game::enemy_layouts::EnemyLayout40 as Lava;
use crate::game::enemy_projectiles::SpawnEprojWithGfx;
use crate::game::enemy_speeds::enemy_speed;
use crate::game::imported::{Imported, load};
use crate::game::music::QueueSfx2_Max6;
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
use crate::ram_map::{slot, word};
use sm_data::magdollite::{
    ARM_BYTES, ARM_GROWTH_ADDRESS, ARM_INSTRUCTION_ADDRESS, ARM_Y_OFFSET_ADDRESS, ArmSteps,
    PALETTE_ADDRESS, PALETTE_BYTES, Palettes, import_palettes,
};
use sm_game::enemy_script::EnemyScript;

#[derive(Default)]
pub(crate) struct ImportedTables {
    palettes: Option<Imported<Palettes>>,
    arm_steps: Option<Imported<ArmSteps>>,
}

pub(crate) fn import(image: &crate::rom::RomImage) -> ImportedTables {
    ImportedTables {
        palettes: Imported::import(image, PALETTE_ADDRESS, PALETTE_BYTES, import_palettes),
        arm_steps: Imported::import(image, ARM_GROWTH_ADDRESS, ARM_BYTES, ArmSteps::import),
    }
}

/// Colours 9-12 of the glow palette whose colour 0 is word `source`.
fn glow_colors(game: &Game, source: isize) -> [u16; 4] {
    Imported::read(
        &game.magdollite.palettes,
        game,
        |table| {
            let palette = table.row(crate::game::imported::row_of(source, 16)?)?;
            Some(core::array::from_fn(|i| palette[9 + i]))
        },
        |rom| core::array::from_fn(|i| load(rom, PALETTE_ADDRESS, source + 9 + i as isize)),
    )
}

/// Arm step `index`'s field; other images and indices run the original load
/// at `address`.
fn arm(game: &Game, address: u32, index: isize) -> u16 {
    Imported::read(
        &game.magdollite.arm_steps,
        game,
        |table| {
            let step = table.step(usize::try_from(index).ok()?)?;
            Some(match address {
                ARM_GROWTH_ADDRESS => step.growth,
                ARM_INSTRUCTION_ADDRESS => step.instruction_list,
                _ => step.y_offset,
            })
        },
        |rom| load(rom, address, index),
    )
}

fn native_word(at: usize, game: &Game) -> u16 {
    u16::from_ne_bytes([game.ram().read_byte(at), game.ram().read_byte(at + 1)])
}
fn spawn(k: u16) -> usize {
    0x7000 + usize::from(k) + 31 * core::mem::size_of::<crate::types::EnemySpawnData>()
}
fn player_left(game: &mut Game) -> bool {
    let key = Key::from_legacy(word!(game.cur_enemy_index));
    api::player_delta(&Runtime::new(game), key, api::Axis::X) & 0x8000 != 0
}
fn animate(game: &mut Game) {
    let E = word!(game.cur_enemy_index) as i32;
    let instruction = slot!(game, Lava[E].var_D);
    if instruction != slot!(game, Lava[E].var_C) {
        slot!(game, Lava[E].base.current_instruction = instruction);
        slot!(game, Lava[E].var_C = instruction);
        slot!(game, Lava[E].base; instruction_timer = 1, timer = 0);
    }
}
fn initialize_base(game: &mut Game) {
    let E = word!(game.cur_enemy_index) as i32;
    slot!(game, Lava[E]; var_C = 0, var_00 = 0, var_02 = 0);
    slot!(game, Lava[E].var_03 = slot!(game, Lava[E].base.y_pos));
    if player_left(game) {
        slot!(game, Lava[E].var_00 = 1);
    }
    slot!(game, Lava[E].var_D = 0xac9c);
    if slot!(game, Lava[E].var_00) == 0 {
        slot!(game, Lava[E].var_D = 0xad3c);
    }
    animate(game);
    slot!(game, Lava[E].var_F = 0xb11a);
}
fn initialize_upper(game: &mut Game) {
    let E = word!(game.cur_enemy_index) as i32;
    slot!(game, Lava[E].var_03 = slot!(game, Lava[E].base.y_pos));
    slot!(game, Lava[E]; var_C = 0, var_0C = 0, var_0D = 1, var_D = 0xaddc);
    animate(game);
    slot!(game, Lava[E].base.y_pos, |old| old.wrapping_add(32));
    slot!(game, Lava[E].var_F = 0xb1dd);
    slot!(game, Lava[E].base.properties, |old| old | 0x100);
}
fn initialize_thrower(game: &mut Game) {
    let E = word!(game.cur_enemy_index) as i32;
    slot!(game, Lava[E].var_03 = slot!(game, Lava[E].base.y_pos));
    slot!(game, Lava[E].var_11 = slot!(game, Lava[E].base.x_pos));
    slot!(game, Lava[E]; var_C = 0, var_04 = 0, var_D = 0xae0c);
    animate(game);
    slot!(game, Lava[E].base.y_pos, |old| old.wrapping_add(32));
    slot!(game, Lava[E].var_F = 0xb30d);
    slot!(game, Lava[E].base.properties, |old| old | 0x100);
}
pub(crate) fn initialize(game: &mut Game) {
    let index = 2u16.wrapping_mul(slot!(
        game,
        Lava[word!(game.cur_enemy_index) as i32].parameter_1
    )) >> 1;
    let initialize: [fn(&mut Game); 3] = [initialize_base, initialize_upper, initialize_thrower];
    initialize[index as usize](game);
    let E = word!(game.cur_enemy_index) as i32;
    let speed = 4 * slot!(byte game, Lava[E].parameter_2[1]) as isize;
    slot!(game, Lava[E]; var_08 = enemy_speed!(game, Linear, speed), var_07 = enemy_speed!(game, Linear, speed + 1),
        var_0A = enemy_speed!(game, Linear, speed + 2), var_09 = enemy_speed!(game, Linear, speed + 3));
    let k = word!(game.cur_enemy_index) as i32;
    game.ram().copy_in(0x178c, &0xb0b2u16.to_ne_bytes());
    game.ram().copy_in(0x178e, &0xa8u16.to_ne_bytes());
    word!(
        game.variables_for_enemy_graphics_drawn_hook[0] =
            ((slot!(game, Lava[k].base.palette_index) & 0xe00) >> 4) + 256
    );
    word!(game.variables_for_enemy_graphics_drawn_hook[2] = 8);
    word!(game.variables_for_enemy_graphics_drawn_hook[1] = 0);
}
pub(crate) fn palette(game: &mut Game) {
    if word!(game.door_transition_flag_enemies) != 0
        || word!(game.variables_for_enemy_graphics_drawn_hook[2] -= 1, reread) != 0
    {
        return;
    }
    word!(game.variables_for_enemy_graphics_drawn_hook[2] = 8);
    word!(game.variables_for_enemy_graphics_drawn_hook[1] += 1);
    let source = 16 * (word!(game.variables_for_enemy_graphics_drawn_hook[1]) & 3) as isize;
    let colors: [u16; 4] = glow_colors(game, source);
    let target = word!(game.variables_for_enemy_graphics_drawn_hook[0]) as usize >> 1;
    for (i, color) in colors.into_iter().enumerate() {
        word!(game.palette_buffer[target + 9 + i] = color);
    }
}
pub(crate) fn state_address(game: &mut Game) -> u32 {
    let E = word!(game.cur_enemy_index) as i32;
    slot!(game, Lava[E].var_04, |old| old.wrapping_sub(1));
    0xa80000 | u32::from(slot!(game, Lava[E].var_F))
}
pub(crate) fn damage(address: u32, game: &mut Game) {
    match address {
        11056128 => {
            combat::power_bomb_reaction(&mut Runtime::new(game), api::DeathHandling::Finalize)
        }
        11056134 => combat::contact_reaction(&mut Runtime::new(game), api::DeathHandling::Finalize),
        11056140 => combat::shot_reaction(&mut Runtime::new(game), api::ShotHandling::Finalize),
        _ => unreachable!("Lava Man damage {address:06x}"),
    }
    let E = word!(game.cur_enemy_index) as i32;
    let E1 = word!(game.cur_enemy_index) as i32 + 64;
    let E2 = word!(game.cur_enemy_index) as i32 + 128;
    if slot!(game, Lava[E].base.health) == 0 {
        slot!(game, Lava[E1].base.properties, |old| old | 0x200);
        slot!(game, Lava[E2].base.properties, |old| old | 0x200);
    }
    let frozen = slot!(game, Lava[E].base.frozen_timer);
    if frozen != 0 {
        slot!(game, Lava[E1].base.frozen_timer = frozen);
        slot!(game, Lava[E1].base.ai_handler_bits, |old| old | 4);
        slot!(game, Lava[E2].base.frozen_timer = frozen);
        slot!(game, Lava[E2].base.ai_handler_bits, |old| old | 4);
    }
}
fn select_animation(E: i32, left: u16, right: u16, game: &mut Game) {
    slot!(game, Lava[E].var_D = left);
    if !player_left(game) {
        slot!(game, Lava[E].var_D = right);
    }
    animate(game);
}
fn align_body(k: u16, game: &mut Game) {
    let index = k as usize >> 1;
    let height = word!(game.enemy_drawing_queue[index + 61])
        .wrapping_sub(slot!(game, Lava[k as i32].base.y_pos))
        .wrapping_add(2);
    word!(game.enemy_drawing_queue[index + 64] = height);
    if height.wrapping_sub(8) & 0x8000 != 0 {
        word!(game.enemy_drawing_queue[index + 64] = 8);
    }
}
/// Capture the continuation only after movement, at the former helper call boundary.
pub(crate) fn prepare_descent(game: &mut Game) -> (u16, Option<i32>) {
    let E = word!(game.cur_enemy_index) as i32;
    let source = spawn(word!(game.cur_enemy_index));
    let value =
        u32::from(native_word(source + 0x12, game)) + u32::from(slot!(game, Lava[E].base.y_subpos));
    slot!(game, Lava[E].base.y_subpos = value as u16);
    slot!(game, Lava[E].base.y_pos, |old| (u32::from(old)
        + u32::from(native_word(source + 0x14, game))
        + (value >> 16))
        as u16);
    slot!(game, Lava[E].var_E, |old| (u32::from(old)
        + u32::from(native_word(source + 0x14, game))
        + (value >> 16)) as u16);
    let k = word!(game.cur_enemy_index);
    let diagnostic = if -(slot!(game, Lava[k as i32].var_E) as i32) - 108 & 0x8000 == 0 {
        Some(-(slot!(game, Lava[k as i32].var_E) as i16 as i32))
    } else {
        None
    };
    (k, diagnostic)
}
pub(crate) fn finish_descent(k: u16, game: &mut Game) {
    let E = k as i32;
    let phase = slot!(game, Lava[E].var_B);
    let mut source = 0isize;
    let below_player = -(slot!(game, Lava[E].var_E) as i32) - 108 & 0x8000 == 0 || {
        source = (phase >> 1) as isize;
        ((slot!(game, Lava[E].base.y_pos) as i32
            - arm(game, ARM_Y_OFFSET_ADDRESS, source) as i32
            - word!(game.samus_y_pos) as i32) as i16)
            < 0
    };
    let advance = if below_player {
        slot!(game, Lava[E]; var_F = 0xb291, var_0C = 1);
        if ((-(slot!(game, Lava[E].var_E) as i32)
            - arm(game, ARM_GROWTH_ADDRESS, (phase >> 1) as isize) as i32) as i16)
            < 0
        {
            return;
        }
        true
    } else {
        (-(slot!(game, Lava[E].var_E) as i32) - arm(game, ARM_GROWTH_ADDRESS, source) as i32) as i16
            >= 0
    };
    if advance {
        slot!(game, Lava[E].var_B, |old| old.wrapping_add(1));
        slot!(game, Lava[E].var_B, |old| old.wrapping_add(1));
        slot!(game, Lava[E].base.y_pos, |old| old.wrapping_add(8));
        slot!(
            game,
            Lava[E].var_D = arm(
                game,
                ARM_INSTRUCTION_ADDRESS,
                (slot!(game, Lava[E].var_B) >> 1) as isize
            )
        );
        animate(game);
    }
}
pub(crate) fn ascend(game: &mut Game) {
    let E = word!(game.cur_enemy_index) as i32;
    let source = spawn(word!(game.cur_enemy_index));
    let carry = native_word(source + 0x0e, game)
        .overflowing_add(slot!(game, Lava[E].base.y_subpos))
        .1;
    slot!(
        game,
        Lava[E].base.y_subpos =
            native_word(source + 0x0e, game).wrapping_add(slot!(game, Lava[E].base.y_subpos))
    );
    slot!(game, Lava[E].base.y_pos, |old| (old as i32
        + native_word(source + 0x10, game) as i32
        + i32::from(carry))
        as u16);
    slot!(game, Lava[E].var_E, |old| (old as i32
        + native_word(source + 0x10, game) as i32
        + i32::from(carry))
        as u16);
    let k = word!(game.cur_enemy_index);
    let E = k as i32;
    let height = slot!(game, Lava[E].var_E) as i16;
    if height >= 0 {
        slot!(game, Lava[E].var_0D = 1);
        let at = spawn(k) + 4;
        game.ram().copy_in(at, &0u16.to_ne_bytes());
        slot!(game, Lava[E].var_F = 0xb1dd);
    } else if ((-(height as i32)
        - arm(
            game,
            ARM_GROWTH_ADDRESS,
            (slot!(game, Lava[E].var_B).wrapping_sub(2) >> 1) as isize,
        ) as i32) as i16)
        < 0
    {
        slot!(game, Lava[E].var_B, |old| old.wrapping_sub(2));
        slot!(game, Lava[E].base.y_pos, |old| old.wrapping_sub(8));
        slot!(
            game,
            Lava[E].var_D = arm(
                game,
                ARM_INSTRUCTION_ADDRESS,
                (slot!(game, Lava[E].var_B) >> 1) as isize
            )
        );
        animate(game);
    }
}
pub(crate) fn advance(state: u16, game: &mut Game) {
    let E = word!(game.cur_enemy_index) as i32;
    match state {
        0xb11a => {
            slot!(game, Lava[E]; var_00 = 0, var_D = 0xac9c);
            if !player_left(game) {
                slot!(game, Lava[E]; var_D = 0xad3c, var_00 = 1);
            }
            animate(game);
            let E2 = word!(game.cur_enemy_index) as i32 + 128;
            if slot!(game, Lava[E2].var_04) & 0x8000 != 0 {
                slot!(game, Lava[E2].var_04 = 0);
                let key = Key::from_legacy(word!(game.cur_enemy_index));
                let distance = slot!(byte game, Lava[E].parameter_2[0]) as u16;
                if api::player_within(&mut Runtime::new(game), key, api::Axis::X, distance) {
                    slot!(game, Lava[E].var_D = 0xacde);
                    if slot!(game, Lava[E].var_00) != 0 {
                        slot!(game, Lava[E].var_D = 0xad7e);
                    }
                    animate(game);
                    slot!(game, Lava[E].var_F = 0xb175);
                }
            }
        }
        0xb175 => {
            if slot!(game, Lava[E].var_01) == 0 {
                slot!(game, Lava[E].var_02 = 1);
                slot!(
                    game,
                    Lava[word!(game.cur_enemy_index) as i32 + 64].var_0D = 0
                );
                slot!(game, Lava[E].var_F = 0xb193);
            }
        }
        0xb193 => {
            if slot!(game, Lava[E].var_02) == 0 {
                select_animation(E, 0xad0c, 0xadac, game);
                slot!(game, Lava[E].var_F = 0xb1b8);
            }
        }
        0xb1b8 => {
            if slot!(game, Lava[E].var_01) == 0 {
                select_animation(E, 0xac9c, 0xad3c, game);
                slot!(game, Lava[E].var_F = 0xb11a);
            }
        }
        0xb1dd => {
            if slot!(game, Lava[E].var_0D) == 0 {
                slot!(game, Lava[E]; var_F = 0xb204, var_E = 0, var_0D = 0, var_B = 2);
                slot!(game, Lava[E].base.y_pos = slot!(game, Lava[E].var_03));
            }
        }
        0xb30d => {
            let at = 0xe84 + 2 * ((word!(game.cur_enemy_index) as usize >> 1) + 87);
            if native_word(at, game) == 0xb175 {
                slot!(
                    game,
                    Lava[word!(game.cur_enemy_index) as i32].var_F = 0xb356
                );
            }
        }
        0xb31f => {
            if slot!(game, Lava[E].var_01) == 0 {
                slot!(game, Lava[E].var_D = 0xae0c);
                animate(game);
                let at = 0xf68 + 2 * ((word!(game.cur_enemy_index) as usize >> 1) + 5);
                game.ram().copy_in(at, &0xb295u16.to_ne_bytes());
                slot!(game, Lava[E].var_F = 0xb3a7);
                let at = spawn(word!(game.cur_enemy_index)) + 0x18;
                game.ram().copy_in(at, &0u16.to_ne_bytes());
                slot!(game, Lava[E].base.x_pos = slot!(game, Lava[E].var_12));
                slot!(game, Lava[E].base.y_pos = slot!(game, Lava[E].var_13));
            }
            align_body(word!(game.cur_enemy_index), game);
        }
        0xb356 => {
            let at = spawn(word!(game.cur_enemy_index)) + 0x18;
            if native_word(at, game) != 0 {
                let E = word!(game.cur_enemy_index) as i32;
                slot!(game, Lava[E]; var_D = 0xacb0, var_B = 0);
                if !player_left(game) {
                    slot!(game, Lava[E]; var_D = 0xad50, var_B = 1);
                }
                animate(game);
                slot!(game, Lava[E]; var_F = 0xb31f, var_12 = slot!(game, Lava[E].base.x_pos), var_13 = slot!(game, Lava[E].base.y_pos));
            } else {
                let at = 0xe84 + 2 * ((word!(game.cur_enemy_index) as usize >> 1) + 93);
                let y = native_word(at, game);
                let offset = arm(game, ARM_Y_OFFSET_ADDRESS, {
                    let at = 0xf68 + 2 * ((word!(game.cur_enemy_index) as usize >> 1) + 1);
                    (native_word(at, game) >> 1) as isize
                });
                slot!(
                    game,
                    Lava[word!(game.cur_enemy_index) as i32].base.y_pos = y.wrapping_sub(offset)
                );
            }
            align_body(word!(game.cur_enemy_index), game);
        }
        0xb3a7 => {
            let index = word!(game.cur_enemy_index) as usize >> 1;
            if word!(game.enemy_drawing_queue[index + 87]) == 0xb11a {
                slot!(
                    game,
                    Lava[word!(game.cur_enemy_index) as i32].var_F = 0xb30d
                );
            } else {
                let y = word!(game.enemy_drawing_queue[index + 93]);
                let offset = arm(
                    game,
                    ARM_Y_OFFSET_ADDRESS,
                    (word!(game.enemy_drawing_queue_sizes[index + 1]) >> 1) as isize,
                );
                slot!(
                    game,
                    Lava[word!(game.cur_enemy_index) as i32].base.y_pos = y.wrapping_sub(offset)
                );
            }
            align_body(word!(game.cur_enemy_index), game);
        }
        _ => unreachable!("Lava Man state {state:04x}"),
    }
}

pub(crate) fn instruction(address: u32, script: EnemyScript, game: &mut Game) -> EnemyScript {
    if address == 11054610 {
        if !api::current_outside_screen(&Runtime::new(game)) {
            QueueSfx2_Max6(script.word(0), game);
        }
        return script.offset(1);
    }
    let E = word!(game.cur_enemy_index) as i32;
    match address {
        11054630 | 11054640 | 11054672 | 11054682 => {
            let delta = if matches!(address, 11054630 | 11054682) {
                1
            } else {
                u16::MAX
            };
            slot!(game, Lava[E].base.y_pos, |old| old.wrapping_add(delta));
            let other = if matches!(address, 11054630 | 11054640) {
                E
            } else {
                word!(game.cur_enemy_index) as i32 + 64
            };
            slot!(game, Lava[other].base.y_pos, |old| old.wrapping_add(delta));
        }
        11054650 | 11054661 => {
            slot!(game, Lava[E].var_01 = u16::from(address == 11054650));
        }
        11054692 | 11054742 => {
            let hiding = address == 11054742;
            let y = slot!(game, Lava[E].var_03).wrapping_add(if hiding { 4 } else { 24 });
            slot!(game, Lava[E].base.y_pos = y);
            let E1 = word!(game.cur_enemy_index) as i32 + 64;
            slot!(game, Lava[E1].base.y_pos = y);
            slot!(game, Lava[E1].base.properties, |old| if hiding {
                old | 0x100
            } else {
                old & !0x100
            });
            let E2 = word!(game.cur_enemy_index) as i32 + 128;
            slot!(game, Lava[E2].base.properties, |old| if hiding {
                old | 0x100
            } else {
                old & !0x100
            });
        }
        11054728 => {
            let y = slot!(game, Lava[E].var_03);
            slot!(game, Lava[E].base.y_pos = y);
            slot!(
                game,
                Lava[word!(game.cur_enemy_index) as i32 + 64].base.y_pos = y
            );
        }
        11054778 => {
            SpawnEprojWithGfx(
                slot!(game, Lava[E].var_B),
                word!(game.cur_enemy_index),
                crate::game::constants::bank_86::addr_kEproj_LavaThrownByLavaman as u16,
                game,
            );
        }
        11054794 | 11054820 | 11054846 | 11054872 => {
            let right = matches!(address, 11054794 | 11054846);
            slot!(
                game,
                Lava[E].base.x_pos = slot!(game, Lava[E].var_12).wrapping_add(if right {
                    8
                } else {
                    (-8i16) as u16
                })
            );
            slot!(
                game,
                Lava[E].base.y_pos = slot!(game, Lava[E].var_13)
                    .wrapping_sub(if address == 11054846 { 8 } else { 4 })
            );
        }
        11054916 => {
            slot!(game, Lava[E].var_04 = 256);
        }
        _ => unreachable!("Lava Man instruction {address:06x}"),
    }
    script
}
