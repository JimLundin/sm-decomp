use super::*;
use crate::level::ScrollArea;
use crate::rom::Rom;

/// The file offset of `$8F:offset`.
fn room_bank(offset: u16) -> usize {
    usize::from(offset & 0x7fff) | 0x78000
}

fn put(image: &mut [u8], at: usize, bytes: &[u8]) {
    image[at..][..bytes.len()].copy_from_slice(bytes);
}

/// A state header whose level data is at `$8F:level`, its other pointers counting up.
fn state_bytes(level: u16, tileset: u8) -> Vec<u8> {
    let [low, high] = level.to_le_bytes();
    let mut bytes = vec![low, high, 0x8f, tileset, 0x06, 0x07];
    for pointer in [0x8290_u16, 0x93ac, 0x8b9e] {
        bytes.extend(pointer.to_le_bytes());
    }
    bytes.extend([0xc1, 0xc1]);
    for pointer in [0x9eed_u16, 0x0000, 0x0000, 0x867e, 0xbaf4, 0x91d5] {
        bytes.extend(pointer.to_le_bytes());
    }
    bytes
}

/// A one-bank image with a two-state room at `$8F:8000`: event 0 selects `$8F:8100`, else
/// the default state follows the list.
fn synthetic() -> Vec<u8> {
    let mut image = vec![0; 0x80000];
    let header = [
        0x13, 0x01, 0x2b, 0x02, 0x08, 0x03, 0x70, 0xa0, 0x00, 0x91, 0x9e,
    ];
    put(&mut image, room_bank(0x8000), &header);
    // Event 0 -> $8100, then the default condition.
    put(
        &mut image,
        room_bank(0x800b),
        &[0x12, 0xe6, 0x00, 0x00, 0x81, 0xe6, 0xe5],
    );
    put(&mut image, room_bank(0x8012), &state_bytes(0x9000, 6));
    put(&mut image, room_bank(0x8100), &state_bytes(0x9010, 7));
    // Level data: a layer 1 size word of 4, then 6 fill bytes; and a one-block level of
    // five direct bytes.
    put(
        &mut image,
        room_bank(0x9000),
        &[0x01, 0x04, 0x00, 0x25, 0x0a, 0xff],
    );
    put(
        &mut image,
        room_bank(0x9010),
        &[0x04, 0x02, 0x00, 0x11, 0x22, 0x33, 0xff],
    );
    image
}

const SYNTHETIC: RoomKey = RoomKey::from_room_pointer(0x8000);

#[test]
fn a_header_and_its_states_decode() {
    let image = synthetic();
    let room = decode(Lorom::new(&image), SYNTHETIC).unwrap();
    assert_eq!(room.index, 0x13);
    assert_eq!(room.area, Area::Brinstar);
    assert_eq!(room.map_position, MapPosition { x: 0x2b, y: 0x02 });
    assert_eq!(
        room.size,
        Screens {
            width: 8,
            height: 3
        }
    );
    assert_eq!(
        room.scrollers,
        Scrollers {
            up: 0x70,
            down: 0xa0
        }
    );
    assert_eq!(room.doors, RomAddress::new(0x8f, 0x9e91));
    assert_eq!(room.conditional.len(), 1);
    assert_eq!(
        room.conditional[0].condition,
        Condition::Event(Event::PlanetAwake)
    );
    assert_eq!(room.conditional[0].state.at, RomAddress::new(0x8f, 0x8100));
    assert_eq!(room.conditional[0].state.tileset, 7);
    assert_eq!(
        room.conditional[0].state.level.bytes,
        [0x02, 0x00, 0x11, 0x22, 0x33]
    );
    let spike_air = crate::level::Block {
        kind: crate::level::BlockKind::SpikeAir(crate::level::Bts(0x33)),
        tile: crate::level::Tile::from_word(0x0211),
    };
    assert_eq!(room.conditional[0].state.layers.layer1, [spike_air]);

    let default = &room.default;
    assert_eq!(default.at, RomAddress::new(0x8f, 0x8012));
    assert_eq!(default.tileset, 6);
    assert_eq!(default.music, Music { data: 6, track: 7 });
    assert_eq!(default.fx, RomAddress::new(0x83, 0x8290));
    assert_eq!(default.enemy_population, RomAddress::new(0xa1, 0x93ac));
    assert_eq!(default.enemy_set, RomAddress::new(0xb4, 0x8b9e));
    assert_eq!(default.layer2_scroll, [0xc1, 0xc1]);
    assert_eq!(default.scroll, RomAddress::new(0x8f, 0x9eed));
    assert_eq!(default.plm_population, RomAddress::new(0x8f, 0x867e));
    assert_eq!(default.library_background, RomAddress::new(0x8f, 0xbaf4));
    assert_eq!(default.setup_code, RomAddress::new(0x8f, 0x91d5));
    assert_eq!(default.level.at, RomAddress::new(0x8f, 0x9000));
    assert_eq!(default.level.layer1_bytes(), Some(4));
    assert_eq!(default.level.bytes.len(), 8);
    // Two blocks of 0x0A0A: air, tile 0x20A flipped top to bottom, BTS 0x0A.
    let air = crate::level::Block::new(0x0a0a, 0x0a);
    assert_eq!(
        air.kind,
        crate::level::BlockKind::Air(crate::level::Bts(0x0a))
    );
    assert!(air.tile.y_flip && !air.tile.x_flip);
    assert_eq!(default.layers.layer1, [air, air]);
    assert_eq!(default.layers.layer2, None);
    // The image holds zeros at the scroll table: every screen red.
    assert_eq!(default.scroll_areas, vec![ScrollArea::Red; 24]);
}

