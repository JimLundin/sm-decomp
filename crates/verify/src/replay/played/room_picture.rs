//! Native qualification of the first room's direct OAM write and NMI palette publication.

use super::{Run, appearance, connected, movement, open_rows, polled, read_lock, seed_controller};
use crate::replay::landing_site;

#[test]
#[ignore = "pinned record cache and player ROM"]
fn arrival_writes_the_first_oam_word_and_publishes_the_room_palette() {
    let sources = super::tests::sources();
    let lock = read_lock(&sources).unwrap();
    let row = "any-ceres-arrival";
    let mut rows = open_rows(&sources, &lock, row).unwrap();
    let seed = rows.record(0).unwrap();
    let content = connected::Content {
        movement: movement::Content::load(&sources.rom).unwrap(),
        appearance: appearance::Content::load(&sources.rom).unwrap(),
    };
    let mut run = Run::admit(&seed, &content, &sources.rom, row).unwrap();
    let mut controller = seed_controller(&seed).unwrap();
    // This scope admits the completed flight's OAM, which the room receives.
    let carried = landing_site::oam(&seed).unwrap();
    run.state.room_video.video.batch.oam = carried;
    run.state.room_video.video.memory.oam = carried;
    let mut previous = seed;
    for cut in 1..=222 {
        let current = rows.record(cut).unwrap();
        run.step(controller, &previous, &current, cut, (row, cut))
            .unwrap();
        let video = &run.state.room_video.video;
        if cut <= 192 {
            let native = landing_site::oam(&current).unwrap();
            let physical_matches = video.memory.oam == native;
            let prepared_retained = video.batch.oam == carried;
            assert!(physical_matches, "physical OAM at cut {cut}");
            assert!(prepared_retained, "prepared OAM at cut {cut}");
        }
        if cut >= 192 {
            let physical_matches = video.memory.cgram.map(|colour| colour.0) == *current.cgram;
            let prepared_matches = video
                .batch
                .cgram
                .iter()
                .zip(current.wram[0xc000..0xc200].as_chunks::<2>().0)
                .all(|(actual, expected)| actual.0 == u16::from_le_bytes(*expected));
            assert!(physical_matches, "physical CGRAM at cut {cut}");
            assert!(prepared_matches, "prepared CGRAM at cut {cut}");
        }
        controller = controller.read(current.pad.0, polled(&current));
        previous = current;
    }
}
