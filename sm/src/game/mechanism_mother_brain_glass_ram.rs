//! Checked glass fields; native shard construction remains an explicit callback.
#![forbid(unsafe_code)]
use crate::game::mechanism_mother_brain_glass::{self as glass, Command, Context, Program};
use crate::game::enemy_script::EnemyScript;
use crate::game_state::Game;
fn word(g: &Game, at: usize) -> u16 {
    u16::from_le_bytes([g.g_ram[at], g.g_ram[at + 1]])
}
fn put(g: &mut Game, at: usize, v: u16) {
    g.g_ram[at..at + 2].copy_from_slice(&v.to_le_bytes());
}
pub struct GlassFields<'a>(pub &'a mut Game);
impl Context for GlassFields<'_> {
    type Block = u16;
    fn argument(&self, s: usize) -> u16 {
        word(self.0, 0x1dc7 + 2 * s)
    }
    fn set_argument(&mut self, s: usize, v: u16) {
        put(self.0, 0x1dc7 + 2 * s, v)
    }
    fn signal(&self, s: usize) -> u16 {
        word(self.0, 0x1d77 + 2 * s)
    }
    fn clear_signal(&mut self, s: usize) {
        put(self.0, 0x1d77 + 2 * s, 0)
    }
    fn block(&self, s: usize) -> u16 {
        word(self.0, 0x1c87 + 2 * s)
    }
    fn install(&mut self, b: u16) {
        crate::game::sm_84::WriteLevelDataBlockTypeAndBts(b, 0x8044, self.0)
    }
}
pub fn setup(j: u16, g: &mut Game) -> u8 {
    glass::setup(&mut GlassFields(g), usize::from(j >> 1))
}
pub fn poll(j: u16, g: &mut Game) {
    glass::poll(&mut GlassFields(g), usize::from(j >> 1))
}
pub fn add_hits(slot: usize, amount: u16, g: &mut Game) {
    glass::add_hits(&mut GlassFields(g), slot, amount)
}
struct Script<'a> {
    game: &'a mut Game,
    position: EnemyScript,
    spawn: fn(u16, &mut Game),
}
impl Program for Script<'_> {
    type Position = EnemyScript;
    fn argument(&self, s: usize) -> u16 {
        word(self.game, 0x1dc7 + 2 * s)
    }
    fn word(&self, offset: isize) -> u16 {
        self.position.word(offset)
    }
    fn jump(&self, address: u16) -> EnemyScript {
        EnemyScript::jump(address)
    }
    fn advance(&self, bytes: isize) -> EnemyScript {
        self.position.advance(bytes)
    }
    fn sound(&mut self) {
        crate::game::sm_80::QueueSfx3_Max15(0x2e, self.game)
    }
    fn shard(&mut self, parameter: u16) {
        (self.spawn)(parameter, self.game)
    }
}
pub fn command(
    position: EnemyScript,
    j: u16,
    kind: Command,
    game: &mut Game,
    spawn: fn(u16, &mut Game),
) -> EnemyScript {
    glass::command(
        &mut Script {
            game,
            position,
            spawn,
        },
        usize::from(j >> 1),
        kind,
    )
}
