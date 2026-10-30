//! The statues of bank $AA: the Tourian entrance statue, the n00b tube cracks
//! and the two chozo statues that carry Samus, in Wrecked Ship across the
//! spikes and in Lower Norfair down into the acid (bank $AA:D745-$AA:E7DC).
//!
//! Was `TourianEntranceStatue_*`, `N00bTubeCracks_Init`, `ChozoStatue_*` and
//! the misnamed `Shaktool_Instr_7`-`14`, `Shaktool_PreInstr_0` and
//! `sub_AAE445`/`sub_AAE784` in the original decompilation. A chozo statue
//! runs the bank $AA function in its `function` word; the compat host
//! dispatches it, so the function ids are unchanged.
#![forbid(unsafe_code)]
use crate::actors::host::{Slopes, abs16};
use crate::actors::layouts::part_view;
use crate::constants::enemy_properties::{
    kEnemyProps_DisableSamusColl, kEnemyProps_ProcessedOffscreen,
};
use crate::enemy_script::EnemyScript;
use crate::ram::RawRam as _;
use crate::ram::{slot, word};
use crate::statue_tables;

/// What the statues need beyond the shared boss effects.
pub trait Host: crate::actors::host::BossEffects + crate::records::Records {
    /// Run the bank $AA function at `address` for the current statue.
    fn run_function(&mut self, address: u32);
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
        words: [var_A, var_B, movement, var_D, var_E, function, activated, variant],
    }
}

part_view! {
    /// The Tourian entrance statue's and n00b tube cracks' view of a slot:
    /// `kind` picks the statue part.
    Decoration: EnemyLayout8 {
        words: [var_A, var_B, var_C, var_D, var_E, var_F, kind, parameter_2],
    }
}

