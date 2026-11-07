use data::enemy::face_block::{FaceBlockLists, GlowColours};
use data::enemy::program::Target;
use data::id::ProgramId;

use super::{Glow, glow, init, main_ai};
use crate::enemy::instructions::label;
use crate::state::room::{
    EnemyCommon, EnemyPaletteCycle, FaceBlock, GlowSet, GraphicsDrawnHook, PaletteIndex,
    SpritePalette,
};
use crate::state::samus::{Item, Samus};
use crate::state::slots::Frames;
use crate::state::video::Colour;

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
    let mut samus = Samus::default();
    samus.motion.position.x = crate::model::num::Fixed::new(0x100_u16.wrapping_add(x), 0);
    samus.motion.position.y = crate::model::num::Fixed::new(0x80_u16.wrapping_add(y), 0);
    samus.equipment.collected.bits = Item::MorphBall.bit();
    let (mut hook, mut cycle) = (GraphicsDrawnHook::None, EnemyPaletteCycle::default());
    let mut glow = Glow {
        hook: &mut hook,
        cycle: &mut cycle,
    };
    init(&mut common, &LISTS, &samus, &mut glow);
    (common, samus)
}

/// The main AI's frame, with the glow's controls thrown away.
fn step(common: &mut EnemyCommon, block: &mut FaceBlock, samus: &Samus) {
    let (mut hook, mut cycle) = (GraphicsDrawnHook::None, EnemyPaletteCycle::default());
    let mut glow = Glow {
        hook: &mut hook,
        cycle: &mut cycle,
    };
    main_ai(common, block, samus, &LISTS, &mut glow);
}

/// Palette 7's index, as an enemy's palette word holds it.
const PALETTE_7: PaletteIndex = PaletteIndex(0x0e00);

/// The hook and cycle the init leaves for a face block on palette 7, its cycle at set 5.
fn initialised(samus: &Samus) -> (GraphicsDrawnHook, EnemyPaletteCycle) {
    let mut common = EnemyCommon {
        palette: PALETTE_7,
        ..EnemyCommon::default()
    };
    let mut hook = GraphicsDrawnHook::FaceBlockGlow;
    let mut cycle = EnemyPaletteCycle {
        set: GlowSet(5),
        ..EnemyPaletteCycle::default()
    };
    let mut glow = Glow {
        hook: &mut hook,
        cycle: &mut cycle,
    };
    init(&mut common, &LISTS, samus, &mut glow);
    (hook, cycle)
}

#[test]
fn init_sets_the_glow_with_the_morph_ball_and_an_rtl_without() {
    let (_, mut samus) = scene(0, 0, 0);
    let cycle = EnemyPaletteCycle {
        palette: SpritePalette(7),
        set: GlowSet(5),
        timer: Frames(0x10),
    };
    assert_eq!(
        initialised(&samus),
        (GraphicsDrawnHook::FaceBlockGlow, cycle)
    );
    samus.equipment.collected.bits = 0;
    assert_eq!(initialised(&samus), (GraphicsDrawnHook::None, cycle));
}

#[test]
fn the_main_ai_sets_the_glow_and_its_turn_restarts_the_wait() {
    // Never left, with Samus $10 columns right: it turns.
    let (mut common, samus) = scene(1, 0x10, 0);
    common.palette = PALETTE_7;
    let mut block = FaceBlock::default();
    let mut hook = GraphicsDrawnHook::None;
    let mut cycle = EnemyPaletteCycle {
        timer: Frames(3),
        ..EnemyPaletteCycle::default()
    };
    let mut glow = Glow {
        hook: &mut hook,
        cycle: &mut cycle,
    };
    main_ai(&mut common, &mut block, &samus, &LISTS, &mut glow);
    assert!(block.activated);
    assert_eq!(hook, GraphicsDrawnHook::FaceBlockGlow);
    assert_eq!(
        (cycle.palette, cycle.timer),
        (SpritePalette(7), Frames(0x10))
    );
    // Turned, it keeps setting the hook and palette but leaves the wait alone.
    hook = GraphicsDrawnHook::None;
    cycle.timer = Frames(3);
    let mut glow = Glow {
        hook: &mut hook,
        cycle: &mut cycle,
    };
    main_ai(&mut common, &mut block, &samus, &LISTS, &mut glow);
    assert_eq!(
        (hook, cycle.timer),
        (GraphicsDrawnHook::FaceBlockGlow, Frames(3))
    );
}

#[test]
fn without_the_morph_ball_the_main_ai_leaves_the_glow() {
    let (mut common, mut samus) = scene(0, 0x10, 0);
    samus.equipment.collected.bits = 0;
    let mut block = FaceBlock::default();
    let mut hook = GraphicsDrawnHook::None;
    let mut cycle = EnemyPaletteCycle::default();
    let mut glow = Glow {
        hook: &mut hook,
        cycle: &mut cycle,
    };
    main_ai(&mut common, &mut block, &samus, &LISTS, &mut glow);
    assert_eq!(
        (hook, cycle),
        (GraphicsDrawnHook::None, EnemyPaletteCycle::default())
    );
}

