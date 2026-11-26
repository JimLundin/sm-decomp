//! A single discovery encounter from the preceding text cut through the return to page three.
//! The second cache row continues the first without admission or state replacement.

mod observations;

use super::{
    Sources,
    compare::Observation,
    data_failure,
    index::Case,
    readers::{self, ExpectedCut},
    result::Outcome,
};
use crate::{
    cache::Cache,
    lock::{Lock, RowId},
};
use data::rom::Rom;
use game::production::{
    AdvanceInput, Production, ProductionContent, ProductionOutputs, ProductionRules,
    ProductionSeed,
    played::discovery::{
        Cut, Discovery, DiscoveryOrder, DiscoveryState, Frame, IntroRequest, Publication, RoomView,
    },
};
use game::state::owned::OwnedState;
use oracle_record::FrameRecord;
use types::Pair;

pub(super) const ROW: &str = "any-intro-2219";
pub(super) const SEED: u64 = 1297;
const LAST: u64 = 2103;
const RETURN: &str = "any-intro-4323";
pub(super) const END: u64 = LAST + 1 + 129;

pub(super) struct Inputs(Vec<Frame>);
pub(super) struct Expectations(Vec<ExpectedCut>);

fn index(cut: u64) -> Option<usize> {
    usize::try_from(cut.checked_sub(SEED + 1)?).ok()
}

impl Inputs {
    pub(in crate::replay) fn advance(
        &self,
        prod: &mut Production,
        cut: u64,
    ) -> Result<ProductionOutputs, Outcome> {
        let frame = index(cut)
            .and_then(|i| self.0.get(i))
            .ok_or_else(|| data_failure(Some(cut), "discovery input cut"))?;
        prod.advance(AdvanceInput::Discovery(*frame))
            .map_err(|_| data_failure(Some(cut), "discovery production turn"))
    }
}

impl Expectations {
    pub(in crate::replay) fn at(&self, cut: u64) -> Result<ExpectedCut, Outcome> {
        index(cut)
            .and_then(|i| self.0.get(i))
            .cloned()
            .ok_or_else(|| data_failure(Some(cut), "discovery expected cut"))
    }
}

fn word(r: &FrameRecord, at: usize) -> u16 {
    r.wram
        .get(at..at.saturating_add(2))
        .and_then(|v| v.try_into().ok())
        .map_or(0, u16::from_le_bytes)
}

fn input(previous: &FrameRecord, current: &FrameRecord) -> Frame {
    let stage = word(current, 0x1f51);
    let request = if stage == 0xb250 && stage != word(previous, 0x1f51) {
        IntroRequest::StartDiscovery
    } else if stage == 0xb3f4 && word(current, 0x1a4b) == 0 {
        IntroRequest::FinishDiscovery
    } else {
        IntroRequest::None
    };
    let cut = if word(current, 0x05b6) == word(previous, 0x05b6) {
        Cut::Unfinished
    } else {
        Cut::Finished
    };
    let point = |x, y| Pair {
        x: i32::from(word(current, x).cast_signed()),
        y: i32::from(word(current, y).cast_signed()),
    };
    Frame {
        cut,
        request,
        room: RoomView {
            centre: point(0x0af6, 0x0afa),
            scroll: point(0x0911, 0x0915),
        },
    }
}

