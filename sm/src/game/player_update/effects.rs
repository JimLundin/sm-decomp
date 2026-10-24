//! Finite live observations and retained gamma effects for player update.
#![forbid(unsafe_code)]
use super::content::Content;
use crate::game::player_movement::effects::Host as MovementHost;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GammaAction {
    Inactive,
    Legacy(u16),
}
pub(crate) trait Effects {
    /// Exceptional positions relative to the exploration window. A preceding
    /// map-row write may use a negative displacement; never wrap or clamp it.
    fn imported_exploration_byte(&self, offset: isize) -> u8;
    fn set_imported_exploration_byte(&mut self, offset: isize, value: u8);
    /// Read at the original observation point, including preceding live aliases.
    fn area_map_available(&self) -> bool;
    fn minimap_blink(&self) -> u8;
    fn minimap_boss_id(&self) -> u16;
    /// Select after movement and map writes, not from the beta-entry snapshot.
    fn gamma_action(&self) -> GammaAction;
    /// Publish Session before the selected legacy action; reimport afterward.
    fn run_gamma(&mut self, selector: u16);
}
pub(crate) trait Host: MovementHost + Content + Effects {}
impl<T: MovementHost + Content + Effects> Host for T {}
