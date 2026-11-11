//! `PadChart`: every situation and pad the chart can meet is decided, off the ground or on it,
//! moving or not, with the spring ball or without, and whatever her moves met. Where rows of her
//! situation both fit, the one that loses to the other by rank or by a beats clause drops out;
//! at most one fitting row may be left, beaten by no other (with none, the last row decides), so
//! the chart never meets a tie and needs no rule for one. No two fitting rows beat each other,
//! every beats clause, with a condition or not, names a row of its own situation, and no two
//! rows of a situation or a table share an identity.

use super::{Moment, Reading, table};
use crate::rows::{BACK, FORWARD, Met, Next, Row, RowId, Situation, Test};
use crate::rules::Rules;
use crate::situation::{CLASSES, UnknownPose, classify};
use clean_types::Action::{Crouching, Morphing, Rolling, StandingUp, Unmorphing};
use clean_types::button::{DOWN, PAD, SIDES, UP};
use clean_types::{Aim, Body, Buttons, Facing, Fixed, Gun, Legs, Pose, ZERO, px};
use core::num::Wrapping;
use core::ptr;

/// Every situation the classes give, each once.
fn situations() -> Vec<&'static Situation> {
    let mut all: Vec<&'static Situation> = Vec::new();
    for class in CLASSES {
        if !all.iter().any(|s| ptr::eq(*s, class.is)) {
            all.push(class.is);
        }
    }
    all
}

