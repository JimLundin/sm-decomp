//! X-ray background capture, hidden-block overlay and restoration.
use crate::game::constants::bank_8f::*;
use crate::game::prelude::*;

use crate::game::sm_80::QueueSfx1_Max6;

use crate::game::sm_91::{__ASSERT_FUNCTION, ResponsibleForXrayStandupGlitch};

const RomPtr_8F: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x8f>;
const RomPtr_91: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x91>;
const true_0: i32 = 1;
const false_0: i32 = 0;
crate::typed_struct! {
    [
        #[derive(Copy, Clone)]
        #[repr(C, packed)]
    ];
    struct BlockDefinition {
        pub value: u16;
        pub addr: VoidP;
    }
    struct RoomBlock {
        pub x_block, y_block: u8;
        pub level_data_block: u16;
    }
}

unsafe fn upload_screen(source: u16, screen: u16, game: &mut Game) {
    unsafe {
        let tail = word!(game.vram_write_queue_tail);
        let entry = ram_ptr!(game; 0xd0, tail as isize);
        (entry as *mut u16).write_unaligned(2048);
        (entry.add(2) as *mut u16).write_unaligned(source);
        (entry.add(4) as *mut u16).write_unaligned(126);
        (entry.add(5) as *mut u16)
            .write_unaligned(((game.g_ram[0x59] as u16 & 0xfc) << 8).wrapping_add(screen * 1024));
        word!(game.vram_write_queue_tail = tail.wrapping_add(7));
    }
}

// Capture all four source tiles before the reverse-order writes, even when RAM aliases.
unsafe fn write_block(destination: u16, tile: u16, offsets: [usize; 4], game: &mut Game) {
    unsafe {
        let source = (*(ram_ptr!(game, TileTables; 0xa000))).tables[tile as usize];
        let tiles = [
            source.top_left,
            source.top_right,
            source.bottom_left,
            source.bottom_right,
        ];
        for corner in (0..4).rev() {
            ram_ptr!(game, u16; 0x4000, destination as isize + 2 * offsets[corner] as isize)
                .write_unaligned(tiles[corner]);
        }
    }
}

unsafe fn copy_background(game: &mut Game) {
    unsafe {
        let mut origin = (4
            * ((word!(game.layer1_y_pos) as u32 + word!(game.bg1_y_offset) as u32) & 0xf0)
            + (((word!(game.layer1_x_pos) as u32 + word!(game.bg1_x_offset) as u32) & 0xf0) >> 3)
            + 4 * ((word!(game.layer1_x_pos) as u32 + word!(game.bg1_x_offset) as u32) & 0x100))
            as u16;
        let corners = [0usize, 1, 32, 33];
        for row in 0..16 {
            let mut screen_row = origin & 0x7e0;
            let mut column = origin & 0x1f;
            for block in 0..17 {
                if column >= 32 {
                    screen_row = (screen_row + 1024) & 0x7e0;
                    column = 0;
                }
                let source = usize::from(screen_row + column) + 4096;
                let mut tiles = [0u16; 4];
                for corner in 0..4 {
                    tiles[corner] = (*(ram_ptr!(game, Ram4000; 0x4000))).xray_tilemaps
                        [source + corners[corner]];
                }
                let destination = row * 64 + if block < 16 { block * 2 } else { 1024 };
                for corner in (0..4).rev() {
                    ram_ptr!(game, u16; 0x4000, (2 * (destination + corners[corner])) as isize)
                        .write_unaligned(tiles[corner]);
                }
                column += 2;
            }
            origin = (origin & 0x400) + (origin.wrapping_add(64) & 0x3ff);
        }
    }
}

