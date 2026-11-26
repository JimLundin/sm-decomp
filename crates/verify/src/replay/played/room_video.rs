//! Native qualification of the shaft overlay graphics write.

#[cfg(test)]
use super::{open_rows, read_lock, tests::sources};
#[cfg(test)]
use data::rom::Rom;
use game::production::played::room_video::RoomVideo;
#[cfg(test)]
use game::production::played::room_video::ShaftOverlay;
use game::state::video::Video;

type Fields<'a> = (
    &'a mut Vec<super::Observation>,
    &'a mut Vec<super::Observation>,
);

#[test]
#[ignore = "pinned record cache and player ROM"]
fn shaft_overlay_placement_publishes_its_complete_strip() {
    let sources = sources();
    let rom = Rom::new(std::fs::read(&sources.rom).unwrap()).unwrap();
    let art = import::shaft::import(rom.lorom()).unwrap();
    let lock = read_lock(&sources).unwrap();
    let mut rows = open_rows(&sources, &lock, "chain-any-0010-DF8D").unwrap();
    let prior = rows.record(224).unwrap();
    let placed = rows.record(225).unwrap();
    let mut video = Video::default();
    video.memory.vram = prior.vram.to_vec();
    let before = video.clone();
    let strip = 0x7000..0x7200;

    let matches = prior.vram[strip.clone()] != placed.vram[strip.clone()];
    assert!(matches, "native strip changed");
    assert_eq!(ShaftOverlay::publish(&mut video, &art), Some(()));
    let matches = video.memory.vram[strip.clone()] == placed.vram[strip.clone()];
    assert!(matches, "published strip");
    let matches = video.memory.vram[..strip.start] == before.memory.vram[..strip.start];
    assert!(matches, "prefix retained");
    let matches = video.memory.vram[strip.end..] == before.memory.vram[strip.end..];
    assert!(matches, "suffix retained");
    let matches = video.batch == before.batch;
    assert!(matches, "pending batch retained");
    let matches = video.committed == before.committed;
    assert!(matches, "registers retained");
    assert!(video.memory.cgram == before.memory.cgram && video.memory.oam == before.memory.oam);
}

pub(super) fn compare(
    room: &RoomVideo,
    overlay_placed: bool,
    frame: &super::Frame<'_>,
    expected: &mut Vec<super::Observation>,
    actual: &mut Vec<super::Observation>,
) -> Result<(), super::Outcome> {
    compare_live_overlay(
        &room.video,
        overlay_placed,
        frame.current,
        frame.place,
        frame.cut,
        (expected, actual),
    )?;
    compare_shaft_picture(
        room,
        frame.current,
        frame.place,
        frame.cut,
        (expected, actual),
    )?;
    compare_background(room, frame.current, expected, actual)
}

fn compare_live_overlay(
    video: &Video,
    placed: bool,
    record: &oracle_record::FrameRecord,
    (row, local): (&str, u64),
    cut: u64,
    (expected_fields, actual_fields): Fields<'_>,
) -> Result<(), super::Outcome> {
    if row == "chain-any-0010-DF8D" && local == 225 && !placed {
        return Err(super::Outcome::Difference {
            cut,
            field: "room-video.shaft-overlay-placement".into(),
        });
    }
    if placed {
        let strip = 0x7000..0x7200;
        let expected = record
            .vram
            .get(strip.clone())
            .ok_or_else(|| super::data_failure(Some(cut), "shaft overlay VRAM"))?;
        let field = "room-video.shaft-overlay-strip";
        let actual = video
            .memory
            .vram
            .get(strip)
            .ok_or_else(|| super::Outcome::Difference {
                cut,
                field: field.into(),
            })?;
        let bytes = |words: &[u16]| words.iter().copied().flat_map(u16::to_le_bytes).collect();
        record_pair(
            cut,
            field,
            bytes(expected),
            bytes(actual),
            (expected_fields, actual_fields),
        )?;
    }
    Ok(())
}

