//! Carry the return hall into the locked shaft and compare the selected C1 picture.

use super::{HALL, SHAFT, word};
use crate::replay::{
    Sources, appearance,
    blackout::{observe::oam_bytes, standing::packed_oam},
    compare::Observation,
    connected, data_failure,
    index::{
        Case, CaseKind, ContentProvider, ExpectedProvider, ObservationId, Responsibility, Start,
    },
    movement,
    played::{PictureScope, Run, open_rows, polled, read_lock, seed_controller},
    readers::ExpectedCut,
    result::{CaseResult, Outcome},
    runner,
};
use game::production::played::{room_video::RoomVideo, shaft::ShaftEffects};
use oracle_record::FrameRecord;

const FIRST: u64 = 225;
const LAST: u64 = 518;
const HALL_PREPARES: u64 = 239;
const HALL_PUBLISHES: u64 = 240;
const LOCKED_PREFIX: usize = 24;

pub(in crate::replay) fn valid(case: &Case) -> bool {
    case.kind == CaseKind::Integrated
        && case.row == HALL
        && case.start
            == (Start::PlayedContinuation {
                seed: 0,
                frame: FIRST - 1,
            })
        && case.end.cut_inclusive == LAST
        && case.responsibilities == [Responsibility::ShaftPicture]
        && case.content == ContentProvider::PlayerRomImport
        && case.expectations.provider == ExpectedProvider::PinnedRecordCache
        && case.expectations.fields
            == [
                ObservationId::ShaftPictureClocks,
                ObservationId::ShaftPictureEffects,
            ]
        && case.expectations.outputs == [ObservationId::ShaftPictureOam]
        && case.expectations.output_cut.is_none()
        && case.checkpoints
}

fn c1_field(field: &Observation) -> bool {
    field.field.starts_with("shaft-effects.slot-") || field.field.starts_with("shaft-actors.clock.")
}

fn packed(
    cut: u64,
    field: &str,
    native: &[u8],
    produced: &game::state::video::Oam,
    previous: &game::state::video::Oam,
    slots: core::ops::Range<usize>,
) -> Result<(Observation, Observation), Outcome> {
    let bad = || data_failure(Some(cut), "shaft picture OAM span");
    let expected = packed_oam(native, slots.clone()).ok_or_else(bad)?;
    let actual = packed_oam(&oam_bytes(produced), slots.clone()).ok_or_else(bad)?;
    let prior = packed_oam(&oam_bytes(previous), slots).ok_or_else(bad)?;
    held(cut, field, expected == actual)?;
    Ok((
        Observation {
            field: field.into(),
            bytes: expected,
        },
        Observation {
            field: field.into(),
            bytes: prior,
        },
    ))
}

fn held(cut: u64, field: &str, matches: bool) -> Result<(), Outcome> {
    if matches {
        Ok(())
    } else {
        Err(Outcome::Difference {
            cut,
            field: field.into(),
        })
    }
}

fn picture(
    room: &RoomVideo,
    prior: &RoomVideo,
    current: &FrameRecord,
    cut: u64,
    (row, local): (&str, u64),
    retained: usize,
) -> Result<(Vec<Observation>, Vec<Observation>), Outcome> {
    let video = &room.video;
    let shadow = || {
        current
            .wram
            .get(super::super::super::ridley_warning_native::SHADOW_OAM)
            .ok_or_else(|| data_failure(Some(cut), "shaft shadow OAM"))
    };
    let pending = |name, slots| {
        packed(
            cut,
            name,
            shadow()?,
            &video.batch.oam,
            &prior.video.batch.oam,
            slots,
        )
    };
    let published = |name, slots| {
        packed(
            cut,
            name,
            &current.oam,
            &video.memory.oam,
            &prior.video.memory.oam,
            slots,
        )
    };
    Ok(match (row, local) {
        (HALL, HALL_PREPARES) => {
            held(
                cut,
                "shaft-picture.pending-cursor",
                word(current, 0x0590)? == video.batch.sprites.0.saturating_mul(4)
                    && video.batch.sprites.0 == 16,
            )?;
            vec![pending("shaft-picture.prepared-prefix", 0..16)?]
        }
        (HALL, HALL_PUBLISHES) => {
            held(
                cut,
                "shaft-picture.consumed-cursor",
                video.batch.sprites.0 == 0,
            )?;
            vec![published("shaft-picture.published-prefix", 0..16)?]
        }
        (HALL, 241 | 256) => vec![pending("shaft-picture.pending-first-effect", 0..1)?],
        (HALL, 243) => vec![published("shaft-picture.published-first-effect", 0..1)?],
        (SHAFT, 0 | 1 | 28) => {
            let first: usize = match local {
                0 => 23,
                1 => 24,
                _ => 29,
            };
            vec![pending(
                "shaft-picture.overlay-after-effects",
                first..first.saturating_add(12),
            )?]
        }
        (SHAFT, 165..=239) => {
            let mut observations = vec![
                pending("shaft-picture.pending-locked-prefix", 0..LOCKED_PREFIX)?,
                published("shaft-picture.published-locked-prefix", 0..LOCKED_PREFIX)?,
            ];
            if retained > LOCKED_PREFIX {
                observations.extend([
                    pending("mounted-vent.pending-oam", LOCKED_PREFIX..retained)?,
                    published("mounted-vent.published-oam", LOCKED_PREFIX..retained)?,
                ]);
            }
            if local == 239 {
                held(
                    cut,
                    "shaft-picture.suspended-cursor",
                    word(current, 0x0590)? == video.batch.sprites.0.saturating_mul(4),
                )?;
            }
            observations
        }
        _ => Vec::new(),
    }
    .into_iter()
    .unzip())
}

