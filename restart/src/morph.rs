//! Ordinary-input Morph Ball continuation and an observed pickup checkpoint.
//! Geometry and message behavior use the ticket16 retained C/ROM source packet.
use sm_native_host::State;

const PREFIX: u64 = 18_072;
const TAIL: u64 = 8_192;
const ROOMS: [u16; 5] = [0x92fd, 0x96ba, 0x975c, 0x97b5, 0x9e9f];
const LEFT: u16 = 0x040;
const RIGHT: u16 = 0x080;
const DOWN: u16 = 0x020;

fn word(ram: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([ram[at], ram[at + 1]])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(ram: &mut [u8], at: usize, value: u16) {
        ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
    }
    fn prefix() -> (State, Vec<u8>) {
        let state = State {
            host_steps: PREFIX,
            game_state: 8,
            loading_game_state: 5,
            area: 0,
            room: ROOMS[0],
            x: 1211,
            y: 139,
            polls: 17086,
            updates: 17086,
            nmis: 18072,
            waits: 986,
            requested_buttons: 0x41,
            effective_buttons: 0x41,
            joy1_current: 0x8200,
            ..State::default()
        };
        let mut ram = vec![0; 0x20000];
        set(&mut ram, 0x9c2, 99);
        set(&mut ram, 0xb00, 21);
        for (at, native) in [(0x9b6, 1_u16), (0x9b4, 0x100), (0x9b2, 0x200)] {
            set(&mut ram, at, native.reverse_bits());
        }
        (state, ram)
    }
    fn advance(state: &mut State) {
        state.host_steps += 1;
        state.nmis += 1;
        state.polls += 1;
        state.updates += 1;
    }
    fn route(morph: &mut Morph) -> (State, Vec<u8>) {
        let (mut state, ram) = prefix();
        morph.observe(&state, &ram);
        for (stage, room) in ROOMS.iter().enumerate().skip(1) {
            advance(&mut state);
            state.room = *room;
            state.area = u16::from(stage == 4);
            morph.observe(&state, &ram);
        }
        (state, ram)
    }
    fn pickup(morph: &mut Morph, state: &mut State, ram: &mut [u8]) {
        advance(state);
        set(ram, 0x9a2, 4);
        set(ram, 0x9a4, 4);
        ram[0xd873] = 4;
        set(ram, 0xdc8, 9);
        set(ram, 0x1c1f, 9);
        ram[0x614] = 2;
        morph.observe(state, ram);
        state.host_steps += 1;
        state.nmis += 1;
        state.waits += 1;
        morph.observe(state, ram);
    }
    fn close(morph: &mut Morph, state: &mut State, ram: &mut [u8]) {
        advance(state);
        set(ram, 0xdc8, 0);
        ram[0x614] = 0;
        // message_box_index remains09 after DisplayMessageBox_Async returns.
        morph.observe(state, ram);
    }
    fn response(morph: &mut Morph, state: &mut State, ram: &[u8]) {
        for _ in 0..8 {
            advance(state);
            state.x = state.x.saturating_sub(2);
            state.requested_buttons = LEFT;
            state.effective_buttons = LEFT;
            state.joy1_current = LEFT.reverse_bits();
            morph.observe(state, ram);
        }
    }

    #[test]
    fn complete_requires_ordered_pickup_message_end_and_consumed_movement() {
        let mut morph = Morph::default();
        let (mut state, mut ram) = route(&mut morph);
        pickup(&mut morph, &mut state, &mut ram);
        assert!(!morph.complete());
        close(&mut morph, &mut state, &mut ram);
        assert!(!morph.complete());
        response(&mut morph, &mut state, &ram);
        assert!(morph.complete());
        let mut report = sm_binary_evidence::Map::new();
        morph.report(&mut report);
        assert_eq!(report["morph_response_polls"].as_integer(), Some(8));
        assert_eq!(report["morph_response_left_pixels"].as_integer(), Some(16));
        assert_eq!(report["morph_message_waits"].as_integer(), Some(1));
    }

    #[test]
    fn changed_prefix_existing_item_and_skipped_room_fail_closed() {
        for variant in 0..5 {
            let mut morph = Morph::default();
            let (mut state, mut ram) = prefix();
            match variant {
                0 => state.x += 1,
                1 => set(&mut ram, 0x9a2, 4),
                2 => set(&mut ram, 0x9a4, 4),
                3 => ram[0xd873] = 4,
                _ => state.host_steps -= 1,
            }
            morph.observe(&state, &ram);
            assert!(!morph.prefix_verified);
            assert_eq!(morph.buttons(&state, &ram), 0);
        }
        let mut morph = Morph::default();
        let (mut state, ram) = prefix();
        morph.observe(&state, &ram);
        advance(&mut state);
        state.room = ROOMS[2];
        morph.observe(&state, &ram);
        assert!(morph.invalid);
        assert_eq!(morph.buttons(&state, &ram), 0);
    }

    #[test]
    fn all_three_clear_to_set_transitions_must_occur_in_morph_room() {
        for missing in 0..3 {
            let mut morph = Morph::default();
            let (mut state, mut ram) = route(&mut morph);
            pickup(&mut morph, &mut state, &mut ram);
            match missing {
                0 => set(&mut ram, 0x9a2, 0),
                1 => set(&mut ram, 0x9a4, 0),
                _ => ram[0xd873] = 0,
            }
            close(&mut morph, &mut state, &mut ram);
            response(&mut morph, &mut state, &ram);
            assert!(!morph.complete());
        }
        let mut morph = Morph::default();
        let (mut state, mut ram) = prefix();
        morph.observe(&state, &ram);
        advance(&mut state);
        set(&mut ram, 0x9a2, 4);
        morph.observe(&state, &ram);
        assert!(morph.invalid);
    }

    #[test]
    fn state8_and_inventory_do_not_substitute_for_presentation_completion() {
        for variant in 0..5 {
            let mut morph = Morph::default();
            let (mut state, mut ram) = route(&mut morph);
            if variant == 0 {
                advance(&mut state);
                set(&mut ram, 0x9a2, 4);
                set(&mut ram, 0x9a4, 4);
                ram[0xd873] = 4;
                morph.observe(&state, &ram);
            } else {
                pickup(&mut morph, &mut state, &mut ram);
                match variant {
                    1 => {}
                    2 => {
                        set(&mut ram, 0xdc8, 0);
                    }
                    3 => {
                        ram[0x614] = 0;
                    }
                    _ => {
                        set(&mut ram, 0xdc8, 0);
                        ram[0x614] = 0;
                        set(&mut ram, 0xa78, 1);
                    }
                }
                advance(&mut state);
                morph.observe(&state, &ram);
            }
            response(&mut morph, &mut state, &ram);
            assert!(!morph.complete(), "presentation variant {variant}");
        }
    }

    #[test]
    fn fresh_polls_updates_input_and_positive_health_are_required_after_message() {
        for variant in 0..6 {
            let mut morph = Morph::default();
            let (mut state, mut ram) = route(&mut morph);
            pickup(&mut morph, &mut state, &mut ram);
            close(&mut morph, &mut state, &mut ram);
            for _ in 0..8 {
                advance(&mut state);
                state.requested_buttons = LEFT;
                state.effective_buttons = LEFT;
                state.joy1_current = LEFT.reverse_bits();
                if variant != 0 {
                    state.x -= 2;
                }
                match variant {
                    1 => state.polls -= 1,
                    2 => state.updates -= 1,
                    3 => state.effective_buttons = 0,
                    4 => state.joy1_current = 0,
                    5 => set(&mut ram, 0x9c2, 0),
                    _ => {}
                }
                morph.observe(&state, &ram);
            }
            assert!(!morph.complete(), "response variant {variant}");
        }
    }

    #[test]
    fn mapped_message_pulses_advance_while_game_updates_are_frozen() {
        let mut morph = Morph::default();
        let (mut state, mut ram) = route(&mut morph);
        pickup(&mut morph, &mut state, &mut ram);
        state.host_steps = 18112;
        let updates = state.updates;
        assert_eq!(morph.buttons(&state, &ram), 0x100);
        state.host_steps += 4;
        assert_eq!(morph.buttons(&state, &ram), 0);
        assert_eq!(state.updates, updates);
        set(&mut ram, 0x9b4, 0x0002_u16.reverse_bits());
        state.host_steps += 12;
        assert_eq!(morph.buttons(&state, &ram), 2);
    }

    #[test]
    fn elevator_down_edges_and_descent_detour_are_observation_based() {
        let mut morph = Morph::default();
        let (mut state, mut ram) = prefix();
        morph.observe(&state, &ram);
        state.x = 384;
        state.y = 300;
        morph.stalled_polls = 24;
        assert_eq!(morph.descent_target(&state, 384), 288);
        state.x -= 6;
        morph.stalled_polls = 0;
        assert_eq!(morph.descent_target(&state, 384), 288, "detour must not reverse after its first moving poll");
        state.y += 16;
        assert_eq!(morph.descent_target(&state, 384), 384);
        morph.stage = 3;
        state.room = ROOMS[3];
        state.x = 128;
        state.y = 136;
        state.host_steps = 18112;
        // Down is admitted by observed native pseudo-door contact, not by
        // treating the elevator's eventual snapped position as an entry gate.
        set(&mut ram, 0xe16, 1);
        assert_eq!(morph.buttons(&state, &ram), DOWN);
        state.host_steps += 4;
        assert_eq!(morph.buttons(&state, &ram), 0);
        state.host_steps = PREFIX + TAIL;
        assert_eq!(morph.buttons(&state, &ram), 0);
    }

    #[test]
    fn crouched_elevator_edge_restores_pose_then_requires_observed_contact() {
        let (mut state, mut ram) = prefix();
        let mut morph = Morph { prefix_verified: true, stage: 3, ..Morph::default() };
        state.host_steps = 22112;
        state.room = ROOMS[3];
        state.x = 121;
        state.y = 144;
        set(&mut ram, 0xa1c, 0x28);
        set(&mut ram, 0xb00, 16);
        // author02 stopped here, crouched over the pseudo-door tiles, with
        // elevator flags/status both zero. Right/run cannot activate it.
        assert_eq!(morph.buttons(&state, &ram), 0x010, "restore crouch using ordinary Up");
        state.y = 139;
        set(&mut ram, 0xb00, 21);
        set(&mut ram, 0xa1c, 2);
        assert_eq!(morph.buttons(&state, &ram), 0, "standing alone does not establish native contact");
        set(&mut ram, 0xe16, 1);
        assert_eq!(morph.buttons(&state, &ram), DOWN, "native contact permits fresh Down even atx121");
        state.host_steps += 4;
        assert_eq!(morph.buttons(&state, &ram), 0, "release before another Down edge");
        state.host_steps += 12;
        set(&mut ram, 0xe18, 1);
        assert_eq!(morph.buttons(&state, &ram), 0, "native elevator owns active motion");
    }

    #[test]
    fn grounded_climb_detour_clears_lip_with_bounded_jump_and_release() {
        let (mut state, mut ram) = prefix();
        let mut morph = Morph::default();
        morph.observe(&state, &ram);
        advance(&mut state);
        state.room = ROOMS[1];
        state.x = 395;
        state.y = 107;
        morph.observe(&state, &ram);
        morph.descent_detour = Some(480);
        morph.detour_y = 107;
        // Reproduce observed, consumed right input against the same low lip.
        for _ in 0..12 {
            advance(&mut state);
            state.requested_buttons = RIGHT | 1;
            state.effective_buttons = RIGHT | 1;
            state.joy1_current = (RIGHT | 1).reverse_bits();
            morph.observe(&state, &ram);
        }
        let mut airborne = Morph {
            prefix_verified: true,
            stage: 1,
            stalled_polls: 12,
            descent_detour: Some(480),
            detour_y: 107,
            ..Morph::default()
        };
        set(&mut ram, 0xb2e, 1);
        assert_eq!(airborne.buttons(&state, &ram) & 0x100, 0, "airborne descent cannot start a new jump");
        set(&mut ram, 0xb2e, 0);
        assert_eq!(
            morph.buttons(&state, &ram) & (LEFT | RIGHT | 0x100),
            RIGHT | 0x100,
            "grounded rightward detour must jump over the reached Climb lip"
        );
        let start = state.updates;
        state.updates = start + 23;
        assert_ne!(morph.buttons(&state, &ram) & 0x100, 0);
        state.updates = start + 24;
        assert_eq!(morph.buttons(&state, &ram) & 0x100, 0, "held jump must release");
        state.updates = start + 59;
        assert_eq!(morph.buttons(&state, &ram) & 0x100, 0, "release must persist before another attempt");
    }
}
fn bits(ram: &[u8]) -> [bool; 3] {
    [word(ram, 0x9a2) & 4 != 0, word(ram, 0x9a4) & 4 != 0, ram[0xd873] & 4 != 0]
}
fn item_room(state: &State) -> bool {
    state.game_state == 8 && state.area == 1 && state.room == ROOMS[4]
}
fn message(ram: &[u8]) -> bool {
    word(ram, 0xdc8) == 9 && word(ram, 0x1c1f) == 9 && ram[0x614] != 0
}
fn presentation_finished(ram: &[u8]) -> bool {
    word(ram, 0xdc8) == 0 && ram[0x614] == 0 && word(ram, 0xa78) == 0
}
fn toward(x: u16, target: u16) -> u16 {
    if x.abs_diff(target) <= 4 {
        0
    } else if x < target {
        RIGHT
    } else {
        LEFT
    }
}
fn grounded(ram: &[u8]) -> bool {
    word(ram, 0xb2e) == 0 && word(ram, 0xb2c) == 0 && word(ram, 0xb36) == 0 && word(ram, 0xb00) != 0
}