fn compare_background(
    video: &RoomVideo,
    record: &oracle_record::FrameRecord,
    expected: &mut Vec<super::Observation>,
    actual: &mut Vec<super::Observation>,
) -> Result<(), super::Outcome> {
    if let Some(words) = video.standard_bg3() {
        let native = record
            .vram
            .get(0x4000..0x4800)
            .ok_or_else(|| super::data_failure(None, "standard BG3 tile area"))?;
        let field = |cells: &[u16]| super::Observation {
            field: "room-video.standard-bg3".into(),
            bytes: cells.iter().copied().flat_map(u16::to_le_bytes).collect(),
        };
        expected.push(field(native));
        actual.push(field(words));
    }
    Ok(())
}

/// The regular arrival reaches the loading write and carries the words through its HUD publish.
#[test]
#[ignore = "pinned record cache and player ROM"]
fn arrival_produces_standard_background_before_hud() {
    use super::{Run, Stop, appearance, connected, movement, seed_controller};

    let sources = sources();
    let lock = read_lock(&sources).unwrap();
    let row = "any-ceres-arrival";
    let mut rows = open_rows(&sources, &lock, row).unwrap();
    let seed = rows.record(0).unwrap();
    let content = connected::Content {
        movement: movement::Content::load(&sources.rom).unwrap(),
        appearance: appearance::Content::load(&sources.rom).unwrap(),
    };
    let mut run = Run::admit(&seed, &content, &sources.rom, row).unwrap();
    let controller = seed_controller(&seed).unwrap();
    assert_eq!(
        run.through(&mut rows, seed, controller, 1..=1).unwrap(),
        (1, Stop::End)
    );
    assert!(run.state.room_video.standard_bg3().is_none());
    let prior = rows.record(1).unwrap();
    let controller = seed_controller(&prior).unwrap();
    assert_eq!(
        run.through(&mut rows, prior, controller, 2..=2).unwrap(),
        (1, Stop::End)
    );
    let written = run.state.room_video.standard_bg3().unwrap().to_vec();
    let sampled = rows.record(2).unwrap();
    let matches = written == sampled.vram[0x4000..0x4800];
    assert!(matches, "standard BG3 native words");
    let controller = seed_controller(&sampled).unwrap();
    assert_eq!(
        run.through(&mut rows, sampled, controller, 3..=222)
            .unwrap(),
        (220, Stop::End)
    );
    let matches = run.state.room_video.standard_bg3() == Some(written.as_slice());
    assert!(matches, "standard BG3 retained");
}

fn compare_shaft_picture(
    room: &RoomVideo,
    record: &oracle_record::FrameRecord,
    (row, local): (&str, u64),
    cut: u64,
    (expected, actual): Fields<'_>,
) -> Result<(), super::Outcome> {
    let hall = row == "chain-any-0010-DF8D";
    let shaft = row == "chain-any-0011-DF45";
    if hall && (239..=240).contains(&local) {
        let (oam, recorded, field) = if local == 239 {
            let shadow = record
                .wram
                .get(super::super::ridley_warning_native::SHADOW_OAM)
                .ok_or_else(|| super::data_failure(Some(cut), "shaft shadow OAM"))?;
            (
                &room.video.batch.oam,
                shadow,
                "room-video.shaft-overlay-prepared",
            )
        } else {
            (
                &room.video.memory.oam,
                record.oam.as_ref(),
                "room-video.shaft-overlay-published",
            )
        };
        // The first actor work starts with an empty draw cursor. Every later
        // figure can follow other owners’ draws.
        let native_low = recorded
            .get(..48)
            .ok_or_else(|| super::data_failure(Some(cut), "shaft OAM low"))?;
        let native_high = recorded
            .get(512..515)
            .ok_or_else(|| super::data_failure(Some(cut), "shaft OAM high"))?;
        let native = native_low.iter().chain(native_high).copied().collect();
        let played = oam
            .low
            .iter()
            .take(12)
            .flat_map(|entry| [entry.x, entry.y, entry.tile, entry.attributes])
            .chain(oam.high.iter().take(3).copied())
            .collect();
        record_pair(cut, field, native, played, (expected, actual))?;
        record_pair(
            cut,
            "room-video.shaft-platform-pending",
            vec![u8::from(local == 239)],
            vec![u8::from(room.pending_platform().is_some())],
            (expected, actual),
        )?;
    }
    if hall && (240..=278).contains(&local) || shaft && local <= 163 {
        let span = 0x060e..0x0612;
        let native = record
            .vram
            .get(span.clone())
            .ok_or_else(|| super::data_failure(Some(cut), "shaft platform VRAM"))?;
        let played = room
            .video
            .memory
            .vram
            .get(span)
            .ok_or_else(|| super::data_failure(Some(cut), "played platform VRAM"))?;
        let low = |words: &[u16]| words.iter().map(|word| word.to_le_bytes()[0]).collect();
        record_pair(
            cut,
            "room-video.shaft-platform-low",
            low(native),
            low(played),
            (expected, actual),
        )?;
    }
    Ok(())
}

