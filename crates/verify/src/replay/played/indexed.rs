//! Indexed flight-tail and retreat windows through the ordinary played room runner.

use super::{Run, Stop, actual, contact, open_rows, read_lock, ridley, seed_controller};
use crate::replay::{
    Sources, appearance, connected, data_failure,
    index::{
        Case, CaseKind, ContentProvider, ExpectedProvider, ObservationId, Responsibility, Start,
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

pub(in crate::replay) fn valid(case: &Case) -> bool {
    case.kind == CaseKind::Integrated
        && matches!(case.start, Start::CacheCut { .. })
        && case.content == ContentProvider::PlayerRomImport
        && case.expectations.provider == ExpectedProvider::PinnedRecordCache
        && (case.expectations.fields == STATE && case.expectations.outputs == OUTPUTS
            || case.expectations.fields == RETREAT_STATE
                && case.expectations.outputs == RETREAT_OUTPUTS)
        && case.expectations.output_cut.is_none()
        && case.responsibilities
            == [
                Responsibility::Ceres,
                Responsibility::Movement,
                Responsibility::Health,
            ]
        && !case.checkpoints
}

/// Comparison fields for the indexed claim. Other played fields still gate every turn.
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
    let mut frozen = actual(&run.state, &content.appearance, &run.hud, true, false)?;
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
