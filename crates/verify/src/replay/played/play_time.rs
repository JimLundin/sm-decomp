//! Independently read the complete elapsed play clock.

use crate::replay::{data_failure, result::Outcome};
use types::{PlayFrames, PlayHours, PlayMinutes, PlaySeconds, PlayTime};

pub(super) fn read(wram: &[u8]) -> Result<PlayTime, Outcome> {
    let invalid = || data_failure(None, "elapsed play time");
    let component = |offset: usize| {
        let bytes: [u8; 2] = wram
            .get(offset..offset.saturating_add(2))?
            .try_into()
            .ok()?;
        u8::try_from(u16::from_le_bytes(bytes)).ok()
    };
    Ok(PlayTime {
        hours: component(0x09e0)
            .and_then(PlayHours::new)
            .ok_or_else(invalid)?,
        minutes: component(0x09de)
            .and_then(PlayMinutes::new)
            .ok_or_else(invalid)?,
        seconds: component(0x09dc)
            .and_then(PlaySeconds::new)
            .ok_or_else(invalid)?,
        frames: component(0x09da)
            .and_then(PlayFrames::new)
            .ok_or_else(invalid)?,
    })
}

pub(super) fn observation(time: PlayTime) -> crate::replay::compare::Observation {
    crate::replay::compare::Observation {
        field: "game-time.elapsed".into(),
        bytes: [
            time.hours.get(),
            time.minutes.get(),
            time.seconds.get(),
            time.frames.get(),
        ]
        .to_vec(),
    }
}
