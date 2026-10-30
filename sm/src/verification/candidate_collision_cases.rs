//! Actual-old bounded terrain and power-bomb contact observations.
//! No raw unaligned operand accesses or intentional extension cycles.
use super::super::collision::{
    EprojPowerBombCollision, move_horizontally_with_block_collision,
    move_vertically_with_block_collision,
};
use super::{bytes, put, slot};
use crate::game_state::Game;
use crate::ram_map::{word_arrays as a, words as w};
use crate::rom::RomImage;
pub(super) const CASES: u32 = 196;

fn projectile(game: &mut Game, handle: u16, radius: u16, negative: bool) {
    for (field, value) in [
        (a::eproj_x_pos, 136),
        (a::eproj_y_pos, 136),
        (a::eproj_x_subpos, 0xffff),
        (a::eproj_y_subpos, 1),
        (a::eproj_x_vel, if negative { 0xfeff } else { 0x101 }),
        (a::eproj_y_vel, if negative { 0xfeff } else { 0x101 }),
        (a::eproj_radius, radius),
    ] {
        slot(game, field, handle, value);
    }
}
fn room(game: &mut Game, kind: u16, bts: u8) {
    put(game, w::room_width_in_blocks, 32);
    for block in 0..1024 {
        put(game, 0x10002 + block * 2, kind << 12);
        game.ram().write_byte(0x16402 + block, bts);
    }
}
unsafe fn move_one(horizontal: bool, handle: u16, game: &mut Game) -> Option<usize> {
    Some(usize::from({
        super::super::with_execution(game, |ctx| {
            if horizontal {
                move_horizontally_with_block_collision(handle, ctx)
            } else {
                move_vertically_with_block_collision(handle, ctx)
            }
        })
    }))
}
pub(super) unsafe fn execute(id: u32, game: &mut Game, image: &RomImage) -> Option<usize> {
    assert!(id < CASES);
    let n = id as usize;
    if n < 32 {
        // Both axes/signs, empty/solid, canonical/odd/broad handles, fraction carry.
        let handle = [0, 1, 34, 0xffff][n % 4];
        let horizontal = n & 4 == 0;
        let negative = n & 8 != 0;
        room(game, if n & 16 == 0 { 0 } else { 8 }, 0);
        projectile(game, handle, 0x0303, negative);
        put(game, w::eproj_index, handle);
        return unsafe { move_one(horizontal, handle, game) };
    }
    if n < 112 {
        // All five square-slope kinds and four orientations, both axes,
        // single/multiple-block spans, selected slot distinct from input slot.
        let n = n - 32;
        let horizontal = n % 2 == 0;
        let kind = (n / 2 % 5) as u8;
        let orientation = n / 10 % 4;
        let multiple = n / 40 != 0;
        let handle = [0, 1, 34, 0xffff][orientation];
        let selected = if handle == 34 { 0 } else { 34 };
        let radius = if multiple { 0x0909 } else { 0x0303 };
        room(game, 1, kind | ((orientation as u8) << 6));
        projectile(game, handle, radius, orientation & 1 != 0);
        projectile(game, selected, radius, orientation & 1 != 0);
        put(game, w::eproj_index, selected);
        return unsafe { move_one(horizontal, handle, game) };
    }
    if n < 160 {
        // Curved slopes: flips, three heights, two spans and same/distinct selector.
        // Distinct raw and loaded allocations prove which ROM supplies heights.
        let n = n - 112;
        let orientation = n % 4;
        let height = [0u8, 8, 31][n / 4 % 3];
        let radius = if n / 12 % 2 == 0 { 0x0303 } else { 0x0909 };
        let handle = if n & 1 == 0 { 0 } else { 1 };
        let selected = if n / 24 == 0 { handle } else { 34 };
        room(game, 1, 5 | ((orientation as u8) << 6));
        projectile(game, handle, radius, orientation & 2 != 0);
        projectile(game, selected, radius, orientation & 2 != 0);
        put(game, w::eproj_index, selected);
        let raw = RomImage::zeroed(image.len());
        let table = crate::rtl::lorom_offset(0x948b2b) as usize;
        for index in 0..512 {
            bytes(&raw, table + index, &[height]);
            bytes(image, table + index, &[31 - height]);
        }
        game.register_native_rom(raw.clone());
        let result = unsafe { move_one(false, handle, game) };
        game.register_native_rom(image.clone());
        return result;
    }
    if n < 172 {
        // Finite extension chains, signed BTS, zero redirects, low-byte/full-width
        // room arithmetic; all resulting cells are in the explicitly seeded room.
        let n = n - 160;
        let horizontal = n & 1 == 0;
        let variant = n / 2;
        room(game, 8, 0);
        projectile(game, 0, 0x0303, false);
        put(game, w::eproj_index, 0);
        let width = match variant {
            4 => 0,
            5 => 256,
            _ => 32,
        };
        put(game, w::room_width_in_blocks, width);
        let first = if width == 32 { 264 } else { 8 };
        let kind = if variant == 1 || variant >= 4 { 13 } else { 5 };
        put(game, 0x10002 + first * 2, kind << 12);
        game.ram().write_byte(
            0x16402 + first,
            match variant {
                1 => 0xff,
                2 | 4 => 0,
                _ => 1,
            },
        );
        if variant == 3 {
            // Horizontal extension followed by a vertical extension and a solid cell.
            put(game, 0x10002 + (first + 1) * 2, 13 << 12);
            game.ram().write_byte(0x16402 + first + 1, 1);
        }
        return unsafe { move_one(horizontal, 0, game) };
    }
    if n < 180 {
        // Curved slope ignored horizontally, or facing away from vertical motion.
        // A zero low-byte room width also exercises SNES modulus-by-zero behavior.
        let n = n - 172;
        let orientation = n / 2;
        let horizontal = n % 2 == 0;
        room(game, 1, 5 | ((orientation as u8) << 6));
        put(game, w::room_width_in_blocks, 0);
        projectile(game, 0, 0x0303, orientation & 2 == 0);
        put(game, w::eproj_index, 0);
        return unsafe { move_one(horizontal, 0, game) };
    }
    // Power-bomb: zero/odd/high radii, exact X/Y boundaries, signed 0x8000 distance.
    let n = n - 180;
    let handle = [0, 1, 34, 0xffff][n % 4];
    let radius = [0u16, 0x100, 0x300, 0xffff][n / 4];
    let horizontal_radius = radius >> 8;
    let vertical_radius = (horizontal_radius + (horizontal_radius & 1) + (radius >> 9)) >> 1;
    projectile(game, handle, 0x0303, false);
    slot(game, a::eproj_id, handle, 0xa135);
    put(game, w::samus_x_speed_divisor, 7);
    put(game, w::power_bomb_explosion_radius, radius);
    let (dx, dy) = match n % 4 {
        0 => (0, 0),
        1 => (horizontal_radius, 0),
        2 => (0, vertical_radius),
        _ => (0x8000, 0),
    };
    put(game, w::power_bomb_explosion_x_pos, 136u16.wrapping_add(dx));
    put(game, w::power_bomb_explosion_y_pos, 136u16.wrapping_add(dy));
    super::super::with_execution(game, |ctx| EprojPowerBombCollision(handle, ctx));
    None
}
