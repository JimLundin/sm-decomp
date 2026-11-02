//! The statues of bank $AA: the Tourian entrance statue, the n00b tube cracks
//! and the two chozo statues that carry Samus, in Wrecked Ship across the
//! spikes and in Lower Norfair down into the acid (bank $AA:D745-$AA:E7DC).
//!
//! Was `TourianEntranceStatue_*`, `N00bTubeCracks_Init`, `ChozoStatue_*` and
//! the misnamed `Shaktool_Instr_7`-`14`, `Shaktool_PreInstr_0` and
//! `sub_AAE445`/`sub_AAE784` in the original decompilation. A chozo statue
//! runs the bank $AA function in its `function` word, whose ids are the
//! original's.
use crate::actors::boss_lease::BossLease;
use crate::actors::host::BossEffects;
use crate::actors::layouts::part_view;

/// A lease on a game with the statues' effects.
pub trait Lease: BossLease<Game: BossEffects> {
    /// The boss word of `area`, in native byte order.
    fn boss_word_native(&mut self, area: u16) -> u16;
    /// Start the room's acid lowering.
    fn start_lowering_acid(&mut self);
    /// Set the room's acid to its lowered position.
    fn lower_acid(&mut self);
    /// Select the level block at (`x`, `y`).
    fn select_block_at(&mut self, x: u16, y: u16);
    /// Spawn the collision PLM with `header` at the selected block.
    fn spawn_collision_plm(&mut self, header: u16);
}

part_view! {
    /// A chozo statue's view of its slot. `movement` indexes the carry
    /// movement tables; `activated` starts the carry; `variant` is 0 in
    /// Wrecked Ship and 2 in Lower Norfair.
    Statue: EnemyLayout8 {
        words: [var_a, var_b, movement, var_d, var_e, function, activated, variant],
    }
}

part_view! {
    /// The Tourian entrance statue's and n00b tube cracks' view of a slot:
    /// `kind` picks the statue part.
    Decoration: EnemyLayout8 {
        words: [var_a, var_b, var_c, var_d, var_e, var_f, kind, parameter_2],
    }
}

/// Bank $AA chozo statue functions, stored in `function`.
pub mod function {
    /// The common clear-pre-instruction command selects the bank $AA RTS.
    pub const CLEARED: u16 = 0x807b;
    /// The Wrecked Ship carry script selects this bank $AA RTS.
    pub const SCRIPTED: u16 = 0xe7da;
    pub const LOWER_NORFAIR: u16 = 0xe445;
    pub const NOTHING: u16 = 0xe7a6;
    pub const WRECKED_SHIP: u16 = 0xe7ae;
}

/// Bank $AA chozo statue instruction lists.
pub mod instruction_list {
    pub const LOWER_NORFAIR_ACTIVATED: u16 = 0xe3a7;
    pub const WRECKED_SHIP_ACTIVATED: u16 = 0xe461;
    pub const NOTHING_SPRITEMAP: u16 = 0x804d;
}

/// Bank $AA palettes: the Tourian entrance statue's Phantoon and base
/// decoration, and the n00b tube cracks.
const TOURIAN_STATUE_PHANTOON_PALETTE: u32 = 0xaa_d765;
const TOURIAN_STATUE_BASE_PALETTE: u32 = 0xaa_d785;
const TUBE_CRACKS_PALETTE: u32 = 0xaa_e2dd;
/// The chozo statue palettes, sprite palettes 1 and 2, as the original's
/// copies of $AA:E31D-$AA:E39C.
const WRECKED_SHIP_PALETTE_1: [u16; 16] = [
    0x3800, 0x633f, 0x4a9f, 0x2ddf, 0x6739, 0x4e73, 0x318c, 0x18c6, 0x27ff, 0x1af7, 0xdce, 0xc6,
    0x3fff, 0x2b39, 0x7fff, 0,
];
const WRECKED_SHIP_PALETTE_2: [u16; 16] = [
    0x3800, 0x633f, 0x4a9f, 0x2ddf, 0x4210, 0x318c, 0x2108, 0x1084, 0x27ff, 0x1af7, 0xdce, 0xc6,
    0x3fff, 0x2b39, 0x5294, 0,
];
const LOWER_NORFAIR_PALETTE_1: [u16; 16] = [
    0x3800, 0x633f, 0x4a9f, 0x2ddf, 0x2f7c, 0x2295, 0x118d, 0x8e8, 0x27ff, 0x1af7, 0xdce, 0xc6,
    0x3fff, 0x2b39, 0x73df, 0x43,
];
const LOWER_NORFAIR_PALETTE_2: [u16; 16] = [
    0x3800, 0x633f, 0x4a9f, 0x2ddf, 0x2295, 0x118d, 0x8e8, 0x85, 0x27ff, 0x1af7, 0xdce, 0xc6,
    0x3fff, 0x2b39, 0x5294, 1,
];
/// Bank $86 enemy projectiles.
const BASE_DECORATION: u16 = 0xbabe;
const RIDLEY: u16 = 0xbaa2;
const PHANTOON: u16 = 0xbab0;
const SPIKE_FOOTSTEPS: u16 = 0xaf68;
/// Bank $84 PLMs: the Lower Norfair statue's and the Wrecked Ship statue's
/// slope blocks, the Wrecked Ship spikes and the spike-clearing collision.
const LOWER_NORFAIR_SLOPE_PLM: u16 = 0xd6d6;
const WRECKED_SHIP_SPIKES_PLM: u16 = 0xd6ee;
const WRECKED_SHIP_SLOPE_PLM: u16 = 0xd6fc;
const CLEAR_SPIKE_PLM: u16 = 0xd113;
/// Block type Ah, spikes, in a level data word's top nibble.
const SPIKE_BLOCK: u16 = 0xa000;
/// $7F:0002, the level data.
/// $7E:CD26-$7E:CD2E, which the release clears and sets.
const GRAB_SOUND: u8 = 0x1c;
const FOOTSTEP_SOUND: u8 = 0x4b;
/// Samus command 1: unlock Samus.
const UNLOCK_SAMUS: u16 = 1;

