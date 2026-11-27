//! Indexed Ceres windows through the ordinary played room runner.

pub(in crate::replay) mod shaft;

use super::{Run, Stop, contact, fight_picture, open_rows, read_lock, ridley, seed_controller};
use crate::replay::{
    Sources, appearance,
    compare::Observation,
    connected, data_failure,
    index::{
        Case, CaseKind, ContentProvider, ExpectedProvider, InputProvider, ObservationId,
        Responsibility, Start,
    },
    movement,
    readers::ExpectedCut,
    result::{CaseResult, Outcome},
    runner,
};

const STATE: &[ObservationId] = &[
    ObservationId::CeresTailState,
    ObservationId::CeresTailKnockback,
    ObservationId::CeresTailHealth,
];
const OUTPUTS: &[ObservationId] = &[
    ObservationId::CeresTailPlaces,
    ObservationId::CeresTailHit,
    ObservationId::CeresTailSwish,
];

const RETREAT_STATE: &[ObservationId] = &[
    ObservationId::CeresRetreatBody,
    ObservationId::CeresTailState,
    ObservationId::CeresTailKnockback,
    ObservationId::CeresTailHealth,
];
const RETREAT_OUTPUTS: &[ObservationId] = &[
    ObservationId::CeresRetreatBound,
    ObservationId::CeresBackdropWalls,
    ObservationId::CeresBackdropColours,
    ObservationId::CeresTailPlaces,
    ObservationId::CeresTailHit,
    ObservationId::CeresTailSwish,
];

const FLYBY_STATE: &[ObservationId] = &[ObservationId::CeresFlybyState];
const FLYBY_OUTPUTS: &[ObservationId] = &[
    ObservationId::CeresFlybyPlane,
    ObservationId::CeresFlybyColours,
    ObservationId::CeresFlybyTiles,
    ObservationId::CeresFlybySound,
    ObservationId::CeresFlybyPush,
    ObservationId::CeresFlybyRoom,
];

const WARNING_STATE: &[ObservationId] = &[ObservationId::CeresWarningState];
const WARNING_OUTPUTS: &[ObservationId] = &[
    ObservationId::CeresWarningPicture,
    ObservationId::CeresWarningAudio,
    ObservationId::CeresWarningHandoff,
];

const FIGHT_PICTURE_FIELDS: &[ObservationId] = &[ObservationId::CeresFightDoorCursor];
const FIGHT_PICTURE_OUTPUTS: &[ObservationId] = &[
    ObservationId::CeresFightPreparedOam,
    ObservationId::CeresFightPhysicalOam,
];

pub(in crate::replay) fn valid(case: &Case) -> bool {
    if case.inputs == InputProvider::CeresShaftActors {
        return shaft::valid(case);
    }
    if case.inputs == InputProvider::ScopedCeresFightPicture {
        return case.kind == CaseKind::Scoped
            && fight_picture::valid_window(case)
            && case.responsibilities == [Responsibility::Ceres]
            && case.content == ContentProvider::PlayerRomImport
            && case.expectations.provider == ExpectedProvider::PinnedRecordCache
            && case.expectations.fields == FIGHT_PICTURE_FIELDS
            && case.expectations.outputs == FIGHT_PICTURE_OUTPUTS
            && case.expectations.output_cut.is_none()
            && case.checkpoints;
    }
    if super::ridley_ribs::is_case(case) {
        return super::ridley_ribs::valid(case);
    }
    let scope = if case.inputs == crate::replay::index::InputProvider::ScopedCeresFlyby {
        case.kind == CaseKind::Scoped
            && case.responsibilities == [Responsibility::Ceres]
            && matches!((&case.start, &case.expectations.played),
                (Start::CacheCut { frame }, Some(crate::replay::index::PlayedWindow::Flyby { first, output_fields }))
                    if frame < first && *first <= case.end.cut_inclusive && *output_fields > 0)
            && case.expectations.fields == FLYBY_STATE
            && case.expectations.outputs == FLYBY_OUTPUTS
    } else if case.inputs == InputProvider::ScopedCeresWarning {
        case.kind == CaseKind::Scoped
            && case.responsibilities == [Responsibility::Ceres]
            && case.expectations.fields == WARNING_STATE
            && case.expectations.outputs == WARNING_OUTPUTS
    } else {
        case.kind == CaseKind::Integrated
            && case.responsibilities
                == [
                    Responsibility::Ceres,
                    Responsibility::Movement,
                    Responsibility::Health,
                ]
            && (case.expectations.fields == STATE && case.expectations.outputs == OUTPUTS
                || case.expectations.fields == RETREAT_STATE
                    && case.expectations.outputs == RETREAT_OUTPUTS)
    };
    scope
        && matches!(case.start, Start::CacheCut { .. })
        && case.content == ContentProvider::PlayerRomImport
        && case.expectations.provider == ExpectedProvider::PinnedRecordCache
        && case.expectations.output_cut.is_none()
        && case.checkpoints == (case.inputs == InputProvider::ScopedCeresWarning)
}

