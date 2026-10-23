//! Ordinary, power and spread bomb production with live initialization boundaries.
#![forbid(unsafe_code)]
use super::host::{Host, PaletteEffect};
use crate::game::{
    projectile_initialization::Initialization, projectile_weapon_definitions as definitions,
};
fn cancel_charge(h: &mut impl Host) {
    if h.session().controls.charge != 0 {
        h.sound(1, 9, 2);
        h.session_mut().controls.charge = 0;
        super::weapons::clear_animation(h);
        h.palette(PaletteEffect::Suit);
    }
}
fn reserve(h: &mut impl Host) -> bool {
    if h.inputs().shoot_button & h.inputs().pressed == 0
        || h.session().bomb_count != 0
            && (h.session().bomb_count.wrapping_sub(5) & 0x8000 == 0
                || h.session().controls.cooldown as u8 != 0)
    {
        cancel_charge(h);
        return false;
    }
    h.session_mut().controls.cooldown = h.session().controls.cooldown.wrapping_add(1);
    h.session_mut().bomb_count = h.session().bomb_count.wrapping_add(1);
    true
}
fn prepare(h: &mut impl Host) -> bool {
    if h.inputs().equipped_items & 0x1000 != 0 {
        if h.session().controls.charge.wrapping_sub(60) & 0x8000 != 0 || h.session().bomb_count != 0
        {
            return reserve(h);
        }
        if h.inputs().held & 0x400 != 0 && h.session().controls.spread_charge & 0xc0 != 0xc0 {
            h.session_mut().controls.spread_charge =
                h.session().controls.spread_charge.wrapping_add(1);
        } else {
            spread(h);
            h.palette(PaletteEffect::Suit);
            h.sound(1, 9, 2);
        }
    }
    false
}
pub(super) fn fire(h: &mut impl Host) {
    if h.inputs().shoot_button & h.inputs().held == 0 {
        return cancel_charge(h);
    }
    let power = h.session().controls.selected_weapon == 3;
    if power && h.session().power_bomb.flag & 0x8000 != 0 {
        return;
    }
    if !(if power { reserve(h) } else { prepare(h) }) {
        return;
    }
    if power {
        if h.session().player.power_bombs == 0 {
            return;
        }
        h.session_mut().player.power_bombs = h.session().player.power_bombs.wrapping_sub(1);
        if h.session().player.power_bombs & 0x8000 != 0 {
            return;
        }
        h.session_mut().power_bomb.flag = 0xffff;
    }
    let raw = (10..20)
        .step_by(2)
        .find(|&raw| shot!(h, raw, kind) == 0)
        .unwrap_or(18);
    let key = if power {
        let hud = h.session().controls.selected_weapon.swap_bytes();
        let kind = hud | shot!(h, raw, kind);
        shot!(h, raw, kind = kind);
        kind.to_ne_bytes()[1] & 15
    } else {
        shot!(h, raw, kind = 0x500);
        5
    };
    shot!(h, raw, direction = 0);
    shot!(h, raw, x = h.session().player.x);
    shot!(h, raw, y = h.session().player.y);
    shot!(h, raw, variable = 60);
    super::animation::initialize(h, raw, Initialization::Bomb);
    shot!(h, raw, motion = if power { 0xb0ae } else { 0xb099 });
    h.session_mut().controls.cooldown = u16::from(definitions::COOLDOWNS[usize::from(key)]);
    if power {
        if h.session().controls.automatic_weapon != 0 {
            h.session_mut().controls.selected_weapon = 0;
            h.session_mut().controls.automatic_weapon = 0;
        } else if h.session().controls.selected_weapon == 3 && h.session().player.power_bombs == 0 {
            h.session_mut().controls.selected_weapon = 0;
        }
    }
}
pub(super) fn spread(h: &mut impl Host) {
    if shot!(h, 10, motion) == 0xd8f7 {
        return;
    }
    for (pattern, raw) in (10..20).step_by(2).enumerate() {
        shot!(h, raw, kind = 0x8500);
        shot!(h, raw, direction = 0);
        shot!(h, raw, motion = 0xd8f7);
        super::animation::initialize(h, raw, Initialization::Bomb);
        shot!(h, raw, x = h.session().player.x);
        shot!(h, raw, x_fraction = 0);
        shot!(h, raw, y = h.session().player.y);
        shot!(h, raw, y_fraction = 0);
        shot!(h, raw, x_speed = [0x8100, 0x8080, 0, 0x80, 0x100][pattern]);
        shot!(h, raw, trail_time = super::bombs::SPREAD_FRACTIONS[pattern]);
        let speed = 0u16.wrapping_sub(
            [0, 1, 2, 1, 0][pattern] + ((h.session().controls.spread_charge >> 6) & 3),
        );
        shot!(h, raw, y_speed = speed);
        shot!(h, raw, auxiliary = speed);
        shot!(h, raw, variable = [120, 110, 100, 110, 120][pattern]);
    }
    h.session_mut().bomb_count = 5;
    // Live byte at C23 after every initializer: high byte of slot five's kind.
    let key = shot!(h, 10, kind).to_le_bytes()[1] & 15;
    h.session_mut().controls.cooldown = u16::from(definitions::COOLDOWNS[usize::from(key)]);
    h.session_mut().controls.spread_charge = 0;
    h.session_mut().controls.charge = 0;
}
