//! Indexed shaft actors from the carried Ceres return.

use super::{
    Case, CaseKind, CaseResult, ContentProvider, ExpectedCut, ExpectedProvider, ObservationId,
    Outcome, Responsibility, Run, Start, data_failure, runner,
};
use crate::replay::{
    compare::Observation,
    played::{Controller, at, frame, polled, rows::Rows, shaft_restore::Checkpoint},
    result::Counts,
};
use oracle_record::FrameRecord;

use crate::replay::index::PlayedWindow;
const SEMANTICS: usize = 5;
const EVENTS: usize = 5;

pub(super) fn valid(case: &Case) -> bool {
    case.kind == CaseKind::Integrated
        && matches!((&case.start, &case.expectations.played),
            (Start::CacheCut { frame }, Some(PlayedWindow::ShaftActors { first, picture_fields }))
            if frame < first && *first <= case.end.cut_inclusive && *picture_fields > 0)
        && case.responsibilities == [Responsibility::Ceres]
        && case.content == ContentProvider::PlayerRomImport
        && case.expectations.provider == ExpectedProvider::PinnedRecordCache
        && case.expectations.fields == [ObservationId::CeresShaftState]
        && case.expectations.outputs
            == [
                ObservationId::CeresShaftEvents,
                ObservationId::CeresShaftPicture,
            ]
        && case.expectations.output_cut.is_none()
        && case.checkpoints
}

pub(in crate::replay) const fn valid_counts(case: &Case, counts: &Counts) -> bool {
    let Some(PlayedWindow::ShaftActors { picture_fields, .. }) = case.expectations.played else {
        return false;
    };
    counts.fields == counts.cuts.saturating_mul(SEMANTICS)
        && counts.outputs
            == counts
                .cuts
                .saturating_mul(EVENTS)
                .saturating_add(picture_fields)
        && counts.publications == 0
        && counts.replies == 0
        && counts.pending_boundaries == 1
}

fn semantic(field: &str) -> bool {
    matches!(
        field,
        "random"
            | "shaft-actors.overlay"
            | "shaft-actors.mounted-offsets"
            | "shaft-actors.rotation"
            | "shaft-actors.locked"
    )
}

fn event(field: &str) -> bool {
    matches!(
        field,
        "shaft-actors.placed"
            | "shaft-actors.reach"
            | "shaft-actors.burst-particle"
            | "shaft-actors.burst-sound-limit"
            | "shaft-actors.lights"
    )
}

pub(super) fn run(
    run: &mut Run<'_>,
    rows: &mut Rows<'_>,
    mut previous: FrameRecord,
    mut controller: Controller,
    case: &Case,
    result: &mut CaseResult,
) -> Result<(), Outcome> {
    let first = case
        .first_cut()
        .ok_or_else(|| data_failure(None, "shaft actor comparison window"))?;
    let mut digest = Vec::new();
    for cut in previous.index.saturating_add(1)..=case.end.cut_inclusive {
        let current = rows.record(cut)?;
        if current.video_frame != previous.video_frame.wrapping_add(1) {
            return Err(data_failure(Some(cut), "played rows join"));
        }
        let (row, local) = rows.place(cut)?;
        let place = (row.as_str(), local);
        let step =
            |run: &mut Run<'_>| run.step_with_outputs(controller, &previous, &current, cut, place);
        if cut < first {
            step(run).map_err(|outcome| at(cut, outcome))?;
        } else {
            let mut frozen = super::super::shaft_native::actual(&run.state, None);
            frozen.push(frame::random_observation(run.state.frame.random));
            frozen.extend(super::super::room_video::held_shaft(
                &run.state.room_video,
                cut,
            )?);
            let pending = run.state.room_video.pending_platform().is_some();
            let mut restored = Checkpoint::capture(run)
                .and_then(|saved| saved.restore(run))
                .map_err(|_| Outcome::CheckpointFailure { cut })?;
            let (mut fields, outputs) = step(run).map_err(|outcome| at(cut, outcome))?;
            let (replayed, effects) = step(&mut restored).map_err(|outcome| at(cut, outcome))?;
            if replayed != fields
                || effects != outputs
                || restored.state != run.state
                || restored.return_timing != run.return_timing
                || restored.warning_picture != run.warning_picture
            {
                return Err(Outcome::CheckpointFailure { cut });
            }
            fields.retain(|field| {
                semantic(&field.field)
                    || event(&field.field)
                    || field.field.starts_with("room-video.shaft-")
            });
            let semantics: Vec<Observation> = fields
                .iter()
                .filter(|field| semantic(&field.field))
                .cloned()
                .collect();
            if semantics.len() != SEMANTICS
                || fields.iter().filter(|field| event(&field.field)).count() != EVENTS
            {
                return Err(data_failure(
                    Some(cut),
                    "shaft actor comparison opportunity",
                ));
            }
            let observed = ExpectedCut {
                semantic_count: semantics.len(),
                output_count: fields.len().saturating_sub(semantics.len()),
                semantics,
                fields,
                replies: 0,
            };
            runner::record_cut(result, &mut digest, &observed, &frozen, cut)?;
            result.observation_sha256 = crate::lock::Sha256::of(&digest).to_string();
            result.counts.restorations = result.counts.restorations.saturating_add(1);
            if pending {
                result.counts.pending_boundaries =
                    result.counts.pending_boundaries.saturating_add(1);
                result.counts.pending_restorations =
                    result.counts.pending_restorations.saturating_add(1);
            }
        }
        controller = controller.read(current.pad.0, polled(&current));
        previous = current;
    }
    if !valid_counts(case, &result.counts) {
        return Err(data_failure(
            Some(case.end.cut_inclusive),
            "shaft actor picture and pending witnesses",
        ));
    }
    Ok(())
}
