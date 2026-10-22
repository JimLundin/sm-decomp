//! ROM spritemap decoding and ordered writes into the SNES OAM staging bytes.
#![forbid(unsafe_code)]
use crate::{game_state::Game, rom::RomImage};

#[derive(Clone, Copy)]
pub(super) enum Clip {
    None,
    Inside,
    Outside,
    Bounded,
    BoundedOutside,
}

pub(super) struct SpriteMap {
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
        mut self,
        position: (u16, u16),
        tile_word: impl Fn(u16) -> u16,
        clip: Clip,
        game: &mut Game,
    ) {
        let mut index = u16::from_ne_bytes([game.g_ram[0x590], game.g_ram[0x591]]) as usize;
        let count = self.word(0);
        self.offset += 2;
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
        for _ in 0..count {
            if bounded && index >= 0x200 {
                break;
            }
            let mut x = position.0.wrapping_add(self.word(0));
            if bounded {
                let mut y = u16::from(position.1 as u8) + u16::from(self.byte(2));
                let hidden = if self.byte(2) & 0x80 == 0 {
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
                game.g_ram[0x370 + index] = x as u8;
                game.g_ram[0x371 + index] = y as u8;
                let tile = tile_word(self.word(3));
                game.g_ram[0x372 + index..0x374 + index].copy_from_slice(&tile.to_ne_bytes());
                self.extension(index, x, game);
                index += 4;
            } else {
                game.g_ram[0x370 + index] = x as u8;
                self.extension(index, x, game);
                let y = u16::from(position.1 as u8) + u16::from(self.byte(2));
                let inside = (y & 0x100 == 0) == (self.byte(2) & 0x80 == 0);
                game.g_ram[0x371 + index] =
                    if matches!(clip, Clip::None) || inside == matches!(clip, Clip::Inside) {
                        y as u8
                    } else {
                        0xf0
                    };
                let tile = tile_word(self.word(3));
                game.g_ram[0x372 + index..0x374 + index].copy_from_slice(&tile.to_ne_bytes());
                index = (index + 4) & 0x1ff;
            }
            self.offset += 5;
        }
        game.g_ram[0x590..0x592].copy_from_slice(&(index as u16).to_ne_bytes());
    }
    fn extension(&self, index: usize, x: u16, game: &mut Game) {
        let offset = 0x570 + 2 * (index >> 5);
        let old = u16::from_ne_bytes([game.g_ram[offset], game.g_ram[offset + 1]]);
        let bits =
            ((x & 0x100) >> 8 | u16::from((self.word(0) as i16) < 0) * 2) << (2 * (index >> 2 & 7));
        game.g_ram[offset..offset + 2].copy_from_slice(&(old | bits).to_ne_bytes());
    }
}
