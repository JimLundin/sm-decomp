//! ROM spritemap decoding and the native host for shared object staging.
#![forbid(unsafe_code)]
use crate::ram::RawRam as _;
use crate::{scene::objects, sprite_output::SpriteHost};
use sm_data::cartridge::RomImage;
use std::convert::Infallible;

pub use objects::Clip;

pub struct SpriteMap {
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
    pub fn draw<H: SpriteHost + ?Sized>(
        self,
        position: (u16, u16),
        tile_word: impl Fn(u16) -> u16,
        clip: Clip,
        game: &mut H,
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
struct NativeGeometry<'a, G>(&'a G);
impl<G: Geometry> objects::Geometry for NativeGeometry<'_, G> {
    type Error = Infallible;
    fn count(&mut self) -> Result<u16, Infallible> {
        Ok(self.0.count())
    }
    fn x(&mut self, record: usize) -> Result<u16, Infallible> {
        Ok(self.0.x(record))
    }
    fn y(&mut self, record: usize) -> Result<u8, Infallible> {
        Ok(self.0.y(record))
    }
    fn tile(&mut self, record: usize) -> Result<u16, Infallible> {
        Ok(self.0.tile(record))
    }
}
pub struct NativeObjects<'a, H: ?Sized>(pub &'a mut H);
impl<H: SpriteHost + ?Sized> objects::Host for NativeObjects<'_, H> {
    type Error = Infallible;
    fn read_native_word(&mut self, offset: usize) -> Result<u16, Infallible> {
        let at = 0x370 + offset;
        Ok(u16::from_ne_bytes([
            self.0.ram().read_byte(at),
            self.0.ram().read_byte(at + 1),
        ]))
    }
    fn read_little_word(&mut self, offset: usize) -> Result<u16, Infallible> {
        let at = 0x370 + offset;
        Ok(u16::from_le_bytes([
            self.0.ram().read_byte(at),
            self.0.ram().read_byte(at + 1),
        ]))
    }
    fn write_byte(&mut self, offset: usize, value: u8) -> Result<(), Infallible> {
        self.0.ram().write_byte(0x370 + offset, value);
        Ok(())
    }
    fn write_native_slice(&mut self, offset: usize, value: u16) -> Result<(), Infallible> {
        self.0.ram().copy_in(0x370 + offset, &value.to_ne_bytes());
        Ok(())
    }
    fn write_native_bytes(&mut self, offset: usize, value: u16) -> Result<(), Infallible> {
        let at = 0x370 + offset;
        let bytes = value.to_ne_bytes();
        self.0.ram().write_byte(at, bytes[0]);
        self.0.ram().write_byte(at + 1, bytes[1]);
        Ok(())
    }
    fn started(&mut self, clip: Clip) -> Result<(), Infallible> {
        crate::probe::hit(match clip {
            Clip::None => "sprites.wrap",
            Clip::Inside => "sprites.inside",
            Clip::Outside => "sprites.outside",
            Clip::Bounded => "sprites.bounded",
            Clip::BoundedOutside => "sprites.bounded-outside",
        });
        Ok(())
    }
}
pub fn draw<H: SpriteHost + ?Sized>(
    map: &impl Geometry,
    position: (u16, u16),
    tile_word: impl Fn(u16) -> u16,
    clip: Clip,
    game: &mut H,
) {
    objects::draw(
        &mut NativeObjects(game),
        &mut NativeGeometry(map),
        position,
        clip,
        |word| Ok(tile_word(word)),
    )
    .unwrap();
}
pub fn finish_frame<H: SpriteHost + ?Sized>(game: &mut H) {
    objects::finish_frame(&mut NativeObjects(game)).unwrap();
}
pub fn clear_high_table<H: SpriteHost + ?Sized>(game: &mut H) {
    objects::clear_high_table(&mut NativeObjects(game)).unwrap();
}
