//! Mother Brain's visuals run in `sm_game::mother_brain_visuals`; Game supplies
//! the imported tables and keeps the earthquake start its hosts call.
use crate::game::music::queue_sfx2_max6;
use crate::game::prelude::*;
pub(crate) use sm_game::mother_brain_visuals::*;

impl PaletteHost for Game {
    fn mother_brain_visuals(&self) -> &ImportedTables {
        &self.mother_brain_visuals
    }
}

#[forbid(unsafe_code)]
pub fn start_earthquake(a: u16, game: &mut Game) {
    {
        word! {game; EARTHQUAKE_TYPE = a; EARTHQUAKE_TIMER = 20; }
        queue_sfx2_max6(0x25, game);
    }
}
