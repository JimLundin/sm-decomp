//! Fixed-point movement and complete native tail mechanics.
use super::*;

const MOVEMENT_FLAGS: [u8; 28] = [
    0x80, 0x80, 0x80, 0, 0xff, 0x80, 0x80, 0xff, 0xff, 0xff, 0x80, 0, 0, 0x80, 0x80, 0x80, 0x80, 0xff, 0xff, 0xff,
    0x80, 0x80, 0, 0x80, 0x80, 0, 0, 0x80,
];
impl<H: Host> Context<'_, '_, H> {
    pub(super) fn can_carry(&self) -> Result<bool> {
        MOVEMENT_FLAGS
            .get(usize::from(self.ram.byte(0xa1f)))
            .map(|value| value & 0x80 != 0)
            .ok_or("Ridley Samus movement type outside native table")
    }
    pub(super) fn morph_carry(&self) -> Result<bool> {
        MOVEMENT_FLAGS
            .get(usize::from(self.ram.byte(0xa1f)))
            .map(|value| value & 0x40 != 0)
            .ok_or("Ridley Samus movement type outside native table")
    }
    pub(super) fn speed_profile(&self) -> Result<u16> {
        [4, 8, 10, 12].get(usize::from(self.v(0x12))).copied().ok_or("Ridley speed tier outside native table")
    }
    pub(super) fn force_axis(
        &mut self,
        slot: u16,
        destination: u16,
        divisor: u16,
        braking: Option<u16>,
        vertical: bool,
    ) -> Result<()> {
        let e = record(slot)?;
        let position = e + if vertical { 6 } else { 2 };
        let velocity = e + if vertical { 0x34 } else { 0x32 };
        let delta = self.w(position).wrapping_sub(destination);
        if delta == 0 {
            return Ok(());
        }
        let distance = if negative(delta) { delta.wrapping_neg() } else { delta };
        let quotient = if divisor as u8 == 0 { 0xffff } else { distance / u16::from(divisor as u8) };
        let step = quotient.max(1);
        let mut speed = self.w(velocity);
        if negative(delta) {
            if negative(speed) {
                speed = speed.wrapping_add(match braking {
                    Some(extra) => extra.wrapping_add(8).wrapping_add(step),
                    None => step.wrapping_mul(2),
                });
            }
            speed = speed.wrapping_add(step);
            if !negative(speed.wrapping_sub(1280)) {
                speed = 1280;
            }
        } else {
            if !negative(speed) {
                speed = speed.wrapping_sub(match braking {
                    Some(extra) => extra.wrapping_add(8).wrapping_add(step),
                    None => step.wrapping_mul(2),
                });
            }
            speed = speed.wrapping_sub(step);
            if negative(speed.wrapping_add(1280)) {
                speed = (-1280i16) as u16;
            }
        }
        self.set(velocity, speed);
        Ok(())
    }
    pub(super) fn seek(&mut self, slot: u16, profile: u16, braking: Option<u16>, x: u16, y: u16) -> Result<()> {
        if profile >= 16 {
            return Err("Ridley movement profile exceeds native table");
        }
        let table = if braking.is_some() { 0xd61f } else { 0xd712 };
        let divisor = u16::from(rom_byte(self.rom, 0xa60000 | (table + u32::from(profile)))?);
        self.force_axis(slot, x, divisor, braking, false)?;
        self.force_axis(slot, y, divisor, braking, true)
    }
    pub(super) fn arc(&mut self, acceleration: u16, target_angle: u16, target_speed: u16) {
        let speed = self.v(0x0b);
        if speed != target_speed {
            let value = if negative(target_speed.wrapping_sub(speed)) {
                let candidate = speed.wrapping_sub(32);
                if negative(candidate.wrapping_sub(target_speed)) { target_speed } else { candidate }
            } else {
                let candidate = speed.wrapping_add(32);
                if !negative(candidate.wrapping_sub(target_speed)) { target_speed } else { candidate }
            };
            self.sv(0x0b, value);
        }
        let mut angle = self.v(0x0a).wrapping_add(acceleration);
        if if negative(acceleration) {
            negative(angle.wrapping_sub(target_angle))
        } else {
            !negative(angle.wrapping_sub(target_angle))
        } {
            angle = target_angle;
        }
        self.sv(0x0a, angle);
        self.sb(self.component(self.v(0x0b), (angle >> 8) as u8));
        self.sc(self.component(self.v(0x0b), ((angle >> 8) as u8).wrapping_add(64)));
    }
    pub(super) fn integrate(&mut self, slot: u16, bounded: bool) -> Result<()> {
        let e = record(slot)?;
        if bounded {
            self.sv(0x1f, 0);
        }
        for (position, fraction, velocity, lower, upper, low_flag, high_flag) in
            [(2, 4, 0x32, 0x22, 0x23, 1, 2), (6, 8, 0x34, 0x20, 0x21, 4, 8)]
        {
            let speed = self.w(e + velocity);
            let carry = u16::from(self.ram.byte(e + fraction + 1)) + u16::from(speed as u8);
            self.ram.set_byte(e + fraction + 1, carry as u8);
            let mut value =
                self.w(e + position).wrapping_add((speed >> 8) as u8 as i8 as i16 as u16).wrapping_add(carry >> 8);
            if bounded {
                if negative(value.wrapping_sub(self.v(lower))) {
                    if position == 2 && self.w(0x79f) != 2 && abs(self.w(e + 0x32)).max(abs(self.w(e + 0x34))) >= 0x280
                    {
                        self.set(0x183e, 33);
                        self.set(0x1840, 12);
                    }
                    value = self.v(lower);
                    self.set(e + velocity, 0);
                    self.sv(0x1f, low_flag);
                } else if !negative(value.wrapping_sub(self.v(upper))) {
                    value = self.v(upper);
                    self.set(e + velocity, 0);
                    self.sv(0x1f, high_flag);
                }
            }
            self.set(e + position, value);
        }
        Ok(())
    }
    pub(super) fn update_wing_animation(&mut self) {
        let x = abs(self.b());
        let total = x.wrapping_add(abs(self.c()));
        if total == 0 {
            self.sv(8, 0);
        } else {
            let mut max = total.wrapping_sub(x);
            if negative(max.wrapping_sub(x)) {
                max = x;
            }
            let index = usize::from((max.wrapping_mul(4) & 0xf00) >> 8).min(7);
            let mut rate = [12, 14, 16, 18, 28, 32, 40, 48][index];
            if !negative(self.c()) {
                rate >>= 1;
            }
            self.sv(8, rate);
        }
        let value = self.v(9).wrapping_sub(self.v(8));
        self.sv(9, value);
        if negative(value) {
            self.sv(9, 32);
            let next = self.v(7).wrapping_add(1);
            self.sv(7, if next >= 10 { 0 } else { next });
        }
    }
    pub(super) fn tail_copy(&mut self, destination: usize, source: u16) -> Result<()> {
        for i in 0..7 {
            self.set(destination + i * 20, self.table(source, i as u16)?);
        }
        Ok(())
    }
    pub(super) fn initialize_tail(&mut self) -> Result<()> {
        for (index, value) in [(10, 1), (11, 16368), (12, 16448), (13, 0xffff), (14, 0xffff), (9, 240), (15, 16)] {
            self.st(index, value);
        }
        for index in (17..=77).step_by(10) {
            self.st(index, 17);
        }
        for (dst, src) in [(0x2024, 0xd36e), (0x2026, 0xd37c), (0x202a, 0xd38a), (0x202c, 0xd398), (0x202e, 0xd3a6)] {
            self.tail_copy(dst, src)?;
        }
        self.tail_flags(0);
        Ok(())
    }
    pub(super) fn tail_flags(&mut self, value: u16) {
        for index in (16..=76).step_by(10) {
            self.st(index, value);
        }
    }
    pub(super) fn flip_tail(&mut self) -> Result<()> {
        self.tail_copy(0x2026, 0xd37c)?;
        for index in (0..=60).step_by(10) {
            self.st(index + 21, 0x8000u16.wrapping_sub(self.t(index + 21)));
            self.st(index + 18, self.t(index + 18) | 0x8000);
        }
        Ok(())
    }
    pub(super) fn all_tail(&self) -> bool {
        (16..=76).step_by(10).fold(0xffff, |value, index| value & self.t(index)) != 0
    }
    pub(super) fn no_tail(&self) -> bool {
        (16..=76).step_by(10).all(|index| self.t(index) == 0)
    }
    pub(super) fn tail_speed(&mut self) -> Result<()> {
        let o = self.v(0x10);
        if o > 2 {
            return Err("Ridley orientation table extent");
        }
        self.st(11, self.table(0xcc12, o)?);
        self.st(12, self.table(0xcc18, o)?);
        Ok(())
    }
    pub(super) fn tail_aim_target(&self) -> Option<(u16, u16)> {
        if self.w(0xcce) == 0 {
            return None;
        }
        (0..10).step_by(2).find_map(|k| {
            if ![1, 2].contains(&((self.w(0xc18 + k) >> 8) & 15)) {
                return None;
            }
            let dx = abs(self.w(0xb64 + k).wrapping_sub(self.t(82)));
            let dy = abs(self.w(0xb78 + k).wrapping_sub(self.t(83)));
            ((dx < self.w(0xbb4 + k) || dx.wrapping_sub(self.w(0xbb4 + k)) < 64)
                && (dy < self.w(0xbc8 + k) || dy.wrapping_sub(self.w(0xbc8 + k)) < 64))
                .then_some((self.w(0xb64 + k), self.w(0xb78 + k)))
        })
    }
    pub(super) fn tail_aim(&mut self, parameter: u16) -> Result<()> {
        self.st(5, (parameter as u8 as u16) << 8);
        let orientation = self.v(0x10);
        if orientation == 1 || self.v(1) == 0 {
            return Ok(());
        }
        let (x, y) = self.tail_aim_target().unwrap_or((self.w(0xaf6), self.w(0xafa).wrapping_add(24)));
        let angle =
            self.angle(x.wrapping_sub(self.t(22)), y.wrapping_sub(self.t(23)))?.wrapping_add(128).wrapping_neg();
        if orientation != 0 {
            let angle = if (0x18..0xe8).contains(&angle) { 24 } else { u16::from(angle) };
            let target = self.t(5).wrapping_add(angle).wrapping_add(0x4000);
            if target >= self.t(21) {
                self.st(14, target);
                self.st(10, 8);
            }
        } else {
            let angle = if (0x18..0xe8).contains(&angle) { 232 } else { u16::from(angle) };
            let target = angle.wrapping_add(16128).wrapping_sub(self.t(5));
            if target < self.t(21) {
                self.st(13, target);
                self.st(10, 8);
            }
        }
        Ok(())
    }
    pub(super) fn tail_turn(&mut self, forced: bool) -> Result<bool> {
        let armed = self.t(2) != 0 && self.t(13) & self.t(14) & 0x8000 != 0;
        let trigger = if forced {
            armed
        } else {
            armed
                || (self.t(1) != 0
                    && (self.ram.byte(0x5e5) >= 0xf0 || abs(self.w(0xaf6).wrapping_sub(self.w(ENEMY + 2))) < 0x80))
        };
        if trigger {
            let parameter = if forced {
                1
            } else if armed || self.ram.byte(0x5e5) >= 0xf0 {
                self.t(2).wrapping_sub(1)
            } else {
                0
            };
            self.tail_aim(parameter)?;
        }
        if !forced || trigger {
            self.st(2, 0);
        }
        Ok(trigger)
    }
    pub(super) fn tail_phase(&mut self, state: u16) -> Result<()> {
        match state {
            0 => return Ok(()),
            2 | 3 | 6 | 7 => {
                self.st(
                    10,
                    match state {
                        2 | 3 => 8,
                        6 => 3,
                        _ => 2,
                    },
                );
                if self.v(0x10) != 0 {
                    self.st(13, 0x4000);
                    self.st(14, 0xffff);
                } else {
                    self.st(14, 0x4000);
                    self.st(13, 0xffff);
                }
                self.tail_phase(4)?;
                if self.no_tail() {
                    self.st(0, 4);
                }
                if state == 2 {
                    self.st(0, 3);
                }
                if state == 6 {
                    self.st(0, 6);
                }
                if state == 7 {
                    self.st(0, 7);
                }
                return Ok(());
            }
            1 | 8 => {
                self.tail_speed()?;
                if self.all_tail() && self.tail_turn(state == 8)? {
                    return self.tail_segments();
                }
                if self.no_tail() {
                    self.st(16, 0x8000);
                    self.st(13, 0xffff);
                    self.st(14, 0xffff);
                    self.st(10, if self.w(0xaf6) < 0x70 { 1 } else { 2 });
                }
            }
            4 | 5 => {
                self.tail_speed()?;
                if self.all_tail()
                    && (self.ram.byte(0x5e5) >= 0xf0 || abs(self.w(0xaf6).wrapping_sub(self.w(ENEMY + 2))) < 0x80)
                    && self.t(13) & self.t(14) & 0x8000 != 0
                {
                    self.st(13, 16128);
                    self.st(10, 8);
                } else if self.no_tail() {
                    if !negative(self.c()) {
                        self.st(0, if state == 4 { 5 } else { 6 });
                        if state == 5 {
                            for i in (20..=80).step_by(10) {
                                self.st(i, 2560);
                                self.st(i + 1, 0x4000);
                            }
                            self.st(16, 0x8000);
                        }
                    }
                    self.st(13, 0xffff);
                    self.st(14, 0xffff);
                    let previous = self.t(8);
                    let delay = if previous != 0 { previous - 1 } else { 0 };
                    if previous != 0 {
                        self.st(8, delay);
                    }
                    if previous == 1 {
                        self.st(16, 0x8000);
                        for i in (21..=81).step_by(10) {
                            self.st(i, (self.t(i) & 255) | 0x4000);
                        }
                        self.st(61, (self.t(21) & 0x4ff) | 0x4000);
                    }
                    self.st(10, if previous == 1 { 8 } else { delay });
                }
            }
            _ => return Err("Ridley tail state outside native state machine"),
        }
        self.tail_segments()
    }
    pub(super) fn tail_segments(&mut self) -> Result<()> {
        for i in (0..=60).step_by(10) {
            self.tail_segment(i)?;
        }
        Ok(())
    }
    pub(super) fn tail_segment(&mut self, i: usize) -> Result<()> {
        let mut angle = self.t(i + 21);
        if self.t(i + 16) & 0x8000 != 0 {
            let elapsed = self.t(i + 17);
            if elapsed < self.t(15) {
                self.st(i + 17, elapsed.wrapping_add(self.t(10)));
                return Ok(());
            }
            if elapsed != 0xffff {
                self.st(i + 17, 0xffff);
                self.st(i + 26, 0x8000);
                self.st(i + 28, self.t(i + 18));
            }
            let decreasing = self.t(i + 18) & 0x8000 != 0;
            let fence = if decreasing { 13 } else { 14 };
            let extreme = if decreasing { 11 } else { 12 };
            let candidate = if decreasing {
                angle.wrapping_sub(self.t(10)).wrapping_sub(1)
            } else {
                angle.wrapping_add(self.t(10))
            };
            let crossed = |value: u16, target: u16| {
                if decreasing { negative(value.wrapping_sub(target)) } else { !negative(value.wrapping_sub(target)) }
            };
            if self.t(fence) & 0x8000 != 0 {
                if crossed(candidate, self.t(extreme)) {
                    self.st(i + 18, if decreasing { 0 } else { 0x8000 });
                    angle = self.t(extreme);
                } else {
                    angle = if decreasing { candidate.wrapping_add(1) } else { candidate };
                }
            } else {
                self.st(i + 20, 3072);
                if crossed(candidate, self.t(fence)) {
                    if i == 0 || self.t(i + 6) == 0 {
                        self.st(i + 21, self.t(fence));
                        self.st(i + 16, 0);
                        self.st(i + 17, 0);
                        self.st(i + 18, self.t(i + 18) ^ 0x8000);
                        return Ok(());
                    }
                    self.st(i + 18, if decreasing { 0 } else { 0x8000 });
                    angle = self.t(extreme);
                } else {
                    angle = if decreasing { candidate.wrapping_add(1) } else { candidate };
                }
            }
            self.st(i + 21, angle);
        }
        let angle = (angle as u8).wrapping_add(if i != 0 { self.t(i + 11) as u8 } else { 0 });
        let factor = (self.t(i + 19) >> 8) as u8;
        let sine = enemy_projectiles::SINE[usize::from(angle) + 64];
        let cosine = enemy_projectiles::SINE[usize::from(angle.wrapping_add(64)) + 64];
        let horizontal = self.host.mode7_multiply(sine, factor)?;
        self.st(i + 24, horizontal);
        let vertical = self.host.mode7_multiply(cosine, factor)?;
        self.st(i + 25, vertical);
        Ok(())
    }
    pub(super) fn tail_positions(&mut self) -> Result<()> {
        for (index, previous, relative) in [
            (23, None, 25),
            (33, Some(23), 35),
            (43, Some(33), 45),
            (53, Some(43), 55),
            (63, Some(53), 65),
            (73, Some(63), 75),
            (83, Some(73), 85),
        ] {
            self.st(
                index,
                self.t(relative).wrapping_add(previous.map_or(self.w(ENEMY + 6).wrapping_add(16), |i| self.t(i))),
            );
        }
        let o = self.v(0x10);
        if o > 2 {
            return Err("Ridley tail orientation outside native table");
        }
        let offset = if o == 1 { 0 } else { self.t(24).wrapping_add([32, 0, (-32i16) as u16][usize::from(o)]) };
        self.st(22, self.w(ENEMY + 2).wrapping_add(offset));
        for (index, prev, relative) in
            [(32, 22, 34), (42, 32, 44), (52, 42, 54), (62, 52, 64), (72, 62, 74), (82, 72, 84)]
        {
            self.st(index, if o == 1 { self.w(ENEMY + 2) } else { self.t(prev).wrapping_add(self.t(relative)) });
        }
        Ok(())
    }
    pub(super) fn tail_lengths(&mut self) {
        for (index, maximum, rest) in [
            (30, 6144, 2048u16),
            (40, 6144, 2048),
            (50, 5632, 2048),
            (60, 5632, 2048),
            (70, 4608, 2048),
            (80, 1280, 1280),
        ] {
            let length = self.t(index - 1);
            if self.t(index) != 0 {
                if negative(self.t(index).wrapping_sub(length)) {
                    self.st(index, 0);
                }
                let next = self.t(9).wrapping_add(length);
                self.st(index - 1, if !negative(next.wrapping_sub(maximum)) { maximum } else { next });
            } else if negative(rest.wrapping_sub(length)) {
                self.st(index - 1, length.wrapping_sub(128));
            }
        }
    }
    pub(super) fn tail_main(&mut self) -> Result<()> {
        if !negative(self.t(10).wrapping_sub(8)) {
            let value = self.v(0x0e).wrapping_add(1);
            self.sv(0x0e, value);
            if !negative(value.wrapping_sub(16)) {
                if self.v(0x0f) == 0 {
                    self.sound(3, 0x21, 6)?;
                }
                self.sv(0x0e, 0);
            }
        } else {
            self.sv(0x0e, 0);
        }
        self.tail_phase(self.t(0))?;
        self.tail_positions()?;
        self.tail_lengths();
        if self.v(0x1b) | self.w(0x18a8) | (self.w(ENEMY + 0xe) & TANGIBLE) == 0
            && self.samus_rect(self.t(82), self.t(83), 14, 14)
        {
            self.damage(self.v(0x1c))?;
            self.knockback(self.t(82));
        }
        Ok(())
    }
}
