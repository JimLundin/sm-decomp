//! The scoped passage draw witness borrows timing and sampled controls from the pinned row.

use super::{
    Sources, compare::Observation, data_failure, index::Case, movement, played, readers,
    result::Outcome,
};
use crate::{cache::Cache, lock::RowId, source};
use game::{
    production::{
        AdvanceInput, Production, ProductionContent, ProductionOutputs, ProductionRules,
        ProductionSeed,
    },
    state::owned::OwnedState,
};
use oracle_record::FrameRecord;
use samus_movement::{DrawnJumpHistory, FrameBefore, JumpButton, JumpHeld};

const PHASE: source::WramWord = source::WramWord {
    name: "passage phase",
    offset: 0x099c,
};
const JUMP_BINDING: source::WramWord = source::WramWord {
    name: "jump binding",
    offset: 0x09b4,
};

pub(super) fn admit(case: &Case, sources: &Sources) -> Result<readers::AdmittedCase, Outcome> {
    let window = case.cache_window()?;
    let lock = super::read_lock(sources)?;
    let row = lock
        .row(&RowId(case.row.clone()))
        .ok_or_else(|| data_failure(None, "jump history lock row"))?;
    if case.end.cut_inclusive >= row.frames.0 {
        return Err(data_failure(
            Some(case.end.cut_inclusive),
            "jump history row extent",
        ));
    }
    let cache = Cache::open(&sources.cache, row)
        .map_err(|_| data_failure(None, "jump history record pins"))?;
    let mut previous = readers::record(&cache, window.origin)?;
    let seed = observed(&previous.wram)?;
    let content = ProductionContent::JumpHistory;
    let rules = ProductionRules::JumpHistory;
    let production = Production::admit(
        ProductionSeed::JumpHistory(seed),
        content.clone(),
        rules.clone(),
    )
    .map_err(|_| data_failure(Some(window.origin), "jump history production admission"))?;
    let first = *window.cuts.start();
    let mut samples = Vec::new();
    let mut expected = Vec::new();
    for cut in window.cuts.clone() {
        let current = readers::record(&cache, cut)?;
        samples.push(sample(&previous, &current)?);
        let fields = parts(observed(&current.wram)?);
        expected.push(readers::ExpectedCut {
            semantic_count: fields.len(),
            output_count: 0,
            replies: 0,
            semantics: fields.clone(),
            fields,
        });
        previous = current;
    }
    if samples.iter().all(Option::is_none) {
        return Err(data_failure(
            None,
            "passage window has no history-writing draw",
        ));
    }
    Ok(readers::AdmittedCase {
        production,
        content,
        rules,
        schedule: window.cuts.collect(),
        inputs: readers::InputReader::JumpHistory(readers::Tape::new(first, samples)),
        actual: readers::ActualReader::JumpHistory,
        expected: readers::ExpectedReader::JumpHistory(readers::Tape::new(first, expected)),
    })
}

fn read(record: &FrameRecord, word: source::WramWord) -> Result<u16, Outcome> {
    word.read(&record.wram)
        .map_err(|_| data_failure(Some(record.index), word.name))
}

fn observed(wram: &[u8]) -> Result<DrawnJumpHistory, Outcome> {
    Ok(DrawnJumpHistory {
        jump_held: movement::jump_held(wram)?,
        last_draw: played::samus::last_draw(wram)?,
    })
}

fn sample(previous: &FrameRecord, current: &FrameRecord) -> Result<Option<JumpButton>, Outcome> {
    let complete = played::platform::finished(current);
    let drawn = match read(previous, PHASE)? {
        0xe29e => complete,
        0xe2db => complete && read(current, PHASE)? == 0xe2db,
        0xe737 => played::platform::finished(previous),
        _ => false,
    };
    if !drawn {
        return Ok(None);
    }
    let buttons = read(previous, source::JOYPAD1_LASTKEYS)?;
    let binding = read(previous, JUMP_BINDING)?;
    Ok(Some(if buttons & binding != 0 {
        JumpButton::Held
    } else {
        JumpButton::Up
    }))
}

pub(super) fn advance(
    tape: &readers::Tape<Option<JumpButton>>,
    production: &mut Production,
    cut: u64,
) -> Result<ProductionOutputs, Outcome> {
    production
        .advance(AdvanceInput::JumpHistory(*tape.at(cut)?))
        .map_err(|fault| super::production_failure(cut, fault))
}

pub(super) fn actual(production: &Production) -> Result<Vec<Observation>, Outcome> {
    let OwnedState::JumpHistory(state) = production.state() else {
        return Err(data_failure(None, "jump history owned scope"));
    };
    Ok(parts(state.0))
}

fn parts(state: DrawnJumpHistory) -> Vec<Observation> {
    vec![
        jump_part(state.jump_held),
        played::samus::last_draw_observation(state.last_draw),
    ]
}

pub(super) fn jump_part(held: JumpHeld) -> Observation {
    let mut bytes = held.frames.0.to_le_bytes().to_vec();
    bytes.push(u8::from(held.before == FrameBefore::LandedOrEnded));
    Observation {
        field: "movement.drawn-jump-held".to_owned(),
        bytes,
    }
}