#[derive(Default)]
struct Response {
    start: u64,
    end: u64,
    polls: u64,
    moving: u64,
    pixels: u16,
}

#[derive(Default)]
pub struct Morph {
    previous: Option<State>,
    prefix_verified: bool,
    invalid: bool,
    stage: usize,
    room_steps: [Option<u64>; 5],
    pickup_bits: [bool; 3],
    bit_steps: [Option<u64>; 3],
    acquired_step: Option<u64>,
    message_step: Option<u64>,
    message_waits: u64,
    presentation_finished_step: Option<u64>,
    checkpoint_step: Option<u64>,
    response: Response,
    health: u16,
    stalled_polls: u64,
    jump_until: u64,
    jump_ready: u64,
    elevator_arrived: bool,
    presentation_clear: bool,
    descent_detour: Option<u16>,
    detour_y: u16,
    detour_right: bool,
}

impl Morph {
    pub fn observe(&mut self, state: &State, ram: &[u8]) {
        if ram.len() != 0x20000 {
            self.invalid = true;
            return;
        }
        if let Some(previous) = self.previous
            && (state.host_steps != previous.host_steps + 1
                || state.polls < previous.polls
                || state.updates < previous.updates)
        {
            self.invalid = true;
        }
        self.health = word(ram, 0x9c2);
        self.presentation_clear = presentation_finished(ram);
        if state.host_steps == PREFIX {
            self.prefix_verified = state.game_state == 8
                && state.loading_game_state == 5
                && state.area == 0
                && state.room == ROOMS[0]
                && (state.x, state.y, state.timer) == (1211, 139, 0)
                && (state.polls, state.updates, state.nmis, state.waits) == (17086, 17086, 18072, 986)
                && (state.requested_buttons, state.effective_buttons, state.joy1_current, state.joy1_new)
                    == (0x41, 0x41, 0x8200, 0)
                && state.cpu_fallback_calls == 0
                && state.cpu_instructions == 0
                && self.health == 99
                && bits(ram) == [false; 3];
            if self.prefix_verified {
                self.room_steps[0] = Some(PREFIX);
            }
        }
        if self.prefix_verified && state.host_steps > PREFIX {
            if state.host_steps > PREFIX + TAIL
                || self.health == 0
                || state.cpu_fallback_calls != 0
                || state.cpu_instructions != 0
            {
                self.invalid = true;
            }
            if state.game_state == 8 {
                let room = ROOMS.iter().position(|room| *room == state.room);
                match room {
                    Some(stage) if stage == self.stage || stage == self.stage + 1 => {
                        if state.area != u16::from(stage == 4) {
                            self.invalid = true;
                        }
                        if stage != self.stage {
                            self.stage = stage;
                            self.room_steps[stage] = Some(state.host_steps);
                            self.stalled_polls = 0;
                            self.jump_until = 0;
                            self.jump_ready = 0;
                            self.descent_detour = None;
                        }
                    }
                    _ => self.invalid = true,
                }
            }
            let now = bits(ram);
            for (i, set) in now.into_iter().enumerate() {
                if self.pickup_bits[i] && !set {
                    self.invalid = true;
                }
                if set && !self.pickup_bits[i] {
                    if self.stage != 4 || !item_room(state) {
                        self.invalid = true;
                    }
                    self.bit_steps[i] = Some(state.host_steps);
                }
            }
            self.pickup_bits = now;
            if now == [true; 3] && self.stage == 4 && item_room(state) {
                self.acquired_step.get_or_insert(state.host_steps);
            }
            if self.acquired_step.is_some() && item_room(state) && message(ram) {
                self.message_step.get_or_insert(state.host_steps);
                if self.previous.is_some_and(|previous| previous.updates == state.updates) {
                    self.message_waits += 1;
                }
            }
            if self.message_step.is_some()
                && self.message_waits > 0
                && item_room(state)
                && self.pickup_bits == [true; 3]
                && presentation_finished(ram)
            {
                self.presentation_finished_step.get_or_insert(state.host_steps);
            }
            if item_room(state) && (state.x, state.y) == (1408, 680) {
                self.elevator_arrived = true;
            }
            if let Some(previous) = self.previous {
                self.observe_response(&previous, state, ram);
                if state.polls > previous.polls {
                    if previous.room == state.room
                        && previous.game_state == 8
                        && state.game_state == 8
                        && (previous.x, previous.y) == (state.x, state.y)
                    {
                        self.stalled_polls += 1;
                    } else {
                        self.stalled_polls = 0;
                    }
                }
            }
        }
        self.previous = Some(*state);
    }

