//! `PadChart`: every situation and pad the chart can meet is decided. Where rows of her
//! situation both fit, the one that loses to the other by rank or by a beats clause drops out;
//! exactly one fitting row must be left, beaten by no other, so the chart never meets a tie and
//! needs no rule for one. No two rows beat each other, and every beats clause names a row of its
//! own situation.

use super::{Moment, Row, beats};
use crate::situation::classify;
use clean_types::Action::{
    AgainstWall, Crouching, Falling, Jumping, Landing, Moonwalking, Morphing, Rolling, Running,
    Spinning, StandingUp, Still, TakingOff, Turning, Unmorphing, WallJumping,
};
use clean_types::{Aim, Body, Equipment, Facing, Gun, LandedFrom, Legs, Pad, Pose, Turn};

/// Every body and action, each pose part the rows read, both sides of the ground.
fn situations() -> Vec<(Pose, bool)> {
    let upright = [
        Still,
        Landing(LandedFrom::Jump),
        Landing(LandedFrom::Spin),
        Running,
        TakingOff,
        Jumping,
        Spinning,
        Falling,
        Turning(Turn::Ground),
        Turning(Turn::MoonwalkJump),
        Turning(Turn::Jump),
        Turning(Turn::Fall),
        Moonwalking,
        WallJumping,
        AgainstWall,
    ]
    .map(|a| (Body::Upright, a));
    let other = [
        (Body::Crouched, Still),
        (Body::Crouched, Turning(Turn::Ground)),
        (Body::Crouched, Crouching),
        (Body::Crouched, Unmorphing),
        (Body::Upright, StandingUp),
        (Body::Ball, Morphing),
        (Body::Ball, Still),
        (Body::Ball, Rolling),
        (Body::Ball, Falling),
        (Body::SpringBall, Still),
        (Body::SpringBall, Rolling),
        (Body::SpringBall, Jumping),
    ];
    let mut all = Vec::new();
    for (body, action) in upright.into_iter().chain(other) {
        for facing in [Facing::Left, Facing::Right] {
            for aim in [
                Aim::Forward,
                Aim::Up,
                Aim::UpForward,
                Aim::DownForward,
                Aim::Down,
            ] {
                for gun in [Gun::Drawn, Gun::Stowed] {
                    for legs in [Legs::Striding, Legs::Together] {
                        for airborne in [false, true] {
                            let pose = Pose {
                                body,
                                action,
                                facing,
                                aim,
                                gun,
                                legs,
                            };
                            all.push((pose, airborne));
                        }
                    }
                }
            }
        }
    }
    all
}

/// The pad from bits, one per button in `Pad`'s order.
fn pad(bits: u32) -> Pad {
    let on = |n: u32| bits >> n & 1 == 1;
    Pad {
        run: on(0),
        jump: on(1),
        shoot: on(2),
        left: on(3),
        right: on(4),
        up: on(5),
        down: on(6),
        aim_up: on(7),
        aim_down: on(8),
    }
}

/// The fitting rows that no other fitting row beats.
fn unbeaten<'a>(fit: &[&'a Row]) -> Vec<&'a Row> {
    fit.iter()
        .copied()
        .filter(|a| !fit.iter().any(|b| beats(b, a)))
        .collect()
}

#[test]
fn beats_clauses_name_rows_of_their_situation_and_go_one_way() {
    for (pose, airborne) in situations() {
        let rows = classify(pose, !airborne).rows;
        for a in rows {
            for name in a.beats {
                assert!(
                    rows.iter().any(|b| b.name == *name),
                    "{:?}: {:?} beats {name:?}, which is not a row of its situation",
                    pose.action,
                    a.name
                );
            }
            for b in rows {
                assert!(
                    !(a.beats.contains(&b.name) && b.beats.contains(&a.name)),
                    "{:?}: {:?} and {:?} beat each other",
                    pose.action,
                    a.name,
                    b.name
                );
            }
        }
    }
}

#[test]
fn every_tie_is_decided() {
    let mut failures = Vec::new();
    for (pose, airborne) in situations() {
        let situation = classify(pose, !airborne);
        for held in 0..512_u32 {
            let h = pad(held);
            // The pad can't hold both of left and right, or of up and down.
            if h.left && h.right || h.up && h.down {
                continue;
            }
            // Just pressed: any of the buttons a row names pressed (down, jump, up, left, right,
            // shoot), among those held.
            let pressable = held & 0b111_1110;
            for pressed in 0..128_u32 {
                if pressed & !pressable != 0 {
                    continue;
                }
                let moment = Moment {
                    situation,
                    pose,
                    airborne,
                    pad: h,
                    pressed: pad(pressed),
                    equipment: Equipment::default(),
                    met: None,
                };
                let fit: Vec<&Row> = situation
                    .rows
                    .iter()
                    .filter(|r| moment.fits(r.needs))
                    .collect();
                let won = unbeaten(&fit);
                if won.len() != 1 {
                    // One line per situation's rows left tied, with a first pad that ties them.
                    let tied: Vec<&str> = won.iter().map(|r| r.name).collect();
                    let line = format!(
                        "{:?} rows {tied:?} are left of {:?}, e.g. {pose:?} airborne {airborne} held {held:09b} pressed {pressed:07b}",
                        pose.action,
                        fit.iter().map(|r| r.name).collect::<Vec<_>>()
                    );
                    if !failures.iter().any(|f: &(Vec<&str>, String)| f.0 == tied) {
                        failures.push((tied, line));
                    }
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} ties left undecided:\n{}",
        failures.len(),
        failures
            .iter()
            .map(|f| f.1.clone())
            .collect::<Vec<_>>()
            .join("\n")
    );
}
