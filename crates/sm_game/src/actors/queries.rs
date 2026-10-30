//! Common actor/player spatial queries over borrowed actor state.
#![forbid(unsafe_code)]
use crate::actors::overlap::axis_overlaps;
use crate::geometry::Point;
use {
    crate::actors::api::{Axis, DistanceDirection, PlayerDirection},
    crate::actors::{
        access::{AccessMode::LittleEndian, ActorAccess, ActorKey},
        content::ActorContent,
        effects::ActorHost,
    },
};

fn current(h: &impl ActorAccess) -> ActorKey {
    ActorKey::from_legacy(h.session().actors.controls.selected)
}

pub fn touches_player_from_above<H: ActorAccess>(h: &H, key: ActorKey) -> bool {
    let actor = h.resolve_actor(key, LittleEndian);
    if !axis_overlaps(
        h.session().player.x,
        actor.x.whole,
        h.session().player.contact.x_radius,
        actor.radius.x,
    ) {
        return false;
    }
    let player_y = h.session().player.y;
    let actor_y = actor.y.whole;
    if player_y.wrapping_add(3).wrapping_sub(actor_y) as i16 >= 0 {
        return false;
    }
    let distance = actor_y.wrapping_sub(player_y.wrapping_add(3));
    let radius = h.session().player.contact.radius;
    distance < radius || distance.wrapping_sub(radius) <= actor.radius.y
}
pub fn overlaps_player_with_padding<H: ActorAccess>(h: &H, key: ActorKey) -> bool {
    let actor = h.resolve_actor(key, LittleEndian);
    let distance = (h.session().player.x.wrapping_sub(actor.x.whole) as i16).wrapping_abs() as u16;
    let radius = h.session().player.contact.x_radius;
    let gap = distance.wrapping_sub(radius);
    if distance >= radius && gap >= actor.radius.x && gap >= 8 {
        return false;
    }
    axis_overlaps(
        h.session().player.y,
        actor.y.whole,
        h.session().player.contact.radius,
        actor.radius.y,
    )
}
fn outside(position: u16, camera: u16, padding: u16, upper: u16) -> bool {
    (position.wrapping_add(padding).wrapping_sub(camera) as i16) < 0
        || (camera.wrapping_add(upper).wrapping_sub(position) as i16) < 0
}
pub fn current_outside_screen<H: ActorAccess>(h: &H) -> bool {
    let actor = h.resolve_actor(current(h), LittleEndian);
    outside(actor.x.whole, h.inputs().camera_x, 0, 256)
        || outside(actor.y.whole, h.inputs().camera_y, 0, 256)
}
pub fn current_outside_padded_screen<H: ActorAccess>(h: &H, padding: u16) -> bool {
    let actor = h.resolve_actor(current(h), LittleEndian);
    outside(
        actor.x.whole,
        h.inputs().camera_x,
        padding,
        padding.wrapping_add(256),
    ) || outside(
        actor.y.whole,
        h.inputs().camera_y,
        padding,
        padding.wrapping_add(256),
    )
}
pub fn current_normal_sprite_outside_screen<H: ActorAccess>(h: &H) -> bool {
    let actor = h.resolve_actor(current(h), LittleEndian);
    outside(
        actor.x.whole,
        h.inputs().camera_x,
        actor.radius.x,
        actor.radius.x.wrapping_add(256),
    ) || outside(actor.y.whole, h.inputs().camera_y, 8, 248)
}
pub fn leaving_screen<H: ActorAccess>(h: &H, key: ActorKey) -> bool {
    let actor = h.resolve_actor(key, LittleEndian);
    let x = actor.x.whole;
    if (x as i16) < 0 {
        return true;
    }
    let edge = actor
        .radius
        .x
        .wrapping_add(x)
        .wrapping_sub(h.inputs().camera_x);
    let past = edge.wrapping_sub(256).wrapping_sub(actor.radius.x);
    (edge as i16) < 0 || (past as i16) >= 0
}
pub fn player_delta<H: ActorAccess>(h: &H, key: ActorKey, axis: Axis) -> u16 {
    let actor = h.resolve_actor(key, LittleEndian);
    match axis {
        Axis::X => h.session().player.x.wrapping_sub(actor.x.whole),
        Axis::Y => h.session().player.y.wrapping_sub(actor.y.whole),
    }
}
pub fn player_within<H: ActorAccess>(h: &mut H, key: ActorKey, axis: Axis, distance: u16) -> bool {
    h.session_mut().compatibility_enemy_scratch = distance;
    let delta = player_delta(h, key, axis);
    (((delta as i16).wrapping_abs() as u16).wrapping_sub(distance) as i16) < 0
}
pub fn direction_to_player<H: ActorAccess>(h: &mut H) -> PlayerDirection {
    if player_within(h, current(h), Axis::Y, 32) {
        return if player_delta(h, current(h), Axis::X) & 0x8000 != 0 {
            PlayerDirection::Left
        } else {
            PlayerDirection::Right
        };
    }
    if player_within(h, current(h), Axis::X, 32) {
        return if player_delta(h, current(h), Axis::Y) & 0x8000 != 0 {
            PlayerDirection::Up
        } else {
            PlayerDirection::Down
        };
    }
    let left = player_delta(h, current(h), Axis::X) & 0x8000 != 0;
    let above = player_delta(h, current(h), Axis::Y) & 0x8000 != 0;
    match (left, above) {
        (true, true) => PlayerDirection::UpLeft,
        (true, false) => PlayerDirection::DownLeft,
        (false, true) => PlayerDirection::UpRight,
        (false, false) => PlayerDirection::DownRight,
    }
}
pub fn angle_from_delta(x: u16, y: u16) -> u16 {
    let quadrant = (u8::from(x & 0x8000 != 0) << 1) | u8::from(y & 0x8000 != 0);
    let x = (x as i16).wrapping_abs() as u16;
    let y = (y as i16).wrapping_abs() as u16;
    let x_dominant = y < x;
    let (numerator, denominator) = if x_dominant { (y, x) } else { (x, y) };
    let numerator = numerator.wrapping_shl(8);
    let denominator = denominator as u8;
    let divided = if denominator == 0 {
        0xffff
    } else {
        numerator / u16::from(denominator)
    };
    let offset = (divided >> 3) as u8;
    u16::from(match (x_dominant, quadrant) {
        (true, 0) => offset.wrapping_add(64),
        (true, 1) => 64u8.wrapping_sub(offset),
        (true, 2) => 192u8.wrapping_sub(offset),
        (true, 3) => offset.wrapping_sub(64),
        (false, 0) => 128u8.wrapping_sub(offset),
        (false, 1) => offset,
        (false, 2) => offset.wrapping_add(128),
        (false, 3) => 0u8.wrapping_sub(offset),
        _ => unreachable!(),
    })
}
pub fn angle_to_player<H: ActorAccess>(h: &H, key: ActorKey) -> u16 {
    let x = player_delta(h, key, Axis::X);
    let y = player_delta(h, key, Axis::Y);
    angle_from_delta(x, y)
}
pub fn angle_between<H: ActorAccess>(h: &H, from: ActorKey, to: ActorKey) -> u16 {
    let x = h
        .resolve_actor(from, LittleEndian)
        .x
        .whole
        .wrapping_sub(h.resolve_actor(to, LittleEndian).x.whole);
    let y = h
        .resolve_actor(from, LittleEndian)
        .y
        .whole
        .wrapping_sub(h.resolve_actor(to, LittleEndian).y.whole);
    angle_from_delta(x, y)
}
/// Shared with retained sine/cosine compatibility callers; no duplicate bank rule.
pub fn sine_product<H: ActorContent>(h: &mut H, amplitude: u16, phase: u16) -> u32 {
    let sample = h.actor_sine_sample((phase & 0x7f) as u8);
    let product = u16::from(sample) * u16::from(amplitude as u8);
    let mut integer = product >> 8;
    let mut fraction = product << 8;
    if phase & 0x80 != 0 {
        integer = integer.wrapping_neg();
        fraction = fraction.wrapping_neg();
    }
    (u32::from(integer) << 16) | u32::from(fraction)
}
pub fn distance_and_direction<H: ActorHost>(
    h: &mut H,
    origin: Point,
    target: Point,
) -> DistanceDirection {
    let dx = target.x.wrapping_sub(origin.x);
    let x = (dx as i16).wrapping_abs() as u16;
    if x.wrapping_sub(255) & 0x8000 == 0 {
        return DistanceDirection {
            distance: x,
            direction: 0,
        };
    }
    h.session_mut().compatibility_enemy_scratch = x;
    let dy = target.y.wrapping_sub(origin.y);
    let y = (dy as i16).wrapping_abs() as u16;
    if y.wrapping_sub(255) & 0x8000 == 0 {
        return DistanceDirection {
            distance: y,
            direction: 0,
        };
    }
    let angle = angle_from_delta(x, y);
    let horizontal = ((sine_product(h, x, u16::from(angle.wrapping_add(0x80) as u8)) >> 16) as i16)
        .wrapping_abs() as u16;
    let vertical = ((sine_product(h, y, u16::from(angle.wrapping_add(64) as u8)) >> 16) as i16)
        .wrapping_abs() as u16;
    DistanceDirection {
        distance: horizontal.wrapping_add(vertical),
        direction: angle_from_delta(dx, dy),
    }
}
