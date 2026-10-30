//! Player displacement, speed publication and shinespark orchestration.
#![forbid(unsafe_code)]
use super::{
    effects::Host,
    geometry,
    operations::{
        Axis, Direction, DisplacementRequest, HorizontalDirection, HorizontalRequest, MotionResult,
        Obstruction, VerticalDirection,
    },
};
use crate::geometry::FixedMotion;
use crate::launch_motion::MovementDirection;

fn fixed(value: &FixedMotion) -> u32 {
    u32::from(value.whole) << 16 | u32::from(value.fraction)
}
fn put_fixed(value: &mut FixedMotion, amount: u32) {
    value.fraction = amount as u16;
    value.whole = (amount >> 16) as u16;
}
/// Shared with terrain movement. Reaction-specific fraction changes precede
/// this addition; publication stores the fraction before the whole coordinate.
pub fn commit_position<H: Host>(h: &mut H, axis: Axis, amount: i32) {
    let player = &mut h.session_mut().player;
    let (whole, fraction) = match axis {
        Axis::X => (&mut player.x, &mut player.x_fraction),
        Axis::Y => (&mut player.y, &mut player.y_fraction),
    };
    let next = ((u32::from(*whole) << 16) | u32::from(*fraction)).wrapping_add(amount as u32);
    *fraction = next as u16;
    *whole = (next >> 16) as u16;
}
fn record<H: Host>(h: &mut H, direction: Direction, amount: i32) {
    let direction = match direction {
        Direction::Left => MovementDirection::Left,
        Direction::Right => MovementDirection::Right,
        Direction::Up => MovementDirection::Up,
        Direction::Down => MovementDirection::Down,
    };
    h.record_launch(direction, amount);
    h.session_mut()
        .projectiles
        .launch
        .record_movement(direction, amount);
}
fn commit_and_record<H: Host>(h: &mut H, direction: Direction, signed_amount: i32) {
    commit_position(
        h,
        match direction {
            Direction::Left | Direction::Right => Axis::X,
            _ => Axis::Y,
        },
        signed_amount,
    );
    record(h, direction, signed_amount);
}
fn clamp_speed(amount: i32) -> i32 {
    let whole = ((amount >> 16) as i16).clamp(-15, 15);
    (i32::from(whole) << 16) | (amount & 0xffff)
}
fn clamp_high(amount: i32, limit: i32) -> i32 {
    // This is the source bit15 comparison, not a signed mathematical minimum.
    if ((amount >> 16) - limit) & 0x8000 == 0 {
        limit << 16 | amount & 0xffff
    } else {
        amount
    }
}
fn horizontal_displacement<H: Host>(h: &mut H, base: i32, left: bool) -> i32 {
    let player = &mut h.session_mut().player;
    let total =
        (base as u32).wrapping_add(fixed(&player.contact.extra_run)) >> player.speed_divisor.min(4);
    put_fixed(&mut player.contact.total_x, total);
    player.contact.collision_direction = u16::from(!left);
    let external = fixed(&player.contact.external_x);
    clamp_speed(if left {
        external.wrapping_sub(total)
    } else {
        total.wrapping_add(external)
    } as i32)
}
fn clear_horizontal_speed<H: Host>(h: &mut H) {
    if h.session().player.collision.flags != 0 {
        h.session_mut().player.collision.solid_actor_flags =
            if h.session().player.contact.collision_direction != 0 {
                8
            } else {
                4
            };
        h.cancel_speed_boost();
        // Reacquire after synchronous palette/echo work. Preserve the original
        // whole/fraction clear order instead of restoring a captured player.
        let contact = &mut h.session_mut().player.contact;
        contact.extra_run.whole = 0;
        contact.extra_run.fraction = 0;
        contact.base_run.whole = 0;
        contact.base_run.fraction = 0;
        contact.acceleration_mode = 0;
    } else {
        let collision = &mut h.session_mut().player.collision;
        collision.pose_input = 0;
        collision.solid_actor_flags = 0;
    }
}
/// The private direct-contact entry also serves unchanged directed fixtures.
/// `left` selects the source entry; `amount` retains its signed input convention.
pub fn horizontal_contact<H: Host>(h: &mut H, amount: i32, left: bool) -> MotionResult {
    h.probe(if left {
        "samus.move.left"
    } else {
        "samus.move.right"
    });
    let direction = if left {
        Direction::Left
    } else {
        Direction::Right
    };
    let contact = geometry::solid_contact(h, if left { amount.wrapping_neg() } else { amount });
    let blocked = matches!(contact.obstruction, Obstruction::SolidActor(_));
    h.session_mut().player.collision.flags = if blocked { 0xffff } else { 0 };
    let signed = if left {
        contact.displacement.wrapping_neg()
    } else {
        contact.displacement
    };
    let result = if blocked {
        clear_horizontal_speed(h);
        commit_and_record(h, direction, signed);
        h.probe("player.collision.horizontal.actor_blocked");
        MotionResult {
            displacement: signed,
            obstruction: contact.obstruction,
        }
    } else {
        let result = geometry::terrain_move(h, Axis::X, signed);
        record(h, direction, result.displacement);
        h.probe("player.collision.horizontal.terrain_committed");
        if (h.session().player.contact.collision_direction & 1 != 0) == left {
            h.session_mut().player.collision.flags = 0;
        }
        clear_horizontal_speed(h);
        result
    };
    geometry::align_slopes(h);
    result
}
pub fn horizontal<H: Host>(h: &mut H, request: HorizontalRequest) -> MotionResult {
    let left = match request.direction {
        HorizontalDirection::Left => true,
        HorizontalDirection::Right => false,
        HorizontalDirection::FromFacingAndAcceleration => {
            h.probe("samus.move.x");
            let player = &h.session().player;
            player.movement_and_facing as u8
                == if matches!(player.contact.acceleration_mode, 0 | 2) {
                    4
                } else {
                    8
                }
        }
    };
    let amount = horizontal_displacement(h, request.base, left);
    horizontal_contact(h, amount, amount < 0)
}

