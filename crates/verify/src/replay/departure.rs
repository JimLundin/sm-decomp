//! Indexed handoffs reached by uninterrupted production play from the elevator.
mod observations;
pub(super) use observations::actual;

use super::index::{
    Case, CaseKind, ContentProvider, ExpectedProvider, ObservationId, Responsibility, Start,
};
use super::played::{explosion as joined, rows::Rows};
use super::readers::{ActualReader, AdmittedCase, ExpectedReader, InputReader};
use super::{Sources, data_failure, explosion, result::Outcome};
use game::production::{
    AdvanceInput, Production, ProductionContent, ProductionRules, played::departure::Input,
};
use oracle_record::FrameRecord;
use std::collections::BTreeMap;
use std::path::Path;

pub(super) struct Tape<T>(BTreeMap<u64, T>);
impl<T> Tape<T> {
    pub(super) fn get(&self, cut: u64) -> Result<&T, Outcome> {
        self.0
            .get(&cut)
            .ok_or_else(|| data_failure(Some(cut), "departure tape coordinate"))
    }
}

pub(super) fn valid(case: &Case) -> bool {
    use ObservationId as O;
    use Responsibility as R;
    case.kind == CaseKind::Integrated
        && case.row == "chain-any-0011-DF45"
        && matches!(case.start,Start::PlayedContinuation{seed:239 | 241,frame} if (242..2124).contains(&frame))
        && case.end.cut_inclusive <= 2124
        && case.responsibilities
            == [
                R::StationExplosion,
                R::Zebes,
                R::Health,
                R::ZebesScheduler,
                R::ZebesDrawing,
                R::LandingSite,
            ]
        && case.content == ContentProvider::PlayerRomImport
        && case.expectations.provider == ExpectedProvider::PinnedRecordCache
        && case.expectations.fields
            == [
                O::DepartureSchedule,
                O::DepartureHealth,
                O::DepartureRandom,
                O::DepartureRoom,
            ]
        && case.expectations.outputs == [O::DeparturePicture, O::DepartureLoader]
        && case.expectations.output_cut.is_none()
        && case.checkpoints
        && !case.borrowed.is_empty()
        && !case.limits.is_empty()
}

pub(super) fn admit(case: &Case, sources: &Sources) -> Result<AdmittedCase, Outcome> {
    let Start::PlayedContinuation { seed, frame } = case.start else {
        return Err(data_failure(None, "departure continuation origin"));
    };
    let lock = std::fs::read_to_string(&sources.lock)
        .ok()
        .and_then(|s| crate::lock::Lock::parse(&s).ok())
        .ok_or_else(|| data_failure(None, "departure lock"))?;
    let root = sources
        .lock
        .ancestors()
        .nth(3)
        .ok_or_else(|| data_failure(None, "departure lock path"))?;
    let mut rows = Rows::open(&lock, root, &sources.cache, &case.row)?;
    let mut previous = rows.record(seed)?;
    let (mut production, content, rules) = admit_origin(&mut rows, seed, &previous, &sources.rom)?;
    let assets = if let ProductionContent::Departure(assets) = &content {
        assets.as_ref()
    } else if let ProductionContent::Escape(assets) = &content {
        &assets.departure
    } else {
        return Err(data_failure(None, "departure assets"));
    };
    let mut inputs = BTreeMap::new();
    let mut expected = BTreeMap::new();
    for cut in seed.saturating_add(1)..=case.end.cut_inclusive {
        let now = rows.record(cut)?;
        if now.video_frame != previous.video_frame.wrapping_add(1) {
            return Err(data_failure(Some(cut), "departure cache join"));
        }
        let cadence = explosion::timing(&now)?;
        // The receiving capture records the external bus working on the ROM sprite
        // transfer. It supplies the T1 receipt, independently of the expected room.
        let receiving = matches!(now.cut, oracle_record::Cut::VblankWithoutNmi)
            && now.ppu.dma[1][4] == assets.landing.sprite_source.bank
            && u16::from_le_bytes([now.ppu.dma[1][2], now.ppu.dma[1][3]])
                >= assets.landing.sprite_source.offset
            && now.cpu.k == 0x82
            && now.cpu.pc == 0x8327;
        let input = if production.departure().is_none() {
            Input::Shaft {
                work: super::blackout::work(&previous, &now, false)?,
                station: cadence,
            }
        } else if receiving {
            Input::Loader(joined::receiving_work(&now, &assets.landing)?)
        } else {
            Input::Scene(cadence)
        };
        if cut <= frame {
            production
                .advance(AdvanceInput::Departure(input))
                .map_err(|fault| super::production_failure(cut, fault))?;
        } else {
            inputs.insert(cut, input);
            expected.insert(
                cut,
                observations::native(&previous, &now, &assets.landing, input)?,
            );
        }
        previous = now;
    }
    Ok(AdmittedCase {
        production,
        content,
        rules,
        schedule: inputs.keys().copied().collect(),
        inputs: InputReader::Departure(Tape(inputs)),
        expected: ExpectedReader::Departure(Tape(expected)),
        actual: ActualReader::Departure,
    })
}

fn admit_origin(
    rows: &mut Rows<'_>,
    seed: u64,
    record: &FrameRecord,
    path: &Path,
) -> Result<(Production, ProductionContent, ProductionRules), Outcome> {
    match seed {
        239 if joined::shaft_starts(record)? => joined::admit_shaft(path, record),
        241 if explosion::pending(record)? && explosion::enters_from(&rows.record(240)?)? => {
            joined::admit_seed(path, record)
        }
        _ => Err(data_failure(Some(seed), "departure elevator origin")),
    }
}

pub(super) const fn valid_counts(counts: &super::result::Counts) -> bool {
    // Per cut: five semantic groups and the pipeline. Settled display memory adds
    // four streams; the receiving loader publishes its registers and DMA channels too.
    counts.fields == counts.cuts.saturating_mul(5)
        && counts.outputs >= counts.cuts
        && counts.outputs <= counts.cuts.saturating_mul(9)
        && counts.publications == 0
        && counts.replies == 0
        && counts.pending_boundaries == 0
}

pub(super) fn check_receiving(
    production: &game::production::Production,
    previous: &oracle_record::FrameRecord,
    current: &oracle_record::FrameRecord,
    graphics: &data::assets::landing_site::Graphics,
    input: Input,
    cut: u64,
) -> Result<(), Outcome> {
    super::compared(
        cut,
        &observations::native(previous, current, graphics, input)?.fields,
        &actual(production, cut)?,
    )
}
