//! Native shaft actor placement, rotation and elevator observations.

mod effect_pool;
mod effects;
pub(in crate::replay) mod indexed;

use super::Frame;
#[cfg(test)]
use super::{Sources, open_rows, read_lock};
use crate::replay::{compare::Observation, data_failure, result::Outcome};
use boss_ceres_ridley::{BurstPoint, ShaftOverlayPhase};
use game::production::played::{Outputs, State};
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

fn reach(before: &FrameRecord, after: &FrameRecord) -> Result<Option<[u16; 3]>, Outcome> {
    if word(before, 0x0998)? == 8 && word(after, 0x0998)? == 32 {
        return Ok(Some([
            word(after, 0x0aa0)?,
            word(after, 0x0723)?,
            word(after, 0x0725)?,
        ]));
    }
    Ok(None)
}

// The completed slide requests NMI within the fetch, then returns on the first landing cut.
pub(super) fn fetch_retires(
    station: &game::production::played::station::State,
    doors: world_doors::Doors,
) -> bool {
    use world_doors::{Doors, Passage, Phase, WorkFrames};
    station.escape_shaft.is_some()
        && matches!(
            doors,
            Doors::Passing(Passage { phase: Phase::Landing(frame), .. })
                if frame == WorkFrames::FIRST
        )
}

pub(super) fn in_elevator(frame: &Frame<'_>) -> Result<bool, Outcome> {
    Ok(frame.place.0 == SHAFT && matches!(word(frame.current, 0x0998)?, 32 | 33))
}

pub(super) fn partial_palette(frame: &Frame<'_>, fade_steps: u8) -> bool {
    use world_doors::{Doors, FadeTime, Passage, Phase};
    matches!(
        frame.doors_before,
        Doors::Passing(Passage { phase: Phase::Brightening(FadeTime(step)), .. })
            if (1..=u16::from(fade_steps).saturating_add(1)).contains(&step)
    )
}

pub(super) fn compared(frame: &Frame<'_>) -> bool {
    let (row, cut) = frame.place;
    (row == HALL && (225..=278).contains(&cut)) || (row == SHAFT && cut <= 239)
}

fn fields(
    overlay: Overlay,
    rotation: Rotation,
    mount_offsets: impl IntoIterator<Item = (u16, i32, i32)>,
    placement: bool,
    reach: Option<[u16; 3]>,
    locked: bool,
) -> Vec<Observation> {
    let part = |name: &str, bytes: Vec<u8>| Observation {
        field: format!("shaft-actors.{name}"),
        bytes,
    };
    let mut rotation_bytes = vec![u8::from(rotation.turning), u8::from(rotation.forward)];
    rotation_bytes.extend(rotation.sample.to_le_bytes());
    rotation_bytes.extend(rotation.hold.to_le_bytes());
    rotation_bytes.extend(rotation.matrix.into_iter().flat_map(i16::to_le_bytes));
    let offsets = mount_offsets
        .into_iter()
        .flat_map(|(slot, x, y)| {
            slot.to_le_bytes()
                .into_iter()
                .chain(x.to_le_bytes())
                .chain(y.to_le_bytes())
        })
        .collect();
    let mut reach_bytes = vec![u8::from(reach.is_some())];
    reach_bytes.extend(reach.into_iter().flatten().flat_map(u16::to_le_bytes));
    vec![
        part("placed", vec![u8::from(placement)]),
        part("overlay", format!("{overlay:?}").into_bytes()),
        part("mounted-offsets", offsets),
        part("rotation", rotation_bytes),
        part("reach", reach_bytes),
        part("locked", vec![u8::from(locked)]),
    ]
}

pub(super) fn expected(frame: &Frame<'_>) -> Result<Vec<Observation>, Outcome> {
    let current = frame.current;
    let mut offsets = Vec::new();
    for slot in super::steam::slots(&current.wram)? {
        let at = 0x7010_usize.saturating_add(64_usize.saturating_mul(usize::from(slot)));
        offsets.push((
            slot,
            i32::from(word(current, at)?.cast_signed()),
            i32::from(word(current, at.saturating_add(2))?.cast_signed()),
        ));
    }
    let after = overlay(current)?;
    let placed = frame.place == (HALL, 225);
    let mut observations = fields(
        after,
        rotation(current)?,
        offsets,
        placed,
        reach(frame.previous, current)?,
        in_elevator(frame)?,
    );
    observations.extend(effects::expected(frame, placed, after)?);
    observations.extend(effect_pool::expected(current)?);
    let slots = core::iter::once(0).chain(super::steam::slots(&current.wram)?);
    for slot in slots {
        let offset = 64_usize.wrapping_mul(usize::from(slot));
        observations.push(clock(
            usize::from(slot),
            word(current, 0xfa4_usize.wrapping_add(offset))?,
            word(current, 0xfa2_usize.wrapping_add(offset))?,
        ));
    }
    Ok(observations)
}

