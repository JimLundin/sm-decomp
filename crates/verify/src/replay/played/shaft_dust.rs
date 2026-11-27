//! Scoped observations of the carried wall-jump puff and its placement in both OAM buffers.

use core::ops::Range;
use data::spritemap::SpriteAttributes;
use enum_map::Enum;
use game::state::video::{OAM_HIGH_BYTES, OAM_SPRITES};
use oracle_record::FrameRecord;

use super::{
    PictureScope, Run, appearance, connected, movement, open_rows, platform, read_lock,
    seed_controller,
};
use crate::replay::{
    Sources,
    blackout::observe::oam_bytes,
    compare::Observation,
    data_failure,
    index::{
        Case, CaseKind, ContentProvider, ExpectedProvider, InputProvider, ObservationId,
        Responsibility, Start,
    },
    readers::ExpectedCut,
    result::{CaseResult, Counts, Outcome},
    runner,
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

const HALL: &str = "chain-any-0010-DF8D";
const SHAFT: &str = "chain-any-0011-DF45";
const FINAL_ROOM_WORK: u64 = 239;
const HALL_FINAL_CUT: u64 = 278;
const SHAFT_FINAL_ROOM_WORK: u64 = HALL_FINAL_CUT + 1 + FINAL_ROOM_WORK;

pub(in crate::replay) fn valid(case: &Case) -> bool {
    case.kind == CaseKind::Integrated
        && case.row == HALL
        && case.start
            == (Start::PlayedContinuation {
                seed: 0,
                frame: HALL_FINAL_CUT,
            })
        && case.end.cut_inclusive == SHAFT_FINAL_ROOM_WORK
        && case.responsibilities
            == [
                Responsibility::Appearance,
                Responsibility::Movement,
                Responsibility::Camera,
                Responsibility::Ceres,
            ]
        && case.content == ContentProvider::PlayerRomImport
        && case.inputs == InputProvider::PlayedShaftDust
        && case.expectations.provider == ExpectedProvider::PinnedRecordCache
        && case.expectations.fields == [ObservationId::ShaftDustState]
        && case.expectations.outputs
            == [
                ObservationId::ShaftDustPrepared,
                ObservationId::ShaftDustPublished,
                ObservationId::ShaftDustFollowingVent,
            ]
        && case.expectations.output_cut.is_none()
        && case.checkpoints
}

pub(in crate::replay) const fn valid_counts(counts: &Counts, horizon: usize) -> bool {
    counts.cuts == horizon
        && counts.fields == horizon
        && counts.outputs == horizon.saturating_mul(2).saturating_add(1)
        && counts.replies == 0
        && counts.pending_boundaries == 0
        && counts.restorations > 0
        && counts.restorations <= counts.publications
}

fn compared((row, local): (&str, u64)) -> bool {
    row == SHAFT && local <= FINAL_ROOM_WORK
}

pub(super) fn observations(
    run: &Run<'_>,
    frame: &super::Frame<'_>,
    (native, written): (&mut Vec<Observation>, &mut Vec<Observation>),
) -> Result<(), Outcome> {
    if compared(frame.place) {
        let looks = looks(run)?;
        native.extend(expected(frame.current, looks)?);
        written.extend(actual(run, looks)?);
        if frame.place.1 == 17 && run.picture_scope != PictureScope::ShaftActors {
            // The dust and aggregate witnesses check the shifted vent entry. The shaft
            // actor witness compares its own mounted spans during locked work.
            let video = &run.state.room_video.video;
            let prepared = frame.current.wram.get(PREPARED_OAM).unwrap_or_default();
            native.push(following_vent(prepared, &frame.current.oam)?);
            written.push(following_vent(
                &oam_bytes(&video.batch.oam),
                &oam_bytes(&video.memory.oam),
            )?);
        }
    }
    Ok(())
}

fn following_vent(prepared: &[u8], published: &[u8]) -> Result<Observation, Outcome> {
    const SLOT: usize = 63;
    let mut bytes = Vec::with_capacity(10);
    for picture in [prepared, published] {
        let entry = picture
            .get(SLOT * 4..SLOT * 4 + 4)
            .ok_or_else(|| data_failure(None, "shaft dust following vent OAM entry"))?;
        let high = picture
            .get(LOW_BYTES + SLOT / 4)
            .ok_or_else(|| data_failure(None, "shaft dust following vent OAM high bit"))?;
        bytes.extend(entry);
        bytes.push((high >> ((SLOT % 4) * 2)) & 3);
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

pub(in crate::replay) fn run(
    case: &Case,
    sources: &Sources,
    result: &mut CaseResult,
) -> Result<(), Outcome> {
    let lock = read_lock(sources)?;
    let mut rows = open_rows(sources, &lock, &case.row)?;
    let Start::PlayedContinuation { seed, frame } = case.start else {
        return Err(data_failure(None, "shaft dust continuation start"));
    };
    let mut previous = rows.record(seed)?;
    let mut controller = seed_controller(&previous)?;
    let content = connected::Content {
        movement: movement::Content::load(&sources.rom)?,
        appearance: appearance::Content::load(&sources.rom)?,
    };
    let mut run = Run::admit(&previous, &content, &sources.rom, &case.row)?;
    let mut witness = Witness::default();
    for cut in seed.saturating_add(1)..=case.end.cut_inclusive {
        let current = rows.record(cut)?;
        let (row, local) = rows.place(cut)?;
        let place = (row.as_str(), local);
        let null = (cut > frame)
            .then(|| {
                let mut fields = actual(&run, looks(&run)?)?.to_vec();
                if local == 17 {
                    let video = &run.state.room_video.video;
                    fields.push(following_vent(
                        &oam_bytes(&video.batch.oam),
                        &oam_bytes(&video.memory.oam),
                    )?);
                }
                Ok::<_, Outcome>(fields)
            })
            .transpose()?;
        let fields = run.step(controller, &previous, &current, cut, place)?;
        if let Some(null) = null {
            witness.cut(
                &run,
                &Cut {
                    previous: &previous,
                    current: &current,
                    number: cut,
                    place,
                },
                controller,
                fields,
                result,
                &null,
            )?;
        }
        controller = controller.read(current.pad.0, platform::polled(&current));
        previous = current;
    }
    if witness.restored.is_some() {
        return Err(Outcome::CheckpointFailure {
            cut: case.end.cut_inclusive,
        });
    }
    result.observation_sha256 = crate::lock::Sha256::of(&witness.digest).to_string();
    Ok(())
}

struct Cut<'a> {
    previous: &'a FrameRecord,
    current: &'a FrameRecord,
    number: u64,
    place: (&'a str, u64),
}

#[derive(Default)]
struct Witness<'a> {
    digest: Vec<u8>,
    restored: Option<(u64, Run<'a>)>,
}

impl<'a> Witness<'a> {
    fn cut(
        &mut self,
        run: &Run<'a>,
        frame: &Cut<'_>,
        controller: platform::Controller,
        mut fields: Vec<Observation>,
        result: &mut CaseResult,
        null: &[Observation],
    ) -> Result<(), Outcome> {
        fields.retain(|field| {
            matches!(
                field.field.as_str(),
                STATE | PREPARED | PUBLISHED | FOLLOWING_VENT
            )
        });
        let field_count = if frame.place.1 == 17 { 4 } else { 3 };
        if fields.len() != field_count {
            return Err(data_failure(
                Some(frame.number),
                "wall-jump dust comparison opportunity",
            ));
        }
        let state = fields
            .iter()
            .filter(|field| field.field == STATE)
            .cloned()
            .collect::<Vec<_>>();
        let occupied = run.state.samus.appearance.0.dust.is_some();
        let published = fields
            .iter()
            .any(|field| field.field == PUBLISHED && !field.bytes.is_empty());
        let expected = ExpectedCut {
            semantics: state,
            semantic_count: 1,
            fields,
            output_count: field_count.saturating_sub(1),
            replies: 0,
        };
        runner::record_cut(result, &mut self.digest, &expected, null, frame.number)?;
        result.counts.publications = result
            .counts
            .publications
            .saturating_add(usize::from(published));
        if let Some((origin, continuation)) = &mut self.restored {
            continuation
                .step(
                    controller,
                    frame.previous,
                    frame.current,
                    frame.number,
                    frame.place,
                )
                .map_err(|_| Outcome::CheckpointFailure { cut: *origin })?;
            if continuation.state != run.state || continuation.blackout != run.blackout {
                return Err(Outcome::CheckpointFailure { cut: *origin });
            }
            if !occupied {
                result.counts.restorations = result.counts.restorations.saturating_add(1);
                self.restored = None;
            }
        } else if occupied {
            self.restored = Some((frame.number, restore(run, frame.number)?));
        }
        Ok(())
    }
}

fn restore<'a>(run: &Run<'a>, cut: u64) -> Result<Run<'a>, Outcome> {
    let mut resumed = run.clone();
    let failed = || Outcome::CheckpointFailure { cut };
    let appearance = &mut resumed.state.samus.appearance;
    *appearance = canonical::from_bytes(&canonical::to_bytes(appearance)).map_err(|_| failed())?;
    let saved_picture = run.state.room_video.picture_checkpoint();
    *resumed.state.room_video.video = game::state::video::Video::default();
    resumed
        .state
        .room_video
        .restore_picture(&saved_picture)
        .map_err(|_| failed())?;
    if resumed.state != run.state {
        return Err(failed());
    }
    Ok(resumed)
}
