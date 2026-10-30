//! X-ray background capture, hidden-block overlay and restoration.
#![forbid(unsafe_code)]
use crate::display_refresh::enqueue_readback;
use crate::player_transitions::xray::BlendPhase;
use crate::ram::{RawRam, word};
use crate::runtime_access::RuntimeAccess;
use crate::word_transfers;
use sm_data::cartridge::RomRecord;

/// What X-ray needs besides the display host, sound and the native image:
/// the blend mask and blending owned by the player view, and the original's
/// failure reports.
pub trait Host:
    crate::display_refresh::Host
    + crate::effects::SoundEffects
    + crate::cartridge::NativeDiagnostics
    + RuntimeAccess
{
    fn xray_blend_mask(&mut self) -> u16;
    fn apply_xray_blending(&mut self, phase: BlendPhase);
    fn finish_xray(&mut self);
    /// The original's unreachable reveal routine.
    fn xray_unreachable(&mut self);
    /// The original's failed assertion that layer 1's Y position is below $1000.
    fn overlay_origin_assertion(&mut self);
}

fn mult8x8(a: u8, b: u8) -> u16 {
    (a as i32 * b as i32) as u16
}

fn upload_screen<H: Host + ?Sized>(source: u16, screen: u16, game: &mut H) {
    let tail = word!(game.vram_write_queue_tail);
    let entry = 0xd0 + usize::from(tail);
    word_transfers!(game, ascending, 1;
        native(entry) <- value(2048),
        native(entry + 2) <- value(source),
        native(entry + 4) <- value(126));
    let target = ((game.ram().read_byte(0x59) as u16 & 0xfc) << 8).wrapping_add(screen * 1024);
    word_transfers!(game, ascending, 1; native(entry + 5) <- value(target));
    word!(game.vram_write_queue_tail = tail.wrapping_add(7));
}

// Capture all four source tiles before the reverse-order writes, even when RAM aliases.
fn write_block<H: Host + ?Sized>(destination: u16, tile: u16, offsets: [usize; 4], game: &mut H) {
    let index = usize::from(tile);
    let () = [(); 0x2000 / 8][index];
    let mut tiles = [0; 8];
    game.ram().copy_out(0xa000 + 8 * index, &mut tiles);
    for corner in (0..4).rev() {
        let value = u16::from_ne_bytes([tiles[2 * corner], tiles[2 * corner + 1]]);
        let at = 0x4000 + usize::from(destination) + 2 * offsets[corner];
        game.ram().copy_in(at, &value.to_ne_bytes());
    }
}

fn copy_background<H: Host + ?Sized>(game: &mut H) {
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
                let index = source + corner;
                let () = [(); 0x3000 / 2][index];
                let mut bytes = [0; 2];
                game.ram().copy_out(0x4000 + 2 * index, &mut bytes);
                u16::from_ne_bytes(bytes)
            });
            let destination = row * 64 + if block < 16 { block * 2 } else { 1024 };
            for corner in (0..4).rev() {
                let at = 0x4000 + 2 * (destination + corners[corner]);
                game.ram().copy_in(at, &tiles[corner].to_ne_bytes());
            }
            column += 2;
        }
        origin = (origin & 0x400) + (origin.wrapping_add(64) & 0x3ff);
    }
}

pub fn freeze_and_backup<H: Host + ?Sized>(k: u16, game: &mut H) {
    game.ram().write_byte(0xa78, 1);
    let index = usize::from(k as u8);
    game.ram()
        .write_byte(0x1914 + index, word!(game.reg_BG2HOFS) as u8);
    game.ram()
        .write_byte(0x1915 + index, game.ram().read_byte(0xb6));
    game.ram()
        .write_byte(0x1920 + index, word!(game.reg_BG2VOFS) as u8);
    game.ram()
        .write_byte(0x1921 + index, game.ram().read_byte(0xb8));
    game.ram()
        .write_byte(0x192c + index, game.ram().read_byte(0x59));
}
pub fn capture_bg1_right<H: Host + ?Sized>(game: &mut H) {
    enqueue_readback(0x58, 1024, 0x6800, 2048, false, game);
}
pub fn capture_bg1_left<H: Host + ?Sized>(game: &mut H) {
    enqueue_readback(0x58, 0, 0x6000, 2048, false, game);
}
pub fn build_overlay<H: Host + ?Sized>(game: &mut H) {
    copy_background(game);
    if (word!(game.layer1_y_pos) >> 12) != 0 {
        game.overlay_origin_assertion();
    }
    let mut r34: u16 = (mult8x8(
        (word!(game.layer1_y_pos) >> 4) as u8,
        word!(game.room_width_in_blocks) as u8,
    ) as i32
        + (word!(game.layer1_x_pos) as i32 >> 4)) as u16;
    let mut r22: u16 = 0;
    let mut m = 16;
    loop {
        reveal_left_neighbor(r22, r34, game);
        let mut r36: u16 = r34;
        for n_0 in (1i32..=16).rev() {
            reveal_block(n_0 as u16, r22, r36, game);
            r22 = (r22).wrapping_add(4);
            r36 = (r36).wrapping_add(1);
        }
        let v12: u16 = r22;
        r22 = r22.wrapping_add(1984);
        reveal_block(0, r22, r36, game);
        r22 = (v12).wrapping_add(64);
        r34 = (r34).wrapping_add(word!(game.room_width_in_blocks));
        m -= 1;
        if m == 0 {
            break;
        }
    }
    reveal_room_items(game);
    enqueue_readback(0x59, 0, 0x5000, 2048, false, game);
}

