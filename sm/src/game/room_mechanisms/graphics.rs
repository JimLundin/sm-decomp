//! Item graphics installation and packed VRAM transfer records.
#![forbid(unsafe_code)]
use super::{fields::Fields, port::Execution, state::Column};
use crate::game::enemy_script::{EnemyScript as Script, bank_base};
impl Execution<'_> {
    pub(super) fn load_graphics(&mut self, mut p: Script, k: u16) -> Script {
        let gfx_index = self.port().read_le(0x1c2d);
        let current_index = self.port().read_le(0x1c2d);
        self.set_slot(Column::Work1, usize::from(k >> 1), current_index);
        self.port_mut()
            .write_slice_le(0x1c2d, gfx_index.wrapping_add(2) & 6);
        let table_offset = usize::from(gfx_index >> 1) * 2;
        let destination = self.foreign(|g| g.read_rom_word(0x8487cd, table_offset));
        let tile_offset = usize::from(self.foreign(|g| g.read_rom_word(0x8487d5, table_offset)));
        let mut tile_number = self.foreign(|g| g.read_rom_word(0x8487dd, table_offset));
        let instruction = p.offset_from(bank_base(0x84)) as u16;
        self.port_mut()
            .write_slice_le(0x1c2f + usize::from(gfx_index >> 1) * 2, instruction);
        let tail = self.port().read_le(0x330);
        let entry = 0xd0 + usize::from(self.port().read_le(0x330));
        self.port_mut().write_slice_le(entry, 256);
        self.port_mut().write_slice_le(entry + 2, p.word(0));
        self.port_mut().set_byte(entry + 4, 0x89);
        self.port_mut().write_slice_le(entry + 5, destination);
        self.port_mut().write_slice_le(0x330, tail.wrapping_add(7));
        p = p.advance(2);
        for tile in 0..8 {
            let attributes = u16::from(p.byte(0)) << 10;
            let value = tile_number.wrapping_add(attributes);
            self.port_mut()
                .write_slice_le(0xa000 + tile_offset + tile * 2, value);
            tile_number = tile_number.wrapping_add(1);
            p = p.advance(1);
        }
        p
    }
    pub(super) fn copy_vram(&mut self, p: Script) -> Script {
        let tail = self.port().read_le(0x330);
        let entry = 0xd0 + usize::from(self.port().read_le(0x330));
        self.port_mut().write_slice_le(entry, p.word(0));
        self.port_mut().write_slice_le(entry + 2, p.word(2));
        self.port_mut().set_byte(entry + 4, p.byte(4));
        self.port_mut().write_slice_le(entry + 5, p.word(5));
        self.port_mut().write_slice_le(0x330, tail.wrapping_add(7));
        p.advance(7)
    }
    pub(super) fn reload_graphics(&mut self) {
        self.port_mut().write_slice_le(0x1c2d, 0);
        for k in (0..8).step_by(2) {
            if self.port().read_le(0x1c2f + k) != 0 {
                let address = self.port().read_le(0x1c2f + k);
                let record = self.record::<0x84>(address);
                self.load_graphics(Script::at(record.image, record.base), k as u16);
            }
        }
    }
}
