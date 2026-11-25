//! Scoped replay of the production encounter. The room, intro, platform and weapons are
//! admitted at their encounter boundaries. Expected encounter State never seeds a later cut.

mod expected;
mod inputs;

use super::{Sources, compare::Observation, data_failure, index::Case, readers, result::Outcome};
use crate::{
    cache::Cache,
    lock::{Lock, RowId},
};
use data::rom::Rom;
use game::{
    production::{
        Production, ProductionContent, ProductionRules, ProductionSeed,
        played::flashback::{EncounterState, Publication, RoomBackgroundY, Rules, publish_flash},
    },
    state::owned::OwnedState,
};
use oracle_record::FrameRecord;
use types::{Colour, PALETTE_SLOTS, Palette, ROW_COLOURS};

pub(super) use expected::Expectations;
pub(super) use inputs::Inputs;

pub(super) const ROW: &str = "any-intro-2219";
pub(super) const END: u64 = 650;
const HANDOFF_ROW: &str = "any-intro-0568";
const HANDOFF_CUT: u64 = 1650;

pub(super) fn admit(case: &Case, sources: &Sources) -> Result<readers::AdmittedCase, Outcome> {
    let lock = std::fs::read_to_string(&sources.lock)
        .map_err(|_| data_failure(None, "flashback record lock file"))?;
    let lock =
        Lock::parse(&lock).map_err(|_| data_failure(None, "flashback record lock schema"))?;
    let open = |id: &str| {
        let row = lock
            .row(&RowId(id.to_owned()))
            .ok_or_else(|| data_failure(None, "flashback indexed lock row"))?;
        Cache::open(&sources.cache, row)
            .map_err(|_| data_failure(None, "flashback record cache pins"))
    };
    let bad_rom = || data_failure(None, "flashback player ROM");
    let image =
        Rom::new(std::fs::read(&sources.rom).map_err(|_| bad_rom())?).map_err(|_| bad_rom())?;
    let rom = image.lorom();
    let art = import::flashback::load(rom)
        .map_err(|_| data_failure(None, "flashback clip and palette import"))?;
    let handoff = readers::record(&open(HANDOFF_ROW)?, HANDOFF_CUT)?;
    let cache = open(&case.row)?;
    let schedule: Vec<_> = (0..=case.end.cut_inclusive).collect();
    let mut stimuli = Inputs::new(&handoff)?;
    let mut expected = Expectations::new(rom)?;
    let mut previous = handoff;
    let mut last_turn = None;
    for (&cut, current) in schedule.iter().zip(cache.records_without_pictures()) {
        let (actual_cut, current) =
            current.map_err(|_| data_failure(Some(cut), "flashback record bytes"))?;
        if actual_cut.0 != cut || current.wram.len() != 0x2_0000 {
            return Err(data_failure(Some(cut), "flashback record shape"));
        }
        let timing = inputs::timing(&previous, &current, cut)?;
        let prior_turn = last_turn.as_ref().unwrap_or(&previous);
        stimuli.read(prior_turn, &current, timing, cut)?;
        expected.read_record(prior_turn, &current, timing, cut)?;
        if timing.takes_turn() {
            last_turn = Some(current.clone());
        }
        previous = current;
    }

    if previous.index != case.end.cut_inclusive {
        return Err(data_failure(None, "complete flashback record horizon"));
    }

    let content = ProductionContent::Flashback(Box::new(art));
    let rules = ProductionRules::Flashback(Rules::REFERENCE);
    let production = Production::admit(
        ProductionSeed::Flashback(Box::new(EncounterState::before_encounter(RoomBackgroundY(
            0,
        )))),
        content.clone(),
        rules.clone(),
    )
    .map_err(|_| data_failure(None, "flashback production admission"))?;

    Ok(readers::AdmittedCase {
        production,
        content,
        rules,
        inputs: readers::InputReader::Flashback(stimuli),
        expected: readers::ExpectedReader::Flashback(expected),
        actual: readers::ActualReader::Flashback,
        schedule,
    })
}

pub(super) fn actual(
    production: &Production,
    output: &Publication,
    cut: Option<u64>,
) -> Result<Vec<Observation>, Outcome> {
    let OwnedState::Flashback(state) = production.state() else {
        return Err(data_failure(cut, "flashback owned scope"));
    };
    let mut observations = expected::state(&state.0);
    observations.extend(expected::effects(output.effects));
    let colours = if let Some(colours) = output.colours {
        let mut palette = Palette([Colour::BLACK; PALETTE_SLOTS]);
        publish_flash(&mut palette, colours);
        let [.., row] = palette.0.as_chunks::<ROW_COLOURS>().0 else {
            return Err(data_failure(cut, "flashback published palette row"));
        };
        Some(row.map(u16::from))
    } else {
        None
    };
    observations.extend(expected::palette(colours));
    observations.push(expected::oam(output.draw.as_ref().map(expected::draw)));
    observations.push(expected::background(output.background_y.map(|y| y.0)));
    Ok(observations)
}

fn word(record: &FrameRecord, address: usize) -> u16 {
    let bytes = record.wram.get(address..address.saturating_add(2));
    bytes
        .and_then(|bytes| bytes.try_into().ok())
        .map_or(0, u16::from_le_bytes)
}
