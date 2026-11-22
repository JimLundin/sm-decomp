//! Physical Random requests and retained live multiplication samples.

use super::caller::{Caller, Dependency, Preparation, PreparationStage, RANDOM};
use crate::production::{ProductionFault, Unsupported, timing};
use canonical::{
    Canonical,
    model::{Model, Pinned},
};
use data::timing::{
    DeviceEffect, DeviceWrite, ExposureKind, MULTIPLICAND_PORT, MULTIPLIER_PORT, Publication,
    PublicationPort, Run, Sample, Template, TemplateEffect,
};
use game_random::{RandomNumber, RandomSequence};

pub(super) const fn missing() -> ProductionFault {
    ProductionFault::Unsupported(Unsupported::Frame(Dependency::RandomPreparation))
}

/// Current Random seed and checked physical work from its admitted producer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Qualification {
    /// Independently admitted current seed.
    pub seed: RandomSeed,
    /// Imported immutable request operand.
    pub requests: SeedRequests,
    /// Checked physical request, sample and return work.
    pub template: Template,
}

impl Qualification {
    pub(super) fn validate(
        &self,
        sequence: &Sequence,
        caller: &Caller,
        timing: &timing::State,
    ) -> Result<(), ProductionFault> {
        let profile = self
            .requests
            .bind(self.seed, &self.template)
            .map_err(|_| missing())?;
        let mut current = Sequence::new(self.seed);
        if at_builder(caller) {
            current.draw();
        }
        if *sequence != current {
            return Err(missing());
        }
        if let Caller::RandomRunning {
            requested, samples, ..
        } = caller
        {
            let [work] = timing.work.as_slice() else {
                return Err(missing());
            };
            if *requested != self.seed
                || work.kind != timing::WorkKind::Semantic(RANDOM)
                || work.profile != profile
            {
                return Err(missing());
            }
            let retained = RetainedArithmetic::at_cursor(work, timing.hardware.math.dividend)?;
            retained.validate(&timing.hardware.math, *samples)?;
        } else if at_builder(caller) {
            let mut retained = RetainedArithmetic::new(timing.hardware.math.dividend);
            for run in &profile.runs {
                retained.complete_run(run, run.count)?;
            }
            retained.validate(&timing.hardware.math, retained.samples)?;
        }
        Ok(())
    }

    pub(super) fn advance(
        &self,
        sequence: &mut Sequence,
        caller: &mut Caller,
        timing: &mut timing::State,
        budget: data::timing::MasterTicks,
    ) -> Result<(timing::Progress, Vec<timing::Event>), ProductionFault> {
        if at_builder(caller) {
            return Err(ProductionFault::Unsupported(Unsupported::Frame(
                Dependency::BuilderPreparation,
            )));
        }
        if matches!(caller, Caller::Random { .. }) {
            begin(caller, timing, sequence, &self.template, self.requests)?;
        }
        let (progress, events) = timing
            .step(budget, &[])
            .map_err(ProductionFault::FrameHardware)?;
        if events.iter().any(|event| {
            !matches!(
                event.kind,
                timing::EventKind::Publication(_) | timing::EventKind::Sample(_, _)
            )
        }) {
            return Err(missing());
        }
        accept_events(caller, &events)?;
        if progress == timing::Progress::Completed(RANDOM) {
            finish(caller, sequence, self.requests)?;
        } else if progress != timing::Progress::Advanced {
            return Err(missing());
        }
        Ok((progress, events))
    }
}

pub(super) const fn at_builder(caller: &Caller) -> bool {
    matches!(
        caller,
        Caller::Preparation {
            prepared: Preparation {
                builder: None,
                gameplay: None,
                actors: None,
                debug: None,
                palette: None,
                ..
            },
            stage: PreparationStage::Builder,
            charging: false,
        }
    )
}

// Derive only the arithmetic this owner has already caused. The origin's unrelated
// dividend and pre-request multiplication are outside the selected Random work.
struct RetainedArithmetic {
    math: timing::Math,
    latched: bool,
    requested: bool,
    samples: Samples,
}

impl RetainedArithmetic {
    fn new(dividend: u16) -> Self {
        Self {
            math: timing::Math {
                dividend,
                ..timing::Math::default()
            },
            latched: false,
            requested: false,
            samples: Samples::Empty,
        }
    }

    fn at_cursor(work: &timing::Work, dividend: u16) -> Result<Self, ProductionFault> {
        use timing::{Charge, Stage};
        let index = usize::try_from(work.cursor.run).map_err(|_| missing())?;
        let mut retained = Self::new(dividend);
        for run in work.profile.runs.get(..index).ok_or_else(missing)? {
            retained.complete_run(run, run.count)?;
        }
        if let Some(run) = work.profile.runs.get(index) {
            retained.complete_run(run, work.cursor.occurrence)?;
            let stage = &work.cursor.stage;
            let write = run.exposure.kind == ExposureKind::Write;
            if write && !matches!(stage, Stage::Poll | Stage::Before) {
                retained.edges(1);
            }
            let spent = if matches!(run.exposure.effect, DeviceEffect::Sample(_)) {
                matches!(
                    stage,
                    Stage::Charge(Charge::AfterSample { .. })
                        | Stage::Publish
                        | Stage::After
                        | Stage::Finish
                )
            } else {
                matches!(stage, Stage::After | Stage::Finish)
            };
            if spent {
                retained.apply(&run.exposure.effect)?;
            }
            if !write && matches!(stage, Stage::Finish) {
                retained.edges(1);
            }
        }
        Ok(retained)
    }

