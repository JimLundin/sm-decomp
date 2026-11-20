use super::{Fault, Receipt};
use canonical::Canonical;
use data::timing::{DeviceWrite, Publication, Sample, Template, TemplateEffect};
use types::Parts;

/// Actual operands for the two current vertical scroll-zone multiplications.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Requests {
    /// Screen row of the actual viewport before vertical settling.
    pub row: u8,
    /// Complete current room width, checked for the hardware operand's range.
    pub width: u8,
}

impl Requests {
    /// Derive operands from current camera and room owners, never from observed products.
    /// # Errors
    /// Current room width cannot be represented by the reached physical multiplier.
    pub fn from_receipt(receipt: &Receipt) -> Result<Self, Fault> {
        let [_, row] = receipt.initial.0.position().y.pixel().0.to_le_bytes();
        let [width, _] = receipt.inputs.room.size;
        Ok(Self {
            row,
            width: u8::try_from(width).map_err(|_| Fault::Profile)?,
        })
    }

    /// Product expected from these actual operands, only for checking live completion.
    #[must_use]
    pub fn product(self) -> u16 {
        u16::from(self.row).wrapping_mul(u16::from(self.width))
    }

    /// Publish both complete requests from actual current operands.
    /// # Errors
    /// Missing/extra publications, incompatible ports or wrong result sample order.
    pub fn bind(self, template: &Template) -> Result<data::timing::Profile, Fault> {
        let mut values = [self.row, self.width, self.row, self.width].into_iter();
        let mut writes = Vec::new();
        let mut samples = Vec::new();
        for run in &template.runs {
            for _ in 0..run.count {
                match run.effect {
                    TemplateEffect::None => {}
                    TemplateEffect::Sample(sample) => {
                        let expected = match (writes.len(), samples.len()) {
                            (2, 0) | (4, 2) => Sample::ProductLow,
                            (2, 1) | (4, 3) => Sample::ProductHigh,
                            _ => return Err(Fault::Profile),
                        };
                        if sample != expected {
                            return Err(Fault::Profile);
                        }
                        samples.push(sample);
                    }
                    TemplateEffect::Publish(port) => {
                        let value = values.next().ok_or(Fault::Profile)?;
                        let write = DeviceWrite::from_cpu_port(port.0, value)
                            .map_err(|_| Fault::Profile)?;
                        let valid = match (writes.len(), samples.len()) {
                            (0, 0) | (2, 2) => matches!(write.effect, Publication::Multiplicand(_)),
                            (1, 0) | (3, 2) => matches!(write.effect, Publication::Multiplier(_)),
                            _ => false,
                        };
                        if !valid {
                            return Err(Fault::Profile);
                        }
                        writes.push(write);
                    }
                }
            }
        }
        if values.next().is_some()
            || samples
                != [
                    Sample::ProductLow,
                    Sample::ProductHigh,
                    Sample::ProductLow,
                    Sample::ProductHigh,
                ]
        {
            return Err(Fault::Profile);
        }
        template.bind(&writes).map_err(|_| Fault::Profile)
    }
}

/// Complete live Camera sample continuation, checkpointable between either product's bytes.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Samples {
    /// No actual product byte has been read.
    #[default]
    Empty,
    /// Actual first product's low byte.
    FirstLow(u8),
    /// Complete first actual product.
    First(u16),
    /// Actual second low byte alongside the complete first product.
    SecondLow {
        /// Complete first product.
        first: u16,
        /// Actual second product's low byte.
        low: u8,
    },
    /// Both complete actual products.
    Complete([u16; 2]),
}

impl Samples {
    /// Retain one actual result without accepting duplicated or reordered bytes.
    /// # Errors
    /// Current sample is outside the reached two-product sequence.
    pub const fn accept(&mut self, device: Sample, value: u8) -> Result<(), Fault> {
        *self = match (*self, device) {
            (Self::Empty, Sample::ProductLow) => Self::FirstLow(value),
            (Self::FirstLow(low), Sample::ProductHigh) => {
                Self::First(u16::from_le_bytes([low, value]))
            }
            (Self::First(first), Sample::ProductLow) => Self::SecondLow { first, low: value },
            (Self::SecondLow { first, low }, Sample::ProductHigh) => {
                Self::Complete([first, u16::from_le_bytes([low, value])])
            }
            _ => return Err(Fault::Samples),
        };
        Ok(())
    }

    /// Actual products retained at full completion; an incomplete sequence never fills zeros.
    /// # Errors
    /// Either product is incomplete.
    pub const fn completed(self) -> Result<[u16; 2], Fault> {
        match self {
            Self::Complete(products) => Ok(products),
            Self::Empty | Self::FirstLow(_) | Self::First(_) | Self::SecondLow { .. } => {
                Err(Fault::Samples)
            }
        }
    }
}
