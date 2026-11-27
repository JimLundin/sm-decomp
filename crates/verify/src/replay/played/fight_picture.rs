//! Independently compare the room's ordinary sprite append and its NMI publication.

use core::ops::Range;

use game::state::video::{Oam, OamEntry, Video};
use oracle_record::FrameRecord;

use crate::replay::index::{Case, PlayedWindow, Start};
use crate::replay::{compare::Observation, data_failure, result::Outcome};

pub(super) const fn valid_window(case: &Case) -> bool {
    matches!((&case.start, &case.expectations.played),
        (Start::CacheCut { frame }, Some(PlayedWindow::FightPicture { appends: [first, second] }))
        if *frame < first.cut && first.cut < second.cut && second.cut == case.end.cut_inclusive
            && first.cursor != second.cursor && first.cursor <= 120 && second.cursor <= 120)
}

pub(super) const DOOR_CURSOR: &str = "ridley.ordinary.door-cursor";
pub(super) const PREPARED: &str = "ridley.ordinary.prepared-oam";
pub(super) const PHYSICAL: &str = "ridley.ordinary.physical-oam";
const SHADOW: Range<usize> = 0x0370..0x0590;
const LOW_BYTES: usize = 512;

fn image(field: &str, bytes: Vec<u8>) -> Observation {
    Observation {
        field: field.into(),
        bytes,
    }
}

fn packed(oam: &Oam) -> Vec<u8> {
    oam.low
        .iter()
        .flat_map(|entry| [entry.x, entry.y, entry.tile, entry.attributes])
        .chain(oam.high)
        .collect()
}

pub(super) fn admit(video: &mut Video, seed: &FrameRecord) -> Result<(), Outcome> {
    let bad = || data_failure(Some(seed.index), "ordinary fight prepared OAM seed");
    let image = seed.wram.get(SHADOW).ok_or_else(bad)?;
    let (low, high) = image.split_at_checked(LOW_BYTES).ok_or_else(bad)?;
    let high = high.try_into().map_err(|_| bad())?;
    let mut prepared = Oam {
        high,
        ..Oam::default()
    };
    for (output, &[x, y, tile, attributes]) in prepared.low.iter_mut().zip(low.as_chunks::<4>().0) {
        *output = OamEntry {
            x,
            y,
            tile,
            attributes,
        };
    }
    video.batch.oam = prepared;
    video.memory.oam = crate::replay::landing_site::oam(seed)?;
    Ok(())
}

pub(super) fn append_observations(
    record: &FrameRecord,
    expected_cursor: u8,
    video: &Video,
    door: Option<&Range<usize>>,
    (expected, actual): (&mut Vec<Observation>, &mut Vec<Observation>),
) -> Result<(), Outcome> {
    // The independent writer trace identifies the door span in the shared OAM buffer.
    let prepared = record
        .wram
        .get(SHADOW)
        .ok_or_else(|| data_failure(Some(record.index), PREPARED))?;
    let cursor = |start: i64| Observation::quantity(DOOR_CURSOR, start);
    let observed_cursor = door
        .filter(|span| span.len() == 8)
        .and_then(|span| i64::try_from(span.start).ok())
        .unwrap_or(-1);
    expected.extend([
        cursor(i64::from(expected_cursor)),
        image(PREPARED, prepared.to_vec()),
        image(PHYSICAL, record.oam.to_vec()),
    ]);
    actual.extend([
        cursor(observed_cursor),
        image(PREPARED, packed(&video.batch.oam)),
        image(PHYSICAL, packed(&video.memory.oam)),
    ]);
    Ok(())
}

fn restored<'a>(run: &super::Run<'a>, cut: u64) -> Result<super::Run<'a>, Outcome> {
    use canonical::model::Model;
    use game::state::owned::FightWingsPin;

    let failed = || Outcome::CheckpointFailure { cut };
    let mut restored = run.clone();
    let state = &mut restored.state;
    let ridley = state.station.ridley.as_mut().ok_or_else(failed)?;
    let clock = canonical::to_bytes(&Model::<FightWingsPin>(ridley.wings));
    ridley.wings = canonical::from_bytes::<Model<FightWingsPin>>(&clock)
        .map_err(|_| failed())?
        .0;
    let camera = canonical::to_bytes(&state.camera);
    state.camera = canonical::from_bytes(&camera).map_err(|_| failed())?;
    let view = &mut state.passage.room.entry.view;
    let encoded_view = canonical::to_bytes(view);
    *view = canonical::from_bytes(&encoded_view).map_err(|_| failed())?;
    let video = state.room_video.picture_checkpoint();
    *state.room_video.video = Video::default();
    state
        .room_video
        .restore_picture(&video)
        .map_err(|_| failed())?;
    if state != &run.state {
        return Err(failed());
    }
    Ok(restored)
}

