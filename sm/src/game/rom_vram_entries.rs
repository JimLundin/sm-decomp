//! VRAM write-queue entries copied from cartridge tables through the native image.
#![forbid(unsafe_code)]
use crate::game::session_runtime::native_record as read;
use crate::game_state::Game;
use sm_game::display::transfer::push_tilemap_record;

/// Queue the Mother Brain sprite-tile entry at `at` in `bank` unless its size
/// is 0; the size. The body's cursor through the list is the fight's.
pub(crate) fn queue_sprite_tile_entry(bank: u8, at: u16, game: &mut Game) -> u16 {
    let entry = read(bank, at, game);
    let size = entry.word(0);
    if size != 0 {
        push_tilemap_record(game, &entry);
    }
    size
}
