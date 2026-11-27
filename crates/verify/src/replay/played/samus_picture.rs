//! Compare the body contribution in the ordinary Ridley room.

use core::ops::Range;

use game::state::video::{OAM_SPRITES, Oam, Video};

use super::{Frame, Outcome};
use crate::replay::{compare::Observation, data_failure};
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
        crate::source::OAM.read(&frame.current.wram).ok(),
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
    let prepared = crate::source::OAM
        .read(&record.wram)
        .map_err(|_| data_failure(Some(record.index), "ordinary body prepared OAM rectangle"))?;
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

// The room supplies context. Resume the existing canonical owners that carry
// Samus, her protection, her view, and the pending and published sprite data.
