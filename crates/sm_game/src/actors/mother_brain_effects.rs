//! Mother Brain's rainbow-beam palette cycle and death explosions.
#![forbid(unsafe_code)]
#![allow(non_snake_case)]
use crate::actors::layouts::EnemyLayout72 as Enemy_MotherBrain;
use crate::cartridge::native_record;
use crate::cartridge::rom_table_word;
use crate::palette_copies::copy_rom_colors_to_palette;
use crate::ram::RawRam as _;
use crate::ram::slot;
use sm_data::cartridge::lorom_offset;

/// What Mother Brain's effects need beyond the common actor host.
pub trait Host: crate::actors::host::ActorHost + crate::effects::SoundEffects {
    /// Where the next enemy projectile spawns.
    fn set_projectile_spawn_point(&mut self, x: u16, y: u16);
    /// Spawn a death explosion of `kind` at the projectile spawn point.
    fn spawn_death_explosion(&mut self, kind: u16);
}

/// Rainbow palettes chained through $AD:E434: each word names the next entry.
const RAINBOW: u32 = 0xade434;

pub fn MotherBrain_WritePalette(j: u16, game: &mut (impl Host + ?Sized)) {
    copy_rom_colors_to_palette(0x82, 0xad, j, 15, game);
    copy_rom_colors_to_palette(0x122, 0xad, j, 15, game);
    copy_rom_colors_to_palette(0x162, 0xad, j.wrapping_add(30), 15, game);
}

/// Every other frame, follow the chain from the beam's cursor to its next
/// nonzero link, store the link plus 2 and write the palette it names.
pub fn MotherBrain_HandleRainbowBeamPalette(game: &mut (impl Host + ?Sized)) {
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
    let image = game.native_image();
    let at = (lorom_offset(RAINBOW) + next as isize - 2) as usize;
    MotherBrain_WritePalette(
        u16::from_ne_bytes([image.read(at), image.read(at + 1)]),
        game,
    );
}

/// Count down the explosion timer; when it expires, spawn `count` explosions at
/// the positions of the current row, choosing each kind from the three words
/// at `kinds` by a random draw.
pub fn MotherBrain_GenerateExplosions(
    delay: u16,
    kinds: u16,
    count: u16,
    game: &mut (impl Host + ?Sized),
) {
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
        let x = rom_table_word!(game, 0xa9b099, index);
        let y = rom_table_word!(game, 0xa9b099, index + 1);
        game.set_projectile_spawn_point(x, y);
        let table = native_record(0xa9, kinds, game);
        let mut kind = table.word(0);
        let random = game.next_random();
        if random >= 0x4000 {
            kind = table.word(2);
            if random >= 0xe000 {
                kind = table.word(4);
            }
        }
        game.spawn_death_explosion(kind);
        position = position.wrapping_add(4);
        remaining = current.wrapping_sub(1);
        if current == 1 {
            break;
        }
    }
    game.queue_sound(2, 0x13, 3);
}
