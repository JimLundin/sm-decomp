//! Declaration-only finite effects for the proposed player transition owner.
use super::content::Content;
use crate::{lease::Lease, runtime_access::SessionView};
use crate::{player_collision::effects::Host as CollisionHost, visual_host::Host as VisualHost};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GrappleTransition {
    SwingPose,    // existing BD95 animation and position update
    AnchoredPose, // existing BEEB anchor-relative coordinate update
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BounceFault {
    BlockMorph,
    BlockSpring,
    LandingMorph,
    LandingSpring,
}
/// Sound is inherited from visual_host::Effects::sound(channel, limit, cue).
/// Sound limits remain specific to the calling gameplay command.
/// Already-owned collision, palette, charge, jump and landing algorithms compose directly.
pub trait Effects {
    /// Original script-dispatch diagnostic; preserve its defined continuation value.
    fn invalid_demo_instruction(&mut self) -> u16;
    /// Actual synchronous NextRandom; reacquire all subsequent live state.
    fn random_word(&mut self) -> u16;
    /// The two retained grapple operations, with true Runtime authority handoff.
    fn grapple_transition(&mut self, transition: GrappleTransition);
    /// Existing Unreachable diagnostic at four bounce paths; no conversion to
    /// Result/rollback. Preserve native assertion behavior and any defined return
    /// path. Invalid array selectors remain ordinary checked-index failures.
    fn invalid_bounce(&mut self, site: BounceFault, value: u16);
}
/// Existing owners/algorithms, no copied Session and no generic legacy callback.
/// Use qualified Content/Effects methods where inherited names are ambiguous.
pub trait Host: CollisionHost + VisualHost + Content + Effects {}
/// A step of the X-ray background restore, run on the game.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum XrayRestore {
    Left,
    Right,
    Finish,
}
/// Game-side calls the Samus rules make outside the lease, as the original's.
pub trait GameEffects {
    fn pose_command(&mut self, command: u16);
    /// Spawn a bank $91 HDMA object.
    fn spawn_hdma_object(&mut self, definition: &crate::display::hdma_spawn::HdmaObjectDefinition);
    /// Rebuild the beam tiles and palette for the equipped beams.
    fn update_beam_graphics(&mut self);
    fn draw_xray_window_table(&mut self, window: crate::beam_windows::XrayWindow);
    fn restore_xray_background(&mut self, step: XrayRestore);
    /// Spawn the scope's HDMA object from its $91:CAF2 definition.
    fn spawn_xray_scope_hdma(&mut self);
}
/// A Session lease the Samus rules run on.
pub trait SamusLease: SessionView + Lease + GameEffects {}
impl<T: SessionView + Lease + GameEffects> SamusLease for T {}
