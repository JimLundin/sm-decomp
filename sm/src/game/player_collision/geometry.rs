//! Shared player solid contact, terrain traversal and slope alignment over owned state.
#![forbid(unsafe_code)]
use super::{
    effects::Host,
    motion::commit_position,
    operations::{Axis, MotionResult, Obstruction},
};
use sm_game::contact_rules::collision_span as span;
use sm_game::projectile_geometry::{SLOPE_FACTORS, SLOPE_HEIGHTS, square_slope_occupied};
use {
    crate::game::{actors::axis_overlaps, room_entry::EntryPoll},
    sm_game::{
        actors::access::{AccessMode::LittleEndian, ActorKey},
        session::{Player, Point},
    },
};

// Only source-local geometry survives a synchronous effect. Player and terrain
// are always reacquired from the host after it returns.
struct Scan {
    displacement: i32,
    target: u16,
    edge: u16,
    remaining: u16,
    span: u16,
}
struct ScanResult {
    blocked: bool,
    displacement: i32,
}
fn other_axis(axis: Axis) -> Axis {
    match axis {
        Axis::X => Axis::Y,
        Axis::Y => Axis::X,
    }
}
fn player_position(player: &Player, axis: Axis) -> u16 {
    match axis {
        Axis::X => player.x,
        Axis::Y => player.y,
    }
}
fn player_fraction(player: &Player, axis: Axis) -> u16 {
    match axis {
        Axis::X => player.x_fraction,
        Axis::Y => player.y_fraction,
    }
}
fn player_radius(player: &Player, axis: Axis) -> u16 {
    match axis {
        Axis::X => player.contact.x_radius,
        Axis::Y => player.contact.radius,
    }
}
fn set_player_fraction(player: &mut Player, axis: Axis, value: u16) {
    match axis {
        Axis::X => player.x_fraction = value,
        Axis::Y => player.y_fraction = value,
    }
}
pub(super) fn level(h: &impl Host, block: u16) -> u16 {
    if let Some(cell) = h.session().room.terrain.cells().get(usize::from(block)) {
        cell.level
    } else {
        h.imported_level_word(usize::from(block) * 2)
    }
}
pub(super) fn behavior(h: &impl Host, block: u16) -> u8 {
    if let Some(cell) = h.session().room.terrain.cells().get(usize::from(block)) {
        cell.behavior
    } else {
        h.imported_behavior_byte(usize::from(block))
    }
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
fn player_probe(h: &impl Host, amount: i32) -> (Point, Point) {
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
            x: h.session().player.contact.x_radius,
            y: h.session().player.contact.radius,
        },
    )
}
pub(super) fn solid_contact<H: Host>(h: &mut H, amount: i32) -> MotionResult {
    if h.session().actors.schedule.interactive_write_position == 0 {
        return MotionResult {
            displacement: amount,
            obstruction: Obstruction::Clear,
        };
    }
    let (position, radius) = player_probe(h, amount);
    h.session_mut().room.collision_detection_index = 0;
    let mut list_index = 0usize;
    loop {
        let index = if let Some(index) = h.session().actors.schedule.interactive.get(list_index) {
            *index
        } else {
            h.imported_interactive_word(list_index * 2)
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
                            h.session().player.contact.x_radius,
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
                return MotionResult {
                    displacement: if exact { 0 } else { i32::from(distance) << 16 },
                    obstruction: Obstruction::SolidActor(ActorKey::from_legacy(index)),
                };
            }
        }
        list_index += 1;
    }
    MotionResult {
        displacement: amount,
        obstruction: Obstruction::Clear,
    }
}
fn boundary_delta(edge: u16, radius: u16, position: u16, negative: bool, mask: u16) -> i32 {
    let delta = if negative {
        radius
            .wrapping_add(edge | mask)
            .wrapping_add(1)
            .wrapping_sub(position)
    } else {
        (edge & !mask).wrapping_sub(radius).wrapping_sub(position)
    } as i16;
    i32::from(if negative { delta.min(0) } else { delta.max(0) }) << 16
}
fn solid(h: &mut impl Host, s: &mut Scan, axis: Axis, mask: u16) -> u8 {
    let negative = s.displacement < 0;
    let radius = player_radius(&h.session().player, axis);
    let position = player_position(&h.session().player, axis);
    s.displacement = boundary_delta(s.edge, radius, position, negative, mask);
    set_player_fraction(
        &mut h.session_mut().player,
        axis,
        if negative { 0 } else { 0xffff },
    );
    1
}
fn square(h: &mut impl Host, s: &mut Scan, axis: Axis, slope: u16, block: u16) -> u8 {
    let orientation = u16::from(behavior(h, block) >> 6);
    let (shift, other) = if axis == Axis::X { (3, 2) } else { (2, 1) };
    let index = slope
        .wrapping_mul(4)
        .wrapping_add(orientation ^ ((s.edge & 8) >> shift));
    let position = player_position(&h.session().player, other_axis(axis));
    let radius = player_radius(&h.session().player, other_axis(axis));
    let occupied = square_slope_occupied(index, s.remaining, s.span, position, radius, other);
    if !occupied {
        return 0;
    }
    let negative = s.displacement < 0;
    solid(h, s, axis, 7);
    if axis == Axis::Y && !negative {
        h.session_mut().player.collision.slope_adjusted = 1;
    }
    1
}
fn nonsquare(h: &mut impl Host, s: &mut Scan, axis: Axis, block: u16) -> u8 {
    if axis == Axis::X {
        let bts = h.session().room.slope_behavior;
        if bts & 0x80 != 0 {
            return 0;
        }
        let speed = u32::from(h.session().player.contact.y_speed.fraction)
            | u32::from(h.session().player.contact.y_speed.whole) << 16;
        if speed != 0 {
            return 0;
        }
        let displacement = s.displacement;
        let shifted = (displacement >> 8) as u16;
        let factor = u32::from(SLOPE_FACTORS[usize::from(bts & 0x1f)]);
        let multiplicand = if displacement < 0 {
            shifted.wrapping_neg()
        } else {
            shifted
        };
        h.session_mut().multiply_scratch.left = multiplicand;
        h.session_mut().multiply_scratch.right = factor as u16;
        s.displacement = if displacement >= 0 {
            (u32::from(shifted) * factor) as i32
        } else {
            ((u32::from(shifted.wrapping_neg()) * factor) as i32).wrapping_neg()
        };
        return 0;
    }
    let upward = h.session().player.contact.collision_direction & 1 != 0;
    let x = player_position(&h.session().player, Axis::X);
    let width = h.session().room.width as u8;
    let current = h.session().room.collision_cursor;
    let column = if width == 0 {
        current
    } else {
        current % u16::from(width)
    };
    if x >> 4 != column {
        return 0;
    }
    let bts = behavior(h, block);
    if (bts & 0x80 != 0) == upward {
        return 0;
    }
    let offset = if bts & 0x40 != 0 { x ^ 15 } else { x };
    let height = SLOPE_HEIGHTS[usize::from(bts & 0x1f) * 16 + usize::from(offset & 15)] & 0x1f;
    let radius = player_radius(&h.session().player, Axis::Y);
    let offset = if upward {
        radius.wrapping_add(s.target).wrapping_sub(1) & 15
    } else {
        (s.target.wrapping_sub(radius) & 15) ^ 15
    };
    let adjustment = i16::from(height) - (offset as i16) - 1;
    if adjustment > 0 {
        return 0;
    }
    let displacement = ((s.displacement >> 16) as i16)
        .wrapping_add(adjustment)
        .max(0);
    s.displacement = i32::from(displacement) << 16;
    1
}
fn slope(h: &mut impl Host, s: &mut Scan, axis: Axis) -> u8 {
    let block = h.session().room.collision_cursor;
    let kind = u16::from(behavior(h, block) & 0x1f);
    if kind < 5 {
        let block = h.session().room.collision_cursor;
        return square(h, s, axis, kind, block);
    }
    let bts = if axis == Axis::X {
        u16::from(behavior(h, h.session().room.collision_cursor))
    } else {
        h.vertical_slope_bts(h.session().room.collision_cursor)
    };
    h.session_mut().room.slope_behavior = bts;
    let block = h.session().room.collision_cursor;
    nonsquare(h, s, axis, block)
}
pub(super) fn spike_damage(h: &mut impl sm_game::session_access::SessionAccess, amount: u16) {
    if h.session().player.invincibility != 0 {
        return;
    }
    h.session_mut().player.invincibility = 60;
    h.session_mut().player.knockback_timer = 10;
    let damage = h.session().player.periodic_damage;
    h.session_mut().player.periodic_damage = damage.wrapping_add(amount);
    h.session_mut().player.knockback_x_direction =
        u16::from(((h.session().player.movement_and_facing & 0xff) ^ 12) & 8 != 0);
}
fn spike(h: &mut impl Host, s: &mut Scan, axis: Axis) -> u8 {
    let bts = behavior(h, h.session().room.collision_cursor);
    assert!(bts < 16, "spike-block BTS index out of bounds: {bts}");
    match bts {
        0 => {
            if h.session().room.area != 3 || h.boss_defeated() {
                spike_damage(h, 60);
            }
        }
        1 | 3 => spike_damage(h, 16),
        _ => {}
    }
    solid(h, s, axis, 15)
}
fn door(h: &mut impl Host, s: &mut Scan, axis: Axis) -> u8 {
    if !matches!(h.enter_door(), EntryPoll::Rejected) {
        h.probe("player.collision.door.admitted");
        0
    } else {
        solid(h, s, axis, 15)
    }
}