pub unsafe fn freeze_and_backup(k: u16, game: &mut Game) {
    unsafe {
        byte!(@pointer ram_ptr!(game, u16; 0xa78); 0) = 1;
        byte!(@pointer ram_ptr!(game, u16; 0x1914); k as u8 as i32 as isize) =
            word!(game.reg_BG2HOFS) as u8;
        *ram_ptr!(game; 0x1914, k as u8 as i32 as isize, 1) =
            byte!(@pointer ram_ptr!(game, u16; 0xb5); 1);
        byte!(@pointer ram_ptr!(game, u16; 0x1920); k as u8 as i32 as isize) =
            word!(game.reg_BG2VOFS) as u8;
        *ram_ptr!(game; 0x1920, k as u8 as i32 as isize, 1) =
            byte!(@pointer ram_ptr!(game, u16; 0xb7); 1);
        byte!(@pointer ram_ptr!(game, u16; 0x192c); k as u8 as i32 as isize) =
            *ram_ptr!(game; 0x59);
    }
}
#[forbid(unsafe_code)]
pub fn capture_bg1_right(game: &mut Game) {
    crate::game::vram_read_queue::enqueue(0x58, 1024, 0x6800, 2048, false, game);
}
#[forbid(unsafe_code)]
pub fn capture_bg1_left(game: &mut Game) {
    crate::game::vram_read_queue::enqueue(0x58, 0, 0x6000, 2048, false, game);
}
pub unsafe fn build_overlay(game: &mut Game) {
    unsafe {
        copy_background(game);
        crate::native_assertion! {
            '_c2rust_label: ((word!(game.layer1_y_pos) as i32 >> 12) as u16) == 0;
            b"(uint16)(layer1_y_pos >> 12) == 0\0", b"src/sm_91.c\0", 921,
            pointer __ASSERT_FUNCTION.as_ptr()
        };
        let mut r34: u16 = (Mult8x8(
            (word!(game.layer1_y_pos) >> 4) as u8,
            word!(game.room_width_in_blocks) as u8,
        ) as i32
            + (word!(game.layer1_x_pos) as i32 >> 4)) as u16;
        let mut r22: u16 = 0;
        let mut m = 16;
        do_until! { (m == 0);
            reveal_left_neighbor(r22, r34, game);
            let mut r36: u16 = r34;
            for n_0 in (1i32..=16).rev() {
                reveal_block(n_0 as u16, r22, r36, game);
                r22 = (r22).wrapping_add(4);
                r36 = (r36).wrapping_add(1);
            }
            crate::update_place! { previous v12: u16 = r22 => (r22).wrapping_add(1984); }
            reveal_block(0, r22, r36, game);
            r22 = (v12).wrapping_add(64);
            r34 = (r34).wrapping_add(word!(game.room_width_in_blocks));
            m -= 1;
        }
        reveal_room_items(game);
        crate::game::vram_read_queue::enqueue(0x59, 0, 0x5000, 2048, false, game);
    }
}

