//! HUD icons, resource counters and their ordered tilemap upload.
use crate::game::{
    display_refresh::setup_transfer, menu_tilemap::native_record, music::QueueSfx1_Max6,
};
use crate::game_state::Game;
use crate::ram_map::{checked_word as at, word, words as fields};
use crate::rtl::{MDMAEN, VMADDL, VMAIN, WriteReg};

const WEAPON_DIGITS: u32 = 8429011;
const HEALTH_DIGITS: u32 = 8428991;
const ITEM_OFFSETS: [u16; 5] = [0x14, 0x1c, 0x22, 0x28, 0x2e];

#[derive(Clone, Copy)]
pub(crate) enum Item {
    Missiles,
    SuperMissiles,
    PowerBombs,
    Grapple,
    Xray,
}

static INITIAL_TILES: [u16; 96] = [
    0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f,
    0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f,
    0x2c0f, 0x2c0f, 0x2c12, 0x2c12, 0x2c23, 0x2c12, 0x2c12, 0x2c1e, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f,
    0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f,
    0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2822, 0x2822,
    0x2823, 0x2813, 0x2c14, 0x2c1e, 0x2c0f, 0x2c0b, 0x2c0c, 0x2c0d, 0x2c32, 0x2c0f, 0x2c09, 0x2c0f,
    0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f,
    0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c12, 0x2c12, 0xa824, 0x2815, 0x2c16, 0x2c1e,
];

fn stamp(base: usize, columns: usize, count: usize, tile: impl Fn(usize) -> u16, game: &mut Game) {
    for index in 0..count {
        word!(game.hud_tilemap[base + 32 * (index / columns) + index % columns] = tile(index));
    }
}

/// Native icon reads retain their address-formation contract, even if already installed.
pub(crate) unsafe fn install(item: Item, game: &mut Game) {
    let index = item as usize;
    let base = usize::from(ITEM_OFFSETS[index] >> 1);
    let native = (index != 0).then(|| native_record(0x8099a3 + 4 + 8 * index as u32, game));
    if word!(game.hud_tilemap[base]) & 0x3ff == 15 {
        let columns = if index == 0 { 3 } else { 2 };
        const MISSILES: [u16; 6] = [0x344b, 0x3449, 0x744b, 0x344c, 0x344a, 0x744c];
        stamp(
            base,
            columns,
            2 * columns,
            |i| match &native {
                Some(record) => record.word(2 * i),
                None => MISSILES[i],
            },
            game,
        );
    }
}

/// Requires initialized display/DMA hardware and the native HUD definitions.
pub(crate) unsafe fn initialize(game: &mut Game) {
    unsafe {
        crate::rtl::register_writes! { game;
            word(VMADDL as u16, 0x5800); word(VMAIN as u16, 0x80);
        }
        setup_transfer(u16::from_ne_bytes([1, 0x18]), 0x80988b, 0x40, game);
        WriteReg(MDMAEN as u16, 2, game);
        word_transfers!(game, ascending, 96; le(hud_tilemap, 0) <- table(INITIAL_TILES, 0));
        for (item, mask) in [(Item::Xray, 0x8000), (Item::Grapple, 0x4000)] {
            if word!(game.equipped_items) & mask != 0 {
                install(item, game);
            }
        }
        for (item, maximum) in [
            (Item::Missiles, fields::samus_max_missiles),
            (Item::SuperMissiles, fields::samus_max_super_missiles),
            (Item::PowerBombs, fields::samus_max_power_bombs),
        ] {
            if at!(game, maximum;) != 0 {
                install(item, game);
            }
        }
        word! { game, 0; samus_prev_health, samus_prev_missiles, samus_prev_super_missiles,
        samus_prev_power_bombs, samus_prev_hud_item_index, }
        ammunition(true, game);
        highlight(word!(game.hud_item_index), 0x1000, game);
        highlight(word!(game.samus_prev_hud_item_index), 0x1400, game);
        advance(game);
    }
}

unsafe fn ammunition(initial: bool, game: &mut Game) {
    unsafe {
        const ROWS: [(usize, usize, usize, u16); 3] = [
            (
                fields::samus_max_missiles,
                fields::samus_missiles,
                fields::samus_prev_missiles,
                0x94,
            ),
            (
                fields::samus_max_super_missiles,
                fields::samus_super_missiles,
                fields::samus_prev_super_missiles,
                0x9c,
            ),
            (
                fields::samus_max_power_bombs,
                fields::samus_power_bombs,
                fields::samus_prev_power_bombs,
                0xa2,
            ),
        ];
        for (index, &(maximum, current, previous, offset)) in ROWS.iter().enumerate() {
            if at!(game, maximum;) != 0 && (initial || at!(game, current;) != at!(game, previous;))
            {
                if !initial {
                    at!(game, previous; = at!(game, current;));
                }
                let three = index == 0
                    || (!initial && index == 1 && word!(game.joypad_dbg_flags) & 0x1f40 != 0);
                let count = at!(game, if !initial && index == 1 { previous } else { current };);
                number(WEAPON_DIGITS, count, offset, three, game);
            }
        }
    }
}