    fn observe_response(&mut self, previous: &State, state: &State, ram: &[u8]) {
        if self.checkpoint_step.is_some() {
            return;
        }
        if self.presentation_finished_step.is_none_or(|step| previous.host_steps < step)
            || !item_room(previous)
            || !item_room(state)
            || !presentation_finished(ram)
            || self.pickup_bits != [true; 3]
            || self.health == 0
        {
            self.response = Response::default();
            return;
        }
        if state.polls == previous.polls {
            return;
        }
        let pixels = previous.x.checked_sub(state.x);
        if state.polls != previous.polls + 1
            || state.updates != previous.updates + 1
            || state.requested_buttons & (LEFT | RIGHT) != LEFT
            || state.effective_buttons & (LEFT | RIGHT) != LEFT
            || state.joy1_current & (LEFT | RIGHT).reverse_bits() != LEFT.reverse_bits()
            || pixels.is_none_or(|n| n > 16)
        {
            self.response = Response::default();
            return;
        }
        if self.response.polls == 0 || state.host_steps - self.response.start > 64 {
            self.response = Response { start: previous.host_steps, ..Response::default() };
        }
        let pixels = pixels.unwrap();
        self.response.end = state.host_steps;
        self.response.polls += 1;
        self.response.moving += u64::from(pixels != 0);
        self.response.pixels += pixels;
        if self.response.polls >= 8 && self.response.moving >= 4 && self.response.pixels >= 16 {
            self.checkpoint_step = Some(state.host_steps);
        }
    }

