//! ROM spritemap decoding and ordered writes into the SNES OAM staging bytes.
#![forbid(unsafe_code)]
use crate::{game_state::Game, rom::RomImage};

#[derive(Clone, Copy)]
pub(crate) enum Clip {
    None,
    Inside,
    Outside,
    Bounded,
    BoundedOutside,
}

pub(crate) struct SpriteMap {
    pub image: RomImage,
    pub offset: usize,
}
impl SpriteMap {
    fn byte(&self, offset: usize) -> u8 {
        self.image.read(self.offset + offset)
    }
    fn word(&self, offset: usize) -> u16 {
        u16::from_ne_bytes([self.byte(offset), self.byte(offset + 1)])
    }

    pub fn draw(
        self,
        position: (u16, u16),
        tile_word: impl Fn(u16) -> u16,
        clip: Clip,
        game: &mut Game,
    ) {
        draw(&self, position, tile_word, clip, game);
    }
}
pub trait Geometry {
    fn count(&self) -> u16;
    fn x(&self, record: usize) -> u16;
    fn y(&self, record: usize) -> u8;
    fn tile(&self, record: usize) -> u16;
}
impl Geometry for SpriteMap {
    fn count(&self) -> u16 {
        self.word(0)
    }
    fn x(&self, record: usize) -> u16 {
        self.word(2 + 5 * record)
    }
    fn y(&self, record: usize) -> u8 {
        self.byte(4 + 5 * record)
    }
    fn tile(&self, record: usize) -> u16 {
        self.word(5 + 5 * record)
    }
}
pub fn draw(
    map: &impl Geometry,
    position: (u16, u16),
    tile_word: impl Fn(u16) -> u16,
    clip: Clip,
    game: &mut Game,
) {
    let mut index =
        u16::from_ne_bytes([game.ram().read_byte(0x590), game.ram().read_byte(0x591)]) as usize;
    let count = map.count();
    let bounded = matches!(clip, Clip::Bounded | Clip::BoundedOutside);
    if count != 0 && (!bounded || index < 0x200) {
        crate::verify_probe::hit(match clip {
            Clip::None => "sprites.wrap",
            Clip::Inside => "sprites.inside",
            Clip::Outside => "sprites.outside",
            Clip::Bounded => "sprites.bounded",
            Clip::BoundedOutside => "sprites.bounded-outside",
        });
    }
    for record in 0..usize::from(count) {
        if bounded && index >= 0x200 {
            break;
        }
        let mut x = position.0.wrapping_add(map.x(record));
        if bounded {
            let mut y = u16::from(position.1 as u8) + u16::from(map.y(record));
            let hidden = if map.y(record) & 0x80 == 0 {
                y >= 0xe0
            } else if y & 0x100 != 0 {
                y as u8 >= 0xe0
            } else {
                (y as u8) < 0xe0
            };
            if hidden != matches!(clip, Clip::BoundedOutside) {
                x = 0x180;
                y = 0xe0;
            }
            game.ram().write_byte(0x370 + index, x as u8);
            game.ram().write_byte(0x371 + index, y as u8);
            let tile = tile_word(map.tile(record));
            game.ram().copy_in(0x372 + index, &tile.to_ne_bytes());
            extension(map, record, index, x, game);
            index += 4;
        } else {
            game.ram().write_byte(0x370 + index, x as u8);
            extension(map, record, index, x, game);
            let y = u16::from(position.1 as u8) + u16::from(map.y(record));
            let inside = (y & 0x100 == 0) == (map.y(record) & 0x80 == 0);
            game.ram().write_byte(
                0x371 + index,
                if matches!(clip, Clip::None) || inside == matches!(clip, Clip::Inside) {
                    y as u8
                } else {
                    0xf0
                },
            );
            let tile = tile_word(map.tile(record));
            game.ram().copy_in(0x372 + index, &tile.to_ne_bytes());
            index = (index + 4) & 0x1ff;
        }
    }
    game.ram().copy_in(0x590, &(index as u16).to_ne_bytes());
}
fn extension(map: &impl Geometry, record: usize, index: usize, x: u16, game: &mut Game) {
    let offset = 0x570 + 2 * (index >> 5);
    let old = u16::from_ne_bytes([
        game.ram().read_byte(offset),
        game.ram().read_byte(offset + 1),
    ]);
    let bits =
        ((x & 0x100) >> 8 | u16::from((map.x(record) as i16) < 0) * 2) << (2 * (index >> 2 & 7));
    game.ram().copy_in(offset, &(old | bits).to_ne_bytes());
}

pub(crate) fn finish_frame(game: &mut Game) {
    let first_unused_sprite =
        u16::from_le_bytes([game.ram().read_byte(0x590), game.ram().read_byte(0x591)]) >> 2;
    for sprite in first_unused_sprite..128 {
        // Each four-byte OAM entry stores its Y coordinate in byte 1.
        game.ram().write_byte(0x371 + usize::from(sprite) * 4, 0xf0);
    }
    word_transfers!(game, ascending, 1; native(0x590) <- value(0));
}

pub(crate) fn clear_high_table(game: &mut Game) {
    word_transfers!(game, ascending, 16; native(0x570) <- value(0));
}