/// Native table capture retains its original diagnostic and allocation contract.
pub(crate) unsafe fn advance(game: &mut Game) {
    unsafe {
        if word!(game.reserve_health_mode) == 1 {
            let reserve = native_record(0x80998b, game);
            let start = if word!(game.samus_reserve_health) == 0 {
                12
            } else {
                0
            };
            stamp(8, 2, 6, |i| reserve.word(start + 2 * i), game);
        }
        if word!(game.samus_health) != word!(game.samus_prev_health) {
            word!(game.samus_prev_health = word!(game.samus_health));
            let mut filled = word!(game.samus_health) / 100;
            let remainder = word!(game.samus_health) % 100;
            let count = usize::from(word!(game.samus_max_health) / 100);
            const TANKS: [u16; 14] = [
                0x42, 0x44, 0x46, 0x48, 0x4a, 0x4c, 0x4e, 2, 4, 6, 8, 0xa, 0xc, 0xe,
            ];
            for &offset in TANKS.iter().take(count) {
                let tile = if filled != 0 {
                    filled -= 1;
                    10289
                } else {
                    13360
                };
                word_transfers!(game, ascending, 1; native(0xc608 + usize::from(offset)) <- value(tile));
            }
            number(HEALTH_DIGITS, remainder, 0x8c, false, game);
        }
        ammunition(false, game);
        if word!(game.hud_item_index) != word!(game.samus_prev_hud_item_index) {
            highlight(word!(game.hud_item_index), 0x1000, game);
            highlight(word!(game.samus_prev_hud_item_index), 0x1400, game);
            word!(game.samus_prev_hud_item_index = word!(game.hud_item_index));
            if game.g_ram[0xa1f] != 3
                && game.g_ram[0xa1f] != 20
                && word!(game.grapple_beam_function) == 0xc4f0
                && word!(game.time_is_frozen_flag) == 0
            {
                QueueSfx1_Max6(0x39, game);
            }
        }
        let palette = if game.g_ram[0x5b5] & 0x10 != 0 {
            0x1000
        } else {
            0x1400
        };
        highlight(word!(game.samus_auto_cancel_hud_item_index), palette, game);
        let tail = word!(game.vram_write_queue_tail);
        // Wrap each field cursor before widening; the native bank word overlaps destination.
        for (offset, value) in [(0, 192), (2, 0xc608), (4, 126), (5, 0x5820)] {
            word_transfers!(game, ascending, 1;
                native(0xd0 + usize::from(tail.wrapping_add(offset))) <- value(value));
        }
        word!(game.vram_write_queue_tail = tail.wrapping_add(7));
    }
}

fn highlight(selection: u16, palette: u16, game: &mut Game) {
    word!(game.hud_item_tilemap_palette_bits = palette);
    let item = (selection as i16).wrapping_sub(1);
    if item >= 0 {
        let base = usize::from(ITEM_OFFSETS[item as usize] >> 1);
        let offsets: &[usize] = if item == 0 {
            &[0, 1, 32, 33, 2, 34]
        } else {
            &[0, 1, 32, 33]
        };
        for &offset in offsets {
            let index = base + offset;
            if word!(game.hud_tilemap[index]) != 11279 {
                word!(
                    game.hud_tilemap[index] = word!(game.hud_item_tilemap_palette_bits)
                        | word!(game.hud_tilemap[index]) & 0xe3ff
                );
            }
        }
    }
}

unsafe fn number(address: u32, mut value: u16, mut cursor: u16, three: bool, game: &mut Game) {
    if three {
        word!(
            game.hud_tilemap[cursor >> 1] =
                native_record(address.wrapping_add(2 * u32::from(value / 100)), game).word(0)
        );
        value %= 100;
        cursor = cursor.wrapping_add(2);
    }
    let base = usize::from(cursor >> 1);
    // Units follow the widened pair index; they do not wrap the u16 byte cursor.
    for (index, digit) in [(base, value / 10), (base + 1, value % 10)] {
        word!(
            game.hud_tilemap[index] =
                native_record(address.wrapping_add(2 * u32::from(digit)), game).word(0)
        );
    }
}
