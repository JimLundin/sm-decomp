//! Phantoon's actions run in `sm_game::actors::phantoon_actions`; the fireball
//! spawn comes from here.
#![forbid(unsafe_code)]
use crate::game::constants::bank_86::addr_kEproj_DestroyableFireballs;
use crate::game::enemy_projectiles::SpawnEprojWithGfx;
use crate::game_state::Game;
use crate::types::PairU16;
use sm_game::actors::phantoon_actions;
pub(crate) use sm_game::actors::phantoon_actions::{
    Host, approach_speed, debug_toggle_sprite_map, spawn_fireball, spawn_fireball_circle,
    spawn_fireball_pattern,
};

#[allow(non_snake_case)]
pub(crate) fn Phantoon_SetColorBasedOnHp_FindEntry(base: u16, game: &mut Game) -> PairU16 {
    let (k, j) = phantoon_actions::Phantoon_SetColorBasedOnHp_FindEntry(base, game);
    PairU16 { k, j }
}

impl Host for Game {
    fn spawn_fireball_projectile(&mut self, parameter: u16, k: u16) {
        SpawnEprojWithGfx(parameter, k, addr_kEproj_DestroyableFireballs as u16, self);
    }
}
