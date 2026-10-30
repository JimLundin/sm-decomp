//! Checked drawing compatibility boundary. Only the native warning remains unsafe.
use crate::ram::RawRam as _;
use crate::ram::checked_word as word;
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
    let y = word!(game.SPEED_ECHO_YPOS[index]);
    let image = content.image(game);
    let height = image.read(
        sm_data::cartridge::lorom_offset(0x91b629) as usize
            + usize::from(word!(game.SAMUS_POSE)) * 8
            + 4,
    );
    let layer1 = crate::camera::current(game).layer1;
    let y = y.wrapping_sub(u16::from(height)).wrapping_sub(layer1[1]);
    if (y as i16) >= 0 && y.wrapping_sub(248) & 0x8000 != 0 {
        let sprite = word!(game.SAMUS_TOP_HALF_SPRITEMAP_INDEX);
        let x = word!(game.SPEED_ECHO_XPOS[index]).wrapping_sub(layer1[0]);
        samus(sprite, (x, y), game, content);
        if word!(game.SAMUS_BOTTOM_HALF_SPRITEMAP_INDEX) != 0 {
            let sprite = word!(game.SAMUS_BOTTOM_HALF_SPRITEMAP_INDEX);
            let x = word!(game.SPEED_ECHO_XPOS[index]).wrapping_sub(layer1[0]);
            samus(sprite, (x, y), game, content);
        }
    }
}
#[forbid(unsafe_code)]
pub fn crash_echoes<H: SpriteHost + ?Sized>(game: &mut H, content: &mut impl Content<H>) {
    if word!(game.NMI_FRAME_COUNTER_WORD) & 1 != 0 {
        for index in [3, 2] {
            if word!(game.SPEED_ECHO_XSPEED[index]) != 0 {
                echo((index * 2) as u16, game, content);
            }
        }
    }
}

/// Ordered native OAM stores; end coordinates and extension bits remain live aliases.
#[forbid(unsafe_code)]
pub fn grapple<H: SpriteHost + ?Sized>(game: &mut H, segment: Option<(u16, u16, u16)>) {
    let idx = word!(game.OAM_NEXT_PTR);
    let at = 0x370 + usize::from(idx);
    if let Some((x, y, tile)) = segment {
        game.ram().write_byte(at, x as u8);
        game.ram().write_byte(at + 1, y as u8);
        game.ram().copy_in(at + 2, &tile.to_ne_bytes());
    } else {
        let layer1 = crate::camera::current(game).layer1;
        let x = word!(game.GRAPPLE_BEAM_END_X_POS)
            .wrapping_sub(layer1[0])
            .wrapping_sub(4);
        game.ram().write_byte(at, x as u8);
        let y = word!(game.GRAPPLE_BEAM_END_Y_POS)
            .wrapping_sub(layer1[1])
            .wrapping_sub(4);
        game.ram().write_byte(at + 1, y as u8);
        game.ram().copy_in(at + 2, &14880u16.to_ne_bytes());
        let extension = word!(game.OAM_EXT[(idx >> 5)]);
        let bit = (((i32::from(x) & 0x100) >> 8) << (2 * (i32::from(idx) >> 2 & 7))) as u16;
        word!(game.OAM_EXT[(idx >> 5)] = extension | bit);
    }
    word!(game.OAM_NEXT_PTR = idx.wrapping_add(4));
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
/// The menu sprites of the helmet's animation frames.
const HELMET_SPRITES: [u16; 9] = [0x2c, 0x2d, 0x2e, 0x2f, 0x30, 0x31, 0x32, 0x33, 0x33];
/// Frames between helmet animation steps.
const HELMET_FRAME_DELAY: u16 = 8;
/// The helmet's resting frame once its animation finishes.
const HELMET_RESTING_FRAME: u16 = 7;
/// The palette of the file select menu's sprites.
const FILE_SELECT_PALETTE: u16 = 0xe00;

/// Advance each file helmet immediately before drawing it; idle menus clear each timer first.
pub fn file_helmets<H: SpriteHost + ?Sized>(animate: bool, game: &mut H) {
    use crate::enemy_projectiles::table::with_file_select;
    for helmet in 0..3 {
        let (frame, x, y) = with_file_select(game, |menu| {
            if !animate {
                *menu.helmet_animation_timer(helmet) = 0;
            }
            let timer = *menu.helmet_animation_timer(helmet);
            if timer != 0 {
                let timer = timer - 1;
                *menu.helmet_animation_timer(helmet) = timer;
                if timer == 0 {
                    advance_helmet(menu, helmet);
                }
            }
            (
                *menu.helmet_animation_frame(helmet),
                *menu.helmet_x(helmet),
                *menu.helmet_y(helmet),
            )
        });
        // The original doubles the frame into a byte offset and halves it
        // back, dropping bit 15.
        let sprite = HELMET_SPRITES[usize::from(frame & 0x7fff)];
        menu(sprite, x, y, FILE_SELECT_PALETTE, game);
    }
}

/// Step helmet `helmet` to its next frame and restart its delay; a helmet
/// on or past its resting frame stays there and stops.
fn advance_helmet(menu: &mut crate::enemy_projectiles::table::FileSelect<'_>, helmet: usize) {
    *menu.helmet_animation_timer(helmet) = HELMET_FRAME_DELAY;
    let frame = *menu.helmet_animation_frame(helmet);
    // The original's signed compare against 7 treats frames 7..=$8006 as finished.
    if (HELMET_RESTING_FRAME..=0x8006).contains(&frame) {
        *menu.helmet_animation_timer(helmet) = 0;
        *menu.helmet_animation_frame(helmet) = HELMET_RESTING_FRAME;
    } else {
        *menu.helmet_animation_frame(helmet) = frame.wrapping_add(1);
    }
}

/// The selection missile's (y, x) beside each file select row.
const FILE_MENU_MISSILE_POSITIONS: [(u16, u16); 6] = [
    (0x30, 0xe),
    (0x58, 0xe),
    (0x80, 0xe),
    (0xa3, 0xe),
    (0xbb, 0xe),
    (0xd3, 0xe),
];

/// Place the selection missile beside the selected save slot's row.
pub fn file_menu_cursor<H: SpriteHost + ?Sized>(game: &mut H) {
    // The original doubles the row into a pair offset, dropping bit 14.
    let row = usize::from(word!(game.SELECTED_SAVE_SLOT) & 0x3fff);
    let (y, x) = FILE_MENU_MISSILE_POSITIONS[row];
    crate::enemy_projectiles::table::with_file_select(game, |menu| {
        *menu.missile_y() = y;
        *menu.missile_x() = x;
    });
}

/// The selection missile's x beside the file copy and clear rows.
const FILE_SLOT_MISSILE_X: u16 = 22;

/// Place the selection missile beside the file copy or clear selection;
/// the fourth row, the exit, is at `exit_y`.
pub fn file_slot_cursor<H: SpriteHost + ?Sized>(exit_y: u16, game: &mut H) {
    let rows = [72, 104, 136, exit_y];
    crate::enemy_projectiles::table::with_file_select(game, |menu| {
        *menu.missile_y() = rows[usize::from(*menu.file_copy_selection())];
        *menu.missile_x() = FILE_SLOT_MISSILE_X;
    });
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
        |tile| (i32::from(palette) | (i32::from(base_tile) + i32::from(tile))) as u16,
        clip,
        game,
    );
}

