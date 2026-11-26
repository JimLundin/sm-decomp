//! Native shaft actor placement, rotation and elevator observations.

use super::{Sources, open_rows, read_lock};
use crate::replay::{data_failure, result::Outcome};
use oracle_record::FrameRecord;

const HALL: &str = "chain-any-0010-DF8D";
const SHAFT: &str = "chain-any-0011-DF45";

fn word(record: &FrameRecord, at: usize) -> Result<u16, Outcome> {
    record
        .wram
        .get(at..at.saturating_add(2))
        .and_then(|bytes| bytes.try_into().ok())
        .map(u16::from_le_bytes)
        .ok_or_else(|| data_failure(Some(record.index), "shaft native word"))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Overlay {
    Opening,
    Rumble { left: u16, wait: u16, next: u16 },
    Hidden,
}

fn overlay(record: &FrameRecord) -> Result<Overlay, Outcome> {
    let visible = word(record, 0x0f86)? & 0x100 == 0;
    match (word(record, 0x0fa8)?, visible) {
        (0xf7bd, true) => Ok(Overlay::Opening),
        (0xf7dc, true) if word(record, 0x0f8e)? == 0xf921 => {
            let point = word(record, 0x0fb2)?;
            Ok(Overlay::Rumble {
                left: word(record, 0x0fae)?,
                wait: word(record, 0x0fb0)?,
                next: 4_u16.wrapping_sub(point) % 4,
            })
        }
        (0xf850, false) => Ok(Overlay::Hidden),
        _ => Err(data_failure(Some(record.index), "shaft overlay phase")),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Rotation {
    turning: bool,
    sample: u16,
    forward: bool,
    hold: u16,
    matrix: [i16; 4],
}

fn rotation(record: &FrameRecord) -> Result<Rotation, Outcome> {
    let track = word(record, 0x07e1)?;
    Ok(Rotation {
        turning: word(record, 0x093f)? & 0x8000 != 0,
        sample: track & 0x7fff,
        forward: track & 0x8000 == 0,
        hold: word(record, 0x07e3)?,
        matrix: [
            word(record, 0x78)?.cast_signed(),
            word(record, 0x7a)?.cast_signed(),
            word(record, 0x7c)?.cast_signed(),
            word(record, 0x7e)?.cast_signed(),
        ],
    })
}

fn reached(before: &FrameRecord, after: &FrameRecord) -> Result<bool, Outcome> {
    Ok(word(before, 0x0998)? == 8 && word(after, 0x0998)? == 32)
}

#[test]
#[ignore = "pinned record cache"]
fn shaft_native_cuts_preserve_placement_and_reach() -> Result<(), Outcome> {
    use super::{platform::actor_fetch, tests};
    use game::production::played::passage::ActorFetch;
    use types::{RandomNumber, RandomSequence};

    let sources: Sources = tests::sources();
    let lock = read_lock(&sources)?;
    let mut hall = open_rows(&sources, &lock, HALL)?;
    let before = hall.record(224)?;
    let installed = hall.record(225)?;
    assert_eq!(actor_fetch(&installed), ActorFetch::Place);
    assert_eq!(overlay(&installed)?, Overlay::Opening);
    let mut random = RandomSequence {
        number: RandomNumber(word(&before, 0x05e5)?),
    };
    for slot in 2_usize..13 {
        game_random::advance_random(&mut random);
        let at = 0x0fae_usize.saturating_add(slot.saturating_mul(64));
        assert_eq!(
            word(&installed, at)?,
            1_u16.saturating_add(random.number.0 % 32),
            "shaft vent slot {slot}"
        );
    }
    assert_eq!(word(&installed, 0x05e5)?, random.number.0);
    assert_eq!(overlay(&hall.record(238)?)?, Overlay::Opening);
    assert_eq!(
        overlay(&hall.record(239)?)?,
        Overlay::Rumble {
            left: 48,
            wait: 0,
            next: 0
        }
    );

    let mut previous = hall.record(278)?;
    let mut shaft = open_rows(&sources, &lock, SHAFT)?;
    let mut reach = Vec::new();
    let mut sample_reads = 0_usize;
    for cut in 0..=239 {
        let current = shaft.record(cut)?;
        assert_eq!(
            current.video_frame,
            previous.video_frame.wrapping_add(1),
            "shaft video join {cut}"
        );
        let state = rotation(&current)?;
        let [aa, bb, cc, dd, pivot_x, pivot_y] = current.ppu.m7;
        assert_eq!(
            [aa, bb, cc, dd],
            state.matrix,
            "shaft physical matrix {cut}"
        );
        assert_eq!(
            (pivot_x, pivot_y),
            (128, 1008),
            "shaft physical pivot {cut}"
        );
        if rotation(&previous)?.sample != state.sample {
            sample_reads = sample_reads.saturating_add(1);
        }
        if reached(&previous, &current)? {
            reach.push(cut);
            assert!(state.turning);
            assert_eq!(word(&current, 0x0aa0)?, 60);
            assert_eq!(word(&current, 0x0723)?, 0);
            assert_eq!(word(&current, 0x0725)?, 0);
        }
        previous = current;
    }
    assert_eq!(sample_reads, 27);
    assert_eq!(reach, [164]);
    Ok(())
}
