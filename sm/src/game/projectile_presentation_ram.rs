//! Checked drawing compatibility boundary. Only the native warning remains unsafe.
use crate::game::sprite_drawing::{Clip, SpriteMap};
use crate::game_state::Game;
use crate::ram_map::checked_word as word;
use crate::ram_map::word as game_word;
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

/// Ordered native OAM stores; end coordinates and extension bits remain live aliases.
#[forbid(unsafe_code)]
pub(crate) fn grapple(game: &mut Game, segment: Option<(u16, u16, u16)>) {
    let idx = word!(game.oam_next_ptr);
    let at = 0x370 + usize::from(idx);
    if let Some((x, y, tile)) = segment {
        game.g_ram[at] = x as u8;
        game.g_ram[at + 1] = y as u8;
        game.g_ram[at + 2..at + 4].copy_from_slice(&tile.to_ne_bytes());
    } else {
        let x = word!(game.grapple_beam_end_x_pos)
            .wrapping_sub(word!(game.layer1_x_pos))
            .wrapping_sub(4);
        game.g_ram[at] = x as u8;
        let y = word!(game.grapple_beam_end_y_pos)
            .wrapping_sub(word!(game.layer1_y_pos))
            .wrapping_sub(4);
        game.g_ram[at + 1] = y as u8;
        game.g_ram[at + 2..at + 4].copy_from_slice(&14880u16.to_ne_bytes());
        let extension = word!(game.oam_ext[(idx >> 5)]);
        let bit = (((i32::from(x) & 0x100) >> 8) << (2 * (i32::from(idx) >> 2 & 7))) as u16;
        word!(game.oam_ext[(idx >> 5)] = extension | bit);
    }
    word!(game.oam_next_ptr = idx.wrapping_add(4));
}

#[forbid(unsafe_code)]
pub(crate) fn paletted(
    bank: u8,
    address: u16,
    position: (u16, u16),
    palette: u16,
    clip: Clip,
    game: &mut Game,
) {
    sprite_map(bank, address, game, &mut Native).draw(
        position,
        |tile| palette | tile & 0xf1ff,
        clip,
        game,
    );
}
#[forbid(unsafe_code)]
pub(crate) fn menu(index: u16, x: u16, y: u16, palette: u16, game: &mut Game) {
    let address = crate::rtl::rom_table_word!(game, 0x82c569, index as isize);
    paletted(0x82, address, (x, y), palette, Clip::None, game);
}
/// Advance each file helmet immediately before drawing it; idle menus clear each timer first.
#[forbid(unsafe_code)]
pub(crate) fn file_helmets(animate: bool, game: &mut Game) {
    const SPRITES: [u16; 9] = [0x2c, 0x2d, 0x2e, 0x2f, 0x30, 0x31, 0x32, 0x33, 0x33];
    for slot in 2..5 {
        let timer = 0x198d + 2 * slot;
        if !animate {
            game.g_ram[timer..timer + 2].fill(0);
        }
        let remaining = u16::from_ne_bytes([game.g_ram[timer], game.g_ram[timer + 1]]);
        if remaining != 0 {
            let remaining = remaining.wrapping_sub(1);
            game.g_ram[timer..timer + 2].copy_from_slice(&remaining.to_ne_bytes());
            if remaining == 0 {
                game.g_ram[timer..timer + 2].copy_from_slice(&8u16.to_ne_bytes());
                let mut frame = word!(game.eproj_id[slot]).wrapping_add(1);
                if i32::from(word!(game.eproj_id[slot])) - 7 & 0x8000 == 0 {
                    game.g_ram[timer..timer + 2].fill(0);
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
pub(crate) fn file_menu_cursor(game: &mut Game) {
    const POSITIONS: [u16; 12] = [
        0x30, 0xe, 0x58, 0xe, 0x80, 0xe, 0xa3, 0xe, 0xbb, 0xe, 0xd3, 0xe,
    ];
    let index = usize::from(word!(game.selected_save_slot).wrapping_mul(4) >> 1);
    word!(game.eproj_id[10] = POSITIONS[index]);
    word!(game.eproj_id[5] = POSITIONS[index + 1]);
}

#[forbid(unsafe_code)]
pub(crate) fn file_slot_cursor(exit_y: u16, game: &mut Game) {
    let positions = [72, 104, 136, exit_y];
    word!(game.eproj_id[10] = positions[usize::from(word!(game.eproj_id[15]))]);
    word!(game.eproj_id[5] = 22);
}

#[forbid(unsafe_code)]
pub(crate) fn beam(index: u16, position: (u16, u16), game: &mut Game) {
    let address = crate::rtl::rom_table_word!(game, 0x93a1a1, index as isize);
    sprite_map(0x93, address, game, &mut Native).draw(position, |tile| tile, Clip::None, game);
}
#[forbid(unsafe_code)]
pub(crate) fn with_base_tile(
    bank: u8,
    address: u16,
    position: (u16, u16),
    palette: u16,
    base_tile: u16,
    clip: Clip,
    game: &mut Game,
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
pub(crate) fn menu_selection(game: &mut Game) {
    const SPRITES: [u16; 4] = [0x37, 0x36, 0x35, 0x34];
    if word!(game.cinematic_var5) != 0 {
        if game_word!(game.cinematic_var5 -= 1, reread) == 0 {
            word!(game.eproj_id[0] = (u16::from(game.g_ram[0x1997]) + 1) & 3);
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
pub(crate) fn file_copy_arrow(game: &mut Game) {
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
    let image = game.capture_native_rom();
    let base = crate::rtl::lorom_offset(0x82bb0c) as usize;
    menu(
        image.table_word(base, 3 * v0 as isize),
        image.table_word(base, 3 * v0 as isize + 1),
        image.table_word(base, 3 * v0 as isize + 2),
        512,
        game,
    );
}