#[test]
fn a_state_index_counts_the_conditional_states_then_the_default() {
    let image = synthetic();
    let room = decode(Lorom::new(&image), SYNTHETIC).unwrap();
    assert_eq!(room.state(StateIndex(0)), Some(&room.conditional[0].state));
    assert_eq!(room.state(StateIndex(1)), Some(&room.default));
    assert_eq!(room.state(StateIndex(2)), None);
}

#[test]
fn unknown_areas_conditions_and_events_are_errors() {
    let mut image = synthetic();
    image[room_bank(0x8001)] = 9;
    assert_eq!(
        decode(Lorom::new(&image), SYNTHETIC),
        Err(RoomError::Area {
            room: SYNTHETIC,
            value: 9
        })
    );
    let mut image = synthetic();
    put(&mut image, room_bank(0x800b), &[0x34, 0x12]);
    assert_eq!(
        decode(Lorom::new(&image), SYNTHETIC),
        Err(RoomError::Condition {
            at: RomAddress::new(0x8f, 0x800b),
            code: 0x1234
        })
    );
    // Event 16h names nothing.
    let mut image = synthetic();
    image[room_bank(0x800d)] = 0x16;
    assert_eq!(
        decode(Lorom::new(&image), SYNTHETIC),
        Err(RoomError::Event {
            at: RomAddress::new(0x8f, 0x800b),
            number: 0x16
        })
    );
}

#[test]
fn the_room_map_gives_dense_ids_in_load_order() {
    let other = RoomKey::from_room_pointer(0x91f8);
    let map = RoomMap::new(vec![RoomKey::MORPH_BALL_ROOM, other]).unwrap();
    assert_eq!(map.id(RoomKey::MORPH_BALL_ROOM), Some(RoomId(0)));
    assert_eq!(map.id(other), Some(RoomId(1)));
    assert_eq!(map.key(RoomId(1)), Some(other));
    assert_eq!(map.id(RoomKey::from_room_pointer(0x92fd)), None);
    assert_eq!(RoomMap::new(vec![other, other]), None);
    assert_eq!(
        RoomMap::slice_zero().id(RoomKey::MORPH_BALL_ROOM),
        Some(RoomId(0))
    );
    assert_eq!(RoomKey::MORPH_BALL_ROOM.room_pointer(), Some(0x9e9f));
}

#[test]
fn loading_records_each_location() {
    let image = synthetic();
    let (rooms, addresses) = Rooms::load(Lorom::new(&image), &[SYNTHETIC]).unwrap();
    let id = rooms.map().id(SYNTHETIC).unwrap();
    assert_eq!(rooms.get(id).map(|room| room.key), Some(SYNTHETIC));
    assert_eq!(addresses.get(Location::RoomHeader(id)), Some(SYNTHETIC.0));
    assert_eq!(
        addresses.get(Location::State(id, StateIndex(1), StatePart::LevelData)),
        Some(RomAddress::new(0x8f, 0x9000))
    );
    assert_eq!(
        addresses.get(Location::State(id, StateIndex(0), StatePart::Header)),
        Some(RomAddress::new(0x8f, 0x8100))
    );
}

