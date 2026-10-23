//! Checked statue fields; RAM remains authoritative across music and debris effects.
#![forbid(unsafe_code)]
use crate::game::mechanism_torizo_statue::{self as statue, Actor, Plm};
use crate::game::enemy_script::EnemyScript;
use crate::game_state::Game;
use crate::ram_map::words as ram;

fn word(g: &Game, at: usize) -> u16 {
    u16::from_le_bytes([g.g_ram[at], g.g_ram[at + 1]])
}
fn put(g: &mut Game, at: usize, value: u16) {
    g.g_ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
pub struct StatueFields<'a>(pub &'a mut Game);
impl Plm for StatueFields<'_> {
    fn defeated(&mut self) -> bool {
        crate::game::progress_flags::CheckBossBitForCurArea(4, self.0) != 0
    }
    fn release(&mut self, slot: usize) {
        put(self.0, 0x1c37 + 2 * slot, 0);
    }
    fn bombs_collected(&self) -> bool {
        word(self.0, ram::collected_items) & 0x1000 != 0
    }
    fn set_timer(&mut self, slot: usize, value: u16) {
        put(self.0, 0xde1c + 2 * slot, value);
    }
    fn instruction(&self, slot: usize) -> u16 {
        word(self.0, 0x1d27 + 2 * slot)
    }
    fn set_instruction(&mut self, slot: usize, value: u16) {
        put(self.0, 0x1d27 + 2 * slot, value);
    }
    fn stop_polling(&mut self, slot: usize) {
        put(self.0, 0x1cd7 + 2 * slot, 0xd356);
    }
}
pub fn setup(j: u16, game: &mut Game) -> u8 {
    statue::setup(&mut StatueFields(game), usize::from(j >> 1))
}
pub fn poll(j: u16, game: &mut Game) {
    statue::poll(&mut StatueFields(game), usize::from(j >> 1));
}
pub fn debris(position: EnemyScript, game: &mut Game, spawn: fn(u16, &mut Game)) -> EnemyScript {
    spawn(position.word(0), game);
    position.advance(2)
}
pub fn music(game: &mut Game) {
    crate::game::music::QueueMusic_Delayed8(6, game);
}

pub struct ActorFields<'a> {
    pub game: &'a mut Game,
    pub index: u16,
    pub music: fn(&mut Game),
}
impl ActorFields<'_> {
    fn word(&self, offset: usize) -> u16 {
        let at = 0xf78 + usize::from(self.index) + offset;
        u16::from_ne_bytes([self.game.g_ram[at], self.game.g_ram[at + 1]])
    }
    fn put(&mut self, offset: usize, value: u16) {
        let at = 0xf78 + usize::from(self.index) + offset;
        self.game.g_ram[at..at + 2].copy_from_slice(&value.to_ne_bytes());
    }
}
impl Actor for ActorFields<'_> {
    fn set_tangible(&mut self, value: bool) {
        let properties = self.word(core::mem::offset_of!(crate::types::EnemyData, properties));
        self.put(
            core::mem::offset_of!(crate::types::EnemyData, properties),
            if value {
                properties | 0x400
            } else {
                properties & !0x400
            },
        );
    }
    fn statue_present(&self, slot: usize) -> bool {
        word(self.game, 0x1c37 + 2 * slot) == 0xd6ea
    }
    fn music(&mut self) {
        (self.music)(self.game);
    }
    fn instruction(&self) -> u16 {
        self.word(core::mem::offset_of!(
            crate::types::EnemyData,
            current_instruction
        ))
    }
    fn set_instruction(&mut self, value: u16) {
        self.put(
            core::mem::offset_of!(crate::types::EnemyData, current_instruction),
            value,
        );
    }
    fn set_timer(&mut self, value: u16) {
        self.put(
            core::mem::offset_of!(crate::types::EnemyData, instruction_timer),
            value,
        );
    }
}
pub fn wait(index: u16, game: &mut Game) {
    statue::wait(&mut ActorFields { game, index, music });
}
