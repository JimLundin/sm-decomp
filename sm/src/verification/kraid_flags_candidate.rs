//! Calls actual migrated Kraid entries; never old typed-native odd-area bodies.
use super::Game;
use crate::{kraid_flags_case as case, kraid_flags_observer as observer};
pub unsafe fn capture(id: u32, original: &[u8], game: &mut Game) -> Result<Vec<u8>, String> {
    unsafe {
        let result = super::flags_save_oracle::with_initialized_state(game, |game| {
            super::kraid_flags_oracle::guard(game, original)?;
            let fade = case::seed(id, game)?;
            observer::begin(id);
            if fade {
                sm_game::actors::kraid::fade_in_bg6(game);
            } else {
                observer::observe(0, game);
                let result = sm_game::actors::kraid::is_dead(game);
                observer::observe_result(1, result, game);
            }
            observer::take(if fade { 4 } else { 2 })
        });
        observer::clear();
        result
    }
}
