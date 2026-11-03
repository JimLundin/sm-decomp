//! Controller values for a legacy execution interval, with no invented capture/policy.
//! Owned execution uses the original Control metadata InputHistory directly.
#![forbid(unsafe_code)]
use crate::scene::start::{ActionBindings, PortHistory};
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ControllerState {
    pub ports: [PortHistory; 2],
    pub repeat_first: u16,
    pub repeat_next: u16,
    pub released: u16,
    pub held_timer: u16,
    pub held_timer_reset: u16,
    pub timed_held: u16,
    pub previous_timed_held: u16,
    pub newly_held: u16,
    pub samus_held: u16,
    pub samus_pressed: u16,
    pub bindings: ActionBindings,
    pub bug_fix_counter: u16,
}
impl Default for ControllerState {
    fn default() -> Self {
        let port = PortHistory {
            sampled: 0,
            previous: 0,
            pressed: 0,
            repeated: 0,
            repeat_counter: 0,
        };
        Self {
            ports: [port; 2],
            repeat_first: 0,
            repeat_next: 0,
            released: 0,
            held_timer: 0,
            held_timer_reset: 0,
            timed_held: 0,
            previous_timed_held: 0,
            newly_held: 0,
            samus_held: 0,
            samus_pressed: 0,
            bindings: ActionBindings {
                shoot: 0,
                jump: 0,
                run: 0,
                cancel: 0,
                switch: 0,
                aim_down: 0,
                aim_up: 0,
            },
            bug_fix_counter: 0,
        }
    }
}
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct LegacyControl {
    pub primary: u8,
    pub secondary: u8,
    pub input: ControllerState,
    /// The audio request bytes, which an owned start keeps with its meanings.
    pub audio: crate::session_owners::LegacyAudio,
}
/// Pure input edge/repeat transition for a validated controller port.
pub fn repeat(port: &mut PortHistory, sampled: u16, first: u16, next: u16) {
    port.sampled = sampled;
    port.pressed = sampled & (port.previous ^ sampled);
    port.repeated = sampled & (port.previous ^ sampled);
    if sampled != 0 && sampled == port.previous {
        port.repeat_counter = port.repeat_counter.wrapping_sub(1);
        if port.repeat_counter == 0 {
            port.repeated = port.sampled;
            port.repeat_counter = next;
        }
    } else {
        port.repeat_counter = first;
    }
    port.previous = port.sampled;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Word {
    Joypad1Lastkeys,
    Joypad1Newkeys,
    Joypad1Newkeys2Unused,
    Joypad1Prev,
    Joypad1KeyrepeatCtr,
    Joypad2Last,
    Joypad2NewKeys,
    Joypad2Newkeys2,
    Joypad2Prev,
    Joypad2KeyrepeatCtr,
    JoypadCtrRepeatFirst,
    JoypadCtrRepeatNext,
    JoypadReleasedKeys,
    TimedHeldInputTimer,
    TimedHeldInputTimerReset,
    TimedHeldInput,
    PreviousTimedHeldInput,
    NewlyHeldDownTimedHeldInput,
    Joypad1InputSamusfilter,
    Joypad1NewinputSamusfilter,
    BugFixCounter,
    ButtonConfigShootX,
    ButtonConfigJumpA,
    ButtonConfigRunB,
    ButtonConfigItemcancelY,
    ButtonConfigItemswitch,
    ButtonConfigAimDownL,
    ButtonConfigAimUpR,
}
impl Word {
    pub const ALL: &[Self] = &[
        Self::Joypad1Lastkeys,
        Self::Joypad1Newkeys,
        Self::Joypad1Newkeys2Unused,
        Self::Joypad1Prev,
        Self::Joypad1KeyrepeatCtr,
        Self::Joypad2Last,
        Self::Joypad2NewKeys,
        Self::Joypad2Newkeys2,
        Self::Joypad2Prev,
        Self::Joypad2KeyrepeatCtr,
        Self::JoypadCtrRepeatFirst,
        Self::JoypadCtrRepeatNext,
        Self::JoypadReleasedKeys,
        Self::TimedHeldInputTimer,
        Self::TimedHeldInputTimerReset,
        Self::TimedHeldInput,
        Self::PreviousTimedHeldInput,
        Self::NewlyHeldDownTimedHeldInput,
        Self::Joypad1InputSamusfilter,
        Self::Joypad1NewinputSamusfilter,
        Self::BugFixCounter,
        Self::ButtonConfigShootX,
        Self::ButtonConfigJumpA,
        Self::ButtonConfigRunB,
        Self::ButtonConfigItemcancelY,
        Self::ButtonConfigItemswitch,
        Self::ButtonConfigAimDownL,
        Self::ButtonConfigAimUpR,
    ];
    pub fn address(self) -> usize {
        match self {
            Self::Joypad1Lastkeys => crate::ram::words::JOYPAD1_LASTKEYS,
            Self::Joypad1Newkeys => crate::ram::words::JOYPAD1_NEWKEYS,
            Self::Joypad1Newkeys2Unused => crate::ram::words::JOYPAD1_NEWKEYS2_UNUSED,
            Self::Joypad1Prev => crate::ram::words::JOYPAD1_PREV,
            Self::Joypad1KeyrepeatCtr => crate::ram::words::JOYPAD1_KEYREPEAT_CTR,
            Self::Joypad2Last => crate::ram::words::JOYPAD2_LAST,
            Self::Joypad2NewKeys => crate::ram::words::JOYPAD2_NEW_KEYS,
            Self::Joypad2Newkeys2 => crate::ram::words::JOYPAD2_NEWKEYS2,
            Self::Joypad2Prev => crate::ram::words::JOYPAD2_PREV,
            Self::Joypad2KeyrepeatCtr => crate::ram::words::JOYPAD2_KEYREPEAT_CTR,
            Self::JoypadCtrRepeatFirst => crate::ram::words::JOYPAD_CTR_REPEAT_FIRST,
            Self::JoypadCtrRepeatNext => crate::ram::words::JOYPAD_CTR_REPEAT_NEXT,
            Self::JoypadReleasedKeys => crate::ram::words::JOYPAD_RELEASED_KEYS,
            Self::TimedHeldInputTimer => crate::ram::words::TIMED_HELD_INPUT_TIMER,
            Self::TimedHeldInputTimerReset => crate::ram::words::TIMED_HELD_INPUT_TIMER_RESET,
            Self::TimedHeldInput => crate::ram::words::TIMED_HELD_INPUT,
            Self::PreviousTimedHeldInput => crate::ram::words::PREVIOUS_TIMED_HELD_INPUT,
            Self::NewlyHeldDownTimedHeldInput => {
                crate::ram::words::NEWLY_HELD_DOWN_TIMED_HELD_INPUT
            }
            Self::Joypad1InputSamusfilter => crate::ram::words::JOYPAD1_INPUT_SAMUSFILTER,
            Self::Joypad1NewinputSamusfilter => crate::ram::words::JOYPAD1_NEWINPUT_SAMUSFILTER,
            Self::BugFixCounter => crate::ram::words::BUG_FIX_COUNTER,
            Self::ButtonConfigShootX => crate::ram::words::BUTTON_CONFIG_SHOOT_X,
            Self::ButtonConfigJumpA => crate::ram::words::BUTTON_CONFIG_JUMP_A,
            Self::ButtonConfigRunB => crate::ram::words::BUTTON_CONFIG_RUN_B,
            Self::ButtonConfigItemcancelY => crate::ram::words::BUTTON_CONFIG_ITEMCANCEL_Y,
            Self::ButtonConfigItemswitch => crate::ram::words::BUTTON_CONFIG_ITEMSWITCH,
            Self::ButtonConfigAimDownL => crate::ram::words::BUTTON_CONFIG_AIM_DOWN_L,
            Self::ButtonConfigAimUpR => crate::ram::words::BUTTON_CONFIG_AIM_UP_R,
        }
    }
    pub fn get_state(self, state: &ControllerState) -> &u16 {
        match self {
            Self::Joypad1Lastkeys => &state.ports[0].sampled,
            Self::Joypad1Newkeys => &state.ports[0].pressed,
            Self::Joypad1Newkeys2Unused => &state.ports[0].repeated,
            Self::Joypad1Prev => &state.ports[0].previous,
            Self::Joypad1KeyrepeatCtr => &state.ports[0].repeat_counter,
            Self::Joypad2Last => &state.ports[1].sampled,
            Self::Joypad2NewKeys => &state.ports[1].pressed,
            Self::Joypad2Newkeys2 => &state.ports[1].repeated,
            Self::Joypad2Prev => &state.ports[1].previous,
            Self::Joypad2KeyrepeatCtr => &state.ports[1].repeat_counter,
            Self::JoypadCtrRepeatFirst => &state.repeat_first,
            Self::JoypadCtrRepeatNext => &state.repeat_next,
            Self::JoypadReleasedKeys => &state.released,
            Self::TimedHeldInputTimer => &state.held_timer,
            Self::TimedHeldInputTimerReset => &state.held_timer_reset,
            Self::TimedHeldInput => &state.timed_held,
            Self::PreviousTimedHeldInput => &state.previous_timed_held,
            Self::NewlyHeldDownTimedHeldInput => &state.newly_held,
            Self::Joypad1InputSamusfilter => &state.samus_held,
            Self::Joypad1NewinputSamusfilter => &state.samus_pressed,
            Self::BugFixCounter => &state.bug_fix_counter,
            Self::ButtonConfigShootX => &state.bindings.shoot,
            Self::ButtonConfigJumpA => &state.bindings.jump,
            Self::ButtonConfigRunB => &state.bindings.run,
            Self::ButtonConfigItemcancelY => &state.bindings.cancel,
            Self::ButtonConfigItemswitch => &state.bindings.switch,
            Self::ButtonConfigAimDownL => &state.bindings.aim_down,
            Self::ButtonConfigAimUpR => &state.bindings.aim_up,
        }
    }
    pub fn get_state_mut(self, state: &mut ControllerState) -> &mut u16 {
        match self {
            Self::Joypad1Lastkeys => &mut state.ports[0].sampled,
            Self::Joypad1Newkeys => &mut state.ports[0].pressed,
            Self::Joypad1Newkeys2Unused => &mut state.ports[0].repeated,
            Self::Joypad1Prev => &mut state.ports[0].previous,
            Self::Joypad1KeyrepeatCtr => &mut state.ports[0].repeat_counter,
            Self::Joypad2Last => &mut state.ports[1].sampled,
            Self::Joypad2NewKeys => &mut state.ports[1].pressed,
            Self::Joypad2Newkeys2 => &mut state.ports[1].repeated,
            Self::Joypad2Prev => &mut state.ports[1].previous,
            Self::Joypad2KeyrepeatCtr => &mut state.ports[1].repeat_counter,
            Self::JoypadCtrRepeatFirst => &mut state.repeat_first,
            Self::JoypadCtrRepeatNext => &mut state.repeat_next,
            Self::JoypadReleasedKeys => &mut state.released,
            Self::TimedHeldInputTimer => &mut state.held_timer,
            Self::TimedHeldInputTimerReset => &mut state.held_timer_reset,
            Self::TimedHeldInput => &mut state.timed_held,
            Self::PreviousTimedHeldInput => &mut state.previous_timed_held,
            Self::NewlyHeldDownTimedHeldInput => &mut state.newly_held,
            Self::Joypad1InputSamusfilter => &mut state.samus_held,
            Self::Joypad1NewinputSamusfilter => &mut state.samus_pressed,
            Self::BugFixCounter => &mut state.bug_fix_counter,
            Self::ButtonConfigShootX => &mut state.bindings.shoot,
            Self::ButtonConfigJumpA => &mut state.bindings.jump,
            Self::ButtonConfigRunB => &mut state.bindings.run,
            Self::ButtonConfigItemcancelY => &mut state.bindings.cancel,
            Self::ButtonConfigItemswitch => &mut state.bindings.switch,
            Self::ButtonConfigAimDownL => &mut state.bindings.aim_down,
            Self::ButtonConfigAimUpR => &mut state.bindings.aim_up,
        }
    }
    pub fn get_history(self, history: &crate::scene::start::InputHistory) -> &u16 {
        match self {
            Self::Joypad1Lastkeys => &history.ports[0].sampled,
            Self::Joypad1Newkeys => &history.ports[0].pressed,
            Self::Joypad1Newkeys2Unused => &history.ports[0].repeated,
            Self::Joypad1Prev => &history.ports[0].previous,
            Self::Joypad1KeyrepeatCtr => &history.ports[0].repeat_counter,
            Self::Joypad2Last => &history.ports[1].sampled,
            Self::Joypad2NewKeys => &history.ports[1].pressed,
            Self::Joypad2Newkeys2 => &history.ports[1].repeated,
            Self::Joypad2Prev => &history.ports[1].previous,
            Self::Joypad2KeyrepeatCtr => &history.ports[1].repeat_counter,
            Self::JoypadCtrRepeatFirst => &history.repeat_first,
            Self::JoypadCtrRepeatNext => &history.repeat_next,
            Self::JoypadReleasedKeys => &history.released,
            Self::TimedHeldInputTimer => &history.held_timer,
            Self::TimedHeldInputTimerReset => &history.held_timer_reset,
            Self::TimedHeldInput => &history.timed_held,
            Self::PreviousTimedHeldInput => &history.previous_timed_held,
            Self::NewlyHeldDownTimedHeldInput => &history.newly_held,
            Self::Joypad1InputSamusfilter => &history.samus_held,
            Self::Joypad1NewinputSamusfilter => &history.samus_pressed,
            Self::BugFixCounter => &history.bug_fix_counter,
            Self::ButtonConfigShootX => &history.bindings.shoot,
            Self::ButtonConfigJumpA => &history.bindings.jump,
            Self::ButtonConfigRunB => &history.bindings.run,
            Self::ButtonConfigItemcancelY => &history.bindings.cancel,
            Self::ButtonConfigItemswitch => &history.bindings.switch,
            Self::ButtonConfigAimDownL => &history.bindings.aim_down,
            Self::ButtonConfigAimUpR => &history.bindings.aim_up,
        }
    }
    pub fn get(self, control: &crate::session_owners::Control) -> &u16 {
        match control {
            crate::session_owners::Control::Legacy(value) => match self {
                Self::Joypad1Lastkeys => &value.input.ports[0].sampled,
                Self::Joypad1Newkeys => &value.input.ports[0].pressed,
                Self::Joypad1Newkeys2Unused => &value.input.ports[0].repeated,
                Self::Joypad1Prev => &value.input.ports[0].previous,
                Self::Joypad1KeyrepeatCtr => &value.input.ports[0].repeat_counter,
                Self::Joypad2Last => &value.input.ports[1].sampled,
                Self::Joypad2NewKeys => &value.input.ports[1].pressed,
                Self::Joypad2Newkeys2 => &value.input.ports[1].repeated,
                Self::Joypad2Prev => &value.input.ports[1].previous,
                Self::Joypad2KeyrepeatCtr => &value.input.ports[1].repeat_counter,
                Self::JoypadCtrRepeatFirst => &value.input.repeat_first,
                Self::JoypadCtrRepeatNext => &value.input.repeat_next,
                Self::JoypadReleasedKeys => &value.input.released,
                Self::TimedHeldInputTimer => &value.input.held_timer,
                Self::TimedHeldInputTimerReset => &value.input.held_timer_reset,
                Self::TimedHeldInput => &value.input.timed_held,
                Self::PreviousTimedHeldInput => &value.input.previous_timed_held,
                Self::NewlyHeldDownTimedHeldInput => &value.input.newly_held,
                Self::Joypad1InputSamusfilter => &value.input.samus_held,
                Self::Joypad1NewinputSamusfilter => &value.input.samus_pressed,
                Self::BugFixCounter => &value.input.bug_fix_counter,
                Self::ButtonConfigShootX => &value.input.bindings.shoot,
                Self::ButtonConfigJumpA => &value.input.bindings.jump,
                Self::ButtonConfigRunB => &value.input.bindings.run,
                Self::ButtonConfigItemcancelY => &value.input.bindings.cancel,
                Self::ButtonConfigItemswitch => &value.input.bindings.switch,
                Self::ButtonConfigAimDownL => &value.input.bindings.aim_down,
                Self::ButtonConfigAimUpR => &value.input.bindings.aim_up,
            },
            crate::session_owners::Control::Owned(value) => match self {
                Self::Joypad1Lastkeys => &value.metadata.input_history.ports[0].sampled,
                Self::Joypad1Newkeys => &value.metadata.input_history.ports[0].pressed,
                Self::Joypad1Newkeys2Unused => &value.metadata.input_history.ports[0].repeated,
                Self::Joypad1Prev => &value.metadata.input_history.ports[0].previous,
                Self::Joypad1KeyrepeatCtr => &value.metadata.input_history.ports[0].repeat_counter,
                Self::Joypad2Last => &value.metadata.input_history.ports[1].sampled,
                Self::Joypad2NewKeys => &value.metadata.input_history.ports[1].pressed,
                Self::Joypad2Newkeys2 => &value.metadata.input_history.ports[1].repeated,
                Self::Joypad2Prev => &value.metadata.input_history.ports[1].previous,
                Self::Joypad2KeyrepeatCtr => &value.metadata.input_history.ports[1].repeat_counter,
                Self::JoypadCtrRepeatFirst => &value.metadata.input_history.repeat_first,
                Self::JoypadCtrRepeatNext => &value.metadata.input_history.repeat_next,
                Self::JoypadReleasedKeys => &value.metadata.input_history.released,
                Self::TimedHeldInputTimer => &value.metadata.input_history.held_timer,
                Self::TimedHeldInputTimerReset => &value.metadata.input_history.held_timer_reset,
                Self::TimedHeldInput => &value.metadata.input_history.timed_held,
                Self::PreviousTimedHeldInput => &value.metadata.input_history.previous_timed_held,
                Self::NewlyHeldDownTimedHeldInput => &value.metadata.input_history.newly_held,
                Self::Joypad1InputSamusfilter => &value.metadata.input_history.samus_held,
                Self::Joypad1NewinputSamusfilter => &value.metadata.input_history.samus_pressed,
                Self::BugFixCounter => &value.metadata.input_history.bug_fix_counter,
                Self::ButtonConfigShootX => &value.metadata.input_history.bindings.shoot,
                Self::ButtonConfigJumpA => &value.metadata.input_history.bindings.jump,
                Self::ButtonConfigRunB => &value.metadata.input_history.bindings.run,
                Self::ButtonConfigItemcancelY => &value.metadata.input_history.bindings.cancel,
                Self::ButtonConfigItemswitch => &value.metadata.input_history.bindings.switch,
                Self::ButtonConfigAimDownL => &value.metadata.input_history.bindings.aim_down,
                Self::ButtonConfigAimUpR => &value.metadata.input_history.bindings.aim_up,
            },
        }
    }
    pub fn get_mut(self, control: &mut crate::session_owners::Control) -> &mut u16 {
        match control {
            crate::session_owners::Control::Legacy(value) => match self {
                Self::Joypad1Lastkeys => &mut value.input.ports[0].sampled,
                Self::Joypad1Newkeys => &mut value.input.ports[0].pressed,
                Self::Joypad1Newkeys2Unused => &mut value.input.ports[0].repeated,
                Self::Joypad1Prev => &mut value.input.ports[0].previous,
                Self::Joypad1KeyrepeatCtr => &mut value.input.ports[0].repeat_counter,
                Self::Joypad2Last => &mut value.input.ports[1].sampled,
                Self::Joypad2NewKeys => &mut value.input.ports[1].pressed,
                Self::Joypad2Newkeys2 => &mut value.input.ports[1].repeated,
                Self::Joypad2Prev => &mut value.input.ports[1].previous,
                Self::Joypad2KeyrepeatCtr => &mut value.input.ports[1].repeat_counter,
                Self::JoypadCtrRepeatFirst => &mut value.input.repeat_first,
                Self::JoypadCtrRepeatNext => &mut value.input.repeat_next,
                Self::JoypadReleasedKeys => &mut value.input.released,
                Self::TimedHeldInputTimer => &mut value.input.held_timer,
                Self::TimedHeldInputTimerReset => &mut value.input.held_timer_reset,
                Self::TimedHeldInput => &mut value.input.timed_held,
                Self::PreviousTimedHeldInput => &mut value.input.previous_timed_held,
                Self::NewlyHeldDownTimedHeldInput => &mut value.input.newly_held,
                Self::Joypad1InputSamusfilter => &mut value.input.samus_held,
                Self::Joypad1NewinputSamusfilter => &mut value.input.samus_pressed,
                Self::BugFixCounter => &mut value.input.bug_fix_counter,
                Self::ButtonConfigShootX => &mut value.input.bindings.shoot,
                Self::ButtonConfigJumpA => &mut value.input.bindings.jump,
                Self::ButtonConfigRunB => &mut value.input.bindings.run,
                Self::ButtonConfigItemcancelY => &mut value.input.bindings.cancel,
                Self::ButtonConfigItemswitch => &mut value.input.bindings.switch,
                Self::ButtonConfigAimDownL => &mut value.input.bindings.aim_down,
                Self::ButtonConfigAimUpR => &mut value.input.bindings.aim_up,
            },
            crate::session_owners::Control::Owned(value) => match self {
                Self::Joypad1Lastkeys => &mut value.metadata.input_history.ports[0].sampled,
                Self::Joypad1Newkeys => &mut value.metadata.input_history.ports[0].pressed,
                Self::Joypad1Newkeys2Unused => &mut value.metadata.input_history.ports[0].repeated,
                Self::Joypad1Prev => &mut value.metadata.input_history.ports[0].previous,
                Self::Joypad1KeyrepeatCtr => {
                    &mut value.metadata.input_history.ports[0].repeat_counter
                }
                Self::Joypad2Last => &mut value.metadata.input_history.ports[1].sampled,
                Self::Joypad2NewKeys => &mut value.metadata.input_history.ports[1].pressed,
                Self::Joypad2Newkeys2 => &mut value.metadata.input_history.ports[1].repeated,
                Self::Joypad2Prev => &mut value.metadata.input_history.ports[1].previous,
                Self::Joypad2KeyrepeatCtr => {
                    &mut value.metadata.input_history.ports[1].repeat_counter
                }
                Self::JoypadCtrRepeatFirst => &mut value.metadata.input_history.repeat_first,
                Self::JoypadCtrRepeatNext => &mut value.metadata.input_history.repeat_next,
                Self::JoypadReleasedKeys => &mut value.metadata.input_history.released,
                Self::TimedHeldInputTimer => &mut value.metadata.input_history.held_timer,
                Self::TimedHeldInputTimerReset => {
                    &mut value.metadata.input_history.held_timer_reset
                }
                Self::TimedHeldInput => &mut value.metadata.input_history.timed_held,
                Self::PreviousTimedHeldInput => {
                    &mut value.metadata.input_history.previous_timed_held
                }
                Self::NewlyHeldDownTimedHeldInput => &mut value.metadata.input_history.newly_held,
                Self::Joypad1InputSamusfilter => &mut value.metadata.input_history.samus_held,
                Self::Joypad1NewinputSamusfilter => &mut value.metadata.input_history.samus_pressed,
                Self::BugFixCounter => &mut value.metadata.input_history.bug_fix_counter,
                Self::ButtonConfigShootX => &mut value.metadata.input_history.bindings.shoot,
                Self::ButtonConfigJumpA => &mut value.metadata.input_history.bindings.jump,
                Self::ButtonConfigRunB => &mut value.metadata.input_history.bindings.run,
                Self::ButtonConfigItemcancelY => &mut value.metadata.input_history.bindings.cancel,
                Self::ButtonConfigItemswitch => &mut value.metadata.input_history.bindings.switch,
                Self::ButtonConfigAimDownL => &mut value.metadata.input_history.bindings.aim_down,
                Self::ButtonConfigAimUpR => &mut value.metadata.input_history.bindings.aim_up,
            },
        }
    }
}

pub fn pressed(sampled: u16, previous: u16) -> u16 {
    sampled & (previous ^ sampled)
}
