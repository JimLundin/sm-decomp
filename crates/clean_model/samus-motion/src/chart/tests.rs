//! `PadChart`: every situation and pad the chart can meet is decided. Where rows of her
//! situation both fit, the one that loses to the other by rank or by a beats clause drops out;
//! at most one fitting row may be left, beaten by no other (with none, the last row decides), so
//! the chart never meets a tie and needs no rule for one. No two fitting rows beat each other,
//! every beats clause, with a condition or not, names a row of its own situation, and no two
//! rows of a situation or a table share an identity.

use super::{Moment, Reading, table};
use crate::rows::{BACK, FORWARD, Row, RowId};
use crate::rules::Rules;
use crate::situation::{ACTIONS, CLASSES, STANDING, classify};
use crate::tick::Frame;
use clean_types::Action::{
    self, AgainstWall, Crouching, Falling, Jumping, Landing, Moonwalking, Morphing, Rolling,
    Running, Spinning, StandingUp, Still, TakingOff, Turning, Unmorphing, WallJumping,
};
use clean_types::button::{DOWN, JUMP, LEFT, PAD, RIGHT, SHOOT, UP};
use clean_types::{
    Aim, Body, Buttons, Facing, Gun, LandedFrom, Legs, Pair, Parity, Pose, Radius, Set, Turn, ZERO,
    px,
};
use core::num::Wrapping;

/// Every row of the chart: each situation's pad chart rows and its last row, and the rows of
/// `WhatHerMovesMet` and `AnimationEnds`.
fn every_row() -> Vec<&'static Row> {
    let situations = CLASSES.iter().map(|c| c.is).chain([&STANDING]);
    let pad = situations.flat_map(|s| s.rows.iter().chain([&s.last]));
    pad.chain(table::MOVES_MET)
        .chain(table::ANIMATION_ENDS)
        .collect()
}

/// `ShrinkingLowersHer`: in an adjusted game whose crouch and morph lower her by other
/// distances, a row into `Crouching` lowers her by the crouch's, a row into `Morphing` by the
/// morph's, standing up and unmorphing by 0 px, and no other change lowers her.
#[test]
fn a_crouch_and_a_morph_lower_her_by_their_distances() {
    let adjusted = Rules {
        crouch_lowers: px(Wrapping(6)),
        morph_lowers: px(Wrapping(11)),
        ..Rules::REFERENCE
    };
    for row in every_row() {
        let lowered = row.next.lower.map(|by| adjusted.lowers(by));
        let expected = match row.next.action {
            Some(Crouching) => Some(adjusted.crouch_lowers),
            Some(Morphing) => Some(adjusted.morph_lowers),
            Some(StandingUp | Unmorphing) => Some(ZERO),
            Some(_) | None => None,
        };
        assert_eq!(lowered, expected, "the row {:?}", row.id);
    }
    let vanilla = Rules::REFERENCE;
    assert_eq!(vanilla.crouch_lowers, px(Wrapping(5)), "the crouch's 5 px");
    assert_eq!(vanilla.morph_lowers, px(Wrapping(9)), "the morph's 9 px");
}

/// `PadChart`: forward and back, the facing-relative view, are no button of the pad, nor each
/// other, so `seen` alone makes them.
#[test]
fn forward_and_back_are_apart_from_the_pad() {
    assert_eq!(FORWARD.0 & PAD.0, 0, "forward is a pad button");
    assert_eq!(BACK.0 & PAD.0, 0, "back is a pad button");
    assert_eq!(FORWARD.0 & BACK.0, 0, "forward and back overlap");
    assert_ne!(FORWARD.0, 0, "forward is empty");
    assert_ne!(BACK.0, 0, "back is empty");
}

/// The action after `a` in `ACTIONS`, by a match on `Action` with no wildcard: an action added to
/// the type doesn't compile here until it is given its place, and then `every_action_is_listed`
/// fails until `ACTIONS` holds it.
const fn after(a: Action) -> Option<Action> {
    match a {
        AgainstWall => Some(Crouching),
        Crouching => Some(Falling),
        Falling => Some(Jumping),
        Jumping => Some(Landing(LandedFrom::Jump)),
        Landing(LandedFrom::Jump) => Some(Landing(LandedFrom::Spin)),
        Landing(LandedFrom::Spin) => Some(Moonwalking),
        Moonwalking => Some(Morphing),
        Morphing => Some(Rolling),
        Rolling => Some(Running),
        Running => Some(Spinning),
        Spinning => Some(StandingUp),
        StandingUp => Some(Still),
        Still => Some(TakingOff),
        TakingOff => Some(Turning(Turn::Fall)),
        Turning(Turn::Fall) => Some(Turning(Turn::Ground)),
        Turning(Turn::Ground) => Some(Turning(Turn::Jump)),
        Turning(Turn::Jump) => Some(Turning(Turn::MoonwalkJump)),
        Turning(Turn::MoonwalkJump) => Some(Unmorphing),
        Unmorphing => Some(WallJumping),
        WallJumping => None,
    }
}

