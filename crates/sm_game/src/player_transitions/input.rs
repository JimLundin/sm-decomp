//! Button-driven pose requests on the existing player transition owner.
use super::state::InputMode;
use super::{
    content::TransitionTable,
    effects::Host,
    jump::initialize_jump,
    state::{BombJump, JumpKind, MomentumAction, PoseRequestWord},
};
use crate::controller_state::Word as Input;
use crate::session_access::SessionAccess;
use crate::{
    external_inputs::ExternalInputs,
    visual_host::{Content, Record},
};
use crate::{lease::Lease, runtime_access::SessionView};

pub fn handle_input(h: &mut impl Host) {
    let movement = h.session().player.movement_and_facing >> 8;
    sm_data::table::check_index(usize::from(movement), 28);
    if matches!(movement, 7 | 9 | 11 | 15)
        || (movement == 0
            && matches!(h.session().player.pose, 0 | 0x9b)
            && h.session().room.entry.elevator_status != 0)
    {
        return;
    }
    lookup_pose(h);
    let current = h.session().player.movement_and_facing >> 8;
    if movement == 5 && current == 0 {
        let player = &mut h.session_mut().player;
        player.y = player.y.wrapping_sub(5);
        player.previous_position.y.whole = player.previous_position.y.whole.wrapping_sub(5);
    } else if movement == 10 && current != 10 {
        initialize_jump(h, JumpKind::Normal);
        h.session_mut().player.knockback_timer = 0;
    }
}

fn translated_buttons(inputs: &ExternalInputs, keys: u16) -> u16 {
    let mut translated = keys & 0x0f00;
    for (binding, button) in [
        (inputs.shoot_button, 0x0040),
        (inputs.jump_button, 0x0080),
        (inputs.run_button, 0x8000),
        (inputs.cancel_button, 0x4000),
        (inputs.aim_up_button, 0x0010),
        (inputs.aim_down_button, 0x0020),
    ] {
        if binding & keys != 0 && (button & 0x0030 == 0 || binding & 0x0030 != 0) {
            translated |= button;
        }
    }
    !translated
}

fn lookup_pose(h: &mut impl Host) {
    if h.inputs().held != 0 {
        let pressed = translated_buttons(h.inputs(), h.inputs().pressed);
        let held = translated_buttons(h.inputs(), h.inputs().held);
        let programs = h.transition_table(TransitionTable::InputPrograms);
        let program = programs.native_word(usize::from(h.session().player.pose) * 2);
        let entries = Content::legacy_native_record(h, 0x91, program);
        // An initially empty program preserves both the request and compatibility word.
        if entries.native_word(0) == 0xffff {
            return;
        }
        let mut row = 0;
        loop {
            if pressed & entries.native_word(row) == 0 && held & entries.native_word(row + 2) == 0 {
                if entries.native_word(row + 4) != h.session().player.pose {
                    h.session_mut().player.transitions.ordinary =
                        PoseRequestWord::from_raw(entries.native_word(row + 4));
                    h.session_mut().player.transitions.bomb_jump = BombJump::from_raw(0);
                }
                return;
            }
            // Records retain one allocation; successive rows advance linearly across banks.
            row += 6;
            if entries.native_word(row) == 0xffff {
                break;
            }
        }
    }
    h.session_mut().player.transitions.landing_compatibility = 0;
    request_fallback_pose(h);
}

pub fn request_fallback_pose(h: &mut impl Host) {
    const ACTIONS: [u16; 28] = [
        2, 1, 1, 0, 6, 2, 8, 2, 1, 6, 2, 2, 2, 6, 2, 2, 2, 6, 6, 6, 6, 2, 6, 2, 2, 2, 2, 2,
    ];
    let mut action = ACTIONS[usize::from(h.session().player.movement_and_facing >> 8)];
    if action == 1 {
        let speed = &h.session().player.contact.base_run;
        if speed.whole == 0 && speed.fraction == 0 {
            action = 2;
        }
    }
    h.session_mut().player.transitions.momentum = MomentumAction::from_raw(action);
    let pose = if action == 1
        || h.pose_table()
            .byte(usize::from(h.session().player.pose) * 8 + 2)
            == 255
    {
        h.session().player.pose
    } else {
        u16::from(
            h.pose_table()
                .byte(usize::from(h.session().player.pose) * 8 + 2),
        )
    };
    h.session_mut().player.transitions.ordinary = PoseRequestWord::from_raw(pose);
}

