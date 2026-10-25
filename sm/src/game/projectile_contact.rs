//! Player/enemy projectile contact, with ordered damage and reaction effects.
#![deny(unsafe_op_in_unsafe_fn)]
use crate::game::actors;
use crate::game::contact_rules::axis;
use crate::game::session_runtime::Runtime;
use crate::game::session_runtime::damage_player;
use crate::game_state::Game;
use crate::ram_map::{checked_word as word, word_arrays as arrays};

fn mark(index: u16, game: &mut Game) {
    word!(game.projectile_dir[index] = word!(game.projectile_dir[index]) | 0x10);
}
pub unsafe fn player_projectiles_hit_samus(game: &mut Game) {
    word!(game.enemy_processing_stage = 10);
    if word!(game.flag_disable_projectile_interaction) != 0 {
        return;
    }
    let count = if word!(game.bomb_counter) != 0 {
        10
    } else {
        if word!(game.projectile_counter) == 0 {
            return;
        }
        5
    };
    if word!(game.projectile_invincibility_timer) != 0
        || word!(game.samus_contact_damage_index) != 0
    {
        return;
    }
    for index in 0..count {
        word!(game.collision_detection_index = index);
        let kind = word!(game.projectile_type[index]);
        if word!(game.projectile_damage[index]) == 0
            || kind & 0x8000 != 0
            || kind & 0xf00 >= 0x700
            || word!(game.projectile_dir[index]) & 0x10 != 0
        {
            continue;
        }
        if !axis(
            word!(game.projectile_x_pos[index]),
            word!(game.samus_x_pos),
            word!(game.projectile_x_radius[index]),
            word!(game.samus_x_radius),
        ) || !axis(
            word!(game.projectile_y_pos[index]),
            word!(game.samus_y_pos),
            word!(game.projectile_y_radius[index]),
            word!(game.samus_y_radius),
        ) {
            continue;
        }
        if kind & 0xff00 != 0x300 && kind & 0xff00 != 0x500 {
            mark(index, game);
            let amount = word!(game.projectile_damage[index]);
            let damage = actors::combat::suit_damage(&mut Runtime::new(game), amount);
            damage_player(damage, game);
            word!(game.samus_invincibility_timer = 96);
            word!(game.samus_knockback_timer = 5);
            unsafe {
                crate::c_library::__assert_fail(
                    c"0".as_ptr(),
                    c"src/sm_a0.c".as_ptr(),
                    2048,
                    c"void SamusProjectileInteractionHandler(void)".as_ptr(),
                );
            }
            word!(
                game.knockback_x_dir = (word!(game.samus_x_pos).wrapping_sub(u16::from_ne_bytes([
                    game.g_ram[arrays::eproj_x_pos],
                    game.g_ram[arrays::eproj_x_pos + 1]
                ])) as i16
                    >= 0) as u16
            );
            return;
        }
        if word!(game.projectile_variables[index]) == 8 {
            let x = word!(game.samus_x_pos);
            let bomb_x = word!(game.projectile_x_pos[index]);
            word!(
                game.bomb_jump_dir = if x == bomb_x {
                    2
                } else if (x.wrapping_sub(bomb_x) as i16) < 0 {
                    1
                } else {
                    3
                }
            );
        }
    }
}
