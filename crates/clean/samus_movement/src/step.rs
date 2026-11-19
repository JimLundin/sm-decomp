//! Spec: `SuspendedMovement`. Ordinary updating drains the same owned continuation,
//! without pauses or a separately reconstructed decision path.
use crate::suspended::SuspendedMovement;
use crate::{Frame, Outcome, Rules, Samus, UnknownPose};
use types::World;
impl Samus {
    /// Spec: `PoseFollows`, `SuspendedMovement`. Complete one ordinary update.
    /// # Errors
    /// An unknown starting pose leaves State untouched.
    pub fn step(
        &mut self,
        frame: &Frame,
        world: &dyn World,
        rules: &Rules,
    ) -> Result<Outcome, UnknownPose> {
        let mut work = SuspendedMovement::new(*self)?;
        while !work.is_complete() {
            work.offer(frame, world, rules)?;
            work.complete(frame, world, rules)?;
        }
        *self = work.state;
        Ok(work.outputs)
    }
}
