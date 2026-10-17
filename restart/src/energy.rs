//! Prefix-preserving ordinary-input route to Terminator's exposed energy tank.
//! Source facts: recovered C578f90b; 8F8432 places EED7/0008 at cell07,2A;
//! 848968 at operandE0B8 refills health to max+100 and queues message1.
//! Reverse climbing/terrain steering below is an unvalidated author candidate.
//! Native runtime owns all gameplay, tape authentication and callback evidence.
use sha2::{Digest, Sha256};
use sm_native_host::State;

const PREFIX: u64 = 24_832;
const TAIL: u64 = 16_384;
const PREFIX_WRAM: &str = "cb3362278173187dcda825bea8c4ec5c06952c1db7a5a948affb1c8d96a97483";
const ROOMS: [u16; 8] = [0xa107, 0x9f11, 0x9e9f, 0x97b5, 0x975c, 0x96ba, 0x92fd, 0x990d];
const LEFT: u16 = 0x40;
const RIGHT: u16 = 0x80;
const DOWN: u16 = 0x20;
const UP: u16 = 0x10;

fn word(ram: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([ram[at], ram[at + 1]])
}
fn normal(state: &State, stage: usize) -> bool {
    state.game_state == 8 && state.room == ROOMS[stage] && state.area == u16::from(stage < 3)
}
fn clear(ram: &[u8]) -> bool {
    word(ram, 0xdc8) == 0 && ram[0x614] == 0 && word(ram, 0xa78) == 0
}
fn message(ram: &[u8]) -> bool {
    word(ram, 0xdc8) == 1 && word(ram, 0x1c1f) == 1 && ram[0x614] != 0
}
fn grounded(ram: &[u8]) -> bool {
    word(ram, 0xb2e) == 0 && word(ram, 0xb2c) == 0 && word(ram, 0xb36) == 0 && word(ram, 0xb00) != 0
}
fn ball(ram: &[u8]) -> bool {
    matches!(ram[0xa1f], 4 | 8 | 0x11..=0x13)
}
fn toward(x: u16, target: u16) -> u16 {
    if x.abs_diff(target) <= 3 {
        0
    } else if x < target {
        RIGHT
    } else {
        LEFT
    }
}
fn controls(ram: &[u8]) -> Option<[u16; 3]> {
    let pads = [0x9b6, 0x9b4, 0x9b2].map(|at| word(ram, at).reverse_bits());
    (pads.iter().all(|pad| pad.is_power_of_two() && pad & !0x0f0f == 0)
        && pads[0] != pads[1]
        && pads[0] != pads[2]
        && pads[1] != pads[2])
        .then_some(pads)
}
fn state_matches_ram(state: &State, ram: &[u8]) -> bool {
    [
        (state.game_state, 0x998),
        (state.loading_game_state, 0xd914),
        (state.area, 0x79f),
        (state.room, 0x79b),
        (state.x, 0xaf6),
        (state.y, 0xafa),
        (state.timer, 0x943),
        (state.joy1_current, 0x8b),
        (state.joy1_new, 0x8f),
    ]
    .into_iter()
    .all(|(value, at)| word(ram, at) == value)
}
fn inventory(ram: &[u8]) -> bool {
    word(ram, 0x9a2) == 4
        && word(ram, 0x9a4) == 4
        && word(ram, 0x9a6) == 0
        && word(ram, 0x9a8) == 0
        && word(ram, 0x9c8) == 5
        && word(ram, 0x9c6) <= 5
        && [0x9c0, 0x9ca, 0x9cc, 0x9ce, 0x9d0, 0x9d4, 0x9d6, 0x9d8].into_iter().all(|at| word(ram, at) == 0)
}

#[derive(Clone, Copy)]
struct Tank {
    slot: usize,
    ip: u16,
    pre: u16,
    draw: u16,
    timer: u16,
}
impl Tank {
    fn read(ram: &[u8]) -> Option<Self> {
        let mut slots = (0..40).filter(|i| word(ram, 0x1c37 + 2 * i) == 0xeed7 && word(ram, 0x1dc7 + 2 * i) == 8);
        let slot = slots.next()?;
        if slots.next().is_some()
            || word(ram, 0x1c87 + 2 * slot) != 0x1f8e
            || word(ram, 0xdf0c + 2 * slot) != 8
            || word(ram, 0xdebc + 2 * slot) != 0xe0b1
            || word(ram, 0x7a5) != 96
            || word(ram, 0x7a7) != 48
        {
            return None;
        }
        Some(Self {
            slot,
            ip: word(ram, 0x1d27 + 2 * slot),
            pre: word(ram, 0x1cd7 + 2 * slot),
            draw: word(ram, 0xde6c + 2 * slot),
            timer: word(ram, 0x1d77 + 2 * slot),
        })
    }
    fn waiting(self, ram: &[u8]) -> bool {
        self.pre == 0xdf89
            && self.timer & 0xff != 0xff
            && matches!((self.ip, self.draw, word(ram, 0x11f90)), (0xe0a9, 0xa2df, 0xb04a) | (0xe0ad, 0xa2e5, 0xb04b))
            && ram[0x16402 + 0xfc7] == 0x45
    }
    fn collected(self) -> bool {
        self.pre == 0xdfa8 && self.timer & 0xff == 0xff && self.ip == 0xe0ba && matches!(self.draw, 0xa2df | 0xa2e5)
    }
}

#[derive(Default)]
struct Response {
    first: u64,
    last: u64,
    polls: u64,
    moving: u64,
    pixels: u64,
}

pub struct Energy {
    previous: Option<State>,
    prefix_verified: bool,
    invalid: bool,
    reason: &'static str,
    stage: usize,
    rooms: [Option<u64>; 8],
    item_map: [u8; 64],
    target_bit: bool,
    health: u16,
    maximum: u16,
    presentation_clear: bool,
    present_step: Option<u64>,
    slot: Option<usize>,
    previous_waiting: bool,
    previous_unfrozen: bool,
    acquired_step: Option<u64>,
    message_step: Option<u64>,
    message_waits: u64,
    acknowledge_step: Option<u64>,
    message_end: Option<u64>,
    checkpoint: Option<u64>,
    response: Response,
    stalled: u64,
    jump_until: u64,
    jump_ready: u64,
    climb_target: Option<(u16, u16)>,
    landing_committed: bool,
    construction_best_y: Option<u16>,
    construction_no_progress_polls: u64,
    navigation: &'static str,
    navigation_blocked: bool,
}
impl Default for Energy {
    fn default() -> Self {
        Self {
            previous: None,
            prefix_verified: false,
            invalid: false,
            reason: "",
            stage: 0,
            rooms: [None; 8],
            item_map: [0; 64],
            target_bit: false,
            health: 0,
            maximum: 0,
            presentation_clear: false,
            present_step: None,
            slot: None,
            previous_waiting: false,
            previous_unfrozen: false,
            acquired_step: None,
            message_step: None,
            message_waits: 0,
            acknowledge_step: None,
            message_end: None,
            checkpoint: None,
            response: Response::default(),
            stalled: 0,
            jump_until: 0,
            jump_ready: 0,
            climb_target: None,
            landing_committed: false,
            construction_best_y: None,
            construction_no_progress_polls: 0,
            navigation: "prefix",
            navigation_blocked: false,
        }
    }
}

