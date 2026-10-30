//! Dachora, the Brinstar creature that teaches the shinespark: its palette
//! loader, the part of bank $A7:F225-$A7:FF81 that runs here.
#![forbid(unsafe_code)]
use crate::actors::host::ActorHost;
use crate::actors::layouts::EnemyLayout8 as Dachora;
use crate::cartridge::native_record;
use crate::cartridge::rom_table_word;
use crate::ram::RawRam as _;
use crate::ram::slot;

/// $A7:F55F: the palette RAM address of each sprite palette, $7E:C100 + 20h * palette.
const PALETTE_DESTINATIONS: u32 = 0xa7_f55f;
/// Dachora's code and palettes are in bank $A7.
const BANK: u8 = 0xa7;
const PALETTE_BYTES: usize = 32;

/// Load dachora palette ($A7:F535): copy the 32-byte palette at `source` into
/// the enemy's sprite palette. Was `Dachora_Func_1`.
pub fn load_palette(source: u16, k: u16, game: &mut (impl ActorHost + ?Sized)) {
    // The palette index's bits 9-11 select the sprite palette.
    let palette = u16::swap_bytes(slot!(game, Dachora[i32::from(k)].base.palette_index));
    let destination =
        usize::try_from(
            rom_table_word!(game, PALETTE_DESTINATIONS, (palette >> 1) as isize) as i32 as isize,
        )
        .expect("palette destination outside RAM");
    let source = native_record(BANK, source, game);
    let bytes: [u8; PALETTE_BYTES] = core::array::from_fn(|i| source.byte(i));
    game.ram().copy_in(destination, &bytes);
}
