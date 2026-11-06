use super::*;
use crate::rom::Rom;
use crate::spritemap::{SpriteAttributes, SpriteSize, SpritemapEntry};

/// The ROM, from `SM_ROM` or the checkout's `rom/sm.smc`; `None` where it isn't present.
fn rom() -> Option<Rom> {
    let path = std::env::var_os("SM_ROM").map_or_else(
        || std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../rom/sm.smc"),
        std::path::PathBuf::from,
    );
    let image = std::fs::read(path).ok()?;
    Some(Rom::new(image).unwrap())
}

#[test]
fn a_half_adds_the_frame_to_its_poses_first_index() {
    let maps = SamusSpritemaps::new(
        vec![SpritemapIndex(1)],
        vec![SpritemapIndex(0)],
        vec![None, Some(Spritemap::default()), None],
        PositionOffsets::default(),
    );
    let top = maps.index(Half::Top, PoseId(0), AnimationFrame(1));
    assert_eq!(top, Some(SpritemapIndex(2)));
    assert_eq!(top.and_then(|index| maps.spritemap(index)), None);
    assert_eq!(
        maps.spritemap(SpritemapIndex(1)),
        Some(&Spritemap::default())
    );
    assert_eq!(maps.spritemap(SpritemapIndex(9)), None);
    assert_eq!(maps.index(Half::Bottom, PoseId(1), AnimationFrame(0)), None);
}

#[test]
fn the_spritemaps_decode_as_the_bank_logs_say() {
    let Some(cartridge) = rom() else {
        return;
    };
    let maps = SamusSpritemaps::load(cartridge.lorom()).unwrap();
    // Pose 1, facing right: top half from index $19A, bottom half from $4AA.
    let top = maps.index(Half::Top, PoseId(1), AnimationFrame(0)).unwrap();
    let bottom = maps
        .index(Half::Bottom, PoseId(1), AnimationFrame(0))
        .unwrap();
    assert_eq!(
        (top, bottom),
        (SpritemapIndex(0x19A), SpritemapIndex(0x4AA))
    );
    // $92:A072, four sprites; its first is large at (-7, -8), tile 0, palette 4, priority 2.
    let map = maps.spritemap(top).unwrap();
    assert_eq!(map.entries.len(), 4);
    assert_eq!(
        map.entries.first(),
        Some(&SpritemapEntry {
            x: -7,
            y: 0xF8,
            size: SpriteSize::Large,
            attributes: SpriteAttributes::from_word(0x2800),
        })
    );
    assert_eq!(maps.spritemap(bottom).map(|map| map.entries.len()), Some(3));
    // No Samus spritemap has more than 26 sprites.
    assert!(
        (0..u16::try_from(SPRITEMAP_COUNT).unwrap())
            .filter_map(|index| maps.spritemap(SpritemapIndex(index)))
            .all(|map| map.entries.len() <= 26)
    );
    assert_eq!(maps.offsets.landing[..4], [3, 6, 0, 0]);
    assert_eq!(maps.offsets.transition[..4], [-8, 0, -8, 0]);
    assert_eq!(maps.offsets.drained[..3], [7, 5, -8]);
}
