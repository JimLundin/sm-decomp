//! The Morph Ball Room eye's AI (D14), bank $A8 log (<https://patrickjohnston.org/bank/A8>,
//! `$A8:9058` to `$A8:91DC`).
//!
//! The eye is two parts: the eye, which waits closed until Samus has the morph ball and comes
//! near, opens, follows her with its beam while she stays near and closes when she leaves;
//! and its mount, which shows one frame and does nothing.

use data::enemy::eye::{EyeLists, Facing, MountSide};
use data::enemy::program::Target;

use crate::enemy::instructions::label;
use crate::state::room::{EnemyCommon, EnemyProperty, EyePhase, MorphBallEye};
use crate::state::samus::{Item, Samus};
use crate::state::slots::Frames;

/// How far Samus may be from the eye, in pixels: the `$A8:9050` table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Proximity {
    /// Pixel columns.
    columns: u16,
    /// Pixel rows.
    rows: u16,
}

/// Near enough to open (`$A8:9050`, `$A8:9054`).
const ACTIVATE: Proximity = Proximity {
    columns: 0x80,
    rows: 0x80,
};

/// Near enough to stay open (`$A8:9052`, `$A8:9056`).
const DEACTIVATE: Proximity = Proximity {
    columns: 0xb0,
    rows: 0x80,
};

/// Frames opening or closing takes (`$A8:90F1`, `$A8:9160`).
const TURN_FRAMES: Frames = Frames(0x20);

/// How far a mount sits from the eye's placed centre (`$A8:90CA`, `$A8:90D2`).
const MOUNT_OFFSET: u16 = 8;

/// The second parameter's bit that makes the part a mount, and the nybble naming its side
/// (`$A8:9058`).
const MOUNT: u16 = 0x8000;
const MOUNT_SIDE: u16 = 0x000f;

/// A mount's second parameter names a side past the init's tables; the original reads the
/// words after them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnknownMountSide(pub u16);

/// Whether a 16-bit compare's difference is negative: the original's `BMI`.
const fn negative(value: u16) -> bool {
    (value & 0x8000) != 0
}

/// The magnitude of a signed 16-bit word, `$8000` staying `$8000`.
const fn magnitude(value: u16) -> u16 {
    if negative(value) {
        value.wrapping_neg()
    } else {
        value
    }
}

/// Whether Samus is within `distance` of `centre` on one axis: her offset's magnitude less
/// the distance is negative (bank $A0 log, `$A0:AEED` rows, `$A0:AF0B` columns).
pub(crate) const fn within(samus: u16, centre: u16, distance: u16) -> bool {
    negative(magnitude(samus.wrapping_sub(centre)).wrapping_sub(distance))
}

/// Whether Samus is within `proximity` of the eye, rows then columns.
const fn near(common: &EnemyCommon, samus: &Samus, proximity: Proximity) -> bool {
    within(
        samus.position.y.pixels,
        common.position.y.pixels,
        proximity.rows,
    ) && within(
        samus.position.x.pixels,
        common.position.x.pixels,
        proximity.columns,
    )
}

/// The angle of the offset (`x`, `y`): 0 up, 40h right, 80h down, C0h left (`$A0:C0AE`).
///
/// Bank $A0 log: the smaller magnitude over the larger's low byte, through the hardware
/// divider ($FFFF for a zero divisor); an eighth of it is how far past the nearest axis.
#[must_use]
pub fn angle(x: u16, y: u16) -> u8 {
    let (across, down) = (magnitude(x), magnitude(y));
    let wide = down < across;
    let (minor, major) = if wide { (down, across) } else { (across, down) };
    let [divisor, _] = major.to_le_bytes();
    let quotient = (minor << 8)
        .checked_div(u16::from(divisor))
        .unwrap_or(0xffff);
    let [past, _] = (quotient >> 3).to_le_bytes();
    match (wide, negative(x), negative(y)) {
        (true, false, false) => 0x40_u8.wrapping_add(past),
        (true, false, true) => 0x40_u8.wrapping_sub(past),
        (true, true, false) => 0xc0_u8.wrapping_sub(past),
        (true, true, true) => 0xc0_u8.wrapping_add(past),
        (false, false, false) => 0x80_u8.wrapping_sub(past),
        (false, false, true) => past,
        (false, true, false) => 0x80_u8.wrapping_add(past),
        (false, true, true) => past.wrapping_neg(),
    }
}

