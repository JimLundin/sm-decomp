//! Physical operand binding and retained live samples for borrowed Random completion.

use super::borrowed_preparation::{
    CompletedSeedProducts, FirstSeedProduct, RandomSeed, SecondSeedProductLow,
};
use canonical::Canonical;
use data::timing::{DeviceWrite, Publication, Sample, Template, TemplateEffect};

/// Checked immutable multiplier imported at the content edge.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImportedRandomMultiplier(pub u8);

/// Complete current-seed envelope for the two actual hardware requests.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SeedRequests {
    /// Original seed read by both current requests.
    pub dependency: RandomSeed,
    /// Independently checked immutable operand.
    pub multiplier: ImportedRandomMultiplier,
}

/// Invalid current dependency, physical request shape or live sample sequence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// Current original seed differs from the qualified request dependency.
    Seed,
    /// Physical owner does not contain exactly the two named multiplication requests.
    Profile,
    /// The live result sequence is incomplete, duplicated or reordered.
    Samples,
}

impl SeedRequests {
    /// Bind both requests from the current original seed, without generating any result.
    /// # Errors
    /// Rejects stale seed and incompatible publication/sample namespaces before execution.
    pub fn bind(
        self,
        current: RandomSeed,
        template: &Template,
    ) -> Result<data::timing::Profile, Fault> {
        if current != self.dependency {
            return Err(Fault::Seed);
        }
        let [low, high] = current.0.to_le_bytes();
        let mut values = [low, self.multiplier.0, high, self.multiplier.0].into_iter();
        let mut writes = Vec::new();
        let mut samples = Vec::new();
        for run in &template.runs {
            for _ in 0..run.count {
                match run.effect {
                    TemplateEffect::None => {}
                    TemplateEffect::Sample(sample) => samples.push(sample),
                    TemplateEffect::Publish(port) => {
                        let value = values.next().ok_or(Fault::Profile)?;
                        let write = DeviceWrite::from_cpu_port(port.0, value)
                            .map_err(|_| Fault::Profile)?;
                        let valid = match writes.len() {
                            0 | 2 => matches!(write.effect, Publication::Multiplicand(_)),
                            1 | 3 => matches!(write.effect, Publication::Multiplier(_)),
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
            || samples != [Sample::ProductLow, Sample::ProductHigh, Sample::ProductLow]
        {
            return Err(Fault::Profile);
        }
        template.bind(&writes).map_err(|_| Fault::Profile)
    }
}

/// Live result bytes retained across hardware pauses and interrupt suspension.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Samples {
    /// No result has been sampled.
    #[default]
    Empty,
    /// First request's low byte has been sampled.
    FirstLow(u8),
    /// Complete first result, still independent of the second request.
    First(FirstSeedProduct),
    /// Both actual requests have supplied their required result bytes.
    Complete(CompletedSeedProducts),
}

impl Samples {
    /// Retain one actual multiplication sample, rejecting duplicates and reordering.
    /// # Errors
    /// The current live sample is outside the qualified request sequence.
    pub const fn accept(&mut self, device: Sample, value: u8) -> Result<(), Fault> {
        *self = match (*self, device) {
            (Self::Empty, Sample::ProductLow) => Self::FirstLow(value),
            (Self::FirstLow(low), Sample::ProductHigh) => {
                Self::First(FirstSeedProduct(u16::from_le_bytes([low, value])))
            }
            (Self::First(first), Sample::ProductLow) => Self::Complete(CompletedSeedProducts {
                first,
                second_low: SecondSeedProductLow(value),
            }),
            _ => return Err(Fault::Samples),
        };
        Ok(())
    }

    /// Complete actual hardware products, never source-required product predicates.
    /// # Errors
    /// At least one required live sample has not completed.
    pub const fn completed(self) -> Result<CompletedSeedProducts, Fault> {
        match self {
            Self::Complete(products) => Ok(products),
            Self::Empty | Self::FirstLow(_) | Self::First(_) => Err(Fault::Samples),
        }
    }
}