/// The ROM, from `SM_ROM` or the checkout's `rom/sm.smc`; `None` where it isn't present.
/// CI's Verify job has it; Quality runs only the synthetic tests above.
fn rom() -> Option<Rom> {
    let path = std::env::var_os("SM_ROM").map_or_else(
        || std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../rom/sm.smc"),
        std::path::PathBuf::from,
    );
    let image = std::fs::read(path).ok()?;
    Some(Rom::new(image).unwrap())
}

#[test]
fn the_morph_ball_room_decodes_as_the_bank_log_says() {
    let Some(cartridge) = rom() else {
        return;
    };
    let (loaded, addresses) = Rooms::load(cartridge.lorom(), &SLICE_ZERO_ROOMS).unwrap();
    let id = RoomMap::slice_zero().id(RoomKey::MORPH_BALL_ROOM).unwrap();
    let room = loaded.get(id).unwrap();
    assert_eq!(room.area, Area::Brinstar);
    assert_eq!(
        room.size,
        Screens {
            width: 8,
            height: 3
        }
    );
    assert_eq!(room.conditional.len(), 1);
    assert_eq!(
        room.conditional[0].condition,
        Condition::Event(Event::PlanetAwake)
    );
    assert_eq!(room.conditional[0].state.at, RomAddress::new(0x8f, 0x9ecb));

    // The default state, which slice 0's seed loads before the planet wakes.
    let index = StateIndex(1);
    let state = room.state(index).unwrap();
    assert_eq!(state.at, RomAddress::new(0x8f, 0x9eb1));
    assert_eq!(state.level.at, RomAddress::new(0xc5, 0xded5));
    assert_eq!(state.tileset, 6);
    assert_eq!(state.music, Music { data: 6, track: 7 });
    assert_eq!(state.fx, RomAddress::new(0x83, 0x8290));
    assert_eq!(state.enemy_population, RomAddress::new(0xa1, 0x93ac));
    assert_eq!(state.layer2_scroll, [0xc1, 0xc1]);
    assert_eq!(state.scroll, RomAddress::new(0x8f, 0x9eed));
    assert_eq!(state.plm_population, RomAddress::new(0x8f, 0x867e));
    assert_eq!(state.library_background, RomAddress::new(0x8f, 0xbaf4));
    assert_eq!(state.setup_code, RomAddress::new(0x8f, 0x91d5));

    // Layer 1 is two bytes a block, BTS one; this room's level data has no layer 2.
    let layer1 = room.size.blocks().wrapping_mul(2);
    assert_eq!(state.level.layer1_bytes().map(usize::from), Some(layer1));
    let bts = room.size.blocks();
    assert_eq!(
        state.level.bytes.len(),
        layer1.wrapping_add(bts).wrapping_add(2)
    );

    // 6,144 blocks, which go back out as the level data's own bytes.
    let blocks = &state.layers.layer1;
    assert_eq!(blocks.len(), room.size.blocks());
    assert_eq!(state.layers.layer2, None);
    let (words, bts) = state.level.bytes[2..].split_at(layer1);
    for ((block, word), bts) in blocks.iter().zip(words.as_chunks::<2>().0).zip(bts) {
        assert_eq!(&block.word().to_le_bytes(), word);
        assert_eq!(block.kind.bts(), *bts);
    }
    // The scroll table at $8F:9EED gives each of the 24 screens an area.
    assert_eq!(state.scroll_areas.len(), 24);
    let mut kinds: Vec<u8> = blocks.iter().map(|block| block.kind.nybble()).collect();
    kinds.sort_unstable();
    kinds.dedup();
    // Air, slopes, solid, doors, shot blocks, vertical extensions and bomb blocks.
    assert_eq!(kinds, [0x0, 0x1, 0x8, 0x9, 0xc, 0xd, 0xf]);
    let (red, blue, green) = (ScrollArea::Red, ScrollArea::Blue, ScrollArea::Green);
    let mut areas = vec![red; 16];
    areas[5] = green;
    areas.extend([blue; 8]);
    assert_eq!(state.scroll_areas, areas);

    assert_eq!(
        addresses.get(Location::State(id, index, StatePart::PlmPopulation)),
        Some(RomAddress::new(0x8f, 0x867e))
    );
}
