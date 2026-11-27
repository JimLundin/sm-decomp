//! The scoped W1 cinematic replay through the same weapon update, hit and draw entries as play.

mod expected;
mod inputs;
mod observe;

use super::{Sources, compare::Observation, data_failure, index::Case, readers, result::Outcome};
use crate::replay::cache_failure;
use crate::{cache::Cache, lock::RowId};
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

pub(super) fn admit(case: &Case, sources: &Sources) -> Result<readers::AdmittedCase, Outcome> {
    let super::index::Start::CacheCut { frame: first } = case.start else {
        return Err(data_failure(None, "missile cache origin"));
    };
    let lock = super::read_lock(sources)?;
    let row = lock
        .row(&RowId(case.row.clone()))
        .ok_or_else(|| data_failure(None, "missile lock row"))?;
    if case.end.cut_inclusive >= row.frames.0 {
        return Err(data_failure(
            Some(case.end.cut_inclusive),
            "missile row horizon",
        ));
    }
    let cache = Cache::open(&sources.cache, row).map_err(cache_failure(None))?;
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
    let offset =
        usize::try_from(first).map_err(|_| data_failure(Some(first), "missile row offset"))?;
    let mut records = cache.records().skip(offset);
    let mut read = |cut| {
        let (actual, frame) = records
            .next()
            .ok_or_else(|| data_failure(Some(cut), "missile record cut"))?
            .map_err(cache_failure(Some(cut)))?;
        if actual.0 != cut {
            return Err(data_failure(Some(cut), "missile record coordinate"));
        }
        Ok(frame)
    };
    let origin = read(first)?;
    let seed = seed(&origin)?;
    let world = super::weapons::terrain(&origin.wram)?;
    let schedule: Vec<_> = (first..=case.end.cut_inclusive).collect();
    let mut inputs = Inputs::new(inputs::Demo::load(rom)?, world, first);
    let observations = Expected::load(rom)?;
    let mut expected = Vec::with_capacity(schedule.len());
    let mut previous = None;
    // The witness ends by the first held cut after the intro has hidden its actors.
    let mut ended = false;
    for record in std::iter::once(Ok(origin)).chain(schedule.iter().skip(1).copied().map(read)) {
        let frame = record?;
        if ended {
            return Err(data_failure(Some(frame.index), "missile battle ended"));
        }
        if let Some(prior) = previous.as_ref() {
            ended = inputs::hidden(prior)? && inputs::hidden(&frame)?;
        }
        inputs.read(previous.as_ref(), &frame, &content)?;
        expected.push(observations.read(previous.as_ref(), &frame)?);
        previous = Some(frame);
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
        expected: readers::ExpectedReader::RecordedCuts(readers::CutTape {
            first,
            values: expected,
        }),
        actual: readers::ActualReader::IntroMissiles,
    })
}

fn seed(frame: &FrameRecord) -> Result<ProductionSeed, Outcome> {
    if !inputs::starts_demo(frame)? || word(frame, 0x0ccc)? != 0 || word(frame, 0x0cce)? != 0 {
        return Err(data_failure(
            Some(frame.index),
            "missile battle demo origin",
        ));
    }
    let mut fractions = enum_map::EnumMap::from_fn(|_| w1::Axes {
        x: w1::PixelFraction(0),
        y: w1::PixelFraction(0),
    });
    for (index, fraction) in fractions.values_mut().enumerate() {
        let offset = index.saturating_mul(2);
        let w = |column: usize| word(frame, column.saturating_add(offset));
        if w(0x0c40)? != 0 || w(0x0c90)? != 0 {
            return Err(data_failure(
                Some(frame.index),
                "cinematic weapon origin slots",
            ));
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
