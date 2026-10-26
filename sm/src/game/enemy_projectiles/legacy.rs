//! Qualified legacy effects; these are not counted as unsafe elimination.
use crate::game::{actors, session_runtime::Runtime};
use crate::game_state::Game;

pub(super) fn call_player_code(code: u16, game: &mut Game) -> u16 {
    crate::game::session_runtime::player_command(code, game)
}

pub(super) fn calculate_angle_from_xy(x: u16, y: u16, _game: &mut Game) -> u16 {
    crate::game::actors::api::angle_from_delta(x, y)
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
    use crate::game::sprite_drawing::Clip;
    let clip = if offscreen {
        Clip::Outside
    } else {
        Clip::Inside
    };
    crate::game::projectile_presentation_ram::with_base_tile(
        bank,
        map,
        (x, y),
        graphics,
        tile,
        clip,
        game,
    );
}

pub(super) fn respawn_enemy(index: u16, game: &mut Game) {
    actors::population::respawn(
        &mut Runtime::new(game),
        actors::access::ActorKey::from_legacy(index),
    );
}
