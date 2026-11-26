//! The shaft handoff on one seed, plus a second seed inside the final unfinished room batch.
pub(super) mod observe;
pub(super) mod seed;
use super::index::{
    Case, CaseKind, ContentProvider, ExpectedProvider, ObservationId as O, Responsibility, Start,
};
use super::readers::{
    ActualReader, AdmittedCase, ExpectedCut, ExpectedReader, InputReader, record,
};
use super::{Sources, data_failure, production_failure, result::Outcome};
use crate::{
    cache::Cache,
    lock::{Lock, RowId},
};
use data::rom::Rom;
use game::production::{
    self, AdvanceInput, Production, ProductionContent, ProductionOutputs, ProductionRules,
    ProductionSeed, blackout,
};
pub(super) use observe::actual;
use oracle_record::{Cut, NmiKind};
const ROW: &str = "chain-any-0011-DF45";

pub(super) fn witnessed_fields(rule: &str) -> Option<&'static [O]> {
    match rule {
        "screens-blackout/ElevatorStopsSamus"
        | "screens-blackout/PlatformWait"
        | "screens-blackout/DepartureHandoff" => Some(&[
            O::BlackoutScene,
            O::BlackoutRoom,
            O::BlackoutRequests,
            O::BlackoutBattery,
            O::BlackoutTimerDrawing,
        ]),
        _ => None,
    }
}

pub(super) fn valid(case: &Case) -> bool {
    let late = matches!(case.start, Start::CacheCut { frame: 239 });
    let mut outputs = vec![
        O::BlackoutTimerDrawing,
        O::BlackoutBattery,
        O::BlackoutRequests,
    ];
    if late {
        outputs.push(O::BlackoutMemory);
    }
    case.kind == CaseKind::Scoped
        && case.row == ROW
        && (late || matches!(case.start, Start::CacheCut { frame: 163 }))
        && case.end.cut_inclusive == 242
        && case.checkpoints
        && case.content == ContentProvider::PlayerRomImport
        && case.expectations.provider == ExpectedProvider::PinnedRecordCache
        && case.responsibilities == [Responsibility::Blackout]
        && case.expectations.fields == [O::BlackoutScene, O::BlackoutRoom]
        && case.expectations.outputs == outputs
}

pub(super) struct Tape<T> {
    first: u64,
    values: Vec<T>,
}
impl<T> Tape<T> {
    pub(super) fn at(&self, cut: u64) -> Result<&T, Outcome> {
        cut.checked_sub(self.first)
            .and_then(|n| usize::try_from(n).ok())
            .and_then(|n| self.values.get(n))
            .ok_or_else(|| data_failure(Some(cut), "blackout schedule"))
    }
}
impl Tape<blackout::Cut> {
    pub(super) fn advance(
        &self,
        production: &mut Production,
        cut: u64,
    ) -> Result<ProductionOutputs, Outcome> {
        production
            .advance(AdvanceInput::Blackout(*self.at(cut)?))
            .map_err(|why| production_failure(cut, why))
    }
}

pub(super) fn admit(case: &Case, sources: &Sources) -> Result<AdmittedCase, Outcome> {
    let Start::CacheCut { frame } = case.start else {
        return Err(data_failure(None, "blackout seed"));
    };
    let late = frame == 239;
    let lock = Lock::parse(
        &std::fs::read_to_string(&sources.lock).map_err(|_| data_failure(None, "blackout lock"))?,
    )
    .map_err(|_| data_failure(None, "blackout lock format"))?;
    let row = lock
        .row(&RowId(case.row.clone()))
        .ok_or_else(|| data_failure(None, "blackout row"))?;
    let cache = Cache::open(&sources.cache, row)
        .map_err(|_| data_failure(None, "blackout pinned cache"))?;
    let rom =
        Rom::new(std::fs::read(&sources.rom).map_err(|_| data_failure(None, "blackout ROM file"))?)
            .map_err(|_| data_failure(None, "blackout ROM pin"))?;
    let format = game::exact::save_files::Format::load(rom.lorom())
        .map_err(|_| data_failure(None, "cartridge format"))?;
    let figures =
        import::countdown::import(rom.lorom()).map_err(|_| data_failure(None, "timer art"))?;
    let mut previous = record(&cache, frame)?;
    let state = seed::read(&previous, rom.lorom(), &format, late)?;
    let content = ProductionContent::Blackout(Box::new(blackout::Content {
        figures,
        format,
        graphics: data::pose::tiles::Graphics::default(),
    }));
    let rules = ProductionRules::Blackout;
    let production = Production::admit(
        ProductionSeed::Blackout(Box::new(state)),
        content.clone(),
        rules.clone(),
    )
    .map_err(|why| data_failure(Some(frame), &format!("blackout admission {why:?}")))?;
    let schedule: Vec<_> = (frame.saturating_add(1)..=case.end.cut_inclusive).collect();
    let mut inputs = Vec::new();
    let mut expected = Vec::new();
    for &cut in &schedule {
        let current = record(&cache, cut)?;
        inputs.push(work(
            &previous,
            &current,
            !late && cut == frame.saturating_add(1),
        )?);
        expected.push(observe::expected(&previous, &current, late)?);
        previous = current;
    }
    let first = frame.saturating_add(1);
    Ok(AdmittedCase {
        production,
        content,
        rules,
        schedule,
        inputs: InputReader::Blackout(Tape {
            first,
            values: inputs,
        }),
        expected: ExpectedReader::Blackout(Tape {
            first,
            values: expected,
        }),
        actual: ActualReader::Blackout(late),
    })
}

pub(super) fn work(
    previous: &oracle_record::FrameRecord,
    current: &oracle_record::FrameRecord,
    reached: bool,
) -> Result<blackout::Cut, Outcome> {
    let Cut::NmiReturn(nmi) = current.cut else {
        return Err(data_failure(Some(current.index), "blackout NMI"));
    };
    let work = match previous.cut {
        Cut::NmiReturn(prior) => match prior.interrupted_pc {
            0x0080_82ce => blackout::Work::VerticalEnded,
            0x0081_8066 if nmi.kind == NmiKind::Requested => blackout::Work::SaveCompleted,
            _ if prior.kind == NmiKind::Requested => blackout::Work::Fresh { reached },
            _ => blackout::Work::Interrupted,
        },
        Cut::VblankWithoutNmi | Cut::NmiStillRunning { .. } => blackout::Work::Interrupted,
    };
    Ok(blackout::Cut {
        work,
        publish: nmi.kind == NmiKind::Requested,
    })
}

fn byte(r: &[u8], at: usize) -> Result<u8, Outcome> {
    r.get(at)
        .copied()
        .ok_or_else(|| data_failure(None, "blackout record byte"))
}
fn word(r: &[u8], at: usize) -> Result<u16, Outcome> {
    r.get(at..)
        .and_then(<[_]>::first_chunk::<2>)
        .map(|b| u16::from_le_bytes(*b))
        .ok_or_else(|| data_failure(None, "blackout record word"))
}

fn digits(r: &[u8]) -> Result<u32, Outcome> {
    let decimal = |at| {
        r.get(at)
            .copied()
            .map(|n: u8| {
                u32::from(n >> 4)
                    .saturating_mul(10)
                    .saturating_add(u32::from(n & 15))
            })
            .ok_or_else(|| data_failure(None, "timer digits"))
    };
    Ok(decimal(0x947)?
        .saturating_mul(6000)
        .saturating_add(decimal(0x946)?.saturating_mul(100))
        .saturating_add(decimal(0x945)?))
}
