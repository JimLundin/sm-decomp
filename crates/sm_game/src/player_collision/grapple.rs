//! Terrain clearance after releasing grapple, over the shared player and room.
use super::{
    effects::Host,
    geometry::{behavior as room_bts, level as room_level},
};
use crate::{
    contact_rules::collision_span,
    projectile_geometry::{SLOPE_HEIGHTS, square_slope_occupied},
};

// The horizontal profile differs only for these non-square slope rows.
fn horizontal_height(index: usize) -> u8 {
    match index {
        112..=119 | 354..=360 => 16,
        120..=127 | 368..=375 => 0,
        361..=367 => ((368 - index) * 2) as u8,
        376..=383 => ((index - 376) * 2) as u8,
        _ => SLOPE_HEIGHTS[index],
    }
}
struct Sample {
    remaining: u16,
    span: u16,
    edge: u16,
}
fn contact(h: &mut impl Host, s: &Sample, horizontal: bool, offset: u16) -> u16 {
    h.session_mut().room.collision_cursor = offset >> 1;
    let cell = h.session().room.collision_cursor;
    match room_level(h, cell) >> 12 {
        1 => {}
        8..=15 => return s.edge & 15,
        _ => return 0xffff,
    }
    let bts = room_bts(h, cell);
    let negative = h.session().player.contact.collision_direction & 1 != 0;
    let player = &h.session().player;
    if bts & 31 < 5 {
        let (shift, position, radius, other) = if horizontal {
            (3, player.y, player.contact.radius, 2)
        } else {
            (2, player.x, player.contact.x_radius, 1)
        };
        let index = u16::from(bts & 31) * 4 + (u16::from(bts >> 6) ^ ((s.edge & 8) >> shift));
        if !square_slope_occupied(index, s.remaining, s.span, position, radius, other) {
            return 0xffff;
        }
        return (s.edge & 7) ^ if negative { 0 } else { 7 };
    }
    let width = u16::from(h.session().room.width as u8);
    let column = if width == 0 { cell } else { cell % width };
    if player.x >> 4 != column {
        return 0xffff;
    }
    let (facing, flip, position) = if horizontal {
        (0x40, 0x80, player.y)
    } else {
        (0x80, 0x40, player.x)
    };
    if (bts & facing != 0) == negative {
        return s.edge & 15;
    }
    let position = if bts & flip != 0 {
        position ^ 15
    } else {
        position
    };
    let index = usize::from(bts & 31) * 16 + usize::from(position & 15);
    let boundary = if horizontal {
        horizontal_height(index)
    } else {
        SLOPE_HEIGHTS[index]
    } & 31;
    let distance = i16::from(boundary) - (s.edge & 15) as i16 - 1;
    match (negative, distance) {
        (true, ..=0) => !(distance as u16),
        (false, 0..) => distance as u16,
        _ => 0xffff,
    }
}
fn traverse(h: &mut impl Host, horizontal: bool, positive: bool) {
    let index = if horizontal { 0 } else { 2 } + usize::from(!positive);
    h.session_mut().player.contact.collision_direction = (index ^ 1) as u16;
    h.session_mut().player.collision.grapple_ejection[index] = 0;
    let player = &h.session().player;
    let span = if horizontal {
        collision_span(player.y, player.contact.radius)
    } else {
        collision_span(player.x, player.contact.x_radius)
    };
    let (position, radius) = if horizontal {
        (player.x, player.contact.x_radius)
    } else {
        (player.y, player.contact.radius)
    };
    let edge = if positive {
        position.wrapping_add(radius).wrapping_sub(1)
    } else {
        position.wrapping_sub(radius)
    };
    let row = if horizontal {
        player.y.wrapping_sub(player.contact.radius) >> 4
    } else {
        edge >> 4
    };
    let column = if horizontal {
        edge >> 4
    } else {
        player.x.wrapping_sub(player.contact.x_radius) >> 4
    };
    let row_start = u16::from(row as u8) * u16::from(h.session().room.width as u8);
    h.session_mut().room.collision_cursor = row_start.wrapping_add(column);
    let mut offset = h.session().room.collision_cursor.wrapping_mul(2);
    let mut sample = Sample {
        remaining: span,
        span,
        edge,
    };
    loop {
        let result = contact(h, &sample, horizontal, offset) as i16;
        if result >= 0 {
            let distance = (result as u16).wrapping_add(1);
            if distance >= h.session().player.collision.grapple_ejection[index] {
                h.session_mut().player.collision.grapple_ejection[index] = distance;
            }
        }
        let step = if horizontal {
            h.session().room.width.wrapping_mul(2)
        } else {
            2
        };
        offset = offset.wrapping_add(step);
        sample.remaining = sample.remaining.wrapping_sub(1);
        if sample.remaining & 0x8000 != 0 {
            break;
        }
    }
}
fn vertical(h: &mut impl Host) {
    traverse(h, false, true);
    traverse(h, false, false);
}
pub fn settle(h: &mut impl Host) {
    traverse(h, true, true);
    traverse(h, true, false);
    vertical(h);
    let ejection = &h.session().player.collision.grapple_ejection;
    if (ejection[3] == 0 || ejection[2] == 0) && ejection[2] != 0 {
        h.session_mut().player.y = h.session().player.y.wrapping_sub(ejection[2]);
        if h.session().player.contact.radius.wrapping_sub(16) & 0x8000 == 0 {
            vertical(h);
            h.session_mut().player.y = h
                .session()
                .player
                .y
                .wrapping_sub(h.session().player.collision.grapple_ejection[2]);
        }
    }
}

