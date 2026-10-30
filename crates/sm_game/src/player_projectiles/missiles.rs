//! Missile creation and live weapon selection over ordinary state.
#![forbid(unsafe_code)]
use super::host::Host;
use crate::shot;
use crate::{
    projectile_initialization::Initialization, projectile_weapon_definitions as definitions,
};
fn ammo(h: &impl Host) -> bool {
    if h.session().projectiles.controls.selected_weapon == 2 {
        h.session().inventory.ammunition[1].current != 0
    } else {
        h.session().inventory.ammunition[0].current != 0
    }
}
pub fn fire(h: &mut impl Host) {
    if h.inputs().shoot_button & h.inputs().pressed == 0
        && h.inputs().shoot_button & h.inputs().filtered_pressed == 0
    {
        return;
    }
    let capacity = if h.session().projectiles.controls.selected_weapon == 2 {
        4
    } else {
        5
    };
    if h.session().projectiles.count.wrapping_sub(capacity) & 0x8000 == 0
        || h.session().projectiles.controls.cooldown as u8 != 0
    {
        return;
    }
    h.session_mut().projectiles.controls.cooldown = 1;
    h.session_mut().projectiles.count = h.session().projectiles.count.wrapping_add(1);
    let slot = if ammo(h) {
        (0..5)
            .map(|i| 2 * i)
            .find(|&raw| shot!(h, raw, damage) == 0)
    } else {
        None
    };
    let Some(raw) = slot else {
        h.session_mut().projectiles.count = h.session().projectiles.count.wrapping_sub(1);
        return;
    };
    if !super::placement::place(h, raw) {
        return;
    }
    h.session_mut().projectiles.controls.protection = 20;
    if h.session().projectiles.controls.selected_weapon == 2 {
        h.session_mut().inventory.ammunition[1].current =
            h.session().inventory.ammunition[1].current.wrapping_sub(1);
    } else {
        h.session_mut().inventory.ammunition[0].current =
            h.session().inventory.ammunition[0].current.wrapping_sub(1);
    }
    shot!(h, raw, trail_time = 4);
    let key = h.session().projectiles.controls.selected_weapon;
    let kind = h
        .session()
        .projectiles
        .controls
        .selected_weapon
        .swap_bytes();
    let kind = shot!(h, raw, kind) | kind | super::kinds::NO_SAMUS_CONTACT;
    shot!(h, raw, kind = kind);
    if h.inputs().cinematic == 0 {
        h.sound(1, 6, definitions::SOUNDS[usize::from(key & 15)]);
    }
    super::motion::initialize_speed(h, raw, 0);
    super::animation::initialize(h, raw, Initialization::Projectile);
    let kind = shot!(h, raw, kind);
    shot!(
        h,
        raw,
        motion = if kind & super::kinds::SUPER_MISSILE_BIT != 0 {
            super::handlers::SUPER_MISSILE
        } else {
            super::handlers::MISSILE
        }
    );
    h.session_mut().projectiles.controls.cooldown =
        u16::from(definitions::COOLDOWNS[usize::from(kind.to_ne_bytes()[1] & 15)]);
    if h.session().projectiles.controls.automatic_weapon != 0 {
        h.session_mut().projectiles.controls.selected_weapon = 0;
        h.session_mut().projectiles.controls.automatic_weapon = 0;
    } else if !ammo(h) {
        h.session_mut().projectiles.controls.selected_weapon = 0;
    }
}