fn special(h: &mut impl Host, s: &mut Scan, axis: Axis, solid_block: bool) -> u8 {
    let bts = behavior(h, h.session().room.collision_cursor);
    let header = h.special_header(bts, solid_block);
    let result = h.apply_mechanism(header, &mut s.displacement) & 1;
    h.probe("player.collision.mechanism.returned");
    if solid_block {
        if result != 0 {
            solid(h, s, axis, 15)
        } else {
            result
        }
    } else if bts & 0x80 != 0 {
        result
    } else {
        0
    }
}
fn bombable(h: &mut impl Host, s: &mut Scan, axis: Axis, solid_block: bool) -> u8 {
    let bts = behavior(h, h.session().room.collision_cursor);
    if bts & 0x80 != 0 {
        return if solid_block {
            solid(h, s, axis, 15)
        } else {
            0
        };
    }
    let header = h.bomb_header(bts);
    let result = h.apply_mechanism(header, &mut s.displacement) & 1;
    h.probe("player.collision.mechanism.returned");
    if solid_block && result != 0 {
        solid(h, s, axis, 15)
    } else {
        0
    }
}
pub(super) fn redirect(h: &mut impl Host, axis: Axis) -> bool {
    let step = i16::from(behavior(h, h.session().room.collision_cursor) as i8);
    if step == 0 {
        return false;
    }
    let delta = if axis == Axis::X {
        step as u16
    } else {
        (step as u16).wrapping_mul(h.session().room.width)
    };
    let block = h.session().room.collision_cursor;
    h.session_mut().room.collision_cursor = block.wrapping_add(delta);
    true
}
fn react(h: &mut impl Host, s: &mut Scan, axis: Axis, offset: u16) -> u8 {
    h.session_mut().room.collision_cursor = offset >> 1;
    loop {
        let material = level(h, h.session().room.collision_cursor) >> 12;
        match material {
            0 | 2 | 4 | 6 => return 0,
            1 => return slope(h, s, axis),
            3 => return special(h, s, axis, false),
            5 => {
                if !redirect(h, Axis::X) {
                    return 0;
                }
            }
            7 => return bombable(h, s, axis, false),
            8 | 12 | 14 => return solid(h, s, axis, 15),
            9 => return door(h, s, axis),
            10 => return spike(h, s, axis),
            11 => return special(h, s, axis, true),
            13 => {
                if !redirect(h, Axis::Y) {
                    return 0;
                }
            }
            15 => return bombable(h, s, axis, true),
            _ => unreachable!(),
        }
    }
}
fn traverse(h: &mut impl Host, amount: i32, axis: Axis, reverse: bool) -> ScanResult {
    // Preserve initial observation order; callbacks subsequently mutate only live fields.
    let span = span(
        player_position(&h.session().player, other_axis(axis)),
        player_radius(&h.session().player, other_axis(axis)),
    );
    let (target, edge, mut offset) = if axis == Axis::X {
        let y = player_position(&h.session().player, Axis::Y);
        let radius = player_radius(&h.session().player, Axis::Y);
        let row = (y.wrapping_sub(radius) >> 4) as u8;
        let width = h.session().room.width as u8;
        let row_start = u16::from(row) * u16::from(width);
        let x = player_position(&h.session().player, Axis::X);
        let fraction = player_fraction(&h.session().player, Axis::X);
        let target =
            (((u32::from(x) << 16) | u32::from(fraction)).wrapping_add(amount as u32) >> 16) as u16;
        let radius = player_radius(&h.session().player, Axis::X);
        let edge = if amount >= 0 {
            radius.wrapping_add(target).wrapping_sub(1)
        } else {
            target.wrapping_sub(radius)
        };
        (
            target,
            edge,
            row_start.wrapping_add(edge >> 4).wrapping_mul(2),
        )
    } else {
        let y = player_position(&h.session().player, Axis::Y);
        let fraction = player_fraction(&h.session().player, Axis::Y);
        let target =
            (((u32::from(y) << 16) | u32::from(fraction)).wrapping_add(amount as u32) >> 16) as u16;
        let radius = player_radius(&h.session().player, Axis::Y);
        let edge = if amount >= 0 {
            radius.wrapping_add(target).wrapping_sub(1)
        } else {
            target.wrapping_sub(radius)
        };
        let row = (edge >> 4) as u8;
        let width = h.session().room.width as u8;
        let row_start = u16::from(row) * u16::from(width);
        let x = player_position(&h.session().player, Axis::X);
        let radius = player_radius(&h.session().player, Axis::X);
        let column = if reverse {
            radius.wrapping_add(x).wrapping_sub(1) >> 4
        } else {
            x.wrapping_sub(radius) >> 4
        };
        h.session_mut().room.collision_cursor = row_start.wrapping_add(column);
        (
            target,
            edge,
            h.session().room.collision_cursor.wrapping_mul(2),
        )
    };
    let mut scan = Scan {
        displacement: amount,
        target,
        edge,
        remaining: if reverse { 0 } else { span },
        span,
    };
    loop {
        if react(h, &mut scan, axis, offset) & 1 != 0 {
            return ScanResult {
                blocked: true,
                displacement: scan.displacement,
            };
        }
        if axis == Axis::X {
            offset = offset.wrapping_add(h.session().room.width.wrapping_mul(2));
            scan.remaining = scan.remaining.wrapping_sub(1);
            if scan.remaining & 0x8000 != 0 {
                break;
            }
        } else if reverse {
            scan.remaining = scan.remaining.wrapping_add(1);
            if scan.span < scan.remaining {
                break;
            }
            offset = offset.wrapping_sub(2);
        } else {
            scan.remaining = scan.remaining.wrapping_sub(1);
            if scan.remaining & 0x8000 != 0 {
                break;
            }
            offset = offset.wrapping_add(2);
        }
    }
    ScanResult {
        blocked: false,
        displacement: scan.displacement,
    }
}
fn probe_once(h: &mut impl Host, amount: i32, axis: Axis) -> i32 {
    let direction = h.session().player.contact.collision_direction;
    h.session_mut().player.contact.collision_direction = direction | 15;
    h.session_mut().player.contact.sand_visits = 0;
    let reverse = axis == Axis::Y && h.inputs().frame & 1 != 0;
    let result = traverse(h, amount, axis, reverse);
    h.session_mut().player.collision.flags = u16::from(result.blocked);
    result.displacement.wrapping_abs()
}
pub(super) fn wall_probe<H: Host>(h: &mut H, amount: i32) -> i32 {
    probe_once(h, amount, Axis::X)
}
pub(super) fn pose_probe<H: Host>(h: &mut H, amount: i32) -> i32 {
    let integer = (amount >> 16) as i16;
    if integer.wrapping_abs() as u16 & 0xfff8 == 0 {
        return probe_once(h, amount, Axis::Y);
    }
    let probe = i32::from(((integer as u16 & 0xfff0) | 8) as i16) << 16;
    let adjusted = probe_once(h, probe, Axis::Y);
    if h.session().player.collision.flags != 0 {
        return adjusted;
    }
    h.probe("player.collision.pose.second_probe");
    probe_once(h, amount, Axis::Y)
}
fn move_right(h: &mut impl Host, mut amount: i32) -> i32 {
    let blocked = if amount != 0 {
        h.session_mut().player.contact.sand_visits = 0;
        let result = traverse(h, amount, Axis::X, false);
        amount = result.displacement;
        result.blocked
    } else {
        false
    };
    h.session_mut().player.collision.flags = u16::from(blocked);
    commit_position(h, Axis::X, amount);
    amount
}
fn move_down(h: &mut impl Host, mut amount: i32) -> i32 {
    if amount != 0 {
        h.session_mut().player.collision.slope_adjusted = 0;
        h.session_mut().player.contact.sand_visits = 0;
        let reverse = h.inputs().frame & 1 != 0;
        let result = traverse(h, amount, Axis::Y, reverse);
        amount = result.displacement;
        if result.blocked {
            h.session_mut().player.collision.flags = 1;
            commit_position(h, Axis::Y, amount);
            return amount;
        }
    }
    let sand = h.session().player.contact.sand_visits;
    h.session_mut().player.collision.flags = u16::from(sand != 0);
    commit_position(h, Axis::Y, amount);
    amount
}

