//! Ordinary, power and spread bomb production with live initialization boundaries.
#![forbid(unsafe_code)]
use super::handlers;
use super::host::{Host, PaletteEffect};
use crate::shot;
use crate::{
    projectile_initialization::Initialization, projectile_weapon_definitions as definitions,
};
fn cancel_charge(h: &mut impl Host) {
    if h.session().projectiles.controls.charge != 0 {
        h.sound(1, 9, 2);
        h.session_mut().projectiles.controls.charge = 0;
        super::weapons::clear_animation(h);
        h.palette(PaletteEffect::Suit);
    }
}
fn reserve(h: &mut impl Host) -> bool {
    if h.inputs().shoot_button & h.inputs().pressed == 0
        || h.session().projectiles.bomb_count != 0
            && (h.session().projectiles.bomb_count.wrapping_sub(5) & 0x8000 == 0
                || h.session().projectiles.controls.cooldown as u8 != 0)
    {
        cancel_charge(h);
        return false;
    }
    h.session_mut().projectiles.controls.cooldown =
        h.session().projectiles.controls.cooldown.wrapping_add(1);
    h.session_mut().projectiles.bomb_count = h.session().projectiles.bomb_count.wrapping_add(1);
    true
}
fn prepare(h: &mut impl Host) -> bool {
    if h.session().inventory.equipment.active & 0x1000 != 0 {
        if h.session().projectiles.controls.charge.wrapping_sub(60) & 0x8000 != 0
            || h.session().projectiles.bomb_count != 0
        {
            return reserve(h);
        }
        if h.inputs().held & 0x400 != 0
            && h.session().projectiles.controls.spread_charge & 0xc0 != 0xc0
        {
            h.session_mut().projectiles.controls.spread_charge = h
                .session()
                .projectiles
                .controls
                .spread_charge
                .wrapping_add(1);
        } else {
            spread(h);
            h.palette(PaletteEffect::Suit);
            h.sound(1, 9, 2);
        }
    }
    false
}
pub fn fire(h: &mut impl Host) {
    if h.inputs().shoot_button & h.inputs().held == 0 {
        return cancel_charge(h);
    }
    let power = h.session().projectiles.controls.selected_weapon == 3;
    if power && h.session().projectiles.power_bomb.flag & 0x8000 != 0 {
        return;
    }
    if !(if power { reserve(h) } else { prepare(h) }) {
        return;
    }
    if power {
        if h.session().inventory.ammunition[2].current == 0 {
            return;
        }
        h.session_mut().inventory.ammunition[2].current =
            h.session().inventory.ammunition[2].current.wrapping_sub(1);
        if h.session().inventory.ammunition[2].current & 0x8000 != 0 {
            return;
        }
        h.session_mut().projectiles.power_bomb.flag = 0xffff;
    }
    let raw = (10..20)
        .step_by(2)
        .find(|&raw| shot!(h, raw, kind) == 0)
        .unwrap_or(18);
    let key = if power {
        let hud = h
            .session()
            .projectiles
            .controls
            .selected_weapon
            .swap_bytes();
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
    shot!(
        h,
        raw,
        motion = if power {
            handlers::POWER_BOMB
        } else {
            handlers::BOMB
        }
    );
    h.session_mut().projectiles.controls.cooldown =
        u16::from(definitions::COOLDOWNS[usize::from(key)]);
    if power {
        if h.session().projectiles.controls.automatic_weapon != 0 {
            h.session_mut().projectiles.controls.selected_weapon = 0;
            h.session_mut().projectiles.controls.automatic_weapon = 0;
        } else if h.session().projectiles.controls.selected_weapon == 3
            && h.session().inventory.ammunition[2].current == 0
        {
            h.session_mut().projectiles.controls.selected_weapon = 0;
        }
    }
}
pub fn spread(h: &mut impl Host) {
    if shot!(h, 10, motion) == handlers::SPREAD_BOMB {
        return;
    }
    for (pattern, raw) in (10..20).step_by(2).enumerate() {
        shot!(h, raw, kind = 0x8500);
        shot!(h, raw, direction = 0);
        shot!(h, raw, motion = handlers::SPREAD_BOMB);
        super::animation::initialize(h, raw, Initialization::Bomb);
        shot!(h, raw, x = h.session().player.x);
        shot!(h, raw, x_fraction = 0);
        shot!(h, raw, y = h.session().player.y);
        shot!(h, raw, y_fraction = 0);
        shot!(h, raw, x_speed = [0x8100, 0x8080, 0, 0x80, 0x100][pattern]);
        shot!(h, raw, trail_time = super::bombs::SPREAD_FRACTIONS[pattern]);
        let speed = 0u16.wrapping_sub(
            [0, 1, 2, 1, 0][pattern] + ((h.session().projectiles.controls.spread_charge >> 6) & 3),
        );
        shot!(h, raw, y_speed = speed);
        shot!(h, raw, auxiliary = speed);
        shot!(h, raw, variable = [120, 110, 100, 110, 120][pattern]);
    }
    h.session_mut().projectiles.bomb_count = 5;
    // Live byte at C23 after every initializer: high byte of slot five's kind.
    let key = shot!(h, 10, kind).to_le_bytes()[1] & 15;
    h.session_mut().projectiles.controls.cooldown =
        u16::from(definitions::COOLDOWNS[usize::from(key)]);
    h.session_mut().projectiles.controls.spread_charge = 0;
    h.session_mut().projectiles.controls.charge = 0;
}
