//! Ordinary-input first missile continuation; no gameplay or inventory writes.
use sm_native_host::State;

const PREFIX: u64 = 23_036;
const TAIL: u64 = 8_192;
const ROOMS: [u16; 3] = [0x9e9f, 0x9f11, 0xa107];
const LEFT: u16 = 0x040;
const RIGHT: u16 = 0x080;
const DOWN: u16 = 0x020;
const UP: u16 = 0x010;

fn word(ram: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([ram[at], ram[at + 1]])
}
fn normal(state: &State, stage: usize) -> bool {
    state.game_state == 8 && state.area == 1 && state.room == ROOMS[stage]
}
fn morphed(ram: &[u8]) -> bool {
    ram[0xa1f] == 4 && matches!(word(ram, 0xa1c), 0x1d | 0x1e | 0x1f | 0x41)
}
fn grounded(ram: &[u8]) -> bool {
    word(ram, 0xb2e) == 0 && word(ram, 0xb2c) == 0 && word(ram, 0xb36) == 0 && word(ram, 0xb00) != 0
}
fn ball_form(ram: &[u8]) -> bool {
    matches!(ram[0xa1f], 4 | 8 | 0x11..=0x13)
}
fn message(ram: &[u8]) -> bool {
    word(ram, 0xdc8) == 2 && word(ram, 0x1c1f) == 2 && ram[0x614] != 0
}
fn clear(ram: &[u8]) -> bool {
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

#[derive(Clone, Copy)]
struct Orb {
    slot: usize,
    instruction: u16,
    link: u16,
    pre: u16,
    draw: u16,
}
impl Orb {
    fn read(ram: &[u8]) -> Option<Self> {
        let mut slots = (0..40).filter(|i| word(ram, 0x1c37 + 2 * i) == 0xef2f && word(ram, 0x1dc7 + 2 * i) == 0x22);
        let slot = slots.next()?;
        if slots.next().is_some() || word(ram, 0xdf0c + 2 * slot) != 0xa {
            return None;
        }
        Some(Self {
            slot,
            instruction: word(ram, 0x1d27 + 2 * slot),
            link: word(ram, 0xdebc + 2 * slot),
            pre: word(ram, 0x1cd7 + 2 * slot),
            draw: word(ram, 0xde6c + 2 * slot),
        })
    }
    // ROM84:E47C calls DFAF (shell) with return E484; E484 calls DFC7
    // (three 3-update burst frames) with return E488, then arms pickup E49F.
    // PLM handler stores the pointer AFTER a timed draw (pinned sm84:255).
    fn shell(self) -> bool {
        self.link == 0xe484
            && self.pre == 0xdf89
            && matches!(
                (self.instruction, self.draw),
                (0xdfb7, 0xa2c7) | (0xdfbb, 0xa2cd) | (0xdfbf, 0xa2d3) | (0xdfc3, 0xa2cd)
            )
    }
    fn revealed(self) -> bool {
        (self.link == 0xe488
            && self.pre == 0x86d0
            && matches!((self.instruction, self.draw), (0xdfcd, 0xa2b5) | (0xdfd1, 0xa2d9) | (0xdfd5, 0xa2b5)))
            || (self.link == 0xe49f
                && self.pre == 0xdf89
                && matches!((self.instruction, self.draw), (0xe497, 0xa2eb) | (0xe49b, 0xa2f1)))
    }
}

#[derive(Default)]
struct Movement {
    first: u64,
    last: u64,
    polls: u64,
    moving: u64,
    pixels: u64,
}
impl Movement {
    fn observe(&mut self, previous: &State, state: &State) -> bool {
        if state.polls == previous.polls {
            return false;
        }
        let pixels = state.x.checked_sub(previous.x);
        if state.polls != previous.polls + 1
            || state.updates != previous.updates + 1
            || state.requested_buttons & (LEFT | RIGHT) != RIGHT
            || state.effective_buttons & (LEFT | RIGHT) != RIGHT
            || state.joy1_current & (LEFT | RIGHT).reverse_bits() != RIGHT.reverse_bits()
            || pixels.is_none_or(|n| n > 16)
        {
            *self = Self::default();
            return false;
        }
        if self.polls == 0 || state.host_steps - self.first > 64 {
            *self = Self { first: previous.host_steps, ..Self::default() };
        }
        let pixels = pixels.unwrap();
        self.last = state.host_steps;
        self.polls += 1;
        self.moving += u64::from(pixels > 0);
        self.pixels += u64::from(pixels);
        self.polls >= 8 && self.moving >= 4 && self.pixels >= 16
    }
}

#[derive(Default)]
pub struct Missile {
    previous: Option<State>,
    previous_morphed: bool,
    previous_ball: bool,
    prefix_verified: bool,
    invalid: bool,
    stage: usize,
    rooms: [Option<u64>; 3],
    baseline_ammo: u16,
    baseline_capacity: u16,
    ammo: u16,
    capacity: u16,
    item: bool,
    health: u16,
    presentation_clear: bool,
    morph: Movement,
    morph_step: Option<u64>,
    shell_step: Option<u64>,
    shell_slot: Option<usize>,
    shot_step: Option<u64>,
    reveal_step: Option<u64>,
    acquired_step: Option<u64>,
    message_step: Option<u64>,
    message_waits: u64,
    message_end: Option<u64>,
    response: Movement,
    checkpoint: Option<u64>,
    stalled: u64,
    morph_recovering: bool,
    morph_recovery_jump: Option<u64>,
    morph_recovery_x: u16,
    morph_recovery_left: bool,
    jump_until: u64,
    jump_ready: u64,
    detour: Option<u16>,
    detour_y: u16,
    detour_right: bool,
}

impl Missile {
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
        self.ammo = word(ram, 0x9c6);
        self.capacity = word(ram, 0x9c8);
        self.presentation_clear = clear(ram);
        let item = ram[0xd874] & 4 != 0;
        if state.host_steps == PREFIX && self.previous.is_none() {
            self.baseline_ammo = self.ammo;
            self.baseline_capacity = self.capacity;
            self.prefix_verified = normal(state, 0)
                && state.loading_game_state == 5
                && (state.x, state.y, state.timer) == (1108, 651, 0)
                && (state.polls, state.updates, state.nmis, state.waits) == (21426, 21414, 23036, 1622)
                && (state.requested_buttons, state.effective_buttons, state.joy1_current, state.joy1_new)
                    == (0x41, 0x41, 0x8200, 0)
                && state.cpu_fallback_calls == 0
                && state.cpu_instructions == 0
                && self.health == 99
                && !item
                && self.ammo == 0
                && self.capacity == 0
                && word(ram, 0x9a2) & 4 != 0
                && word(ram, 0x9a4) & 4 != 0
                && word(ram, 0x9d2) == 0
                && clear(ram);
            if self.prefix_verified {
                self.rooms[0] = Some(PREFIX);
            }
        }
        if self.prefix_verified && state.host_steps > PREFIX {
            if state.host_steps > PREFIX + TAIL
                || self.health == 0
                || state.cpu_fallback_calls != 0
                || state.cpu_instructions != 0
                || word(ram, 0x9d2) != 0
                || word(ram, 0x9a2) & 4 == 0
                || word(ram, 0x9a4) & 4 == 0
            {
                self.invalid = true;
            }
            if state.game_state == 8 {
                match ROOMS.iter().position(|room| *room == state.room) {
                    Some(stage) if state.area == 1 && (stage == self.stage || stage == self.stage + 1) => {
                        if stage != self.stage {
                            if self.morph_step.is_none() {
                                self.invalid = true;
                            }
                            self.stage = stage;
                            self.rooms[stage] = Some(state.host_steps);
                            self.stalled = 0;
                            self.jump_until = 0;
                            self.jump_ready = 0;
                            self.detour = None;
                        }
                    }
                    _ => self.invalid = true,
                }
            }
            if let Some(previous) = self.previous {
                if self.morph_step.is_none() {
                    if normal(&previous, 0) && normal(state, 0) && self.previous_morphed && morphed(ram) && clear(ram) {
                        if self.morph.observe(&previous, state) {
                            self.morph_step = Some(state.host_steps);
                        }
                    } else {
                        self.morph = Movement::default();
                    }
                }
                let shoot = word(ram, 0x9b2).reverse_bits();
                let consumed_shot = state.polls > previous.polls
                    && state.updates > previous.updates
                    && state.effective_buttons & shoot != 0
                    && state.joy1_current & shoot.reverse_bits() != 0;
                if consumed_shot && (self.previous_ball || ball_form(ram) || self.acquired_step.is_some()) {
                    self.invalid = true;
                }
                if consumed_shot
                    && state.requested_buttons & shoot != 0
                    && normal(state, 2)
                    && self.shell_step.is_some()
                    && !item
                    && self.reveal_step.is_none()
                {
                    self.shot_step.get_or_insert(state.host_steps);
                }
                if state.polls > previous.polls {
                    self.stalled = if previous.room == state.room
                        && previous.game_state == 8
                        && state.game_state == 8
                        && (previous.x, previous.y) == (state.x, state.y)
                    {
                        self.stalled + 1
                    } else {
                        0
                    };
                }
            }
            if normal(state, 2)
                && self.morph_step.is_some()
                && !item
                && self.ammo == self.baseline_ammo
                && self.capacity == self.baseline_capacity
                && let Some(orb) = Orb::read(ram)
            {
                if orb.shell() {
                    self.shell_step.get_or_insert(state.host_steps);
                    self.shell_slot.get_or_insert(orb.slot);
                }
                if orb.revealed()
                    && self.shell_slot == Some(orb.slot)
                    && self.shell_step.is_some_and(|step| step < state.host_steps)
                    && self.shot_step.is_some_and(|step| step <= state.host_steps)
                {
                    self.reveal_step.get_or_insert(state.host_steps);
                }
            }
            if self.acquired_step.is_some()
                || item != self.item
                || self.ammo != self.baseline_ammo
                || self.capacity != self.baseline_capacity
            {
                if !item
                    || self.ammo != self.baseline_ammo + 5
                    || self.capacity != self.baseline_capacity + 5
                    || !normal(state, 2)
                    || self.reveal_step.is_none_or(|step| step >= state.host_steps)
                {
                    self.invalid = true;
                } else {
                    self.acquired_step.get_or_insert(state.host_steps);
                }
            }
            if self.acquired_step.is_some() && normal(state, 2) && message(ram) {
                self.message_step.get_or_insert(state.host_steps);
                if self.previous.is_some_and(|p| p.updates == state.updates) {
                    self.message_waits += 1;
                }
            }
            if self.message_step.is_some() && self.message_waits > 0 && normal(state, 2) && item && clear(ram) {
                self.message_end.get_or_insert(state.host_steps);
            }
            if self.checkpoint.is_none()
                && let Some(previous) = self.previous
            {
                if self.message_end.is_some_and(|step| previous.host_steps >= step)
                    && normal(&previous, 2)
                    && normal(state, 2)
                    && item
                    && clear(ram)
                {
                    if self.response.observe(&previous, state) {
                        self.checkpoint = Some(state.host_steps);
                    }
                } else {
                    self.response = Movement::default();
                }
            }
        }
        self.item = item;
        self.previous_morphed = morphed(ram);
        self.previous_ball = ball_form(ram);
        self.previous = Some(*state);
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
        // Message2 has a360 NMI-only wait: update-based pulses cannot dismiss it.
        if self.acquired_step.is_some() && message(ram) {
            return if state.host_steps % 16 < 4 { jump } else { 0 };
        }
        if !normal(state, self.stage) {
            return 0;
        }
        if self.acquired_step.is_some() {
            return if self.message_end.is_some() && clear(ram) { RIGHT | run } else { 0 };
        }
        let direction = match self.stage {
            0 => {
                // author03's captured room has a one-block-high passage at
                // (76,44), blocked by C19A/BTS04. Native94:9EA6 selects D074;
                // its B3C1/CBB7 setup/script changes C->8 during the break and
                // finally draws air. Crouch27 shoots at centerY+1 (90:BA56),
                // inside704..719; standing fire/jumps hit the solid roof.
                let entrance = 44 * 128 + 76;
                if self.morph_step.is_some()
                    && word(ram, 0x7a5) == 128
                    && (1184..1288).contains(&state.x)
                    && (688..=720).contains(&state.y)
                {
                    match word(ram, 0x10002 + 2 * entrance) >> 12 {
                        0xc if ram[0x16402 + entrance] == 4 => {
                            if ball_form(ram) {
                                return UP;
                            }
                            if ram[0xa1f] == 0x0f || !grounded(ram) {
                                return 0;
                            }
                            if word(ram, 0xa1c) == 0x27 && ram[0xa1f] == 5 {
                                return if state.updates % 12 < 2 { shoot } else { 0 };
                            }
                            if ram[0xa1f] == 5 {
                                return RIGHT;
                            }
                            return if state.host_steps % 16 < 4 { DOWN } else { 0 };
                        }
                        8 => return 0,
                        0 => {
                            if ball_form(ram) {
                                return RIGHT;
                            }
                            return if state.host_steps % 16 < 4 { DOWN } else { 0 };
                        }
                        _ => {}
                    }
                }
                // author01 reaches ground-ball x1099/y713 against a pedestal
                // before earning any movement proof. Recovery cannot depend
                // on that proof: stand, clear the obstruction, then try Morph
                // again. This changes only authored controls, never evidence.
                if self.morph_recovering {
                    let direction = if self.morph_recovery_left { LEFT } else { RIGHT };
                    if ball_form(ram) || ram[0xa1f] == 0x0f {
                        return UP;
                    }
                    if self.morph_recovery_jump.is_none() && !grounded(ram) {
                        return direction | run;
                    }
                    if self.morph_recovery_jump.is_none() {
                        self.morph_recovery_x = state.x;
                    }
                    let start = *self.morph_recovery_jump.get_or_insert(state.updates);
                    if state.updates < start + 24 {
                        return direction | run | jump;
                    }
                    if state.updates < start + 60 || !grounded(ram) {
                        return direction | run;
                    }
                    // author02's low overhang at x1211 defeats right jumps.
                    // A completed timer is not successful repositioning: move
                    // back out once before making another ground Morph trial.
                    if !self.morph_recovery_left && state.x.saturating_sub(self.morph_recovery_x) < 16 {
                        self.morph_recovery_left = true;
                        self.morph_recovery_jump = None;
                        return LEFT | run;
                    }
                    self.morph_recovering = false;
                    self.morph_recovery_jump = None;
                    self.morph_recovery_left = false;
                }
                if self.morph_step.is_none() && morphed(ram) && grounded(ram) && self.stalled >= 24 {
                    self.morph_recovering = true;
                    return UP;
                }
                if self.morph_step.is_none() && !ball_form(ram) {
                    return if state.host_steps % 16 < 4 { DOWN } else { 0 };
                }
                if ball_form(ram) {
                    if self.morph_step.is_some() && (state.x >= 1850 || self.stalled >= 24) {
                        return UP;
                    }
                    return RIGHT;
                }
                RIGHT
            }
            1 => {
                if word(ram, 0x7a5) == 16 && word(ram, 0x7a7) == 32 {
                    return self.construction_buttons(state, ram, jump, shoot);
                }
                if ball_form(ram) {
                    return UP;
                }
                if state.y >= 320 {
                    LEFT
                } else {
                    let target = self.descent_target(state);
                    toward(state.x, target)
                }
            }
            2 => {
                if ball_form(ram) {
                    return UP;
                }
                // Approach from the right, facing the orb before ordinary fire.
                if self.reveal_step.is_some() {
                    toward(state.x, 72)
                } else if state.x > 128 {
                    LEFT
                } else if state.x < 112 {
                    RIGHT
                } else {
                    LEFT
                }
            }
            _ => return 0,
        };
        if direction != 0 && grounded(ram) && self.stalled >= 12 && state.updates >= self.jump_ready {
            self.jump_until = state.updates + 24;
            self.jump_ready = state.updates + 60;
        }
        let mut pad = direction;
        if self.stage != 2 && direction != 0 {
            pad |= run;
        }
        if state.updates < self.jump_until {
            pad |= jump;
        }
        if state.updates % 12 < 2
            && matches!(ram[0xa1f], 0 | 1 | 2 | 5 | 6 | 0x15)
            && self.reveal_step.is_none()
            && ((self.stage == 0 && state.x >= 1850)
                || (self.stage == 1 && state.y >= 320)
                || (self.stage == 2 && direction == LEFT))
        {
            pad |= shoot;
        }
        pad
    }
    fn construction_buttons(&mut self, state: &State, ram: &[u8], jump: u16, shoot: u16) -> u16 {
        // author04's actual map: shot floors at rows8/10/12/16, then an
        // open shaft in columns8/9 through rows21..24. Row25 is the low west
        // passage; its floor is row26, so standing center395 is its bottom.
        if state.y >= 395 {
            if ball_form(ram) {
                return if state.x > 64 { LEFT } else { UP };
            }
            if ram[0xa1f] == 0x0f {
                return 0;
            }
            if state.x > 64 {
                return if state.host_steps % 16 < 4 { DOWN } else { 0 };
            }
            return LEFT | if state.updates % 12 < 2 { shoot } else { 0 };
        }
        if ball_form(ram) {
            return UP;
        }
        if ram[0xa1f] == 0x0f {
            return 0;
        }
        // A one-block hole needs Samus's radius inside its edges:120 centers
        // column7, whereas the old target128 straddled two shot blocks.
        let target = if state.y < 272 { 120 } else { 144 };
        let direction = if state.x.abs_diff(target) <= 2 {
            0
        } else if state.x < target {
            RIGHT
        } else {
            LEFT
        };
        if direction != 0 {
            if grounded(ram) && self.stalled >= 12 && state.updates >= self.jump_ready {
                self.jump_until = state.updates + 24;
                self.jump_ready = state.updates + 60;
            }
            return direction | if state.updates < self.jump_until { jump } else { 0 };
        }
        if grounded(ram) {
            let floor_row = usize::from(state.y + word(ram, 0xb00)) / 16;
            let block = floor_row * 16 + usize::from(state.x) / 16;
            if floor_row < 32 && word(ram, 0x10002 + 2 * block) >> 12 == 0xc && state.updates >= self.jump_ready {
                self.jump_until = state.updates + 12;
                self.jump_ready = state.updates + 30;
                return jump;
            }
            return 0;
        }
        // Down while airborne selects straight-down aim; wait for the actual
        // pose before firing so an earlier horizontal pose cannot waste shots.
        let mut pad = DOWN;
        if state.updates < self.jump_until {
            pad |= jump;
        }
        if matches!(word(ram, 0xa1c), 0x17 | 0x18 | 0x2d | 0x2e) && state.updates % 12 < 2 {
            pad |= shoot;
        }
        pad
    }
    fn descent_target(&mut self, state: &State) -> u16 {
        if let Some(target) = self.detour {
            if state.x.abs_diff(target) <= 4 || state.y.saturating_sub(self.detour_y) >= 16 {
                self.detour = None;
            } else {
                return target;
            }
        }
        if self.stalled >= 24 {
            let target = if self.detour_right { 208 } else { 48 };
            self.detour_right = !self.detour_right;
            self.detour_y = state.y;
            self.detour = Some(target);
            target
        } else {
            128
        }
    }
    pub fn revealed(&self) -> bool {
        self.reveal_step.is_some()
    }
    pub fn acquired(&self) -> bool {
        self.acquired_step.is_some()
    }
    pub fn complete(&self) -> bool {
        self.prefix_verified
            && !self.invalid
            && self.rooms.iter().all(Option::is_some)
            && self.morph_step.is_some()
            && self.revealed()
            && self.acquired()
            && self.item
            && self.ammo == self.baseline_ammo + 5
            && self.capacity == self.baseline_capacity + 5
            && self.message_step.is_some()
            && self.message_waits > 0
            && self.message_end.is_some()
            && self.checkpoint.is_some()
            && self.health > 0
            && self.presentation_clear
            && self.previous.as_ref().is_some_and(|state| normal(state, 2))
    }
    pub fn report(&self, report: &mut sm_binary_evidence::Map) {
        for (key, value) in [
            ("checkpoint", self.complete()),
            ("prefix_verified", self.prefix_verified),
            ("invalid", self.invalid),
            ("revealed", self.revealed()),
            ("acquired", self.acquired()),
            ("item_bit", self.item),
        ] {
            report.insert(format!("missile_{key}"), value.into());
        }
        for (key, value) in [
            ("stage", self.stage as u64),
            ("health", u64::from(self.health)),
            ("baseline_ammo", u64::from(self.baseline_ammo)),
            ("baseline_capacity", u64::from(self.baseline_capacity)),
            ("ammo", u64::from(self.ammo)),
            ("capacity", u64::from(self.capacity)),
            ("message_waits", self.message_waits),
        ] {
            report.insert(format!("missile_{key}"), (value as i64).into());
        }
        for (key, step) in [
            ("morph_step", self.morph_step),
            ("shell_step", self.shell_step),
            ("shot_step", self.shot_step),
            ("revealed_step", self.reveal_step),
            ("acquired_step", self.acquired_step),
            ("message_step", self.message_step),
            ("message_end_step", self.message_end),
            ("checkpoint_step", self.checkpoint),
        ] {
            report.insert(format!("missile_{key}"), step.map_or(-1, |s| s as i64).into());
        }
        for (key, movement) in [("morph", &self.morph), ("response", &self.response)] {
            for (suffix, value) in [
                ("polls", movement.polls),
                ("moving_polls", movement.moving),
                ("right_pixels", movement.pixels),
                ("first_step", movement.first),
                ("last_step", movement.last),
            ] {
                report.insert(format!("missile_{key}_{suffix}"), (value as i64).into());
            }
        }
        report.insert(
            "missile_room_steps".into(),
            self.rooms
                .iter()
                .map(|s| sm_binary_evidence::Value::Integer(s.map_or(-1, |s| s as i64)))
                .collect::<Vec<_>>()
                .into(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn set(ram: &mut [u8], at: usize, value: u16) {
        ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
    }
    fn prefix() -> (Missile, State, Vec<u8>) {
        let state = State {
            host_steps: PREFIX,
            game_state: 8,
            loading_game_state: 5,
            area: 1,
            room: ROOMS[0],
            x: 1108,
            y: 651,
            polls: 21426,
            updates: 21414,
            nmis: 23036,
            waits: 1622,
            requested_buttons: 0x41,
            effective_buttons: 0x41,
            joy1_current: 0x8200,
            ..State::default()
        };
        let mut ram = vec![0; 0x20000];
        for (at, value) in [(0x9c2, 99), (0x9a2, 4), (0x9a4, 4), (0xb00, 21)] {
            set(&mut ram, at, value);
        }
        for (at, native) in [(0x9b6, 1_u16), (0x9b4, 0x100), (0x9b2, 0x200)] {
            set(&mut ram, at, native.reverse_bits());
        }
        let mut missile = Missile::default();
        missile.observe(&state, &ram);
        (missile, state, ram)
    }
    fn advance(state: &mut State, pad: u16) {
        state.host_steps += 1;
        state.polls += 1;
        state.updates += 1;
        state.nmis += 1;
        state.requested_buttons = pad;
        state.effective_buttons = pad;
        state.joy1_current = pad.reverse_bits();
    }
    fn morph(missile: &mut Missile, state: &mut State, ram: &mut [u8]) {
        set(ram, 0xa1c, 0x1d);
        ram[0xa1f] = 4;
        for _ in 0..9 {
            advance(state, RIGHT);
            state.x += 2;
            missile.observe(state, ram);
        }
    }
    fn route(missile: &mut Missile, state: &mut State, ram: &mut [u8]) {
        morph(missile, state, ram);
        set(ram, 0xa1c, 1);
        ram[0xa1f] = 0;
        for room in &ROOMS[1..] {
            advance(state, 0);
            state.room = *room;
            state.x = 120;
            state.y = 107;
            missile.observe(state, ram);
        }
    }
    fn orb(ram: &mut [u8], burst: bool) {
        for (at, value) in [
            (0x1c37, 0xef2f),
            (0x1dc7, 0x22),
            (0xdf0c, 0xa),
            (0x1d27, if burst { 0xdfcd } else { 0xdfb7 }),
            (0xdebc, if burst { 0xe488 } else { 0xe484 }),
            (0x1cd7, if burst { 0x86d0 } else { 0xdf89 }),
            (0xde6c, if burst { 0xa2b5 } else { 0xa2c7 }),
        ] {
            set(ram, at, value);
        }
    }
    fn reveal(missile: &mut Missile, state: &mut State, ram: &mut [u8]) {
        orb(ram, false);
        advance(state, 0);
        missile.observe(state, ram);
        advance(state, LEFT | 0x200);
        missile.observe(state, ram);
        orb(ram, true);
        advance(state, 0);
        missile.observe(state, ram);
    }
    fn pickup(missile: &mut Missile, state: &mut State, ram: &mut [u8]) {
        ram[0xd874] = 4;
        for (at, value) in [(0x9c6, 5), (0x9c8, 5), (0xdc8, 2), (0x1c1f, 2)] {
            set(ram, at, value);
        }
        ram[0x614] = 2;
        advance(state, 0);
        missile.observe(state, ram);
        state.host_steps += 1;
        state.nmis += 1;
        state.waits += 1;
        missile.observe(state, ram);
    }
    fn finish(missile: &mut Missile, state: &mut State, ram: &mut [u8]) {
        set(ram, 0xdc8, 0);
        ram[0x614] = 0;
        advance(state, 0);
        missile.observe(state, ram);
        for _ in 0..8 {
            advance(state, RIGHT);
            state.x += 2;
            missile.observe(state, ram);
        }
    }
    #[test]
    fn complete_requires_morph_shell_reveal_pickup_message_and_resumed_control() {
        let (mut missile, mut state, mut ram) = prefix();
        route(&mut missile, &mut state, &mut ram);
        assert!(missile.morph_step.is_some());
        reveal(&mut missile, &mut state, &mut ram);
        assert!(missile.revealed());
        assert!(!missile.acquired());
        pickup(&mut missile, &mut state, &mut ram);
        assert!(missile.acquired());
        assert!(!missile.complete());
        finish(&mut missile, &mut state, &mut ram);
        assert!(missile.complete());
        let mut report = sm_binary_evidence::Map::new();
        missile.report(&mut report);
        assert_eq!(report["missile_morph_right_pixels"].as_integer(), Some(16));
        assert_eq!(report["missile_response_right_pixels"].as_integer(), Some(16));
    }
    #[test]
    fn inventory_stationary_morph_wrong_pose_and_unconsumed_motion_are_not_ability_use() {
        for variant in 0..4 {
            let (mut missile, mut state, mut ram) = prefix();
            set(&mut ram, 0xa1c, 0x1d);
            ram[0xa1f] = 4;
            for _ in 0..12 {
                advance(&mut state, RIGHT);
                match variant {
                    0 => {
                        ram[0xa1f] = 0;
                        state.x += 2;
                    }
                    1 => {}
                    2 => {
                        set(&mut ram, 0xa1c, 1);
                        state.x += 2;
                    }
                    _ => {
                        state.x += 2;
                        state.effective_buttons = 0;
                    }
                }
                missile.observe(&state, &ram);
            }
            assert!(missile.morph_step.is_none());
            advance(&mut state, 0);
            state.room = ROOMS[1];
            missile.observe(&state, &ram);
            assert!(missile.invalid);
        }
    }
    #[test]
    fn reveal_rejects_missing_shell_shot_wrong_slot_draw_and_existing_item() {
        for variant in 0..6 {
            let (mut missile, mut state, mut ram) = prefix();
            route(&mut missile, &mut state, &mut ram);
            orb(&mut ram, false);
            if variant != 0 {
                advance(&mut state, 0);
                missile.observe(&state, &ram);
            }
            if variant != 1 {
                advance(&mut state, 0x200);
                missile.observe(&state, &ram);
            }
            orb(&mut ram, true);
            match variant {
                2 => set(&mut ram, 0xde6c, 0xa2c7),
                3 => set(&mut ram, 0xdebc, 0xe484),
                4 => {
                    set(&mut ram, 0x1c39, 0xef2f);
                    set(&mut ram, 0x1dc9, 0x22);
                }
                5 => ram[0xd874] = 4,
                _ => {}
            }
            advance(&mut state, 0);
            missile.observe(&state, &ram);
            assert!(!missile.revealed(), "variant {variant}");
            assert!(!missile.complete());
        }
    }
    #[test]
    fn pickup_requires_all_effects_and_waited_message_two() {
        for variant in 0..6 {
            let (mut missile, mut state, mut ram) = prefix();
            route(&mut missile, &mut state, &mut ram);
            reveal(&mut missile, &mut state, &mut ram);
            ram[0xd874] = if variant == 0 { 0 } else { 4 };
            set(&mut ram, 0x9c6, if variant == 1 { 0 } else { 5 });
            set(&mut ram, 0x9c8, if variant == 2 { 0 } else { 5 });
            set(&mut ram, 0xdc8, if variant == 3 { 9 } else { 2 });
            set(&mut ram, 0x1c1f, if variant == 3 { 9 } else { 2 });
            ram[0x614] = 2;
            advance(&mut state, 0);
            missile.observe(&state, &ram);
            if variant != 4 {
                state.host_steps += 1;
                state.nmis += 1;
                missile.observe(&state, &ram);
            }
            if variant == 5 {
                set(&mut ram, 0xa78, 1);
            }
            finish(&mut missile, &mut state, &mut ram);
            assert!(!missile.complete(), "variant {variant}");
        }
    }
    #[test]
    fn policy_releases_down_never_fires_morphed_and_dismisses_by_host_steps() {
        let (mut missile, mut state, mut ram) = prefix();
        state.host_steps = PREFIX + 4;
        assert_eq!(missile.buttons(&state, &ram), DOWN);
        state.host_steps += 4;
        assert_eq!(missile.buttons(&state, &ram), 0);
        set(&mut ram, 0xa1c, 0x1d);
        ram[0xa1f] = 4;
        assert_eq!(missile.buttons(&state, &ram), RIGHT);
        ram[0xa1f] = 8;
        state.x = 1900;
        state.updates = 24;
        assert_eq!(missile.buttons(&state, &ram) & 0x200, 0, "falling Morph must not request bombs");
        missile.acquired_step = Some(state.host_steps);
        set(&mut ram, 0xdc8, 2);
        set(&mut ram, 0x1c1f, 2);
        ram[0x614] = 2;
        state.host_steps = PREFIX + 4;
        assert_eq!(missile.buttons(&state, &ram), 0x100);
        state.host_steps += 4;
        assert_eq!(missile.buttons(&state, &ram), 0);
    }
    #[test]
    fn prefix_room_order_counter_discontinuity_and_hud_selection_fail_closed() {
        for variant in 0..5 {
            let (mut missile, mut state, mut ram) = prefix();
            match variant {
                0 => {
                    missile = Missile::default();
                    state.x += 1;
                }
                1 => {
                    advance(&mut state, 0);
                    state.room = ROOMS[2];
                }
                2 => state.host_steps += 2,
                3 => {
                    advance(&mut state, 0);
                    set(&mut ram, 0x9d2, 1);
                }
                _ => {
                    advance(&mut state, 0);
                    state.cpu_fallback_calls = 1;
                }
            }
            missile.observe(&state, &ram);
            assert_eq!(missile.buttons(&state, &ram), 0);
            assert!(!missile.complete());
        }
    }

    #[test]
    fn stalled_morph_before_ability_proof_unmorphs_and_jumps_clear_of_pedestal() {
        let (mut missile, mut state, mut ram) = prefix();
        // author01: right is consumed, but the ground ball is blocked by the
        // thin pedestal at x1099/y713. No material Morph use has occurred.
        state.x = 1099;
        state.y = 713;
        set(&mut ram, 0xa1c, 0x1e);
        set(&mut ram, 0xb00, 7);
        ram[0xa1f] = 4;
        for _ in 0..26 {
            advance(&mut state, RIGHT);
            missile.observe(&state, &ram);
        }
        assert!(missile.morph_step.is_none());
        assert_eq!(missile.buttons(&state, &ram), UP, "blocked ball must recover even before Morph proof");
        // After native unmorph, ordinary right+jump clears the pedestal. The
        // recovery must not immediately enter the same Down/morph trap again.
        set(&mut ram, 0xa1c, 1);
        set(&mut ram, 0xb00, 21);
        ram[0xa1f] = 0;
        state.y = 699;
        advance(&mut state, UP);
        missile.observe(&state, &ram);
        assert_eq!(missile.buttons(&state, &ram), RIGHT | 1 | 0x100);
        state.updates += 24;
        assert_eq!(missile.buttons(&state, &ram) & 0x100, 0, "recovery jump is bounded");
        assert!(missile.morph_step.is_none());
        assert!(!missile.complete());
    }

    #[test]
    fn unsuccessful_right_recovery_repositions_left_before_retrying_morph() {
        let (mut missile, mut state, mut ram) = prefix();
        state.x = 1211;
        state.y = 713;
        set(&mut ram, 0xa1c, 0x1e);
        set(&mut ram, 0xb00, 7);
        ram[0xa1f] = 4;
        for _ in 0..26 {
            advance(&mut state, RIGHT);
            missile.observe(&state, &ram);
        }
        assert_eq!(missile.buttons(&state, &ram), UP);
        set(&mut ram, 0xa1c, 0x27);
        set(&mut ram, 0xb00, 16);
        ram[0xa1f] = 5;
        state.y = 704;
        advance(&mut state, UP);
        missile.observe(&state, &ram);
        assert_eq!(missile.buttons(&state, &ram), RIGHT | 1 | 0x100);
        // author02's x1211 jump hits the overhang and lands without any
        // horizontal progress, then keeps retrying the same Morph/jump cycle.
        for elapsed in 1..=60 {
            advance(&mut state, if elapsed < 24 { RIGHT | 1 | 0x100 } else { RIGHT | 1 });
            state.y = 699;
            set(&mut ram, 0xa1c, 0x89);
            set(&mut ram, 0xb00, 21);
            ram[0xa1f] = 0x15;
            missile.observe(&state, &ram);
        }
        let pad = missile.buttons(&state, &ram);
        assert_eq!(pad & (LEFT | RIGHT | DOWN), LEFT, "failed right recovery must leave the blocked spot");
        assert_eq!(pad & 0x200, 0);
        assert!(missile.morph_step.is_none());
        assert!(!missile.complete());
    }

    #[test]
    fn reached_morph_tunnel_requires_crouched_shot_then_ball_through_open_aperture() {
        let (mut missile, mut state, mut ram) = prefix();
        morph(&mut missile, &mut state, &mut ram);
        assert!(missile.morph_step.is_some());
        // author03 final collision map: width128, tile(76,44)=C19A/BTS04,
        // solids immediately above and air behind a one-block-high entrance.
        set(&mut ram, 0x7a5, 128);
        let entrance = 44 * 128 + 76;
        set(&mut ram, 0x10002 + 2 * entrance, 0xc19a);
        ram[0x16402 + entrance] = 4;
        for row in [42, 43] {
            for column in 76..80 {
                set(&mut ram, 0x10002 + 2 * (row * 128 + column), 0x81e9);
            }
        }
        state.x = 1211;
        state.y = 704;
        state.updates = 24;
        set(&mut ram, 0xa1c, 0x27);
        set(&mut ram, 0xb00, 16);
        ram[0xa1f] = 5;
        assert_eq!(missile.buttons(&state, &ram), 0x200, "crouched beam must open the low shot block");
        state.updates += 2;
        assert_eq!(missile.buttons(&state, &ram), 0, "ordinary fire pulse releases without standing");
        set(&mut ram, 0x10002 + 2 * entrance, 0x819a);
        assert_eq!(missile.buttons(&state, &ram), 0, "wait for native block-break animation");
        set(&mut ram, 0x10002 + 2 * entrance, 0x00ff);
        state.host_steps = PREFIX + 20;
        assert_eq!(missile.buttons(&state, &ram), DOWN, "open low aperture requires Morph again");
        set(&mut ram, 0xa1c, 0x1e);
        set(&mut ram, 0xb00, 7);
        ram[0xa1f] = 4;
        state.y = 713;
        assert_eq!(missile.buttons(&state, &ram), RIGHT);
        assert!(!missile.complete());
    }

    #[test]
    fn construction_shot_floor_requires_aligned_airborne_down_fire() {
        let (mut missile, mut state, mut ram) = prefix();
        morph(&mut missile, &mut state, &mut ram);
        advance(&mut state, 0);
        state.room = ROOMS[1];
        state.x = 120;
        state.y = 107;
        set(&mut ram, 0x7a5, 16);
        set(&mut ram, 0x7a7, 32);
        set(&mut ram, 0xa1c, 1);
        set(&mut ram, 0xb00, 21);
        ram[0xa1f] = 0;
        // Captured author04 floor: respawning shot blocks across row8,
        // nonrespawning shot floor across row10. x120 centers column7.
        for (row, columns, bts) in [(8, 6..10, 0), (10, 4..12, 4)] {
            for column in columns {
                set(&mut ram, 0x10002 + 2 * (row * 16 + column), 0xc19a);
                ram[0x16402 + row * 16 + column] = bts;
            }
        }
        missile.observe(&state, &ram);
        assert_eq!(
            missile.buttons(&state, &ram) & (LEFT | RIGHT | 0x100),
            0x100,
            "jump over the shot floor at the block center, not sideways toward the boundary"
        );
        state.y = 100;
        state.updates = 24;
        set(&mut ram, 0xa1c, 0x17);
        set(&mut ram, 0xb2e, 1);
        ram[0xa1f] = 2;
        let pad = missile.buttons(&state, &ram);
        assert_eq!(pad & (LEFT | RIGHT | DOWN | 0x200), DOWN | 0x200);
        // Do not turn west above the bottom of the narrow lower shaft.
        state.x = 144;
        state.y = 320;
        set(&mut ram, 0xa1c, 0x2d);
        ram[0xa1f] = 6;
        assert_eq!(missile.buttons(&state, &ram) & (LEFT | RIGHT), 0);
        assert!(!missile.complete());
    }
}
