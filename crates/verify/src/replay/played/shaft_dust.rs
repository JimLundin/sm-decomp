//! Scoped observations of the carried wall-jump puff and its placement in both OAM buffers.

use core::ops::Range;
use data::spritemap::SpriteAttributes;
use enum_map::Enum;
use game::state::video::{OAM_HIGH_BYTES, OAM_SPRITES};
use oracle_record::FrameRecord;

use super::{PictureScope, Run};
use crate::replay::{
    blackout::observe::oam_bytes, compare::Observation, data_failure, result::Outcome,
};

use super::shaft_native::word;

const STATE: &str = "shaft-dust.state";
const PREPARED: &str = "shaft-dust.prepared";
const PUBLISHED: &str = "shaft-dust.published";
const FOLLOWING_VENT: &str = "shaft-dust.following-vent";

const SLOT_3_KIND_AND_LOOK: usize = 0x0af2;
const SLOT_3_X: usize = 0x0ae2;
const SLOT_3_Y: usize = 0x0aea;
const SLOT_3_CLOCK: usize = 0x0ada;
const WALL_JUMP_DUST: u16 = 6;
const HIDDEN_Y: u8 = 240;
const LOW_BYTES: usize = OAM_SPRITES * 4;
const PREPARED_OAM: Range<usize> = 0x0370..0x0590;

fn expected(
    record: &FrameRecord,
    looks: &[SpriteAttributes; 4],
) -> Result<[Observation; 3], Outcome> {
    let kind_and_look = word(record, SLOT_3_KIND_AND_LOOK)?;
    let state = match kind_and_look >> 8 {
        0 if kind_and_look == 0 => None,
        WALL_JUMP_DUST => {
            let [look, _] = kind_and_look.to_le_bytes();
            if looks.get(usize::from(look)).is_none() {
                return Err(data_failure(
                    Some(record.index),
                    "native wall-jump dust look",
                ));
            }
            Some((
                word(record, SLOT_3_X)?,
                word(record, SLOT_3_Y)?,
                look,
                word(record, SLOT_3_CLOCK)?,
            ))
        }
        _ => {
            return Err(data_failure(
                Some(record.index),
                "native wall-jump dust kind",
            ));
        }
    };
    let prepared = record
        .wram
        .get(PREPARED_OAM)
        .and_then(|bytes| picture(PREPARED, bytes, looks))
        .ok_or_else(|| data_failure(Some(record.index), "native prepared dust OAM"))?;
    let published = picture(PUBLISHED, &record.oam, looks)
        .ok_or_else(|| data_failure(Some(record.index), "native published dust OAM"))?;
    Ok([state_observation(state), prepared, published])
}

fn actual(run: &Run<'_>, looks: &[SpriteAttributes; 4]) -> Result<[Observation; 3], Outcome> {
    let state = run.state.samus.appearance.0.dust.map(|dust| {
        (
            dust.centre.x.0,
            dust.centre.y.0,
            u8::try_from(dust.look.into_usize()).unwrap_or_default(),
            dust.visits_left.get(),
        )
    });
    let video = &run.state.room_video.video;
    let prepared = picture(PREPARED, &oam_bytes(&video.batch.oam), looks)
        .ok_or_else(|| data_failure(None, "produced prepared dust OAM"))?;
    let published = picture(PUBLISHED, &oam_bytes(&video.memory.oam), looks)
        .ok_or_else(|| data_failure(None, "produced published dust OAM"))?;
    Ok([state_observation(state), prepared, published])
}

fn state_observation(state: Option<(u16, u16, u8, u16)>) -> Observation {
    let mut bytes = Vec::new();
    if let Some((x, y, look, clock)) = state {
        bytes.extend(x.to_le_bytes());
        bytes.extend(y.to_le_bytes());
        bytes.push(look);
        bytes.extend(clock.to_le_bytes());
    }
    Observation {
        field: STATE.into(),
        bytes,
    }
}

fn picture(field: &str, bytes: &[u8], looks: &[SpriteAttributes; 4]) -> Option<Observation> {
    let (low, high) = bytes.split_at_checked(LOW_BYTES)?;
    if high.len() != OAM_HIGH_BYTES {
        return None;
    }
    let words = looks.map(SpriteAttributes::word);
    let mut pieces = Vec::new();
    for (slot, &entry) in low.as_chunks::<4>().0.iter().enumerate() {
        let [_, y, tile, attributes] = entry;
        // Finalising moves unused entries below the screen while retaining their look.
        if y != HIDDEN_Y && words.contains(&u16::from_le_bytes([tile, attributes])) {
            pieces.push(u8::try_from(slot).ok()?);
            pieces.extend(entry);
            pieces.push((high.get(slot / 4).copied()? >> (slot % 4).wrapping_mul(2)) & 3);
        }
    }
    Some(Observation {
        field: field.into(),
        bytes: pieces,
    })
}

pub(super) fn observations(
    run: &Run<'_>,
    frame: &super::Frame<'_>,
    (native, written): (&mut Vec<Observation>, &mut Vec<Observation>),
) -> Result<(), Outcome> {
    let trace = frame.witnesses.shaft()?;
    if trace.elevator.contains(frame.place) {
        let looks = looks(run)?;
        native.extend(expected(frame.current, looks)?);
        written.extend(actual(run, looks)?);
        if let Some(slot) = trace
            .following_vent(frame.place)
            .filter(|_| run.picture_scope != PictureScope::ShaftActors)
        {
            // The dust and aggregate witnesses check the shifted vent entry. The shaft
            // actor witness compares its own mounted spans during locked work.
            let video = &run.state.room_video.video;
            let prepared = frame.current.wram.get(PREPARED_OAM).unwrap_or_default();
            native.push(following_vent(slot, prepared, &frame.current.oam)?);
            written.push(following_vent(
                slot,
                &oam_bytes(&video.batch.oam),
                &oam_bytes(&video.memory.oam),
            )?);
        }
    }
    Ok(())
}

fn following_vent(slot: usize, prepared: &[u8], published: &[u8]) -> Result<Observation, Outcome> {
    let offset = slot.saturating_mul(4);
    let mut bytes = Vec::with_capacity(10);
    for picture in [prepared, published] {
        let entry = picture
            .get(offset..offset.saturating_add(4))
            .ok_or_else(|| data_failure(None, "shaft dust following vent OAM entry"))?;
        let high = picture
            .get(LOW_BYTES.saturating_add(slot / 4))
            .ok_or_else(|| data_failure(None, "shaft dust following vent OAM high bit"))?;
        bytes.extend(entry);
        bytes.push((high >> ((slot % 4).wrapping_mul(2))) & 3);
    }
    Ok(Observation {
        field: FOLLOWING_VENT.into(),
        bytes,
    })
}

fn looks<'r>(run: &'r Run<'_>) -> Result<&'r [SpriteAttributes; 4], Outcome> {
    run.played
        .wall_jump_dust_art
        .as_ref()
        .map(|art| &art.looks)
        .ok_or_else(|| data_failure(None, "wall-jump dust art"))
}
