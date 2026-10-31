//! X-ray entry gates, turning and the original stand-up exit adjustment.
use super::{
    effects::Host,
    state::{InputMode, MovementMode, PoseRequestWord},
};
use crate::player_transitions::effects::SamusLease;
use crate::ram::words as ram;
use crate::runtime_access::SessionRam;
use crate::{session::Session, visual_host::Record};

pub fn request(h: &mut impl Host) -> bool {
    const POSTURES: [u8; 28] = [
        1, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0,
    ];
    let s = h.session();
    let pose = s.player.pose;
    if s.projectiles.controls.cooldown == 7
        && s.projectiles.bomb_count == 5
        && s.player.speed_divisor == 2
        || pose.wrapping_sub(0xa4) & 0x8000 == 0
            && (pose.wrapping_sub(0xa8) & 0x8000 != 0
                || pose.wrapping_sub(0xe0) & 0x8000 == 0 && pose.wrapping_sub(0xe8) & 0x8000 != 0)
        || h.inputs().transitions.game_state != 8
        || h.inputs().power_bomb_active != 0
        || s.player.contact.y_speed.whole != 0
        || s.player.contact.y_speed.fraction != 0
        || POSTURES[usize::from(s.projectiles.preparation.previous_movement_and_facing >> 8)] == 0
    {
        return false;
    }
    let posture = POSTURES[usize::from(s.player.movement_and_facing >> 8)];
    let pose = match posture {
        1 => 0xd5,
        2 => 0xd9,
        _ => return false,
    } + u16::from(s.player.movement_and_facing as u8 == 4);
    h.session_mut().player.transitions.interrupted = PoseRequestWord::from_raw(pose);
    true
}

/// Each scheduler retains its other flags; their original update order is fixed.
pub fn set_world_running(s: &mut Session, running: bool) {
    let visual = s.presentation.visual_mut();
    for flags in [
        &mut s.enemy_projectiles.flags,
        &mut s.mechanisms.flags,
        &mut visual.animated_tiles.flags,
        &mut visual.palette_effects.flags,
    ] {
        *flags = if running {
            *flags | 0x8000
        } else {
            *flags & 0x7fff
        };
    }
}

pub fn finish(h: &mut impl Host) {
    let state = h.session_mut();
    let p = &mut state.player;
    p.pose = if p.movement_and_facing >> 8 == 5 {
        0x27
    } else {
        1
    } + u16::from(p.movement_and_facing as u8 == 4);
    super::finish_pose_change(h);
    let state = h.session_mut();
    let p = &mut state.player;
    p.movement_handler = MovementMode::Normal;
    p.transitions.input_mode = InputMode::Normal;
    state.presentation.palette_mut().xray_progress = u16::MAX;
    // Capture native content before rereading the pose changed by callbacks.
    let table = h.pose_table();
    let adjustment = u16::from(table.byte(usize::from(h.session().player.pose) * 8 + 6))
        .wrapping_sub(h.session().player.contact.radius);
    if adjustment & 0x8000 == 0 {
        let state = h.session_mut();
        let p = &mut state.player;
        p.y = p.y.wrapping_sub(adjustment);
        p.previous_position.y.whole = p.y;
    }
    set_world_running(h.session_mut(), true);
}

pub fn turn(h: &mut impl Host, opposite_direction_held: bool) {
    let state = h.session_mut();
    let p = &mut state.player;
    let left = p.movement_and_facing as u8 == 4;
    if p.movement_and_facing >> 8 == 14 {
        if p.animation_clock.frame != 2 || p.animation_clock.remaining != 1 {
            return;
        }
        let standing = p.pose == if left { 0x25 } else { 0x26 };
        p.pose = if standing { 0xd5 } else { 0xd9 } + u16::from(left);
    } else {
        if !opposite_direction_held {
            return;
        }
        p.transitions.xray_angle = 256u16.wrapping_sub(p.transitions.xray_angle);
        p.pose = if p.movement_and_facing >> 8 == 5 {
            0x43
        } else {
            0x25
        } + u16::from(left);
    }
    super::finish_pose_change(h);
}

