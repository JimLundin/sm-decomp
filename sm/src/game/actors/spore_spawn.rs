//! Spore Spawn stem motion, native orbit words and scripted palettes.
//! RAM retains the boss-specific state while common actors/projectiles stay owned.
#![forbid(unsafe_code)]
use crate::game::actors::{api, combat};
use crate::game::enemy_projectiles::{self, SpawnEprojWithRoomGfx};
use crate::game::gameplay_flag_runtime as flags;
use crate::game::music::QueueSfx2_Max6;
use crate::game::session_runtime::{Runtime, actor_cosine, actor_direction_vector, actor_sine};
use crate::game::sprite_effect_runtime::create_sprite;
use crate::game_state::Game;
use crate::ram_map::{slot, word, word_arrays};
use crate::rtl::{abs16, words};
use crate::types::Point16U;
use sm_game::actors::layouts::EnemyLayout104 as Spore;
pub(crate) use sm_game::actors::spore_spawn::{
    clear_damaged_flag, harden, increase_max_x_radius, set_function,
    set_max_x_radius_and_angle_delta,
};
use sm_game::enemy_script::EnemyScript;
use sm_game::gameplay_flag_host::BossWordOrder;
use sm_game::geometry::Vector32;
use sm_game::room_mechanisms::{
    creation,
    host::{Contact, SpawnSource},
};

