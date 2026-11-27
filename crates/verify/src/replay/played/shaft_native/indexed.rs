//! Carry the return hall into the locked shaft and compare the selected C1 picture.

use super::word;
use crate::replay::played::room_video::{Lane, Picture};
use crate::replay::{
    blackout::{observe::oam_bytes, standing::packed_oam},
    compare::Observation,
    data_failure,
    index::{
        Case, CaseKind, ContentProvider, ExpectedProvider, ObservationId, Responsibility, Start,
    },
    played::{PictureScope, Run},
    readers::ExpectedCut,
    result::Outcome,
    runner,
};
use game::production::played::{room_video::RoomVideo, shaft::ShaftEffects};
use oracle_record::FrameRecord;

pub(in crate::replay) fn valid(case: &Case) -> bool {
    case.kind == CaseKind::Integrated
        && matches!(case.start, Start::PlayedContinuation { .. })
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

struct Selection<'a> {
    writer: Option<&'a Picture>,
    prepared_prefix: u8,
    overlay_pieces: u8,
    locked_prefix: usize,
    locked: bool,
    suspended: bool,
}

impl<'a> Selection<'a> {
    fn at(
        witnesses: &'a crate::replay::played::witnesses::Witnesses,
        place: (&str, u64),
        previous: &FrameRecord,
        current: &FrameRecord,
        cut: u64,
    ) -> Result<Self, Outcome> {
        let shaft = witnesses.shaft()?;
        if !shaft.actors(place) {
            return Err(data_failure(Some(cut), "shaft picture actor trace"));
        }
        let writers = witnesses.room_video()?;
        Ok(Self {
            writer: writers.picture(place),
            prepared_prefix: shaft.prepared_prefix,
            overlay_pieces: writers.overlay_pieces,
            locked_prefix: usize::from(shaft.locked_prefix),
            locked: shaft.elevator.contains(place)
                && matches!(word(previous, 0x0998)?, 32 | 33)
                && matches!(word(current, 0x0998)?, 32 | 33),
            suspended: crate::replay::blackout::final_room_overrun(previous, current),
        })
    }
}

fn picture(
    room: &RoomVideo,
    prior: &RoomVideo,
    current: &FrameRecord,
    cut: u64,
    selected: &Selection<'_>,
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
    let prefix = usize::from(selected.prepared_prefix);
    Ok(match selected.writer {
        Some(Picture::Prepare) => {
            held(
                cut,
                "shaft-picture.pending-cursor",
                word(current, 0x0590)? == video.batch.sprites.0.saturating_mul(4)
                    && video.batch.sprites.0 == u16::from(selected.prepared_prefix),
            )?;
            vec![pending("shaft-picture.prepared-prefix", 0..prefix)?]
        }
        Some(Picture::Publish) => {
            held(
                cut,
                "shaft-picture.consumed-cursor",
                video.batch.sprites.0 == 0,
            )?;
            vec![published("shaft-picture.published-prefix", 0..prefix)?]
        }
        Some(Picture::FirstEffect { lane, slot }) => {
            let first = usize::from(*slot);
            let span = first..first.saturating_add(1);
            vec![match lane {
                Lane::Prepared => pending("shaft-picture.pending-first-effect", span)?,
                Lane::Published => published("shaft-picture.published-first-effect", span)?,
            }]
        }
        Some(Picture::Overlay { slot }) => {
            let first = usize::from(*slot);
            vec![pending(
                "shaft-picture.overlay-after-effects",
                first..first.saturating_add(usize::from(selected.overlay_pieces)),
            )?]
        }
        None if selected.locked => {
            let mut observations = vec![
                pending(
                    "shaft-picture.pending-locked-prefix",
                    0..selected.locked_prefix,
                )?,
                published(
                    "shaft-picture.published-locked-prefix",
                    0..selected.locked_prefix,
                )?,
            ];
            if retained > selected.locked_prefix {
                observations.extend([
                    pending("mounted-vent.pending-oam", selected.locked_prefix..retained)?,
                    published(
                        "mounted-vent.published-oam",
                        selected.locked_prefix..retained,
                    )?,
                ]);
            }
            if selected.suspended {
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

fn check_hall_publication(
    before: &Run<'_>,
    after: &Run<'_>,
    cut: u64,
    writer: Option<&Picture>,
) -> Result<(), Outcome> {
    let previous = &before.state.room_video.video;
    let published = &after.state.room_video.video.memory;
    if matches!(writer, Some(Picture::Prepare)) {
        held(
            cut,
            "shaft-picture.physical-hold",
            &previous.memory == published,
        )?;
    }
    if matches!(writer, Some(Picture::Publish)) {
        held(
            cut,
            "shaft-picture.pending-publication",
            before.state.room_video.pending_platform().is_some(),
        )?;
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

pub(in crate::replay::played) fn run(
    mut room: crate::replay::played::indexed::AdmittedRoom<'_, '_>,
    case: &Case,
    result: &mut runner::Receipt,
) -> Result<(), Outcome> {
    let Start::PlayedContinuation { frame, .. } = case.start else {
        return Err(data_failure(None, "shaft picture continuation start"));
    };
    room.run.picture_scope = PictureScope::ShaftActors;
    let mut retained = usize::from(room.run.witnesses.shaft()?.locked_prefix);
    room.replay(case, |run, turn, _| {
        let cut = turn.cut;
        let current = turn.current;
        let previous = turn.previous;
        let place = turn.place;
        let prior = cut.saturating_sub(1);
        let mut checkpoint = (cut > frame).then(|| restore(run, prior)).transpose()?;
        let matched = turn.fields(run)?;
        if let Some(restored) = &mut checkpoint {
            let selected = Selection::at(run.witnesses, place, previous, current, cut)?;
            check_hall_publication(restored, run, cut, selected.writer)?;
            if selected.locked {
                let art = run
                    .played
                    .shaft_vent_art
                    .as_ref()
                    .ok_or_else(|| data_failure(Some(cut), "mounted vent art"))?;
                retained = retained.max(super::super::steam::mounted_picture::live_end(
                    previous, current, art,
                )?);
            }
            let mut frozen = super::actual(&restored.state, None);
            let (outputs, held_picture) = picture(
                &run.state.room_video,
                &restored.state.room_video,
                current,
                cut,
                &selected,
                retained,
            )?;
            frozen.extend(held_picture);
            let replayed = turn.fields(restored);
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
            result.restored(1)?;
            if matches!(selected.writer, Some(Picture::Publish)) {
                result.published()?;
                result.pending()?;
                result.resumed()?;
            }
            result.record(&observed, &frozen, cut)?;
        }
        Ok(())
    })?;
    if result.counts.publications != 1 || result.counts.outputs == 0 {
        return Err(data_failure(
            Some(case.end.cut_inclusive),
            "shaft picture publication witness",
        ));
    }
    Ok(())
}
