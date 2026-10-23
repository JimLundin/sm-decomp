//! Checked escape fields; the shared liquid backend still owns its motion state.
#![forbid(unsafe_code)]
use crate::game::mechanism_booster_escape::{self, Context, Stage};
use crate::game::progress_flags::{CheckEventHappened, SetEventHappened};
use crate::game_state::Game;
pub struct EscapeFields<'a>(pub &'a mut Game);
impl EscapeFields<'_> {
    fn word(&self, at: usize) -> u16 {
        u16::from_le_bytes([self.0.g_ram[at], self.0.g_ram[at + 1]])
    }
    fn put(&mut self, at: usize, value: u16) {
        self.0.g_ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
    }
}
impl Context for EscapeFields<'_> {
    fn completed(&mut self) -> bool {
        CheckEventHappened(0x15, self.0) != 0
    }
    fn complete(&mut self) {
        SetEventHappened(0x15, self.0)
    }
    fn booster_collected(&self) -> bool {
        self.word(0x9a4) & 0x2000 != 0
    }
    fn player_x(&self) -> u16 {
        self.word(0xaf6)
    }
    fn target(&self) -> u16 {
        self.word(0x197a)
    }
    fn set_target(&mut self, v: u16) {
        self.put(0x197a, v)
    }
    fn height(&self) -> u16 {
        self.word(0x1978)
    }
    fn set_height(&mut self, v: u16) {
        self.put(0x1978, v)
    }
    fn set_velocity(&mut self, v: u16) {
        self.put(0x197c, v)
    }
    fn set_motion_delay(&mut self, v: u16) {
        self.put(0x1980, v)
    }
    fn stop_quake(&mut self) {
        self.put(0x1840, 0)
    }
    fn release(&mut self, s: usize) {
        self.put(0x1c37 + 2 * s, 0)
    }
    fn set_instruction_timer(&mut self, s: usize, v: u16) {
        self.put(0xde1c + 2 * s, v)
    }
    fn advance_program_one(&mut self, s: usize) {
        let at = 0x1d27 + 2 * s;
        self.put(at, self.word(at).wrapping_add(1))
    }
    fn progress(&self, s: usize) -> u16 {
        self.word(0x1d77 + 2 * s)
    }
    fn set_progress(&mut self, s: usize, v: u16) {
        self.put(0x1d77 + 2 * s, v)
    }
}
pub fn begin(j: u16, g: &mut Game) -> u8 {
    mechanism_booster_escape::begin(&mut EscapeFields(g), usize::from(j >> 1))
}
pub fn poll(stage: Stage, j: u16, g: &mut Game) {
    mechanism_booster_escape::poll(&mut EscapeFields(g), stage, usize::from(j >> 1));
}
