//! Scoped tube fields and explicit runtime effects; RAM remains authoritative.
#![forbid(unsafe_code)]
use crate::game::mechanism_noob_tube::{
    self as tube, Block, Command, Effect, Effects, Setup, Terrain, Wake,
};
use crate::game_state::Game;
use crate::ram_map::words as ram;
fn word(g: &Game, at: usize) -> u16 {
    u16::from_le_bytes([g.g_ram[at], g.g_ram[at + 1]])
}
fn put(g: &mut Game, at: usize, value: u16) {
    g.g_ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
pub struct TubeFields<'a>(pub &'a mut Game);
impl Terrain for TubeFields<'_> {
    fn install(&mut self, slot: usize, block: Block) {
        let address = 0x1c87 + 2 * slot;
        // Each installation reacquires its inputs after the preceding byte writes.
        let index = match block {
            Block::Air | Block::Solid => word(self.0, address),
            Block::RightExtension => word(self.0, address).wrapping_add(2),
            Block::BelowExtension | Block::BelowRightExtension => {
                let width = word(self.0, ram::room_width_in_blocks);
                let second_width = word(self.0, ram::room_width_in_blocks);
                width
                    .wrapping_add(second_width)
                    .wrapping_add(word(self.0, address))
                    .wrapping_add(if block == Block::BelowRightExtension {
                        2
                    } else {
                        0
                    })
            }
        };
        let value = match block {
            Block::Air => 0x0044,
            Block::Solid => 0x8044,
            Block::RightExtension => 0x50ff,
            Block::BelowExtension | Block::BelowRightExtension => 0xd0ff,
        };
        crate::game::mechanism_terrain_setup_ram::install_block(index, value, self.0);
    }
}
impl Wake for TubeFields<'_> {
    fn pressed(&self) -> bool {
        word(self.0, ram::joypad1_newkeys) & 0xc3c0 != 0
    }
    fn set_timer(&mut self, slot: usize) {
        put(self.0, 0xde1c + 2 * slot, 1);
    }
    fn advance(&mut self, slot: usize) {
        let at = 0x1d27 + 2 * slot;
        put(self.0, at, word(self.0, at).wrapping_add(1));
    }
}
pub fn setup(j: u16, kind: Setup, game: &mut Game) -> u8 {
    tube::setup(&mut TubeFields(game), usize::from(j >> 1), kind)
}
pub fn poll(j: u16, game: &mut Game) {
    tube::poll(&mut TubeFields(game), usize::from(j >> 1));
}
struct Runtime<'a> {
    game: &'a mut Game,
    effect: fn(Effect, &mut Game),
}
impl Effects for Runtime<'_> {
    fn water(&mut self) {
        put(
            self.game,
            ram::fx_liquid_options,
            word(self.game, ram::fx_liquid_options) & !4,
        );
    }
    fn quake(&mut self) {
        put(self.game, ram::earthquake_type, 11);
        put(self.game, ram::earthquake_timer, 64);
    }
    fn emit(&mut self, effect: Effect) {
        (self.effect)(effect, self.game);
    }
}
pub fn command(opcode: u16, game: &mut Game, effect: fn(Effect, &mut Game)) {
    let kind = match opcode {
        0xd525 => Command::Water,
        0xd52c => Command::Crack,
        0xd536 => Command::Quake,
        0xd543 => Command::Shatter,
        0xd5e6 => Command::Lock,
        0xd5ee => Command::Unlock,
        _ => unreachable!("tube command"),
    };
    tube::command(&mut Runtime { game, effect }, kind);
}
