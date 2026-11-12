//! The engine's one contact with the samus-movement model.
//!
//! Her frame is stepped by the model over the engine's world; the cells her moves and looks
//! reached become the engine's touches, and the pose change and her animation
//! (`samus_appearance_bridge`) read the rest.
//!
//! The model answers `UnknownPose` for a pose no situation names. Every pose the pose map holds
//! is one a situation names (`state::named`, pinned by its test), so the frame refuses it as a
//! fault that can't happen, rather than invent a motion for it.

use samus_movement::{Outcome, Rules, Samus as Model, SpeedCap};
use types::{Action, ClipEnd, Parts, Pose, Radius};

use super::frame::{FrameSource, body};
use super::state::{clean_of, pose_id, write_back};
use super::{SamusFault, SamusWorld};
use crate::collision::{Surroundings, Touches};
use crate::pose::{Asked, Content, PoseRequests};
use crate::session;
use crate::state::Controller;
use crate::state::room::ElevatorRide;
use crate::state::samus::Samus;

/// What the model's frame leaves for the pose change and her animation.
#[derive(Clone, Debug)]
pub struct Stepped {
    /// The pose the frame ends in.
    pub pose: Pose,
    /// The frame's outputs, which her animation reads.
    pub outcome: Outcome,
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
    /// The content.
    pub content: &'a Content,
}

/// What the Tick's frame reads besides Samus.
#[derive(Clone, Copy, Debug)]
pub struct Inputs<'a> {
    /// The pad, the elevator, the frame count and the content.
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
    /// §1.3 body size: the body of the pose she starts the frame in, the one her moves test: her
    /// radius word, which is the pose table's except on the elevator, whose ride keeps the pose
    /// before's.
    pub body_before: Radius,
    /// The pose table's body for the pose she ends the frame in.
    pub body_after: Radius,
    /// `AnimationEnds`: the end her animation reached this frame, if any.
    pub animation_ends: Option<ClipEnd>,
    /// `RidingTheElevator`: the elevator moved.
    pub elevator_moving: bool,
}

/// Her frame through the model over `world`, written back to the engine's motion; `None`,
/// moving her not at all, when her pose is outside it.
///
/// # Errors
///
/// A [`SamusFault`]: a pose no situation names, or a stand-in chain her moves met that loops.
pub fn run(
    samus: &mut Samus,
    reads: Reads<'_>,
    world: &SamusWorld,
    rules: &Rules,
) -> Result<Option<Ran>, SamusFault> {
    let Some(mut after) = clean_of(samus) else {
        return Ok(None);
    };
    let before = after;
    let sized = |pose| body(reads.content, samus, pose);
    let frame = FrameSource {
        samus,
        controller: reads.controller,
        ride: reads.ride,
        completed: reads.completed,
        content: reads.content,
    }
    .frame(&sized);
    let outcome = after
        .step(&frame, world, rules)
        .map_err(|_| SamusFault::UnnamedPose(samus.pose))?;
    world.refused()?;
    let (animation_ends, elevator_moving) = (frame.animation_ends, frame.elevator_moving);
    let (body_before, body_after) = (sized(before.pose), sized(after.pose));
    write_back(&after, outcome.rising, samus);
    Ok(Some(Ran {
        before,
        after,
        outcome,
        body_before,
        body_after,
        animation_ends,
        elevator_moving,
    }))
}

/// The Tick's frame through the model; `None`, moving her not at all, when her pose is outside
/// it.
///
/// # Errors
///
/// A [`SamusFault`] the frame met.
pub fn step(
    samus: &mut Samus,
    inputs: Inputs<'_>,
    touches: &mut Touches,
) -> Result<Option<Stepped>, SamusFault> {
    let world = SamusWorld::new(inputs.surroundings.room, inputs.surroundings.enemies);
    let rules = Rules {
        speed_cap: match inputs.speed_cap {
            session::SpeedCap::Wordwise => SpeedCap::Wordwise,
            session::SpeedCap::Exact => SpeedCap::Exact,
        },
        ..Rules::REFERENCE
    };
    let Some(ran) = run(samus, inputs.reads, &world, &rules)? else {
        return Ok(None);
    };
    touches.extend(
        world
            .touches(&ran.outcome.reached)
            .touched()
            .iter()
            .copied(),
    );
    settle(samus, &ran);
    Ok(Some(Stepped {
        pose: ran.after.pose,
        outcome: ran.outcome,
    }))
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
        samus.y_radius = ran.body_after.y.0;
    }
    if let Some(placed) = ran.outcome.placed {
        samus.previous_position.y.pixels = placed.pixel().0;
    }
}

/// The pose the frame's pose change is asked for, as the engine's pose id.
///
/// `None` when the frame asks for none, so the change records no history. Where the model
/// ran, the pose it chose, in the sprite her dress shows: on a hand-over always, as the pose
/// handed over (her animation's end, `$90:84B6`); otherwise asked when the pad asked for one or
/// it differs from hers. Elsewhere the pad's request.
#[must_use]
pub fn next(samus: &Samus, stepped: Option<&Stepped>, requests: PoseRequests) -> Option<Asked> {
    let Some(stepped) = stepped else {
        return requests.ordinary.map(Asked::Pose);
    };
    if stepped.outcome.handed_over {
        let id = pose_id(stepped.pose, samus, None).unwrap_or(samus.pose);
        return Some(Asked::HandedOver(id));
    }
    let id = pose_id(stepped.pose, samus, requests.ordinary).unwrap_or(samus.pose);
    (requests.ordinary.is_some() || id != samus.pose).then_some(Asked::Pose(id))
}
