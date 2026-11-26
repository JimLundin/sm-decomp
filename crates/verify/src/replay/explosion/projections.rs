//! Native cinematic object and event projection. All observations are read after the cut.

use super::super::{compare::Observation, data_failure, readers::ExpectedCut, result::Outcome};
use super::drawing::{bytes, quantity, visible, words};
use super::{DESTINATION, FINISHED, FLOW, PENDING, STAGE, byte, word};
use canonical::model::Model;
use oracle_record::FrameRecord;

/// The native drawing slots retain their launch identity until the track ends.
#[derive(Default)]
pub(in crate::replay) struct Observed {
    slots: [Option<(usize, u16)>; 13],
    launched: [u16; 4],
}

fn phase(record: &FrameRecord) -> Result<u8, Outcome> {
    Ok(match word(record, STAGE)? {
        PENDING => 0,
        0xc2e4 => 1,
        0xc2f1 => 2,
        0xc345 => 3,
        0xc5ca => 4,
        0xc610 => 5,
        0xc627 => 6,
        0xc699 => 7,
        _ => return Err(data_failure(Some(record.index), "native station stage")),
    })
}

fn sample(group: usize, picture: u16) -> Option<u16> {
    if picture == 0 {
        return Some(0);
    }
    let pictures: &[u16] = match group {
        0 | 1 => &[0x97f7, 0x97fe, 0x9805, 0x981b, 0x9831, 0x9847],
        2 => &[0x98d2, 0x98d9, 0x98e0, 0x98e7],
        3 => &[0x98ee, 0x9904, 0x991a, 0x9930, 0x996e, 0x9998],
        _ => return None,
    };
    pictures
        .iter()
        .position(|&id| id == picture)
        .and_then(|index| u16::try_from(index.saturating_add(1)).ok())
}

fn track(pointer: u16, timer: u16, loops: u16) -> Option<(usize, i32)> {
    // The native cursor already names the next sample. Its timer counts down the
    // remaining part of the sample currently displayed, including blank samples.
    let (group, first, frames, pictures, pass_frames, passes, blank) = match pointer {
        0xccdb => return Some((0, i32::from(timer).saturating_neg())),
        0xccf5 => return Some((1, i32::from(timer).saturating_neg())),
        0xcd1b => return Some((2, i32::from(timer).saturating_neg())),
        0xce1b => return Some((3, i32::from(timer).saturating_neg())),
        0xccdf..=0xccf3 => (0, 0xccdf, 3u16, 6u16, 0u16, None, 0u16),
        0xccfd..=0xcd15 => (1, 0xccfd, 3, 6, 34, Some(6u16), 16),
        0xcd23..=0xcd33 => (2, 0xcd23, 5, 4, 28, Some(7), 8),
        0xce1f..=0xce33 => (3, 0xce1f, 5, 6, 0, None, 0),
        _ => return None,
    };
    let index = pointer.checked_sub(first)?.checked_div(4)?;
    let elapsed = passes
        .map_or(Some(0), |count| count.checked_sub(loops))?
        .checked_mul(pass_frames)?
        .checked_add(frames.checked_mul(index.min(pictures))?)?
        .checked_add(if index == pictures { blank } else { frames })?;
    Some((group, i32::from(elapsed).checked_sub(i32::from(timer))?))
}

fn native_point(record: &FrameRecord, offset: usize) -> Result<[u16; 4], Outcome> {
    Ok([
        word(record, 0x1a7dusize.saturating_add(offset))?,
        word(record, 0x1addusize.saturating_add(offset))?,
        word(record, 0x1a9dusize.saturating_add(offset))?,
        word(record, 0x1afdusize.saturating_add(offset))?,
    ])
}

impl Observed {
    fn bursts(&mut self, record: &FrameRecord) -> Result<Observation, Outcome> {
        let mut out = Vec::new();
        for (rank, slot) in self.slots.iter_mut().enumerate() {
            let offset = 14usize.saturating_sub(rank).saturating_mul(2);
            let pointer = word(record, 0x1b1dusize.saturating_add(offset))?;
            if matches!(pointer, 0 | 0xce39 | 0xce3f | 0xce47) {
                *slot = None;
                continue;
            }
            let (group, age) = track(
                pointer,
                word(record, 0x1b5dusize.saturating_add(offset))?,
                word(record, 0x1b7dusize.saturating_add(offset))?,
            )
            .ok_or_else(|| data_failure(Some(record.index), "native station burst track"))?;
            let occurrence = if let Some((previous, id)) = *slot
                && group == previous
            {
                id
            } else {
                let count = self
                    .launched
                    .get_mut(group)
                    .ok_or_else(|| data_failure(Some(record.index), "native burst group"))?;
                *count = count.saturating_add(1);
                *slot = Some((group, *count));
                *count
            };
            out.extend([
                u16::try_from(group.saturating_add(1)).unwrap_or_default(),
                occurrence,
                i16::try_from(age)
                    .map_err(|_| data_failure(Some(record.index), "native burst age"))?
                    .cast_unsigned(),
                sample(group, word(record, 0x1a5dusize.saturating_add(offset))?)
                    .ok_or_else(|| data_failure(Some(record.index), "native burst picture"))?,
            ]);
            out.extend(native_point(record, offset)?);
        }
        Ok(words("bursts", out))
    }

