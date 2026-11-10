use super::*;

#[test]
fn every_block_word_and_bts_goes_back_out_exact() {
    for word in (0..=u16::MAX).step_by(7) {
        for bts in 0..=u8::MAX {
            let block = Block::new(word, bts);
            assert_eq!(block.word(), word, "{word:04x}");
            assert_eq!(block.kind.bts(), bts, "{word:04x} {bts:02x}");
        }
    }
    for nybble in 0..16_u8 {
        assert_eq!(BlockKind::new(nybble, 0).nybble(), nybble);
    }
}

#[test]
fn each_bts_is_typed_for_its_block_type() {
    assert_eq!(
        BlockKind::new(0x1, 0xc5),
        BlockKind::Slope(Slope {
            shape: 5,
            bit5: false,
            x_flip: true,
            y_flip: true,
        })
    );
    assert_eq!(
        BlockKind::new(0x5, 0xff),
        BlockKind::HorizontalExtension(Offset(-1))
    );
    assert_eq!(
        BlockKind::new(0xd, 0xf0),
        BlockKind::VerticalExtension(Offset(-16))
    );
    assert_eq!(BlockKind::new(0x9, 3), BlockKind::Door(DoorIndex(3)));
    assert_eq!(
        BlockKind::new(0xc, 0x04),
        BlockKind::Shootable(PlmChoice::Common(4))
    );
    assert_eq!(
        BlockKind::new(0xc, 0x82),
        BlockKind::Shootable(PlmChoice::Area(2))
    );
    assert_eq!(
        BlockKind::new(0xb, 0x0e),
        BlockKind::Special(PlmChoice::Common(0x0e))
    );
    assert_eq!(
        BlockKind::new(0xf, 0x01),
        BlockKind::Bombable(BombableBts::Plm(1))
    );
    assert_eq!(
        BlockKind::new(0x7, 0x80),
        BlockKind::BombableAir(BombableBts::Inert(0))
    );
    assert_eq!(BlockKind::new(0x8, 0x12), BlockKind::Solid(Bts(0x12)));
}

/// Level data of `blocks` blocks, each word `word` and BTS `bts`, and layer 2's words.
fn level(blocks: u16, word: u16, bts: u8, layer2: Option<u16>) -> Vec<u8> {
    let mut bytes = blocks.wrapping_mul(2).to_le_bytes().to_vec();
    for _ in 0..blocks {
        bytes.extend(word.to_le_bytes());
    }
    bytes.extend(std::iter::repeat_n(bts, usize::from(blocks)));
    if let Some(layer2) = layer2 {
        for _ in 0..blocks {
            bytes.extend(layer2.to_le_bytes());
        }
    }
    bytes
}

#[test]
fn the_length_says_whether_layer_2_is_there() {
    let without = Level::decode(&level(3, 0x8123, 0, None)).unwrap();
    assert_eq!(without.layer1.len(), 3);
    assert_eq!(without.layer1[0].kind, BlockKind::Solid(Bts(0)));
    assert_eq!(without.layer2, None);

    let with = Level::decode(&level(3, 0x8123, 0, Some(0x7c45))).unwrap();
    let layer2 = with.layer2.unwrap();
    assert_eq!(layer2.len(), 3);
    assert_eq!(
        layer2[0],
        Layer2Block {
            tile: Tile::from_word(0x0c45),
            unread: 7,
        }
    );
}

#[test]
fn level_data_of_another_length_is_an_error() {
    assert_eq!(Level::decode(&[4]), Err(LevelError::NoSize));
    assert_eq!(Level::decode(&[3, 0, 0, 0, 0]), Err(LevelError::OddSize(3)));
    let mut long = level(2, 0, 0, None);
    long.push(0);
    assert_eq!(
        Level::decode(&long),
        Err(LevelError::Length { blocks: 2, len: 9 })
    );
    assert_eq!(
        Level::decode(&[4, 0, 0, 0]),
        Err(LevelError::Length { blocks: 2, len: 4 })
    );
}

#[test]
fn scroll_tables_give_an_area_a_screen() {
    let mut image = vec![0; 0x8000];
    image[0x10..0x16].copy_from_slice(&[0, 1, 2, 2, 1, 0]);
    let size = Screens {
        width: 3,
        height: 2,
    };
    image[0x16] = 0x1f;
    let table = scroll_areas(Lorom::new(&image), RomAddress::new(0x80, 0x8010), size).unwrap();
    let (screens, run) = table.split_at(6);
    assert_eq!(
        screens,
        [
            ScrollArea::Red,
            ScrollArea::Blue,
            ScrollArea::Green,
            ScrollArea::Green,
            ScrollArea::Blue,
            ScrollArea::Red
        ]
    );
    // The table goes on past the room's screens, a byte no area has kept as it is.
    assert_eq!(table.len(), SCROLL_TABLE_BYTES);
    assert_eq!(run.first(), Some(&ScrollArea::Unknown(0x1f)));
    assert!(run.iter().skip(1).all(|&area| area == ScrollArea::Red));
}