fn record_pair(
    cut: u64,
    field: &str,
    native: Vec<u8>,
    played: Vec<u8>,
    (expected, actual): Fields<'_>,
) -> Result<(), super::Outcome> {
    if native != played {
        return Err(super::Outcome::Difference {
            cut,
            field: field.into(),
        });
    }
    expected.push(super::Observation {
        field: field.into(),
        bytes: native,
    });
    actual.push(super::Observation {
        field: field.into(),
        bytes: played,
    });
    Ok(())
}

#[test]
#[ignore = "pinned record cache and player ROM"]
fn return_hall_prepares_then_publishes_the_shaft_picture() {
    use super::{Run, Stop, appearance, connected, movement, seed_controller};

    let sources = sources();
    let lock = read_lock(&sources).unwrap();
    let row = "chain-any-0010-DF8D";
    let mut rows = open_rows(&sources, &lock, row).unwrap();
    let content = connected::Content {
        movement: movement::Content::load(&sources.rom).unwrap(),
        appearance: appearance::Content::load(&sources.rom).unwrap(),
    };
    let seed = rows.record(0).unwrap();
    let mut run = Run::admit(&seed, &content, &sources.rom, row).unwrap();
    let controller = seed_controller(&seed).unwrap();
    assert_eq!(
        run.through(&mut rows, seed, controller, 1..=238).unwrap(),
        (238, Stop::End)
    );
    let prior = rows.record(238).unwrap();
    let controller = seed_controller(&prior).unwrap();
    let before = run.state.room_video.video.memory.clone();
    assert_eq!(
        run.through(&mut rows, prior, controller, 239..=239)
            .unwrap(),
        (1, Stop::End)
    );
    let held = run.state.room_video.video.memory == before;
    assert!(held, "the interrupted cut holds physical video");

    let mut restored = run.clone();
    let interrupted = rows.record(239).unwrap();
    let controller = seed_controller(&interrupted).unwrap();
    assert_eq!(
        run.through(&mut rows, interrupted.clone(), controller, 240..=240)
            .unwrap(),
        (1, Stop::End)
    );
    assert_eq!(
        restored
            .through(&mut rows, interrupted, controller, 240..=240)
            .unwrap(),
        (1, Stop::End)
    );
    let same = run.state.room_video == restored.state.room_video;
    assert!(same, "the restored pending picture publishes identically");

    let completed = rows.record(240).unwrap();
    let controller = seed_controller(&completed).unwrap();
    assert_eq!(
        run.through(&mut rows, completed, controller, 241..=440)
            .unwrap(),
        (200, Stop::End)
    );
}
