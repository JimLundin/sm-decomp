//! The shaft's late turn, after motion and after the actors have seen the incoming view.

use types::ceres_shaft::{ShaftRotation, ShaftView, TiltDirection};
use types::{Facing, Pair, Px, Side, VerticalSpeed, ZERO};

use crate::shaft_overlay::StartShaftRotation;

const REACH_LEFT: u16 = 112;
const REACH_RIGHT: u16 = 144;
const REACH_TOP: u16 = 75;
const REACH_BOTTOM: u16 = 128;
const ELEVATOR_HOLD: u8 = 60;

/// Spec: `ShaftTilts`. One entry in the room's ordered tilt track.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TiltSample {
    /// Turns before the next sample may appear.
    pub hold: u16,
    /// Signed 8.8 diagonal coefficient.
    pub diagonal: i16,
    /// Signed 8.8 cross coefficient.
    pub cross: i16,
}

impl TiltSample {
    const fn matrix(self) -> [i16; 4] {
        [
            self.diagonal,
            self.cross,
            self.cross.wrapping_neg(),
            self.diagonal,
        ]
    }
}

/// Spec: `ShaftTilts`. An ordered track with at least two samples.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TiltTrack<'a>(&'a [TiltSample]);

impl<'a> TiltTrack<'a> {
    /// Use the room's samples if the track has distinct first and last entries.
    #[must_use]
    pub const fn new(samples: &'a [TiltSample]) -> Option<Self> {
        match samples {
            [_, _, ..] => Some(Self(samples)),
            _ => None,
        }
    }

    const fn advance(self, rotation: &mut ShaftRotation) {
        let last = self.0.len().saturating_sub(1);
        let direction = inward_at_end(rotation.next, last, rotation.direction);
        let next = match direction {
            TiltDirection::Forward => rotation.next.saturating_add(1),
            TiltDirection::Backward => rotation.next.saturating_sub(1),
        };
        rotation.next = next;
        rotation.direction = inward_at_end(next, last, direction);
    }
}

const fn inward_at_end(index: usize, last: usize, direction: TiltDirection) -> TiltDirection {
    if index == 0 {
        TiltDirection::Forward
    } else if index == last {
        TiltDirection::Backward
    } else {
        direction
    }
}

/// Spec: `ShaftTilts`. Spend the setup hold even on the enabling turn. Each sample supplies
/// the new matrix and hold; the next sample points inward at either end of the track.
///
/// Call only on a shaft turn, after the actors. The pivot never changes here.
pub fn turn_shaft_rotation(
    rotation: &mut ShaftRotation,
    start: Option<StartShaftRotation>,
    track: TiltTrack<'_>,
) -> ShaftView {
    rotation.active |= start.is_some();
    if rotation.active {
        if rotation.hold > 0 {
            rotation.hold = rotation.hold.saturating_sub(1);
        } else if let Some(sample) = track.0.get(rotation.next) {
            rotation.view.matrix = sample.matrix();
            rotation.hold = sample.hold;
            track.advance(rotation);
        }
    }
    rotation.view
}

/// Spec: `ElevatorReached`. Which exact earlier pose appearance uses for transitions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EarlierShaftPose {
    /// Exactly the destination standing pose for this side.
    Standing(Side),
    /// Any pose other than the two destination poses.
    Other,
}

/// Spec: `ElevatorReached`. Samus's placement after motion, with the earlier appearance pose.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShaftPlacement {
    /// Centre in whole room pixels after movement.
    pub centre: Pair<Px>,
    /// Full vertical speed after movement, including its fraction.
    pub vertical_speed: VerticalSpeed,
    /// Facing after movement.
    pub facing: Facing,
    /// Exact earlier pose, not the pose left by movement.
    pub earlier_pose: EarlierShaftPose,
}

impl ShaftPlacement {
    fn at_elevator(self) -> bool {
        let Pair { x, y } = self.centre;
        x.0 > REACH_LEFT
            && x.0 <= REACH_RIGHT
            && (REACH_TOP..REACH_BOTTOM).contains(&y.0)
            && self.vertical_speed.0 == ZERO
    }
}

/// Spec: `ElevatorReached`. How appearance should handle the chosen standing clip.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ElevatorAnimation {
    /// Keep the current animation frame and its remaining count.
    Preserve,
    /// Begin the chosen standing clip.
    StartStanding,
}

/// Spec: `ElevatorReached`. Start the scene, cancel pending pose changes and lock Samus
/// in the selected standing pose without moving her.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ElevatorLock {
    /// Side of the destination standing pose.
    pub standing: Side,
    /// Animation intent for Samus appearance.
    pub animation: ElevatorAnimation,
    /// Initial scene hold in turns.
    pub hold: u8,
    /// Both starting fade values for the scene.
    pub scene_fade: [u8; 2],
}

/// Spec: `ElevatorReached`. Only ordinary play can request the lock. The scene owns the
/// transition out of ordinary play; its hold and fade still let the shaft turn.
#[must_use]
pub fn elevator_reached(
    active: bool,
    ordinary_play: Option<ShaftPlacement>,
) -> Option<ElevatorLock> {
    let her = ordinary_play.filter(|her| active && her.at_elevator())?;
    let standing = Side::of(her.facing);
    let animation = if her.earlier_pose == EarlierShaftPose::Standing(standing) {
        ElevatorAnimation::Preserve
    } else {
        ElevatorAnimation::StartStanding
    };
    Some(ElevatorLock {
        standing,
        animation,
        hold: ELEVATOR_HOLD,
        scene_fade: [0, 0],
    })
}

/// The view for drawing and an optional arrival for the elevator scene.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShaftTurn {
    /// Current view, including a sample installed on this turn.
    pub view: ShaftView,
    /// A new scene start and movement lock, if Samus has reached the elevator.
    pub elevator: Option<ElevatorLock>,
}

/// Spec: `ShaftTilts`, `ElevatorReached`. Take the late shaft turn. Supply placement only
/// during ordinary play. During the elevator scene the view still updates, without a new lock.
pub fn turn_shaft(
    rotation: &mut ShaftRotation,
    start: Option<StartShaftRotation>,
    track: TiltTrack<'_>,
    ordinary_play: Option<ShaftPlacement>,
) -> ShaftTurn {
    let view = turn_shaft_rotation(rotation, start, track);
    ShaftTurn {
        view,
        elevator: elevator_reached(rotation.active, ordinary_play),
    }
}
