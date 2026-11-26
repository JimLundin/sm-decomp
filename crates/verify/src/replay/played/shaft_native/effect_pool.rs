//! Independent observations of occupied stationary effect slots, their centres and clip phases.

use super::word;
use crate::replay::{compare::Observation, data_failure, result::Outcome};
use core::num::NonZeroU16;
use enemy_interactions::{
    effect::{ActiveEffect, Burst, BurstKind, EffectPhase, EffectSlots},
    picture::RoomPoint,
};
use oracle_record::FrameRecord;

const SLOT_HEADER: usize = 0x1997;
const SLOT_X: usize = 0x1a4b;
const SLOT_Y: usize = 0x1a93;
const SLOT_CURSOR: usize = 0x1b47;
const SLOT_MAP: usize = 0x1b6b;
const SLOT_TIMER: usize = 0x1b8f;
const DUST_OR_EXPLOSION: u16 = 0xe509;
const NO_MAP: u16 = 0x8000;

pub(super) fn expected(record: &FrameRecord) -> Result<Vec<Observation>, Outcome> {
    (0..18)
        .map(|index| decode_slot(record, index).map(|effect| observation(index, effect)))
        .collect()
}

fn decode_slot(record: &FrameRecord, index: usize) -> Result<Option<ActiveEffect>, Outcome> {
    let read = |field: usize| word(record, field.saturating_add(index.saturating_mul(2)));
    let bad = || data_failure(Some(record.index), "shaft effect slot");
    match read(SLOT_HEADER)? {
        0 => return Ok(None),
        DUST_OR_EXPLOSION => (),
        _ => return Err(bad()),
    }
    let cursor = read(SLOT_CURSOR)?;
    let (kind, first) = match cursor {
        0xe138..=0xe150 => (BurstKind::SmallExplosion, 0xe138),
        0xe1ea..=0xe1fa => (BurstKind::Smoke, 0xe1ea),
        _ => return Err(bad()),
    };
    let offset = cursor.saturating_sub(first);
    let map_pending = read(SLOT_MAP)? == NO_MAP;
    let remaining = NonZeroU16::new(read(SLOT_TIMER)?).ok_or_else(bad)?;
    let phase = match (offset, map_pending) {
        (0, true) => EffectPhase::Waiting,
        (offset, false) if offset.is_multiple_of(4) => EffectPhase::Showing {
            frame: usize::from(offset / 4).checked_sub(1).ok_or_else(bad)?,
            remaining,
        },
        _ => return Err(bad()),
    };
    Ok(Some(ActiveEffect {
        burst: Burst {
            kind,
            centre: RoomPoint {
                x: read(SLOT_X)?,
                y: read(SLOT_Y)?,
            },
        },
        phase,
    }))
}

fn observation(index: usize, effect: Option<ActiveEffect>) -> Observation {
    let mut bytes = Vec::new();
    if let Some(effect) = effect {
        bytes.push(match effect.burst.kind {
            BurstKind::Smoke => 0,
            BurstKind::SmallExplosion => 1,
        });
        bytes.extend(effect.burst.centre.x.to_le_bytes());
        bytes.extend(effect.burst.centre.y.to_le_bytes());
        if let EffectPhase::Showing { frame, remaining } = effect.phase {
            bytes.extend(u16::try_from(frame).unwrap_or(u16::MAX).to_le_bytes());
            bytes.extend(remaining.get().to_le_bytes());
        }
    }
    Observation {
        field: format!("shaft-effects.slot-{index}"),
        bytes,
    }
}

pub(super) fn actual(pool: &EffectSlots) -> impl Iterator<Item = Observation> + '_ {
    pool.0
        .iter()
        .copied()
        .enumerate()
        .map(|(index, effect)| observation(index, effect))
}

