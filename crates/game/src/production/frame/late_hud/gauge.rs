//! Live arithmetic exposures for a late HUD gauge redraw.

use crate::production::timing::{self, Event, EventKind, SampleValue, WorkId};
use canonical::Canonical;
use data::timing::{
    DIVIDEND_HIGH_PORT, DIVIDEND_LOW_PORT, DIVISOR_PORT, DeviceWrite, Profile, PublicationPort,
    Sample, Template, TemplateEffect,
};

pub(super) const TANKS: WorkId = WorkId(1325);
pub(super) const DIGITS: WorkId = WorkId(1326);

const OPERANDS: [PublicationPort; 3] = [DIVIDEND_LOW_PORT, DIVIDEND_HIGH_PORT, DIVISOR_PORT];
const DIGIT_SAMPLES: [Sample; 4] = [
    Sample::QuotientLow,
    Sample::QuotientHigh,
    Sample::ProductLow,
    Sample::ProductHigh,
];
const TANK_SAMPLES: [Sample; 6] = [
    Sample::QuotientLow,
    Sample::QuotientHigh,
    Sample::ProductLow,
    Sample::ProductHigh,
    Sample::QuotientLow,
    Sample::QuotientHigh,
];
const ENERGY_PER_TANK: u8 = 100;
const DECIMAL: u8 = 10;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Contract {
    tanks: Template,
    digits: Template,
}

impl Contract {
    pub(super) fn new(template: &Template) -> Option<Self> {
        let split = template
            .runs
            .iter()
            .enumerate()
            .filter(|(_, run)| run.effect == TemplateEffect::Publish(DIVIDEND_LOW_PORT))
            .nth(2)?
            .0;
        let (tanks, digits) = template.runs.split_at_checked(split)?;
        let tanks = Template {
            runs: tanks.to_vec(),
        };
        let digits = Template {
            runs: digits.to_vec(),
        };
        tanks.validate().ok()?;
        digits.validate().ok()?;
        let writes = OPERANDS.map(TemplateEffect::Publish);
        let expected_tanks = writes
            .into_iter()
            .chain(TANK_SAMPLES.into_iter().take(4).map(TemplateEffect::Sample))
            .chain(writes)
            .chain(TANK_SAMPLES.into_iter().skip(4).map(TemplateEffect::Sample));
        let expected_digits = writes
            .into_iter()
            .chain(DIGIT_SAMPLES.map(TemplateEffect::Sample));
        if !effects(&tanks).eq(expected_tanks) || !effects(&digits).eq(expected_digits) {
            return None;
        }
        Some(Self { tanks, digits })
    }

    pub(super) fn tanks(&self, energy: u16, capacity: u16) -> Option<Profile> {
        let writes: Vec<_> = [
            division(energy, ENERGY_PER_TANK)?,
            division(capacity, ENERGY_PER_TANK)?,
        ]
        .into_iter()
        .flatten()
        .collect();
        self.tanks.bind(&writes).ok()
    }

    pub(super) fn digits(&self, remainder: u16) -> Option<Profile> {
        self.digits.bind(&division(remainder, DECIMAL)?).ok()
    }

    pub(super) fn prefix_holds(
        &self,
        progress: &Progress,
        energy: interface_hud::Energy,
        work: Option<&timing::Work>,
        hardware: &timing::Hardware,
        origin: timing::Math,
        events: &[Event],
    ) -> Option<()> {
        let tanks = self.tanks(energy.now, energy.capacity)?;
        let cursor = |id, profile: &Profile| {
            work.filter(|work| {
                work.kind == timing::WorkKind::Semantic(id) && &work.profile == profile
            })
            .map(|work| &work.cursor)
        };
        let holds = match progress {
            Progress::Tanks(_) => timing::division_prefix_holds(
                hardware,
                origin,
                &[(&tanks, Some(cursor(TANKS, &tanks)?))],
                events,
            ),
            Progress::DigitsReady(_) if work.is_none() => {
                timing::division_prefix_holds(hardware, origin, &[(&tanks, None)], events)
            }
            Progress::Digits { remainder, .. } => {
                let digits = self.digits(*remainder)?;
                timing::division_prefix_holds(
                    hardware,
                    origin,
                    &[(&tanks, None), (&digits, Some(cursor(DIGITS, &digits)?))],
                    events,
                )
            }
            Progress::DigitsReady(_) => false,
        };
        holds.then_some(())
    }
}

fn effects(template: &Template) -> impl Iterator<Item = TemplateEffect> {
    template
        .runs
        .iter()
        .filter(|run| run.effect != TemplateEffect::None)
        .flat_map(|run| (0..run.count).map(|_| run.effect))
}

