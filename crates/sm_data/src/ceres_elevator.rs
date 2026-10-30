//! The Ceres elevator room's rotation at $89:AD5F: one three-word step per
//! frame delay. Each step sets the Mode 7 matrix to [[A, B], [-B, A]].

/// LoROM address of the first step.
pub const ADDRESS: u32 = 0x89_ad5f;
/// Steps the rotation visits: 0 to 67 going out, 68 down to 1 coming back.
pub const STEPS: usize = 69;
pub const BYTES: usize = STEPS * 6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RotationStep {
    /// Frames until the next step.
    pub frames: u16,
    pub mode7_b: u16,
    pub mode7_a: u16,
}

pub use crate::table::LengthError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rotation {
    steps: Box<[RotationStep; STEPS]>,
}

impl Rotation {
    pub fn import(bytes: &[u8]) -> Result<Self, LengthError> {
        LengthError::check(bytes, BYTES)?;
        let word = |at: usize| u16::from_le_bytes([bytes[at], bytes[at + 1]]);
        Ok(Self {
            steps: Box::new(core::array::from_fn(|step| RotationStep {
                frames: word(6 * step),
                mode7_b: word(6 * step + 2),
                mode7_a: word(6 * step + 4),
            })),
        })
    }

    pub fn step(&self, index: usize) -> Option<RotationStep> {
        self.steps.get(index).copied()
    }
}