impl Energy {
    fn fail(&mut self, reason: &'static str) {
        if !self.invalid {
            self.reason = reason;
        }
        self.invalid = true;
    }
    pub fn observe(&mut self, state: &State, ram: &[u8]) {
        if ram.len() != 0x20000 {
            self.fail("incomplete WRAM");
            return;
        }
        if !state_matches_ram(state, ram) {
            self.fail("State disagrees with WRAM");
        }
        self.health = word(ram, 0x9c2);
        self.maximum = word(ram, 0x9c4);
        self.presentation_clear = clear(ram);
        if self.previous.is_none() {
            self.prefix_verified = state.host_steps == PREFIX
                && normal(state, 0)
                && state.loading_game_state == 5
                && (state.x, state.y, state.timer) == (102, 139, 0)
                && (state.polls, state.updates, state.nmis, state.waits) == (22710, 22690, 24832, 2142)
                && (state.requested_buttons, state.effective_buttons, state.joy1_current, state.joy1_new)
                    == (0x81, 0x81, 0x8100, 0)
                && state.cpu_fallback_calls == 0
                && state.cpu_instructions == 0
                && format!("{:x}", Sha256::digest(ram)) == PREFIX_WRAM;
            if !self.prefix_verified {
                self.fail("wrong first observation or prefix");
            }
            self.item_map.copy_from_slice(&ram[0xd870..0xd8b0]);
            self.rooms[0] = self.prefix_verified.then_some(PREFIX);
            self.previous = Some(*state);
            self.previous_unfrozen = word(ram, 0xa78) == 0;
            return;
        }
        let previous = self.previous.unwrap();
        if state.host_steps != previous.host_steps + 1
            || state.nmis != previous.nmis + 1
            || state.updates.checked_sub(previous.updates).is_none_or(|n| n > 1)
            || state.polls.checked_sub(previous.polls).is_none_or(|n| n > 1)
            || state.waits.checked_sub(previous.waits).is_none_or(|n| n > 1)
            || state.nmis != state.host_steps
            || state.updates.checked_add(state.waits) != Some(state.host_steps)
            || state.c_gameplay_calls < previous.c_gameplay_calls
        {
            self.fail("gapped or inconsistent counters");
        }
        if state.host_steps > PREFIX + TAIL {
            self.fail("tail budget exceeded");
        }
        if state.cpu_fallback_calls != 0 || state.cpu_instructions != 0 {
            self.fail("CPU fallback");
        }
        if !inventory(ram) || controls(ram).is_none() {
            self.fail("changed inventory or controls");
        }
        if self.health == 0 || self.health > self.maximum {
            self.fail("invalid health");
        }
        if state.game_state == 8 {
            match ROOMS.iter().position(|room| *room == state.room) {
                Some(stage)
                    if (stage == self.stage || stage == self.stage + 1) && state.area == u16::from(stage < 3) =>
                {
                    if stage != self.stage {
                        self.stage = stage;
                        self.rooms[stage] = Some(state.host_steps);
                        self.stalled = 0;
                        self.jump_until = 0;
                        self.jump_ready = 0;
                        self.climb_target = None;
                        self.landing_committed = false;
                        self.construction_best_y = None;
                        self.construction_no_progress_polls = 0;
                    }
                }
                _ => self.fail("wrong normal-room order or area"),
            }
        }
        let bit = ram[0xd871] & 1 != 0;
        if self
            .item_map
            .iter()
            .zip(&ram[0xd870..0xd8b0])
            .enumerate()
            .any(|(i, (&baseline, &current))| current & !(if i == 1 { 1 } else { 0 }) != baseline)
        {
            self.fail("unrelated persistent item change");
        }
        let tank = normal(state, 7).then(|| Tank::read(ram)).flatten();
        let waiting = tank.is_some_and(|tank| tank.waiting(ram));
        if normal(state, 7) && !bit && waiting && self.maximum == 99 && clear(ram) {
            let found = tank.unwrap();
            if self.slot.is_some_and(|slot| slot != found.slot) {
                self.fail("target slot changed");
            }
            self.slot.get_or_insert(found.slot);
            self.present_step.get_or_insert(state.host_steps);
        }
        if bit && !self.target_bit {
            // Pickup starts the native message coroutine before the enclosing
            // update completes: a real consumed poll may end as an NMI wait.
            let consumed = state.polls == previous.polls + 1
                && state.effective_buttons != 0
                && state.requested_buttons == state.effective_buttons
                && state.joy1_current == state.effective_buttons.reverse_bits();
            let contact = state.x.abs_diff(120) <= 32
                && state.y.abs_diff(680) <= 48
                && previous.x.abs_diff(120) <= 48
                && previous.y.abs_diff(680) <= 64;
            if !normal(&previous, 7)
                || !normal(state, 7)
                || !self.previous_waiting
                || !self.previous_unfrozen
                || self.present_step.is_none_or(|step| step >= state.host_steps)
                || !tank.is_some_and(|tank| self.slot == Some(tank.slot) && tank.collected())
                || !consumed
                || !contact
                || self.maximum != 199
                || self.health != 199
                || !message(ram)
            {
                self.fail("acquisition lacks target contact or collector/message state");
            } else {
                self.acquired_step = Some(state.host_steps);
                self.message_step = Some(state.host_steps);
            }
        } else if bit != self.target_bit {
            self.fail("target bit cleared");
        }
        if self.maximum != if self.acquired_step.is_some() { 199 } else { 99 } {
            self.fail("unexpected maximum energy");
        }
        self.target_bit = bit;
        if let Some(start) = self.message_step {
            if self.message_end.is_none() {
                if !normal(state, 7) || state.host_steps > start + 512 {
                    self.fail("message did not complete in bounded target room");
                }
                if message(ram) {
                    if state.updates == previous.updates && state.waits == previous.waits + 1 {
                        self.message_waits += 1;
                    }
                    if self.message_waits >= 360
                        && state.polls == previous.polls + 1
                        && state.requested_buttons != 0
                        && state.requested_buttons == state.effective_buttons
                        && state.joy1_current == state.effective_buttons.reverse_bits()
                    {
                        self.acknowledge_step.get_or_insert(state.host_steps);
                    }
                } else if clear(ram)
                    && self.message_waits >= 360
                    && self.acknowledge_step.is_some_and(|step| step < state.host_steps)
                {
                    self.message_end = Some(state.host_steps);
                } else {
                    self.fail("synthetic or wrong message completion");
                }
            }
        } else if !clear(ram) && word(ram, 0xdc8) != 0 {
            self.fail("unexpected pickup message before target");
        }
        if self.message_end.is_some_and(|step| previous.host_steps >= step)
            && normal(&previous, 7)
            && normal(state, 7)
            && clear(ram)
        {
            self.observe_response(&previous, state);
        }
        if state.polls > previous.polls {
            // author02 moved continuously while repeating the same jump arc.
            // Count only consumed normal Construction updates, never transition
            // or NMI-only waits. New ascent by8px restarts this bounded window.
            if normal(&previous, 1) && normal(state, 1) && state.updates == previous.updates + 1 && clear(ram) {
                if self.construction_best_y.is_none_or(|best| best.saturating_sub(state.y) >= 8) {
                    self.construction_best_y = Some(state.y);
                    self.construction_no_progress_polls = 0;
                } else {
                    self.construction_no_progress_polls += 1;
                    if self.construction_no_progress_polls >= 1024 {
                        self.navigation_blocked = true;
                        self.navigation = "Construction repeated ascent without new height";
                    }
                }
            }
            self.stalled = if normal(&previous, self.stage)
                && normal(state, self.stage)
                && (previous.x, previous.y) == (state.x, state.y)
            {
                self.stalled + 1
            } else {
                0
            };
            if self.stalled >= 256 && self.acquired_step.is_none() {
                self.navigation_blocked = true;
            }
        }
        self.previous_waiting = waiting;
        self.previous_unfrozen = word(ram, 0xa78) == 0;
        self.previous = Some(*state);
    }

