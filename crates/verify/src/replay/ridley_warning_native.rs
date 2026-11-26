//! Native warning fields at the authenticated room entry and its successors.

use super::{data_failure, result::Outcome};
use oracle_record::FrameRecord;

const ROUTINE: usize = 0x0fa8;
const WARNING: u16 = 0xc04e;
const ESCAPE: u16 = 0xaa50;
const PHASE: usize = 0x0fb2;
const PROGRESS: usize = 0x0fb0;
const GLOW: usize = 0x8030;
const CURSOR: usize = 0x8036;
const WAIT: usize = 0x803a;
const AUDIBLE: usize = 0x803e;
const STATION: usize = 0x093f;
#[cfg(test)]
const SUBTITLES: usize = 0x09e2;
const BOSS_PROGRESS: usize = 0xd82e;
const DOOR_LOCK: usize = 0x0fea;
const WALLS: [usize; 2] = [0x1006, 0x1046];
const MUSIC_TAIL: usize = 0x0639;
const MUSIC_IDS: usize = 0x0619;
const MUSIC_DELAYS: usize = 0x0629;
#[cfg(test)]
const CURRENT_TRACK: usize = 0x07f5;
const STROKE_TAIL: usize = 0x0647;
const STROKE_IDS: usize = 0x0666;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Phase {
    Ready,
    Art(u16),
    Heading(u16),
    Typing {
        consumed: u16,
        wait: u16,
        audible: bool,
    },
    EscapeReady,
    Escape,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Cut {
    pub(super) phase: Phase,
    pub(super) glow: u16,
    pub(super) station: u16,
    pub(super) main_boss: bool,
    pub(super) locked: bool,
    pub(super) walls_hidden: [bool; 2],
    pub(super) countdown: interface_hud::Countdown,
}

pub(super) fn is_warning_room(seed: &FrameRecord) -> bool {
    // A seed in an earlier room cannot admit untouched foreground cells from this room.
    word(&seed.wram, ROUTINE).is_ok_and(|routine| matches!(routine, WARNING | ESCAPE))
}

fn byte(wram: &[u8], at: usize) -> Result<u8, Outcome> {
    wram.get(at)
        .copied()
        .ok_or_else(|| data_failure(None, "native warning byte"))
}

fn word(wram: &[u8], at: usize) -> Result<u16, Outcome> {
    let end = at.saturating_add(2);
    wram.get(at..end)
        .and_then(|bytes| bytes.try_into().ok())
        .map(u16::from_le_bytes)
        .ok_or_else(|| data_failure(None, "native warning word"))
}

fn loaded(pointer: u16, start: u16) -> Option<u16> {
    let bytes = pointer.checked_sub(start)?;
    bytes.is_multiple_of(7).then_some(bytes / 7)
}

const fn consumed(pointer: u16) -> Option<u16> {
    match pointer {
        0xc450 => Some(0),
        0xc459..=0xc46e => pointer.checked_sub(0xc458),
        0xc473..=0xc484 => pointer.checked_sub(0xc45c),
        0xc489..=0xc49a => pointer.checked_sub(0xc460),
        _ => None,
    }
}

fn phase(wram: &[u8]) -> Result<Phase, Outcome> {
    let bad = || data_failure(None, "native warning phase or progress");
    Ok(match (word(wram, ROUTINE)?, word(wram, PHASE)?) {
        (WARNING, 0) => Phase::Ready,
        (WARNING, 2) => Phase::Art(
            loaded(word(wram, PROGRESS)?, 0xc4cb)
                .filter(|run| (1..=6).contains(run))
                .ok_or_else(bad)?,
        ),
        (WARNING, 4) => Phase::Art(
            loaded(word(wram, PROGRESS)?, 0xc4fe)
                .filter(|run| (1..=7).contains(run))
                .ok_or_else(bad)?
                .saturating_add(6),
        ),
        (WARNING, 6) => Phase::Heading(word(wram, PROGRESS)?),
        (WARNING, 10) => Phase::Typing {
            consumed: consumed(word(wram, CURSOR)?).ok_or_else(bad)?,
            wait: word(wram, WAIT)?,
            audible: word(wram, AUDIBLE)? != 0,
        },
        (WARNING, 12) => Phase::EscapeReady,
        (ESCAPE, 0) => Phase::Escape,
        _ => return Err(bad()),
    })
}

pub(super) fn state(record: &FrameRecord) -> Result<Cut, Outcome> {
    let wram = &record.wram;
    let [left, right] = WALLS;
    let hidden = |slot| word(wram, slot).map(|flags| flags & 0x0100 != 0);
    Ok(Cut {
        phase: phase(wram)?,
        glow: word(wram, GLOW)?,
        station: word(wram, STATION)?,
        main_boss: byte(wram, BOSS_PROGRESS)? & 1 != 0,
        locked: word(wram, DOOR_LOCK)? != 0,
        walls_hidden: [hidden(left)?, hidden(right)?],
        countdown: super::hud::countdown_origin(wram)?,
    })
}

pub(super) fn music(before: &FrameRecord, after: &FrameRecord) -> Result<Option<u16>, Outcome> {
    let mut at = usize::from(word(&before.wram, MUSIC_TAIL)?);
    let tail = usize::from(word(&after.wram, MUSIC_TAIL)?);
    if [at, tail]
        .into_iter()
        .any(|cursor| cursor >= 16 || !cursor.is_multiple_of(2))
    {
        return Err(data_failure(None, "native warning music cursor"));
    }
    let mut delay = None;
    while at != tail {
        if word(&after.wram, MUSIC_IDS.wrapping_add(at))? == 7 {
            if delay.is_some() {
                return Err(data_failure(None, "native warning music duplicate"));
            }
            delay = Some(word(&after.wram, MUSIC_DELAYS.wrapping_add(at))?);
        }
        at = at.wrapping_add(2) % 16;
    }
    Ok(delay)
}

pub(super) fn strokes(before: &FrameRecord, after: &FrameRecord) -> Result<u8, Outcome> {
    let mut at = usize::from(byte(&before.wram, STROKE_TAIL)?);
    let tail = usize::from(byte(&after.wram, STROKE_TAIL)?);
    if at >= 16 || tail >= 16 {
        return Err(data_failure(None, "native warning stroke cursor"));
    }
    let mut count = 0u8;
    while at != tail {
        if byte(&after.wram, STROKE_IDS.wrapping_add(at))? == 0x45 {
            count = count.saturating_add(1);
        }
        at = at.wrapping_add(1) % 16;
    }
    Ok(count)
}

#[test]
#[ignore = "pinned record cache"]
fn warning_native_admits_both_room_origins() {
    use crate::cache::Cache;
    use crate::lock::{Lock, RowId};
    let root = std::path::PathBuf::from(std::env::var_os("SM_CACHE").unwrap());
    let lock = std::fs::read_to_string(std::env::var_os("SM_LOCK").unwrap()).unwrap();
    let lock = Lock::parse(&lock).unwrap();
    for row in ["chain-any-0006-E0B5", "samus-ceres-push"] {
        let cache = Cache::open(&root, lock.row(&RowId(row.to_owned())).unwrap()).unwrap();
        let record = |number| super::readers::record(&cache, number).unwrap();
        let origin = record(1270);
        let anchor = state(&origin).unwrap();
        assert_eq!(anchor.phase, Phase::Ready, "{row} admitted phase");
        assert_eq!(anchor.station, 1, "{row} admitted Getaway stage");
        assert_eq!(word(&origin.wram, SUBTITLES).unwrap(), 0, "{row} English");
        assert!(anchor.locked && !anchor.main_boss, "{row} admitted room");
        assert_eq!(anchor.countdown, interface_hud::Countdown::Dormant);
        assert_eq!(state(&record(1277)).unwrap().phase, Phase::Art(7));
        assert_eq!(state(&record(1284)).unwrap().phase, Phase::Heading(128));
        let mut prior = origin;
        for number in 1271..=1294 {
            let after = record(number);
            assert_eq!(
                music(&prior, &after).unwrap(),
                (number == 1284).then_some(8),
                "{row} cut {number} escape track offer"
            );
            let track = byte(&after.wram, CURRENT_TRACK).unwrap();
            assert_eq!(
                byte(&prior.wram, CURRENT_TRACK).unwrap() != track,
                number == 1293,
                "{row} cut {number} track reply"
            );
            if number == 1293 {
                assert_eq!(track, 7, "{row} selected escape track");
            }
            prior = after;
        }
        assert!(matches!(
            state(&record(1412)).unwrap().phase,
            Phase::Typing { consumed: 0, .. }
        ));
        let mut prior = record(1412);
        let mut count = 0u16;
        for number in 1413..=1584 {
            let after = record(number);
            assert!(
                matches!(state(&after).unwrap().phase, Phase::Typing { .. }),
                "{row} {number}"
            );
            count = count.saturating_add(u16::from(strokes(&prior, &after).unwrap()));
            prior = after;
        }
        assert_eq!(count, 27, "{row} native stroke offers");
        assert_eq!(
            state(&prior).unwrap().phase,
            Phase::Typing {
                consumed: 58,
                wait: 0,
                audible: false,
            }
        );
        assert_eq!(state(&record(1585)).unwrap().phase, Phase::EscapeReady);
        let started = record(1586);
        let activated = state(&started).unwrap();
        assert_eq!(activated.station, 2, "{row} activated Escape stage");
        assert_eq!(
            byte(&record(1585).wram, BOSS_PROGRESS).unwrap()
                ^ byte(&started.wram, BOSS_PROGRESS).unwrap(),
            1,
            "{row} only Ceres MainBoss changes"
        );
        assert_eq!(activated.phase, Phase::Escape);
        assert!(activated.main_boss && !activated.locked);
        assert_eq!(activated.walls_hidden, [true; 2]);
        assert!(matches!(
            activated.countdown,
            interface_hud::Countdown::Begun(_)
        ));
    }
}

// The room's prepared sprite buffer spans the low OAM entries and the high bits.
pub(super) const SHADOW_OAM: std::ops::Range<usize> = 0x0370..0x0590;

pub(super) fn contains_sprite_draw(
    image: &[u8],
    batch: &game::state::video::NmiBatch,
    count: usize,
) -> bool {
    use game::state::video::OAM_SPRITES;
    let Some((low, high)) = image.split_at_checked(OAM_SPRITES.saturating_mul(4)) else {
        return false;
    };
    let Some(entries) = batch.oam.low.get(..count) else {
        return false;
    };
    count != 0
        && low
            .as_chunks::<4>()
            .0
            .windows(count)
            .enumerate()
            .any(|(start, window)| {
                window
                    .iter()
                    .zip(entries)
                    .enumerate()
                    .all(|(offset, (raw, entry))| {
                        let at = start.saturating_add(offset);
                        let bits = |table: &[u8], index: usize| {
                            table
                                .get(index / 4)
                                .map(|byte| (byte >> ((index % 4).wrapping_mul(2))) & 3)
                        };
                        *raw == [entry.x, entry.y, entry.tile, entry.attributes]
                            && bits(high, at) == bits(&batch.oam.high, offset)
                    })
            })
}

#[cfg(test)]
fn assert_clock_sprite(
    record: &FrameRecord,
    figures: &data::assets::countdown::Figures,
    clock: interface_hud::BegunCountdown,
    expected_draw: bool,
) {
    let draw =
        game::exact::countdown::sprites(figures, clock.place.whole_pixels(), clock.time_left)
            .unwrap();
    let mut batch = game::state::video::NmiBatch::default();
    batch.draw_sprites(&draw);
    assert_eq!(
        contains_sprite_draw(
            record.wram.get(SHADOW_OAM).unwrap(),
            &batch,
            draw.pieces.len(),
        ),
        expected_draw,
        "native clock shadow OAM at cut {}",
        record.index
    );
    assert_eq!(
        contains_sprite_draw(&record.oam, &batch, draw.pieces.len()),
        expected_draw,
        "native physical clock OAM at cut {}",
        record.index
    );
}

#[test]
#[ignore = "pinned record cache and player ROM"]
fn warning_clock_holds_across_the_first_passage_steps() {
    use crate::cache::Cache;
    use crate::lock::{Lock, RowId};
    use std::path::PathBuf;
    let fixture = |name| PathBuf::from(std::env::var_os(name).unwrap());
    let root = fixture("SM_CACHE");
    let lock = Lock::parse(&std::fs::read_to_string(fixture("SM_LOCK")).unwrap()).unwrap();
    let rom = data::rom::Rom::new(std::fs::read(fixture("SM_ROM")).unwrap()).unwrap();
    let figures = import::countdown::import(rom.lorom()).unwrap();
    let witness = |row: &str, checks: &[(u64, u8, u8, bool)]| {
        let cache = Cache::open(&root, lock.row(&RowId(row.to_owned())).unwrap()).unwrap();
        for &(cut, age, hundredths, draws) in checks {
            let record = super::readers::record(&cache, cut).unwrap();
            let observed = state(&record).unwrap();
            assert_eq!(observed.phase, Phase::Escape, "{row} cut {cut}");
            assert!(observed.main_boss && !observed.locked, "{row} cut {cut}");
            let clock = match observed.countdown {
                interface_hud::Countdown::Begun(clock) => Some(clock),
                interface_hud::Countdown::Dormant => None,
            }
            .expect("native countdown is begun after warning activation");
            assert_eq!(
                clock.time_left,
                interface_hud::TimeLeft::new(0, 59, hundredths).unwrap(),
                "{row} cut {cut} clock"
            );
            assert_eq!(u8::from(clock.age), age, "{row} cut {cut} clock age");
            assert_clock_sprite(&record, &figures, clock, draws);
        }
    };
    witness(
        "chain-any-0006-E0B5",
        &[
            (1610, 24, 87, true),
            (1611, 24, 87, false),
            (1612, 24, 87, true),
        ],
    );
    witness(
        "samus-ceres-push",
        &[(1610, 24, 87, true), (1611, 25, 85, true)],
    );
}