#[test]
#[ignore = "pinned record cache and player ROM"]
fn live_bursts_restore_pending_fade_and_ordinary_room_effects() -> Result<(), Outcome> {
    use super::super::{
        Run, appearance, connected, movement, open_rows, polled, read_lock, seed_controller, tests,
    };
    use super::HALL;
    use crate::replay::compare::compare;

    let sources = tests::sources();
    let lock = read_lock(&sources)?;
    let mut rows = open_rows(&sources, &lock, HALL)?;
    let mut previous = rows.record(0)?;
    let mut controller = seed_controller(&previous)?;
    let content = connected::Content {
        movement: movement::Content::load(&sources.rom).unwrap(),
        appearance: appearance::Content::load(&sources.rom).unwrap(),
    };
    let mut run = Run::admit(&previous, &content, &sources.rom, HALL)?;
    let mut restored = Vec::new();
    let mut slot_checks = 0_usize;
    for cut in 1..=518 {
        let current = rows.record(cut)?;
        let (row, local) = rows.place(cut)?;
        let fields = run.step(controller, &previous, &current, cut, (&row, local))?;
        slot_checks = slot_checks.saturating_add(
            fields
                .iter()
                .filter(|field| field.field.starts_with("shaft-effects.slot-"))
                .count(),
        );
        for continuation in &mut restored {
            let resumed: &mut Run<'_> = continuation;
            let replayed = resumed.step(controller, &previous, &current, cut, (&row, local))?;
            assert!(
                compare(&fields, &replayed).is_ok(),
                "restored effect observations at {cut}"
            );
            assert!(
                run.state.station.escape_shaft == resumed.state.station.escape_shaft,
                "restored shaft owners at {cut}"
            );
            assert!(
                run.state.room_video == resumed.state.room_video,
                "restored pending and published video at {cut}"
            );
        }
        if matches!(cut, 241 | 285) {
            restored.push(restore_effects(&run, cut));
        }
        controller = controller.read(current.pad.0, polled(&current));
        previous = current;
    }
    assert_eq!(slot_checks, 5292);
    Ok(())
}

#[cfg(test)]
fn restore_effects<'a>(run: &super::super::Run<'a>, cut: u64) -> super::super::Run<'a> {
    use data::{
        assets::room_effects::{Picture, ShaftBurst},
        spritemap::Spritemap,
    };
    use game::production::played::{Fault, shaft::BurstArt};

    let shaft = run.state.station.escape_shaft.as_ref().unwrap();
    assert!(
        shaft.effects.slots().0.iter().any(Option::is_some),
        "occupied checkpoint at {cut}"
    );
    let bytes = canonical::to_bytes(&shaft.effects);
    let mut continuation = run.clone();
    let restored = continuation.state.station.escape_shaft.as_mut().unwrap();
    restored.effects = game::production::played::shaft::ShaftEffects::default();
    restored.actor_clocks =
        canonical::from_bytes(&canonical::to_bytes(&shaft.actor_clocks)).unwrap();
    let picture = || Picture {
        hold: NonZeroU16::MIN,
        sprites: Spritemap::default(),
    };
    let foreign = BurstArt::new(ShaftBurst {
        smoke: core::array::from_fn(|_| picture()),
        small_explosion: core::array::from_fn(|_| picture()),
    });
    for rejected in [None, Some(&foreign)] {
        assert_eq!(
            restored.restore_effects(&bytes, rejected),
            Err(Fault::ShaftEffectContent)
        );
        assert!(restored.effects.slots().0.iter().all(Option::is_none));
    }
    restored
        .restore_effects(&bytes, continuation.played.shaft_burst_art.as_ref())
        .unwrap();
    assert_eq!(restored.effects, shaft.effects, "restored pool at {cut}");
    let video = &mut continuation.state.room_video.video;
    *video = canonical::from_bytes(&canonical::to_bytes(video)).unwrap();
    if cut == 241 {
        assert!(
            video.batch.sprites.0 != 0,
            "effect picture still waiting for publication"
        );
    }
    continuation
}
