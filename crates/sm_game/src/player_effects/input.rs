//! Player input dispatch on the current Control and player owners.
#![forbid(unsafe_code)]
use crate::player_transitions as transitions;
use transitions::state::InputMode;

/// The four game input lanes retain their different histories and lifetimes.
#[derive(Clone, Copy)]
pub enum InputLane {
    Held,
    Pressed,
    FilteredHeld,
    FilteredPressed,
}
pub trait Host: transitions::effects::Host {
    fn set_input_lane(&mut self, lane: InputLane, value: u16);
    fn direction_button(&self, left: bool) -> u16;
    fn input_unknown(&mut self) -> bool;
}
fn xray(h: &mut impl Host) {
    let left = h.session().player.movement_and_facing.to_le_bytes()[0] == 4;
    let opposite = h.direction_button(!left) & h.inputs().held != 0;
    transitions::xray::turn(h, opposite);
}
fn normal(h: &mut impl Host) {
    if matches!(h.session().player.movement_and_facing >> 8, 5 | 21) && h.inputs().frozen != 0 {
        xray(h);
    } else {
        transitions::handle_input(h);
    }
}
/// Temporary AutoJump publishes Pressed before dispatch. A failed dispatch
/// retains that prefix, just as the original does; restoration is on success.
pub fn dispatch(h: &mut impl Host) {
    match h.session().player.transitions.input_mode {
        InputMode::Normal => normal(h),
        InputMode::Demo => {
            if transitions::demo::advance(h) {
                let demo = h.session().player.demo.clone();
                h.set_input_lane(InputLane::FilteredHeld, demo.previous_held);
                h.set_input_lane(InputLane::FilteredPressed, demo.previous_pressed);
                h.set_input_lane(InputLane::Held, demo.held);
                h.session_mut().player.demo.previous_held = demo.held;
                h.set_input_lane(InputLane::Pressed, demo.pressed);
                h.session_mut().player.demo.previous_pressed = demo.pressed;
            }
            normal(h);
        }
        InputMode::Xray => xray(h),
        InputMode::AutoJump => {
            let pressed = h.inputs().pressed;
            let timer = h.session().player.contact.auto_jump;
            if timer != 0 && timer.wrapping_sub(9) & 0x8000 != 0 {
                h.set_input_lane(InputLane::Pressed, pressed | h.inputs().jump_button);
                h.session_mut().player.contact.auto_jump = 0;
            }
            normal(h);
            h.set_input_lane(InputLane::Pressed, pressed);
            h.session_mut().player.transitions.input_mode = InputMode::Normal;
        }
        InputMode::Disabled => {}
        InputMode::Unknown(_) => {
            h.input_unknown();
        }
    }
}
