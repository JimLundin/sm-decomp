//! Kraid's lint, the belly spikes Kraid fires at Samus: its collision with
//! Samus against the lint hitbox (bank $A7:B96A).
#![forbid(unsafe_code)]
use crate::actors::layouts::KraidLayout as Kraid;
use crate::cartridge::native_record;
use crate::constants::enemy_properties::K_ENEMY_PROPS_TANGIBLE;
use crate::ram::RawRam as _;
use crate::ram::{slot, word};

/// What Kraid's lint needs beyond the common actor host.
pub trait Host: crate::actors::host::ActorHost {
    /// The shared enemy contact reaction, finalizing a death it causes.
    fn contact_reaction_finalizing_death(&mut self);
}

const BANK: u8 = 0xa7;
/// The entry of "Hitbox - Kraid lint" ($A7:92B5) after its count word: left,
/// top, right and bottom words, each read where the original read it.
const HITBOX: u16 = 0x92b7;
const LEFT: usize = 0;
const TOP: usize = 2;
const BOTTOM: usize = 6;
/// The box is narrowed by this many pixels at the left, top and bottom.
const INSET: i32 = 2;
/// Samus is pushed out at most this far.
const MAX_PUSH: u16 = 16;
const SIGN: i32 = 0x8000;

fn hitbox(field: usize, game: &mut (impl Host + ?Sized)) -> i32 {
    i32::from(native_record(BANK, HITBOX, game).word(field))
}

/// Push Samus out of the lint's box, clamp her extra displacement and make the
/// lint tangible; sign tests are on the low 16 bits as in the original.
/// Kraid lint / Samus collision handling ($A7:B96A). Was `Kraid_EnemyTouch_Lint`.
pub fn lint_touch_samus(k: u16, game: &mut (impl Host + ?Sized)) {
    let e = i32::from(k);
    if slot!(game, Kraid[e].base.properties) as i32 & K_ENEMY_PROPS_TANGIBLE as i32 != 0
        || game.invincibility_timer() != 0
    {
        return;
    }
    let (samus_x, x_radius) = (
        i32::from(game.player_x()),
        i32::from(word!(game.samus_x_radius)),
    );
    let left =
        (hitbox(LEFT, game) + i32::from(slot!(game, Kraid[e].base.x_pos)) - INSET) as u16 as i32;
    if (x_radius + samus_x - left) & SIGN != 0 || (samus_x - x_radius - left) & SIGN == 0 {
        return;
    }
    let top =
        (hitbox(TOP, game) + i32::from(slot!(game, Kraid[e].base.y_pos)) + INSET) as u16 as i32;
    let (samus_y, y_radius) = (
        i32::from(game.player_y()),
        i32::from(word!(game.samus_y_radius)),
    );
    if (y_radius + samus_y - top) & SIGN != 0 {
        return;
    }
    let bottom =
        (hitbox(BOTTOM, game) + i32::from(slot!(game, Kraid[e].base.y_pos)) - INSET) as u16 as i32;
    if (samus_y - y_radius - bottom) & SIGN == 0 {
        return;
    }
    let mut push = (i32::from(word!(game.extra_samus_x_displacement))
        + !(x_radius + i32::from(MAX_PUSH))) as u16;
    if (push as i32 - i32::from(MAX_PUSH)) & SIGN == 0 {
        push = MAX_PUSH;
    }
    word!(game.extra_samus_x_displacement = push);
    game.contact_reaction_finalizing_death();
    slot!(
        game,
        Kraid[e].base.properties =
            slot!(game, Kraid[e].base.properties) | K_ENEMY_PROPS_TANGIBLE as u16
    );
}
