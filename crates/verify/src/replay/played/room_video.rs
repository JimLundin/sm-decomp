//! Native qualification of the shaft overlay graphics write.

use super::{open_rows, read_lock, tests::sources};
use data::rom::Rom;
use game::production::played::room_video::ShaftOverlay;
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

    assert!(prior.vram[strip.clone()] != placed.vram[strip.clone()]);
    assert_eq!(ShaftOverlay::publish(&mut video, &art), Some(()));
    assert!(video.memory.vram[strip.clone()] == placed.vram[strip.clone()]);
    assert!(video.memory.vram[..strip.start] == before.memory.vram[..strip.start]);
    assert!(video.memory.vram[strip.end..] == before.memory.vram[strip.end..]);
    assert!(video.batch == before.batch);
    assert!(video.committed == before.committed);
    assert!(video.memory.cgram == before.memory.cgram && video.memory.oam == before.memory.oam);
}