/// Every outcome of the chart's rows: each situation's pad chart rows and its last row, the rows
/// of `WhatHerMovesMet` and `AnimationEnds`, and the blocked step's, named by their row.
fn every_outcome() -> Vec<(&'static Next, String)> {
    let pad = situations()
        .into_iter()
        .flat_map(|s| s.rows.iter().chain([&s.last]));
    pad.chain(table::MOVES_MET)
        .chain(table::ANIMATION_ENDS)
        .map(|r| (&r.next, format!("{:?}", r.id)))
        .chain([(&table::STEP_BLOCKED, "the blocked step's".to_owned())])
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
    for (next, row) in every_outcome() {
        let lowered = next.lower.map(|by| adjusted.lowers(by));
        let expected = match next.action {
            Some(Crouching) => Some(adjusted.crouch_lowers),
            Some(Morphing) => Some(adjusted.morph_lowers),
            Some(StandingUp | Unmorphing) => Some(ZERO),
            Some(_) | None => None,
        };
        assert_eq!(lowered, expected, "the row {row}");
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

/// `MovesAsShePosed`: a pose no situation names (upright `Rolling`) is reported, by the
/// situations and by the chart, never given a situation's motion.
#[test]
fn a_pose_no_situation_names_is_reported() {
    let pose = Pose {
        body: Body::Upright,
        action: Rolling,
        facing: Facing::Right,
        aim: Aim::Forward,
        gun: Gun::Stowed,
        legs: Legs::Together,
    };
    assert_eq!(classify(pose).err(), Some(UnknownPose(pose)));
    let still = Moment {
        pad: Buttons(0),
        pressed: Buttons(0),
        spring_ball: false,
        animation_ends: None,
        speed: ZERO,
        airborne: false,
        met: None,
    };
    assert_eq!(still.next_pose(pose), Err(UnknownPose(pose)));
}

/// The poses that put her in situation `s`: those of its classes that no earlier class takes,
/// facing either side, with either gun (the rows read no legs, so they are together).
fn poses(s: &'static Situation) -> Vec<Pose> {
    let mut poses: Vec<Pose> = Vec::new();
    for class in CLASSES.iter().filter(|c| ptr::eq(c.is, s)) {
        let parts = class
            .body
            .iter()
            .flat_map(|&body| class.action.iter().map(move |&action| (body, action)));
        let aimed = parts.flat_map(|(body, action)| {
            class.aim.iter().map(move |&aim| Pose {
                body,
                action,
                facing: Facing::Left,
                aim,
                gun: Gun::Stowed,
                legs: Legs::Together,
            })
        });
        poses.extend(aimed);
    }
    poses
        .into_iter()
        .flat_map(|p| [Facing::Left, Facing::Right].map(|facing| Pose { facing, ..p }))
        .flat_map(|p| [Gun::Drawn, Gun::Stowed].map(|gun| Pose { gun, ..p }))
        .filter(|p| classify(*p).is_ok_and(|c| ptr::eq(c, s)))
        .collect()
}

/// A row test's buttons as pad bits: forward and back as left and right.
const fn pad_bits(b: Buttons) -> u16 {
    let sides = if b.any(Buttons(FORWARD.0 | BACK.0)) {
        SIDES.0
    } else {
        0
    };
    b.0 & PAD.0 | sides
}

/// The buttons these tests name, held and just pressed, as pad bits.
fn named(tests: &[Test]) -> (u16, u16) {
    tests
        .iter()
        .fold((0, 0), |(held, pressed), test| match *test {
            Test::Held(b) => (held | pad_bits(b), pressed),
            Test::Pressed(b) => (held, pressed | pad_bits(b)),
            Test::Not(inner) => {
                let (h, p) = named(core::slice::from_ref(inner));
                (held | h, pressed | p)
            }
            Test::Aimed(_)
            | Test::Gun(_)
            | Test::Faces(_)
            | Test::Body(_)
            | Test::Action(_)
            | Test::Doing(_)
            | Test::Airborne
            | Test::SpringBall
            | Test::Moving
            | Test::Met(_) => (held, pressed),
        })
}

/// Every set of the bits of `of`.
fn subsets(of: u16) -> impl Iterator<Item = u16> {
    (0..=of).filter(move |b| b & !of == 0)
}

/// The pads situation `s` can tell apart: every set of the buttons its rows and its beats
/// clauses' conditions name, held with at most one of up and down (§7: no record holds both;
/// left and right may be held together), and of those it names just pressed, every set of the
/// ones held.
fn pads(s: &Situation) -> Vec<(Buttons, Buttons)> {
    let tests = s.rows.iter().chain([&s.last]).flat_map(|r| {
        let conditions = r.beats_unless.iter().flat_map(|c| c.unless.iter().copied());
        core::iter::once(r.needs).chain(conditions)
    });
    let (held, pressed) = tests
        .map(named)
        .fold((0, 0), |(h, p), (th, tp)| (h | th, p | tp));
    let up_down = UP.0 | DOWN.0;
    subsets(held | pressed)
        .filter(|h| h & up_down != up_down)
        .flat_map(|h| subsets(pressed & h).map(move |p| (Buttons(h), Buttons(p))))
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
        .flat_map(|s| s.rows.iter().filter_map(|a| misnamed(s.rows, a)))
        .collect();
    assert!(problems.is_empty(), "{problems:?}");
}

/// `PadChart`: a row's identity is its own within its situation, its last row included, and
/// within its table, so a beats clause names one row.
#[test]
fn no_two_rows_share_an_identity() {
    for s in situations() {
        let twice = shared(s.rows.iter().chain([&s.last]));
        assert!(twice.is_empty(), "rows of one situation share {twice:?}");
    }
    for rows in [table::MOVES_MET, table::ANIMATION_ENDS] {
        let twice = shared(rows.iter());
        assert!(twice.is_empty(), "rows of one table share {twice:?}");
    }
}

/// A horizontal speed at the frame's start that isn't 0.
const MOVING: Fixed = Wrapping(0x1_0000);

/// Whether she is off the ground, and her horizontal speed at the frame's start: the parts of
/// her motion the rows read.
const MOTIONS: [(bool, Fixed); 4] = [(false, ZERO), (false, MOVING), (true, ZERO), (true, MOVING)];

/// Each thing her moves can meet (`WhatHerMovesMet`), and nothing.
const EVERY_MET: [Option<Met>; 6] = [
    None,
    Some(Met::Wall),
    Some(Met::WallKick),
    Some(Met::Floor),
    Some(Met::NoFloor),
    Some(Met::Ceiling),
];

/// Every moment the rows can tell apart besides the pad: her motion, with the spring ball or
/// without, and each thing her moves can meet. The pad chart's rows read no animation's end.
fn moments() -> Vec<Moment> {
    let motions = MOTIONS
        .into_iter()
        .flat_map(|motion| [false, true].map(|spring_ball| (motion, spring_ball)));
    motions
        .flat_map(|((airborne, speed), spring_ball)| {
            EVERY_MET.map(|met| Moment {
                pad: Buttons(0),
                pressed: Buttons(0),
                spring_ball,
                animation_ends: None,
                speed,
                airborne,
                met,
            })
        })
        .collect()
}

/// The rows left tied in situation `s` for a pose and a moment, with a line naming them. Jump
/// held through only adds jump to the presses, which `pads` already gives with jump held.
fn tie(s: &'static Situation, pose: Pose, moment: &Moment) -> Option<(Vec<RowId>, String)> {
    let reading = Reading {
        moment,
        pose,
        situation: s,
    };
    let fit: Vec<&Row> = s.rows.iter().filter(|r| reading.fits(r.needs)).collect();
    let won = unbeaten(&reading, &fit);
    let mutual = fit.iter().find(|a| {
        fit.iter()
            .any(|b| reading.beats(a, b) && reading.beats(b, a))
    });
    let tied: Vec<RowId> =
        mutual.map_or_else(|| won.iter().map(|r| r.id).collect(), |a| vec![a.id]);
    (won.len() > 1 || mutual.is_some()).then(|| {
        let line = format!(
            "{:?} rows {tied:?} are left, e.g. {pose:?} {moment:?}",
            pose.action
        );
        (tied, line)
    })
}

/// Every tie in situation `s`, over its own poses, the moments the rows read and its own pads.
fn ties(s: &'static Situation) -> Vec<(Vec<RowId>, String)> {
    let pads = pads(s);
    let moments: Vec<Moment> = moments()
        .into_iter()
        .flat_map(|m| {
            pads.iter()
                .map(move |&(pad, pressed)| Moment { pad, pressed, ..m })
        })
        .collect();
    poses(s)
        .into_iter()
        .flat_map(|pose| moments.iter().filter_map(move |m| tie(s, pose, m)))
        .collect()
}

#[test]
fn every_tie_is_decided() {
    let mut failures: Vec<(Vec<RowId>, String)> = Vec::new();
    // One line per situation's rows left tied, with a first case that ties them.
    for found in situations().into_iter().flat_map(ties) {
        if !failures.iter().any(|f| f.0 == found.0) {
            failures.push(found);
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