/// Terrain publishes its own flags and coordinates, but never LaunchMotion.
/// Sand can publish a collision even when traversal itself did not block.
pub(super) fn terrain_move<H: Host>(h: &mut H, axis: Axis, amount: i32) -> MotionResult {
    let displacement = match axis {
        Axis::X => move_right(h, amount),
        Axis::Y => move_down(h, amount),
    };
    MotionResult {
        displacement,
        obstruction: if h.session().player.collision.flags != 0 {
            h.probe("player.collision.terrain.blocked");
            Obstruction::Terrain
        } else {
            h.probe("player.collision.terrain.clear");
            Obstruction::Clear
        },
    }
}

fn select_block(h: &mut impl Host, x: u16, y: u16) {
    let block = if x < 4096 && y < 4096 {
        let width = u16::from(h.session().room.width as u8);
        (x >> 4) + (y >> 4) * width
    } else {
        0xffff
    };
    // Do not stop after publishing FFFF: the following widened level read
    // retains the source's checked failure and partial state.
    h.session_mut().room.collision_cursor = block;
}
pub(super) fn align_slopes<H: Host>(h: &mut H) {
    if h.session().player.collision.horizontal_slope_flags & 2 == 0 {
        return;
    }
    let x = h.session().player.x;
    let y = h.session().player.y;
    let radius = h.session().player.contact.radius;
    select_block(h, x, radius.wrapping_add(y).wrapping_sub(1));
    if level(h, h.session().room.collision_cursor) & 0xf000 == 0x1000
        && behavior(h, h.session().room.collision_cursor) & 0x1f >= 5
    {
        let radius = h.session().player.contact.radius;
        let y = h.session().player.y;
        let offset = radius.wrapping_add(y).wrapping_sub(1) & 15;
        let bts = behavior(h, h.session().room.collision_cursor);
        if bts & 0x80 == 0 {
            let x = h.session().player.x;
            let x = if bts & 0x40 != 0 { x ^ 15 } else { x };
            let index = usize::from(bts & 0x1f) * 16 + usize::from(x & 15);
            let adjustment = i16::from(SLOPE_HEIGHTS[index] & 0x1f) - offset as i16 - 1;
            if adjustment < 0 {
                h.session_mut().player.y = y.wrapping_add(adjustment as u16);
                h.session_mut().player.collision.slope_adjusted = 1;
            }
        }
    }
    // Bottom alignment may already have changed the live player position.
    let x = h.session().player.x;
    let y = h.session().player.y;
    let radius = h.session().player.contact.radius;
    select_block(h, x, y.wrapping_sub(radius));
    if level(h, h.session().room.collision_cursor) & 0xf000 == 0x1000
        && behavior(h, h.session().room.collision_cursor) & 0x1f >= 5
    {
        let y = h.session().player.y;
        let radius = h.session().player.contact.radius;
        let offset = (y.wrapping_sub(radius) & 15) ^ 15;
        let bts = behavior(h, h.session().room.collision_cursor);
        if bts & 0x80 != 0 {
            let x = h.session().player.x;
            let x = if bts & 0x40 != 0 { x ^ 15 } else { x };
            let index = usize::from(bts & 0x1f) * 16 + usize::from(x & 15);
            let adjustment = i16::from(SLOPE_HEIGHTS[index] & 0x1f) - offset as i16 - 1;
            // Player top alignment includes equality; actor alignment does not.
            if adjustment <= 0 {
                h.session_mut().player.y = y.wrapping_sub(adjustment as u16);
                h.session_mut().player.collision.slope_adjusted = 1;
            }
        }
    }
}