/// `MovesAsShePosed`: the classes' list of every action holds each action of `Action` once, so
/// a class of any action (`situation`'s `ANY`) can't miss one.
#[test]
fn every_action_is_listed() {
    let every: Vec<Action> = core::iter::successors(Some(AgainstWall), |a| after(*a))
        .take(ACTIONS.len().saturating_add(1))
        .collect();
    assert_eq!(every, ACTIONS, "ACTIONS isn't every action, each once");
}

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
    let up_down = UP.0 | DOWN.0;
    let pressable = Buttons(DOWN.0 | JUMP.0 | UP.0 | LEFT.0 | RIGHT.0 | SHOOT.0);
    (0..=PAD.0)
        .filter(|held| held & !PAD.0 == 0 && held & up_down != up_down)
        .flat_map(|held| {
            (0..=pressable.0)
                .filter(move |pressed| pressed & !(held & pressable.0) == 0)
                .map(move |pressed| (held, pressed))
        })
        .collect()
}

/// The fitting rows that no other fitting row beats.
fn unbeaten<'a>(reading: &Reading, fit: &[&'a Row]) -> Vec<&'a Row> {
    fit.iter()
        .copied()
        .filter(|a| !fit.iter().any(|b| reading.beats(b, a)))
        .collect()
}

/// A beats clause that names no row of its situation, or two rows that beat each other.
fn misnamed(rows: &[Row], a: &Row) -> Option<String> {
    let conditional = a.beats_unless.iter().map(|c| &c.row);
    let unknown = a
        .beats
        .iter()
        .chain(conditional)
        .find(|id| !rows.iter().any(|b| b.id == **id));
    let mutual = rows
        .iter()
        .find(|b| a.beats.contains(&b.id) && b.beats.contains(&a.id));
    unknown
        .map(|id| format!("{:?} beats {id:?}, not a row of its situation", a.id))
        .or_else(|| mutual.map(|b| format!("{:?} and {:?} beat each other", a.id, b.id)))
}

/// The identities that more than one of these rows has.
fn shared<'a>(rows: impl Iterator<Item = &'a Row>) -> Vec<RowId> {
    let ids: Vec<RowId> = rows.map(|r| r.id).collect();
    let mut twice: Vec<RowId> = ids
        .iter()
        .enumerate()
        .filter(|&(n, id)| ids[..n].contains(id))
        .map(|(_, id)| *id)
        .collect();
    twice.dedup();
    twice
}

#[test]
fn beats_clauses_name_rows_of_their_situation_and_go_one_way() {
    let problems: Vec<String> = situations()
        .into_iter()
        .flat_map(|(pose, _)| {
            let rows = classify(pose).rows;
            rows.iter().filter_map(|a| misnamed(rows, a))
        })
        .collect();
    assert!(problems.is_empty(), "{problems:?}");
}

/// `PadChart`: a row's identity is its own within its situation, its last row included, and
/// within its table, so a beats clause names one row.
#[test]
fn no_two_rows_share_an_identity() {
    let situations = CLASSES.iter().map(|c| c.is).chain([&STANDING]);
    for s in situations {
        let twice = shared(s.rows.iter().chain([&s.last]));
        assert!(twice.is_empty(), "rows of one situation share {twice:?}");
    }
    for rows in [table::MOVES_MET, table::ANIMATION_ENDS] {
        let twice = shared(rows.iter());
        assert!(twice.is_empty(), "rows of one table share {twice:?}");
    }
}

/// The rows left tied for a pose (with whether she is in the air), one pad and start speed, with
/// a line naming them.
fn tie(
    (pose, airborne): (Pose, bool),
    (held, pressed): (u16, u16),
    moving: bool,
) -> Option<(Vec<RowId>, String)> {
    let situation = classify(pose);
    let body = |_: Pose| -> Radius {
        Pair {
            x: Wrapping(0),
            y: Wrapping(0),
        }
    };
    let frame = Frame {
        pad: Buttons(held),
        pressed: Buttons(pressed),
        items: Set::default(),
        spin_turned_far: false,
        elevator_moving: false,
        parity: Parity::Even,
        animation_ends: false,
        body: &body,
    };
    // Jump held through only adds jump to the presses, which `pads` already gives with jump held.
    let moment = Moment {
        moving,
        airborne,
        jump_held_through: false,
        frame: &frame,
        met: None,
    };
    let reading = Reading {
        moment: &moment,
        pose,
        situation,
    };
    let fit: Vec<&Row> = situation
        .rows
        .iter()
        .filter(|r| reading.fits(r.needs))
        .collect();
    let won = unbeaten(&reading, &fit);
    let mutual = fit.iter().find(|a| {
        fit.iter()
            .any(|b| reading.beats(a, b) && reading.beats(b, a))
    });
    let tied: Vec<RowId> =
        mutual.map_or_else(|| won.iter().map(|r| r.id).collect(), |a| vec![a.id]);
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
    let mut failures: Vec<(Vec<RowId>, String)> = Vec::new();
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
