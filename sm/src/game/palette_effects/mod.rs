//! Palette effects using the shared gameplay Session and runtime.
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
pub(crate) use sm_game::palette_effects::execution;

/// The palette effect entries on a fresh lease of `game`, for the checks that
/// drive them from Game.
pub(crate) fn set_enabled(enabled: bool, game: &mut Game) {
    execution::set_enabled(Runtime::new(game).session_mut(), enabled);
}
pub(crate) fn clear(game: &mut Game) {
    execution::clear(Runtime::new(game).session_mut());
}
pub(crate) fn spawn(definition: u16, game: &mut Game) {
    execution::spawn(&mut Runtime::new(game), definition);
}
pub(crate) fn advance(game: &mut Game) {
    execution::advance(&mut Runtime::new(game));
}

/// The game's fades store their colours in the work RAM image, outside the
/// projection, as they always have.
impl sm_game::palette_effects::fading::Host for Game {
    fn store_faded_colour(&mut self, at: usize, bytes: [u8; 2]) {
        self.ram().copy_in(at, &bytes);
    }
}

/// A lease's fades store their colours in the Session, low byte then high.
impl sm_game::palette_effects::fading::Host for Runtime<'_> {
    fn store_faded_colour(&mut self, at: usize, bytes: [u8; 2]) {
        for (at, byte) in (at..).zip(bytes) {
            self.set_byte(at, byte);
        }
    }
}

/// Numeric identities remain private to the existing actual-code comparison carrier.
pub(crate) mod checks {
    pub(crate) fn guard(game: &mut crate::game_state::Game) -> bool {
        crate::game::visual_runtime::check_palette_guard(game)
    }
    use super::*;
    pub(crate) fn setup(ea: u32, _k: u16, slot: u16, game: &mut Game) {
        execution::setup(&mut Runtime::new(game), ea, slot);
    }
    pub(crate) fn pre(ea: u32, k: u16, game: &mut Game) {
        execution::pre(&mut Runtime::new(game), ea, k);
    }
    pub(crate) fn process_one(k: u16, game: &mut Game) {
        execution::process(&mut Runtime::new(game), k);
    }
    pub(crate) fn command(ea: u32, k: u16, j: u16, game: &mut Game) -> crate::types::PairU16 {
        // Only bank $8D holds palette commands; any other bank decodes as none.
        let handler = if ea >> 16 == 0x8d { ea as u16 } else { 0 };
        let (k, j) = execution::command(&mut Runtime::new(game), handler, k, j);
        crate::types::PairU16 { k, j }
    }
}
