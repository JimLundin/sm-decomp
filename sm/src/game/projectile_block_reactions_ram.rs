//! Checked room sampling; mechanism setup remains a synchronous legacy boundary.
use crate::game::projectile_block_reactions::{self as reactions, Definitions, Progress};
use crate::game::sm_94::CollInfo;
use crate::game_state::Game;
use crate::ram_map::checked_word as word;
use crate::rom::RomImage;

impl Progress for CollInfo {
    fn span(&self) -> u16 {
        self.ci_r26
    }
    fn edge(&self) -> u16 {
        self.ci_r28
    }
    fn missile(&self) -> u16 {
        self.ci_r30
    }
    fn remaining(&self) -> u16 {
        self.ci_r38
    }
    fn unblocked(&self) -> u16 {
        self.ci_r40
    }
    fn set_remaining(&mut self, value: u16) {
        self.ci_r38 = value;
    }
    fn set_unblocked(&mut self, value: u16) {
        self.ci_r40 = value;
    }
}
pub trait Content {
    fn image(&self, game: &Game) -> RomImage;
    fn warning(&mut self, address: u32, game: &mut Game);
    fn spawn(&mut self, header: u16, game: &mut Game);
}
pub struct Native;
impl Content for Native {
    fn image(&self, game: &Game) -> RomImage {
        game.loaded_rom
            .as_ref()
            .expect("ROM must be loaded")
            .clone()
    }
    fn warning(&mut self, address: u32, _: &mut Game) {
        unsafe {
            crate::c_library::printf(c"RomPtr - Invalid access 0x%x!\n".as_ptr(), address);
        }
    }
    fn spawn(&mut self, header: u16, game: &mut Game) {
        unsafe {
            crate::game::room_mechanisms::spawn_collision(header, game);
        }
    }
}
pub struct Table {
    image: RomImage,
    offset: usize,
}
impl reactions::Table for Table {
    fn word(&self, index: usize) -> u16 {
        self.image.table_word(self.offset, index as isize)
    }
}
pub struct Room<'a, C> {
    pub game: &'a mut Game,
    pub content: C,
}
#[forbid(unsafe_code)]
impl<C: Content> Definitions for Room<'_, C> {
    type Table = Table;
    fn resolve(&mut self, address: u32) -> Table {
        if address & 0x8000 == 0 {
            self.content.warning(address, self.game);
            self.game.g_fail = true;
        }
        Table {
            image: self.content.image(self.game),
            offset: crate::rtl::lorom_offset(address) as usize,
        }
    }
    fn bts(&self) -> u8 {
        let game = &*self.game;
        game.g_ram[0x16402 + usize::from(word!(game.cur_block_index))]
    }
    fn area(&self) -> u16 {
        let game = &*self.game;
        word!(game.area_index)
    }
    fn spawn(&mut self, header: u16) {
        self.content.spawn(header, self.game);
    }
}
#[forbid(unsafe_code)]
pub fn trigger(scan: &mut impl Progress, bombable: bool, solid: bool, game: &mut Game) -> u8 {
    reactions::trigger(
        bombable,
        solid,
        scan,
        &mut Room {
            game,
            content: Native,
        },
    );
    u8::from(solid)
}
