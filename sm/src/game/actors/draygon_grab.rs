//! Draygon acquisition, carrying, tail attacks and release share the player owner.
//! Boss-specific words retain their native compatibility layout; no player copy exists.
#![forbid(unsafe_code)]
use super::{
    api, combat,
    draygon_flight::{facing_animation, turret},
};
use crate::game::enemy_layouts::EnemyLayout104 as Draygon;
use crate::game::player_movement::forced;
use crate::game::session_runtime::{
    Runtime, actor_cosine, actor_direction_vector, actor_sine, correct_previous_position,
    damage_player, player_command,
};
use crate::game::sprite_effect_runtime::create_sprite;
use crate::game_state::Game;
use crate::ram_map::{slot, word};
use crate::rtl::{abs16, words};
use crate::types::EnemyDef;
use core::mem::offset_of;
use sm_game::enemy_script::EnemyScript;

fn pursue(dx: u16, dy: u16, game: &mut Game) {
    let phase = 64u8.wrapping_sub(api::angle_from_delta(dx, dy) as u8) as u16;
    Runtime::new(game).session_mut().compatibility_enemy_scratch = phase;
    let vector = actor_direction_vector(phase, 2, game);
    api::apply_vector(&mut Runtime::new(game), phase, vector);
}
fn electric_flash(game: &mut Game) -> bool {
    if word!(game.samus_grapple_flags) & 1 == 0 {
        return false;
    }
    let definition = slot!(game, Draygon[0].base.enemy_ptr);
    let duration = Runtime::new(game)
        .actor_legacy_record(0xa0, definition)
        .byte(offset_of!(EnemyDef, hurt_ai_time));
    slot!(
        game,
        Draygon[0].base.flash_timer = u16::from(duration).wrapping_add(8)
    );
    slot!(game, Draygon[0].base.ai_handler_bits, |old| old | 2);
    true
}
pub(crate) fn acquire(game: &mut Game) {
    turret(game);
    if word!(game.samus_x_speed_divisor) == 0 {
        slot!(game, Draygon[0].var_A = 0x9154);
        return;
    }
    slot!(game, Draygon[0].base.properties, |old| old | 0x400);
    let dx = if slot!(game, Draygon[0].var_20) == 0 {
        0u16.wrapping_sub(8)
    } else {
        8
    };
    let distance = abs16(
        slot!(game, Draygon[0].base.x_pos)
            .wrapping_add(dx)
            .wrapping_sub(word!(game.samus_x_pos)),
    );
    if distance.wrapping_sub(8) & 0x8000 != 0
        && abs16(slot!(game, Draygon[0].base.y_pos).wrapping_sub(word!(game.samus_y_pos)))
            .wrapping_sub(8)
            & 0x8000
            != 0
    {
        facing_animation(0, 0xc0, [0x9845, 0x9c38], game);
        if player_command(0xd, game) != 0 {
            slot!(game, Draygon[0].var_A = 0x8f10);
        } else {
            let direction = u16::from(slot!(game, Draygon[0].var_20) != 0);
            forced::grab_by_draygon(direction, &mut Runtime::new(game));
            slot!(game, Draygon[0]; var_06 = 256, var_07 = 384, var_05 = 0, var_09 = 0, var_08 = 192, var_0B = 2048, var_A = 0x8f1e);
        }
    } else {
        let dx = word!(game.samus_x_pos).wrapping_sub(slot!(game, Draygon[0].base.x_pos));
        let dy = word!(game.samus_y_pos).wrapping_sub(slot!(game, Draygon[0].base.y_pos));
        pursue(dx, dy, game);
    }
}
pub(crate) fn cancel_grapple(game: &mut Game) {
    word!(game.grapple_beam_function = 0xc8c5);
    slot!(game, Draygon[0].var_A = 0x9128);
}
pub(crate) fn center(k: u16, game: &mut Game) {
    if electric_flash(game) {
        return;
    }
    let dx = abs16(slot!(game, Draygon[0].base.x_pos).wrapping_sub(256));
    if dx.wrapping_sub(2) & 0x8000 != 0
        && abs16(slot!(game, Draygon[0].base.y_pos).wrapping_sub(384)).wrapping_sub(2) & 0x8000 != 0
    {
        slot!(game, Draygon[0].var_A = 0x8fd6);
        facing_animation(0, k as i32, [0x9922, 0x9cb4], game);
        slot!(game, Draygon[0].base.properties, |old| old | 0x400);
    } else {
        let dx = 256u16.wrapping_sub(slot!(game, Draygon[0].base.x_pos));
        let dy = 384u16.wrapping_sub(slot!(game, Draygon[0].base.y_pos));
        pursue(dx, dy, game);
        attach(k, game);
    }
}
pub(crate) fn orbit(k: u16, game: &mut Game) {
    if electric_flash(game) {
        return;
    }
    if word!(game.random_number) as u8 == 0 {
        slot!(game, Draygon[0]; var_0C = 64, var_A = 0x90d4);
        return;
    }
    slot!(
        game,
        Draygon[0].base.x_pos = slot!(game, Draygon[0].var_06).wrapping_add(actor_cosine(
            slot!(game, Draygon[0].var_08),
            slot!(game, Draygon[0].var_05),
            game
        ))
    );
    slot!(
        game,
        Draygon[0].base.y_pos = slot!(game, Draygon[0].var_07).wrapping_add(actor_sine(
            slot!(game, Draygon[0].var_08),
            slot!(game, Draygon[0].var_05) >> 2,
            game
        ))
    );
    super::draygon_flight::shed_particle(k, 7, game);
    slot!(game, Draygon[0](var_05, var_09) += 0x2000);
    let mut finished = slot!(game, Draygon[0].var_05).wrapping_sub(160) & 0x8000 == 0;
    if !finished {
        let counter = slot!(game, Draygon[0].var_0B).wrapping_sub(1);
        slot!(game, Draygon[0].var_0B = counter);
        let phase = slot!(game, Draygon[0].var_08).to_ne_bytes()[0];
        slot!(
            game,
            Draygon[0].var_08 = u16::from(phase.wrapping_add(counter.to_ne_bytes()[1]))
        );
        let fraction = slot!(game, Draygon[0].var_0A);
        slot!(game, Draygon[0].var_0A = fraction.wrapping_sub(0x4000));
        let y =
            (words!((slot!(game, Draygon[0].var_07), fraction)).wrapping_sub(0x4000) >> 16) as u16;
        slot!(game, Draygon[0].var_07 = y);
        finished = y.wrapping_sub(64) & 0x8000 != 0;
    }
    if finished {
        slot!(game, Draygon[0].var_A = 0x9105);
    } else {
        attach(k, game);
    }
}
pub(crate) fn pause(k: u16, game: &mut Game) {
    attach(k, game);
    let remaining = slot!(game, Draygon[0].var_0C).wrapping_sub(1);
    slot!(game, Draygon[0].var_0C = remaining);
    if remaining == 0 {
        slot!(game, Draygon[0].var_A = 0x8fd6);
    } else if remaining == 63 {
        facing_animation(0, 0x80, [0x9ae8, 0x9ea1], game);
    }
}
pub(crate) fn start_release(k: u16, game: &mut Game) {
    attach(k, game);
    facing_animation(k as i32, 0x80, [0x9a68, 0x9e21], game);
    slot!(game, Draygon[0].var_A = 0x9124);
}
pub(crate) fn attach(k: u16, game: &mut Game) {
    let dx = if slot!(game, Draygon[k as i32].var_20) == 0 {
        0u16.wrapping_sub(8)
    } else {
        8
    };
    word!(game.samus_x_pos = slot!(game, Draygon[0].base.x_pos).wrapping_add(dx));
    word!(game.samus_y_pos = slot!(game, Draygon[0].base.y_pos).wrapping_add(40));
    correct_previous_position(game);
    if word!(game.samus_grapple_flags) & 2 != 0 {
        word!(game.samus_grapple_flags = 0);
        slot!(game, Draygon[0].var_A = 0x9154);
    }
}
pub(crate) fn release(game: &mut Game) {
    forced::release_from_draygon(&mut Runtime::new(game));
    word!(game.samus_grapple_flags = 0);
    slot!(game, Draygon[0].var_A = 0x9154);
    slot!(game, Draygon[0].base.properties, |old| old & !0x400);
    facing_animation(0, 0x80, [0x9b5a, 0x9f15], game);
}
pub(crate) fn depart(game: &mut Game) {
    turret(game);
    let y = (slot!(game, Draygon[0].base.y_pos) as i16).wrapping_sub(4);
    slot!(game, Draygon[0].base.y_pos = y as u16);
    if y < 0 {
        slot!(game, Draygon[0].base.properties, |old| old & !0x400);
        slot!(game, Draygon[0]; var_A = 0x87f4, var_B = 0);
        slot!(game, Draygon[0].base; x_pos = slot!(game, Draygon[0].var_00), y_pos = slot!(game, Draygon[0].var_01));
    }
}

pub(crate) fn slam(script: EnemyScript, game: &mut Game) -> EnemyScript {
    slot!(game, Draygon[0].var_0F = 24);
    let definition = slot!(game, Draygon[0].base.enemy_ptr);
    let amount = Runtime::new(game)
        .legacy_native_record(0xa0, definition)
        .word(6);
    let amount = combat::suit_damage(&mut Runtime::new(game), amount);
    damage_player(amount, game);
    word! {game; earthquake_timer = 32; earthquake_type = 7; }
    create_sprite(
        word!(game.samus_x_pos),
        word!(game.samus_y_pos).wrapping_add(16),
        21,
        0,
        game,
    );
    script
}