#[derive(Clone, Copy)]
pub enum BlendPhase {
    Setup,
    Beam,
    Restore,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BeamPhase {
    Initial,
    Widening,
    Full,
}

/// Keep the original wide equality tests and signed wrapping carry/borrow tests.
pub fn steer(angle: &mut u16, width: u16, up: bool) {
    if (angle.wrapping_sub(128) & 0x8000 != 0) == up {
        let limit = if up { 0 } else { 128 };
        let gap = i32::from(*angle) - i32::from(width);
        if gap != limit
            && (((gap - limit) as i16) < 0 || {
                let borrow = i32::from(*angle == 0);
                *angle = angle.wrapping_sub(1);
                ((i32::from(*angle) - i32::from(width) - borrow - limit) as i16) < 0
            })
        {
            *angle = width.wrapping_add(limit as u16);
        }
    } else {
        let limit = if up { 256 } else { 128 };
        let sum = i32::from(width) + i32::from(*angle);
        if sum != limit
            && ((sum - limit) as i16 >= 0 || {
                let carried = width
                    .wrapping_add(*angle)
                    .wrapping_add(u16::from(*angle == u16::MAX));
                *angle = angle.wrapping_add(1);
                carried != limit as u16 && (carried as i16).wrapping_sub(limit as i16) >= 0
            })
        {
            *angle = (limit as u16).wrapping_sub(width);
        }
    }
}

pub fn widen(demo: &mut super::demo::DemoInput) {
    let speed = ((u32::from(demo.delay) << 16) | u32::from(demo.program)).wrapping_add(2048);
    demo.program = speed as u16;
    demo.delay = (speed >> 16) as u16;
    let width = ((u32::from(demo.held) << 16) | u32::from(demo.pressed)).wrapping_add(speed);
    demo.pressed = width as u16;
    demo.held = (width >> 16) as u16;
    if demo.held.wrapping_sub(11) & 0x8000 == 0 {
        demo.pressed = 0;
        demo.held = 10;
        demo.pre_action = demo.pre_action.wrapping_add(1);
    }
}

/// Store word `index` of the X-ray and suit-pickup HDMA window table ($7E:9800).
pub fn set_window_word<H: SessionRam + ?Sized>(h: &mut H, index: u16, value: u16) {
    h.write_slice_le(0x9800 + usize::from(index) * 2, value);
}

/// Fill the window table with the closed window, last row first.
pub fn clear_window(h: &mut (impl SessionRam + ?Sized)) {
    for row in (0..256).rev() {
        set_window_word(h, row, 255);
    }
}

/// The beam's blend layer: BG3 in FX $24, BG2 in some rooms and boss fights, else BG1.
pub fn blend_mask(h: &(impl SessionRam + ?Sized)) -> u16 {
    if h.word(ram::FX_TYPE) == 0x24 {
        0x1000
    } else if matches!(h.word(ram::ROOM_PTR), 0xa66a | 0xcefb)
        || matches!(h.word(ram::BOSS_ID), 3 | 6 | 7 | 8 | 10)
    {
        0x2000
    } else {
        0x4000
    }
}

/// Set the beam's blend layer and, for BG1 (or BG3 in a dark haze), its colour.
pub fn apply_blending(h: &mut (impl SessionRam + ?Sized), phase: BlendPhase) {
    let mask = blend_mask(h);
    let update_colors = match phase {
        BlendPhase::Restore => false,
        BlendPhase::Beam => mask == 0x4000,
        BlendPhase::Setup => mask == 0x4000 || (mask == 0x1000 && (h.byte(0x74) & 0x1f) < 7),
    };
    if update_colors {
        // Preserve the overlapping native words, including the final zero byte.
        for (offset, value) in [(0x74, 0x27), (0x75, 0x47), (0x76, 0x87)] {
            h.write_slice_native(offset, value);
        }
    }
    h.write_slice_le(
        ram::FX_LAYER_BLENDING_CONFIG_C,
        h.word(ram::FX_LAYER_BLENDING_CONFIG_C) | mask,
    );
}

/// Turn the scope when the direction Samus faces away from is held.
pub fn handle_input(h: &mut (impl Host + SessionRam)) {
    let binding = if h.session().player.movement_and_facing as u8 == 4 {
        ram::BUTTON_CONFIG_RIGHT
    } else {
        ram::BUTTON_CONFIG_LEFT
    };
    let held = h.word(binding) & h.inputs().held != 0;
    turn(h, held);
}

/// Start the scope if Samus can: freeze time, close the window, set up its HDMA.
pub fn initialize(h: &mut (impl Host + SessionRam)) -> bool {
    if !request(h) {
        return false;
    }
    h.write_slice_le(ram::TIME_IS_FROZEN_FLAG, 1);
    h.session_mut().player.transitions.interruption = super::state::InterruptionAction::from_raw(5);
    clear_window(h);
    set_world_running(h.session_mut(), false);
    h.write_slice_native(0xa88, 1);
    h.set_byte(0xa8a, 0x98);
    h.write_slice_native(0xa8b, 0);
    h.session_mut().player.demo.previous_held = 0x98c8;
    h.write_slice_native(0xa8e, 0x9098);
    h.set_byte(0xa90, 0x99);
    h.write_slice_le(ram::XRAY_HDMA_TABLE_END, 0);
    let demo = &mut h.session_mut().player.demo;
    demo.pre_action = 0;
    demo.delay = 0;
    demo.program = 0;
    demo.held = 0;
    demo.pressed = 0;
    h.session_mut().player.transitions.xray_angle =
        if h.session().player.movement_and_facing as u8 == 4 {
            192
        } else {
            64
        };
    true
}
/// Draw the beam's window around Samus, or close it when she faces off screen.
pub fn draw_window<H: SamusLease>(h: &mut H) {
    let player = &h.session().player;
    let facing = player.movement_and_facing as u8;
    let x = player
        .x
        .wrapping_sub(h.inputs().camera_x)
        .wrapping_add(if facing == 4 { 65533 } else { 3 });
    let crouched = player.movement_and_facing >> 8 == 5;
    let y = player
        .y
        .wrapping_sub(h.inputs().camera_y)
        .wrapping_sub(if crouched { 12 } else { 16 });
    let off_screen = (x as i16) < 0 || x >= 256;
    if ((x as i16) < 0 && facing == 4) || ((x as i16) >= 256 && facing == 8) {
        clear_window(h);
        return;
    }
    let (angle, width) = (player.transitions.xray_angle, player.demo.held);
    let window = crate::beam_windows::XrayWindow {
        x,
        y,
        angle,
        width,
        off_screen,
        table: 0x9800,
    };
    h.draw_xray_window_table(window);
}
/// The X-ray background step: blend, then the beam's or the restore's stage.
pub fn advance_background<H: SamusLease>(h: &mut H) {
    use super::effects::XrayRestore;
    apply_blending(h, BlendPhase::Beam);
    let stages: [fn(&mut H); 6] = [
        |h| advance_beam(h, BeamPhase::Initial),
        |h| advance_beam(h, BeamPhase::Widening),
        |h| advance_beam(h, BeamPhase::Full),
        |h| h.restore_xray_background(XrayRestore::Left),
        |h| h.restore_xray_background(XrayRestore::Right),
        |h| h.restore_xray_background(XrayRestore::Finish),
    ];
    // Retain the six-entry bounds check after the blending writes.
    stages[usize::from(h.session().player.demo.pre_action)](h);
}
/// Widen or steer the beam while run is held, and redraw its window.
pub fn advance_beam<H: SamusLease>(h: &mut H, phase: BeamPhase) {
    if h.inputs().run_button & h.inputs().held == 0 {
        let demo = &mut h.session_mut().player.demo;
        demo.pre_action = if phase == BeamPhase::Full {
            demo.pre_action.wrapping_add(1)
        } else {
            3
        };
        return;
    }
    if phase == BeamPhase::Widening {
        widen(&mut h.session_mut().player.demo);
    } else if phase == BeamPhase::Full {
        let up = h.word(ram::BUTTON_CONFIG_UP) & h.inputs().held != 0;
        if up || h.word(ram::BUTTON_CONFIG_DOWN) & h.inputs().held != 0 {
            let player = &mut h.session_mut().player;
            steer(&mut player.transitions.xray_angle, player.demo.held, up);
        }
    }
    draw_window(h);
    if phase == BeamPhase::Initial {
        let demo = &mut h.session_mut().player.demo;
        demo.pre_action = demo.pre_action.wrapping_add(1);
    }
}
/// Run held and time running: start the scope and spawn its HDMA.
pub fn request_scope<H: SamusLease>(h: &mut H) {
    if h.inputs().frozen != 0 || h.inputs().run_button & h.inputs().held == 0 || !initialize(h) {
        return;
    }
    h.spawn_xray_scope_hdma();
}
