//! `PadChart`: every situation and pad the chart can meet is decided. Where rows of her
//! situation both fit, the one that loses to the other by rank or by a beats clause drops out;
//! at most one fitting row may be left, beaten by no other (with none, the last row decides), so
//! the chart never meets a tie and needs no rule for one. No two fitting rows beat each other,
//! and every beats clause, with a condition or not, names a row of its own situation.

use super::Moment;
use crate::rows::Row;
use crate::situation::classify;
use clean_types::Action::{
    AgainstWall, Crouching, Falling, Jumping, Landing, Moonwalking, Morphing, Rolling, Running,
    Spinning, StandingUp, Still, TakingOff, Turning, Unmorphing, WallJumping,
};
use clean_types::{Aim, Body, Buttons, Equipment, Facing, Gun, LandedFrom, Legs, Pose, Turn};

/// Every aim.
const AIMS: [Aim; 5] = [
    Aim::Forward,
    Aim::Up,
    Aim::UpForward,
    Aim::DownForward,
    Aim::Down,
];

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
        (Body::SpringBall, Falling),
        (Body::SpringBall, Jumping),
    ];
    upright
        .into_iter()
        .chain(other)
        .flat_map(|(body, action)| {
            [Facing::Left, Facing::Right].map(|facing| Pose {
                body,
                action,
                facing,
                aim: Aim::Forward,
                gun: Gun::Stowed,
                legs: Legs::Together,
            })
        })
        .flat_map(|p| AIMS.map(|aim| Pose { aim, ..p }))
        .flat_map(|p| [Gun::Drawn, Gun::Stowed].map(|gun| Pose { gun, ..p }))
        .flat_map(|p| [Legs::Striding, Legs::Together].map(|legs| Pose { legs, ..p }))
        .flat_map(|p| [false, true].map(|airborne| (p, airborne)))
        .collect()
}

/// Every pad the chart can meet: the buttons held, with at most one of up and down (§7: no
/// record holds both; left and right may be held together), and those just pressed among the
/// ones a row names pressed (down, jump, up, left, right, shoot) that are held.
fn pads() -> Vec<(u16, u16)> {
    let (up_down, pressable) = (0b110_0000, 0b111_1110);
    (0..512_u16)
        .filter(|held| held & up_down != up_down)
        .flat_map(|held| {
            (0..128_u16)
                .filter(move |pressed| pressed & !(held & pressable) == 0)
                .map(move |pressed| (held, pressed))
        })
        .collect()
}

/// The fitting rows that no other fitting row beats.
fn unbeaten<'a>(moment: &Moment, fit: &[&'a Row]) -> Vec<&'a Row> {
    fit.iter()
        .copied()
        .filter(|a| !fit.iter().any(|b| moment.beats(b, a)))
        .collect()
}

/// A beats clause that names no row of its situation, or two rows that beat each other.
fn misnamed(rows: &[Row], a: &Row) -> Option<String> {
    let conditional = a.beats_if.iter().map(|c| &c.row);
    let unknown = a
        .beats
        .iter()
        .chain(conditional)
        .find(|n| !rows.iter().any(|b| b.name == **n));
    let mutual = rows
        .iter()
        .find(|b| a.beats.contains(&b.name) && b.beats.contains(&a.name));
    unknown
        .map(|n| format!("{:?} beats {n:?}, not a row of its situation", a.name))
        .or_else(|| mutual.map(|b| format!("{:?} and {:?} beat each other", a.name, b.name)))
}

#[test]
fn beats_clauses_name_rows_of_their_situation_and_go_one_way() {
    let problems: Vec<String> = situations()
        .into_iter()
        .flat_map(|(pose, airborne)| {
            let rows = classify(pose, !airborne).rows;
            rows.iter().filter_map(|a| misnamed(rows, a))
        })
        .collect();
    assert!(problems.is_empty(), "{problems:?}");
}

/// The rows left tied for a pose (with whether she is in the air), one pad and start speed, with
/// a line naming them.
fn tie(
    (pose, airborne): (Pose, bool),
    (held, pressed): (u16, u16),
    moving: bool,
) -> Option<(Vec<&'static str>, String)> {
    let situation = classify(pose, !airborne);
    let moment = Moment {
        situation,
        pose,
        moving,
        airborne,
        pad: Buttons(held),
        pressed: Buttons(pressed),
        equipment: Equipment::default(),
        met: None,
    };
    let fit: Vec<&Row> = situation
        .rows
        .iter()
        .filter(|r| moment.fits(r.needs))
        .collect();
    let won = unbeaten(&moment, &fit);
    let mutual = fit
        .iter()
        .find(|a| fit.iter().any(|b| moment.beats(a, b) && moment.beats(b, a)));
    let tied: Vec<&str> =
        mutual.map_or_else(|| won.iter().map(|r| r.name).collect(), |a| vec![a.name]);
    (won.len() > 1 || mutual.is_some()).then(|| {
        let line = format!(
            "{:?} rows {tied:?} are left, e.g. {pose:?} airborne {airborne} held {held:09b} pressed {pressed:07b}",
            pose.action
        );
        (tied, line)
    })
}

#[test]
fn every_tie_is_decided() {
    let (situations, pads) = (situations(), pads());
    let mut failures: Vec<(Vec<&str>, String)> = Vec::new();
    let cases = situations
        .iter()
        .flat_map(|s| pads.iter().map(move |p| (s, p)));
    for (&situation, &pad) in cases {
        for moving in [false, true] {
            // One line per situation's rows left tied, with a first pad that ties them.
            let found =
                tie(situation, pad, moving).filter(|t| !failures.iter().any(|f| f.0 == t.0));
            failures.extend(found);
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
