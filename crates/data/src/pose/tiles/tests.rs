use super::*;
use crate::rom::Rom;

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
fn a_definition_is_a_long_source_and_two_sizes() {
    let definition = TileDefinition::decode([0x00, 0xE1, 0x9C, 0xC0, 0x00, 0x80, 0x00]);
    assert_eq!(
        definition,
        TileDefinition {
            source: GraphicsRef {
                bank: 0x9C,
                offset: 0xE100
            },
            first: 0xC0,
            second: 0x80,
        }
    );
    assert_eq!(
        definition.second_source(),
        GraphicsRef {
            bank: 0x9C,
            offset: 0xE1C0
        }
    );
}

#[test]
fn graphics_resolve_within_a_loaded_bank() {
    let graphics = Graphics::new(vec![(0x9E, vec![1, 2, 3, 4])]);
    let at = |offset| GraphicsRef { bank: 0x9E, offset };
    assert_eq!(graphics.bytes(at(0x8001), 2), Some(&[2, 3][..]));
    assert_eq!(graphics.bytes(at(0x8003), 2), None);
    assert_eq!(graphics.bytes(at(0x7FFF), 1), None);
    assert_eq!(
        graphics.bytes(
            GraphicsRef {
                bank: 0x9D,
                offset: 0x8000
            },
            1
        ),
        None
    );
}

#[test]
fn the_parts_constructor_keeps_definitions_seven_bytes_apart() {
    let tiles = SamusTiles::new(
        vec![vec![FrameTiles {
            top: TileDefinitionId(1),
            bottom: None,
        }]],
        vec![TileDefinition::default(); 2],
    );
    assert_eq!(
        tiles
            .frame(PoseId(0), AnimationFrame(0))
            .map(|frame| frame.top),
        Some(TileDefinitionId(1))
    );
    assert_eq!(tiles.frame(PoseId(0), AnimationFrame(1)), None);
    assert_eq!(tiles.at_pointer(7), Some(TileDefinitionId(1)));
    assert_eq!(tiles.pointer(TileDefinitionId(1)), Some(7));
    assert_eq!(tiles.at_pointer(8), None);
}

#[test]
fn the_tables_decode_as_the_bank_logs_say() {
    let Some(cartridge) = rom() else {
        return;
    };
    let delays = AnimationDelays::load(cartridge.lorom()).unwrap();
    let tiles = SamusTiles::load(cartridge.lorom(), &delays).unwrap();
    let definition = |pose, frame| {
        let frame = tiles.frame(PoseId(pose), AnimationFrame(frame)).unwrap();
        let top = tiles.definition(frame.top).copied().unwrap();
        let bottom = frame
            .bottom
            .and_then(|id| tiles.definition(id))
            .copied()
            .unwrap();
        (
            tiles.pointer(frame.top),
            top,
            frame.bottom.and_then(|id| tiles.pointer(id)),
            bottom,
        )
    };
    let tile = |bank, offset, first, second| TileDefinition {
        source: GraphicsRef { bank, offset },
        first,
        second,
    };
    // Pose 1, facing right, frame 0: top list 7 entry 12, bottom list 0 entry 6.
    assert_eq!(
        definition(1, 0),
        (
            Some(0xD0B0),
            tile(0x9C, 0xE100, 0xC0, 0x80),
            Some(0xD1C8),
            tile(0x9D, 0x8820, 0xC0, 0xC0)
        )
    );
    // Pose $1D, morph ball: the first entries of top list 10 and bottom list 0.
    assert_eq!(
        definition(0x1D, 0),
        (
            Some(0xD613),
            tile(0x9C, 0x8000, 0x60, 0x40),
            Some(0xD19E),
            tile(0x9D, 0x8000, 0x80, 0x80)
        )
    );
    assert!(tiles.definition_count() > 300);
    let graphics = Graphics::load(cartridge.lorom(), GRAPHICS_BANKS).unwrap();
    for pose in 0..u16::try_from(POSE_COUNT).unwrap() {
        for frame in 0.. {
            let Some(frame) = tiles.frame(PoseId(pose), AnimationFrame(frame)) else {
                break;
            };
            for id in std::iter::once(frame.top).chain(frame.bottom) {
                let definition = tiles.definition(id).unwrap();
                assert!(
                    graphics
                        .bytes(definition.source, usize::from(definition.first))
                        .is_some()
                );
                assert!(
                    graphics
                        .bytes(definition.second_source(), usize::from(definition.second))
                        .is_some()
                );
            }
        }
    }
}
