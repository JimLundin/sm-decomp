use data::enemy::face_block::FaceBlockLists;
use data::enemy::program::Target;
use data::id::ProgramId;

use super::{init, main_ai};
use crate::enemy::instructions::label;
use crate::state::room::{EnemyCommon, FaceBlock};
use crate::state::samus::{Item, Samus};
use crate::state::slots::Frames;

const fn place(program: u16) -> Target {
    Target {
        program: ProgramId(program),
        step: 0,
    }
}

const LISTS: FaceBlockLists = FaceBlockLists {
    turning_left: place(1),
    turning_right: place(2),
    facing_forward: place(3),
};

/// A face block at ($100, $80) with a range of $40, as its init leaves it with
/// `parameter_2`, and Samus, with the morph ball, `x` and `y` pixels from it.
fn scene(parameter_2: u16, x: u16, y: u16) -> (EnemyCommon, Samus) {
    let mut common = EnemyCommon {
        parameter_1: 0x40,
        parameter_2,
        ..EnemyCommon::default()
    };
    common.position.x.pixels = 0x100;
    common.position.y.pixels = 0x80;
    init(&mut common, &LISTS);
    let mut samus = Samus::default();
    samus.position.x.pixels = 0x100_u16.wrapping_add(x);
    samus.position.y.pixels = 0x80_u16.wrapping_add(y);
    samus.equipment.collected.bits = Item::MorphBall.bit();
    (common, samus)
}

#[test]
fn init_faces_forward_and_keeps_the_never_side_as_a_sign() {
    // Bit 0 set: never left, $8000; clear: never right, 0. The other bits go.
    for (parameter_2, never) in [(0x0001, 0x8000), (0x0000, 0), (0x00fe, 0), (0xffff, 0x8000)] {
        let (common, _) = scene(parameter_2, 0, 0);
        assert_eq!(common.parameter_2, never);
        assert_eq!(common.cursor.at, label(LISTS.facing_forward));
    }
}

#[test]
fn nothing_happens_without_the_morph_ball() {
    let (mut common, mut samus) = scene(0, 0x10_u16.wrapping_neg(), 0);
    samus.equipment.collected.bits = 0;
    let mut block = FaceBlock::default();
    main_ai(&mut common, &mut block, &samus, &LISTS);
    assert_eq!(block, FaceBlock::default());
    assert_eq!(common, scene(0, 0, 0).0);
}

#[test]
fn samus_within_range_on_its_turning_side_turns_it_once() {
    // Never right; Samus $3F columns left and $3F rows down: |-$3F| - $40 is negative.
    let (mut common, samus) = scene(0, 0x3f_u16.wrapping_neg(), 0x3f);
    let mut block = FaceBlock::default();
    main_ai(&mut common, &mut block, &samus, &LISTS);
    assert!(block.activated);
    assert_eq!(block.samus_offset, 0xffc1);
    assert_eq!(common.cursor.at, label(LISTS.turning_left));
    assert_eq!(common.cursor.delay, Frames(1));
    // Turned, it no longer measures her.
    let mut moved = samus;
    moved.position.x.pixels = 0x110;
    let before = (common, block);
    main_ai(&mut common, &mut block, &moved, &LISTS);
    assert_eq!((common, block), before);
}

#[test]
fn never_left_turns_right_only() {
    let (mut common, samus) = scene(1, 0x3f_u16.wrapping_neg(), 0);
    let mut block = FaceBlock::default();
    main_ai(&mut common, &mut block, &samus, &LISTS);
    assert!(!block.activated);
    assert_eq!(block.samus_offset, 0xffc1);
    assert_eq!(common.cursor.at, label(LISTS.facing_forward));
    // Zero columns is the right side.
    let (mut common, samus) = scene(1, 0, 0);
    main_ai(&mut common, &mut block, &samus, &LISTS);
    assert!(block.activated);
    assert_eq!(block.samus_offset, 0);
    assert_eq!(common.cursor.at, label(LISTS.turning_right));
}

#[test]
fn out_of_rows_it_doesnt_measure_and_out_of_columns_it_measures_only() {
    // $40 rows: |$40| - $40 is zero, not negative.
    let (mut common, samus) = scene(0, 0x10_u16.wrapping_neg(), 0x40);
    let mut block = FaceBlock::default();
    main_ai(&mut common, &mut block, &samus, &LISTS);
    assert_eq!(block, FaceBlock::default());
    // $40 columns left, in rows: the offset is stored, but it doesn't turn.
    let (mut common, samus) = scene(0, 0x40_u16.wrapping_neg(), 0);
    main_ai(&mut common, &mut block, &samus, &LISTS);
    assert_eq!(
        block,
        FaceBlock {
            samus_offset: 0xffc0,
            ..FaceBlock::default()
        }
    );
    assert_eq!(common.cursor.at, label(LISTS.facing_forward));
}

#[test]
fn a_second_parameter_the_init_didnt_write_turns_both_ways() {
    let (mut common, samus) = scene(0, 0x10, 0);
    common.parameter_2 = 1;
    let mut block = FaceBlock::default();
    main_ai(&mut common, &mut block, &samus, &LISTS);
    assert!(block.activated);
    assert_eq!(common.cursor.at, label(LISTS.turning_right));
}
