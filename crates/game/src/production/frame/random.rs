//! Physical Random requests and retained live multiplication samples.

use canonical::{
    Canonical,
    model::{Model, Pinned},
};
use data::timing::{DeviceWrite, Publication, Sample, Template, TemplateEffect};
use game_random::{RandomNumber, RandomSequence};

pub(super) struct RandomPin;

impl Pinned for RandomPin {
    type State = RandomSequence;
    const NAME: &'static str = "game-random";
    const SHAPE: u64 = 0xee18_3037_285b_17ec;
}

/// Current shared sequence, independently retained between physical draws.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Sequence(Model<RandomPin>);

impl Sequence {
    /// Admit the actual current number, including zero.
    #[must_use]
    pub const fn new(number: RandomSeed) -> Self {
        Self(Model(RandomSequence {
            number: RandomNumber(number.0),
        }))
    }

    /// Read the number without advancing.
    #[must_use]
    pub const fn current(&self) -> RandomSeed {
        RandomSeed(self.0.0.number.0)
    }

    pub(super) fn draw(&mut self) {
        game_random::advance_random(&mut self.0.0);
    }
}

/// Shared current number supplied to the hardware requests.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct RandomSeed(pub u16);

/// Complete first product, retained across the second multiplication.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FirstSeedProduct(pub u16);

/// Ordered live bytes sampled from both hardware requests.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompletedSeedProducts {
    /// Low and high bytes sampled from the first request.
    first: FirstSeedProduct,
    /// Low byte sampled from the second request.
    second_low: u8,
}

/// Checked immutable multiplier imported at the content edge.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImportedRandomMultiplier(pub u8);

/// Complete current-seed envelope for the two actual hardware requests.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SeedRequests {
    /// Independently checked immutable operand.
    pub multiplier: ImportedRandomMultiplier,
}

/// Invalid current dependency, physical request shape or live sample sequence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// Physical owner does not contain exactly the two named multiplication requests.
    Profile,
    /// The live result sequence is incomplete, out of order or inconsistent with the operands.
    Samples,
}

impl SeedRequests {
    /// Bind both requests from the current original seed, without generating any result.
    /// # Errors
    /// Rejects incompatible publication and sample namespaces before execution.
    pub fn bind(
        self,
        current: RandomSeed,
        template: &Template,
    ) -> Result<data::timing::Profile, Fault> {
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

    /// Validate all sampled product bytes against the operands bound to the request.
    /// # Errors
    /// The physical samples are incomplete or inconsistent with their operands.
    pub fn validate_samples(self, requested: RandomSeed, samples: Samples) -> Result<(), Fault> {
        let products = samples.completed()?;
        let [low, high] = requested.0.to_le_bytes();
        let multiplier = u16::from(self.multiplier.0);
        // Each operand is a byte, so its product fits in a word.
        let first = u16::from(low).wrapping_mul(multiplier);
        let [second_low, _] = u16::from(high).wrapping_mul(multiplier).to_le_bytes();
        if products.first.0 != first || products.second_low != second_low {
            return Err(Fault::Samples);
        }
        Ok(())
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
                second_low: value,
            }),
            _ => return Err(Fault::Samples),
        };
        Ok(())
    }

    /// Complete actual hardware products, never source-required product predicates.
    /// # Errors
    /// At least one required live sample has not completed.
    const fn completed(self) -> Result<CompletedSeedProducts, Fault> {
        match self {
            Self::Complete(products) => Ok(products),
            Self::Empty | Self::FirstLow(_) | Self::First(_) => Err(Fault::Samples),
        }
    }
}
