//! Checked Chozo fields with explicit progression, player-lock and creation effects.
use crate::game::mechanism_displacement::Displacement;
use crate::game::mechanism_hands::{self, Context, GroundedPose, Hand, ShipScrollPair};
use crate::game_state::Game;

pub trait Effects {
    fn boss_defeated(&mut self, game: &mut Game) -> bool;
    fn mark_event(&mut self, game: &mut Game);
    fn lock_player(&mut self, game: &mut Game);
    fn spawn_child(&mut self, hand: Hand, game: &mut Game);
}
pub struct HandFields<'a, E> {
    pub game: &'a mut Game,
    pub effects: E,
}
impl<E> HandFields<'_, E> {
    fn word(&self, at: usize) -> u16 {
        u16::from_le_bytes([self.game.g_ram[at], self.game.g_ram[at + 1]])
    }
    fn put(&mut self, at: usize, value: u16) {
        self.game.g_ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
    }
}
impl<E: Effects> Context for HandFields<'_, E> {
    type Block = u16;
    fn block_at(&self, x: u8, y: u8) -> u16 {
        (u16::from(y) * u16::from(self.word(0x7a5) as u8) + u16::from(x)).wrapping_mul(2)
    }
    fn install_trigger(&mut self, block: u16, hand: Hand) {
        // Preserve the exact level-byte offset and rounded BTS index.
        let high = 0x10003 + usize::from(block);
        self.game.g_ram[high] = (self.game.g_ram[high] & 0xf) | 0xb0;
        self.game.g_ram[0x16402 + usize::from(block >> 1)] = match hand {
            Hand::LowerNorfair => 0x83,
            Hand::WreckedShip => 0x80,
        };
    }
    fn selected_slot(&self) -> usize {
        usize::from(self.word(0x1c27) >> 1)
    }
    fn set_acid_base(&mut self, level: u16) {
        self.put(0x1978, level);
    }

    fn space_jump(&self) -> bool {
        self.word(0x9a4) & 0x200 != 0
    }
    fn boss_defeated(&mut self) -> bool {
        self.effects.boss_defeated(self.game)
    }
    fn downward(&self) -> bool {
        self.word(0xb02) & 15 == 3
    }
    fn matches_pose(&self, pose: GroundedPose) -> bool {
        self.word(0xa1c)
            == match pose {
                GroundedPose::MorphRight => 0x1d,
                GroundedPose::SpringRight => 0x79,
                GroundedPose::SpringLeft => 0x7a,
            }
    }
    fn mark_event(&mut self) {
        self.effects.mark_event(self.game);
    }
    fn signal_statue(&mut self) {
        let at = 0xf78 + core::mem::offset_of!(crate::types::EnemyData, parameter_1);
        self.game.g_ram[at..at + 2].copy_from_slice(&1u16.to_ne_bytes());
    }
    fn set_ship_scroll_pair(&mut self, pair: ShipScrollPair) {
        let (at, value) = match pair {
            ShipScrollPair::First => (0xcd27, 0x202u16),
            ShipScrollPair::Second => (0xcd2d, 0x101),
        };
        self.game.g_ram[at..at + 2].copy_from_slice(&value.to_ne_bytes());
    }
    fn block(&self, slot: usize) -> u16 {
        self.word(0x1c87 + 2 * slot)
    }
    fn level(&self, block: u16) -> u16 {
        self.word(0x10002 + 2 * usize::from(block >> 1))
    }
    fn set_level(&mut self, block: u16, value: u16) {
        self.put(0x10002 + 2 * usize::from(block >> 1), value);
    }
    fn lock_player(&mut self) {
        self.effects.lock_player(self.game);
    }
    fn spawn_child(&mut self, hand: Hand) {
        self.effects.spawn_child(hand, self.game);
    }
    fn release(&mut self, slot: usize) {
        self.put(0x1c37 + 2 * slot, 0);
    }
}
struct RuntimeEffects<'a, 'b> {
    displacement: &'a mut Displacement<'b>,
}
impl Effects for RuntimeEffects<'_, '_> {
    fn boss_defeated(&mut self, game: &mut Game) -> bool {
        crate::game::progress_flags::CheckBossBitForCurArea(1, game) & 1 != 0
    }
    fn mark_event(&mut self, game: &mut Game) {
        crate::game::progress_flags::SetEventHappened(0xc, game);
    }
    fn lock_player(&mut self, game: &mut Game) {
        unsafe {
            crate::game::sm_90::CallSomeSamusCode(0, game);
        }
    }
    fn spawn_child(&mut self, hand: Hand, game: &mut Game) {
        let (x, header) = match hand {
            Hand::LowerNorfair => (12, 0xd113),
            Hand::WreckedShip => (23, 0xd6f8),
        };
        unsafe {
            crate::game::plm_spawning::spawn_hardcoded_plm(
                crate::types::SpawnHardcodedPlmArgs::new(x, 29, header),
                self.displacement,
                game,
            );
        }
    }
}
/// # Safety
/// Runtime state must satisfy the existing player-lock and nested setup contracts.
pub unsafe fn setup(
    j: u16,
    hand: Hand,
    displacement: &mut Displacement<'_>,
    game: &mut Game,
) -> u8 {
    mechanism_hands::setup(
        &mut HandFields {
            game,
            effects: RuntimeEffects { displacement },
        },
        usize::from(j >> 1),
        hand,
    )
}

struct NoEffects;
impl Effects for NoEffects {
    fn boss_defeated(&mut self, _: &mut Game) -> bool {
        unreachable!()
    }
    fn mark_event(&mut self, _: &mut Game) {
        unreachable!()
    }
    fn lock_player(&mut self, _: &mut Game) {
        unreachable!()
    }
    fn spawn_child(&mut self, _: Hand, _: &mut Game) {
        unreachable!()
    }
}
fn fields(game: &mut Game) -> HandFields<'_, NoEffects> {
    HandFields {
        game,
        effects: NoEffects,
    }
}
pub fn prepare_ship_trigger(j: u16, game: &mut Game) -> u8 {
    mechanism_hands::prepare_ship_trigger(&mut fields(game), usize::from(j >> 1))
}
pub fn monitor_norfair(game: &mut Game) {
    mechanism_hands::monitor_norfair(&mut fields(game));
}
pub fn lower_acid(game: &mut Game) {
    mechanism_hands::lower_acid(&mut fields(game));
}