    pub(in crate::replay) fn read(
        &mut self,
        prior: &FrameRecord,
        now: &FrameRecord,
    ) -> Result<ExpectedCut, Outcome> {
        let stage = phase(now)?;
        let began = phase(prior)?;
        if word(now, DESTINATION)? != 0x22 || word(now, FLOW)? != 0x22 {
            return Err(data_failure(
                Some(now.index),
                "station retained landing destination",
            ));
        }
        let mut fields = vec![
            quantity("stage", stage),
            quantity("light", byte(now, 0x51)?),
            words(
                "context",
                [
                    word(now, FINISHED)?,
                    u16::from(word(now, DESTINATION)? == 0x22),
                    word(now, 0x09c4)?,
                    u16::from(word(now, 0x09e2)? != 0),
                    u16::from(stage == 7),
                ],
            ),
            bytes(
                "health",
                canonical::to_bytes(&Model::<game::production::health_tail::HealthPin>(
                    super::super::health_tail::decode(&now.wram, Some(now.index))?,
                )),
            ),
        ];
        if stage != 0 {
            fields.push(words(
                "pose",
                [
                    word(now, 0x198d)?,
                    word(now, 0x198f)?,
                    word(now, 0x1993)?,
                    word(now, 0x1991)?,
                    word(now, 0x1997)?,
                    word(now, 0x1995)?,
                ],
            ));
            fields.push(words(
                "transform",
                [
                    word(now, 0x78)?,
                    word(now, 0x7a)?,
                    word(now, 0x7c)?,
                    word(now, 0x7e)?,
                    word(now, 0xb1)?,
                    word(now, 0xb3)?,
                    word(now, 0x80)?,
                    word(now, 0x82)?,
                ],
            ));
            fields.push(words(
                "timers",
                [
                    word(now, 0x1a51)?,
                    if matches!(stage, 1 | 2 | 6) {
                        word(now, 0x0725)?
                    } else {
                        0
                    },
                    if stage == 5 { word(now, 0x1a49)? } else { 0 },
                ],
            ));
            let mut sky = Vec::from(native_point(now, 30)?);
            sky.extend(native_point(now, 2)?);
            sky.push(match word(now, 0x1a5d)? {
                0x8fe7 => 1,
                0x93d1 => 0,
                _ => return Err(data_failure(Some(now.index), "native station backdrop")),
            });
            fields.push(words("sky", sky));
            fields.push(self.bursts(now)?);
        }
        let semantics = fields.clone();
        fields.extend(outputs(prior, now, began, stage)?);
        Ok(ExpectedCut {
            semantic_count: semantics.len(),
            output_count: fields.len().saturating_sub(semantics.len()),
            fields,
            semantics,
            replies: 0,
        })
    }
}

fn outputs(
    prior: &FrameRecord,
    now: &FrameRecord,
    began: u8,
    stage: u8,
) -> Result<Vec<Observation>, Outcome> {
    let mut fields = Vec::new();
    let setup = began == 0 && stage == 1;
    let flash = word(prior, 0x18b4)? == 0 && word(now, 0x18b4)? != 0;
    let mut events = Vec::new();
    if setup {
        events.extend([1, 2]);
    }
    let mut from = word(prior, 0x0639)?;
    let end = word(now, 0x0639)?;
    if from > 14 || end > 14 || !from.is_multiple_of(2) || !end.is_multiple_of(2) {
        return Err(data_failure(Some(now.index), "station music queue cursors"));
    }
    while from != end {
        let delta = usize::from(from);
        let song = match word(now, 0x0619usize.saturating_add(delta))? {
            0 => 0,
            0xff2d => 1,
            7 => 2,
            _ => {
                return Err(data_failure(
                    Some(now.index),
                    "native station music request",
                ));
            }
        };
        events.extend([3, song, word(now, 0x0629usize.saturating_add(delta))?]);
        from = from.wrapping_add(2) % 16;
    }
    if flash {
        events.extend([4, word(now, 0x0ce2)?, word(now, 0x0ce4)?, 5, 6]);
        fields.push(map(now, "ship-front", 0..0x300)?);
        fields.push(map(now, "empty-lower", 0x300..0x600)?);
    }
    if stage == 7 && began != 7 {
        events.push(7);
    }
    if setup {
        fields.push(map(now, "map", 0..0x4000)?);
        fields.push(bytes(
            "plane-tiles",
            vram(now, 0..0x4000)?
                .iter()
                .map(|w| w.to_le_bytes()[1])
                .collect(),
        ));
        fields.push(words(
            "sprite-tiles",
            vram(now, 0x6000..0x8000)?.iter().copied(),
        ));
    }
    fields.push(words("requests", events));
    if stage != 0 {
        let (low, high) = now
            .oam
            .split_at_checked(512)
            .ok_or_else(|| data_failure(Some(now.index), "station OAM"))?;
        fields.push(bytes(
            "oam",
            visible(low.as_chunks::<4>().0.iter().copied(), high),
        ));
        fields.push(words("palette", now.cgram.iter().copied()));
    }
    Ok(fields)
}

fn vram(record: &FrameRecord, range: std::ops::Range<usize>) -> Result<&[u16], Outcome> {
    record
        .vram
        .get(range)
        .ok_or_else(|| data_failure(Some(record.index), "station drawing page"))
}
fn map(
    record: &FrameRecord,
    name: &str,
    range: std::ops::Range<usize>,
) -> Result<Observation, Outcome> {
    Ok(bytes(
        name,
        vram(record, range)?
            .iter()
            .map(|w| w.to_le_bytes()[0])
            .collect(),
    ))
}
