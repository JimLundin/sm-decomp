//! Observation-only, bounded ordinary-input continuation from Landing to Parlor.
use sm_native_host::State;

const PREFIX: u64 = 16_991;
const TAIL: u64 = 4_096;
const LANDING: u16 = 0x91f8;
const PARLOR: u16 = 0x92fd;
const LEFT: u16 = 0x040;
const RIGHT: u16 = 0x080;
const RESPONSE_POLLS: u64 = 8;
const RESPONSE_PIXELS: u16 = 16;

fn normal(state: &State, room: u16) -> bool {
    state.game_state == 8 && state.area == 0 && state.room == room
}
fn landing_prefix(state: &State) -> bool {
    state.host_steps == PREFIX
        && normal(state, LANDING)
        && state.loading_game_state == 0x22
        && (state.x, state.y, state.timer) == (1152, 263, 0)
        && (state.polls, state.updates, state.nmis, state.waits) == (16068, 16068, 16991, 923)
        && state.cpu_fallback_calls == 0
        && state.cpu_instructions == 0
        && (state.requested_buttons, state.effective_buttons, state.joy1_current, state.joy1_new) == (0, 0, 0, 0)
}
fn left_input(state: &State) -> bool {
    state.requested_buttons & (LEFT | RIGHT) == LEFT
        && state.effective_buttons & (LEFT | RIGHT) == LEFT
        && state.joy1_current & (LEFT | RIGHT).reverse_bits() == LEFT.reverse_bits()
}

#[derive(Default)]
struct Response {
    first_step: u64,
    last_step: u64,
    polls: u64,
    moving_polls: u64,
    pixels: u16,
}

#[derive(Default)]
pub struct Parlor {
    previous: Option<State>,
    prefix_verified: bool,
    invalid_sequence: bool,
    departure_step: Option<u64>,
    resumed_step: Option<u64>,
    response: Response,
    checkpoint_step: Option<u64>,
    landing_stalled_polls: u64,
    jump_until_update: u64,
    jump_ready_update: u64,
    unsupported_room: Option<u16>,
}

impl Parlor {
    pub fn observe(&mut self, state: &State) {
        if let Some(previous) = self.previous
            && (state.host_steps != previous.host_steps + 1
                || state.polls < previous.polls
                || state.updates < previous.updates)
        {
            self.invalid_sequence = true;
        }
        if state.host_steps == PREFIX {
            self.prefix_verified = landing_prefix(state);
        }
        if state.host_steps > PREFIX && self.prefix_verified {
            if state.host_steps > PREFIX + TAIL || state.cpu_fallback_calls != 0 || state.cpu_instructions != 0 {
                self.invalid_sequence = true;
            }
            if matches!(state.game_state, 9 | 11)
                && state.area == 0
                && self.previous.as_ref().is_some_and(|previous| normal(previous, LANDING))
            {
                self.departure_step.get_or_insert(state.host_steps);
            }
            if normal(state, PARLOR) && self.departure_step.is_some() {
                self.resumed_step.get_or_insert(state.host_steps);
            }
            if state.game_state == 8 && state.area == 0 && ![LANDING, PARLOR].contains(&state.room) {
                self.unsupported_room.get_or_insert(state.room);
            }
            if let Some(previous) = self.previous {
                self.observe_response(&previous, state);
                if normal(&previous, LANDING) && normal(state, LANDING) && state.polls > previous.polls {
                    if left_input(state) && state.x == previous.x {
                        self.landing_stalled_polls += 1;
                    } else {
                        self.landing_stalled_polls = 0;
                    }
                }
            }
        }
        self.previous = Some(*state);
    }

    fn observe_response(&mut self, previous: &State, state: &State) {
        if self.checkpoint_step.is_some() {
            return;
        }
        if self.resumed_step.is_none() || !normal(previous, PARLOR) || !normal(state, PARLOR) {
            self.response = Response::default();
            return;
        }
        if state.polls == previous.polls {
            return;
        }
        // A new poll and update must actually consume left input. Transition
        // placement, rightward motion, coordinate wrap and a large jump in X
        // cannot supply the short ordinary-control interval.
        let left_pixels = previous.x.checked_sub(state.x);
        if state.polls != previous.polls + 1
            || state.updates != previous.updates + 1
            || !left_input(state)
            || left_pixels.is_none_or(|pixels| pixels > 16)
        {
            self.response = Response::default();
            return;
        }
        if self.response.polls == 0 || state.host_steps - self.response.first_step > 64 {
            self.response = Response { first_step: previous.host_steps, ..Response::default() };
        }
        let pixels = left_pixels.unwrap();
        self.response.polls += 1;
        self.response.moving_polls += u64::from(pixels != 0);
        self.response.pixels += pixels;
        self.response.last_step = state.host_steps;
        if self.response.polls >= RESPONSE_POLLS
            && self.response.moving_polls >= 4
            && self.response.pixels >= RESPONSE_PIXELS
        {
            self.checkpoint_step = Some(state.host_steps);
        }
    }

