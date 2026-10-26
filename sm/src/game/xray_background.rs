//! X-ray background capture, hidden-block overlay and restoration.
use crate::game::player_transitions::xray::BlendPhase;
use crate::game::prelude::*;
use crate::game::session_runtime::Runtime;

use crate::game::music::QueueSfx1_Max6;

use crate::game::session_runtime::finish_xray;
use crate::rom::RomRecord;

fn upload_screen(source: u16, screen: u16, game: &mut Game) {
    let tail = word!(game.vram_write_queue_tail);
    let entry = 0xd0 + usize::from(tail);
    word_transfers!(game, ascending, 1;
        native(entry) <- value(2048),
        native(entry + 2) <- value(source),
        native(entry + 4) <- value(126));
    let target = ((game.g_ram[0x59] as u16 & 0xfc) << 8).wrapping_add(screen * 1024);
    word_transfers!(game, ascending, 1; native(entry + 5) <- value(target));
    word!(game.vram_write_queue_tail = tail.wrapping_add(7));
}

// Capture all four source tiles before the reverse-order writes, even when RAM aliases.
#[forbid(unsafe_code)]
fn write_block(destination: u16, tile: u16, offsets: [usize; 4], game: &mut Game) {
    let tiles = game.g_ram[0xa000..0xc000].as_chunks::<8>().0[tile as usize];
    for corner in (0..4).rev() {
        let value = u16::from_ne_bytes([tiles[2 * corner], tiles[2 * corner + 1]]);
        let at = 0x4000 + usize::from(destination) + 2 * offsets[corner];
        game.g_ram[at..at + 2].copy_from_slice(&value.to_ne_bytes());
    }
}

#[forbid(unsafe_code)]
fn copy_background(game: &mut Game) {
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
            let tiles = corners.map(|corner| {
                u16::from_ne_bytes(game.g_ram[0x4000..0x7000].as_chunks::<2>().0[source + corner])
            });
            let destination = row * 64 + if block < 16 { block * 2 } else { 1024 };
            for corner in (0..4).rev() {
                let at = 0x4000 + 2 * (destination + corners[corner]);
                game.g_ram[at..at + 2].copy_from_slice(&tiles[corner].to_ne_bytes());
            }
            column += 2;
        }
        origin = (origin & 0x400) + (origin.wrapping_add(64) & 0x3ff);
    }
}

