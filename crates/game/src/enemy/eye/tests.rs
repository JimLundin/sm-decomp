use data::enemy::eye::{ByFacing, ByMountSide, EyeLists};
use data::enemy::program::{SpritemapId, Target};
use data::id::ProgramId;

use super::{UnknownMountSide, angle, init, main_ai};
use crate::enemy::instructions::label;
use crate::state::room::{EnemyCommon, EnemyProperty, EyePhase, MorphBallEye};
use crate::state::samus::{Item, Samus};
use crate::state::slots::Frames;

const fn place(program: u16, step: u16) -> Target {
    Target {
        program: ProgramId(program),
        step,
    }
}

/// Distinct places for every list: the active frames are program 0's steps, then one
/// program per list.
fn lists() -> EyeLists {
    EyeLists {
        active: std::array::from_fn(|frame| place(0, u16::try_from(frame).unwrap())),
        closed: ByFacing {
            right: place(1, 0),
            left: place(2, 0),
        },
        activating: ByFacing {
            right: place(3, 0),
            left: place(4, 0),
        },
        deactivating: ByFacing {
            right: place(5, 0),
            left: place(6, 0),
        },
        mount: ByMountSide {
            left: place(7, 0),
            right: place(8, 0),
            up: place(9, 0),
            down: place(10, 0),
        },
        empty: SpritemapId(3),
    }
}

/// The eye at ($100, $80) and Samus, with the morph ball, `x` and `y` pixels from it.
fn scene(x: u16, y: u16) -> (EnemyCommon, Samus) {
    let mut common = EnemyCommon::default();
    common.position.x.pixels = 0x100;
    common.position.y.pixels = 0x80;
    let mut samus = Samus::default();
    samus.motion.position.x = crate::model::num::Fixed::new(0x100_u16.wrapping_add(x), 0);
    samus.motion.position.y = crate::model::num::Fixed::new(0x80_u16.wrapping_add(y), 0);
    samus.equipment.collected.bits = Item::MorphBall.bit();
    (common, samus)
}

fn eye(phase: EyePhase, function_timer: u16) -> MorphBallEye {
    MorphBallEye {
        phase,
        function_timer: Frames(function_timer),
        ..MorphBallEye::default()
    }
}

#[test]
fn the_angle_is_0_up_and_turns_clockwise() {
    assert_eq!(angle(0, 5_u16.wrapping_neg()), 0);
    assert_eq!(angle(5, 0), 0x40);
    assert_eq!(angle(0, 5), 0x80);
    assert_eq!(angle(5_u16.wrapping_neg(), 0), 0xc0);
    // Diagonals: $10 * $100 / $10 = $100, an eighth is $20 past the axis.
    assert_eq!(angle(0x10, 0x10), 0x60);
    assert_eq!(angle(0x10, 0x10_u16.wrapping_neg()), 0x20);
    assert_eq!(angle(0x10_u16.wrapping_neg(), 0x10), 0xa0);
    // $8 * $100 / $10 = $80, an eighth $10: right and a little down.
    assert_eq!(angle(0x10, 8), 0x50);
}

#[test]
fn nothing_happens_without_the_morph_ball() {
    let (mut common, mut samus) = scene(0, 0);
    samus.equipment.collected.bits = 0;
    let mut body = eye(EyePhase::WaitingForSamus, 0);
    main_ai(&mut common, &mut body, &samus, &lists());
    assert_eq!(body, eye(EyePhase::WaitingForSamus, 0));
    assert_eq!(common, scene(0, 0).0);
}

#[test]
fn samus_within_80h_opens_it_facing_its_way() {
    // $7F columns left: |-$7F| - $80 is negative.
    let (mut common, samus) = scene(0x7f_u16.wrapping_neg(), 0x10);
    common.parameter_1 = 1;
    let mut body = eye(EyePhase::WaitingForSamus, 0);
    main_ai(&mut common, &mut body, &samus, &lists());
    assert_eq!(body.phase, EyePhase::Activating);
    assert_eq!(body.function_timer, Frames(0x20));
    assert_eq!(common.cursor.delay, Frames(1));
    assert_eq!(common.cursor.at, label(lists().activating.left));
}

#[test]
fn samus_80h_away_leaves_it_closed() {
    for (x, y) in [(0x80, 0), (0, 0x80)] {
        let (mut common, samus) = scene(x, y);
        let mut body = eye(EyePhase::WaitingForSamus, 0);
        main_ai(&mut common, &mut body, &samus, &lists());
        assert_eq!(body.phase, EyePhase::WaitingForSamus);
        assert_eq!(common.cursor, scene(x, y).0.cursor);
    }
}

