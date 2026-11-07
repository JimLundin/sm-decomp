//! The Blue Brinstar face block's AI (D14), bank $A8 log
//! (<https://patrickjohnston.org/bank/A8>, `$A8:E82E` to `$A8:E91B`).
//!
//! The face block faces forward until Samus has the morph ball and comes within its range,
//! its first parameter, on both axes; then it turns to look at her once, unless she is on
//! the side its second parameter says it never turns to. With the morph ball, its palette
//! glows: the enemy graphics drawn hook cycles four of its colours every 16 frames.

use data::enemy::face_block::FaceBlockLists;
use data::enemy::face_block::{GLOW_SET_COLOURS, GlowColours};
use data::enemy::program::Target;

use super::eye::within;
use crate::enemy::instructions::label;
use crate::state::room::{
    EnemyCommon, EnemyPaletteCycle, FaceBlock, GraphicsDrawnHook, SpritePalette,
};
use crate::state::samus::{Item, Samus};
use crate::state::slots::Frames;
use crate::state::video::Colour;

/// The second parameter's bit the init keeps: set, the face block never turns left
/// (`$A8:E862`).
const NEVER_LEFT: u16 = 0x0001;

/// The sign bit of a 16-bit word, where the init keeps that bit (`ROR`, `ROR`).
const SIGN: u16 = 0x8000;

/// Frames each glow colour set shows (`$A8:E87C`), and the wait the init and the turn set.
const GLOW_FRAMES: Frames = Frames(0x10);

/// The first of the four palette colours the glow writes: colour 9 (`$A8:E896`).
const GLOW_FIRST_COLOUR: usize = 9;

/// The glow's controls as the face block's AI sets them: the enemy graphics drawn hook and
/// the palette cycle's variables.
#[derive(Debug, PartialEq, Eq)]
pub struct Glow<'s> {
    /// The hook.
    pub hook: &'s mut GraphicsDrawnHook,
    /// The cycle.
    pub cycle: &'s mut EnemyPaletteCycle,
}

/// A side of the face block.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Side {
    /// Samus's X less its X is negative.
    Left,
    /// Zero or positive.
    Right,
}

impl Side {
    /// The side an X offset is on: its sign.
    const fn of(offset: u16) -> Self {
        if (offset & SIGN) == 0 {
            Self::Right
        } else {
            Self::Left
        }
    }

    /// The side's sign bit, as the init stores the side it never turns to.
    const fn sign(self) -> u16 {
        match self {
            Self::Left => SIGN,
            Self::Right => 0,
        }
    }
}

/// Point the slot's instruction list at `target`, run on this frame's instruction pass.
const fn start(common: &mut EnemyCommon, target: Target) {
    common.cursor.at = label(target);
    common.cursor.delay = Frames(1);
}

/// The face block's init AI (`$A8:E82E`): it faces forward, and its glow's hook is set.
///
/// Its second parameter becomes the sign of the side it never turns to, bit 0 set for left.
/// The glow's hook is the cycle with the morph ball and an `RTL` without, on its palette, 16
/// frames from its next colour set.
pub const fn init(
    common: &mut EnemyCommon,
    lists: &FaceBlockLists,
    samus: &Samus,
    glow: &mut Glow<'_>,
) {
    common.cursor.at = label(lists.facing_forward);
    *glow.hook = if samus.equipment.collected.contains(Item::MorphBall) {
        GraphicsDrawnHook::FaceBlockGlow
    } else {
        GraphicsDrawnHook::None
    };
    glow.cycle.palette = SpritePalette::of(common.palette);
    glow.cycle.timer = GLOW_FRAMES;
    let never = if (common.parameter_2 & NEVER_LEFT) == 0 {
        Side::Right
    } else {
        Side::Left
    };
    common.parameter_2 = never.sign();
}

/// The face block's main AI (`$A8:E8AE`): with the morph ball, it turns towards Samus once.
///
/// With the morph ball it sets the glow's hook on its palette. Until it has turned, once she
/// is within its range of rows it measures her X offset, and within its range of columns too
/// it turns her way, unless that is its never side; the turn restarts the glow's wait.
pub const fn main_ai(
    common: &mut EnemyCommon,
    block: &mut FaceBlock,
    samus: &Samus,
    lists: &FaceBlockLists,
    glow: &mut Glow<'_>,
) {
    if !samus.equipment.collected.contains(Item::MorphBall) {
        return;
    }
    *glow.hook = GraphicsDrawnHook::FaceBlockGlow;
    glow.cycle.palette = SpritePalette::of(common.palette);
    if block.activated {
        return;
    }
    let range = common.parameter_1;
    if !within(
        samus.motion.position.y.pixel(),
        common.position.y.pixels,
        range,
    ) {
        return;
    }
    block.samus_offset = samus
        .motion
        .position
        .x
        .pixel()
        .wrapping_sub(common.position.x.pixels);
    if !within(
        samus.motion.position.x.pixel(),
        common.position.x.pixels,
        range,
    ) {
        return;
    }
    let side = Side::of(block.samus_offset);
    // The compare is with the whole word: a second parameter the init didn't write turns
    // both ways.
    if side.sign() == common.parameter_2 {
        return;
    }
    start(
        common,
        match side {
            Side::Left => lists.turning_left,
            Side::Right => lists.turning_right,
        },
    );
    block.activated = true;
    glow.cycle.timer = GLOW_FRAMES;
}

/// The face block's glow, its enemy graphics drawn hook (`$A8:E86E`): every 16 frames the
/// next of the eight colour sets goes to colours 9 to 12 of its palette in the palette
/// buffer.
// open: the hook returns at once in a door transition (`$A8:E86E` reads the door transition
// flag for enemies); slice 0's Draw phase never runs in one.
pub fn glow(cycle: &mut EnemyPaletteCycle, palettes: &mut [Colour], colours: &GlowColours) {
    cycle.timer = Frames(cycle.timer.0.wrapping_sub(1));
    if cycle.timer != Frames(0) {
        return;
    }
    cycle.timer = GLOW_FRAMES;
    let first = cycle.palette.colour(GLOW_FIRST_COLOUR);
    let targets = palettes.get_mut(first..first.saturating_add(GLOW_SET_COLOURS));
    if let (Some(set), Some(targets)) = (colours.0.get(cycle.set.index()), targets) {
        for (target, &colour) in targets.iter_mut().zip(set) {
            *target = Colour(colour);
        }
    }
    cycle.set = cycle.set.next();
}

#[cfg(test)]
mod tests;