// Both actual owners execute the same complete statue kernels.
pub use owned::{
    footstep_sound, grab_sound, initialize_chozo_statue, initialize_tourian_statue,
    initialize_tube_cracks, lower_norfair, main_ai, movement, release_samus, set_lowered_acid,
    spike_clearing_footstep, start_lowering_acid, unlock_samus, wrecked_ship,
};

const _: () = {
    use crate::actors::statues::{function as f, instruction_list as l};
    use crate::constants::bank_aa::*;
    const fn at(word: u16) -> i32 {
        0xaa_0000 | word as i32
    }
    assert!(at(f::LOWER_NORFAIR) == 11199557 && at(f::WRECKED_SHIP) == 11200430);
    assert!(at(f::NOTHING) == FNNULLSUB_276);
    assert!(at(f::CLEARED) == FNNULLSUB_171_AA);
    assert!(at(f::SCRIPTED) == 11200474);
    assert!(l::LOWER_NORFAIR_ACTIVATED == ADDR_K_SHAKTOOL_ILIST_E3A7 as u16);
    assert!(l::WRECKED_SHIP_ACTIVATED == ADDR_K_SHAKTOOL_ILIST_E461 as u16);
    assert!(l::NOTHING_SPRITEMAP == ADDR_K_SPRITEMAP_NOTHING_AA as u16);
};

#[path = "statues_owned.rs"]
pub mod owned;

/// The original two loaded allocations are captured before descending copies.
pub fn install_tourian_palettes<H: crate::actors::remaining_core::Core>(game: &mut H) {
    let first = game.content_image();
    let second = game.content_image();
    let first_base = usize::try_from(sm_data::cartridge::lorom_offset(
        TOURIAN_STATUE_BASE_PALETTE,
    ))
    .expect("statue palette");
    let second_base = usize::try_from(sm_data::cartridge::lorom_offset(
        TOURIAN_STATUE_PHANTOON_PALETTE,
    ))
    .expect("statue palette");
    for offset in (0..16).rev() {
        let at = first_base + 2 * offset;
        let value = u16::from_ne_bytes([first.read(at), first.read(at + 1)]);
        game.set_remaining_color(true, 2 * (240 + offset), value, false);
        let at = second_base + 2 * offset;
        let value = u16::from_ne_bytes([second.read(at), second.read(at + 1)]);
        game.set_remaining_color(true, 2 * (160 + offset), value, false);
    }
}
pub fn install_tube_palette<H: crate::actors::remaining_core::Core>(game: &mut H) {
    let image = game.content_image();
    let base = usize::try_from(sm_data::cartridge::lorom_offset(TUBE_CRACKS_PALETTE))
        .expect("tube palette");
    for offset in (0..32).rev() {
        let at = base + 2 * offset;
        let value = u16::from_ne_bytes([image.read(at), image.read(at + 1)]);
        game.set_remaining_color(true, 2 * (144 + offset), value, false);
    }
}
pub fn install_chozo_palettes<H: crate::actors::remaining_core::Core>(
    lower_norfair: bool,
    game: &mut H,
) {
    let (first, second) = if lower_norfair {
        (&LOWER_NORFAIR_PALETTE_1, &LOWER_NORFAIR_PALETTE_2)
    } else {
        (&WRECKED_SHIP_PALETTE_1, &WRECKED_SHIP_PALETTE_2)
    };
    for (offset, (first, second)) in first.iter().zip(second).enumerate().rev() {
        game.set_remaining_color(true, 2 * (160 + offset), *second, false);
        game.set_remaining_color(true, 2 * (144 + offset), *first, false);
    }
}
