//! Complete special-beam creation, orbit and retirement over ordinary state.
use super::host::Host;
use crate::projectile_initialization::Initialization;
use crate::{shot, shot_native};
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    Wave,
    Ice,
    IceReleased,
    Plasma,
    Spazer,
    SpazerReleased,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Attack {
    Wave,
    Ice,
    Spazer,
    Plasma,
}
fn instruction(kind: Kind) -> u16 {
    match kind {
        Kind::Wave => super::handlers::WAVE_SPECIAL,
        Kind::Ice => super::handlers::ICE_SPECIAL,
        Kind::IceReleased => super::handlers::ICE_SPECIAL_RELEASED,
        Kind::Plasma => super::handlers::PLASMA_SPECIAL,
        Kind::Spazer => super::handlers::SPAZER_SPECIAL,
        Kind::SpazerReleased => super::handlers::SPAZER_SPECIAL_RELEASED,
    }
}
fn invalid_plasma_phase(h: &mut impl Host) -> ! {
    h.unknown();
    loop {
        core::hint::spin_loop();
    }
}
// Compile-time field tokens keep both pursuit axes on the same arithmetic.
macro_rules! pursue {
    ($h:ident, $raw:expr, $position:ident, $fraction:ident, $speed:ident, $target:expr) => {{
        let raw = $raw;
        let target = $target;
        let previous = shot!($h, raw, $speed);
        if (target as i16).wrapping_sub(shot!($h, raw, $position) as i16) < 0 {
            if previous.wrapping_add(2047) & 0x8000 == 0 { shot!($h, raw, $speed = previous.wrapping_sub(64); SliceLe); }
        } else if previous.wrapping_sub(2048) & 0x8000 != 0 { shot!($h, raw, $speed = previous.wrapping_add(64); SliceLe); }
        let speed = shot!($h, raw, $speed);
        let low = u32::from(shot_native!($h, raw, $fraction));
        let high = u32::from(shot_native!($h, raw, $position));
        let next = (low | (high << 16)).wrapping_add((i32::from(speed as i16) << 8) as u32);
        shot!($h, raw, $fraction = next as u16; SliceNative);
        shot!($h, raw, $position = (next >> 16) as u16; SliceNative);
    }};
}
pub fn fire(h: &mut impl Host) -> u8 {
    if h.session().projectiles.controls.selected_weapon != 3 {
        return 0;
    }
    let selector = usize::from(h.session().inventory.beams.active & 15);
    let costs = [0i16, 1, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0];
    let cost = costs[selector];
    let remaining = (h.session().inventory.ammunition[2].current as i16)
        .wrapping_sub(cost)
        .max(0) as u16;
    h.session_mut().inventory.ammunition[2].current = remaining;
    let attack = match selector {
        1 => Some(Attack::Wave),
        2 => Some(Attack::Ice),
        4 => Some(Attack::Spazer),
        8 => Some(Attack::Plasma),
        _ => None,
    };
    let result = if let Some(attack) = attack {
        create(h, attack)
    } else {
        0
    };
    if h.session().inventory.ammunition[2].current == 0 {
        h.session_mut().projectiles.controls.selected_weapon = 0;
        h.session_mut().projectiles.controls.automatic_weapon = 0;
    }
    result
}
pub fn create(h: &mut impl Host, attack: Attack) -> u8 {
    if (attack == Attack::Ice
        && ((shot!(h, 0, motion) == instruction(Kind::IceReleased))
            || (shot!(h, 0, motion) == instruction(Kind::Ice))))
        || (attack == Attack::Plasma && (shot!(h, 0, motion) == instruction(Kind::Plasma)))
    {
        return 0;
    }
    crate::probe::hit(match attack {
        Attack::Wave => "special_beam.wave",
        Attack::Ice => "special_beam.ice",
        Attack::Spazer => "special_beam.spazer",
        Attack::Plasma => "special_beam.plasma",
    });
    for index in (0..4).rev() {
        let slot = 2 * index as u16;
        let initialization = match attack {
            Attack::Wave => {
                shot!(h, slot, trail_time = 4; SliceLe);
                let weapon = h.session().inventory.beams.active & 0x100f | 0x8010;
                shot!(h, slot, kind = weapon; SliceLe);
                shot!(h, slot, direction = 0; SliceLe);
                shot!(h, slot, motion = instruction(Kind::Wave));
                shot!(h, slot, y_speed = 600; SliceLe);
                shot!(h, slot, x_speed = 0; SliceLe);
                shot!(h, slot, variable = 0; SliceLe);
                shot!(h, slot, x_fraction = 0; SliceLe);
                shot!(h, slot, y_fraction = 0; SliceLe);
                let x = h
                    .session()
                    .player
                    .x
                    .wrapping_add([128, 128, 0xff80, 0xff80][index]);
                shot!(h, slot, x = x; SliceLe);
                let y = h
                    .session()
                    .player
                    .y
                    .wrapping_add([128, 0xff80, 0xff80, 128][index]);
                shot!(h, slot, y = y; SliceLe);
                Initialization::SpecialAttack
            }
            Attack::Ice => {
                shot!(h, slot, trail_time = 4; SliceLe);
                let weapon = h.session().inventory.beams.active & 0x100f | 0x8010;
                shot!(h, slot, kind = weapon; SliceLe);
                shot!(h, slot, direction = 0; SliceLe);
                shot!(h, slot, motion = instruction(Kind::Ice));
                shot!(h, slot, variable = 64 * index as u16; SliceLe);
                shot!(h, slot, y_speed = 600; SliceLe);
                Initialization::Projectile
            }
            Attack::Spazer => {
                shot!(h, slot, trail_time = [0, 0, 4, 4][index]; SliceLe);
                shot!(h, slot, direction = 5; SliceLe);
                shot!(h, slot, motion = instruction(Kind::Spazer));
                shot!(h, slot, x_speed = 40; SliceLe);
                shot!(h, slot, y_speed = [4, 0xfffc, 4, 0xfffc][index]; SliceLe);
                shot!(h, slot, variable = 0; SliceLe);
                shot!(h, slot, auxiliary = 0; SliceLe);
                shot!(h, slot, x_fraction = 0; SliceLe);
                shot!(h, slot, y_fraction = 0; SliceLe);
                if index >= 2 {
                    shot!(h, slot, kind = 0x8024; SliceLe);
                    Initialization::Echo
                } else {
                    let weapon = h.session().inventory.beams.active & 0x100f | 0x8010;
                    shot!(h, slot, kind = weapon; SliceLe);
                    Initialization::SpecialAttack
                }
            }
            Attack::Plasma => {
                let weapon = h.session().inventory.beams.active & 0x100f | 0x8010;
                shot!(h, slot, kind = weapon; SliceLe);
                shot!(h, slot, direction = 0; SliceLe);
                shot!(h, slot, motion = instruction(Kind::Plasma));
                shot!(h, slot, variable = 64 * index as u16; SliceLe);
                shot!(h, slot, x_speed = 40; SliceLe);
                shot!(h, slot, y_speed = 0; SliceLe);
                Initialization::SpecialAttack
            }
        };
        super::animation::initialize(h, slot, initialization);
    }
    h.session_mut().projectiles.count = 4;
    let index = usize::from(shot!(h, 0, kind) & 0x3f);
    let cooldown = u16::from(crate::beam_definitions::MANUAL_COOLDOWN[index]);
    h.session_mut().projectiles.controls.cooldown = cooldown;
    let step = match attack {
        Attack::Wave => 4,
        Attack::Spazer => 0,
        _ => {
            if u16::from(h.session().player.movement_and_facing as u8) == 4 {
                0xfffc
            } else {
                4
            }
        }
    };
    h.session_mut().projectiles.controls.orbit_step = step;
    h.sound(
        1,
        6,
        match attack {
            Attack::Wave => 0x28,
            Attack::Ice => 0x23,
            Attack::Spazer => 0x25,
            Attack::Plasma => 0x27,
        },
    );
    1
}
pub fn update(h: &mut impl Host, mut selected: u16, kind: Kind) {
    let slot = selected & !1;
    if shot!(h, slot, direction) & 0xf0 != 0 {
        if matches!(kind, Kind::Wave | Kind::Ice) {
            h.sound(1, 6, if kind == Kind::Wave { 0x29 } else { 0x24 });
        }
        if kind == Kind::Spazer {
            let paired = [4, 6, 0, 2][usize::from(slot >> 1)];
            super::motion::clear(h, selected);
            return super::motion::clear(h, paired);
        }
        return super::motion::clear(h, selected);
    }
    if kind == Kind::Plasma {
        hold(h);
        let angle = shot!(h, slot, variable);
        let radius = shot!(h, slot, x_speed);
        let point = super::motion::point(angle, radius);
        let x = point.0.wrapping_add(h.session().player.x);
        shot!(h, slot, x = x; SliceLe);
        let y = point.1.wrapping_add(h.session().player.y);
        shot!(h, slot, y = y; SliceLe);
        orbit_step(h, slot);
        return match shot!(h, slot, y_speed).wrapping_mul(2) {
            0 => plasma_radius(h, selected, false),
            2 => plasma_radius(h, selected, true),
            4 => depart_plasma(h, selected),
            _ => invalid_plasma_phase(h),
        };
    }
    if kind == Kind::Wave {
        let previous = shot!(h, slot, y_speed);
        shot!(h, slot, y_speed = previous.wrapping_sub(1); SliceLe);
        if previous == 1 || (previous as i16).wrapping_sub(1) < 0 {
            h.sound(1, 6, 0x29);
            return super::motion::clear(h, selected);
        }
    }
    let previous_x = shot!(h, slot, x_speed);
    let previous = shot!(h, slot, trail_time);
    shot!(h, slot, trail_time = previous.wrapping_sub(1); SliceLe);
    if previous == 1 {
        shot!(h, slot, trail_time = 4; SliceLe);
        super::presentation::emit(h, selected);
        selected = h.session().projectiles.selected;
    }
    resume(h, selected, kind, slot, previous_x)
}
pub fn resume(h: &mut impl Host, selected: u16, kind: Kind, original: u16, previous_x: u16) {
    if kind == Kind::Wave {
        let target = h.session().player.x;
        pursue!(h, original, x, x_fraction, x_speed, target);
        let target = h.session().player.y;
        pursue!(h, original, y, y_fraction, variable, target);
        if selected == 6 && (shot!(h, 6, x_speed) ^ previous_x) & 0x8000 != 0 {
            h.sound(1, 6, 0x28);
        }
    } else if kind == Kind::SpazerReleased {
        let y = shot!(h, selected, y).wrapping_add(8);
        shot!(h, selected, y = y; SliceLe);
        if y.wrapping_sub(h.inputs().camera_y).wrapping_sub(248) & 0x8000 == 0 {
            return super::motion::clear(h, selected);
        }
    } else {
        let radius = if kind == Kind::Ice {
            32
        } else {
            shot!(h, selected, x_speed)
        };
        let angle = shot!(h, selected, variable);
        let point = super::motion::point(angle, radius);
        let x = point.0.wrapping_add(h.session().player.x);
        shot!(h, selected, x = x; SliceLe);
        if kind == Kind::IceReleased && outside(shot!(h, selected, x), h.inputs().camera_x, 32, 288)
        {
            return super::motion::clear(h, selected);
        }
        if kind == Kind::Spazer {
            advance_spazer(h, selected, point.1);
        } else {
            let y = point.1.wrapping_add(h.session().player.y);
            shot!(h, selected, y = y; SliceLe);
            if kind == Kind::IceReleased
                && outside(shot!(h, selected, y), h.inputs().camera_y, 0xfff0, 256)
            {
                return super::motion::clear(h, selected);
            }
            orbit_step(h, selected);
            if kind == Kind::IceReleased {
                let radius = u16::from((shot!(h, selected, x_speed) as u8).wrapping_add(8));
                shot!(h, selected, x_speed = radius; SliceLe);
            } else {
                let previous = shot!(h, selected, y_speed);
                shot!(h, selected, y_speed = previous.wrapping_sub(1); SliceLe);
                if previous == 1 {
                    shot!(h, selected, motion = instruction(Kind::IceReleased));
                    shot!(h, selected, x_speed = 40; SliceLe);
                    h.sound(1, 6, 0x24);
                }
            }
        }
    }
    hold(h);
}
pub fn plasma_radius(h: &mut impl Host, selected: u16, contracting: bool) {
    let radius = shot!(h, selected, x_speed) as u8;
    let radius = if contracting {
        radius.wrapping_sub(4)
    } else {
        radius.wrapping_add(4)
    };
    shot!(h, selected, x_speed = u16::from(radius); SliceLe);
    if if contracting {
        radius < 45
    } else {
        radius >= 192
    } {
        shot!(h, selected, y_speed = if contracting { 2 } else { 1 }; SliceLe);
    }
}
pub fn depart_plasma(h: &mut impl Host, selected: u16) {
    if outside(shot!(h, selected, x), h.inputs().camera_x, 32, 288)
        || outside(shot!(h, selected, y), h.inputs().camera_y, 0xfff0, 256)
    {
        super::motion::clear(h, selected)
    } else {
        let radius = u16::from((shot!(h, selected, x_speed) as u8).wrapping_add(4));
        shot!(h, selected, x_speed = radius; SliceLe);
    }
}
pub fn advance_spazer(h: &mut impl Host, selected: u16, y: u16) {
    let index = usize::from(selected >> 1);
    let phase = usize::from(shot!(h, selected, auxiliary) >> 1);
    sm_data::table::check_index(phase, 3);
    let y = y
        .wrapping_add(h.session().player.y)
        .wrapping_sub(if phase == 0 { 0 } else { 114 });
    shot!(h, selected, y = y; SliceLe);
    if phase != 0 && y.wrapping_sub(h.inputs().camera_y).wrapping_sub(16) & 0x8000 != 0 {
        return finish_spazer(h, selected);
    }
    let step = (shot!(h, selected, y_speed) as u8);
    let angle = (shot!(h, selected, variable) as u8).wrapping_add(step);
    shot!(h, selected, variable = u16::from(angle); SliceLe);
    if phase == 0 {
        if angle == 128 {
            shot!(h, selected, x_speed = 160; SliceLe);
            let speed = [2, 0xfffe, 2, 0xfffe][index];
            shot!(h, selected, y_speed = speed; SliceLe);
            shot!(h, selected, direction = 0; SliceLe);
            shot!(h, selected, auxiliary = 2; SliceLe);
        }
    } else if phase == 1 {
        let radius = shot!(h, selected, x_speed).wrapping_sub(5);
        shot!(h, selected, x_speed = radius; SliceLe);
        if radius == 0 {
            let speed = [0xfffe, 2, 0xfffe, 2][index];
            shot!(h, selected, y_speed = speed; SliceLe);
            let angle = u16::from((shot!(h, selected, variable) as u8).wrapping_add(128));
            shot!(h, selected, variable = angle; SliceLe);
            shot!(h, selected, auxiliary = 4; SliceLe);
            if selected == 0 {
                h.sound(1, 6, 0x26);
            }
        }
    } else {
        let radius = shot!(h, selected, x_speed).wrapping_add(5);
        shot!(h, selected, x_speed = radius; SliceLe);
        if radius.wrapping_sub(96) & 0x8000 == 0 {
            finish_spazer(h, selected)
        }
    }
}
pub fn finish_spazer(h: &mut impl Host, selected: u16) {
    let index = usize::from(selected >> 1);
    // The compound assignment reads X before demanding the table entry.
    let x = shot!(h, selected, x);
    let offset = [16, 16, 0xfff0, 0xfff0][index];
    shot!(h, selected, x = x.wrapping_add(offset); SliceLe);
    shot!(h, selected, direction = 5; SliceLe);
    shot!(h, selected, trail_time = 4; SliceLe);
    shot!(h, selected, motion = instruction(Kind::SpazerReleased));
    if (selected as i16).wrapping_sub(4) < 0 {
        shot!(h, selected, kind = 0x8024; SliceLe);
        super::animation::initialize(h, selected, Initialization::Echo);
    }
}

pub fn hold(h: &mut impl Host) {
    h.session_mut().projectiles.controls.cooldown = 2;
    h.session_mut().projectiles.controls.charge = 0;
}
pub fn orbit_step(h: &mut impl Host, slot: u16) {
    let angle = (h.session().projectiles.controls.orbit_step as u8)
        .wrapping_add(shot!(h, slot, variable) as u8);
    shot!(h, slot, variable = u16::from(angle); SliceLe);
}
fn outside(position: u16, camera: u16, lower: u16, upper: u16) -> bool {
    let relative = position.wrapping_sub(camera);
    relative.wrapping_add(lower) & 0x8000 != 0 || relative.wrapping_sub(upper) & 0x8000 == 0
}
