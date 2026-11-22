//! Independent input and actual device comparison for the carried Random owner.

use super::{
    Sources,
    compare::Observation,
    data_failure,
    frame_origin::admission::quiet_profile,
    frame_steady,
    index::{Case, Start},
    readers::ExpectedCut,
    result::Outcome,
};
use data::{
    rom::{Rom, RomAddress},
    timing::{MULTIPLICAND_PORT, MULTIPLIER_PORT, Sample},
};
use game::production::{
    frame::random::{ImportedRandomMultiplier, Qualification, RandomSeed, SeedRequests},
    timing::{Event, EventKind, SampleValue},
};
use serde::Deserialize;
use std::path::Path;

const PACKET: &str = "shot-random-v1";
const INPUT: &str = "random-input.toml";
const INPUT_PIN: &str = "f7af85853a634a71af3a9c4cb0bc5919f9a61ed64e95fc3b709809ecc34da120";
const EXPECTED_PIN: &str = "263c2f4c034105698f97e4f45ccd50b96bb9ec93da20119403deead598142283";
const PROFILE_PIN: &str = "e81422269eba3a9d8d1ff3139d8c9e0baefa38d667e5dced69506bccb6ba11a8";

#[derive(Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum Arm {
    Passive,
    SeedCarry,
    LowSeed,
}

impl Arm {
    pub(super) fn for_case(case: &Case) -> Option<Self> {
        if !matches!(&case.start, Start::NativeSeed { path, sha256 } if
            path == &Path::new(PACKET).join(INPUT) && sha256 == INPUT_PIN)
        {
            return None;
        }
        match case.id.0.as_str() {
            "frame/random-through-completion" => Some(Self::Passive),
            "frame/random-seed-carry" => Some(Self::SeedCarry),
            "frame/random-low-seed" => Some(Self::LowSeed),
            _ => None,
        }
    }

    pub(super) fn input(self, sources: &Sources, wram: &[u8]) -> Result<Qualification, Outcome> {
        let root = sources.native.join(PACKET);
        let inputs: Inputs = frame_steady::decode(&root, INPUT, INPUT_PIN)?;
        let seed = inputs
            .arm
            .into_iter()
            .find(|arm| arm.name == self)
            .ok_or_else(|| data_failure(None, "current Random input arm"))?
            .seed;
        let basis = super::frame_origin::preparation_receipts::random_origin(wram)
            .map_err(|_| data_failure(None, "current Random origin"))?
            .0;
        let [low, high] = basis.to_le_bytes();
        let admitted = match self {
            Self::Passive => basis,
            Self::SeedCarry => u16::MAX,
            Self::LowSeed => u16::from_le_bytes([low.wrapping_add(1), high]),
        };
        if seed != admitted {
            return Err(data_failure(None, "live Random seed origin"));
        }
        let multiplier = multiplier(sources, inputs.multiplier)?;
        let template = quiet_profile::current_random(&frame_steady::member(
            &root,
            "random-profile.toml",
            PROFILE_PIN,
        )?)
        .map_err(|_| data_failure(None, "live Random profile"))?;
        Ok(Qualification {
            seed: RandomSeed(seed),
            requests: SeedRequests { multiplier },
            template,
        })
    }

    pub(super) fn expected(self, sources: &Sources) -> Result<Expected, Outcome> {
        let source: Expectations = frame_steady::decode(
            &sources.native.join(PACKET),
            "random-expected.toml",
            EXPECTED_PIN,
        )?;
        source.arm.into_iter().find(|arm| arm.name == self)
            .filter(|arm| matches!(arm.seed_store_ticks, [low, high] if low < high && high < arm.complete))
            .ok_or_else(|| data_failure(None, "Random expectation arm"))
    }
}

fn multiplier(sources: &Sources, declared: u8) -> Result<ImportedRandomMultiplier, Outcome> {
    let fault = || data_failure(None, "checked Random multiplier");
    let bytes = std::fs::read(&sources.rom).map_err(|_| fault())?;
    let rom = Rom::new(bytes).map_err(|_| fault())?;
    for offset in [0x811a, 0x812e] {
        let value = rom
            .lorom()
            .read(RomAddress { bank: 0x80, offset }, 1)
            .map_err(|_| fault())?;
        if value != [declared] {
            return Err(fault());
        }
    }
    ImportedRandomMultiplier::new(declared).map_err(|_| fault())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Inputs {
    multiplier: u8,
    arm: Vec<Input>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    name: Arm,
    seed: u16,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Expectations {
    arm: Vec<Expected>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Expected {
    name: Arm,
    complete: i64,
    result: u16,
    device_ticks: [u64; 7],
    device_values: [u8; 7],
    seed_store_ticks: [i64; 2],
}

fn transfer(position: usize, tick: u64, port: u16, value: u8) -> Observation {
    let bytes = tick
        .to_le_bytes()
        .into_iter()
        .chain(port.to_le_bytes())
        .chain([value])
        .collect();
    Observation {
        field: format!("frame.random.device.{position}"),
        bytes,
    }
}

pub(super) fn actual(events: &[Event]) -> Result<Vec<Observation>, Outcome> {
    if !events.is_empty() && events.len() != 7 {
        return Err(data_failure(Some(4), "complete Random transfer count"));
    }
    events
        .iter()
        .enumerate()
        .map(|(position, event)| {
            let (port, value) = match &event.kind {
                EventKind::Publication(write) => (write.port.0, write.value),
                EventKind::Sample(Sample::ProductLow, SampleValue::Byte(value)) => (0x4216, *value),
                EventKind::Sample(Sample::ProductHigh, SampleValue::Byte(value)) => {
                    (0x4217, *value)
                }
                EventKind::Sample(_, _)
                | EventKind::Payload(_)
                | EventKind::BulkService(_)
                | EventKind::LineService(_)
                | EventKind::Interrupt(_)
                | EventKind::ControllerStart
                | EventKind::ControllerEnd
                | EventKind::ControllerCommitted
                | EventKind::AudioPortRead { .. } => {
                    return Err(data_failure(Some(4), "Random physical transfer"));
                }
            };
            Ok(transfer(position, event.at.0, port, value))
        })
        .collect()
}

impl Expected {
    pub(super) fn read(&self) -> ExpectedCut {
        let semantics = vec![
            Observation::quantity("frame.time", self.complete),
            Observation::quantity("frame.random.seed", i64::from(self.result)),
        ];
        let low = 0x4216;
        let high = 0x4217;
        let a = MULTIPLICAND_PORT.0;
        let b = MULTIPLIER_PORT.0;
        let fields = semantics
            .iter()
            .cloned()
            .chain(
                self.device_ticks
                    .into_iter()
                    .zip([a, b, low, high, a, b, low])
                    .zip(self.device_values)
                    .enumerate()
                    .map(|(i, ((tick, port), value))| transfer(i, tick, port, value)),
            )
            .collect();
        ExpectedCut {
            semantic_count: semantics.len(),
            semantics,
            fields,
            output_count: 3,
            replies: 0,
        }
    }
}
