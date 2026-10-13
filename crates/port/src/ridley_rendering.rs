//! Native Ridley palette, OAM, VRAM/Mode7 queues and baby-metroid animation.
use super::*;

impl<H: Host> Context<'_, '_, H> {
    pub(super) fn palette(&mut self, target: bool, offset: u16, source: u16, count: u16) -> Result<()> {
        let base = if target { 0xc200 } else { 0xc000 };
        if count == 0 || usize::from(offset) / 2 + usize::from(count) > 256 {
            return Err("Ridley palette copy extent");
        }
        for i in 0..count {
            self.set(base + usize::from(offset & !1) + usize::from(i) * 2, self.table(source, i)?);
        }
        Ok(())
    }
    pub(super) fn queue_vram_pair(
        &mut self,
        source0: u16,
        source1: u16,
        dest0: u16,
        dest1: u16,
        size: u16,
    ) -> Result<()> {
        let tail = self.w(0x330);
        let p = 0xd0 + usize::from(tail);
        // Native stores a word over source-high/bank, then replaces source-high.
        // Queue cursors are raw WRAM offsets and can overlap adjacent fields.
        self.set(p + 3, 0xb000);
        self.set(p + 10, 0xb000);
        self.set(p + 2, source0);
        self.set(p + 9, source1);
        self.set(p + 5, dest0);
        self.set(p + 12, dest1);
        self.set(p, size);
        self.set(p + 7, size);
        self.set(0x330, tail.wrapping_add(14));
        self.set(p + 14, 0);
        Ok(())
    }
    fn queue_transfer_entry(&mut self, destination: usize, source: u16) -> Result<()> {
        let p = rom_offset(0xa60000 | u32::from(source));
        let rom = self.rom;
        let bytes = rom.get(p..p + 7).ok_or("Ridley transfer ROM extent")?;
        let word = |i| u16::from_le_bytes([bytes[i], bytes[i + 1]]);
        if destination + 7 > 0x20000 {
            return Err("Ridley transfer WRAM extent");
        }
        self.set(destination, word(0));
        self.set(destination + 3, word(3));
        self.set(destination + 2, word(2));
        self.set(destination + 5, word(5));
        Ok(())
    }
    pub(super) fn queue_transfer(&mut self, source: u16) -> Result<()> {
        let tail = self.w(0x330);
        self.queue_transfer_entry(0xd0 + usize::from(tail), source)?;
        self.set(0x330, tail.wrapping_add(7));
        Ok(())
    }
    pub(super) fn queue_transfer_list(&mut self, source: u16) -> Result<()> {
        let mut source = source;
        let mut tail = usize::from(self.w(0x330));
        for _ in 0..=u16::MAX {
            if self.rw(source)? == 0 {
                self.set(0x330, tail as u16);
                return Ok(());
            }
            self.queue_transfer_entry(0xd0 + tail, source)?;
            source = source.wrapping_add(7);
            tail += 7;
        }
        Err("Ridley transfer table terminator missing")
    }
    pub(super) fn queue_mode7(&mut self, source: u16) -> Result<()> {
        let mut src = rom_offset(0xa60000 | u32::from(source));
        let mut tail = usize::from(self.w(0x334));
        let rom = self.rom;
        loop {
            let flag = *rom.get(src).ok_or("Ridley Mode7 ROM extent")?;
            let len = if flag & 0x80 != 0 {
                9
            } else if flag & 0x40 != 0 {
                7
            } else {
                break;
            };
            let bytes = rom.get(src..src + len).ok_or("Ridley Mode7 ROM extent")?;
            if 0x2d0 + tail + len >= 0x20000 {
                return Err("Ridley Mode7 WRAM extent");
            }
            self.ram.set_byte(0x2d0 + tail + len, 0);
            for (i, byte) in bytes.iter().enumerate() {
                self.ram.set_byte(0x2d0 + tail + i, *byte);
            }
            src += len;
            tail += len;
        }
        self.set(0x334, tail as u16);
        Ok(())
    }
    pub(super) fn graphics_timer(&mut self) -> Result<()> {
        let timer = self.v(6).wrapping_sub(1);
        self.sv(6, timer);
        if timer != 0 {
            return Ok(());
        }
        let mut pc = self.v(5);
        let mut visits = 0;
        while negative(self.rw(pc)?) {
            pc = self.rw(pc)?;
            self.sv(5, pc);
            visits += 1;
            if visits > 64 {
                return Err("Ridley graphics script cycle");
            }
        }
        self.sv(6, self.rw(pc)?);
        self.queue_vram_pair(self.rw(pc.wrapping_add(2))?, self.rw(pc.wrapping_add(4))?, 29216, 29472, 64)?;
        self.sv(5, pc.wrapping_add(6));
        Ok(())
    }
    pub(super) fn carry_graphics(&mut self, carry: bool) -> Result<()> {
        let source = if carry { 0xdad4 } else { 0xdad0 };
        self.queue_vram_pair(self.rw(source)?, self.rw(source + 2)?, 31424, 31680, 128)
    }
    pub(super) fn draw_ridley(&mut self, map: u16, x: u16, y: u16, attributes: u16) -> Result<()> {
        let count = self.rw(map)?;
        if count == 0 {
            return Err("Ridley spritemap must contain an entry");
        }
        let start = rom_offset(0xa60000 | u32::from(map)) + 2;
        let rom = self.rom;
        let entries = rom.get(start..start + usize::from(count) * 5).ok_or("Ridley spritemap ROM extent")?;
        let mut index = usize::from(self.w(0x590));
        for p in entries.as_chunks::<5>().0 {
            let y_offset = p[2] as i8 as i16 as u16;
            let rendered_y = y.wrapping_add(y_offset).wrapping_sub(self.w(0x915));
            if !negative(rendered_y) && negative(rendered_y.wrapping_sub(224)) {
                let x_word = u16::from_le_bytes([p[0], p[1]]);
                let rendered_x = x.wrapping_add(x_word).wrapping_sub(self.w(0x911));
                self.ram.set_byte(0x370 + index, rendered_x as u8);
                self.ram.set_byte(0x370 + index + 1, rendered_y as u8);
                self.set(0x370 + index + 2, attributes | u16::from_le_bytes([p[3], p[4]]));
                let shift = 2 * ((index >> 2) & 7);
                let bits = ((rendered_x & 0x100) >> 8) | (u16::from(negative(x_word)) * 2);
                let ext = 0x570 + (index >> 5) * 2;
                self.set(ext, self.w(ext) | (bits << shift));
                index = (index + 4) & 0x1ff;
            }
        }
        self.set(0x590, index as u16);
        Ok(())
    }
    pub(super) fn draw_spritemap(&mut self, map: u16, x: u16, y: u16, attributes: u16) -> Result<()> {
        let mut index = usize::from(self.w(0x590));
        let count = usize::from(self.rw(map)?).min(0x200usize.saturating_sub(index).div_ceil(4));
        let start = rom_offset(0xa60000 | u32::from(map)) + 2;
        let rom = self.rom;
        let entries = rom.get(start..start + count * 5).ok_or("Ceres spritemap ROM extent")?;
        for p in entries.as_chunks::<5>().0 {
            let word = u16::from_le_bytes([p[0], p[1]]);
            let y_delta = p[2];
            let mut rendered_x = x.wrapping_add(word);
            let mut rendered_y = u16::from(y as u8) + u16::from(y_delta);
            let hidden = if y_delta & 0x80 == 0 {
                rendered_y >= 0xe0
            } else if rendered_y & 0x100 != 0 {
                rendered_y as u8 >= 0xe0
            } else {
                (rendered_y as u8) < 0xe0
            };
            if hidden {
                rendered_x = 0x180;
                rendered_y = 0xe0;
            }
            self.ram.set_byte(0x370 + index, rendered_x as u8);
            self.ram.set_byte(0x370 + index + 1, rendered_y as u8);
            self.set(0x370 + index + 2, attributes | (u16::from_le_bytes([p[3], p[4]]) & 0xf1ff));
            let ext = 0x570 + (index >> 5) * 2;
            let bits = ((rendered_x & 0x100) >> 8) | (u16::from(negative(word)) * 2);
            self.set(ext, self.w(ext) | (bits << (2 * ((index >> 2) & 7))));
            index += 4;
        }
        self.set(0x590, index as u16);
        Ok(())
    }
    pub(super) fn draw_body(&mut self) -> Result<()> {
        let orientation = self.v(0x10);
        if orientation == 1 {
            return Ok(());
        }
        let offset = if orientation != 0 { 10 } else { 0 };
        let map = self.table(0xdb02, self.v(7).wrapping_add(offset))?;
        self.draw_ridley(map, self.w(ENEMY + 2), self.w(ENEMY + 6), self.v(0x0c))
    }
    pub(super) fn draw_tail(&mut self) -> Result<()> {
        if self.w(ENEMY + 0xe) & INVISIBLE != 0 {
            return Ok(());
        }
        let index = ((self.t(71) as u8).wrapping_add(self.t(81) as u8).wrapping_add(8) & 0xf0) >> 4;
        for (map, x, y) in [
            (self.table(0xdcba, u16::from(index))?, 82, 83),
            (0xdc9e, 72, 73),
            (0xdc9e, 62, 63),
            (0xdc97, 52, 53),
            (0xdc97, 42, 43),
            (0xdc90, 32, 33),
            (0xdc90, 22, 23),
        ] {
            self.draw_ridley(map, self.t(x), self.t(y), self.v(0x0c))?;
        }
        Ok(())
    }
    pub(super) fn baby_instruction(&mut self, opcode: u16, argument: u16) -> Result<u16> {
        match opcode {
            0xbfc9 => {
                if self.v(0x46) == 0 && self.w(0x5e5) & 1 != 0 {
                    self.rw(argument)
                } else {
                    self.sound(3, 0x24, 6)?;
                    Ok(argument.wrapping_add(2))
                }
            }
            0xbfe1 => {
                self.palette(false, 0x162, self.rw(argument)?, 15)?;
                Ok(argument.wrapping_add(2))
            }
            0xbff2 => {
                if self.v(0x46) != 0 {
                    self.rw(argument)
                } else {
                    Ok(argument.wrapping_add(2))
                }
            }
            0xbff8 => self.rw(argument),
            _ => Err("unsupported native baby-metroid instruction"),
        }
    }
    pub(super) fn baby_frame(&mut self, state: usize) -> Result<Option<u16>> {
        let ip = self.w(state);
        if !negative(ip) {
            return Ok(None);
        }
        let mut pc = ip;
        let word = self.rw(pc)?;
        if !negative(word) && self.w(state + 2) != word {
            self.set(state + 2, self.w(state + 2).wrapping_add(1));
            return Ok(Some(self.rw(pc + 2)?));
        }
        if !negative(word) {
            pc = pc.wrapping_add(4);
        }
        let mut limit = 0;
        while negative(self.rw(pc)?) {
            pc = self.baby_instruction(self.rw(pc)?, pc.wrapping_add(2))?;
            limit += 1;
            if limit > 4096 {
                return Err("baby-metroid instruction cycle");
            }
        }
        self.set(state + 2, 1);
        self.set(state, pc);
        Ok(Some(self.rw(pc.wrapping_add(2))?))
    }
    pub(super) fn draw_baby(&mut self) -> Result<()> {
        if let Some(map) = self.baby_frame(0x7806)? {
            self.draw_ridley(map, self.v(0x42), self.v(0x44), 0)?;
        }
        Ok(())
    }
    pub(super) fn baby_position(&mut self) -> Result<()> {
        match self.v(0x40) {
            0xbe9c => {
                self.sv(0x42, self.w(ENEMY + 2).wrapping_sub(16));
                self.sv(0x44, self.w(ENEMY + 6).wrapping_add(22));
            }
            0xbeb3 => {
                self.sv(0x42, self.w(ENEMY + 2).wrapping_add(14));
                self.sv(0x44, self.w(ENEMY + 6).wrapping_add(66));
            }
            0xbeca => {
                self.sv(0x45, 0);
                self.sv(0x46, 0);
                self.sv(0x40, 0xbedc);
                self.baby_position()?;
            }
            0xbedc => {
                let velocity = self.v(0x46).wrapping_add(8);
                self.sv(0x46, velocity);
                let carry = u16::from((self.v(0x43) >> 8) as u8) + u16::from(velocity as u8);
                self.sv(0x43, (self.v(0x43) & 255) | (carry << 8));
                let y = self.v(0x44).wrapping_add((velocity >> 8) as u8 as i8 as i16 as u16).wrapping_add(carry >> 8);
                self.sv(0x44, if !negative(y.wrapping_sub(192)) { 192 } else { y });
                if !negative(y.wrapping_sub(192)) {
                    self.sv(0x40, 0xbf19);
                }
            }
            0xbf19 => (),
            _ => return Err("Ridley baby-metroid state unsupported"),
        }
        Ok(())
    }
    pub(super) fn ceres_draw_hook(&mut self) -> Result<()> {
        self.baby_position()?;
        if self.v(2) != 0 {
            self.set(0x178c, 0xa2f2);
            self.ram.set_byte(0x178e, 0xa6);
            Ok(())
        } else {
            self.ceres_draw()
        }
    }
    pub(super) fn ceres_draw(&mut self) -> Result<()> {
        if self.w(0x93f) == 0 {
            self.draw_baby()?;
        }
        if self.w(ENEMY + 0x40 + 0x32) != 0 {
            let x = self
                .w(ENEMY + 0x40 + 2)
                .wrapping_add([0, (-1024i16) as u16, (-4i16) as u16, 0xffff][usize::from(self.w(0x1840) & 3)]);
            self.draw_spritemap(0xa329, x, self.w(ENEMY + 0x40 + 6), 1024)?;
        }
        Ok(())
    }
    pub(super) fn update_palette_index(&mut self) {
        self.sv(0x0c, if self.w(ENEMY + 0x24) >= 2 && (self.w(0xe44).wrapping_add(1) & 2) != 0 { 0 } else { 3584 });
    }
    pub(super) fn health_palette(&mut self) -> Result<()> {
        let health = self.w(ENEMY + 0x14);
        let tier = if health < 1800 {
            3
        } else if health < 5400 {
            2
        } else if health < 9000 {
            1
        } else {
            0
        };
        self.sv(0x12, tier);
        if tier != 0 && self.w(0xc400) == 0 {
            self.palette(false, 0x1e2, (28u16.wrapping_mul(tier - 1)).wrapping_sub(7062), 14)?;
        }
        Ok(())
    }
    pub(super) fn shot_palette(&mut self) -> Result<()> {
        if self.v(1) != 0 && self.v(0x0d) >= 50 && self.w(0xc400) == 0 {
            self.palette(false, 0x1e2, (if self.v(0x0d) >= 70 { 56u16 } else { 0 }).wrapping_sub(7062), 14)?;
        }
        Ok(())
    }
}
