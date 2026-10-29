//! Ridley's tables in bank $A6: hover and pogo targets, feet offsets, lunge
//! checks, tail angle limits, wing and tail tip spritemaps, and the Ceres
//! escape "emergency" text glow.
use crate::table::{LengthError, Rows};

/// Rows indexed by Ridley's facing direction: left, forward, right.
pub const FACINGS: usize = 3;

/// $A6:B60D: hovering to the wall behind Ridley, the target X per facing.
pub const HOVER_BEHIND_ADDRESS: u32 = 0xa6_b60d;
/// $A6:B63B: hovering to the wall in front of Ridley, the target X per facing.
pub const HOVER_IN_FRONT_ADDRESS: u32 = 0xa6_b63b;
/// $A6:B6C8: the pogo's target X per facing.
pub const POGO_TARGET_ADDRESS: u32 = 0xa6_b6c8;
/// $A6:B9D5: the feet's X offset per facing.
pub const FEET_X_ADDRESS: u32 = 0xa6_b9d5;
/// $A6:B9DB: the feet's Y offset per feet distance index.
pub const FEET_Y_ADDRESS: u32 = 0xa6_b9db;
pub const FEET_DISTANCES: usize = 3;
/// $A6:BB48: the lunge's facing sign, compared with Ridley's X minus Samus's.
pub const LUNGE_SIDE_ADDRESS: u32 = 0xa6_bb48;
/// $A6:BB4E: the lunge's acceleration factor per acceleration index.
pub const LUNGE_ACCELERATION_ADDRESS: u32 = 0xa6_bb4e;
pub const LUNGE_ACCELERATIONS: usize = 4;

/// $A6:CC12: the tail's angle limits per facing, minimum clockwise angles then
/// maximum anticlockwise angles ($CC18).
pub const TAIL_ANGLE_ADDRESS: u32 = 0xa6_cc12;
pub const TAIL_ANGLE_BYTES: usize = FACINGS * 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TailAngleLimits {
    pub min_clockwise: u16,
    pub max_anticlockwise: u16,
}

pub type TailAngles = Rows<TailAngleLimits, FACINGS>;

pub fn import_tail_angles(bytes: &[u8]) -> Result<TailAngles, LengthError> {
    Rows::import_columns(bytes, |[min_clockwise, max_anticlockwise]| {
        TailAngleLimits {
            min_clockwise,
            max_anticlockwise,
        }
    })
}

/// $A6:DB02: wing spritemap pointers, ten animation frames facing left then
/// ten facing right.
pub const WING_SPRITEMAP_ADDRESS: u32 = 0xa6_db02;
pub const WING_SPRITEMAPS: usize = 20;

/// $A6:DCBA: tail tip spritemap pointers by angle, sixteen 10h-wide sectors
/// from down.
pub const TAIL_TIP_SPRITEMAP_ADDRESS: u32 = 0xa6_dcba;
pub const TAIL_TIP_SPRITEMAPS: usize = 16;

/// $A6:C1DF: sixteen steps of the Ceres escape "emergency" text glow, BG1/2
/// palette 6 colours 1..3.
pub const EMERGENCY_GLOW_ADDRESS: u32 = 0xa6_c1df;
pub const EMERGENCY_GLOW_COLOURS: usize = 3;
pub const EMERGENCY_GLOW_STEPS: usize = 16;
