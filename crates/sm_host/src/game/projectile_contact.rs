//! Player/enemy projectile contact, with ordered damage and reaction effects.
#![deny(unsafe_op_in_unsafe_fn)]
use crate::game::actors;
use crate::game::session_runtime::Runtime;
use crate::projection::ProjectedBytes;
use crate::ram_map::checked_word as word;
use sm_game::contact_rules::axis;

fn mark(index: u16, runtime: &mut Runtime<'_>) {
    word!(runtime.PROJECTILE_DIR[index] = word!(runtime.PROJECTILE_DIR[index]) | 0x10);
}
/// Samus's own projectiles against her (the bomb jump), on the gameplay pass's lease.
pub(crate) fn player_projectiles_hit_samus(runtime: &mut Runtime<'_>) {
    word!(runtime.ENEMY_PROCESSING_STAGE = 10);
    if word!(runtime.FLAG_DISABLE_PROJECTILE_INTERACTION) != 0 {
        return;
    }
    let count = if word!(runtime.BOMB_COUNTER) != 0 {
        10
    } else {
        if word!(runtime.PROJECTILE_COUNTER) == 0 {
            return;
        }
        5
    };
    if word!(runtime.PROJECTILE_INVINCIBILITY_TIMER) != 0
        || word!(runtime.SAMUS_CONTACT_DAMAGE_INDEX) != 0
    {
        return;
    }
    for index in 0..count {
        word!(runtime.COLLISION_DETECTION_INDEX = index);
        let kind = word!(runtime.PROJECTILE_TYPE[index]);
        if word!(runtime.PROJECTILE_DAMAGE[index]) == 0
            || kind & 0x8000 != 0
            || kind & 0xf00 >= 0x700
            || word!(runtime.PROJECTILE_DIR[index]) & 0x10 != 0
        {
            continue;
        }
        if !axis(
            word!(runtime.PROJECTILE_X_POS[index]),
            word!(runtime.SAMUS_X_POS),
            word!(runtime.PROJECTILE_X_RADIUS[index]),
            word!(runtime.SAMUS_X_RADIUS),
        ) || !axis(
            word!(runtime.PROJECTILE_Y_POS[index]),
            word!(runtime.SAMUS_Y_POS),
            word!(runtime.PROJECTILE_Y_RADIUS[index]),
            word!(runtime.SAMUS_Y_RADIUS),
        ) {
            continue;
        }
        if kind & 0xff00 != 0x300 && kind & 0xff00 != 0x500 {
            mark(index, runtime);
            let amount = word!(runtime.PROJECTILE_DAMAGE[index]);
            let damage = actors::combat::suit_damage(runtime, amount);
            sm_game::samus_periodic_damage::contact(runtime, damage);
            word!(runtime.SAMUS_INVINCIBILITY_TIMER = 96);
            word!(runtime.SAMUS_KNOCKBACK_TIMER = 5);
            unsafe {
                crate::c_library::__assert_fail(
                    c"0".as_ptr(),
                    c"src/sm_a0.c".as_ptr(),
                    2048,
                    c"void SamusProjectileInteractionHandler(void)".as_ptr(),
                );
            }
        }
        if word!(runtime.PROJECTILE_VARIABLES[index]) == 8 {
            let x = word!(runtime.SAMUS_X_POS);
            let bomb_x = word!(runtime.PROJECTILE_X_POS[index]);
            word!(
                runtime.BOMB_JUMP_DIR = if x == bomb_x {
                    2
                } else if (x.wrapping_sub(bomb_x) as i16) < 0 {
                    1
                } else {
                    3
                }
            );
        }
    }
    // $A0:9884: the scan leaves the projectile index at the count it checked.
    word!(runtime.COLLISION_DETECTION_INDEX = count);
}