/// Comparison fields for the indexed claim. Other played fields still gate every turn.
fn flyby_selection(field: &str) -> Option<bool> {
    if field.starts_with("ceres.") || field.starts_with("quake.") || field == "finished-frames" {
        return Some(true);
    }
    field.strip_prefix("ridley.flyby.").map(|suffix| {
        !matches!(
            suffix,
            "phase" | "shown" | "position" | "turn" | "baby" | "wings"
        )
    })
}

fn selection(field: &str, retreat: bool) -> Option<bool> {
    if retreat && (field == "ridley.retreat.bound" || field.starts_with("ridley.backdrop.")) {
        return Some(true);
    }
    if matches!(field, "ridley.tail.hit" | "ridley.tail.swish-request")
        || field.starts_with("ridley.tail.place.")
    {
        return Some(true);
    }
    if field.starts_with("ridley.tail.")
        || field.starts_with("health.")
        || field == "movement.pose"
        || field == "movement.knockback"
        || (retreat && field.starts_with("ridley.") && !field.starts_with("ridley.ribs."))
    {
        return Some(false);
    }
    None
}

pub(in crate::replay) fn run(
    case: &Case,
    sources: &Sources,
    result: &mut CaseResult,
) -> Result<(), Outcome> {
    let Start::CacheCut { frame: start } = case.start else {
        return Err(data_failure(None, "Ceres tail start"));
    };
    let lock = read_lock(sources)?;
    let mut rows = open_rows(sources, &lock, &case.row)?;
    let seed = rows.record(start)?;
    let controller = seed_controller(&seed)?;
    let content = connected::Content {
        movement: movement::Content::load(&sources.rom)?,
        appearance: appearance::Content::load(&sources.rom)?,
    };
    let mut run = Run::admit(&seed, &content, &sources.rom, &case.row)?;
    if case.inputs == InputProvider::CeresShaftActors {
        return shaft::run(&mut run, &mut rows, seed, controller, case, result);
    }
    if case.inputs == InputProvider::ScopedCeresFightPicture {
        return fight_picture::run(&mut run, &mut rows, seed, controller, case, result);
    }
    if case.inputs == InputProvider::ScopedCeresWarning {
        return warning(&mut run, &mut rows, seed, controller, case, result);
    }
    if super::ridley_ribs::is_case(case) {
        return super::ridley_ribs::run(&mut run, &mut rows, seed, controller, case, result);
    }
    if case.expectations.fields == FLYBY_STATE {
        return flyby(&mut run, &mut rows, seed, case, result);
    }
    let retreat = case.expectations.fields == RETREAT_STATE;
    let mut held = tail_held(&run, retreat);
    let mut digest = Vec::new();
    let played = run.witness_through(
        &mut rows,
        seed,
        controller,
        start.saturating_add(1)..=case.end.cut_inclusive,
        |run, cut, mut fields| {
            if !fields.iter().any(|field| field.field == "ridley.tail.pace") {
                return Err(data_failure(
                    Some(cut),
                    "flight tail comparison opportunity",
                ));
            }
            fields.retain(|field| selection(&field.field, retreat).is_some());
            let semantics: Vec<_> = fields
                .iter()
                .filter(|field| selection(&field.field, retreat) == Some(false))
                .cloned()
                .collect();
            let observed = ExpectedCut {
                semantic_count: semantics.len(),
                output_count: fields.len().saturating_sub(semantics.len()),
                semantics,
                fields,
                replies: 0,
            };
            runner::record_cut(result, &mut digest, &observed, &held, cut)?;
            held = tail_held(run, retreat);
            Ok(())
        },
    );
    if !digest.is_empty() {
        result.observation_sha256 = crate::lock::Sha256::of(&digest).to_string();
    }
    match played?.1 {
        Stop::End => Ok(()),
        Stop::At(outcome) => Err(outcome),
    }
}

fn tail_held(run: &Run<'_>, retreat: bool) -> Vec<Observation> {
    let mut held = super::samus::actual(&run.state.samus, &run.content.appearance);
    held.extend(super::health::parts(&run.state.health.0));
    held.extend(ridley::actual(&run.state.station));
    held.push(contact::actual("ridley.tail.hit", None));
    held.push(ridley::swish(false));
    if let Some(tail) = run
        .state
        .station
        .ridley
        .and_then(|ridley| ridley.flight_tail)
    {
        held.extend(ridley::places(&tail.places()));
    }
    if retreat {
        held.extend(ridley::retreat::actual(&run.state, None));
    }
    held
}

fn flyby(
    run: &mut Run<'_>,
    rows: &mut super::rows::Rows<'_>,
    seed: oracle_record::FrameRecord,
    case: &Case,
    result: &mut CaseResult,
) -> Result<(), Outcome> {
    let Some(crate::replay::index::PlayedWindow::Flyby {
        first,
        output_fields,
    }) = case.expectations.played
    else {
        return Err(data_failure(None, "flyby comparison window"));
    };
    let mut digest = Vec::new();
    run.flyby_scope(
        rows,
        seed,
        first,
        case.end.cut_inclusive,
        |cut, mut fields, held| {
            fields.retain(|field| flyby_selection(&field.field).is_some());
            let semantics: Vec<_> = fields
                .iter()
                .filter(|field| flyby_selection(&field.field) == Some(false))
                .cloned()
                .collect();
            let observed = ExpectedCut {
                semantic_count: semantics.len(),
                output_count: fields.len().saturating_sub(semantics.len()),
                semantics,
                fields,
                replies: 0,
            };
            runner::record_cut(result, &mut digest, &observed, held, cut)
        },
    )?;
    result.observation_sha256 = crate::lock::Sha256::of(&digest).to_string();
    if result.counts.outputs != output_fields {
        return Err(data_failure(
            Some(case.end.cut_inclusive),
            "flyby output witnesses",
        ));
    }
    Ok(())
}

