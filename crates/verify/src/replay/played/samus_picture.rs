//! Compare the body contribution in the ordinary Ridley room.

use core::ops::Range;

use game::state::video::{OAM_SPRITES, Oam, Video};

use super::{Frame, Outcome, Run, open_rows, read_lock, seed_controller, witnesses};
use crate::replay::{
    Sources, appearance,
    compare::Observation,
    connected, data_failure,
    index::{
        Case, CaseKind, ContentProvider, ExpectedProvider, ObservationId, Responsibility, Start,
    },
    movement,
    readers::ExpectedCut,
    result::CaseResult,
    runner,
};
use oracle_record::FrameRecord;

pub(super) fn compare(
    frame: &Frame<'_>,
    video: &Video,
    span: Option<Range<usize>>,
    expected: &mut Vec<Observation>,
) -> Result<(), Outcome> {
    let (row, local) = frame.place;
    let Some(native) = frame
        .witnesses
        .body(row, local, local)
        .and_then(|trace| trace.slots(local))
    else {
        return Ok(());
    };
    let differs = |field: &str| Outcome::Difference {
        cut: frame.cut,
        field: format!("samus-oam.{field}"),
    };
    let Some(span) = span.filter(|span| {
        span.start <= span.end && span.end <= OAM_SPRITES && span.len() == native.len()
    }) else {
        return Err(differs("piece-count"));
    };
    compare_picture(
        &video.batch.oam,
        frame.current.wram.get(0x370..0x590),
        span.clone(),
        native.clone(),
    )
    .map_err(|slot| differs(&format!("prepared[{slot}]")))?;
    compare_picture(
        &video.memory.oam,
        Some(&frame.current.oam),
        span,
        native.clone(),
    )
    .map_err(|slot| differs(&format!("physical[{slot}]")))?;
    expected.extend(native_picture(frame.current, native)?);
    Ok(())
}

fn compare_picture(
    actual: &Oam,
    expected: Option<&[u8]>,
    span: Range<usize>,
    native: Range<usize>,
) -> Result<(), usize> {
    for (slot, native_slot) in span.zip(native) {
        let observed = expected
            .and_then(|bytes| native_piece(bytes, native_slot))
            .ok_or(native_slot)?;
        if piece(actual, slot) != Some(observed) {
            return Err(native_slot);
        }
    }
    Ok(())
}

fn piece(oam: &Oam, slot: usize) -> Option<([u8; 4], u8)> {
    let entry = oam.low.get(slot)?;
    let bits = oam.high.get(slot / 4)? >> (slot % 4).wrapping_mul(2);
    Some(([entry.x, entry.y, entry.tile, entry.attributes], bits & 3))
}

fn native_piece(bytes: &[u8], slot: usize) -> Option<([u8; 4], u8)> {
    let low = bytes
        .get(slot.wrapping_mul(4)..)?
        .first_chunk::<4>()
        .copied()?;
    let bits = bytes.get(512_usize.wrapping_add(slot / 4))? >> (slot % 4).wrapping_mul(2);
    Some((low, bits & 3))
}

fn native_picture(record: &FrameRecord, slots: Range<usize>) -> Result<[Observation; 4], Outcome> {
    let prepared = record
        .wram
        .get(0x370..0x590)
        .ok_or_else(|| data_failure(Some(record.index), "ordinary body prepared OAM rectangle"))?;
    if record.oam.len() != prepared.len() {
        return Err(data_failure(
            Some(record.index),
            "ordinary body physical OAM rectangle",
        ));
    }
    Ok(picture(slots, |slot| {
        (
            native_piece(prepared, slot),
            native_piece(&record.oam, slot),
        )
    }))
}

type Piece = ([u8; 4], u8);

fn picture(
    span: Range<usize>,
    mut entries: impl FnMut(usize) -> (Option<Piece>, Option<Piece>),
) -> [Observation; 4] {
    let mut prepared = Vec::new();
    let mut physical = Vec::new();
    let visible = Observation::quantity("samus-oam.visible", i64::from(!span.is_empty()));
    let count = Observation::quantity(
        "samus-oam.piece-count",
        i64::try_from(span.len()).unwrap_or_default(),
    );
    for slot in span {
        let (batch, memory) = entries(slot);
        for (bytes, entry) in [(&mut prepared, batch), (&mut physical, memory)] {
            if let Some((low, high)) = entry {
                bytes.extend(low);
                bytes.push(high);
            }
        }
    }
    [
        visible,
        count,
        Observation {
            field: "samus-oam.prepared".into(),
            bytes: prepared,
        },
        Observation {
            field: "samus-oam.physical".into(),
            bytes: physical,
        },
    ]
}

