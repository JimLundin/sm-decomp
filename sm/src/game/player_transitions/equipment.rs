//! Reconcile the current pose and feedback after equipment changes.
#![forbid(unsafe_code)]
use super::{content::AnimationTable, effects::Host, state::FrameAlphaMode};
use crate::game::{
    samus_palette,
    session::Session,
    visual_host::{Effects as VisualEffects, Record},
    visual_state::PaletteBuffer,
};

pub(crate) fn facing(s: &Session, right: u16, left: u16) -> u16 {
    if s.player.movement_and_facing as u8 == 4 {
        left
    } else {
        right
    }
}

fn reconcile_pose(h: &mut impl Host) {
    let p = &h.session().player;
    let items = h.session().inventory.equipment.active;
    let pose = match p.movement_and_facing >> 8 {
        0 if matches!(p.pose, 0 | 0x9b) => {
            let pose = if items & 0x21 == 0 { 0 } else { 0x9b };
            (pose != p.pose).then_some(pose)
        }
        3 => {
            h.session_mut().player.history.previous.movement_and_facing =
                facing(h.session(), 0x108, 0x104);
            let pose = h.session().player.pose;
            let next = match pose {
                0x81 | 0x82 if items & 8 == 0 => Some(facing(h.session(), 0x19, 0x1a)),
                0x1b | 0x1c => Some(if items & 8 != 0 {
                    facing(h.session(), 0x81, 0x82)
                } else {
                    facing(h.session(), 0x19, 0x1a)
                }),
                _ => None,
            };
            if let Some(pose) = next {
                h.session_mut().player.pose = pose;
            }
            super::refresh_pose(h);
            h.session_mut().player.history.previous.movement_and_facing =
                facing(h.session(), 0x308, 0x304);
            super::record_pose_history(h.session_mut());
            return;
        }
        4 | 8 if items & 2 != 0 => Some(facing(h.session(), 0x79, 0x7a)),
        17..=19 if items & 2 == 0 => Some(facing(h.session(), 0x1d, 0x41)),
        20 => {
            h.session_mut().player.animation_clock.frame = if items & 8 != 0 { 23 } else { 3 };
            None
        }
        0..=27 => None,
        _ => panic!("invalid equipment movement kind"),
    };
    if let Some(pose) = pose {
        h.session_mut().player.pose = pose;
        super::finish_pose_change(h);
    }
}

pub(crate) fn refresh(h: &mut impl Host) {
    reconcile_pose(h);
    if h.session().inventory.equipment.active & 0x2000 != 0 {
        let p = &mut h.session_mut().player;
        if p.contact.momentum != 0 && p.contact.speed_boost == 0 {
            p.palette.timer = p.contact.speed_boost;
            p.palette_frame = 0;
            h.session_mut().player.contact.speed_boost = h
                .animation_table(AnimationTable::BoosterStageDurations)
                .native_word(0);
        }
    } else {
        let s = h.session_mut();
        s.projectiles.echoes.index = 0;
        s.projectiles.echoes.x_speed[..2].fill(0);
        s.player.contact.momentum = 0;
        s.player.contact.speed_boost = 0;
        s.player.palette_frame = 0;
        s.player.palette.timer = 0;
        s.projectiles.echoes.x[..2].fill(0);
        s.projectiles.echoes.y[..2].fill(0);
    }
    if h.session().player.grapple == 0xc4f0 {
        if h.session().inventory.beams.active & 0x1000 != 0 {
            if h.session().projectiles.controls.charge.wrapping_sub(16) & 0x8000 == 0 {
                VisualEffects::sound(h, 1, 6, 0x41);
            }
        } else {
            h.session_mut().projectiles.controls.charge = 0;
            h.session_mut().projectiles.controls.clear_flare_animation();
        }
    } else {
        samus_palette::write_beam_palette(h, 2);
        VisualEffects::sound(h, 1, 6, 6);
    }
    samus_palette::load_suit(h, PaletteBuffer::Current);
    if h.session().inventory.health.current.wrapping_sub(31) & 0x8000 != 0 {
        VisualEffects::sound(h, 3, 6, 2);
    }
}

pub(crate) fn face_forward(h: &mut impl Host) {
    h.session_mut().player.pose = if h.session().inventory.equipment.active & 0x21 == 0 {
        0
    } else {
        0x9b
    };
    super::finish_pose_change(h);
    let p = &mut h.session_mut().player;
    if p.contact.radius != 24 {
        p.y = p.y.wrapping_sub(3);
        p.previous_position.y.whole = p.y;
    }
    p.transitions.frame_alpha = FrameAlphaMode::from_raw(0xe713);
    p.transitions.frame_beta = 0xe8dc;
    p.transitions.clear_pose_requests();
    super::cancel_grapple(h);
    let p = &mut h.session_mut().player;
    p.contact.extra_run = Default::default();
    p.contact.base_run = Default::default();
    p.contact.y_speed = Default::default();
    p.contact.y_direction = 0;
    p.transitions.bounce = Default::default();
    p.contact.acceleration_mode = 0;
    h.session_mut().projectiles.controls.charge = 0;
    h.session_mut().projectiles.controls.clear_flare_animation();
    samus_palette::load_suit(h, PaletteBuffer::Current);
}
