use super::*;
use crate::level::{BombableBts, PlmChoice};
use crate::rom::Rom;

/// A file offset in a `LoROM` image.
fn file(bank: u8, offset: u16) -> usize {
    (usize::from(bank & 0x7f) << 15) | usize::from(offset & 0x7fff)
}

fn put(image: &mut [u8], offset: u16, bytes: &[u8]) {
    image[file(PLM_BANK, offset)..][..bytes.len()].copy_from_slice(bytes);
}

fn words(values: &[u16]) -> Vec<u8> {
    values.iter().flat_map(|word| word.to_le_bytes()).collect()
}

/// The ROM, from `SM_ROM` or the checkout's `rom/sm.smc`; `None` where it isn't present.
fn rom() -> Option<Rom> {
    let path = std::env::var_os("SM_ROM").map_or_else(
        || std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../rom/sm.smc"),
        std::path::PathBuf::from,
    );
    Rom::new(std::fs::read(path).ok()?).ok()
}

/// An invented list: a sound, a go-to into a body that shows a row of two cells and, past an
/// offset header, a cell below, then grows back and ends.
fn invented() -> Vec<u8> {
    let mut image = vec![0; 0x30_0000];
    let mut list = words(&[op::SOUNDS[4].0]);
    list.push(0x2a);
    list.extend(words(&[op::GO_TO, 0x9100]));
    put(&mut image, 0x9000, &list);
    put(
        &mut image,
        0x9100,
        &words(&[3, 0x9200, op::REGROW[0], op::END]),
    );
    // A row of two (air, picture 12h flipped across; itself, 13h), then one cell at (0, 1).
    put(
        &mut image,
        0x9200,
        &words(&[0x0002, 0x0412, 0xb013, 0x0100, 0x0001, 0x5014, DRAW_END]),
    );
    image
}

#[test]
fn a_list_reads_as_its_steps_its_go_to_followed() {
    let image = invented();
    let (steps, positions) = script(Lorom::new(&image), 0x9000).unwrap();
    let picture = |index| Picture::new(index).unwrap();
    let pattern = vec![
        PatternCell {
            offset: Pair { x: 0, y: 0 },
            becomes: Becomes::Air,
            look: Look {
                picture: picture(0x12),
                flip: Flip::Across,
            },
        },
        PatternCell {
            offset: Pair { x: 1, y: 0 },
            becomes: Becomes::Itself,
            look: Look {
                picture: picture(0x13),
                flip: Flip::None,
            },
        },
        PatternCell {
            offset: Pair { x: 0, y: 1 },
            becomes: Becomes::StandInAcross,
            look: Look {
                picture: picture(0x14),
                flip: Flip::None,
            },
        },
    ];
    assert_eq!(
        steps,
        [
            Step::Sound(Sound {
                library: 2,
                number: 0x2a,
                queue_limit: 1,
            }),
            Step::Show { pattern, frames: 3 },
            Step::Regrow,
            Step::End,
        ]
    );
    // The show is reached by the go-to as well as at its own place.
    assert_eq!(
        positions,
        [
            vec![0x9000],
            vec![0x9003, 0x9100],
            vec![0x9104],
            vec![0x9106]
        ]
    );
}

#[test]
fn a_show_of_no_frames_and_a_list_that_loops_are_refused() {
    let mut image = invented();
    put(&mut image, 0x9100, &words(&[0, 0x9200]));
    assert_eq!(
        script(Lorom::new(&image), 0x9100),
        Err(Refused::ShowFrames(RomAddress::new(PLM_BANK, 0x9100)))
    );
    put(&mut image, 0x9100, &words(&[op::GO_TO, 0x9000]));
    assert_eq!(
        script(Lorom::new(&image), 0x9000),
        Err(Refused::Loop(RomAddress::new(PLM_BANK, 0x9000)))
    );
}

#[test]
fn a_cell_past_its_tables_is_refused_by_its_kind() {
    assert!(past_table(0x3, 0x50));
    assert!(!past_table(0x3, 0x46));
    assert!(past_table(0xb, 0x1a));
    assert!(!past_table(0x3, 0x1a));
    assert!(past_table(0xc, 0x88));
    assert!(!past_table(0x4, 0x88));
    assert!(past_table(0xf, 0x10));
    assert!(!past_table(0x8, 0x50));
}

#[test]
fn the_games_reactions_import_with_their_marks() {
    let Some(rom) = rom() else {
        return;
    };
    let reactions = Reactions::load(rom.lorom()).unwrap();
    assert_eq!(reactions.all().len(), ENTRIES.len());
    for (reaction, index) in reactions.all().iter().zip(0_u16..) {
        let id = ReactionId(index);
        assert_eq!(reactions.named(reaction.name), Some(id));
        assert!(matches!(reaction.broken.last(), Some(Step::End)));
        let touched = reaction.family == Family::Bomb;
        assert_eq!(!reaction.touched.is_empty(), touched, "{}", reaction.name);
        for script in [Script::Broken, Script::Touched, Script::Revealed] {
            let steps = reaction.script(script);
            for step in 0..steps.len() {
                let position = reactions.position_of(id, script, step).unwrap();
                assert_eq!(reactions.step_at(id, script, position), Some(step));
            }
        }
        let plm = reactions.plm_of(id, Script::Broken).unwrap();
        assert_eq!(reactions.runs(plm), Some(Runs::Script(id, Script::Broken)));
    }
    // A special block's mark is its touch's reaction; a bombable one with its high bit set
    // holds nothing.
    let crumble = reactions.named("crumble-1x1-regrows").unwrap();
    assert_eq!(
        reactions.mark(BlockKind::Special(PlmChoice::Common(0)), Area::Crateria),
        Ok(Some(Mark::Reaction(crumble)))
    );
    assert_eq!(
        reactions.mark(BlockKind::Bombable(BombableBts::Inert(0)), Area::Crateria),
        Ok(Some(Mark::Inert))
    );
    assert_eq!(
        reactions.mark(BlockKind::Special(PlmChoice::Common(0x46)), Area::Crateria),
        Ok(Some(Mark::Scroll))
    );
}
