//! Checked drawing compatibility boundary. Only the native warning remains unsafe.
use crate::game::sprite_drawing::{Clip, SpriteMap};
use crate::game_state::Game;
use crate::ram_map::checked_word as word;
use crate::rom::RomImage;

/// Image capture follows warnings, so a returning warning may replace the ROM.
pub trait Content {
    fn image(&self, game: &Game) -> RomImage;
    fn warning(&mut self, address: u32, game: &mut Game);
}
pub struct Native;
impl Content for Native {
    fn image(&self, game: &Game) -> RomImage {
        game.loaded_rom
            .as_ref()
            .expect("ROM must be loaded")
            .clone()
    }
    fn warning(&mut self, address: u32, _: &mut Game) {
        unsafe {
            crate::c_library::printf(c"RomPtr - Invalid access 0x%x!\n".as_ptr(), address);
        }
    }
}

#[forbid(unsafe_code)]
pub(crate) fn sprite_map(
    bank: u8,
    cursor: u16,
    game: &mut Game,
    content: &mut impl Content,
) -> SpriteMap {
    let address = u32::from(bank) << 16 | u32::from(cursor);
    if cursor & 0x8000 == 0 {
        content.warning(address, game);
        game.g_fail = true;
    }
    SpriteMap {
        image: content.image(game),
        offset: crate::rtl::lorom_offset(address) as usize,
    }
}
#[forbid(unsafe_code)]
pub fn projectile(slot: u16, position: (u16, u16), game: &mut Game, content: &mut impl Content) {
    let at = 0xcb8 + 2 * usize::from(slot >> 1);
    let cursor = u16::from_ne_bytes([game.g_ram[at], game.g_ram[at + 1]]);
    sprite_map(0x93, cursor, game, content).draw(position, |tile| tile, Clip::None, game);
}
#[forbid(unsafe_code)]
pub fn samus(index: u16, position: (u16, u16), game: &mut Game, content: &mut impl Content) {
    let base = crate::rtl::lorom_offset(0x92808d) as usize;
    if content.image(game).table_word(base, index as isize) != 0 {
        let cursor = content.image(game).table_word(base, index as isize);
        sprite_map(0x92, cursor, game, content).draw(position, |tile| tile, Clip::None, game);
    }
}
#[forbid(unsafe_code)]
pub fn echo(slot: u16, game: &mut Game, content: &mut impl Content) {
    let index = usize::from(slot >> 1);
    let y = word!(game.speed_echo_ypos[index]);
    let image = content.image(game);
    let height = image.read(
        crate::rtl::lorom_offset(0x91b629) as usize + usize::from(word!(game.samus_pose)) * 8 + 4,
    );
    let y = y
        .wrapping_sub(u16::from(height))
        .wrapping_sub(word!(game.layer1_y_pos));
    if (y as i16) >= 0 && y.wrapping_sub(248) & 0x8000 != 0 {
        let sprite = word!(game.samus_top_half_spritemap_index);
        let x = word!(game.speed_echo_xpos[index]).wrapping_sub(word!(game.layer1_x_pos));
        samus(sprite, (x, y), game, content);
        if word!(game.samus_bottom_half_spritemap_index) != 0 {
            let sprite = word!(game.samus_bottom_half_spritemap_index);
            let x = word!(game.speed_echo_xpos[index]).wrapping_sub(word!(game.layer1_x_pos));
            samus(sprite, (x, y), game, content);
        }
    }
}
#[forbid(unsafe_code)]
pub fn crash_echoes(game: &mut Game, content: &mut impl Content) {
    if word!(game.nmi_frame_counter_word) & 1 != 0 {
        for index in [3, 2] {
            if word!(game.speed_echo_xspeed[index]) != 0 {
                echo((index * 2) as u16, game, content);
            }
        }
    }
}
