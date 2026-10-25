//! Scrolling sky setup, parallax, HDMA bands and tile uploads.
use crate::game::liquid_effects::WATER_WAVE;
use crate::game::prelude::*;
use crate::game::visual_runtime::spawn_hdma;
crate::ram_map::ram_pointer_functions! { gVramWriteEntry(a: i32) -> VramWriteEntry = [(0xd0 + a) as isize]; }
crate::typed_struct! {
    #[derive(Copy, Clone)]
    #[repr(C, packed)]
    pub struct HdmaScrollEntry {
        pub top_pos, scroll_subspeed, scroll_speed: u16;
        pub hdma_data_table_entry: VoidP;
    }
}

pub unsafe fn initialize_banded_sky(game: &mut Game) {
    unsafe {
        word! {game; fx_y_pos = 1248; fx_type = 6; }
        static unk_88A62E: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0x2, 0x12, 0xad63 as VoidP);
        static unk_88A636: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0x42, 0x11, 0xad4e as VoidP);
        static unk_88A63E: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0x42, 0xd, 0xad39 as VoidP);
        spawn_hdma(0x88, &unk_88A62E, game);
        spawn_hdma(0x88, &unk_88A636, game);
        spawn_hdma(0x88, &unk_88A63E, game);
        update_vertical_scroll(0, game);
    }
}
unsafe fn build_vertical_bands(game: &mut Game) -> u16 {
    unsafe {
        word!(game.g_word_7E0598 = 32);
        *(ram_ptr!(game, u16; 0x9c00)) = 31;
        *(ram_ptr!(game, u16; 0x9c00, 1)) = 0;
        let mut row = 3u16;
        let mut y = word!(game.layer1_y_pos).wrapping_add(32);
        let mut band = 0usize;
        let mut inset = y & if y.wrapping_sub(1248) & 0x8000 != 0 {
            15
        } else {
            31
        };
        loop {
            while (y as i16)
                .wrapping_sub(*(rom_ptr!(game, u16; 0x88a8e8)).offset((band * 3) as isize) as i16)
                < 0
                || (y as i16).wrapping_sub(
                    *(rom_ptr!(game, u16; 0x88a8ee)).offset((band * 3) as isize) as i16,
                ) >= 0
            {
                band += 1;
                if band >= 13 {
                    return row;
                }
            }
            *(ram_ptr!(game, u16; 0x9c00, row as isize)) =
                (*(rom_ptr!(game, u16; 0x88a8ea)).offset((band * 3) as isize)).wrapping_sub(inset);
            *(ram_ptr!(game, u16; 0x9c00, row as isize + 1)) = (*(rom_ptr!(game, u16; 0x88a8ec))
                .offset((band * 3) as isize))
            .wrapping_add(inset)
            .wrapping_sub(word!(game.g_word_7E0598));
            let height = *ram_ptr!(game; 0x9c00, row as isize) as u16;
            y = y.wrapping_add(height);
            row = row.wrapping_add(3);
            word!(game.g_word_7E0598 += height);
            if word!(game.g_word_7E0598).wrapping_sub(224) & 0x8000 == 0 {
                return row;
            }
            inset = 0;
        }
    }
}
pub unsafe fn update_vertical_scroll(mut k: u16, game: &mut Game) {
    unsafe {
        damage_above_screen(game);
        k = build_vertical_bands(game);
        *(ram_ptr!(game, u16; 0x9c00, k as isize)) = 0;
    }
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, hdp: *const u8, game: &mut Game) -> *const u8;
    start_wave [hdp] { word!(game.hdma_object_B[(k >> 1)] = 1); };
}
pub unsafe fn update_horizontal_scroll(k: u16, game: &mut Game) {
    unsafe {
        if word!(game.layer1_y_pos) as i32 - 1024i32 & 0x8000 != 0 {
            if (*ram_ptr!(game; 0x5b5) & 1) == 0 {
                word!(game.g_word_7E0596 = (word!(game.g_word_7E0596) as i32 + 2 & 0x1e) as u16);
            }
            let mut v1: u16 = word!(game.g_word_7E0596);
            let mut v2: u16 = 0;
            do_until! { ((v1) as i16).wrapping_sub(32) >= 0 ;
                word!(
                    game.g_word_7E9E80[(v2 >> 1)] =
                        crate::rtl::rom_table_word!(game, 0x88a938, (v1 >> 1) as isize)
                );
                v2 = (v2).wrapping_add(2);
                v1 = (v1).wrapping_add(2);
            }
        } else {
            let v3: i32 = k as i32 >> 1;
            crate::update_place! { unaligned_word fresh104 [fresh105] = &raw mut (*(ram_ptr!(game, u16; 0x1920)).offset(v3 as isize)) => (u16::from_le(fresh104.read_unaligned())).wrapping_sub(1); }
            if fresh105 == 1 {
                word! {game; hdma_object_B[v3] = 6;
                hdma_object_A[v3] = (word!(game.hdma_object_A[v3]) as i32 + 2 & 0x1f) as u16; }
            }
            let mut v5: u16 = word!(game.hdma_object_A[v3]);
            for i in (0i32..=30).rev().step_by(2) {
                word!(
                    game.g_word_7E9E80[(v5 >> 1)] = ((WATER_WAVE[(i >> 1) as usize]) as u16)
                        .wrapping_add(word!(game.reg_BG1HOFS))
                );
                v5 = (v5 as i32 - 2i32 & 0x1f) as u16;
            }
        }
        write_scroll_pair(0, 0, game);
        write_scroll_pair(2, 31, game);
        let mut r18: u16 = 31;
        write_scroll_pair(3, -25088i32 as u16, game);
        if word!(game.layer1_y_pos) as i32 - 1024i32 & 0x8000 != 0 {
            *(ram_ptr!(game, u16; 0x9f00, 2)) = 0;
        } else {
            r18 = append_upper_band(0x105, r18, 177, 0, game);
        }
        let mut v7: u16 = 5;
        r18 = append_upper_band(5, r18, -24960i32 as u16, 128, game);
        let mut v8: i16 = (224i16).wrapping_sub((r18) as i16);
        while (v8).wrapping_sub(16) >= 0 {
            v8 = (v8).wrapping_sub(16);
            write_scroll_pair(v7 as usize, 144, game);
            write_scroll_pair(v7 as usize + 1, -24960i32 as u16, game);
            v7 = (v7).wrapping_add(3);
        }
        let mut v9: i16 = v8;
        if v8 != 0 {
            v9 = (v8).wrapping_add(128);
        }
        write_scroll_pair(v7 as usize, v9 as u16, game);
        write_scroll_pair(v7 as usize + 1, -24960i32 as u16, game);
        write_scroll_pair(v7 as usize + 3, 0, game);
    }
}
unsafe fn append_upper_band(k: u16, mut r18: u16, r20: u16, r22: u16, game: &mut Game) -> u16 {
    unsafe {
        let mut v0: u16 = k;
        let mut v1: u16 = (1216u16).wrapping_sub(word!(game.layer1_y_pos));
        let mut r24: u16 = (1216u16).wrapping_sub(word!(game.layer1_y_pos));
        if (1216i16).wrapping_sub((word!(game.layer1_y_pos)) as i16) >= 0
            && word!(game.layer1_y_pos) != 1216
        {
            if v1 as i32 - 128i32 & 0x8000 != 0 {
                r18 = (1216u16).wrapping_sub(word!(game.layer1_y_pos));
            } else {
                r18 = (1216u16).wrapping_sub(word!(game.layer1_y_pos));
                if v1 as i32 - 193i32 & 0x8000 == 0 {
                    v1 = 193;
                }
                r18 = v1;
                r24 = v1;
                loop {
                    let mut v2: i16 = ((v1) as i16).wrapping_sub(16);
                    if v2 < 0 {
                        break;
                    }
                    r24 = v2 as u16;
                    *(ram_ptr!(game, u16; 0x9e00, v0 as isize)) = (r22) | (0x10u16);
                    *(ram_ptr!(game, u16; 0x9e00, v0 as isize + 1)) = r20;
                    v1 = r24;
                    v0 = (v0).wrapping_add(3);
                }
            }
            *(ram_ptr!(game, u16; 0x9e00, v0 as isize)) = (r22) | (r24);
            *(ram_ptr!(game, u16; 0x9e00, v0 as isize + 1)) = r20;
        }
        r18
    }
}
static LAND_SCROLL: SpawnHdmaObject_Args = SpawnHdmaObject_Args::new(0x42, 0xf, 0xad76 as VoidP);
static OCEAN_SCROLL: SpawnHdmaObject_Args = SpawnHdmaObject_Args::new(0x42, 0xf, 0xad89 as VoidP);
pub unsafe fn initialize_land(game: &mut Game) {
    unsafe {
        crate::update_place! { reference fresh47 = *ram_ptr!(game; 0x91b) => (*fresh47) | (1u8); }
        crate::update_place! { reference fresh48 = *ram_ptr!(game; 0x91c) => (*fresh48) | (1u8); }
        spawn_hdma(0x88, &LAND_SCROLL, game);
        word! {game; set_to_e0_by_scrolling_sky = 224; UNUSED_word_7E059C = 0; }
    }
}
pub unsafe fn initialize_ocean(game: &mut Game) {
    unsafe {
        *ram_ptr!(game; 0x59) = 74;
        spawn_hdma(0x88, &OCEAN_SCROLL, game);
        word! {game; set_to_e0_by_scrolling_sky = 224; UNUSED_word_7E059C = 0; }
    }
}
unsafe fn damage_above_screen(game: &mut Game) {
    {
        if ((word!(game.samus_y_pos)) as i16).wrapping_sub((word!(game.samus_y_radius)) as i16) < 0
            || word!(game.samus_y_pos) as i32 - word!(game.samus_y_radius) as i32 - 17i32 & 0x8000
                != 0
        {
            word!(game.samus_periodic_damage = 8);
        }
    }
}
pub unsafe fn advance_parallax(k: u16, game: &mut Game) {
    unsafe {
        let mut r24: u16 = 0;
        *ram_ptr!(game; 0x59) = 74;
        for v1 in 0u16..23 {
            let entry = (rom_ptr!(game, HdmaScrollEntry; 0x88aec1)).offset(v1 as isize);
            let v3 = ram_ptr!(game, u16; (*entry).hdma_data_table_entry as isize);
            AddToHiLo(
                v3.offset(1),
                v3,
                words!(((*entry).scroll_speed, (*entry).scroll_subspeed)),
            );
        }
        word! { game, 0; scrolling_sky_bg2_hdma_data[44], scrolling_sky_bg2_hdma_data[45] }
        *(ram_ptr!(game, u16; 0x9f00)) = 31;
        *(ram_ptr!(game, u16; 0x9f00, 1)) = 1438;
        let mut r18: u16 = (word!(game.layer1_y_pos)).wrapping_add(32);
        let r20: u16 = (word!(game.layer1_y_pos)).wrapping_add(224);
        let mut v6: u16 = 3;
        for v5 in 0u16..23 {
            let entry = (rom_ptr!(game, HdmaScrollEntry; 0x88aec1)).offset(v5 as isize);
            while ((r18) as i16).wrapping_sub(((*entry).top_pos) as i16) >= 0
                && ((r18) as i16).wrapping_sub(((*entry.offset(1)).top_pos) as i16) < 0
            {
                let mut v8: u16 = ((*entry.offset(1)).top_pos).wrapping_sub(r18);
                r24 = v8;
                if v8 as i32 - 128i32 & 0x8000 == 0 {
                    *(ram_ptr!(game, u16; 0x9f00, v6 as isize)) = 127;
                    *(ram_ptr!(game, u16; 0x9f00, (v6 as i32 + 1) as isize)) =
                        ((*entry).hdma_data_table_entry).wrapping_add(2);
                    v6 = (v6).wrapping_add(3);
                    v8 = (r24).wrapping_sub(127);
                }
                *(ram_ptr!(game, u16; 0x9f00, v6 as isize)) = v8;
                *(ram_ptr!(game, u16; 0x9f00, (v6 as i32 + 1) as isize)) =
                    ((*entry).hdma_data_table_entry).wrapping_add(2);
                r18 = (r18).wrapping_add(r24);
                v6 = (v6).wrapping_add(3);
                if r18 as i32 - r20 as i32 & 0x8000 == 0 {
                    *(ram_ptr!(game, u16; 0x9f00, (v6 as i32 + 3) as isize)) = 0;
                    return;
                }
            }
        }
        let mut i: u16 = (1535u16).wrapping_sub(r18);
        loop {
            r24 = i;
            if i as i32 - 128i32 & 0x8000 != 0 {
                break;
            }
            *(ram_ptr!(game, u16; 0x9f00, v6 as isize)) = 127;
            *(ram_ptr!(game, u16; 0x9f00, (v6 as i32 + 1) as isize)) = 181;
            v6 = (v6).wrapping_add(3);
            i = (r24).wrapping_sub(127);
        }
        *(ram_ptr!(game, u16; 0x9f00, v6 as isize)) = i;
        *(ram_ptr!(game, u16; 0x9f00, (v6 as i32 + 1) as isize)) = 181;
        *(ram_ptr!(game, u16; 0x9f00, (v6 as i32 + 3) as isize)) = 0;
    }
}
pub unsafe fn update_parallax(k: u16, game: &mut Game) {
    unsafe {
        if word!(game.time_is_frozen_flag) == 0 {
            advance_parallax(k, game);
        }
    }
}
unsafe fn queue_tile_rows(src: *const u16, game: &mut Game) {
    unsafe {
        if word!(game.time_is_frozen_flag) != 0 {
            *(ram_ptr!(game, u16; 0x9f00, 0)) = 0;
        } else {
            word!(game.reg_BG2VOFS = word!(game.layer1_y_pos));
            let v0: u16 = word!(game.vram_write_queue_tail);
            let v1 = gVramWriteEntry(word!(game.vram_write_queue_tail) as i32, game);
            for row in 0..4 {
                (*v1.offset(row)).size = 64;
            }
            let mut tt: i32 = (word!(game.layer1_y_pos) as i32 & 0x7f8) - 16i32;
            if tt & 0x8000 != 0 {
                tt = 0;
            }
            let v2: i32 = 8 * tt as u8 as i32;
            let y1: i32 = (tt >> 8) as u8 as i32;
            let y2: i32 = ((tt >> 8) + 1) as u8 as i32;
            let v3: VoidP = (*src.offset(y1 as isize) as i32 + v2) as VoidP;
            (*v1).src.addr = v3;
            (*v1.offset(1)).src.addr = (v3).wrapping_add(64);
            let v5: VoidP = (*src.offset(y2 as isize) as i32 + v2) as VoidP;
            (*v1.offset(2)).src.addr = v5;
            (*v1.offset(3)).src.addr = (v5).wrapping_add(64);
            for row in 0..4 {
                (*v1.offset(row)).src.bank = 0x8a;
            }
            let t: i32 = (*ram_ptr!(game; 0x59) as i32 & 0xfc) << 8;
            let v6: u16 = (t + 4 * (word!(game.layer1_y_pos) as i32 - 16i32 & 0x1f8)) as u16;
            (*v1).vram_dst = v6;
            (*v1.offset(1)).vram_dst = (v6).wrapping_add(32);
            let v7: u16 = (t + 4 * (word!(game.layer1_y_pos) as i32 + 240 & 0x1f8)) as u16;
            (*v1.offset(2)).vram_dst = v7;
            (*v1.offset(3)).vram_dst = (v7).wrapping_add(32);
            word!(game.vram_write_queue_tail = (v0).wrapping_add(28));
        };
    }
}
crate::forward_calls! { pub unsafe fn (game: &mut Game);
    queue_land_rows =>
        queue_tile_rows(rom_ptr!(game, u16; 0x88ad9c), game);
    queue_ocean_rows =>
        queue_tile_rows(rom_ptr!(game, u16; 0x88ada6), game);
}
// HDMA entries overlap word stores at byte offsets; retain channel write order.
unsafe fn write_scroll_pair(offset: usize, value: u16, game: &mut Game) {
    unsafe {
        *(ram_ptr!(game, u16; 0x9e00, offset as isize)) = value;
        *(ram_ptr!(game, u16; 0x9f00, offset as isize)) = value;
    }
}