#[test]
fn opening_ends_when_the_timer_reaches_zero_or_below_and_aims() {
    let (mut common, samus) = scene(0x10, 0);
    let mut body = eye(EyePhase::Activating, 2);
    main_ai(&mut common, &mut body, &samus, &lists());
    assert_eq!(body, eye(EyePhase::Activating, 1));
    main_ai(&mut common, &mut body, &samus, &lists());
    assert_eq!(body.phase, EyePhase::Active);
    assert_eq!(body.function_timer, Frames(0));
    assert_eq!(body.angle, 0x40);
    // From zero the timer wraps negative, which also ends it.
    let mut body = eye(EyePhase::Activating, 0);
    main_ai(&mut common, &mut body, &samus, &lists());
    assert_eq!(body.phase, EyePhase::Active);
    assert_eq!(body.function_timer, Frames(0xffff));
}

#[test]
fn open_it_follows_samus_with_the_frame_for_her_angle() {
    // Right and a little down: angle $50, the active frame $50 / $10 = 5.
    let (mut common, samus) = scene(0x10, 8);
    let mut body = eye(EyePhase::Active, 0);
    body.activated = true;
    main_ai(&mut common, &mut body, &samus, &lists());
    assert_eq!(body.phase, EyePhase::Active);
    assert_eq!(body.angle, 0x50);
    assert!(body.activated);
    assert_eq!(common.cursor.at, label(place(0, 5)));
    assert_eq!(common.cursor.delay, Frames(1));
}

#[test]
fn it_stays_open_to_b0h_columns_and_closes_past() {
    let (mut common, samus) = scene(0xaf, 0);
    let mut body = eye(EyePhase::Active, 0);
    main_ai(&mut common, &mut body, &samus, &lists());
    assert_eq!(body.phase, EyePhase::Active);
    let (mut common, samus) = scene(0xb0, 0);
    let mut body = eye(EyePhase::Active, 0);
    body.activated = true;
    body.angle = 0x40;
    main_ai(&mut common, &mut body, &samus, &lists());
    assert_eq!(body.phase, EyePhase::Deactivating);
    assert!(!body.activated);
    assert_eq!(body.function_timer, Frames(0x20));
    // The angle stays as it last aimed.
    assert_eq!(body.angle, 0x40);
    assert_eq!(common.cursor.at, label(lists().deactivating.right));
    assert_eq!(common.cursor.delay, Frames(1));
}

#[test]
fn closing_waits_for_samus_again_when_the_timer_runs_out() {
    let (mut common, samus) = scene(0x100, 0);
    let mut body = eye(EyePhase::Deactivating, 1);
    main_ai(&mut common, &mut body, &samus, &lists());
    assert_eq!(body, eye(EyePhase::WaitingForSamus, 0));
}

#[test]
fn a_mount_does_nothing() {
    let (mut common, samus) = scene(0, 0);
    let mut body = eye(EyePhase::Mount, 5);
    main_ai(&mut common, &mut body, &samus, &lists());
    assert_eq!(body, eye(EyePhase::Mount, 5));
    assert_eq!(common, scene(0, 0).0);
}

#[test]
fn init_closes_an_eye_facing_its_way() {
    let (mut common, _) = scene(0, 0);
    common.timer = 7;
    common.cursor.delay = Frames(9);
    let mut body = eye(EyePhase::Mount, 0);
    assert_eq!(init(&mut common, &mut body, &lists()), Ok(()));
    assert!(
        common
            .properties
            .contains(EnemyProperty::ProcessesInstructions)
    );
    assert_eq!(common.spritemap, SpritemapId(3));
    assert_eq!(common.cursor.delay, Frames(1));
    assert_eq!(common.timer, 0);
    assert_eq!(common.cursor.at, label(lists().closed.right));
    assert_eq!(body.phase, EyePhase::WaitingForSamus);
}

#[test]
fn init_moves_a_mount_to_its_side() {
    // Side 1, right: 8 pixels right of ($100, $80).
    let (mut common, _) = scene(0, 0);
    common.parameter_2 = 0x8001;
    let mut body = MorphBallEye::default();
    assert_eq!(init(&mut common, &mut body, &lists()), Ok(()));
    assert_eq!(common.position.x.pixels, 0x108);
    assert_eq!(common.position.y.pixels, 0x80);
    assert_eq!(body.phase, EyePhase::Mount);
    assert_eq!(common.cursor.at, label(lists().mount.right));
    // Side 2, up: 8 pixels up.
    let (mut common, _) = scene(0, 0);
    common.parameter_2 = 0x8002;
    assert_eq!(init(&mut common, &mut body, &lists()), Ok(()));
    assert_eq!(common.position.y.pixels, 0x78);
    assert_eq!(common.cursor.at, label(lists().mount.up));
    // Side 4 is past the tables.
    common.parameter_2 = 0x8004;
    assert_eq!(
        init(&mut common, &mut body, &lists()),
        Err(UnknownMountSide(4))
    );
}
