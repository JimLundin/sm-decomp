//! Native qualification of the shaft overlay graphics write.

#[cfg(test)]
use super::{open_rows, read_lock, tests::sources};
#[cfg(test)]
use data::rom::Rom;
use game::production::played::room_video::RoomVideo;
#[cfg(test)]
use game::production::played::room_video::ShaftOverlay;
#[cfg(test)]
use game::state::video::Video;

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

pub(super) fn compare_background(
    video: &RoomVideo,
    record: &oracle_record::FrameRecord,
    expected: &mut Vec<super::Observation>,
    actual: &mut Vec<super::Observation>,
) -> Result<(), super::Outcome> {
    if let Some(words) = &video.standard_bg3 {
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
    assert!(run.state.room_video.standard_bg3.is_none());
    let prior = rows.record(1).unwrap();
    let controller = seed_controller(&prior).unwrap();
    assert_eq!(
        run.through(&mut rows, prior, controller, 2..=2).unwrap(),
        (1, Stop::End)
    );
    let written = run.state.room_video.standard_bg3.clone().unwrap();
    let sampled = rows.record(2).unwrap();
    let matches = written == sampled.vram[0x4000..0x4800];
    assert!(matches, "standard BG3 native words");
    let controller = seed_controller(&sampled).unwrap();
    assert_eq!(
        run.through(&mut rows, sampled, controller, 3..=222)
            .unwrap(),
        (220, Stop::End)
    );
    let matches = run.state.room_video.standard_bg3.as_ref() == Some(&written);
    assert!(matches, "standard BG3 retained");
}