pub(super) fn actual(state: &State, outputs: &Outputs) -> Vec<Observation> {
    let Some(shaft) = &state.station.escape_shaft else {
        return Vec::new();
    };
    let overlay = match shaft.overlay.phase {
        ShaftOverlayPhase::Opening => Overlay::Opening,
        ShaftOverlayPhase::Hidden => Overlay::Hidden,
        ShaftOverlayPhase::Rumbling {
            remaining,
            wait,
            point,
        } => Overlay::Rumble {
            left: u16::from(remaining.get()),
            wait: u16::from(wait.get()),
            next: match point {
                BurstPoint::First => 0,
                BurstPoint::Second => 1,
                BurstPoint::Third => 2,
                BurstPoint::Fourth => 3,
            },
        },
    };
    let rotation = shaft.rotation;
    let mut observations = fields(
        overlay,
        Rotation {
            turning: rotation.active,
            forward: rotation.direction == types::ceres_shaft::TiltDirection::Forward,
            sample: u16::try_from(rotation.next).unwrap_or(u16::MAX),
            hold: rotation.hold,
            matrix: rotation.view.matrix,
        },
        state
            .station
            .slots
            .vents
            .iter()
            .zip(&shaft.vents)
            .map(|(&slot, vent)| {
                (
                    u16::try_from(slot).unwrap_or(u16::MAX),
                    vent.offset.0.x,
                    vent.offset.0.y,
                )
            }),
        outputs.shaft_overlay_placement.is_some(),
        outputs.elevator_reach.map(|arrival| {
            let [fade_delay, fade_count] = arrival.scene_fade;
            [
                u16::from(arrival.hold),
                u16::from(fade_delay),
                u16::from(fade_count),
            ]
        }),
        shaft.locked,
    );
    observations.extend(effects::actual(outputs.shaft_actors));
    observations.extend(effect_pool::actual(shaft.effects.slots()));
    let clocks = &shaft.actor_clocks;
    let overlay = clocks.overlay();
    observations.push(clock(0, overlay.count, overlay.shake));
    observations.extend(clocks.vents.iter().map(|vent| {
        let state = vent.state();
        clock(usize::from(vent.slot), state.count, state.shake)
    }));
    observations
}

fn clock(slot: usize, count: u16, shake: u16) -> Observation {
    Observation {
        field: format!("shaft-actors.clock.{slot}"),
        bytes: count
            .to_le_bytes()
            .into_iter()
            .chain(shake.to_le_bytes())
            .collect(),
    }
}