fn division(dividend: u16, divisor: u8) -> Option<[DeviceWrite; 3]> {
    let [low, high] = dividend.to_le_bytes();
    let [lo, hi, start] = OPERANDS;
    Some([
        DeviceWrite::from_cpu_port(lo.0, low).ok()?,
        DeviceWrite::from_cpu_port(hi.0, high).ok()?,
        DeviceWrite::from_cpu_port(start.0, divisor).ok()?,
    ])
}

#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub(super) enum Progress {
    Tanks(Vec<u8>),
    DigitsReady(u16),
    Digits { remainder: u16, samples: Vec<u8> },
}

impl Progress {
    pub(super) const fn work(&self) -> Option<WorkId> {
        match self {
            Self::Tanks(_) => Some(TANKS),
            Self::DigitsReady(_) => None,
            Self::Digits { .. } => Some(DIGITS),
        }
    }

    pub(super) fn sample(&mut self, work: WorkId, events: &[Event]) -> Option<()> {
        let (samples, order): (&mut Vec<u8>, &[Sample]) = match (self, work) {
            (Self::Tanks(samples), TANKS) => (samples, &TANK_SAMPLES),
            (Self::Digits { samples, .. }, DIGITS) => (samples, &DIGIT_SAMPLES),
            _ => return None,
        };
        for event in events {
            if let EventKind::Sample(sample, SampleValue::Byte(value)) = event.kind {
                if order.get(samples.len()) != Some(&sample) {
                    return None;
                }
                samples.push(value);
            }
        }
        Some(())
    }

    pub(super) fn complete_tanks(&mut self, energy: u16, capacity: u16) -> Option<()> {
        let Self::Tanks(samples) = self else {
            return None;
        };
        let [_, _, low, high, _, _] = samples.as_slice() else {
            return None;
        };
        if samples.as_slice() != tank_values(energy, capacity) {
            return None;
        }
        *self = Self::DigitsReady(u16::from_le_bytes([*low, *high]));
        Some(())
    }

    pub(super) fn digits_complete(&self) -> bool {
        matches!(self, Self::Digits { remainder, samples } if samples.as_slice() == digit_values(*remainder))
    }

    pub(super) fn holds(&self, energy: u16, capacity: u16, events: &[Event]) -> bool {
        let complete_tanks = tank_values(energy, capacity);
        let [_, _, low, high, _, _] = complete_tanks;
        let expected_remainder = u16::from_le_bytes([low, high]);
        let (tanks, digits): (&[u8], &[u8]) = match self {
            Self::Tanks(samples) if complete_tanks.starts_with(samples) => (samples, &[]),
            Self::DigitsReady(remainder) if *remainder == expected_remainder => {
                (&complete_tanks, &[])
            }
            Self::Digits { remainder, samples }
                if *remainder == expected_remainder
                    && digit_values(*remainder).starts_with(samples) =>
            {
                (&complete_tanks, samples)
            }
            Self::Tanks(_) | Self::DigitsReady(_) | Self::Digits { .. } => return false,
        };
        let expected = TANK_SAMPLES
            .into_iter()
            .zip(tanks.iter().copied())
            .chain(DIGIT_SAMPLES.into_iter().zip(digits.iter().copied()));
        events
            .iter()
            .filter_map(|event| {
                if let EventKind::Sample(sample, SampleValue::Byte(value)) = event.kind {
                    Some((sample, value))
                } else {
                    None
                }
            })
            .eq(expected)
    }
}

fn tank_values(energy: u16, capacity: u16) -> [u8; 6] {
    let divisor = u16::from(ENERGY_PER_TANK);
    let [quotient_low, quotient_high] = energy
        .checked_div(divisor)
        .unwrap_or_default()
        .to_le_bytes();
    let [remainder_low, remainder_high] = energy
        .checked_rem(divisor)
        .unwrap_or_default()
        .to_le_bytes();
    let [capacity_low, capacity_high] = capacity
        .checked_div(divisor)
        .unwrap_or_default()
        .to_le_bytes();
    [
        quotient_low,
        quotient_high,
        remainder_low,
        remainder_high,
        capacity_low,
        capacity_high,
    ]
}

fn digit_values(remainder: u16) -> [u8; 4] {
    let divisor = u16::from(DECIMAL);
    let [tens_low, tens_high] = remainder
        .checked_div(divisor)
        .unwrap_or_default()
        .to_le_bytes();
    let [ones_low, ones_high] = remainder
        .checked_rem(divisor)
        .unwrap_or_default()
        .to_le_bytes();
    [tens_low, tens_high, ones_low, ones_high]
}