fn warning_semantic(field: &str) -> bool {
    matches!(
        field,
        "ridley.warning.phase" | "ridley.warning.glow" | "ridley.warning.stage"
    )
}

fn warning_field(field: &str) -> bool {
    field.starts_with("ridley.warning.")
        || field.starts_with("hud.countdown.")
        || field.starts_with("ceres.")
        || field.starts_with("quake.")
        || field == "finished-frames"
}

fn warning(
    run: &mut Run<'_>,
    rows: &mut super::rows::Rows<'_>,
    mut previous: oracle_record::FrameRecord,
    mut controller: super::Controller,
    case: &Case,
    result: &mut CaseResult,
) -> Result<(), Outcome> {
    use crate::replay::ridley_warning_native as native;
    let start = previous.index;
    if !matches!(native::state(&previous)?.phase, native::Phase::Ready) {
        return Err(data_failure(Some(start), "warning-ready origin"));
    }
    let mut digest = Vec::new();
    for cut in start.saturating_add(1)..=case.end.cut_inclusive {
        let current = rows.record(cut)?;
        let (row, local) = rows.place(cut)?;
        let held = run.warning_picture.held(&run.state)?;
        let mut restored = run.clone();
        let mut fields = run.step(controller, &previous, &current, cut, (&row, local))?;
        let replayed = restored.step(controller, &previous, &current, cut, (&row, local));
        if replayed.as_ref() != Ok(&fields)
            || restored.state != run.state
            || restored.warning_picture != run.warning_picture
            || restored.return_timing != run.return_timing
        {
            return Err(Outcome::CheckpointFailure { cut });
        }
        result.counts.restorations = result.counts.restorations.saturating_add(1);
        if !fields
            .iter()
            .any(|field| field.field == "ridley.warning.phase")
        {
            return Err(data_failure(Some(cut), "warning comparison opportunity"));
        }
        fields.retain(|field| warning_field(&field.field));
        let semantics: Vec<_> = fields
            .iter()
            .filter(|field| warning_semantic(&field.field))
            .cloned()
            .collect();
        let observed = ExpectedCut {
            semantic_count: semantics.len(),
            output_count: fields.len().saturating_sub(semantics.len()),
            semantics,
            fields,
            replies: 0,
        };
        runner::record_cut(result, &mut digest, &observed, &held, cut)?;
        controller = controller.read(current.pad.0, super::polled(&current));
        previous = current;
    }
    result.observation_sha256 = crate::lock::Sha256::of(&digest).to_string();
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::replay::index::{DoorAppend, Index, PlayedWindow, Start};

    #[test]
    fn played_comparisons_require_distinct_witnesses_inside_the_declared_run() {
        let index =
            Index::parse(include_str!("../../../../../scenarios/replay/index.toml")).unwrap();
        let case = |id| {
            index
                .cases
                .iter()
                .find(|case| case.id.0 == id)
                .unwrap()
                .clone()
        };
        let mut picture = case("ridley/ordinary-picture");
        picture.id.0 = "invented/fight".into();
        picture.start = Start::CacheCut { frame: 9 };
        picture.end.cut_inclusive = 14;
        picture.expectations.played = Some(PlayedWindow::FightPicture {
            appends: [
                DoorAppend { cut: 10, cursor: 3 },
                DoorAppend { cut: 14, cursor: 4 },
            ],
        });
        assert!(picture.validate().is_ok());
        picture.end.cut_inclusive = 13;
        assert!(picture.validate().is_err());
        picture.end.cut_inclusive = 14;
        if let Some(PlayedWindow::FightPicture {
            appends: [first, second],
        }) = &mut picture.expectations.played
        {
            first.cut = second.cut;
        }
        assert!(picture.validate().is_err());

        let mut flyby = case("ridley/flyby");
        let mut shaft = case("ridley/shaft-actors");
        std::mem::swap(
            &mut flyby.expectations.played,
            &mut shaft.expectations.played,
        );
        assert!(flyby.validate().is_err());
        assert!(shaft.validate().is_err());
        shaft.expectations.played = Some(PlayedWindow::ShaftActors {
            first: 100,
            picture_fields: 1,
        });
        assert!(shaft.validate().is_err());

        let mut ribs = case("ridley/ordinary-ribs");
        ribs.expectations.played = Some(PlayedWindow::RibsReveal {
            cut: ribs.end.cut_inclusive,
        });
        assert!(ribs.validate().is_err());
        ribs.expectations.played = None;
        assert!(ribs.validate().is_err());
    }
}