    fn observe_response(&mut self, previous: &State, state: &State) {
        if self.checkpoint.is_some() || state.polls == previous.polls {
            return;
        }
        let pixels = state.x.checked_sub(previous.x);
        if state.polls != previous.polls + 1
            || state.updates != previous.updates + 1
            || state.requested_buttons & (LEFT | RIGHT) != RIGHT
            || state.effective_buttons & (LEFT | RIGHT) != RIGHT
            || state.joy1_current & (LEFT | RIGHT).reverse_bits() != RIGHT.reverse_bits()
            || pixels.is_none_or(|n| n > 16)
        {
            self.response = Response::default();
            return;
        }
        if self.response.polls == 0 || state.host_steps - self.response.first > 64 {
            self.response = Response { first: previous.host_steps, ..Response::default() };
        }
        let pixels = pixels.unwrap();
        self.response.last = state.host_steps;
        self.response.polls += 1;
        self.response.moving += u64::from(pixels > 0);
        self.response.pixels += u64::from(pixels);
        if self.response.polls >= 8 && self.response.moving >= 4 && self.response.pixels >= 16 {
            self.checkpoint = Some(state.host_steps);
        }
    }

    pub fn buttons(&mut self, state: &State, ram: &[u8]) -> u16 {
        if !self.prefix_verified
            || self.invalid
            || self.complete()
            || self.navigation_blocked
            || ram.len() != 0x20000
            || self.previous != Some(*state)
            || !state_matches_ram(state, ram)
            || !(PREFIX..PREFIX + TAIL).contains(&state.host_steps)
        {
            return 0;
        }
        let Some([run, jump, shoot]) = controls(ram) else {
            return 0;
        };
        if self.acquired_step.is_some() {
            self.navigation = "message or resumed movement";
            if message(ram) {
                return if state.host_steps % 16 < 4 { jump } else { 0 };
            }
            return if self.message_end.is_some() && clear(ram) { RIGHT | run } else { 0 };
        }
        if !normal(state, self.stage) || !clear(ram) {
            self.navigation = "native transition";
            return 0;
        }
        let direction = match self.stage {
            0 => {
                self.navigation = "missile east exit";
                RIGHT
            }
            1 => {
                self.navigation = "Construction reverse ascent";
                // author01: at130 the radius5 still overlaps solid column7.
                // Roll to144 so the full standing span fits open columns8/9.
                if state.y >= 384 && state.x < 144 {
                    return if ball(ram) {
                        RIGHT
                    } else if state.host_steps % 16 < 4 {
                        DOWN
                    } else {
                        0
                    };
                }
                if ball(ram) {
                    return UP;
                }
                if state.y > 168 {
                    return self.ascend(state, ram, 144, jump);
                }
                LEFT
            }
            2 => {
                self.navigation = "Morph upward elevator";
                if word(ram, 0xe18) != 0 {
                    return 0;
                }
                if ball(ram) {
                    return UP;
                }
                if word(ram, 0xa1c) < 9 && word(ram, 0xe16) != 0 {
                    return if state.host_steps % 16 < 4 { UP } else { 0 };
                }
                if state.x.abs_diff(1408) <= 4 && grounded(ram) {
                    return if word(ram, 0xa1c) < 9 { 0 } else { UP };
                }
                toward(state.x, 1408)
            }
            3 => {
                self.navigation = "elevator west exit";
                if word(ram, 0xe18) != 0 {
                    return 0;
                }
                LEFT
            }
            4 => {
                self.navigation = "Pit west exit";
                LEFT
            }
            5 | 6 => {
                self.navigation = if self.stage == 5 { "Climb reverse ascent" } else { "Parlor ascent and west exit" };
                if ball(ram) {
                    return UP;
                }
                if state.y > 160 {
                    return self.ascend(state, ram, 384, jump);
                }
                if self.stage == 5 {
                    let direction = toward(state.x, 384);
                    if direction != 0 {
                        return direction;
                    }
                    self.start_jump(state.updates);
                    return if state.updates < self.jump_until { jump | UP } else { UP };
                }
                LEFT
            }
            7 => {
                self.navigation = "Terminator west descent to cell07,2A";
                if ball(ram) {
                    return UP;
                }
                toward(state.x, 120)
            }
            _ => return 0,
        };
        if ball(ram) {
            return UP;
        }
        if direction != 0 && grounded(ram) && self.stalled >= 12 {
            self.start_jump(state.updates);
        }
        let mut pad = direction;
        if direction != 0 {
            pad |= run;
        }
        if state.updates < self.jump_until {
            pad |= jump;
        }
        if direction != 0 && state.updates % 12 < 2 && (self.stage != 7 || state.x > 192) {
            pad |= shoot;
        }
        pad
    }

