//! Qualified legacy effects; these are not counted as unsafe elimination.
use crate::game_state::Game;

pub(super) fn ridley_offscreen(x: u16, y: u16, game: &mut Game) -> u8 {
    u8::from(sm_game::actors::ridley::solid_block_at(x, y, game))
}

pub(super) fn draw_spritemap(
    offscreen: bool,
    bank: u8,
    map: u16,
    x: u16,
    y: u16,
    tile: u16,
    graphics: u16,
    game: &mut impl sm_game::sprite_output::MapHost,
) {
    use sm_game::sprite_drawing::Clip;
    let clip = if offscreen {
        Clip::Outside
    } else {
        Clip::Inside
    };
    sm_game::sprite_output::with_base_tile(bank, map, (x, y), graphics, tile, clip, game);
}
