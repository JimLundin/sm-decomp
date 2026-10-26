//! Scripted controller playback shares the live player's transition state.
#![forbid(unsafe_code)]
use super::{
    effects::Host,
    state::{DrawMode, FrameAlphaMode, FrameBetaMode, InputMode},
};
use crate::game::{
    session::Session,
    visual_host::{Content, Record, Source},
};

/// Mode-dependent words also projected through the retained X-ray/suit HDMA path.
/// The timer and angle reuse their existing palette/transition owners.
#[derive(Clone, Default, Debug, PartialEq)]
pub struct DemoInput {
    pub(crate) pre_action: u16,
    pub(crate) delay: u16,
    pub(crate) program: u16,
    pub(crate) held: u16,
    pub(crate) pressed: u16,
    pub(crate) enabled: u16,
    pub(crate) previous_held: u16,
    pub(crate) previous_pressed: u16,
}

pub(crate) fn start(s: &mut Session) {
    s.player.demo = DemoInput {
        enabled: 0x8000,
        ..DemoInput::default()
    };
    s.player.palette.xray_progress = 0;
    s.player.transitions.xray_angle = 0;
    s.player.transitions.input_mode = InputMode::Demo;
}
fn disable(s: &mut Session) {
    s.player.transitions.input_mode = InputMode::Normal;
    s.player.demo.enabled &= 0x7fff;
}
fn check_window(h: &mut impl Host, program: u16) {
    if program & 0x8000 == 0 {
        Content::legacy_native_record(h, 0x91, program);
    }
}
fn loaded_word(h: &impl Host, program: u16, offset: usize) -> u16 {
    u16::from_le_bytes(
        h.table(Source::Loaded, 0x918000 | u32::from(program))
            .bytes(offset),
    )
}
pub(crate) fn load(h: &mut impl Host, angle: u16, program: u16) {
    h.session_mut().player.transitions.xray_angle = angle;
    check_window(h, program);
    h.session_mut().player.demo.pre_action = loaded_word(h, program, 2);
    h.session_mut().player.demo.program = loaded_word(h, program, 4);
    h.session_mut().player.demo.delay = 1;
    h.session_mut().player.palette.xray_progress = 0;
}

pub(crate) fn load_data(h: &mut impl Host, set: u16, scene: u16) {
    let scene_offset = scene.wrapping_mul(16);
    let definition = scene_offset.wrapping_add(loaded_word(h, 0x8885, usize::from(set) * 2));
    check_window(h, definition);
    h.session_mut().inventory.equipment.owned = loaded_word(h, definition, 0);
    h.session_mut().inventory.equipment.active = h.session().inventory.equipment.owned;
    for index in 0..4 {
        let value = loaded_word(h, definition, 2 + index * 2);
        let inventory = &mut h.session_mut().inventory;
        let supply = if index < 3 {
            &mut inventory.ammunition[index]
        } else {
            &mut inventory.health
        };
        supply.maximum = value;
        supply.current = value;
    }
    h.session_mut().inventory.beams.owned = loaded_word(h, definition, 10);
    h.session_mut().inventory.beams.active = loaded_word(h, definition, 12);
    h.session_mut().inventory.reserve_health = 0;
    start(h.session_mut());
    check_window(h, definition);
    let object = loaded_word(h, definition, 14);
    load(h, object, object);
}

/// True requests the retained face-forward sequence before finishing setup.
pub(crate) fn initialize_scene(h: &mut impl Host, set: u16, scene: u16) -> bool {
    const POSES: [&[u16]; 4] = [
        &[0, 1, 1, 2, 1, 0x1f],
        &[2, 2, 1, 2, 1, 0x2a],
        &[2, 0xcd, 1, 1, 1, 1],
        &[0xca, 2, 2, 1, 2],
    ];
    let pose = POSES[usize::from(set)][usize::from(scene)];
    if pose == 0 {
        return true;
    }
    if (set, scene) == (3, 4) {
        h.session_mut().inventory.health.current = 20;
    }
    let shinespark = matches!((set, scene), (2, 1) | (3, 0));
    if shinespark {
        h.session_mut().player.transitions.draw_handler = DrawMode::Normal;
        super::start_shinespark(h);
    }
    h.session_mut().player.pose = pose;
    super::refresh_pose(h);
    if !shinespark {
        h.session_mut().player.transitions.draw_handler = DrawMode::Normal;
    }
    false
}

