//! Common actor/player spatial queries over borrowed actor state.
#![forbid(unsafe_code)]
use super::{
    access::{AccessMode::LittleEndian, ActorAccess, ActorCollections, ActorKey, CollectionWindow},
    api::{Axis, DistanceDirection, PlayerDirection, SolidContact},
    content::ActorContent,
    effects::ActorHost,
};
use crate::game::session::Point;

fn current(h: &impl ActorAccess) -> ActorKey {
    ActorKey::from_legacy(h.session().actors.controls.selected)
}
fn axis_overlaps(actor: u16, player: u16, actor_radius: u16, player_radius: u16) -> bool {
    let distance = (actor.wrapping_sub(player) as i16).wrapping_abs() as u16;
    distance < actor_radius || distance.wrapping_sub(actor_radius) < player_radius
}

fn probe_coordinate(position: u16, fraction: u16, amount: i32, negative: bool) -> u16 {
    let integer = (amount >> 16) as u16;
    let fractional = amount as u16;
    if negative {
        let mut coordinate = position.wrapping_sub(integer);
        let mut exact = fraction == fractional;
        if fraction < fractional {
            // Equality is tested before borrowing the integer coordinate.
            exact = coordinate == 1;
            coordinate = coordinate.wrapping_sub(1);
        }
        if !exact {
            coordinate = coordinate.wrapping_sub(1);
        }
        coordinate
    } else {
        let mut coordinate = position.wrapping_add(integer);
        let sum = u32::from(fraction) + u32::from(fractional);
        let mut exact = sum == 0;
        if sum > 0xffff {
            exact = coordinate == 0xffff;
            coordinate = coordinate.wrapping_add(1);
        }
        if !exact {
            coordinate = coordinate.wrapping_add(1);
        }
        coordinate
    }
}
fn player_probe(h: &impl ActorAccess, amount: i32) -> (Point, Point) {
    let direction = h.session().player.contact.collision_direction & 3;
    let position = if direction < 2 {
        Point {
            x: probe_coordinate(
                h.session().player.x,
                h.session().player.x_fraction,
                amount,
                direction == 0,
            ),
            y: h.session().player.y,
        }
    } else {
        let y = probe_coordinate(
            h.session().player.y,
            h.session().player.y_fraction,
            amount,
            direction == 2,
        );
        Point {
            x: h.session().player.x,
            y,
        }
    };
    (
        position,
        Point {
            x: h.inputs().player_x_radius,
            y: h.session().player.contact.radius,
        },
    )
}
pub(in crate::game) fn probe_player_solid_contact<H: ActorAccess + ActorCollections>(
    h: &mut H,
    amount: i32,
) -> SolidContact {
    if h.session().actors.schedule.interactive_write_position == 0 {
        return SolidContact::Clear { amount };
    }
    let (position, radius) = player_probe(h, amount);
    h.session_mut().room.collision_detection_index = 0;
    let mut list_index = 0usize;
    loop {
        let index = if let Some(index) = h.session().actors.schedule.interactive.get(list_index) {
            *index
        } else {
            h.collection_word(CollectionWindow::Interactive, list_index * 2, LittleEndian)
        };
        if index == 0xffff {
            break;
        }
        h.session_mut().room.collision_detection_index = index;
        let contact = {
            let actor = h.resolve_actor(ActorKey::from_legacy(index), LittleEndian);
            if (actor.frozen != 0 || actor.properties & 0x8000 != 0)
                && axis_overlaps(actor.x.whole, position.x, actor.radius.x, radius.x)
                && axis_overlaps(actor.y.whole, position.y, actor.radius.y, radius.y)
            {
                let direction = h.session().player.contact.collision_direction & 3;
                let (player_position, player_radius, actor_position, actor_radius) =
                    if direction < 2 {
                        (
                            h.session().player.x,
                            h.inputs().player_x_radius,
                            actor.x.whole,
                            actor.radius.x,
                        )
                    } else {
                        (
                            h.session().player.y,
                            h.session().player.contact.radius,
                            actor.y.whole,
                            actor.radius.y,
                        )
                    };
                Some(if direction == 0 || direction == 2 {
                    let actor_edge = actor_position.wrapping_add(actor_radius);
                    let player_edge = i32::from(player_position) - i32::from(player_radius);
                    (
                        (player_edge - i32::from(actor_edge)) as i16,
                        player_edge == i32::from(actor_edge),
                    )
                } else {
                    let distance = (i32::from(actor_position)
                        - i32::from(actor_radius)
                        - i32::from(player_radius)
                        - i32::from(player_position)) as i16;
                    (distance, distance == 0)
                })
            } else {
                None
            }
        };
        if let Some((distance, exact)) = contact {
            if exact || distance >= 0 {
                if exact {
                    h.session_mut().player.y_fraction = 0;
                }
                let index = h.session().room.collision_detection_index;
                let direction = h.session().player.contact.collision_direction;
                // The actual Game setter masks direction before its slice write.
                h.session_mut().actors.schedule.colliding[usize::from(direction & 3)] = index;
                return SolidContact::Blocked {
                    amount: if exact { 0 } else { i32::from(distance) << 16 },
                };
            }
        }
        list_index += 1;
    }
    SolidContact::Clear { amount }
}
pub(in crate::game) fn touches_player_from_above<H: ActorAccess>(h: &H, key: ActorKey) -> bool {
    let actor = h.resolve_actor(key, LittleEndian);
    if !axis_overlaps(
        h.session().player.x,
        actor.x.whole,
        h.inputs().player_x_radius,
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
pub(in crate::game) fn overlaps_player_with_padding<H: ActorAccess>(h: &H, key: ActorKey) -> bool {
    let actor = h.resolve_actor(key, LittleEndian);
    let distance = (h.session().player.x.wrapping_sub(actor.x.whole) as i16).wrapping_abs() as u16;
    let radius = h.inputs().player_x_radius;
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
pub(in crate::game) fn current_outside_screen<H: ActorAccess>(h: &H) -> bool {
    let actor = h.resolve_actor(current(h), LittleEndian);
    outside(actor.x.whole, h.inputs().camera_x, 0, 256)
        || outside(actor.y.whole, h.inputs().camera_y, 0, 256)
}
pub(in crate::game) fn current_outside_padded_screen<H: ActorAccess>(h: &H, padding: u16) -> bool {
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
pub(in crate::game) fn current_normal_sprite_outside_screen<H: ActorAccess>(h: &H) -> bool {
    let actor = h.resolve_actor(current(h), LittleEndian);
    outside(
        actor.x.whole,
        h.inputs().camera_x,
        actor.radius.x,
        actor.radius.x.wrapping_add(256),
    ) || outside(actor.y.whole, h.inputs().camera_y, 8, 248)
}
pub(in crate::game) fn leaving_screen<H: ActorAccess>(h: &H, key: ActorKey) -> bool {
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
pub(in crate::game) fn player_delta<H: ActorAccess>(h: &H, key: ActorKey, axis: Axis) -> u16 {
    let actor = h.resolve_actor(key, LittleEndian);
    match axis {
        Axis::X => h.session().player.x.wrapping_sub(actor.x.whole),
        Axis::Y => h.session().player.y.wrapping_sub(actor.y.whole),
    }
}
pub(in crate::game) fn player_within<H: ActorAccess>(
    h: &H,
    key: ActorKey,
    axis: Axis,
    distance: u16,
) -> bool {
    let delta = player_delta(h, key, axis);
    (((delta as i16).wrapping_abs() as u16).wrapping_sub(distance) as i16) < 0
}
pub(in crate::game) fn direction_to_player<H: ActorAccess>(h: &H) -> PlayerDirection {
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
pub(in crate::game) fn angle_from_delta(x: u16, y: u16) -> u16 {
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
pub(in crate::game) fn angle_to_player<H: ActorAccess>(h: &H, key: ActorKey) -> u16 {
    let x = player_delta(h, key, Axis::X);
    let y = player_delta(h, key, Axis::Y);
    angle_from_delta(x, y)
}
pub(in crate::game) fn angle_between<H: ActorAccess>(h: &H, from: ActorKey, to: ActorKey) -> u16 {
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
pub(in crate::game) fn sine_product<H: ActorContent>(h: &mut H, amplitude: u16, phase: u16) -> u32 {
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
pub(in crate::game) fn distance_and_direction<H: ActorHost>(
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
