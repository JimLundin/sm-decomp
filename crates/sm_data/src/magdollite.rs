//! Magdollite (Norfair lava man) tables in bank $A8: its palettes and the
//! arm growth steps.
pub use crate::table::LengthError;
use crate::table::Rows;

/// $A8:AC1C: the enemy palette and three more glow frames, up to $A8:AC9C.
pub const PALETTE_ADDRESS: u32 = 0xa8_ac1c;
pub const PALETTES: usize = 4;
pub const PALETTE_BYTES: usize = 32 * PALETTES;

/// Sixteen BGR555 colours.
pub type Palette = [u16; 16];
pub type Palettes = Rows<Palette, PALETTES>;

pub fn import_palettes(bytes: &[u8]) -> Result<Palettes, LengthError> {
    Rows::import(bytes, |colors| colors)
}

/// $A8:AF55: the arm growth table, followed by its parallel instruction list
/// and Y offset tables at $A8:AF67 and $A8:AF79.
pub const ARM_GROWTH_ADDRESS: u32 = 0xa8_af55;
pub const ARM_INSTRUCTION_ADDRESS: u32 = 0xa8_af67;
pub const ARM_Y_OFFSET_ADDRESS: u32 = 0xa8_af79;
pub const ARM_STEPS: usize = 9;
pub const ARM_BYTES: usize = 3 * 2 * ARM_STEPS;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArmStep {
    /// Height the arm must reach before this step.
    pub growth: u16,
    /// Bank $A8 instruction list for this step.
    pub instruction_list: u16,
    /// The head's height above the base.
    pub y_offset: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArmSteps {
    steps: [ArmStep; ARM_STEPS],
}

impl ArmSteps {
    pub fn import(bytes: &[u8]) -> Result<Self, LengthError> {
        if bytes.len() != ARM_BYTES {
            return Err(LengthError(bytes.len()));
        }
        let column = |table: usize, step: usize| {
            let at = 2 * (ARM_STEPS * table + step);
            u16::from_le_bytes([bytes[at], bytes[at + 1]])
        };
        Ok(Self {
            steps: core::array::from_fn(|step| ArmStep {
                growth: column(0, step),
                instruction_list: column(1, step),
                y_offset: column(2, step),
            }),
        })
    }

    pub fn step(&self, index: usize) -> Option<ArmStep> {
        self.steps.get(index).copied()
    }
}