fn native_record<H: Host + ?Sized>(bank: u8, address: u16, game: &mut H) -> RomRecord {
    crate::cartridge::native_record(bank, address, game)
}

fn definition_word<H: Host + ?Sized>(index: isize, game: &H) -> u16 {
    game.native_image()
        .table_word(sm_data::cartridge::lorom_offset(0x91d2d6) as usize, index)
}

fn block_bts<H: Host + ?Sized>(block: i32, game: &H) -> u8 {
    let at = usize::try_from(0x16402isize + block as isize).expect("X-ray BTS outside RAM");
    game.ram().read_byte(at)
}

fn block_definition<H: Host + ?Sized>(k: u16, game: &mut H) -> Option<RomRecord> {
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
fn write_neighbor<H: Host + ?Sized>(
    destination: u16,
    tile: u16,
    offsets: [usize; 4],
    game: &mut H,
) {
    for (corner, offset) in offsets.into_iter().enumerate() {
        let source = 0xa000 + 8 * usize::from(tile) + 2 * corner;
        let value = u16::from_ne_bytes([
            game.ram().read_byte(source),
            game.ram().read_byte(source + 1),
        ]);
        let at = 0x4000 + usize::from(destination) + 2 * offset;
        game.ram().copy_in(at, &value.to_ne_bytes());
    }
}

pub fn reveal_left_neighbor<H: Host + ?Sized>(dst_r22: u16, r34: u16, game: &mut H) {
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

fn follow_extension<H: Host + ?Sized>(dst_r22: u16, r36: u16, which_dir: bool, game: &mut H) {
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

fn dispatch_reveal<H: Host + ?Sized>(
    ea: u32,
    jp: &RomRecord,
    columns_left: u16,
    destination: u16,
    block: u16,
    game: &mut H,
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
            game.xray_unreachable();
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

pub fn reveal_block<H: Host + ?Sized>(r18: u16, r22: u16, r36: u16, game: &mut H) {
    if let Some(jp) = block_definition(r36, game) {
        dispatch_reveal(u32::from(jp.word(0)) | 0x910000, &jp, r18, r22, r36, game);
    }
}

pub fn reveal_room_block<H: Host + ?Sized>(a: u16, k: u16, j: u16, game: &mut H) {
    let r24: u16 = (k as i32 - (word!(game.layer1_x_pos) as i32 >> 4)) as u16;
    let r26: u16 = (j as i32 - (word!(game.layer1_y_pos) as i32 >> 4)) as u16;
    if r24 & 0x8000 == 0
        && r24 as i32 - 16i32 & 0x8000 != 0
        && r26 & 0x8000 == 0
        && r26 as i32 - 16i32 & 0x8000 != 0
    {
        let r22: u16 = (4 * (r24 as i32 + 32 * r26 as i32)) as u16;
        let offsets = [0, 1, 32, 33].map(|corner| corner ^ usize::from((a & 0x800) >> 6));
        write_block(r22, a & 0x3ff, offsets, game);
    }
}
pub fn capture_bg2_right<H: Host + ?Sized>(game: &mut H) {
    if can_reveal_blocks(game) {
        if word!(game.earthquake_timer) != 0 {
            word! { game;
                reg_BG1HOFS = (word!(game.layer1_x_pos)).wrapping_add(word!(game.bg1_x_offset));
                reg_BG1VOFS = (word!(game.layer1_y_pos)).wrapping_add(word!(game.bg1_y_offset));
            }
        }
        word!(game.reg_BG2HOFS = (word!(game.reg_BG1HOFS)) & (0xfu16));
        word!(game.reg_BG2VOFS = (word!(game.reg_BG1VOFS)) & (0xfu16));
        game.ram().write_byte(0x59, 73);
    }
    enqueue_readback(0x59, 1024, 0x5800, 2048, true, game);
}
pub fn can_reveal_blocks<H: Host + ?Sized>(game: &mut H) -> bool {
    game.xray_blend_mask() == 0x4000
}
pub fn upload_left<H: Host + ?Sized>(game: &mut H) {
    if can_reveal_blocks(game) {
        upload_screen(0x4000, 0, game);
    }
}
pub fn upload_right<H: Host + ?Sized>(game: &mut H) {
    if can_reveal_blocks(game) {
        upload_screen(0x4800, 1, game);
    }
    for (at, value) in [(0xa88, 0x9800e4u32), (0xa8b, 0x98c8e4), (0xa8e, 0x999098)] {
        word_transfers!(game, ascending, 1; native(at) <- value(value as u16));
        game.ram().write_byte(at + 2, (value >> 16) as u8);
    }
    word! { game, 0;
        hdma_var_1, demo_input_pre_instr, demo_input_instr_timer, demo_input_instr_ptr, demo_input,
        demo_input_new,
    }
    if game.ram().read_byte(0xa1e) == 4 {
        word!(game.xray_angle = 192);
    } else {
        word!(game.xray_angle = 64);
    };
}
pub fn setup_blending<H: Host + ?Sized>(_k: u16, game: &mut H) {
    game.apply_xray_blending(BlendPhase::Setup);
}
pub fn set_backdrop<H: Host + ?Sized>(game: &mut H) {
    {
        word!(game.palette_buffer[0] = 3171);
    }
}
pub fn restore_left<H: Host + ?Sized>(_k: u16, game: &mut H) {
    word_transfers!(game, ascending, 1; native(0xa88) <- value(1));
    game.ram().write_byte(0xa8a, 0x98);
    word_transfers!(game, ascending, 1; native(0xa8b) <- value(0));
    word! { game, 0;
        demo_input_prev, demo_input_prev_new, demo_backup_prev_controller_input,
    }
    word!(game.hdma_table_1[0] = 255);
    game.apply_xray_blending(BlendPhase::Restore);
    word!(game.palette_buffer[0] = 0);
    let v2: i32 = word!(game.hdma_object_index) as i32 >> 1;
    word!(game.reg_BG2HOFS = word!(game.hdma_object_A[v2]));
    word!(game.reg_BG2VOFS = word!(game.hdma_object_B[v2]));
    game.ram().write_byte(
        0x59,
        game.ram()
            .read_byte(0x192c + usize::from(word!(game.hdma_object_index))),
    );
    if (word!(game.vram_write_queue_tail) as i16).wrapping_sub(240) < 0 {
        upload_screen(0x5000, 0, game);
        word!(game.demo_input_pre_instr += 1);
    }
}
pub fn restore_right<H: Host + ?Sized>(_k: u16, game: &mut H) {
    game.apply_xray_blending(BlendPhase::Restore);
    if (word!(game.vram_write_queue_tail) as i16).wrapping_sub(240) < 0 {
        upload_screen(0x5800, 1, game);
        word!(game.demo_input_pre_instr += 1);
    }
}
pub fn finish_restore<H: Host + ?Sized>(_k: u16, game: &mut H) {
    game.apply_xray_blending(BlendPhase::Restore);
    if word!(game.time_is_frozen_flag) != 0 {
        word! { game, 0;
            time_is_frozen_flag, demo_input_pre_instr, demo_input_instr_timer, demo_input_instr_ptr,
            xray_angle, demo_input, demo_input_new,
        }
        word_transfers!(game, ascending, 1; native(0xa88) <- value(1));
        game.ram().write_byte(0xa8a, 0x98);
        word_transfers!(game, ascending, 1; native(0xa8b) <- value(0));
        word! { game, 0;
            demo_input_prev, demo_input_prev_new, demo_backup_prev_controller_input,
        }
        game.finish_xray();
        word_transfers!(game, ascending, 1;
            native(0x18b4 + 2 * usize::from(word!(game.hdma_object_index) >> 1)) <- value(0));
        game.queue_sound(0, 0xa, 6);
        if (word!(game.fx_type) as u8) != 36 {
            game.ram().write_byte(0x76, 0x80);
            game.ram().write_byte(0x75, 64);
            game.ram().write_byte(0x74, 32);
        }
        word_transfers!(game, descending, 256; le(hdma_table_1, 255) <- value(0xff));
        if word!(game.samus_auto_cancel_hud_item_index) != 0 {
            word! { game, 0; hud_item_index, samus_auto_cancel_hud_item_index }
        }
    }
}

pub fn reveal_room_items<H: Host + ?Sized>(game: &mut H) {
    crate::room_mechanisms::graphics::reveal_items(&mut game.runtime());
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
