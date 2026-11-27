//! Scoped replay of the production encounter. The room, intro, platform and weapons are
//! admitted at their encounter boundaries. Expected encounter State never seeds a later cut.

mod expected;
mod inputs;

use super::intro_sources::{self, Join};
use super::{Sources, compare::Observation, data_failure, index::Case, readers, result::Outcome};
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

pub(super) use inputs::Inputs;

pub(super) fn admit(case: &Case, sources: &Sources) -> Result<readers::AdmittedCase, Outcome> {
    let Some(Join::Flashback { handoff, segments }) = &case.intro_join else {
        return Err(data_failure(None, "flashback sources"));
    };
    let first = case
        .first_cut()
        .ok_or_else(|| data_failure(None, "flashback first comparison"))?;
    let cuts = first..=case.end.cut_inclusive;
    let lock = super::read_lock(sources)?;
    let (rows, press) = handoff.before(case, first, sources, &lock)?;
    let bad_rom = || data_failure(None, "flashback player ROM");
    let image =
        Rom::new(std::fs::read(&sources.rom).map_err(|_| bad_rom())?).map_err(|_| bad_rom())?;
    let rom = image.lorom();
    let art = import::flashback::load(rom)
        .map_err(|_| data_failure(None, "flashback clip and palette import"))?;
    let mut stimuli = Inputs::new(&press, first)?;
    let native = expected::Expectations::new(rom)?;
    let mut expected = Vec::new();
    let mut last_turn = None;
    intro_sources::visit(
        rows,
        press,
        segments,
        cuts.clone(),
        |cut, prior, current| {
            let timing = inputs::timing(prior, current, cut, cut == first)?;
            let previous_turn = last_turn.as_ref().unwrap_or(prior);
            stimuli.read(previous_turn, current, timing, cut)?;
            expected.push(native.read_record(prior, current, cut)?);
            if timing.takes_turn() {
                last_turn = Some(current.clone());
            }
            Ok(())
        },
    )?;

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
        expected: readers::ExpectedReader::RecordedCuts(readers::CutTape {
            first,
            values: expected,
        }),
        actual: readers::ActualReader::Flashback,
        schedule: cuts.collect(),
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

/// State the encounter publishes at a joined intro cut. A pending native start still has
/// its first scripts installed without pictures.
pub(super) fn played_state(
    record: &FrameRecord,
    shown: bool,
    encounter: Option<&types::flashback::EncounterState>,
) -> Result<(Vec<Observation>, Vec<Observation>), Outcome> {
    let script = word(record, 0x1b1d);
    let started = shown && script != 0;
    let field = "intro-encounter.started";
    let mut expected = vec![Observation::quantity(field, i64::from(started))];
    let mut actual = vec![Observation::quantity(field, i64::from(encounter.is_some()))];
    if started {
        let pending = script == 0xcb05 && word(record, 0x1a5d) == 0;
        let observed = expected::decode(record, pending)
            .ok_or_else(|| data_failure(None, "played intro encounter"))?;
        expected.extend(expected::state(&observed));
    }
    if let Some(encounter) = encounter {
        actual.extend(expected::state(encounter));
    }
    Ok((expected, actual))
}

#[test]
#[ignore = "pinned record cache and player ROM"]
fn an_observed_actor_turn_survives_a_held_input_clock() {
    let sources = super::played::tests::sources();
    let lock = super::read_lock(&sources).unwrap();
    let index = super::index::Index::parse(include_str!("../../../../scenarios/replay/index.toml"))
        .unwrap();
    let case = index
        .cases
        .iter()
        .find(|case| case.id.0 == "flashback/zebes-encounter")
        .unwrap();
    let (mut rows, pending) = intro_sources::origin(case, &sources, &lock).unwrap();
    let first_turn = pending.index.checked_add(1).unwrap();
    let mut current = rows.record(first_turn).unwrap();
    let clock = inputs::CINEMATIC_FRAME..inputs::CINEMATIC_FRAME + 2;
    current.wram[clock.clone()].copy_from_slice(&pending.wram[clock]);
    assert!(
        !inputs::timing(&pending, &current, first_turn, false)
            .unwrap()
            .takes_turn()
    );
    let rom = Rom::new(std::fs::read(&sources.rom).unwrap()).unwrap();
    let native = expected::Expectations::new(rom.lorom()).unwrap();
    let receipt = native.read_record(&pending, &current, first_turn).unwrap();
    assert!(
        receipt
            .fields
            .contains(&Observation::quantity("flashback.turn", 1))
    );
}
