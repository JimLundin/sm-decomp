//! Dachora, the Brinstar creature that teaches the shinespark: its palette
//! loader, the part of bank $A7:F225-$A7:FF81 that runs here.
use crate::actors::boss_lease::BossState;
use crate::actors::layouts::EnemyLayout8 as Dachora;
use crate::actors::session_slots::session_slot;
use crate::cartridge::native_record;
use crate::cartridge::rom_table_word;

/// $A7:F55F: the palette RAM address of each sprite palette, $7E:C100 + 20h * palette.
const PALETTE_DESTINATIONS: u32 = 0xa7_f55f;
/// Dachora's code and palettes are in bank $A7.
const BANK: u8 = 0xa7;
const PALETTE_BYTES: usize = 32;

/// Load dachora palette ($A7:F535): copy the 32-byte palette at `source` into
/// the enemy's sprite palette. Was `Dachora_Func_1`.
pub fn load_palette(source: u16, k: u16, game: &mut impl BossState) {
    // The palette index's bits 9-11 select the sprite palette.
    let palette = u16::swap_bytes(session_slot!(
        game,
        Dachora[i32::from(k)].base.palette_index
    ));
    let destination =
        usize::try_from(
            rom_table_word!(game, PALETTE_DESTINATIONS, (palette >> 1) as isize) as i32 as isize,
        )
        .expect("palette destination outside RAM");
    let source = native_record(BANK, source, game);
    for i in 0..PALETTE_BYTES {
        game.set_slot_byte(destination + i, source.byte(i));
    }
}
