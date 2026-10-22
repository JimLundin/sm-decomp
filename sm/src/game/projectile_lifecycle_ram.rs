//! Checked SNES encoding of projectile impact and retirement.
#![forbid(unsafe_code)]
use crate::game::{
    projectile_lifecycle::{self as life, *},
    projectile_room::Axis,
};
use crate::game::sm_80::QueueSfx2_Max6;
use crate::game::sm_90::ClearProjectile;
use crate::game_state::Game;
use crate::ram_map::{checked_word as word, word_arrays as arrays};

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
impl State for &mut Game {
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
struct Ram;
impl<'a> Definitions<&'a mut Game> for Ram {
    fn entry(&mut self, game: &'a mut Game, entry: Entry) -> (&'a mut Game, u16) {
        let address = match entry {
            Entry::BeamImpact => 0x93867b,
            Entry::MissileImpact => 0x93867f,
            Entry::HeavyImpact => 0x938693,
            Entry::BombExplosion => 0x938683,
        };
        let cursor = crate::rtl::rom_table_word!(game, address, 0);
        (game, cursor)
    }
    fn feedback(&mut self, game: &'a mut Game, feedback: Feedback) -> &'a mut Game {
        match feedback {
            Feedback::BeamImpact => QueueSfx2_Max6(12, game),
            Feedback::MissileImpact => QueueSfx2_Max6(7, game),
            Feedback::HeavyImpactShake => {
                word!(game.earthquake_type = 20);
                word!(game.earthquake_timer = 30);
            }
        }
        game
    }
}
impl<'a> RetirementContext<&'a mut Game> for Ram {
    fn retirement(&self, game: &&mut Game, slot: u16) -> Retirement {
        match word!(game.projectile_type[(slot >> 1)]) & 0xf00 {
            0 => Retirement::Beam,
            0x100 | 0x200 => Retirement::Missile,
            _ => Retirement::Release,
        }
    }
    fn move_to_impact(&mut self, game: &'a mut Game, slot: u16) -> &'a mut Game {
        let direction = usize::from(word!(game.projectile_dir[(slot >> 1)]) & 0xf);
        let directions = DIRECTIONS[direction];
        let index = usize::from(slot & !1);
        impact_offsets(directions, |axis, step| {
            let (position, radius) = match axis {
                Axis::X => (arrays::projectile_x_pos, arrays::projectile_x_radius),
                Axis::Y => (arrays::projectile_y_pos, arrays::projectile_y_radius),
            };
            let at = position + index;
            let position = u16::from_le_bytes([game.g_ram[at], game.g_ram[at + 1]]);
            let radius = radius + index;
            let radius = u16::from_le_bytes([game.g_ram[radius], game.g_ram[radius + 1]]);
            game.g_ram[at..at + 2].copy_from_slice(&offset(position, radius, step).to_le_bytes());
        });
        game
    }
    fn impact(&mut self, game: &'a mut Game, slot: u16) -> &'a mut Game {
        life::impact(game, usize::from(slot >> 1), self)
    }
    fn release(&mut self, game: &'a mut Game, slot: u16) -> &'a mut Game {
        ClearProjectile(slot, game);
        game
    }
    fn stop(&mut self, game: &mut &'a mut Game, slot: u16) {
        word!(game.projectile_bomb_pre_instructions[(slot >> 1)] = 0xb169);
    }
}
pub fn impact(slot: u16, game: &mut Game) {
    life::impact(game, usize::from(slot >> 1), &mut Ram);
}
pub fn retire(slot: u16, game: &mut Game) {
    life::retire(game, slot, &mut Ram);
}
