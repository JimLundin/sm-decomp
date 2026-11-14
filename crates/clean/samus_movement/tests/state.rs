//! Caller-held State saves complete values, not a projection of visible movement.

#![cfg(test)]

use core::num::Wrapping;
use samus_movement::{
    Bounces, Carry, Dashing, Direction, DirectionalTravels, FrameBefore, JumpHeld, Phase, Run,
    Samus, Seat, Spin, TravelDistance, TravelKind, TravelledDistances, Vertical,
};
use serde::{Deserialize, Serialize};
use types::{Action, Aim, Body, Facing, Gun, LandedFrom, Legs, Position, Side, Turn};

/// A caller saves movement and the preceding interval available to firing together.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct SavedMovement {
    samus: Samus,
    distances: TravelledDistances,
}

/// Every remembered field differs from an empty start; all four travels retain fractions.
fn saved_movement() -> SavedMovement {
    SavedMovement {
        samus: Samus {
            position: Position {
                x: Wrapping(0x0106_8000),
                y: Wrapping(0x0100_C000),
            },
            run: Run {
                speed: Wrapping(0x0002_6000),
                way: Side::Left,
                carry: Carry::CarriesOn,
                dash: Wrapping(0x0001_7000),
                dashing: Dashing::On,
            },
            vertical: Vertical {
                speed: Wrapping(0xFFFF_C000),
                phase: Phase::Rising,
                bounces: Bounces::Twice,
            },
            pose: types::Pose {
                body: Body::SpringBall,
                action: Action::Falling,
                facing: Facing::Left,
                aim: Aim::Forward,
                gun: Gun::Stowed,
                legs: Legs::Together,
            },
            spin: Spin::CarriedOn,
            seat: Seat::Seated,
            jump_held: JumpHeld {
                frames: Wrapping(32777),
                before: FrameBefore::LandedOrEnded,
            },
        },
        distances: TravelledDistances {
            travels: DirectionalTravels::from_fn(|direction| {
                TravelDistance(Wrapping(match direction {
                    Direction::Down => 0x0001_2000,
                    Direction::Left => 0xFFFF_C000,
                    Direction::Right => 0x0003_E000,
                    Direction::Up => 0xFFFD_8000,
                }))
            }),
        },
    }
}

/// Spec: `TravelledDistances`, `Jumping`, `Dashing`. Save, deserialize, and restore every field.
#[test]
fn complete_state_round_trip_and_retention() {
    let saved = saved_movement();
    let encoded = toml::to_string(&saved).expect("complete state serializes as TOML");
    let mut restored: SavedMovement =
        toml::from_str(&encoded).expect("complete state deserializes as TOML");
    assert_eq!(restored, saved, "no remembered value may be dropped");
    restored
        .distances
        .record(TravelKind::Look, Direction::Up, TravelDistance::ZERO);
    assert_eq!(
        restored.distances, saved.distances,
        "a look preserves the interval"
    );
    restored
        .distances
        .record(TravelKind::Walk, Direction::Right, TravelDistance::ZERO);
    for direction in [Direction::Left, Direction::Up, Direction::Down] {
        assert_eq!(
            restored.distances.travels[direction],
            saved.distances.travels[direction]
        );
    }
    assert_eq!(
        restored.distances.travels[Direction::Right],
        TravelDistance::ZERO
    );
    assert_eq!(restored.samus, saved.samus);
}

/// Spec: `TravelledDistances`. Default and fresh both start the complete map at zero.
#[test]
fn default_is_exactly_fresh() {
    let default = TravelledDistances::default();
    assert_eq!(default, TravelledDistances::fresh());
    for (_, travel) in default.travels {
        assert_eq!(travel, TravelDistance::ZERO);
    }
}

/// Pose payloads and nonempty gun/legs values are part of State as well.
#[test]
fn pose_payloads_round_trip() {
    let upright = types::Pose {
        body: Body::Upright,
        action: Action::Jumping,
        facing: Facing::Right,
        aim: Aim::Forward,
        gun: Gun::Stowed,
        legs: Legs::Together,
    };
    for pose in [
        types::Pose {
            action: Action::Turning(Turn::MoonwalkJump),
            aim: Aim::UpForward,
            ..upright
        },
        types::Pose {
            action: Action::Landing(LandedFrom::Spin),
            ..upright
        },
        types::Pose {
            gun: Gun::Drawn,
            ..upright
        },
        types::Pose {
            legs: Legs::Striding,
            ..upright
        },
    ] {
        let mut saved = saved_movement();
        saved.samus.pose = pose;
        let encoded = toml::to_string(&saved).expect("pose payload serializes");
        let restored: SavedMovement = toml::from_str(&encoded).expect("pose payload deserializes");
        assert_eq!(restored, saved);
    }
}
