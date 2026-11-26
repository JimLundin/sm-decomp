//! The scoped W1 cinematic replay through the same weapon update, hit and draw entries as play.

mod expected;
mod inputs;
mod observe;

use super::{Sources, compare::Observation, data_failure, index::Case, readers, result::Outcome};
use crate::{
    cache::Cache,
    lock::{Lock, RowId},
};
use canonical::model::Model;
use data::rom::Rom;
use game::{
    production::{
        Production, ProductionContent, ProductionRules, ProductionSeed,
        intro_missile_scope::{Output, State},
        played::intro_missiles::{Content, Supplies},
    },
    state::owned::OwnedState,
    weapons_projectiles_bridge::PowerContent,
};
use oracle_record::FrameRecord;
use weapons_projectiles as w1;

pub(super) use expected::Expected;
pub(super) use inputs::Inputs;

pub(super) const ROW: &str = "any-intro-2219";
pub(super) const END: u64 = 649;

pub(super) fn admit(case: &Case, sources: &Sources) -> Result<readers::AdmittedCase, Outcome> {
    let lock = std::fs::read_to_string(&sources.lock)
        .map_err(|_| data_failure(None, "missile record lock"))?;
    let lock = Lock::parse(&lock).map_err(|_| data_failure(None, "missile lock schema"))?;
    let row = lock
        .row(&RowId(case.row.clone()))
        .ok_or_else(|| data_failure(None, "missile lock row"))?;
    let cache =
        Cache::open(&sources.cache, row).map_err(|_| data_failure(None, "missile record pins"))?;
    let bad_rom = || data_failure(None, "missile player ROM");
    let image =
        Rom::new(std::fs::read(&sources.rom).map_err(|_| bad_rom())?).map_err(|_| bad_rom())?;
    let rom = image.lorom();
    let art = import::projectile::import_intro_missile(rom)
        .map_err(|_| data_failure(None, "missile clip import"))?;
    let beam = import::projectile::import(rom)
        .map_err(|_| data_failure(None, "ordinary weapon import"))?;
    let power = PowerContent::new(beam.content)
        .map_err(|_| data_failure(None, "ordinary weapon content"))?;
    let content =
        Content::new(art, power).map_err(|_| data_failure(None, "cinematic weapon content"))?;
    let origin = readers::record(&cache, 0)?;
    let seed = seed(&origin)?;
    let world = super::weapons::terrain(&origin.wram)?;
    let schedule: Vec<_> = (0..=case.end.cut_inclusive).collect();
    let mut inputs = Inputs::new(inputs::Demo::load(rom)?, world);
    let mut expected = Expected::load(rom)?;
    let mut previous = None;
    for (&cut, bytes) in schedule.iter().zip(cache.records_without_pictures()) {
        let (actual, frame) = bytes.map_err(|_| data_failure(Some(cut), "missile record cut"))?;
        if actual.0 != cut {
            return Err(data_failure(Some(cut), "missile record coordinate"));
        }
        let timing = inputs::timing(previous.as_ref(), &frame)?;
        inputs.read(previous.as_ref(), &frame, &timing, &content)?;
        expected.read(previous.as_ref(), &frame, &timing)?;
        previous = Some(frame);
    }
    if previous.is_none_or(|frame| frame.index != case.end.cut_inclusive) {
        return Err(data_failure(None, "complete missile replay horizon"));
    }

    let content = ProductionContent::IntroMissiles(Box::new(content));
    let rules = ProductionRules::IntroMissiles(w1::Rules::REFERENCE);
    let production = Production::admit(seed, content.clone(), rules.clone())
        .map_err(|_| data_failure(None, "cinematic weapon admission"))?;
    Ok(readers::AdmittedCase {
        production,
        content,
        rules,
        schedule,
        inputs: readers::InputReader::IntroMissiles(inputs),
        expected: readers::ExpectedReader::IntroMissiles(expected),
        actual: readers::ActualReader::IntroMissiles,
    })
}

fn seed(frame: &FrameRecord) -> Result<ProductionSeed, Outcome> {
    let mut fractions = enum_map::EnumMap::from_fn(|_| w1::Axes {
        x: w1::PixelFraction(0),
        y: w1::PixelFraction(0),
    });
    for (index, fraction) in fractions.values_mut().enumerate() {
        let offset = index.saturating_mul(2);
        let w = |column: usize| word(frame, column.saturating_add(offset));
        if w(0x0c40)? != 0 || w(0x0c90)? != 0 {
            return Err(data_failure(Some(0), "cinematic weapon origin slots"));
        }
        fraction.x = w1::PixelFraction(w(0x0b8c)?);
        fraction.y = w1::PixelFraction(w(0x0ba0)?);
    }
    Ok(ProductionSeed::IntroMissiles(Box::new(State {
        weapons: Model(Box::new(w1::State::reset(fractions))),
        supplies: Supplies {
            ammo: word(frame, 0x09c6)?,
            selected: word(frame, 0x09d2)? == 1,
            cancel_after_shot: word(frame, 0x0a04)? != 0,
            protection: word(frame, 0x18ac)?,
        },
        pending: None,
    })))
}

pub(super) fn actual(
    production: &Production,
    output: Option<&Output>,
) -> Result<Vec<Observation>, Outcome> {
    let OwnedState::IntroMissiles(state) = production.state() else {
        return Err(data_failure(None, "cinematic weapon owned scope"));
    };
    let mut observations = observe::actual_state(state);
    observations.extend(observe::actual_output(output));
    Ok(observations)
}

fn word(frame: &FrameRecord, address: usize) -> Result<u16, Outcome> {
    super::weapons::word(&frame.wram, address)
}