    /// Return an ordinary native pad mask only. Startup ship control may still
    /// be locked at state8; decisions use observed movement and grounding.
    pub fn buttons(&mut self, state: &State, ram: &[u8]) -> u16 {
        if !self.prefix_verified
            || self.invalid_sequence
            || self.unsupported_room.is_some()
            || !(PREFIX..PREFIX + TAIL).contains(&state.host_steps)
            || ram.len() != 0x20000
            || state.area != 0
            || state.game_state != 8
            || ![LANDING, PARLOR].contains(&state.room)
            || self.complete()
        {
            return 0;
        }
        let word = |at| u16::from_le_bytes([ram[at], ram[at + 1]]);
        let [run, jump, shoot] = [0x9b6, 0x9b4, 0x9b2].map(|at| word(at).reverse_bits());
        if [run, jump, shoot].iter().any(|pad| !pad.is_power_of_two() || pad & !0x0f3f != 0)
            || run == jump
            || run == shoot
            || jump == shoot
        {
            return 0;
        }
        let mut pad = LEFT | run;
        if state.room == PARLOR {
            return pad;
        }
        // Brief repeated shooting edges open the ordinary blue door without
        // assuming its animation duration. Jump only after observed obstruction,
        // away from the door, with an explicit release between attempts.
        if state.updates % 12 < 2 {
            pad |= shoot;
        }
        let grounded = word(0xb2e) == 0 && word(0xb2c) == 0 && word(0xb36) == 0 && word(0xb00) != 0;
        if grounded && state.x >= 64 && self.landing_stalled_polls >= 12 && state.updates >= self.jump_ready_update {
            self.jump_until_update = state.updates + 24;
            self.jump_ready_update = state.updates + 60;
        }
        if state.updates < self.jump_until_update {
            pad |= jump;
        }
        pad
    }

    pub fn complete(&self) -> bool {
        self.prefix_verified
            && !self.invalid_sequence
            && self.unsupported_room.is_none()
            && self.departure_step.is_some()
            && self.resumed_step.is_some()
            && self.checkpoint_step.is_some()
            && self.previous.as_ref().is_some_and(|state| normal(state, PARLOR))
    }

