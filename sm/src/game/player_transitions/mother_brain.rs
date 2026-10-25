//! Player reactions during Mother Brain's drain and Hyper Beam sequence.
#![forbid(unsafe_code)]
use super::{
    effects::Host,
    state::{AnimationSkip, FrameGammaMode},
};
use crate::game::{
    item_pickups::VisualUpdate, room_mechanisms::host::Effects as RoomEffects, samus_palette,
    visual_host::Record, visual_state::PaletteBuffer,
};

fn settle_drained_motion(h: &mut impl Host) {
    let table = h.pose_table();
    let radius = table.byte(usize::from(h.session().player.pose) * 8 + 6);
    let p = &mut h.session_mut().player;
    p.contact.radius = u16::from(radius);
    p.contact.base_run = Default::default();
    p.contact.y_speed = Default::default();
    p.contact.y_direction = 2;
}

pub(crate) fn react(h: &mut (impl Host + RoomEffects), event: u16) {
    match event {
        0 => {
            let p = &mut h.session_mut().player;
            p.y = p.y.wrapping_add(p.contact.radius).wrapping_sub(21);
            p.previous_position.y.whole = p.y;
            p.pose = if p.movement_and_facing as u8 == 4 {
                0xe9
            } else {
                0xe8
            };
            p.transitions.animation_skip = AnimationSkip::from_raw(2);
            super::refresh_pose(h);
            settle_drained_motion(h);
            h.session_mut().projectiles.controls.charge = 0;
            h.session_mut().projectiles.controls.clear_flare_animation();
            samus_palette::load_suit(h, PaletteBuffer::Current);
        }
        1 | 4 => {
            let p = &mut h.session_mut().player;
            p.animation_clock.remaining = 16;
            p.animation_clock.frame = if event == 1 { 0 } else { 8 };
            p.pose =
                if event == 1 { 0xea } else { 0xe8 } + u16::from(p.movement_and_facing as u8 == 4);
            if event == 1 {
                p.transitions.frame_gamma = FrameGammaMode::Disabled;
            }
        }
        2 => {
            let p = &mut h.session_mut().player;
            if let Some(frame) = match p.pose {
                0xe8 | 0xe9 => Some(13),
                0xea | 0xeb => Some(4),
                _ => None,
            } {
                p.animation_clock.remaining = 1;
                p.animation_clock.frame = frame;
            }
            settle_drained_motion(h);
        }
        3 => {
            h.session_mut().inventory.beams.active = 4105;
            RoomEffects::pickup_visual(h, VisualUpdate::Beams);
            h.spawn_palette(0xe1f0);
            h.session_mut().inventory.hyper_beam = 0x8000;
            h.session_mut().projectiles.controls.resume_sound = 0;
            return;
        }
        _ => panic!("invalid Mother Brain player event"),
    }
    super::record_pose_history(h.session_mut());
    h.session_mut().player.transitions.clear_pose_requests();
}
