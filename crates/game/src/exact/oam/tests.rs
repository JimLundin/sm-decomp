use data::spritemap::{SpriteAttributes, SpriteSize, Spritemap, SpritemapEntry};
use types::Pair;

use super::HIDDEN_Y;
use crate::intent::{Placing, SpriteDraw, SpritePiece};
use crate::state::video::{NmiBatch, OAM_SPRITES, OamEntry, SpriteCount};

fn batch(sprites: u16) -> NmiBatch {
    let mut batch = NmiBatch::default();
    for (index, entry) in batch.oam.low.iter_mut().enumerate() {
        let tile = u8::try_from(index).unwrap();
        *entry = OamEntry {
            x: 1,
            y: 2,
            tile,
            attributes: 3,
        };
    }
    batch.oam.high = [0xFF; _];
    batch.sprites = SpriteCount(sprites);
    batch
}

fn look(tile: u16, palette: u8, priority: u8) -> SpriteAttributes {
    SpriteAttributes {
        tile,
        palette,
        priority,
        flip_x: false,
        flip_y: false,
    }
}

fn piece(x: i32, y: i32, placing: Placing) -> SpritePiece {
    SpritePiece {
        at: Pair { x, y },
        look: look(0x21, 1, 3),
        placing,
    }
}

fn drawn(pieces: Vec<SpritePiece>, from: u16) -> NmiBatch {
    let mut batch = NmiBatch {
        sprites: SpriteCount(from),
        ..NmiBatch::default()
    };
    batch.draw_sprites(&SpriteDraw { pieces });
    batch
}

#[test]
fn finalising_hides_only_the_undrawn_entries() {
    let mut drawn = batch(3);
    drawn.finalise_oam();
    let ys: Vec<u8> = drawn.oam.low.iter().map(|entry| entry.y).collect();
    assert_eq!(ys[..3], [2, 2, 2]);
    assert!(ys[3..].iter().all(|&y| y == HIDDEN_Y));
    assert_eq!(drawn.oam.low[100].tile, 100);
    assert_eq!(drawn.oam.low[100].x, 1);
    assert_eq!(drawn.oam.high, [0xFF; _]);
    assert_eq!(drawn.sprites, SpriteCount(0));
}

#[test]
fn a_spritemap_places_its_pieces_by_their_offsets() {
    let map = Spritemap {
        entries: vec![
            SpritemapEntry {
                x: -7,
                y: 0xF8,
                size: SpriteSize::Large,
                attributes: look(0, 4, 2),
            },
            SpritemapEntry {
                x: 10,
                y: 5,
                size: SpriteSize::Small,
                attributes: look(0x21, 0, 0),
            },
        ],
    };
    let mut draw = SpriteDraw::default();
    draw.place(&map, Pair { x: 3, y: 0x80 });
    let places: Vec<_> = draw.pieces.iter().map(|piece| piece.at).collect();
    assert_eq!(places, [Pair { x: -4, y: 0x78 }, Pair { x: 13, y: 0x85 }]);
    assert_eq!(draw.pieces[0].placing, Placing::Mapped(SpriteSize::Large));
}

#[test]
fn mapped_pieces_write_from_the_count_with_their_high_bits() {
    let large = Placing::Mapped(SpriteSize::Large);
    let small = Placing::Mapped(SpriteSize::Small);
    let drawn = drawn(vec![piece(-4, 0x78, large), piece(13, 0x185, small)], 3);
    // -4 is $1FC in nine bits: low byte $FC with the ninth bit set; Y keeps its low byte.
    assert_eq!(
        [drawn.oam.low[3], drawn.oam.low[4]],
        [
            OamEntry {
                x: 0xFC,
                y: 0x78,
                tile: 0x21,
                attributes: 0x32
            },
            OamEntry {
                x: 13,
                y: 0x85,
                tile: 0x21,
                attributes: 0x32
            }
        ]
    );
    // Entry 3: X high and large (0b11) at bits 6-7; entry 4: nothing.
    assert_eq!(drawn.oam.high[0], 0b1100_0000);
    assert_eq!(drawn.oam.high[1], 0);
    assert_eq!(drawn.sprites, SpriteCount(5));
}

#[test]
fn a_far_piece_wraps_its_x_at_512() {
    let small = Placing::Mapped(SpriteSize::Small);
    let drawn = drawn(vec![piece(512 + 20, 256 + 7, small)], 0);
    assert_eq!((drawn.oam.low[0].x, drawn.oam.low[0].y), (20, 7));
    assert_eq!(drawn.oam.high[0], 0);
}

#[test]
fn the_mapped_count_wraps_past_the_last_entry() {
    let small = Placing::Mapped(SpriteSize::Small);
    let drawn = drawn(vec![piece(0x100, 0, small), piece(0x100, 0, small)], 127);
    assert_eq!(drawn.sprites, SpriteCount(1));
    assert_eq!(drawn.oam.high[31], 0b0100_0000);
    assert_eq!(drawn.oam.high[0], 0b0000_0001);
}

#[test]
fn a_lone_sprite_keeps_the_low_table_only_and_is_lost_once_the_table_is_full() {
    let lone = drawn(vec![piece(0x105, 9, Placing::Lone)], 0);
    assert_eq!((lone.oam.low[0].x, lone.oam.low[0].y), (5, 9));
    assert_eq!(lone.oam.high[0], 0);
    assert_eq!(lone.sprites, SpriteCount(1));

    let full = u16::try_from(OAM_SPRITES).unwrap();
    let past = drawn(vec![piece(5, 9, Placing::Lone)], full);
    assert_eq!(past.oam.low[0], OamEntry::default());
    assert_eq!(past.sprites, SpriteCount(full + 1));
}

#[test]
fn a_full_table_hides_nothing() {
    let full = u16::try_from(OAM_SPRITES).unwrap();
    for count in [full, full + 1] {
        let mut drawn = batch(count);
        drawn.finalise_oam();
        assert!(drawn.oam.low.iter().all(|entry| entry.y == 2));
        assert_eq!(drawn.sprites, SpriteCount(0));
    }
}

#[test]
fn an_empty_frame_hides_every_entry_and_clears_the_high_table() {
    let mut empty = batch(9);
    empty.clear_high_oam();
    empty.restart_sprites();
    empty.finalise_oam();
    assert!(empty.oam.low.iter().all(|entry| entry.y == HIDDEN_Y));
    assert_eq!(empty.oam.high, [0; _]);
}
