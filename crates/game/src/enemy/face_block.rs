//! The Blue Brinstar face block's AI (D14), bank $A8 log
//! (<https://patrickjohnston.org/bank/A8>, `$A8:E82E` to `$A8:E91B`).
//!
//! The face block faces forward until Samus has the morph ball and comes within its range,
//! its first parameter, on both axes; then it turns to look at her once, unless she is on
//! the side its second parameter says it never turns to.

use data::enemy::face_block::FaceBlockLists;
use data::enemy::program::Target;

use super::eye::within;
use crate::enemy::instructions::label;
use crate::state::room::{EnemyCommon, FaceBlock};
use crate::state::samus::{Item, Samus};
use crate::state::slots::Frames;

/// The second parameter's bit the init keeps: set, the face block never turns left
/// (`$A8:E862`).
const NEVER_LEFT: u16 = 0x0001;

/// The sign bit of a 16-bit word, where the init keeps that bit (`ROR`, `ROR`).
const SIGN: u16 = 0x8000;

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

/// The face block's init AI (`$A8:E82E`): it faces forward, and its second parameter
/// becomes the sign of the side it never turns to, bit 0 set for left.
// open: the enemy graphics drawn hook it sets, the palette cycle (`$A8:E86E`) with the morph
// ball and an `RTL` without, and the cycle's palette index and timer, join with the palette
// effects.
pub const fn init(common: &mut EnemyCommon, lists: &FaceBlockLists) {
    common.cursor.at = label(lists.facing_forward);
    let never = if (common.parameter_2 & NEVER_LEFT) == 0 {
        Side::Right
    } else {
        Side::Left
    };
    common.parameter_2 = never.sign();
}

/// The face block's main AI (`$A8:E8AE`): with the morph ball, it turns towards Samus once.
///
/// Until it has turned, once she is within its range of rows it measures her X offset, and
/// within its range of columns too it turns her way, unless that is its never side.
// open: the palette cycle's hook, palette index and timer it sets join with the palette
// effects.
pub const fn main_ai(
    common: &mut EnemyCommon,
    block: &mut FaceBlock,
    samus: &Samus,
    lists: &FaceBlockLists,
) {
    if !samus.equipment.collected.contains(Item::MorphBall) || block.activated {
        return;
    }
    let range = common.parameter_1;
    if !within(samus.position.y.pixels, common.position.y.pixels, range) {
        return;
    }
    block.samus_offset = samus
        .position
        .x
        .pixels
        .wrapping_sub(common.position.x.pixels);
    if !within(samus.position.x.pixels, common.position.x.pixels, range) {
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
}

#[cfg(test)]
mod tests;
