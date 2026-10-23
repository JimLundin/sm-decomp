//! Missile creation and live weapon selection over ordinary state.
#![forbid(unsafe_code)]
use super::host::Host;
use crate::game::{
    projectile_initialization::Initialization, projectile_weapon_definitions as definitions,
};
fn ammo(h: &impl Host) -> bool {
    if h.session().controls.selected_weapon == 2 {
        h.session().player.super_missiles != 0
    } else {
        h.session().player.missiles != 0
    }
}
pub(super) fn fire(h: &mut impl Host) {
    if h.inputs().shoot_button & h.inputs().pressed == 0
        && h.inputs().shoot_button & h.inputs().filtered_pressed == 0
    {
        return;
    }
    let capacity = if h.session().controls.selected_weapon == 2 {
        4
    } else {
        5
    };
    if h.session().count.wrapping_sub(capacity) & 0x8000 == 0
        || h.session().controls.cooldown as u8 != 0
    {
        return;
    }
    h.session_mut().controls.cooldown = 1;
    h.session_mut().count = h.session().count.wrapping_add(1);
    let slot = if ammo(h) {
        (0..5)
            .map(|i| 2 * i)
            .find(|&raw| shot!(h, raw, damage) == 0)
    } else {
        None
    };
    let Some(raw) = slot else {
        h.session_mut().count = h.session().count.wrapping_sub(1);
        return;
    };
    if !super::placement::place(h, raw) {
        return;
    }
    h.session_mut().controls.protection = 20;
    if h.session().controls.selected_weapon == 2 {
        h.session_mut().player.super_missiles = h.session().player.super_missiles.wrapping_sub(1);
    } else {
        h.session_mut().player.missiles = h.session().player.missiles.wrapping_sub(1);
    }
    shot!(h, raw, trail_time = 4);
    let key = h.session().controls.selected_weapon;
    let kind = h.session().controls.selected_weapon.swap_bytes();
    let kind = shot!(h, raw, kind) | kind | 0x8000;
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
        motion = if kind & 0x200 != 0 { 0xafe5 } else { 0xaf68 }
    );
    h.session_mut().controls.cooldown =
        u16::from(definitions::COOLDOWNS[usize::from(kind.to_ne_bytes()[1] & 15)]);
    if h.session().controls.automatic_weapon != 0 {
        h.session_mut().controls.selected_weapon = 0;
        h.session_mut().controls.automatic_weapon = 0;
    } else if !ammo(h) {
        h.session_mut().controls.selected_weapon = 0;
    }
}