    pub fn buttons(&mut self, state: &State, ram: &[u8]) -> u16 {
        if !self.prefix_verified
            || self.invalid
            || self.complete()
            || ram.len() != 0x20000
            || !(PREFIX..PREFIX + TAIL).contains(&state.host_steps)
            || word(ram, 0x9c2) == 0
        {
            return 0;
        }
        let [run, jump, shoot] = [0x9b6, 0x9b4, 0x9b2].map(|at| word(ram, at).reverse_bits());
        if [run, jump, shoot].iter().any(|pad| !pad.is_power_of_two() || pad & !0x0f0f != 0)
            || run == jump
            || run == shoot
            || jump == shoot
        {
            return 0;
        }
        // Message09 waits360 NMI-only iterations, then reads new/held keys.
        // Use host-step pulses, since ordinary update counts freeze in this wait.
        if self.acquired_step.is_some() && message(ram) {
            return if state.host_steps % 16 < 4 { jump } else { 0 };
        }
        if state.game_state != 8 || state.room != ROOMS[self.stage] {
            return 0;
        }
        if self.acquired_step.is_some() {
            return if self.presentation_finished_step.is_some() && presentation_finished(ram) {
                LEFT | run
            } else {
                0
            };
        }
        let mut allow_jump = true;
        let direction = match self.stage {
            0 => {
                // South cap spans four blocks from352: its center is384.
                if state.y >= 1160 && state.x.abs_diff(384) <= 24 {
                    if grounded(ram) && self.stalled_polls >= 12 {
                        self.start_jump(state.updates);
                    }
                    DOWN
                } else {
                    let target = self.descent_target(state, 384);
                    allow_jump = state.x.abs_diff(384) > 96;
                    toward(state.x, target)
                }
            }
            1 => {
                if state.y >= 2080 {
                    RIGHT
                } else {
                    allow_jump = self.descent_detour.is_some();
                    toward(state.x, self.descent_target(state, 384))
                }
            }
            2 => RIGHT,
            3 => {
                // The snapped128/136 position is an activation result. Actual
                // admission is standing pseudo-door contact published after
                // movement (sm94), consumed by next update's EnemyMain (sm82).
                if word(ram, 0xe18) != 0 {
                    return 0;
                }
                let standing = word(ram, 0xa1c) < 9;
                if standing && word(ram, 0xe16) != 0 {
                    return if state.host_steps % 16 < 4 { DOWN } else { 0 };
                }
                // Room97B5 pseudo-door tiles occupy x112..143 at y160.
                // Restore crouch with Up; release movement while waiting for
                // native contact. Crouched y144/radius16 is not standing136.
                let feet = u32::from(state.y) + u32::from(word(ram, 0xb00));
                if (112..144).contains(&state.x) && feet.abs_diff(160) <= 4 && grounded(ram) {
                    return if standing { 0 } else { 0x010 };
                }
                // Approach without run to avoid overshooting this small pad.
                return toward(state.x, 128);
            }
            4 => {
                if !self.elevator_arrived {
                    return 0;
                }
                if state.x.abs_diff(1112) <= 8 && grounded(ram) && self.stalled_polls >= 8 {
                    self.start_jump(state.updates);
                }
                toward(state.x, 1112)
            }
            _ => return 0,
        };
        if allow_jump && direction != 0 && grounded(ram) && self.stalled_polls >= 12 {
            self.start_jump(state.updates);
        }
        let mut pad = direction;
        if direction & (LEFT | RIGHT) != 0 {
            pad |= run;
        }
        if self.stage <= 2 && state.updates % 12 < 2 {
            pad |= shoot;
        }
        if state.updates < self.jump_until {
            pad |= jump;
        }
        pad
    }

