//! Lazy spritemap geometry and access to the canonical object staging owner.
use crate::runtime_access::SessionRam;
use crate::scene::objects;
use crate::session_access::SessionAccess;
use crate::session_owners::OBJECTS;
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
    pub fn draw<H: ObjectAccess + ?Sized>(
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
/// Canonical object storage, with narrow compatibility handling for actual spill aliases.
/// Offsets retain the original full byte identity relative to staging.
pub trait ObjectAccess {
    fn object_native_word(&self, offset: usize) -> u16;
    fn object_little_word(&self, offset: usize) -> u16;
    fn object_write_byte(&mut self, offset: usize, value: u8);
    /// Preflight both bytes before storing either.
    fn object_write_native_slice(&mut self, offset: usize, value: u16);
    /// Store the first byte before attempting the second.
    fn object_write_native_bytes(&mut self, offset: usize, value: u16);
}
/// The same canonical owner serves ordinary, Samus and boss drawing.
pub struct SessionObjects<'a, H: ?Sized>(pub &'a mut H);
impl<H: ObjectAccess + ?Sized> objects::Host for SessionObjects<'_, H> {
    type Error = Infallible;
    fn read_native_word(&mut self, offset: usize) -> Result<u16, Infallible> {
        Ok(self.0.object_native_word(offset))
    }
    fn read_little_word(&mut self, offset: usize) -> Result<u16, Infallible> {
        Ok(self.0.object_little_word(offset))
    }
    fn write_byte(&mut self, offset: usize, value: u8) -> Result<(), Infallible> {
        self.0.object_write_byte(offset, value);
        Ok(())
    }
    fn write_native_slice(&mut self, offset: usize, value: u16) -> Result<(), Infallible> {
        self.0.object_write_native_slice(offset, value);
        Ok(())
    }
    fn write_native_bytes(&mut self, offset: usize, value: u16) -> Result<(), Infallible> {
        self.0.object_write_native_bytes(offset, value);
        Ok(())
    }
    fn started(&mut self, _clip: Clip) -> Result<(), Infallible> {
        Ok(())
    }
}
pub fn draw<H: ObjectAccess + ?Sized>(
    map: &impl Geometry,
    position: (u16, u16),
    tile_word: impl Fn(u16) -> u16,
    clip: Clip,
    game: &mut H,
) {
    objects::draw(
        &mut SessionObjects(game),
        &mut NativeGeometry(map, std::marker::PhantomData),
        position,
        clip,
        |word| Ok(tile_word(word)),
    )
    .unwrap();
}
/// Point the object cursor back at the first entry, on the projected objects.
pub fn reset_cursor<H: ObjectAccess + ?Sized>(game: &mut H) {
    let Ok(()) = objects::reset_cursor(&mut SessionObjects(game));
}
/// The ending's unused-sprite fill, on the projected objects; see
/// `objects::hide_unused_large`.
pub fn hide_unused_large<H: ObjectAccess + ?Sized>(game: &mut H) {
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

/// Boss room drawing uses the same storage protocol as every public draw path.
pub trait DisplayObjects: ObjectAccess {
    fn draw_room_spritemap(
        &mut self,
        map: &SpriteMap,
        origin: (u16, u16),
        camera: (u16, u16),
        palette: u16,
        clip: objects::RoomClip,
    ) {
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
pub fn finish_frame<H: ObjectAccess + ?Sized>(game: &mut H) {
    objects::finish_frame(&mut SessionObjects(game)).unwrap();
}
pub fn clear_high_table<H: ObjectAccess + ?Sized>(game: &mut H) {
    objects::clear_high_table(&mut SessionObjects(game)).unwrap();
}

/// The object cursor ($0590) as the CPU reads it, low byte first.
pub fn object_cursor<H: SessionAccess + ?Sized>(host: &H) -> u16 {
    let objects = host.session().presentation.objects();
    u16::from_le_bytes(objects.native_order.encode(objects.cursor))
}
/// Publish the object cursor, low byte first.
pub fn publish_object_cursor<H: SessionAccess + ?Sized>(host: &mut H, value: u16) {
    let objects = host.session_mut().presentation.objects_mut();
    objects.cursor = objects.native_order.decode(value.to_le_bytes());
    host.words_stored();
}
/// Store `value` at `offset` from the first object entry. Past the cursor
/// word, the store reaches the RAM after it, as the original's indexed store did.
pub fn store_object_byte<H: SessionAccess + SessionRam + ?Sized>(
    host: &mut H,
    offset: usize,
    value: u8,
) {
    let objects = host.session_mut().presentation.objects_mut();
    if objects.write_byte(offset, value).is_ok() {
        host.words_stored();
    } else {
        host.set_byte(OBJECTS.start + offset, value);
    }
}
/// Store `value` in native byte order at `offset` from the first object entry.
/// A word that does not lie within the objects and cursor keeps the original
/// slice store, which checks its whole extent before storing either byte.
pub fn store_object_native_word<H: SessionAccess + SessionRam + ?Sized>(
    host: &mut H,
    offset: usize,
    value: u16,
) {
    let bytes = value.to_ne_bytes();
    if offset + bytes.len() <= OBJECTS.len() {
        let objects = host.session_mut().presentation.objects_mut();
        for (lane, byte) in bytes.into_iter().enumerate() {
            objects
                .write_byte(offset + lane, byte)
                .expect("object staging offset");
        }
        host.words_stored();
    } else {
        host.write_slice(OBJECTS.start + offset, &bytes);
    }
}
