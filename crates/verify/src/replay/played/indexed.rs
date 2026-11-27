//! Indexed Ceres windows through the ordinary played room runner.

use super::{Run, Stop, contact, fight_picture, open_rows, read_lock, ridley, seed_controller};
use crate::replay::{
    Sources, appearance,
    compare::Observation,
    connected, data_failure,
    index::{Case, InputProvider, Start},
    movement,
    readers::{ExpectedCut, FieldRole},
    result::Outcome,
    runner,
};

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

pub(in crate::replay) const RETREAT_BODY: &str = "ridley.retreat.body";
/// The names a tail case declares, the retreat's included.
pub(in crate::replay) const TAIL_NAMES: [&str; 10] = [
    "ridley.tail.state",
    "movement.tail-reaction",
    "health.tail-reaction",
    "ridley.tail.places",
    "ridley.tail.hit",
    "ridley.tail.swish-request",
    RETREAT_BODY,
    "ridley.retreat.bound",
    "ridley.backdrop.walls",
    "ridley.backdrop.colours",
];

fn tail(
    mut room: AdmittedRoom<'_, '_>,
    case: &Case,
    result: &mut runner::Receipt,
) -> Result<(), Outcome> {
    let start = room.origin;
    let retreat = case.declares(&TAIL_NAMES, RETREAT_BODY)?;
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
    use crate::replay::index::{DoorAppend, Index, PlayedWindow};

    #[test]
    fn a_played_window_belongs_to_its_provider() {
        let index =
            Index::parse(include_str!("../../../../../scenarios/replay/index.toml")).unwrap();
        let mut flyby = index
            .cases
            .iter()
            .find(|case| case.id.0 == "ridley/alternate-flyby")
            .unwrap()
            .clone();
        assert!(flyby.validate().is_ok());
        flyby.expectations.played = Some(PlayedWindow::FightPicture {
            appends: [
                DoorAppend { cut: 10, cursor: 3 },
                DoorAppend { cut: 14, cursor: 4 },
            ],
        });
        assert!(flyby.validate().is_err());
    }
}