const FALLING_POSE_INPUT: [u8; 28] = [
    0, 0, 4, 4, 1, 0, 4, 2, 4, 4, 0, 0, 0, 0, 4, 4, 0, 3, 4, 4, 4, 0, 4, 4, 4, 4, 4, 4,
];
const LANDED_POSE_INPUT: [u8; 28] = [
    4, 4, 0, 0, 4, 4, 0, 4, 1, 2, 0, 4, 4, 0, 4, 4, 4, 4, 3, 3, 0, 4, 4, 4, 4, 0, 4, 4,
];
fn vertical_pose_input<H: Host>(h: &mut H, up: bool) {
    let player = &mut h.session_mut().player;
    if up {
        player.collision.pose_input = if player.collision.flags != 0 { 4 } else { 0 };
    } else if player.collision.flags != 0 {
        player.collision.pose_input = 1;
        // Publish the low-word prefix before the checked movement-type lookup.
        let high = LANDED_POSE_INPUT[usize::from(player.movement_and_facing >> 8)];
        player.collision.pose_input = (player.collision.pose_input & 0xff) | u16::from(high) << 8;
    } else if player.collision.pose_input as u8 != 5 {
        player.collision.pose_input = 2;
        let high = FALLING_POSE_INPUT[usize::from(player.movement_and_facing >> 8)];
        player.collision.pose_input = (player.collision.pose_input & 0xff) | u16::from(high) << 8;
    }
}
pub fn displace<H: Host>(h: &mut H, request: DisplacementRequest) -> MotionResult {
    match request {
        DisplacementRequest::PoseClearanceProbe { amount } => {
            let displacement = geometry::pose_probe(h, amount);
            let obstruction = if h.session().player.collision.flags != 0 {
                Obstruction::Terrain
            } else {
                Obstruction::Clear
            };
            MotionResult {
                displacement,
                obstruction,
            }
        }
        DisplacementRequest::TerrainOnly { axis, amount } => {
            geometry::terrain_move(h, axis, amount)
        }
        DisplacementRequest::Unchecked { direction, amount } => {
            let signed = if matches!(direction, Direction::Left | Direction::Up) {
                amount.wrapping_neg()
            } else {
                amount
            };
            commit_and_record(h, direction, signed);
            MotionResult {
                displacement: signed,
                obstruction: Obstruction::Unchecked,
            }
        }
        DisplacementRequest::Vertical { direction, amount } => {
            let up = matches!(direction, VerticalDirection::Up);
            let direction = if up { Direction::Up } else { Direction::Down };
            h.session_mut().player.contact.collision_direction = if up { 2 } else { 3 };
            let contact =
                geometry::solid_contact(h, if up { amount.wrapping_neg() } else { amount });
            let blocked = matches!(contact.obstruction, Obstruction::SolidActor(_));
            h.session_mut().player.collision.flags = if blocked { 0xffff } else { 0 };
            let signed = if up {
                contact.displacement.wrapping_neg()
            } else {
                contact.displacement
            };
            if blocked {
                vertical_pose_input(h, up);
                commit_and_record(h, direction, signed);
                MotionResult {
                    displacement: signed,
                    obstruction: contact.obstruction,
                }
            } else {
                let result = geometry::terrain_move(h, Axis::Y, signed);
                record(h, direction, result.displacement);
                vertical_pose_input(h, up);
                result
            }
        }
    }
}
fn shinespark_x<H: Host>(h: &mut H) -> MotionResult {
    {
        let player = &mut h.session_mut().player;
        player.shine_timer = 15;
        let extra =
            fixed(&player.contact.extra_run).wrapping_add(fixed(&player.contact.y_acceleration));
        put_fixed(&mut player.contact.extra_run, extra);
        if (i32::from(player.contact.extra_run.whole) - 15) & 0x8000 == 0 {
            put_fixed(&mut player.contact.extra_run, 15 << 16);
        }
    }
    let left = h.session().player.movement_and_facing as u8 == 4;
    let amount = horizontal_displacement(h, 0, left);
    let amount = clamp_high(
        if left { amount.wrapping_neg() } else { amount },
        if left { 15 } else { 25 },
    );
    let contact = geometry::solid_contact(h, amount);
    let signed = if left {
        contact.displacement.wrapping_neg()
    } else {
        contact.displacement
    };
    let result = if matches!(contact.obstruction, Obstruction::SolidActor(_)) {
        h.session_mut().player.collision.flags = 0xffff;
        MotionResult {
            displacement: signed,
            obstruction: contact.obstruction,
        }
    } else {
        let result = geometry::terrain_move(h, Axis::X, signed);
        geometry::align_slopes(h);
        result
    };
    let player = &mut h.session_mut().player;
    let difference = (player.x as i16).wrapping_sub(player.previous_position.x.whole as i16);
    if difference < -15 {
        player.previous_position.x.whole = player.x.wrapping_add(15);
    } else if difference >= 16 {
        player.previous_position.x.whole = player.x.wrapping_sub(15);
    }
    result
}
fn shinespark_y<H: Host>(h: &mut H) -> MotionResult {
    {
        let session = h.session_mut();
        session.player.shine_timer = 15;
        let sequence = (u32::from(session.shared_sequence.stage) << 16)
            | u32::from(session.shared_sequence.value);
        let next = sequence.wrapping_add(fixed(&session.player.contact.y_acceleration));
        session.shared_sequence.value = next as u16;
        session.shared_sequence.stage = (next >> 16) as u16;
        let speed = fixed(&session.player.contact.y_speed).wrapping_add(next);
        put_fixed(&mut session.player.contact.y_speed, speed);
    }
    let amount = clamp_high(fixed(&h.session().player.contact.y_speed) as i32, 14);
    let amount = (amount as u32).wrapping_sub(fixed(&h.session().player.contact.external_y)) as i32;
    h.session_mut().player.contact.collision_direction = 2;
    let contact = geometry::solid_contact(h, clamp_high(amount, 15));
    let signed = contact.displacement.wrapping_neg();
    let result = if matches!(contact.obstruction, Obstruction::SolidActor(_)) {
        h.session_mut().player.collision.flags = 0xffff;
        MotionResult {
            displacement: signed,
            obstruction: contact.obstruction,
        }
    } else {
        geometry::terrain_move(h, Axis::Y, signed)
    };
    let player = &mut h.session_mut().player;
    if (i32::from(player.y) - i32::from(player.previous_position.y.whole) + 14) & 0x8000 != 0 {
        player.previous_position.y.whole = player.y.wrapping_add(14);
    }
    result
}
pub fn shinespark<H: Host>(h: &mut H, axis: Axis) -> MotionResult {
    match axis {
        Axis::X => shinespark_x(h),
        Axis::Y => shinespark_y(h),
    }
}
