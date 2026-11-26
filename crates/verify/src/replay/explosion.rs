//! One carried production explosion with independently decoded cache expectations.

mod drawing;
mod projections;

use super::index::{Case, Start};
use super::readers::{
    ActualReader, AdmittedCase, ExpectedCut, ExpectedReader, InputReader, record,
};
use super::result::Outcome;
use super::{Sources, data_failure};
use data::rom::Rom;
use game::production::played::explosion::{Cut, Destination, Fault, State};
use game::production::{
    AdvanceInput, Production, ProductionContent, ProductionFault, ProductionOutputs,
    ProductionRules, ProductionSeed, Unsupported,
};
use game::state::video::{Colour, Oam, OamEntry};
use oracle_record::{Cut as RecordCut, FrameRecord, NmiKind};

pub(super) use drawing::actual;
pub(super) use projections::Observed;

const STAGE: usize = 0x1f51;
const FINISHED: usize = 0x05b6;
const FLOW: usize = 0x0998;
const DESTINATION: usize = 0xd914;
const PENDING: u16 = 0xc11b;

fn byte(record: &FrameRecord, at: usize) -> Result<u8, Outcome> {
    record
        .wram
        .get(at)
        .copied()
        .ok_or_else(|| data_failure(Some(record.index), "station record byte"))
}
pub(super) fn word(record: &FrameRecord, at: usize) -> Result<u16, Outcome> {
    Ok(u16::from_le_bytes([
        byte(record, at)?,
        byte(record, at.saturating_add(1))?,
    ]))
}

pub(super) fn pending(seed: &FrameRecord) -> Result<bool, Outcome> {
    Ok(word(seed, FLOW)? == 0x22
        && word(seed, STAGE)? == PENDING
        && word(seed, DESTINATION)? == 0x22
        && byte(seed, 0x51)? & 0x80 != 0)
}

pub(super) fn enters_from(before: &FrameRecord) -> Result<bool, Outcome> {
    Ok(word(before, STAGE)? != PENDING)
}

pub(super) fn admit_seed(
    path: &std::path::Path,
    seed: &FrameRecord,
) -> Result<(Production, ProductionContent, ProductionRules), Outcome> {
    let image = std::fs::read(path).map_err(|_| data_failure(None, "station ROM file"))?;
    let rom = Rom::new(image).map_err(|_| data_failure(None, "station ROM pin"))?;
    let station = import::explosion::import(rom.lorom())
        .map_err(|why| data_failure(None, &format!("station content {why:?}")))?;
    let content = ProductionContent::Explosion(Box::new(station));
    let rules = ProductionRules::Explosion(screens_explosion::Rules::REFERENCE);
    let cols: &[u16; 256] = seed
        .cgram
        .as_ref()
        .try_into()
        .map_err(|_| data_failure(Some(seed.index), "station seed palette"))?;
    let mut oam = Oam::default();
    let (low, high) = seed
        .oam
        .split_at_checked(512)
        .ok_or_else(|| data_failure(Some(seed.index), "station seed sprite entries"))?;
    oam.high = high
        .try_into()
        .map_err(|_| data_failure(Some(seed.index), "station high sprite entries"))?;
    for (entry, &[x, y, tile, attributes]) in oam.low.iter_mut().zip(low.as_chunks::<4>().0) {
        *entry = OamEntry {
            x,
            y,
            tile,
            attributes,
        };
    }
    let state = State::start(
        word(seed, FINISHED)?,
        super::health_tail::decode(&seed.wram, Some(seed.index))?,
        word(seed, 0x09c4)?,
        match word(seed, DESTINATION)? {
            0x22 => Destination::LandingSite,
            0x1f => Destination::CeresStart,
            _ => return Err(data_failure(Some(seed.index), "station entry destination")),
        },
        word(seed, 0x09e2)? != 0,
        cols.map(Colour),
        oam,
    );
    let production = Production::admit(
        ProductionSeed::Explosion(Box::new(state)),
        content.clone(),
        rules.clone(),
    )
    .map_err(|why| data_failure(Some(seed.index), &format!("station intake {why:?}")))?;
    Ok((production, content, rules))
}

