//! Mother Brain's effects run in `sm_game::actors::mother_brain_effects`; the
//! explosion spawns come from here.
#![forbid(unsafe_code)]
use crate::game::constants::bank_86::addr_kEproj_MotherBrainDeathExplosion;
use crate::game::enemy_projectiles::SpawnEprojWithRoomGfx;
use crate::game_state::Game;
use crate::types::Point16U;
pub(crate) use sm_game::actors::mother_brain_effects::*;

impl Host for Game {
    fn set_projectile_spawn_point(&mut self, x: u16, y: u16) {
        self.eproj_spawn_pt = Point16U { x, y };
    }

    fn spawn_death_explosion(&mut self, kind: u16) {
        SpawnEprojWithRoomGfx(addr_kEproj_MotherBrainDeathExplosion as u16, kind, self);
    }
}