    fn start_jump(&mut self, update: u64) {
        if update >= self.jump_ready {
            self.jump_until = update + 24;
            self.jump_ready = update + 40;
        }
    }
    fn ascend(&mut self, state: &State, ram: &[u8], center: u16, jump: u16) -> u16 {
        if self.stage == 1 {
            if grounded(ram) {
                self.climb_target = ledge_above(state, ram);
                self.landing_committed = false;
                self.start_jump(state.updates);
            }
            if let Some((target_x, top)) = self.climb_target {
                // author02 landed at165/315 then steered left into the underside
                // of row16 (x96..159,y256). Clear the contiguous ledge's outer
                // edge vertically before steering over its top; retain this
                // target through the airborne landing approach.
                let width = word(ram, 0x7a5);
                let row = top / 16;
                let mut left = target_x / 16;
                let mut right = left + 1;
                while left > 0 && solid(ram, left - 1, row) {
                    left -= 1;
                }
                while right < width && solid(ram, right, row) {
                    right += 1;
                }
                if state.y.saturating_add(word(ram, 0xb00)) <= top {
                    self.landing_committed = true;
                }
                let direction = if self.landing_committed {
                    toward(state.x, target_x)
                } else {
                    let radius = word(ram, 0xafe).min(16) + 1;
                    let outside_left = (left * 16).saturating_sub(radius);
                    let outside_right = right * 16 + radius;
                    // author03 at x164/radius5 still overlapped pixel159.
                    // The ordinary four-pixel steering tolerance cannot
                    // establish clearance. Move fully outside, then avoid
                    // steering back toward the edge until the feet clear it.
                    if state.x.abs_diff(outside_left) < state.x.abs_diff(outside_right) {
                        if state.x > outside_left { LEFT } else { 0 }
                    } else if state.x < outside_right {
                        RIGHT
                    } else {
                        0
                    }
                };
                self.navigation = if self.landing_committed {
                    "Construction landing over cleared ledge"
                } else {
                    "Construction jump outside ledge edge"
                };
                return direction | if state.updates < self.jump_until { jump } else { 0 };
            }
        }
        // Native wall jump requires previous spin/wall-jump movement and a
        // fresh Jump after the away-direction check reaches animation frame11.
        let radius = word(ram, 0xafe).min(16);
        if matches!(ram[0xa1f], 3 | 0x14) && matches!(ram[0xa27], 3 | 0x14) {
            let left = solid(ram, state.x.saturating_sub(radius + 8) / 16, state.y / 16);
            let right = solid(ram, state.x.saturating_add(radius + 8) / 16, state.y / 16);
            if left || right {
                self.navigation = "observed wall-jump edge";
                let away = if left { RIGHT } else { LEFT };
                return away | if word(ram, 0xa96) >= 11 && state.effective_buttons & jump == 0 { jump } else { 0 };
            }
        }
        if self.climb_target.is_some_and(|(_, y)| state.y.saturating_add(24) < y) {
            self.climb_target = None;
        }
        if grounded(ram) {
            self.climb_target = ledge_above(state, ram);
            self.start_jump(state.updates);
        }
        let target = self.climb_target.map_or_else(|| nearest_wall(state, ram).unwrap_or(center), |(x, _)| x);
        let direction = toward(state.x, target);
        // Horizontal input at takeoff requests the ordinary spin jump needed
        // by the native wall-jump path; release Jump between attempts.
        let direction =
            if direction == 0 && grounded(ram) { if state.x < center { RIGHT } else { LEFT } } else { direction };
        direction | if state.updates < self.jump_until { jump } else { 0 }
    }

    pub fn present(&self) -> bool {
        self.prefix_verified && !self.invalid && self.present_step.is_some()
    }
    pub fn blocked(&self) -> bool {
        self.navigation_blocked
    }
    pub fn acquired(&self) -> bool {
        self.prefix_verified && !self.invalid && self.acquired_step.is_some()
    }
    pub fn complete(&self) -> bool {
        self.acquired()
            && self.rooms.iter().all(Option::is_some)
            && self.target_bit
            && self.maximum == 199
            && self.health > 0
            && self.message_waits >= 360
            && self.acknowledge_step.is_some()
            && self.message_end.is_some()
            && self.checkpoint.is_some()
            && self.presentation_clear
            && self.previous.is_some_and(|state| normal(&state, 7))
    }
    pub fn report(&self, report: &mut sm_binary_evidence::Map) {
        for (key, value) in [
            ("checkpoint", self.complete()),
            ("prefix_verified", self.prefix_verified),
            ("invalid", self.invalid),
            ("present", self.present()),
            ("acquired", self.acquired()),
            ("item_bit", self.target_bit),
            ("navigation_blocked", self.navigation_blocked),
        ] {
            report.insert(format!("energy_{key}"), value.into());
        }
        report.insert("energy_invalid_reason".into(), self.reason.into());
        report.insert("energy_navigation".into(), self.navigation.into());
        report.insert("energy_prefix_expected_wram_sha256".into(), PREFIX_WRAM.into());
        for (key, value) in [
            ("stage", self.stage as u64),
            ("last_room", u64::from(self.previous.map_or(0, |state| state.room))),
            ("health", u64::from(self.health)),
            ("maximum", u64::from(self.maximum)),
            ("stalled_polls", self.stalled),
            ("construction_no_progress_polls", self.construction_no_progress_polls),
            ("message_waits", self.message_waits),
            ("response_polls", self.response.polls),
            ("response_moving_polls", self.response.moving),
            ("response_right_pixels", self.response.pixels),
            ("response_first_step", self.response.first),
            ("response_last_step", self.response.last),
        ] {
            report.insert(format!("energy_{key}"), (value as i64).into());
        }
        for (key, value) in [
            ("present_step", self.present_step),
            ("acquired_step", self.acquired_step),
            ("message_step", self.message_step),
            ("acknowledge_step", self.acknowledge_step),
            ("message_end_step", self.message_end),
            ("checkpoint_step", self.checkpoint),
        ] {
            report.insert(format!("energy_{key}"), value.map_or(-1, |n| n as i64).into());
        }
        report.insert("energy_target_slot".into(), self.slot.map_or(-1, |slot| slot as i64).into());
        report.insert("energy_construction_best_y".into(), self.construction_best_y.map_or(-1, i64::from).into());
        report.insert(
            "energy_room_steps".into(),
            self.rooms
                .iter()
                .map(|s| sm_binary_evidence::Value::Integer(s.map_or(-1, |n| n as i64)))
                .collect::<Vec<_>>()
                .into(),
        );
    }
}

