//! Checked room sampling; mechanism setup remains a synchronous legacy boundary.
use crate::game::projectile_block_reactions::{
    self as reactions, Context, Definitions, Material, Mechanism, Progress, SlopeInputs,
};
use crate::game::projectile_room::Axis;
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
            crate::game::plm_spawning::SpawnPLM(header, game);
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
impl<C: Content> Definitions<()> for Room<'_, C> {
    type Table = Table;
    fn resolve(&mut self, _: (), address: u32) -> ((), Table) {
        if address & 0x8000 == 0 {
            self.content.warning(address, self.game);
            self.game.g_fail = true;
        }
        (
            (),
            Table {
                image: self.content.image(self.game),
                offset: crate::rtl::lorom_offset(address) as usize,
            },
        )
    }
    fn bts(&self) -> u8 {
        let game = &*self.game;
        game.g_ram[0x16402 + usize::from(word!(game.cur_block_index))]
    }
    fn area(&self) -> u16 {
        let game = &*self.game;
        word!(game.area_index)
    }
}
#[forbid(unsafe_code)]
impl<C: Content> Context<()> for Room<'_, C> {
    type Header = u16;
    fn byte_limit(&self) -> u16 {
        let game = &*self.game;
        word!(game.room_size_in_blocks)
    }
    fn cursor(&self) -> u16 {
        self.game.cur_block_index()
    }
    fn select(&mut self, cell: u16) {
        let game = &mut *self.game;
        word!(game.cur_block_index = cell);
    }
    fn material(&self) -> Material {
        reactions::material(self.game.current_level_block())
    }
    fn extension(&self) -> i16 {
        i16::from(self.game.current_block_bts() as i8)
    }
    fn width(&self) -> u16 {
        self.game.room_width_in_blocks()
    }
    fn slope(&self, _: &(), axis: Axis, scan: &mut impl Progress) -> bool {
        reactions::slope(axis, scan, &*self.game)
    }
    fn mechanism(
        &mut self,
        owner: (),
        bombable: bool,
        solid: bool,
    ) -> ((), Option<Mechanism<u16>>) {
        reactions::mechanism(owner, bombable, solid, self)
    }
    fn spawn(&mut self, _: (), header: u16) {
        self.content.spawn(header, self.game);
    }
}
#[forbid(unsafe_code)]
impl SlopeInputs for Game {
    fn block(&self) -> u16 {
        word!(self.cur_block_index)
    }
    fn selected(&self) -> u16 {
        word!(self.projectile_index)
    }
    fn width(&self) -> u16 {
        word!(self.room_width_in_blocks)
    }
    fn bts(&self, block: u16) -> u8 {
        self.g_ram[0x16402 + usize::from(block)]
    }
    fn position(&self, slot: u16, axis: Axis) -> u16 {
        match axis {
            Axis::X => word!(self.projectile_x_pos[slot >> 1]),
            Axis::Y => word!(self.projectile_y_pos[slot >> 1]),
        }
    }
    fn radius(&self, slot: u16, axis: Axis) -> u16 {
        match axis {
            Axis::X => word!(self.projectile_x_radius[slot >> 1]),
            Axis::Y => word!(self.projectile_y_radius[slot >> 1]),
        }
    }
    fn square_position(&self, slot: u16, axis: Axis) -> u16 {
        let at = if axis == Axis::X { 0xb64 } else { 0xb78 } + 2 * usize::from(slot >> 1);
        u16::from_ne_bytes([self.g_ram[at], self.g_ram[at + 1]])
    }
    fn square_radius(&self, slot: u16, axis: Axis) -> u16 {
        let at = if axis == Axis::X { 0xbb4 } else { 0xbc8 } + 2 * usize::from(slot >> 1);
        u16::from_ne_bytes([self.g_ram[at], self.g_ram[at + 1]])
    }
    fn square_probe(&self, axis: Axis) {
        crate::verify_probe::hit(if axis == Axis::X {
            "square-slope.shot.horizontal"
        } else {
            "square-slope.shot.vertical"
        });
    }
    fn quadrant(&self, index: usize) -> bool {
        crate::game::sm_94::kTab948E54[index] != 0
    }
    fn height(&self, index: usize) -> u8 {
        crate::game::sm_94::kAlignYPos_Tab0[index]
    }
}
#[forbid(unsafe_code)]
pub fn react(scan: &mut impl Progress, block: u16, axis: Axis, game: &mut Game) -> u8 {
    reactions::react_progress(
        (),
        axis,
        block,
        scan,
        &mut Room {
            game,
            content: Native,
        },
    )
    .1
}
#[forbid(unsafe_code)]
pub fn trigger(scan: &mut impl Progress, bombable: bool, solid: bool, game: &mut Game) -> u8 {
    reactions::trigger(
        (),
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
