//! Checked gate fields with the retained sound and enemy-projectile boundaries.
use crate::game::mechanism_gates::{self, Context, Direction, Setup, Side};
use crate::game_state::Game;

pub struct GateFields<'a>(pub &'a mut Game);
impl GateFields<'_> {
    fn word(&self, at: usize) -> u16 {
        u16::from_le_bytes([self.0.g_ram[at], self.0.g_ram[at + 1]])
    }
    fn put(&mut self, at: usize, value: u16) {
        self.0.g_ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
    }
    fn table(&self, slot: usize, direction: Direction, offset: usize) -> u16 {
        let image = self
            .0
            .loaded_rom
            .as_ref()
            .expect("ROM must be loaded")
            .clone();
        let argument = self.word(0x1dc7 + 2 * slot);
        let base = match direction {
            Direction::Down => 0x84c70a,
            Direction::Up => 0x84c764,
        };
        image.table_word(
            crate::rtl::lorom_offset(base) as usize + offset,
            (argument >> 1) as isize,
        )
    }
}
impl Context for GateFields<'_> {
    type Block = u16;
    type Row = u16;
    type Program = u16;
    fn block(&self, slot: usize) -> u16 {
        self.word(0x1c87 + 2 * slot)
    }
    fn neighbor(&self, block: u16, side: Side) -> u16 {
        match side {
            Side::Left => block.wrapping_sub(2),
            Side::Right => block.wrapping_add(2),
        }
    }
    fn timer(&self, slot: usize) -> u16 {
        self.word(0x1d77 + 2 * slot)
    }
    fn set_timer(&mut self, slot: usize, value: u16) {
        self.put(0x1d77 + 2 * slot, value);
    }
    fn release(&mut self, slot: usize) {
        self.put(0x1c37 + 2 * slot, 0);
    }
    fn projectile_type(&self) -> u16 {
        let at = 0xc18 + 2 * usize::from(self.word(0xdde) >> 1);
        u16::from_ne_bytes([self.0.g_ram[at], self.0.g_ram[at + 1]])
    }
    fn rejection_sound(&mut self) {
        crate::game::sm_80::QueueSfx2_Max6(0x57, self.0);
    }
    fn spawn_closed_gate(&mut self, direction: Direction, accumulator: u16) {
        let definition = match direction {
            Direction::Down => 0xe659,
            Direction::Up => 0xe675,
        };
        unsafe {
            crate::game::sm_86::SpawnEprojWithRoomGfx(definition, accumulator, self.0);
        }
    }
    fn row(&self, block: u16) -> u16 {
        block >> 1
    }
    fn next_behavior(&self, row: u16) -> u8 {
        self.0.g_ram[0x16403 + usize::from(row)]
    }
    fn set_behavior_word(&mut self, row: u16, value: u16) {
        let at = 0x16402 + usize::from(row);
        self.0.g_ram[at..at + 2].copy_from_slice(&value.to_ne_bytes());
    }
    fn advance_row(&self, row: u16, direction: Direction) -> u16 {
        let width = self.word(0x7a5);
        match direction {
            Direction::Down => row.wrapping_add(width),
            Direction::Up => row.wrapping_sub(width),
        }
    }
    fn program(&self, slot: usize, direction: Direction) -> u16 {
        self.table(slot, direction, 0)
    }
    fn set_program(&mut self, slot: usize, program: u16) {
        self.put(0x1d27 + 2 * slot, program);
    }
    fn optional_block(&self, slot: usize, direction: Direction, side: Side) -> u16 {
        self.table(slot, direction, if side == Side::Left { 16 } else { 32 })
    }
    fn install_block(&mut self, block: u16, value: u16) {
        let high = 0x10003 + usize::from(block);
        self.0.g_ram[high] = (value >> 8) as u8 | (self.0.g_ram[high] & 15);
        self.0.g_ram[0x16402 + usize::from(block >> 1)] = value as u8;
    }
}
pub fn setup(j: u16, kind: Setup, accumulator: u16, game: &mut Game) -> u8 {
    mechanism_gates::setup(
        &mut GateFields(game),
        usize::from(j >> 1),
        kind,
        accumulator,
    )
}
