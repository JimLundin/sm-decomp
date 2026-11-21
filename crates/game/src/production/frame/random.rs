//! Physical Random requests and retained live multiplication samples.

use canonical::{
    Canonical,
    model::{Model, Pinned},
};
use data::timing::{
    DeviceEffect, DeviceWrite, MULTIPLICAND_PORT, MULTIPLIER_PORT, PublicationPort, Sample,
    Template, TemplateEffect,
};
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
pub struct ImportedRandomMultiplier(u8);

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
    /// Imported operand would disagree with the fixed sequence rule.
    Multiplier,
    /// The live result sequence is incomplete, out of order or inconsistent with the operands.
    Samples,
}

const FIXED_RANDOM_MULTIPLIER: u8 = 5;

impl ImportedRandomMultiplier {
    /// Check that an imported operand matches the fixed random draw.
    /// # Errors
    /// Rejects a coefficient incompatible with the shared random sequence.
    pub const fn new(value: u8) -> Result<Self, Fault> {
        if value == FIXED_RANDOM_MULTIPLIER {
            Ok(Self(value))
        } else {
            Err(Fault::Multiplier)
        }
    }

    pub(in crate::production) const fn checked(self) -> Result<u8, Fault> {
        match Self::new(self.0) {
            Ok(_) => Ok(self.0),
            Err(fault) => Err(fault),
        }
    }
}

#[derive(PartialEq, Eq)]
enum Request {
    Write(PublicationPort, u8),
    Read(Sample),
}

const fn requests(current: RandomSeed) -> [Request; 7] {
    let [low, high] = current.0.to_le_bytes();
    [
        Request::Write(MULTIPLICAND_PORT, low),
        Request::Write(MULTIPLIER_PORT, FIXED_RANDOM_MULTIPLIER),
        Request::Read(Sample::ProductLow),
        Request::Read(Sample::ProductHigh),
        Request::Write(MULTIPLICAND_PORT, high),
        Request::Write(MULTIPLIER_PORT, FIXED_RANDOM_MULTIPLIER),
        Request::Read(Sample::ProductLow),
    ]
}

pub(super) fn validate_profile(
    current: RandomSeed,
    profile: &data::timing::Profile,
) -> Result<(), Fault> {
    let mut expected = requests(current).into_iter();
    for run in &profile.runs {
        let actual = match &run.exposure.effect {
            DeviceEffect::None => continue,
            DeviceEffect::Publish(write) => {
                write.validate().map_err(|_| Fault::Profile)?;
                Request::Write(write.port, write.value)
            }
            DeviceEffect::Sample(sample) => Request::Read(*sample),
        };
        if run.count != 1 || expected.next() != Some(actual) {
            return Err(Fault::Profile);
        }
    }
    if expected.next().is_some() {
        return Err(Fault::Profile);
    }
    Ok(())
}

impl SeedRequests {
    /// Bind both requests from the current original seed, without generating any result.
    /// # Errors
    /// Rejects an incompatible coefficient or physical request sequence before execution.
    pub fn bind(
        self,
        current: RandomSeed,
        template: &Template,
    ) -> Result<data::timing::Profile, Fault> {
        self.multiplier.checked()?;
        let mut expected = requests(current).into_iter();
        let mut writes = Vec::new();
        for run in &template.runs {
            if run.effect == TemplateEffect::None {
                continue;
            }
            if run.count != 1 {
                return Err(Fault::Profile);
            }
            match (run.effect, expected.next()) {
                (TemplateEffect::Sample(actual), Some(Request::Read(sample)))
                    if actual == sample => {}
                (TemplateEffect::Publish(actual), Some(Request::Write(port, value)))
                    if actual == port =>
                {
                    writes.push(
                        DeviceWrite::from_cpu_port(port.0, value).map_err(|_| Fault::Profile)?,
                    );
                }
                _ => return Err(Fault::Profile),
            }
        }
        if expected.next().is_some() {
            return Err(Fault::Profile);
        }
        template.bind(&writes).map_err(|_| Fault::Profile)
    }

    /// Validate all sampled product bytes against the operands bound to the request.
    /// # Errors
    /// The coefficient is incompatible or the samples disagree with the operands.
    pub fn validate_samples(self, requested: RandomSeed, samples: Samples) -> Result<(), Fault> {
        let products = samples.completed()?;
        let [low, high] = requested.0.to_le_bytes();
        let multiplier = u16::from(self.multiplier.checked()?);
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

// These controls intentionally use equal seed bytes so a byte-value check cannot detect reordering.
#[cfg(test)]
mod tests {
    use super::*;
    use data::timing::{Cycle, ExposureKind, TemplateRun};

    const SEED: RandomSeed = RandomSeed(0x0303);

    fn template() -> Template {
        use TemplateEffect::{Publish, Sample as Read};
        let multiplicand = Publish(MULTIPLICAND_PORT);
        let multiplier = Publish(MULTIPLIER_PORT);
        Template {
            runs: [
                multiplicand,
                multiplier,
                Read(Sample::ProductLow),
                Read(Sample::ProductHigh),
                multiplicand,
                multiplier,
                Read(Sample::ProductLow),
            ]
            .into_iter()
            .map(|effect| TemplateRun {
                count: 1,
                cycle: Cycle::Six,
                kind: if matches!(effect, Publish(_)) {
                    ExposureKind::Write
                } else {
                    ExposureKind::Read
                },
                poll_before: false,
                effect,
            })
            .collect(),
        }
    }

    #[test]
    fn rejects_interleaved_requests_with_equal_seed_bytes() {
        let mut template = template();
        let requests = SeedRequests {
            multiplier: ImportedRandomMultiplier::new(FIXED_RANDOM_MULTIPLIER).unwrap(),
        };
        assert!(requests.bind(SEED, &template).is_ok());
        template.runs.swap(2, 4);
        template.runs.swap(3, 5);
        assert!(requests.bind(SEED, &template).is_err());
    }

    #[test]
    fn rejects_wrong_multiplier_even_with_consistent_products() {
        let requests: SeedRequests = canonical::from_bytes(&[7]).unwrap();
        let mut samples = Samples::default();
        samples.accept(Sample::ProductLow, 21).unwrap();
        samples.accept(Sample::ProductHigh, 0).unwrap();
        samples.accept(Sample::ProductLow, 21).unwrap();
        assert!(requests.validate_samples(SEED, samples).is_err());
        assert!(requests.bind(SEED, &template()).is_err());
    }
}