/// The angle from the eye to Samus (`$A8:912E`, `$A8:91A2`).
fn aim(common: &EnemyCommon, samus: &Samus) -> u16 {
    u16::from(angle(
        samus
            .position
            .x
            .pixels
            .wrapping_sub(common.position.x.pixels),
        samus
            .position
            .y
            .pixels
            .wrapping_sub(common.position.y.pixels),
    ))
}

/// The way the eye faces: its first parameter's bit 0.
const fn facing(common: &EnemyCommon) -> Facing {
    if (common.parameter_1 & 1) == 0 {
        Facing::Right
    } else {
        Facing::Left
    }
}

/// Point the slot's instruction list at `target`.
const fn start(common: &mut EnemyCommon, target: Target) {
    common.cursor.at = label(target);
}

/// Count the function timer down; whether it ran out, reaching zero or below (`DEC`, `BEQ`,
/// `BPL`).
const fn ran_out(eye: &mut MorphBallEye) -> bool {
    let left = eye.function_timer.0.wrapping_sub(1);
    eye.function_timer = Frames(left);
    left == 0 || negative(left)
}

/// The eye's init AI (`$A8:9058`).
///
/// It processes instructions, shows nothing and runs its list on the next frame; a mount
/// moves 8 pixels to its side and shows its side's frame, an eye waits closed facing its way.
///
/// # Errors
///
/// [`UnknownMountSide`] for a mount whose side nybble is past the init's tables.
// open: a mount fills the beam's window table, which the beam (an HDMA object) reads; it
// joins with the beam.
pub fn init(
    common: &mut EnemyCommon,
    eye: &mut MorphBallEye,
    lists: &EyeLists,
) -> Result<(), UnknownMountSide> {
    common
        .properties
        .insert(EnemyProperty::ProcessesInstructions);
    common.spritemap = lists.empty;
    common.cursor.delay = Frames(1);
    common.timer = 0;
    if (common.parameter_2 & MOUNT) == 0 {
        eye.phase = EyePhase::WaitingForSamus;
        start(common, lists.closed.get(facing(common)));
        return Ok(());
    }
    let side = MountSide::try_from(common.parameter_2 & MOUNT_SIDE).map_err(UnknownMountSide)?;
    let (x, y) = (&mut common.position.x.pixels, &mut common.position.y.pixels);
    match side {
        MountSide::Left => *x = x.wrapping_sub(MOUNT_OFFSET),
        MountSide::Right => *x = x.wrapping_add(MOUNT_OFFSET),
        MountSide::Up => *y = y.wrapping_sub(MOUNT_OFFSET),
        MountSide::Down => *y = y.wrapping_add(MOUNT_OFFSET),
    }
    eye.phase = EyePhase::Mount;
    start(common, lists.mount.get(side));
    Ok(())
}

/// The eye's main AI (`$A8:90E2`): nothing until Samus has collected the morph ball, then
/// its phase's function.
// open: the ray's sound (17h) and its silence (71h, library 2), and the beam's HDMA object
// the opening spawns, join with sound (C11) and the beam.
pub fn main_ai(common: &mut EnemyCommon, eye: &mut MorphBallEye, samus: &Samus, lists: &EyeLists) {
    if !samus.equipment.collected.contains(Item::MorphBall) {
        return;
    }
    match eye.phase {
        EyePhase::WaitingForSamus => {
            if near(common, samus, ACTIVATE) {
                eye.function_timer = TURN_FRAMES;
                common.cursor.delay = Frames(1);
                start(common, lists.activating.get(facing(common)));
                eye.phase = EyePhase::Activating;
            }
        }
        EyePhase::Activating => {
            if ran_out(eye) {
                eye.phase = EyePhase::Active;
                eye.angle = aim(common, samus);
            }
        }
        EyePhase::Active => {
            if near(common, samus, DEACTIVATE) {
                eye.angle = aim(common, samus);
                let frame = usize::from((eye.angle & 0xf0) >> 4);
                if let Some(&target) = lists.active.get(frame) {
                    start(common, target);
                }
            } else {
                eye.activated = false;
                eye.function_timer = TURN_FRAMES;
                start(common, lists.deactivating.get(facing(common)));
                eye.phase = EyePhase::Deactivating;
            }
            common.cursor.delay = Frames(1);
        }
        EyePhase::Deactivating => {
            if ran_out(eye) {
                eye.phase = EyePhase::WaitingForSamus;
            }
        }
        EyePhase::Mount => {}
    }
}

#[cfg(test)]
mod tests;
