//! Actual repaired SaveToSram entry; never substitute a serialization model.
use super::Game;
use crate::{flags_save_case as case, flags_save_observer as observer};
pub unsafe fn capture(id: u32, original: &[u8], game: &mut Game) -> Result<Vec<u8>, String> {
    unsafe {
        let result = super::flags_save_oracle::with_initialized_state(game, |game| {
            super::flags_save_oracle::guard(game, original)?;
            let slot = case::seed(id, game)?;
            observer::begin(id);
            crate::game::save_slots::save(slot, game);
            observer::take()
        });
        observer::clear();
        result
    }
}
