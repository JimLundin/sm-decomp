//! Pose clearance and walking obstruction over the one live player owner.
#![forbid(unsafe_code)]
use super::{
    effects::Host,
    geometry,
    operations::{Axis, Obstruction},
};
use crate::game::visual_host::Record;

// Each call acquires before reading its live pose index, as the source pointer did.
fn radius<H: Host>(h: &mut H, previous: bool) -> u16 {
    let table = h.pose_table();
    let pose = if previous {
        h.session().player.history.previous.pose
    } else {
        h.session().player.pose
    };
    u16::from(table.byte(usize::from(pose) * 8 + 6))
}
fn walking_transition<H: Host>(h: &mut H, current: bool) -> u16 {
    // rom_table_word! captured its loaded receiver before evaluating native index.
    let transitions = h.walked_pose_table();
    let poses = h.pose_table();
    let pose = if current {
        h.session().player.pose
    } else {
        h.session().player.transitions.ordinary.to_raw()
    };
    let direction = poses.byte(usize::from(pose) * 8 + 3);
    transitions.native_word(usize::from(direction) * 2)
}

pub(crate) fn walked_obstacle<H: Host>(h: &mut H) {
    if h.session().player.collision.solid_actor_flags != 0
        && h.session().player.movement_and_facing >> 8 == 1
    {
        let next = walking_transition(h, true);
        h.session_mut().player.transitions.ordinary =
            crate::game::player_transitions::state::PoseRequestWord::from_raw(next);
        h.session_mut().player.collision.solid_actor_flags = 0;
        return;
    }
    if h.session().player.transitions.ordinary.to_raw() == 0xffff {
        h.session_mut().player.collision.solid_actor_flags = 0;
        return;
    }
    let poses = h.pose_table();
    let pending = h.session().player.transitions.ordinary.to_raw();
    if poses.byte(usize::from(pending) * 8 + 1) != 1 {
        h.session_mut().player.collision.solid_actor_flags = 0;
        return;
    }
    let left = h.session().player.movement_and_facing as u8 == 4;
    h.session_mut().player.contact.collision_direction = u16::from(!left);
    let result = geometry::solid_contact(h, 1 << 16);
    if matches!(result.obstruction, Obstruction::Clear) {
        let amount = if left { -(1 << 16) } else { 1 << 16 };
        h.session_mut().player.contact.collision_direction = u16::from(!left);
        geometry::terrain_move(h, Axis::X, amount);
        if h.session().player.collision.flags == 0 {
            h.session_mut().player.collision.solid_actor_flags = 0;
            return;
        }
    }
    let next = walking_transition(h, false);
    h.session_mut().player.transitions.ordinary =
        crate::game::player_transitions::state::PoseRequestWord::from_raw(next);
    h.session_mut().player.collision.solid_actor_flags = 0;
}

#[derive(Clone, Copy)]
enum ContactSides {
    Neither,
    Above,
    Below,
    Both,
}
const SIDES: [ContactSides; 4] = [
    ContactSides::Neither,
    ContactSides::Above,
    ContactSides::Below,
    ContactSides::Both,
];

