//! Wall and grapple admission retain their different input and publication rules.
#![forbid(unsafe_code)]
use super::{
    effects::Host,
    geometry,
    operations::{JumpKind, JumpRequest, Obstruction},
};

pub fn jump_admission<H: Host>(h: &mut H, request: JumpRequest) -> bool {
    match request.kind {
        JumpKind::Wall => wall(h, request.amount),
        JumpKind::Grapple => grapple(h, request.amount),
    }
}
fn grapple<H: Host>(h: &mut H, amount: i32) -> bool {
    h.session_mut().actors.controls.shake_request = 0xffff;
    let direction = match h.session().player.movement_and_facing as u8 {
        4 => 0,
        8 => 1,
        _ => return false,
    };
    h.session_mut().player.contact.collision_direction = direction;
    let result = geometry::solid_contact(h, amount);
    if matches!(result.obstruction, Obstruction::Clear) {
        geometry::wall_probe(h, result.displacement);
        if h.session().player.collision.flags == 0 {
            return false;
        }
        let jump = h.inputs().jump_button;
        let pressed = h.inputs().pressed;
        return jump & pressed != 0;
    }
    let jump = h.inputs().jump_button;
    let pressed = h.inputs().pressed;
    if jump & pressed == 0 {
        return false;
    }
    let actor = h.session().room.collision_detection_index;
    h.session_mut().actors.controls.shake_request = actor;
    true
}
fn wall<H: Host>(h: &mut H, amount: i32) -> bool {
    if !matches!(
        h.session()
            .player
            .history
            .last_different
            .movement_and_facing
            >> 8,
        3 | 20
    ) {
        return false;
    }
    let pose = h.session().player.pose;
    let screw = matches!(pose, 0x81 | 0x82);
    let animation = h.session().player.animation_clock.frame;
    let threshold = if screw { 27 } else { 11 };
    let ready = animation.wrapping_sub(threshold) & 0x8000 == 0;
    if ready {
        h.session_mut().actors.controls.shake_request = 0xffff;
    }
    let held = h.inputs().held;
    let left = if held & 0x0200 != 0 {
        true
    } else if held & 0x0100 != 0 {
        false
    } else {
        return false;
    };
    h.session_mut().player.contact.collision_direction = u16::from(left);
    let result = geometry::solid_contact(h, amount);
    let actor_contact = matches!(result.obstruction, Obstruction::SolidActor(_));
    let mut distance = result.displacement;
    if !actor_contact {
        if left {
            distance = geometry::wall_probe(h, distance);
        } else {
            distance = distance.wrapping_neg();
            geometry::wall_probe(h, distance);
        }
        if h.session().player.collision.flags == 0 {
            return false;
        }
    }
    if !ready {
        // Terrain effects may have changed the pose since the readiness decision.
        let pose = h.session().player.pose;
        let frame = if matches!(pose, 0x81 | 0x82) { 26 } else { 10 };
        h.session_mut().player.animation_clock.remaining = 1;
        h.session_mut().player.animation_clock.frame = frame;
        return false;
    }
    if h.inputs().jump_button & h.inputs().pressed == 0 {
        return false;
    }
    if ((distance >> 16) - 8) & 0x8000 == 0 {
        return false;
    }
    h.session_mut().player.collision.pose_input = 5;
    if actor_contact {
        let actor = h.session().room.collision_detection_index;
        h.session_mut().actors.controls.shake_request = actor;
    }
    true
}