#[cfg(test)]
fn check_placement(
    sources: &Sources,
    before: &FrameRecord,
    installed: &FrameRecord,
) -> Result<(), Outcome> {
    use boss_ceres_ridley::Stage;
    use core::num::Wrapping;
    use data::{assets::world::doorways::FALLING_TILE_TO_ELEVATOR, rom::Rom};
    use game::production::room_loading;
    use types::{Pair, RandomNumber, RandomSequence};

    let native_slots = super::steam::slots(&installed.wram)?;
    assert_eq!(native_slots, (2..13).collect::<Vec<_>>());
    let vents = super::steam::decode_shaft(&installed.wram)?;
    let mut random = RandomSequence {
        number: RandomNumber(word(before, 0x05e5)?),
    };
    let mut placed_random = random;
    let image = Rom::new(std::fs::read(&sources.rom).map_err(|_| data_failure(None, "shaft ROM"))?)
        .map_err(|_| data_failure(None, "shaft ROM"))?;
    let rooms = super::seed::rooms(&image)?;
    let progress = crate::replay::progress::decode(&before.wram)?;
    let view = crate::replay::shaft_entry::observed(before)?.view;
    let actors = room_loading::ceres_shaft_actors(
        FALLING_TILE_TO_ELEVATOR,
        room_loading::Run {
            progress: &progress,
            ceres: Stage::Escape,
        },
        &rooms,
        &view,
        &mut placed_random,
    )
    .map_err(|_| data_failure(Some(installed.index), "shaft population contents"))?;
    assert_eq!(actors.vents, vents);
    assert_eq!(actors.mounted_vents, (2..13).collect::<Vec<_>>());
    assert_eq!(actors.slots.vents, actors.mounted_vents);
    for (&slot, vent) in native_slots.iter().zip(&actors.vents) {
        use crate::source::{ENEMY_DATA, ENEMY_PARAMETER_1};
        let tag = ENEMY_DATA
            .word(ENEMY_PARAMETER_1, slot)
            .and_then(|field| field.read(&installed.wram))
            .map_err(|_| data_failure(Some(installed.index), "shaft mount tag"))?;
        let expected = match vent.facing {
            boss_ceres_ridley::VentFacing::Left => 4,
            boss_ceres_ridley::VentFacing::Right => 5,
            boss_ceres_ridley::VentFacing::Up | boss_ceres_ridley::VentFacing::Down => {
                panic!("shaft vent slot {slot} has nonhorizontal facing")
            }
        };
        assert_eq!(tag, expected, "shaft vent slot {slot} mounted tag");
    }
    assert_eq!(actors.slots.doors, [1]);
    assert_eq!(
        actors.shaft_overlay,
        Some(Pair {
            x: Wrapping(word(installed, 0x0f7a)?),
            y: Wrapping(word(installed, 0x0f7e)?),
        })
    );
    for (slot, vent) in (2..).zip(vents) {
        game_random::advance_random(&mut random);
        let boss_ceres_ridley::VentPhase::Placed(wait) = vent.phase else {
            panic!("shaft vent slot {slot} has already run");
        };
        assert_eq!(
            u16::from(wait.runs()),
            1_u16.saturating_add(random.number.0 % 32),
            "shaft vent slot {slot}"
        );
    }
    assert_eq!(word(installed, 0x05e5)?, random.number.0);
    assert_eq!(random, placed_random);
    Ok(())
}

#[test]
#[ignore = "pinned record cache"]
fn shaft_native_cuts_preserve_placement_and_reach() -> Result<(), Outcome> {
    use super::{platform::actor_fetch, tests};
    use game::production::played::passage::ActorFetch;

    let sources: Sources = tests::sources();
    let lock = read_lock(&sources)?;
    let mut hall = open_rows(&sources, &lock, HALL)?;
    let before = hall.record(224)?;
    let installed = hall.record(225)?;
    assert_eq!(actor_fetch(&installed), ActorFetch::Place);
    assert_eq!(overlay(&installed)?, Overlay::Opening);
    check_placement(&sources, &before, &installed)?;

    let before_first = hall.record(238)?;
    let first = hall.record(239)?;
    let published = hall.record(240)?;
    assert!(super::finished(&before_first));
    assert!(!super::finished(&first));
    assert!(super::finished(&published));
    assert_eq!(overlay(&before_first)?, Overlay::Opening);
    assert_eq!(
        overlay(&first)?,
        Overlay::Rumble {
            left: 48,
            wait: 0,
            next: 0
        }
    );

    assert_eq!(overlay(&first)?, overlay(&published)?);

    let mut previous = hall.record(278)?;
    let mut shaft = open_rows(&sources, &lock, SHAFT)?;
    let mut reached = Vec::new();
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
        if let Some(payload) = reach(&previous, &current)? {
            reached.push(cut);
            assert!(state.turning);
            assert_eq!(payload, [60, 0, 0]);
        }
        previous = current;
    }
    assert_eq!(sample_reads, 27);
    assert_eq!(reached, [164]);
    Ok(())
}

#[test]
#[ignore = "pinned record cache and player ROM"]
fn escape_shaft_from_boot_and_room_origin() -> Result<(), Outcome> {
    let sources = super::tests::sources();
    for (row, first, last) in [
        ("any-power-on", 0_u64, 13237_u64),
        ("chain-any-0010-DF8D", 0, 444),
        ("chain-any-0006-E0B5", 1690, 3075),
        ("chain-any-0006-E0B5", 1270, 3075),
    ] {
        let report = super::play(&sources, row, first, last)?;
        assert_eq!(report.stop, super::Stop::End, "{row} from {first}");
        assert_eq!(
            report.held,
            last.saturating_sub(first),
            "{row} from {first}"
        );
    }
    Ok(())
}