/// The menu sprites of the selection missile's animation frames.
const MISSILE_SPRITES: [u16; 4] = [0x37, 0x36, 0x35, 0x34];
/// Frames between selection missile animation steps.
const MISSILE_FRAME_DELAY: u16 = 8;

/// Animate and draw the menu selection missile.
pub fn menu_selection<H: SpriteHost + ?Sized>(game: &mut H) {
    let (frame, x, y) = crate::enemy_projectiles::table::with_file_select(game, |menu| {
        let timer = *menu.missile_animation_timer();
        if timer != 0 {
            *menu.missile_animation_timer() = timer - 1;
            if timer == 1 {
                // The original steps the frame's low byte.
                let frame = *menu.missile_animation_frame() as u8;
                *menu.missile_animation_frame() = (u16::from(frame) + 1) & 3;
                *menu.missile_animation_timer() = MISSILE_FRAME_DELAY;
            }
        }
        (
            *menu.missile_animation_frame(),
            *menu.missile_x(),
            *menu.missile_y(),
        )
    });
    menu(
        MISSILE_SPRITES[usize::from(frame)],
        x,
        y,
        FILE_SELECT_PALETTE,
        game,
    );
}

/// The copy arrow's (sprite, x, y) rows, three words each.
const COPY_ARROWS: u32 = 0x82bb0c;

/// The copy arrow row for the source and destination save slots.
fn copy_arrow(source: u16, destination: u16) -> u16 {
    if (source as i16).wrapping_sub(destination as i16) >= 0 {
        let adjacent = (i32::from(source) - i32::from(destination) - 2) & 0x8000 != 0;
        match (adjacent, source) {
            (false, _) => 3,
            (true, 1) => 4,
            (true, _) => 5,
        }
    } else {
        let adjacent = (i32::from(destination) - i32::from(source) - 2) & 0x8000 != 0;
        match (adjacent, source) {
            (false, _) => 0,
            (true, 0) => 1,
            (true, _) => 2,
        }
    }
}

/// Draw the arrow from the file copy source slot to its destination.
pub fn file_copy_arrow<H: SpriteHost + ?Sized>(game: &mut H) {
    let (source, destination) = crate::enemy_projectiles::table::with_file_select(game, |menu| {
        (
            *menu.file_copy_source_slot(),
            *menu.file_copy_destination_slot(),
        )
    });
    let arrow = 3 * copy_arrow(source, destination) as isize;
    let image = game.native_image();
    let base = sm_data::cartridge::lorom_offset(COPY_ARROWS) as usize;
    menu(
        image.table_word(base, arrow),
        image.table_word(base, arrow + 1),
        image.table_word(base, arrow + 2),
        0x200,
        game,
    );
}

#[cfg(test)]
mod tests {
    /// The original's nested branches, kept as the reference.
    fn original_copy_arrow(source: u16, destination: u16) -> u16 {
        if (source as i16).wrapping_sub(destination as i16) >= 0 {
            if (i32::from(source) - i32::from(destination) - 2) & 0x8000 != 0 {
                if source != 1 { 5 } else { 4 }
            } else {
                3
            }
        } else if (i32::from(destination) - i32::from(source) - 2) & 0x8000 != 0 {
            if source != 0 { 2 } else { 1 }
        } else {
            0
        }
    }

    #[test]
    fn copy_arrow_matches_the_original_branches() {
        let edges = (0..=0x300).chain(0x7f00..=0x8100).chain(0xfe00..=0xffff);
        let values: Vec<u16> = edges.collect();
        for &source in &values {
            for &destination in &values {
                assert_eq!(
                    super::copy_arrow(source, destination),
                    original_copy_arrow(source, destination),
                    "{source:#x} {destination:#x}"
                );
            }
        }
    }
}
