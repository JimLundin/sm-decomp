//! Kraid's belly-lint contact with Samus against the hitbox at $A7:92B7.
#![forbid(unsafe_code)]
use crate::actors::layouts::KraidLayout as Kraid;
use crate::cartridge::native_record;
use crate::constants::enemy_properties::kEnemyProps_Tangible;
use crate::ram::RawRam as _;
use crate::ram::{slot, word};

/// What Kraid's lint needs beyond the common actor host.
pub trait Host: crate::actors::host::ActorHost {
    /// The shared enemy contact reaction, finalizing a death it causes.
    fn contact_reaction_finalizing_death(&mut self);
}

/// Left, top, right and bottom words; each is read where the original read it.
const HITBOX: u16 = 0x92b7;
const LEFT: usize = 0;
const TOP: usize = 2;
const BOTTOM: usize = 6;

fn hitbox(field: usize, game: &mut (impl Host + ?Sized)) -> i32 {
    i32::from(native_record(0xa7, HITBOX, game).word(field))
}

/// Push Samus out of the lint's box, clamp her extra displacement and make the
/// lint tangible; sign tests are on the low 16 bits as in the original.
/// Was Kraid_EnemyTouch_Lint.
pub fn lint_touch_samus(k: u16, game: &mut (impl Host + ?Sized)) {
    let e = i32::from(k);
    if slot!(game, Kraid[e].base.properties) as i32 & kEnemyProps_Tangible as i32 != 0
        || word!(game.samus_invincibility_timer) != 0
    {
        return;
    }
    let (samus_x, x_radius) = (
        i32::from(word!(game.samus_x_pos)),
        i32::from(word!(game.samus_x_radius)),
    );
    let left = (hitbox(LEFT, game) + i32::from(slot!(game, Kraid[e].base.x_pos)) - 2) as u16 as i32;
    if (x_radius + samus_x - left) & 0x8000 != 0 || (samus_x - x_radius - left) & 0x8000 == 0 {
        return;
    }
    let top = (hitbox(TOP, game) + i32::from(slot!(game, Kraid[e].base.y_pos)) + 2) as u16 as i32;
    let (samus_y, y_radius) = (
        i32::from(word!(game.samus_y_pos)),
        i32::from(word!(game.samus_y_radius)),
    );
    if (y_radius + samus_y - top) & 0x8000 != 0 {
        return;
    }
    let bottom =
        (hitbox(BOTTOM, game) + i32::from(slot!(game, Kraid[e].base.y_pos)) - 2) as u16 as i32;
    if (samus_y - y_radius - bottom) & 0x8000 == 0 {
        return;
    }
    let mut push = (i32::from(word!(game.extra_samus_x_displacement)) + !(x_radius + 16)) as u16;
    if (push as i32 - 16) & 0x8000 == 0 {
        push = 16;
    }
    word!(game.extra_samus_x_displacement = push);
    game.contact_reaction_finalizing_death();
    slot!(
        game,
        Kraid[e].base.properties =
            slot!(game, Kraid[e].base.properties) | kEnemyProps_Tangible as u16
    );
}