pub(crate) fn descend(game: &mut Game) {
    enemy_projectiles::update_spore_stem(game);
    let k = word!(game.cur_enemy_index) as i32;
    let y = slot!(game, Spore[k].base.y_pos).wrapping_add(1);
    slot!(game, Spore[k].base.y_pos = y);
    if y.wrapping_sub(624) & 0x8000 == 0 {
        slot!(game, Spore[k].base; current_instruction = 0xe6d5, instruction_timer = 1);
    }
    slot!(game, Spore[0]; var_0B = 48, var_0C = 1, var_0A = 192);
}
pub(crate) fn initialize(game: &mut Game) {
    word_transfers!(game, ascending, 16; le(target_palettes, 240) <- rom(0xa5e359, 0));
    for part in 0..4 {
        enemy_projectiles::SpawnEprojWithGfx(part, word!(game.cur_enemy_index), 0xde6c, game);
    }
    let k = word!(game.cur_enemy_index) as i32;
    slot!(game, Spore[k].var_04 = slot!(game, Spore[k].base.x_pos));
    slot!(
        game,
        Spore[k].var_05 = slot!(game, Spore[k].base.y_pos).wrapping_sub(72)
    );
    slot!(game, Spore[k].var_C = slot!(game, Spore[k].base.x_pos));
    slot!(game, Spore[k].var_D = slot!(game, Spore[k].base.y_pos));
    slot!(game, Spore[k].var_F = 0);
    if flags::boss_byte_at(word!(game.area_index), game) & 2 != 0 {
        slot!(game, Spore[0].base.current_instruction = 0xe6b9);
        slot!(game, Spore[0].var_A = 0xeb1a);
        slot!(game, Spore[0].base.properties, |old| old | 0x8000);
        enemy_projectiles::update_spore_stem(game);
        creation::spawn(
            &mut Runtime::new(game),
            SpawnSource::Hardcoded {
                x: 7,
                y: 30,
                header: 0xb793,
            },
            &mut Contact::Absent,
        );
        word!(game.scrolling_finished_hook = 0);
    } else {
        slot!(game, Spore[0].base.current_instruction = 0xe6c7);
        word!(game.flag_process_all_enemies = 0xffff);
        let spawn_key = word!(game.cur_enemy_index);
        slot!(game, Spore[k].var_A = 0xeb1a);
        word!(game.scrolling_finished_hook = 0x9589);
        slot!(game, Spore[k].base.y_pos, |old| old.wrapping_sub(128));
        for part in 0..4 {
            enemy_projectiles::SpawnEprojWithGfx(part, spawn_key, 0xde88, game);
        }
        enemy_projectiles::update_spore_stem(game);
    }
}
pub(crate) fn update(game: &mut Game) {
    let k = word!(game.cur_enemy_index) as i32;
    match u32::from(slot!(game, Spore[k].var_A)) | 0xa50000 {
        10873626 => {}
        10873627 => descend(game),
        10873682 => orbit(word!(game.cur_enemy_index), game),
        10873755 => begin_return(game),
        10873838 => finish_return(game),
        _ => {
            crate::game::animated_tiles::native::unreachable(game);
        }
    }
}
pub(crate) fn after_damage(game: &mut Game) {
    let k = word!(game.cur_enemy_index) as i32;
    if slot!(game, Spore[k].base.health) != 0 {
        return;
    }
    // The original reselects after its health query, before clearing actor timers.
    let k = word!(game.cur_enemy_index) as i32;
    slot!(game, Spore[0].var_0E = 0);
    slot!(game, Spore[k].base; invincibility_timer = 0, flash_timer = 0, ai_handler_bits = 0);
    slot!(game, Spore[k].base.properties, |old| old | 0x400);
    word_transfers!(game, descending, 14; native(0x19b1) <- value(0));
    slot!(game, Spore[0].base; current_instruction = 0xe77d, instruction_timer = 1);
    let area = word!(game.area_index);
    let bits = flags::boss_word_at(area, BossWordOrder::Little, game);
    flags::set_boss_word_le(area, bits | 2, game);
    word!(game.scrolling_finished_hook = 0);
    creation::spawn(
        &mut Runtime::new(game),
        SpawnSource::Hardcoded {
            x: 7,
            y: 30,
            header: 0xb78f,
        },
        &mut Contact::Absent,
    );
}
pub(crate) fn touch(game: &mut Game) {
    combat::contact_reaction(&mut Runtime::new(game), api::DeathHandling::LeavePending);
    after_damage(game);
}
pub(crate) fn shot(game: &mut Game) {
    let _projectile = word!(game.collision_detection_index).wrapping_mul(2);
    let kind = word!(game.projectile_type[word!(game.collision_detection_index)]);
    if kind & 0x700 == 0 && kind & 0x10 == 0 {
        return;
    }
    combat::shot_reaction(&mut Runtime::new(game), api::ShotHandling::Flash);
    let k = word!(game.cur_enemy_index) as i32;
    if slot!(game, Spore[k].base.flash_timer) != 0 {
        slot!(game, Spore[k].var_A = 0xeb52);
        if slot!(game, Spore[k].base.health).wrapping_sub(400) & 0x8000 != 0 {
            let speed = if slot!(game, Spore[0].var_0C) & 0x8000 != 0 {
                0xfffe
            } else {
                2
            };
            slot!(game, Spore[0].var_0C = speed);
        }
        if slot!(game, Spore[0].var_2F) == 0 {
            slot!(game, Spore[0].var_0C, |old| 0u16.wrapping_sub(old));
            slot!(game, Spore[0].var_2F = 1);
            slot!(game, Spore[k].base; current_instruction = 0xe729, instruction_timer = 1);
            let health = slot!(game, Spore[k].base.health);
            let mut cursor = 96;
            for threshold in [70, 410, 770] {
                if health.wrapping_sub(threshold) & 0x8000 != 0 {
                    break;
                }
                cursor -= 32;
            }
            if health != slot!(game, Spore[0].var_40) {
                slot!(game, Spore[0].var_40 = health);
                palette(cursor, game);
            }
        }
    }
    after_damage(game);
}
pub(crate) fn orbit(k: u16, game: &mut Game) {
    enemy_projectiles::update_spore_stem(game);
    let x = slot!(game, Spore[k as i32].var_C).wrapping_add(actor_cosine(
        slot!(game, Spore[0].var_0A),
        slot!(game, Spore[0].var_0B),
        game,
    ));
    slot!(game, Spore[k as i32].base.x_pos = x);
    let y = slot!(game, Spore[k as i32].var_D).wrapping_add(actor_sine(
        (2 * (i32::from(slot!(game, Spore[0].var_0A)) - 64)) as u16,
        slot!(game, Spore[0].var_0B).wrapping_sub(16),
        game,
    ));
    slot!(game, Spore[k as i32].base.y_pos = y);
    slot!(
        game,
        Spore[0].var_0A = u16::from(slot!(byte game, Spore[0].var_0C[0]).wrapping_add(slot!(
            game,
            Spore[0].var_0A
        )
            as u8))
    );
}
pub(crate) fn begin_return(game: &mut Game) {
    let dx = 128u16.wrapping_sub(slot!(game, Spore[0].base.x_pos));
    let dy = 624u16.wrapping_sub(slot!(game, Spore[0].base.y_pos));
    slot!(
        game,
        Spore[0].var_43 = u16::from(64u8.wrapping_sub(api::angle_from_delta(dx, dy) as u8))
    );
    let k = word!(game.cur_enemy_index) as i32;
    let vector = actor_direction_vector(u16::from(slot!(byte game, Spore[0].var_43[0])), 1, game);
    slot!(game, Spore[k]; var_28 = (vector.x >> 16) as u16, var_29 = vector.x as u16, var_2A = (vector.y >> 16) as u16, var_2B = vector.y as u16);
}
pub(crate) fn finish_return(game: &mut Game) {
    let k = word!(game.cur_enemy_index) as i32;
    let phase = u16::from(slot!(byte game, Spore[0].var_43[0]));
    let vector = Vector32 {
        x: words!((slot!(game, Spore[k].var_28), slot!(game, Spore[k].var_29))),
        y: words!((slot!(game, Spore[k].var_2A), slot!(game, Spore[k].var_2B))),
    };
    Runtime::new(game).session_mut().compatibility_enemy_scratch = phase;
    api::apply_vector(&mut Runtime::new(game), phase, vector);
    if abs16(slot!(game, Spore[0].base.x_pos).wrapping_sub(128)).wrapping_sub(8) & 0x8000 != 0
        && abs16(slot!(game, Spore[0].base.y_pos).wrapping_sub(624)).wrapping_sub(8) & 0x8000 != 0
    {
        slot!(game, Spore[0].var_A = 0xeb1a);
    }
    enemy_projectiles::update_spore_stem(game);
    bubbles(game);
}
pub(crate) fn bubbles(game: &mut Game) {
    if word!(game.nmi_frame_counter_word) & 0xf == 0 {
        game.next_random();
        let x = (word!(game.random_number) & 0x3f) + 96;
        let y = ((word!(game.random_number) >> 8) & 0xf) + 480;
        create_sprite(x, y, 21, 0, game);
    }
}
pub(crate) fn palette(mut cursor: u16, game: &mut Game) {
    for index in 144..160 {
        word!(
            game.palette_buffer[index] =
                sm_game::gameplay_tables::spore_spawn_colour(game, (cursor >> 1) as isize)
        );
        cursor = cursor.wrapping_add(2);
    }
}
pub(crate) fn fade(target: bool, script: EnemyScript, game: &mut Game) -> EnemyScript {
    let cursor = script.word(0) >> 1;
    let fields = [word_arrays::palette_buffer, word_arrays::target_palettes];
    let field = fields[usize::from(target)];
    word_transfers!(game, ascending, 16;
        little(field + 144 * 2) <- rom(0xa5e3f9, cursor),
    );
    word_transfers!(game, ascending, 16;
        little(field + 64 * 2) <- rom(0xa5e4f9, cursor),
    );
    word_transfers!(game, ascending, 16;
        little(field + 112 * 2) <- rom(0xa5e5d9, cursor),
    );
    script.offset(1)
}
pub(crate) fn emit_spore(script: EnemyScript, game: &mut Game) -> EnemyScript {
    game.next_random();
    let dx = (word!(game.random_number) & 0x7f).wrapping_sub(64);
    let x = slot!(game, Spore[0].base.x_pos).wrapping_add(dx);
    let dy = ((word!(game.random_number) >> 8) & 0x7f).wrapping_sub(64);
    let y = slot!(game, Spore[0].base.y_pos).wrapping_add(dy);
    game.eproj_spawn_pt = Point16U { x, y };
    SpawnEprojWithRoomGfx(0xe509, 0x15, game);
    QueueSfx2_Max6(0x29, game);
    script
}
pub(crate) fn emit_hurt(script: EnemyScript, game: &mut Game) -> EnemyScript {
    game.next_random();
    let x = slot!(game, Spore[0].base.x_pos)
        .wrapping_add(word!(game.random_number) & 0x7f)
        .wrapping_sub(64);
    let y = slot!(game, Spore[0].base.y_pos)
        .wrapping_add((word!(game.random_number) >> 8) & 0x3f)
        .wrapping_sub(32);
    create_sprite(x, y, 3, 0, game);
    QueueSfx2_Max6(0x25, game);
    script
}

// The named bank $A5 instructions are the ones `sm_a5` dispatches on.
const _: () = {
    use sm_game::actors::spore_spawn::instruction as i;
    assert!(i::INCREASE_MAX_X_RADIUS == 10872671 && i::CLEAR_DAMAGED_FLAG == 10872689);
    assert!(i::SET_MAX_X_RADIUS_AND_ANGLE_DELTA == 10872877 && i::HARDEN == 10872956);
    assert!(i::SET_FUNCTION == 10873018);
};
