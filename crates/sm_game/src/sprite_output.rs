//! Checked drawing compatibility boundary. Only the native warning remains unsafe.
// The original's zero-initialised assignments are kept.
#![allow(unused_assignments)]
use crate::ram::RawRam as _;
use crate::ram::checked_word as word;
use crate::ram::word as game_word;
use crate::sprite_drawing::{Clip, SpriteMap};
use sm_data::cartridge::RomImage;

/// What sprite output reads and writes: named and raw work RAM, the cartridge,
/// the original's invalid-ROM warning and its failure flag.
pub trait SpriteHost:
    crate::ram::CompatibilityRam
    + crate::records::Records
    + crate::ram::WorkRamAccess
    + crate::cartridge::LoadedImage
    + crate::cartridge::NativeImage
    + crate::runtime_access::RuntimeAccess
{
    /// Print the original "RomPtr - Invalid access" warning for `address`.
    fn warn_invalid_rom(&mut self, address: u32);
    /// Set the original's failure flag.
    fn set_failure(&mut self);
}

/// Image capture follows warnings, so a returning warning may replace the ROM.
pub trait Content<H: ?Sized> {
    fn image(&self, game: &H) -> RomImage;
    fn warning(&mut self, address: u32, game: &mut H);
}
pub struct Native;
impl<H: SpriteHost + ?Sized> Content<H> for Native {
    fn image(&self, game: &H) -> RomImage {
        game.loaded_image().clone()
    }
    fn warning(&mut self, address: u32, game: &mut H) {
        game.warn_invalid_rom(address);
    }
}

