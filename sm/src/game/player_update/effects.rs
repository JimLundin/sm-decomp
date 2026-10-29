//! Finite live observations and retained gamma effects for player update.
#![forbid(unsafe_code)]
use super::content::Content;
use crate::game::player_movement::effects::Host as MovementHost;

pub(crate) trait Effects: sm_game::progress::exploration::ExplorationHost {
    /// Read at the original observation point, including preceding live aliases.
    fn area_map_available(&self) -> bool;
    fn minimap_blink(&self) -> u8;
    fn minimap_boss_id(&self) -> u16;
    /// Publish Session before the selected legacy action; reimport afterward.
    fn run_gamma(&mut self, mode: crate::game::player_transitions::state::FrameGammaMode);
}
pub(crate) trait Host: MovementHost + Content + Effects {}
impl<T: MovementHost + Content + Effects> Host for T {}
