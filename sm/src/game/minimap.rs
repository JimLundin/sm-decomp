//! HUD minimap rendering, exploration marking and boss-room suppression.
use crate::game::prelude::*;
const RomPtr_90: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x90>;
const RomPtr_82: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x82>;
crate::typed_struct! {
    #[derive(Copy, Clone)]
    #[repr(C, packed)]
    struct BossMapTiles {
        pub boss_id_: u16;
        pub ptrs: VoidP;
    }
}
static kShr0x80: [u8; 8] = [0x80, 0x40, 0x20, 0x10, 8, 4, 2, 1];
static kShr0xFc00: [u16; 8] = [0xfc00, 0x7e00, 0x3f00, 0x1f80, 0xfc0, 0x7e0, 0x3f0, 0x1f8];
unsafe fn mark_explored(r18: u16, r24: u16, game: &mut Game) {
    let v0: u8 = (((r18) & (0xff00u16)) >> 8) as u8;
    let R34: u16 = (word!(game.room_x_coordinate_on_map) as i32 + v0 as i32 & 0x20) as u16;
    let r20: u16 = ((word!(game.room_x_coordinate_on_map) as i32 + v0 as i32 & 0x1f) >> 3) as u16;
    let R22: u16 =
        (word!(game.room_y_coordinate_on_map) as i32 + ((r24 as i32 & 0xff00) >> 8) + 1) as u16;
    crate::update_place! { reference fresh13 = *ram_ptr!(game; 0x7f7, (r20 as i32 + 4 * (R34 as i32 + R22 as i32)) as u16 as isize) => (*fresh13)
    | (kShr0x80
        [(word!(game.room_x_coordinate_on_map) as i32 + v0 as i32 & 7) as usize]); }
}
pub unsafe fn disable_for_boss(game: &mut Game) {
    word!(game.debug_disable_minimap = 1);
    word_transfers!(game, ascending, 5;
        le(hud_tilemap, 26) <- value(11295),
        le(hud_tilemap, 58) <- value(11295),
        le(hud_tilemap, 90) <- value(11295),
    );
    let mut v2: u16 = 5;
    while word!(game.boss_id)
        != (*(rom_ptr!(game, BossMapTiles; 0x90a83a)).offset(v2 as isize)).boss_id_
    {
        v2 = v2.wrapping_sub(1);
        if (v2 & 0x8000) != 0 {
            return;
        }
    }
    let mut i: i32 = (*(rom_ptr!(game, BossMapTiles; 0x90a83a)).offset(v2 as isize)).ptrs as i32;
    loop {
        let v4 = RomPtr_90(i as u16, game) as *const u16;
        if (*v4 & 0x8000) != 0 {
            break;
        }
        mark_explored(*v4, *v4.offset(1), game);
        i += 4;
    }
}
pub unsafe fn update(game: &mut Game) {
    if word!(game.debug_disable_minimap) != 0
        || word!(game.samus_x_pos) as i32 >> 4 >= word!(game.room_width_in_blocks) as i32
        || word!(game.samus_y_pos) as i32 >> 4 >= word!(game.room_height_in_blocks) as i32
    {
        return;
    }
    let x = word!(game.room_x_coordinate_on_map).wrapping_add(word!(game.samus_x_pos) >> 8);
    let y = word!(game.room_y_coordinate_on_map)
        .wrapping_add(word!(game.samus_y_pos) >> 8)
        .wrapping_add(1);
    let half = x & 0x20;
    let column = x & 0x1f;
    let bit = (x & 7) as usize;
    let byte_index = (column >> 3).wrapping_add(4u16.wrapping_mul(half.wrapping_add(y)));
    *ram_ptr!(game; 0x7f7, byte_index as isize) |= kShr0x80[bit];

    let crossed_byte = bit < 2;
    let shift = (bit + 6) & 7;
    let origin = byte_index.wrapping_sub(if crossed_byte { 5 } else { 4 });
    let mut explored = [0u16; 3];
    for row in 0..3 {
        explored[row] = kShr0xFc00[shift]
            & (*ram_ptr!(game, u16; 0x7f7, origin as isize + 4 * row as isize)).swap_bytes();
    }
    let map = RomPtr_82(
        crate::rtl::rom_table_word!(game, 0x829717, word!(game.area_index) as isize),
        game,
    );
    let mut mapped = [0u16; 3];
    for row in 0..3 {
        mapped[row] =
            (*(map.offset(origin as isize + 4 * row as isize) as *const u16)).swap_bytes();
    }
    if origin & 3 == 3 && (if crossed_byte { shift } else { bit }) >= 6 {
        let other_half = if half != 0 {
            origin.wrapping_sub(124)
        } else {
            origin.wrapping_add(125)
        } as u8;
        let lane = if half == 32 { 8 } else { 0 };
        for row in 0..3 {
            let offset = other_half as isize + 4 * row as isize;
            let explored_byte = *ram_ptr!(game; 0x7f7, offset) as u16;
            let mapped_byte = *map.offset(offset) as u16;
            mapped[row] = (mapped[row] & !(0xff << lane)) | (mapped_byte << lane);
            explored[row] = (explored[row] & !(0xff << lane)) | (explored_byte << lane);
        }
    }
    for row in 0..3 {
        explored[row] <<= shift;
        mapped[row] <<= shift;
    }

    let tile = column.wrapping_add(half.wrapping_add(y).swap_bytes() >> 3);
    let adjustment = if half != 0 && tile & 0x1f < 2 {
        1026
    } else {
        34
    };
    let mut source_offset = tile.wrapping_sub(adjustment).wrapping_mul(2);
    let tiles = RomPtr(
        Load24((rom_ptr!(game, LongPtr; 0x82964a)).offset(word!(game.area_index) as isize)),
        game,
    ) as *const u16;
    for column in 0..5 {
        for row in 0..3 {
            let source = tiles
                .offset(32 * row as isize)
                .offset((source_offset >> 1) as isize);
            let mut tile = 31;
            if mapped[row] & 0x8000 != 0 {
                tile = *source;
                if word!(game.has_area_map) == 0 {
                    tile = 31;
                }
            }
            mapped[row] = mapped[row].wrapping_mul(2);
            let destination = 26 + column + 32 * row;
            word!(game.hud_tilemap[destination] = tile & 0xc3ff | 0x2c00);
            let visited = explored[row] & 0x8000 != 0;
            explored[row] = explored[row].wrapping_mul(2);
            if visited {
                word!(game.hud_tilemap[destination] = *source & 0xc3ff | 0x2800);
                if row == 1 && column == 2 && word!(game.hud_tilemap[destination]) & 0x1ff == 40 {
                    // Keep the original write one map row before the current exploration byte.
                    *ram_ptr!(game; 0x7f3, byte_index as isize) |= kShr0x80[bit];
                }
            }
        }
        source_offset = source_offset.wrapping_add(2);
        if source_offset & 0x3f == 0 {
            source_offset = source_offset.wrapping_add(1984);
        }
    }
    if *ram_ptr!(game; 0x5b5) & 8 == 0 {
        word!(game.hud_tilemap[60] |= 0x1c00);
    }
}