/// Eight sets whose colours name the set and the colour.
fn colours() -> GlowColours {
    let mut sets = [[0; 4]; 8];
    for (set, colours) in (0_u16..).zip(sets.iter_mut()) {
        for (colour, value) in (0_u16..).zip(colours.iter_mut()) {
            *value = (set << 4) | colour;
        }
    }
    GlowColours(sets)
}

#[test]
fn the_glow_writes_a_set_to_colours_9_to_12_every_16_frames() {
    let mut cycle = EnemyPaletteCycle {
        palette: SpritePalette(7),
        set: GlowSet(7),
        timer: Frames(2),
    };
    let mut palettes = [Colour::default(); 256];
    glow(&mut cycle, &mut palettes, &colours());
    assert_eq!(cycle.timer, Frames(1));
    assert_eq!(palettes, [Colour::default(); 256]);
    glow(&mut cycle, &mut palettes, &colours());
    assert_eq!((cycle.timer, cycle.set), (Frames(0x10), GlowSet(0)));
    // Sprite palette 7 is CGRAM's last sixteen colours: colours 9 to 12 are $F9 to $FC.
    let written: Vec<usize> = (0..256)
        .filter(|&index| palettes[index] != Colour::default())
        .collect();
    assert_eq!(written, [0xf9, 0xfa, 0xfb, 0xfc]);
    assert_eq!(palettes[0xf9..0xfd], [0x70, 0x71, 0x72, 0x73].map(Colour));
    for _ in 0..0x10 {
        glow(&mut cycle, &mut palettes, &colours());
    }
    assert_eq!(cycle.set, GlowSet(1));
    assert_eq!(palettes[0xf9], Colour(0));
}

#[test]
fn a_palette_index_names_its_sprite_palette_by_bits_9_to_11() {
    assert_eq!(SpritePalette::of(PALETTE_7), SpritePalette(7));
    assert_eq!(SpritePalette::of(PaletteIndex(0x0200)), SpritePalette(1));
    // The priority bits above are not the palette's.
    assert_eq!(SpritePalette::of(PaletteIndex(0x3000)), SpritePalette(0));
    assert_eq!(SpritePalette(1).colour(9), 0x99);
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
    step(&mut common, &mut block, &samus);
    assert_eq!(block, FaceBlock::default());
    assert_eq!(common, scene(0, 0, 0).0);
}

#[test]
fn samus_within_range_on_its_turning_side_turns_it_once() {
    // Never right; Samus $3F columns left and $3F rows down: |-$3F| - $40 is negative.
    let (mut common, samus) = scene(0, 0x3f_u16.wrapping_neg(), 0x3f);
    let mut block = FaceBlock::default();
    step(&mut common, &mut block, &samus);
    assert!(block.activated);
    assert_eq!(block.samus_offset, 0xffc1);
    assert_eq!(common.cursor.at, label(LISTS.turning_left));
    assert_eq!(common.cursor.delay, Frames(1));
    // Turned, it no longer measures her.
    let mut moved = samus;
    moved.motion.position.x = crate::model::num::Fixed::new(0x110, 0);
    let before = (common, block);
    step(&mut common, &mut block, &moved);
    assert_eq!((common, block), before);
}

#[test]
fn never_left_turns_right_only() {
    let (mut common, samus) = scene(1, 0x3f_u16.wrapping_neg(), 0);
    let mut block = FaceBlock::default();
    step(&mut common, &mut block, &samus);
    assert!(!block.activated);
    assert_eq!(block.samus_offset, 0xffc1);
    assert_eq!(common.cursor.at, label(LISTS.facing_forward));
    // Zero columns is the right side.
    let (mut common, samus) = scene(1, 0, 0);
    step(&mut common, &mut block, &samus);
    assert!(block.activated);
    assert_eq!(block.samus_offset, 0);
    assert_eq!(common.cursor.at, label(LISTS.turning_right));
}

#[test]
fn out_of_rows_it_doesnt_measure_and_out_of_columns_it_measures_only() {
    // $40 rows: |$40| - $40 is zero, not negative.
    let (mut common, samus) = scene(0, 0x10_u16.wrapping_neg(), 0x40);
    let mut block = FaceBlock::default();
    step(&mut common, &mut block, &samus);
    assert_eq!(block, FaceBlock::default());
    // $40 columns left, in rows: the offset is stored, but it doesn't turn.
    let (mut common, samus) = scene(0, 0x40_u16.wrapping_neg(), 0);
    step(&mut common, &mut block, &samus);
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
    step(&mut common, &mut block, &samus);
    assert!(block.activated);
    assert_eq!(common.cursor.at, label(LISTS.turning_right));
}
