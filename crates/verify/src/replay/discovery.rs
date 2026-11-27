//! A single discovery encounter from the preceding text cut through the return to page three.
//! The second cache row continues the first without admission or state replacement.

mod observations;

use super::{
    Sources,
    compare::Observation,
    data_failure,
    index::Case,
    intro_sources::{self, Join},
    readers::{self, CutTape, ExpectedCut},
    result::Outcome,
};
use data::rom::Rom;
use game::production::{
    Production, ProductionContent, ProductionRules, ProductionSeed,
    played::discovery::{
        Cut, Discovery, DiscoveryOrder, DiscoveryState, Frame, IntroRequest, Publication, RoomView,
    },
};
use game::state::owned::OwnedState;
use oracle_record::FrameRecord;
use types::Pair;

fn word(r: &FrameRecord, at: usize) -> u16 {
    r.wram
        .get(at..at.saturating_add(2))
        .and_then(|v| v.try_into().ok())
        .map_or(0, u16::from_le_bytes)
}

pub(super) fn completed(previous: &FrameRecord, current: &FrameRecord) -> bool {
    word(current, 0x05b6) != word(previous, 0x05b6)
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
    let cut = if completed(previous, current) {
        Cut::Finished
    } else {
        Cut::Unfinished
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
    let Some(Join::Discovery { segments }) = &case.intro_join else {
        return Err(data_failure(None, "discovery sources"));
    };
    let window = case.cache_window()?;
    let first = *window.cuts.start();
    let lock = super::read_lock(sources)?;
    let bad = || data_failure(None, "discovery player ROM");
    let rom = Rom::new(std::fs::read(&sources.rom).map_err(|_| bad())?).map_err(|_| bad())?;
    let assets = import::discovery::load(rom.lorom())
        .map_err(|_| data_failure(None, "discovery scene import"))?;
    let scene =
        Discovery::new(assets).map_err(|_| data_failure(None, "discovery scene admission"))?;
    let (rows, seed) = intro_sources::origin(case, sources, &lock)?;
    if seed.wram.len() != 0x2_0000
        || !super::played::intro::waiting_for_room(
            &seed.wram,
            game::production::played::intro_rooms::Start::Discovery,
        )?
        || word(&seed, 0x1a57) != 0
        || word(&seed, 0x1b39) != 0
        || word(&seed, 0x1b37) != 0
    {
        return Err(data_failure(
            Some(window.origin),
            "dormant discovery origin",
        ));
    }
    let mut inputs = Vec::new();
    let mut expected = Vec::new();
    let mut native = observations::Native::default();
    intro_sources::visit(
        rows,
        seed,
        segments,
        window.cuts.clone(),
        |cut, previous, current| {
            let frame = input(previous, current);
            if (frame.request == IntroRequest::StartDiscovery) != (cut == first) {
                return Err(data_failure(Some(cut), "discovery room opening"));
            }
            inputs.push(frame);
            expected.push(
                native
                    .at(previous, current)
                    .ok_or_else(|| data_failure(Some(cut), "discovery native observations"))?,
            );
            Ok(())
        },
    )?;
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
        inputs: readers::InputReader::Discovery(CutTape {
            first,
            values: inputs,
        }),
        expected: readers::ExpectedReader::RecordedCuts(CutTape {
            first,
            values: expected,
        }),
        actual: readers::ActualReader::Discovery,
        schedule: window.cuts.collect(),
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
        sound: Vec<game::production::played::front_end::Sound>,
        cut: u64,
    ) -> Result<(), Outcome> {
        let sent = sound.into_iter().filter_map(|offer| {
            let game::production::played::front_end::Sound::Discovery(offer) = offer else {
                return None;
            };
            Some(offer)
        });
        let expected_offers = publication
            .into_iter()
            .flat_map(|cut| cut.sounds.iter().copied());
        assert!(sent.eq(expected_offers), "discovery sent sounds at {cut}");
        let expected = self
            .0
            .at(previous, current)
            .ok_or_else(|| data_failure(Some(cut), "native discovery cut"))?;
        let actual = actual_fields(state, publication)?;
        super::compared(cut, &expected.fields, &actual)
    }
}