// Bounded WRAM terrain queries guide ordinary input only. They are not a
// collision implementation or evidence that a proposed ascent is traversable.
fn solid(ram: &[u8], x: u16, y: u16) -> bool {
    let (width, height) = (word(ram, 0x7a5), word(ram, 0x7a7));
    if width == 0 || width > 255 || x >= width || y >= height {
        return false;
    }
    let at = 0x10002 + 2 * (usize::from(y) * usize::from(width) + usize::from(x));
    at + 1 < ram.len() && matches!(word(ram, at) >> 12, 8 | 9 | 10 | 12 | 15)
}
fn ledge_above(state: &State, ram: &[u8]) -> Option<(u16, u16)> {
    let mut best = None;
    for y in state.y.saturating_sub(96) / 16..state.y.saturating_sub(24) / 16 {
        for x in state.x.saturating_sub(96) / 16..=state.x.saturating_add(96) / 16 {
            if y >= 2 && solid(ram, x, y) && !solid(ram, x, y - 1) && !solid(ram, x, y - 2) {
                let point = (x * 16 + 8, y * 16);
                let score = state.x.abs_diff(point.0) + state.y.abs_diff(point.1);
                if best.is_none_or(|(_, old)| score < old) {
                    best = Some((point, score));
                }
            }
        }
    }
    best.map(|(point, _)| point)
}
fn nearest_wall(state: &State, ram: &[u8]) -> Option<u16> {
    let radius = word(ram, 0xafe).min(16);
    for distance in 1..=5 {
        let column = state.x / 16;
        if let Some(x) = column.checked_sub(distance)
            && solid(ram, x, state.y / 16)
        {
            return Some((x + 1) * 16 + radius);
        }
        if let Some(x) = column.checked_add(distance)
            && solid(ram, x, state.y / 16)
        {
            return Some(x * 16 - radius);
        }
    }
    None
}

#[cfg(any(test, feature = "private-checks"))]
mod tests {
    use super::*;
    use std::sync::OnceLock;

    fn set(ram: &mut [u8], at: usize, value: u16) {
        ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
    }
    fn sync(state: &State, ram: &mut [u8]) {
        for (value, at) in [
            (state.game_state, 0x998),
            (state.loading_game_state, 0xd914),
            (state.area, 0x79f),
            (state.room, 0x79b),
            (state.x, 0xaf6),
            (state.y, 0xafa),
            (state.timer, 0x943),
            (state.joy1_current, 0x8b),
            (state.joy1_new, 0x8f),
        ] {
            set(ram, at, value);
        }
    }
    fn prefix_data() -> (State, Vec<u8>) {
        static PREFIX_BYTES: OnceLock<Vec<u8>> = OnceLock::new();
        let ram = PREFIX_BYTES
            .get_or_init(|| {
                let path = std::env::var_os("SM_ENERGY_PREFIX_WRAM").unwrap_or_else(|| {
                    "/home/jim/sm-decomp-restart/restart/out/first-missile/author-05/final.wram".into()
                });
                use std::io::Read;
                let mut bytes = Vec::with_capacity(131072);
                std::fs::File::open(path)
                    .expect("retained exact missile endpoint WRAM")
                    .take(131073)
                    .read_to_end(&mut bytes)
                    .expect("read exact missile endpoint WRAM");
                assert_eq!(bytes.len(), 131072, "exact prefix input size");
                assert_eq!(
                    format!("{:x}", Sha256::digest(&bytes)),
                    "cb3362278173187dcda825bea8c4ec5c06952c1db7a5a948affb1c8d96a97483",
                    "exact prefix input hash"
                );
                bytes
            })
            .clone();
        let state = State {
            host_steps: PREFIX,
            polls: 22710,
            updates: 22690,
            nmis: PREFIX,
            waits: 2142,
            game_state: 8,
            loading_game_state: 5,
            area: 1,
            room: ROOMS[0],
            x: 102,
            y: 139,
            requested_buttons: 0x81,
            effective_buttons: 0x81,
            joy1_current: 0x8100,
            ..State::default()
        };
        (state, ram)
    }
    fn prefix() -> (Energy, State, Vec<u8>) {
        let (state, ram) = prefix_data();
        let mut energy = Energy::default();
        energy.observe(&state, &ram);
        assert!(energy.prefix_verified && !energy.invalid, "{}", energy.reason);
        (energy, state, ram)
    }
    fn step(state: &mut State, ram: &mut [u8], pad: u16, wait: bool, poll: bool) {
        state.host_steps += 1;
        state.nmis += 1;
        if wait {
            state.waits += 1;
        } else {
            state.updates += 1;
        }
        if poll {
            state.polls += 1;
        }
        state.requested_buttons = pad;
        state.effective_buttons = pad;
        if poll {
            state.joy1_new = pad.reverse_bits() & !state.joy1_current;
            state.joy1_current = pad.reverse_bits();
        }
        sync(state, ram);
    }
    fn install_tank(ram: &mut [u8], slot: usize, collected: bool) {
        for i in 0..40 {
            set(ram, 0x1c37 + 2 * i, 0);
        }
        for (at, value) in [
            (0x1c37, 0xeed7),
            (0x1dc7, 8),
            (0x1c87, 0x1f8e),
            (0xdf0c, 8),
            (0xdebc, 0xe0b1),
            (0x1cd7, if collected { 0xdfa8 } else { 0xdf89 }),
            (0x1d27, if collected { 0xe0ba } else { 0xe0a9 }),
            (0xde6c, 0xa2df),
            (0x1d77, if collected { 0xff } else { 0 }),
        ] {
            set(ram, at + 2 * slot, value);
        }
        set(ram, 0x7a5, 96);
        set(ram, 0x7a7, 48);
        set(ram, 0x11f90, 0xb04a);
        ram[0x173c9] = 0x45;
    }
    fn target(damaged: bool) -> (Energy, State, Vec<u8>) {
        let (mut energy, mut state, mut ram) = prefix();
        for (stage, room) in ROOMS.into_iter().enumerate().skip(1) {
            state.room = room;
            state.area = u16::from(stage < 3);
            state.x = 120;
            state.y = 659;
            if damaged {
                set(&mut ram, 0x9c2, 70);
            }
            if stage == 7 {
                install_tank(&mut ram, 2, false);
            }
            step(&mut state, &mut ram, LEFT, false, true);
            energy.observe(&state, &ram);
        }
        assert!(energy.present(), "{}", energy.reason);
        (energy, state, ram)
    }
    fn pickup_state(state: &mut State, ram: &mut [u8]) {
        state.x = 119;
        install_tank(ram, 2, true);
        ram[0xd871] |= 1;
        set(ram, 0x9c2, 199);
        set(ram, 0x9c4, 199);
        set(ram, 0xdc8, 1);
        set(ram, 0x1c1f, 1);
        ram[0x614] = 2;
        step(state, ram, LEFT, true, true);
    }
    fn acquired() -> (Energy, State, Vec<u8>) {
        let (mut energy, mut state, mut ram) = target(true);
        pickup_state(&mut state, &mut ram);
        energy.observe(&state, &ram);
        assert!(energy.acquired(), "{}", energy.reason);
        (energy, state, ram)
    }
    fn finish_message(energy: &mut Energy, state: &mut State, ram: &mut [u8], ack: bool) {
        for _ in 0..360 {
            step(state, ram, 0, true, false);
            energy.observe(state, ram);
        }
        if ack {
            let jump = word(ram, 0x9b4).reverse_bits();
            step(state, ram, jump, true, true);
            energy.observe(state, ram);
        }
        set(ram, 0xdc8, 0);
        ram[0x614] = 0;
        step(state, ram, 0, false, false);
        energy.observe(state, ram);
    }

