//! X-ray background capture, hidden-block overlay and restoration.
#![forbid(unsafe_code)]
use crate::display_refresh::enqueue_readback;
use crate::hdma::HdmaObject;
use crate::player_transitions::xray::BlendPhase;
use crate::ram::{RawRam, word};
use crate::runtime_access::{RuntimeAccess, SessionView};
use crate::session_access::SessionAccess;
use crate::visual_host::{Content, Record, Source};
use crate::word_transfers;

/// What X-ray needs besides the display host, sound and the native image:
/// the blend mask and blending owned by the player view, and the original's
/// failure reports.
pub trait Host:
    crate::display_refresh::Host
    + crate::player_position::PlayerFacts
    + crate::effects::SoundEffects
    + crate::cartridge::NativeDiagnostics
    + RuntimeAccess
{
    fn xray_blend_mask(&mut self) -> u16;
    fn apply_xray_blending(&mut self, phase: BlendPhase);
    fn finish_xray(&mut self);
    /// The original's failed assertion that layer 1's Y position is below $1000.
    fn overlay_origin_assertion(&mut self);
}

// The hidden-block reveal runs on the owned-Session view: room terrain,
// tiles and geometry are Session fields; the overlay is work RAM.

fn level<R: SessionView>(h: &R, block: usize) -> u16 {
    match h.session().room.terrain.cells().get(block) {
        Some(cell) => cell.level,
        None => h.word(0x10002 + 2 * block),
    }
}

fn behavior<R: SessionView>(h: &R, block: i32) -> u8 {
    let block = usize::try_from(block).expect("X-ray BTS outside RAM");
    match h.session().room.terrain.cells().get(block) {
        Some(cell) => cell.behavior,
        None => h.byte(0x16402 + block),
    }
}

fn camera<H: Host + ?Sized>(game: &mut H) -> crate::camera::Camera {
    game.runtime().session().room.camera
}

/// Stop demo playback's program and input and set the beam's angle.
fn stop_demo<H: Host + ?Sized>(angle: u16, game: &mut H) {
    let mut runtime = game.runtime();
    let player = &mut runtime.session_mut().player;
    let demo = &mut player.demo;
    demo.pre_action = 0;
    demo.delay = 0;
    demo.program = 0;
    demo.held = 0;
    demo.pressed = 0;
    player.transitions.xray_angle = angle;
}
fn set_backdrop_color<H: Host + ?Sized>(color: u16, game: &mut H) {
    let mut runtime = game.runtime();
    runtime.session_mut().presentation.visual_mut().colors[0] = color;
}
fn upload_screen<H: Host + ?Sized>(source: u16, screen: u16, game: &mut H) {
    let tail = word!(game.VRAM_WRITE_QUEUE_TAIL);
    let entry = 0xd0 + usize::from(tail);
    word_transfers!(game, ascending, 1;
        native(entry) <- value(2048),
        native(entry + 2) <- value(source),
        native(entry + 4) <- value(126));
    let target = ((game.ram().read_byte(0x59) as u16 & 0xfc) << 8).wrapping_add(screen * 1024);
    word_transfers!(game, ascending, 1; native(entry + 5) <- value(target));
    word!(game.VRAM_WRITE_QUEUE_TAIL = tail.wrapping_add(7));
}

// Capture all four source tiles before the reverse-order writes, even when RAM aliases.
fn write_block<R: SessionView>(destination: u16, tile: u16, offsets: [usize; 4], h: &mut R) {
    let tiles = h.session().room.terrain.tiles()[usize::from(tile)];
    for corner in (0..4).rev() {
        let at = 0x4000 + usize::from(destination) + 2 * offsets[corner];
        h.write_slice(at, &tiles[corner].to_le_bytes());
    }
}