unsafe fn block_definition(k: u16, game: &mut Game) -> *const u8 {
    unsafe {
        let bts: u16 = *ram_ptr!(game; 0x16402, k as isize) as u16;
        let r40: u16 = (word!(game.level_data[k])) & (0xf000u16);
        let mut i: i32 = 0;
        loop {
            let mut value: u16 =
                (*(rom_ptr!(game, BlockDefinition; 0x91d2d6)).offset(i as isize)).value;
            if value == 0xffff {
                break;
            }
            if value == r40 {
                let mut p = RomPtr_91(
                    (*(rom_ptr!(game, BlockDefinition; 0x91d2d6)).offset(i as isize)).addr,
                    game,
                );
                loop {
                    value = *(p as *mut u16);
                    if value == 0xffff {
                        break;
                    }
                    if value == 0xff00 || value == bts {
                        return RomPtr_91(*(p.offset(2) as *mut u16), game);
                    }
                    p = p.offset(4);
                }
                return null::<u8>();
            }
            i += 1;
        }
        null::<u8>()
    }
}
pub unsafe fn reveal_left_neighbor(dst_r22: u16, r34: u16, game: &mut Game) {
    unsafe {
        if r34 == 0 {
            return;
        }
        let jp = block_definition((r34).wrapping_sub(1), game);
        let dst = (&raw mut (*(ram_ptr!(game, Ram4000; 0x4000))).xray_tilemaps as *mut u16
            as *mut u8)
            .offset(dst_r22 as i32 as isize) as *mut u16;
        if jp.is_null() {
            return;
        }
        if *(jp as *mut u16) as i32 != 0x91cf4e & 0xffff {
            if *(jp as *mut u16) as i32 != 0x91cf6f & 0xffff {
                return;
            }
            let src = (&raw mut (*(ram_ptr!(game, TileTables; 0xa000))).tables as *mut TileTable)
                .offset(*(jp.offset(8) as *mut u16) as i32 as isize);
            *dst.offset(64) = (*src).top_left;
            *dst.offset(65) = (*src).top_right;
            *dst.offset(96) = (*src).bottom_left;
            *dst.offset(97) = (*src).bottom_right;
        }
        let src_0 = (&raw mut (*(ram_ptr!(game, TileTables; 0xa000))).tables as *mut TileTable)
            .offset(*(jp.offset(4) as *mut u16) as i32 as isize);
        *dst = (*src_0).top_left;
        *dst.offset(1) = (*src_0).top_right;
        *dst.offset(32) = (*src_0).bottom_left;
        *dst.offset(33) = (*src_0).bottom_right;
    }
}
unsafe fn follow_extension(dst_r22: u16, r36: u16, which_dir: bool, game: &mut Game) {
    unsafe {
        let mut y: u16 = (r36 as i32 / word!(game.room_width_in_blocks) as i32) as u16;
        let mut x: u16 = (r36 as i32 % word!(game.room_width_in_blocks) as i32) as u16;
        let mut r48: u16 = (word!(game.level_data[r36])) & (0xf000u16);
        let mut step: i32 = *ram_ptr!(game; 0x16402, r36 as isize) as i8 as i32;
        if step != 0 {
            let mut variable = if which_dir { &raw mut y } else { &raw mut x };
            loop {
                if ((*variable as i32 + step) as i16) < 0 {
                    write_block(dst_r22, 0xff, [0, 1, 32, 33], game);
                    return;
                }
                *variable = (*variable as i32 + step) as u16;
                let pos: i32 = y as i32 * word!(game.room_width_in_blocks) as i32 + x as i32;
                step = *ram_ptr!(game; 0x16402, pos as isize) as i8 as i32;
                r48 = (word!(game.level_data[pos])) & (0xf000u16);
                if r48 == 0xd000 {
                    variable = &raw mut y;
                } else {
                    if r48 != 0x5000 {
                        break;
                    }
                    if step < 0 {
                        variable = &raw mut x;
                    }
                }
            }
        }
        if (*(rom_ptr!(game, BlockDefinition; 0x91d2d6)).offset(1)).value == r48 {
            let mut t = RomPtr_91(
                (*(rom_ptr!(game, BlockDefinition; 0x91d2d6)).offset(1)).addr,
                game,
            ) as *mut u16;
            while *t != 0xffff {
                if *t == 0xff00 || *t as i32 == step & 0xff {
                    let tile = *(RomPtr_91((*t.offset(1)).wrapping_add(2), game) as *const u16);
                    write_block(dst_r22, tile, [0, 1, 32, 33], game);
                    return;
                }
                t = t.offset(2);
            }
        }
    }
}

unsafe fn dispatch_reveal(
    ea: u32,
    jp: *const u8,
    columns_left: u16,
    destination: u16,
    block: u16,
    game: &mut Game,
) {
    unsafe {
        let (width, height, repeat_tile) = match ea {
            0x91ce79 => {
                follow_extension(destination, block, true, game);
                return;
            }
            0x91cebb => {
                follow_extension(destination, block, false, game);
                return;
            }
            0x91cf3e if word!(game.area_index) != 1 => return,
            0x91cf36 | 0x91cf3e => (1, 1, false),
            0x91cf4e => (2, 1, false),
            0x91cf62 => (1, 2, true),
            0x91cf6f => (2, 2, false),
            _ => {
                Unreachable(game);
                return;
            }
        };
        for row in 0..height {
            for column in 0..width {
                if column == 1 && columns_left == 1 {
                    continue;
                }
                let descriptor = if repeat_tile {
                    2
                } else {
                    2 + 2 * (row * width + column)
                };
                let tile = *(jp.add(descriptor) as *const u16);
                let base = row * 64 + column * 2;
                write_block(
                    destination,
                    tile,
                    [base, base + 1, base + 32, base + 33],
                    game,
                );
            }
        }
    }
}

pub unsafe fn reveal_block(r18: u16, r22: u16, r36: u16, game: &mut Game) {
    unsafe {
        let jp = block_definition(r36, game);
        if !jp.is_null() {
            dispatch_reveal(
                ((*(jp as *mut u16)) as u32) | (0x910000u32),
                jp,
                r18,
                r22,
                r36,
                game,
            );
        }
    }
}

