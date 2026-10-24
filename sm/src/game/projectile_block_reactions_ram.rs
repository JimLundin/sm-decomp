//! Checked room sampling; mechanism setup remains a synchronous legacy boundary.
use crate::game::projectile_block_reactions::Definitions;
use crate::game_state::Game;
use crate::ram_map::checked_word as word;
use crate::rom::RomRecord;

impl Definitions for Game {
    type Table = RomRecord;
    fn resolve(&mut self, address: u32) -> RomRecord {
        if address & 0x8000 == 0 {
            unsafe {
                crate::c_library::printf(c"RomPtr - Invalid access 0x%x!\n".as_ptr(), address);
            }
            self.g_fail = true;
        }
        RomRecord {
            image: self
                .loaded_rom
                .as_ref()
                .expect("ROM must be loaded")
                .clone(),
            base: crate::rtl::lorom_offset(address) as usize,
        }
    }
    fn bts(&self) -> u8 {
        let game = self;
        game.g_ram[0x16402 + usize::from(word!(game.cur_block_index))]
    }
    fn area(&self) -> u16 {
        let game = self;
        word!(game.area_index)
    }
    fn spawn(&mut self, header: u16) {
        unsafe {
            crate::game::room_mechanisms::spawn_collision(header, self);
        }
    }
}