/// Bank $AA chozo statue functions, stored in `function`.
pub mod function {
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

use instruction_list as list;

const BANK: u32 = 0xaa;
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
const LEVEL_DATA: usize = 0x10002;
/// $7E:CD26-$7E:CD2E, which the release clears and sets.
const RELEASE_LIMITS: usize = 0xcd26;
const GRAB_SOUND: u8 = 0x1c;
const FOOTSTEP_SOUND: u8 = 0x4b;
/// Samus command 1: unlock Samus.
const UNLOCK_SAMUS: u16 = 1;

/// Initialisation AI - Tourian entrance statue ($AA:D7C8): the part's
/// instruction list; the base also spawns the decoration, Ridley and
/// Phantoon. Was `TourianEntranceStatue_Init`.
pub fn initialize_tourian_statue<H: Host + ?Sized>(game: &mut H) {
    let me = word!(game.cur_enemy_index) as i32;
    slot!(game, Decoration[me].base.palette_index = 0);
    slot!(game, Decoration[me].base.instruction_timer = 1);
    slot!(game, Decoration[me].base.timer = 0);
    let list =
        statue_tables::tourian_statue(game, (slot!(game, Decoration[me].kind) >> 1) as isize)
            .instruction_list;
    slot!(game, Decoration[me].base.current_instruction = list);
    if slot!(game, Decoration[me].kind) == 0 {
        game.spawn_room_projectile(BASE_DECORATION, 0);
        game.spawn_room_projectile(RIDLEY, 0);
        game.spawn_room_projectile(PHANTOON, 0);
    }
    crate::word_transfers!(game, descending, 16;
        le(target_palettes, 255) <- rom(TOURIAN_STATUE_BASE_PALETTE, 15),
        le(target_palettes, 175) <- rom(TOURIAN_STATUE_PHANTOON_PALETTE, 15),
    );
}

/// Initialisation AI - n00b tube cracks ($AA:E716): their palette. Was
/// `N00bTubeCracks_Init`.
pub fn initialize_tube_cracks<H: Host + ?Sized>(game: &mut H) {
    crate::word_transfers!(game, descending, 32; le(target_palettes, 175) <- rom(TUBE_CRACKS_PALETTE, 31));
}

/// Initialisation AI - chozo statue ($AA:E725): waits, with its variant's
/// palettes and slope and spike blocks. Was `ChozoStatue_Init`.
pub fn initialize_chozo_statue<H: Host + ?Sized>(game: &mut H) {
    let me = word!(game.cur_enemy_index) as i32;
    slot!(
        game,
        Statue[me].base.properties = (slot!(game, Statue[me].base.properties) as i32
            | (kEnemyProps_DisableSamusColl as i32
                | kEnemyProps_ProcessedOffscreen as i32
                | 0x8000)) as u16
    );
    slot!(
        game,
        Statue[me].base.spritemap_pointer = list::NOTHING_SPRITEMAP
    );
    slot!(game, Statue[me].base.instruction_timer = 1);
    slot!(game, Statue[me].base.timer = 0);
    slot!(game, Statue[me].function = function::NOTHING);
    slot!(game, Statue[me].activated = 0);
    slot!(game, Statue[me].base.palette_index = 0);
    // The original clears slot 0's layer, not the statue's.
    slot!(game, Statue[0].base.layer = 0);
    let variant: u16 = slot!(game, Statue[me].variant);
    slot!(
        game,
        Statue[me].base.current_instruction =
            statue_tables::chozo_statue(game, (variant >> 1) as isize).instruction_list
    );
    if variant != 0 {
        crate::word_transfers!(game, descending, 16;
            le(target_palettes, 175) <- table(LOWER_NORFAIR_PALETTE_2, 15),
            le(target_palettes, 159) <- table(LOWER_NORFAIR_PALETTE_1, 15),
        );
        game.spawn_plm(0xc, 0x1d, LOWER_NORFAIR_SLOPE_PLM);
    } else {
        crate::word_transfers!(game, descending, 16;
            le(target_palettes, 175) <- table(WRECKED_SHIP_PALETTE_2, 15),
            le(target_palettes, 159) <- table(WRECKED_SHIP_PALETTE_1, 15),
        );
        game.spawn_plm(0x4a, 0x17, WRECKED_SHIP_SPIKES_PLM);
        game.spawn_plm(0x17, 0x1d, WRECKED_SHIP_SLOPE_PLM);
    }
}

/// Main AI - chozo statue ($AA:E7A7): the function. Was `ChozoStatue_Main`.
pub fn main_ai<H: Host + ?Sized>(game: &mut H) {
    let me = word!(game.cur_enemy_index) as i32;
    let address = (BANK << 16) | slot!(game, Statue[me].function) as u32;
    game.run_function(address);
}

/// Chozo statue function - Lower Norfair ($AA:E445): activated, it carries
/// Samus down. Was `sub_AAE445`.
pub fn lower_norfair<H: Host + ?Sized>(game: &mut H, k: u16) {
    let me = k as i32;
    if slot!(game, Statue[me].activated) != 0 {
        slot!(
            game,
            Statue[me].base.current_instruction = list::LOWER_NORFAIR_ACTIVATED
        );
        slot!(game, Statue[me].base.instruction_timer = 1);
    }
}

/// Chozo statue function - Wrecked Ship ($AA:E7AE): once Phantoon is dead and
/// it is activated, it carries Samus across. Was `Shaktool_PreInstr_0`.
pub fn wrecked_ship<H: Host + ?Sized>(game: &mut H, k: u16) {
    let area = word!(game.area_index);
    if (game.boss_word_native(area) & 1) != 0 {
        let me = k as i32;
        if slot!(game, Statue[me].activated) != 0 {
            slot!(
                game,
                Statue[me].base.current_instruction = list::WRECKED_SHIP_ACTIVATED
            );
            slot!(game, Statue[me].base.instruction_timer = 1);
            // Never read.
            slot!(game, Statue[me].var_A = 0xff00);
            slot!(game, Statue[me].var_B = 256);
        }
    }
}

/// Instruction - start lowering acid ($AA:E429). Was `Shaktool_Instr_9`.
pub fn start_lowering_acid<H: Host + ?Sized>(game: &mut H) {
    game.start_lowering_acid();
}

/// Instruction - set lowered acid position ($AA:E436). Was `Shaktool_Instr_11`.
pub fn set_lowered_acid<H: Host + ?Sized>(game: &mut H) {
    game.lower_acid();
}

/// Instruction - unlock Samus ($AA:E43D). Was `Shaktool_Instr_10`.
pub fn unlock_samus<H: Host + ?Sized>(game: &mut H) {
    game.command_player(UNLOCK_SAMUS);
}

/// Instruction - queue chozo grabs Samus sound effect ($AA:E57F). Was
/// `Shaktool_Instr_8`.
pub fn grab_sound<H: Host + ?Sized>(game: &mut H) {
    game.queue_sound(1, GRAB_SOUND, 6);
}

/// Instruction - queue chozo footsteps sound effect ($AA:E587). Was
/// `Shaktool_Instr_13`.
pub fn footstep_sound<H: Host + ?Sized>(game: &mut H) {
    game.queue_sound(1, FOOTSTEP_SOUND, 6);
}

/// Instruction - spawn chozo spike clearing footstep enemy projectile with X
/// offset [[Y]] ($AA:E58F): a spike block under the foot is cleared. Was
/// `Shaktool_Instr_12`.
pub fn spike_clearing_footstep<H: Host + ?Sized>(
    game: &mut H,
    k: u16,
    jp: EnemyScript,
) -> EnemyScript {
    let me = k as i32;
    let offset: u16 = jp.word(0);
    let x = (slot!(game, Statue[me].base.x_pos)).wrapping_add(offset);
    let y = (slot!(game, Statue[me].base.y_pos)).wrapping_add(28);
    game.select_block_at(x, y);
    // The block below the selected one.
    let index = word!(game.room_width_in_blocks) as i32 + word!(game.cur_block_index) as i32;
    let at = LEVEL_DATA + 2 * index as usize;
    let block = u16::from_ne_bytes([game.ram().read_byte(at), game.ram().read_byte(at + 1)]);
    if (block & 0xf000) == SPIKE_BLOCK {
        game.spawn_collision_plm(CLEAR_SPIKE_PLM);
        game.spawn_room_projectile(SPIKE_FOOTSTEPS, offset);
    }
    jp.offset(1)
}

/// Instruction - chozo statue movement - index [[Y]] ($AA:E5D8): step the
/// statue and carry Samus with it. Was `Shaktool_Instr_7`.
pub fn movement<H: Host + ?Sized>(game: &mut H, k: u16, jp: EnemyScript) -> EnemyScript {
    let me = k as i32;
    slot!(game, Statue[me].movement = jp.word(0));
    let step = statue_tables::movement(game, (slot!(game, Statue[me].movement) >> 1) as isize);
    game.move_horizontal(k, (step.x_velocity as i16 as i32) << 8, Slopes::Ignore);
    let step = statue_tables::movement(game, (slot!(game, Statue[me].movement) >> 1) as isize);
    game.move_vertical(k, (abs16(step.x_velocity) as i16 as i32) << 8);
    game.snap_to_slopes(k);
    let index = slot!(game, Statue[me].movement) as i32 >> 1;
    let step = statue_tables::movement(game, index as isize);
    game.set_player_x((step.samus_x_offset).wrapping_add(slot!(game, Statue[me].base.x_pos)));
    game.set_player_y((step.samus_y_offset).wrapping_add(slot!(game, Statue[me].base.y_pos)));
    jp.offset(1)
}

/// Instruction - release Samus and block slope access ($AA:E6F0). Was
/// `Shaktool_Instr_14`.
pub fn release_samus<H: Host + ?Sized>(game: &mut H) {
    game.command_player(UNLOCK_SAMUS);
    game.ram().copy_in(RELEASE_LIMITS, &0u16.to_ne_bytes());
    game.ram().copy_in(RELEASE_LIMITS + 2, &0u16.to_ne_bytes());
    game.ram().copy_in(RELEASE_LIMITS + 3, &0u16.to_ne_bytes());
    game.ram().copy_in(RELEASE_LIMITS + 7, &1u16.to_ne_bytes());
    game.spawn_plm(0x17, 0x1d, WRECKED_SHIP_SLOPE_PLM);
}
