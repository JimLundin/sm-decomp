//! The Tick's one contact with the samus-motion model.
//!
//! Her frame is stepped by the model over the engine's world; the cells her moves and looks
//! reached become the engine's touches, and the engine's animation and pose change read the
//! rest.

use clean_model_samus_motion::{Change, Met, Rules, Samus as Model, SpeedCap, WallTest};
use clean_types::{Action, Aim, Body, Fixed, Pose, Radius};

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
    /// What her moves met.
    pub met: Option<Met>,
    /// `WallJump`: a wall is near enough to kick off, but the spin hasn't turned far enough
    /// yet; the animation holds the spin a frame short of the kick.
    pub armed: bool,
}

/// What the frame reads besides Samus.
#[derive(Clone, Copy, Debug)]
pub struct Inputs<'a> {
    /// The pad.
    pub controller: &'a Controller,
    /// The elevator under her.
    pub ride: ElevatorRide,
    /// The requested-NMI count, whose parity orders the vertical scans.
    pub completed: u16,
    /// The pad's pose request, which the auto-jump hack reads.
    pub requests: PoseRequests,
    /// Where her moves happen.
    pub surroundings: Surroundings<'a>,
    /// The Session's speed cap.
    pub speed_cap: session::SpeedCap,
    /// The content.
    pub content: &'a Content,
}

/// Her frame through the model; `None`, moving her not at all, when her pose is outside it.
pub fn step(samus: &mut Samus, inputs: Inputs<'_>, touches: &mut Touches) -> Option<Stepped> {
    let mut model = clean_of(samus)?;
    let start = (model.pose, model.radius);
    let worn = dress(samus);
    let body = |pose| body(inputs.content, worn, pose);
    let frame = FrameSource {
        samus,
        controller: inputs.controller,
        ride: inputs.ride,
        completed: inputs.completed,
        requests: inputs.requests,
        content: inputs.content,
    }
    .frame(&body);
    let world = SamusWorld::new(inputs.surroundings.room, inputs.surroundings.enemies);
    let rules = Rules {
        speed_cap: match inputs.speed_cap {
            session::SpeedCap::Wordwise => SpeedCap::Wordwise,
            session::SpeedCap::Exact => SpeedCap::Exact,
        },
        ..Rules::REFERENCE
    };
    let outcome = model.step(&frame, &world, &rules);
    touches.extend(world.touches(&outcome.reached).touched().iter().copied());
    write_back(&model, samus);
    settle(samus, start, &model, outcome.change, rules.crouch_lift);
    Some(Stepped {
        pose: model.pose,
        met: outcome.met,
        armed: outcome.wall_test == WallTest::Readied,
    })
}

/// The words the pose change leaves that the model doesn't hold.
///
/// Her radius word keeps the pose she had until the next frame sets it, but the shrink gives
/// it the new pose's at once. Where pose fit pushes her, the crouch taken instead raises her
/// or the shrink lowers her, the camera's previous y pixel becomes hers as that left her
/// (before a crouch take-off's lift), so the camera doesn't follow the move.
fn settle(
    samus: &mut Samus,
    (pose, radius): (Pose, Radius),
    model: &Model,
    change: Change,
    crouch_lift: Fixed,
) {
    let changed = model.pose != pose;
    let shrinks = changed && matches!(model.pose.action, Action::Crouching | Action::Morphing);
    let lifted = changed
        && pose.body == Body::Crouched
        && pose.aim == Aim::Forward
        && model.pose.action == Action::TakingOff;
    let placed = match change {
        Change::Chosen(Some(_)) => true,
        Change::Chosen(None) => shrinks,
        Change::Crouched => radius.y < model.radius.y,
        Change::Refused => false,
    };
    if change == Change::Chosen(None) && shrinks {
        samus.y_radius = model.radius.y;
    }
    if placed {
        let lift = if lifted && matches!(change, Change::Chosen(_)) {
            crouch_lift
        } else {
            Fixed(0)
        };
        samus.previous_position.y.pixels = model.position.y.plus(lift).pixel();
    }
}

/// The pose the frame's pose change is asked for, as the engine's pose id.
///
/// `None` when the frame asks for none, so the change records no history. Where the model
/// ran, the pose it chose, asked when the pad asked for one, her moves met something or it
/// differs from hers; elsewhere the pad's request.
#[must_use]
pub fn next(samus: &Samus, stepped: Option<Stepped>, requests: PoseRequests) -> Option<PoseId> {
    let Some(stepped) = stepped else {
        return requests.ordinary;
    };
    let asked = samus.super_special.pose.or(requests.ordinary);
    let id = pose_id(stepped.pose, samus, asked).unwrap_or(samus.pose);
    (requests.ordinary.is_some() || stepped.met.is_some() || id != samus.pose).then_some(id)
}
