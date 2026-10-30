//! Phantoon's actions run in `sm_game::actors::phantoon_actions`; the fireball
//! spawn comes from here.
#![forbid(unsafe_code)]
use crate::game::constants::bank_86::addr_kEproj_DestroyableFireballs;
use crate::game::enemy_projectiles::SpawnEprojWithGfx;
use crate::game_state::Game;
pub(crate) use sm_game::actors::phantoon_actions::{Host, spawn_casual_flame};

impl Host for Game {
    fn spawn_destroyable_flame(&mut self, parameter: u16, k: u16) {
        SpawnEprojWithGfx(parameter, k, addr_kEproj_DestroyableFireballs as u16, self);
    }
}
