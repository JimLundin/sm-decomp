//! Mother Brain's rainbow-beam palette cycle and death explosions.
#![forbid(unsafe_code)]
#![allow(non_snake_case)]
use crate::game::constants::bank_86::addr_kEproj_MotherBrainDeathExplosion;
use crate::game::enemy_projectiles::SpawnEprojWithRoomGfx;
use crate::game::music::QueueSfx3_Max3;
use crate::game::palette_copies::WriteColorsToPalette;
use crate::game::session_runtime::native_record;
use crate::game_state::Game;
use crate::ram_map::slot;
use crate::rom::lorom_offset;
use crate::rtl::rom_table_word;
use crate::types::{Enemy_MotherBrain, Point16U};

/// Rainbow palettes chained through $AD:E434: each word names the next entry.
const RAINBOW: u32 = 0xade434;

pub(crate) fn MotherBrain_WritePalette(j: u16, game: &mut Game) {
    WriteColorsToPalette(0x82, 0xad, j, 15, game);
    WriteColorsToPalette(0x122, 0xad, j, 15, game);
    WriteColorsToPalette(0x162, 0xad, j.wrapping_add(30), 15, game);
}

/// Every other frame, follow the chain from the beam's cursor to its next
/// nonzero link, store the link plus 2 and write the palette it names.
pub(crate) fn MotherBrain_HandleRainbowBeamPalette(game: &mut Game) {
    if slot!(game, Enemy_MotherBrain[0].base.frame_counter) & 2 == 0 {
        return;
    }
    let mut cursor = slot!(game, Enemy_MotherBrain[0x40].var_01);
    let mut link;
    loop {
        link = cursor;
        cursor = rom_table_word!(game, 0xade434, (cursor >> 1) as isize);
        if cursor != 0 {
            break;
        }
    }
    let next = link.wrapping_add(2);
    slot!(game, Enemy_MotherBrain[0x40].var_01 = next);
    // The palette word is read from the native image two bytes before `next`.
    let image = game.capture_native_rom();
    let at = (lorom_offset(RAINBOW) + next as isize - 2) as usize;
    MotherBrain_WritePalette(
        u16::from_ne_bytes([image.read(at), image.read(at + 1)]),
        game,
    );
}

/// Count down the explosion timer; when it expires, spawn `count` explosions at
/// the positions of the current row, choosing each kind from the three words
/// at `kinds` by a random draw.
pub(crate) fn MotherBrain_GenerateExplosions(delay: u16, kinds: u16, count: u16, game: &mut Game) {
    let timer = slot!(game, Enemy_MotherBrain[0x40].var_E).wrapping_sub(1);
    slot!(game, Enemy_MotherBrain[0x40].var_E = timer);
    if timer & 0x8000 == 0 {
        return;
    }
    slot!(game, Enemy_MotherBrain[0x40].var_E = delay);
    let row = slot!(game, Enemy_MotherBrain[0x40].var_F).wrapping_sub(1);
    slot!(game, Enemy_MotherBrain[0x40].var_F = row);
    if row & 0x8000 != 0 {
        slot!(game, Enemy_MotherBrain[0x40].var_F = 6);
    }
    let mut position = 16u16.wrapping_mul(slot!(game, Enemy_MotherBrain[0x40].var_F));
    let mut remaining = count;
    loop {
        let current = remaining;
        let index = (position >> 1) as isize;
        game.eproj_spawn_pt = Point16U {
            x: rom_table_word!(game, 0xa9b099, index),
            y: rom_table_word!(game, 0xa9b099, index + 1),
        };
        let table = native_record(0xa9, kinds, game);
        let mut kind = table.word(0);
        let random = game.next_random();
        if random >= 0x4000 {
            kind = table.word(2);
            if random >= 0xe000 {
                kind = table.word(4);
            }
        }
        SpawnEprojWithRoomGfx(addr_kEproj_MotherBrainDeathExplosion as u16, kind, game);
        position = position.wrapping_add(4);
        remaining = current.wrapping_sub(1);
        if current == 1 {
            break;
        }
    }
    QueueSfx3_Max3(0x13, game);
}
