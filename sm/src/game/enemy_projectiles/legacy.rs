//! Qualified legacy effects; these are not counted as unsafe elimination.
use crate::game::{actors, session_runtime::Runtime};
use crate::game_state::Game;

pub(super) fn next_random(game: &mut Game) -> u16 {
    unsafe { crate::game::sm_80::NextRandom(game) }
}

pub(super) fn call_player_code(code: u16, game: &mut Game) -> u16 {
    crate::game::session_runtime::player_command(code, game)
}

pub(super) fn calculate_angle_from_xy(x: u16, y: u16, _game: &mut Game) -> u16 {
    crate::game::actors::api::angle_from_delta(x, y)
}

pub(super) fn damage_player(damage: u16, game: &mut Game) -> () {
    unsafe { crate::game::sm_91::Samus_DealDamage(damage, game) }
}

pub(super) fn ridley_offscreen(x: u16, y: u16, game: &mut Game) -> u8 {
    unsafe { crate::game::sm_a6::Ridley_Func_103(x, y, game) }
}

pub(super) fn unreachable(game: &mut Game) {
    unsafe {
        crate::rtl::Unreachable(game);
    }
}
pub(super) fn draw_spritemap(
    offscreen: bool,
    bank: u8,
    map: u16,
    x: u16,
    y: u16,
    tile: u16,
    graphics: u16,
    game: &mut Game,
) {
    unsafe {
        if offscreen {
            crate::game::sm_81::DrawEprojSpritemapWithBaseTileOffscreen(
                bank, map, x, y, tile, graphics, game,
            );
        } else {
            crate::game::sm_81::DrawEprojSpritemapWithBaseTile(
                bank, map, x, y, tile, graphics, game,
            );
        }
    }
}

pub(super) fn respawn_enemy(index: u16, game: &mut Game) {
    actors::population::respawn(
        &mut Runtime::new(game),
        actors::access::ActorKey::from_legacy(index),
    );
}
