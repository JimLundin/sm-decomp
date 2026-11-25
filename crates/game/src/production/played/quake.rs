//! The room's quake: the shake under way, the starts and stops other owners make, and its update
//! in a frame of play.
//!
//! The room's Ceres doors start shakes door by door as the enemies run (`ceres_doors`): each
//! reads the frames left as the doors before it left them, and a shake it starts replaces the
//! one under way before the next door runs. Building the next room in a passage stops the shake (room
//! loading's effects set-up, inside the building), and so does Ridley's placing. A frame of play
//! updates the quake last, after her moves, the camera and the HUD, with time running (a seed
//! that holds time is refused); a passage has no update. The offsets and the actors' request it
//! publishes have no consumer in this loop, which draws neither the layers nor the actors, and
//! Drawing's query for the actors' projectiles is not asked.

use super::Fault;
use canonical::model::Pinned;
use world_quake::{Effects, Opportunity, Outcome, ShakeOrder, State, Time};

/// The room quake's complete State, carried between whole frames.
pub struct QuakePin;
impl Pinned for QuakePin {
    type State = State;
    const NAME: &'static str = "production-frame-quake";
    const SHAPE: u64 = 0x6086_4A64_538D_FAF9;
}

/// The shake under way ended: the next room's building, or Ridley's placing.
pub(super) const fn stop(quake: &mut State) {
    world_quake::start_or_stop(quake, ShakeOrder::Stop);
}

/// A frame of play's update: the offsets and the actors' request, from the shake as it begins.
///
/// # Errors
/// [`Fault::Quake`] if the quake refuses the frame's ordinary opportunity.
pub(super) fn update(quake: &mut State) -> Result<Effects, Fault> {
    match world_quake::step(quake, Opportunity::Ordinary, Time::Running) {
        Outcome::Updated(effects) => Ok(effects),
        Outcome::NoOpportunity | Outcome::Unsupported => Err(Fault::Quake),
    }
}

#[cfg(test)]
mod tests {
    use super::{QuakePin, stop, update};
    use core::num::NonZeroU16;
    use world_quake::{QuakeDirection, QuakeKind, QuakePhase, QuakeReach, QuakeSize, Shake, State};

    #[test]
    fn quake_shape_remains_pinned() {
        assert_eq!(canonical::model::pin_holds::<QuakePin>(), Ok(()));
    }

    #[test]
    fn a_frame_of_plays_update_moves_what_the_shake_reaches_and_a_stop_ends_it() {
        let mut quake = State {
            phase: QuakePhase(7),
            shake: Some(Shake {
                kind: QuakeKind {
                    reach: QuakeReach::Back,
                    size: QuakeSize::new(3).unwrap(),
                    direction: QuakeDirection::Diagonal,
                },
                remaining: NonZeroU16::new(4).unwrap(),
            }),
        };
        // 4 mod 4 is 0: the back layer moves +3 on both axes, the front stays, the actors shake.
        let effects = update(&mut quake).unwrap();
        assert_eq!((effects.back.x, effects.back.y), (3, 3));
        assert_eq!((effects.front.x, effects.front.y), (0, 0));
        assert!(effects.actors_asked);
        assert_eq!(quake.remaining().0, 3);
        assert_eq!(quake.phase, QuakePhase(8));
        stop(&mut quake);
        assert_eq!(quake.shake, None);
        assert_eq!(quake.phase, QuakePhase(8));
    }
}