/// End-of-frame input history and hurt-flash health share the player owner.
pub fn finish_input(s: &mut crate::session::Session, inputs: &ExternalInputs) {
    let p = &mut s.player;
    p.contact.auto_jump = if inputs.held & inputs.jump_button != 0
        && inputs.filtered_held & inputs.jump_button != 0
    {
        p.contact.auto_jump.wrapping_add(1)
    } else {
        0
    };
    let previous = p.previous_health_for_flash;
    if (s.inventory.health.current as i16).wrapping_sub(previous as i16) < 0 {
        if s.presentation.palette().hurt_flash == 0 {
            s.presentation.palette_mut().hurt_flash = 1;
        }
        if (inputs.transitions.debug_invincibility.wrapping_sub(7) as i16) >= 0 {
            s.inventory.health.current = previous;
            return;
        }
    }
    p.previous_health_for_flash = s.inventory.health.current;
}
/// The frame's input step for the current input mode.
pub fn dispatch_input<H: SessionView + Lease>(h: &mut H) {
    match h.session().player.transitions.input_mode {
        InputMode::Normal => handle_frame_input(h),
        InputMode::Demo => handle_demo_input(h),
        InputMode::Xray => super::xray::handle_input(h),
        InputMode::AutoJump => {
            let pressed = h.inputs().pressed;
            let timer = h.session().player.contact.auto_jump;
            if timer != 0 && (timer.wrapping_sub(9) as i16) < 0 {
                let jump = h.inputs().jump_button;
                store_input(h, Input::Joypad1Newkeys, pressed | jump);
                h.session_mut().player.contact.auto_jump = 0;
            }
            handle_frame_input(h);
            store_input(h, Input::Joypad1Newkeys, pressed);
            h.session_mut().player.transitions.input_mode = InputMode::Normal;
        }
        InputMode::Disabled => {}
        InputMode::Unknown(_) => _ = crate::player_projectiles::host::Host::unknown(h),
    }
}
/// Frozen crouch and turn input uses the same owned X-ray transition.
fn handle_frame_input<H: SessionView + Lease>(h: &mut H) {
    if matches!(h.session().player.movement_and_facing >> 8, 5 | 21) && h.inputs().frozen != 0 {
        h.owned_call(|h| super::xray::handle_input(h));
    } else {
        handle_input(h);
    }
}
/// Play the demo's next input into the joypad words, then handle it.
fn handle_demo_input<H: SessionView + Lease>(h: &mut H) {
    if super::demo::advance(h) {
        // The joypad words do not alias the demo's ($0A84-$0A8E).
        let demo = h.session().player.demo.clone();
        store_input(h, Input::Joypad1InputSamusfilter, demo.previous_held);
        store_input(h, Input::Joypad1NewinputSamusfilter, demo.previous_pressed);
        store_input(h, Input::Joypad1Lastkeys, demo.held);
        h.session_mut().player.demo.previous_held = demo.held;
        store_input(h, Input::Joypad1Newkeys, demo.pressed);
        h.session_mut().player.demo.previous_pressed = demo.pressed;
    }
    handle_frame_input(h);
}
/// The player frame's end: filtered input, movement sound, debug, escape state.
pub fn finish_frame<H: SessionView + Lease>(h: &mut H) {
    let inputs = h.inputs().clone();
    finish_input(h.session_mut(), &inputs);
    store_input(h, Input::Joypad1InputSamusfilter, inputs.held);
    store_input(h, Input::Joypad1NewinputSamusfilter, inputs.pressed);
    h.owned_call(crate::player_projectiles::feedback::movement_sound);
    if h.session().game_loop.debug.enabled != 0 {
        if matches!(h.session().player.pose, 0 | 0x9b) {
            let control = &h.session().control;
            if *Input::Joypad2Last.get(control) & 0x30 == 0x30
                && *Input::Joypad2NewKeys.get(control) & 0x80 != 0
            {
                set_debug_invincibility(h, 7);
            }
        } else {
            let invincibility = h.session().menu_screens.options_menu.debug_invincibility;
            if (invincibility.wrapping_sub(7) as i16) >= 0 {
                return;
            }
            set_debug_invincibility(h, 0);
        }
    }
    if h.owned_call(|h| crate::progress::event_happened(h, 0xe))
        && h.session().player.transitions.frame_gamma
            == super::state::FrameGammaMode::DrawEscapeTimer
        && h.session().game_loop.mode != 35
    {
        h.session_mut().game_loop.mode = 35;
        h.words_stored();
    }
}

/// Store joypad word `word`; the observations refresh after it.
fn store_input<H: SessionAccess + ?Sized>(h: &mut H, word: Input, value: u16) {
    *word.get_mut(&mut h.session_mut().control) = value;
    h.words_stored();
}

/// Store the debug invincibility; the observations refresh after it.
pub(super) fn set_debug_invincibility<H: SessionAccess + ?Sized>(h: &mut H, value: u16) {
    h.session_mut()
        .menu_screens
        .options_menu
        .debug_invincibility = value;
    h.words_stored();
}