pub(super) fn run(
    run: &mut super::Run<'_>,
    rows: &mut super::rows::Rows<'_>,
    mut previous: FrameRecord,
    mut controller: super::Controller,
    case: &crate::replay::index::Case,
    result: &mut crate::replay::result::CaseResult,
) -> Result<(), Outcome> {
    use crate::replay::{readers::ExpectedCut, runner};

    let Some(PlayedWindow::FightPicture { appends }) = &case.expectations.played else {
        return Err(data_failure(None, "ordinary fight picture witnesses"));
    };
    admit(&mut run.state.room_video.video, &previous)?;
    let mut digest = Vec::new();
    let mut continuation = None;
    let video = &run.state.room_video.video;
    let frozen = [
        Observation::quantity(DOOR_CURSOR, -1),
        image(PREPARED, packed(&video.batch.oam)),
        image(PHYSICAL, packed(&video.memory.oam)),
    ];
    for cut in previous.index.saturating_add(1)..=case.end.cut_inclusive {
        let current = rows.record(cut)?;
        let (row, local) = rows.place(cut)?;
        let append = appends.iter().find(|append| append.cut == cut);
        let frame = |run: &mut super::Run<'_>| {
            run.picture_scope = append.map_or(super::PictureScope::Room, |point| {
                super::PictureScope::Fight(point.cursor)
            });
            run.step(controller, &previous, &current, cut, (&row, local))
        };
        let mut fields = frame(run)?;
        if let Some(checkpoint) = &mut continuation {
            let rerun = frame(checkpoint).map_err(|_| Outcome::CheckpointFailure { cut })?;
            if rerun != fields || checkpoint.state != run.state {
                return Err(Outcome::CheckpointFailure { cut });
            }
        }
        if appends.first().is_some_and(|first| first.cut == cut) {
            continuation = Some(restored(run, cut)?);
        }
        if append.is_some() {
            fields
                .retain(|field| matches!(field.field.as_str(), DOOR_CURSOR | PREPARED | PHYSICAL));
            let semantics: Vec<_> = fields
                .iter()
                .filter(|field| field.field == DOOR_CURSOR)
                .cloned()
                .collect();
            if semantics.len() != 1 || fields.len() != 3 {
                return Err(data_failure(Some(cut), "ordinary fight picture comparison"));
            }
            let observed = ExpectedCut {
                semantic_count: 1,
                output_count: 2,
                semantics,
                fields,
                replies: 0,
            };
            runner::record_cut(result, &mut digest, &observed, &frozen, cut)?;
        }
        controller = controller.read(current.pad.0, super::polled(&current));
        previous = current;
    }
    if continuation.is_none() {
        return Err(data_failure(
            Some(case.end.cut_inclusive),
            "ordinary fight picture checkpoint",
        ));
    }
    result.counts.restorations = result.counts.restorations.saturating_add(1);
    result.observation_sha256 = crate::lock::Sha256::of(&digest).to_string();
    Ok(())
}

#[test]
#[ignore = "pinned record cache and player ROM"]
fn ordinary_fight_image_reaches_both_native_door_cursors() {
    use std::io::Write as _;

    let index = crate::replay::index::Index::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../scenarios/replay/index.toml")
            .as_path(),
    )
    .unwrap();
    let case = index
        .cases
        .iter()
        .find(|case| case.id.0 == "ridley/ordinary-picture")
        .unwrap();
    let result = crate::replay::run(case, &super::tests::sources());
    assert_eq!(result.outcome, Outcome::Pass, "{result:#?}");
    assert_eq!(result.counts.cuts, 2);
    assert_eq!(result.counts.restorations, 1);
    assert_eq!(result.counts.null_cuts, 0);
    writeln!(
        std::io::stdout().lock(),
        "{} {:?} {}",
        result.id.0,
        result.counts,
        result.observation_sha256,
    )
    .unwrap();
}
