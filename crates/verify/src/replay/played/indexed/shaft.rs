//! Indexed shaft actors from the carried Ceres return.

use super::{
    Case, CaseKind, ContentProvider, ExpectedCut, ExpectedProvider, ObservationId, Outcome,
    Responsibility, Start, data_failure, runner,
};
use crate::replay::{
    played::{at, frame, shaft_restore::Checkpoint},
    readers::FieldRole,
    result::Counts,
};

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
    room: crate::replay::played::indexed::AdmittedRoom<'_, '_>,
    case: &Case,
    result: &mut runner::Receipt,
) -> Result<(), Outcome> {
    let first = case
        .first_cut()
        .ok_or_else(|| data_failure(None, "shaft actor comparison window"))?;
    room.replay(case, |run, turn, _| {
        let cut = turn.cut;
        if cut < first {
            turn.advance(run).map_err(|outcome| at(cut, outcome))?;
        } else {
            let mut frozen = super::super::shaft_native::actual(&run.state, None);
            frozen.push(frame::random_observation(run.state.frame.random));
            frozen.extend(super::super::room_video::held_shaft(
                &run.state.room_video,
                run.witnesses.room_video()?.overlay_pieces,
                cut,
            )?);
            let pending = run.state.room_video.pending_platform().is_some();
            let mut restored = Checkpoint::capture(run)
                .and_then(|saved| saved.restore(run))
                .map_err(|_| Outcome::CheckpointFailure { cut })?;
            let direct = turn.advance(run).map_err(|outcome| at(cut, outcome))?;
            turn.check_continuation(&mut restored, run, &direct)
                .map_err(|outcome| at(cut, outcome))?;
            let (fields, _) = direct;
            let observed = ExpectedCut::select(fields, |field| {
                if semantic(field) {
                    Some(FieldRole::Semantic)
                } else {
                    (event(field) || field.starts_with("room-video.shaft-"))
                        .then_some(FieldRole::Output)
                }
            });
            if observed.semantic_count != SEMANTICS
                || observed
                    .fields
                    .iter()
                    .filter(|field| event(&field.field))
                    .count()
                    != EVENTS
            {
                return Err(data_failure(
                    Some(cut),
                    "shaft actor comparison opportunity",
                ));
            }
            result.record(&observed, &frozen, cut)?;

            result.restored(1)?;
            if pending {
                result.pending()?;
                result.resumed()?;
            }
        }
        Ok(())
    })?;
    if !valid_counts(case, &result.counts) {
        return Err(data_failure(
            Some(case.end.cut_inclusive),
            "shaft actor picture and pending witnesses",
        ));
    }
    Ok(())
}