pub(super) fn timing(record: &FrameRecord) -> Result<Cut, Outcome> {
    if !matches!(record.cut,RecordCut::NmiReturn(nmi) if nmi.kind==NmiKind::Requested) {
        return Ok(Cut::Unfinished);
    }
    let mut busy = false;
    for timer in (0x0629..0x0639).step_by(2) {
        busy |= word(record, timer)? != 0;
    }
    Ok(Cut::Finished { music_busy: busy })
}

pub(super) fn advance(
    production: &mut Production,
    input: Cut,
    cut: u64,
) -> Result<ProductionOutputs, Outcome> {
    match production.advance(AdvanceInput::Explosion(input)) {
        Ok(output @ ProductionOutputs::Explosion(_)) => Ok(output),
        Err(ProductionFault::Unsupported(Unsupported::Explosion(Fault::ZebesFlight))) => {
            Err(Outcome::Unsupported {
                cut: Some(cut),
                dependency: "station dispatched the flight to Zebes".to_owned(),
            })
        }
        Err(fault) => Err(data_failure(
            Some(cut),
            &format!("station advance {fault:?}"),
        )),
        Ok(_) => Err(data_failure(Some(cut), "station output scope")),
    }
}

pub(super) struct Tape<T> {
    start: u64,
    cuts: Vec<T>,
}
impl<T> Tape<T> {
    fn get(&self, cut: u64) -> Result<&T, Outcome> {
        cut.checked_sub(self.start)
            .and_then(|i| i.checked_sub(1))
            .and_then(|i| usize::try_from(i).ok())
            .and_then(|i| self.cuts.get(i))
            .ok_or_else(|| data_failure(Some(cut), "station tape range"))
    }
}
impl Tape<Cut> {
    pub fn advance(
        &self,
        production: &mut Production,
        cut: u64,
    ) -> Result<ProductionOutputs, Outcome> {
        advance(production, *self.get(cut)?, cut)
    }
}
impl Tape<ExpectedCut> {
    pub fn read(&self, cut: u64) -> Result<ExpectedCut, Outcome> {
        self.get(cut).cloned()
    }
}

pub(super) fn admit(case: &Case, sources: &Sources) -> Result<AdmittedCase, Outcome> {
    let Start::CacheCut { frame: start } = case.start else {
        return Err(data_failure(None, "station cache start"));
    };
    let lock = std::fs::read_to_string(&sources.lock)
        .ok()
        .and_then(|text| crate::lock::Lock::parse(&text).ok())
        .ok_or_else(|| data_failure(None, "station cache lock"))?;
    let row = lock
        .row(&crate::lock::RowId(case.row.clone()))
        .ok_or_else(|| data_failure(None, "station cache row"))?;
    let cache = crate::cache::Cache::open(&sources.cache, row)
        .map_err(|_| data_failure(None, "station cache pins"))?;
    let mut previous = record(&cache, start)?;
    if !pending(&previous)?
        || !enters_from(&record(
            &cache,
            start
                .checked_sub(1)
                .ok_or_else(|| data_failure(Some(start), "station predecessor"))?,
        )?)?
    {
        return Err(data_failure(
            Some(start),
            "station incoming save and blackout",
        ));
    }
    let (production, content, rules) = admit_seed(&sources.rom, &previous)?;
    let schedule: Vec<_> = (start.saturating_add(1)..=case.end.cut_inclusive).collect();
    let mut inputs = Tape {
        start,
        cuts: Vec::with_capacity(schedule.len()),
    };
    let mut expected = Tape {
        start,
        cuts: Vec::with_capacity(schedule.len()),
    };
    let mut observer = Observed::default();
    for &cut in &schedule {
        let current = record(&cache, cut)?;
        inputs.cuts.push(timing(&current)?);
        expected.cuts.push(observer.read(&previous, &current)?);
        previous = current;
    }
    Ok(AdmittedCase {
        production,
        content,
        rules,
        schedule,
        inputs: InputReader::Explosion(inputs),
        expected: ExpectedReader::Explosion(expected),
        actual: ActualReader::Explosion,
    })
}