    fn complete_run(&mut self, run: &Run, count: u32) -> Result<(), ProductionFault> {
        if count == 0 {
            return Ok(());
        }
        let write = run.exposure.kind == ExposureKind::Write;
        if write {
            self.edges(count);
        }
        self.apply(&run.exposure.effect)?;
        if !write {
            self.edges(count);
        }
        Ok(())
    }

    fn edges(&mut self, count: u32) {
        for _ in 0..count.min(u32::from(self.math.remaining)) {
            self.math.edge();
        }
    }

    fn apply(&mut self, effect: &DeviceEffect) -> Result<(), ProductionFault> {
        match effect {
            DeviceEffect::None => {}
            DeviceEffect::Publish(write) => match write.effect {
                Publication::Multiplicand(value) => {
                    self.math.multiplicand = value;
                    self.latched = true;
                }
                Publication::Multiplier(value) if self.latched && self.math.remaining == 0 => {
                    self.math.start_multiply(value);
                    self.requested = true;
                }
                Publication::Multiplier(_)
                | Publication::DividendLow(_)
                | Publication::InterruptEnable(_)
                | Publication::HorizontalLow(_)
                | Publication::HorizontalHigh(_)
                | Publication::VerticalLow(_)
                | Publication::VerticalHigh(_)
                | Publication::ChannelUpdate(_, _)
                | Publication::BulkEnable(_)
                | Publication::LineEnable(_)
                | Publication::Downstream => {
                    return Err(missing());
                }
            },
            DeviceEffect::Sample(sample) => {
                if !self.requested || self.math.remaining != 0 {
                    return Err(missing());
                }
                let [low, high] = self.math.product.to_le_bytes();
                let value = if *sample == Sample::ProductLow {
                    low
                } else {
                    high
                };
                self.samples.accept(*sample, value).map_err(|_| missing())?;
            }
        }
        Ok(())
    }

    fn validate(&self, actual: &timing::Math, samples: Samples) -> Result<(), ProductionFault> {
        if self.samples != samples
            || self.latched && self.math.multiplicand != actual.multiplicand
            || self.requested && self.math != *actual
        {
            return Err(missing());
        }
        Ok(())
    }
}

pub(super) fn accept_events(
    caller: &mut Caller,
    events: &[timing::Event],
) -> Result<(), ProductionFault> {
    if let Caller::RandomRunning { samples, .. } = caller {
        for event in events {
            if let timing::EventKind::Sample(
                device @ (Sample::ProductLow | Sample::ProductHigh),
                timing::SampleValue::Byte(value),
            ) = &event.kind
            {
                samples.accept(*device, *value).map_err(|_| missing())?;
            }
        }
    }
    Ok(())
}

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

pub(super) fn begin(
    caller: &mut Caller,
    timing: &mut timing::State,
    sequence: &Sequence,
    template: &Template,
    requests: SeedRequests,
) -> Result<(), ProductionFault> {
    let Caller::Random {
        prelude,
        publication,
        music,
        haze,
        layers,
    } = &*caller
    else {
        return Err(ProductionFault::WrongScope);
    };
    let fault = ProductionFault::Unsupported(Unsupported::Frame(Dependency::RandomPreparation));
    let current = sequence.current();
    let profile = requests.bind(current, template).map_err(|_| fault)?;
    timing
        .begin(RANDOM, profile)
        .map_err(ProductionFault::FrameHardware)?;
    *caller = Caller::RandomRunning {
        prelude: prelude.clone(),
        publication: *publication,
        music: music.clone(),
        haze: haze.clone(),
        layers: layers.clone(),
        requested: current,
        samples: Samples::default(),
    };
    Ok(())
}

pub(super) fn finish(
    caller: &mut Caller,
    sequence: &mut Sequence,
    requests: SeedRequests,
) -> Result<(), ProductionFault> {
    let Caller::RandomRunning {
        prelude,
        publication,
        music,
        haze,
        layers,
        requested,
        samples,
    } = &*caller
    else {
        return Err(ProductionFault::WrongScope);
    };
    let fault = ProductionFault::Unsupported(Unsupported::Frame(Dependency::RandomPreparation));
    if sequence.current() != *requested {
        return Err(fault);
    }
    requests
        .validate_samples(*requested, *samples)
        .map_err(|_| fault)?;
    sequence.draw();
    *caller = Caller::Preparation {
        prepared: Preparation {
            prelude: prelude.clone(),
            publication: *publication,
            music: music.clone(),
            haze: haze.clone(),
            layers: layers.clone(),
            builder: None,
            gameplay: None,
            actors: None,
            debug: None,
            palette: None,
        },
        stage: PreparationStage::Builder,
        charging: false,
    };
    Ok(())
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
