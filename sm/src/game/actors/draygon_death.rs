//! Draygon damage, death approach, ring, sinking and rewards.
//! Boss words remain native compatibility state; player and sprites use their owners.
#![forbid(unsafe_code)]
use crate::game::actors::{api, combat, draygon_flight::facing_animation};
use crate::game::enemy_layouts::EnemyLayout104 as Draygon;
use crate::game::gameplay_flag_runtime;
use crate::game::music::QueueMusic_Delayed8;
use crate::game::player_movement::forced;
use crate::game::session_runtime::{Runtime, actor_direction_vector};
use crate::game::sprite_effect_runtime as sprites;
use crate::game_state::Game;
use crate::ram_map::{slot, word};
use crate::rtl::abs16;
use sm_game::gameplay_flag_host::BossWordOrder;
use {crate::game::enemy_projectiles::scatter_drop, sm_game::enemy_projectiles::drops::DropSource};

pub(crate) fn damage_palette(native: bool, game: &mut Game) {
    let mut cursor = slot!(game, Draygon[0].var_0E).wrapping_mul(4);
    for index in 89..93 {
        let value = crate::rtl::rom_table_word!(game, 0xa596af, (cursor >> 1) as isize);
        if native {
            let bytes = value.to_ne_bytes();
            let at = 0xc000 + index * 2;
            game.ram().write_byte(at, bytes[0]);
            game.ram().write_byte(at + 1, bytes[1]);
        } else {
            word!(game.palette_buffer[index] = value);
        }
        cursor = cursor.wrapping_add(2);
    }
}
fn copy_hurt_palette(default: u16, target: usize, game: &mut Game) {
    let k = word!(game.cur_enemy_index) as i32;
    let palette = if slot!(game, Draygon[k].base.flash_timer) & 2 != 0 {
        0xa297
    } else {
        default
    };
    let address = 0xa50000 | u32::from(palette);
    if address & 0x8000 == 0 {
        crate::game::session_runtime::native_rom_diagnostic(address, game);
    }
    let image = game.capture_native_rom();
    let source = crate::rtl::lorom_offset(address) as usize;
    for offset in 0..32 {
        let byte = image.read(source + offset);
        game.ram().write_byte(target + offset, byte);
    }
}
pub(crate) fn hurt(game: &mut Game) {
    copy_hurt_palette(0xa277, 0xc0a0, game);
    let k = word!(game.cur_enemy_index) as i32;
    if slot!(game, Draygon[k].base.flash_timer) & 2 == 0 {
        damage_palette(false, game);
    }
    copy_hurt_palette(0xa1f7, 0xc1e0, game);
    if word!(game.samus_grapple_flags) & 1 != 0
        && slot!(game, Draygon[0].base.frame_counter) & 7 == 0
    {
        let health = (slot!(game, Draygon[0].base.health) as i16)
            .wrapping_sub(256)
            .max(0);
        slot!(game, Draygon[0].base.health = health as u16);
        after_damage(game);
    }
}
fn update_damage_palette(game: &mut Game) {
    let mut cursor: i32 = 0;
    while (slot!(game, Draygon[0].base.health) as i16).wrapping_sub(crate::rtl::rom_table_word!(
        game,
        0xa596ef,
        (cursor >> 1) as isize
    ) as i16)
        < 0
    {
        cursor += 2;
    }
    if cursor != i32::from(slot!(game, Draygon[0].var_0E)) {
        slot!(game, Draygon[0].var_0E = cursor as u16);
        damage_palette(true, game);
    }
}
pub(crate) fn shot(game: &mut Game) {
    if slot!(game, Draygon[0].var_0F).wrapping_sub(152) & 0x8000 != 0 {
        slot!(game, Draygon[0].var_0F, |old| old.wrapping_add(8));
    }
    combat::shot_reaction(&mut Runtime::new(game), api::ShotHandling::ReactionOnly);
    after_damage(game);
}
pub(crate) fn touch(game: &mut Game) {
    combat::contact_reaction(&mut Runtime::new(game), api::DeathHandling::LeavePending);
    after_damage(game);
}
pub(crate) fn power_bomb(game: &mut Game) {
    combat::power_bomb_reaction(&mut Runtime::new(game), api::DeathHandling::LeavePending);
    after_damage(game);
}
pub(crate) fn after_damage(game: &mut Game) {
    let k = word!(game.cur_enemy_index) as i32;
    if slot!(game, Draygon[k].base.health) != 0 {
        update_damage_palette(game);
        return;
    }
    word!(game.grapple_beam_function = 0xc8c5);
    facing_animation(0, 0, [0x9867, 0x9c5a], game);
    facing_animation(0, 0x40, [0x997a, 0x9d1c], game);
    slot!(game, Draygon[0x40].var_A = 0x804b);
    slot!(game, Draygon[0].var_A = 0x9185);
    forced::release_from_draygon(&mut Runtime::new(game));
    word!(game.samus_grapple_flags = 0);
    let dx = 256u16.wrapping_sub(slot!(game, Draygon[0].base.x_pos));
    let dy = 480u16.wrapping_sub(slot!(game, Draygon[0].base.y_pos));
    slot!(
        game,
        Draygon[0].var_43 = u16::from(64u8.wrapping_sub(api::angle_from_delta(dx, dy) as u8))
    );
    let vector = actor_direction_vector(u16::from(slot!(byte game, Draygon[0].var_43[0])), 1, game);
    slot!(game, Draygon[0]; var_28 = (vector.x >> 16) as u16, var_29 = vector.x as u16, var_2A = (vector.y >> 16) as u16, var_2B = vector.y as u16);
}
pub(crate) fn approach(game: &mut Game) {
    let k = word!(game.cur_enemy_index) as i32;
    crate::game::actors::draygon_flight::shed_particle(k as u16, 0xf, game);
    let dx = (64 - (i32::from(slot!(game, Draygon[0].base.x_pos)) >> 2)) as u16;
    let dy = (120 - (i32::from(slot!(game, Draygon[0].base.y_pos)) >> 2)) as u16;
    slot!(
        game,
        Draygon[0].var_43 = u16::from(64u8.wrapping_sub(api::angle_from_delta(dx, dy) as u8))
    );
    let vector = actor_direction_vector(u16::from(slot!(byte game, Draygon[0].var_43[0])), 1, game);
    Runtime::new(game).session_mut().compatibility_enemy_scratch =
        u16::from(slot!(byte game, Draygon[0].var_43[0]));
    let phase = u16::from(slot!(byte game, Draygon[0].var_43[0]));
    api::apply_vector(&mut Runtime::new(game), phase, vector);
    if abs16(slot!(game, Draygon[0].base.x_pos).wrapping_sub(256)).wrapping_sub(4) & 0x8000 == 0 {
        return;
    }
    if abs16(slot!(game, Draygon[0].base.y_pos).wrapping_sub(480)).wrapping_sub(4) & 0x8000 == 0 {
        return;
    }
    sprites::draygon_spawn_ring(game);
    slot!(game, Draygon[0].var_A = 0x9294);
    QueueMusic_Delayed8(3, game);
    slot!(game, Draygon[0].var_B = 416);
    slot!(game, Draygon[0].base; current_instruction = 0x97b9, instruction_timer = 1);
    let properties = slot!(game, Draygon[0].base.properties) | 0x200;
    slot!(game, Draygon[0x80].base.properties = properties);
    slot!(game, Draygon[0xc0].base.properties = properties);
    facing_animation(0, 0x40, [0x999c, 0x9d3e], game);
    slot!(game, Draygon[0x40].var_A = 0x804b);
}
pub(crate) fn ring(game: &mut Game) {
    sprites::draygon_death_bubble(game);
    slot!(game, Draygon[0].var_B, |old| old.wrapping_sub(1));
    if slot!(game, Draygon[0].var_B) != 0 {
        sprites::draygon_motion(0, game);
    } else {
        slot!(game, Draygon[0].var_A = 0x92ab);
    }
}
pub(crate) fn sink(game: &mut Game) {
    sprites::draygon_death_bubble(game);
    sprites::draygon_motion(0, game);
    slot!(game, Draygon[0].base.y_pos, |old| old.wrapping_add(1));
    if slot!(game, Draygon[0].base.y_pos).wrapping_sub(576) & 0x8000 == 0 {
        slot!(game, Draygon[0].base; current_instruction = 0x98ed, instruction_timer = 1);
        let properties = slot!(game, Draygon[0].base.properties) | 0x200;
        slot!(game, Draygon[0].base.properties = properties);
        slot!(game, Draygon[0x40].base.properties = properties);
        let drop_index = word!(game.area_index);
        let area = word!(game.area_index);
        let bits = gameplay_flag_runtime::boss_word_at(area, BossWordOrder::Little, game);
        gameplay_flag_runtime::set_boss_word_le(area, bits | 1, game);
        scatter_drop(DropSource::Draygon, drop_index, game);
        sprites::retire_all(game);
    }
}
