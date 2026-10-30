//! Pose metadata, animation reset, and shared geometry over owned player state.
use super::{
    content::{AnimationProgram, AnimationTable},
    effects::Host,
    state::AnimationSkip,
};
use crate::{
    liquid_surface::{self, LiquidAtHeight},
    session::Session,
    visual_host::Record,
    visual_state::PaletteBuffer,
};

pub fn loaded_top(h: &mut impl Host) -> u16 {
    let y = h.session().player.y;
    let index = usize::from(h.session().player.pose) * 8 + 6;
    y.wrapping_sub(u16::from(h.loaded_pose_table().byte(index)))
}
pub fn loaded_bottom(h: &mut impl Host) -> u16 {
    let y = h.session().player.y;
    let index = usize::from(h.session().player.pose) * 8 + 6;
    y.wrapping_add(u16::from(h.loaded_pose_table().byte(index)))
        .wrapping_sub(1)
}
pub fn liquid_at_height(h: &impl Host, y: u16) -> LiquidAtHeight {
    liquid_surface::at_height(
        h.inputs().fx_y,
        h.inputs().lava_y,
        h.session().room.liquid_options,
        y,
    )
}
pub fn apply_selected_pose(h: &mut impl Host) -> bool {
    let original = h.session().player.pose;
    if original != h.session().player.history.previous.pose {
        crate::player_collision::operations::fit_pose(h);
        refresh_pose_rules(h);
        super::jump::handle_transition(h);
        reset_pose_animation(h);
        h.session_mut().player.transitions.animation_skip = AnimationSkip::from_raw(0);
    }
    original != h.session().player.pose
}
pub fn refresh_pose_rules(h: &mut impl Host) {
    h.probe("transitions.phase.2");
    #[cfg(sm_verify_dll)]
    let observed_before = h.session().player.movement_and_facing;
    let table = h.pose_table();
    let value = table.native_word(usize::from(h.session().player.pose) * 8);
    h.session_mut().player.movement_and_facing = value;
    #[cfg(sm_verify_dll)]
    crate::transition_trace::publication(
        1,
        0xa1e,
        observed_before,
        h.session().player.movement_and_facing,
    );
    h.probe("transitions.phase.3");
    super::normalization::normalize(h);
    if matches!(
        h.session().player.history.previous.movement_and_facing >> 8,
        3 | 20
    ) && h.session().inventory.equipment.active & 8 != 0
    {
        crate::samus_palette::load_suit(h, PaletteBuffer::Current);
    }
}
pub fn reset_pose_animation(h: &mut impl Host) {
    let delay = if h.session().inventory.equipment.active & 0x20 != 0 {
        h.session().player.speed_divisor
    } else {
        let y = h.session().player.y;
        let table = h.pose_table();
        let radius = table.byte(usize::from(h.session().player.pose) * 8 + 6);
        match liquid_at_height(h, y.wrapping_add(u16::from(radius)).wrapping_sub(1)) {
            LiquidAtHeight::Water => 3,
            LiquidAtHeight::LavaAcid => 2,
            LiquidAtHeight::None => h.session().player.speed_divisor,
        }
    };
    if h.session().player.transitions.animation_skip.to_raw() & 0x8000 == 0
        && h.session().player.pose != h.session().player.history.previous.pose
    {
        h.session_mut().player.animation_clock.frame =
            h.session().player.transitions.animation_skip.to_raw();
        let table = h.animation_table(AnimationTable::PosePrograms);
        let program = AnimationProgram(table.native_word(usize::from(h.session().player.pose) * 2));
        let advance = h.session().player.transitions.animation_skip.to_raw();
        let bytes = h.animation_program(program, advance);
        h.session_mut().player.animation_clock.remaining =
            delay.wrapping_add(u16::from(bytes.byte(0)));
    }
}
pub fn record_pose_history(s: &mut Session) {
    s.player.history.last_different = s.player.history.previous.clone();
    s.player.history.previous.pose = s.player.pose;
    s.player.history.previous.movement_and_facing = s.player.movement_and_facing;
}
pub fn refresh_pose(h: &mut impl Host) {
    refresh_pose_rules(h);
    reset_pose_animation(h);
}
pub fn finish_pose_change(h: &mut impl Host) {
    refresh_pose(h);
    record_pose_history(h.session_mut());
}
pub fn align_bottom_with_previous_pose(h: &mut impl Host) {
    let index = usize::from(h.session().player.pose) * 8 + 6;
    let current = h.loaded_pose_table().byte(index);
    let index = usize::from(h.session().player.history.previous.pose) * 8 + 6;
    let previous = h.loaded_pose_table().byte(index);
    let adjustment = u16::from(previous).wrapping_sub(u16::from(current));
    let p = &mut h.session_mut().player;
    p.y = p.y.wrapping_add(adjustment);
    p.previous_position.y.whole = p.previous_position.y.whole.wrapping_add(adjustment);
}
pub fn clamp_previous_position(s: &mut Session, captured_x_delta: i16) {
    let p = &mut s.player;
    if (p.x.wrapping_sub(p.previous_position.x.whole) as i16) < 0 {
        if (i32::from(captured_x_delta) + 12) & 0x8000 != 0 {
            p.previous_position.x.whole = p.x.wrapping_add(12);
        }
    } else if (i32::from(captured_x_delta) - 13) & 0x8000 == 0 {
        p.previous_position.x.whole = p.x.wrapping_sub(12);
    }
    let delta = p.y.wrapping_sub(p.previous_position.y.whole) as i16;
    if (p.y.wrapping_sub(p.previous_position.y.whole) as i16) < 0 {
        if (i32::from(delta) + 12) & 0x8000 != 0 {
            p.previous_position.y.whole = p.y.wrapping_add(12);
        }
    } else if (i32::from(delta) - 13) & 0x8000 == 0 {
        p.previous_position.y.whole = p.y.wrapping_sub(12);
    }
}
