//! Priority and observation order for the three pose-request channels.
use super::{actions, effects::Host, pose};
use crate::player_collision::operations;

pub fn resolve_transitions(h: &mut impl Host) {
    let transitional = if h.session().player.transitions.transitional.to_raw() & 0x8000 == 0 {
        if h.session().player.transitions.hurt.to_raw() != 3 {
            Some(h.session().player.transitions.hurt.to_raw() != 1)
        } else if h.session().player.transitions.interruption.to_raw() != 9 {
            Some(true)
        } else {
            None
        }
    } else {
        None
    };
    let changed = if let Some(apply_pose) = transitional {
        if apply_pose {
            h.session_mut().player.pose = h.session().player.transitions.transitional.to_raw();
            pose::refresh_pose(h);
        }
        let action = h.session().player.transitions.hurt;
        actions::hurt(h, action);
        true
    } else if h.session().player.transitions.interrupted.to_raw() & 0x8000 == 0 {
        h.session_mut().player.pose = h.session().player.transitions.interrupted.to_raw();
        if !pose::apply_selected_pose(h) {
            let action = h.session().player.transitions.interruption;
            actions::interruption(h, action);
        }
        true
    } else {
        operations::walked_obstacle(h);
        if h.session().player.transitions.ordinary.to_raw() & 0x8000 == 0 {
            h.session_mut().player.pose = h.session().player.transitions.ordinary.to_raw();
            if !pose::apply_selected_pose(h) {
                let action = h.session().player.transitions.momentum;
                actions::momentum(h, action);
            }
            true
        } else {
            false
        }
    };
    if changed {
        pose::record_pose_history(h.session_mut());
    }
    h.session_mut().player.collision.pose_input = 0;
}
