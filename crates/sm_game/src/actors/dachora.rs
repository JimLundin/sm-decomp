//! Dachora's palette rows: 32 bytes from bank $A7 to the slot's palette RAM.
#![forbid(unsafe_code)]
#![allow(non_snake_case)]
use crate::actors::host::ActorHost;
use crate::actors::layouts::EnemyLayout8 as Dachora;
use crate::cartridge::native_record;
use crate::cartridge::rom_table_word;
use crate::ram::RawRam as _;
use crate::ram::slot;

/// Palette RAM destinations, indexed by the slot's palette (high byte first).
const DESTINATIONS: u32 = 0xa7f55f;

pub fn Dachora_Func_1(j: u16, k: u16, game: &mut (impl ActorHost + ?Sized)) {
    let palette = u16::swap_bytes(slot!(game, Dachora[i32::from(k)].base.palette_index));
    let destination = usize::try_from(
        rom_table_word!(game, DESTINATIONS, (palette >> 1) as isize) as i32 as isize
    )
    .expect("palette destination outside RAM");
    let source = native_record(0xa7, j, game);
    let bytes: [u8; 32] = core::array::from_fn(|i| source.byte(i));
    game.ram().copy_in(destination, &bytes);
}
