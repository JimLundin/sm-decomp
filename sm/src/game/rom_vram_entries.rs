//! Boss transfers run on the admitted Runtime's typed actor/queue owners.
#![forbid(unsafe_code)]
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
use sm_game::actors::boss_vram_entries_owned as owned;
pub(crate) fn queue_mother_brain_sprite_tiles(bank: u8, start: u16, game: &mut Game) -> u8 {
    Runtime::new(game).owned_call(|host| owned::queue_mother_brain_sprite_tiles(bank, start, host))
}