#[forbid(unsafe_code)]
pub fn sprite_map<H: SpriteHost + ?Sized>(
    bank: u8,
    cursor: u16,
    game: &mut H,
    content: &mut impl Content<H>,
) -> SpriteMap {
    let address = u32::from(bank) << 16 | u32::from(cursor);
    if cursor & 0x8000 == 0 {
        content.warning(address, game);
        game.set_failure();
    }
    SpriteMap {
        image: content.image(game),
        offset: sm_data::cartridge::lorom_offset(address) as usize,
    }
}
#[forbid(unsafe_code)]
pub fn projectile<H: SpriteHost + ?Sized>(
    slot: u16,
    position: (u16, u16),
    game: &mut H,
    content: &mut impl Content<H>,
) {
    let at = 0xcb8 + 2 * usize::from(slot >> 1);
    let cursor = u16::from_ne_bytes([game.ram().read_byte(at), game.ram().read_byte(at + 1)]);
    sprite_map(0x93, cursor, game, content).draw(position, |tile| tile, Clip::None, game);
}
#[forbid(unsafe_code)]
pub fn samus<H: SpriteHost + ?Sized>(
    index: u16,
    position: (u16, u16),
    game: &mut H,
    content: &mut impl Content<H>,
) {
    crate::scene::samus::draw(&mut NativeSamus { game, content }, index, position).unwrap();
}
struct NativeSamus<'a, H: ?Sized, C> {
    game: &'a mut H,
    content: &'a mut C,
}
struct NativeSamusMap(SpriteMap);
impl crate::scene::objects::Geometry for NativeSamusMap {
    type Error = std::convert::Infallible;
    fn count(&mut self) -> Result<u16, Self::Error> {
        Ok(crate::sprite_drawing::Geometry::count(&self.0))
    }
    fn x(&mut self, record: usize) -> Result<u16, Self::Error> {
        Ok(crate::sprite_drawing::Geometry::x(&self.0, record))
    }
    fn y(&mut self, record: usize) -> Result<u8, Self::Error> {
        Ok(crate::sprite_drawing::Geometry::y(&self.0, record))
    }
    fn tile(&mut self, record: usize) -> Result<u16, Self::Error> {
        Ok(crate::sprite_drawing::Geometry::tile(&self.0, record))
    }
}
impl<H: SpriteHost + ?Sized, C: Content<H>> crate::scene::objects::Host for NativeSamus<'_, H, C> {
    type Error = std::convert::Infallible;
    fn read_native_word(&mut self, offset: usize) -> Result<u16, Self::Error> {
        crate::scene::objects::Host::read_native_word(
            &mut crate::sprite_drawing::NativeObjects(self.game),
            offset,
        )
    }
    fn read_little_word(&mut self, offset: usize) -> Result<u16, Self::Error> {
        crate::scene::objects::Host::read_little_word(
            &mut crate::sprite_drawing::NativeObjects(self.game),
            offset,
        )
    }
    fn write_byte(&mut self, offset: usize, value: u8) -> Result<(), Self::Error> {
        crate::scene::objects::Host::write_byte(
            &mut crate::sprite_drawing::NativeObjects(self.game),
            offset,
            value,
        )
    }
    fn write_native_slice(&mut self, offset: usize, value: u16) -> Result<(), Self::Error> {
        crate::scene::objects::Host::write_native_slice(
            &mut crate::sprite_drawing::NativeObjects(self.game),
            offset,
            value,
        )
    }
    fn write_native_bytes(&mut self, offset: usize, value: u16) -> Result<(), Self::Error> {
        crate::scene::objects::Host::write_native_bytes(
            &mut crate::sprite_drawing::NativeObjects(self.game),
            offset,
            value,
        )
    }
    fn started(&mut self, clip: Clip) -> Result<(), Self::Error> {
        crate::scene::objects::Host::started(
            &mut crate::sprite_drawing::NativeObjects(self.game),
            clip,
        )
    }
}
impl<H: SpriteHost + ?Sized, C: Content<H>> crate::scene::samus::Host for NativeSamus<'_, H, C> {
    type Map = NativeSamusMap;
    fn table_word(&mut self, index: u16) -> Result<u16, Self::Error> {
        let base = sm_data::cartridge::lorom_offset(0x92808d) as usize;
        Ok(self
            .content
            .image(self.game)
            .table_word(base, index as isize))
    }
    fn sprite_map(&mut self, cursor: u16) -> Result<NativeSamusMap, Self::Error> {
        Ok(NativeSamusMap(sprite_map(
            0x92,
            cursor,
            self.game,
            self.content,
        )))
    }
    fn published(&mut self, _: usize, _: usize, _: u16, _: u16, _: u16) -> Result<(), Self::Error> {
        Ok(())
    }
}
#[forbid(unsafe_code)]
pub fn echo<H: SpriteHost + ?Sized>(slot: u16, game: &mut H, content: &mut impl Content<H>) {
    let index = usize::from(slot >> 1);
    let y = word!(game.speed_echo_ypos[index]);
    let image = content.image(game);
    let height = image.read(
        sm_data::cartridge::lorom_offset(0x91b629) as usize
            + usize::from(word!(game.samus_pose)) * 8
            + 4,
    );
    let layer1 = crate::camera::current(game).layer1;
    let y = y.wrapping_sub(u16::from(height)).wrapping_sub(layer1[1]);
    if (y as i16) >= 0 && y.wrapping_sub(248) & 0x8000 != 0 {
        let sprite = word!(game.samus_top_half_spritemap_index);
        let x = word!(game.speed_echo_xpos[index]).wrapping_sub(layer1[0]);
        samus(sprite, (x, y), game, content);
        if word!(game.samus_bottom_half_spritemap_index) != 0 {
            let sprite = word!(game.samus_bottom_half_spritemap_index);
            let x = word!(game.speed_echo_xpos[index]).wrapping_sub(layer1[0]);
            samus(sprite, (x, y), game, content);
        }
    }
}
#[forbid(unsafe_code)]
pub fn crash_echoes<H: SpriteHost + ?Sized>(game: &mut H, content: &mut impl Content<H>) {
    if word!(game.nmi_frame_counter_word) & 1 != 0 {
        for index in [3, 2] {
            if word!(game.speed_echo_xspeed[index]) != 0 {
                echo((index * 2) as u16, game, content);
            }
        }
    }
}

