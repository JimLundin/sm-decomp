//! Actor movement, terrain reactions and ordered fixed-point commits.
#![forbid(unsafe_code)]
use super::{
    access::{
        AccessMode::{LittleEndian, Native},
        ActorAccess, ActorKey,
    },
    api::{MoveScan, MovementResult, SlopePolicy, VerticalProbeEdge},
    effects::ActorHost,
};
#[cfg(not(sm_verify_dll))]
use crate::game::room_mechanisms::{
    creation,
    host::{Contact, SpawnSource},
};
use crate::game::{
    projectile_geometry::square_slope_occupied,
    room_mechanisms::host::{ByteOrder, Host as MechanismHost, ImportedAliases},
};
use sm_game::geometry::{Point, Position, Vector32};
fn current(h: &impl ActorAccess) -> ActorKey {
    ActorKey::from_legacy(h.session().actors.controls.selected)
}
fn level<H: ActorAccess + ImportedAliases>(h: &H, byte_offset: usize) -> u16 {
    if byte_offset & 1 == 0 {
        if let Some(cell) = h.session().room.terrain.cells().get(byte_offset / 2) {
            return cell.level;
        }
    }
    ImportedAliases::terrain_word(h, byte_offset, ByteOrder::Little)
}
fn behavior<H: ActorAccess + ImportedAliases>(h: &H, block: usize) -> u8 {
    if let Some(cell) = h.session().room.terrain.cells().get(block) {
        cell.behavior
    } else {
        ImportedAliases::behavior_byte(h, block)
    }
}
fn packed(p: Position) -> u32 {
    (u32::from(p.whole) << 16) | u32::from(p.fraction)
}
fn extent(position: u16, radius: u16) -> (u16, u16) {
    let first = position.wrapping_sub(radius);
    let count = radius
        .wrapping_add(position)
        .wrapping_sub(1)
        .wrapping_sub(first & 0xfff0)
        >> 4;
    (first, count)
}
fn leading(amount: i32, position: Position, radius: u16) -> u16 {
    let adjusted = (packed(position).wrapping_add(amount as u32) >> 16) as u16;
    if amount < 0 {
        adjusted.wrapping_sub(radius)
    } else {
        radius.wrapping_add(adjusted).wrapping_sub(1)
    }
}
pub(in crate::game) fn probe_horizontal<H: ActorAccess + ImportedAliases>(
    h: &mut H,
    key: ActorKey,
    amount: i32,
) -> MovementResult {
    let (top, mut rows, edge) = {
        let actor = h.resolve_actor(key, LittleEndian);
        let (top, rows) = extent(actor.y.whole, actor.radius.y);
        (top, rows, leading(amount, actor.x, actor.radius.x))
    };
    let row_offset = u16::from((top >> 4) as u8) * u16::from(h.session().room.width as u8);
    let mut block = row_offset.wrapping_add(edge >> 4).wrapping_mul(2);
    loop {
        if level(h, usize::from(block)) & 0x8000 != 0 {
            return MovementResult::Blocked;
        }
        block = block.wrapping_add(h.session().room.width.wrapping_mul(2));
        rows = rows.wrapping_sub(1);
        if rows & 0x8000 != 0 {
            return MovementResult::Clear;
        }
    }
}
fn probe_row<H: ActorAccess + ImportedAliases>(
    h: &H,
    edge: u16,
    left: u16,
    mut columns: u16,
) -> MovementResult {
    let row_offset = u16::from((edge >> 4) as u8) * u16::from(h.session().room.width as u8);
    // Keep this cursor wider than the horizontal scan's wrapping word.
    let mut block = 2 * (usize::from(row_offset) + usize::from(left >> 4));
    loop {
        if level(h, block) & 0x8000 != 0 {
            return MovementResult::Blocked;
        }
        columns = columns.wrapping_sub(1);
        if columns & 0x8000 != 0 {
            return MovementResult::Clear;
        }
        block += 2;
    }
}
pub(in crate::game) fn probe_vertical<H: ActorAccess + ImportedAliases>(
    h: &mut H,
    key: ActorKey,
    amount: i32,
) -> MovementResult {
    let actor = h.resolve_actor(key, LittleEndian);
    let (left, columns) = extent(actor.x.whole, actor.radius.x);
    probe_row(h, leading(amount, actor.y, actor.radius.y), left, columns)
}
pub(in crate::game) fn probe_vertical_offset<H: ActorAccess + ImportedAliases>(
    h: &mut H,
    key: ActorKey,
    direction: VerticalProbeEdge,
    amount: i32,
) -> MovementResult {
    let actor = h.resolve_actor(key, LittleEndian);
    let (left, columns) = extent(actor.x.whole, actor.radius.x);
    let edge = match direction {
        VerticalProbeEdge::Positive => actor
            .radius
            .y
            .wrapping_add((packed(actor.y).wrapping_add(amount as u32) >> 16) as u16)
            .wrapping_sub(1),
        VerticalProbeEdge::Negative => ((packed(actor.y).wrapping_sub(amount as u32) >> 16) as u16)
            .wrapping_sub(actor.radius.y),
    };
    probe_row(h, edge, left, columns)
}
pub(in crate::game) fn apply_vector<H: ActorAccess>(h: &mut H, phase: u16, vector: Vector32) {
    let key = current(h);
    let x = if phase.wrapping_add(64) & 0x80 != 0 {
        vector.x.wrapping_neg()
    } else {
        vector.x
    };
    let position = packed(h.resolve_actor(key, LittleEndian).x).wrapping_add(x);
    h.edit_actor(key, LittleEndian, |a| {
        a.x.fraction = position as u16;
    });
    #[cfg(sm_verify_dll)]
    super::movement_verification::vector_store(h, key, false);
    h.edit_actor(key, LittleEndian, |a| {
        a.x.whole = (position >> 16) as u16;
    });
    #[cfg(sm_verify_dll)]
    super::movement_verification::vector_store(h, key, false);
    let y = if phase.wrapping_add(128) & 0x80 != 0 {
        vector.y.wrapping_neg()
    } else {
        vector.y
    };
    let position = packed(h.resolve_actor(key, LittleEndian).y).wrapping_add(y);
    h.edit_actor(key, LittleEndian, |a| {
        a.y.fraction = position as u16;
    });
    #[cfg(sm_verify_dll)]
    super::movement_verification::vector_store(h, key, true);
    h.edit_actor(key, LittleEndian, |a| {
        a.y.whole = (position >> 16) as u16;
    });
    #[cfg(sm_verify_dll)]
    super::movement_verification::vector_store(h, key, true);
}
fn square<H: ActorAccess>(h: &H, scan: &MoveScan, bts: u8, vertical: bool) -> bool {
    let index = usize::from(bts & 0x1f) * 4
        + usize::from((bts >> 6) ^ ((scan.leading_edge & 8) >> if vertical { 2 } else { 3 }) as u8);
    let actor = h.resolve_actor(current(h), LittleEndian);
    let (position, size, other) = if vertical {
        (actor.x.whole, actor.radius.x, 1)
    } else {
        (actor.y.whole, actor.radius.y, 2)
    };
    h.probe(if vertical {
        "square-slope.enemy.vert"
    } else {
        "square-slope.enemy.horiz"
    });
    square_slope_occupied(
        index as u16,
        scan.remaining_extent,
        scan.original_extent,
        position,
        size,
        other,
    )
}
fn horizontal_curve<H: ActorHost>(h: &mut H, scan: &mut MoveScan) -> bool {
    match scan.slopes {
        SlopePolicy::Ignore => return false,
        SlopePolicy::Solid => return true,
        SlopePolicy::Follow => {}
    }
    let slope = h.session().room.slope_behavior & 0x1f;
    let multiplier = i32::from(h.actor_slope_multiplier(slope * 2 + 1));
    let displacement = scan.displacement >> 8;
    let displacement = if scan.displacement < 0 {
        i32::from(displacement as i16)
    } else {
        displacement
    };
    scan.displacement = displacement.wrapping_mul(multiplier);
    false
}
fn vertical_curve<H: ActorHost + ImportedAliases>(h: &mut H, scan: &MoveScan) -> bool {
    let block = h.session().room.collision_cursor;
    let width = u16::from(h.session().room.width as u8);
    let column = if width == 0 { block } else { block % width };
    let key = current(h);
    let (x, height) = {
        let actor = h.resolve_actor(key, LittleEndian);
        (actor.x.whole, actor.radius.y)
    };
    if x >> 4 != column {
        return false;
    }
    let up = scan.displacement < 0;
    let bts = behavior(h, usize::from(h.session().room.collision_cursor));
    if (bts & 0x80 != 0) != up {
        return false;
    }
    let edge = if up {
        (scan.proposed_coordinate.wrapping_sub(height) & 0xf) ^ 0xf
    } else {
        height
            .wrapping_add(scan.proposed_coordinate)
            .wrapping_sub(1)
            & 0xf
    };
    let x_in_block = (x ^ if bts & 0x40 != 0 { 0xf } else { 0 }) & 0xf;
    let surface = h.actor_slope_surface(u16::from(bts & 0x1f) * 16 + x_in_block) & 0x1f;
    let correction = surface as i16 - edge as i16 - 1;
    if correction > 0 {
        return false;
    }
    let position = if up {
        scan.proposed_coordinate.wrapping_sub(correction as u16)
    } else {
        scan.proposed_coordinate.wrapping_add(correction as u16)
    };
    h.edit_actor(key, LittleEndian, |a| {
        a.y.whole = position;
    });
    h.edit_actor(key, LittleEndian, |a| {
        a.y.fraction = if up { 0 } else { u16::MAX };
    });
    true
}
/// One common type switch, while keeping the two axis-specific slope rules.
fn react<H: ActorHost + MechanismHost>(
    h: &mut H,
    scan: &mut MoveScan,
    block: u16,
    vertical: bool,
) -> bool {
    h.session_mut().room.collision_cursor = block >> 1;
    loop {
        let kind = level(h, usize::from(h.session().room.collision_cursor) * 2) >> 12;
        match kind {
            0 | 2 | 3 | 4 | 6 | 7 => return false,
            1 => {
                let bts = behavior(h, usize::from(h.session().room.collision_cursor));
                if bts & 0x1f < 5 {
                    return square(h, scan, bts, vertical);
                }
                if vertical {
                    return vertical_curve(h, scan);
                }
                h.session_mut().room.slope_behavior = u16::from(bts);
                return horizontal_curve(h, scan);
            }
            5 | 13 => {
                let delta =
                    behavior(h, usize::from(h.session().room.collision_cursor)) as i8 as i16;
                if delta == 0 {
                    return false;
                }
                let delta = if kind == 13 {
                    (delta as u16).wrapping_mul(h.session().room.width)
                } else {
                    delta as u16
                };
                let cursor = h.session().room.collision_cursor.wrapping_add(delta);
                h.session_mut().room.collision_cursor = cursor;
                // Original reaction resultFF retries using the live cursor.
            }
            10 => {
                let variant = behavior(h, usize::from(h.session().room.collision_cursor)) & 0x7f;
                let header = h.actor_spike_mechanism(u16::from(variant));
                if header == 0 {
                    return true;
                }
                #[cfg(sm_verify_dll)]
                super::movement_verification::spike(h, header);
                #[cfg(not(sm_verify_dll))]
                creation::spawn(h, SpawnSource::Collision { header }, &mut Contact::Absent);
                return false;
            }
            _ => return true,
        }
    }
}
fn coordinate(actor: &mut super::state::Actor, vertical: bool) -> &mut Position {
    if vertical { &mut actor.y } else { &mut actor.x }
}
fn move_axis<H: ActorHost + MechanismHost>(
    h: &mut H,
    key: ActorKey,
    amount: i32,
    slopes: SlopePolicy,
    vertical: bool,
) -> MovementResult {
    if amount == 0 {
        return MovementResult::Clear;
    }
    let (first, extent, position, edge) = {
        let actor = h.resolve_actor(key, LittleEndian);
        let (along, radius, across, across_radius) = if vertical {
            (actor.y, actor.radius.y, actor.x.whole, actor.radius.x)
        } else {
            (actor.x, actor.radius.x, actor.y.whole, actor.radius.y)
        };
        let (first, extent) = extent(across, across_radius);
        (
            first,
            extent,
            packed(along).wrapping_add(amount as u32),
            leading(amount, along, radius),
        )
    };
    let (row, column) = if vertical {
        (edge, first)
    } else {
        (first, edge)
    };
    let row_offset = u16::from((row >> 4) as u8) * u16::from(h.session().room.width as u8);
    let mut scan = MoveScan {
        displacement: amount,
        proposed_coordinate: (position >> 16) as u16,
        leading_edge: edge,
        remaining_extent: extent,
        original_extent: extent,
        slopes,
    };
    let mut block = row_offset.wrapping_add(column >> 4).wrapping_mul(2);
    while !react(h, &mut scan, block, vertical) {
        if !vertical {
            block = block.wrapping_add(h.session().room.width.wrapping_mul(2));
        }
        scan.remaining_extent = scan.remaining_extent.wrapping_sub(1);
        if scan.remaining_extent & 0x8000 != 0 {
            // Only horizontal movement rereads position and scaled displacement after callbacks.
            let position = if vertical {
                position
            } else {
                packed(h.resolve_actor(key, LittleEndian).x).wrapping_add(scan.displacement as u32)
            };
            h.edit_actor(key, LittleEndian, |a| {
                coordinate(a, vertical).fraction = position as u16;
            });
            h.edit_actor(key, LittleEndian, |a| {
                coordinate(a, vertical).whole = (position >> 16) as u16;
            });
            h.probe(if vertical {
                "actors.movement.vertical.clear"
            } else {
                "actors.movement.horizontal.clear"
            });
            return MovementResult::Clear;
        }
        if vertical {
            block = block.wrapping_add(2);
        }
    }
    let negative = amount < 0;
    h.edit_actor(key, LittleEndian, |a| {
        let radius = if vertical { a.radius.y } else { a.radius.x };
        let boundary = if negative {
            radius.wrapping_add(1).wrapping_add(scan.leading_edge | 0xf)
        } else {
            (scan.leading_edge & 0xfff0).wrapping_sub(radius)
        };
        let position = coordinate(a, vertical);
        if if negative {
            boundary <= position.whole
        } else {
            boundary >= position.whole
        } {
            position.whole = boundary;
        }
    });
    h.edit_actor(key, LittleEndian, |a| {
        coordinate(a, vertical).fraction = if negative { 0 } else { u16::MAX };
    });
    h.probe(if vertical {
        "actors.movement.vertical.blocked"
    } else {
        "actors.movement.horizontal.blocked"
    });
    MovementResult::Blocked
}
pub(in crate::game) fn move_horizontal<H: ActorHost + MechanismHost>(
    h: &mut H,
    key: ActorKey,
    amount: i32,
    slopes: SlopePolicy,
) -> MovementResult {
    move_axis(h, key, amount, slopes, false)
}
pub(in crate::game) fn move_vertical<H: ActorHost + MechanismHost>(
    h: &mut H,
    key: ActorKey,
    amount: i32,
) -> MovementResult {
    move_axis(h, key, amount, SlopePolicy::Ignore, true)
}
pub(in crate::game) fn select_block_at<H: ActorAccess>(h: &mut H, point: Point) {
    let product = u16::from((point.y >> 4) as u8) * u16::from(h.session().room.width as u8);
    h.session_mut().room.collision_cursor = product.wrapping_add(point.x >> 4);
}
pub(in crate::game) fn snap_to_slopes<H: ActorHost + ImportedAliases>(
    h: &mut H,
    key: ActorKey,
) -> bool {
    let mut hit = false;
    let (x, y) = {
        let a = h.resolve_actor(key, Native);
        (
            a.x.whole,
            a.radius.y.wrapping_add(a.y.whole).wrapping_sub(1),
        )
    };
    select_block_at(h, Point { x, y });
    if level(h, usize::from(h.session().room.collision_cursor) * 2) & 0xf000 == 0x1000
        && behavior(h, usize::from(h.session().room.collision_cursor)) & 0x1f >= 5
    {
        hit = true;
        let edge = {
            let a = h.resolve_actor(key, Native);
            a.radius.y.wrapping_add(a.y.whole).wrapping_sub(1) & 0xf
        };
        let table =
            16 * u16::from(behavior(h, usize::from(h.session().room.collision_cursor)) & 0x1f);
        if behavior(h, usize::from(h.session().room.collision_cursor)) & 0x80 == 0 {
            let flip = behavior(h, usize::from(h.session().room.collision_cursor)) & 0x40 != 0;
            let x = h.resolve_actor(key, Native).x.whole ^ if flip { 0xf } else { 0 };
            let surface = h.actor_slope_surface_native(table + (x & 0xf));
            let correction = i16::from(surface & 0x1f) - edge as i16 - 1;
            if correction < 0 {
                h.edit_actor(key, Native, |a| {
                    a.y.whole = a.y.whole.wrapping_add(correction as u16);
                });
            }
        }
    }
    let (x, y) = {
        let a = h.resolve_actor(key, Native);
        (a.x.whole, a.y.whole.wrapping_sub(a.radius.y))
    };
    select_block_at(h, Point { x, y });
    if level(h, usize::from(h.session().room.collision_cursor) * 2) & 0xf000 == 0x1000
        && behavior(h, usize::from(h.session().room.collision_cursor)) & 0x1f >= 5
    {
        hit = true;
        let edge = {
            let a = h.resolve_actor(key, Native);
            (a.y.whole.wrapping_sub(a.radius.y) & 0xf) ^ 0xf
        };
        let table =
            16 * u16::from(behavior(h, usize::from(h.session().room.collision_cursor)) & 0x1f);
        if behavior(h, usize::from(h.session().room.collision_cursor)) & 0x80 != 0 {
            let flip = behavior(h, usize::from(h.session().room.collision_cursor)) & 0x40 != 0;
            let x = h.resolve_actor(key, Native).x.whole ^ if flip { 0xf } else { 0 };
            let surface = h.actor_slope_surface_native(table + (x & 0xf));
            let correction = i16::from(surface & 0x1f) - edge as i16 - 1;
            if correction < 0 {
                h.edit_actor(key, Native, |a| {
                    a.y.whole = a.y.whole.wrapping_sub(correction as u16);
                });
            }
        }
    }
    hit
}
