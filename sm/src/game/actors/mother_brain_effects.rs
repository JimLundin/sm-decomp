//! Mother Brain's effects run in `sm_game::actors::mother_brain_effects`; the
//! explosion spawns come from here.
#![forbid(unsafe_code)]
use crate::game::constants::bank_86::ADDR_K_EPROJ_MOTHER_BRAIN_DEATH_EXPLOSION;
use crate::game::enemy_projectiles::spawn_with_room_gfx;
use crate::game_state::Game;
use crate::types::Point16U;
pub(crate) use sm_game::actors::mother_brain_effects::*;

impl Host for Game {
    fn set_projectile_spawn_point(&mut self, x: u16, y: u16) {
        self.eproj_spawn_pt = Point16U { x, y };
    }

    fn spawn_death_explosion(&mut self, kind: u16) {
        spawn_with_room_gfx(ADDR_K_EPROJ_MOTHER_BRAIN_DEATH_EXPLOSION as u16, kind, self);
    }
}