/// Ordered native OAM stores; end coordinates and extension bits remain live aliases.
#[forbid(unsafe_code)]
pub fn grapple<H: SpriteHost + ?Sized>(game: &mut H, segment: Option<(u16, u16, u16)>) {
    let idx = word!(game.oam_next_ptr);
    let at = 0x370 + usize::from(idx);
    if let Some((x, y, tile)) = segment {
        game.ram().write_byte(at, x as u8);
        game.ram().write_byte(at + 1, y as u8);
        game.ram().copy_in(at + 2, &tile.to_ne_bytes());
    } else {
        let layer1 = crate::camera::current(game).layer1;
        let x = word!(game.grapple_beam_end_x_pos)
            .wrapping_sub(layer1[0])
            .wrapping_sub(4);
        game.ram().write_byte(at, x as u8);
        let y = word!(game.grapple_beam_end_y_pos)
            .wrapping_sub(layer1[1])
            .wrapping_sub(4);
        game.ram().write_byte(at + 1, y as u8);
        game.ram().copy_in(at + 2, &14880u16.to_ne_bytes());
        let extension = word!(game.oam_ext[(idx >> 5)]);
        let bit = (((i32::from(x) & 0x100) >> 8) << (2 * (i32::from(idx) >> 2 & 7))) as u16;
        word!(game.oam_ext[(idx >> 5)] = extension | bit);
    }
    word!(game.oam_next_ptr = idx.wrapping_add(4));
}

#[forbid(unsafe_code)]
pub fn paletted<H: SpriteHost + ?Sized>(
    bank: u8,
    address: u16,
    position: (u16, u16),
    palette: u16,
    clip: Clip,
    game: &mut H,
) {
    sprite_map(bank, address, game, &mut Native).draw(
        position,
        |tile| palette | tile & 0xf1ff,
        clip,
        game,
    );
}
#[forbid(unsafe_code)]
pub fn menu<H: SpriteHost + ?Sized>(index: u16, x: u16, y: u16, palette: u16, game: &mut H) {
    let address = crate::menu_lists::menu_spritemap(game, index as isize);
    paletted(0x82, address, (x, y), palette, Clip::None, game);
}
/// Advance each file helmet immediately before drawing it; idle menus clear each timer first.
#[forbid(unsafe_code)]
pub fn file_helmets<H: SpriteHost + ?Sized>(animate: bool, game: &mut H) {
    const SPRITES: [u16; 9] = [0x2c, 0x2d, 0x2e, 0x2f, 0x30, 0x31, 0x32, 0x33, 0x33];
    for slot in 2..5 {
        let timer = 0x198d + 2 * slot;
        if !animate {
            game.ram().fill(timer..timer + 2, 0);
        }
        let remaining =
            u16::from_ne_bytes([game.ram().read_byte(timer), game.ram().read_byte(timer + 1)]);
        if remaining != 0 {
            let remaining = remaining.wrapping_sub(1);
            game.ram().copy_in(timer, &remaining.to_ne_bytes());
            if remaining == 0 {
                game.ram().copy_in(timer, &8u16.to_ne_bytes());
                let mut frame = word!(game.eproj_id[slot]).wrapping_add(1);
                if i32::from(word!(game.eproj_id[slot])) - 7 & 0x8000 == 0 {
                    game.ram().fill(timer..timer + 2, 0);
                    frame = 7;
                }
                word!(game.eproj_id[slot] = frame);
            }
        }
        // The doubled frame wraps before the sprite-table index is recovered.
        let sprite = word!(game.eproj_id[slot]).wrapping_mul(2) >> 1;
        menu(
            SPRITES[usize::from(sprite)],
            word!(game.eproj_id[slot + 5]),
            word!(game.eproj_id[slot + 10]),
            3584,
            game,
        );
    }
}

