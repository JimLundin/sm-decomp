//! The Tick's one contact with the samus-motion model.
//!
//! Her frame is stepped by the model over the engine's world; the cells her moves reached
//! become the engine's touches, and the engine's animation and pose change read the rest.

use clean_model_samus_motion::{Met, Rules, Samus as Model, SpeedCap, Spin};
use clean_types::{Action, Pose};

use super::SamusWorld;
use super::frame::FrameSource;
use super::state::{clean_of, pose_id, write_back};
use crate::collision::{Surroundings, Touches};
use crate::pose::{Content, PoseRequests};
use crate::session;
use crate::state::Controller;
use crate::state::room::ElevatorRide;
use crate::state::samus::{PoseId, Samus};

/// What the model's frame leaves for the engine's animation and pose change.
#[derive(Clone, Copy, Debug)]
pub struct Stepped {
    /// The pose the model chose.
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
    let frame = FrameSource {
        samus,
        controller: inputs.controller,
        ride: inputs.ride,
        completed: inputs.completed,
        requests: inputs.requests,
        content: inputs.content,
    }
    .frame();
    let world = SamusWorld::new(inputs.surroundings.room, inputs.surroundings.enemies);
    let rules = Rules {
        speed_cap: match inputs.speed_cap {
            session::SpeedCap::Wordwise => SpeedCap::Wordwise,
            session::SpeedCap::Exact => SpeedCap::Exact,
        },
        ..Rules::REFERENCE
    };
    let armed = armed(&model, frame, samus, &world, &rules);
    let outcome = model.step(&frame, &world, &rules);
    touches.extend(world.touches(&outcome.reached).touched().iter().copied());
    write_back(&model, samus);
    Some(Stepped {
        pose: model.pose,
        met: outcome.met,
        armed,
    })
}

/// `WallJump`'s test without the kick: spinning carried on with a side held and the spin not
/// yet turned far, whether the kick would fire were it turned far and jump pressed. The model
/// reports only the kick (gaps, item 10), so the test runs on a copy of her.
fn armed(
    model: &Model,
    frame: clean_model_samus_motion::Frame,
    samus: &Samus,
    world: &SamusWorld,
    rules: &Rules,
) -> bool {
    let testing = model.pose.action == Action::Spinning
        && model.spin == Spin::CarriedOn
        && (frame.pad.left || frame.pad.right)
        && !frame.spin_turned_far;
    let Some(mut copy) = clean_of(samus).filter(|_| testing) else {
        return false;
    };
    let mut pressed = frame.pressed;
    pressed.jump = true;
    let kick = clean_model_samus_motion::Frame {
        spin_turned_far: true,
        pressed,
        ..frame
    };
    copy.step(&kick, world, rules).met == Some(Met::WallKick)
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