fn copy_background<H: Host + ?Sized>(game: &mut H) {
    let camera = camera(game);
    let [x, y] = [0, 1].map(|axis| camera.layer1[axis] as u32 + camera.bg1_offset[axis] as u32);
    let mut origin = (4 * (y & 0xf0) + ((x & 0xf0) >> 3) + 4 * (x & 0x100)) as u16;
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

/// Back up BG2's scroll in A and B and the layer blending byte in C's low
/// byte; `restore_left` puts them back.
pub fn freeze_and_backup<H: Host + ?Sized>(object: &mut HdmaObject, game: &mut H) {
    game.ram().write_byte(0xa78, 1);
    object.a = u16::from_le_bytes([word!(game.REG_BG2HOFS) as u8, game.ram().read_byte(0xb6)]);
    object.b = u16::from_le_bytes([word!(game.REG_BG2VOFS) as u8, game.ram().read_byte(0xb8)]);
    object.c = (object.c & 0xff00) | u16::from(game.ram().read_byte(0x59));
}
pub fn capture_bg1_right<H: Host + ?Sized>(game: &mut H) {
    enqueue_readback(0x58, 1024, 0x6800, 2048, false, game);
}
pub fn capture_bg1_left<H: Host + ?Sized>(game: &mut H) {
    enqueue_readback(0x58, 0, 0x6000, 2048, false, game);
}
/// Copy the visible BG1 into the overlay and reveal the hidden blocks of its
/// 16 rows of 17 blocks, then the room's revealed items, and queue the BG2
/// readback.
pub fn build_overlay<H: Host + ?Sized>(game: &mut H) {
    copy_background(game);
    if (camera(game).layer1[1] >> 12) != 0 {
        game.overlay_origin_assertion();
    }
    {
        // Reveal the screen's 16 rows of 16 blocks and the block past the right edge.
        let h = &mut game.runtime();
        let room = &h.session().room;
        let width = room.width;
        let [x, y] = room.camera.layer1.map(|position| position >> 4);
        let mut row_start = (u16::from(y as u8) * u16::from(width as u8)).wrapping_add(x);
        let mut destination: u16 = 0;
        for _ in 0..16 {
            reveal_left_neighbor(destination, row_start, h);
            let mut block = row_start;
            for columns_left in (1..=16).rev() {
                reveal_block(columns_left, destination, block, h);
                destination = destination.wrapping_add(4);
                block = block.wrapping_add(1);
            }
            reveal_block(0, destination.wrapping_add(1984), block, h);
            destination = destination.wrapping_add(64);
            row_start = row_start.wrapping_add(width);
        }
        reveal_room_items(h);
    }
    enqueue_readback(0x59, 0, 0x5000, 2048, false, game);
}

fn native_record<R: SessionView>(bank: u8, address: u16, h: &mut R) -> <R as Content>::Record {
    h.legacy_native_record(bank, address)
}

fn definition_word<R: SessionView>(index: usize, h: &R) -> u16 {
    h.table(Source::Native, 0x91d2d6).native_word(2 * index)
}

fn block_definition<R: SessionView>(k: u16, game: &mut R) -> Option<<R as Content>::Record> {
    let bts = u16::from(behavior(game, i32::from(k)));
    let kind = level(game, usize::from(k)) & 0xf000;
    let mut index = 0i32;
    loop {
        let value = definition_word(2 * index as usize, game);
        if value == 0xffff {
            return None;
        }
        if value == kind {
            let entries = native_record(0x91, definition_word(2 * index as usize + 1, game), game);
            let mut offset = 0;
            loop {
                let value = entries.native_word(offset);
                if value == 0xffff {
                    return None;
                }
                if value == 0xff00 || value == bts {
                    return Some(native_record(0x91, entries.native_word(offset + 2), game));
                }
                offset += 4;
            }
        }
        index += 1;
    }
}

// Neighbor tiles read each corner immediately before its ascending store, including aliases.
fn write_neighbor<R: SessionView>(destination: u16, tile: u16, offsets: [usize; 4], h: &mut R) {
    for (corner, offset) in offsets.into_iter().enumerate() {
        let bytes = match h.session().room.terrain.tiles().get(usize::from(tile)) {
            Some(tile) => tile[corner].to_le_bytes(),
            None => {
                let source = 0xa000 + 8 * usize::from(tile) + 2 * corner;
                [h.byte(source), h.byte(source + 1)]
            }
        };
        let at = 0x4000 + usize::from(destination) + 2 * offset;
        h.write_slice(at, &bytes);
    }
}

/// Reveal the part of a two-wide block left of the screen that overhangs the
/// first column at `destination`.
pub fn reveal_left_neighbor<R: SessionView>(destination: u16, row_block: u16, game: &mut R) {
    if row_block == 0 {
        return;
    }
    let Some(descriptor) = block_definition(row_block.wrapping_sub(1), game) else {
        return;
    };
    if descriptor.native_word(0) != 0xcf4e {
        if descriptor.native_word(0) != 0xcf6f {
            return;
        }
        write_neighbor(
            destination,
            descriptor.native_word(8),
            [64, 65, 96, 97],
            game,
        );
    }
    write_neighbor(destination, descriptor.native_word(4), [0, 1, 32, 33], game);
}

/// Follow an extension block's chain to the block it extends and reveal that
/// block's tile at `destination`, starting vertically when `vertical`.
fn follow_extension<R: SessionView>(destination: u16, block: u16, vertical: bool, game: &mut R) {
    let width = game.session().room.width;
    let mut y = (block as i32 / width as i32) as u16;
    let mut x = (block as i32 % width as i32) as u16;
    let mut kind = level(game, usize::from(block)) & 0xf000;
    let mut step = behavior(game, i32::from(block)) as i8 as i32;
    if step != 0 {
        let mut vertical = vertical;
        loop {
            let variable = if vertical { &mut y } else { &mut x };
            if ((*variable as i32 + step) as i16) < 0 {
                write_block(destination, 0xff, [0, 1, 32, 33], game);
                return;
            }
            *variable = (*variable as i32 + step) as u16;
            let pos = y as i32 * game.session().room.width as i32 + x as i32;
            step = behavior(game, pos) as i8 as i32;
            let at = usize::try_from(pos).expect("word address outside RAM");
            kind = level(game, at) & 0xf000;
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
        while entries.native_word(offset) != 0xffff {
            if entries.native_word(offset) == 0xff00
                || entries.native_word(offset) as i32 == step & 0xff
            {
                let address = entries.native_word(offset + 2).wrapping_add(2);
                let tile = native_record(0x91, address, game).native_word(0);
                write_block(destination, tile, [0, 1, 32, 33], game);
                return;
            }
            offset += 4;
        }
    }
}

fn dispatch_reveal<R: SessionView>(
    ea: u32,
    descriptor: &<R as Content>::Record,
    columns_left: u16,
    destination: u16,
    block: u16,
    game: &mut R,
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
        0x91cf3e if game.session().room.area != 1 => return,
        0x91cf36 | 0x91cf3e => (1, 1, false),
        0x91cf4e => (2, 1, false),
        0x91cf62 => (1, 2, true),
        0x91cf6f => (2, 2, false),
        _ => {
            crate::player_projectiles::host::Host::unknown(game);
            return;
        }
    };
    for row in 0..height {
        for column in 0..width {
            if column == 1 && columns_left == 1 {
                continue;
            }
            let tile_offset = if repeat_tile {
                2
            } else {
                2 + 2 * (row * width + column)
            };
            let tile = descriptor.native_word(tile_offset);
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

/// Reveal hidden block `block` at `destination`, `columns_left` columns from
/// the screen's right edge.
pub fn reveal_block<R: SessionView>(columns_left: u16, destination: u16, block: u16, game: &mut R) {
    if let Some(descriptor) = block_definition(block, game) {
        dispatch_reveal(
            u32::from(descriptor.native_word(0)) | 0x910000,
            &descriptor,
            columns_left,
            destination,
            block,
            game,
        );
    }
}

/// Reveal room item block `tile` at room block (`x`, `y`) when it is on screen.
pub fn reveal_room_block<R: SessionView>(tile: u16, x: u16, y: u16, game: &mut R) {
    let layer1 = game.session().room.camera.layer1;
    let screen_x = (x as i32 - (layer1[0] as i32 >> 4)) as u16;
    let screen_y = (y as i32 - (layer1[1] as i32 >> 4)) as u16;
    if screen_x < 16 && screen_y < 16 {
        let destination = 4 * (screen_x + 32 * screen_y);
        let offsets = [0, 1, 32, 33].map(|corner| corner ^ usize::from((tile & 0x800) >> 6));
        write_block(destination, tile & 0x3ff, offsets, game);
    }
}
pub fn capture_bg2_right<H: Host + ?Sized>(game: &mut H) {
    if can_reveal_blocks(game) {
        if crate::room_view::effects(game).earthquake_timer != 0 {
            let camera = camera(game);
            word! { game;
                REG_BG1HOFS = camera.layer1[0].wrapping_add(camera.bg1_offset[0]);
                REG_BG1VOFS = camera.layer1[1].wrapping_add(camera.bg1_offset[1]);
            }
        }
        word!(game.REG_BG2HOFS = (word!(game.REG_BG1HOFS)) & (0xfu16));
        word!(game.REG_BG2VOFS = (word!(game.REG_BG1VOFS)) & (0xfu16));
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
    word!(game.XRAY_HDMA_TABLE_END = 0);
    let angle = if game.facing() == 4 { 192 } else { 64 };
    stop_demo(angle, game);
}
pub fn setup_blending<H: Host + ?Sized>(_k: u16, game: &mut H) {
    game.apply_xray_blending(BlendPhase::Setup);
}
pub fn set_backdrop<H: Host + ?Sized>(game: &mut H) {
    set_backdrop_color(0xc63, game);
}
pub fn restore_left<H: Host + ?Sized>(object: &mut HdmaObject, game: &mut H) {
    word_transfers!(game, ascending, 1; native(0xa88) <- value(1));
    game.ram().write_byte(0xa8a, 0x98);
    word_transfers!(game, ascending, 1; native(0xa8b) <- value(0));
    clear_previous_demo_input(game);
    word!(game.HDMA_TABLE_1[0] = 255);
    game.apply_xray_blending(BlendPhase::Restore);
    set_backdrop_color(0, game);
    word!(game.REG_BG2HOFS = object.a);
    word!(game.REG_BG2VOFS = object.b);
    game.ram().write_byte(0x59, object.c as u8);
    if (word!(game.VRAM_WRITE_QUEUE_TAIL) as i16).wrapping_sub(240) < 0 {
        upload_screen(0x5000, 0, game);
        advance_stage(game);
    }
}
/// Step the x-ray background's restore stage, kept in the demo pre-action.
fn advance_stage<H: Host + ?Sized>(game: &mut H) {
    let mut runtime = game.runtime();
    let demo = &mut runtime.session_mut().player.demo;
    demo.pre_action = demo.pre_action.wrapping_add(1);
}
/// Clear the demo's previous input and the controller backup after it.
fn clear_previous_demo_input<H: Host + ?Sized>(game: &mut H) {
    word!(game.DEMO_BACKUP_PREV_CONTROLLER_INPUT = 0);
    let mut runtime = game.runtime();
    let demo = &mut runtime.session_mut().player.demo;
    (demo.previous_held, demo.previous_pressed) = (0, 0);
}
pub fn restore_right<H: Host + ?Sized>(game: &mut H) {
    game.apply_xray_blending(BlendPhase::Restore);
    if (word!(game.VRAM_WRITE_QUEUE_TAIL) as i16).wrapping_sub(240) < 0 {
        upload_screen(0x5800, 1, game);
        advance_stage(game);
    }
}
pub fn finish_restore<H: Host + ?Sized>(object: &mut HdmaObject, game: &mut H) {
    game.apply_xray_blending(BlendPhase::Restore);
    if word!(game.TIME_IS_FROZEN_FLAG) != 0 {
        word!(game.TIME_IS_FROZEN_FLAG = 0);
        stop_demo(0, game);
        word_transfers!(game, ascending, 1; native(0xa88) <- value(1));
        game.ram().write_byte(0xa8a, 0x98);
        word_transfers!(game, ascending, 1; native(0xa8b) <- value(0));
        clear_previous_demo_input(game);
        game.finish_xray();
        object.channels = 0;
        game.queue_sound(0, 0xa, 6);
        if (crate::liquid::LiquidWords::read(game).liquid.kind as u8) != 36 {
            game.ram().write_byte(0x76, 0x80);
            game.ram().write_byte(0x75, 64);
            game.ram().write_byte(0x74, 32);
        }
        word_transfers!(game, descending, 256; le(HDMA_TABLE_1, 255) <- value(0xff));
        let mut runtime = game.runtime();
        let controls = &mut runtime.session_mut().projectiles.controls;
        if controls.automatic_weapon != 0 {
            (controls.selected_weapon, controls.automatic_weapon) = (0, 0);
        }
    }
}

pub fn reveal_room_items<R: SessionView>(game: &mut R) {
    crate::room_mechanisms::graphics::reveal_items(game);
    let state = game.session().state_key();
    let state = native_record(0x8f, state, game);
    if state.native_word(0x10) != 0 {
        let entries = native_record(0x8f, state.native_word(0x10), game);
        let mut offset = 0;
        while entries.byte(offset) != 0 || entries.byte(offset + 1) != 0 {
            reveal_room_block(
                entries.native_word(offset + 2),
                entries.byte(offset) as u16,
                entries.byte(offset + 1) as u16,
                game,
            );
            offset += 4;
        }
    }
}
