//! Indexed Ceres windows through the ordinary played room runner.

pub(in crate::replay) mod shaft;

use super::{Run, Stop, actual, contact, open_rows, read_lock, ridley, seed_controller};
use crate::replay::{
    Sources, appearance, connected, data_failure,
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

pub(in crate::replay) fn valid(case: &Case) -> bool {
    if case.inputs == InputProvider::CeresShaftActors {
        return shaft::valid(case);
    }
    let scope = if case.inputs == crate::replay::index::InputProvider::ScopedCeresFlyby {
        case.kind == CaseKind::Scoped
            && case.responsibilities == [Responsibility::Ceres]
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
        || (retreat && field.starts_with("ridley."))
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
    if case.inputs == InputProvider::ScopedCeresWarning {
        return warning(&mut run, &mut rows, seed, controller, case, result);
    }
    if case.expectations.fields == FLYBY_STATE {
        return flyby(
            &mut run,
            &mut rows,
            seed,
            start,
            case.end.cut_inclusive,
            result,
        );
    }
    let mut frozen = actual(
        &run.state,
        &content.appearance,
        &run.hud,
        true,
        false,
        false,
    )?;
    frozen.push(contact::actual("ridley.tail.hit", None));
    frozen.push(ridley::swish(false));
    if let Some(tail) = run
        .state
        .station
        .ridley
        .and_then(|ridley| ridley.flight_tail)
    {
        frozen.extend(ridley::places(&tail.places()));
    }
    let retreat = case.expectations.fields == RETREAT_STATE;
    if retreat {
        frozen.extend(ridley::retreat::actual(&run.state, None));
    }
    let mut digest = Vec::new();
    let played = run.witness_through(
        &mut rows,
        seed,
        controller,
        start.saturating_add(1)..=case.end.cut_inclusive,
        |cut, mut fields| {
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
            runner::record_cut(result, &mut digest, &observed, &frozen, cut)
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

fn flyby(
    run: &mut Run<'_>,
    rows: &mut super::rows::Rows<'_>,
    seed: oracle_record::FrameRecord,
    start: u64,
    end: u64,
    result: &mut CaseResult,
) -> Result<(), Outcome> {
    let mut digest = Vec::new();
    let frozen = ridley::flyby::frozen();
    run.flyby_scope(rows, seed, start, end, |cut, mut fields| {
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
        runner::record_cut(result, &mut digest, &observed, &frozen, cut)
    })?;
    result.observation_sha256 = crate::lock::Sha256::of(&digest).to_string();
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
    use crate::replay::{compare::Observation, ridley_warning_native as native};
    let start = previous.index;
    if !matches!(native::state(&previous)?.phase, native::Phase::Ready) {
        return Err(data_failure(Some(start), "warning-ready origin"));
    }
    let frozen = [
        Observation {
            field: "ridley.warning.phase".into(),
            bytes: vec![0; 8],
        },
        Observation::quantity("ridley.warning.glow", 0),
        Observation::quantity("ridley.warning.stage", 1),
    ];
    let mut digest = Vec::new();
    for cut in start.saturating_add(1)..=case.end.cut_inclusive {
        let current = rows.record(cut)?;
        let (row, local) = rows.place(cut)?;
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
        runner::record_cut(result, &mut digest, &observed, &frozen, cut)?;
        controller = controller.read(current.pad.0, super::polled(&current));
        previous = current;
    }
    result.observation_sha256 = crate::lock::Sha256::of(&digest).to_string();
    Ok(())
}
