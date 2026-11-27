//! Indexed Ceres windows through the ordinary played room runner.

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
    readers::{ExpectedCut, FieldRole},
    result::Outcome,
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
fn flyby_selection(field: &str) -> Option<FieldRole> {
    if field.starts_with("ceres.") || field.starts_with("quake.") || field == "finished-frames" {
        return Some(FieldRole::Output);
    }
    field.strip_prefix("ridley.flyby.").map(|suffix| {
        if matches!(
            suffix,
            "phase" | "shown" | "position" | "turn" | "baby" | "wings"
        ) {
            FieldRole::Semantic
        } else {
            FieldRole::Output
        }
    })
}

fn selection(field: &str, retreat: bool) -> Option<FieldRole> {
    if retreat && (field == "ridley.retreat.bound" || field.starts_with("ridley.backdrop.")) {
        return Some(FieldRole::Output);
    }
    if matches!(field, "ridley.tail.hit" | "ridley.tail.swish-request")
        || field.starts_with("ridley.tail.place.")
    {
        return Some(FieldRole::Output);
    }
    if field.starts_with("ridley.tail.")
        || field.starts_with("health.")
        || field == "movement.pose"
        || field == "movement.knockback"
        || (retreat && field.starts_with("ridley.") && !field.starts_with("ridley.ribs."))
    {
        return Some(FieldRole::Semantic);
    }
    None
}

pub(super) struct AdmittedRoom<'content, 'sources> {
    pub(super) run: Run<'content>,
    pub(super) rows: super::rows::Rows<'sources>,
    pub(super) seed: oracle_record::FrameRecord,
    controller: super::Controller,
    origin: u64,
}

impl<'content> AdmittedRoom<'content, '_> {
    pub(super) fn replay(
        mut self,
        case: &Case,
        mut step: impl FnMut(
            &mut Run<'content>,
            super::rows::Turn<'_>,
            &mut super::rows::Rows<'_>,
        ) -> Result<(), Outcome>,
    ) -> Result<(), Outcome> {
        self.rows.replay(
            self.seed,
            self.controller,
            self.origin.saturating_add(1)..=case.end.cut_inclusive,
            (),
            |(), turn, rows| step(&mut self.run, turn, rows),
        )
    }
}

type RoomAdapter = fn(AdmittedRoom<'_, '_>, &Case, &mut runner::Receipt) -> Result<(), Outcome>;

pub(in crate::replay) fn run_selected(
    case: &Case,
    sources: &Sources,
    result: &mut runner::Receipt,
) -> Option<Result<(), Outcome>> {
    const ADAPTERS: &[(InputProvider, RoomAdapter)] = &[
        (InputProvider::ScopedCeresFightPicture, fight_picture::run),
        (InputProvider::ScopedCeresWarning, warning),
        (InputProvider::ScopedCeresFlyby, flyby),
        (InputProvider::PlayedCeresInputs, tail),
        (InputProvider::PlayedShaftSound, super::shaft_sound::run),
        (
            InputProvider::ShaftPictures,
            super::shaft_native::indexed::run,
        ),
    ];
    let &(_, adapter) = ADAPTERS
        .iter()
        .find(|(provider, _)| *provider == case.inputs)?;
    Some(run(adapter, case, sources, result))
}

fn run(
    adapter: RoomAdapter,
    case: &Case,
    sources: &Sources,
    result: &mut runner::Receipt,
) -> Result<(), Outcome> {
    let (Start::CacheCut { frame: start } | Start::PlayedContinuation { seed: start, .. }) =
        case.start
    else {
        return Err(data_failure(None, "played room origin"));
    };
    let lock = read_lock(sources)?;
    let mut rows = open_rows(sources, &lock, &case.row)?;
    let seed = rows.record(start)?;
    let controller = seed_controller(&seed)?;
    let content = connected::Content {
        movement: movement::Content::load(&sources.rom)?,
        appearance: appearance::Content::load(&sources.rom)?,
    };
    let run = Run::for_case(&seed, &content, sources, case)?;
    adapter(
        AdmittedRoom {
            run,
            rows,
            seed,
            controller,
            origin: start,
        },
        case,
        result,
    )
}

fn tail(
    mut room: AdmittedRoom<'_, '_>,
    case: &Case,
    result: &mut runner::Receipt,
) -> Result<(), Outcome> {
    let start = room.origin;
    let retreat = case.expectations.fields == RETREAT_STATE;
    let mut held = tail_held(&room.run, retreat);
    let played = room.run.witness_through(
        &mut room.rows,
        room.seed,
        room.controller,
        start.saturating_add(1)..=case.end.cut_inclusive,
        |run, cut, fields| {
            if !fields.iter().any(|field| field.field == "ridley.tail.pace") {
                return Err(data_failure(
                    Some(cut),
                    "flight tail comparison opportunity",
                ));
            }
            let observed = ExpectedCut::select(fields, |field| selection(field, retreat));
            result.record(&observed, &held, cut)?;
            held = tail_held(run, retreat);
            Ok(())
        },
    );

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
    mut room: AdmittedRoom<'_, '_>,
    case: &Case,
    result: &mut runner::Receipt,
) -> Result<(), Outcome> {
    let Some(crate::replay::index::PlayedWindow::Flyby {
        first,
        output_fields,
    }) = case.expectations.played
    else {
        return Err(data_failure(None, "flyby comparison window"));
    };
    room.run.flyby_scope(
        &mut room.rows,
        room.seed,
        first,
        case.end.cut_inclusive,
        |cut, fields, held| {
            let observed = ExpectedCut::select(fields, flyby_selection);
            result.record(&observed, held, cut)
        },
    )?;

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
    room: AdmittedRoom<'_, '_>,
    case: &Case,
    result: &mut runner::Receipt,
) -> Result<(), Outcome> {
    use crate::replay::ridley_warning_native as native;
    let start = room.seed.index;
    if !matches!(native::state(&room.seed)?.phase, native::Phase::Ready) {
        return Err(data_failure(Some(start), "warning-ready origin"));
    }
    room.replay(case, |run, turn, _| {
        let cut = turn.cut;
        let held = run.warning_picture.held(&run.state)?;
        let mut restored = run.clone();
        let fields = turn.fields(run)?;
        let replayed = turn.fields(&mut restored);
        if replayed.as_ref() != Ok(&fields) || !super::rows::Turn::same_room(&restored, run) {
            return Err(Outcome::CheckpointFailure { cut });
        }
        result.restored(1)?;
        if !fields
            .iter()
            .any(|field| field.field == "ridley.warning.phase")
        {
            return Err(data_failure(Some(cut), "warning comparison opportunity"));
        }
        let observed = ExpectedCut::select(fields, |field| {
            if warning_semantic(field) {
                Some(FieldRole::Semantic)
            } else {
                warning_field(field).then_some(FieldRole::Output)
            }
        });
        result.record(&observed, &held, cut)?;
        Ok(())
    })?;

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

        let mut flyby = case("ridley/alternate-flyby");
        flyby.expectations.played = Some(PlayedWindow::FightPicture {
            appends: [
                DoorAppend { cut: 10, cursor: 3 },
                DoorAppend { cut: 14, cursor: 4 },
            ],
        });
        assert!(flyby.validate().is_err());
        flyby.expectations.played = None;
        assert!(flyby.validate().is_err());
    }
}
