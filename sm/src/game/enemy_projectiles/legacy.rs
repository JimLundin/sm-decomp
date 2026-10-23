//! Qualified legacy effects; these are not counted as unsafe elimination.
use crate::game_state::Game;

pub(super) fn next_random(game: &mut Game) -> u16 {
    unsafe { crate::game::sm_80::NextRandom(game) }
}

pub(super) fn call_player_code(code: u16, game: &mut Game) -> u16 {
    unsafe { crate::game::sm_90::CallSomeSamusCode(code, game) }
}

pub(super) fn calculate_angle_from_xy(x: u16, y: u16, game: &mut Game) -> u16 {
    unsafe { crate::game::sm_a0::CalculateAngleFromXY(x, y, game) }
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
    unsafe {
        crate::game::enemy_population_ram::respawn(index, game);
    }
}
