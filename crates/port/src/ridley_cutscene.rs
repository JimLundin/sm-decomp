//! Ceres appearance/escape and the complete Ridley death lifecycle.
use super::*;

impl<H: Host> Context<'_, '_, H> {
    pub(super) fn mark_map(&mut self) -> Result<()> {
        self.set(0x5f7, 1);
        for index in 0..5 {
            for row in [26, 58, 90] {
                self.set(0xc608 + (row + index) * 2, 11295);
            }
        }
        let mut found = None;
        for i in (0..6).rev() {
            let p = 0x90a83a + i * 4;
            if rom_word(self.rom, p)? == self.w(0x179c) {
                found = Some(rom_word(self.rom, p + 2)?);
                break;
            }
        }
        if let Some(mut p) = found {
            for _ in 0..=u16::MAX {
                let x = rom_word(self.rom, 0x900000 | u32::from(p))?;
                if negative(x) {
                    return Ok(());
                }
                let y = rom_word(self.rom, 0x900000 | (u32::from(p) + 2))?;
                let mapx = self.w(0x7a1).wrapping_add(x >> 8);
                let page = mapx & 0x20;
                let column = (mapx & 0x1f) >> 3;
                let row = self.w(0x7a3).wrapping_add(y >> 8).wrapping_add(1);
                let index = usize::from(column.wrapping_add(4u16.wrapping_mul(page.wrapping_add(row))));
                let bit = 0x80 >> (mapx & 7);
                self.ram.set_byte(0x7f7 + index, self.ram.byte(0x7f7 + index) | bit);
                p = p.wrapping_add(4);
            }
            return Err("Ridley map table terminator missing");
        }
        Ok(())
    }
    pub(super) fn initialize(&mut self) -> Result<()> {
        let area = usize::from(self.w(0x79f));
        if area >= 8 {
            return Err("Ridley initialization area extent");
        }
        if self.ram.byte(0xd828 + area) & 1 != 0 {
            self.set(ENEMY + 0xe, self.w(ENEMY + 0xe) | TANGIBLE | DELETED | INVISIBLE);
            return Ok(());
        }
        for i in (0..4096).step_by(2) {
            self.set(0x2000 + i, 0);
        }
        self.mark_map()?;
        self.set(ENEMY + 0x3c, 0);
        self.set(ENEMY + 0x3e, 0);
        self.animation(0xe538, 1);
        self.set(ENEMY + 0x1e, 3584);
        self.sv(0x0c, 3584);
        self.set(ENEMY + 0x10, self.w(ENEMY + 0x10) | 4);
        self.sv(1, 0);
        self.sv(0x0d, 0);
        self.set(0x183e, 0);
        self.set(0x1840, 0);
        self.initialize_tail()?;
        self.sv(5, 0xda71);
        self.sv(6, 1);
        self.set(ENEMY + 0xe, self.w(ENEMY + 0xe) | 0x1000 | TANGIBLE);
        self.sa(0xa35b);
        self.sb(0);
        self.sc(0);
        if area == 2 {
            self.set(ENEMY + 2, 96);
            self.set(ENEMY + 6, 394);
            self.set(ENEMY + 0x22, 5);
            self.sv(1, 0);
            self.sv(2, 1);
            for (i, value) in [(0x20, 64), (0x21, 416), (0x22, 64), (0x23, 224), (0x10, 2), (0x1c, 120)] {
                self.sv(i, value);
            }
            self.palette(true, 0x140, 0xe1cf, 32)?;
            for i in 0..15 {
                self.set(0xc200 + (113 + i) * 2, 0);
                self.set(0xc200 + (241 + i) * 2, 0);
            }
        } else {
            self.set(ENEMY + 2, 186);
            self.set(ENEMY + 6, 169);
            self.set(0x93f, 0);
            self.sv(0x10, 0);
            self.sv(2, 0);
            self.st(1, 1);
            self.sv(0x1c, 15);
            for (i, value) in [(0x20, (-32i16) as u16), (0x21, 176), (0x22, 40), (0x23, 224)] {
                self.sv(i, value);
            }
            self.palette(true, 0x140, 0xe16f, 32)?;
            for i in 0..15 {
                self.set(0xc200 + 482 + i * 2, 0);
            }
            self.sv(3, 0xbf31);
            self.sv(4, 1);
            self.sv(0x40, 0xbe9c);
            self.sv(7, 5);
            self.music(0)?;
        }
        Ok(())
    }
    pub(super) fn ceres_main(&mut self) -> Result<()> {
        self.set(ENEMY + 0x14, 0x7fff);
        self.run_state(self.a())?;
        if self.w(0x93f) == 0 {
            if self.v(2) != 0 {
                self.update_palette_index();
                self.integrate(self.w(CURRENT), true)?;
                self.update_wing_animation();
                if self.w(0x5e5) >= 0xff00 {
                    self.st(15, (self.w(0x5e5) & 15) + 8);
                }
                self.tail_main()?;
                self.shot_palette()?;
                self.graphics_timer()?;
                self.draw_tail()?;
                self.draw_body()?;
            }
            self.ceres_draw_hook()?;
        }
        Ok(())
    }
    pub(super) fn ceres_state(&mut self, address: u16) -> Result<bool> {
        match address {
            0xa354 => {
                self.sb(0);
                self.sc(0);
            }
            0xa35b => {
                if self.w(0x797) == 0 {
                    self.sa(0xa377);
                    self.sf(if self.w(0x79f) == 2 { 170 } else { 512 });
                    self.run_state(0xa377)?;
                }
            }
            0xa377 => {
                if self.decrement_f() {
                    self.sa(0xa389);
                    self.se(0);
                    self.sf(0);
                }
            }
            0xa389 => {
                if !negative(self.e()) {
                    let frame = self.e();
                    self.sf(self.f().wrapping_add(1));
                    if self.f() != 0 {
                        self.sf(0);
                        let index = rom_byte(self.rom, 0xa6e269 + u32::from(frame))?;
                        if index == 255 {
                            self.se(0);
                            self.sa(0xa3df);
                            self.sv(2, 1);
                        } else {
                            self.se(frame.wrapping_add(1));
                            for i in 0..3 {
                                self.set(0xc000 + (252 + i) * 2, self.table(0xe2aa, u16::from(index) * 3 + i as u16)?);
                            }
                        }
                    }
                }
            }
            0xa3df => {
                self.sf(self.f().wrapping_add(1));
                if self.f() >= 2 {
                    self.sf(0);
                    let mut index = self.e();
                    for i in 0..11 {
                        let color = self.table(0xe30a, index >> 1)?;
                        self.set(0xc000 + 290 + i * 2, color);
                        self.set(0xc000 + 482 + i * 2, color);
                        index = index.wrapping_add(2);
                    }
                    if index >= 0x160 {
                        if self.w(0x79f) == 2 {
                            self.set(ENEMY + 0x22, 2);
                        }
                        self.set(ENEMY + 0xe, self.w(ENEMY + 0xe) & !TANGIBLE);
                        self.se(0);
                        self.sa(0xa455);
                        self.sf(4);
                        self.music(5)?;
                    } else {
                        self.se(index);
                    }
                }
            }
            0xa455 => {
                if self.decrement_f() {
                    self.animation(0xe690, 1);
                    self.sf(if self.w(0x79f) != 2 { 252 } else { 0 });
                    self.sa(0xa478);
                }
            }
            0xa478 => {
                if self.decrement_f() {
                    let mut done = true;
                    if self.w(0x79f) == 2 {
                        self.sf(2);
                        let value = self.e();
                        self.se(value.wrapping_add(1));
                        let source = self.table(0xa4eb, value)?;
                        done = source == 0;
                        if !done {
                            self.palette(false, 0xe2, source, 14)?;
                        } else {
                            self.set(0x197a, 440);
                            self.set(0x197c, (-96i16) as u16);
                            self.set(0x1980, 32);
                        }
                    }
                    if done {
                        self.se(0);
                        self.animation(0xe91d, 1);
                        self.sv(8, 8);
                        self.sv(9, 8);
                        self.tail_flags(0x8000);
                        self.st(0, 1);
                        self.sa(0xa354);
                    }
                }
            }
            0xa6af => {
                self.sc(self.c().wrapping_sub(16));
                if negative(self.w(ENEMY + 6).wrapping_sub(112)) {
                    self.sa(0xa6c8);
                    self.run_state(0xa6c8)?;
                }
            }
            0xa6c8 => {
                self.sc(self.c().wrapping_add(20));
                if negative(self.w(ENEMY + 6).wrapping_sub(80)) {
                    self.sa(0xa6e8);
                    self.sv(1, 1);
                }
            }
            0xa6e8 => {
                if self.v(0x0d) >= 100 {
                    self.sv(1, 0);
                    self.sa(0xbd9a);
                    self.run_state(0xbd9a)?;
                } else if negative(self.w(0x9c2).wrapping_sub(30)) {
                    self.sv(1, 0);
                    self.sa(0xa971);
                    self.run_state(0xa971)?;
                } else {
                    self.seek(0, 0, None, 192, 100)?;
                    let outside = self.outside_rect(0, 192, 100, 8, 8)?;
                    let timeout = if outside {
                        self.sv(0, self.v(0).wrapping_add(1));
                        self.v(0) >= 124
                    } else {
                        false
                    };
                    if !outside || timeout {
                        self.sa(self.table(0xa743, self.w(0x5e5) & 15)?);
                        self.sv(0, 0);
                    }
                }
            }
            0xa782 => {
                let magnitude = abs(self.c()).max(128);
                self.sc(if negative(self.c()) { magnitude.wrapping_neg() } else { magnitude });
                self.seek(0, 0, None, self.w(ENEMY + 2), 88)?;
                let y = self.w(ENEMY + 6);
                if negative(y.wrapping_sub(80)) {
                    self.sv(0, self.v(0).wrapping_add(1));
                    if self.v(0) >= 48 {
                        self.sa(0xa88d);
                    }
                } else if negative(y.wrapping_sub(128)) {
                    self.sv(0x17, self.w(ENEMY + 2));
                    self.sv(0x18, y);
                    self.animation(0xe73a, 1);
                    self.sa(0xa7f9);
                    self.sv(0, 224);
                    self.run_state(0xa7f9)?;
                }
            }
            0xa7f9 => {
                let random = self.w(0x5e5);
                let jitter = if negative(random) { (random & 7).wrapping_neg() } else { random & 7 };
                self.seek(0, 0, None, self.v(0x17).wrapping_add(jitter), self.v(0x18).wrapping_add(jitter))?;
                let timer = self.v(0).wrapping_sub(1);
                self.sv(0, timer);
                if negative(timer) {
                    self.sv(0, 0);
                    self.sa(0xa6e8);
                }
            }
            0xa83c => {
                self.animation(0xe548, 1);
                self.sa(0xa84e);
                self.sf(64);
                self.run_state(0xa84e)?;
            }
            0xa84e => {
                let x = self.w(0xaf6);
                let y = if negative(self.w(0xafa).wrapping_sub(132)) { 64 } else { self.w(0xafa).wrapping_sub(68) };
                self.seek(0, 13, None, x, y)?;
                if !self.outside_rect(0, x, y, 2, 2)? || self.decrement_f() {
                    self.sv(0, 0);
                    self.sa(0xa6e8);
                }
            }
            0xa88d => {
                self.sa(0xa8a4);
                self.sf(10);
                self.sv(0x0a, 0);
                self.st(1, 0);
                self.run_state(0xa8a4)?;
            }
            0xa8a4 => {
                self.seek(0, 1, None, 192, 80)?;
                if negative(self.w(ENEMY + 6).wrapping_sub(96)) {
                    self.sa(0xa8d4);
                    self.sf(10);
                    self.sv(0x0a, 0);
                }
            }
            0xa8d4 | 0xa8f8 | 0xa923 | 0xa947 => {
                let (accel, target, next, timer) = match address {
                    0xa8d4 => ((-32i16) as u16, (-1024i16) as u16, 0xa8f8, 36),
                    0xa8f8 => ((-512i16) as u16, (-16384i16) as u16, 0xa923, 28),
                    0xa923 => ((-512i16) as u16, (-30720i16) as u16, 0xa947, 1),
                    _ => ((-768i16) as u16, (-30720i16) as u16, 0xa6e8, 0),
                };
                self.arc(accel, target, 768);
                if self.decrement_f() {
                    self.sa(next);
                    if address != 0xa947 {
                        self.sf(timer);
                    }
                    if address == 0xa8f8 {
                        self.st(2, 1);
                    }
                    if address == 0xa947 {
                        self.sv(0, 0);
                        self.st(1, 1);
                    }
                }
            }
            0xa971 => {
                self.sv(0x20, (-192i16) as u16);
                self.seek(0, 1, None, 192, (-128i16) as u16)?;
                if negative(self.w(ENEMY + 6).wrapping_add(128)) {
                    self.sa(0xa9a0);
                    self.sf(64);
                    self.run_state(0xa9a0)?;
                }
            }
            0xa9a0 => {
                if self.decrement_f() {
                    for population in [0xaa2f, 0xaa3f] {
                        self.service(Service::SpawnEnemy { bank: 0xa6, population })?;
                    }
                    self.sb(0);
                    self.sc(0);
                    self.st(0, 0);
                    self.sa(0xaa11);
                    self.palette(false, 0xa2, 0xa9e3, 15)?;
                    self.palette(false, 0x42, 0xaa01, 8)?;
                    self.palette(false, 0x1e2, 0xaa01, 8)?;
                }
            }
            0xaa11 => {
                self.sa(0xaa4f);
                self.set(0x93f, 1);
                self.setup_getaway();
            }
            0xaa4f | 0xc600 => (),
            0xaa50 => self.ceres_palette()?,
            _ => return Ok(false),
        }
        Ok(true)
    }
    pub(super) fn setup_getaway(&mut self) {
        self.ram.set_byte(0x56, 7);
        self.set(0x783, 1);
        self.ram.set_byte(0x5f, 0x80);
        for at in [0x78, 0x7a, 0x7c, 0x7e] {
            self.set(at, 256);
        }
        self.set(0x80, 64);
        self.set(0x82, 64);
        for (i, value) in
            [(0x32, 0), (0x33, 0), (0x0a, 1), (0x34, (-128i16) as u16), (0x35, 32), (0x30, 2048), (0x31, 1024)]
        {
            self.sv(i, value);
        }
        self.set(0xb1, (-128i16) as u16);
        self.set(0xb3, 32);
    }
    pub(super) fn getaway(&mut self) -> Result<()> {
        let frame = self.v(0x33);
        self.sv(0x33, frame.wrapping_add(2));
        if frame == 0 {
            self.sound(2, 0x4e, 6)?;
        }
        if frame == 208 {
            self.set(0xa58, 0xe90e);
            self.set(0xa5a, 0xe12e);
            self.set(0x183e, 35);
            self.set(0x1840, 64);
        }
        let index = frame >> 1;
        let scale = self.table(0xae4d, index)?;
        if scale == 0xffff {
            self.sv(0x32, self.v(0x32).wrapping_add(1));
            self.ram.set_byte(0x56, 9);
            self.ram.set_byte(0x5f, 0);
            for at in [0x78, 0x7a, 0x7c, 0x7e, 0x80, 0x82, 0xb1, 0xb3] {
                self.set(at, 0);
            }
            self.sa(0xc04e);
            self.sf(0);
            return Ok(());
        }
        self.sv(0x30, scale);
        self.sv(0x35, self.table(0xaf2f, index)?.wrapping_add(self.v(0x35)));
        self.set(0xb3, self.v(0x35));
        self.sv(0x34, self.v(0x34).wrapping_sub(self.table(0xb00f, index)?));
        self.set(0xb1, self.v(0x34));
        self.palette(false, 0xa2, (32u16.wrapping_mul(scale >> 8)).wrapping_sub(0x4ef9), 15)?;
        self.sv(0x0a, self.v(0x0a).wrapping_add(48));
        let angle = (self.v(0x0a) >> 8) as u8;
        let native_sine = |angle: u8| {
            let value = enemy_projectiles::SINE[usize::from(angle) + 64];
            ((i32::from(value) * 256) >> 8) as i16 as u16
        };
        let multiply = |value: u16| {
            let product = ((u32::from(abs(value)) * u32::from(scale)) >> 8) as u16;
            if negative(value) { product.wrapping_neg() } else { product }
        };
        self.set(0x78, multiply(native_sine(angle.wrapping_add(64))));
        self.set(0x7a, multiply(native_sine(angle)));
        self.set(0x7c, multiply(native_sine(angle).wrapping_neg()));
        self.set(0x7e, multiply(native_sine(angle.wrapping_add(64))));
        if self.w(0x5b6) & 3 == 0 {
            let state = self.v(0x36).wrapping_add(1) & 3;
            self.sv(0x36, state);
            self.queue_mode7(self.table(0xacda, state)?)?;
        }
        if self.w(0x5b6) & 7 == 0 {
            let state = self.v(0x37).wrapping_add(1) & 1;
            self.sv(0x37, state);
            self.queue_mode7(self.table(0xad45, state)?)?;
        }
        Ok(())
    }
    pub(super) fn ceres_palette(&mut self) -> Result<()> {
        if self.w(0x797) | self.w(0xc400) == 0 && self.w(0x5b6) & 3 == 0 {
            let next = self.v(0x38).wrapping_add(1);
            let index = if next >= 16 { 0 } else { next };
            self.sv(0x38, index);
            for i in 0..3 {
                self.set(0xc000 + (97 + i) * 2, self.table(0xc1df, index * 3 + i as u16)?);
            }
        }
        Ok(())
    }
    pub(super) fn escape_transfers(&mut self) -> Result<bool> {
        let p = self.e();
        if self.rw(p)? == 0 {
            return Ok(true);
        }
        self.queue_transfer(p)?;
        self.se(p.wrapping_add(7));
        Ok(self.rw(self.e())? == 0)
    }
    pub(super) fn escape_text(&mut self) -> Result<bool> {
        let timer = self.v(0x3d);
        if timer != 0 {
            self.sv(0x3d, timer - 1);
            return Ok(false);
        }
        self.sv(0x3d, self.v(0x3e));
        let mut pc = self.v(0x3b);
        for _ in 0..4096 {
            let word = self.rw(pc)?;
            match word {
                0 => return Ok(true),
                1 => {
                    self.sv(0x3e, self.rw(pc + 2)?);
                    pc = pc.wrapping_add(4);
                }
                13 => {
                    self.sv(0x3c, self.rw(pc + 2)?);
                    pc = pc.wrapping_add(4);
                }
                _ => {
                    let character = word as u8;
                    self.sv(0x3b, pc.wrapping_add(1));
                    if character == 32 {
                        self.sv(0x3c, self.v(0x3c).wrapping_add(1));
                        return Ok(false);
                    }
                    let character = if character == 33 { 91 } else { u16::from(character) };
                    let tail = self.w(0x330);
                    let p = 0xd0 + usize::from(tail);
                    self.set(p, 2);
                    self.set(p + 3, 0x7e00);
                    self.sv(0x3a, 0x3582u16.wrapping_add(character).wrapping_sub(65));
                    self.set(p + 2, 0x8034);
                    self.set(p + 5, self.v(0x3c));
                    self.sv(0x3c, self.v(0x3c).wrapping_add(1));
                    self.set(0x330, tail.wrapping_add(7));
                    let sounds = self.v(0x3f).wrapping_add(1);
                    self.sv(0x3f, sounds);
                    if !negative(sounds.wrapping_sub(2)) {
                        self.sv(0x3f, 0);
                        if self.w(0x79f) == 6 {
                            self.sound(2, 0x45, 3)?;
                        } else {
                            self.sound(3, 13, 3)?;
                        }
                    }
                    return Ok(false);
                }
            }
        }
        Err("escape text instruction cycle")
    }
    pub(super) fn escape_state(&mut self) -> Result<()> {
        match self.f() {
            0 => {
                for (dst, src) in [(97, 1), (99, 3), (81, 17), (83, 19)] {
                    self.set(0xc000 + dst * 2, self.w(0xc000 + src * 2));
                }
                self.se((-15157i16) as u16);
                self.sf(self.f().wrapping_add(2));
                self.escape_state()?;
            }
            2 => {
                if self.escape_transfers()? {
                    self.se((-15106i16) as u16);
                    self.sf(self.f().wrapping_add(2));
                    self.escape_state()?;
                }
            }
            4 => {
                if self.escape_transfers()? {
                    self.sf(self.f().wrapping_add(2));
                    self.queue_transfer(0xc15d)?;
                    self.se(128);
                    self.music(7)?;
                }
            }
            6 => {
                self.ceres_palette()?;
                let previous = self.e();
                self.se(previous.wrapping_sub(1));
                if previous == 1 {
                    for (i, value) in [(0x3b, (-15280i16) as u16), (0x3c, 0), (0x3d, 0), (0x3e, 0), (0x3f, 0)] {
                        self.sv(i, value);
                    }
                    self.se(32);
                    if self.w(0x9e2) == 0 {
                        self.sf(self.f().wrapping_add(2));
                    }
                    self.sf(self.f().wrapping_add(2));
                }
            }
            8 => {
                let previous = self.e();
                self.se(previous.wrapping_sub(1));
                if previous == 1 {
                    self.sf(self.f().wrapping_add(2));
                    self.queue_transfer_list(0xc3b8)?;
                }
                self.ceres_palette()?;
                if self.escape_text()? {
                    self.sf(self.f().wrapping_add(2));
                }
            }
            10 => {
                self.ceres_palette()?;
                if self.escape_text()? {
                    self.sf(self.f().wrapping_add(2));
                }
            }
            12 => {
                self.ceres_palette()?;
                self.sb(0);
                self.sc(0);
                self.sf(0);
                self.sa(0xaa50);
                self.set(0x943, 1);
                self.set(0x93f, 2);
                self.boss_bit()?;
            }
            _ => return Err("Ceres escape substate unsupported"),
        }
        Ok(())
    }
    pub(super) fn explosion_init(&mut self) -> Result<()> {
        let slot = self.w(CURRENT);
        let e = record(slot)?;
        self.set(e + 0x1c, 1);
        self.set(e + 0x18, 0);
        self.set(e + 0x20, 0);
        self.set(e + 0x1e, 3584);
        let parameter = self.w(e + 0x3c);
        if parameter > 22 || parameter & 1 != 0 {
            return Err("Ridley explosion parameter extent");
        }
        self.set(e + 0x3a, self.table(0xc6ce, parameter >> 1)?);
        let random = self.random();
        self.set(e + 0x32, if negative(random) { (random & 0x130).wrapping_neg() } else { random & 0x130 });
        self.set(e + 0x34, 0);
        let (x, y, map) = if parameter <= 12 {
            let i = 22 + usize::from(parameter) * 5;
            let map = if parameter == 12 {
                self.table(
                    0xc7ba,
                    u16::from(((self.t(71) as u8).wrapping_add(self.t(81) as u8).wrapping_add(8) & 0xf0) >> 4),
                )?
            } else if parameter <= 2 {
                0xca47
            } else if parameter <= 6 {
                0xca4d
            } else {
                0xca53
            };
            (self.t(i), self.t(i + 1), map)
        } else {
            let (table_x, table_map, yoffset) = match parameter {
                14 => (0xc804, 0xc808, 0),
                16 => (0xc836, 0xc83a, 22),
                18 => (0xc868, 0xc86c, (-24i16) as u16),
                20 => (0xc89a, 0xc89e, 0),
                _ => (0xc8cc, 0xc8d0, 7),
            };
            let orientation = u16::from(self.v(0x10) != 0);
            (
                self.w(ENEMY + 2).wrapping_add(self.table(table_x, orientation)?),
                self.w(ENEMY + 6).wrapping_add(yoffset),
                self.table(table_map, orientation)?,
            )
        };
        self.set(e + 2, x);
        self.set(e + 6, y);
        self.set(e + 0x1a, map);
        Ok(())
    }
    pub(super) fn explosion_main(&mut self) -> Result<()> {
        let slot = self.w(CURRENT);
        let e = record(slot)?;
        let frame = self.w(e + 0x2c) & 1;
        self.set(e + 0x2c, frame);
        self.set(e + 0xe, if frame != 0 { self.w(e + 0xe) | INVISIBLE } else { self.w(e + 0xe) & !INVISIBLE });
        let speed = self.w(e + 0x32);
        let magnitude = abs(speed).wrapping_sub(4);
        let magnitude = if negative(magnitude) { 0 } else { magnitude };
        self.set(e + 0x32, if negative(speed) { magnitude.wrapping_neg() } else { magnitude });
        self.set(e + 0x34, self.w(e + 0x34).wrapping_add(4));
        self.integrate(slot, false)?;
        let timer = self.w(e + 0x3a).wrapping_sub(1);
        self.set(e + 0x3a, timer);
        if negative(timer) {
            self.service(Service::EnemyDeath { slot, animation: 0 })?;
        }
        Ok(())
    }
    pub(super) fn death_dust(&mut self) -> Result<()> {
        let timer = self.v(0x27).wrapping_sub(1);
        if negative(timer) {
            self.sv(0x27, 4);
            let previous = self.v(0x28);
            let index = if !negative(previous.wrapping_sub(9)) { 0 } else { previous.wrapping_add(1) };
            self.sv(0x28, index);
            let offsets = [-24i16, -24, -20, 20, 16, -30, 30, -3, 14, -13, -2, 18, -2, -32, -31, 8, -4, -10, 19, 19];
            self.projectile.x = self.w(ENEMY + 2).wrapping_add(offsets[usize::from(index) * 2] as u16);
            self.projectile.y = self.w(ENEMY + 6).wrapping_add(offsets[usize::from(index) * 2 + 1] as u16);
            self.spawn(0xe509, 3)?;
            self.sound(2, 0x24, 3)?;
        } else {
            self.sv(0x27, timer);
        }
        Ok(())
    }
    pub(super) fn death_state(&mut self, address: u16) -> Result<bool> {
        match address {
            0xc04e => self.escape_state()?,
            0xc538 => {
                self.seek(0, 0, Some(16), 128, 328)?;
                if !self.outside_rect(0, 128, 328, 4, 4)? {
                    self.run_state(0xc53e)?;
                }
            }
            0xc53e => {
                self.animation(0xe6c8, 1);
                self.sa(0xc551);
                self.sf(32);
            }
            0xc551 => {
                self.seek(0, 0, Some(16), 128, 328)?;
                if self.decrement_f() {
                    self.sb(0);
                    self.sc(0);
                    self.set(0x197a, 528);
                    self.set(0x197c, 64);
                    self.set(0x1980, 1);
                    self.sv(0x27, 0);
                    self.sv(0x28, 0);
                    self.sa(0xc588);
                    self.sf(160);
                    self.run_state(0xc588)?;
                }
            }
            0xc588 => {
                self.death_dust()?;
                if self.decrement_f() {
                    if self.v(0x1b) != 0 {
                        self.release()?;
                    }
                    self.sa(0xc5a8);
                    self.sb(0);
                    self.sc(0);
                    for population in
                        [0xc9e7, 0xc9d7, 0xc9c7, 0xc9b7, 0xc9a7, 0xc997, 0xc987, 0xc9f7, 0xca07, 0xca27, 0xca17, 0xca37]
                    {
                        self.service(Service::SpawnEnemy { bank: 0xa6, population })?;
                    }
                }
            }
            0xc5a8 => {
                self.death_dust()?;
                self.sv(2, 0);
                self.set(ENEMY + 0xe, self.w(ENEMY + 0xe) | INVISIBLE);
                self.sa(0xc5c8);
                self.sf(32);
            }
            0xc5c8 => {
                if self.decrement_f() {
                    self.sa(0xc5da);
                    self.sf(256);
                }
            }
            0xc5da => {
                if self.decrement_f() {
                    self.boss_bit()?;
                    self.service(Service::EnemyItemDrop { slot: 0 })?;
                    self.music(3)?;
                    self.set(ENEMY + 0xe, self.w(ENEMY + 0xe) | DELETED);
                    self.sa(0xc600);
                }
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}
