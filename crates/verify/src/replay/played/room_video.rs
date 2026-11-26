//! Native qualification of the shaft overlay graphics write.

#[cfg(test)]
use super::{open_rows, read_lock, tests::sources};
#[cfg(test)]
use data::rom::Rom;
use game::production::played::{
    arrival::Readiness,
    room_video::{EnemyStaging, RoomVideo},
};
use oracle_record::FrameRecord;

use super::{Observation, Outcome};
#[cfg(test)]
use game::production::played::room_video::ShaftOverlay;
#[cfg(test)]
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
    readiness: Readiness,
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
    compare_shaft_sheet(room, frame.current, frame.place, frame.cut)?;
    compare_shaft_raster(room, frame.current, expected, actual);
    compare_arrival(room, readiness, frame.previous, frame.current, expected, actual)
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

fn compare_arrival(
    video: &RoomVideo,
    readiness: Readiness,
    previous: &FrameRecord,
    current: &FrameRecord,
    expected: &mut Vec<Observation>,
    actual: &mut Vec<Observation>,
) -> Result<(), Outcome> {
    compare_idle_hdma(video, current, expected, actual);
    if readiness == Readiness::RoomEffects {
        let field = |bytes: &[u8]| Observation {
            field: "room-video.enemy-staging".into(),
            bytes: bytes.to_vec(),
        };
        let native = current
            .wram
            .get(EnemyStaging::WRAM)
            .ok_or_else(|| super::data_failure(None, "enemy tile staging buffer"))?;
        expected.push(field(native));
        actual.push(field(video.staged_enemy_tiles().unwrap_or_default()));
    }
    compare_background(video, current, expected, actual)?;
    if let Some(words) = super::arrival::enemy_tiles_written(previous, current)? {
        compare_enemy_tiles(video, current, words, expected, actual)?;
    }
    Ok(())
}

fn compare_idle_hdma(
    video: &RoomVideo,
    record: &oracle_record::FrameRecord,
    expected: &mut Vec<super::Observation>,
    actual: &mut Vec<super::Observation>,
) {
    let Some([channel5, channel6]) = video.idle_hdma() else {
        return;
    };
    for (field, physical, native) in [
        ("video.dma[5]", channel5, record.ppu.dma[5]),
        ("video.dma[6]", channel6, record.ppu.dma[6]),
    ] {
        expected.push(super::Observation {
            field: field.into(),
            bytes: native.to_vec(),
        });
        actual.push(super::Observation {
            field: field.into(),
            bytes: crate::replay::landing_site::io_bytes(physical).to_vec(),
        });
    }
}

