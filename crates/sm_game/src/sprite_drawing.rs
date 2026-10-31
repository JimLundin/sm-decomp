//! ROM spritemap decoding and the native host for shared object staging.
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
/// ROM geometry for a host whose errors are `E`; reading it cannot fail.
struct NativeGeometry<'a, G, E = Infallible>(&'a G, std::marker::PhantomData<E>);
impl<G: Geometry, E> objects::Geometry for NativeGeometry<'_, G, E> {
    type Error = E;
    fn count(&mut self) -> Result<u16, E> {
        Ok(self.0.count())
    }
    fn x(&mut self, record: usize) -> Result<u16, E> {
        Ok(self.0.x(record))
    }
    fn y(&mut self, record: usize) -> Result<u8, E> {
        Ok(self.0.y(record))
    }
    fn tile(&mut self, record: usize) -> Result<u16, E> {
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
            Clip::Inside | Clip::EnemyInside => "sprites.inside",
            Clip::Outside | Clip::EnemyOutside => "sprites.outside",
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
        &mut NativeGeometry(map, std::marker::PhantomData),
        position,
        clip,
        |word| Ok(tile_word(word)),
    )
    .unwrap();
}
/// Object staging as the host's session view projects it ($7E:0370..$0591).
pub struct SessionObjects<'a, H: ?Sized>(pub &'a mut H);
impl<H: crate::runtime_access::SessionRam + ?Sized> objects::Host for SessionObjects<'_, H> {
    type Error = Infallible;
    fn read_native_word(&mut self, offset: usize) -> Result<u16, Infallible> {
        Ok(self.0.native_word(0x370 + offset))
    }
    fn read_little_word(&mut self, offset: usize) -> Result<u16, Infallible> {
        Ok(self.0.word(0x370 + offset))
    }
    fn write_byte(&mut self, offset: usize, value: u8) -> Result<(), Infallible> {
        self.0.set_byte(0x370 + offset, value);
        Ok(())
    }
    fn write_native_slice(&mut self, offset: usize, value: u16) -> Result<(), Infallible> {
        self.0.write_slice_native(0x370 + offset, value);
        Ok(())
    }
    fn write_native_bytes(&mut self, offset: usize, value: u16) -> Result<(), Infallible> {
        let [first, second] = value.to_ne_bytes();
        self.0.set_byte(0x370 + offset, first);
        self.0.set_byte(0x371 + offset, second);
        Ok(())
    }
    fn started(&mut self, _clip: Clip) -> Result<(), Infallible> {
        Ok(())
    }
}
/// Point the object cursor back at the first entry, on the projected objects.
pub fn reset_cursor<H: crate::runtime_access::SessionRam + ?Sized>(game: &mut H) {
    let Ok(()) = objects::reset_cursor(&mut SessionObjects(game));
}
/// The ending's unused-sprite fill, on the projected objects; see
/// `objects::hide_unused_large`.
pub fn hide_unused_large<H: crate::runtime_access::SessionRam + ?Sized>(game: &mut H) {
    let Ok(()) = objects::hide_unused_large(&mut SessionObjects(game));
}
/// A boss's room-coordinate spritemap on `host`'s objects; see
/// `objects::draw_room`.
pub fn draw_room<H: objects::Host>(
    host: &mut H,
    map: &SpriteMap,
    origin: (u16, u16),
    camera: (u16, u16),
    palette: u16,
    clip: objects::RoomClip,
) -> Result<(), H::Error> {
    let mut geometry = NativeGeometry(map, std::marker::PhantomData);
    objects::draw_room(host, &mut geometry, origin, camera, palette, clip)
}

/// The display's objects, for the bosses' own spritemap routines: the
/// display's staging when the Session owns the display, otherwise the
/// objects as the session view projects them.
pub trait DisplayObjects: crate::runtime_access::SessionRam {
    fn owned_objects(&mut self) -> Option<&mut crate::scene::state::ObjectStaging> {
        None
    }
    /// Add `map` at room position `origin` to OAM; see `objects::draw_room`.
    fn draw_room_spritemap(
        &mut self,
        map: &SpriteMap,
        origin: (u16, u16),
        camera: (u16, u16),
        palette: u16,
        clip: objects::RoomClip,
    ) {
        if let Some(objects) = self.owned_objects() {
            draw_room(objects, map, origin, camera, palette, clip).expect("display objects");
        } else {
            let Ok(()) = draw_room(
                &mut SessionObjects(self),
                map,
                origin,
                camera,
                palette,
                clip,
            );
        }
    }
}
pub fn finish_frame<H: SpriteHost + ?Sized>(game: &mut H) {
    objects::finish_frame(&mut NativeObjects(game)).unwrap();
}
pub fn clear_high_table<H: SpriteHost + ?Sized>(game: &mut H) {
    objects::clear_high_table(&mut NativeObjects(game)).unwrap();
}
