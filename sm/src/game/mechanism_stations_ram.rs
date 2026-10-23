//! Checked station fields with explicit coordinate and player-lock dependencies.
use crate::game::mechanism_stations::{self, Context, Pose, Service, Setup, Side, StationBlock};
use crate::game_state::Game;
pub trait Effects {
    fn coordinates(&mut self, slot: usize, game: &mut Game);
    fn lock_player(&mut self, game: &mut Game);
}
pub struct RuntimeEffects;
impl Effects for RuntimeEffects {
    fn coordinates(&mut self, slot: usize, game: &mut Game) {
        unsafe {
            crate::game::sm_84::CalculatePlmBlockCoords((slot * 2) as u16, game);
        }
    }
    fn lock_player(&mut self, game: &mut Game) {
        unsafe {
            crate::game::sm_90::CallSomeSamusCode(6, game);
        }
    }
}
pub struct StationFields<'a, E> {
    pub game: &'a mut Game,
    pub effects: E,
}
impl<E> StationFields<'_, E> {
    fn word(&self, at: usize) -> u16 {
        u16::from_le_bytes([self.game.g_ram[at], self.game.g_ram[at + 1]])
    }
    fn put(&mut self, at: usize, value: u16) {
        self.game.g_ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
    }
}
impl<E: Effects> Context for StationFields<'_, E> {
    type Block = u16;
    type Program = u16;
    fn block(&self, slot: usize) -> u16 {
        self.word(0x1c87 + 2 * slot)
    }
    fn adjacent(&self, block: u16, displacement: i16) -> u16 {
        block.wrapping_add(displacement as u16)
    }
    fn clear_block(&mut self, slot: usize) {
        self.put(0x1c87 + 2 * slot, 0);
    }
    fn release(&mut self, slot: usize) {
        self.put(0x1c37 + 2 * slot, 0);
    }
    fn direction(&self) -> u16 {
        self.word(0xb02)
    }
    fn pose(&self, pose: Pose) -> bool {
        self.word(0xa1c)
            == match pose {
                Pose::FacingLeftWall => 0x8a,
                Pose::FacingRightWall => 0x89,
                Pose::StandingRight => 1,
                Pose::StandingLeft => 2,
            }
    }
    fn facing(&self, side: Side) -> bool {
        self.game.g_ram[0xa1e]
            & match side {
                Side::Right => 4,
                Side::Left => 8,
            }
            != 0
    }
    fn needs_service(&self, service: Service) -> bool {
        let at = match service {
            Service::Energy => 0x9c2,
            Service::Missile => 0x9c6,
            Service::Map => unreachable!(),
        };
        self.word(at) != self.word(at + 2)
    }
    fn selected(&self) -> usize {
        usize::from(self.word(0x1c27) >> 1)
    }
    fn coordinates(&mut self, slot: usize) {
        self.effects.coordinates(slot, self.game);
    }
    fn cannon_aligned(&self) -> bool {
        16u16.wrapping_mul(self.word(0x1c2b)) | 0xb == self.word(0xafa)
    }
    fn save_aligned(&self) -> bool {
        self.word(0xaf6).wrapping_sub(8) >> 4 == self.word(0x1c29)
    }
    fn return_program(&self, slot: usize) -> u16 {
        self.word(0xdebc + 2 * slot)
    }
    fn set_program(&mut self, slot: usize, program: u16) {
        self.put(0x1d27 + 2 * slot, program);
    }
    fn advance_one(&mut self, slot: usize) {
        let at = 0x1d27 + 2 * slot;
        self.put(at, self.word(at).wrapping_add(1));
    }
    fn set_timer(&mut self, slot: usize) {
        self.put(0xde1c + 2 * slot, 1);
    }
    fn lock_player(&mut self) {
        self.effects.lock_player(self.game);
    }
    fn power_bomb_active(&self) -> bool {
        self.word(0x592) != 0
    }
    fn save_locked(&self) -> bool {
        self.word(0x1e75) != 0
    }
    fn solidify(&mut self, block: u16) {
        let at = 0x10002 + 2 * usize::from(block >> 1);
        self.put(at, self.word(at) & 0xfff | 0x8000);
    }
    fn map_used(&self) -> bool {
        self.game.g_ram[0xd908 + usize::from(self.word(0x79f))] != 0
    }
    fn map_used_program(&self) -> u16 {
        0xad76
    }
    fn install(&mut self, block: u16, tile: StationBlock) {
        let value = match tile {
            StationBlock::Access(Service::Map, Side::Right) => 0xb047,
            StationBlock::Access(Service::Map, Side::Left) => 0xb048,
            StationBlock::Access(Service::Energy, Side::Right) => 0xb049,
            StationBlock::Access(Service::Energy, Side::Left) => 0xb04a,
            StationBlock::Access(Service::Missile, Side::Right) => 0xb04b,
            StationBlock::Access(Service::Missile, Side::Left) => 0xb04c,
            StationBlock::Save => 0xb04d,
        };
        crate::game::sm_84::WriteLevelDataBlockTypeAndBts(block, value, self.game);
    }
}
/// # Safety
/// Runtime state must satisfy the retained coordinate and player-lock contracts.
pub unsafe fn setup(j: u16, kind: Setup, game: &mut Game) -> u8 {
    mechanism_stations::setup(
        &mut StationFields {
            game,
            effects: RuntimeEffects,
        },
        usize::from(j >> 1),
        kind,
    )
}