pub fn freeze_and_backup(k: u16, game: &mut Game) {
    game.g_ram[0xa78] = 1;
    let index = usize::from(k as u8);
    game.g_ram[0x1914 + index] = word!(game.reg_BG2HOFS) as u8;
    game.g_ram[0x1915 + index] = game.g_ram[0xb6];
    game.g_ram[0x1920 + index] = word!(game.reg_BG2VOFS) as u8;
    game.g_ram[0x1921 + index] = game.g_ram[0xb8];
    game.g_ram[0x192c + index] = game.g_ram[0x59];
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
            pointer c"uint8 *RomPtr_RAM(uint16_t)".as_ptr()
        };
        let mut r34: u16 = (Mult8x8(
            (word!(game.layer1_y_pos) >> 4) as u8,
            word!(game.room_width_in_blocks) as u8,
        ) as i32
            + (word!(game.layer1_x_pos) as i32 >> 4)) as u16;
        let mut r22: u16 = 0;
        let mut m = 16;
        do_until! { m == 0 ;
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

#[forbid(unsafe_code)]
fn native_record(bank: u8, address: u16, game: &mut Game) -> RomRecord {
    let absolute = (u32::from(bank) << 16) | u32::from(address);
    if address & 0x8000 == 0 {
        crate::game::session_runtime::native_rom_diagnostic(absolute, game);
    }
    drop(game.capture_native_rom_binding());
    RomRecord {
        image: game.capture_native_rom(),
        base: crate::rtl::lorom_offset(absolute) as usize,
    }
}

#[forbid(unsafe_code)]
fn definition_word(index: isize, game: &Game) -> u16 {
    game.capture_native_rom()
        .table_word(crate::rtl::lorom_offset(0x91d2d6) as usize, index)
}

#[forbid(unsafe_code)]
fn block_bts(block: i32, game: &Game) -> u8 {
    let at = usize::try_from(0x16402isize + block as isize).expect("X-ray BTS outside RAM");
    game.g_ram[at]
}

#[forbid(unsafe_code)]
fn block_definition(k: u16, game: &mut Game) -> Option<RomRecord> {
    let bts = u16::from(block_bts(i32::from(k), game));
    let kind = word!(game.level_data[k]) & 0xf000;
    let mut index = 0i32;
    loop {
        let value = definition_word(2 * index as isize, game);
        if value == 0xffff {
            return None;
        }
        if value == kind {
            let entries = native_record(0x91, definition_word(2 * index as isize + 1, game), game);
            let mut offset = 0;
            loop {
                let value = entries.word(offset);
                if value == 0xffff {
                    return None;
                }
                if value == 0xff00 || value == bts {
                    return Some(native_record(0x91, entries.word(offset + 2), game));
                }
                offset += 4;
            }
        }
        index += 1;
    }
}

// Neighbor tiles read each corner immediately before its ascending store, including aliases.
#[forbid(unsafe_code)]
fn write_neighbor(destination: u16, tile: u16, offsets: [usize; 4], game: &mut Game) {
    for (corner, offset) in offsets.into_iter().enumerate() {
        let source = 0xa000 + 8 * usize::from(tile) + 2 * corner;
        let value = u16::from_ne_bytes([game.g_ram[source], game.g_ram[source + 1]]);
        let at = 0x4000 + usize::from(destination) + 2 * offset;
        game.g_ram[at..at + 2].copy_from_slice(&value.to_ne_bytes());
    }
}

#[forbid(unsafe_code)]
pub fn reveal_left_neighbor(dst_r22: u16, r34: u16, game: &mut Game) {
    if r34 == 0 {
        return;
    }
    let Some(descriptor) = block_definition(r34.wrapping_sub(1), game) else {
        return;
    };
    if descriptor.word(0) != 0xcf4e {
        if descriptor.word(0) != 0xcf6f {
            return;
        }
        write_neighbor(dst_r22, descriptor.word(8), [64, 65, 96, 97], game);
    }
    write_neighbor(dst_r22, descriptor.word(4), [0, 1, 32, 33], game);
}

#[forbid(unsafe_code)]
fn follow_extension(dst_r22: u16, r36: u16, which_dir: bool, game: &mut Game) {
    let mut y = (r36 as i32 / word!(game.room_width_in_blocks) as i32) as u16;
    let mut x = (r36 as i32 % word!(game.room_width_in_blocks) as i32) as u16;
    let mut kind = word!(game.level_data[r36]) & 0xf000;
    let mut step = block_bts(i32::from(r36), game) as i8 as i32;
    if step != 0 {
        let mut vertical = which_dir;
        loop {
            let variable = if vertical { &mut y } else { &mut x };
            if ((*variable as i32 + step) as i16) < 0 {
                write_block(dst_r22, 0xff, [0, 1, 32, 33], game);
                return;
            }
            *variable = (*variable as i32 + step) as u16;
            let pos = y as i32 * word!(game.room_width_in_blocks) as i32 + x as i32;
            step = block_bts(pos, game) as i8 as i32;
            kind = word!(game.level_data[pos]) & 0xf000;
            if kind == 0xd000 {
                vertical = true;
            } else {
                if kind != 0x5000 {
                    break;
                }
                if step < 0 {
                    vertical = false;
                }
            }
        }
    }
    if definition_word(2, game) == kind {
        let entries = native_record(0x91, definition_word(3, game), game);
        let mut offset = 0;
        while entries.word(offset) != 0xffff {
            if entries.word(offset) == 0xff00 || entries.word(offset) as i32 == step & 0xff {
                let tile =
                    native_record(0x91, entries.word(offset + 2).wrapping_add(2), game).word(0);
                write_block(dst_r22, tile, [0, 1, 32, 33], game);
                return;
            }
            offset += 4;
        }
    }
}

#[forbid(unsafe_code)]
fn dispatch_reveal(
    ea: u32,
    jp: &RomRecord,
    columns_left: u16,
    destination: u16,
    block: u16,
    game: &mut Game,
) {
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
            let tile = jp.word(descriptor);
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

#[forbid(unsafe_code)]
pub fn reveal_block(r18: u16, r22: u16, r36: u16, game: &mut Game) {
    if let Some(jp) = block_definition(r36, game) {
        dispatch_reveal(u32::from(jp.word(0)) | 0x910000, &jp, r18, r22, r36, game);
    }
}

#[forbid(unsafe_code)]
pub fn reveal_room_block(a: u16, k: u16, j: u16, game: &mut Game) {
    let R24: u16 = (k as i32 - (word!(game.layer1_x_pos) as i32 >> 4)) as u16;
    let R26: u16 = (j as i32 - (word!(game.layer1_y_pos) as i32 >> 4)) as u16;
    if R24 & 0x8000 == 0
        && R24 as i32 - 16i32 & 0x8000 != 0
        && R26 & 0x8000 == 0
        && R26 as i32 - 16i32 & 0x8000 != 0
    {
        let R22: u16 = (4 * (R24 as i32 + 32 * R26 as i32)) as u16;
        let offsets = [0, 1, 32, 33].map(|corner| corner ^ usize::from((a & 0x800) >> 6));
        write_block(R22, a & 0x3ff, offsets, game);
    }
}
pub fn capture_bg2_right(game: &mut Game) {
    if can_reveal_blocks(game) {
        if word!(game.earthquake_timer) != 0 {
            word! { game;
                reg_BG1HOFS = (word!(game.layer1_x_pos)).wrapping_add(word!(game.bg1_x_offset));
                reg_BG1VOFS = (word!(game.layer1_y_pos)).wrapping_add(word!(game.bg1_y_offset));
            }
        }
        word!(game.reg_BG2HOFS = (word!(game.reg_BG1HOFS)) & (0xfu16));
        word!(game.reg_BG2VOFS = (word!(game.reg_BG1VOFS)) & (0xfu16));
        game.g_ram[0x59] = 73;
    }
    crate::game::vram_read_queue::enqueue(0x59, 1024, 0x5800, 2048, true, game);
}
pub fn can_reveal_blocks(game: &mut Game) -> bool {
    Runtime::new(game).xray_blend_mask() == 0x4000
}
pub fn upload_left(game: &mut Game) {
    if can_reveal_blocks(game) {
        upload_screen(0x4000, 0, game);
    }
}
pub fn upload_right(game: &mut Game) {
    if can_reveal_blocks(game) {
        upload_screen(0x4800, 1, game);
    }
    for (at, value) in [(0xa88, 0x9800e4u32), (0xa8b, 0x98c8e4), (0xa8e, 0x999098)] {
        word_transfers!(game, ascending, 1; native(at) <- value(value as u16));
        game.g_ram[at + 2] = (value >> 16) as u8;
    }
    word! { game, 0;
        hdma_var_1, demo_input_pre_instr, demo_input_instr_timer, demo_input_instr_ptr, demo_input,
        demo_input_new,
    }
    if game.g_ram[0xa1e] == 4 {
        word!(game.xray_angle = 192);
    } else {
        word!(game.xray_angle = 64);
    };
}
pub fn setup_blending(_k: u16, game: &mut Game) {
    Runtime::new(game).apply_xray_blending(BlendPhase::Setup);
}
pub fn set_backdrop(game: &mut Game) {
    {
        word!(game.palette_buffer[0] = 3171);
    }
}
pub fn restore_left(k: u16, game: &mut Game) {
    word_transfers!(game, ascending, 1; native(0xa88) <- value(1));
    game.g_ram[0xa8a] = 0x98;
    word_transfers!(game, ascending, 1; native(0xa8b) <- value(0));
    word! { game, 0;
        demo_input_prev, demo_input_prev_new, demo_backup_prev_controller_input,
    }
    word!(game.hdma_table_1[0] = 255);
    Runtime::new(game).apply_xray_blending(BlendPhase::Restore);
    word!(game.palette_buffer[0] = 0);
    let v2: i32 = word!(game.hdma_object_index) as i32 >> 1;
    word!(game.reg_BG2HOFS = word!(game.hdma_object_A[v2]));
    word!(game.reg_BG2VOFS = word!(game.hdma_object_B[v2]));
    game.g_ram[0x59] = game.g_ram[0x192c + usize::from(word!(game.hdma_object_index))];
    if (word!(game.vram_write_queue_tail) as i16).wrapping_sub(240) < 0 {
        upload_screen(0x5000, 0, game);
        word!(game.demo_input_pre_instr += 1);
    }
}
pub fn restore_right(k: u16, game: &mut Game) {
    Runtime::new(game).apply_xray_blending(BlendPhase::Restore);
    if (word!(game.vram_write_queue_tail) as i16).wrapping_sub(240) < 0 {
        upload_screen(0x5800, 1, game);
        word!(game.demo_input_pre_instr += 1);
    }
}
pub unsafe fn finish_restore(k: u16, game: &mut Game) {
    Runtime::new(game).apply_xray_blending(BlendPhase::Restore);
    if word!(game.time_is_frozen_flag) != 0 {
        word! { game, 0;
            time_is_frozen_flag, demo_input_pre_instr, demo_input_instr_timer, demo_input_instr_ptr,
            xray_angle, demo_input, demo_input_new,
        }
        word_transfers!(game, ascending, 1; native(0xa88) <- value(1));
        game.g_ram[0xa8a] = 0x98;
        word_transfers!(game, ascending, 1; native(0xa8b) <- value(0));
        word! { game, 0;
            demo_input_prev, demo_input_prev_new, demo_backup_prev_controller_input,
        }
        finish_xray(game);
        word_transfers!(game, ascending, 1;
            native(0x18b4 + 2 * usize::from(word!(game.hdma_object_index) >> 1)) <- value(0));
        QueueSfx1_Max6(0xa, game);
        if (word!(game.fx_type) as u8) != 36 {
            game.g_ram[0x76] = 0x80;
            game.g_ram[0x75] = 64;
            game.g_ram[0x74] = 32;
        }
        word_transfers!(game, descending, 256; le(hdma_table_1, 255) <- value(0xff));
        if word!(game.samus_auto_cancel_hud_item_index) != 0 {
            word! { game, 0; hud_item_index, samus_auto_cancel_hud_item_index }
        }
    }
}

pub unsafe fn reveal_room_items(game: &mut Game) {
    unsafe {
        crate::game::room_mechanisms::reveal_items(game);
    }
    let state = native_record(0x8f, word!(game.roomdefroomstate_ptr), game);
    if state.word(0x10) != 0 {
        let entries = native_record(0x8f, state.word(0x10), game);
        let mut offset = 0;
        while entries.byte(offset) != 0 || entries.byte(offset + 1) != 0 {
            reveal_room_block(
                entries.word(offset + 2),
                entries.byte(offset) as u16,
                entries.byte(offset + 1) as u16,
                game,
            );
            offset += 4;
        }
    }
}
