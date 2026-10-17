//! Ordinary return-route authoring through observed east-exit and first-ledge gates.
//! Gameplay remains native; this policy only emits mapped controller inputs.
use sha2::{Digest, Sha256};
use sm_native_host::State;

pub const PREFIX: u64 = 24832;
const PREFIX_WRAM: &str = "cb3362278173187dcda825bea8c4ec5c06952c1db7a5a948affb1c8d96a97483";

#[derive(Default)]
pub struct Return {
    previous: Option<State>,
    prefix_verified: bool,
    reason: Option<&'static str>,
    exit_step: Option<u64>,
    stationary_polls: u64,
    beam_step: Option<u64>,
    climb_phase: u8,
    climb_polls: u64,
    feet_cleared: bool,
    landing_step: Option<u64>,
    ceiling_beam_step: Option<u64>,
    ceiling_polls: u64,
    ceiling_response_step: Option<u64>,
    upper_landing_step: Option<u64>,
    upper_anchor: Option<u16>,
    upper_polls: u64,
    upper_beam_step: Option<u64>,
    upper_response_step: Option<u64>,
    row10_polls: u64,
    row10_beam_step: Option<u64>,
    row10_response_step: Option<u64>,
    row8_polls: u64,
    row8_beam_step: Option<u64>,
    row8_response_step: Option<u64>,
    survey_polls: u64,
    survey_jump_step: Option<u64>,
    survey_window_step: Option<u64>,
    survey_launch_pose: Option<u16>,
    aerial_polls: u64,
    aerial_previous: Option<(u16, u16)>,
    aerial_creation: Option<(u16, u16)>,
    aerial_beam_step: Option<u64>,
    aerial_response_step: Option<u64>,
    transfer_polls: u64,
    transfer_release_step: Option<u64>,
    transfer_contact: Option<TransferContact>,
    support_polls: u64,
    support_release_step: Option<u64>,
    support_standing_step: Option<u64>,
    support_span: Option<TransferContact>,
    brake_polls: u64,
    brake_release_step: Option<u64>,
    brake_rest: Option<(u64, u16, u16)>,
    brake_wall_step: Option<u64>,
    brake_aim_step: Option<u64>,
    third_beam_polls: u64,
    third_aim_step: Option<u64>,
    third_beam_step: Option<u64>,
    third_response_step: Option<u64>,
    third_neighbors: Option<[u16; 3]>,
    third_after: Option<[u16; 3]>,
    third_plm_slot: Option<u16>,
    third_ready: bool,
    third_before_bts: Option<[u8; 3]>,
    third_after_bts: Option<[u8; 3]>,
    third_plm: Option<[u16; 4]>,
    upper_transfer_polls: u64,
    upper_release_step: Option<u64>,
    upper_jump_step: Option<u64>,
    upper_feet_step: Option<u64>,
    upper_contact: Option<TransferContact>,
    upper_hold: Option<(u64, u16, u16)>,
    upper_hold_motion: Option<[u16; 6]>,
    upper_left_release_step: Option<u64>,
    upper_rest_step: Option<u64>,
    upper_launch_ready: bool,
    door: DoorReaction,
}

#[derive(Default)]
struct DoorReaction {
    polls: u64,
    ready: bool,
    shot_step: Option<u64>,
    release_step: Option<u64>,
    response_step: Option<u64>,
    before: Option<[u16; 4]>,
    before_bts: Option<[u8; 4]>,
    allocation: Option<(u16, u16)>,
    after: Option<[u16; 4]>,
    after_bts: Option<[u8; 4]>,
    plm: Option<[u16; 5]>,
    projectile: Option<[u16; 4]>,
    opening_stage: usize,
    opening_polls: u64,
    air_step: Option<u64>,
    air_tiles: Option<[u16; 4]>,
    air_bts: Option<[u8; 4]>,
    air_timer: Option<u16>,
    air_plm: Option<[u16; 5]>,
    approach_polls: u64,
    approach_left_step: Option<u64>,
    approach_fall: Option<[u16; 5]>,
    approach_fall_step: Option<u64>,
    approach_motion: Option<[u16; 8]>,
    traversal_polls: u64,
    traversal_landing: Option<[u16; 4]>,
    traversal_landing_step: Option<u64>,
    exit_polls: u64,
    transition_step: Option<u64>,
    transition_door: Option<[u16; 2]>,
    loading_polls: u64,
    loading_stage: usize,
    loading_stages: u16,
    arrival_step: Option<u64>,
    arrival: Option<[u16; 4]>,
    departure_polls: u64,
    departure_fall_step: Option<u64>,
    departure_fall: Option<[u16; 4]>,
    departure_motion: Option<[u16; 8]>,
    lower_floor_polls: u64,
    lower_floor_step: Option<u64>,
    lower_floor: Option<[u16; 4]>,
    lower_floor_motion: Option<[u16; 8]>,
    platform_polls: u64,
    platform_step: Option<u64>,
    platform: Option<[u16; 4]>,
    platform_motion: Option<[u16; 8]>,
    platform_collision: Option<u16>,
    boarding_polls: u64,
    boarding_aim_step: Option<u64>,
    boarding_release_step: Option<u64>,
    boarding_jump_step: Option<u64>,
    boarding_jump: Option<[u16; 7]>,
    boarding_clear_step: Option<u64>,
    boarding_clear: Option<[u16; 7]>,
    crossing_polls: u64,
    crossing_left_step: Option<u64>,
    crossing_left: Option<[u16; 10]>,
    crossing_ceiling_step: Option<u64>,
    crossing_ceiling: Option<[u16; 10]>,
    crossing_ceiling_motion: Option<[u16; 3]>,
    crossing_y: Option<u32>,
    crossing_span_step: Option<u64>,
    crossing_span: Option<[u16; 10]>,
    crossing_span_motion: Option<[u16; 3]>,
    descent_polls: u64,
    descent_y: Option<u32>,
    descent_release_step: Option<u64>,
    descent_release: Option<[u16; 13]>,
    descent_landing_step: Option<u64>,
    descent_landing: Option<[u16; 13]>,
}

struct TransferContact {
    step: u64,
    pose: u16,
    radius: u16,
    x: u16,
    speed: u16,
    subspeed: u16,
    accel_mode: u16,
    speed_table: u16,
    x_subposition: u16,
    extra_speed: u16,
    extra_subspeed: u16,
}

fn word(ram: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([ram[at], ram[at + 1]])
}

impl Return {
    pub fn observe(&mut self, state: &State, ram: &[u8]) {
        if self.reason.is_some() {
            return;
        }
        if ram.len() != 131072 {
            self.reason = Some("incomplete return WRAM");
            return;
        }
        if self.previous.is_none() {
            self.prefix_verified = state.host_steps == PREFIX
                && state.game_state == 8
                && state.area == 1
                && state.room == 0xa107
                && format!("{:x}", Sha256::digest(ram)) == PREFIX_WRAM;
            if !self.prefix_verified {
                self.reason = Some("return does not start at exact missile prefix");
                return;
            }
        } else if let Some(previous) = self.previous {
            if state.host_steps != previous.host_steps + 1 {
                self.reason = Some("gapped return observation");
                return;
            }
            if state.polls > previous.polls && state.game_state == 8 && previous.game_state == 8 {
                self.stationary_polls =
                    if (state.x, state.y) == (previous.x, previous.y) { self.stationary_polls + 1 } else { 0 };
            }
        }
        if word(ram, 0x9c2) == 0 || word(ram, 0x9c6) != 5 || word(ram, 0x9c8) != 5 {
            self.reason = Some("return lost health or preserved missiles");
            return;
        }
        if state.cpu_fallback_calls != 0 || state.cpu_instructions != 0 {
            self.reason = Some("return CPU fallback");
            return;
        }
        if self.climb_phase == 46 {
            self.observe_loading(state, ram);
            self.previous = Some(*state);
            return;
        }
        if matches!(self.climb_phase, 47 | 48) {
            self.observe_departure(state, ram);
            self.previous = Some(*state);
            return;
        }
        if matches!(self.climb_phase, 49..=53) {
            self.observe_platform(state, ram);
            self.previous = Some(*state);
            return;
        }
        if self.climb_phase == 54 {
            self.observe_crossing(state, ram);
            self.previous = Some(*state);
            return;
        }
        if self.climb_phase == 55 {
            self.observe_platform_descent(state, ram);
            self.previous = Some(*state);
            return;
        }
        if matches!(self.climb_phase, 44 | 45) && state.game_state == 9 {
            let exiting = self.climb_phase == 45;
            let consumed = self.previous.is_some_and(|previous| {
                state.polls == previous.polls + 1
                    && previous.game_state == 8
                    && previous.area == 1
                    && previous.room == 0x9f11
                    && previous.effective_buttons == 0x40
                    && previous.joy1_current == 0x0200
                    && previous.joy1_new & !0x0200 == 0
            });
            if !consumed
                || state.room != 0x9f11
                || state.area != 1
                || state.effective_buttons != 0x40
                || state.joy1_current != 0x0200
                || state.joy1_new & !0x0200 != 0
                || word(ram, 0x78d) != 0x8ec2
                || word(ram, 0x7b5) != 0x9f57
                || word(ram, 0x99c) != 0xe17d
                || word(ram, 0xe16) != 0
                || self.door.approach_fall_step.is_none()
                || (exiting && self.door.traversal_landing_step.is_none())
                || (if exiting { self.door.exit_polls } else { self.door.traversal_polls }) + 1 >= 120
            {
                self.reason = Some("return upper-left transition lost its owned door event");
                return;
            }
            self.door.transition_step = Some(state.host_steps);
            if exiting {
                self.door.exit_polls += 1;
            } else {
                self.door.traversal_polls += 1;
            }
            self.door.transition_door = Some([word(ram, 0x78d), word(ram, 0x7b5)]);
            self.climb_phase = 46;
            self.door.loading_stages = 1;
            self.previous = Some(*state);
            return;
        }
        if matches!(state.game_state, 0x19 | 0x1a | 0x23 | 0x24 | 0x25) {
            self.reason = Some("return entered a gameplay failure state");
            return;
        }
        if state.game_state == 8 {
            if state.area != 1 || !matches!(state.room, 0xa107 | 0x9f11) {
                self.reason = Some("return exited the admitted room sequence");
                return;
            } else if state.room == 0x9f11 {
                if word(ram, 0x7a5) != 16 || word(ram, 0x7a7) != 32 {
                    self.reason = Some("first return ledge has unsupported room geometry");
                    return;
                }
                if self.exit_step.is_none() {
                    self.exit_step = Some(state.host_steps);
                    self.stationary_polls = 0;
                }
                if self.previous.is_some_and(|previous| state.polls == previous.polls + 1) {
                    self.climb_polls += 1;
                }
                if self.climb_polls >= 1024 {
                    self.reason = Some("first return ledge exceeded 1024 consumed polls");
                    return;
                }
                let ball = matches!(ram[0xa1f], 4 | 8 | 0x11..=0x13);
                if self.landing_step.is_some() && self.upper_landing_step.is_none() {
                    if word(ram, 0x9d2) != 0 {
                        self.reason = Some("return ceiling beam has selected ammunition");
                        return;
                    }
                    if self.ceiling_polls
                        + u64::from(self.previous.is_some_and(|previous| state.polls == previous.polls + 1))
                        >= 120
                    {
                        self.reason = Some("return ceiling beam exceeded 120 consumed polls");
                        return;
                    }
                }
                let previous_phase = self.climb_phase;
                if self.climb_phase == 8
                    && matches!(ram[0xa1f], 0 | 1 | 0x0e)
                    && grounded(ram)
                    && word(ram, 0xb00) == 21
                    && state.y == 235
                    && upper_ledge_support(state, ram)
                {
                    // Ground turns can select a new spin jump from held Jump.
                    // Keep it released throughout settling, including unpolled steps.
                    self.climb_phase = 9;
                }
                match self.climb_phase {
                    0 if state.x >= 144 && ball => self.climb_phase = 1,
                    1 if ram[0xa1f] == 0
                        && word(ram, 0xb00) == 21
                        && state.x.saturating_sub(word(ram, 0xafe)) > 128
                        && grounded(ram) =>
                    {
                        self.climb_phase = 2;
                    }
                    2 if ram[0xa1f] == 2
                        && word(ram, 0xb00) == 19
                        && state.y.saturating_add(word(ram, 0xb00)) <= 336 =>
                    {
                        self.feet_cleared = true;
                        self.climb_phase = 3;
                    }
                    3 if !ball
                        && grounded(ram)
                        && word(ram, 0xb00) == 21
                        && state.y == 315
                        && state.x == 112
                        && first_ledge_support(state, ram) =>
                    {
                        self.landing_step = Some(state.host_steps);
                        self.climb_phase = 4;
                    }
                    4 if word(ram, 0xa1c) == 4
                        && state.joy1_current & 0x0800 != 0
                        && state.joy1_current & word(ram, 0x9b4) == 0 =>
                    {
                        self.climb_phase = 5
                    }
                    7 if ram[0xa1f] == 2 && word(ram, 0xb00) == 19 && state.y.saturating_add(19) <= 256 => {
                        self.climb_phase = 8;
                    }
                    8 | 9
                        if ram[0xa1f] == 0
                            && grounded(ram)
                            && word(ram, 0xb00) == 21
                            && state.y == 235
                            && (140..149).contains(&state.x)
                            && upper_ledge_support(state, ram) =>
                    {
                        self.upper_landing_step = Some(state.host_steps);
                        self.climb_phase = 10;
                    }
                    _ => {}
                }
                if self.upper_landing_step.is_some() {
                    self.observe_upper_beam(state, ram, previous_phase);
                    self.previous = Some(*state);
                    return;
                }
                if self.landing_step.is_some() {
                    let shoot_key = word(ram, 0x9b2);
                    if self.climb_phase == 5
                        && previous_phase == 5
                        && self.previous.is_some_and(|previous| state.polls == previous.polls + 1)
                        && state.effective_buttons & shoot_key.reverse_bits() != 0
                        && state.joy1_current & shoot_key != 0
                        && state.joy1_new & shoot_key != 0
                    {
                        self.ceiling_beam_step = Some(state.host_steps);
                        self.climb_phase = 6;
                    }
                    if self.climb_phase < 6
                        && (word(ram, 0x10002 + 2 * (16 * 16 + 6)) != 0xc19a || ram[0x16402 + 16 * 16 + 6] != 0)
                    {
                        self.reason = Some("return ceiling beam target differs before the shot");
                        return;
                    }
                    if self.climb_phase <= 6
                        && (state.x != 112
                            || state.y != 315
                            || ram[0xa1f] != 0
                            || word(ram, 0xb00) != 21
                            || !grounded(ram)
                            || !first_ledge_support(state, ram))
                    {
                        self.reason = Some("return ceiling beam lost the admitted landing support");
                        return;
                    }
                    if self.previous.is_some_and(|previous| state.polls == previous.polls + 1) {
                        self.ceiling_polls += 1;
                    }
                    if word(ram, 0x9d2) != 0 {
                        self.reason = Some("return ceiling beam has selected ammunition");
                        return;
                    }
                    if self.climb_phase == 6 && word(ram, 0x10002 + 2 * (16 * 16 + 6)) != 0xc19a {
                        self.ceiling_response_step = Some(state.host_steps);
                        if [6, 7].into_iter().all(|column| word(ram, 0x10002 + 2 * (16 * 16 + column)) >> 12 == 0) {
                            self.climb_phase = 7;
                        } else {
                            self.reason = Some("return ceiling beam response has no full jump opening");
                            return;
                        }
                    }
                    if (self.climb_phase == 7 || previous_phase == 7)
                        && ![6, 7].into_iter().all(|column| word(ram, 0x10002 + 2 * (16 * 16 + column)) >> 12 == 0)
                    {
                        self.reason = Some("return second ledge opening closed before feet clearance");
                        return;
                    }
                    if self.ceiling_polls >= 120 {
                        self.reason = Some("return ceiling beam exceeded 120 consumed polls");
                        return;
                    }
                }
            }
        }
        if self.stationary_polls >= 120 && state.room == 0xa107 {
            self.reason = Some("missile east exit stationary for 120 consumed polls");
        }
        let shoot_key = word(ram, 0x9b2);
        if self.previous.is_some_and(|previous| state.polls == previous.polls + 1)
            && state.effective_buttons & shoot_key.reverse_bits() != 0
            && state.joy1_new & shoot_key != 0
            && state.joy1_current & shoot_key != 0
            && state.host_steps > PREFIX
        {
            self.beam_step.get_or_insert(state.host_steps);
        }
        self.previous = Some(*state);
    }

    fn observe_upper_beam(&mut self, state: &State, ram: &[u8], previous_phase: u8) {
        if self.upper_response_step.is_some() {
            self.observe_upper_shot_chain(state, ram, previous_phase);
            return;
        }
        let consumed = self.upper_landing_step.is_some_and(|landing| state.host_steps > landing)
            && self.previous.is_some_and(|previous| state.polls == previous.polls + 1);
        if self.upper_polls + u64::from(consumed) >= 120 {
            self.reason = Some("return upper ceiling beam exceeded 120 consumed polls");
            return;
        }
        self.upper_polls += u64::from(consumed);
        if word(ram, 0x9d2) != 0
            || ram[0xa1f] != 0
            || word(ram, 0xb00) != 21
            || !grounded(ram)
            || state.y != 235
            || !(140..149).contains(&state.x)
            || !upper_ledge_support(state, ram)
            || self.upper_anchor.is_some_and(|anchor| state.x != anchor)
        {
            self.reason = Some("return upper ceiling beam lost the admitted landing support");
            return;
        }
        if self.climb_phase == 10 {
            if state.joy1_current & (0x0300 | word(ram, 0x9b4)) == 0
                && [0xb42, 0xb44, 0xb46, 0xb48].into_iter().all(|at| word(ram, at) == 0)
                && word(ram, 0xa1c) == 2
            {
                self.upper_anchor = Some(state.x);
                self.climb_phase = 11;
            }
            return;
        }
        let Some(anchor) = self.upper_anchor else {
            self.reason = Some("return upper ceiling beam has no resting anchor");
            return;
        };
        let column = usize::from((anchor - 2) / 16);
        let block = 12 * 16 + column;
        let shoot = word(ram, 0x9b2);
        if self.climb_phase == 11
            && word(ram, 0xa1c) == 4
            && state.joy1_current & 0x0800 != 0
            && state.joy1_current & word(ram, 0x9b4) == 0
        {
            self.climb_phase = 12;
        }
        if self.climb_phase == 12
            && previous_phase == 12
            && consumed
            && word(ram, 0xa1c) == 4
            && state.joy1_current & 0x0800 != 0
            && state.joy1_current & word(ram, 0x9b4) == 0
            && state.effective_buttons & shoot.reverse_bits() != 0
            && state.joy1_current & shoot != 0
            && state.joy1_new & shoot != 0
        {
            self.upper_beam_step = Some(state.host_steps);
            self.climb_phase = 13;
        }
        let tile = word(ram, 0x10002 + 2 * block);
        if self.upper_beam_step.is_none() && (tile != 0xc19a || ram[0x16402 + block] != 0) {
            self.reason = Some("return upper ceiling beam target differs before the shot");
        } else if self.climb_phase == 13 && tile != 0xc19a {
            self.upper_response_step = Some(state.host_steps);
            self.climb_phase = 14;
        }
    }

    fn observe_upper_shot_chain(&mut self, state: &State, ram: &[u8], previous_phase: u8) {
        if self.row8_response_step.is_some() {
            self.observe_airborne_survey(state, ram, previous_phase);
            return;
        }
        let row8 = self.row10_response_step.is_some();
        let response = if row8 { self.row10_response_step } else { self.upper_response_step };
        let polls = if row8 { &mut self.row8_polls } else { &mut self.row10_polls };
        let consumed = response.is_some_and(|response| state.host_steps > response)
            && self.previous.is_some_and(|previous| state.polls == previous.polls + 1);
        if *polls + u64::from(consumed) >= 120 {
            self.reason = Some(if row8 {
                "return row8 beam exceeded 120 consumed polls"
            } else {
                "return row10 beam exceeded 120 consumed polls"
            });
            return;
        }
        *polls += u64::from(consumed);
        let Some(anchor) = self.upper_anchor else {
            self.reason = Some(if row8 {
                "return row8 beam has no resting anchor"
            } else {
                "return row10 beam has no resting anchor"
            });
            return;
        };
        if state.x != anchor
            || state.y != 235
            || ram[0xa1f] != 0
            || word(ram, 0xb00) != 21
            || !grounded(ram)
            || !upper_ledge_support(state, ram)
            || word(ram, 0x9d2) != 0
            || word(ram, 0xa1c) != 4
            || state.joy1_current & 0x0800 == 0
            || state.joy1_current & (word(ram, 0x9b4) | 0x0300) != 0
            || ![0xb42, 0xb44, 0xb46, 0xb48].into_iter().all(|at| word(ram, at) == 0)
            || ![8, 9].into_iter().all(|column| word(ram, 0x10002 + 2 * (12 * 16 + column)) >> 12 == 0)
            || (row8 && ![8, 9].into_iter().all(|column| word(ram, 0x10002 + 2 * (10 * 16 + column)) >> 12 == 0))
        {
            self.reason = Some(if row8 {
                "return row8 beam lost its aiming support or opening"
            } else {
                "return row10 beam lost its aiming support or opening"
            });
            return;
        }
        let block = (if row8 { 8 } else { 10 }) * 16 + usize::from((anchor - 2) / 16);
        let shoot = word(ram, 0x9b2);
        if ram[0x16402 + block] != if row8 { 0 } else { 4 } {
            self.reason = Some(if row8 {
                "return row8 beam target has unsupported BTS"
            } else {
                "return row10 beam target has unsupported BTS"
            });
            return;
        }
        let waiting_phase = if row8 { 17 } else { 14 };
        let shot_phase = waiting_phase + 1;
        let response_phase = shot_phase + 1;
        if self.climb_phase == waiting_phase
            && word(ram, 0xccc) == 0
            && word(ram, 0xcce) < 5
            && state.joy1_current & shoot == 0
        {
            self.climb_phase = shot_phase;
        }
        if self.climb_phase == shot_phase
            && previous_phase == shot_phase
            && consumed
            && state.effective_buttons & shoot.reverse_bits() != 0
            && state.joy1_current & shoot != 0
            && state.joy1_new & shoot != 0
        {
            if row8 {
                self.row8_beam_step = Some(state.host_steps);
            } else {
                self.row10_beam_step = Some(state.host_steps);
            }
            self.climb_phase = response_phase;
        }
        let tile = word(ram, 0x10002 + 2 * block);
        let beam = if row8 { self.row8_beam_step } else { self.row10_beam_step };
        if beam.is_none() && tile != 0xc19a {
            self.reason = Some(if row8 {
                "return row8 beam target differs before the shot"
            } else {
                "return row10 beam target differs before the shot"
            });
        } else if self.climb_phase == response_phase && tile != 0xc19a {
            if row8 {
                self.row8_response_step = Some(state.host_steps);
                self.climb_phase = 20;
            } else {
                self.row10_response_step = Some(state.host_steps);
                self.climb_phase = 17;
            }
        }
    }

    fn observe_airborne_survey(&mut self, state: &State, ram: &[u8], previous_phase: u8) {
        if self.transfer_contact.is_some() {
            self.observe_full_support(state, ram);
            return;
        }
        if self.aerial_response_step.is_some() {
            self.observe_transfer_contact(state, ram);
            return;
        }
        if self.survey_window_step.is_some() {
            self.observe_aerial_beam(state, ram, previous_phase);
            return;
        }
        let consumed = self.row8_response_step.is_some_and(|response| state.host_steps > response)
            && self.previous.is_some_and(|previous| state.polls == previous.polls + 1);
        if self.survey_polls + u64::from(consumed) >= 120 {
            self.reason = Some("return airborne pose survey exceeded 120 consumed polls");
            return;
        }
        self.survey_polls += u64::from(consumed);
        let jump = word(ram, 0x9b4);
        let shoot = word(ram, 0x9b2);
        if state.x != 148
            || word(ram, 0x9d2) != 0
            || ![0xb42, 0xb44, 0xb46, 0xb48].into_iter().all(|at| word(ram, at) == 0)
            || ![8, 10, 12]
                .into_iter()
                .all(|row| [8, 9].into_iter().all(|column| word(ram, 0x10002 + 2 * (row * 16 + column)) >> 12 == 0))
            || (self.climb_phase >= 21 && state.joy1_current & (shoot | 0x0b00) != 0)
        {
            self.reason = Some("return airborne pose survey lost its position or opening");
            return;
        }
        if self.climb_phase == 20 {
            if state.y != 235
                || ram[0xa1f] != 0
                || word(ram, 0xb00) != 21
                || !grounded(ram)
                || !upper_ledge_support(state, ram)
            {
                self.reason = Some("return airborne pose survey lost its launching support");
                return;
            }
            if word(ram, 0xa1c) == 2 && state.joy1_current & (jump | shoot | 0x0b00) == 0 {
                self.climb_phase = 21;
            }
        }
        let jump_consumed = self.climb_phase == 21
            && previous_phase == 21
            && consumed
            && self.survey_launch_pose == Some(2)
            && state.effective_buttons & jump.reverse_bits() != 0
            && state.joy1_current & jump != 0
            && state.joy1_new & jump != 0;
        if self.climb_phase == 21
            && !jump_consumed
            && (word(ram, 0xa1c) != 2
                || state.y != 235
                || ram[0xa1f] != 0
                || word(ram, 0xb00) != 21
                || !grounded(ram)
                || !upper_ledge_support(state, ram))
        {
            self.reason = Some("return airborne pose survey changed before consumed jump");
            return;
        }
        if jump_consumed {
            self.survey_jump_step = Some(state.host_steps);
            self.climb_phase = 22;
        }
        if self.climb_phase == 21 {
            self.survey_launch_pose = Some(word(ram, 0xa1c));
        }
        if self.climb_phase == 22
            && matches!(word(ram, 0xa1c), 0x14 | 0x4e | 0x52)
            && ram[0xa1f] == 2
            && word(ram, 0xb00) == 19
            && word(ram, 0xb36) == 1
            && state.joy1_current & jump != 0
            && word(ram, 0xb5e) == 0
            && (167..176).contains(&state.y)
        {
            self.survey_window_step = Some(state.host_steps);
            if !aerial_ready(state, ram) {
                self.reason = Some("return aerial beam lost its pre-shot window");
                return;
            }
            self.aerial_previous = Some((word(ram, 0xa1c), state.y));
            self.climb_phase = 23;
        }
    }

    fn observe_aerial_beam(&mut self, state: &State, ram: &[u8], previous_phase: u8) {
        let consumed = self.survey_window_step.is_some_and(|window| state.host_steps > window)
            && self.previous.is_some_and(|previous| state.polls == previous.polls + 1);
        if self.aerial_polls + u64::from(consumed) >= 120 {
            self.reason = Some("return aerial beam exceeded 120 consumed polls");
            return;
        }
        self.aerial_polls += u64::from(consumed);
        let jump = word(ram, 0x9b4);
        let shoot = word(ram, 0x9b2);
        let target = 10 * 16 + 7;
        let shot_consumed = self.climb_phase == 23
            && previous_phase == 23
            && consumed
            && self
                .aerial_previous
                .is_some_and(|(pose, y)| matches!(pose, 0x14 | 0x4e | 0x52) && (167..176).contains(&y))
            && state.effective_buttons & shoot.reverse_bits() != 0
            && state.joy1_current & shoot != 0
            && state.joy1_new & shoot != 0;
        // Beta records direction7 after Alfa fires; only the owned edge may
        // introduce this marker. The preceding creation state requires zero.
        let direction = word(ram, 0xb5e);
        let direction_valid =
            direction == 0 || (direction == 0x8007 && (shot_consumed || self.aerial_beam_step.is_some()));
        if state.x != 148
            || ram[0xa1f] != 2
            || word(ram, 0xb00) != 19
            || !matches!(word(ram, 0xa1c), 0x14 | 0x4e | 0x52)
            || word(ram, 0xb36) != 1
            || !direction_valid
            || state.joy1_current & jump == 0
            || state.joy1_current & 0x0b00 != 0
            || word(ram, 0x9d2) != 0
            || ram[0x16402 + target] != 4
            || ![0xb42, 0xb44, 0xb46, 0xb48].into_iter().all(|at| word(ram, at) == 0)
            || ![8, 10, 12]
                .into_iter()
                .all(|row| [8, 9].into_iter().all(|column| word(ram, 0x10002 + 2 * (row * 16 + column)) >> 12 == 0))
        {
            self.reason = Some("return aerial beam lost its pose, opening or target BTS");
            return;
        }
        if shot_consumed {
            self.aerial_creation = self.aerial_previous;
            self.aerial_beam_step = Some(state.host_steps);
            self.climb_phase = 24;
        }
        let tile = word(ram, 0x10002 + 2 * target);
        if self.aerial_beam_step.is_none() && tile != 0xc19a {
            self.reason = Some("return aerial beam target changed before the shot");
            return;
        }
        if self.climb_phase == 24 && tile != 0xc19a {
            self.aerial_response_step = Some(state.host_steps);
            if tile >> 12 != 0 || state.y.saturating_add(word(ram, 0xb00)) > 192 {
                self.reason = Some("return transfer has no open target or cleared feet");
                return;
            }
            self.climb_phase = 25;
            return;
        }
        if self.climb_phase == 23 {
            if !aerial_ready(state, ram) {
                self.reason = Some("return aerial beam lost its pre-shot window");
                return;
            }
            self.aerial_previous = Some((word(ram, 0xa1c), state.y));
        }
    }

    pub fn failure(&self) -> Option<&'static str> {
        self.reason
    }

    fn observe_transfer_contact(&mut self, state: &State, ram: &[u8]) {
        let consumed = self.aerial_response_step.is_some_and(|response| state.host_steps > response)
            && self.previous.is_some_and(|previous| state.polls == previous.polls + 1);
        if self.transfer_polls + u64::from(consumed) >= 120 {
            self.reason = Some("return transfer exceeded 120 consumed polls");
            return;
        }
        self.transfer_polls += u64::from(consumed);
        let jump = word(ram, 0x9b4);
        let shoot = word(ram, 0x9b2);
        let radius = word(ram, 0xb00);
        let movement = ram[0xa1f];
        // Alfa sets radius before Beta selects the A5 landing pose. On that
        // first contact, A5/movement0 still carries the prior airborne radius.
        let landing_transition = word(ram, 0xa1c) == 0xa5 && movement == 0 && radius == 19;
        if !(117..149).contains(&state.x)
            || ram[0xa1e] != 4
            || !(matches!((movement, radius), (2 | 6, 19) | (0 | 1 | 0x0e, 21)) || landing_transition)
            || state.y.saturating_add(radius) > 192
            || !matches!(word(ram, 0xb5e), 0 | 0x8007)
            || word(ram, 0x9d2) != 0
            || state.joy1_current & 0x0900 != 0
            || (consumed && state.joy1_current & shoot != 0)
            || (consumed && state.joy1_new & shoot != 0)
            || (consumed && state.effective_buttons & (0x90 | shoot.reverse_bits()) != 0)
            || state.joy1_current & jump == 0
            || word(ram, 0x10002 + 2 * (10 * 16 + 7)) >> 12 != 0
            || ram[0x16402 + 10 * 16 + 7] != 4
            || word(ram, 0x10002 + 2 * (12 * 16 + 7)) != 0xc19a
            || ![8, 10, 12]
                .into_iter()
                .all(|row| [8, 9].into_iter().all(|column| word(ram, 0x10002 + 2 * (row * 16 + column)) >> 12 == 0))
        {
            self.reason = Some("return transfer lost its ordinary motion, clearance or support");
            return;
        }
        if consumed
            && state.effective_buttons & shoot.reverse_bits() == 0
            && (state.joy1_current | state.joy1_new) & shoot == 0
        {
            self.transfer_release_step.get_or_insert(state.host_steps);
        }
        if consumed
            && state.y == 171
            && matches!(movement, 0 | 1 | 0x0e)
            && grounded(ram)
            && word(ram, 0xafe) == 5
            && (state.x - 5) / 16 == 7
            && self.transfer_release_step.is_some()
            && state.joy1_current & 0x0200 != 0
            && state.effective_buttons & (0x40 | jump.reverse_bits()) == (0x40 | jump.reverse_bits())
        {
            // Supporting contact does not prove full-span landing or rest.
            self.transfer_contact = Some(TransferContact {
                step: state.host_steps,
                pose: word(ram, 0xa1c),
                radius,
                x: state.x,
                speed: word(ram, 0xb46),
                subspeed: word(ram, 0xb48),
                accel_mode: word(ram, 0xb4a),
                speed_table: word(ram, 0xa6c),
                x_subposition: word(ram, 0xaf8),
                extra_speed: word(ram, 0xb42),
                extra_subspeed: word(ram, 0xb44),
            });
            self.climb_phase = 26;
        }
    }

    fn observe_full_support(&mut self, state: &State, ram: &[u8]) {
        if self.support_span.is_some() {
            self.observe_brake(state, ram);
            return;
        }
        let consumed = self.transfer_contact.as_ref().is_some_and(|contact| state.host_steps > contact.step)
            && self.previous.is_some_and(|previous| state.polls == previous.polls + 1);
        if self.support_polls + u64::from(consumed) >= 120 {
            self.reason = Some("return full support exceeded 120 consumed polls");
            return;
        }
        self.support_polls += u64::from(consumed);
        let jump = word(ram, 0x9b4);
        let shoot = word(ram, 0x9b2);
        let released = state.joy1_current == 0 && state.joy1_new == 0 && state.effective_buttons == 0;
        let pose = word(ram, 0xa1c);
        let movement = ram[0xa1f];
        let radius = word(ram, 0xb00);
        if !(117..129).contains(&state.x)
            || state.y != 171
            || ram[0xa1e] != 4
            || word(ram, 0xafe) != 5
            || !grounded(ram)
            || !(matches!((movement, radius), (0 | 1 | 0x0e, 21)) || (pose == 0xa5 && movement == 0 && radius == 19))
            || word(ram, 0x9d2) != 0
            || word(ram, 0xb5e) != 0
            || (consumed && (state.joy1_current | state.joy1_new) & (jump | shoot | 0x0900) != 0)
            || (consumed && state.effective_buttons & (jump.reverse_bits() | shoot.reverse_bits() | 0x90) != 0)
            || word(ram, 0x10002 + 2 * (10 * 16 + 7)) >> 12 != 0
            || ram[0x16402 + 10 * 16 + 7] != 4
            || word(ram, 0x10002 + 2 * (12 * 16 + 7)) != 0xc19a
            || (state.x - 5) / 16 != 7
            || ![8, 10, 12]
                .into_iter()
                .all(|row| [8, 9].into_iter().all(|column| word(ram, 0x10002 + 2 * (row * 16 + column)) >> 12 == 0))
        {
            self.reason = Some("return full support lost its released ground motion or terrain");
            return;
        }
        if self.climb_phase == 26 {
            if consumed && released {
                self.support_release_step.get_or_insert(state.host_steps);
            }
            if consumed
                && self.support_release_step.is_some()
                && pose == 2
                && movement == 0
                && radius == 21
                && self
                    .transfer_contact
                    .as_ref()
                    .is_some_and(|contact| state.x == contact.x && word(ram, 0xaf8) == contact.x_subposition)
                && [0xb42, 0xb44, 0xb46, 0xb48].into_iter().all(|at| word(ram, at) == 0)
                && released
            {
                self.support_standing_step = Some(state.host_steps);
                self.climb_phase = 27;
            }
        } else if consumed
            && radius == 21
            && state.joy1_current & 0x0200 != 0
            && state.effective_buttons & 0x40 != 0
            && (state.x + 5) / 16 == 7
        {
            self.support_span = Some(TransferContact {
                step: state.host_steps,
                pose,
                radius,
                x: state.x,
                speed: word(ram, 0xb46),
                subspeed: word(ram, 0xb48),
                accel_mode: word(ram, 0xb4a),
                speed_table: word(ram, 0xa6c),
                x_subposition: word(ram, 0xaf8),
                extra_speed: word(ram, 0xb42),
                extra_subspeed: word(ram, 0xb44),
            });
            self.climb_phase = 28;
        }
    }

    fn observe_brake(&mut self, state: &State, ram: &[u8]) {
        if self.brake_rest.is_some() {
            self.observe_third_beam(state, ram);
            return;
        }
        let consumed = self.support_span.as_ref().is_some_and(|span| state.host_steps > span.step)
            && self.previous.is_some_and(|previous| state.polls == previous.polls + 1);
        if self.brake_polls + u64::from(consumed) >= 120 {
            self.reason = Some("return brake exceeded 120 consumed polls");
            return;
        }
        self.brake_polls += u64::from(consumed);
        let neutral = state.effective_buttons == 0 && state.joy1_current == 0 && state.joy1_new == 0;
        let up =
            state.effective_buttons == 0x10 && state.joy1_current == 0x0800 && matches!(state.joy1_new, 0 | 0x0800);
        let zero_motion = [0xb42, 0xb44, 0xb46, 0xb48, 0xb4a].into_iter().all(|at| word(ram, at) == 0);
        let wall = word(ram, 0xa1c) == 0x8a
            && ram[0xa1f] == 0x15
            && state.x == 117
            && (word(ram, 0xdd0) == 1 || self.brake_wall_step.is_some());
        if !(117..123).contains(&state.x)
            || state.y != 171
            || word(ram, 0xb00) != 21
            || word(ram, 0xafe) != 5
            || !(matches!(ram[0xa1f], 0 | 1) || wall)
            || ram[0xa1e] != 4
            || !grounded(ram)
            || (consumed && !(if self.climb_phase == 29 { up } else { neutral }))
            || (self.climb_phase >= 29 && (state.x != 117 || word(ram, 0xaf8) != 0 || !zero_motion))
            || (self.climb_phase == 29 && !(wall || (word(ram, 0xa1c) == 4 && ram[0xa1f] == 0)))
            || (self.climb_phase == 30 && !matches!(word(ram, 0xa1c), 2 | 4))
            || word(ram, 0xb5e) != 0
            || word(ram, 0x9d2) != 0
            || word(ram, 0x10002 + 2 * (10 * 16 + 7)) >> 12 != 0
            || ram[0x16402 + 10 * 16 + 7] != 4
            || word(ram, 0x10002 + 2 * (12 * 16 + 7)) != 0xc19a
            || ![8, 10, 12]
                .into_iter()
                .all(|row| [8, 9].into_iter().all(|column| word(ram, 0x10002 + 2 * (row * 16 + column)) >> 12 == 0))
        {
            self.reason = Some("return brake lost its neutral supported ground domain");
            return;
        }
        if consumed && self.climb_phase == 29 && up {
            if word(ram, 0xa1c) == 4 && ram[0xa1f] == 0 {
                self.brake_aim_step = Some(state.host_steps);
                self.climb_phase = 30;
            }
        } else if consumed && neutral {
            self.brake_release_step.get_or_insert(state.host_steps);
            if wall {
                self.brake_wall_step.get_or_insert(state.host_steps);
                if zero_motion && word(ram, 0xaf8) == 0 {
                    self.climb_phase = 29;
                }
            }
            if word(ram, 0xa1c) == 2
                && ram[0xa1f] == 0
                && ram[0xa1e] == 4
                && [0xb42, 0xb44, 0xb46, 0xb48, 0xb4a].into_iter().all(|at| word(ram, at) == 0)
            {
                self.brake_rest = Some((state.host_steps, state.x, word(ram, 0xaf8)));
                self.climb_phase = 31;
            }
        }
    }

    fn observe_third_beam(&mut self, state: &State, ram: &[u8]) {
        if self.third_response_step.is_some() {
            self.observe_upper_transfer(state, ram);
            return;
        }
        let consumed = self.brake_rest.is_some_and(|(step, _, _)| state.host_steps > step)
            && self.previous.is_some_and(|previous| state.polls == previous.polls + 1);
        if self.third_beam_polls + u64::from(consumed) >= 120 {
            self.reason = Some("return third ceiling beam exceeded 120 consumed polls");
            return;
        }
        self.third_beam_polls += u64::from(consumed);
        let shoot = word(ram, 0x9b2);
        let pad = 0x10 | if self.climb_phase == 32 { shoot.reverse_bits() } else { 0 };
        let held = 0x0800 | if self.climb_phase == 32 { shoot } else { 0 };
        let shot = self.climb_phase == 32
            && consumed
            && self.third_ready
            && self.third_aim_step.is_some_and(|step| state.host_steps > step)
            && state.joy1_new & shoot != 0;
        if self.brake_rest.map(|(_, x, sub)| (x, sub)) != Some((117, 0))
            || (state.x, state.y) != (117, 171)
            || word(ram, 0xaf8) != 0
            || ram[0xa1e] != 4
            || ram[0xa1f] != 0
            || word(ram, 0xb00) != 21
            || word(ram, 0xafe) != 5
            || !grounded(ram)
            || ![0xb42, 0xb44, 0xb46, 0xb48, 0xb4a].into_iter().all(|at| word(ram, at) == 0)
            || word(ram, 0xb5e) != 0
            || word(ram, 0x9d2) != 0
            || (consumed
                && (state.effective_buttons != pad || state.joy1_current != held || state.joy1_new & !held != 0))
            || !matches!(word(ram, 0xa1c), 2 | 4)
            || (self.climb_phase >= 32 && word(ram, 0xa1c) != 4)
            || (self.climb_phase == 32 && consumed && !shot)
            || word(ram, 0x10002 + 2 * (12 * 16 + 7)) != 0xc19a
            || word(ram, 0x10002 + 2 * (10 * 16 + 7)) >> 12 != 0
            || ram[0x16402 + 10 * 16 + 7] != 4
            || ram[0x16402 + 8 * 16 + 7] != 0
            || ![8, 10, 12]
                .into_iter()
                .all(|row| [8, 9].into_iter().all(|col| word(ram, 0x10002 + 2 * (row * 16 + col)) >> 12 == 0))
        {
            self.reason = Some("return third ceiling beam lost its stationary aiming domain");
            return;
        }
        let tile = word(ram, 0x10002 + 2 * (8 * 16 + 7));
        if self.third_beam_step.is_none() && !shot && tile != 0xc19a {
            self.reason = Some("return third ceiling target changed before the owned shot");
            return;
        }
        if self.climb_phase == 31 && consumed && word(ram, 0xa1c) == 4 && word(ram, 0xccc) == 0 && word(ram, 0xcce) < 5
        {
            self.third_aim_step = Some(state.host_steps);
            self.third_neighbors = Some([6, 7, 8].map(|col| word(ram, 0x10002 + 2 * (8 * 16 + col))));
            self.climb_phase = 32;
        } else if shot {
            self.third_beam_step = Some(state.host_steps);
            self.climb_phase = 33;
        }
        if self.climb_phase == 32 {
            self.third_ready = word(ram, 0xccc) == 0 && word(ram, 0xcce) < 5 && state.joy1_current & shoot == 0;
            if !self.third_ready {
                self.reason = Some("return third ceiling lost firing readiness before its shot");
                return;
            }
            self.third_neighbors = Some([6, 7, 8].map(|col| word(ram, 0x10002 + 2 * (8 * 16 + col))));
            self.third_before_bts = Some([6, 7, 8].map(|col| ram[0x16402 + 8 * 16 + col]));
        }
        if consumed && self.third_beam_step.is_some() && tile != 0xc19a {
            let plm = (0..40).find(|slot| {
                word(ram, 0x1c37 + 2 * slot) == 0xd064
                    && word(ram, 0x1c87 + 2 * slot) == 0x010e
                    && word(ram, 0x1e17 + 2 * slot) == 0xc052
                    && word(ram, 0x1d27 + 2 * slot) == 0xcae6
            });
            if tile != 0x0053 || plm.is_none() {
                self.reason = Some("return third ceiling reaction has an unsupported first tile");
                return;
            }
            self.third_response_step = Some(state.host_steps);
            self.third_after = Some([6, 7, 8].map(|col| word(ram, 0x10002 + 2 * (8 * 16 + col))));
            self.third_plm_slot = plm.map(|slot| slot as u16);
            self.third_after_bts = Some([6, 7, 8].map(|col| ram[0x16402 + 8 * 16 + col]));
            self.third_plm = plm.map(|slot| [0x1c37, 0x1c87, 0x1d27, 0x1e17].map(|at| word(ram, at + 2 * slot)));
            self.climb_phase = 34;
        }
    }

    fn observe_upper_transfer(&mut self, state: &State, ram: &[u8]) {
        let consumed = self.third_response_step.is_some_and(|step| state.host_steps > step)
            && self.previous.is_some_and(|previous| state.polls == previous.polls + 1);
        if matches!(self.climb_phase, 43..=45) {
            self.observe_door_approach(state, ram, consumed);
            return;
        }
        if self.upper_rest_step.is_none() && self.upper_transfer_polls + u64::from(consumed) >= 120 {
            self.reason = Some("return upper transfer exceeded 120 consumed polls");
            return;
        }
        if self.upper_rest_step.is_none() {
            self.upper_transfer_polls += u64::from(consumed);
        }
        let jump = word(ram, 0x9b4);
        let radius = word(ram, 0xb00);
        let movement = ram[0xa1f];
        let pose = word(ram, 0xa1c);
        let neutral = state.effective_buttons == 0 && state.joy1_current == 0 && state.joy1_new == 0;
        let launch = self.climb_phase == 35
            && consumed
            && self.upper_launch_ready
            && self.upper_release_step.is_some()
            && state.effective_buttons == jump.reverse_bits()
            && state.joy1_current == jump
            && state.joy1_new == jump;
        let landing_lag = pose == 0xa5 && movement == 0 && radius == 19;
        let launch_lag = launch && pose == 0x4c && movement == 2 && radius == 21;
        if !(85..118).contains(&state.x)
            || state.y > 171
            || ram[0xa1e] != 4
            || word(ram, 0xafe) != 5
            || word(ram, 0x9d2) != 0
            || word(ram, 0xb5e) != 0
            || ![6, 7].into_iter().all(|col| {
                word(ram, 0x10002 + 2 * (128 + col)) >> 12 == 0
                    && ram[0x16402 + 128 + col] == 0
                    && (0..40).any(|slot| {
                        word(ram, 0x1c37 + 2 * slot) == 0xd064
                            && word(ram, 0x1c87 + 2 * slot) == 2 * (128 + col) as u16
                            && word(ram, 0x1e17 + 2 * slot) == 0xc052
                    })
            })
            || word(ram, 0x10002 + 2 * 133) != 0x81eb
            || word(ram, 0x10002 + 2 * 166) >> 12 != 0
            || ram[0x16402 + 166] != 4
            || !(4..8).all(|row| (4..10).all(|col| word(ram, 0x10002 + 2 * (16 * row + col)) >> 12 == 0))
        {
            self.reason = Some("return upper transfer lost its temporary opening or support");
            return;
        }
        if self.upper_rest_step.is_some() {
            self.observe_door_reaction(state, ram, consumed);
            return;
        }
        if self.climb_phase == 39 {
            if self.upper_contact.as_ref().map(|contact| (contact.x, contact.x_subposition))
                != Some((state.x, word(ram, 0xaf8)))
                || !(85..=90).contains(&state.x)
                || state.y != 107
                || movement != 0
                || !matches!((pose, radius), (0xa5, 19 | 21) | (2, 21))
                || !grounded(ram)
                || ![0xb42, 0xb44, 0xb46, 0xb48, 0xb4a].into_iter().all(|at| word(ram, at) == 0)
                || (consumed && !neutral)
            {
                self.reason = Some("return upper rest lost its released C5 standing domain");
                return;
            }
            if consumed && neutral && pose == 2 && radius == 21 {
                self.upper_rest_step = Some(state.host_steps);
                self.climb_phase = 40;
                // Door readiness is admitted on a later consumed neutral poll,
                // after its cap and firing path have been validated.
                self.door.ready = false;
                self.door.before = Some([6, 7, 8, 9].map(|row| word(ram, 0x10002 + 2 * (16 * row + 1))));
                self.door.before_bts = Some([6, 7, 8, 9].map(|row| ram[0x16402 + 16 * row + 1]));
            }
            return;
        }
        if self.climb_phase == 34 || (self.climb_phase == 35 && !launch) {
            if state.x != 117
                || word(ram, 0xaf8) != 0
                || state.y != 171
                || movement != 0
                || radius != 21
                || !matches!(pose, 2 | 4)
                || (self.climb_phase == 35 && pose != 2)
                || !grounded(ram)
                || ![0xb42, 0xb44, 0xb46, 0xb48, 0xb4a].into_iter().all(|at| word(ram, at) == 0)
                || (consumed && !neutral)
            {
                self.reason = Some("return upper jump lost its released launch anchor");
                return;
            }
            if self.climb_phase == 34 && consumed && neutral && pose == 2 {
                self.upper_release_step = Some(state.host_steps);
                self.climb_phase = 35;
                self.upper_launch_ready = true;
            }
            return;
        }
        let hold = self.climb_phase == 38;
        if hold
            && (self.upper_hold.map(|(_, x, sub)| (x, sub)) != Some((state.x, word(ram, 0xaf8)))
                || !(85..=90).contains(&state.x)
                || ![0xb42, 0xb44, 0xb4a].into_iter().all(|at| word(ram, at) == 0)
                || (consumed && ![0xb46, 0xb48].into_iter().all(|at| word(ram, at) == 0)))
        {
            self.reason = Some("return upper hold lost its stationary C5 footprint");
            return;
        }
        let left = self.climb_phase == 36;
        let pad = jump.reverse_bits() | if left { 0x40 } else { 0 };
        let held = jump | if left { 0x0200 } else { 0 };
        if (consumed && (state.effective_buttons != pad || state.joy1_current != held || state.joy1_new & !held != 0))
            || !(matches!((movement, radius), (2 | 6, 19) | (0 | 1, 21)) || landing_lag || launch_lag)
            || (!left && !hold && state.x != 117)
            || (launch && (pose != 0x4c || movement != 2 || !matches!(radius, 19 | 21)))
            || (left && state.x <= 100 && state.y.saturating_add(radius) > 128)
        {
            self.reason = Some("return upper transfer lost its ordinary jump input");
            return;
        }
        if launch {
            self.upper_jump_step = Some(state.host_steps);
            self.climb_phase = 37;
        }
        if self.climb_phase == 37 && consumed && movement == 2 && radius == 19 && state.y + radius <= 128 {
            self.upper_feet_step = Some(state.host_steps);
            self.climb_phase = 36;
        }
        if hold && consumed {
            self.upper_left_release_step.get_or_insert(state.host_steps);
        }
        if left && consumed && matches!(movement, 2 | 6) && radius == 19 && (85..=90).contains(&state.x) {
            if ![0xb42, 0xb44, 0xb4a].into_iter().all(|at| word(ram, at) == 0) {
                self.reason = Some("return upper hold cannot stop foreign airborne momentum");
                return;
            }
            self.upper_hold = Some((state.host_steps, state.x, word(ram, 0xaf8)));
            self.upper_hold_motion = Some([0xb46, 0xb48, 0xb42, 0xb44, 0xb4a, 0xa6c].map(|at| word(ram, at)));
            self.climb_phase = 38;
        }
        if (left || (hold && self.upper_left_release_step.is_some()))
            && consumed
            && state.y == 107
            && matches!(movement, 0 | 1)
            && grounded(ram)
            && (state.x - 5) / 16 == 5
            && self.upper_feet_step.is_some()
        {
            self.upper_contact = Some(TransferContact {
                step: state.host_steps,
                pose,
                radius,
                x: state.x,
                speed: word(ram, 0xb46),
                subspeed: word(ram, 0xb48),
                accel_mode: word(ram, 0xb4a),
                speed_table: word(ram, 0xa6c),
                x_subposition: word(ram, 0xaf8),
                extra_speed: word(ram, 0xb42),
                extra_subspeed: word(ram, 0xb44),
            });
            self.climb_phase = 39;
        }
    }

    fn observe_door_reaction(&mut self, state: &State, ram: &[u8], consumed: bool) {
        let polls = if self.climb_phase == 42 { &mut self.door.opening_polls } else { &mut self.door.polls };
        if *polls + u64::from(consumed) >= 120 {
            self.reason = Some("return upper-left door exceeded 120 consumed polls");
            return;
        }
        *polls += u64::from(consumed);
        let shoot = word(ram, 0x9b2);
        let firing = self.climb_phase == 40 && self.door.ready;
        let shot = firing && consumed && state.joy1_new == shoot;
        let pad = if firing { shoot.reverse_bits() } else { 0 };
        let held = if firing { shoot } else { 0 };
        let tiles = [6, 7, 8, 9].map(|row| word(ram, 0x10002 + 2 * (16 * row + 1)));
        let bts = [6, 7, 8, 9].map(|row| ram[0x16402 + 16 * row + 1]);
        // SpawnPLM scans downward and clears plural plm_variables at DF0C,
        // while singular plm_variable at 1E17 survives deletion and reuse.
        let allocation = (0..40)
            .rev()
            .find(|slot| word(ram, 0x1c37 + 2 * slot) == 0)
            .map(|slot| (slot as u16, word(ram, 0x1e17 + 2 * slot)));
        if self.upper_contact.as_ref().map(|contact| (contact.x, contact.x_subposition)) != Some((90, 0))
            || (state.x, state.y) != (90, 107)
            || word(ram, 0xaf8) != 0
            || word(ram, 0xa1c) != 2
            || ram[0xa1f] != 0
            || word(ram, 0xb00) != 21
            || !grounded(ram)
            || ![0xb42, 0xb44, 0xb46, 0xb48, 0xb4a].into_iter().all(|at| word(ram, at) == 0)
            || bts != [0x41, 0xff, 0xfe, 0xfd]
            || !(6..8).all(|row| (2..6).all(|col| word(ram, 0x10002 + 2 * (16 * row + col)) >> 12 == 0))
            || !(6..10).all(|row| word(ram, 0x10002 + 2 * 16 * row) >> 12 == 9 && ram[0x16402 + 16 * row] == 0)
            || (consumed
                && (state.effective_buttons != pad || state.joy1_current != held || state.joy1_new & !held != 0))
            || (firing && consumed && !shot)
            || (shot
                && (self.door.before != Some([0xc40c, 0xd42c, 0xdc2c, 0xdc0c])
                    || self.door.before_bts != Some([0x41, 0xff, 0xfe, 0xfd])))
        {
            self.reason = Some("return upper-left door lost its stationary firing domain");
            return;
        }
        if self.climb_phase == 42 {
            self.observe_door_opening(state, ram, consumed, tiles);
            return;
        }
        if self.door.shot_step.is_none() && !shot && tiles != [0xc40c, 0xd42c, 0xdc2c, 0xdc0c] {
            self.reason = Some("return upper-left cap changed before the owned shot");
            return;
        }
        if firing
            && ((!shot || tiles == [0xc40c, 0xd42c, 0xdc2c, 0xdc0c])
                && (allocation.is_none() || allocation != self.door.allocation)
                || self.door.allocation.is_none())
        {
            self.reason = Some("return upper-left door lost its preceding PLM allocation");
            return;
        }
        if self.climb_phase == 40 && !shot {
            // The preceding consumed neutral observation owns firing readiness.
            // A pending host step must not arm a shot from unconsumed input.
            if consumed {
                self.door.ready =
                    word(ram, 0xccc) == 0 && word(ram, 0xcce) == 0 && state.joy1_current == 0 && allocation.is_some();
                self.door.allocation = allocation;
            } else if self.door.ready && (word(ram, 0xccc) != 0 || word(ram, 0xcce) != 0) {
                self.reason = Some("return upper-left door lost firing readiness before its shot");
                return;
            }
            self.door.before = Some(tiles);
            self.door.before_bts = Some(bts);
        }
        if shot {
            self.door.shot_step = Some(state.host_steps);
            self.climb_phase = 41;
            // This is the native post-update projectile witness, not the source
            // origin: movement may already have advanced its X coordinate.
            self.door.projectile = (0..5).find_map(|slot| {
                let kind = word(ram, 0xc18 + 2 * slot);
                (kind != 0 && kind & 0x0f00 == 0 && word(ram, 0xc04 + 2 * slot) == 7)
                    .then(|| [slot as u16, word(ram, 0xb64 + 2 * slot), word(ram, 0xb78 + 2 * slot), 7])
            });
        } else if consumed && self.climb_phase == 41 {
            self.door.release_step.get_or_insert(state.host_steps);
        }
        if consumed && self.door.shot_step.is_some() && tiles != [0xc40c, 0xd42c, 0xdc2c, 0xdc0c] {
            let plm = (0..40).find(|slot| {
                word(ram, 0x1c37 + 2 * slot) == 0xc8a8
                    && word(ram, 0x1c87 + 2 * slot) == 0x00c2
                    && word(ram, 0x1d27 + 2 * slot) == 0xc4c1
                    && self.door.allocation == Some((*slot as u16, word(ram, 0x1e17 + 2 * slot)))
            });
            if tiles != [0x840d, 0x842d, 0x8c2d, 0x8c0d] || plm.is_none() {
                self.reason = Some("return upper-left cap has an unsupported first draw");
                return;
            }
            self.door.response_step = Some(state.host_steps);
            self.door.after = Some(tiles);
            self.door.after_bts = Some(bts);
            self.door.plm = plm.map(|slot| {
                [
                    slot as u16,
                    word(ram, 0x1c37 + 2 * slot),
                    word(ram, 0x1c87 + 2 * slot),
                    word(ram, 0x1d27 + 2 * slot),
                    word(ram, 0x1e17 + 2 * slot),
                ]
            });
            self.climb_phase = 42;
        }
    }

    fn observe_door_opening(&mut self, state: &State, ram: &[u8], consumed: bool, tiles: [u16; 4]) {
        // ROM C4BD/C4C1/C4C5/C4C9 draw durations advance the instruction
        // pointer by four. AA13 opens only the middle two cap cells.
        let stages = [
            ([0x840d, 0x842d, 0x8c2d, 0x8c0d], 0xc4c1),
            ([0x840e, 0x842e, 0x8c2e, 0x8c0e], 0xc4c5),
            ([0x840f, 0x042f, 0x0c2f, 0x8c0f], 0xc4c9),
            ([0x0482, 0x04a2, 0x0ca2, 0x0c82], 0xc4cd),
        ];
        let Some((slot, variable)) = self.door.allocation else {
            self.reason = Some("return upper-left opening lost its preceding PLM allocation");
            return;
        };
        let slot = usize::from(slot);
        let identity = [0x1c37, 0x1c87, 0x1d27, 0x1e17].map(|at| word(ram, at + 2 * slot));
        let stage =
            stages.iter().position(|(words, ip)| tiles == *words && identity == [0xc8a8, 0x00c2, *ip, variable]);
        let Some(stage) =
            stage.filter(|stage| *stage >= self.door.opening_stage && *stage <= self.door.opening_stage + 1)
        else {
            self.reason = Some("return upper-left opening has an unsupported draw or PLM order");
            return;
        };
        if consumed {
            self.door.release_step.get_or_insert(state.host_steps);
            self.door.opening_stage = stage;
            if stage == 3 {
                self.door.air_step = Some(state.host_steps);
                self.door.air_tiles = Some(tiles);
                self.door.air_bts = Some([6, 7, 8, 9].map(|row| ram[0x16402 + 16 * row + 1]));
                self.door.air_timer = Some(word(ram, 0xde1c + 2 * slot));
                self.door.air_plm = Some([slot as u16, identity[0], identity[1], identity[2], identity[3]]);
                self.climb_phase = 43;
            }
        }
    }

    fn observe_door_approach(&mut self, state: &State, ram: &[u8], consumed: bool) {
        let traversing = self.climb_phase == 44;
        let exiting = self.climb_phase == 45;
        let polls = if exiting {
            &mut self.door.exit_polls
        } else if traversing {
            &mut self.door.traversal_polls
        } else {
            &mut self.door.approach_polls
        };
        if *polls + u64::from(consumed) >= 120 {
            self.reason = Some("return upper-left approach exceeded 120 consumed polls");
            return;
        }
        *polls += u64::from(consumed);
        let pose = word(ram, 0xa1c);
        let movement = ram[0xa1f];
        let radius = word(ram, 0xb00);
        let falling =
            matches!(pose, 0x2a | 0x68) && movement == 6 && matches!(radius, 19 | 21) && word(ram, 0xb36) == 2;
        let walking = matches!(pose, 2 | 0x0a)
            && movement == u8::from(pose == 0x0a)
            && radius == 21
            && state.y == 107
            && grounded(ram);
        let landing = traversing
            && pose == 0xa5
            && movement == 0
            && matches!(radius, 19 | 21)
            // Changed-pose collision uses A5's nominal radius21 and raises Y
            // by two; the stored falling radius19 can survive this first frame.
            && state.y.checked_add(21) == Some(160)
            && word(ram, 0xafc) == 0xffff
            && grounded(ram);
        let exit_walk = exiting
            && self.door.traversal_landing_step.is_some()
            && matches!((pose, movement, radius), (0xa5, 0, 19 | 21) | (2, 0, 21) | (0x0a, 1, 21))
            && state.y == 139
            && word(ram, 0xafc) == 0xffff
            && grounded(ram);
        let allocation = self.door.allocation;
        if self.door.air_step.is_none()
            || self.door.release_step.is_none()
            || !(if exiting {
                5..=25
            } else if traversing {
                5..=59
            } else {
                32..=90
            })
            .contains(&state.x)
            || !(if traversing { 110..=141 } else { 107..=139 }).contains(&state.y)
            || (if exiting {
                !exit_walk
            } else if traversing {
                !falling && !landing
            } else {
                !walking && !falling
            })
            || ram[0xa1e] != 4
            || word(ram, 0xafe) != 5
            || !matches!(word(ram, 0xb36), 0 | 2)
            || ![0xb42, 0xb44].into_iter().all(|at| word(ram, at) == 0)
            || word(ram, 0xb5e) != 0
            || word(ram, 0x9d2) != 0
            || !(6..10).all(|row| word(ram, 0x10002 + 2 * 16 * row) >> 12 == 9 && ram[0x16402 + 16 * row] == 0)
            || [6, 7, 8, 9].map(|row| word(ram, 0x10002 + 2 * (16 * row + 1))) != [0x0482, 0x04a2, 0x0ca2, 0x0c82]
            || [6, 7, 8, 9].map(|row| ram[0x16402 + 16 * row + 1]) != [0x41, 0xff, 0xfe, 0xfd]
            || !allocation.is_some_and(|(slot, variable)| {
                let slot = usize::from(slot);
                [0x1c37, 0x1c87, 0x1d27, 0x1e17].map(|at| word(ram, at + 2 * slot))
                    == [0xc8a8, 0x00c2, 0xc4cd, variable]
            })
            || word(ram, 0x10002 + 2 * 133) != 0x81eb
            || word(ram, 0x10002 + 2 * 132) != 0x80ff
            || !(6..10).all(|row| (2..4).all(|col| word(ram, 0x10002 + 2 * (16 * row + col)) >> 12 == 0))
            || !(0..4).all(|col| word(ram, 0x10002 + 2 * (160 + col)) == 0x81ec)
            || self.previous.is_some_and(|previous| state.x > previous.x || (!landing && state.y < previous.y))
            || (consumed
                && (state.effective_buttons != 0x40 || state.joy1_current != 0x0200 || state.joy1_new & !0x0200 != 0))
        {
            self.reason = Some("return upper-left approach lost its open door or ordinary Left domain");
            return;
        }
        if consumed {
            self.door.approach_left_step.get_or_insert(state.host_steps);
            if falling && !traversing {
                self.door.approach_fall_step = Some(state.host_steps);
                self.door.approach_fall = Some([state.x, state.y, pose, u16::from(movement), radius]);
                self.door.approach_motion =
                    Some([0xaf8, 0xafc, 0xb46, 0xb48, 0xb4a, 0xb2e, 0xb2c, 0xb36].map(|at| word(ram, at)));
                self.climb_phase = 44;
            } else if landing {
                self.door.traversal_landing_step = Some(state.host_steps);
                self.door.traversal_landing = Some([state.x, state.y, pose, radius]);
                self.climb_phase = 45;
            }
        }
    }

    fn observe_loading(&mut self, state: &State, ram: &[u8]) {
        // Synchronous nested calls can skip observable functions and state10.
        // The remaining observations must follow the recovered dispatch order.
        let functions = [
            0xe17d, 0xe29e, 0xe2db, 0xe2f7, 0xe310, 0xe353, 0xe36e, 0xe38e, 0xe3c0, 0xe4a9, 0xe659, 0xe664, 0xe6a2,
            0xe737,
        ];
        let Some(entry) = self.door.transition_step else {
            self.reason = Some("return loading lost its causal door event");
            return;
        };
        let consumed = self.previous.is_some_and(|previous| state.polls == previous.polls + 1);
        let stage = functions.iter().position(|function| *function == word(ram, 0x99c));
        let Some(stage) = stage.filter(|stage| *stage >= self.door.loading_stage) else {
            self.reason = Some("return loading lost native function order");
            return;
        };
        let loaded_door = stage >= 4;
        let loaded_room = stage >= 7;
        let arrival = state.game_state == 8;
        if self.door.transition_door != Some([0x8ec2, 0x9f57])
            || state.host_steps <= entry
            || state.host_steps - entry >= 240
            || self.door.loading_polls + u64::from(consumed) >= 240
            || !matches!(state.game_state, 8..=11)
            || state.area != 1
            || word(ram, 0x78d) != 0x8ec2
            || word(ram, 0xe16) != 0
            || state.room != if loaded_door { 0x9e9f } else { 0x9f11 }
            || word(ram, 0x7b5) != if loaded_room { 0x9ee5 } else { 0x9f57 }
            || word(ram, 0x7a5) != if loaded_room { 128 } else { 16 }
            || word(ram, 0x7a7) != if loaded_room { 48 } else { 32 }
            || (loaded_door && (word(ram, 0x791) != 5 || word(ram, 0x927) != 0x700 || word(ram, 0x929) != 0x200))
            || (consumed && (state.effective_buttons != 0 || state.joy1_current != 0 || state.joy1_new != 0))
            || (arrival
                && (!consumed
                    || stage != 13
                    || self.door.loading_stage != 13
                    || !self.previous.is_some_and(|previous| previous.game_state == 11)
                    || word(ram, 0x795) != 0
                    || word(ram, 0x797) != 0))
            || (!arrival && state.game_state != 11 && stage != 0)
        {
            self.reason = Some("return loading lost its neutral owned destination domain");
            return;
        }
        self.door.loading_polls += u64::from(consumed);
        self.door.loading_stage = stage;
        self.door.loading_stages |= 1 << stage;
        if arrival {
            self.door.arrival_step = Some(state.host_steps);
            self.door.arrival = Some([state.x, state.y, word(ram, 0xa1c), word(ram, 0xb00)]);
            self.climb_phase = 47;
        }
    }

    fn observe_departure(&mut self, state: &State, ram: &[u8]) {
        let descending = self.climb_phase == 48;
        let consumed = self.previous.is_some_and(|previous| state.polls == previous.polls + 1);
        let pose = word(ram, 0xa1c);
        let movement = ram[0xa1f];
        let radius = word(ram, 0xb00);
        let walking = matches!(pose, 2 | 0x0a)
            && movement == u8::from(pose == 0x0a)
            && radius == 21
            && state.y == 651
            && grounded(ram);
        let falling =
            matches!(pose, 0x2a | 0x68) && movement == 6 && matches!(radius, 19 | 21) && word(ram, 0xb36) == 2;
        let landing = descending
            && pose == 0xa5
            && movement == 0
            && matches!(radius, 19 | 21)
            // Native changed-pose collision uses nominal A5 radius21; the
            // first landed frame can retain the falling radius19 in WRAM.
            && state.y.checked_add(21) == Some(720)
            && word(ram, 0xafc) == 0xffff
            && [0xb46, 0xb48, 0xb4a].into_iter().all(|at| word(ram, at) == 0)
            && grounded(ram);
        let polls = if descending { self.door.lower_floor_polls } else { self.door.departure_polls };
        let elevator = 0xf78 + 2 * 64;
        if self.door.arrival_step.is_none()
            || (descending && self.door.departure_fall_step.is_none())
            || polls + u64::from(consumed) >= 120
            || state.game_state != 8
            || state.area != 1
            || state.room != 0x9e9f
            || word(ram, 0x7bb) != 0x9eb1
            || word(ram, 0x7cf) != 0x93ac
            || word(ram, 0x7a5) != 128
            || word(ram, 0x7a7) != 48
            || word(ram, 0x7b5) != 0x9ee5
            || word(ram, elevator) != 0xd73f
            || word(ram, elevator + 2) != 1408
            || word(ram, elevator + 6) != 706
            || word(ram, elevator + 0x3c) != 2
            || word(ram, elevator + 0x3e) != 24
            || word(ram, 0xe16) != 0
            || word(ram, 0xe18) != 0
            || !(if descending { 1861..=1947 } else { 1900..=2008 }).contains(&state.x)
            || !(if descending { 655..=701 } else { 651..=683 }).contains(&state.y)
            || (if descending { !falling && !landing } else { !walking && !falling })
            || ram[0xa1e] != 4
            || word(ram, 0xafe) != 5
            || word(ram, 0xa6c) != 0x9f55
            || word(ram, 0xb42) != 0
            || word(ram, 0xb44) != 0
            || word(ram, 0xb5e) != 0
            || word(ram, 0x9d2) != 0
            || [122, 123, 124, 125, 126, 127].map(|col| word(ram, 0x10002 + 2 * (42 * 128 + col)))
                != [0x81ea, 0x81ea, 0x81e9, 0x81e9, 0x81e9, 0x81ec]
            || !(39..=44).all(|row| (116..=121).all(|col| word(ram, 0x10002 + 2 * (row * 128 + col)) == 0x00ff))
            || (descending
                && [116, 117, 118, 119, 120, 121].map(|col| word(ram, 0x10002 + 2 * (45 * 128 + col)))
                    != [0x81e9, 0x81e9, 0x81e9, 0x81ea, 0x81ea, 0x81ea])
            || self.previous.is_some_and(|previous| state.x > previous.x || (!landing && state.y < previous.y))
            || (consumed
                && (state.effective_buttons != 0x40 || state.joy1_current != 0x0200 || state.joy1_new & !0x0200 != 0))
        {
            self.reason = Some(if descending {
                "return elevator approach lost its lower-floor landing domain"
            } else {
                "return elevator approach lost its first shelf departure domain"
            });
            return;
        }
        if descending {
            self.door.lower_floor_polls += u64::from(consumed);
            if consumed && landing {
                self.door.lower_floor_step = Some(state.host_steps);
                self.door.lower_floor = Some([state.x, state.y, pose, radius]);
                self.door.lower_floor_motion =
                    Some([0xaf8, 0xafc, 0xb46, 0xb48, 0xb4a, 0xb2e, 0xb2c, 0xb36].map(|at| word(ram, at)));
                self.climb_phase = 49;
            }
        } else if consumed {
            self.door.departure_polls += 1;
        }
        if !descending && consumed && falling {
            self.door.departure_fall_step = Some(state.host_steps);
            self.door.departure_fall = Some([state.x, state.y, pose, radius]);
            self.door.departure_motion =
                Some([0xaf8, 0xafc, 0xb46, 0xb48, 0xb4a, 0xb2e, 0xb2c, 0xb36].map(|at| word(ram, at)));
            self.climb_phase = 48;
        }
    }

    fn observe_platform(&mut self, state: &State, ram: &[u8]) {
        let boarding = self.climb_phase >= 50;
        let consumed = self.previous.is_some_and(|previous| state.polls == previous.polls + 1);
        let pose = word(ram, 0xa1c);
        let movement = ram[0xa1f];
        let radius = word(ram, 0xb00);
        let wall = pose == 0x8a
            && movement == 0x15
            && radius == 21
            && (word(ram, 0xdd0) == 1 || boarding)
            && [0xaf8, 0xb46, 0xb48, 0xb4a].into_iter().all(|at| word(ram, at) == 0)
            && state.x.checked_sub(word(ram, 0xafe)) == Some(1472);
        let walking = matches!((pose, movement, radius), (0xa5, 0, 19 | 21) | (2, 0, 21) | (0x0a, 1, 21));
        let neutral = state.effective_buttons == 0 && state.joy1_current == 0 && state.joy1_new == 0;
        let jumping =
            matches!(pose, 0x4c | 0x4e) && movement == 2 && matches!(radius, 19 | 21) && word(ram, 0xb36) == 1;
        let standing = pose == 2 && movement == 0 && radius == 21 && grounded(ram);
        let aimed = pose == 4 && movement == 0 && radius == 21 && grounded(ram);
        let boarding_pose = if self.climb_phase == 53 {
            wall || aimed
        } else if self.climb_phase == 50 {
            aimed || standing
        } else {
            jumping || (self.climb_phase == 51 && !consumed && standing)
        };
        let elevator = 0xf78 + 128;
        if self.door.lower_floor_step.is_none()
            || (boarding && self.door.platform_step.is_none())
            || (matches!(self.climb_phase, 50..=52) && self.door.boarding_aim_step.is_none())
            || (matches!(self.climb_phase, 51 | 52) && self.door.boarding_release_step.is_none())
            || (self.climb_phase == 52
                && (self.door.boarding_jump_step.is_none()
                    || (consumed && radius != 19)
                    || (!consumed
                        && radius == 21
                        && self.door.boarding_jump
                            != Some([
                                state.y,
                                pose,
                                word(ram, 0xb2e),
                                word(ram, 0xb2c),
                                word(ram, 0xb36),
                                radius,
                                word(ram, 0xafc),
                            ]))))
            || (self.climb_phase == 51
                && consumed
                && (pose != 0x4c
                    || radius != 21
                    || state.y != 699
                    || word(ram, 0xafc) != 0xffff
                    || word(ram, 0xb2e) != 4
                    || word(ram, 0xb2c) != 0xe000))
            || (if boarding {
                self.door.boarding_polls + u64::from(consumed) >= 120
            } else {
                self.door.platform_polls + u64::from(consumed) >= 240
            })
            || state.game_state != 8
            || state.area != 1
            || state.room != 0x9e9f
            || word(ram, 0x7bb) != 0x9eb1
            || word(ram, 0x7cf) != 0x93ac
            || word(ram, 0x7a5) != 128
            || word(ram, 0x7a7) != 48
            || word(ram, 0x7b5) != 0x9ee5
            || [
                word(ram, elevator),
                word(ram, elevator + 2),
                word(ram, elevator + 6),
                word(ram, elevator + 0x3c),
                word(ram, elevator + 0x3e),
            ] != [0xd73f, 1408, 706, 2, 24]
            || word(ram, 0xe16) != 0
            || word(ram, 0xe18) != 0
            || (if boarding { state.x != 1477 || word(ram, 0xaf8) != 0 } else { !(1477..=1905).contains(&state.x) })
            || (if matches!(self.climb_phase, 51 | 52) {
                !(660..=699).contains(&state.y)
            } else {
                state.y != 699 || word(ram, 0xafc) != 0xffff
            })
            || (if boarding { !boarding_pose } else { !walking && !wall })
            || ram[0xa1e] != 4
            || word(ram, 0xafe) != 5
            || (!boarding && !grounded(ram))
            || word(ram, 0xa6c) != 0x9f55
            || ![0xb42, 0xb44, 0xb5e, 0x9d2].into_iter().all(|at| word(ram, at) == 0)
            || !(92..=121)
                .all(|col| word(ram, 0x10002 + 2 * (45 * 128 + col)) >> 12 == 8 && ram[0x16402 + 45 * 128 + col] == 0)
            || !(41..=43).all(|row| {
                (85..=121).all(|col| {
                    word(ram, 0x10002 + 2 * (row * 128 + col)) == 0x00ff && ram[0x16402 + row * 128 + col] == 0
                })
            })
            || !(92..=121)
                .all(|col| word(ram, 0x10002 + 2 * (44 * 128 + col)) == 0x00ff && ram[0x16402 + 44 * 128 + col] == 0)
            || [85, 86, 87, 88, 89, 90, 91].map(|col| word(ram, 0x10002 + 2 * (44 * 128 + col)))
                != [0x81bf, 0x81bf, 0x90ff, 0x90ff, 0x81bf, 0x81bf, 0x81bf]
            || [85, 86, 87, 88, 89, 90, 91].map(|col| ram[0x16402 + 44 * 128 + col]) != [0, 0, 3, 3, 0, 0, 0]
            || self.previous.is_some_and(|previous| state.x > previous.x)
            || (!boarding
                && consumed
                && (state.effective_buttons != 0x40 || state.joy1_current != 0x0200 || state.joy1_new & !0x0200 != 0))
            || (boarding
                && (![0xb46, 0xb48, 0xb4a].into_iter().all(|at| word(ram, at) == 0)
                    || word(ram, 0x9a2) != 4
                    || word(ram, 0x9b4) != 0x80
                    || word(ram, 0x195e) != 0xffff
                    || word(ram, 0x1962) != 0xffff
                    || word(ram, 0x1968) != 0
                    || word(ram, 0xb32) != 0x1c00
                    || word(ram, 0xb34) != 0
                    || (matches!(self.climb_phase, 51 | 52)
                        && self.previous.is_some_and(|previous| state.y > previous.y))
                    || (consumed
                        && (if self.climb_phase == 53 {
                            state.effective_buttons != 0x10
                                || state.joy1_current != 0x800
                                || state.joy1_new & !0x800 != 0
                        } else if self.climb_phase == 50 {
                            !neutral
                        } else {
                            state.effective_buttons != 0x100
                                || state.joy1_current != 0x80
                                || state.joy1_new & !0x80 != 0
                                || (self.climb_phase == 51 && state.joy1_new != 0x80)
                        }))))
        {
            self.reason = Some("return elevator approach lost its raised-platform corridor");
            return;
        }
        if boarding {
            self.door.boarding_polls += u64::from(consumed);
            if consumed && self.climb_phase == 53 && aimed {
                self.door.boarding_aim_step = Some(state.host_steps);
                self.climb_phase = 50;
            } else if consumed && self.climb_phase == 50 && standing {
                self.door.boarding_release_step = Some(state.host_steps);
                self.climb_phase = 51;
            } else if consumed && jumping {
                if self.climb_phase == 51 {
                    self.door.boarding_jump_step = Some(state.host_steps);
                    self.door.boarding_jump = Some([
                        state.y,
                        pose,
                        word(ram, 0xb2e),
                        word(ram, 0xb2c),
                        word(ram, 0xb36),
                        radius,
                        word(ram, 0xafc),
                    ]);
                    self.climb_phase = 52;
                }
                if state
                    .y
                    .checked_add(radius)
                    .and_then(|bottom| bottom.checked_sub(1))
                    .is_some_and(|bottom| bottom < 704)
                {
                    self.door.boarding_clear_step = Some(state.host_steps);
                    self.door.boarding_clear = Some([
                        state.y,
                        pose,
                        word(ram, 0xb2e),
                        word(ram, 0xb2c),
                        word(ram, 0xb36),
                        radius,
                        word(ram, 0xafc),
                    ]);
                    self.climb_phase = 54;
                }
            }
            return;
        }
        self.door.platform_polls += u64::from(consumed);
        if consumed && wall {
            self.door.platform_step = Some(state.host_steps);
            self.door.platform = Some([state.x, state.y, pose, radius]);
            self.door.platform_motion =
                Some([0xaf8, 0xafc, 0xb46, 0xb48, 0xb4a, 0xb2e, 0xb2c, 0xb36].map(|at| word(ram, at)));
            self.door.platform_collision = Some(word(ram, 0xdd0));
            self.climb_phase = 53;
        }
    }

    fn observe_crossing(&mut self, state: &State, ram: &[u8]) {
        let consumed = self.previous.is_some_and(|previous| state.polls == previous.polls + 1);
        let pose = word(ram, 0xa1c);
        let elevator = 0xf78 + 128;
        let y = (u32::from(state.y) << 16) | u32::from(word(ram, 0xafc));
        let ceiling = self.door.crossing_ceiling_step.is_none()
            && self.door.crossing_left_step.is_some()
            && state.x.checked_sub(word(ram, 0xafe)).is_some_and(|left| left < 1472)
            && state.y == 675
            && word(ram, 0xafc) == 0
            && word(ram, 0xb2e) == 0
            && word(ram, 0xb2c) == 0
            && word(ram, 0xb36) == 2
            && word(ram, 0xdd0) == 1;
        let descending = self.door.crossing_ceiling_step.is_some();
        if self.door.boarding_clear_step.is_none()
            || self.door.crossing_polls + u64::from(consumed) >= 120
            || state.game_state != 8
            || state.area != 1
            || state.room != 0x9e9f
            || [word(ram, 0x7bb), word(ram, 0x7cf), word(ram, 0x7a5), word(ram, 0x7a7), word(ram, 0x7b5)]
                != [0x9eb1, 0x93ac, 128, 48, 0x9ee5]
            || [
                word(ram, elevator),
                word(ram, elevator + 2),
                word(ram, elevator + 6),
                word(ram, elevator + 0x3c),
                word(ram, elevator + 0x3e),
            ] != [0xd73f, 1408, 706, 2, 24]
            || word(ram, 0xe16) != 0
            || word(ram, 0xe18) != 0
            || !(1460..=1477).contains(&state.x)
            || !(675..=685).contains(&state.y)
            || !matches!(pose, 0x4e | 0x52)
            || ram[0xa1e] != 4
            || ram[0xa1f] != 2
            || word(ram, 0xb00) != 19
            || word(ram, 0xafe) != 5
            || word(ram, 0xb36) != if descending || ceiling { 2 } else { 1 }
            || word(ram, 0xb4a) != 0
            || (descending && pose != 0x52)
            || word(ram, 0xa6c) != 0x9f55
            || ![0xb42, 0xb44, 0xb5e, 0x9d2].into_iter().all(|at| word(ram, at) == 0)
            || !(41..=43).all(|row| {
                (90..=92).all(|col| {
                    word(ram, 0x10002 + 2 * (row * 128 + col)) == 0x00ff && ram[0x16402 + row * 128 + col] == 0
                })
            })
            || !(90..=91).all(|col| {
                word(ram, 0x10002 + 2 * (44 * 128 + col)) == 0x81bf
                    && ram[0x16402 + 44 * 128 + col] == 0
                    && word(ram, 0x10002 + 2 * (40 * 128 + col)) == 0x81e9
                    && ram[0x16402 + 40 * 128 + col] == 0
            })
            || word(ram, 0x10002 + 2 * (44 * 128 + 92)) != 0x00ff
            || ram[0x16402 + 44 * 128 + 92] != 0
            || self.previous.is_some_and(|previous| state.x > previous.x)
            || self.door.crossing_y.is_some_and(|previous| if descending { y < previous } else { y > previous })
            || (consumed
                && (state.effective_buttons != 0x140
                    || state.joy1_current != 0x280
                    || state.joy1_new & !0x280 != 0
                    || pose != 0x52))
        {
            self.reason = Some("return platform crossing lost its held-jump clearance domain");
            return;
        }
        self.door.crossing_polls += u64::from(consumed);
        self.door.crossing_y = Some(y);
        if consumed {
            let snapshot = [
                state.x,
                state.y,
                pose,
                word(ram, 0xb00),
                word(ram, 0xaf8),
                word(ram, 0xafc),
                word(ram, 0xb46),
                word(ram, 0xb48),
                word(ram, 0xb2e),
                word(ram, 0xb2c),
            ];
            if self.door.crossing_left_step.is_none() {
                self.door.crossing_left_step = Some(state.host_steps);
                self.door.crossing_left = Some(snapshot);
            }
            if ceiling {
                self.door.crossing_ceiling_step = Some(state.host_steps);
                self.door.crossing_ceiling = Some(snapshot);
                self.door.crossing_ceiling_motion = Some([0xb36, 0xdd0, 0xb4a].map(|at| word(ram, at)));
            }
            if state
                .x
                .checked_add(word(ram, 0xafe))
                .and_then(|right| right.checked_sub(1))
                .is_some_and(|right| right < 1472)
            {
                self.door.crossing_span_step = Some(state.host_steps);
                self.door.crossing_span = Some(snapshot);
                self.door.crossing_span_motion = Some([0xb36, 0xdd0, 0xb4a].map(|at| word(ram, at)));
                self.climb_phase = 55;
            }
        }
    }

    fn observe_platform_descent(&mut self, state: &State, ram: &[u8]) {
        let consumed = self.previous.is_some_and(|previous| state.polls == previous.polls + 1);
        let Some(span) = self.door.crossing_span else {
            self.reason = Some("return platform descent lacks consumed full-span witness");
            return;
        };
        let pose = word(ram, 0xa1c);
        let radius = word(ram, 0xb00);
        let y = (u32::from(state.y) << 16) | u32::from(word(ram, 0xafc));
        // Block collision uses A5's nominal radius21 for pose growth before
        // the stored jumping radius19 is necessarily updated. This separate
        // landing adjustment may raise Y by two pixels; air remains monotonic.
        let landing = pose == 0xa5
            && ram[0xa1f] == 0
            && matches!(radius, 19 | 21)
            && state.y.checked_add(21) == Some(704)
            && word(ram, 0xafc) == 0xffff
            // The first native A5 frame's pose-height collision clears DD0.
            // The grounded pose and stopped Y motion retain the landing witness.
            && word(ram, 0xdd0) == 0
            && grounded(ram);
        let released = self.door.descent_release_step.is_some() || consumed;
        let elevator = 0xf78 + 128;
        if self.door.crossing_span_step.is_none()
            || self.door.crossing_ceiling_step.is_none()
            || self.door.crossing_span_motion != Some([2, 0, 0])
            || self.door.descent_polls + u64::from(consumed) > 32
            || state.game_state != 8
            || state.area != 1
            || state.room != 0x9e9f
            || [word(ram, 0x7bb), word(ram, 0x7cf), word(ram, 0x7a5), word(ram, 0x7a7), word(ram, 0x7b5)]
                != [0x9eb1, 0x93ac, 128, 48, 0x9ee5]
            || [
                word(ram, elevator),
                word(ram, elevator + 2),
                word(ram, elevator + 4),
                word(ram, elevator + 6),
                word(ram, elevator + 8),
                word(ram, elevator + 0x3c),
                word(ram, elevator + 0x3e),
                word(ram, elevator + 0x30),
            ] != [0xd73f, 1408, 0, 706, 0, 2, 24, 706]
            || !(0x32..=0x3a).step_by(2).all(|offset| word(ram, elevator + offset) == 0)
            || word(ram, 0xe16) != 0
            || word(ram, 0xe18) != 0
            || state.x != span[0]
            || word(ram, 0xaf8) != span[4]
            || word(ram, 0xafe) != 5
            || state.x.checked_sub(5).is_none_or(|left| left < 1440)
            || state.x.checked_add(4).is_none_or(|right| right >= 1472)
            || ram[0xa1e] != 4
            || word(ram, 0xb4a) != 0
            || word(ram, 0xa6c) != 0x9f55
            || word(ram, 0xb34) != 0
            || word(ram, 0xb32) != 0x1c00
            || word(ram, 0x9a2) != 4
            || word(ram, 0x9b4) != 0x80
            || word(ram, 0x195e) != 0xffff
            || word(ram, 0x1962) != 0xffff
            || word(ram, 0x1968) != 0
            || ![0xb42, 0xb44, 0xb5e, 0x9d2].into_iter().all(|at| word(ram, at) == 0)
            || (released && (word(ram, 0xb46) != 0 || word(ram, 0xb48) != 0))
            || !(41..=43).all(|row| {
                (90..=92).all(|col| {
                    word(ram, 0x10002 + 2 * (row * 128 + col)) == 0x00ff && ram[0x16402 + row * 128 + col] == 0
                })
            })
            || !(90..=91).all(|col| {
                word(ram, 0x10002 + 2 * (44 * 128 + col)) == 0x81bf
                    && ram[0x16402 + 44 * 128 + col] == 0
                    && word(ram, 0x10002 + 2 * (40 * 128 + col)) == 0x81e9
                    && ram[0x16402 + 40 * 128 + col] == 0
            })
            || word(ram, 0x10002 + 2 * (44 * 128 + 92)) != 0x00ff
            || ram[0x16402 + 44 * 128 + 92] != 0
            || (!landing
                && (pose != 0x52
                    || ram[0xa1f] != 2
                    || radius != 19
                    || word(ram, 0xb36) != 2
                    || word(ram, 0xdd0) != 0
                    || !(675..=685).contains(&state.y)
                    || word(ram, 0xb2e) >= 5
                    || ((u32::from(word(ram, 0xb2e)) << 16) | u32::from(word(ram, 0xb2c)))
                        < ((u32::from(span[8]) << 16) | u32::from(span[9]))
                    || y < self.door.descent_y.unwrap_or((u32::from(span[1]) << 16) | u32::from(span[5]))))
            || (landing && self.door.descent_release_step.is_none())
            || (consumed && (state.effective_buttons != 0 || state.joy1_current != 0 || state.joy1_new != 0))
        {
            self.reason = Some("return platform descent lost its neutral ordinary-landing domain");
            return;
        }
        if consumed {
            self.door.descent_polls += 1;
            self.door.descent_y = Some(y);
            let snapshot = [
                state.x,
                state.y,
                pose,
                radius,
                word(ram, 0xaf8),
                word(ram, 0xafc),
                word(ram, 0xb46),
                word(ram, 0xb48),
                word(ram, 0xb2e),
                word(ram, 0xb2c),
                word(ram, 0xb36),
                word(ram, 0xdd0),
                word(ram, 0xb4a),
            ];
            if self.door.descent_release_step.is_none() {
                self.door.descent_release_step = Some(state.host_steps);
                self.door.descent_release = Some(snapshot);
            }
            if landing {
                self.door.descent_landing_step = Some(state.host_steps);
                self.door.descent_landing = Some(snapshot);
                self.reason = Some("return platform first ordinary landing requires reached-state review");
            }
        }
    }

    pub fn buttons(&self, state: &State, ram: &[u8]) -> u16 {
        if !self.prefix_verified || self.reason.is_some() || ram.len() != 131072 || state.game_state != 8 {
            return 0;
        }
        if matches!(self.climb_phase, 46 | 55) {
            return 0;
        }
        if matches!(self.climb_phase, 47..=49) {
            return 0x40;
        }
        if self.climb_phase == 50 {
            return 0;
        }
        if self.climb_phase == 53 {
            return 0x10;
        }
        if self.climb_phase == 54 {
            return 0x140;
        }
        if matches!(self.climb_phase, 51 | 52) {
            return 0x100;
        }
        if word(ram, 0xdc8) != 0 || ram[0x614] != 0 || word(ram, 0xa78) != 0 {
            return 0;
        }
        if state.room == 0x9f11 {
            let jump = word(ram, 0x9b4).reverse_bits();
            let shoot = word(ram, 0x9b2).reverse_bits();
            if !jump.is_power_of_two()
                || !shoot.is_power_of_two()
                || (jump | shoot) & !0x0f0f != 0
                || jump & (0xf0 | shoot) != 0
                || (self.climb_phase >= 4 && shoot & 0xf0 != 0)
            {
                return 0;
            }
            let ball = matches!(ram[0xa1f], 4 | 8 | 0x11..=0x13);
            if matches!(self.climb_phase, 5 | 12 | 15 | 18)
                && (word(ram, 0xa1c) != 4
                    || state.joy1_current & 0x0800 == 0
                    || state.joy1_current & word(ram, 0x9b4) != 0)
            {
                return 0;
            }
            let pad = match self.climb_phase {
                0 if ball => 0x80,
                0 => {
                    // Keep the request through unpolled host steps, then release
                    // until the native held key clears before another Down edge.
                    if state.joy1_current & 0x0400 == 0 { 0x20 } else { 0 }
                }
                1 => {
                    // Unmorph reaches crouch first. Its native pose table
                    // requires another new Up edge to enter standing.
                    if state.joy1_current & 0x0800 == 0 { 0x10 } else { 0 }
                }
                2 => jump,
                3 if state.x > 112 => 0x40 | jump,
                3 if state.x < 108 => 0x80 | jump,
                4 | 6 => 0x10,
                5 => 0x10 | shoot,
                7 => jump,
                8 if state.x > 144 => 0x40 | jump,
                8 if state.x < 144 => 0x80 | jump,
                8 => jump,
                9 if state.x > 144 => 0x40,
                9 if state.x < 144 => 0x80,
                9 => 0,
                10 => 0,
                11 | 13 | 14 | 16 | 17 | 19 => 0x10,
                12 | 15 | 18 => 0x10 | shoot,
                20 => 0,
                21 | 22 => jump,
                23 => jump | shoot,
                24 => jump,
                25 => jump | 0x40,
                26 => 0,
                27 => 0x40,
                28 | 30 => 0,
                29 | 31 | 33 => 0x10,
                32 if self.third_ready => 0x10 | shoot,
                32 => 0,
                34 | 39 | 41 | 42 => 0,
                43..=45 => 0x40,
                40 if self.door.ready => shoot,
                40 => 0,
                35 | 37 | 38 => jump,
                36 => 0x40 | jump,
                _ => jump,
            };
            return if pad & shoot == 0 || matches!(self.climb_phase, 5 | 12 | 15 | 18 | 23 | 32 | 40) {
                pad
            } else {
                0
            };
        }
        // The observed author01 endpoint has the closed blue east door at219/139.
        // One ordinary beam edge is admitted only with no missile selection.
        let run = word(ram, 0x9b6).reverse_bits();
        let shoot = word(ram, 0x9b2).reverse_bits();
        if !run.is_power_of_two()
            || !shoot.is_power_of_two()
            || (run | shoot) & !0x0f0f != 0
            || (run | 0x80) & shoot != 0
        {
            return 0;
        }
        if self.stationary_polls >= 12
            && self.beam_step.is_none()
            && state.room == 0xa107
            && (state.x, state.y) == (219, 139)
            && word(ram, 0x9d2) == 0
        {
            return 0x80 | shoot;
        }
        0x80 | run
    }

    pub fn report(&self, report: &mut sm_binary_evidence::Map) {
        report.insert("rio_return_prefix_verified".into(), self.prefix_verified.into());
        report.insert("rio_return_complete".into(), false.into());
        report.insert("rio_return_door_traversal_polls".into(), (self.door.traversal_polls as i64).into());
        report.insert(
            "rio_return_door_transition_step".into(),
            self.door.transition_step.map_or(-1, |step| step as i64).into(),
        );
        for (index, key) in ["definition", "list"].into_iter().enumerate() {
            report.insert(
                format!("rio_return_door_transition_{key}"),
                self.door.transition_door.map_or(-1, |door| i64::from(door[index])).into(),
            );
        }
        report.insert(
            "rio_return_door_traversal_landing_step".into(),
            self.door.traversal_landing_step.map_or(-1, |step| step as i64).into(),
        );
        report.insert("rio_return_door_exit_polls".into(), (self.door.exit_polls as i64).into());
        report.insert("rio_return_loading_polls".into(), (self.door.loading_polls as i64).into());
        report.insert("rio_return_loading_stage".into(), (self.door.loading_stage as i64).into());
        report.insert("rio_return_loading_stages".into(), i64::from(self.door.loading_stages).into());
        report.insert("rio_return_arrival_step".into(), self.door.arrival_step.map_or(-1, |step| step as i64).into());
        for (index, key) in ["x", "y", "pose", "radius"].into_iter().enumerate() {
            report.insert(
                format!("rio_return_arrival_{key}"),
                self.door.arrival.map_or(-1, |value| i64::from(value[index])).into(),
            );
        }
        report.insert("rio_return_departure_polls".into(), (self.door.departure_polls as i64).into());
        report.insert("rio_return_lower_floor_polls".into(), (self.door.lower_floor_polls as i64).into());
        report.insert("rio_return_platform_polls".into(), (self.door.platform_polls as i64).into());
        report.insert("rio_return_boarding_polls".into(), (self.door.boarding_polls as i64).into());
        report.insert("rio_return_crossing_polls".into(), (self.door.crossing_polls as i64).into());
        report.insert("rio_return_descent_polls".into(), (self.door.descent_polls as i64).into());
        for (stage, step, snapshot) in [
            ("release", self.door.descent_release_step, self.door.descent_release),
            ("landing", self.door.descent_landing_step, self.door.descent_landing),
        ] {
            report.insert(format!("rio_return_descent_{stage}_step"), step.map_or(-1, |value| value as i64).into());
            for (index, key) in [
                "x",
                "y",
                "pose",
                "radius",
                "x_subposition",
                "y_subposition",
                "x_speed",
                "x_subspeed",
                "y_speed",
                "y_subspeed",
                "y_direction",
                "collision",
                "x_acceleration",
            ]
            .into_iter()
            .enumerate()
            {
                report.insert(
                    format!("rio_return_descent_{stage}_{key}"),
                    snapshot.map_or(-1, |value| i64::from(value[index])).into(),
                );
            }
        }
        for (stage, step, snapshot) in [
            ("left", self.door.crossing_left_step, self.door.crossing_left),
            ("ceiling", self.door.crossing_ceiling_step, self.door.crossing_ceiling),
            ("span", self.door.crossing_span_step, self.door.crossing_span),
        ] {
            report.insert(format!("rio_return_crossing_{stage}_step"), step.map_or(-1, |value| value as i64).into());
            for (index, key) in [
                "x",
                "y",
                "pose",
                "radius",
                "x_subposition",
                "y_subposition",
                "x_speed",
                "x_subspeed",
                "y_speed",
                "y_subspeed",
            ]
            .into_iter()
            .enumerate()
            {
                report.insert(
                    format!("rio_return_crossing_{stage}_{key}"),
                    snapshot.map_or(-1, |value| i64::from(value[index])).into(),
                );
            }
        }
        for (stage, motion) in
            [("ceiling", self.door.crossing_ceiling_motion), ("span", self.door.crossing_span_motion)]
        {
            for (index, key) in ["y_direction", "collision", "x_acceleration"].into_iter().enumerate() {
                report.insert(
                    format!("rio_return_crossing_{stage}_{key}"),
                    motion.map_or(-1, |value| i64::from(value[index])).into(),
                );
            }
        }
        report.insert(
            "rio_return_boarding_aim_step".into(),
            self.door.boarding_aim_step.map_or(-1, |step| step as i64).into(),
        );
        for (key, step) in [
            ("release", self.door.boarding_release_step),
            ("jump", self.door.boarding_jump_step),
            ("clear", self.door.boarding_clear_step),
        ] {
            report.insert(format!("rio_return_boarding_{key}_step"), step.map_or(-1, |value| value as i64).into());
        }
        for (stage, snapshot) in [("jump", self.door.boarding_jump), ("clear", self.door.boarding_clear)] {
            for (index, key) in
                ["y", "pose", "y_speed", "y_subspeed", "y_direction", "radius", "y_subposition"].into_iter().enumerate()
            {
                report.insert(
                    format!("rio_return_boarding_{stage}_{key}"),
                    snapshot.map_or(-1, |value| i64::from(value[index])).into(),
                );
            }
        }
        report.insert("rio_return_platform_step".into(), self.door.platform_step.map_or(-1, |step| step as i64).into());
        report
            .insert("rio_return_platform_collision".into(), self.door.platform_collision.map_or(-1, i64::from).into());
        for (index, key) in ["x", "y", "pose", "radius"].into_iter().enumerate() {
            report.insert(
                format!("rio_return_platform_{key}"),
                self.door.platform.map_or(-1, |value| i64::from(value[index])).into(),
            );
        }
        for (index, key) in [
            "x_subposition",
            "y_subposition",
            "x_speed",
            "x_subspeed",
            "x_accel_mode",
            "y_speed",
            "y_subspeed",
            "y_direction",
        ]
        .into_iter()
        .enumerate()
        {
            report.insert(
                format!("rio_return_platform_{key}"),
                self.door.platform_motion.map_or(-1, |value| i64::from(value[index])).into(),
            );
        }
        report.insert(
            "rio_return_lower_floor_step".into(),
            self.door.lower_floor_step.map_or(-1, |step| step as i64).into(),
        );
        for (index, key) in ["x", "y", "pose", "radius"].into_iter().enumerate() {
            report.insert(
                format!("rio_return_lower_floor_{key}"),
                self.door.lower_floor.map_or(-1, |value| i64::from(value[index])).into(),
            );
        }
        for (index, key) in [
            "x_subposition",
            "y_subposition",
            "x_speed",
            "x_subspeed",
            "x_accel_mode",
            "y_speed",
            "y_subspeed",
            "y_direction",
        ]
        .into_iter()
        .enumerate()
        {
            report.insert(
                format!("rio_return_lower_floor_{key}"),
                self.door.lower_floor_motion.map_or(-1, |value| i64::from(value[index])).into(),
            );
        }
        report.insert(
            "rio_return_departure_fall_step".into(),
            self.door.departure_fall_step.map_or(-1, |step| step as i64).into(),
        );
        for (index, key) in ["x", "y", "pose", "radius"].into_iter().enumerate() {
            report.insert(
                format!("rio_return_departure_fall_{key}"),
                self.door.departure_fall.map_or(-1, |value| i64::from(value[index])).into(),
            );
        }
        for (index, key) in [
            "x_subposition",
            "y_subposition",
            "x_speed",
            "x_subspeed",
            "x_accel_mode",
            "y_speed",
            "y_subspeed",
            "y_direction",
        ]
        .into_iter()
        .enumerate()
        {
            report.insert(
                format!("rio_return_departure_fall_{key}"),
                self.door.departure_motion.map_or(-1, |value| i64::from(value[index])).into(),
            );
        }
        for (index, key) in ["x", "y", "pose", "radius"].into_iter().enumerate() {
            report.insert(
                format!("rio_return_door_traversal_landing_{key}"),
                self.door.traversal_landing.map_or(-1, |landing| i64::from(landing[index])).into(),
            );
        }
        for (index, key) in [
            "x_subposition",
            "y_subposition",
            "x_speed",
            "x_subspeed",
            "x_accel_mode",
            "y_speed",
            "y_subspeed",
            "y_direction",
        ]
        .into_iter()
        .enumerate()
        {
            report.insert(
                format!("rio_return_door_approach_fall_{key}"),
                self.door.approach_motion.map_or(-1, |motion| i64::from(motion[index])).into(),
            );
        }
        report.insert("rio_return_door_approach_polls".into(), (self.door.approach_polls as i64).into());
        report.insert(
            "rio_return_door_approach_left_step".into(),
            self.door.approach_left_step.map_or(-1, |step| step as i64).into(),
        );
        report.insert(
            "rio_return_door_approach_fall_step".into(),
            self.door.approach_fall_step.map_or(-1, |step| step as i64).into(),
        );
        for (index, key) in ["x", "y", "pose", "movement", "radius"].into_iter().enumerate() {
            report.insert(
                format!("rio_return_door_approach_fall_{key}"),
                self.door.approach_fall.map_or(-1, |fall| i64::from(fall[index])).into(),
            );
        }
        report.insert("rio_return_door_air_plm_timer".into(), self.door.air_timer.map_or(-1, i64::from).into());
        report.insert("rio_return_door_opening_polls".into(), (self.door.opening_polls as i64).into());
        report.insert("rio_return_door_air_step".into(), self.door.air_step.map_or(-1, |step| step as i64).into());
        report.insert("rio_return_door_opening_stage".into(), (self.door.opening_stage as i64).into());
        for (index, row) in (6..10).enumerate() {
            report.insert(
                format!("rio_return_door_air_bts_row{row}"),
                self.door.air_bts.map_or(-1, |bts| i64::from(bts[index])).into(),
            );
            report.insert(
                format!("rio_return_door_air_row{row}"),
                self.door.air_tiles.map_or(-1, |tiles| i64::from(tiles[index])).into(),
            );
        }
        for (index, key) in ["slot", "header", "block", "instruction", "variable"].into_iter().enumerate() {
            report.insert(
                format!("rio_return_door_air_plm_{key}"),
                self.door.air_plm.map_or(-1, |plm| i64::from(plm[index])).into(),
            );
        }
        report.insert(
            "rio_return_door_allocation_slot".into(),
            self.door.allocation.map_or(-1, |(slot, _)| i64::from(slot)).into(),
        );
        report.insert(
            "rio_return_door_allocation_variable".into(),
            self.door.allocation.map_or(-1, |(_, variable)| i64::from(variable)).into(),
        );
        report.insert("rio_return_phase".into(), "east exit, two ledges and aerial beam probe".into());
        report.insert("rio_return_failure".into(), self.reason.unwrap_or("unfinished return route").into());
        report.insert("rio_return_construction_entry_step".into(), self.exit_step.map_or(-1, |n| n as i64).into());
        report.insert("rio_return_beam_input_step".into(), self.beam_step.map_or(-1, |n| n as i64).into());
        report.insert("rio_return_climb_phase".into(), i64::from(self.climb_phase).into());
        report.insert("rio_return_climb_polls".into(), (self.climb_polls as i64).into());
        report.insert("rio_return_feet_cleared".into(), self.feet_cleared.into());
        report.insert("rio_return_landing_step".into(), self.landing_step.map_or(-1, |n| n as i64).into());
        report.insert("rio_return_ceiling_beam_step".into(), self.ceiling_beam_step.map_or(-1, |n| n as i64).into());
        report.insert("rio_return_ceiling_polls".into(), (self.ceiling_polls as i64).into());
        report.insert(
            "rio_return_ceiling_response_step".into(),
            self.ceiling_response_step.map_or(-1, |n| n as i64).into(),
        );
        report.insert("rio_return_upper_landing_step".into(), self.upper_landing_step.map_or(-1, |n| n as i64).into());
        report.insert("rio_return_upper_anchor_x".into(), self.upper_anchor.map_or(-1, i64::from).into());
        report.insert("rio_return_upper_polls".into(), (self.upper_polls as i64).into());
        report.insert("rio_return_upper_beam_step".into(), self.upper_beam_step.map_or(-1, |n| n as i64).into());
        report
            .insert("rio_return_upper_response_step".into(), self.upper_response_step.map_or(-1, |n| n as i64).into());
        report.insert("rio_return_row10_polls".into(), (self.row10_polls as i64).into());
        report.insert("rio_return_row10_beam_step".into(), self.row10_beam_step.map_or(-1, |n| n as i64).into());
        report
            .insert("rio_return_row10_response_step".into(), self.row10_response_step.map_or(-1, |n| n as i64).into());
        report.insert("rio_return_row8_polls".into(), (self.row8_polls as i64).into());
        report.insert("rio_return_row8_beam_step".into(), self.row8_beam_step.map_or(-1, |n| n as i64).into());
        report.insert("rio_return_row8_response_step".into(), self.row8_response_step.map_or(-1, |n| n as i64).into());
        report.insert("rio_return_survey_polls".into(), (self.survey_polls as i64).into());
        report.insert("rio_return_survey_jump_step".into(), self.survey_jump_step.map_or(-1, |n| n as i64).into());
        report.insert("rio_return_survey_window_step".into(), self.survey_window_step.map_or(-1, |n| n as i64).into());
        report.insert("rio_return_aerial_polls".into(), (self.aerial_polls as i64).into());
        report.insert("rio_return_aerial_beam_step".into(), self.aerial_beam_step.map_or(-1, |n| n as i64).into());
        report.insert(
            "rio_return_aerial_response_step".into(),
            self.aerial_response_step.map_or(-1, |n| n as i64).into(),
        );
        report.insert(
            "rio_return_aerial_creation_pose".into(),
            self.aerial_creation.map_or(-1, |(pose, _)| i64::from(pose)).into(),
        );
        report.insert(
            "rio_return_aerial_creation_y".into(),
            self.aerial_creation.map_or(-1, |(_, y)| i64::from(y)).into(),
        );
        report.insert("rio_return_transfer_polls".into(), (self.transfer_polls as i64).into());
        report.insert(
            "rio_return_transfer_release_step".into(),
            self.transfer_release_step.map_or(-1, |n| n as i64).into(),
        );
        for (key, value) in [
            ("step", self.transfer_contact.as_ref().map_or(-1, |c| c.step as i64)),
            ("pose", self.transfer_contact.as_ref().map_or(-1, |c| i64::from(c.pose))),
            ("radius", self.transfer_contact.as_ref().map_or(-1, |c| i64::from(c.radius))),
            ("x", self.transfer_contact.as_ref().map_or(-1, |c| i64::from(c.x))),
            ("speed", self.transfer_contact.as_ref().map_or(-1, |c| i64::from(c.speed))),
            ("subspeed", self.transfer_contact.as_ref().map_or(-1, |c| i64::from(c.subspeed))),
            ("accel_mode", self.transfer_contact.as_ref().map_or(-1, |c| i64::from(c.accel_mode))),
            ("speed_table", self.transfer_contact.as_ref().map_or(-1, |c| i64::from(c.speed_table))),
            ("x_subposition", self.transfer_contact.as_ref().map_or(-1, |c| i64::from(c.x_subposition))),
            ("extra_speed", self.transfer_contact.as_ref().map_or(-1, |c| i64::from(c.extra_speed))),
            ("extra_subspeed", self.transfer_contact.as_ref().map_or(-1, |c| i64::from(c.extra_subspeed))),
        ] {
            report.insert(format!("rio_return_transfer_contact_{key}"), value.into());
        }
        report.insert("rio_return_support_polls".into(), (self.support_polls as i64).into());
        report.insert(
            "rio_return_support_release_step".into(),
            self.support_release_step.map_or(-1, |n| n as i64).into(),
        );
        report.insert(
            "rio_return_support_standing_step".into(),
            self.support_standing_step.map_or(-1, |n| n as i64).into(),
        );
        report.insert("rio_return_brake_polls".into(), (self.brake_polls as i64).into());
        for (key, step) in
            [("aim", self.third_aim_step), ("beam", self.third_beam_step), ("response", self.third_response_step)]
        {
            report.insert(format!("rio_return_third_{key}_step"), step.map_or(-1, |n| n as i64).into());
        }
        report.insert("rio_return_third_beam_polls".into(), (self.third_beam_polls as i64).into());
        for (i, col) in [6, 7, 8].into_iter().enumerate() {
            report.insert(
                format!("rio_return_third_before_c{col}"),
                self.third_neighbors.map_or(-1, |tiles| i64::from(tiles[i])).into(),
            );
            report.insert(
                format!("rio_return_third_after_c{col}"),
                self.third_after.map_or(-1, |tiles| i64::from(tiles[i])).into(),
            );
            report.insert(
                format!("rio_return_third_before_bts_c{col}"),
                self.third_before_bts.map_or(-1, |bts| i64::from(bts[i])).into(),
            );
            report.insert(
                format!("rio_return_third_after_bts_c{col}"),
                self.third_after_bts.map_or(-1, |bts| i64::from(bts[i])).into(),
            );
        }
        for (i, key) in ["header", "block", "instruction", "variable"].into_iter().enumerate() {
            report.insert(
                format!("rio_return_third_plm_{key}"),
                self.third_plm.map_or(-1, |plm| i64::from(plm[i])).into(),
            );
        }
        report.insert("rio_return_third_plm_slot".into(), self.third_plm_slot.map_or(-1, i64::from).into());
        report.insert("rio_return_upper_transfer_polls".into(), (self.upper_transfer_polls as i64).into());
        report.insert("rio_return_upper_rest_step".into(), self.upper_rest_step.map_or(-1, |step| step as i64).into());
        report.insert("rio_return_door_polls".into(), (self.door.polls as i64).into());
        for (key, step) in
            [("shot", self.door.shot_step), ("release", self.door.release_step), ("response", self.door.response_step)]
        {
            report.insert(format!("rio_return_door_{key}_step"), step.map_or(-1, |step| step as i64).into());
        }
        for (index, row) in [6, 7, 8, 9].into_iter().enumerate() {
            for (key, value) in [
                ("before", self.door.before.map_or(-1, |tiles| i64::from(tiles[index]))),
                ("after", self.door.after.map_or(-1, |tiles| i64::from(tiles[index]))),
                ("before_bts", self.door.before_bts.map_or(-1, |bts| i64::from(bts[index]))),
                ("after_bts", self.door.after_bts.map_or(-1, |bts| i64::from(bts[index]))),
            ] {
                report.insert(format!("rio_return_door_{key}_row{row}"), value.into());
            }
        }
        for (index, key) in ["slot", "header", "block", "instruction", "variable"].into_iter().enumerate() {
            report.insert(
                format!("rio_return_door_plm_{key}"),
                self.door.plm.map_or(-1, |plm| i64::from(plm[index])).into(),
            );
        }
        for (index, key) in ["slot", "x", "y", "direction"].into_iter().enumerate() {
            report.insert(
                format!("rio_return_door_projectile_{key}"),
                self.door.projectile.map_or(-1, |projectile| i64::from(projectile[index])).into(),
            );
        }
        for (key, value) in [
            ("step", self.upper_hold.map_or(-1, |(step, _, _)| step as i64)),
            ("x", self.upper_hold.map_or(-1, |(_, x, _)| i64::from(x))),
            ("x_subposition", self.upper_hold.map_or(-1, |(_, _, sub)| i64::from(sub))),
            ("left_release_step", self.upper_left_release_step.map_or(-1, |step| step as i64)),
        ] {
            report.insert(format!("rio_return_upper_hold_{key}"), value.into());
        }
        for (index, key) in
            ["speed", "subspeed", "extra_speed", "extra_subspeed", "accel_mode", "speed_table"].into_iter().enumerate()
        {
            report.insert(
                format!("rio_return_upper_hold_{key}"),
                self.upper_hold_motion.map_or(-1, |motion| i64::from(motion[index])).into(),
            );
        }
        for (key, step) in
            [("release", self.upper_release_step), ("jump", self.upper_jump_step), ("feet", self.upper_feet_step)]
        {
            report.insert(format!("rio_return_upper_transfer_{key}_step"), step.map_or(-1, |n| n as i64).into());
        }
        for (key, value) in [
            ("step", self.upper_contact.as_ref().map_or(-1, |c| c.step as i64)),
            ("pose", self.upper_contact.as_ref().map_or(-1, |c| i64::from(c.pose))),
            ("radius", self.upper_contact.as_ref().map_or(-1, |c| i64::from(c.radius))),
            ("x", self.upper_contact.as_ref().map_or(-1, |c| i64::from(c.x))),
            ("x_subposition", self.upper_contact.as_ref().map_or(-1, |c| i64::from(c.x_subposition))),
            ("speed", self.upper_contact.as_ref().map_or(-1, |c| i64::from(c.speed))),
            ("subspeed", self.upper_contact.as_ref().map_or(-1, |c| i64::from(c.subspeed))),
            ("extra_speed", self.upper_contact.as_ref().map_or(-1, |c| i64::from(c.extra_speed))),
            ("extra_subspeed", self.upper_contact.as_ref().map_or(-1, |c| i64::from(c.extra_subspeed))),
            ("accel_mode", self.upper_contact.as_ref().map_or(-1, |c| i64::from(c.accel_mode))),
            ("speed_table", self.upper_contact.as_ref().map_or(-1, |c| i64::from(c.speed_table))),
        ] {
            report.insert(format!("rio_return_upper_contact_{key}"), value.into());
        }
        report.insert("rio_return_brake_wall_step".into(), self.brake_wall_step.map_or(-1, |n| n as i64).into());
        report.insert("rio_return_brake_aim_step".into(), self.brake_aim_step.map_or(-1, |n| n as i64).into());
        report.insert("rio_return_brake_release_step".into(), self.brake_release_step.map_or(-1, |n| n as i64).into());
        report.insert("rio_return_brake_rest_step".into(), self.brake_rest.map_or(-1, |(n, _, _)| n as i64).into());
        report.insert("rio_return_brake_rest_x".into(), self.brake_rest.map_or(-1, |(_, x, _)| i64::from(x)).into());
        report.insert(
            "rio_return_brake_rest_subposition".into(),
            self.brake_rest.map_or(-1, |(_, _, n)| i64::from(n)).into(),
        );
        for (key, value) in [
            ("step", self.support_span.as_ref().map_or(-1, |c| c.step as i64)),
            ("pose", self.support_span.as_ref().map_or(-1, |c| i64::from(c.pose))),
            ("radius", self.support_span.as_ref().map_or(-1, |c| i64::from(c.radius))),
            ("x", self.support_span.as_ref().map_or(-1, |c| i64::from(c.x))),
            ("speed", self.support_span.as_ref().map_or(-1, |c| i64::from(c.speed))),
            ("subspeed", self.support_span.as_ref().map_or(-1, |c| i64::from(c.subspeed))),
            ("accel_mode", self.support_span.as_ref().map_or(-1, |c| i64::from(c.accel_mode))),
            ("speed_table", self.support_span.as_ref().map_or(-1, |c| i64::from(c.speed_table))),
            ("x_subposition", self.support_span.as_ref().map_or(-1, |c| i64::from(c.x_subposition))),
            ("extra_speed", self.support_span.as_ref().map_or(-1, |c| i64::from(c.extra_speed))),
            ("extra_subspeed", self.support_span.as_ref().map_or(-1, |c| i64::from(c.extra_subspeed))),
        ] {
            report.insert(format!("rio_return_support_span_{key}"), value.into());
        }
    }
}

fn aerial_ready(state: &State, ram: &[u8]) -> bool {
    // Captured before Alfa creates the projectile; post-shot cooldown is expected.
    (167..176).contains(&state.y)
        && word(ram, 0xccc) == 0
        && word(ram, 0xcce) < 5
        && state.joy1_current & word(ram, 0x9b2) == 0
        && word(ram, 0xb5e) == 0
        && word(ram, 0x10002 + 2 * (10 * 16 + 7)) == 0xc19a
        && ram[0x16402 + 10 * 16 + 7] == 4
}

fn grounded(ram: &[u8]) -> bool {
    word(ram, 0xb2e) == 0 && word(ram, 0xb2c) == 0 && word(ram, 0xb36) == 0 && word(ram, 0xb00) != 0
}

fn first_ledge_support(state: &State, ram: &[u8]) -> bool {
    // This admitted endpoint is row21 of the observed 16-block-wide room.
    // Require solid terrain beneath both sides of the actual horizontal span.
    let radius = word(ram, 0xafe);
    if radius == 0 || radius > 16 || state.x < radius {
        return false;
    }
    [state.x - radius, state.x.saturating_add(radius)].into_iter().all(|x| {
        let column = usize::from(x / 16);
        column < 8 && word(ram, 0x10002 + 2 * (21 * 16 + column)) >> 12 == 8
    })
}

fn upper_ledge_support(state: &State, ram: &[u8]) -> bool {
    let radius = word(ram, 0xafe);
    radius == 5
        && [state.x.saturating_sub(radius), state.x.saturating_add(radius)].into_iter().all(|x| {
            let column = usize::from(x / 16);
            (8..10).contains(&column) && word(ram, 0x10002 + 2 * (16 * 16 + column)) >> 12 == 0xc
        })
}

#[cfg(any(test, feature = "private-checks"))]
mod tests {
    use super::*;

    #[cfg_attr(test, test)]
    pub(super) fn wrong_prefix_is_terminal_and_never_emits_input() {
        let mut route = Return::default();
        let state = State { host_steps: PREFIX, game_state: 8, area: 1, room: 0xa107, ..State::default() };
        let ram = vec![0; 131072];
        route.observe(&state, &ram);
        assert_eq!(route.failure(), Some("return does not start at exact missile prefix"));
        route.observe(&State { host_steps: PREFIX + 1, room: 0x9f64, ..state }, &ram);
        assert_eq!(route.failure(), Some("return does not start at exact missile prefix"));
        assert_eq!(route.buttons(&state, &ram), 0);
    }

    #[cfg_attr(test, test)]
    pub(super) fn beam_request_waits_for_consumed_edge_then_releases() {
        let state = State {
            host_steps: PREFIX + 20,
            polls: 100,
            game_state: 8,
            area: 1,
            room: 0xa107,
            x: 219,
            y: 139,
            effective_buttons: 0x200,
            ..State::default()
        };
        let mut ram = vec![0; 131072];
        for (at, value) in [(0x9c2, 99u16), (0x9c6, 5), (0x9c8, 5), (0x9b2, 0x40), (0x9b6, 0x8000)] {
            ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
        }
        let mut route =
            Return { previous: Some(state), prefix_verified: true, stationary_polls: 12, ..Return::default() };
        let waiting = State { host_steps: state.host_steps + 1, ..state };
        route.observe(&waiting, &ram);
        assert_eq!(route.beam_step, None);
        assert_eq!(route.buttons(&waiting, &ram) & 0x200, 0x200);
        let consumed =
            State { host_steps: waiting.host_steps + 1, polls: 101, joy1_current: 0x40, joy1_new: 0x40, ..waiting };
        route.observe(&consumed, &ram);
        assert_eq!(route.beam_step, Some(consumed.host_steps));
        assert_eq!(route.buttons(&consumed, &ram) & 0x200, 0);
    }

    #[cfg_attr(test, test)]
    pub(super) fn first_ledge_steering_waits_until_the_feet_clear_terrain() {
        let state = State {
            host_steps: PREFIX + 200,
            polls: 100,
            game_state: 8,
            area: 1,
            room: 0x9f11,
            x: 144,
            y: 318,
            ..State::default()
        };
        let mut ram = vec![0; 131072];
        for (at, value) in [
            (0x9c2, 99u16),
            (0x9c6, 5),
            (0x9c8, 5),
            (0x9b2, 0x40),
            (0x9b4, 0x80),
            (0xb00, 19),
            (0xb2e, 1),
            (0x7a5, 16),
            (0x7a7, 32),
        ] {
            ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
        }
        ram[0xa1f] = 2; // Observed native normal jump changes radius21 to19.
        let mut route = Return {
            previous: Some(state),
            prefix_verified: true,
            exit_step: Some(state.host_steps),
            climb_phase: 2,
            ..Return::default()
        };
        let blocked = State { host_steps: state.host_steps + 1, polls: 101, ..state };
        route.observe(&blocked, &ram);
        assert!(!route.feet_cleared);
        assert_eq!(route.buttons(&blocked, &ram) & 0xc0, 0);
        let cleared = State { host_steps: blocked.host_steps + 1, polls: 102, y: 317, ..blocked };
        route.observe(&cleared, &ram);
        assert!(route.feet_cleared);
        assert_eq!(route.buttons(&cleared, &ram) & 0xc0, 0x40);
    }

    fn construction_state() -> (State, Vec<u8>, Return) {
        let state = State {
            host_steps: PREFIX + 200,
            polls: 100,
            game_state: 8,
            area: 1,
            room: 0x9f11,
            x: 112,
            y: 315,
            ..State::default()
        };
        let mut ram = vec![0; 131072];
        for (at, value) in [
            (0x9c2, 99u16),
            (0x9c6, 5),
            (0x9c8, 5),
            (0x9b2, 0x40),
            (0x9b4, 0x80),
            (0xafe, 5),
            (0xb00, 21),
            (0x7a5, 16),
            (0x7a7, 32),
        ] {
            ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
        }
        let route = Return {
            previous: Some(state),
            prefix_verified: true,
            exit_step: Some(state.host_steps),
            ..Return::default()
        };
        (state, ram, route)
    }

    #[cfg_attr(test, test)]
    pub(super) fn first_ledge_clearance_refuses_morph_and_spin_jump_domains() {
        let (state, mut ram, mut route) = construction_state();
        route.climb_phase = 2;
        let mut next = State { y: 300, ..state };
        for (movement, radius) in [(4u8, 7u16), (3, 12)] {
            ram[0xa1f] = movement;
            ram[0xb00..0xb02].copy_from_slice(&radius.to_le_bytes());
            next.host_steps += 1;
            route.observe(&next, &ram);
            assert!(!route.feet_cleared);
            assert_eq!(route.buttons(&next, &ram) & 0xc0, 0);
        }
    }

    #[cfg_attr(test, test)]
    pub(super) fn morph_request_survives_unpolled_steps_and_releases_consumed_down() {
        let (state, ram, mut route) = construction_state();
        let waiting = State { host_steps: state.host_steps + 1, effective_buttons: 0x20, ..state };
        route.observe(&waiting, &ram);
        assert_eq!(route.buttons(&waiting, &ram), 0x20);
        let consumed =
            State { host_steps: waiting.host_steps + 1, polls: 101, joy1_current: 0x0400, joy1_new: 0x0400, ..waiting };
        route.observe(&consumed, &ram);
        assert_eq!(route.buttons(&consumed, &ram), 0);
        let unpolled_release = State { host_steps: consumed.host_steps + 1, effective_buttons: 0, ..consumed };
        route.observe(&unpolled_release, &ram);
        assert_eq!(route.buttons(&unpolled_release, &ram), 0);
        let released = State {
            host_steps: unpolled_release.host_steps + 1,
            polls: 102,
            joy1_current: 0,
            joy1_new: 0,
            ..unpolled_release
        };
        route.observe(&released, &ram);
        assert_eq!(route.buttons(&released, &ram), 0x20);
    }

    #[cfg_attr(test, test)]
    pub(super) fn construction_pads_refuse_shoot_aliases() {
        let (state, mut ram, mut route) = construction_state();
        for (phase, movement, x, pad) in
            [(0, 0, 112, 0x20u16), (0, 4, 112, 0x80), (1, 4, 144, 0x10), (3, 0, 144, 0x40), (3, 0, 100, 0x80)]
        {
            route.climb_phase = phase;
            ram[0xa1f] = movement;
            ram[0x9b2..0x9b4].copy_from_slice(&pad.reverse_bits().to_le_bytes());
            assert_eq!(route.buttons(&State { x, ..state }, &ram), 0);
        }
    }

    #[cfg_attr(test, test)]
    pub(super) fn unmorph_crouch_waits_for_standing_before_new_jump() {
        let (state, mut ram, mut route) = construction_state();
        route.climb_phase = 1;
        ram[0xa1f] = 5;
        ram[0xb00..0xb02].copy_from_slice(&16u16.to_le_bytes());
        let crouched = State {
            host_steps: state.host_steps + 1,
            polls: 101,
            x: 147,
            y: 400,
            effective_buttons: 0x10,
            joy1_current: 0x0800,
            joy1_new: 0x0800,
            ..state
        };
        route.observe(&crouched, &ram);
        assert_eq!(route.climb_phase, 1, "unmorph reaches crouch before standing");
        assert_eq!(route.buttons(&crouched, &ram), 0, "release consumed Up");
        let waiting = State { host_steps: crouched.host_steps + 1, effective_buttons: 0, ..crouched };
        route.observe(&waiting, &ram);
        assert_eq!(route.buttons(&waiting, &ram), 0, "retain release across unpolled step");
        let released =
            State { host_steps: waiting.host_steps + 1, polls: 102, joy1_current: 0, joy1_new: 0, ..waiting };
        route.observe(&released, &ram);
        assert_eq!(route.buttons(&released, &ram), 0x10, "new Up requests standing");
        ram[0xa1f] = 0;
        ram[0xb00..0xb02].copy_from_slice(&21u16.to_le_bytes());
        let standing = State {
            host_steps: released.host_steps + 1,
            polls: 103,
            y: 395,
            effective_buttons: 0x10,
            joy1_current: 0x0800,
            joy1_new: 0x0800,
            ..released
        };
        route.observe(&standing, &ram);
        assert_eq!(route.climb_phase, 2);
        assert_eq!(route.buttons(&standing, &ram), 0x100);
    }

    #[cfg_attr(test, test)]
    pub(super) fn landing_requires_standing_radius_and_solid_support() {
        let (state, mut ram, mut route) = construction_state();
        route.climb_phase = 3;
        for column in [6, 7] {
            let at = 0x10002 + 2 * (21 * 16 + column);
            ram[at..at + 2].copy_from_slice(&0x819fu16.to_le_bytes());
        }
        let target = 0x10002 + 2 * (16 * 16 + 6);
        ram[target..target + 2].copy_from_slice(&0xc19au16.to_le_bytes());
        ram[0xb2e] = 1;
        let airborne = State { host_steps: state.host_steps + 1, ..state };
        route.observe(&airborne, &ram);
        assert_eq!(route.failure(), None);
        ram[0xb2e] = 0;
        ram[0xb00..0xb02].copy_from_slice(&7u16.to_le_bytes());
        let small_radius = State { host_steps: airborne.host_steps + 1, ..state };
        route.observe(&small_radius, &ram);
        assert_eq!(route.failure(), None);
        ram[0xb00..0xb02].copy_from_slice(&21u16.to_le_bytes());
        ram[0x10002 + 2 * (21 * 16 + 7)..0x10004 + 2 * (21 * 16 + 7)].fill(0);
        let unsupported = State { host_steps: small_radius.host_steps + 1, ..state };
        route.observe(&unsupported, &ram);
        assert_eq!(route.failure(), None);
        let at = 0x10002 + 2 * (21 * 16 + 7);
        ram[at..at + 2].copy_from_slice(&0x819fu16.to_le_bytes());
        route.observe(&State { host_steps: unsupported.host_steps + 1, ..state }, &ram);
        assert_eq!(route.failure(), None);
        assert_eq!(route.climb_phase, 4);
        assert_eq!(route.landing_step, Some(unsupported.host_steps + 1));
    }

    #[cfg_attr(test, test)]
    pub(super) fn ceiling_shot_waits_for_aim_and_consumption_then_observes_solid_response() {
        let (state, mut ram, mut route) = construction_state();
        route.climb_phase = 4;
        route.landing_step = Some(state.host_steps);
        for column in [6, 7] {
            let at = 0x10002 + 2 * (21 * 16 + column);
            ram[at..at + 2].copy_from_slice(&0x819fu16.to_le_bytes());
        }
        let target = 0x10002 + 2 * (16 * 16 + 6);
        ram[target..target + 2].copy_from_slice(&0xc19au16.to_le_bytes());
        assert_eq!(route.buttons(&state, &ram), 0x10);
        ram[0xa1c..0xa1e].copy_from_slice(&4u16.to_le_bytes());
        let aimed = State { host_steps: state.host_steps + 1, polls: 101, joy1_current: 0x0800, ..state };
        route.observe(&aimed, &ram);
        assert_eq!(route.buttons(&aimed, &ram), 0x210);
        ram[0xa1c..0xa1e].copy_from_slice(&3u16.to_le_bytes());
        assert_eq!(route.buttons(&aimed, &ram), 0, "changed facing refuses fixed-column shot");
        ram[0xa1c..0xa1e].copy_from_slice(&4u16.to_le_bytes());
        let waiting = State { host_steps: aimed.host_steps + 1, effective_buttons: 0x210, ..aimed };
        route.observe(&waiting, &ram);
        assert_eq!(route.ceiling_beam_step, None);
        assert_eq!(route.buttons(&waiting, &ram), 0x210);
        let consumed =
            State { host_steps: waiting.host_steps + 1, polls: 102, joy1_current: 0x0840, joy1_new: 0x0040, ..waiting };
        route.observe(&consumed, &ram);
        assert_eq!(route.ceiling_beam_step, Some(consumed.host_steps));
        assert_eq!(route.buttons(&consumed, &ram), 0x10);
        ram[target..target + 2].copy_from_slice(&0x8052u16.to_le_bytes());
        route.observe(&State { host_steps: consumed.host_steps + 1, ..consumed }, &ram);
        assert_eq!(route.failure(), Some("return ceiling beam response has no full jump opening"));
        assert_eq!(word(&ram, target) >> 12, 8, "response is still solid, not corridor clearance");
        let mut same_poll = Return {
            previous: Some(waiting),
            prefix_verified: true,
            landing_step: Some(state.host_steps),
            climb_phase: 5,
            ..Return::default()
        };
        same_poll.observe(&consumed, &ram);
        assert_eq!(same_poll.ceiling_beam_step, Some(consumed.host_steps));
        assert_eq!(same_poll.failure(), Some("return ceiling beam response has no full jump opening"));
    }

    #[cfg_attr(test, test)]
    pub(super) fn ceiling_probe_refuses_wrong_target_and_bounds_missing_response() {
        let (state, mut ram, mut route) = construction_state();
        route.climb_phase = 4;
        route.landing_step = Some(state.host_steps);
        for column in [6, 7] {
            let at = 0x10002 + 2 * (21 * 16 + column);
            ram[at..at + 2].copy_from_slice(&0x819fu16.to_le_bytes());
        }
        let target = 0x10002 + 2 * (16 * 16 + 6);
        ram[target..target + 2].copy_from_slice(&0x00ffu16.to_le_bytes());
        let next = State { host_steps: state.host_steps + 1, polls: 101, ..state };
        route.observe(&next, &ram);
        assert_eq!(route.failure(), Some("return ceiling beam target differs before the shot"));
        assert_eq!(route.buttons(&next, &ram), 0);
        ram[target..target + 2].copy_from_slice(&0xc19au16.to_le_bytes());
        let mut wrong_bts = Return {
            previous: Some(state),
            prefix_verified: true,
            landing_step: Some(state.host_steps),
            climb_phase: 4,
            ..Return::default()
        };
        ram[0x16402 + 16 * 16 + 6] = 1;
        wrong_bts.observe(&next, &ram);
        assert_eq!(wrong_bts.failure(), Some("return ceiling beam target differs before the shot"));
        ram[0x16402 + 16 * 16 + 6] = 0;
        let mut bounded = Return {
            previous: Some(state),
            prefix_verified: true,
            landing_step: Some(state.host_steps),
            climb_phase: 6,
            ceiling_polls: 119,
            ceiling_beam_step: Some(state.host_steps),
            ..Return::default()
        };
        bounded.observe(&next, &ram);
        assert_eq!(bounded.failure(), Some("return ceiling beam exceeded 120 consumed polls"));
    }

    #[cfg_attr(test, test)]
    pub(super) fn upper_ledge_jump_waits_for_opening_and_feet_before_steering() {
        let (state, mut ram, mut route) = construction_state();
        route.climb_phase = 6;
        route.landing_step = Some(state.host_steps);
        route.ceiling_beam_step = Some(state.host_steps);
        for column in [6, 7] {
            let at = 0x10002 + 2 * (21 * 16 + column);
            ram[at..at + 2].copy_from_slice(&0x819fu16.to_le_bytes());
            let at = 0x10002 + 2 * (16 * 16 + column);
            ram[at..at + 2].copy_from_slice(&0x0053u16.to_le_bytes());
        }
        let opening = State { host_steps: state.host_steps + 1, polls: 101, ..state };
        route.observe(&opening, &ram);
        assert_eq!(route.climb_phase, 7);
        assert_eq!(route.buttons(&opening, &ram), 0x100);
        let mut closed = Return {
            previous: Some(opening),
            prefix_verified: true,
            landing_step: Some(state.host_steps),
            climb_phase: 7,
            ..Return::default()
        };
        let tile = 0x10002 + 2 * (16 * 16 + 7);
        ram[tile..tile + 2].copy_from_slice(&0xc19au16.to_le_bytes());
        closed.observe(&State { host_steps: opening.host_steps + 1, polls: 102, ..opening }, &ram);
        assert_eq!(closed.failure(), Some("return second ledge opening closed before feet clearance"));
        let mut simultaneous = Return {
            previous: Some(opening),
            prefix_verified: true,
            landing_step: Some(state.host_steps),
            climb_phase: 7,
            ..Return::default()
        };
        ram[0xa1f] = 2;
        ram[0xb00..0xb02].copy_from_slice(&19u16.to_le_bytes());
        simultaneous.observe(&State { host_steps: opening.host_steps + 1, polls: 102, y: 237, ..opening }, &ram);
        assert_eq!(simultaneous.failure(), Some("return second ledge opening closed before feet clearance"));
        ram[tile..tile + 2].copy_from_slice(&0x0053u16.to_le_bytes());
        ram[0xa1f] = 2;
        ram[0xb00..0xb02].copy_from_slice(&19u16.to_le_bytes());
        ram[0xb2e] = 1;
        let blocked = State { host_steps: opening.host_steps + 1, polls: 102, y: 238, ..opening };
        route.observe(&blocked, &ram);
        assert_eq!(route.climb_phase, 7);
        assert_eq!(route.buttons(&blocked, &ram), 0x100);
        let cleared = State { host_steps: blocked.host_steps + 1, polls: 103, y: 237, ..blocked };
        route.observe(&cleared, &ram);
        assert_eq!(route.climb_phase, 8);
        assert_eq!(route.buttons(&cleared, &ram), 0x180);
        ram[0xa1f] = 0;
        ram[0xb00..0xb02].copy_from_slice(&21u16.to_le_bytes());
        ram[0xb2e] = 0;
        for column in [8, 9] {
            let at = 0x10002 + 2 * (16 * 16 + column);
            ram[at..at + 2].copy_from_slice(&0xc19au16.to_le_bytes());
        }
        let landed = State { host_steps: cleared.host_steps + 1, polls: 104, x: 144, y: 235, ..cleared };
        ram[0xb2e] = 1;
        route.observe(&landed, &ram);
        assert_eq!(route.upper_landing_step, None);
        ram[0xb2e] = 0;
        ram[0xb00..0xb02].copy_from_slice(&19u16.to_le_bytes());
        let small = State { host_steps: landed.host_steps + 1, polls: 105, ..landed };
        route.observe(&small, &ram);
        assert_eq!(route.upper_landing_step, None);
        ram[0xb00..0xb02].copy_from_slice(&21u16.to_le_bytes());
        let support = 0x10002 + 2 * (16 * 16 + 9);
        ram[support..support + 2].fill(0);
        let unsupported = State { host_steps: small.host_steps + 1, polls: 106, ..small };
        route.observe(&unsupported, &ram);
        assert_eq!(route.upper_landing_step, None);
        ram[support..support + 2].copy_from_slice(&0xc19au16.to_le_bytes());
        let landed = State { host_steps: unsupported.host_steps + 1, polls: 107, ..unsupported };
        route.observe(&landed, &ram);
        assert_eq!(route.upper_landing_step, Some(landed.host_steps));
        assert_eq!(route.failure(), None);
    }

    #[cfg_attr(test, test)]
    pub(super) fn second_ledge_settling_corrects_the_observed_rightward_overshoot() {
        let (state, mut ram, mut route) = construction_state();
        route.climb_phase = 8;
        let overshot = State { x: 162, y: 235, ..state };
        assert_eq!(route.buttons(&overshot, &ram), 0x140);
        assert_eq!(route.buttons(&State { x: 144, ..overshot }, &ram), 0x100);
        assert_eq!(route.buttons(&State { x: 143, ..overshot }, &ram), 0x180);
        ram[0x9b2..0x9b4].copy_from_slice(&0x0200u16.to_le_bytes());
        assert_eq!(route.buttons(&overshot, &ram), 0);
    }

    #[cfg_attr(test, test)]
    pub(super) fn second_ledge_ground_contact_releases_jump_through_running_and_turning() {
        let (state, mut ram, mut route) = construction_state();
        route.climb_phase = 8;
        ram[0xa1f] = 1;
        for column in [8, 9] {
            let at = 0x10002 + 2 * (16 * 16 + column);
            ram[at..at + 2].copy_from_slice(&0xc19au16.to_le_bytes());
        }
        let running =
            State { host_steps: state.host_steps + 1, polls: 101, x: 133, y: 235, joy1_current: 0x0180, ..state };
        route.observe(&running, &ram);
        assert_eq!(route.upper_landing_step, None);
        assert_eq!(route.buttons(&running, &ram), 0x80);
        let unpolled = State { host_steps: running.host_steps + 1, ..running };
        route.observe(&unpolled, &ram);
        assert_eq!(route.buttons(&unpolled, &ram), 0x80);
        ram[0xa1f] = 0x0e;
        let turning =
            State { host_steps: unpolled.host_steps + 1, polls: 102, x: 149, joy1_current: 0x0200, ..running };
        route.observe(&turning, &ram);
        assert_eq!(route.upper_landing_step, None);
        assert_eq!(route.buttons(&turning, &ram), 0x40);
        ram[0xa1f] = 3;
        ram[0xb00..0xb02].copy_from_slice(&12u16.to_le_bytes());
        ram[0xb2e..0xb30].copy_from_slice(&1u16.to_le_bytes());
        let airborne = State { host_steps: turning.host_steps + 1, polls: 103, y: 228, ..turning };
        route.observe(&airborne, &ram);
        assert_eq!(route.buttons(&airborne, &ram), 0x40);
    }

    #[cfg_attr(test, test)]
    pub(super) fn second_ledge_jump_release_requires_supported_ground_contact() {
        for invalid in 0..4 {
            let (state, mut ram, mut route) = construction_state();
            route.climb_phase = 8;
            ram[0xa1f] = 1;
            for column in [8, 9] {
                let at = 0x10002 + 2 * (16 * 16 + column);
                ram[at..at + 2].copy_from_slice(&0xc19au16.to_le_bytes());
            }
            match invalid {
                0 => ram[0xa1f] = 2,
                1 => ram[0xb00..0xb02].copy_from_slice(&19u16.to_le_bytes()),
                2 => ram[0xb2e..0xb30].copy_from_slice(&1u16.to_le_bytes()),
                _ => ram[0x10002 + 2 * (16 * 16 + 8)..0x10004 + 2 * (16 * 16 + 8)].fill(0),
            }
            let next = State { host_steps: state.host_steps + 1, polls: 101, x: 133, y: 235, ..state };
            route.observe(&next, &ram);
            assert_eq!(route.climb_phase, 8);
            assert_eq!(route.buttons(&next, &ram), 0x180);
        }
    }

    fn upper_beam_state() -> (State, Vec<u8>, Return) {
        let (state, mut ram, mut route) = construction_state();
        let state = State { x: 148, y: 235, joy1_current: 0x0200, ..state };
        ram[0xa1c..0xa1e].copy_from_slice(&2u16.to_le_bytes());
        ram[0xb48..0xb4a].copy_from_slice(&0xc000u16.to_le_bytes());
        for (row, columns) in [(16, vec![8, 9]), (12, vec![8, 9])] {
            for column in columns {
                let at = 0x10002 + 2 * (16 * row + column);
                ram[at..at + 2].copy_from_slice(&0xc19au16.to_le_bytes());
            }
        }
        route.previous = Some(state);
        route.upper_landing_step = Some(state.host_steps);
        route.climb_phase = 10;
        (state, ram, route)
    }

    #[cfg_attr(test, test)]
    pub(super) fn upper_ceiling_beam_waits_for_rest_aim_and_consumed_shot_before_same_poll_reaction() {
        let (state, mut ram, mut route) = upper_beam_state();
        assert_eq!(route.buttons(&state, &ram), 0);
        let waiting = State { host_steps: state.host_steps + 1, ..state };
        route.observe(&waiting, &ram);
        assert_eq!(route.upper_anchor, None);
        assert_eq!(route.buttons(&waiting, &ram), 0);
        ram[0xb48..0xb4a].fill(0);
        let resting = State { host_steps: waiting.host_steps + 1, polls: 101, joy1_current: 0, ..waiting };
        route.observe(&resting, &ram);
        assert_eq!(route.upper_anchor, Some(148));
        assert_eq!(route.buttons(&resting, &ram), 0x10);
        let unpolled = State { host_steps: resting.host_steps + 1, ..resting };
        route.observe(&unpolled, &ram);
        assert_eq!(route.upper_beam_step, None);
        ram[0xa1c..0xa1e].copy_from_slice(&4u16.to_le_bytes());
        let aimed = State { host_steps: unpolled.host_steps + 1, polls: 102, joy1_current: 0x0800, ..unpolled };
        route.observe(&aimed, &ram);
        assert_eq!(route.buttons(&aimed, &ram), 0x210);
        let pending = State { host_steps: aimed.host_steps + 1, effective_buttons: 0x210, ..aimed };
        route.observe(&pending, &ram);
        assert_eq!(route.upper_beam_step, None);
        assert_eq!(route.buttons(&pending, &ram), 0x210);
        let tile = 0x10002 + 2 * (12 * 16 + 9);
        ram[tile..tile + 2].copy_from_slice(&0x0053u16.to_le_bytes());
        let reaction = State {
            host_steps: pending.host_steps + 1,
            polls: 103,
            effective_buttons: 0x210,
            joy1_current: 0x0840,
            joy1_new: 0x0040,
            ..aimed
        };
        route.observe(&reaction, &ram);
        assert_eq!(route.upper_beam_step, Some(reaction.host_steps));
        assert_eq!(route.upper_response_step, Some(reaction.host_steps));
        assert_eq!(route.failure(), None);
        assert_eq!(route.buttons(&reaction, &ram), 0x10);
    }

    #[cfg_attr(test, test)]
    pub(super) fn upper_ceiling_beam_refuses_wrong_target_facing_and_terminal_bound() {
        for invalid in 0..7 {
            let (state, mut ram, mut route) = upper_beam_state();
            route.climb_phase = 12;
            route.upper_anchor = Some(148);
            ram[0xa1c..0xa1e].copy_from_slice(&4u16.to_le_bytes());
            let mut next = State { host_steps: state.host_steps + 1, polls: 101, joy1_current: 0x0800, ..state };
            match invalid {
                0 => ram[0x16402 + 12 * 16 + 9] = 1,
                1 => {
                    ram[0xa1c..0xa1e].copy_from_slice(&3u16.to_le_bytes());
                    assert_eq!(route.buttons(&next, &ram), 0);
                    next.effective_buttons = 0x210;
                    next.joy1_current = 0x0840;
                    next.joy1_new = 0x0040;
                }
                2 => {
                    route.upper_polls = 119;
                    route.upper_beam_step = Some(state.host_steps);
                    route.climb_phase = 13;
                    let tile = 0x10002 + 2 * (12 * 16 + 9);
                    ram[tile..tile + 2].fill(0);
                }
                3 => ram[0x9d2..0x9d4].copy_from_slice(&1u16.to_le_bytes()),
                4 => {
                    let support = 0x10002 + 2 * (16 * 16 + 9);
                    ram[support..support + 2].fill(0);
                }
                5 => next.x = 147,
                _ => {
                    let tile = 0x10002 + 2 * (12 * 16 + 9);
                    ram[tile..tile + 2].fill(0);
                }
            }
            route.observe(&next, &ram);
            assert_eq!(route.upper_response_step, None);
            if invalid == 1 {
                assert_eq!(route.upper_beam_step, None);
            } else {
                assert!(route.failure().is_some());
            }
        }
    }

    #[cfg_attr(test, test)]
    pub(super) fn upper_ceiling_consumed_shot_releases_before_later_reaction() {
        let (state, mut ram, mut route) = upper_beam_state();
        route.climb_phase = 12;
        route.upper_anchor = Some(148);
        ram[0xa1c..0xa1e].copy_from_slice(&4u16.to_le_bytes());
        let shot = State {
            host_steps: state.host_steps + 1,
            polls: 101,
            effective_buttons: 0x210,
            joy1_current: 0x0840,
            joy1_new: 0x0040,
            ..state
        };
        route.observe(&shot, &ram);
        assert_eq!(route.upper_beam_step, Some(shot.host_steps));
        assert_eq!(route.upper_response_step, None);
        assert_eq!(route.buttons(&shot, &ram), 0x10);
        let tile = 0x10002 + 2 * (12 * 16 + 9);
        ram[tile..tile + 2].copy_from_slice(&0x0053u16.to_le_bytes());
        let reaction = State {
            host_steps: shot.host_steps + 1,
            polls: 102,
            effective_buttons: 0x10,
            joy1_current: 0x0800,
            joy1_new: 0,
            ..shot
        };
        route.observe(&reaction, &ram);
        assert_eq!(route.upper_beam_step, Some(shot.host_steps));
        assert_eq!(route.upper_response_step, Some(reaction.host_steps));
    }

    fn row10_state() -> (State, Vec<u8>, Return) {
        let (state, mut ram, mut route) = upper_beam_state();
        let state = State { joy1_current: 0x0800, ..state };
        ram[0xa1c..0xa1e].copy_from_slice(&4u16.to_le_bytes());
        ram[0xb48..0xb4a].fill(0);
        ram[0xccc..0xcce].copy_from_slice(&13u16.to_le_bytes());
        ram[0xcce..0xcd0].copy_from_slice(&1u16.to_le_bytes());
        for column in [8, 9] {
            let opening = 0x10002 + 2 * (12 * 16 + column);
            ram[opening..opening + 2].copy_from_slice(&0x0053u16.to_le_bytes());
            let target = 0x10002 + 2 * (10 * 16 + column);
            ram[target..target + 2].copy_from_slice(&0xc19au16.to_le_bytes());
            ram[0x16402 + 10 * 16 + column] = 4;
        }
        route.previous = Some(state);
        route.upper_anchor = Some(148);
        route.upper_response_step = Some(state.host_steps);
        route.climb_phase = 14;
        (state, ram, route)
    }

    #[cfg_attr(test, test)]
    pub(super) fn row10_beam_waits_for_native_cooldown_and_consumption_then_releases() {
        let (state, mut ram, mut route) = row10_state();
        let waiting = State { host_steps: state.host_steps + 1, polls: 101, ..state };
        route.observe(&waiting, &ram);
        assert_eq!(route.buttons(&waiting, &ram), 0x10);
        ram[0xccc..0xcce].fill(0);
        ram[0xcce..0xcd0].copy_from_slice(&5u16.to_le_bytes());
        let full = State { host_steps: waiting.host_steps + 1, polls: 102, ..waiting };
        route.observe(&full, &ram);
        assert_eq!(route.buttons(&full, &ram), 0x10);
        ram[0xcce..0xcd0].copy_from_slice(&1u16.to_le_bytes());
        let ready = State { host_steps: full.host_steps + 1, polls: 103, ..full };
        route.observe(&ready, &ram);
        assert_eq!(route.buttons(&ready, &ram), 0x210);
        let pending = State { host_steps: ready.host_steps + 1, effective_buttons: 0x210, ..ready };
        route.observe(&pending, &ram);
        assert_eq!(route.row10_beam_step, None);
        assert_eq!(route.buttons(&pending, &ram), 0x210);
        let shot = State {
            host_steps: pending.host_steps + 1,
            polls: 104,
            effective_buttons: 0x210,
            joy1_current: 0x0840,
            joy1_new: 0x0040,
            ..pending
        };
        route.observe(&shot, &ram);
        assert_eq!(route.row10_beam_step, Some(shot.host_steps));
        assert_eq!(route.row10_response_step, None);
        assert_eq!(route.buttons(&shot, &ram), 0x10);
        let target = 0x10002 + 2 * (10 * 16 + 9);
        ram[target..target + 2].copy_from_slice(&0x819au16.to_le_bytes());
        let reaction = State {
            host_steps: shot.host_steps + 1,
            polls: 105,
            effective_buttons: 0x10,
            joy1_current: 0x0800,
            joy1_new: 0,
            ..shot
        };
        route.observe(&reaction, &ram);
        assert_eq!(route.row10_response_step, Some(reaction.host_steps));
        assert_eq!(route.failure(), None);
    }

    #[cfg_attr(test, test)]
    pub(super) fn row10_beam_orders_same_poll_reaction_and_refuses_unowned_changes_or_bounds() {
        for invalid in 0..12 {
            let (state, mut ram, mut route) = row10_state();
            route.climb_phase = 15;
            let mut next = State {
                host_steps: state.host_steps + 1,
                polls: 101,
                effective_buttons: 0x210,
                joy1_current: 0x0840,
                joy1_new: 0x0040,
                ..state
            };
            let target = 0x10002 + 2 * (10 * 16 + 9);
            match invalid {
                0 => ram[target..target + 2].copy_from_slice(&0x819au16.to_le_bytes()),
                1 => {
                    route.climb_phase = 14;
                    ram[target..target + 2].copy_from_slice(&0x819au16.to_le_bytes());
                }
                2 => {
                    route.row10_polls = 119;
                    ram[target..target + 2].fill(0);
                }
                3 => ram[0x16402 + 10 * 16 + 9] = 0,
                4 => {
                    let opening = 0x10002 + 2 * (12 * 16 + 8);
                    ram[opening..opening + 2].copy_from_slice(&0xc19au16.to_le_bytes());
                }
                5 => ram[0x9d2..0x9d4].copy_from_slice(&1u16.to_le_bytes()),
                6 => next.x = 147,
                7 => {
                    let support = 0x10002 + 2 * (16 * 16 + 9);
                    ram[support..support + 2].fill(0);
                }
                8 => ram[0xb48..0xb4a].copy_from_slice(&1u16.to_le_bytes()),
                9 => next.joy1_current |= word(&ram, 0x9b4),
                10 => next.joy1_current |= 0x200,
                _ => ram[0xa1c..0xa1e].copy_from_slice(&3u16.to_le_bytes()),
            }
            route.observe(&next, &ram);
            if invalid == 0 {
                assert_eq!(route.row10_beam_step, Some(next.host_steps));
                assert_eq!(route.row10_response_step, Some(next.host_steps));
            } else {
                assert_eq!(route.row10_response_step, None);
                assert!(route.failure().is_some());
            }
        }
    }

    #[cfg_attr(test, test)]
    pub(super) fn row10_ready_waits_for_shoot_release_and_freezes_prior_counter() {
        let (state, mut ram, mut route) = row10_state();
        ram[0xccc..0xcce].fill(0);
        route.upper_polls = 5;
        let held = State { host_steps: state.host_steps + 1, polls: 101, joy1_current: 0x0840, ..state };
        route.observe(&held, &ram);
        assert_eq!(route.climb_phase, 14);
        assert_eq!(route.buttons(&held, &ram), 0x10);
        assert_eq!(route.upper_polls, 5);
        assert_eq!(route.row10_polls, 1);
        let released = State { host_steps: held.host_steps + 1, polls: 102, joy1_current: 0x0800, ..held };
        route.observe(&released, &ram);
        assert_eq!(route.climb_phase, 15);
        assert_eq!(route.upper_polls, 5);
        assert_eq!(route.row10_polls, 2);
    }

    #[cfg_attr(test, test)]
    pub(super) fn row8_beam_freezes_row10_counter_and_orders_consumed_same_poll_reaction() {
        let (state, mut ram, mut route) = row10_state();
        route.row10_response_step = Some(state.host_steps);
        route.row10_polls = 23;
        route.climb_phase = 17;
        ram[0xccc..0xcce].copy_from_slice(&6u16.to_le_bytes());
        for column in [8, 9] {
            let opening = 0x10002 + 2 * (10 * 16 + column);
            ram[opening..opening + 2].copy_from_slice(&0x0053u16.to_le_bytes());
            let target = 0x10002 + 2 * (8 * 16 + column);
            ram[target..target + 2].copy_from_slice(&0xc19au16.to_le_bytes());
        }
        let waiting = State { host_steps: state.host_steps + 1, polls: 101, ..state };
        route.observe(&waiting, &ram);
        assert_eq!(route.buttons(&waiting, &ram), 0x10);
        assert_eq!(route.row10_polls, 23);
        assert_eq!(route.row8_polls, 1);
        ram[0xccc..0xcce].fill(0);
        let ready = State { host_steps: waiting.host_steps + 1, polls: 102, ..waiting };
        route.observe(&ready, &ram);
        assert_eq!(route.buttons(&ready, &ram), 0x210);
        let pending = State { host_steps: ready.host_steps + 1, effective_buttons: 0x210, ..ready };
        route.observe(&pending, &ram);
        assert_eq!(route.row8_beam_step, None);
        assert_eq!(route.buttons(&pending, &ram), 0x210);
        let target = 0x10002 + 2 * (8 * 16 + 9);
        let shot = State {
            host_steps: pending.host_steps + 1,
            polls: 103,
            effective_buttons: 0x210,
            joy1_current: 0x0840,
            joy1_new: 0x0040,
            ..ready
        };
        route.observe(&shot, &ram);
        assert_eq!(route.row8_beam_step, Some(shot.host_steps));
        assert_eq!(route.row8_response_step, None);
        assert_eq!(route.buttons(&shot, &ram), 0x10);
        ram[target..target + 2].copy_from_slice(&0x0053u16.to_le_bytes());
        let reaction = State {
            host_steps: shot.host_steps + 1,
            polls: 104,
            effective_buttons: 0x10,
            joy1_current: 0x0800,
            joy1_new: 0,
            ..shot
        };
        route.observe(&reaction, &ram);
        assert_eq!(route.row8_response_step, Some(reaction.host_steps));
        assert_eq!(route.failure(), None);
    }

    #[cfg_attr(test, test)]
    pub(super) fn row8_beam_refuses_wrong_bts_closed_intermediate_opening_and_terminal_bound() {
        for invalid in 0..6 {
            let (state, mut ram, mut route) = row10_state();
            route.row10_response_step = Some(state.host_steps);
            route.climb_phase = 18;
            for column in [8, 9] {
                let opening = 0x10002 + 2 * (10 * 16 + column);
                ram[opening..opening + 2].copy_from_slice(&0x0053u16.to_le_bytes());
                let target = 0x10002 + 2 * (8 * 16 + column);
                ram[target..target + 2].copy_from_slice(&0xc19au16.to_le_bytes());
            }
            match invalid {
                0 => ram[0x16402 + 8 * 16 + 9] = 4,
                1 => {
                    let opening = 0x10002 + 2 * (10 * 16 + 8);
                    ram[opening..opening + 2].copy_from_slice(&0xc19au16.to_le_bytes());
                }
                2 => {
                    route.row8_polls = 119;
                    let target = 0x10002 + 2 * (8 * 16 + 9);
                    ram[target..target + 2].fill(0);
                }
                3 => {
                    let opening = 0x10002 + 2 * (12 * 16 + 9);
                    ram[opening..opening + 2].copy_from_slice(&0xc19au16.to_le_bytes());
                }
                _ => {
                    if invalid == 4 {
                        route.climb_phase = 17;
                    }
                    let target = 0x10002 + 2 * (8 * 16 + 9);
                    ram[target..target + 2].copy_from_slice(&0x0053u16.to_le_bytes());
                }
            }
            let shot = State {
                host_steps: state.host_steps + 1,
                polls: 101,
                effective_buttons: 0x210,
                joy1_current: 0x0840,
                joy1_new: 0x0040,
                ..state
            };
            route.observe(&shot, &ram);
            if invalid == 5 {
                assert_eq!(route.row8_beam_step, Some(shot.host_steps));
                assert_eq!(route.row8_response_step, Some(shot.host_steps));
            } else {
                assert_eq!(route.row8_beam_step, None);
                assert_eq!(route.row8_response_step, None);
                assert!(route.failure().is_some());
            }
        }
    }

    fn survey_state() -> (State, Vec<u8>, Return) {
        let (state, mut ram, mut route) = upper_beam_state();
        let state = State { joy1_current: 0x0800, ..state };
        ram[0xa1c..0xa1e].copy_from_slice(&4u16.to_le_bytes());
        ram[0xb48..0xb4a].fill(0);
        let target = 0x10002 + 2 * (10 * 16 + 7);
        ram[target..target + 2].copy_from_slice(&0xc19au16.to_le_bytes());
        ram[0x16402 + 10 * 16 + 7] = 4;
        for row in [8, 10, 12] {
            for column in [8, 9] {
                let at = 0x10002 + 2 * (row * 16 + column);
                ram[at..at + 2].copy_from_slice(&0x00ffu16.to_le_bytes());
            }
        }
        route.previous = Some(state);
        route.upper_response_step = Some(state.host_steps);
        route.row10_response_step = Some(state.host_steps);
        route.row8_response_step = Some(state.host_steps);
        route.climb_phase = 20;
        (state, ram, route)
    }

    fn aerial_state() -> (State, Vec<u8>, Return) {
        let (state, mut ram, mut route) = survey_state();
        route.climb_phase = 22;
        route.survey_jump_step = Some(state.host_steps);
        ram[0xa1c..0xa1e].copy_from_slice(&0x4eu16.to_le_bytes());
        ram[0xa1f] = 2;
        ram[0xb00..0xb02].copy_from_slice(&19u16.to_le_bytes());
        ram[0xb36..0xb38].copy_from_slice(&1u16.to_le_bytes());
        let window = State { host_steps: state.host_steps + 1, polls: 101, y: 174, joy1_current: 0x80, ..state };
        route.observe(&window, &ram);
        (window, ram, route)
    }

    #[cfg_attr(test, test)]
    pub(super) fn aerial_beam_waits_for_consumption_then_releases_shoot_and_records_prior_origin() {
        for immediate in [false, true] {
            let (window, mut ram, mut route) = aerial_state();
            assert_eq!(route.failure(), None);
            assert_eq!(route.buttons(&window, &ram), 0x300);
            let waiting = State { host_steps: window.host_steps + 1, effective_buttons: 0x300, ..window };
            route.observe(&waiting, &ram);
            assert_eq!(route.aerial_beam_step, None);
            assert_eq!(route.aerial_polls, 0);
            assert_eq!(route.buttons(&waiting, &ram), 0x300);
            ram[0xccc..0xcce].copy_from_slice(&13u16.to_le_bytes());
            ram[0xcce..0xcd0].copy_from_slice(&1u16.to_le_bytes());
            ram[0xb5e..0xb60].copy_from_slice(&0x8007u16.to_le_bytes());
            ram[0xa1c..0xa1e].copy_from_slice(&0x14u16.to_le_bytes());
            let target = 0x10002 + 2 * (10 * 16 + 7);
            if immediate {
                ram[target..target + 2].copy_from_slice(&0x0053u16.to_le_bytes());
            }
            let shot = State {
                host_steps: waiting.host_steps + 1,
                polls: 102,
                y: 170,
                joy1_current: 0x80 | word(&ram, 0x9b2),
                joy1_new: word(&ram, 0x9b2),
                ..waiting
            };
            route.observe(&shot, &ram);
            assert_eq!(route.aerial_creation, Some((0x4e, 174)));
            assert_eq!(route.aerial_beam_step, Some(shot.host_steps));
            if !immediate {
                assert_eq!(route.failure(), None);
                assert_eq!(route.buttons(&shot, &ram), 0x100);
                ram[target..target + 2].copy_from_slice(&0x0053u16.to_le_bytes());
                let response = State {
                    host_steps: shot.host_steps + 1,
                    polls: 103,
                    effective_buttons: 0x100,
                    joy1_current: 0x80,
                    joy1_new: 0,
                    ..shot
                };
                route.observe(&response, &ram);
                assert_eq!(route.aerial_response_step, Some(response.host_steps));
            } else {
                assert_eq!(route.aerial_response_step, Some(shot.host_steps));
            }
            assert_eq!(route.failure(), None);
            assert_eq!(route.survey_polls, 1);
            assert_eq!(route.buttons(&shot, &ram), 0x140);
            let mut report = sm_binary_evidence::Map::new();
            route.report(&mut report);
            assert_eq!(report["rio_return_aerial_creation_y"].as_integer(), Some(174));
            assert_eq!(report["rio_return_complete"].as_bool(), Some(false));
        }
    }

    #[cfg_attr(test, test)]
    pub(super) fn aerial_beam_refuses_unready_window_before_emitting_shoot() {
        for invalid in 0..4 {
            let (window, mut ram, mut route) = aerial_state();
            route.climb_phase = 22;
            route.survey_window_step = None;
            route.aerial_previous = None;
            match invalid {
                0 => ram[0xccc..0xcce].copy_from_slice(&1u16.to_le_bytes()),
                1 => ram[0xcce..0xcd0].copy_from_slice(&5u16.to_le_bytes()),
                2 => ram[0x16402 + 10 * 16 + 7] = 0,
                _ => {
                    let target = 0x10002 + 2 * (10 * 16 + 7);
                    ram[target..target + 2].copy_from_slice(&0x00ffu16.to_le_bytes());
                }
            }
            let next = State { host_steps: window.host_steps + 1, polls: 102, ..window };
            route.observe(&next, &ram);
            assert!(route.failure().is_some());
            assert_eq!(route.buttons(&next, &ram), 0);
            assert_eq!(route.aerial_previous, None);
            assert_eq!(route.aerial_beam_step, None);
        }
    }

    fn transfer_state() -> (State, Vec<u8>, Return) {
        let (state, mut ram, mut route) = aerial_state();
        let target = 0x10002 + 2 * (10 * 16 + 7);
        ram[target..target + 2].copy_from_slice(&0x0053u16.to_le_bytes());
        let support = 0x10002 + 2 * (12 * 16 + 7);
        ram[support..support + 2].copy_from_slice(&0xc19au16.to_le_bytes());
        ram[0xa1e] = 4;
        route.aerial_beam_step = Some(state.host_steps);
        route.aerial_response_step = Some(state.host_steps);
        route.climb_phase = 25;
        let state = State { y: 171, joy1_current: 0x80 | word(&ram, 0x9b2), joy1_new: word(&ram, 0x9b2), ..state };
        route.previous = Some(state);
        (state, ram, route)
    }

    #[cfg_attr(test, test)]
    pub(super) fn transfer_waits_for_released_shoot_and_native_ground_contact_then_freezes_momentum() {
        let (state, mut ram, mut route) = transfer_state();
        let pending = State { host_steps: state.host_steps + 1, effective_buttons: 0x140, ..state };
        route.observe(&pending, &ram);
        assert_eq!(route.failure(), None);
        assert_eq!(route.transfer_release_step, None);
        assert_eq!(route.transfer_polls, 0);
        assert_eq!(route.buttons(&pending, &ram), 0x140);
        ram[0xa1f] = 6;
        ram[0xb36..0xb38].copy_from_slice(&2u16.to_le_bytes());
        let falling = State {
            host_steps: pending.host_steps + 1,
            polls: pending.polls + 1,
            x: 134,
            y: 169,
            joy1_current: 0x280,
            joy1_new: 0x200,
            ..pending
        };
        route.observe(&falling, &ram);
        assert_eq!(route.transfer_release_step, Some(falling.host_steps));
        assert!(route.transfer_contact.is_none());
        ram[0xa1f] = 1;
        ram[0xb00..0xb02].copy_from_slice(&21u16.to_le_bytes());
        ram[0xb36..0xb38].fill(0);
        ram[0xb2c..0xb30].fill(0);
        ram[0xb46..0xb48].copy_from_slice(&1u16.to_le_bytes());
        ram[0xb48..0xb4a].copy_from_slice(&0x8000u16.to_le_bytes());
        ram[0xb4a..0xb4c].copy_from_slice(&2u16.to_le_bytes());
        ram[0xa6c..0xa6e].copy_from_slice(&0x9f55u16.to_le_bytes());
        ram[0xb42..0xb44].copy_from_slice(&2u16.to_le_bytes());
        ram[0xb44..0xb46].copy_from_slice(&0x4000u16.to_le_bytes());
        ram[0xaf8..0xafa].copy_from_slice(&0xc000u16.to_le_bytes());
        let contact = State {
            host_steps: falling.host_steps + 1,
            polls: falling.polls + 1,
            x: 132,
            y: 171,
            joy1_new: 0,
            ..falling
        };
        route.observe(&contact, &ram);
        let witness = route.transfer_contact.as_ref().expect("native contact");
        assert_eq!(
            (witness.step, witness.x, witness.speed, witness.subspeed, witness.accel_mode, witness.speed_table),
            (contact.host_steps, 132, 1, 0x8000, 2, 0x9f55)
        );
        assert_eq!(route.failure(), None);
        assert_eq!((witness.extra_speed, witness.extra_subspeed, witness.x_subposition), (2, 0x4000, 0xc000));
        assert_eq!(route.buttons(&contact, &ram), 0);
        assert_eq!(route.aerial_polls, 0);
        let mut report = sm_binary_evidence::Map::new();
        route.report(&mut report);
        assert_eq!(report["rio_return_transfer_contact_x"].as_integer(), Some(132));
        for (key, expected) in [
            ("speed", 1),
            ("subspeed", 0x8000),
            ("extra_speed", 2),
            ("extra_subspeed", 0x4000),
            ("x_subposition", 0xc000),
            ("accel_mode", 2),
            ("speed_table", 0x9f55),
        ] {
            assert_eq!(report[&format!("rio_return_transfer_contact_{key}")].as_integer(), Some(expected));
        }
        assert_eq!(report["rio_return_complete"].as_bool(), Some(false));
    }

    #[cfg_attr(test, test)]
    pub(super) fn transfer_refuses_foreign_input_closed_terrain_and_bound_before_contact() {
        for invalid in 0..10 {
            let (state, mut ram, mut route) = transfer_state();
            ram[0xa1f] = 1;
            ram[0xb00..0xb02].copy_from_slice(&21u16.to_le_bytes());
            ram[0xb36..0xb38].fill(0);
            ram[0xb2c..0xb30].fill(0);
            let mut next = State {
                host_steps: state.host_steps + 1,
                polls: state.polls + 1,
                x: 132,
                y: 171,
                effective_buttons: 0x140,
                joy1_current: 0x280,
                joy1_new: 0,
                ..state
            };
            match invalid {
                0 => next.joy1_current |= word(&ram, 0x9b2),
                1 => next.joy1_current |= 0x100,
                2 => ram[0xa1f] = 3,
                3 => {
                    let at = 0x10002 + 2 * (10 * 16 + 7);
                    ram[at..at + 2].copy_from_slice(&0x819au16.to_le_bytes());
                }
                4 => {
                    let at = 0x10002 + 2 * (12 * 16 + 7);
                    ram[at..at + 2].fill(0);
                }
                5 => route.transfer_polls = 119,
                6 => next.x = 116,
                7 => next.y = 172,
                8 => next.joy1_new = word(&ram, 0x9b2),
                _ => next.effective_buttons |= 0x80,
            }
            route.observe(&next, &ram);
            assert!(route.failure().is_some());
            assert!(route.transfer_contact.is_none());
            assert_eq!(route.buttons(&next, &ram), 0);
        }
    }

    #[cfg_attr(test, test)]
    pub(super) fn transfer_requires_consumed_contact_and_actual_zero_vertical_speed() {
        for unpolled in [false, true] {
            let (state, mut ram, mut route) = transfer_state();
            ram[0xa1f] = 1;
            ram[0xb00..0xb02].copy_from_slice(&21u16.to_le_bytes());
            ram[0xb36..0xb38].fill(0);
            ram[0xb2c..0xb30].fill(0);
            if !unpolled {
                ram[0xb2e..0xb30].copy_from_slice(&1u16.to_le_bytes());
            }
            let next = State {
                host_steps: state.host_steps + 1,
                polls: state.polls + u64::from(!unpolled),
                x: 132,
                y: 171,
                effective_buttons: 0x140,
                joy1_current: 0x280,
                joy1_new: 0,
                ..state
            };
            route.observe(&next, &ram);
            assert_eq!(route.failure(), None);
            assert!(route.transfer_contact.is_none());
            assert_eq!(route.transfer_release_step, if unpolled { None } else { Some(next.host_steps) });
        }
    }

    #[cfg_attr(test, test)]
    pub(super) fn transfer_accepts_only_native_a5_landing_transition_with_prior_airborne_radius() {
        for invalid in 0..4 {
            let (state, mut ram, mut route) = transfer_state();
            ram[0xa1c..0xa1e].copy_from_slice(&(if invalid == 1 { 0xa7u16 } else { 0xa5u16 }).to_le_bytes());
            ram[0xa1f] = if invalid == 2 { 3 } else { 0 };
            ram[0xb36..0xb38].fill(0);
            ram[0xb2c..0xb30].fill(0);
            if invalid == 3 {
                ram[0xb00..0xb02].copy_from_slice(&20u16.to_le_bytes());
            }
            let next = State {
                host_steps: state.host_steps + 1,
                polls: state.polls + 1,
                x: 128,
                y: 171,
                effective_buttons: 0x140,
                joy1_current: 0x280,
                joy1_new: 0,
                ..state
            };
            route.observe(&next, &ram);
            if invalid == 0 {
                let contact = route.transfer_contact.as_ref().expect("A5 first contact");
                assert_eq!((contact.pose, contact.radius, contact.x), (0xa5, 19, 128));
                assert_eq!(route.failure(), None);
                let mut report = sm_binary_evidence::Map::new();
                route.report(&mut report);
                assert_eq!(report["rio_return_transfer_contact_pose"].as_integer(), Some(0xa5));
                assert_eq!(report["rio_return_transfer_contact_radius"].as_integer(), Some(19));
            } else {
                assert!(route.transfer_contact.is_none());
                assert!(route.failure().is_some());
            }
            assert_eq!(route.buttons(&next, &ram), 0);
        }
    }

    fn support_state() -> (State, Vec<u8>, Return) {
        let (state, mut ram, mut route) = transfer_state();
        ram[0xa1c..0xa1e].copy_from_slice(&0xa5u16.to_le_bytes());
        ram[0xa1f] = 0;
        ram[0xb36..0xb38].fill(0);
        ram[0xb2c..0xb30].fill(0);
        ram[0xaf8..0xafa].copy_from_slice(&0x8000u16.to_le_bytes());
        let contact = State {
            host_steps: state.host_steps + 1,
            polls: state.polls + 1,
            x: 128,
            y: 171,
            effective_buttons: 0x140,
            joy1_current: 0x280,
            joy1_new: 0,
            ..state
        };
        route.observe(&contact, &ram);
        assert_eq!(route.climb_phase, 26);
        (contact, ram, route)
    }

    #[cfg_attr(test, test)]
    pub(super) fn support_requires_consumed_neutral_standing_before_left_only_full_span() {
        let (contact, mut ram, mut route) = support_state();
        let pending = State { host_steps: contact.host_steps + 1, effective_buttons: 0, ..contact };
        route.observe(&pending, &ram);
        assert_eq!(route.support_release_step, None);
        assert_eq!(route.buttons(&pending, &ram), 0);
        ram[0xb00..0xb02].copy_from_slice(&21u16.to_le_bytes());
        let released = State {
            host_steps: pending.host_steps + 1,
            polls: pending.polls + 1,
            joy1_current: 0,
            joy1_new: 0,
            ..pending
        };
        route.observe(&released, &ram);
        assert_eq!(route.support_release_step, Some(released.host_steps));
        assert_eq!(route.support_standing_step, None);
        ram[0xa1c..0xa1e].copy_from_slice(&2u16.to_le_bytes());
        let unpolled = State { host_steps: released.host_steps + 1, ..released };
        route.observe(&unpolled, &ram);
        assert_eq!(route.support_standing_step, None);
        assert_eq!(route.buttons(&unpolled, &ram), 0);
        let standing = State { host_steps: unpolled.host_steps + 1, polls: unpolled.polls + 1, ..unpolled };
        route.observe(&standing, &ram);
        assert_eq!(route.support_standing_step, Some(standing.host_steps));
        assert_eq!(route.buttons(&standing, &ram), 0x40);
        ram[0xa1f] = 1;
        ram[0xb46..0xb48].copy_from_slice(&1u16.to_le_bytes());
        ram[0xb48..0xb4a].copy_from_slice(&0x4000u16.to_le_bytes());
        ram[0xb42..0xb44].copy_from_slice(&2u16.to_le_bytes());
        ram[0xb44..0xb46].copy_from_slice(&0x2000u16.to_le_bytes());
        ram[0xb4a..0xb4c].copy_from_slice(&1u16.to_le_bytes());
        ram[0xa6c..0xa6e].copy_from_slice(&0x9f55u16.to_le_bytes());
        ram[0xaf8..0xafa].copy_from_slice(&0xc000u16.to_le_bytes());
        let partial = State {
            host_steps: standing.host_steps + 1,
            polls: standing.polls + 1,
            x: 123,
            effective_buttons: 0x40,
            joy1_current: 0x200,
            joy1_new: 0x200,
            ..standing
        };
        route.observe(&partial, &ram);
        assert!(route.support_span.is_none());
        let unpolled_full = State { host_steps: partial.host_steps + 1, x: 122, ..partial };
        route.observe(&unpolled_full, &ram);
        assert!(route.support_span.is_none());
        let full = State {
            host_steps: unpolled_full.host_steps + 1,
            polls: unpolled_full.polls + 1,
            joy1_new: 0,
            ..unpolled_full
        };
        route.observe(&full, &ram);
        let span = route.support_span.as_ref().expect("fullC7span");
        assert_eq!((span.x, span.speed, span.subspeed), (122, 1, 0x4000));
        assert_eq!(route.failure(), None);
        assert_eq!(route.buttons(&full, &ram), 0);
        let mut report = sm_binary_evidence::Map::new();
        route.report(&mut report);
        assert_eq!(report["rio_return_support_span_speed"].as_integer(), Some(1));
        for (key, expected) in [
            ("pose", 2),
            ("radius", 21),
            ("x", 122),
            ("subspeed", 0x4000),
            ("extra_speed", 2),
            ("extra_subspeed", 0x2000),
            ("x_subposition", 0xc000),
            ("accel_mode", 1),
            ("speed_table", 0x9f55),
        ] {
            assert_eq!(report[&format!("rio_return_support_span_{key}")].as_integer(), Some(expected));
        }
        assert_eq!(report["rio_return_complete"].as_bool(), Some(false));
    }

    #[cfg_attr(test, test)]
    pub(super) fn support_refuses_jump_new_or_effective_foreign_keys_and_bound_before_span() {
        for invalid in 0..8 {
            let (contact, mut ram, mut route) = support_state();
            ram[0xa1c..0xa1e].copy_from_slice(&2u16.to_le_bytes());
            ram[0xb00..0xb02].copy_from_slice(&21u16.to_le_bytes());
            let mut next = State {
                host_steps: contact.host_steps + 1,
                polls: contact.polls + 1,
                effective_buttons: 0,
                joy1_current: 0,
                joy1_new: 0,
                ..contact
            };
            match invalid {
                0 => next.joy1_current = word(&ram, 0x9b4),
                1 => next.joy1_new = word(&ram, 0x9b2),
                2 => next.effective_buttons = 0x80,
                3 => route.support_polls = 119,
                4 => {
                    let at = 0x10002 + 2 * (12 * 16 + 7);
                    ram[at..at + 2].fill(0);
                }
                5 => next.x = 116,
                6 => ram[0xa1f] = 3,
                _ => ram[0xb00..0xb02].copy_from_slice(&20u16.to_le_bytes()),
            }
            route.observe(&next, &ram);
            assert!(route.failure().is_some());
            assert!(route.support_span.is_none());
            assert_eq!(route.buttons(&next, &ram), 0);
        }
    }

    #[cfg_attr(test, test)]
    pub(super) fn support_keeps_neutral_when_contact_fraction_or_speed_differs() {
        for fraction in [false, true] {
            let (contact, mut ram, mut route) = support_state();
            ram[0xa1c..0xa1e].copy_from_slice(&2u16.to_le_bytes());
            ram[0xb00..0xb02].copy_from_slice(&21u16.to_le_bytes());
            if fraction {
                ram[0xaf8..0xafa].copy_from_slice(&0x4000u16.to_le_bytes());
            } else {
                ram[0xb46..0xb48].copy_from_slice(&1u16.to_le_bytes());
            }
            let next = State {
                host_steps: contact.host_steps + 1,
                polls: contact.polls + 1,
                effective_buttons: 0,
                joy1_current: 0,
                joy1_new: 0,
                ..contact
            };
            route.observe(&next, &ram);
            assert_eq!(route.failure(), None);
            assert_eq!(route.support_standing_step, None);
            assert_eq!(route.buttons(&next, &ram), 0);
        }
    }

    #[cfg_attr(test, test)]
    pub(super) fn support_requires_fully_neutral_consumed_keys_before_standing_authorizes_left() {
        for native in [false, true] {
            let (contact, mut ram, mut route) = support_state();
            ram[0xa1c..0xa1e].copy_from_slice(&2u16.to_le_bytes());
            ram[0xb00..0xb02].copy_from_slice(&21u16.to_le_bytes());
            let next = State {
                host_steps: contact.host_steps + 1,
                polls: contact.polls + 1,
                effective_buttons: if native { 0 } else { 0x20 },
                joy1_current: if native { 0x400 } else { 0 },
                joy1_new: 0,
                ..contact
            };
            route.observe(&next, &ram);
            assert_eq!(route.support_standing_step, None);
            assert_eq!(route.support_release_step, None);
            assert_eq!(route.buttons(&next, &ram), 0);
        }
    }

    fn brake_state() -> (State, Vec<u8>, Return) {
        let (state, mut ram, mut route) = support_state();
        ram[0xa1c..0xa1e].copy_from_slice(&0xau16.to_le_bytes());
        ram[0xa1f] = 1;
        ram[0xb00..0xb02].copy_from_slice(&21u16.to_le_bytes());
        ram[0xb46..0xb48].copy_from_slice(&1u16.to_le_bytes());
        ram[0xb48..0xb4a].copy_from_slice(&0x5000u16.to_le_bytes());
        ram[0xaf8..0xafa].copy_from_slice(&0x4000u16.to_le_bytes());
        route.support_span = Some(TransferContact {
            step: state.host_steps,
            pose: 0xa,
            radius: 21,
            x: 122,
            speed: 1,
            subspeed: 0x5000,
            accel_mode: 0,
            speed_table: 0x9f55,
            x_subposition: 0x4000,
            extra_speed: 0,
            extra_subspeed: 0,
        });
        route.climb_phase = 28;
        let state = State { x: 122, effective_buttons: 0x40, joy1_current: 0x200, joy1_new: 0, ..state };
        route.previous = Some(state);
        (state, ram, route)
    }

    #[cfg_attr(test, test)]
    pub(super) fn brake_requires_consumed_release_and_actual_zero_motion_left_standing() {
        let (state, mut ram, mut route) = brake_state();
        let pending = State { host_steps: state.host_steps + 1, effective_buttons: 0, ..state };
        route.observe(&pending, &ram);
        assert_eq!(route.brake_release_step, None);
        assert_eq!(route.buttons(&pending, &ram), 0);
        let moving = State {
            host_steps: pending.host_steps + 1,
            polls: pending.polls + 1,
            x: 120,
            joy1_current: 0,
            joy1_new: 0,
            ..pending
        };
        route.observe(&moving, &ram);
        assert_eq!(route.brake_release_step, Some(moving.host_steps));
        assert_eq!(route.brake_rest, None);
        ram[0xa1c..0xa1e].copy_from_slice(&2u16.to_le_bytes());
        ram[0xa1f] = 0;
        ram[0xb46..0xb4c].fill(0);
        let unpolled = State { host_steps: moving.host_steps + 1, ..moving };
        route.observe(&unpolled, &ram);
        assert_eq!(route.brake_rest, None);
        let rest = State { host_steps: unpolled.host_steps + 1, polls: unpolled.polls + 1, x: 119, ..unpolled };
        route.observe(&rest, &ram);
        assert_eq!(route.brake_rest, Some((rest.host_steps, 119, 0x4000)));
        assert_eq!(route.failure(), None);
        let mut report = sm_binary_evidence::Map::new();
        route.report(&mut report);
        assert_eq!(report["rio_return_brake_rest_x"].as_integer(), Some(119));
        assert_eq!(report["rio_return_brake_rest_step"].as_integer(), Some(rest.host_steps as i64));
        assert_eq!(report["rio_return_brake_rest_subposition"].as_integer(), Some(0x4000));
        assert_eq!(report["rio_return_complete"].as_bool(), Some(false));
    }

    #[cfg_attr(test, test)]
    pub(super) fn brake_refuses_unreleased_keys_lost_span_spin_or_bound_before_rest() {
        for invalid in 0..10 {
            let (state, mut ram, mut route) = brake_state();
            ram[0xa1c..0xa1e].copy_from_slice(&2u16.to_le_bytes());
            ram[0xa1f] = 0;
            ram[0xb46..0xb4c].fill(0);
            let mut next = State {
                host_steps: state.host_steps + 1,
                polls: state.polls + 1,
                effective_buttons: 0,
                joy1_current: 0,
                joy1_new: 0,
                ..state
            };
            match invalid {
                0 => next.joy1_current = 0x200,
                1 => next.joy1_new = word(&ram, 0x9b4),
                2 => next.effective_buttons = 0x40,
                3 => next.x = 116,
                4 => ram[0xa1f] = 3,
                5 => route.brake_polls = 119,
                6 => {
                    let at = 0x10002 + 2 * (12 * 16 + 7);
                    ram[at..at + 2].fill(0);
                }
                7 => ram[0xb00..0xb02].copy_from_slice(&20u16.to_le_bytes()),
                8 => ram[0xb2e..0xb30].copy_from_slice(&1u16.to_le_bytes()),
                _ => {
                    ram[0xa1e] = 8;
                    ram[0xa1f] = 0x0e;
                }
            }
            route.observe(&next, &ram);
            assert!(route.failure().is_some());
            assert_eq!(route.brake_rest, None);
            assert_eq!(route.buttons(&next, &ram), 0);
        }
    }

    #[cfg_attr(test, test)]
    pub(super) fn brake_waits_for_zero_extra_speed_and_acceleration_before_rest() {
        for at in [0xb42, 0xb44, 0xb46, 0xb48, 0xb4a] {
            let (state, mut ram, mut route) = brake_state();
            ram[0xa1c..0xa1e].copy_from_slice(&2u16.to_le_bytes());
            ram[0xa1f] = 0;
            ram[0xb42..0xb4c].fill(0);
            ram[at..at + 2].copy_from_slice(&1u16.to_le_bytes());
            let waiting = State {
                host_steps: state.host_steps + 1,
                polls: state.polls + 1,
                effective_buttons: 0,
                joy1_current: 0,
                joy1_new: 0,
                ..state
            };
            route.observe(&waiting, &ram);
            assert_eq!(route.failure(), None);
            assert_eq!(route.brake_rest, None);
            ram[at..at + 2].fill(0);
            let rest = State { host_steps: waiting.host_steps + 1, polls: waiting.polls + 1, ..waiting };
            route.observe(&rest, &ram);
            assert_eq!(route.brake_rest, Some((rest.host_steps, rest.x, 0x4000)));
            assert_eq!(route.buttons(&rest, &ram), 0x10);
        }
    }

    #[cfg_attr(test, test)]
    pub(super) fn brake_accepts_only_witnessed_left_wall_transition_before_actual_standing_rest() {
        for invalid in 0..4 {
            let (state, mut ram, mut route) = brake_state();
            ram[0xa1c..0xa1e].copy_from_slice(&(if invalid == 1 { 0x89u16 } else { 0x8au16 }).to_le_bytes());
            ram[0xa1f] = 0x15;
            ram[0xdd0..0xdd2].copy_from_slice(&(if invalid == 2 { 0u16 } else { 1u16 }).to_le_bytes());
            ram[0xb46..0xb48].fill(0);
            ram[0xb48..0xb4a].copy_from_slice(&0x8000u16.to_le_bytes());
            ram[0xb4a..0xb4c].copy_from_slice(&2u16.to_le_bytes());
            ram[0xaf8..0xafa].fill(0);
            let wall = State {
                host_steps: state.host_steps + 1,
                polls: state.polls + 1,
                x: if invalid == 3 { 118 } else { 117 },
                effective_buttons: 0,
                joy1_current: 0,
                joy1_new: 0,
                ..state
            };
            route.observe(&wall, &ram);
            if invalid == 0 {
                assert_eq!(route.failure(), None);
                assert_eq!(route.brake_wall_step, Some(wall.host_steps));
                assert_eq!(route.brake_rest, None);
                ram[0xdd0..0xdd2].fill(0);
                ram[0xb42..0xb4c].fill(0);
                let settled = State { host_steps: wall.host_steps + 1, polls: wall.polls + 1, ..wall };
                route.observe(&settled, &ram);
                assert_eq!(route.failure(), None);
                assert_eq!(route.brake_rest, None);
                assert_eq!(route.climb_phase, 29);
                assert_eq!(route.buttons(&settled, &ram), 0x10);
                ram[0xa1c..0xa1e].copy_from_slice(&4u16.to_le_bytes());
                ram[0xa1f] = 0;
                let unpolled = State { host_steps: settled.host_steps + 1, ..settled };
                route.observe(&unpolled, &ram);
                assert_eq!(route.brake_aim_step, None);
                let aim = State {
                    host_steps: unpolled.host_steps + 1,
                    polls: unpolled.polls + 1,
                    effective_buttons: 0x10,
                    joy1_current: 0x0800,
                    joy1_new: 0x0800,
                    ..unpolled
                };
                route.observe(&aim, &ram);
                assert_eq!(route.failure(), None);
                assert_eq!(route.brake_aim_step, Some(aim.host_steps));
                assert_eq!(route.buttons(&aim, &ram), 0);
                assert_eq!(route.brake_rest, None);
                ram[0xa1c..0xa1e].copy_from_slice(&2u16.to_le_bytes());
                let pending_rest =
                    State { host_steps: aim.host_steps + 1, effective_buttons: 0, joy1_current: 0, joy1_new: 0, ..aim };
                route.observe(&pending_rest, &ram);
                assert_eq!(route.brake_rest, None);
                let rest = State { host_steps: pending_rest.host_steps + 1, polls: aim.polls + 1, ..settled };
                route.observe(&rest, &ram);
                assert!(route.brake_rest.is_some());
                assert_eq!(route.failure(), None);
                let mut report = sm_binary_evidence::Map::new();
                route.report(&mut report);
                assert_eq!(report["rio_return_brake_aim_step"].as_integer(), Some(aim.host_steps as i64));
                assert_eq!(report["rio_return_brake_rest_step"].as_integer(), Some(rest.host_steps as i64));
                assert_eq!(route.brake_wall_step, Some(wall.host_steps));
                assert_eq!(route.brake_release_step, Some(wall.host_steps));
            } else {
                assert!(route.failure().is_some());
                assert_eq!(route.brake_wall_step, None);
                assert_eq!(route.brake_rest, None);
            }
        }
    }

    #[cfg_attr(test, test)]
    pub(super) fn wall_aim_refuses_foreign_keys_motion_or_pose_and_requires_release_before_rest() {
        for invalid in 0..8 {
            let (state, mut ram, mut route) = brake_state();
            route.climb_phase = 29;
            route.brake_wall_step = Some(state.host_steps);
            ram[0xa1c..0xa1e].copy_from_slice(&4u16.to_le_bytes());
            ram[0xa1f] = 0;
            ram[0xaf8..0xafa].fill(0);
            ram[0xb42..0xb4c].fill(0);
            let mut aim = State {
                host_steps: state.host_steps + 1,
                polls: state.polls + 1,
                x: 117,
                effective_buttons: 0x10,
                joy1_current: 0x0800,
                joy1_new: 0x0800,
                ..state
            };
            match invalid {
                1 => aim.joy1_current |= 0x0200,
                2 => aim.joy1_new |= word(&ram, 0x9b4),
                3 => aim.effective_buttons |= 0x80,
                4 => ram[0xb48] = 1,
                5 => ram[0xaf8] = 1,
                6 => ram[0xa1c] = 2,
                7 => aim.joy1_new |= word(&ram, 0x9b2),
                _ => {}
            }
            route.observe(&aim, &ram);
            if invalid != 0 {
                assert!(route.failure().is_some());
                assert_eq!(route.brake_aim_step, None);
                continue;
            }
            assert_eq!(route.failure(), None);
            assert_eq!(route.climb_phase, 30);
            ram[0xa1c] = 2;
            let held = State { host_steps: aim.host_steps + 1, polls: aim.polls + 1, ..aim };
            route.observe(&held, &ram);
            assert!(route.failure().is_some());
            assert_eq!(route.brake_rest, None);
        }
    }

    fn third_state() -> (State, Vec<u8>, Return) {
        let (state, mut ram, mut route) = brake_state();
        ram[0xa1c..0xa1e].copy_from_slice(&2u16.to_le_bytes());
        ram[0xa1f] = 0;
        ram[0xaf8..0xafa].fill(0);
        ram[0xb42..0xb4c].fill(0);
        ram[0x10002 + 2 * 135..0x10004 + 2 * 135].copy_from_slice(&0xc19au16.to_le_bytes());
        ram[0x16402 + 135] = 0;
        let rest = State { x: 117, effective_buttons: 0, joy1_current: 0, joy1_new: 0, ..state };
        route.brake_rest = Some((rest.host_steps, 117, 0));
        route.climb_phase = 31;
        route.previous = Some(rest);
        (rest, ram, route)
    }

    #[cfg_attr(test, test)]
    pub(super) fn third_ceiling_requires_consumed_aim_and_fresh_shot_before_same_poll_initial_draw_reaction() {
        let (rest, mut ram, mut route) = third_state();
        let pending = State { host_steps: rest.host_steps + 1, ..rest };
        ram[0xa1c] = 4;
        route.observe(&pending, &ram);
        assert_eq!(route.third_aim_step, None);
        let aim = State {
            host_steps: pending.host_steps + 1,
            polls: pending.polls + 1,
            effective_buttons: 0x10,
            joy1_current: 0x0800,
            joy1_new: 0x0800,
            ..pending
        };
        route.observe(&aim, &ram);
        assert_eq!(route.failure(), None);
        assert_eq!(route.third_aim_step, Some(aim.host_steps));
        let shoot = word(&ram, 0x9b2);
        assert_eq!(route.buttons(&aim, &ram), 0x10 | shoot.reverse_bits());
        ram[0x10002 + 2 * 135..0x10004 + 2 * 135].copy_from_slice(&0x0053u16.to_le_bytes());
        ram[0x1c37..0x1c39].copy_from_slice(&0xd064u16.to_le_bytes());
        ram[0x1c87..0x1c89].copy_from_slice(&0x010eu16.to_le_bytes());
        ram[0x1e17..0x1e19].copy_from_slice(&0xc052u16.to_le_bytes());
        ram[0x1d27..0x1d29].copy_from_slice(&0xcae6u16.to_le_bytes());
        ram[0xccc] = 13;
        let shot = State {
            host_steps: aim.host_steps + 1,
            polls: aim.polls + 1,
            effective_buttons: 0x10 | shoot.reverse_bits(),
            joy1_current: 0x0800 | shoot,
            joy1_new: shoot,
            ..aim
        };
        route.observe(&shot, &ram);
        assert_eq!(route.third_beam_step, Some(shot.host_steps));
        assert_eq!(route.third_response_step, Some(shot.host_steps));
        assert_eq!(route.third_plm_slot, Some(0));
        assert_eq!(route.failure(), None);
        assert_eq!(route.brake_rest, Some((rest.host_steps, 117, 0)));
    }

    #[cfg_attr(test, test)]
    pub(super) fn third_ceiling_refuses_foreign_input_motion_target_change_and_false_plm_reaction() {
        for invalid in 0..8 {
            let (rest, mut ram, mut route) = third_state();
            ram[0xa1c] = 4;
            let mut aim = State {
                host_steps: rest.host_steps + 1,
                polls: rest.polls + 1,
                effective_buttons: 0x10,
                joy1_current: 0x0800,
                joy1_new: 0x0800,
                ..rest
            };
            match invalid {
                0 => aim.joy1_current |= 0x0200,
                1 => aim.joy1_new |= word(&ram, 0x9b4),
                2 => ram[0xb48] = 1,
                3 => ram[0xaf8] = 1,
                4 => ram[0x10002 + 2 * 135] = 0,
                5 => route.third_beam_polls = 119,
                6 => ram[0xb5e] = 9,
                _ => {}
            }
            route.observe(&aim, &ram);
            if invalid == 7 {
                let shoot = word(&ram, 0x9b2);
                ram[0x10002 + 2 * 135..0x10004 + 2 * 135].copy_from_slice(&0x0053u16.to_le_bytes());
                let shot = State {
                    host_steps: aim.host_steps + 1,
                    polls: aim.polls + 1,
                    effective_buttons: 0x10 | shoot.reverse_bits(),
                    joy1_current: 0x0800 | shoot,
                    joy1_new: shoot,
                    ..aim
                };
                route.observe(&shot, &ram);
            }
            assert!(route.failure().is_some());
            assert_eq!(route.third_response_step, None);
        }
    }

    #[cfg_attr(test, test)]
    pub(super) fn third_ceiling_delayed_reaction_requires_consumed_release_and_ready_preceding_observation() {
        for invalid in 0..8 {
            let (rest, mut ram, mut route) = third_state();
            for (col, tile, bts) in [(6, 0xc19au16, 2u8), (8, 0x00ff, 7)] {
                ram[0x10002 + 2 * (128 + col)..0x10004 + 2 * (128 + col)].copy_from_slice(&tile.to_le_bytes());
                ram[0x16402 + 128 + col] = bts;
            }
            ram[0xa1c] = 4;
            let aim = State {
                host_steps: rest.host_steps + 1,
                polls: rest.polls + 1,
                effective_buttons: 0x10,
                joy1_current: 0x0800,
                joy1_new: 0x0800,
                ..rest
            };
            route.observe(&aim, &ram);
            if invalid == 1 || invalid == 2 {
                ram[if invalid == 1 { 0xccc } else { 0xcce }] = if invalid == 1 { 13 } else { 5 };
                route.observe(&State { host_steps: aim.host_steps + 1, ..aim }, &ram);
                assert!(route.failure().is_some());
                assert_eq!(route.third_beam_step, None);
                assert_eq!(route.buttons(&aim, &ram), 0);
                continue;
            }
            let shoot = word(&ram, 0x9b2);
            ram[0xccc] = 13;
            let shot = State {
                host_steps: aim.host_steps + 1,
                polls: aim.polls + 1,
                effective_buttons: 0x10 | shoot.reverse_bits(),
                joy1_current: 0x0800 | shoot,
                joy1_new: shoot,
                ..aim
            };
            route.observe(&shot, &ram);
            assert_eq!(route.failure(), None);
            assert_eq!(route.buttons(&shot, &ram), 0x10);
            ram[0x10002 + 2 * 134..0x10004 + 2 * 134].copy_from_slice(&0x0053u16.to_le_bytes());
            ram[0x16402 + 134] = 3;
            ram[0x10002 + 2 * 135..0x10004 + 2 * 135].copy_from_slice(&0x0053u16.to_le_bytes());
            for (at, value) in [(0x1c37, 0xd064u16), (0x1c87, 0x010e), (0x1e17, 0xc052)] {
                ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
            }
            if (3..6).contains(&invalid) {
                ram[[0x1c37, 0x1c87, 0x1e17][invalid - 3]] = 0;
            }
            ram[0x1d27..0x1d29].copy_from_slice(&0xcae6u16.to_le_bytes());
            if invalid == 6 {
                ram[0x1d27] = 0;
            }
            if invalid == 7 {
                ram[0x10002 + 2 * 135] = 0x54;
            }
            let pending = State { host_steps: shot.host_steps + 1, ..shot };
            route.observe(&pending, &ram);
            assert_eq!(route.third_response_step, None);
            let response = State {
                host_steps: pending.host_steps + 1,
                polls: pending.polls + 1,
                effective_buttons: 0x10,
                joy1_current: 0x0800,
                joy1_new: 0,
                ..pending
            };
            route.observe(&response, &ram);
            if invalid == 0 {
                assert_eq!(route.third_response_step, Some(response.host_steps));
                let mut report = sm_binary_evidence::Map::new();
                route.report(&mut report);
                assert_eq!(report["rio_return_third_before_c7"].as_integer(), Some(0xc19a));
                assert_eq!(report["rio_return_third_after_c7"].as_integer(), Some(0x0053));
                assert_eq!(report["rio_return_third_after_c6"].as_integer(), Some(0x0053));
                assert_eq!(report["rio_return_third_after_bts_c6"].as_integer(), Some(3));
                assert_eq!(report["rio_return_third_plm_header"].as_integer(), Some(0xd064));
                assert_eq!(report["rio_return_third_plm_block"].as_integer(), Some(0x010e));
                assert_eq!(report["rio_return_third_plm_variable"].as_integer(), Some(0xc052));
                for (col, before, after, bts_before, bts_after) in
                    [(6, 0xc19a, 0x0053, 2, 3), (7, 0xc19a, 0x0053, 0, 0), (8, 0x00ff, 0x00ff, 7, 7)]
                {
                    assert_eq!(report[&format!("rio_return_third_before_c{col}")].as_integer(), Some(before));
                    assert_eq!(report[&format!("rio_return_third_after_c{col}")].as_integer(), Some(after));
                    assert_eq!(report[&format!("rio_return_third_before_bts_c{col}")].as_integer(), Some(bts_before));
                    assert_eq!(report[&format!("rio_return_third_after_bts_c{col}")].as_integer(), Some(bts_after));
                }
                assert_eq!(report["rio_return_third_plm_instruction"].as_integer(), Some(0xcae6));
                assert_eq!(report["rio_return_third_plm_slot"].as_integer(), Some(0));
            } else {
                assert_eq!(route.third_response_step, None);
            }
            assert_eq!(route.failure().is_some(), invalid != 0);
        }
    }

    #[cfg_attr(test, test)]
    pub(super) fn upper_transfer_releases_left_at_full_span_and_holds_until_native_contact() {
        let (state, mut ram, mut route) = upper_transfer_state();
        route.climb_phase = 36;
        route.upper_feet_step = Some(state.host_steps);
        ram[0xa1c..0xa1e].copy_from_slice(&0x52u16.to_le_bytes());
        ram[0xa1f] = 2;
        for (at, value) in [(0xb00, 19u16), (0xaf8, 0x8000), (0xb46, 1), (0xb48, 0x4000), (0xb36, 2)] {
            ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
        }
        ram[0xa6c..0xa6e].copy_from_slice(&0x9f55u16.to_le_bytes());
        let jump = word(&ram, 0x9b4);
        let pending = State {
            host_steps: state.host_steps + 1,
            x: 90,
            y: 67,
            effective_buttons: 0x40 | jump.reverse_bits(),
            joy1_current: 0x200 | jump,
            joy1_new: 0,
            ..state
        };
        route.observe(&pending, &ram);
        assert_eq!(route.upper_hold, None);
        let span = State { host_steps: pending.host_steps + 1, polls: pending.polls + 1, ..pending };
        route.observe(&span, &ram);
        assert_eq!(route.failure(), None);
        assert_eq!(route.upper_hold, Some((span.host_steps, 90, 0x8000)));
        assert_eq!(route.buttons(&span, &ram), jump.reverse_bits());
        let unpolled = State { host_steps: span.host_steps + 1, ..span };
        route.observe(&unpolled, &ram);
        assert_eq!(route.upper_left_release_step, None);
        ram[0xb46..0xb4a].fill(0);
        ram[0xa1c..0xa1e].copy_from_slice(&0x4eu16.to_le_bytes());
        let released = State {
            host_steps: unpolled.host_steps + 1,
            polls: unpolled.polls + 1,
            effective_buttons: jump.reverse_bits(),
            joy1_current: jump,
            ..unpolled
        };
        route.observe(&released, &ram);
        assert_eq!(route.failure(), None);
        assert_eq!(route.upper_left_release_step, Some(released.host_steps));
        ram[0xa1c..0xa1e].copy_from_slice(&0xa5u16.to_le_bytes());
        ram[0xa1f] = 0;
        ram[0xb36..0xb38].fill(0);
        let contact_pending = State { host_steps: released.host_steps + 1, y: 107, ..released };
        route.observe(&contact_pending, &ram);
        assert!(route.upper_contact.is_none());
        let contact =
            State { host_steps: contact_pending.host_steps + 1, polls: contact_pending.polls + 1, ..contact_pending };
        route.observe(&contact, &ram);
        assert_eq!(route.failure(), None);
        assert_eq!(route.climb_phase, 39);
        assert_eq!(route.buttons(&contact, &ram), 0);
        let mut report = sm_binary_evidence::Map::new();
        route.report(&mut report);
        for (key, value) in [
            ("step", span.host_steps as i64),
            ("x", 90),
            ("x_subposition", 0x8000),
            ("left_release_step", released.host_steps as i64),
        ] {
            assert_eq!(report[&format!("rio_return_upper_hold_{key}")].as_integer(), Some(value));
        }
        for (key, value) in [
            ("speed", 1),
            ("subspeed", 0x4000),
            ("extra_speed", 0),
            ("extra_subspeed", 0),
            ("accel_mode", 0),
            ("speed_table", 0x9f55),
        ] {
            assert_eq!(report[&format!("rio_return_upper_hold_{key}")].as_integer(), Some(value));
        }
    }

    #[cfg_attr(test, test)]
    pub(super) fn upper_hold_rejects_drift_foreign_momentum_and_unreleased_left() {
        for invalid in 0..8 {
            let (state, mut ram, mut route) = upper_transfer_state();
            route.climb_phase = 38;
            route.upper_hold = Some((state.host_steps, 90, 0x8000));
            route.upper_feet_step = Some(state.host_steps);
            ram[0xa1c..0xa1e].copy_from_slice(&0x52u16.to_le_bytes());
            ram[0xa1f] = 2;
            ram[0xb00..0xb02].copy_from_slice(&19u16.to_le_bytes());
            ram[0xaf8..0xafa].copy_from_slice(&0x8000u16.to_le_bytes());
            ram[0xb36..0xb38].copy_from_slice(&2u16.to_le_bytes());
            let jump = word(&ram, 0x9b4);
            let mut next = State {
                host_steps: state.host_steps + 1,
                polls: state.polls + 1,
                x: 90,
                y: 67,
                effective_buttons: jump.reverse_bits(),
                joy1_current: jump,
                joy1_new: 0,
                ..state
            };
            match invalid {
                0 => next.x = 89,
                1 => ram[0xaf8] = 1,
                2..=6 => ram[[0xb42, 0xb44, 0xb46, 0xb48, 0xb4a][invalid - 2]] = 1,
                7 => {
                    next.effective_buttons |= 0x40;
                    next.joy1_current |= 0x200;
                }
                _ => unreachable!(),
            }
            route.observe(&next, &ram);
            assert!(route.failure().is_some());
            assert_eq!(route.upper_left_release_step, None);
            assert!(route.upper_contact.is_none());
        }
        for at in [0xb42, 0xb44, 0xb4a] {
            let (state, mut ram, mut route) = upper_transfer_state();
            route.climb_phase = 36;
            route.upper_feet_step = Some(state.host_steps);
            ram[0xa1c..0xa1e].copy_from_slice(&0x52u16.to_le_bytes());
            ram[0xa1f] = 2;
            ram[0xb00..0xb02].copy_from_slice(&19u16.to_le_bytes());
            ram[at] = 1;
            let jump = word(&ram, 0x9b4);
            let span = State {
                host_steps: state.host_steps + 1,
                polls: state.polls + 1,
                x: 90,
                y: 67,
                effective_buttons: 0x40 | jump.reverse_bits(),
                joy1_current: 0x200 | jump,
                joy1_new: 0,
                ..state
            };
            route.observe(&span, &ram);
            assert_eq!(route.failure(), Some("return upper hold cannot stop foreign airborne momentum"));
            assert_eq!(route.upper_hold, None);
            assert_eq!(route.upper_hold_motion, None);
            assert_eq!(route.upper_left_release_step, None);
        }
    }

    #[cfg_attr(test, test)]
    pub(super) fn upper_rest_waits_for_consumed_neutral_landing_animation_and_normal_radius() {
        let (state, mut ram, mut route) = upper_transfer_state();
        route.climb_phase = 39;
        route.upper_contact = Some(TransferContact {
            step: state.host_steps,
            x: 90,
            x_subposition: 0x8000,
            pose: 0xa5,
            radius: 19,
            speed: 0,
            subspeed: 0,
            extra_speed: 0,
            extra_subspeed: 0,
            accel_mode: 0,
            speed_table: 0x9f55,
        });
        ram[0xaf8..0xafa].copy_from_slice(&0x8000u16.to_le_bytes());
        ram[0xa1c..0xa1e].copy_from_slice(&0xa5u16.to_le_bytes());
        ram[0xb00..0xb02].copy_from_slice(&19u16.to_le_bytes());
        let pending = State {
            host_steps: state.host_steps + 1,
            x: 90,
            y: 107,
            effective_buttons: word(&ram, 0x9b4).reverse_bits(),
            joy1_current: word(&ram, 0x9b4),
            joy1_new: 0,
            ..state
        };
        route.observe(&pending, &ram);
        assert_eq!(route.failure(), None);
        assert_eq!(route.upper_rest_step, None);
        ram[0xb00..0xb02].copy_from_slice(&21u16.to_le_bytes());
        let landing = State {
            host_steps: pending.host_steps + 1,
            polls: pending.polls + 1,
            effective_buttons: 0,
            joy1_current: 0,
            joy1_new: 0,
            ..pending
        };
        route.observe(&landing, &ram);
        assert_eq!(route.failure(), None);
        assert_eq!(route.upper_rest_step, None);
        assert_eq!(route.buttons(&landing, &ram), 0);
        ram[0xa1c..0xa1e].copy_from_slice(&2u16.to_le_bytes());
        let normal_pending = State { host_steps: landing.host_steps + 1, ..landing };
        route.observe(&normal_pending, &ram);
        assert_eq!(route.upper_rest_step, None);
        let rest =
            State { host_steps: normal_pending.host_steps + 1, polls: normal_pending.polls + 1, ..normal_pending };
        route.observe(&rest, &ram);
        assert_eq!(route.failure(), None);
        assert_eq!(route.climb_phase, 40);
        assert_eq!(route.upper_rest_step, Some(rest.host_steps));
        let mut report = sm_binary_evidence::Map::new();
        route.report(&mut report);
        assert_eq!(report["rio_return_upper_rest_step"].as_integer(), Some(rest.host_steps as i64));
        assert_eq!(report["rio_return_complete"].as_bool(), Some(false));
    }

    #[cfg_attr(test, test)]
    pub(super) fn upper_rest_rejects_stale_jump_drift_motion_wrong_radius_and_partial_support() {
        for invalid in 0..14 {
            let (state, mut ram, mut route) = upper_transfer_state();
            route.climb_phase = 39;
            route.upper_contact = Some(TransferContact {
                step: state.host_steps,
                x: 90,
                x_subposition: 0,
                pose: 0xa5,
                radius: 19,
                speed: 0,
                subspeed: 0,
                extra_speed: 0,
                extra_subspeed: 0,
                accel_mode: 0,
                speed_table: 0x9f55,
            });
            ram[0xa1c..0xa1e].copy_from_slice(&2u16.to_le_bytes());
            let mut rest = State { host_steps: state.host_steps + 1, polls: state.polls + 1, x: 90, y: 107, ..state };
            match invalid {
                0 => {
                    rest.effective_buttons = word(&ram, 0x9b4).reverse_bits();
                    rest.joy1_current = word(&ram, 0x9b4);
                }
                1 => ram[0xaf8] = 1,
                2 => rest.x = 91,
                3..=7 => ram[[0xb42, 0xb44, 0xb46, 0xb48, 0xb4a][invalid - 3]] = 1,
                8 => ram[0xb00] = 19,
                9 => ram[0xa1c] = 4,
                10 => rest.joy1_new = word(&ram, 0x9b4),
                11 => ram[0x10002 + 2 * 133 + 1] = 0,
                12 => rest.effective_buttons = word(&ram, 0x9b4).reverse_bits(),
                13 => rest.joy1_current = word(&ram, 0x9b4),
                _ => unreachable!(),
            }
            route.observe(&rest, &ram);
            assert!(route.failure().is_some());
            assert_eq!(route.upper_rest_step, None);
        }
    }

    fn door_state() -> (State, Vec<u8>, Return) {
        let (state, mut ram, mut route) = upper_transfer_state();
        let state = State { x: 90, y: 107, effective_buttons: 0, joy1_current: 0, joy1_new: 0, ..state };
        route.previous = Some(state);
        route.climb_phase = 40;
        route.upper_rest_step = Some(state.host_steps);
        route.upper_transfer_polls = 74;
        route.upper_contact = Some(TransferContact {
            step: state.host_steps,
            pose: 0xa5,
            radius: 19,
            x: 90,
            x_subposition: 0,
            speed: 0,
            subspeed: 0,
            extra_speed: 0,
            extra_subspeed: 0,
            accel_mode: 0,
            speed_table: 0x9f55,
        });
        ram[0xa1c..0xa1e].copy_from_slice(&2u16.to_le_bytes());
        ram[0xccc..0xcd0].fill(0);
        for (row, tile, bts) in [(6, 0xc40cu16, 0x41), (7, 0xd42c, 0xff), (8, 0xdc2c, 0xfe), (9, 0xdc0c, 0xfd)] {
            ram[0x10002 + 2 * (16 * row + 1)..0x10004 + 2 * (16 * row + 1)].copy_from_slice(&tile.to_le_bytes());
            ram[0x16402 + 16 * row + 1] = bts;
            ram[0x10002 + 2 * 16 * row..0x10004 + 2 * 16 * row].copy_from_slice(&0x9000u16.to_le_bytes());
            ram[0x16402 + 16 * row] = 0;
        }
        for row in 6..8 {
            for col in 2..6 {
                ram[0x10002 + 2 * (16 * row + col)..0x10004 + 2 * (16 * row + col)]
                    .copy_from_slice(&0xffu16.to_le_bytes());
            }
        }
        route.door.before = Some([0xc40c, 0xd42c, 0xdc2c, 0xdc0c]);
        route.door.before_bts = Some([0x41, 0xff, 0xfe, 0xfd]);
        // Leave slot2 highest free, carrying a deleted block's singular value.
        for slot in 3..40 {
            ram[0x1c37 + 2 * slot..0x1c39 + 2 * slot].copy_from_slice(&0xd064u16.to_le_bytes());
        }
        ram[0x1e1b..0x1e1d].copy_from_slice(&0xc052u16.to_le_bytes());
        route.door.allocation = Some((2, 0xc052));
        route.door.ready = true;
        (state, ram, route)
    }

    fn draw_door(ram: &mut [u8]) {
        for (row, tile) in [(6, 0x840du16), (7, 0x842d), (8, 0x8c2d), (9, 0x8c0d)] {
            ram[0x10002 + 2 * (16 * row + 1)..0x10004 + 2 * (16 * row + 1)].copy_from_slice(&tile.to_le_bytes());
        }
        for (at, value) in [(0x1c37, 0xc8a8u16), (0x1c87, 0xc2), (0x1d27, 0xc4c1)] {
            ram[at + 4..at + 6].copy_from_slice(&value.to_le_bytes());
        }
    }

    #[cfg_attr(test, test)]
    pub(super) fn door_shot_releases_and_requires_consumed_first_solid_draw() {
        for (immediate, inherited) in [(false, 0xc052u16), (true, 0xc052), (true, 0)] {
            let (state, mut ram, mut route) = door_state();
            ram[0x1e1b..0x1e1d].copy_from_slice(&inherited.to_le_bytes());
            ram[0xdf10..0xdf12].copy_from_slice(&0xffffu16.to_le_bytes());
            route.door.allocation = Some((2, inherited));
            let shoot = word(&ram, 0x9b2);
            let pending = State { host_steps: state.host_steps + 1, ..state };
            route.observe(&pending, &ram);
            assert_eq!(route.door.shot_step, None);
            assert_eq!(route.buttons(&pending, &ram), shoot.reverse_bits());
            if immediate {
                draw_door(&mut ram);
                ram[0xdf10..0xdf12].fill(0);
            }
            // Native post-update position can differ from source origin79/102.
            for (at, value) in [(0xc1c, 1u16), (0xc08, 7), (0xb68, 75), (0xb7c, 102)] {
                ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
            }
            ram[0xccc..0xcce].copy_from_slice(&13u16.to_le_bytes());
            let shot = State {
                host_steps: pending.host_steps + 1,
                polls: pending.polls + 1,
                effective_buttons: shoot.reverse_bits(),
                joy1_current: shoot,
                joy1_new: shoot,
                ..pending
            };
            route.observe(&shot, &ram);
            assert_eq!(route.door.shot_step, Some(shot.host_steps));
            if !immediate {
                assert_eq!(route.failure(), None);
                assert_eq!(route.buttons(&shot, &ram), 0);
                let release = State {
                    host_steps: shot.host_steps + 1,
                    polls: shot.polls + 1,
                    effective_buttons: 0,
                    joy1_current: 0,
                    joy1_new: 0,
                    ..shot
                };
                route.observe(&release, &ram);
                assert_eq!(route.door.release_step, Some(release.host_steps));
                draw_door(&mut ram);
                ram[0xdf10..0xdf12].fill(0);
                let unpolled = State { host_steps: release.host_steps + 1, ..release };
                route.observe(&unpolled, &ram);
                assert_eq!(route.door.response_step, None);
                let response = State { host_steps: unpolled.host_steps + 1, polls: unpolled.polls + 1, ..unpolled };
                route.observe(&response, &ram);
                assert_eq!(route.door.response_step, Some(response.host_steps));
            } else {
                assert_eq!(route.door.response_step, Some(shot.host_steps));
                assert_eq!(route.door.release_step, None);
            }
            assert_eq!(route.failure(), None);
            assert_eq!(route.climb_phase, 42);
            assert_eq!(route.buttons(&shot, &ram), 0);
            assert_eq!(route.upper_transfer_polls, 74);
            let mut report = sm_binary_evidence::Map::new();
            route.report(&mut report);
            for (row, before, after, bts) in [
                (6, 0xc40c, 0x840d, 0x41),
                (7, 0xd42c, 0x842d, 0xff),
                (8, 0xdc2c, 0x8c2d, 0xfe),
                (9, 0xdc0c, 0x8c0d, 0xfd),
            ] {
                for (key, value) in [("before", before), ("after", after), ("before_bts", bts), ("after_bts", bts)] {
                    assert_eq!(report[&format!("rio_return_door_{key}_row{row}")].as_integer(), Some(value));
                }
            }
            assert_eq!(report["rio_return_complete"].as_bool(), Some(false));
            for (key, value) in [
                ("slot", 2),
                ("header", 0xc8a8),
                ("block", 0xc2),
                ("instruction", 0xc4c1),
                ("variable", i64::from(inherited)),
            ] {
                assert_eq!(report[&format!("rio_return_door_plm_{key}")].as_integer(), Some(value));
            }
            for (key, value) in [("slot", 2), ("x", 75), ("y", 102), ("direction", 7)] {
                assert_eq!(report[&format!("rio_return_door_projectile_{key}")].as_integer(), Some(value));
            }
            assert_eq!(report["rio_return_door_shot_step"].as_integer(), Some(shot.host_steps as i64));
            assert_eq!(
                report["rio_return_door_response_step"].as_integer(),
                Some((shot.host_steps + if immediate { 0 } else { 3 }) as i64)
            );
            assert_eq!(
                report["rio_return_door_release_step"].as_integer(),
                Some(if immediate { -1 } else { (shot.host_steps + 1) as i64 })
            );
            assert_eq!(report["rio_return_door_polls"].as_integer(), Some(if immediate { 1 } else { 3 }));
            assert_eq!(report["rio_return_door_allocation_slot"].as_integer(), Some(2));
            assert_eq!(report["rio_return_door_allocation_variable"].as_integer(), Some(i64::from(inherited)));
        }
        let (state, mut ram, mut route) = door_state();
        route.door.ready = false;
        ram[0xccc..0xcce].copy_from_slice(&1u16.to_le_bytes());
        let wait = State { host_steps: state.host_steps + 1, polls: state.polls + 1, ..state };
        route.observe(&wait, &ram);
        assert_eq!(route.failure(), None);
        assert!(!route.door.ready);
        ram[0xccc..0xcce].fill(0);
        let pending = State { host_steps: wait.host_steps + 1, ..wait };
        route.observe(&pending, &ram);
        assert!(!route.door.ready);
        assert_eq!(route.buttons(&pending, &ram), 0);
        let ready = State { host_steps: pending.host_steps + 1, polls: pending.polls + 1, ..pending };
        route.observe(&ready, &ram);
        assert!(route.door.ready);
        assert_eq!(route.door.allocation, Some((2, 0xc052)));
        assert_eq!(route.buttons(&ready, &ram), word(&ram, 0x9b2).reverse_bits());
        let (state, mut ram, mut route) = door_state();
        route.door.ready = false;
        route.door.allocation = None;
        ram[0x1c3b..0x1c3d].copy_from_slice(&0xd064u16.to_le_bytes());
        let full = State { host_steps: state.host_steps + 1, polls: state.polls + 1, ..state };
        route.observe(&full, &ram);
        assert_eq!(route.failure(), None);
        assert!(!route.door.ready);
        assert_eq!(route.door.allocation, None);
        assert_eq!(route.buttons(&full, &ram), 0);
    }

    #[cfg_attr(test, test)]
    pub(super) fn door_refuses_wrong_first_draw_identity_and_unowned_input() {
        for invalid in 0..16 {
            let (state, mut ram, mut route) = door_state();
            let shoot = word(&ram, 0x9b2);
            let mut next = State {
                host_steps: state.host_steps + 1,
                polls: state.polls + 1,
                effective_buttons: shoot.reverse_bits(),
                joy1_current: shoot,
                joy1_new: shoot,
                ..state
            };
            draw_door(&mut ram);
            match invalid {
                0 => next.joy1_new = 0,
                1 => next.joy1_current |= word(&ram, 0x9b4),
                2 => next.effective_buttons = 0,
                3 => ram[0x1c3b] = 0,
                4 => ram[0x1c8b] = 0,
                5 => ram[0x1d2b] = 0,
                6 => ram[0x1e1b] = 1,
                7 => ram[0x10002 + 2 * 97] = 0x0e,
                8 => ram[0x16402 + 97] = 0,
                9 => ram[0xaf8] = 1,
                10 => ram[0xb48] = 1,
                11 => route.door.polls = 119,
                12 => route.door.before = None,
                13 => route.door.ready = false,
                14 => route.door.allocation = None,
                15 => route.door.allocation = Some((1, 0xc052)),
                _ => unreachable!(),
            }
            route.observe(&next, &ram);
            assert!(route.failure().is_some());
            assert_eq!(route.door.response_step, None);
        }
        for invalid in 0..10 {
            let (state, mut ram, mut route) = door_state();
            let pending = State { host_steps: state.host_steps + 1, ..state };
            match invalid {
                0 => ram[0xccc] = 1,
                1 => ram[0xcce] = 1,
                2 => ram[0x10002 + 2 * 97] = 0x0d,
                3 => {
                    route.door.ready = false;
                    ram[0x16402 + 97] = 0;
                }
                4 => {
                    route.door.ready = false;
                    ram[0x10002 + 2 * 100..0x10004 + 2 * 100].copy_from_slice(&0x8000u16.to_le_bytes());
                }
                5 => {
                    route.door.ready = false;
                    ram[0x16402 + 96] = 1;
                }
                6 | 8 => ram[0x1c3b..0x1c3d].copy_from_slice(&0xd064u16.to_le_bytes()),
                7 | 9 => ram[0x1e1b] = 1,
                _ => unreachable!(),
            }
            let pending =
                if matches!(invalid, 4 | 5) { State { polls: pending.polls + 1, ..pending } } else { pending };
            let shoot = word(&ram, 0x9b2);
            let pending = if invalid >= 8 {
                State {
                    polls: pending.polls + 1,
                    effective_buttons: shoot.reverse_bits(),
                    joy1_current: shoot,
                    joy1_new: shoot,
                    ..pending
                }
            } else {
                pending
            };
            route.observe(&pending, &ram);
            assert!(route.failure().is_some());
            assert_eq!(route.buttons(&pending, &ram), 0);
            assert_eq!(route.door.shot_step, None);
            if matches!(invalid, 4 | 5) {
                assert!(!route.door.ready);
            }
        }
        let (state, mut ram, mut route) = door_state();
        route.upper_rest_step = None;
        route.climb_phase = 39;
        route.door.ready = false;
        ram[0x10002 + 2 * 97] = 0x0d;
        let rest = State { host_steps: state.host_steps + 1, polls: state.polls + 1, ..state };
        route.observe(&rest, &ram);
        assert_eq!(route.failure(), None);
        assert_eq!(route.buttons(&rest, &ram), 0);
        assert!(!route.door.ready);
        let validate = State { host_steps: rest.host_steps + 1, polls: rest.polls + 1, ..rest };
        route.observe(&validate, &ram);
        assert!(route.failure().is_some());
        assert_eq!(route.buttons(&validate, &ram), 0);
        assert_eq!(route.door.shot_step, None);
    }

    fn opening_state() -> (State, Vec<u8>, Return) {
        let (state, mut ram, mut route) = door_state();
        draw_door(&mut ram);
        route.climb_phase = 42;
        route.door.shot_step = Some(state.host_steps - 1);
        route.door.response_step = Some(state.host_steps);
        route.door.polls = 10;
        (state, ram, route)
    }

    fn opening_draw(ram: &mut [u8], tiles: [u16; 4], ip: u16) {
        for (row, tile) in (6..10).zip(tiles) {
            ram[0x10002 + 2 * (16 * row + 1)..0x10004 + 2 * (16 * row + 1)].copy_from_slice(&tile.to_le_bytes());
        }
        ram[0x1d2b..0x1d2d].copy_from_slice(&ip.to_le_bytes());
        ram[0xde20..0xde22].copy_from_slice(&if ip == 0xc4cd { 94u16 } else { 6 }.to_le_bytes());
        ram[0x1d7b..0x1d7d].copy_from_slice(&0x1234u16.to_le_bytes());
    }

    #[cfg_attr(test, test)]
    pub(super) fn opening_requires_ordered_consumed_neutral_full_air_draw() {
        let (mut state, mut ram, mut route) = opening_state();
        for (tiles, ip) in [
            ([0x840e, 0x842e, 0x8c2e, 0x8c0e], 0xc4c5),
            ([0x840f, 0x042f, 0x0c2f, 0x8c0f], 0xc4c9),
            ([0x0482, 0x04a2, 0x0ca2, 0x0c82], 0xc4cd),
        ] {
            opening_draw(&mut ram, tiles, ip);
            let pending = State { host_steps: state.host_steps + 1, ..state };
            route.observe(&pending, &ram);
            assert_eq!(route.door.air_step, None);
            assert_eq!(route.buttons(&pending, &ram), 0);
            state = State { host_steps: pending.host_steps + 1, polls: pending.polls + 1, ..pending };
            route.observe(&state, &ram);
            if ip != 0xc4cd {
                assert_eq!(route.failure(), None);
                assert_eq!(route.door.air_step, None);
            }
        }
        assert_eq!(route.failure(), None);
        assert_eq!(route.climb_phase, 43);
        assert_eq!(route.buttons(&state, &ram), 0x40);
        assert_eq!(route.door.air_step, Some(state.host_steps));
        assert_eq!(route.door.polls, 10);
        assert_eq!(route.upper_transfer_polls, 74);
        let mut report = sm_binary_evidence::Map::new();
        route.report(&mut report);
        assert_eq!(report["rio_return_complete"].as_bool(), Some(false));
        assert_eq!(report["rio_return_door_opening_polls"].as_integer(), Some(3));
        assert_eq!(report["rio_return_door_opening_stage"].as_integer(), Some(3));
        assert_eq!(report["rio_return_door_air_step"].as_integer(), Some(state.host_steps as i64));
        assert_eq!(report["rio_return_door_air_plm_timer"].as_integer(), Some(94));
        for (row, bts) in [(6, 0x41), (7, 0xff), (8, 0xfe), (9, 0xfd)] {
            assert_eq!(report[&format!("rio_return_door_air_bts_row{row}")].as_integer(), Some(bts));
        }
        for (row, tile) in [(6, 0x0482), (7, 0x04a2), (8, 0x0ca2), (9, 0x0c82)] {
            assert_eq!(report[&format!("rio_return_door_air_row{row}")].as_integer(), Some(tile));
        }
        for (key, value) in
            [("slot", 2), ("header", 0xc8a8), ("block", 0xc2), ("instruction", 0xc4cd), ("variable", 0xc052)]
        {
            assert_eq!(report[&format!("rio_return_door_air_plm_{key}")].as_integer(), Some(value));
        }
    }

    #[cfg_attr(test, test)]
    pub(super) fn opening_refuses_skipped_draws_deleted_plm_and_held_inputs() {
        for invalid in 0..12 {
            let (state, mut ram, mut route) = opening_state();
            let mut next = State { host_steps: state.host_steps + 1, polls: state.polls + 1, ..state };
            match invalid {
                0 => opening_draw(&mut ram, [0x0482, 0x04a2, 0x0ca2, 0x0c82], 0xc4cd),
                1 => opening_draw(&mut ram, [0x840e, 0x842e, 0x8c2e, 0x8c0e], 0xc4c9),
                2 => ram[0x1c3b] = 0,
                3 => ram[0x1e1b] = 0,
                4 => next.effective_buttons = 0x200,
                5 => next.joy1_current = word(&ram, 0x9b2),
                6 => next.joy1_new = word(&ram, 0x9b2),
                7 => route.door.opening_polls = 119,
                8 => ram[0x16402 + 97] = 0,
                9 => next.x = 89,
                10 => route.door.opening_stage = 1,
                11 => route.door.allocation = None,
                _ => unreachable!(),
            }
            route.observe(&next, &ram);
            assert!(route.failure().is_some());
            assert_eq!(route.door.air_step, None);
            assert_eq!(route.buttons(&next, &ram), 0);
        }
    }

    fn approach_state() -> (State, Vec<u8>, Return) {
        let (state, mut ram, mut route) = opening_state();
        opening_draw(&mut ram, [0x0482, 0x04a2, 0x0ca2, 0x0c82], 0xc4cd);
        route.climb_phase = 43;
        route.door.air_step = Some(state.host_steps);
        route.door.release_step = Some(state.host_steps - 1);
        route.door.opening_polls = 18;
        ram[0x10002 + 2 * 132..0x10004 + 2 * 132].copy_from_slice(&0x80ffu16.to_le_bytes());
        for row in 8..10 {
            for col in 2..4 {
                ram[0x10002 + 2 * (16 * row + col)..0x10004 + 2 * (16 * row + col)].fill(0);
            }
        }
        for col in 0..4 {
            ram[0x10002 + 2 * (160 + col)..0x10004 + 2 * (160 + col)].copy_from_slice(&0x81ecu16.to_le_bytes());
        }
        (state, ram, route)
    }

    #[cfg_attr(test, test)]
    pub(super) fn left_approach_stops_only_at_consumed_native_falling_pose() {
        for radius in [19u16, 21] {
            let (state, mut ram, mut route) = approach_state();
            assert_eq!(route.buttons(&state, &ram), 0x40);
            assert_eq!(route.buttons(&state, &ram).reverse_bits(), 0x0200);
            ram[0xa1c..0xa1e].copy_from_slice(&0x0au16.to_le_bytes());
            ram[0xa1f] = 1;
            let walking = State {
                host_steps: state.host_steps + 1,
                polls: state.polls + 1,
                x: 80,
                effective_buttons: 0x40,
                joy1_current: 0x200,
                joy1_new: 0x200,
                ..state
            };
            route.observe(&walking, &ram);
            assert_eq!(route.failure(), None);
            assert_eq!(route.door.approach_left_step, Some(walking.host_steps));
            ram[0xa1c..0xa1e].copy_from_slice(&0x2au16.to_le_bytes());
            ram[0xa1f] = 6;
            ram[0xb00..0xb02].copy_from_slice(&radius.to_le_bytes());
            ram[0xb36..0xb38].copy_from_slice(&2u16.to_le_bytes());
            for (at, value) in [(0xaf8, 0x1234u16), (0xafc, 0x5678), (0xb46, 2), (0xb48, 0x8000), (0xb4a, 1)] {
                ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
            }
            let pending = State { host_steps: walking.host_steps + 1, x: 58, joy1_new: 0, ..walking };
            route.observe(&pending, &ram);
            assert_eq!(route.door.approach_fall_step, None);
            let fall = State { host_steps: pending.host_steps + 1, polls: pending.polls + 1, ..pending };
            route.observe(&fall, &ram);
            assert_eq!(route.failure(), None);
            assert_eq!(route.climb_phase, 44);
            assert_eq!(route.buttons(&fall, &ram), 0x40);
            let mut report = sm_binary_evidence::Map::new();
            route.report(&mut report);
            assert_eq!(report["rio_return_complete"].as_bool(), Some(false));
            assert_eq!(report["rio_return_door_approach_left_step"].as_integer(), Some(walking.host_steps as i64));
            assert_eq!(
                report["rio_return_door_approach_fall_step"].as_integer(),
                Some((walking.host_steps + 2) as i64)
            );
            assert_eq!(report["rio_return_door_approach_polls"].as_integer(), Some(2));
            for (key, value) in [("x", 58), ("y", 107), ("pose", 0x2a), ("movement", 6), ("radius", i64::from(radius))]
            {
                assert_eq!(report[&format!("rio_return_door_approach_fall_{key}")].as_integer(), Some(value));
            }
            assert_eq!(route.door.opening_polls, 18);
            assert_eq!(route.door.polls, 10);
            assert_eq!(route.upper_transfer_polls, 74);
            for (key, value) in [
                ("x_subposition", 0x1234),
                ("y_subposition", 0x5678),
                ("x_speed", 2),
                ("x_subspeed", 0x8000),
                ("x_accel_mode", 1),
                ("y_speed", 0),
                ("y_subspeed", 0),
                ("y_direction", 2),
            ] {
                assert_eq!(report[&format!("rio_return_door_approach_fall_{key}")].as_integer(), Some(value));
            }
        }
    }

    #[cfg_attr(test, test)]
    pub(super) fn left_approach_refuses_unowned_input_and_changed_air_domain() {
        for invalid in 0..13 {
            let (state, mut ram, mut route) = approach_state();
            let mut next = State {
                host_steps: state.host_steps + 1,
                polls: state.polls + 1,
                effective_buttons: 0x40,
                joy1_current: 0x200,
                joy1_new: 0x200,
                ..state
            };
            match invalid {
                0 => next.effective_buttons |= 0x200,
                1 => next.joy1_current |= 0x40,
                2 => next.joy1_new |= 0x800,
                3 => ram[0x10002 + 2 * 97 + 1] = 0x80,
                4 => ram[0x16402 + 96] = 1,
                5 => ram[0x1c3b] = 0,
                6 => ram[0x1e1b] = 0,
                7 => next.x = 91,
                8 => next.y = 106,
                9 => ram[0xa1f] = 3,
                10 => route.door.approach_polls = 119,
                11 => route.door.air_step = None,
                12 => next.effective_buttons = 0x80,
                _ => unreachable!(),
            }
            route.observe(&next, &ram);
            assert!(route.failure().is_some());
            assert_eq!(route.door.approach_fall_step, None);
            assert_eq!(route.buttons(&next, &ram), 0);
        }
    }

    fn traversal_state() -> (State, Vec<u8>, Return) {
        let (state, mut ram, mut route) = approach_state();
        let state = State { x: 59, y: 110, effective_buttons: 0x40, joy1_current: 0x200, joy1_new: 0, ..state };
        route.previous = Some(state);
        route.climb_phase = 44;
        route.door.approach_fall_step = Some(state.host_steps);
        route.door.approach_polls = 18;
        ram[0x99c..0x99e].copy_from_slice(&0xe17du16.to_le_bytes());
        ram[0xa1c..0xa1e].copy_from_slice(&0x2au16.to_le_bytes());
        ram[0xa1f] = 6;
        ram[0xb36..0xb38].copy_from_slice(&2u16.to_le_bytes());
        (state, ram, route)
    }

    #[cfg_attr(test, test)]
    pub(super) fn traversal_records_consumed_landing_or_exact_native_door_event() {
        for radius in [19u16, 21] {
            let (state, mut ram, mut route) = traversal_state();
            ram[0xa1c..0xa1e].copy_from_slice(&0xa5u16.to_le_bytes());
            ram[0xa1f] = 0;
            ram[0xb00..0xb02].copy_from_slice(&radius.to_le_bytes());
            ram[0xb36..0xb38].fill(0);
            ram[0xafc..0xafe].copy_from_slice(&0xffffu16.to_le_bytes());
            route.previous = Some(State { y: 140, ..state });
            let pending = State { host_steps: state.host_steps + 1, x: 25, y: 139, ..state };
            route.observe(&pending, &ram);
            assert_eq!(route.door.traversal_landing_step, None);
            assert_eq!(route.failure(), None);
            let landed = State { host_steps: pending.host_steps + 1, polls: pending.polls + 1, ..pending };
            route.observe(&landed, &ram);
            assert_eq!(route.failure(), None);
            assert_eq!(route.climb_phase, 45);
            assert_eq!(route.buttons(&landed, &ram), 0x0200u16.reverse_bits());
            let mut report = sm_binary_evidence::Map::new();
            route.report(&mut report);
            assert_eq!(report["rio_return_complete"].as_bool(), Some(false));
            assert_eq!(
                report["rio_return_door_traversal_landing_step"].as_integer(),
                Some((state.host_steps + 2) as i64)
            );
            for (key, value) in [("x", 25), ("y", 139), ("pose", 0xa5), ("radius", i64::from(radius))] {
                assert_eq!(report[&format!("rio_return_door_traversal_landing_{key}")].as_integer(), Some(value));
            }
            assert_eq!(route.door.approach_polls, 18);
        }
        let (state, mut ram, mut route) = traversal_state();
        ram[0x78d..0x78f].copy_from_slice(&0x8ec2u16.to_le_bytes());
        ram[0x7b5..0x7b7].copy_from_slice(&0x9f57u16.to_le_bytes());
        ram[0xa1c..0xa1e].copy_from_slice(&0xffffu16.to_le_bytes());
        let transition = State { host_steps: state.host_steps + 1, polls: state.polls + 1, game_state: 9, ..state };
        route.observe(&transition, &ram);
        assert_eq!(route.failure(), None);
        assert_eq!(route.climb_phase, 46);
        let mut report = sm_binary_evidence::Map::new();
        route.report(&mut report);
        assert_eq!(report["rio_return_complete"].as_bool(), Some(false));
        assert_eq!(report["rio_return_door_transition_step"].as_integer(), Some((state.host_steps + 1) as i64));
        assert_eq!(report["rio_return_door_transition_definition"].as_integer(), Some(0x8ec2));
        assert_eq!(report["rio_return_door_transition_list"].as_integer(), Some(0x9f57));
        assert_eq!(report["rio_return_door_traversal_polls"].as_integer(), Some(1));
    }

    #[cfg_attr(test, test)]
    pub(super) fn traversal_refuses_foreign_transition_and_unproved_landing() {
        for invalid in 0..13 {
            let (state, mut ram, mut route) = traversal_state();
            ram[0x78d..0x78f].copy_from_slice(&0x8ec2u16.to_le_bytes());
            ram[0x7b5..0x7b7].copy_from_slice(&0x9f57u16.to_le_bytes());
            let mut next = State { host_steps: state.host_steps + 1, polls: state.polls + 1, game_state: 9, ..state };
            match invalid {
                0 => ram[0x78d] = 0xce,
                1 => ram[0x7b5] = 0,
                2 => next.effective_buttons = 0x80,
                3 => next.joy1_current |= 0x40,
                4 => next.joy1_new = 0x800,
                5 => next.polls = state.polls,
                6 => route.door.traversal_polls = 119,
                7 => route.door.approach_fall_step = None,
                8 => route.previous = Some(State { game_state: 9, ..state }),
                9 => route.previous = Some(State { room: 0x9e9f, ..state }),
                10 => route.previous = Some(State { area: 0, ..state }),
                11 => route.previous = Some(State { effective_buttons: 0x80, ..state }),
                12 => route.previous = Some(State { joy1_current: 0, ..state }),
                _ => unreachable!(),
            }
            route.observe(&next, &ram);
            assert!(route.failure().is_some());
            assert_eq!(route.door.transition_step, None);
        }
        for invalid in 0..7 {
            let (state, mut ram, mut route) = traversal_state();
            ram[0xa1c..0xa1e].copy_from_slice(&0xa5u16.to_le_bytes());
            ram[0xa1f] = 0;
            ram[0xb00..0xb02].copy_from_slice(&19u16.to_le_bytes());
            ram[0xb36..0xb38].fill(0);
            ram[0xafc..0xafe].copy_from_slice(&0xffffu16.to_le_bytes());
            let mut landed = State { host_steps: state.host_steps + 1, polls: state.polls + 1, x: 25, y: 139, ..state };
            match invalid {
                0 => landed.y = 141,
                1 => ram[0xafc] = 0,
                2 => ram[0x10002 + 2 * 161 + 1] = 0,
                3 => ram[0xb2e] = 1,
                4 => landed.joy1_current = 0,
                5 => landed.y = 0xffff,
                6 => landed.y = 140,
                _ => unreachable!(),
            }
            route.observe(&landed, &ram);
            assert!(route.failure().is_some());
            assert_eq!(route.door.traversal_landing_step, None);
        }
        let (state, ram, mut route) = traversal_state();
        route.previous = Some(State { y: 130, ..state });
        let upward = State { host_steps: state.host_steps + 1, polls: state.polls + 1, y: 129, ..state };
        route.observe(&upward, &ram);
        assert!(route.failure().is_some());
        assert_eq!(route.door.traversal_landing_step, None);
    }

    fn exit_state() -> (State, Vec<u8>, Return) {
        let (state, mut ram, mut route) = traversal_state();
        let state = State { x: 25, y: 139, ..state };
        route.previous = Some(state);
        route.climb_phase = 45;
        route.door.traversal_polls = 25;
        route.door.traversal_landing_step = Some(state.host_steps);
        route.door.traversal_landing = Some([25, 139, 0xa5, 19]);
        ram[0xa1c..0xa1e].copy_from_slice(&0x0au16.to_le_bytes());
        ram[0xa1f] = 1;
        ram[0xb00..0xb02].copy_from_slice(&21u16.to_le_bytes());
        ram[0xb36..0xb38].fill(0);
        ram[0xafc..0xafe].copy_from_slice(&0xffffu16.to_le_bytes());
        (state, ram, route)
    }

    #[cfg_attr(test, test)]
    pub(super) fn exit_walk_preserves_landing_until_consumed_native_door_event() {
        for (pose, movement, radius) in [(0xa5u16, 0u8, 19u16), (0xa5, 0, 21), (2, 0, 21), (0x0a, 1, 21)] {
            let (state, mut ram, mut route) = exit_state();
            ram[0xa1c..0xa1e].copy_from_slice(&pose.to_le_bytes());
            ram[0xa1f] = movement;
            ram[0xb00..0xb02].copy_from_slice(&radius.to_le_bytes());
            let pending = State { host_steps: state.host_steps + 1, x: 24, ..state };
            route.observe(&pending, &ram);
            assert_eq!(route.failure(), None);
            assert_eq!(route.door.exit_polls, 0);
            let walk = State { host_steps: pending.host_steps + 1, polls: pending.polls + 1, ..pending };
            route.observe(&walk, &ram);
            assert_eq!(route.failure(), None);
            assert_eq!(route.buttons(&walk, &ram), 0x0200u16.reverse_bits());
            ram[0x78d..0x78f].copy_from_slice(&0x8ec2u16.to_le_bytes());
            ram[0x7b5..0x7b7].copy_from_slice(&0x9f57u16.to_le_bytes());
            ram[0xa1c..0xa1e].copy_from_slice(&0xffffu16.to_le_bytes());
            let event = State { host_steps: walk.host_steps + 1, polls: walk.polls + 1, game_state: 9, ..walk };
            route.observe(&event, &ram);
            assert_eq!(route.failure(), None);
            assert_eq!(route.climb_phase, 46);
            assert_eq!(route.door.transition_step, Some(event.host_steps));
            assert_eq!(route.door.exit_polls, 2);
            assert_eq!(route.door.traversal_polls, 25);
            assert_eq!(route.door.traversal_landing, Some([25, 139, 0xa5, 19]));
            assert_eq!(route.door.traversal_landing_step, Some(state.host_steps));
            let mut report = sm_binary_evidence::Map::new();
            route.report(&mut report);
            assert_eq!(report["rio_return_door_exit_polls"].as_integer(), Some(2));
            assert_eq!(report["rio_return_complete"].as_bool(), Some(false));
        }
    }

    #[cfg_attr(test, test)]
    pub(super) fn exit_walk_refuses_lost_landing_domain_and_unowned_event() {
        for invalid in 0..16 {
            let (state, mut ram, mut route) = exit_state();
            let mut next = State { host_steps: state.host_steps + 1, polls: state.polls + 1, x: 24, ..state };
            match invalid {
                0 => route.door.traversal_landing_step = None,
                1 => next.x = 26,
                2 => next.x = 4,
                3 => next.y = 140,
                4 => ram[0xafc] = 0,
                5 => ram[0xb00] = 19,
                6 => ram[0xa1f] = 6,
                7 => next.effective_buttons = 0x80,
                8 => next.joy1_current = 0,
                9 => next.joy1_new = 0x800,
                10 => ram[0x10002 + 2 * 97 + 1] = 0x80,
                11 => ram[0x1c3b] = 0,
                12 => ram[0x10002 + 2 * 160 + 1] = 0,
                13 => ram[0xb42] = 1,
                14 => route.door.exit_polls = 119,
                15 => route.previous = Some(State { x: 23, ..state }),
                _ => unreachable!(),
            }
            route.observe(&next, &ram);
            assert!(route.failure().is_some());
            assert_eq!(route.door.transition_step, None);
        }
        for invalid in 0..8 {
            let (state, mut ram, mut route) = exit_state();
            ram[0x78d..0x78f].copy_from_slice(&0x8ec2u16.to_le_bytes());
            ram[0x7b5..0x7b7].copy_from_slice(&0x9f57u16.to_le_bytes());
            let mut next = State { host_steps: state.host_steps + 1, polls: state.polls + 1, game_state: 9, ..state };
            match invalid {
                0 => route.door.traversal_landing_step = None,
                1 => route.door.exit_polls = 119,
                2 => route.previous = Some(State { room: 0x9e9f, ..state }),
                3 => route.previous = Some(State { effective_buttons: 0x80, ..state }),
                4 => next.polls = state.polls,
                5 => ram[0x78d] = 0xce,
                6 => ram[0x7b5] = 0,
                7 => next.joy1_current = 0,
                _ => unreachable!(),
            }
            route.observe(&next, &ram);
            assert!(route.failure().is_some());
            assert_eq!(route.door.transition_step, None);
        }
    }

    fn loading_state() -> (State, Vec<u8>, Return) {
        let (state, mut ram, mut route) = exit_state();
        ram[0x78d..0x78f].copy_from_slice(&0x8ec2u16.to_le_bytes());
        ram[0x7b5..0x7b7].copy_from_slice(&0x9f57u16.to_le_bytes());
        let event = State { host_steps: state.host_steps + 1, polls: state.polls + 1, game_state: 9, ..state };
        route.observe(&event, &ram);
        assert_eq!(route.failure(), None);
        assert_eq!(route.climb_phase, 46);
        (event, ram, route)
    }

    fn loaded_destination(ram: &mut [u8]) {
        for (at, value) in [(0x7b5, 0x9ee5u16), (0x7a5, 128), (0x7a7, 48), (0x791, 5), (0x927, 0x700), (0x929, 0x200)] {
            ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
        }
    }

    #[cfg_attr(test, test)]
    pub(super) fn loading_follows_native_order_and_records_consumed_destination_arrival() {
        let (event, mut ram, mut route) = loading_state();
        assert_eq!(route.buttons(&event, &ram), 0);
        let mut state = event;
        // E17D calls E1B7 synchronously, so this fixture deliberately skips state10.
        for (index, function) in
            [0xe29eu16, 0xe2db, 0xe2f7, 0xe310, 0xe353, 0xe36e, 0xe38e, 0xe3c0, 0xe4a9, 0xe659, 0xe664, 0xe6a2, 0xe737]
                .into_iter()
                .enumerate()
        {
            ram[0x99c..0x99e].copy_from_slice(&function.to_le_bytes());
            if index >= 3 {
                ram[0x791..0x793].copy_from_slice(&5u16.to_le_bytes());
                ram[0x927..0x929].copy_from_slice(&0x700u16.to_le_bytes());
                ram[0x929..0x92b].copy_from_slice(&0x200u16.to_le_bytes());
            }
            if index >= 6 {
                loaded_destination(&mut ram);
            }
            state = State {
                host_steps: state.host_steps + 1,
                polls: state.polls + 1,
                game_state: 11,
                room: if index >= 3 { 0x9e9f } else { 0x9f11 },
                effective_buttons: 0,
                joy1_current: 0,
                joy1_new: 0,
                ..state
            };
            route.observe(&state, &ram);
            assert_eq!(route.failure(), None);
            assert_eq!(route.buttons(&state, &ram), 0);
        }
        let pending = State { host_steps: state.host_steps + 1, game_state: 8, x: 2020, y: 500, ..state };
        // An unconsumed destination observation cannot claim arrival.
        let mut pending_route = route;
        pending_route.observe(&pending, &ram);
        assert!(pending_route.failure().is_some());
        assert_eq!(pending_route.door.arrival_step, None);
        // Rebuild the same pre-arrival state to test consumed arrival independently.
        let (event, mut ram, mut route) = loading_state();
        loaded_destination(&mut ram);
        ram[0x99c..0x99e].copy_from_slice(&0xe737u16.to_le_bytes());
        let fading = State {
            host_steps: event.host_steps + 1,
            polls: event.polls + 1,
            game_state: 11,
            room: 0x9e9f,
            effective_buttons: 0,
            joy1_current: 0,
            joy1_new: 0,
            ..event
        };
        route.observe(&fading, &ram);
        assert_eq!(route.failure(), None);
        ram[0xa1c..0xa1e].copy_from_slice(&0x02u16.to_le_bytes());
        ram[0xb00..0xb02].copy_from_slice(&21u16.to_le_bytes());
        let arrived = State {
            host_steps: fading.host_steps + 1,
            polls: fading.polls + 1,
            game_state: 8,
            x: 2020,
            y: 500,
            ..fading
        };
        route.observe(&arrived, &ram);
        assert_eq!(route.failure(), None);
        assert_eq!(route.climb_phase, 47);
        assert_eq!(route.door.arrival_step, Some(arrived.host_steps));
        assert_eq!(route.door.exit_polls, 1);
        assert_eq!(route.door.traversal_polls, 25);
        assert_eq!(route.door.traversal_landing, Some([25, 139, 0xa5, 19]));
        assert_eq!(route.door.loading_polls, 2);
        let mut report = sm_binary_evidence::Map::new();
        route.report(&mut report);
        assert_eq!(report["rio_return_arrival_x"].as_integer(), Some(2020));
        assert_eq!(report["rio_return_arrival_y"].as_integer(), Some(500));
        assert_eq!(report["rio_return_arrival_pose"].as_integer(), Some(2));
        assert_eq!(report["rio_return_arrival_radius"].as_integer(), Some(21));
        assert_eq!(report["rio_return_arrival_step"].as_integer(), Some(arrived.host_steps as i64));
        assert_eq!(report["rio_return_loading_polls"].as_integer(), Some(2));
        assert_eq!(report["rio_return_loading_stage"].as_integer(), Some(13));
        assert_eq!(report["rio_return_loading_stages"].as_integer(), Some(1 | (1 << 13)));
        assert_eq!(report["rio_return_complete"].as_bool(), Some(false));
    }

    #[cfg_attr(test, test)]
    pub(super) fn loading_refuses_foreign_headers_input_order_and_unproved_arrival() {
        for invalid in 0..17 {
            let (event, mut ram, mut route) = loading_state();
            loaded_destination(&mut ram);
            ram[0x99c..0x99e].copy_from_slice(&0xe737u16.to_le_bytes());
            let fading = State {
                host_steps: event.host_steps + 1,
                polls: event.polls + 1,
                game_state: 11,
                room: 0x9e9f,
                effective_buttons: 0,
                joy1_current: 0,
                joy1_new: 0,
                ..event
            };
            route.observe(&fading, &ram);
            assert_eq!(route.failure(), None);
            let mut next =
                State { host_steps: fading.host_steps + 1, polls: fading.polls + 1, game_state: 8, ..fading };
            match invalid {
                0 => ram[0x78d] = 0xce,
                1 => ram[0x7b5] = 0,
                2 => ram[0x7a5] = 16,
                3 => ram[0x7a7] = 32,
                4 => ram[0x791] = 4,
                5 => ram[0x927] = 1,
                6 => ram[0x929] = 1,
                7 => ram[0x795] = 1,
                8 => ram[0x797] = 1,
                9 => next.effective_buttons = 0x40,
                10 => next.joy1_current = 0x200,
                11 => next.joy1_new = 0x200,
                12 => next.room = 0x9f64,
                13 => next.area = 0,
                14 => route.door.transition_step = None,
                15 => route.door.transition_step = Some(next.host_steps - 240),
                16 => route.door.loading_polls = 239,
                _ => unreachable!(),
            }
            route.observe(&next, &ram);
            assert!(route.failure().is_some());
            assert_eq!(route.door.arrival_step, None);
            if invalid >= 15 {
                assert_eq!(route.failure(), Some("return loading lost its neutral owned destination domain"));
            }
        }
        let (event, mut ram, mut route) = loading_state();
        ram[0x99c..0x99e].copy_from_slice(&0xe2dbu16.to_le_bytes());
        let next = State {
            host_steps: event.host_steps + 1,
            polls: event.polls + 1,
            game_state: 11,
            effective_buttons: 0,
            joy1_current: 0,
            joy1_new: 0,
            ..event
        };
        route.observe(&next, &ram);
        assert_eq!(route.failure(), None);
        ram[0x99c..0x99e].copy_from_slice(&0xe29eu16.to_le_bytes());
        route.observe(&State { host_steps: next.host_steps + 1, polls: next.polls + 1, ..next }, &ram);
        assert_eq!(route.failure(), Some("return loading lost native function order"));
    }

    fn departure_state() -> (State, Vec<u8>, Return) {
        let (event, mut ram, mut route) = loading_state();
        loaded_destination(&mut ram);
        ram[0x99c..0x99e].copy_from_slice(&0xe737u16.to_le_bytes());
        let fading = State {
            host_steps: event.host_steps + 1,
            polls: event.polls + 1,
            game_state: 11,
            room: 0x9e9f,
            effective_buttons: 0,
            joy1_current: 0,
            joy1_new: 0,
            ..event
        };
        route.observe(&fading, &ram);
        let arrived = State {
            host_steps: fading.host_steps + 1,
            polls: fading.polls + 1,
            game_state: 8,
            x: 2008,
            y: 651,
            ..fading
        };
        route.observe(&arrived, &ram);
        assert_eq!(route.failure(), None);
        for (at, value) in [
            (0x7bb, 0x9eb1u16),
            (0x7cf, 0x93ac),
            (0xa6c, 0x9f55),
            (0xa1c, 0x0a),
            (0xb00, 21),
            (0xf78 + 128, 0xd73f),
            (0xf7a + 128, 1408),
            (0xf7e + 128, 706),
            (0xfb4 + 128, 2),
            (0xfb6 + 128, 24),
        ] {
            ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
        }
        ram[0xa1f] = 1;
        for (col, value) in
            [(122, 0x81eau16), (123, 0x81ea), (124, 0x81e9), (125, 0x81e9), (126, 0x81e9), (127, 0x81ec)]
        {
            let at = 0x10002 + 2 * (42 * 128 + col);
            ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
        }
        for row in 39..=44 {
            for col in 116..=121 {
                let at = 0x10002 + 2 * (row * 128 + col);
                ram[at..at + 2].copy_from_slice(&0xffu16.to_le_bytes());
            }
        }
        (arrived, ram, route)
    }

    #[cfg_attr(test, test)]
    pub(super) fn departure_records_first_consumed_fall_and_preserves_arrival() {
        for radius in [19u16, 21] {
            let (arrival, mut ram, mut route) = departure_state();
            assert_eq!(route.buttons(&arrival, &ram), 0x0200u16.reverse_bits());
            let walking = State {
                host_steps: arrival.host_steps + 1,
                polls: arrival.polls + 1,
                x: 1949,
                effective_buttons: 0x40,
                joy1_current: 0x200,
                joy1_new: 0x200,
                ..arrival
            };
            route.observe(&walking, &ram);
            assert_eq!(route.failure(), None);
            for (at, value) in [
                (0xa1c, 0x2au16),
                (0xb00, radius),
                (0xb36, 2),
                (0xaf8, 0x7000),
                (0xafc, 0x5678),
                (0xb46, 3),
                (0xb48, 0x8000),
                (0xb4a, 1),
            ] {
                ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
            }
            ram[0xa1f] = 6;
            let pending = State { host_steps: walking.host_steps + 1, x: 1947, y: 654, joy1_new: 0, ..walking };
            route.observe(&pending, &ram);
            assert_eq!(route.failure(), None);
            assert_eq!(route.door.departure_fall_step, None);
            let fallen = State { host_steps: pending.host_steps + 1, polls: pending.polls + 1, ..pending };
            route.observe(&fallen, &ram);
            assert_eq!(route.failure(), None);
            assert_eq!(route.climb_phase, 48);
            assert_eq!(route.door.departure_polls, 2);
            assert_eq!(route.door.arrival, Some([2008, 651, 0x0a, 21]));
            assert_eq!(route.door.loading_polls, 2);
            let mut report = sm_binary_evidence::Map::new();
            route.report(&mut report);
            for (key, value) in [
                ("x", 1947),
                ("y", 654),
                ("pose", 0x2a),
                ("radius", i64::from(radius)),
                ("x_subposition", 0x7000),
                ("y_subposition", 0x5678),
                ("x_speed", 3),
                ("x_subspeed", 0x8000),
                ("x_accel_mode", 1),
                ("y_speed", 0),
                ("y_subspeed", 0),
                ("y_direction", 2),
            ] {
                assert_eq!(report[&format!("rio_return_departure_fall_{key}")].as_integer(), Some(value));
            }
            assert_eq!(report["rio_return_departure_fall_step"].as_integer(), Some(fallen.host_steps as i64));
            assert_eq!(report["rio_return_departure_polls"].as_integer(), Some(2));
            assert_eq!(report["rio_return_complete"].as_bool(), Some(false));
        }
    }

    #[cfg_attr(test, test)]
    pub(super) fn departure_refuses_foreign_actor_terrain_and_unowned_input() {
        for invalid in 0..18 {
            let (arrival, mut ram, mut route) = departure_state();
            let mut next = State {
                host_steps: arrival.host_steps + 1,
                polls: arrival.polls + 1,
                x: 2007,
                effective_buttons: 0x40,
                joy1_current: 0x200,
                joy1_new: 0,
                ..arrival
            };
            match invalid {
                0 => ram[0x7bb] = 0xcb,
                1 => ram[0x7cf] = 0x26,
                2 => ram[0xf78 + 128] = 0,
                3 => ram[0xfb4 + 128] = 1,
                4 => ram[0xfb6 + 128] = 0,
                5 => ram[0xe16] = 1,
                6 => ram[0xe18] = 1,
                7 => ram[0x10002 + 2 * (42 * 128 + 122) + 1] = 0,
                8 => ram[0x10002 + 2 * (41 * 128 + 121) + 1] = 0x80,
                9 => next.effective_buttons = 0x80,
                10 => next.joy1_current = 0,
                11 => next.joy1_new = 0x800,
                12 => next.x = 2009,
                13 => next.y = 650,
                14 => route.door.departure_polls = 119,
                15 => route.door.arrival_step = None,
                16 => ram[0xb42] = 1,
                17 => ram[0xa6c] = 0,
                _ => unreachable!(),
            }
            route.observe(&next, &ram);
            assert_eq!(route.failure(), Some("return elevator approach lost its first shelf departure domain"));
            assert_eq!(route.door.departure_fall_step, None);
        }
    }

    fn lower_floor_state() -> (State, Vec<u8>, Return) {
        let (arrival, mut ram, mut route) = departure_state();
        ram[0xa1c..0xa1e].copy_from_slice(&0x2au16.to_le_bytes());
        ram[0xa1f] = 6;
        ram[0xb36..0xb38].copy_from_slice(&2u16.to_le_bytes());
        let fallen = State {
            host_steps: arrival.host_steps + 1,
            polls: arrival.polls + 1,
            x: 1947,
            y: 655,
            effective_buttons: 0x40,
            joy1_current: 0x200,
            joy1_new: 0x200,
            ..arrival
        };
        route.observe(&fallen, &ram);
        assert_eq!(route.failure(), None);
        assert_eq!(route.climb_phase, 48);
        for (col, value) in
            [(116, 0x81e9u16), (117, 0x81e9), (118, 0x81e9), (119, 0x81ea), (120, 0x81ea), (121, 0x81ea)]
        {
            let at = 0x10002 + 2 * (45 * 128 + col);
            ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
        }
        (fallen, ram, route)
    }

    #[cfg_attr(test, test)]
    pub(super) fn lower_floor_latches_consumed_landing_with_native_pose_adjustment() {
        for radius in [19u16, 21] {
            let (fallen, mut ram, mut route) = lower_floor_state();
            assert_eq!(route.buttons(&fallen, &ram), 0x40);
            let airborne = State {
                host_steps: fallen.host_steps + 1,
                polls: fallen.polls + 1,
                x: 1880,
                y: 701,
                joy1_new: 0,
                ..fallen
            };
            route.observe(&airborne, &ram);
            assert_eq!(route.failure(), None);
            for (at, value) in [
                (0xa1c, 0xa5u16),
                (0xb00, radius),
                (0xaf8, 0x3456),
                (0xafc, 0xffff),
                (0xb46, 0),
                (0xb48, 0),
                (0xb4a, 0),
                (0xb2c, 0),
                (0xb2e, 0),
                (0xb36, 0),
            ] {
                ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
            }
            ram[0xa1f] = 0;
            let pending = State { host_steps: airborne.host_steps + 1, x: 1878, y: 699, ..airborne };
            route.observe(&pending, &ram);
            assert_eq!(route.failure(), None);
            assert_eq!(route.door.lower_floor_step, None);
            let landed = State { host_steps: pending.host_steps + 1, polls: pending.polls + 1, ..pending };
            route.observe(&landed, &ram);
            assert_eq!(route.failure(), None);
            assert_eq!(route.climb_phase, 49);
            assert_eq!(route.door.departure_fall, Some([1947, 655, 0x2a, 21]));
            assert_eq!(route.door.departure_polls, 1);
            let mut report = sm_binary_evidence::Map::new();
            route.report(&mut report);
            for (key, value) in [
                ("polls", 2),
                ("step", landed.host_steps as i64),
                ("x", 1878),
                ("y", 699),
                ("pose", 0xa5),
                ("radius", i64::from(radius)),
                ("x_subposition", 0x3456),
                ("y_subposition", 0xffff),
                ("x_speed", 0),
                ("x_subspeed", 0),
                ("x_accel_mode", 0),
                ("y_speed", 0),
                ("y_subspeed", 0),
                ("y_direction", 0),
            ] {
                assert_eq!(report[&format!("rio_return_lower_floor_{key}")].as_integer(), Some(value));
            }
        }
    }

    #[cfg_attr(test, test)]
    pub(super) fn lower_floor_refuses_wrong_landing_terrain_motion_and_ownership() {
        for invalid in 0..12 {
            let (fallen, mut ram, mut route) = lower_floor_state();
            let mut next = State {
                host_steps: fallen.host_steps + 1,
                polls: fallen.polls + 1,
                x: 1900,
                y: 699,
                joy1_new: 0,
                ..fallen
            };
            ram[0xa1c..0xa1e].copy_from_slice(&0xa5u16.to_le_bytes());
            ram[0xa1f] = 0;
            ram[0xafc..0xafe].copy_from_slice(&0xffffu16.to_le_bytes());
            ram[0xb36..0xb38].copy_from_slice(&0u16.to_le_bytes());
            match invalid {
                0 => next.y = 701,
                1 => next.y = 700,
                2 => next.y = u16::MAX,
                3 => ram[0x10002 + 2 * (45 * 128 + 118) + 1] = 0,
                4 => ram[0xb2c] = 1,
                5 => ram[0xb46] = 1,
                6 => route.door.departure_fall_step = None,
                7 => route.door.lower_floor_polls = 119,
                8 => next.effective_buttons = 0x80,
                9 => next.joy1_current = 0,
                10 => {
                    ram[0xa1c] = 0x2a;
                    ram[0xa1f] = 6;
                    ram[0xb36] = 2;
                    next.y = 654;
                }
                11 => ram[0xff8] = 0,
                _ => unreachable!(),
            }
            route.observe(&next, &ram);
            assert_eq!(route.failure(), Some("return elevator approach lost its lower-floor landing domain"));
            assert_eq!(route.door.lower_floor_step, None);
        }
    }

    fn platform_state() -> (State, Vec<u8>, Return) {
        let (fallen, mut ram, mut route) = lower_floor_state();
        for (at, value) in [(0xa1c, 0xa5u16), (0xb00, 19), (0xafc, 0xffff), (0xb36, 0)] {
            ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
        }
        ram[0xa1f] = 0;
        let landed = State {
            host_steps: fallen.host_steps + 1,
            polls: fallen.polls + 1,
            x: 1905,
            y: 699,
            joy1_new: 0,
            ..fallen
        };
        route.observe(&landed, &ram);
        assert_eq!(route.failure(), None);
        assert_eq!(route.climb_phase, 49);
        for row in 41..=44 {
            for col in (if row == 44 { 92 } else { 85 })..=121 {
                let at = 0x10002 + 2 * (row * 128 + col);
                ram[at..at + 2].copy_from_slice(&0x00ffu16.to_le_bytes());
                ram[0x16402 + row * 128 + col] = 0;
            }
        }
        for col in 92..=121 {
            let at = 0x10002 + 2 * (45 * 128 + col);
            ram[at..at + 2].copy_from_slice(&0x81e9u16.to_le_bytes());
            ram[0x16402 + 45 * 128 + col] = 0;
        }
        for col in 85..=91 {
            let special = matches!(col, 87 | 88);
            let at = 0x10002 + 2 * (44 * 128 + col);
            ram[at..at + 2].copy_from_slice(&(if special { 0x90ffu16 } else { 0x81bf }).to_le_bytes());
            ram[0x16402 + 44 * 128 + col] = if special { 3 } else { 0 };
        }
        (landed, ram, route)
    }

    #[cfg_attr(test, test)]
    pub(super) fn platform_records_first_consumed_native_wall_and_preserves_landing() {
        let (landed, mut ram, mut route) = platform_state();
        assert_eq!(route.buttons(&landed, &ram), 0x0200u16.reverse_bits());
        ram[0xa1c..0xa1e].copy_from_slice(&0x0au16.to_le_bytes());
        ram[0xa1f] = 1;
        ram[0xb00..0xb02].copy_from_slice(&21u16.to_le_bytes());
        let walking = State { host_steps: landed.host_steps + 1, polls: landed.polls + 1, x: 1480, ..landed };
        route.observe(&walking, &ram);
        assert_eq!(route.failure(), None);
        for (at, value) in [(0xa1c, 0x8au16), (0xdd0, 1), (0xaf8, 0), (0xb46, 0), (0xb48, 0), (0xb4a, 0)] {
            ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
        }
        ram[0xa1f] = 0x15;
        let pending = State { host_steps: walking.host_steps + 1, x: 1477, ..walking };
        route.observe(&pending, &ram);
        assert_eq!(route.failure(), None);
        assert_eq!(route.door.platform_step, None);
        let contact = State { host_steps: pending.host_steps + 1, polls: pending.polls + 1, ..pending };
        route.observe(&contact, &ram);
        assert_eq!(route.failure(), None);
        assert_eq!(route.climb_phase, 53);
        assert_eq!(route.door.lower_floor_step, Some(landed.host_steps));
        assert_eq!(route.door.lower_floor_polls, 1);
        let mut report = sm_binary_evidence::Map::new();
        route.report(&mut report);
        for (key, value) in [
            ("polls", 2),
            ("step", contact.host_steps as i64),
            ("collision", 1),
            ("x", 1477),
            ("y", 699),
            ("pose", 0x8a),
            ("radius", 21),
            ("x_subposition", 0),
            ("y_subposition", 0xffff),
            ("x_speed", 0),
            ("x_subspeed", 0),
            ("x_accel_mode", 0),
            ("y_speed", 0),
            ("y_subspeed", 0),
            ("y_direction", 0),
        ] {
            assert_eq!(report[&format!("rio_return_platform_{key}")].as_integer(), Some(value));
        }
        assert_eq!(report["rio_return_complete"].as_bool(), Some(false));
        // Distinct snapshot values check report ordering separately from the
        // native contact oracle, which requires cleared motion.
        route.door.platform_motion = Some([0x1234, 0x5678, 3, 0x8000, 2, 4, 0x9000, 1]);
        route.report(&mut report);
        for (key, value) in [
            ("x_subposition", 0x1234),
            ("y_subposition", 0x5678),
            ("x_speed", 3),
            ("x_subspeed", 0x8000),
            ("x_accel_mode", 2),
            ("y_speed", 4),
            ("y_subspeed", 0x9000),
            ("y_direction", 1),
        ] {
            assert_eq!(report[&format!("rio_return_platform_{key}")].as_integer(), Some(value));
        }
    }

    #[cfg_attr(test, test)]
    pub(super) fn platform_refuses_foreign_corridor_input_actor_and_contact() {
        for invalid in 0..20 {
            let (landed, mut ram, mut route) = platform_state();
            ram[0xa1c..0xa1e].copy_from_slice(&0x0au16.to_le_bytes());
            ram[0xa1f] = 1;
            ram[0xb00..0xb02].copy_from_slice(&21u16.to_le_bytes());
            let mut next = State { host_steps: landed.host_steps + 1, polls: landed.polls + 1, x: 1904, ..landed };
            match invalid {
                0 => route.door.lower_floor_step = None,
                1 => route.door.platform_polls = 239,
                2 => ram[0xff8] = 0,
                3 => ram[0xe18] = 1,
                4 => ram[0x7bb] = 0xcb,
                5 => ram[0x10002 + 2 * (45 * 128 + 100) + 1] = 0,
                6 => ram[0x16402 + 45 * 128 + 100] = 1,
                7 => ram[0x10002 + 2 * (43 * 128 + 90) + 1] = 0x80,
                8 => ram[0x10002 + 2 * (44 * 128 + 91) + 1] = 0,
                9 => ram[0x16402 + 44 * 128 + 87] = 0,
                10 => next.effective_buttons = 0x80,
                11 => next.joy1_new = 0x800,
                12 => next.y = 698,
                13 => next.x = 1476,
                14 => ram[0xb2c] = 1,
                15 => {
                    ram[0xa1c] = 0x8a;
                    ram[0xa1f] = 0x15;
                    ram[0xdd0] = 1;
                }
                16..=19 => {
                    next.x = 1477;
                    ram[0xa1c] = 0x8a;
                    ram[0xa1f] = 0x15;
                    ram[0xdd0] = 1;
                    ram[[0xaf8, 0xb46, 0xb48, 0xb4a][invalid - 16]] = 1;
                }
                _ => unreachable!(),
            }
            route.observe(&next, &ram);
            assert_eq!(route.failure(), Some("return elevator approach lost its raised-platform corridor"));
            assert_eq!(route.door.platform_step, None);
        }
    }

    fn boarding_state() -> (State, Vec<u8>, Return) {
        let (landed, mut ram, mut route) = platform_state();
        for (at, value) in [
            (0xa1c, 0x8au16),
            (0xb00, 21),
            (0xdd0, 1),
            (0xaf8, 0),
            (0x9a2, 4),
            (0x9b4, 0x80),
            (0x195e, 0xffff),
            (0x1962, 0xffff),
            (0x1968, 0),
            (0xb32, 0x1c00),
            (0xb34, 0),
        ] {
            ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
        }
        ram[0xa1f] = 0x15;
        let contact = State { host_steps: landed.host_steps + 1, polls: landed.polls + 1, x: 1477, ..landed };
        route.observe(&contact, &ram);
        assert_eq!(route.failure(), None);
        assert_eq!(route.climb_phase, 53);
        assert_eq!(route.buttons(&contact, &ram), 0x10);
        ram[0xa1c] = 4;
        ram[0xa1f] = 0;
        let aim = State {
            host_steps: contact.host_steps + 1,
            polls: contact.polls + 1,
            effective_buttons: 0x10,
            joy1_current: 0x800,
            joy1_new: 0x800,
            ..contact
        };
        route.observe(&aim, &ram);
        assert_eq!(route.failure(), None);
        assert_eq!(route.climb_phase, 50);
        (aim, ram, route)
    }

    #[cfg_attr(test, test)]
    pub(super) fn boarding_requires_consumed_up_aim_before_neutral_and_jump() {
        let (landed, mut ram, mut route) = platform_state();
        for (at, value) in [
            (0xa1c, 0x8au16),
            (0xb00, 21),
            (0xdd0, 1),
            (0x9a2, 4),
            (0x9b4, 0x80),
            (0x195e, 0xffff),
            (0x1962, 0xffff),
            (0xb32, 0x1c00),
        ] {
            ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
        }
        ram[0xa1f] = 0x15;
        let contact = State { host_steps: landed.host_steps + 1, polls: landed.polls + 1, x: 1477, ..landed };
        route.observe(&contact, &ram);
        assert_eq!(route.failure(), None);
        assert_eq!(route.buttons(&contact, &ram), 0x800u16.reverse_bits());
        ram[0xa1c] = 4;
        ram[0xa1f] = 0;
        let pending = State {
            host_steps: contact.host_steps + 1,
            effective_buttons: 0x10,
            joy1_current: 0x800,
            joy1_new: 0x800,
            ..contact
        };
        route.observe(&pending, &ram);
        assert_eq!(route.failure(), None);
        assert_eq!(route.door.boarding_aim_step, None);
        assert_eq!(route.buttons(&pending, &ram), 0x10);
        let aimed = State { host_steps: pending.host_steps + 1, polls: pending.polls + 1, ..pending };
        route.observe(&aimed, &ram);
        assert_eq!(route.failure(), None);
        assert_eq!(route.buttons(&aimed, &ram), 0);
        assert_eq!(route.door.boarding_aim_step, Some(aimed.host_steps));
        assert_eq!(route.door.platform_step, Some(contact.host_steps));
        assert_eq!(route.door.platform_polls, 1);
        let mut report = sm_binary_evidence::Map::new();
        route.report(&mut report);
        assert_eq!(report["rio_return_boarding_aim_step"].as_integer(), Some(aimed.host_steps as i64));
    }

    #[cfg_attr(test, test)]
    pub(super) fn boarding_refuses_unowned_aim_and_missing_aim_checkpoint() {
        for invalid in 0..5 {
            let (aimed, mut ram, mut route) = boarding_state();
            let mut next = State {
                host_steps: aimed.host_steps + 1,
                polls: aimed.polls + 1,
                effective_buttons: 0,
                joy1_current: 0,
                joy1_new: 0,
                ..aimed
            };
            ram[0xa1c] = 2;
            ram[0xa1f] = 0;
            match invalid {
                0 => route.door.boarding_aim_step = None,
                1 => {
                    route.climb_phase = 53;
                    ram[0xa1c] = 4;
                    next.effective_buttons = 0x40;
                    next.joy1_current = 0x200;
                }
                2 => {
                    route.climb_phase = 53;
                    ram[0xa1c] = 4;
                    next.effective_buttons = 0x10;
                    next.joy1_current = 0x800;
                    next.joy1_new = 0x80;
                }
                3 => {
                    route.climb_phase = 53;
                    ram[0xa1c] = 4;
                    next.effective_buttons = 0x10;
                    next.joy1_current = 0x800;
                    ram[0xb46] = 1;
                }
                4 => {
                    ram[0xa1c] = 0x8a;
                    ram[0xa1f] = 0x15;
                }
                _ => unreachable!(),
            }
            let before = route.door.boarding_aim_step;
            route.observe(&next, &ram);
            assert_eq!(route.failure(), Some("return elevator approach lost its raised-platform corridor"));
            assert_eq!(route.door.boarding_aim_step, before);
        }
    }

    #[cfg_attr(test, test)]
    pub(super) fn boarding_releases_then_fresh_jumps_and_latches_only_consumed_feet_clearance() {
        let (contact, mut ram, mut route) = boarding_state();
        assert_eq!(route.buttons(&contact, &ram), 0);
        ram[0xa1c] = 2;
        ram[0xa1f] = 0;
        ram[0xdd0] = 0;
        let pending =
            State { host_steps: contact.host_steps + 1, effective_buttons: 0, joy1_current: 0, joy1_new: 0, ..contact };
        route.observe(&pending, &ram);
        assert_eq!(route.failure(), None);
        assert_eq!(route.door.boarding_release_step, None);
        let release = State { host_steps: pending.host_steps + 1, polls: pending.polls + 1, ..pending };
        route.observe(&release, &ram);
        assert_eq!(route.failure(), None);
        assert_eq!(route.buttons(&release, &ram), 0x80u16.reverse_bits());
        ram[0xa1c] = 0x4c;
        ram[0xa1f] = 2;
        ram[0xb36] = 1;
        ram[0xb2e..0xb30].copy_from_slice(&4u16.to_le_bytes());
        ram[0xb2c..0xb2e].copy_from_slice(&0xe000u16.to_le_bytes());
        let jumped = State {
            host_steps: release.host_steps + 1,
            polls: release.polls + 1,
            y: 699,
            effective_buttons: 0x100,
            joy1_current: 0x80,
            joy1_new: 0x80,
            ..release
        };
        route.observe(&jumped, &ram);
        assert_eq!(route.failure(), None);
        assert_eq!(route.door.boarding_jump_step, Some(jumped.host_steps));
        let pending_jump = State { host_steps: jumped.host_steps + 1, ..jumped };
        route.observe(&pending_jump, &ram);
        assert_eq!(route.failure(), None);
        ram[0xb00] = 19;
        let transition = State {
            host_steps: pending_jump.host_steps + 1,
            polls: pending_jump.polls + 1,
            joy1_new: 0,
            ..pending_jump
        };
        route.observe(&transition, &ram);
        assert_eq!(route.failure(), None);
        let waiting = State { host_steps: transition.host_steps + 1, y: 685, ..transition };
        route.observe(&waiting, &ram);
        assert_eq!(route.failure(), None);
        assert_eq!(route.door.boarding_clear_step, None);
        ram[0xa1c] = 0x4e;
        ram[0xb2e] = 3;
        ram[0xb2c..0xb2e].copy_from_slice(&0x9000u16.to_le_bytes());
        let clear = State { host_steps: waiting.host_steps + 1, polls: waiting.polls + 1, ..waiting };
        route.observe(&clear, &ram);
        assert_eq!(route.failure(), None);
        assert_eq!(route.climb_phase, 54);
        assert_eq!(route.door.platform_step, Some(contact.host_steps - 1));
        assert_eq!(route.door.platform_polls, 1);
        let mut report = sm_binary_evidence::Map::new();
        route.report(&mut report);
        for (key, value) in [
            ("polls", 5),
            ("release_step", release.host_steps as i64),
            ("jump_step", jumped.host_steps as i64),
            ("jump_y", 699),
            ("jump_pose", 0x4c),
            ("jump_y_speed", 4),
            ("jump_y_subspeed", 0xe000),
            ("jump_y_direction", 1),
            ("clear_step", clear.host_steps as i64),
            ("jump_radius", 21),
            ("jump_y_subposition", 0xffff),
            ("clear_y", 685),
            ("clear_pose", 0x4e),
            ("clear_radius", 19),
            ("clear_y_subposition", 0xffff),
            ("clear_y_speed", 3),
            ("clear_y_subspeed", 0x9000),
            ("clear_y_direction", 1),
        ] {
            assert_eq!(report[&format!("rio_return_boarding_{key}")].as_integer(), Some(value));
        }
    }

    #[cfg_attr(test, test)]
    pub(super) fn boarding_refuses_foreign_jump_context_horizontal_motion_and_stale_jump() {
        for invalid in 0..18 {
            let (contact, mut ram, mut route) = boarding_state();
            ram[0xa1c] = 2;
            ram[0xa1f] = 0;
            ram[0xdd0] = 0;
            let release = State {
                host_steps: contact.host_steps + 1,
                polls: contact.polls + 1,
                effective_buttons: 0,
                joy1_current: 0,
                joy1_new: 0,
                ..contact
            };
            route.observe(&release, &ram);
            assert_eq!(route.failure(), None);
            ram[0xa1c] = 0x4c;
            ram[0xa1f] = 2;
            ram[0xb36] = 1;
            ram[0xb2e] = 4;
            ram[0xb2c..0xb2e].copy_from_slice(&0xe000u16.to_le_bytes());
            let mut next = State {
                host_steps: release.host_steps + 1,
                polls: release.polls + 1,
                y: 699,
                effective_buttons: 0x100,
                joy1_current: 0x80,
                joy1_new: 0x80,
                ..release
            };
            match invalid {
                0 => next.joy1_new = 0,
                1 => next.joy1_current = 0x280,
                2 => next.effective_buttons = 0x140,
                3 => ram[0x9b4] = 0,
                4 => ram[0x9a2] = 0,
                5 => ram[0xb33] = 0,
                6 => ram[0x1968] = 1,
                7 => next.x = 1476,
                8 => ram[0xb46] = 1,
                9 => ram[0xb36] = 2,
                10 => route.door.platform_step = None,
                11 => route.door.boarding_polls = 119,
                12 => ram[0xb2e] = 3,
                13 => ram[0xa1c] = 0x4e,
                14 => route.door.boarding_release_step = None,
                15 => {
                    route.climb_phase = 52;
                    route.door.boarding_jump_step = None;
                    ram[0xb00] = 19;
                }
                16 | 17 => {
                    route.climb_phase = 52;
                    route.door.boarding_jump_step = Some(release.host_steps);
                    route.door.boarding_jump = Some([699, 0x4c, 4, 0xe000, 1, 21, 0xffff]);
                    if invalid == 17 {
                        next.polls = release.polls;
                        next.y = 698;
                    }
                }
                _ => unreachable!(),
            }
            let previous_jump_step = route.door.boarding_jump_step;
            route.observe(&next, &ram);
            assert_eq!(route.failure(), Some("return elevator approach lost its raised-platform corridor"));
            assert_eq!(route.door.boarding_jump_step, previous_jump_step);
        }
    }

    #[cfg_attr(test, test)]
    pub(super) fn boarding_refuses_each_non_neutral_release_input() {
        for invalid in 0..3 {
            let (contact, mut ram, mut route) = boarding_state();
            ram[0xa1c] = 2;
            ram[0xa1f] = 0;
            ram[0xdd0] = 0;
            let mut release = State {
                host_steps: contact.host_steps + 1,
                polls: contact.polls + 1,
                effective_buttons: 0,
                joy1_current: 0,
                joy1_new: 0,
                ..contact
            };
            match invalid {
                0 => release.effective_buttons = 0x40,
                1 => release.joy1_current = 0x200,
                2 => release.joy1_new = 0x200,
                _ => unreachable!(),
            }
            route.observe(&release, &ram);
            assert_eq!(route.failure(), Some("return elevator approach lost its raised-platform corridor"));
            assert_eq!(route.door.boarding_release_step, None);
            assert_eq!(route.door.boarding_jump_step, None);
            assert_eq!(route.door.boarding_clear_step, None);
            assert_eq!(route.door.platform_step, Some(contact.host_steps - 1));
        }
    }

    #[cfg_attr(test, test)]
    pub(super) fn boarding_requires_consumed_feet_strictly_above_platform() {
        let (contact, mut ram, mut route) = boarding_state();
        ram[0xa1c] = 2;
        ram[0xa1f] = 0;
        ram[0xdd0] = 0;
        let release = State {
            host_steps: contact.host_steps + 1,
            polls: contact.polls + 1,
            effective_buttons: 0,
            joy1_current: 0,
            joy1_new: 0,
            ..contact
        };
        route.observe(&release, &ram);
        assert_eq!(route.failure(), None);
        ram[0xa1c] = 0x4c;
        ram[0xa1f] = 2;
        ram[0xb36] = 1;
        ram[0xb2e..0xb30].copy_from_slice(&4u16.to_le_bytes());
        ram[0xb2c..0xb2e].copy_from_slice(&0xe000u16.to_le_bytes());
        let jump = State {
            host_steps: release.host_steps + 1,
            polls: release.polls + 1,
            effective_buttons: 0x100,
            joy1_current: 0x80,
            joy1_new: 0x80,
            ..release
        };
        route.observe(&jump, &ram);
        assert_eq!(route.failure(), None);
        ram[0xb00..0xb02].copy_from_slice(&19u16.to_le_bytes());
        let transition = State { host_steps: jump.host_steps + 1, polls: jump.polls + 1, joy1_new: 0, ..jump };
        route.observe(&transition, &ram);
        assert_eq!(route.failure(), None);
        ram[0xa1c] = 0x4e;
        let edge = State { host_steps: transition.host_steps + 1, polls: transition.polls + 1, y: 686, ..transition };
        route.observe(&edge, &ram);
        assert_eq!(route.failure(), None);
        assert_eq!(route.door.boarding_clear_step, None);
        let clear = State { host_steps: edge.host_steps + 1, polls: edge.polls + 1, y: 685, ..edge };
        route.observe(&clear, &ram);
        assert_eq!(route.failure(), None);
        assert_eq!(route.climb_phase, 54);
        assert_eq!(route.door.boarding_clear_step, Some(clear.host_steps));
        assert_eq!(route.door.boarding_jump_step, Some(jump.host_steps));
        assert_eq!(route.door.platform_step, Some(contact.host_steps - 1));
        assert_eq!(route.door.boarding_polls, 6);
    }

    #[cfg_attr(test, test)]
    pub(super) fn lower_floor_refuses_upward_airborne_motion_within_its_bounds() {
        let (fallen, ram, mut route) = lower_floor_state();
        let airborne = State {
            host_steps: fallen.host_steps + 1,
            polls: fallen.polls + 1,
            x: 1945,
            y: 657,
            joy1_new: 0,
            ..fallen
        };
        route.observe(&airborne, &ram);
        assert_eq!(route.failure(), None);
        let upward =
            State { host_steps: airborne.host_steps + 1, polls: airborne.polls + 1, x: 1943, y: 656, ..airborne };
        route.observe(&upward, &ram);
        assert_eq!(route.failure(), Some("return elevator approach lost its lower-floor landing domain"));
        assert_eq!(route.door.lower_floor_step, None);
        assert_eq!(route.door.departure_fall_step, Some(fallen.host_steps));
    }

    fn crossing_state() -> (State, Vec<u8>, Return) {
        let (aimed, mut ram, mut route) = boarding_state();
        ram[0xa1c] = 2;
        ram[0xa1f] = 0;
        ram[0xdd0] = 0;
        let release = State {
            host_steps: aimed.host_steps + 1,
            polls: aimed.polls + 1,
            effective_buttons: 0,
            joy1_current: 0,
            joy1_new: 0,
            ..aimed
        };
        route.observe(&release, &ram);
        assert_eq!(route.failure(), None);
        for (at, value) in [(0xa1c, 0x4cu16), (0xb36, 1), (0xb2e, 4), (0xb2c, 0xe000)] {
            ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
        }
        ram[0xa1f] = 2;
        let jump = State {
            host_steps: release.host_steps + 1,
            polls: release.polls + 1,
            effective_buttons: 0x100,
            joy1_current: 0x80,
            joy1_new: 0x80,
            ..release
        };
        route.observe(&jump, &ram);
        assert_eq!(route.failure(), None);
        ram[0xa1c] = 0x4e;
        ram[0xb00] = 19;
        ram[0xb2c..0xb2e].copy_from_slice(&0x8c00u16.to_le_bytes());
        ram[0xafc..0xafe].copy_from_slice(&0xb3ffu16.to_le_bytes());
        let clear = State { host_steps: jump.host_steps + 1, polls: jump.polls + 1, y: 685, joy1_new: 0, ..jump };
        route.observe(&clear, &ram);
        assert_eq!(route.failure(), None);
        assert_eq!(route.climb_phase, 54);
        for col in 90..=91 {
            let at = 0x10002 + 2 * (40 * 128 + col);
            ram[at..at + 2].copy_from_slice(&0x81e9u16.to_le_bytes());
            ram[0x16402 + 40 * 128 + col] = 0;
        }
        (clear, ram, route)
    }

    fn descent_state() -> (State, Vec<u8>, Return) {
        let (mut state, mut ram, mut route) = crossing_state();
        state.x = 1467;
        state.y = 676;
        state.effective_buttons = 0x140;
        state.joy1_current = 0x280;
        for (at, value) in [
            (0xa1c, 0x52u16),
            (0xaf8, 0x8000),
            (0xafc, 0x1800),
            (0xb46, 1),
            (0xb48, 0x4000),
            (0xb2e, 0),
            (0xb2c, 0x8c00),
            (0xb36, 2),
            (0xb32, 0x1c00),
            (0xb34, 0),
            (0xdd0, 0),
        ] {
            ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
        }
        let elevator = 0xf78 + 128;
        ram[elevator + 0x30..elevator + 0x32].copy_from_slice(&706u16.to_le_bytes());
        ram[elevator + 0x2c..elevator + 0x2e].copy_from_slice(&0xf7u16.to_le_bytes());
        ram[elevator + 0x2e] = 0xa3; // Native animation/frame/bank fields remain foreign.
        route.climb_phase = 55;
        route.previous = Some(state);
        route.door.crossing_span_step = Some(state.host_steps);
        route.door.crossing_ceiling_step = Some(state.host_steps - 5);
        route.door.crossing_span_motion = Some([2, 0, 0]);
        route.door.crossing_span = Some([1467, 676, 0x52, 19, 0x8000, 0x1800, 1, 0x4000, 0, 0x8c00]);
        (state, ram, route)
    }

    #[cfg_attr(test, test)]
    pub(super) fn descent_latches_only_consumed_release_and_first_native_landing() {
        let (span, mut ram, mut route) = descent_state();
        let crossing = route.door.crossing_span;
        let counters = (route.door.boarding_polls, route.door.platform_polls, route.door.crossing_polls);
        let pending = State { host_steps: span.host_steps + 1, ..span };
        route.observe(&pending, &ram);
        assert_eq!(route.failure(), None);
        assert_eq!(route.door.descent_release_step, None);
        ram[0xb46..0xb4a].fill(0);
        let release = State {
            host_steps: pending.host_steps + 1,
            polls: pending.polls + 1,
            effective_buttons: 0,
            joy1_current: 0,
            joy1_new: 0,
            ..pending
        };
        route.observe(&release, &ram);
        assert_eq!(route.failure(), None);
        let witness = route.door.descent_release;
        assert_eq!(witness.unwrap()[9], 0x8c00); // Already descending; release must not require zero speed.
        assert_eq!(route.door.descent_release_step, Some(release.host_steps));
        ram[0xafc..0xafe].copy_from_slice(&0xf400u16.to_le_bytes());
        let edge = State { host_steps: release.host_steps + 1, polls: release.polls + 1, y: 684, ..release };
        route.observe(&edge, &ram);
        assert_eq!(route.failure(), None);
        assert_eq!(route.door.descent_landing_step, None);
        ram[0xa1c] = 0xa5;
        ram[0xa1f] = 0;
        ram[0xafc..0xafe].fill(0xff);
        ram[0xb2c..0xb30].fill(0);
        ram[0xb36..0xb38].fill(0);
        ram[0xdd0] = 0;
        let landing_pending = State { host_steps: edge.host_steps + 1, y: 683, ..edge };
        route.observe(&landing_pending, &ram);
        assert_eq!(route.failure(), None);
        assert_eq!(route.door.descent_landing_step, None);
        assert_eq!(route.door.descent_polls, 2);
        let landing =
            State { host_steps: landing_pending.host_steps + 1, polls: landing_pending.polls + 1, ..landing_pending };
        route.observe(&landing, &ram);
        assert_eq!(route.failure(), Some("return platform first ordinary landing requires reached-state review"));
        assert_eq!(route.door.descent_release, witness);
        assert_eq!(route.door.descent_landing_step, Some(landing.host_steps));
        assert_eq!(route.door.crossing_span, crossing);
        assert_eq!((route.door.boarding_polls, route.door.platform_polls, route.door.crossing_polls), counters);
        let mut report = sm_binary_evidence::Map::new();
        route.report(&mut report);
        assert_eq!(report["rio_return_descent_landing_y_subposition"].as_integer(), Some(65535));
        assert_eq!(report["rio_return_descent_release_y_subspeed"].as_integer(), Some(0x8c00));
    }

    #[cfg_attr(test, test)]
    pub(super) fn descent_refuses_foreign_input_motion_terrain_actor_and_missing_witness() {
        for case in 0..20 {
            let (span, mut ram, mut route) = descent_state();
            ram[0xb46..0xb4a].fill(0);
            let mut next = State {
                host_steps: span.host_steps + 1,
                polls: span.polls + 1,
                effective_buttons: 0,
                joy1_current: 0,
                joy1_new: 0,
                ..span
            };
            match case {
                0 => next.effective_buttons = 0x40,
                1 => next.joy1_current = 0x200,
                2 => next.joy1_new = 0x80,
                3 => next.x -= 1,
                4 => ram[0xaf8] ^= 1,
                5 => ram[0xb48] = 1,
                6 => ram[0xb4a] = 1,
                7 => ram[0xb36] = 1,
                8 => ram[0xafd] = 0, // Fractional regression within the same integer Y.
                9 => ram[0x16402 + 44 * 128 + 91] = 3,
                10 => ram[0xe18] = 1,
                11 => route.door.crossing_span_step = None,
                12 => route.door.crossing_ceiling_step = None,
                13 => route.door.descent_polls = 32,
                14 => ram[0xb32] ^= 1,
                15 => ram[0x9a2] ^= 1,
                16 => ram[0x9b4] ^= 1,
                17 => ram[0x195e] ^= 1,
                18 => ram[0x1962] ^= 1,
                19 => ram[0x1968] = 1,
                _ => unreachable!(),
            }
            route.observe(&next, &ram);
            assert!(route.failure().is_some(), "case {case}");
            assert_eq!(route.door.descent_release_step, None, "case {case}");
            assert_eq!(route.door.descent_landing_step, None, "case {case}");
        }
    }

    #[cfg_attr(test, test)]
    pub(super) fn descent_checks_native_landing_geometry_motion_and_complete_report() {
        // Independent report-only values distinguish every position/motion slot
        // even though the authentic grounded endpoint has several zero speeds.
        let mut report_route = Return::default();
        report_route.door.descent_release = Some([101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113]);
        report_route.door.descent_landing = Some([201, 202, 203, 204, 205, 206, 207, 208, 209, 210, 211, 212, 213]);
        report_route.door.descent_release_step = Some(301);
        report_route.door.descent_landing_step = Some(302);
        report_route.door.descent_polls = 17;
        let mut distinct = sm_binary_evidence::Map::new();
        report_route.report(&mut distinct);
        for (stage, base) in [("release", 101), ("landing", 201)] {
            for (index, key) in [
                "x",
                "y",
                "pose",
                "radius",
                "x_subposition",
                "y_subposition",
                "x_speed",
                "x_subspeed",
                "y_speed",
                "y_subspeed",
                "y_direction",
                "collision",
                "x_acceleration",
            ]
            .into_iter()
            .enumerate()
            {
                assert_eq!(
                    distinct[&format!("rio_return_descent_{stage}_{key}")].as_integer(),
                    Some(base + index as i64)
                );
            }
        }
        assert_eq!(distinct["rio_return_descent_release_step"].as_integer(), Some(301));
        assert_eq!(distinct["rio_return_descent_landing_step"].as_integer(), Some(302));
        assert_eq!(distinct["rio_return_descent_polls"].as_integer(), Some(17));
        for case in 0..11 {
            let (span, mut ram, mut route) = descent_state();
            ram[0xb46..0xb4a].fill(0);
            let release = State {
                host_steps: span.host_steps + 1,
                polls: span.polls + 1,
                effective_buttons: 0,
                joy1_current: 0,
                joy1_new: 0,
                ..span
            };
            route.observe(&release, &ram);
            assert_eq!(route.failure(), None);
            let mut before = sm_binary_evidence::Map::new();
            route.report(&mut before);
            ram[0xa1c] = 0xa5;
            ram[0xa1f] = 0;
            ram[0xafc..0xafe].fill(0xff);
            ram[0xb2c..0xb30].fill(0);
            ram[0xb36..0xb38].fill(0);
            let mut landing = State { host_steps: release.host_steps + 1, polls: release.polls + 1, y: 683, ..release };
            match case {
                0 => {}
                1 => ram[0xb00] = 21,
                2 => ram[0xa1c] = 2,
                3 => ram[0xa1f] = 2,
                4 => ram[0xb00] = 20,
                5 => landing.y = 684,
                6 => ram[0xafc] = 0xfe,
                7 => ram[0xb2c] = 1,
                8 => ram[0xb36] = 2,
                9 => ram[0xdd0] = 1,
                10 => ram[0xf78 + 128 + 0x32] = 1,
                _ => unreachable!(),
            }
            route.observe(&landing, &ram);
            assert_eq!(route.door.descent_landing_step.is_some(), case < 2, "case {case}");
            if case < 2 {
                let mut after = sm_binary_evidence::Map::new();
                route.report(&mut after);
                for (stage, values) in [
                    ("release", [1467, 676, 0x52, 19, 0x8000, 0x1800, 0, 0, 0, 0x8c00, 2, 0, 0]),
                    (
                        "landing",
                        [1467, 683, 0xa5, if case == 0 { 19 } else { 21 }, 0x8000, 0xffff, 0, 0, 0, 0, 0, 0, 0],
                    ),
                ] {
                    for (key, expected) in [
                        "x",
                        "y",
                        "pose",
                        "radius",
                        "x_subposition",
                        "y_subposition",
                        "x_speed",
                        "x_subspeed",
                        "y_speed",
                        "y_subspeed",
                        "y_direction",
                        "collision",
                        "x_acceleration",
                    ]
                    .into_iter()
                    .zip(values)
                    {
                        assert_eq!(after[&format!("rio_return_descent_{stage}_{key}")].as_integer(), Some(expected));
                    }
                }
                for (key, value) in &before {
                    if [
                        "rio_return_crossing_",
                        "rio_return_boarding_",
                        "rio_return_platform_",
                        "rio_return_departure_",
                        "rio_return_lower_floor_",
                    ]
                    .iter()
                    .any(|prefix| key.starts_with(prefix))
                    {
                        assert_eq!(after.get(key), Some(value), "frozen {key}");
                    }
                }
            }
        }
    }

    #[cfg_attr(test, test)]
    pub(super) fn crossing_records_native_left_then_first_consumed_full_span() {
        let (clear, mut ram, mut route) = crossing_state();
        let clear_snapshot = route.door.boarding_clear;
        let platform_step = route.door.platform_step;
        assert_eq!(route.buttons(&clear, &ram), 0x280u16.reverse_bits());
        ram[0xa1c] = 0x52;
        for (at, value) in
            [(0xaf8, 0x1234u16), (0xafc, 0x5678), (0xb46, 1), (0xb48, 0x4000), (0xb2e, 3), (0xb2c, 0x9000)]
        {
            ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
        }
        let left = State {
            host_steps: clear.host_steps + 1,
            polls: clear.polls + 1,
            x: 1476,
            y: 681,
            effective_buttons: 0x140,
            joy1_current: 0x280,
            joy1_new: 0x200,
            ..clear
        };
        route.observe(&left, &ram);
        assert_eq!(route.failure(), None);
        assert_eq!(route.door.crossing_left_step, Some(left.host_steps));
        let edge =
            State { host_steps: left.host_steps + 1, polls: left.polls + 1, x: 1468, y: 679, joy1_new: 0, ..left };
        route.observe(&edge, &ram);
        assert_eq!(route.failure(), None);
        assert_eq!(route.door.crossing_span_step, None);
        let pending = State { host_steps: edge.host_steps + 1, x: 1467, y: 678, ..edge };
        for (at, value) in
            [(0xaf8, 0x2468u16), (0xafc, 0xabcd), (0xb46, 0), (0xb48, 0xc000), (0xb2e, 2), (0xb2c, 0x7000)]
        {
            ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
        }
        route.observe(&pending, &ram);
        assert_eq!(route.failure(), None);
        assert_eq!(route.door.crossing_span_step, None);
        let span = State { host_steps: pending.host_steps + 1, polls: pending.polls + 1, ..pending };
        route.observe(&span, &ram);
        assert_eq!(route.failure(), None);
        assert_eq!(route.climb_phase, 55);
        assert_eq!(route.buttons(&span, &ram), 0);
        assert_eq!(route.door.boarding_clear_step, Some(clear.host_steps));
        assert_eq!(route.door.boarding_clear, clear_snapshot);
        assert_eq!(route.door.platform_step, platform_step);
        assert_eq!(route.door.crossing_left_step, Some(left.host_steps));
        assert_eq!(route.door.boarding_polls, 4);
        let mut report = sm_binary_evidence::Map::new();
        route.report(&mut report);
        for (key, value) in [
            ("x", 1467),
            ("y", 678),
            ("pose", 0x52),
            ("radius", 19),
            ("x_subposition", 0x2468),
            ("y_subposition", 0xabcd),
            ("x_speed", 0),
            ("x_subspeed", 0xc000),
            ("y_speed", 2),
            ("y_subspeed", 0x7000),
        ] {
            assert_eq!(report[&format!("rio_return_crossing_span_{key}")].as_integer(), Some(value));
        }
        for (key, value) in [
            ("x", 1476),
            ("y", 681),
            ("pose", 0x52),
            ("radius", 19),
            ("x_subposition", 0x1234),
            ("y_subposition", 0x5678),
            ("x_speed", 1),
            ("x_subspeed", 0x4000),
            ("y_speed", 3),
            ("y_subspeed", 0x9000),
        ] {
            assert_eq!(report[&format!("rio_return_crossing_left_{key}")].as_integer(), Some(value));
        }
        assert_eq!(report["rio_return_crossing_left_step"].as_integer(), Some(left.host_steps as i64));
        assert_eq!(report["rio_return_crossing_span_step"].as_integer(), Some(span.host_steps as i64));
        assert_eq!(report["rio_return_crossing_polls"].as_integer(), Some(3));
    }

    #[cfg_attr(test, test)]
    pub(super) fn crossing_requires_consumed_ceiling_before_descent_and_preserves_witnesses() {
        let (clear, mut ram, mut route) = crossing_state();
        let clear_snapshot = route.door.boarding_clear;
        ram[0xa1c] = 0x52;
        ram[0xafc..0xafe].copy_from_slice(&0x27ffu16.to_le_bytes());
        let left = State {
            host_steps: clear.host_steps + 1,
            polls: clear.polls + 1,
            x: 1476,
            y: 681,
            effective_buttons: 0x140,
            joy1_current: 0x280,
            joy1_new: 0x200,
            ..clear
        };
        route.observe(&left, &ram);
        assert_eq!(route.failure(), None);
        let left_snapshot = route.door.crossing_left;
        for (at, value) in [(0xaf8, 0xc000u16), (0xafc, 0), (0xb2e, 0), (0xb2c, 0), (0xb36, 2), (0xdd0, 1)] {
            ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
        }
        let pending = State { host_steps: left.host_steps + 1, x: 1473, y: 675, joy1_new: 0, ..left };
        route.observe(&pending, &ram);
        assert_eq!(route.failure(), None);
        assert_eq!(route.door.crossing_ceiling_step, None);
        assert_eq!(route.door.crossing_polls, 1);
        let ceiling = State { host_steps: pending.host_steps + 1, polls: pending.polls + 1, ..pending };
        route.observe(&ceiling, &ram);
        assert_eq!(route.failure(), None);
        assert_eq!(route.door.crossing_ceiling_step, Some(ceiling.host_steps));
        let ceiling_snapshot = route.door.crossing_ceiling;
        // Native displacement precedes acceleration: the first downward update can keep Y unchanged.
        ram[0xdd0] = 0;
        ram[0xb2c..0xb2e].copy_from_slice(&0x1c00u16.to_le_bytes());
        let equal = State { host_steps: ceiling.host_steps + 1, polls: ceiling.polls + 1, x: 1472, ..ceiling };
        route.observe(&equal, &ram);
        assert_eq!(route.failure(), None);
        ram[0xafc..0xafe].copy_from_slice(&0x9000u16.to_le_bytes());
        let edge = State { host_steps: equal.host_steps + 1, polls: equal.polls + 1, x: 1468, y: 676, ..equal };
        route.observe(&edge, &ram);
        assert_eq!(route.failure(), None);
        assert_eq!(route.door.crossing_span_step, None);
        let span = State { host_steps: edge.host_steps + 1, polls: edge.polls + 1, x: 1467, ..edge };
        route.observe(&span, &ram);
        assert_eq!(route.failure(), None);
        assert_eq!(route.climb_phase, 55);
        assert_eq!(route.buttons(&span, &ram), 0);
        assert_eq!(route.door.crossing_ceiling, ceiling_snapshot);
        assert_eq!(route.door.crossing_left, left_snapshot);
        assert_eq!(route.door.boarding_clear, clear_snapshot);
        assert_eq!(route.door.boarding_clear_step, Some(clear.host_steps));
        assert_eq!(route.door.boarding_polls, 4);
        let mut report = sm_binary_evidence::Map::new();
        route.report(&mut report);
        for (key, value) in [
            ("ceiling_step", ceiling.host_steps as i64),
            ("ceiling_x", 1473),
            ("ceiling_y", 675),
            ("ceiling_pose", 0x52),
            ("ceiling_radius", 19),
            ("ceiling_x_subposition", 0xc000),
            ("ceiling_y_subposition", 0),
            ("ceiling_x_speed", 0),
            ("ceiling_x_subspeed", 0),
            ("ceiling_y_speed", 0),
            ("ceiling_y_subspeed", 0),
            ("ceiling_y_direction", 2),
            ("ceiling_collision", 1),
            ("ceiling_x_acceleration", 0),
            ("span_y_direction", 2),
            ("span_collision", 0),
            ("span_x_acceleration", 0),
            ("polls", 5),
        ] {
            assert_eq!(report[&format!("rio_return_crossing_{key}")].as_integer(), Some(value));
        }
    }

    #[cfg_attr(test, test)]
    pub(super) fn crossing_refuses_unproved_ceiling_and_reversed_fractional_motion() {
        for invalid in 0..12 {
            let (clear, mut ram, mut route) = crossing_state();
            ram[0xa1c] = 0x52;
            let left = State {
                host_steps: clear.host_steps + 1,
                polls: clear.polls + 1,
                x: if invalid == 10 { 1477 } else { 1476 },
                y: 681,
                effective_buttons: 0x140,
                joy1_current: 0x280,
                joy1_new: 0x200,
                ..clear
            };
            route.observe(&left, &ram);
            assert_eq!(route.failure(), None);
            for (at, value) in [(0xafc, 0u16), (0xb2e, 0), (0xb2c, 0), (0xb36, 2), (0xdd0, 1)] {
                ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
            }
            let mut next =
                State { host_steps: left.host_steps + 1, polls: left.polls + 1, x: 1473, y: 675, joy1_new: 0, ..left };
            match invalid {
                0 => next.y = 676,
                1 => ram[0xafc] = 1,
                2 => ram[0xb2e] = 1,
                3 => ram[0xb2c] = 1,
                4 => ram[0xdd0] = 0,
                5 => ram[0xb4a] = 1,
                6 => {
                    route.observe(&next, &ram);
                    assert_eq!(route.failure(), None);
                    next.host_steps += 1;
                    next.polls += 1;
                    ram[0xb36] = 1;
                }
                7 | 8 => {
                    route.observe(&next, &ram);
                    assert_eq!(route.failure(), None);
                    next.host_steps += 1;
                    next.polls += 1;
                    next.y = 676;
                    ram[0xafc..0xafe].copy_from_slice(&0x8000u16.to_le_bytes());
                    route.observe(&next, &ram);
                    assert_eq!(route.failure(), None);
                    next.host_steps += 1;
                    next.polls += 1;
                    if invalid == 7 {
                        ram[0xafc..0xafe].copy_from_slice(&0x7fffu16.to_le_bytes());
                    } else {
                        next.y = 686;
                    }
                }
                9 => {
                    next.polls = left.polls;
                    route.observe(&next, &ram);
                    assert_eq!(route.failure(), None);
                    assert_eq!(route.door.crossing_ceiling_step, None);
                    next.host_steps += 1;
                    next.polls += 1;
                    ram[0xafc] = 1;
                }
                10 => {
                    next.x = 1477;
                }
                11 => {
                    ram[0xb36] = 1;
                    ram[0xdd0] = 0;
                    next.y = 681;
                    ram[0xafc..0xafe].copy_from_slice(&0xb3feu16.to_le_bytes());
                    route.observe(&next, &ram);
                    assert_eq!(route.failure(), None);
                    next.host_steps += 1;
                    next.polls += 1;
                    ram[0xafc..0xafe].copy_from_slice(&0xb3ffu16.to_le_bytes());
                }
                _ => unreachable!(),
            }
            let witness = route.door.crossing_ceiling;
            let left_witness = route.door.crossing_left;
            let polls = route.door.crossing_polls;
            route.observe(&next, &ram);
            assert_eq!(route.failure(), Some("return platform crossing lost its held-jump clearance domain"));
            assert_eq!(route.door.crossing_ceiling, witness);
            assert_eq!(route.door.crossing_left, left_witness);
            assert_eq!(route.door.crossing_polls, polls);
            assert_eq!(route.door.crossing_span_step, None);
        }
    }

    #[cfg_attr(test, test)]
    pub(super) fn crossing_refuses_unowned_input_and_foreign_geometry_or_motion() {
        for invalid in 0..12 {
            let (clear, mut ram, mut route) = crossing_state();
            ram[0xa1c] = 0x52;
            let mut next = State {
                host_steps: clear.host_steps + 1,
                polls: clear.polls + 1,
                x: 1476,
                y: 681,
                effective_buttons: 0x140,
                joy1_current: 0x280,
                joy1_new: 0x200,
                ..clear
            };
            match invalid {
                0 => route.door.boarding_clear_step = None,
                1 => route.door.crossing_polls = 119,
                2 => next.effective_buttons = 0x40,
                3 => next.joy1_current = 0x200,
                4 => next.joy1_new = 0x100,
                5 => ram[0xa1c] = 0x4e,
                6 => ram[0xb36] = 2,
                7 => ram[0xb00] = 21,
                8 => ram[0x10002 + 2 * (40 * 128 + 91) + 1] = 0,
                9 => ram[0x16402 + 44 * 128 + 91] = 3,
                10 => next.y = 686,
                11 => ram[0xff8] = 0,
                _ => unreachable!(),
            }
            route.observe(&next, &ram);
            assert_eq!(route.failure(), Some("return platform crossing lost its held-jump clearance domain"));
            assert_eq!(route.door.crossing_left_step, None);
        }
    }

    fn upper_transfer_state() -> (State, Vec<u8>, Return) {
        let (state, mut ram, mut route) = third_state();
        route.third_response_step = Some(state.host_steps);
        route.climb_phase = 34;
        ram[0xa1c] = 4;
        for (slot, col) in [(0, 6), (1, 7)] {
            ram[0x10002 + 2 * (128 + col)..0x10004 + 2 * (128 + col)].copy_from_slice(&0x53u16.to_le_bytes());
            ram[0x1c37 + 2 * slot..0x1c39 + 2 * slot].copy_from_slice(&0xd064u16.to_le_bytes());
            ram[0x1c87 + 2 * slot..0x1c89 + 2 * slot].copy_from_slice(&(2u16 * (128 + col) as u16).to_le_bytes());
            ram[0x1e17 + 2 * slot..0x1e19 + 2 * slot].copy_from_slice(&0xc052u16.to_le_bytes());
        }
        ram[0x10002 + 2 * 133..0x10004 + 2 * 133].copy_from_slice(&0x81ebu16.to_le_bytes());
        ram[0x10002 + 2 * 166..0x10004 + 2 * 166].copy_from_slice(&0x53u16.to_le_bytes());
        ram[0x16402 + 166] = 4;
        (state, ram, route)
    }

    #[cfg_attr(test, test)]
    pub(super) fn upper_transfer_releases_then_fresh_jumps_and_clears_feet_before_partial_contact() {
        let (state, mut ram, mut route) = upper_transfer_state();
        let pending = State { host_steps: state.host_steps + 1, ..state };
        ram[0xa1c] = 2;
        route.observe(&pending, &ram);
        assert_eq!(route.upper_release_step, None);
        let release = State { host_steps: pending.host_steps + 1, polls: pending.polls + 1, ..pending };
        route.observe(&release, &ram);
        assert_eq!(route.upper_release_step, Some(release.host_steps));
        let jump = word(&ram, 0x9b4);
        ram[0xa1c] = 0x4c;
        ram[0xa1f] = 2;
        ram[0xb00..0xb02].copy_from_slice(&21u16.to_le_bytes());
        ram[0xb36..0xb38].copy_from_slice(&1u16.to_le_bytes());
        let launch = State {
            host_steps: release.host_steps + 1,
            polls: release.polls + 1,
            effective_buttons: jump.reverse_bits(),
            joy1_current: jump,
            joy1_new: jump,
            y: 167,
            ..release
        };
        route.observe(&launch, &ram);
        assert_eq!(route.failure(), None);
        assert_eq!(route.upper_jump_step, Some(launch.host_steps));
        assert_eq!(route.buttons(&launch, &ram), jump.reverse_bits());
        ram[0xa1c] = 0x4e;
        ram[0xb00..0xb02].copy_from_slice(&19u16.to_le_bytes());
        let feet_pending = State { host_steps: launch.host_steps + 1, y: 109, ..launch };
        route.observe(&feet_pending, &ram);
        assert_eq!(route.upper_feet_step, None);
        assert_eq!(route.buttons(&feet_pending, &ram), jump.reverse_bits());
        let feet =
            State { host_steps: feet_pending.host_steps + 1, polls: launch.polls + 1, joy1_new: 0, y: 109, ..launch };
        route.observe(&feet, &ram);
        assert_eq!(route.buttons(&feet, &ram), 0x40 | jump.reverse_bits());
        assert_eq!(route.upper_feet_step, Some(feet.host_steps));
        ram[0xa1c] = 0xa5;
        ram[0xa1f] = 0;
        ram[0xb36..0xb38].fill(0);
        for (at, value) in
            [(0xaf8, 0x8000u16), (0xb42, 2), (0xb44, 3), (0xb46, 4), (0xb48, 5), (0xb4a, 6), (0xa6c, 0x9f55)]
        {
            ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
        }
        let unpolled = State { host_steps: feet.host_steps + 1, x: 100, y: 107, ..feet };
        route.observe(&unpolled, &ram);
        assert_eq!(route.upper_contact.as_ref().map(|c| c.step), None);
        let contact = State {
            host_steps: unpolled.host_steps + 1,
            polls: unpolled.polls + 1,
            effective_buttons: 0x40 | jump.reverse_bits(),
            joy1_current: 0x200 | jump,
            ..unpolled
        };
        route.observe(&contact, &ram);
        assert_eq!(route.failure(), None);
        assert_eq!(route.climb_phase, 39);
        assert_eq!(route.buttons(&contact, &ram), 0);
        assert_eq!(route.upper_contact.as_ref().map(|c| (c.x, c.radius, c.pose)), Some((100, 19, 0xa5)));
        let mut report = sm_binary_evidence::Map::new();
        route.report(&mut report);
        assert_eq!(report["rio_return_upper_contact_x"].as_integer(), Some(100));
        for (key, value) in [
            ("pose", 0xa5),
            ("radius", 19),
            ("x_subposition", 0x8000),
            ("extra_speed", 2),
            ("extra_subspeed", 3),
            ("speed", 4),
            ("subspeed", 5),
            ("accel_mode", 6),
            ("speed_table", 0x9f55),
        ] {
            assert_eq!(report[&format!("rio_return_upper_contact_{key}")].as_integer(), Some(value));
        }
        assert_eq!(report["rio_return_upper_contact_step"].as_integer(), Some(contact.host_steps as i64));
        assert_eq!(report["rio_return_complete"].as_bool(), Some(false));
    }

    #[cfg_attr(test, test)]
    pub(super) fn upper_transfer_refuses_restored_opening_foreign_plm_headroom_input_and_uncleared_feet() {
        for invalid in 0..11 {
            let (state, mut ram, mut route) = upper_transfer_state();
            route.climb_phase = 36;
            route.upper_release_step = Some(state.host_steps);
            route.upper_jump_step = Some(state.host_steps);
            route.upper_feet_step = Some(state.host_steps);
            ram[0xa1c] = 0x4e;
            ram[0xa1f] = 2;
            ram[0xb00..0xb02].copy_from_slice(&19u16.to_le_bytes());
            let jump = word(&ram, 0x9b4);
            let mut next = State {
                host_steps: state.host_steps + 1,
                polls: state.polls + 1,
                x: 99,
                y: 109,
                effective_buttons: 0x40 | jump.reverse_bits(),
                joy1_current: 0x200 | jump,
                joy1_new: 0,
                ..state
            };
            match invalid {
                0 => ram[0x10002 + 2 * 134 + 1] = 0xc0,
                1 => ram[0x1c37] = 0,
                2 => ram[0x10002 + 2 * 133 + 1] = 0,
                3 => ram[0x10002 + 2 * (4 * 16 + 5) + 1] = 0x80,
                4 => next.joy1_current |= word(&ram, 0x9b2),
                5 => next.y = 110,
                6 => route.upper_transfer_polls = 119,
                7 => ram[0x1c87] = 0,
                8 => ram[0x1e17] = 0,
                9 => {
                    ram[0xa1c] = 0xa5;
                    ram[0xa1f] = 3;
                    next.y = 107;
                }
                10 => {
                    ram[0xa1c] = 0xa5;
                    ram[0xa1f] = 0;
                    ram[0xb00] = 20;
                    next.y = 107;
                }
                _ => {}
            }
            route.observe(&next, &ram);
            assert!(route.failure().is_some());
            assert!(route.upper_contact.is_none());
        }
    }

    #[cfg_attr(test, test)]
    pub(super) fn upper_jump_rejects_nonfresh_spin_and_wrong_radius_before_recording_ownership() {
        for invalid in 0..5 {
            let (state, mut ram, mut route) = upper_transfer_state();
            ram[0xa1c] = 2;
            let release = State { host_steps: state.host_steps + 1, polls: state.polls + 1, ..state };
            route.observe(&release, &ram);
            let jump = word(&ram, 0x9b4);
            let pending =
                State { host_steps: release.host_steps + 1, effective_buttons: jump.reverse_bits(), ..release };
            route.observe(&pending, &ram);
            assert_eq!(route.upper_jump_step, None);
            ram[0xa1c] = 0x4c;
            ram[0xa1f] = 2;
            ram[0xb00..0xb02].copy_from_slice(&19u16.to_le_bytes());
            if invalid == 4 {
                let unpolled_launch =
                    State { host_steps: pending.host_steps + 1, joy1_current: jump, joy1_new: jump, ..pending };
                route.observe(&unpolled_launch, &ram);
                assert_eq!(route.upper_jump_step, None);
                assert!(route.failure().is_some());
                continue;
            }
            let mut launch = State {
                host_steps: pending.host_steps + 1,
                polls: pending.polls + 1,
                effective_buttons: jump.reverse_bits(),
                joy1_current: jump,
                joy1_new: jump,
                ..pending
            };
            match invalid {
                0 => launch.joy1_new = 0,
                1 => ram[0xa1f] = 3,
                2 => ram[0xb00] = 20,
                3 => ram[0xa1c] = 0x4e,
                _ => {}
            }
            route.observe(&launch, &ram);
            assert!(route.failure().is_some());
            assert_eq!(route.upper_jump_step, None);
            assert!(route.upper_contact.is_none());
        }
    }

    #[cfg_attr(test, test)]
    pub(super) fn aerial_reaction_must_be_air_with_cleared_feet_before_transfer() {
        for solid in [false, true] {
            let (state, mut ram, mut route) = aerial_state();
            let target = 0x10002 + 2 * (10 * 16 + 7);
            ram[target..target + 2].copy_from_slice(&(if solid { 0x819au16 } else { 0x0053u16 }).to_le_bytes());
            let next = State {
                host_steps: state.host_steps + 1,
                polls: state.polls + 1,
                y: if solid { 171 } else { 174 },
                effective_buttons: 0x300,
                joy1_current: 0x80 | word(&ram, 0x9b2),
                joy1_new: word(&ram, 0x9b2),
                ..state
            };
            route.observe(&next, &ram);
            assert_eq!(route.aerial_beam_step, Some(next.host_steps));
            assert_eq!(route.failure(), Some("return transfer has no open target or cleared feet"));
            assert_eq!(route.buttons(&next, &ram), 0);
            assert!(route.transfer_contact.is_none());
        }
    }

    #[cfg_attr(test, test)]
    pub(super) fn aerial_beam_refuses_invalid_consumed_edges_and_bound_before_reaction() {
        for invalid in 0..8 {
            let (window, mut ram, mut route) = aerial_state();
            let target = 0x10002 + 2 * (10 * 16 + 7);
            ram[target..target + 2].copy_from_slice(&0x0053u16.to_le_bytes());
            let mut next = State {
                host_steps: window.host_steps + 1,
                polls: 102,
                effective_buttons: 0x300,
                joy1_current: 0x80 | word(&ram, 0x9b2),
                joy1_new: word(&ram, 0x9b2),
                ..window
            };
            match invalid {
                0 => ram[0x16402 + 10 * 16 + 7] = 0,
                1 => next.joy1_new = 0,
                2 => route.aerial_previous = Some((0x4d, 174)),
                3 => route.aerial_previous = Some((0x4e, 176)),
                4 => route.aerial_polls = 119,
                5 => next.effective_buttons = 0x100,
                6 => ram[0xb5e..0xb60].copy_from_slice(&0x8006u16.to_le_bytes()),
                _ => {
                    ram[0xb5e..0xb60].copy_from_slice(&0x8007u16.to_le_bytes());
                    next.joy1_new = 0;
                }
            }
            route.observe(&next, &ram);
            assert!(route.failure().is_some());
            assert_eq!(route.aerial_beam_step, None);
            assert_eq!(route.aerial_response_step, None);
            assert_eq!(route.buttons(&next, &ram), 0);
        }
    }

    #[cfg_attr(test, test)]
    pub(super) fn airborne_survey_requires_neutral_launch_and_actual_jump_before_horizontal_window() {
        let (state, mut ram, mut route) = survey_state();
        assert_eq!(route.buttons(&state, &ram), 0);
        ram[0xa1c..0xa1e].copy_from_slice(&2u16.to_le_bytes());
        let launch = State { host_steps: state.host_steps + 1, polls: 101, joy1_current: 0, ..state };
        route.observe(&launch, &ram);
        assert_eq!(route.buttons(&launch, &ram), 0x100);
        let waiting = State { host_steps: launch.host_steps + 1, effective_buttons: 0x100, ..launch };
        route.observe(&waiting, &ram);
        assert_eq!(route.survey_jump_step, None);
        ram[0xa1c..0xa1e].copy_from_slice(&0x4cu16.to_le_bytes());
        ram[0xa1f] = 2;
        ram[0xb00..0xb02].copy_from_slice(&19u16.to_le_bytes());
        ram[0xb36..0xb38].copy_from_slice(&1u16.to_le_bytes());
        let jumped = State {
            host_steps: waiting.host_steps + 1,
            polls: 102,
            joy1_current: 0x0080,
            joy1_new: 0x0080,
            y: 230,
            ..waiting
        };
        route.observe(&jumped, &ram);
        assert_eq!(route.survey_jump_step, Some(jumped.host_steps));
        assert_eq!(route.survey_window_step, None);
        ram[0xa1c..0xa1e].copy_from_slice(&0x4eu16.to_le_bytes());
        let window = State { host_steps: jumped.host_steps + 1, polls: 103, y: 175, joy1_new: 0, ..jumped };
        route.observe(&window, &ram);
        assert_eq!(route.survey_window_step, Some(window.host_steps));
        assert_eq!(route.failure(), None);
        assert_eq!(route.buttons(&window, &ram), 0x300);
    }

    #[cfg_attr(test, test)]
    pub(super) fn airborne_survey_refuses_wrong_pose_radius_opening_and_terminal_bound() {
        for invalid in 0..7 {
            let (state, mut ram, mut route) = survey_state();
            route.climb_phase = 22;
            route.survey_jump_step = Some(state.host_steps);
            ram[0xa1c..0xa1e].copy_from_slice(&0x4eu16.to_le_bytes());
            ram[0xa1f] = 2;
            ram[0xb00..0xb02].copy_from_slice(&19u16.to_le_bytes());
            ram[0xb36..0xb38].copy_from_slice(&1u16.to_le_bytes());
            let next = State { host_steps: state.host_steps + 1, polls: 101, y: 175, joy1_current: 0x80, ..state };
            match invalid {
                0 => ram[0xa1c..0xa1e].copy_from_slice(&0x4du16.to_le_bytes()),
                1 => ram[0xb00..0xb02].copy_from_slice(&12u16.to_le_bytes()),
                2 => {
                    let opening = 0x10002 + 2 * (10 * 16 + 8);
                    ram[opening..opening + 2].copy_from_slice(&0xc19au16.to_le_bytes());
                }
                3 => route.survey_polls = 119,
                4 => ram[0xa1f] = 3,
                5 => ram[0xb36..0xb38].copy_from_slice(&2u16.to_le_bytes()),
                _ => ram[0xb5e..0xb60].copy_from_slice(&9u16.to_le_bytes()),
            }
            route.observe(&next, &ram);
            assert_eq!(route.survey_window_step, None);
        }
    }

    #[cfg_attr(test, test)]
    pub(super) fn airborne_survey_refuses_wrong_previous_launch_pose_and_held_foreign_inputs() {
        for invalid in 0..4 {
            let (state, mut ram, mut route) = survey_state();
            route.climb_phase = 21;
            route.survey_launch_pose = Some(if invalid == 0 { 4 } else { 2 });
            ram[0xa1c..0xa1e].copy_from_slice(&0x4cu16.to_le_bytes());
            ram[0xa1f] = 2;
            ram[0xb00..0xb02].copy_from_slice(&19u16.to_le_bytes());
            let mut next = State {
                host_steps: state.host_steps + 1,
                polls: 101,
                effective_buttons: 0x100,
                joy1_current: 0x80,
                joy1_new: 0x80,
                y: 230,
                ..state
            };
            match invalid {
                1 => next.joy1_current |= 0x0800,
                2 => next.joy1_current |= word(&ram, 0x9b2),
                3 => next.joy1_current |= 0x0200,
                _ => {}
            }
            route.observe(&next, &ram);
            assert_eq!(route.survey_jump_step, None);
            assert_eq!(route.survey_window_step, None);
            assert!(route.failure().is_some());
        }
    }
}

#[cfg(feature = "private-checks")]
pub(crate) fn private_assertions() -> Vec<(&'static str, fn())> {
    vec![
        (
            "wrong_prefix_is_terminal_and_never_emits_input",
            tests::wrong_prefix_is_terminal_and_never_emits_input as fn(),
        ),
        (
            "beam_request_waits_for_consumed_edge_then_releases",
            tests::beam_request_waits_for_consumed_edge_then_releases as fn(),
        ),
        (
            "first_ledge_steering_waits_until_the_feet_clear_terrain",
            tests::first_ledge_steering_waits_until_the_feet_clear_terrain as fn(),
        ),
        (
            "first_ledge_clearance_refuses_morph_and_spin_jump_domains",
            tests::first_ledge_clearance_refuses_morph_and_spin_jump_domains as fn(),
        ),
        (
            "morph_request_survives_unpolled_steps_and_releases_consumed_down",
            tests::morph_request_survives_unpolled_steps_and_releases_consumed_down as fn(),
        ),
        ("construction_pads_refuse_shoot_aliases", tests::construction_pads_refuse_shoot_aliases as fn()),
        (
            "unmorph_crouch_waits_for_standing_before_new_jump",
            tests::unmorph_crouch_waits_for_standing_before_new_jump as fn(),
        ),
        (
            "landing_requires_standing_radius_and_solid_support",
            tests::landing_requires_standing_radius_and_solid_support as fn(),
        ),
        (
            "ceiling_shot_waits_for_aim_and_consumption_then_observes_solid_response",
            tests::ceiling_shot_waits_for_aim_and_consumption_then_observes_solid_response as fn(),
        ),
        (
            "ceiling_probe_refuses_wrong_target_and_bounds_missing_response",
            tests::ceiling_probe_refuses_wrong_target_and_bounds_missing_response as fn(),
        ),
        (
            "upper_ledge_jump_waits_for_opening_and_feet_before_steering",
            tests::upper_ledge_jump_waits_for_opening_and_feet_before_steering as fn(),
        ),
        (
            "second_ledge_settling_corrects_the_observed_rightward_overshoot",
            tests::second_ledge_settling_corrects_the_observed_rightward_overshoot as fn(),
        ),
        (
            "second_ledge_ground_contact_releases_jump_through_running_and_turning",
            tests::second_ledge_ground_contact_releases_jump_through_running_and_turning as fn(),
        ),
        (
            "second_ledge_jump_release_requires_supported_ground_contact",
            tests::second_ledge_jump_release_requires_supported_ground_contact as fn(),
        ),
        (
            "upper_ceiling_beam_waits_for_rest_aim_and_consumed_shot_before_same_poll_reaction",
            tests::upper_ceiling_beam_waits_for_rest_aim_and_consumed_shot_before_same_poll_reaction as fn(),
        ),
        (
            "upper_ceiling_beam_refuses_wrong_target_facing_and_terminal_bound",
            tests::upper_ceiling_beam_refuses_wrong_target_facing_and_terminal_bound as fn(),
        ),
        (
            "upper_ceiling_consumed_shot_releases_before_later_reaction",
            tests::upper_ceiling_consumed_shot_releases_before_later_reaction as fn(),
        ),
        (
            "row10_beam_waits_for_native_cooldown_and_consumption_then_releases",
            tests::row10_beam_waits_for_native_cooldown_and_consumption_then_releases as fn(),
        ),
        (
            "row10_beam_orders_same_poll_reaction_and_refuses_unowned_changes_or_bounds",
            tests::row10_beam_orders_same_poll_reaction_and_refuses_unowned_changes_or_bounds as fn(),
        ),
        (
            "row10_ready_waits_for_shoot_release_and_freezes_prior_counter",
            tests::row10_ready_waits_for_shoot_release_and_freezes_prior_counter as fn(),
        ),
        (
            "row8_beam_freezes_row10_counter_and_orders_consumed_same_poll_reaction",
            tests::row8_beam_freezes_row10_counter_and_orders_consumed_same_poll_reaction as fn(),
        ),
        (
            "row8_beam_refuses_wrong_bts_closed_intermediate_opening_and_terminal_bound",
            tests::row8_beam_refuses_wrong_bts_closed_intermediate_opening_and_terminal_bound as fn(),
        ),
        (
            "aerial_beam_waits_for_consumption_then_releases_shoot_and_records_prior_origin",
            tests::aerial_beam_waits_for_consumption_then_releases_shoot_and_records_prior_origin as fn(),
        ),
        (
            "aerial_beam_refuses_unready_window_before_emitting_shoot",
            tests::aerial_beam_refuses_unready_window_before_emitting_shoot as fn(),
        ),
        (
            "transfer_waits_for_released_shoot_and_native_ground_contact_then_freezes_momentum",
            tests::transfer_waits_for_released_shoot_and_native_ground_contact_then_freezes_momentum as fn(),
        ),
        (
            "transfer_refuses_foreign_input_closed_terrain_and_bound_before_contact",
            tests::transfer_refuses_foreign_input_closed_terrain_and_bound_before_contact as fn(),
        ),
        (
            "transfer_requires_consumed_contact_and_actual_zero_vertical_speed",
            tests::transfer_requires_consumed_contact_and_actual_zero_vertical_speed as fn(),
        ),
        (
            "transfer_accepts_only_native_a5_landing_transition_with_prior_airborne_radius",
            tests::transfer_accepts_only_native_a5_landing_transition_with_prior_airborne_radius as fn(),
        ),
        (
            "support_requires_consumed_neutral_standing_before_left_only_full_span",
            tests::support_requires_consumed_neutral_standing_before_left_only_full_span as fn(),
        ),
        (
            "support_refuses_jump_new_or_effective_foreign_keys_and_bound_before_span",
            tests::support_refuses_jump_new_or_effective_foreign_keys_and_bound_before_span as fn(),
        ),
        (
            "support_keeps_neutral_when_contact_fraction_or_speed_differs",
            tests::support_keeps_neutral_when_contact_fraction_or_speed_differs as fn(),
        ),
        (
            "support_requires_fully_neutral_consumed_keys_before_standing_authorizes_left",
            tests::support_requires_fully_neutral_consumed_keys_before_standing_authorizes_left as fn(),
        ),
        (
            "brake_requires_consumed_release_and_actual_zero_motion_left_standing",
            tests::brake_requires_consumed_release_and_actual_zero_motion_left_standing as fn(),
        ),
        (
            "brake_refuses_unreleased_keys_lost_span_spin_or_bound_before_rest",
            tests::brake_refuses_unreleased_keys_lost_span_spin_or_bound_before_rest as fn(),
        ),
        (
            "brake_waits_for_zero_extra_speed_and_acceleration_before_rest",
            tests::brake_waits_for_zero_extra_speed_and_acceleration_before_rest as fn(),
        ),
        (
            "brake_accepts_only_witnessed_left_wall_transition_before_actual_standing_rest",
            tests::brake_accepts_only_witnessed_left_wall_transition_before_actual_standing_rest as fn(),
        ),
        (
            "wall_aim_refuses_foreign_keys_motion_or_pose_and_requires_release_before_rest",
            tests::wall_aim_refuses_foreign_keys_motion_or_pose_and_requires_release_before_rest as fn(),
        ),
        (
            "third_ceiling_requires_consumed_aim_and_fresh_shot_before_same_poll_initial_draw_reaction",
            tests::third_ceiling_requires_consumed_aim_and_fresh_shot_before_same_poll_initial_draw_reaction as fn(),
        ),
        (
            "third_ceiling_refuses_foreign_input_motion_target_change_and_false_plm_reaction",
            tests::third_ceiling_refuses_foreign_input_motion_target_change_and_false_plm_reaction as fn(),
        ),
        (
            "third_ceiling_delayed_reaction_requires_consumed_release_and_ready_preceding_observation",
            tests::third_ceiling_delayed_reaction_requires_consumed_release_and_ready_preceding_observation as fn(),
        ),
        (
            "upper_transfer_releases_left_at_full_span_and_holds_until_native_contact",
            tests::upper_transfer_releases_left_at_full_span_and_holds_until_native_contact as fn(),
        ),
        (
            "upper_hold_rejects_drift_foreign_momentum_and_unreleased_left",
            tests::upper_hold_rejects_drift_foreign_momentum_and_unreleased_left as fn(),
        ),
        (
            "upper_rest_waits_for_consumed_neutral_landing_animation_and_normal_radius",
            tests::upper_rest_waits_for_consumed_neutral_landing_animation_and_normal_radius as fn(),
        ),
        (
            "upper_rest_rejects_stale_jump_drift_motion_wrong_radius_and_partial_support",
            tests::upper_rest_rejects_stale_jump_drift_motion_wrong_radius_and_partial_support as fn(),
        ),
        (
            "door_shot_releases_and_requires_consumed_first_solid_draw",
            tests::door_shot_releases_and_requires_consumed_first_solid_draw as fn(),
        ),
        (
            "door_refuses_wrong_first_draw_identity_and_unowned_input",
            tests::door_refuses_wrong_first_draw_identity_and_unowned_input as fn(),
        ),
        (
            "opening_requires_ordered_consumed_neutral_full_air_draw",
            tests::opening_requires_ordered_consumed_neutral_full_air_draw as fn(),
        ),
        (
            "opening_refuses_skipped_draws_deleted_plm_and_held_inputs",
            tests::opening_refuses_skipped_draws_deleted_plm_and_held_inputs as fn(),
        ),
        (
            "left_approach_stops_only_at_consumed_native_falling_pose",
            tests::left_approach_stops_only_at_consumed_native_falling_pose as fn(),
        ),
        (
            "left_approach_refuses_unowned_input_and_changed_air_domain",
            tests::left_approach_refuses_unowned_input_and_changed_air_domain as fn(),
        ),
        (
            "traversal_records_consumed_landing_or_exact_native_door_event",
            tests::traversal_records_consumed_landing_or_exact_native_door_event as fn(),
        ),
        (
            "traversal_refuses_foreign_transition_and_unproved_landing",
            tests::traversal_refuses_foreign_transition_and_unproved_landing as fn(),
        ),
        (
            "exit_walk_preserves_landing_until_consumed_native_door_event",
            tests::exit_walk_preserves_landing_until_consumed_native_door_event as fn(),
        ),
        (
            "exit_walk_refuses_lost_landing_domain_and_unowned_event",
            tests::exit_walk_refuses_lost_landing_domain_and_unowned_event as fn(),
        ),
        (
            "loading_follows_native_order_and_records_consumed_destination_arrival",
            tests::loading_follows_native_order_and_records_consumed_destination_arrival as fn(),
        ),
        (
            "loading_refuses_foreign_headers_input_order_and_unproved_arrival",
            tests::loading_refuses_foreign_headers_input_order_and_unproved_arrival as fn(),
        ),
        (
            "departure_records_first_consumed_fall_and_preserves_arrival",
            tests::departure_records_first_consumed_fall_and_preserves_arrival as fn(),
        ),
        (
            "departure_refuses_foreign_actor_terrain_and_unowned_input",
            tests::departure_refuses_foreign_actor_terrain_and_unowned_input as fn(),
        ),
        (
            "lower_floor_latches_consumed_landing_with_native_pose_adjustment",
            tests::lower_floor_latches_consumed_landing_with_native_pose_adjustment as fn(),
        ),
        (
            "lower_floor_refuses_wrong_landing_terrain_motion_and_ownership",
            tests::lower_floor_refuses_wrong_landing_terrain_motion_and_ownership as fn(),
        ),
        (
            "platform_records_first_consumed_native_wall_and_preserves_landing",
            tests::platform_records_first_consumed_native_wall_and_preserves_landing as fn(),
        ),
        (
            "platform_refuses_foreign_corridor_input_actor_and_contact",
            tests::platform_refuses_foreign_corridor_input_actor_and_contact as fn(),
        ),
        (
            "boarding_requires_consumed_up_aim_before_neutral_and_jump",
            tests::boarding_requires_consumed_up_aim_before_neutral_and_jump as fn(),
        ),
        (
            "boarding_refuses_unowned_aim_and_missing_aim_checkpoint",
            tests::boarding_refuses_unowned_aim_and_missing_aim_checkpoint as fn(),
        ),
        (
            "boarding_releases_then_fresh_jumps_and_latches_only_consumed_feet_clearance",
            tests::boarding_releases_then_fresh_jumps_and_latches_only_consumed_feet_clearance as fn(),
        ),
        (
            "boarding_refuses_foreign_jump_context_horizontal_motion_and_stale_jump",
            tests::boarding_refuses_foreign_jump_context_horizontal_motion_and_stale_jump as fn(),
        ),
        (
            "boarding_refuses_each_non_neutral_release_input",
            tests::boarding_refuses_each_non_neutral_release_input as fn(),
        ),
        (
            "boarding_requires_consumed_feet_strictly_above_platform",
            tests::boarding_requires_consumed_feet_strictly_above_platform as fn(),
        ),
        (
            "lower_floor_refuses_upward_airborne_motion_within_its_bounds",
            tests::lower_floor_refuses_upward_airborne_motion_within_its_bounds as fn(),
        ),
        (
            "crossing_records_native_left_then_first_consumed_full_span",
            tests::crossing_records_native_left_then_first_consumed_full_span as fn(),
        ),
        (
            "descent_latches_only_consumed_release_and_first_native_landing",
            tests::descent_latches_only_consumed_release_and_first_native_landing as fn(),
        ),
        (
            "descent_refuses_foreign_input_motion_terrain_actor_and_missing_witness",
            tests::descent_refuses_foreign_input_motion_terrain_actor_and_missing_witness as fn(),
        ),
        (
            "descent_checks_native_landing_geometry_motion_and_complete_report",
            tests::descent_checks_native_landing_geometry_motion_and_complete_report as fn(),
        ),
        (
            "crossing_requires_consumed_ceiling_before_descent_and_preserves_witnesses",
            tests::crossing_requires_consumed_ceiling_before_descent_and_preserves_witnesses as fn(),
        ),
        (
            "crossing_refuses_unproved_ceiling_and_reversed_fractional_motion",
            tests::crossing_refuses_unproved_ceiling_and_reversed_fractional_motion as fn(),
        ),
        (
            "crossing_refuses_unowned_input_and_foreign_geometry_or_motion",
            tests::crossing_refuses_unowned_input_and_foreign_geometry_or_motion as fn(),
        ),
        (
            "upper_transfer_releases_then_fresh_jumps_and_clears_feet_before_partial_contact",
            tests::upper_transfer_releases_then_fresh_jumps_and_clears_feet_before_partial_contact as fn(),
        ),
        (
            "upper_transfer_refuses_restored_opening_foreign_plm_headroom_input_and_uncleared_feet",
            tests::upper_transfer_refuses_restored_opening_foreign_plm_headroom_input_and_uncleared_feet as fn(),
        ),
        (
            "upper_jump_rejects_nonfresh_spin_and_wrong_radius_before_recording_ownership",
            tests::upper_jump_rejects_nonfresh_spin_and_wrong_radius_before_recording_ownership as fn(),
        ),
        (
            "aerial_reaction_must_be_air_with_cleared_feet_before_transfer",
            tests::aerial_reaction_must_be_air_with_cleared_feet_before_transfer as fn(),
        ),
        (
            "aerial_beam_refuses_invalid_consumed_edges_and_bound_before_reaction",
            tests::aerial_beam_refuses_invalid_consumed_edges_and_bound_before_reaction as fn(),
        ),
        (
            "airborne_survey_requires_neutral_launch_and_actual_jump_before_horizontal_window",
            tests::airborne_survey_requires_neutral_launch_and_actual_jump_before_horizontal_window as fn(),
        ),
        (
            "airborne_survey_refuses_wrong_pose_radius_opening_and_terminal_bound",
            tests::airborne_survey_refuses_wrong_pose_radius_opening_and_terminal_bound as fn(),
        ),
        (
            "airborne_survey_refuses_wrong_previous_launch_pose_and_held_foreign_inputs",
            tests::airborne_survey_refuses_wrong_previous_launch_pose_and_held_foreign_inputs as fn(),
        ),
    ]
}
