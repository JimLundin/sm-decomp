//! Player/enemy projectile contact, with ordered damage and reaction effects.
#![deny(unsafe_op_in_unsafe_fn)]
use crate::game::actors;
use crate::game::session_runtime::Runtime;
use crate::game::session_runtime::damage_player;
use crate::game_state::Game;
use crate::ram_map::checked_word as word;
use sm_game::contact_rules::axis;

fn mark(index: u16, game: &mut Game) {
    word!(game.PROJECTILE_DIR[index] = word!(game.PROJECTILE_DIR[index]) | 0x10);
}
pub unsafe fn player_projectiles_hit_samus(game: &mut Game) {
    word!(game.ENEMY_PROCESSING_STAGE = 10);
    if word!(game.FLAG_DISABLE_PROJECTILE_INTERACTION) != 0 {
        return;
    }
    let count = if word!(game.BOMB_COUNTER) != 0 {
        10
    } else {
        if word!(game.PROJECTILE_COUNTER) == 0 {
            return;
        }
        5
    };
    if word!(game.PROJECTILE_INVINCIBILITY_TIMER) != 0
        || word!(game.SAMUS_CONTACT_DAMAGE_INDEX) != 0
    {
        return;
    }
    for index in 0..count {
        word!(game.COLLISION_DETECTION_INDEX = index);
        let kind = word!(game.PROJECTILE_TYPE[index]);
        if word!(game.PROJECTILE_DAMAGE[index]) == 0
            || kind & 0x8000 != 0
            || kind & 0xf00 >= 0x700
            || word!(game.PROJECTILE_DIR[index]) & 0x10 != 0
        {
            continue;
        }
        if !axis(
            word!(game.PROJECTILE_X_POS[index]),
            word!(game.SAMUS_X_POS),
            word!(game.PROJECTILE_X_RADIUS[index]),
            word!(game.SAMUS_X_RADIUS),
        ) || !axis(
            word!(game.PROJECTILE_Y_POS[index]),
            word!(game.SAMUS_Y_POS),
            word!(game.PROJECTILE_Y_RADIUS[index]),
            word!(game.SAMUS_Y_RADIUS),
        ) {
            continue;
        }
        if kind & 0xff00 != 0x300 && kind & 0xff00 != 0x500 {
            mark(index, game);
            let amount = word!(game.PROJECTILE_DAMAGE[index]);
            let damage = actors::combat::suit_damage(&mut Runtime::new(game), amount);
            damage_player(damage, game);
            word!(game.SAMUS_INVINCIBILITY_TIMER = 96);
            word!(game.SAMUS_KNOCKBACK_TIMER = 5);
            unsafe {
                crate::c_library::__assert_fail(
                    c"0".as_ptr(),
                    c"src/sm_a0.c".as_ptr(),
                    2048,
                    c"void SamusProjectileInteractionHandler(void)".as_ptr(),
                );
            }
        }
        if word!(game.PROJECTILE_VARIABLES[index]) == 8 {
            let x = word!(game.SAMUS_X_POS);
            let bomb_x = word!(game.PROJECTILE_X_POS[index]);
            word!(
                game.BOMB_JUMP_DIR = if x == bomb_x {
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
