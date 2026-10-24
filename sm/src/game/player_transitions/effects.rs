//! Declaration-only finite effects for the proposed player transition owner.
#![forbid(unsafe_code)]
use super::content::Content;
use crate::game::{
    player_collision::effects::Host as CollisionHost, visual_host::Host as VisualHost,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GrappleTransition {
    SwingPose,    // existing BD95 animation and position update
    AnchoredPose, // existing BEEB anchor-relative coordinate update
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BounceFault {
    BlockMorph,
    BlockSpring,
    LandingMorph,
    LandingSpring,
}
/// Sound is inherited from visual_host::Effects::sound(channel, limit, cue).
/// Allowed source pairs: (1,6), (1,9), (2,6), (3,3), (3,6), (3,9).
/// Do not add callbacks for collision, palette, charge, jump or landing algorithms.
pub(crate) trait Effects {
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
pub(crate) trait Host: CollisionHost + VisualHost + Content + Effects {}
