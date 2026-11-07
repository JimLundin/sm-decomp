//! `PadChart`: every situation and pad the chart can meet is decided. Where rows of her
//! situation both fit, the one that loses to the other by rank or by a "beats" clause drops out;
//! the rows no fitting row beats must be at least one, and must all give the same outcome (the
//! same next pose and the same effect). So no two fitting rows are left tied with different
//! outcomes, and no "beats" clause goes round in a circle.

use super::{Situation, outcome, rows, winners};
use clean_types::Action::{
    AgainstWall, Falling, Jumping, Landing, Moonwalking, Rolling, Running, Spinning, Still,
    TakingOff, Turning, WallJumping,
};
use clean_types::{Aim, Body, Equipment, Facing, Gun, LandedFrom, Legs, Pad, Pose, Turn};

/// Every body and action that has rows, each pose part the rows read, both sides of the ground.
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
        (Body::Ball, Still),
        (Body::Ball, Rolling),
        (Body::Ball, Falling),
        (Body::SpringBall, Still),
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

#[test]
fn every_tie_is_decided() {
    let mut failures = Vec::new();
    for (pose, airborne) in situations() {
        if rows(pose).is_empty() {
            continue;
        }
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
                let s = Situation {
                    pose,
                    airborne,
                    pad: h,
                    pressed: pad(pressed),
                    equipment: Equipment::default(),
                };
                let (won, k) = winners(&s);
                let outcomes: Vec<_> = won.iter().map(|r| (outcome(r, pose, k), r.3)).collect();
                if outcomes.is_empty() || outcomes.iter().any(|o| *o != outcomes[0]) {
                    // One line per situation's rows left tied, with a first pad that ties them.
                    let tied: Vec<u32> = won.iter().map(|r| r.0).collect();
                    let line = format!(
                        "{:?} rows needing keys {tied:#x?} tie, e.g. {pose:?} airborne {airborne} held {held:09b} pressed {pressed:07b}: {outcomes:?}",
                        pose.action
                    );
                    if !failures.iter().any(|f: &(Vec<u32>, String)| f.0 == tied) {
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
