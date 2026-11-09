//! The engine's one contact with the samus-motion model.
//!
//! Her frame is stepped by the model over the engine's world; the cells her moves and looks
//! reached become the engine's touches, and the engine's animation and pose change read the
//! rest.

use clean_model_samus_motion::{Outcome, Rules, Samus as Model, SpeedCap, WallTest};
use clean_types::{Action, Parts, Pose};

use super::SamusWorld;
use super::frame::{FrameSource, body};
use super::state::{clean_of, dress, pose_id, write_back};
use crate::collision::{Surroundings, Touches};
use crate::pose::{Content, PoseRequests};
use crate::session;
use crate::state::Controller;
use crate::state::room::ElevatorRide;
use crate::state::samus::{PoseId, Samus};

/// What the model's frame leaves for the engine's animation and pose change.
#[derive(Clone, Copy, Debug)]
pub struct Stepped {
    /// The pose the frame ends in.
    pub pose: Pose,
    /// `WallJump`: a wall is near enough to kick off, but the spin hasn't turned far enough
    /// yet; the animation holds the spin a frame short of the kick.
    pub armed: bool,
}

/// What the frame reads besides Samus and her world.
#[derive(Clone, Copy, Debug)]
pub struct Reads<'a> {
    /// The pad.
    pub controller: &'a Controller,
    /// The elevator under her.
    pub ride: ElevatorRide,
    /// The requested-NMI count, whose parity orders the vertical scans.
    pub completed: u16,
    /// The pad's pose request, which the auto-jump hack reads.
    pub requests: PoseRequests,
    /// The content.
    pub content: &'a Content,
}

/// What the Tick's frame reads besides Samus.
#[derive(Clone, Copy, Debug)]
pub struct Inputs<'a> {
    /// The pad, the elevator, the frame count, the pose request and the content.
    pub reads: Reads<'a>,
    /// Where her moves happen.
    pub surroundings: Surroundings<'a>,
    /// The Session's speed cap.
    pub speed_cap: session::SpeedCap,
}

/// One frame through the model: her before and after it, what it showed, and the two inputs
/// that bring entries into play by themselves.
pub struct Ran {
    /// The model's Samus as the frame starts.
    pub before: Model,
    /// The model's Samus as it ends.
    pub after: Model,
    /// The frame's outputs.
    pub outcome: Outcome,
    /// `AnimationEnds`: her animation ended this frame.
    pub animation_ends: bool,
    /// `RidingTheElevator`: the elevator moved.
    pub elevator_moving: bool,
}

/// Her frame through the model over `world`, written back to the engine's motion; `None`,
/// moving her not at all, when her pose is outside it.
pub fn run(samus: &mut Samus, reads: Reads<'_>, world: &SamusWorld, rules: &Rules) -> Option<Ran> {
    let mut after = clean_of(samus)?;
    let before = after;
    let worn = dress(samus);
    let sized = |pose| body(reads.content, worn, pose);
    let frame = FrameSource {
        samus,
        controller: reads.controller,
        ride: reads.ride,
        completed: reads.completed,
        requests: reads.requests,
        content: reads.content,
    }
    .frame(&sized);
    let outcome = after.step(&frame, world, rules);
    let (animation_ends, elevator_moving) = (frame.animation_ends, frame.elevator_moving);
    write_back(&after, samus);
    Some(Ran {
        before,
        after,
        outcome,
        animation_ends,
        elevator_moving,
    })
}

/// The Tick's frame through the model; `None`, moving her not at all, when her pose is outside
/// it.
pub fn step(samus: &mut Samus, inputs: Inputs<'_>, touches: &mut Touches) -> Option<Stepped> {
    let world = SamusWorld::new(inputs.surroundings.room, inputs.surroundings.enemies);
    let rules = Rules {
        speed_cap: match inputs.speed_cap {
            session::SpeedCap::Wordwise => SpeedCap::Wordwise,
            session::SpeedCap::Exact => SpeedCap::Exact,
        },
        ..Rules::REFERENCE
    };
    let ran = run(samus, inputs.reads, &world, &rules)?;
    touches.extend(
        world
            .touches(&ran.outcome.reached)
            .touched()
            .iter()
            .copied(),
    );
    settle(samus, &ran);
    Some(Stepped {
        pose: ran.after.pose,
        armed: ran.outcome.wall_test == WallTest::Readied,
    })
}

/// The words the pose change leaves that the model doesn't hold.
///
/// Her radius word keeps the pose she had until the next frame sets it, but the shrink gives
/// it the new pose's at once. Where pose fit or the shrink placed her (`Outcome::placed`), the
/// camera's previous y pixel becomes the placement's, as the game writes it at the pose change;
/// on other frames the engine's word is left as it was, since another mover (the elevator) may
/// already have moved her from where the camera last saw her.
pub fn settle(samus: &mut Samus, ran: &Ran) {
    let Ran { before, after, .. } = ran;
    let shrinks = after.pose != before.pose
        && matches!(after.pose.action, Action::Crouching | Action::Morphing);
    if shrinks {
        samus.y_radius = after.radius.y.0;
    }
    if let Some(placed) = ran.outcome.placed {
        samus.previous_position.y.pixels = placed.pixel().0;
    }
}

/// The pose the frame's pose change is asked for, as the engine's pose id.
///
/// `None` when the frame asks for none, so the change records no history. Where the model
/// ran, the pose it chose, asked when the pad asked for one or it differs from hers; elsewhere
/// the pad's request.
#[must_use]
pub fn next(samus: &Samus, stepped: Option<Stepped>, requests: PoseRequests) -> Option<PoseId> {
    let Some(stepped) = stepped else {
        return requests.ordinary;
    };
    let asked = samus.super_special.pose.or(requests.ordinary);
    let id = pose_id(stepped.pose, samus, asked).unwrap_or(samus.pose);
    (requests.ordinary.is_some() || id != samus.pose).then_some(id)
}