pub unsafe fn reveal_room_block(a: u16, k: u16, j: u16, game: &mut Game) {
    unsafe {
        let R24: u16 = (k as i32 - (word!(game.layer1_x_pos) as i32 >> 4)) as u16;
        let R26: u16 = (j as i32 - (word!(game.layer1_y_pos) as i32 >> 4)) as u16;
        if R24 & 0x8000 == 0
            && R24 as i32 - 16i32 & 0x8000 != 0
            && R26 & 0x8000 == 0
            && R26 as i32 - 16i32 & 0x8000 != 0
        {
            let R22: u16 = (4 * (R24 as i32 + 32 * R26 as i32)) as u16;
            if (a & 0x800) != 0 {
                write_block(R22, a & 0x3ff, [32, 33, 0, 1], game);
            } else {
                write_block(R22, a & 0x3ff, [0, 1, 32, 33], game);
            }
        }
    }
}
pub unsafe fn capture_bg2_right(game: &mut Game) {
    unsafe {
        if can_reveal_blocks(game) {
            if word!(game.earthquake_timer) != 0 {
                word! { game;
                    reg_BG1HOFS = (word!(game.layer1_x_pos)).wrapping_add(word!(game.bg1_x_offset));
                    reg_BG1VOFS = (word!(game.layer1_y_pos)).wrapping_add(word!(game.bg1_y_offset));
                }
            }
            word!(game.reg_BG2HOFS = (word!(game.reg_BG1HOFS)) & (0xfu16));
            word!(game.reg_BG2VOFS = (word!(game.reg_BG1VOFS)) & (0xfu16));
            *ram_ptr!(game; 0x59) = 73;
        }
        crate::game::vram_read_queue::enqueue(0x59, 1024, 0x5800, 2048, true, game);
    }
}
pub unsafe fn can_reveal_blocks(game: &mut Game) -> bool {
    unsafe {
        if word!(game.room_ptr) as i32 == addr_kRoom_a66a as i32
            || word!(game.room_ptr) as i32 == addr_kRoom_cefb as i32
        {
            return false_0 != 0;
        }
        if word!(game.fx_type) == 0x24 {
            return false_0 != 0;
        }
        if word!(game.boss_id) == 3
            || word!(game.boss_id) == 6
            || word!(game.boss_id) == 7
            || word!(game.boss_id) == 8
            || word!(game.boss_id) == 10
        {
            return false_0 != 0;
        }
        true_0 != 0
    }
}
pub unsafe fn upload_left(game: &mut Game) {
    unsafe {
        if can_reveal_blocks(game) {
            upload_screen(0x4000, 0, game);
        }
    }
}
pub unsafe fn upload_right(game: &mut Game) {
    unsafe {
        if can_reveal_blocks(game) {
            upload_screen(0x4800, 1, game);
        }
        mov24(ram_ptr!(game, LongPtr; 0xa88), 0x9800e4);
        mov24(ram_ptr!(game, LongPtr; 0xa8b), 0x98c8e4);
        mov24(ram_ptr!(game, LongPtr; 0xa8e), 0x999098);
        word! { game, 0;
            hdma_var_1, demo_input_pre_instr, demo_input_instr_timer, demo_input_instr_ptr, demo_input,
            demo_input_new,
        }
        if *ram_ptr!(game; 0xa1e) == 4 {
            word!(game.xray_angle = 192);
        } else {
            word!(game.xray_angle = 64);
        };
    }
}
pub unsafe fn setup_blending(k: u16, game: &mut Game) {
    unsafe {
        let mut v1: u16 = 4096;
        let update_window_masks: bool = if word!(game.fx_type) == 36 {
            !((*ram_ptr!(game; 0x74, 0) as i32 & 0x1f) - 7i32 & 0x8000 == 0)
        } else {
            v1 = 0x2000;
            (can_reveal_blocks(game))
                && ({
                    v1 = 0x4000;
                    true
                })
        };
        if update_window_masks {
            *(ram_ptr!(game, u16; 0x74, 0)) = 0x27;
            *(ram_ptr!(game, u16; 0x74, 1)) = 0x47;
            *(ram_ptr!(game, u16; 0x74, 2)) = 0x87;
        }
        word!(game.fx_layer_blending_config_c |= (v1));
    }
}
pub unsafe fn set_backdrop(game: &mut Game) {
    unsafe {
        word!(game.palette_buffer[0] = 3171);
    }
}
pub unsafe fn restore_left(k: u16, game: &mut Game) {
    unsafe {
        mov24(ram_ptr!(game, LongPtr; 0xa88), 0x980001);
        *(ram_ptr!(game, u16; 0xa8a, 1)) = 0;
        word! { game, 0;
            demo_input_prev, demo_input_prev_new, demo_backup_prev_controller_input,
        }
        word!(game.hdma_table_1[0] = 255);
        let mut v1: i16 = 4096;
        if word!(game.fx_type) != 36 {
            v1 = 0x2000;
            if can_reveal_blocks(game) {
                v1 = 0x4000;
            }
        }
        word!(game.fx_layer_blending_config_c |= ((v1) as u16));
        word!(game.palette_buffer[0] = 0);
        let v2: i32 = word!(game.hdma_object_index) as i32 >> 1;
        word!(game.reg_BG2HOFS = word!(game.hdma_object_A[v2]));
        word!(game.reg_BG2VOFS = word!(game.hdma_object_B[v2]));
        *ram_ptr!(game; 0x59) = byte!(@pointer ram_ptr!(game, u16; 0x192c); word!(game.hdma_object_index) as i32 as isize);
        if (word!(game.vram_write_queue_tail) as i16).wrapping_sub(240) < 0 {
            upload_screen(0x5000, 0, game);
            word!(game.demo_input_pre_instr += 1);
        }
    }
}
pub unsafe fn restore_right(k: u16, game: &mut Game) {
    unsafe {
        let mut v1: i16 = 4096;
        if word!(game.fx_type) != 36 {
            v1 = 0x2000;
            if can_reveal_blocks(game) {
                v1 = 0x4000;
            }
        }
        word!(game.fx_layer_blending_config_c |= ((v1) as u16));
        if (word!(game.vram_write_queue_tail) as i16).wrapping_sub(240) < 0 {
            upload_screen(0x5800, 1, game);
            word!(game.demo_input_pre_instr += 1);
        }
    }
}
pub unsafe fn finish_restore(k: u16, game: &mut Game) {
    unsafe {
        let mut v1: i16 = 4096;
        if word!(game.fx_type) != 36 {
            v1 = 0x2000;
            if can_reveal_blocks(game) {
                v1 = 0x4000;
            }
        }
        word!(game.fx_layer_blending_config_c |= ((v1) as u16));
        if word!(game.time_is_frozen_flag) != 0 {
            word! { game, 0;
                time_is_frozen_flag, demo_input_pre_instr, demo_input_instr_timer, demo_input_instr_ptr,
                xray_angle, demo_input, demo_input_new,
            }
            mov24(ram_ptr!(game, LongPtr; 0xa88), 0x980001);
            *(ram_ptr!(game, u16; 0xa8a, 1)) = 0;
            word! { game, 0;
                demo_input_prev, demo_input_prev_new, demo_backup_prev_controller_input,
            }
            ResponsibleForXrayStandupGlitch(game);
            *(ram_ptr!(game, u16; 0x18b4)).offset((word!(game.hdma_object_index) >> 1) as isize) =
                0;
            QueueSfx1_Max6(0xa, game);
            if (word!(game.fx_type) as u8) != 36 {
                *ram_ptr!(game; 0x74, 2) = 0x80;
                *ram_ptr!(game; 0x74, 1) = 64;
                *ram_ptr!(game; 0x74, 0) = 32;
            }
            word_transfers!(game, descending, 256; le(hdma_table_1, 255) <- value(0xff));
            if word!(game.samus_auto_cancel_hud_item_index) != 0 {
                word! { game, 0; hud_item_index, samus_auto_cancel_hud_item_index }
            }
        }
    }
}

pub unsafe fn reveal_room_items(game: &mut Game) {
    unsafe {
        crate::game::room_mechanisms::reveal_items(game);
        let RS = RomPtr_8F(word!(game.roomdefroomstate_ptr), game) as *const RoomDefRoomstate;
        if (*RS).xray_special_casing_ptr != 0 {
            let mut p = RomPtr_8F((*RS).xray_special_casing_ptr, game) as *const RoomBlock;
            while (*p).x_block != 0 || (*p).y_block != 0 {
                reveal_room_block(
                    (*p).level_data_block,
                    (*p).x_block as u16,
                    (*p).y_block as u16,
                    game,
                );
                p = p.offset(1);
            }
        }
    }
}
