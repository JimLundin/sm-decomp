//! One direct impact/retirement binding, including broad byte aliases.
use super::{Context, Transaction};
use crate::game::projectile_lifecycle::{self as life, *};
use crate::game::projectile_room::Axis;
use crate::game::sm_80::QueueSfx2_Max6;
use crate::ram_map::word_arrays as arrays;
impl Kind for u16 {
    fn impact_family(self) -> Family {
        if self & 0xf00 == 0 {
            Family::Beam
        } else {
            Family::Ballistic
        }
    }
    fn uses_heavy_impact(self) -> bool {
        self & 0x200 != 0
    }
    fn enter_impact(self, family: Family) -> Self {
        self & 0xf0ff
            | match family {
                Family::Beam => 0x700,
                Family::Ballistic => 0x800,
            }
    }
}
impl State for Transaction<'_> {
    type Kind = u16;
    type Cursor = u16;
    fn kind(&self, slot: usize) -> u16 {
        word!(self.projectile_type[slot])
    }
    fn set_kind(&mut self, slot: usize, kind: u16) {
        word!(self.projectile_type[slot] = kind);
    }
    fn cinematic(&self) -> bool {
        word!(self.cinematic_function) != 0
    }
    fn set_cursor(&mut self, slot: usize, cursor: u16) {
        word!(self.projectile_bomb_instruction_ptr[slot] = cursor);
    }
    fn cooldown(&self) -> u16 {
        word!(self.cooldown_timer)
    }
    fn set_cooldown(&mut self, cooldown: u16) {
        word!(self.cooldown_timer = cooldown);
    }
    fn finish_impact(&mut self, slot: usize) {
        word!(self.projectile_bomb_instruction_timers[slot] = 1);
        word!(self.projectile_damage[slot] = 8);
    }
}
impl<'a> Definitions<Transaction<'a>> for Context {
    fn entry(&mut self, game: Transaction<'a>, entry: Entry) -> (Transaction<'a>, u16) {
        let address = match entry {
            Entry::BeamImpact => 0x93867b,
            Entry::MissileImpact => 0x93867f,
            Entry::HeavyImpact => 0x938693,
            Entry::BombExplosion => 0x938683,
        };
        let cursor = game
            .rom()
            .table_word(crate::rtl::lorom_offset(address) as usize, 0);
        (game, cursor)
    }
    fn feedback(&mut self, mut game: Transaction<'a>, feedback: Feedback) -> Transaction<'a> {
        match feedback {
            Feedback::BeamImpact => game = game.foreign(|g| QueueSfx2_Max6(12, g)).0,
            Feedback::MissileImpact => game = game.foreign(|g| QueueSfx2_Max6(7, g)).0,
            Feedback::HeavyImpactShake => {
                word!(game.earthquake_type = 20);
                word!(game.earthquake_timer = 30);
            }
        }
        game
    }
}
impl<'a> RetirementContext<Transaction<'a>> for Context {
    fn retirement(&self, game: &Transaction<'_>, slot: u16) -> Retirement {
        match word!(game.projectile_type[(slot >> 1)]) & 0xf00 {
            0 => Retirement::Beam,
            0x100 | 0x200 => Retirement::Missile,
            _ => Retirement::Release,
        }
    }
    fn move_to_impact(&mut self, mut game: Transaction<'a>, slot: u16) -> Transaction<'a> {
        let direction = usize::from(word!(game.projectile_dir[(slot >> 1)]) & 0xf);
        let directions = DIRECTIONS[direction];
        let index = usize::from(slot & !1);
        impact_offsets(directions, |axis, step| {
            let (position, radius) = match axis {
                Axis::X => (arrays::projectile_x_pos, arrays::projectile_x_radius),
                Axis::Y => (arrays::projectile_y_pos, arrays::projectile_y_radius),
            };
            let at = position + index;
            let position = game.read_le(at);
            let radius = radius + index;
            let radius = game.read_le(radius);
            game.write_slice_le(at, offset(position, radius, step));
        });
        game
    }
    fn impact(&mut self, game: Transaction<'a>, slot: u16) -> Transaction<'a> {
        life::impact(game, usize::from(slot >> 1), self)
    }
    fn release(&mut self, game: Transaction<'a>, slot: u16) -> Transaction<'a> {
        super::motion::clear(slot, game)
    }
    fn stop(&mut self, game: &mut Transaction<'a>, slot: u16) {
        word!(game.projectile_bomb_pre_instructions[(slot >> 1)] = 0xb169);
    }
}
pub(super) fn impact(slot: u16, game: Transaction<'_>) -> Transaction<'_> {
    life::impact(game, usize::from(slot >> 1), &mut Context::default())
}
pub(super) fn retire(slot: u16, game: Transaction<'_>) -> Transaction<'_> {
    life::retire(game, slot, &mut Context::default())
}