#[forbid(unsafe_code)]
pub fn file_menu_cursor<H: SpriteHost + ?Sized>(game: &mut H) {
    const POSITIONS: [u16; 12] = [
        0x30, 0xe, 0x58, 0xe, 0x80, 0xe, 0xa3, 0xe, 0xbb, 0xe, 0xd3, 0xe,
    ];
    let index = usize::from(word!(game.selected_save_slot).wrapping_mul(4) >> 1);
    word!(game.eproj_id[10] = POSITIONS[index]);
    word!(game.eproj_id[5] = POSITIONS[index + 1]);
}

#[forbid(unsafe_code)]
pub fn file_slot_cursor<H: SpriteHost + ?Sized>(exit_y: u16, game: &mut H) {
    let positions = [72, 104, 136, exit_y];
    word!(game.eproj_id[10] = positions[usize::from(word!(game.eproj_id[15]))]);
    word!(game.eproj_id[5] = 22);
}

#[forbid(unsafe_code)]
pub fn beam<H: SpriteHost + ?Sized>(index: u16, position: (u16, u16), game: &mut H) {
    let address = crate::menu_lists::beam_spritemap(game, index as isize);
    sprite_map(0x93, address, game, &mut Native).draw(position, |tile| tile, Clip::None, game);
}
#[forbid(unsafe_code)]
pub fn with_base_tile<H: SpriteHost + ?Sized>(
    bank: u8,
    address: u16,
    position: (u16, u16),
    palette: u16,
    base_tile: u16,
    clip: Clip,
    game: &mut H,
) {
    if address == 0 && matches!(clip, Clip::None) {
        return;
    }
    sprite_map(bank, address, game, &mut Native).draw(
        position,
        |tile| (i32::from(palette) | i32::from(base_tile) + i32::from(tile)) as u16,
        clip,
        game,
    );
}

#[forbid(unsafe_code)]
pub fn menu_selection<H: SpriteHost + ?Sized>(game: &mut H) {
    const SPRITES: [u16; 4] = [0x37, 0x36, 0x35, 0x34];
    if word!(game.cinematic_var5) != 0 {
        if game_word!(game.cinematic_var5 -= 1, reread) == 0 {
            word!(game.eproj_id[0] = (u16::from(game.ram().read_byte(0x1997)) + 1) & 3);
            word!(game.cinematic_var5 = 8);
        }
    }
    menu(
        SPRITES[usize::from(word!(game.eproj_id[0]))],
        word!(game.eproj_id[5]),
        word!(game.eproj_id[10]),
        3584,
        game,
    );
}

#[forbid(unsafe_code)]
pub fn file_copy_arrow<H: SpriteHost + ?Sized>(game: &mut H) {
    let mut v0: u16 = 0;
    if ((word!(game.eproj_id[16])) as i16).wrapping_sub((word!(game.eproj_id[17])) as i16) >= 0 {
        v0 = 3;
        if word!(game.eproj_id[16]) as i32 - word!(game.eproj_id[17]) as i32 - 2i32 & 0x8000 != 0 {
            v0 = 4;
            if word!(game.eproj_id[16]) != 1 {
                v0 = 5;
            }
        }
    } else {
        v0 = 0;
        if word!(game.eproj_id[17]) as i32 - word!(game.eproj_id[16]) as i32 - 2i32 & 0x8000 != 0 {
            v0 = 1;
            if word!(game.eproj_id[16]) != 0 {
                v0 = 2;
            }
        }
    }
    let image = game.native_image();
    let base = sm_data::cartridge::lorom_offset(0x82bb0c) as usize;
    menu(
        image.table_word(base, 3 * v0 as isize),
        image.table_word(base, 3 * v0 as isize + 1),
        image.table_word(base, 3 * v0 as isize + 2),
        512,
        game,
    );
}