pub(crate) fn fit_pose<H: Host>(h: &mut H) {
    if h.session().player.pose == 0 || h.session().player.pose == 0x9b {
        return;
    }
    h.session_mut().player.clearance.actor_flags = 0;
    h.session_mut().player.clearance.terrain_flags = 0;
    let previous_radius = radius(h, true);
    if (i32::from(previous_radius) - i32::from(radius(h, false))) & 0x8000 == 0 {
        return;
    }
    let old_radius = radius(h, true);
    h.session_mut().player.contact.radius = old_radius;
    let new_radius = radius(h, false);
    h.session_mut().player.clearance.radius_difference = new_radius.wrapping_sub(previous_radius);
    h.session_mut().player.contact.collision_direction = 2;
    let amount = i32::from(h.session().player.clearance.radius_difference as i16) << 16;
    let result = geometry::solid_contact(h, amount);
    h.session_mut().player.collision.flags =
        u16::from(matches!(result.obstruction, Obstruction::SolidActor(_))).wrapping_neg();
    if h.session().player.collision.flags != 0 {
        h.session_mut().player.clearance.actor_flags = 1;
    }
    h.session_mut().player.clearance.actor_up = (result.displacement >> 16) as u16;
    h.session_mut().player.contact.collision_direction = 3;
    let amount = i32::from(h.session().player.clearance.radius_difference as i16) << 16;
    let result = geometry::solid_contact(h, amount);
    h.session_mut().player.collision.flags =
        u16::from(matches!(result.obstruction, Obstruction::SolidActor(_))).wrapping_neg();
    if h.session().player.collision.flags != 0 {
        h.session_mut().player.clearance.actor_flags |= 2;
    }
    h.session_mut().player.clearance.actor_down = (result.displacement >> 16) as u16;
    // Retain bounds failure for the original four-entry table, not a sanitized flag.
    let rejected = match SIDES[usize::from(h.session().player.clearance.actor_flags)] {
        ContactSides::Above => opposite_actor_probe(h, true),
        ContactSides::Below => opposite_actor_probe(h, false),
        ContactSides::Neither | ContactSides::Both => false,
    };
    if rejected {
        h.session_mut().player.pose = h.session().player.history.previous.pose;
        return;
    }
    let amount = i32::from(
        h.session()
            .player
            .clearance
            .radius_difference
            .wrapping_neg() as i16,
    ) << 16;
    let displacement = geometry::pose_probe(h, amount);
    if h.session().player.collision.flags != 0 {
        h.session_mut().player.clearance.terrain_flags = 1;
    }
    h.session_mut().player.clearance.terrain_up = (displacement >> 16) as u16;
    let amount = i32::from(h.session().player.clearance.radius_difference as i16) << 16;
    let displacement = geometry::pose_probe(h, amount);
    if h.session().player.collision.flags != 0 {
        h.session_mut().player.clearance.terrain_flags |= 2;
    }
    h.session_mut().player.clearance.terrain_down = (displacement >> 16) as u16;
    let rejected = match SIDES[usize::from(h.session().player.clearance.terrain_flags)] {
        ContactSides::Neither => clear_terrain(h),
        ContactSides::Above => terrain_above(h),
        ContactSides::Below => terrain_below(h),
        ContactSides::Both => crouch_or_reject(h),
    };
    if rejected {
        h.session_mut().player.pose = h.session().player.history.previous.pose;
    }
}
fn opposite_actor_probe<H: Host>(h: &mut H, above: bool) -> bool {
    let saved_radius = h.session().player.contact.radius;
    let expanded_radius = radius(h, false);
    h.session_mut().player.contact.radius = expanded_radius;
    h.session_mut().player.contact.collision_direction = if above { 3 } else { 2 };
    let difference = h.session().player.clearance.radius_difference as i16;
    let clearance = if above {
        h.session().player.clearance.actor_up
    } else {
        h.session().player.clearance.actor_down
    } as i16;
    let result = geometry::solid_contact(h, i32::from(difference.wrapping_sub(clearance)) << 16);
    // Restore only after a successful probe: a source failure keeps its write prefix.
    h.session_mut().player.contact.radius = saved_radius;
    h.session_mut().player.collision.flags =
        u16::from(matches!(result.obstruction, Obstruction::SolidActor(_))).wrapping_neg();
    if h.session().player.collision.flags != 0 {
        return true;
    }
    if above {
        h.session_mut().player.clearance.actor_up = (result.displacement >> 16) as u16;
    } else {
        h.session_mut().player.clearance.actor_down = (result.displacement >> 16) as u16;
    }
    false
}
fn terrain_above<H: Host>(h: &mut H) -> bool {
    let difference = h.session().player.clearance.radius_difference as i16;
    let clearance = h.session().player.clearance.terrain_up as i16;
    let amount = geometry::pose_probe(h, i32::from(difference.wrapping_sub(clearance)) << 16);
    if h.session().player.collision.flags != 0 {
        return true;
    }
    if h.session().player.clearance.actor_flags & 2 != 0 {
        return crouch_or_reject(h);
    }
    h.session_mut().player.y = h.session().player.y.wrapping_add((amount >> 16) as u16);
    h.session_mut().player.previous_position.y.whole = h.session().player.y;
    false
}
fn terrain_below<H: Host>(h: &mut H) -> bool {
    let clearance = h.session().player.clearance.terrain_down as i16;
    let difference = h.session().player.clearance.radius_difference as i16;
    let amount = geometry::pose_probe(h, i32::from(clearance.wrapping_sub(difference)) << 16);
    if h.session().player.collision.flags != 0 {
        return true;
    }
    if h.session().player.clearance.actor_flags & 1 != 0 {
        return crouch_or_reject(h);
    }
    h.session_mut().player.y = h.session().player.y.wrapping_sub((amount >> 16) as u16);
    h.session_mut().player.previous_position.y.whole = h.session().player.y;
    false
}
fn clear_terrain<H: Host>(h: &mut H) -> bool {
    match h.session().player.clearance.actor_flags {
        0 => false,
        1 => {
            let clearance = h.session().player.clearance.actor_up;
            h.session_mut().player.y = h.session().player.y.wrapping_add(clearance);
            h.session_mut().player.previous_position.y.whole = h.session().player.y;
            false
        }
        2 => {
            let clearance = h.session().player.clearance.actor_down;
            h.session_mut().player.y = h.session().player.y.wrapping_sub(clearance);
            h.session_mut().player.previous_position.y.whole = h.session().player.y;
            false
        }
        3 => crouch_or_reject(h),
        flags => {
            h.invalid_clearance_flags(flags);
            false
        }
    }
}
fn crouch_or_reject<H: Host>(h: &mut H) -> bool {
    if (i32::from(h.session().player.contact.radius) - 8) & 0x8000 != 0 {
        return true;
    }
    h.session_mut().player.pose = if h.session().player.movement_and_facing as u8 == 4 {
        0x28
    } else {
        0x27
    };
    let crouch_radius = radius(h, false);
    if (i32::from(h.session().player.contact.radius) - i32::from(crouch_radius)) & 0x8000 != 0 {
        let adjustment = h
            .session()
            .player
            .contact
            .radius
            .wrapping_sub(crouch_radius);
        h.session_mut().player.y = h.session().player.y.wrapping_add(adjustment);
        h.session_mut().player.previous_position.y.whole = h.session().player.y;
    }
    false
}