pub(in crate::replay) const OUTPUTS: &[ObservationId] = &[
    ObservationId::OrdinaryBodyVisible,
    ObservationId::OrdinaryBodyCount,
    ObservationId::OrdinaryBodyPrepared,
    ObservationId::OrdinaryBodyPhysical,
];

pub(in crate::replay) fn valid(case: &Case) -> bool {
    let Start::CacheCut { frame } = case.start else {
        return false;
    };
    witnesses::selected(case).is_ok_and(|data| {
        data.body(&case.row, frame, case.end.cut_inclusive)
            .is_some_and(|trace| trace.has_draw(frame.saturating_add(1), case.end.cut_inclusive))
    }) && case.kind == CaseKind::Scoped
        && case.responsibilities == [Responsibility::Appearance]
        && case.content == ContentProvider::PlayerRomImport
        && case.expectations.provider == ExpectedProvider::PinnedRecordCache
        && case.expectations.fields.is_empty()
        && case.expectations.outputs == OUTPUTS
        && case.expectations.output_cut.is_none()
        && case.checkpoints
}

pub(in crate::replay) fn run(
    case: &Case,
    sources: &Sources,
    result: &mut CaseResult,
) -> Result<(), Outcome> {
    let Start::CacheCut { frame: start } = case.start else {
        return Err(data_failure(None, "ordinary body origin"));
    };
    let lock = read_lock(sources)?;
    let mut rows = open_rows(sources, &lock, &case.row)?;
    let mut previous = rows.record(start)?;
    let mut controller = seed_controller(&previous)?;
    let content = connected::Content {
        movement: movement::Content::load(&sources.rom)?,
        appearance: appearance::Content::load(&sources.rom)?,
    };
    let mut run = Run::for_case(&previous, &content, sources, case)?;
    let (row, local) = rows.place(start)?;
    let slots = run
        .witnesses
        .body(&row, local, local)
        .and_then(|trace| trace.slots(local))
        .ok_or_else(|| data_failure(Some(start), "ordinary body origin count"))?;
    let mut null = native_picture(&previous, slots)?;
    let mut digest = Vec::new();
    let mut continuations = Vec::new();
    for cut in start.saturating_add(1)..=case.end.cut_inclusive {
        let current = rows.record(cut)?;
        let (row, local) = rows.place(cut)?;
        if row != case.row || local != cut {
            return Err(data_failure(Some(cut), "ordinary body trace row"));
        }
        let saved_cut = previous.index;
        continuations.push((
            saved_cut,
            restore(&run).ok_or(Outcome::CheckpointFailure { cut: saved_cut })?,
        ));
        let (mut fields, outputs) =
            run.step_with_outputs(controller, &previous, &current, cut, (&row, local))?;
        for (saved_cut, restored) in &mut continuations {
            let replayed = restored.step(controller, &previous, &current, cut, (&row, local));
            if replayed.as_ref() != Ok(&fields) || restored.state != run.state {
                return Err(Outcome::CheckpointFailure { cut: *saved_cut });
            }
        }
        fields.retain(|field| field.field.starts_with("samus-oam."));
        if fields.len() != OUTPUTS.len() {
            return Err(data_failure(
                Some(cut),
                "ordinary body comparison opportunity",
            ));
        }
        let observed = ExpectedCut {
            semantic_count: 0,
            output_count: fields.len(),
            semantics: Vec::new(),
            fields,
            replies: 0,
        };
        runner::record_cut(result, &mut digest, &observed, &null, cut)?;
        let span = outputs
            .samus_body
            .ok_or_else(|| data_failure(Some(cut), "ordinary body span"))?;
        let video = &run.state.room_video.video;
        null = picture(span, |slot| {
            (
                piece(&video.batch.oam, slot),
                piece(&video.memory.oam, slot),
            )
        });
        controller = controller.read(current.pad.0, super::polled(&current));
        previous = current;
    }
    result.counts.restorations = continuations.len();
    result.observation_sha256 = crate::lock::Sha256::of(&digest).to_string();
    Ok(())
}

// The room supplies context. Resume the existing canonical owners that carry
// Samus, her protection, her view, and the pending and published sprite data.
fn restore<'a>(run: &Run<'a>) -> Option<Run<'a>> {
    fn round_trip<T: canonical::Canonical>(state: &T) -> Option<T> {
        canonical::from_bytes(&canonical::to_bytes(state)).ok()
    }
    let mut restored = run.clone();
    restored.state.samus = round_trip(&run.state.samus)?;
    restored.state.health = round_trip(&run.state.health)?;
    restored.state.camera = round_trip(&run.state.camera)?;
    *restored.state.room_video.video = round_trip(&*run.state.room_video.video)?;
    Some(restored)
}