#[test]
fn every_scroll_byte_goes_back_out_exact() {
    for value in 0..=u8::MAX {
        assert_eq!(u8::from(ScrollArea::from(value)), value);
    }
    assert_eq!(ScrollArea::from(3), ScrollArea::Unknown(3));
}

#[test]
fn a_pointer_below_8000_generates_the_areas() {
    let image = vec![0; 0x8000];
    let rom = Lorom::new(&image);
    let size = Screens {
        width: 2,
        height: 2,
    };
    let generated = |offset| scroll_areas(rom, RomAddress::new(0x8f, offset), size);
    let green = ScrollArea::Green;
    let blue = ScrollArea::Blue;
    assert_eq!(generated(0), Ok(vec![green, green, blue, blue]));
    assert_eq!(generated(1), Ok(vec![green; 4]));
    let unknown = ScrollArea::Unknown(3);
    assert_eq!(generated(2), Ok(vec![green, green, unknown, unknown]));
}

/// An invented level four blocks across and three down, all air but `blocks`, each a block's
/// index and its kind.
fn stand_ins(blocks: &[(u16, BlockKind)]) -> Level {
    let mut layer1 = vec![Block::default(); 12];
    for &(index, kind) in blocks {
        if let Some(block) = layer1.get_mut(usize::from(index)) {
            block.kind = kind;
        }
    }
    Level {
        layer1,
        layer2: None,
    }
}

const ACROSS: u16 = 4;

const fn right(by: i8) -> BlockKind {
    BlockKind::HorizontalExtension(Offset(by))
}

const fn down(by: i8) -> BlockKind {
    BlockKind::VerticalExtension(Offset(by))
}

#[test]
fn a_level_whose_stand_ins_end_loads() {
    let level = stand_ins(&[
        (1, right(1)),
        (2, down(1)),
        (6, BlockKind::Solid(Bts(0))),
        (5, right(-1)),
        (4, down(-1)),
        // Off the level's start, where a chain meets air.
        (8, right(-9)),
        (11, down(1)),
    ]);
    assert_eq!(level.refuse_stand_in_loops(ACROSS, []), Ok(()));
    let kind = |index: u16| {
        level
            .layer1
            .get(usize::from(index))
            .map_or_else(BlockKind::default, |block| block.kind)
    };
    assert_eq!(
        stand_in_end(12, ACROSS, kind, 1),
        Ok((6, BlockKind::Solid(Bts(0))))
    );
    assert_eq!(
        stand_in_end(12, ACROSS, kind, 8),
        Ok((u16::MAX, BlockKind::default()))
    );
}

#[test]
fn a_level_whose_stand_ins_loop_is_refused() {
    let along = stand_ins(&[(8, right(1)), (9, right(-1))]);
    assert_eq!(
        along.refuse_stand_in_loops(ACROSS, []),
        Err(LevelError::StandInLoop(StandInLoop(8)))
    );
    let through = stand_ins(&[(1, right(2)), (3, down(1)), (7, right(-2)), (5, down(-1))]);
    assert_eq!(
        through.refuse_stand_in_loops(ACROSS, []),
        Err(LevelError::StandInLoop(StandInLoop(1)))
    );
    let kind = |index: u16| {
        through
            .layer1
            .get(usize::from(index))
            .map_or_else(BlockKind::default, |block| block.kind)
    };
    assert_eq!(stand_in_end(12, ACROSS, kind, 1), Err(StandInLoop(1)));
}

#[test]
fn a_loop_the_rooms_start_closes_is_refused() {
    let level = stand_ins(&[(5, right(1)), (6, BlockKind::Solid(Bts(0xff)))]);
    assert_eq!(level.refuse_stand_in_loops(ACROSS, []), Ok(()));
    assert_eq!(
        level.refuse_stand_in_loops(ACROSS, [(6, right(-1))]),
        Err(LevelError::StandInLoop(StandInLoop(5)))
    );
    // Extensions placed back towards a stand-in that names them close a loop, unless a later
    // placement writes a block that stands for none over that stand-in.
    let placed = [(9, right(-1)), (10, right(-1))];
    let level = stand_ins(&[(8, right(2))]);
    assert_eq!(
        level.refuse_stand_in_loops(ACROSS, placed),
        Err(LevelError::StandInLoop(StandInLoop(8)))
    );
    let ended = placed
        .into_iter()
        .chain([(8, BlockKind::SpecialAir(Bts(0)))]);
    assert_eq!(level.refuse_stand_in_loops(ACROSS, ended), Ok(()));
}