    pub fn report(&self, report: &mut sm_binary_evidence::Map) {
        for (key, value) in [
            ("parlor_checkpoint", self.complete()),
            ("parlor_prefix_verified", self.prefix_verified),
            ("parlor_invalid_sequence", self.invalid_sequence),
        ] {
            report.insert(key.into(), value.into());
        }
        for (key, value) in [
            ("parlor_departure_step", self.departure_step),
            ("parlor_resumed_step", self.resumed_step),
            ("parlor_checkpoint_step", self.checkpoint_step),
        ] {
            report.insert(key.into(), value.map_or(-1, |n| n as i64).into());
        }
        for (key, value) in [
            ("parlor_response_first_step", self.response.first_step),
            ("parlor_response_last_step", self.response.last_step),
            ("parlor_response_polls", self.response.polls),
            ("parlor_response_moving_polls", self.response.moving_polls),
            ("parlor_response_left_pixels", u64::from(self.response.pixels)),
        ] {
            report.insert(key.into(), (value as i64).into());
        }
        report.insert("parlor_unsupported_room".into(), self.unsupported_room.map_or(-1, i64::from).into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prefix() -> State {
        State {
            host_steps: PREFIX,
            game_state: 8,
            loading_game_state: 0x22,
            area: 0,
            room: LANDING,
            x: 1152,
            y: 263,
            polls: 16068,
            updates: 16068,
            nmis: 16991,
            waits: 923,
            ..State::default()
        }
    }
    fn advance(mut state: State) -> State {
        state.host_steps += 1;
        state.nmis += 1;
        state.polls += 1;
        state.updates += 1;
        state
    }
    fn left(mut state: State) -> State {
        state.requested_buttons = LEFT;
        state.effective_buttons = LEFT;
        state.joy1_current = LEFT.reverse_bits();
        state
    }
    fn resume(parlor: &mut Parlor) -> State {
        let mut state = prefix();
        parlor.observe(&state);
        state = advance(state);
        state.game_state = 9;
        parlor.observe(&state);
        state = advance(state);
        state.room = PARLOR;
        state.game_state = 11;
        parlor.observe(&state);
        assert!(!parlor.complete());
        state = advance(state);
        state.game_state = 8;
        state.x = 240;
        parlor.observe(&state);
        assert!(!parlor.complete());
        state
    }
    fn move_left(parlor: &mut Parlor, mut state: State) -> State {
        for _ in 0..RESPONSE_POLLS {
            state = left(advance(state));
            state.x -= 2;
            parlor.observe(&state);
        }
        state
    }

    #[test]
    fn normal_resume_then_short_consumed_left_interval_completes() {
        let mut parlor = Parlor::default();
        let state = resume(&mut parlor);
        move_left(&mut parlor, state);
        assert!(parlor.complete());
        let mut report = sm_binary_evidence::Map::new();
        parlor.report(&mut report);
        assert_eq!(report["parlor_response_polls"].as_integer(), Some(8));
        assert_eq!(report["parlor_response_left_pixels"].as_integer(), Some(16));
        assert_eq!(report["parlor_checkpoint"].as_bool(), Some(true));
    }

    #[test]
    fn exact_landing_prefix_and_departure_are_required() {
        for change in 0..7 {
            let mut parlor = Parlor::default();
            let mut state = prefix();
            match change {
                0 => state.host_steps -= 1,
                1 => state.x += 1,
                2 => state.room = PARLOR,
                3 => state.loading_game_state = 8,
                4 => state.polls -= 1,
                5 => state.cpu_fallback_calls = 1,
                _ => state.requested_buttons = LEFT,
            }
            parlor.observe(&state);
            state.room = PARLOR;
            move_left(&mut parlor, state);
            assert!(!parlor.complete());
            assert!(!parlor.prefix_verified);
        }
        let mut parlor = Parlor::default();
        let mut state = prefix();
        parlor.observe(&state);
        state = advance(state);
        state.room = PARLOR;
        parlor.observe(&state);
        move_left(&mut parlor, state);
        assert!(!parlor.complete(), "room change alone lacks observed departure");
    }

    #[test]
    fn loading_stationary_unpolled_wrong_input_and_teleport_do_not_prove_response() {
        for change in 0..8 {
            let mut parlor = Parlor::default();
            let mut state = resume(&mut parlor);
            for _ in 0..16 {
                let before = state;
                state = left(advance(state));
                state.x = state.x.saturating_sub(2);
                match change {
                    0 => state.game_state = 11,
                    1 => state.x = before.x,
                    2 => {
                        state.polls = before.polls;
                        state.updates = before.updates;
                    }
                    3 => state.requested_buttons = 0,
                    4 => state.effective_buttons = 0,
                    5 => state.joy1_current = 0,
                    6 => state.requested_buttons |= RIGHT,
                    _ => state.x = before.x.saturating_sub(20),
                }
                parlor.observe(&state);
            }
            assert!(!parlor.complete(), "invalid response kind {change}");
        }
    }

    #[test]
    fn response_requires_multiple_moving_polls_and_resets_across_wrong_input() {
        let mut parlor = Parlor::default();
        let mut state = resume(&mut parlor);
        for i in 0..8 {
            state = left(advance(state));
            if i == 0 {
                state.x -= 16;
            }
            parlor.observe(&state);
        }
        assert!(!parlor.complete());
        state = advance(state);
        state.requested_buttons = 0;
        parlor.observe(&state);
        assert_eq!(parlor.response.polls, 0);
        move_left(&mut parlor, state);
        assert!(parlor.complete());
    }

    #[test]
    fn skipped_observations_expired_tail_and_wrong_room_fail_closed() {
        let mut parlor = Parlor::default();
        let state = resume(&mut parlor);
        let mut skipped = state;
        skipped.host_steps += 1;
        move_left(&mut parlor, skipped);
        assert!(!parlor.complete());
        let mut parlor = Parlor::default();
        let mut state = resume(&mut parlor);
        state.host_steps = PREFIX + TAIL + 1;
        parlor.observe(&state);
        assert!(!parlor.complete());
        let mut parlor = Parlor::default();
        let state = resume(&mut parlor);
        let mut state = move_left(&mut parlor, state);
        state = advance(state);
        state.room = LANDING;
        parlor.observe(&state);
        assert!(!parlor.complete());
    }

    fn ram() -> Vec<u8> {
        let mut ram = vec![0; 0x20000];
        for (at, value) in [(0x9b6, 0x0001_u16), (0x9b4, 0x0100), (0x9b2, 0x0200)] {
            ram[at..at + 2].copy_from_slice(&value.reverse_bits().to_le_bytes());
        }
        ram[0xb00] = 21;
        ram
    }

    #[test]
    fn policy_uses_mapped_controls_and_waits_for_observed_grounded_obstruction() {
        let mut parlor = Parlor::default();
        let mut state = prefix();
        let mut ram = ram();
        assert_eq!(parlor.buttons(&state, &ram), 0);
        parlor.observe(&state);
        assert_eq!(parlor.buttons(&state, &ram), LEFT | 1 | 0x200);
        for _ in 0..12 {
            state = left(advance(state));
            parlor.observe(&state);
        }
        ram[0xb00] = 0;
        assert_eq!(parlor.buttons(&state, &ram) & 0x100, 0, "ship lock is not grounding");
        ram[0xb00] = 21;
        assert_ne!(parlor.buttons(&state, &ram) & 0x100, 0);
        state.updates += 24;
        assert_eq!(parlor.buttons(&state, &ram) & 0x100, 0, "jump must release");
        assert_eq!(parlor.buttons(&state, &ram[..32]), 0);
        state.game_state = 11;
        assert_eq!(parlor.buttons(&state, &ram), 0);
    }
}