pub(super) fn compare_background(
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

fn compare_enemy_tiles(
    video: &RoomVideo,
    current: &oracle_record::FrameRecord,
    words: usize,
    expected: &mut Vec<super::Observation>,
    actual: &mut Vec<super::Observation>,
) -> Result<(), super::Outcome> {
    let native = current
        .vram
        .get(0x6c00..0x6c00_usize.saturating_add(words))
        .ok_or_else(|| super::data_failure(None, "startup enemy tile area"))?;
    let field = |cells: &[u16]| super::Observation {
        field: "room-video.enemy-tiles".into(),
        bytes: cells.iter().copied().flat_map(u16::to_le_bytes).collect(),
    };
    expected.push(field(native));
    actual.push(field(video.uploaded_enemy_tiles().unwrap_or_default()));
    let dma = |channel: [u8; 11]| super::Observation {
        field: "room-video.enemy-tile-dma".into(),
        bytes: channel[..7].to_vec(),
    };
    expected.push(dma(current.ppu.dma[1]));
    actual.push(dma(crate::replay::landing_site::io_bytes(
        video.video.dma[1],
    )));
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
    let video = &run.state.room_video;
    let transfer = video.standard_objects().unwrap();
    assert_eq!((transfer.accepted, transfer.committed), (5_881, 5_880));
    assert!(transfer.pending.is_some() && transfer.selected.is_some());
    let dma = crate::replay::landing_site::io_bytes(video.video.dma[1]);
    assert_eq!(dma[..7], sampled.ppu.dma[1][..7]);
    let prefix = 0x6000..0x6000_usize.saturating_add(usize::from(transfer.committed / 2));
    let matches = video.video.memory.vram[prefix.clone()] == sampled.vram[prefix];
    assert!(matches, "interrupted standard object upload");

    let controller = seed_controller(&sampled).unwrap();
    assert_eq!(
        run.through(&mut rows, sampled, controller, 3..=85).unwrap(),
        (83, Stop::End)
    );
    let video = &run.state.room_video;
    let transfer = video.standard_objects().unwrap();
    assert_eq!((transfer.accepted, transfer.committed), (11_776, 11_776));
    assert!(transfer.pending.is_none() && transfer.selected.is_none());
    let prefix = 0x6000..0x6000_usize.saturating_add(usize::from(transfer.committed / 2));
    let retained = rows.record(85).unwrap();
    let matches = video.video.memory.vram[prefix.clone()] == retained.vram[prefix];
    assert!(
        matches,
        "ROM-backed object words carried to the next sprite writer"
    );
    let controller = seed_controller(&retained).unwrap();
    assert_eq!(
        run.through(&mut rows, retained, controller, 86..=222)
            .unwrap(),
        (137, Stop::End)
    );
    let matches = run.state.room_video.standard_bg3() == Some(written.as_slice());
    assert!(matches, "standard BG3 retained");
}

/// The first requested publication after the shaft tile transfer compares the entire sheet.
pub(super) fn compare_shaft_sheet(
    room: &RoomVideo,
    record: &oracle_record::FrameRecord,
    (row, local): (&str, u64),
    cut: u64,
) -> Result<(), super::Outcome> {
    let sheet = ..0x4000;
    if row == "chain-any-0010-DF8D" && (221..=224).contains(&local) {
        let expected = record
            .vram
            .get(sheet)
            .ok_or_else(|| super::data_failure(Some(cut), "shaft sheet VRAM"))?;
        if room.video.memory.vram.get(sheet) != Some(expected) {
            return Err(super::Outcome::Difference {
                cut,
                field: "room-video.shaft-sheet".into(),
            });
        }
    }
    if row == "chain-any-0010-DF8D" && (221..=278).contains(&local) {
        let shadow = record
            .wram
            .get(0x3000..0x3800)
            .ok_or_else(|| super::data_failure(Some(cut), "shaft BG2 shadow extent"))?;
        let expected = shadow
            .as_chunks::<2>()
            .0
            .iter()
            .copied()
            .map(u16::from_le_bytes);
        if room
            .shaft_bg2_shadow()
            .is_none_or(|words| words.iter().copied().ne(expected))
        {
            return Err(super::Outcome::Difference {
                cut,
                field: "room-video.shaft-bg2-shadow".into(),
            });
        }
    }
    Ok(())
}

/// The room runner selects and publishes the shaft tiles at the end of its fetch.
#[test]
#[ignore = "pinned record cache and player ROM"]
fn return_hall_publishes_the_complete_shaft_sheet() {
    use super::{Run, Stop, appearance, connected, movement, seed_controller};

    let sources = sources();
    let lock = read_lock(&sources).unwrap();
    let row = "chain-any-0010-DF8D";
    let mut rows = open_rows(&sources, &lock, row).unwrap();
    let seed = rows.record(0).unwrap();
    let content = connected::Content {
        movement: movement::Content::load(&sources.rom).unwrap(),
        appearance: appearance::Content::load(&sources.rom).unwrap(),
    };
    let mut run = Run::admit(&seed, &content, &sources.rom, row).unwrap();
    let controller = seed_controller(&seed).unwrap();
    assert_eq!(
        run.through(&mut rows, seed, controller, 1..=224).unwrap(),
        (224, Stop::End)
    );
}

pub(super) fn compare_shaft_raster(
    video: &RoomVideo,
    record: &oracle_record::FrameRecord,
    expected: &mut Vec<super::Observation>,
    actual: &mut Vec<super::Observation>,
) {
    if !video.shaft_raster_finished() {
        return;
    }
    for (index, (channel, native)) in video.video.dma.iter().zip(&record.ppu.dma).enumerate() {
        if matches!(index, 2 | 3 | 7) {
            let field = format!("video.dma[{index}]");
            expected.push(super::Observation {
                field: field.clone(),
                bytes: native.to_vec(),
            });
            actual.push(super::Observation {
                field,
                bytes: crate::replay::landing_site::io_bytes(*channel).to_vec(),
            });
        }
    }
}

/// The continuation consumes mapped WRAM captured before the DMA crossed the bank boundary.
#[test]
#[ignore = "pinned record cache and player ROM"]
fn arrival_object_upload_reads_through_the_source_wrap() {
    use game::production::played::arrival::Readiness;

    let sources = sources();
    let rom = Rom::new(std::fs::read(&sources.rom).unwrap()).unwrap();
    let graphics = import::landing_site::import(rom.lorom()).unwrap();
    let lock = read_lock(&sources).unwrap();
    let mut rows = open_rows(&sources, &lock, "any-ceres-arrival").unwrap();
    let prior = rows.record(1).unwrap();
    let interrupted = rows.record(2).unwrap();
    let finished = rows.record(3).unwrap();

    let Readiness::StandardBackground { accepted } =
        super::arrival::start(&prior, &interrupted).unwrap()
    else {
        panic!("the interrupted record has selected its next object read");
    };
    assert_eq!(
        super::arrival::start(&interrupted, &finished),
        Ok(Readiness::StandardObjectsFinished)
    );
    let mut video = RoomVideo::default();
    video.video.memory.vram.fill(0x1234);
    assert_eq!(video.standard_graphics(&graphics, accepted), Some(()));
    let wram = &interrupted.wram[..0x1200];
    assert_eq!(
        video.finish_standard_objects(&graphics, Some(wram)),
        Some(())
    );
    let transfer = video.standard_objects().unwrap();
    assert_eq!((transfer.accepted, transfer.committed), (0x4000, 0x4000));
    assert!(transfer.pending.is_none() && transfer.selected.is_none());
    let matches = video.video.memory.vram[0x6000..] == finished.vram[0x6000..];
    assert!(
        matches,
        "complete standard object upload with the mapped source"
    );
}

/// Room setup prepares the native source once, before the five tile transfers.
#[test]
#[ignore = "pinned record cache and player ROM"]
fn arrival_stages_enemies_and_initializes_idle_hdma() {
    use super::{Run, Stop, appearance, connected, movement, seed_controller};
    use game::production::played::room_video::EnemyStaging;

    let sources = sources();
    let lock = read_lock(&sources).unwrap();
    let mut boot = open_rows(&sources, &lock, "any-power-on").unwrap();
    assert_eq!(EnemyStaging::admit(&boot.record(0).unwrap().wram), None);
    assert_eq!(
        EnemyStaging::admit(&boot.record(257).unwrap().wram),
        Some(EnemyStaging::Cleared)
    );

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
        run.through(&mut rows, seed, controller, 1..=105).unwrap(),
        (105, Stop::End)
    );
    assert!(run.state.room_video.idle_hdma().is_none());
    assert!(run.state.room_video.staged_enemy_tiles().is_none());

    let prior = rows.record(105).unwrap();
    let controller = seed_controller(&prior).unwrap();
    assert_eq!(
        run.through(&mut rows, prior, controller, 106..=183)
            .unwrap(),
        (78, Stop::End)
    );
    assert!(run.state.room_video.idle_hdma().is_some());
    assert_eq!(
        run.state.room_video.uploaded_enemy_tiles().map(<[_]>::len),
        Some(0)
    );
    let prior = rows.record(183).unwrap();
    let controller = seed_controller(&prior).unwrap();
    assert_eq!(
        run.through(&mut rows, prior, controller, 184..=188)
            .unwrap(),
        (5, Stop::End)
    );
    assert_eq!(
        run.state.room_video.uploaded_enemy_tiles().map(<[_]>::len),
        Some(0x1400)
    );
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