    fn descent_target(&mut self, state: &State, center: u16) -> u16 {
        if let Some(target) = self.descent_detour {
            if state.x.abs_diff(target) <= 4 || state.y.saturating_sub(self.detour_y) >= 16 {
                self.descent_detour = None;
            } else {
                return target;
            }
        }
        if self.stalled_polls >= 24 {
            let target = if self.detour_right { center + 96 } else { center - 96 };
            self.detour_right = !self.detour_right;
            self.detour_y = state.y;
            self.descent_detour = Some(target);
            target
        } else {
            center
        }
    }
    fn start_jump(&mut self, update: u64) {
        if update >= self.jump_ready {
            self.jump_until = update + 24;
            self.jump_ready = update + 60;
        }
    }
    pub fn complete(&self) -> bool {
        self.prefix_verified
            && !self.invalid
            && self.stage == 4
            && self.room_steps.iter().all(Option::is_some)
            && self.bit_steps.iter().all(Option::is_some)
            && self.pickup_bits == [true; 3]
            && self.acquired_step.is_some()
            && self.message_step.is_some()
            && self.message_waits > 0
            && self.presentation_finished_step.is_some()
            && self.checkpoint_step.is_some()
            && self.health > 0
            && self.presentation_clear
            && self.previous.as_ref().is_some_and(item_room)
    }
    pub fn report(&self, report: &mut sm_binary_evidence::Map) {
        for (key, value) in [
            ("morph_checkpoint", self.complete()),
            ("morph_prefix_verified", self.prefix_verified),
            ("morph_invalid", self.invalid),
            ("morph_elevator_arrived", self.elevator_arrived),
        ] {
            report.insert(key.into(), value.into());
        }
        for (key, value) in [
            ("morph_stage", self.stage as u64),
            ("morph_message_waits", self.message_waits),
            ("morph_health", u64::from(self.health)),
            ("morph_response_polls", self.response.polls),
            ("morph_response_moving_polls", self.response.moving),
            ("morph_response_left_pixels", u64::from(self.response.pixels)),
            ("morph_response_first_step", self.response.start),
            ("morph_response_last_step", self.response.end),
        ] {
            report.insert(key.into(), (value as i64).into());
        }
        for (key, value) in [
            ("morph_acquired_step", self.acquired_step),
            ("morph_message_step", self.message_step),
            ("morph_presentation_finished_step", self.presentation_finished_step),
            ("morph_checkpoint_step", self.checkpoint_step),
        ] {
            report.insert(key.into(), value.map_or(-1, |n| n as i64).into());
        }
        for (key, values) in
            [("morph_room_steps", self.room_steps.as_slice()), ("morph_bit_steps", self.bit_steps.as_slice())]
        {
            report.insert(
                key.into(),
                values
                    .iter()
                    .map(|n| sm_binary_evidence::Value::Integer(n.map_or(-1, |n| n as i64)))
                    .collect::<Vec<_>>()
                    .into(),
            );
        }
    }
}
