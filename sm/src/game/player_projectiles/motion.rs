//! Flight, release, reflection and numeric motion dispatch on ordinary session state.
#![forbid(unsafe_code)]
use super::{Host, animation, presentation};
use crate::game::projectile_initialization::Initialization;
use crate::game::projectile_room::Axis;
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Flight {
    NonWave,
    Wave,
    Missile,
    SuperMissile,
}
const AXES: [u8; 10] = [2, 3, 1, 3, 2, 2, 3, 1, 3, 2];
const X_ACCELERATION: [i16; 10] = [0, 16, 16, 16, 0, 0, -16, -16, -16, 0];
const Y_ACCELERATION: [i16; 10] = [-16, -16, 0, 16, 16, 16, 16, 0, -16, -16];
pub(super) fn clear(h: &mut impl Host, raw: u16) {
    shot!(h, raw, x = 0);
    shot!(h, raw, y = 0);
    shot!(h, raw, x_fraction = 0);
    shot!(h, raw, y_fraction = 0);
    shot!(h, raw, direction = 0);
    shot!(h, raw, x_speed = 0);
    shot!(h, raw, y_speed = 0);
    shot!(h, raw, x_radius = 0);
    shot!(h, raw, y_radius = 0);
    shot!(h, raw, kind = 0);
    shot!(h, raw, damage = 0);
    shot!(h, raw, cursor = 0);
    shot!(h, raw, remaining = 0);
    shot!(h, raw, variable = 0);
    shot!(h, raw, sprite = 0);
    shot!(h, raw, motion = 0xb169);
    let count = if (raw as i16).wrapping_sub(10) >= 0 {
        &mut h.session_mut().projectiles.bomb_count
    } else {
        &mut h.session_mut().projectiles.count
    };
    *count = count.wrapping_sub(1);
    if *count & 0x8000 != 0 {
        *count = 0;
    }
}
fn clear_extensions(h: &mut impl Host) {
    for raw in (0..5).rev().map(|i| i * 2) {
        if shot!(h, raw, kind) & 0xfff == 0x200 {
            clear(h, raw);
        }
    }
}
pub(super) fn update(h: &mut impl Host, mut raw: u16, kind: Flight, interval: u16) {
    h.probe("player_projectiles.flight");
    if shot!(h, raw, direction) & 0xf0 != 0 {
        clear(h, raw);
        if kind == Flight::SuperMissile {
            clear_extensions(h);
        }
        return;
    }
    if interval != 0 {
        let previous = shot_native!(h, raw, trail_time);
        shot!(h,raw,trail_time = previous.wrapping_sub(1); SliceNative);
        if previous == 1 {
            shot!(h, raw, trail_time = interval);
            presentation::emit(h, raw);
            raw = h.session().projectiles.selected;
        }
    }
    advance(h, raw, kind);
}
pub(super) fn advance(h: &mut impl Host, raw: u16, kind: Flight) {
    let direction = usize::from(shot!(h, raw, direction) & 15);
    if kind != Flight::SuperMissile {
        let old = shot!(h, raw, x_speed);
        shot!(
            h,
            raw,
            x_speed = old.wrapping_add(X_ACCELERATION[direction] as u16)
        );
        let old = shot!(h, raw, y_speed);
        shot!(
            h,
            raw,
            y_speed = old.wrapping_add(Y_ACCELERATION[direction] as u16)
        );
    }
    if matches!(kind, Flight::Missile | Flight::SuperMissile) {
        accelerate_missile(h, raw);
    }
    let axes = AXES[direction];
    collide(h, axes, kind);
    if cull(h) && kind == Flight::SuperMissile {
        clear_extensions(h);
    }
}
pub(super) fn collide(h: &mut impl Host, axes: u8, kind: Flight) {
    // Both axes retain the selected slot across horizontal callbacks.
    let raw = h.session().projectiles.selected;
    let collision = if kind == Flight::SuperMissile {
        Flight::Missile
    } else {
        kind
    };
    if axes & 1 != 0 {
        let value = super::collision::scan(h, raw, collision, Axis::X);
        let hit = if matches!(kind, Flight::NonWave | Flight::Wave) {
            value & 1 != 0
        } else {
            value != 0
        };
        if kind == Flight::SuperMissile {
            extension_collision(h, Axis::X);
        }
        if hit || axes & 2 == 0 {
            return;
        }
    }
    // The original empty mask takes the vertical path too.
    super::collision::scan(h, raw, collision, Axis::Y);
    if kind == Flight::SuperMissile {
        extension_collision(h, Axis::Y);
    }
}
pub(super) fn update_extension(h: &mut impl Host, raw: u16) {
    if shot!(h, raw, direction) & 0xf0 != 0 {
        clear(h, raw);
        clear_extensions(h);
    }
}
fn visible(position: u16, camera: u16) -> bool {
    let delta = position.wrapping_sub(camera);
    delta.wrapping_add(64) & 0x8000 == 0 && delta.wrapping_sub(320) & 0x8000 != 0
}
fn cull(h: &mut impl Host) -> bool {
    if visible(
        shot_native!(h, h.session().projectiles.selected, x),
        h.inputs().camera_x,
    ) && visible(
        shot_native!(h, h.session().projectiles.selected, y),
        h.inputs().camera_y,
    ) {
        return false;
    }
    let raw = h.session().projectiles.selected;
    clear(h, raw);
    true
}
pub(super) fn initialize_speed(h: &mut impl Host, raw: u16, speed: u16) {
    use crate::game::launch_motion::Aim;
    shot!(h,raw,x_fraction = 0; SliceLe);
    shot!(h,raw,y_fraction = 0; SliceLe);
    let Some(aim) = Aim::from_direction(shot!(h, raw, direction)) else {
        h.unknown();
        return;
    };
    let velocity =
        h.session()
            .projectiles
            .launch
            .launch(aim, speed, h.inputs().absolute_y_fraction);
    shot!(h,raw,y_speed = velocity.y as u16; SliceLe);
    // A broad Y-speed write can change the launch history observed by X.
    let velocity =
        h.session()
            .projectiles
            .launch
            .launch(aim, speed, h.inputs().absolute_y_fraction);
    shot!(h,raw,x_speed = velocity.x as u16; SliceLe);
}
pub(super) fn initialize_beam_speed(h: &mut impl Host, raw: u16) {
    use crate::game::launch_motion::{BEAM_SPEEDS, beam_selection};
    let (index, invalid) = beam_selection(shot!(h, raw, kind), shot!(h, raw, direction));
    if invalid {
        h.unknown();
    }
    initialize_speed(h, raw, BEAM_SPEEDS[index]);
}
fn accelerate_missile(h: &mut impl Host, raw: u16) {
    let variable = shot!(h, raw, variable);
    if variable & 0xff00 != 0 {
        let kind = shot!(h, raw, kind);
        let direction = shot!(h, raw, direction);
        let address = if kind & 0x200 != 0 { 0xc32b } else { 0xc303 } + 4 * (direction & 15);
        let record = h.record(0x90, address);
        let old = shot!(h, raw, x_speed);
        shot!(h, raw, x_speed = old.wrapping_add(record.word(0)));
        let old = shot!(h, raw, y_speed);
        shot!(h, raw, y_speed = old.wrapping_add(record.word(2)));
    } else {
        let speed = variable + 0x100;
        shot!(h, raw, variable = speed);
        initialize_speed(h, raw, speed);
        if shot!(h, raw, kind) & 0x200 != 0 {
            spawn_extension(h);
        }
    }
}
fn extension_collision(h: &mut impl Host, axis: Axis) {
    let parent = h.session().projectiles.selected;
    if !matches!(shot!(h, parent, kind) & 0xf00, 0x200 | 0x800) {
        return;
    }
    let child = u16::from(shot!(h, parent, variable) as u8);
    // Unlike word access, this high lane uses raw parent+1 without rounding.
    if h.variable_byte(parent, true) != 0 {
        contact_extension(h, parent, child, axis);
    }
}
fn shift_extension(h: &mut impl Host, parent: u16, child: u16, axis: Axis, delta: i16) {
    match axis {
        Axis::X => {
            let value = if delta > 0 {
                shot_native!(h, parent, x)
            } else {
                shot!(h, parent, x)
            };
            shot!(h,child,x = value.wrapping_add_signed(delta); SliceLe);
        }
        Axis::Y => {
            let value = if delta > 0 {
                shot_native!(h, parent, y)
            } else {
                shot!(h, parent, y)
            };
            shot!(h,child,y = value.wrapping_add_signed(delta); SliceLe);
        }
    }
}
fn contact_extension(h: &mut impl Host, parent: u16, child: u16, axis: Axis) {
    let speed = (match axis {
        Axis::X => shot!(h, parent, x_speed),
        Axis::Y => shot!(h, parent, y_speed),
    }) as i16;
    let magnitude = speed.unsigned_abs() & 0xff00;
    if magnitude.wrapping_sub(0xb00) & 0x8000 != 0 {
        if shot!(h, parent, kind) & 0xf00 == 0x800 {
            clear(h, child);
            return;
        }
        shift_extension(h, parent, child, axis, 0);
    } else {
        let distance = (magnitude >> 8) as i16 - 10;
        let delta = if speed < 0 { distance } else { -distance };
        shift_extension(h, parent, child, axis, delta);
        h.session_mut().projectiles.selected = child;
        super::collision::scan(h, child, Flight::Missile, axis);
        // Restore only after the child collision returns successfully.
        h.session_mut().projectiles.selected = parent;
        if shot!(h, parent, kind) & 0xf00 == 0x800 {
            clear(h, child);
        }
    }
}
pub(super) fn link_extension(h: &mut impl Host, raw: u16) {
    let parent = h.session().projectiles.selected;
    let value = shot_native!(h, parent, variable);
    shot!(h,parent,variable = (value&0xff00).wrapping_add(raw); SliceNative);
}
fn spawn_extension(h: &mut impl Host) {
    let Some(child) = (0..5)
        .map(|i| i * 2)
        .find(|&raw| shot!(h, raw, damage) == 0)
    else {
        return;
    };
    let kind = shot!(h, child, kind);
    shot!(h, child, kind = kind | 0x8200);
    let parent = h.session().projectiles.selected;
    shot!(h, child, x = shot!(h, parent, x));
    shot!(h, child, y = shot!(h, parent, y));
    shot!(h, child, direction = shot!(h, parent, direction));
    // Rejected placement still initializes, arms, links and counts this child.
    super::placement::place(h, child);
    animation::initialize(h, child, Initialization::SuperMissile);
    shot!(h, child, motion = 0xb075);
    link_extension(h, child);
    h.session_mut().projectiles.count = h.session().projectiles.count.wrapping_add(1);
}
pub(super) fn point(angle: u16, radius: u16) -> (u16, u16) {
    let table = &crate::game::projectile_geometry::SINE_COSINE;
    crate::game::special_geometry::point(angle, radius, table.len(), |index| {
        table.get(index).map(|&v| u16::from_le(v as u16))
    })
    .unwrap_or_else(|error| {
        let _ = table[error.index];
        unreachable!()
    })
}
fn speed_echo(h: &mut impl Host, raw: u16) {
    let index = usize::from(raw >> 1);
    let speed = shot!(h, raw, x_speed);
    shot!(h, raw, x_speed = speed.wrapping_add(8));
    let (x, y) = point(
        shot!(h, raw, variable),
        u16::from(shot!(h, raw, x_speed) as u8),
    );
    let position = x.wrapping_add(h.session().player.x);
    h.set_echo_index_native(raw, x.wrapping_add(h.session().player.x));
    shot!(h, raw, x = position);
    let dx = (position as i16).wrapping_sub(h.inputs().camera_x as i16);
    let outside = if dx < 0 || (i32::from(dx) - 256) & 0x8000 == 0 {
        true
    } else {
        let position = y.wrapping_add(h.session().player.y);
        h.set_echo_x(index + 3, y.wrapping_add(h.session().player.y));
        shot!(h, raw, y = position);
        let dy = (position as i16).wrapping_sub(h.inputs().camera_y as i16);
        dy < 0 || (i32::from(dy) - 256) & 0x8000 == 0
    };
    if outside {
        h.set_echo_y(index + 3, 0);
        h.set_echo_index_native(raw, 0);
        h.set_echo_x(index + 3, 0);
        clear(h, raw);
    }
}
fn suit_pickup_shot(h: &mut impl Host, raw: u16) {
    const X: [i16; 4] = [-4, -4, 4, 4];
    const Y: [i16; 4] = [4, -4, -4, 4];
    const RESTART_X: [i16; 4] = [0x80, 0x80, -0x80, -0x80];
    const RESTART_Y: [i16; 4] = [-0x80, 0x80, 0x80, -0x80];
    let index = usize::from(raw >> 1);
    let old = shot!(h, raw, x);
    shot!(h, raw, x = old.wrapping_add(X[index] as u16));
    let y = (Y[index] as u16).wrapping_add(shot!(h, raw, y));
    shot!(h, raw, y = y);
    if y == h.session().player.y {
        if shot!(h, raw, variable) == 1 {
            if raw == 6 {
                h.session_mut().player.movement_handler =
                    crate::game::player_transitions::state::MovementMode::ShinesparkProjectileResponse;
            }
            clear(h, raw);
        } else {
            let old = shot!(h, raw, variable);
            shot!(h, raw, variable = old.wrapping_add(1));
            h.session_mut().player.shine_timer = 180;
            h.session_mut().player.shine_tick = 1;
            h.session_mut().player.palette_frame = 0;
            shot!(
                h,
                raw,
                x = (RESTART_X[index] as u16).wrapping_add(h.session().player.x)
            );
            shot!(
                h,
                raw,
                y = (RESTART_Y[index] as u16).wrapping_add(h.session().player.y)
            );
        }
    }
}
pub(super) fn reflect(h: &mut impl Host, raw: u16) {
    let kind = shot!(h, raw, kind);
    if kind & 0x100 != 0 {
        animation::initialize(h, raw, Initialization::Projectile);
        shot!(h, raw, motion = 0xaf68);
        shot!(h, raw, variable = 240);
    } else if kind & 0x200 != 0 {
        let child = u16::from(shot!(h, raw, variable) as u8);
        clear(h, child);
        animation::initialize(h, raw, Initialization::Projectile);
        shot!(h, raw, motion = 0xafe5);
        shot!(h, raw, variable = 240);
    } else {
        initialize_beam_speed(h, raw);
        animation::initialize(h, raw, Initialization::Projectile);
        shot!(
            h,
            raw,
            motion = super::weapons::charged_instruction(usize::from(shot!(h, raw, kind) & 15))
        );
    }
}
pub(super) fn advance_all(h: &mut impl Host) {
    h.session_mut().projectiles.selected = 18;
    let mut cursor = 18i32;
    while cursor >= 0 {
        let raw = cursor as u16;
        if shot!(h, raw, cursor) != 0 {
            let code = shot!(h, raw, motion);
            run(h, code, raw);
            animation::advance(h);
            cursor = i32::from(h.session().projectiles.selected);
        }
        cursor -= 2;
        h.session_mut().projectiles.selected = cursor as u16;
    }
}
pub(super) fn run(h: &mut impl Host, code: u16, raw: u16) {
    use super::{bombs::Kind as Bomb, specials::Kind as Special};
    match code {
        0xd4d2 => speed_echo(h, raw),
        0xefd3 => suit_pickup_shot(h, raw),
        0xd793 => super::specials::update(h, raw, Special::Plasma),
        0xcf09 => super::specials::update(h, raw, Special::Ice),
        0xcf7a => super::specials::update(h, raw, Special::IceReleased),
        0xda08 => super::specials::update(h, raw, Special::Wave),
        0xdb06 => super::specials::update(h, raw, Special::Spazer),
        0xdc9c => super::specials::update(h, raw, Special::SpazerReleased),
        0xd7e1 => super::specials::plasma_radius(h, raw, false),
        0xd7fa => super::specials::plasma_radius(h, raw, true),
        0xd813 => super::specials::depart_plasma(h, raw),
        0xb099 => super::bombs::bomb_update(h, raw, Bomb::Bomb),
        0xb0ae => super::bombs::bomb_update(h, raw, Bomb::PowerBomb),
        0xd8f7 => super::bombs::bomb_update(h, raw, Bomb::Spread),
        0xb169 => (),
        0xaef3 => update(h, raw, Flight::NonWave, 4),
        0xaf4a => collide(h, 2, Flight::NonWave),
        0xaf52 => collide(h, 3, Flight::NonWave),
        0xaf60 => collide(h, 1, Flight::NonWave),
        0xaf68 => update(h, raw, Flight::Missile, 4),
        0xafc7 => collide(h, 2, Flight::Missile),
        0xafcf => collide(h, 3, Flight::Missile),
        0xafdd => collide(h, 1, Flight::Missile),
        0xafe5 => update(h, raw, Flight::SuperMissile, 2),
        0xb047 => collide(h, 2, Flight::SuperMissile),
        0xb052 => collide(h, 3, Flight::SuperMissile),
        0xb06a => collide(h, 1, Flight::SuperMissile),
        0xb075 => update_extension(h, raw),
        0xb0c3 => update(h, raw, Flight::Wave, 4),
        0xb0e4 => update(h, raw, Flight::Wave, 3),
        0xb103 => advance(h, raw, Flight::Wave),
        0xb13b => collide(h, 2, Flight::Wave),
        0xb143 => collide(h, 3, Flight::Wave),
        0xb151 => collide(h, 1, Flight::Wave),
        0xb159 => update(h, raw, Flight::Wave, 0),
        _ => {
            h.unknown();
        }
    }
}
pub(crate) fn reset(h: &mut impl Host) {
    for shot in &mut h.session_mut().projectiles.shots {
        shot.trail_time = 0;
        shot.x = 0;
        shot.y = 0;
        shot.direction = 0;
        shot.x_speed = 0;
        shot.y_speed = 0;
        shot.x_radius = 0;
        shot.y_radius = 0;
        shot.kind = 0;
        shot.damage = 0;
        shot.cursor = 0;
        shot.remaining = 0;
        shot.variable = 0;
        shot.sprite = 0;
        shot.motion = 0xb169;
    }
    h.session_mut().projectiles.bomb_count = 0;
    h.session_mut().projectiles.controls.cooldown = 0;
    h.session_mut().projectiles.count = 0;
    h.session_mut().projectiles.power_bomb.flag = 0;
    if h.inputs().hud_auto_cancel != 0 {
        h.session_mut().projectiles.controls.selected_weapon = 0;
        h.session_mut().projectiles.controls.automatic_weapon = 0;
    }
    h.session_mut().projectiles.echoes.x.fill(0);
    h.session_mut().projectiles.echoes.y.fill(0);
    h.session_mut().projectiles.echoes.x_speed.fill(0);
    h.session_mut().projectiles.echoes.index = 0;
    if h.session().player.special_palette_flags != 0 {
        h.session_mut().player.special_palette_flags = 0;
        h.palette(super::host::PaletteEffect::SuitTarget);
    }
    if h.inputs().hyper != 0 {
        h.palette(super::host::PaletteEffect::Spawn(0xe1f0));
    }
    h.probe("player_projectiles.reset.completed");
}