pub(super) fn admit(case: &Case, sources: &Sources) -> Result<readers::AdmittedCase, Outcome> {
    let bad = || data_failure(None, "discovery locked inputs");
    let lock = Lock::parse(&std::fs::read_to_string(&sources.lock).map_err(|_| bad())?)
        .map_err(|_| bad())?;
    let open = |id: &str| {
        let row = lock.row(&RowId(id.to_owned())).ok_or_else(bad)?;
        Cache::open(&sources.cache, row).map_err(|_| bad())
    };
    let rom = Rom::new(std::fs::read(&sources.rom).map_err(|_| bad())?).map_err(|_| bad())?;
    let assets = import::discovery::load(rom.lorom())
        .map_err(|_| data_failure(None, "discovery scene import"))?;
    let scene =
        Discovery::new(assets).map_err(|_| data_failure(None, "discovery scene admission"))?;
    let intro = open(&case.row)?;
    let returning = open(RETURN)?;
    let mut previous = readers::record(&intro, SEED)?;
    if previous.wram.len() != 0x2_0000
        || word(&previous, 0x1b39) != 0
        || word(&previous, 0x1b37) != 0
    {
        return Err(data_failure(Some(SEED), "dormant discovery origin"));
    }
    let mut inputs = Vec::new();
    let mut expected = Vec::new();
    let mut native = observations::Native::default();
    for (cache, start, end, base) in [(&intro, SEED + 1, LAST, 0), (&returning, 0, 129, LAST + 1)] {
        let span =
            usize::try_from(end.saturating_sub(start).saturating_add(1)).map_err(|_| bad())?;
        for (wanted, record) in (start..=end).zip(
            cache
                .records_without_pictures()
                .skip(usize::try_from(start).map_err(|_| bad())?)
                .take(span),
        ) {
            let (at, current) = record.map_err(|_| bad())?;
            let cut = base.saturating_add(wanted);
            if at.0 != wanted || current.wram.len() != 0x2_0000 {
                return Err(data_failure(Some(cut), "discovery cache record"));
            }
            inputs.push(input(&previous, &current));
            expected.push(
                native
                    .at(&previous, &current)
                    .ok_or_else(|| data_failure(Some(cut), "discovery native observations"))?,
            );
            previous = current;
        }
    }
    let schedule: Vec<_> = (SEED + 1..=case.end.cut_inclusive).collect();
    if expected.len() != schedule.len() {
        return Err(bad());
    }
    let content = ProductionContent::Discovery(Box::new(scene));
    let rules = ProductionRules::Discovery;
    let production = Production::admit(
        ProductionSeed::Discovery(Box::new(DiscoveryState::dormant())),
        content.clone(),
        rules.clone(),
    )
    .map_err(|_| data_failure(None, "discovery production admission"))?;
    Ok(readers::AdmittedCase {
        production,
        content,
        rules,
        inputs: readers::InputReader::Discovery(Inputs(inputs)),
        expected: readers::ExpectedReader::Discovery(Expectations(expected)),
        actual: readers::ActualReader::Discovery,
        schedule,
    })
}

/// Native discovery state projected against the encounter playing inside the front end.
#[derive(Clone, Default)]
pub(super) struct Played(observations::Native);

impl Played {
    pub(super) fn compared(
        &mut self,
        previous: &FrameRecord,
        current: &FrameRecord,
        state: Option<&DiscoveryState>,
        finished: bool,
    ) -> Result<(Vec<Observation>, Vec<Observation>), Outcome> {
        let visible = |record: &FrameRecord| {
            Ok::<_, Outcome>(
                word(record, 0x1a57) != 0
                    && super::played::intro::delivery::discovery_selected(&record.wram)?,
            )
        };
        if !visible(previous)? && !visible(current)? {
            return Ok((Vec::new(), Vec::new()));
        }
        let expected = self
            .0
            .compared_state(previous, current)
            .ok_or_else(|| data_failure(None, "played discovery native observations"))?;
        if !finished {
            return Ok((Vec::new(), Vec::new()));
        }
        let actual = state.map_or_else(
            || observations::state(&DiscoveryState::dormant()),
            observations::state,
        );
        Ok((expected, actual))
    }
}

pub(super) fn actual(
    prod: &Production,
    output: Option<&Publication>,
) -> Result<Vec<Observation>, Outcome> {
    let OwnedState::Discovery(state) = prod.state() else {
        return Err(data_failure(None, "discovery owned state"));
    };
    actual_fields(&state.0, output)
}

fn actual_fields(
    state: &DiscoveryState,
    output: Option<&Publication>,
) -> Result<Vec<Observation>, Outcome> {
    let mut fields = observations::state(state);
    let sounds = output
        .into_iter()
        .flat_map(|o| &o.sounds)
        .map(|s| (s.sound.0, s.allowance.0));
    fields.extend(observations::effects(
        sounds,
        output.is_some_and(|o| o.cue.is_some()),
        output.is_some_and(|o| o.room_ran),
    ));
    let drawing = if let Some(publication) = output {
        observations::draw(publication)?
    } else {
        None
    };
    fields.push(observations::picture(drawing));
    Ok(fields)
}

#[cfg(test)]
#[derive(Default)]
pub(super) struct PlayedWitness(observations::Native);

#[cfg(test)]
impl PlayedWitness {
    pub(super) fn check(
        &mut self,
        previous: &FrameRecord,
        current: &FrameRecord,
        state: &DiscoveryState,
        publication: Option<&Publication>,
        cut: u64,
    ) -> Result<(), Outcome> {
        let expected = self
            .0
            .at(previous, current)
            .ok_or_else(|| data_failure(Some(cut), "native discovery cut"))?;
        let actual = actual_fields(state, publication)?;
        super::compared(cut, &expected.fields, &actual)
    }
}