pub(crate) fn finish_setup(h: &mut impl Host) {
    use super::state::{HurtAction, InterruptionAction, MomentumAction};
    use crate::game::{samus_palette, visual_state::PaletteBuffer};
    let transitions = &mut h.session_mut().player.transitions;
    transitions.frame_alpha = FrameAlphaMode::from_raw(0xe6c9);
    transitions.frame_beta = FrameBetaMode::DemoMapped;
    transitions.momentum = MomentumAction::from_raw(0);
    transitions.interruption = InterruptionAction::from_raw(0);
    transitions.hurt = HurtAction::from_raw(0);
    samus_palette::load_suit(h, PaletteBuffer::Current);
}

fn pre_action(h: &mut impl Host) {
    let next = match h.session().player.demo.pre_action {
        0x864f if h.session().player.x.wrapping_sub(178) & 0x8000 != 0 => Some((0x866a, 0x8623)),
        0x866a if h.session().enemy_projectiles.slots[0].x.whole == 0 => Some((0x83bf, 0x864b)),
        0x8a9b if h.inputs().transitions.game_state == 44 => Some((0x8a9b, 0x8776)),
        0x8ab0 if h.session().player.movement_and_facing >> 8 != 26 => Some((0x8a9b, 0x9346)),
        0x83bf | 0x864f | 0x866a | 0x8a9b | 0x8ab0 => None,
        _ => {
            h.invalid_demo_instruction();
            None
        }
    };
    if let Some((pre_action, program)) = next {
        let demo = &mut h.session_mut().player.demo;
        demo.pre_action = pre_action;
        demo.program = program;
        demo.delay = 1;
    }
}
fn instruction(h: &mut impl Host, opcode: u16, cursor: u16) -> u16 {
    match opcode {
        0x8427 => {
            let demo = &mut h.session_mut().player.demo;
            demo.program = 0;
            demo.held = 0;
            demo.pressed = 0;
            return 0;
        }
        0x8434 => {
            h.session_mut().player.demo.pre_action =
                Content::legacy_native_record(h, 0x91, cursor).native_word(0);
            return cursor.wrapping_add(2);
        }
        0x843f => h.session_mut().player.demo.pre_action = 0x8447,
        0x8448 => return Content::legacy_native_record(h, 0x91, cursor).native_word(0),
        0x844f => {
            let timer = h.session().player.palette.xray_progress.wrapping_sub(1);
            h.session_mut().player.palette.xray_progress = timer;
            return if timer != 0 {
                Content::legacy_native_record(h, 0x91, cursor).native_word(0)
            } else {
                cursor.wrapping_add(2)
            };
        }
        0x8459 => {
            h.session_mut().player.palette.xray_progress =
                Content::legacy_native_record(h, 0x91, cursor).native_word(0);
            return cursor.wrapping_add(2);
        }
        0x85fc | 0x8682 => {
            h.session_mut().player.transitions.frame_alpha =
                FrameAlphaMode::from_raw(if opcode == 0x85fc { 0xe695 } else { 0xe8cd });
            h.session_mut().player.transitions.frame_beta = if opcode == 0x85fc {
                FrameBetaMode::Gameplay
            } else {
                FrameBetaMode::Frozen
            };
            disable(h.session_mut());
        }
        0x8739 => {
            h.session_mut().player.transitions.frame_alpha = FrameAlphaMode::from_raw(0xe713);
            h.session_mut().player.pose = 2;
            super::finish_pose_change(h);
            disable(h.session_mut());
            h.session_mut().player.transitions.input_mode = InputMode::Disabled;
        }
        _ => return h.invalid_demo_instruction(),
    }
    cursor
}
/// True retains the caller's input publication even when the script finishes.
pub(crate) fn advance(h: &mut impl Host) -> bool {
    if h.session().player.demo.enabled & 0x8000 == 0 || h.session().player.demo.program == 0 {
        return false;
    }
    pre_action(h);
    let delay = h.session().player.demo.delay.wrapping_sub(1);
    h.session_mut().player.demo.delay = delay;
    if delay == 0 {
        let mut program = h.session().player.demo.program;
        loop {
            check_window(h, program);
            let opcode = loaded_word(h, program, 0);
            if loaded_word(h, program, 0) & 0x8000 == 0 {
                h.session_mut().player.demo.delay = loaded_word(h, program, 0);
                check_window(h, program);
                h.session_mut().player.demo.held = loaded_word(h, program, 2);
                h.session_mut().player.demo.pressed = loaded_word(h, program, 4);
                h.session_mut().player.demo.program = program.wrapping_add(6);
                break;
            }
            program = instruction(h, opcode, program.wrapping_add(2));
            if program == 0 {
                break;
            }
        }
    }
    true
}
