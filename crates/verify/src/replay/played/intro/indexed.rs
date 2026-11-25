//! Indexed intro play shares the production frame adapter and its comparison projections.
//! The input tape carries the encounter reply that production consumes between the stage
//! and script steps.

use super::{
    Controller, FrameEnd, Part, Scenes, finished, frame_input, intro, polled, shown, word,
};
use crate::replay::compare::Observation;
use crate::replay::index::{Case, Start};
use crate::replay::played::{at, open_rows, read_lock};
use crate::replay::readers::{
    ActualReader, AdmittedCase, ExpectedCut, ExpectedReader, InputReader,
};
use crate::replay::{Counts, Outcome, Sources, data_failure, production_failure};
use game::production::{
    AdvanceInput, Production, ProductionContent, ProductionOutputs, ProductionRules, ProductionSeed,
};
use game::state::owned::OwnedState;
use screens_intro::{Language, Report};

pub(in crate::replay) struct Inputs {
    first: u64,
    frames: Vec<FrameEnd>,
}

impl Inputs {
    pub(in crate::replay) fn advance(
        &self,
        production: &mut Production,
        cut: u64,
    ) -> Result<ProductionOutputs, Outcome> {
        let frame = lookup(&self.frames, self.first, cut)?;
        production
            .advance(AdvanceInput::Intro(*frame))
            .map_err(|fault| production_failure(cut, fault))
    }
}

pub(in crate::replay) struct Expected {
    first: u64,
    cuts: Vec<ExpectedCut>,
}

impl Expected {
    pub(in crate::replay) fn read(&self, cut: u64) -> Result<ExpectedCut, Outcome> {
        lookup(&self.cuts, self.first, cut).cloned()
    }
}

fn lookup<T>(values: &[T], first: u64, cut: u64) -> Result<&T, Outcome> {
    cut.checked_sub(first)
        .and_then(|offset| usize::try_from(offset).ok())
        .and_then(|offset| values.get(offset))
        .ok_or_else(|| data_failure(Some(cut), "indexed intro cut"))
}

pub(in crate::replay) fn actual(
    production: &Production,
    report: Option<&Report>,
) -> Result<Vec<Observation>, Outcome> {
    let OwnedState::Intro(state) = production.state() else {
        return Err(data_failure(None, "intro owned scope"));
    };
    let (mut fields, outputs) = super::actual(Part::Flashback, state, report);
    fields.extend(outputs);
    Ok(fields)
}

pub(in crate::replay) fn initial(production: &Production) -> Result<Vec<Observation>, Outcome> {
    let OwnedState::Intro(state) = production.state() else {
        return Err(data_failure(None, "intro null scope"));
    };
    let mut fields = super::observations(&super::View {
        part: Part::Flashback,
        state: &state.intro,
        colours: &state.colours,
    });
    fields.extend(super::actual(Part::Flashback, state, None).1);
    Ok(fields)
}

/// The grouped State and output observations in the pinned flashback schedule.
pub(in crate::replay) fn valid_count_shape(counts: &Counts) -> bool {
    const FINISHED: usize = 1293;
    const INTERRUPTED: usize = 5;
    const PALETTE_PUBLICATIONS: usize = 66;
    const SPRITE_SEVEN_PUBLICATIONS: usize = 64;
    (counts == &Counts::default()
        || (counts.cuts == FINISHED + INTERRUPTED
            && counts.fields == FINISHED * 29 + INTERRUPTED * 2
            && counts.outputs
                == FINISHED * 7 + INTERRUPTED + PALETTE_PUBLICATIONS + SPRITE_SEVEN_PUBLICATIONS))
        && counts.publications == 0
        && counts.replies == 0
        && counts.pending_boundaries == 0
}

/// Read each pinned cut once. Only the input tape enters production.
pub(in crate::replay) fn admit(case: &Case, sources: &Sources) -> Result<AdmittedCase, Outcome> {
    let Start::CacheCut { frame: start } = case.start else {
        return Err(data_failure(None, "indexed intro seed"));
    };
    let lock = read_lock(sources)?;
    let mut rows = open_rows(sources, &lock, &case.row)?;
    let seed = rows.record(start)?;
    let Scenes {
        content,
        mut reading,
    } = Scenes::import(&sources.rom, Part::Flashback)?;
    if !finished(&seed) || word(&seed.wram, super::LANGUAGE)? != 0 {
        return Err(data_failure(
            Some(start),
            "English intro seed at finished frame",
        ));
    }
    let state = reading.state(&seed.wram)?;
    if state.stage != types::intro::IntroStage::PageOneWaits {
        return Err(data_failure(Some(start), "intro seed waits after page 1"));
    }
    let state = intro::admit(Language::English, state, shown(&seed.wram)?)
        .map_err(|_| data_failure(Some(start), "indexed intro admission"))?;
    let mut controller = Controller::seeded(&seed.wram)?;
    let mut began = seed.clone();
    let mut previous = seed;
    let first = start.saturating_add(1);
    let schedule: Vec<_> = (first..=case.end.cut_inclusive).collect();
    let mut inputs = Inputs {
        first,
        frames: Vec::with_capacity(schedule.len()),
    };
    let mut expected = Expected {
        first,
        cuts: Vec::with_capacity(schedule.len()),
    };
    for &cut in &schedule {
        let current = rows.record(cut).map_err(|outcome| at(cut, outcome))?;
        if current.video_frame != previous.video_frame.wrapping_add(1) {
            return Err(data_failure(Some(cut), "indexed intro rows join"));
        }
        inputs.frames.push(
            frame_input(Part::Flashback, controller, &began, &previous, &current)
                .map_err(|outcome| at(cut, outcome))?,
        );
        expected.cuts.push(
            reading
                .expected(&began, &current, &content)
                .map_err(|outcome| at(cut, outcome))?,
        );
        controller = controller.read(current.pad.0, polled(&current));
        if finished(&current) {
            began = current.clone();
        }
        previous = current;
    }
    let content = ProductionContent::Intro(Box::new(content));
    let rules = ProductionRules::Intro(super::RULES);
    let production = Production::admit(
        ProductionSeed::Intro(Box::new(state)),
        content.clone(),
        rules.clone(),
    )
    .map_err(|_| data_failure(Some(start), "indexed intro production"))?;
    Ok(AdmittedCase {
        production,
        content,
        rules,
        actual: ActualReader::Intro,
        schedule,
        inputs: InputReader::Intro(inputs),
        expected: ExpectedReader::Intro(expected),
    })
}