/// Follow extension cells while preserving the live cursor and width.
pub fn redirect_block(
    h: &mut impl crate::player_projectiles::host::Host,
    axis: crate::projectile_room::Axis,
) -> u8 {
    use crate::player_projectiles::access::room_bts;
    let bts = room_bts(h, h.session().room.collision_cursor);
    if let Some(cell) = crate::projectile_block_reactions::redirect(
        axis,
        i16::from(bts as i8),
        || h.session().room.width,
        || h.session().room.collision_cursor,
    ) {
        h.session_mut().room.collision_cursor = cell;
        0xff
    } else {
        0
    }
}
/// The grapple beam uses collision setup's return flags, unlike ordinary shots.
pub fn react_beam(
    h: &mut (
             impl crate::player_projectiles::host::Host
             + crate::room_mechanisms::host::Host
             + crate::projectile_block_reactions::Definitions
         ),
) -> u8 {
    use crate::{
        player_projectiles::access::{room_bts, room_level},
        projectile_block_reactions as reactions,
        projectile_room::Axis,
        room_mechanisms::{
            creation::spawn,
            host::{Contact, SpawnSource},
        },
    };
    let endpoint = h.session().player.grapple_endpoint;
    let row = u16::from((endpoint.y >> 4) as u8) * u16::from(h.session().room.width as u8);
    h.session_mut().room.collision_cursor = row.wrapping_add(endpoint.x >> 4);
    loop {
        let kind = room_level(h, h.session().room.collision_cursor) >> 12;
        let result = match kind {
            0 | 2 | 3 | 6 => 0,
            1 | 8 | 9 | 11 => 1,
            kind @ (4 | 7 | 12 | 15) => {
                let solid = matches!(kind, 12 | 15);
                if let Some(mechanism) = reactions::mechanism(matches!(kind, 7 | 15), solid, h) {
                    reactions::Definitions::spawn(h, mechanism.header);
                }
                u8::from(solid)
            }
            5 => redirect_block(h, Axis::X),
            13 => redirect_block(h, Axis::Y),
            10 | 14 => {
                if kind == 14 {
                    h.session_mut().player.grapple_flags &= !0x8000;
                }
                let bts = room_bts(h, h.session().room.collision_cursor);
                if bts & 0x80 != 0 {
                    0
                } else {
                    let header = if kind == 14 {
                        [0xd0d8, 0xd0dc, 0xd0e0, 0xd0d8][usize::from(bts)]
                    } else {
                        const SPIKES: [u16; 16] = {
                            let mut headers = [0xd0e4; 16];
                            headers[3] = 0xd0e8;
                            headers
                        };
                        SPIKES[usize::from(bts)]
                    };
                    spawn(h, SpawnSource::Collision { header }, &mut Contact::Absent)
                }
            }
            _ => unreachable!(),
        };
        if result & 0x80 == 0 {
            return result;
        }
    }
}
/// Four signed fixed-point substeps; each setup callback can change the next one.
pub fn extend_beam(
    h: &mut (
             impl crate::player_projectiles::host::Host
             + crate::room_mechanisms::host::Host
             + crate::projectile_block_reactions::Definitions
         ),
) -> u8 {
    let velocity = h.session().player.grapple_velocity;
    let velocity = [
        i32::from(velocity.x as i16) << 6,
        i32::from(velocity.y as i16) << 6,
    ];
    let mut result = 0;
    for _ in 0..4 {
        for axis in 0..2 {
            let offset = &mut h.session_mut().player.grapple_offset[axis];
            let sum = ((u32::from(offset.whole) << 16) | u32::from(offset.fraction))
                .wrapping_add(velocity[axis] as u32);
            offset.fraction = sum as u16;
            offset.whole = (sum >> 16) as u16;
        }
        for axis in 0..2 {
            let player = &mut h.session_mut().player;
            let offset = &player.grapple_offset[axis];
            let (whole, fraction, origin) = if axis == 0 {
                (player.x, player.x_fraction, player.grapple_origin_offset.x)
            } else {
                (player.y, player.y_fraction, player.grapple_origin_offset.y)
            };
            let sum = ((u32::from(offset.whole) << 16) | u32::from(offset.fraction))
                .wrapping_add((u32::from(whole) << 16) | u32::from(fraction))
                .wrapping_add(u32::from(origin) << 16);
            if axis == 0 {
                player.grapple_endpoint_fraction.x = sum as u16;
                player.grapple_endpoint.x = (sum >> 16) as u16;
            } else {
                player.grapple_endpoint_fraction.y = sum as u16;
                player.grapple_endpoint.y = (sum >> 16) as u16;
            }
        }
        result = react_beam(h);
        if result & 0x41 == 0x41 {
            let endpoint = &mut h.session_mut().player.grapple_endpoint;
            endpoint.x = (endpoint.x & 0xfff0) | 8;
            endpoint.y = (endpoint.y & 0xfff0) | 8;
            return result;
        }
    }
    result
}
