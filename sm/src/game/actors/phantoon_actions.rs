//! Phantoon's actions run in `sm_game::actors::phantoon_actions`; the fireball
//! spawn comes from here.
#![forbid(unsafe_code)]
use crate::game::constants::bank_86::addr_kEproj_DestroyableFireballs;
use crate::game::enemy_projectiles::SpawnEprojWithGfx;
use crate::game_state::Game;
use crate::types::PairU16;
use sm_game::actors::phantoon_actions;
pub(crate) use sm_game::actors::phantoon_actions::{
    Host, broken_nothingness, grow_or_shrink_wave_amplitude, spawn_casual_flame, spawn_flame_rain,
    spawn_flame_spiral,
};

/// Get Phantoon health-based palette colour ($A7:DC0F), as a `PairU16`.
pub(crate) fn health_palette_colour(base: u16, game: &mut Game) -> PairU16 {
    let (k, j) = phantoon_actions::health_palette_colour(base, game);
    PairU16 { k, j }
}

impl Host for Game {
    fn spawn_destroyable_flame(&mut self, parameter: u16, k: u16) {
        SpawnEprojWithGfx(parameter, k, addr_kEproj_DestroyableFireballs as u16, self);
    }
}