    #[cfg_attr(test, test)]
    fn exact_prefix_is_required_first_and_is_not_repaired_by_later_valid_state() {
        for alteration in ["step", "counter", "ram", "inventory", "already-collected"] {
            let (mut state, mut ram) = prefix_data();
            match alteration {
                "step" => state.host_steps -= 1,
                "counter" => state.polls += 1,
                "ram" => ram[0x1ffff] ^= 1,
                "inventory" => set(&mut ram, 0x9c4, 199),
                "already-collected" => ram[0xd871] |= 1,
                _ => unreachable!(),
            }
            let mut energy = Energy::default();
            energy.observe(&state, &ram);
            assert!(energy.invalid, "{alteration}");
            let (state, ram) = prefix_data();
            energy.observe(&state, &ram);
            assert!(!energy.complete());
            assert_eq!(energy.buttons(&state, &ram), 0);
        }
    }

    #[cfg_attr(test, test)]
    fn gaps_state_mismatch_skipped_rooms_and_wrong_elevator_area_fail_closed() {
        for alteration in ["gap", "state-ram", "room", "area", "capacity", "item"] {
            let (mut energy, mut state, mut ram) = prefix();
            step(&mut state, &mut ram, RIGHT, false, true);
            match alteration {
                "gap" => {
                    state.host_steps += 1;
                    state.nmis += 1;
                    state.updates += 1;
                }
                "state-ram" => state.x += 1,
                "room" => {
                    state.room = ROOMS[2];
                    sync(&state, &mut ram);
                }
                "area" => {
                    state.room = ROOMS[1];
                    state.area = 0;
                    sync(&state, &mut ram);
                }
                "capacity" => set(&mut ram, 0x9c8, 10),
                "item" => ram[0xd872] ^= 1,
                _ => unreachable!(),
            }
            energy.observe(&state, &ram);
            assert!(energy.invalid, "{alteration}");
            assert_eq!(energy.buttons(&state, &ram), 0);
        }
    }

    #[cfg_attr(test, test)]
    fn presence_checks_exact_slot_draw_block_and_bts_before_contact() {
        let (mut energy, mut state, mut ram) = target(false);
        let tank = Tank::read(&ram).unwrap();
        assert!(tank.waiting(&ram));
        for at in [0x1c87 + 4, 0x1dc7 + 4, 0xdebc + 4, 0xdf0c + 4, 0x11f90, 0x173c9] {
            let mut bad = ram.clone();
            bad[at] ^= 1;
            assert!(!Tank::read(&bad).is_some_and(|value| value.waiting(&bad)), "{at:x}");
        }
        set(&mut ram, 0x1c37 + 6, 0xeed7);
        set(&mut ram, 0x1dc7 + 6, 8);
        assert!(Tank::read(&ram).is_none());
        step(&mut state, &mut ram, LEFT, false, true);
        energy.observe(&state, &ram);
        pickup_state(&mut state, &mut ram);
        energy.observe(&state, &ram);
        assert!(!energy.acquired(), "missing exact previous target cannot acquire");
    }

    #[cfg_attr(test, test)]
    fn acquisition_rejects_synthetic_capacity_message_contact_and_unconsumed_input() {
        for alteration in ["maximum", "health", "message", "remote", "poll", "slot"] {
            let (mut energy, mut state, mut ram) = target(false);
            let polls = state.polls;
            pickup_state(&mut state, &mut ram);
            match alteration {
                "maximum" => set(&mut ram, 0x9c4, 299),
                "health" => set(&mut ram, 0x9c2, 198),
                "message" => set(&mut ram, 0x1c1f, 2),
                "remote" => {
                    state.x = 900;
                    sync(&state, &mut ram);
                }
                "poll" => state.polls = polls,
                "slot" => install_tank(&mut ram, 3, true),
                _ => unreachable!(),
            }
            energy.observe(&state, &ram);
            assert!(energy.invalid, "{alteration}");
            assert!(!energy.acquired());
        }
    }

    #[cfg_attr(test, test)]
    fn real_wait_and_acknowledgement_are_required_before_resumed_movement() {
        for premature in [true, false] {
            let (mut energy, mut state, mut ram) = acquired();
            if premature {
                set(&mut ram, 0xdc8, 0);
                ram[0x614] = 0;
                step(&mut state, &mut ram, RIGHT, false, true);
                energy.observe(&state, &ram);
            } else {
                finish_message(&mut energy, &mut state, &mut ram, false);
            }
            assert!(energy.invalid);
            assert!(!energy.complete());
        }
        let (mut energy, mut state, mut ram) = acquired();
        finish_message(&mut energy, &mut state, &mut ram, true);
        assert!(!energy.invalid, "{}", energy.reason);
        assert!(energy.message_end.is_some());
        for _ in 0..8 {
            step(&mut state, &mut ram, RIGHT, false, true);
            energy.observe(&state, &ram);
        }
        assert!(!energy.complete(), "held input without displacement is insufficient");
        for _ in 0..8 {
            state.x += 2;
            step(&mut state, &mut ram, RIGHT, false, false);
            energy.observe(&state, &ram);
        }
        assert!(!energy.complete(), "displacement without consumed input is insufficient");
        for _ in 0..8 {
            state.x += 2;
            step(&mut state, &mut ram, RIGHT, false, true);
            energy.observe(&state, &ram);
        }
        assert!(energy.complete(), "{}", energy.reason);
        assert_eq!(energy.health, 199);
        assert_eq!(energy.maximum, 199);
        assert!(energy.rooms.iter().all(Option::is_some));
        let mut report = sm_binary_evidence::Map::new();
        energy.report(&mut report);
        assert_eq!(report["energy_checkpoint"].as_bool(), Some(true));
        assert_eq!(report["energy_target_slot"].as_integer(), Some(2));
    }