pub(in crate::replay::played) fn restore<'a>(run: &Run<'a>, cut: u64) -> Result<Run<'a>, Outcome> {
    let mut candidate = run.clone();
    let failed = || Outcome::CheckpointFailure { cut };
    candidate
        .state
        .room_video
        .restore_picture(&run.state.room_video.picture_checkpoint())
        .map_err(|_| failed())?;
    if let Some(shaft) = &run.state.station.escape_shaft {
        let retained = candidate
            .state
            .station
            .escape_shaft
            .as_mut()
            .ok_or_else(failed)?;
        retained.actor_clocks = canonical::from_bytes(&canonical::to_bytes(&shaft.actor_clocks))
            .map_err(|_| failed())?;
        retained.effects = ShaftEffects::default();
        retained
            .restore_effects(
                &canonical::to_bytes(&shaft.effects),
                run.played.shaft_burst_art.as_ref(),
            )
            .map_err(|_| failed())?;
    }
    if candidate.state.room_video != run.state.room_video
        || candidate.state.station.escape_shaft != run.state.station.escape_shaft
    {
        return Err(failed());
    }
    Ok(candidate)
}

fn check_hall_publication(before: &Run<'_>, after: &Run<'_>, cut: u64) -> Result<(), Outcome> {
    let previous = &before.state.room_video.video;
    let published = &after.state.room_video.video.memory;
    if cut == HALL_PREPARES {
        held(
            cut,
            "shaft-picture.physical-hold",
            &previous.memory == published,
        )?;
    }
    if cut == HALL_PUBLISHES {
        let mut batch = previous.batch.clone();
        batch.finalise_oam();
        held(
            cut,
            "shaft-picture.publish-held-batch",
            batch.oam == published.oam,
        )?;
        let prior = before
            .state
            .station
            .escape_shaft
            .as_ref()
            .ok_or_else(|| data_failure(Some(cut), "shaft publication owners"))?;
        let continued = after
            .state
            .station
            .escape_shaft
            .as_ref()
            .ok_or_else(|| data_failure(Some(cut), "shaft publication owners"))?;
        held(
            cut,
            "shaft-picture.hold-actor-clocks",
            prior.actor_clocks == continued.actor_clocks,
        )?;
        held(
            cut,
            "shaft-picture.hold-effect-slots",
            prior.effects == continued.effects,
        )?;
    }
    Ok(())
}

pub(in crate::replay) fn run(
    case: &Case,
    sources: &Sources,
    result: &mut CaseResult,
) -> Result<(), Outcome> {
    let lock = read_lock(sources)?;
    let mut rows = open_rows(sources, &lock, &case.row)?;
    let mut previous = rows.record(0)?;
    let mut controller = seed_controller(&previous)?;
    let content = connected::Content {
        movement: movement::Content::load(&sources.rom)?,
        appearance: appearance::Content::load(&sources.rom)?,
    };
    let mut run = Run::admit(&previous, &content, &sources.rom, &case.row)?;
    run.picture_scope = PictureScope::ShaftActors;
    let mut retained = LOCKED_PREFIX;
    let mut digest = Vec::new();
    let proof = (|| -> Result<(), Outcome> {
        for cut in 1..=LAST {
            let current = rows.record(cut)?;
            if current.video_frame != previous.video_frame.wrapping_add(1) {
                return Err(data_failure(Some(cut), "played rows join"));
            }
            let (row, local) = rows.place(cut)?;
            let place = (row.as_str(), local);
            let prior = cut.saturating_sub(1);
            let mut checkpoint = (cut >= FIRST).then(|| restore(&run, prior)).transpose()?;
            let matched = run.step(controller, &previous, &current, cut, place)?;
            if let Some(restored) = &mut checkpoint {
                check_hall_publication(restored, &run, cut)?;
                if place.0 == SHAFT && (165..=239).contains(&local) {
                    let art = run
                        .played
                        .shaft_vent_art
                        .as_ref()
                        .ok_or_else(|| data_failure(Some(cut), "mounted vent art"))?;
                    retained = retained.max(super::super::steam::mounted_picture::live_end(
                        &previous, &current, art,
                    )?);
                }
                let cuts = &mut result.counts;
                let mut frozen = super::actual(&restored.state, None);
                let (outputs, held_picture) = picture(
                    &run.state.room_video,
                    &restored.state.room_video,
                    &current,
                    cut,
                    place,
                    retained,
                )?;
                frozen.extend(held_picture);
                let replayed = restored.step(controller, &previous, &current, cut, place);
                if replayed.as_ref() != Ok(&matched)
                    || restored.state.room_video != run.state.room_video
                    || restored.state.station.escape_shaft != run.state.station.escape_shaft
                {
                    return Err(Outcome::CheckpointFailure { cut: prior });
                }
                let semantics: Vec<_> = matched.into_iter().filter(c1_field).collect();
                let observed = ExpectedCut {
                    semantic_count: semantics.len(),
                    output_count: outputs.len(),
                    fields: semantics.iter().cloned().chain(outputs).collect(),
                    semantics,
                    replies: 0,
                };
                cuts.restorations = cuts.restorations.saturating_add(1);
                if cut == HALL_PUBLISHES {
                    cuts.publications = cuts.publications.saturating_add(1);
                    cuts.pending_boundaries = cuts.pending_boundaries.saturating_add(1);
                    cuts.pending_restorations = cuts.pending_restorations.saturating_add(1);
                }
                runner::record_cut(result, &mut digest, &observed, &frozen, cut)?;
            }
            controller = controller.read(current.pad.0, polled(&current));
            previous = current;
        }
        Ok(())
    })();
    if !digest.is_empty() {
        result.observation_sha256 = crate::lock::Sha256::of(&digest).to_string();
    }
    proof
}
