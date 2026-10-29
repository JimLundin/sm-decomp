//! Kraid's tables in bank $A7: the body's hitbox, the boundary his fingernails
//! rebound from, the foot's walking targets and the health-based palettes.
pub use crate::table::LengthError;
use crate::table::Rows;

/// $A7:B161: eight bands, the last a sentinel whose bottom is $8000.
pub const BODY_HITBOX_ADDRESS: u32 = 0xa7_b161;
pub const BODY_HITBOX_ROWS: usize = 8;
pub const BODY_HITBOX_BYTES: usize = BODY_HITBOX_ROWS * 4;

/// One band of the body as offsets from Kraid's position. The next band's
/// bottom is this band's top.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BodyHitboxBand {
    pub bottom: u16,
    pub left: u16,
}

pub type BodyHitbox = Rows<BodyHitboxBand, BODY_HITBOX_ROWS>;

pub fn import_body_hitbox(bytes: &[u8]) -> Result<BodyHitbox, LengthError> {
    Rows::import(bytes, |[bottom, left]| BodyHitboxBand { bottom, left })
}

/// $A7:BF1D, read by the fingernail fire routine.
pub const FINGERNAIL_BOUNDARY_ADDRESS: u32 = 0xa7_bf1d;
pub const FINGERNAIL_BOUNDARY_ROWS: usize = 4;
pub const FINGERNAIL_BOUNDARY_BYTES: usize = FINGERNAIL_BOUNDARY_ROWS * 4;

/// One step of the body boundary a fingernail rebounds from, as offsets from
/// Kraid's position.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FingernailBoundary {
    pub left: u16,
    pub top: u16,
}

pub type FingernailBoundaries = Rows<FingernailBoundary, FINGERNAIL_BOUNDARY_ROWS>;

pub fn import_fingernail_boundaries(bytes: &[u8]) -> Result<FingernailBoundaries, LengthError> {
    Rows::import(bytes, |[left, top]| FingernailBoundary { left, top })
}

/// $A7:BA7D: six X positions, the second the default.
pub const FOOT_TARGET_ADDRESS: u32 = 0xa7_ba7d;
pub const FOOT_TARGET_ROWS: usize = 6;
pub const FOOT_TARGET_BYTES: usize = FOOT_TARGET_ROWS * 4;

/// An X position Kraid can stand at and the movement data chosen from there.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FootTarget {
    pub x_position: u16,
    /// Bank $A7 pointer to the thinking-timer and target-X rows.
    pub movement: u16,
}

pub type FootTargets = Rows<FootTarget, FOOT_TARGET_ROWS>;

pub fn import_foot_targets(bytes: &[u8]) -> Result<FootTargets, LengthError> {
    Rows::import(bytes, |[x_position, movement]| FootTarget {
        x_position,
        movement,
    })
}

/// Sixteen BGR555 colours.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Palette {
    pub colors: [u16; 16],
}

/// $A7:B3D3, BG palette 7: the odd hurt frame, eight health eighths from the
/// lowest, then death.
pub const BG_PALETTE_ADDRESS: u32 = 0xa7_b3d3;
pub const BG_PALETTE_ROWS: usize = 10;
pub const BG_PALETTE_BYTES: usize = BG_PALETTE_ROWS * 32;

/// $A7:B513, sprite palette 7: the odd hurt frame then eight health eighths.
pub const SPRITE_PALETTE_ADDRESS: u32 = 0xa7_b513;
pub const SPRITE_PALETTE_ROWS: usize = 9;
pub const SPRITE_PALETTE_BYTES: usize = SPRITE_PALETTE_ROWS * 32;

pub type BgPalettes = Rows<Palette, BG_PALETTE_ROWS>;
pub type SpritePalettes = Rows<Palette, SPRITE_PALETTE_ROWS>;

pub fn import_bg_palettes(bytes: &[u8]) -> Result<BgPalettes, LengthError> {
    Rows::import(bytes, |colors| Palette { colors })
}

pub fn import_sprite_palettes(bytes: &[u8]) -> Result<SpritePalettes, LengthError> {
    Rows::import(bytes, |colors| Palette { colors })
}