    #[cfg(test)]
    #[test]
    fn construction_author01_keeps_rolling_until_the_standing_span_clears() {
        let path = std::env::var_os("SM_ENERGY_CONSTRUCTION_WRAM")
            .unwrap_or_else(|| "/home/jim/sm-decomp-restart/restart/out/energy-tank/author-01/final.wram".into());
        let mut ram = std::fs::read(path).expect("retained author01 Construction obstruction");
        assert_eq!(
            format!("{:x}", Sha256::digest(&ram)),
            "3dd209cbd468fed3e2118c7084384b06f81b50eab323126623de50af6ec03a20"
        );
        // CSV25040 is the first stationary Up input at130/409 after rolling
        // through124. Final terrain/pose retains the same obstruction.
        let mut state = State {
            host_steps: 25040,
            polls: 22855,
            updates: 22835,
            nmis: 25040,
            waits: 2205,
            game_state: 8,
            loading_game_state: 5,
            area: 1,
            room: 0x9f11,
            x: 130,
            y: 409,
            requested_buttons: UP,
            effective_buttons: UP,
            joy1_current: UP.reverse_bits(),
            ..State::default()
        };
        sync(&state, &mut ram);
        assert_eq!((word(&ram, 0x7a5), word(&ram, 0x7a7)), (16, 32));
        assert_eq!((word(&ram, 0xa1c), ram[0xa1f], word(&ram, 0xafe), word(&ram, 0xb00)), (0x1d, 4, 5, 7));
        assert!(solid(&ram, 7, 24));
        assert!(!solid(&ram, 8, 24) && !solid(&ram, 9, 24));
        assert_eq!((state.x - word(&ram, 0xafe)) / 16, 7, "left radius still intersects the low ceiling");
        let mut energy = Energy { prefix_verified: true, stage: 1, previous: Some(state), ..Energy::default() };
        let before = ram.clone();
        assert_eq!(energy.buttons(&state, &ram), RIGHT, "continue ordinary rolling before requesting standing");
        assert_eq!(ram, before);
        state.x = 144;
        sync(&state, &mut ram);
        energy.previous = Some(state);
        assert_eq!((state.x - word(&ram, 0xafe)) / 16, 8);
        assert_eq!((state.x + word(&ram, 0xafe)) / 16, 9);
        assert_eq!(energy.buttons(&state, &ram), UP, "the full radius now fits the two-column shaft");
    }

    #[cfg(test)]
    #[test]
    fn construction_author02_clears_the_ledge_before_steering_over_it() {
        let path = std::env::var_os("SM_ENERGY_CONSTRUCTION_ASCENT_WRAM")
            .unwrap_or_else(|| "/home/jim/sm-decomp-restart/restart/out/energy-tank/author-02/final.wram".into());
        let mut ram = std::fs::read(path).expect("retained author02 Construction landing terrain");
        assert_eq!(
            format!("{:x}", Sha256::digest(&ram)),
            "caebf8e8b11299f0ab17abd094fd1c65b266fd723eb0dfdd23c803c880f1060f"
        );
        // CSV25120: standing on top336 at165/315. The next selected ledge
        // is row16, columns6..9 (x96..159,y256..271), with BTS zero.
        let mut state = State {
            host_steps: 25120,
            polls: 22935,
            updates: 22915,
            nmis: 25120,
            waits: 2205,
            game_state: 8,
            loading_game_state: 5,
            area: 1,
            room: 0x9f11,
            x: 165,
            y: 315,
            requested_buttons: LEFT,
            effective_buttons: LEFT,
            joy1_current: LEFT.reverse_bits(),
            ..State::default()
        };
        for (at, value) in [(0xa1c, 0x25), (0xb2e, 0), (0xb2c, 0), (0xb36, 0), (0xafe, 5), (0xb00, 21)] {
            set(&mut ram, at, value);
        }
        ram[0xa1f] = 0x0e;
        sync(&state, &mut ram);
        assert!(grounded(&ram));
        for column in 6..=9 {
            assert_eq!(word(&ram, 0x10002 + 2 * (16 * 16 + column)), 0xc19a);
            assert_eq!(ram[0x16402 + 16 * 16 + column], 0);
        }
        assert!(!solid(&ram, 10, 16));
        assert_eq!(ledge_above(&state, &ram), Some((152, 256)));
        let mut energy = Energy { prefix_verified: true, stage: 1, previous: Some(state), ..Energy::default() };
        let pad = energy.buttons(&state, &ram);
        assert_eq!(pad & LEFT, 0, "do not jump left into the selected platform underside");
        assert_ne!(pad & word(&ram, 0x9b4).reverse_bits(), 0, "jump outside the platform edge");
        assert_eq!(energy.climb_target, Some((152, 256)));
        state.y = 228;
        state.updates += 20;
        state.host_steps += 20;
        state.nmis += 20;
        state.polls += 20;
        set(&mut ram, 0xb2e, 1);
        set(&mut ram, 0xb36, 1);
        set(&mut ram, 0xb00, 19);
        ram[0xa1f] = 2;
        sync(&state, &mut ram);
        energy.previous = Some(state);
        assert!(state.y + word(&ram, 0xb00) <= 256);
        assert_eq!(
            energy.buttons(&state, &ram) & (LEFT | RIGHT),
            LEFT,
            "steer onto the ledge only after clearing its top"
        );
        assert_eq!(energy.climb_target, Some((152, 256)), "keep landing target through airborne traversal");
    }

    #[cfg(test)]
    #[test]
    fn construction_author03_requires_full_span_clearance_before_vertical_ascent() {
        let path = std::env::var_os("SM_ENERGY_CONSTRUCTION_CLEARANCE_WRAM")
            .unwrap_or_else(|| "/home/jim/sm-decomp-restart/restart/out/energy-tank/author-03/final.wram".into());
        let mut ram = std::fs::read(path).expect("retained author03 Construction ledge terrain");
        assert_eq!(
            format!("{:x}", Sha256::digest(&ram)),
            "971e217266ca51e8b02306106d668aae19bc7be43b22b96edc939c8058199bcf"
        );
        // CSV25120: x164/315, radius5. The leftmost pixel159 still
        // intersects the selected ledge, despite being near outside_x166.
        let mut state = State {
            host_steps: 25120,
            polls: 22935,
            updates: 22915,
            nmis: 25120,
            waits: 2205,
            game_state: 8,
            loading_game_state: 5,
            area: 1,
            room: 0x9f11,
            x: 164,
            y: 315,
            ..State::default()
        };
        for (at, value) in [(0xa1c, 0xa4), (0xb2e, 0), (0xb2c, 0), (0xb36, 0), (0xafe, 5), (0xb00, 21)] {
            set(&mut ram, at, value);
        }
        ram[0xa1f] = 0;
        sync(&state, &mut ram);
        assert!(grounded(&ram));
        assert_eq!(ledge_above(&state, &ram), Some((152, 256)));
        assert!(solid(&ram, (state.x - word(&ram, 0xafe)) / 16, 16));
        assert!(!solid(&ram, 10, 16));
        let mut energy = Energy { prefix_verified: true, stage: 1, previous: Some(state), ..Energy::default() };
        assert_eq!(
            energy.buttons(&state, &ram) & (LEFT | RIGHT),
            RIGHT,
            "the steering deadband must not permit jumping with the radius overlapping the ledge"
        );
        // CSV25132: the jump remains x164 at y292, immediately before
        // the captured head collision. Continue clearing even while airborne.
        state.y = 292;
        set(&mut ram, 0xa1c, 0x4d);
        set(&mut ram, 0xb2e, 4);
        set(&mut ram, 0xb2c, 0x5400);
        set(&mut ram, 0xb36, 1);
        set(&mut ram, 0xb00, 19);
        ram[0xa1f] = 2;
        for (x, direction) in [(164, RIGHT), (166, 0), (172, 0)] {
            state.x = x;
            sync(&state, &mut ram);
            energy.previous = Some(state);
            assert_eq!(energy.buttons(&state, &ram) & (LEFT | RIGHT), direction, "clearance at x{x}");
            assert!(!energy.landing_committed);
        }
        state.y = 228;
        sync(&state, &mut ram);
        energy.previous = Some(state);
        assert_eq!(energy.buttons(&state, &ram) & (LEFT | RIGHT), LEFT);
        assert!(energy.landing_committed);
    }

    #[cfg_attr(test, test)]
    fn construction_moving_cycles_are_bounded_but_new_height_and_transition_waits_are_preserved() {
        let (mut energy, mut state, mut ram) = prefix();
        state.room = ROOMS[1];
        state.x = 150;
        state.y = 284;
        step(&mut state, &mut ram, RIGHT, false, true);
        energy.observe(&state, &ram);
        step(&mut state, &mut ram, RIGHT, false, true);
        energy.observe(&state, &ram);
        assert_eq!(energy.construction_best_y, Some(284));
        for i in 0..100 {
            state.x = 143 + i % 17;
            state.y = 290 + i % 111;
            step(&mut state, &mut ram, RIGHT, false, true);
            energy.observe(&state, &ram);
        }
        assert_eq!(energy.construction_no_progress_polls, 100);
        assert!(!energy.blocked());
        state.y = 275;
        step(&mut state, &mut ram, RIGHT, false, true);
        energy.observe(&state, &ram);
        assert_eq!(energy.construction_best_y, Some(275));
        assert_eq!(energy.construction_no_progress_polls, 0);
        for i in 0..1023 {
            state.x = 143 + i % 17;
            state.y = 290 + i % 111;
            step(&mut state, &mut ram, RIGHT, false, true);
            energy.observe(&state, &ram);
        }
        assert_eq!(energy.construction_no_progress_polls, 1023);
        assert!(!energy.blocked());
        state.game_state = 9;
        for _ in 0..50 {
            step(&mut state, &mut ram, 0, true, false);
            energy.observe(&state, &ram);
        }
        assert_eq!(energy.construction_no_progress_polls, 1023);
        assert!(!energy.blocked());
        state.game_state = 8;
        step(&mut state, &mut ram, RIGHT, false, true);
        energy.observe(&state, &ram);
        assert!(!energy.blocked(), "transition return does not count as a normal-to-normal update");
        step(&mut state, &mut ram, RIGHT, false, true);
        energy.observe(&state, &ram);
        assert!(energy.blocked());
        assert!(!energy.invalid, "navigation gap is not invalid evidence");
        assert_eq!(energy.buttons(&state, &ram), 0);
    }

    #[cfg_attr(test, test)]
    fn ordinary_policy_uses_east_exit_and_releases_at_bounds_or_stale_state() {
        let (mut energy, mut state, mut ram) = prefix();
        let before = ram.clone();
        assert_eq!(energy.buttons(&state, &ram) & (LEFT | RIGHT), RIGHT);
        assert_eq!(ram, before, "policy only returns ordinary input");
        state.host_steps += 1;
        assert_eq!(energy.buttons(&state, &ram), 0);
        state.host_steps = PREFIX + TAIL;
        energy.previous = Some(state);
        sync(&state, &mut ram);
        assert_eq!(energy.buttons(&state, &ram), 0);
        energy.observe(&State { host_steps: PREFIX + TAIL + 1, ..state }, &ram);
        assert!(energy.invalid);
        assert!(!energy.complete());
    }

    #[cfg(feature = "private-checks")]
    pub(super) fn private_assertions() -> [(&'static str, fn()); 7] {
        [
            (
                "exact_prefix_is_required_first_and_is_not_repaired_by_later_valid_state",
                exact_prefix_is_required_first_and_is_not_repaired_by_later_valid_state as fn(),
            ),
            (
                "gaps_state_mismatch_skipped_rooms_and_wrong_elevator_area_fail_closed",
                gaps_state_mismatch_skipped_rooms_and_wrong_elevator_area_fail_closed as fn(),
            ),
            (
                "presence_checks_exact_slot_draw_block_and_bts_before_contact",
                presence_checks_exact_slot_draw_block_and_bts_before_contact as fn(),
            ),
            (
                "acquisition_rejects_synthetic_capacity_message_contact_and_unconsumed_input",
                acquisition_rejects_synthetic_capacity_message_contact_and_unconsumed_input as fn(),
            ),
            (
                "real_wait_and_acknowledgement_are_required_before_resumed_movement",
                real_wait_and_acknowledgement_are_required_before_resumed_movement as fn(),
            ),
            (
                "construction_moving_cycles_are_bounded_but_new_height_and_transition_waits_are_preserved",
                construction_moving_cycles_are_bounded_but_new_height_and_transition_waits_are_preserved as fn(),
            ),
            (
                "ordinary_policy_uses_east_exit_and_releases_at_bounds_or_stale_state",
                ordinary_policy_uses_east_exit_and_releases_at_bounds_or_stale_state as fn(),
            ),
        ]
    }
}

/// The seven existing prefix assertions; Construction captures retain their Cargo gates.
#[cfg(feature = "private-checks")]
pub(crate) fn private_assertions() -> [(&'static str, fn()); 7] {
    tests::private_assertions()
}
