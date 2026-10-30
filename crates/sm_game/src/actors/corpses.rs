//! The corpses: the dead Torizo, the dead sidehopper the baby Metroid drains, the
//! dead zoomers, rippers and skrees in Tourian, and the corpse rotting that
//! crumbles them (and Mother Brain's brain) row by row.
//!
//! Was `DeadTorizo*`, `DeadSidehopper*`, `DeadZoomer*`, `DeadRipper*`,
//! `DeadSkree*`, `DeadMonsters*` and `*CorpseRotting*` in the original
//! decompilation (bank $A9:D2xx-$A9:ED00). Slot words are named after the
//! bank log's corpse rotting notes. The bank's dispatch ids are unchanged; the
//! functions and their reads and writes keep the original order. Mother
//! Brain's shared helpers come from `mother_brain_fight`.
use crate::actors::host::{Death, Slopes};
use crate::actors::layouts::part_view;
use crate::actors::mother_brain_fight::{
    BANK, BANK_BASE, Lease as FightLease, State, cgram, sound,
};
use crate::actors::mother_brain_fight::{
    add_spritemap_to_oam, bank_unreachable, corpse_rotting_finished, move_enemy_with_velocity,
    native_word, queue_sfx2_max3, queue_sfx2_max6, set_enemy_instruction_list,
    set_graphics_drawn_hook, set_native_word, spawn_room_projectile,
};
use crate::actors::session_slots::session_slot;
use crate::runtime_access::SessionRam;
// The game's traits, for the calls made on it through `foreign`.
use crate::actors::host::ActorRuntime as _;

use crate::constants::bank_86::*;
use crate::constants::bank_a9::*;
use crate::constants::enemy_properties::*;
use crate::enemy_script::EnemyScript;
use crate::menus::menu_tilemap::native_record;
use crate::menus::prelude::do_until;
use crate::palette_copies::copy_rom_colors_to_palette;
use crate::ram::word;
use crate::word_transfers;
use crate::{corpse_tables, torizo_corpse_tables};
use sm_data::cartridge::RomRecord;

/// What the corpses reach beyond Mother Brain's fight.
pub trait Host: crate::actors::mother_brain_fight::Host {
    /// Whether Samus touches one of the dead Torizo's hitboxes.
    fn dead_torizo_contact(&mut self) -> bool;
}

/// A Mother Brain lease whose game is this module's host.
pub trait Lease: FightLease<Game: Host> {}
impl<T: FightLease<Game: Host> + ?Sized> Lease for T {}

/// The bank $A9 functions these slots run, named after the bank log's headings.
pub mod function {
    /// $A9:D3AD: Torizo corpse function - wait for Samus collision.
    pub const TORIZO_WAIT_FOR_SAMUS_COLLISION: u16 = 0xd3ad;
    pub const TORIZO_ROTTED: u16 = 0xd3c7;
    /// $A9:D3C8: Torizo corpse function - pre-rot delay.
    pub const TORIZO_PRE_ROT_DELAY: u16 = 0xd3c8;
    /// $A9:D3E6: Torizo corpse function - rotting.
    pub const TORIZO_ROTTING: u16 = 0xd3e6;
    /// $A9:D8E2: Sidehopper corpse function - alive - waiting for activation.
    pub const SIDEHOPPER_ALIVE_WAITING_FOR_ACTIVATION: u16 = 0xd8e2;
    /// $A9:D8F1: Sidehopper corpse function - hopping.
    pub const SIDEHOPPER_HOPPING: u16 = 0xd8f1;
    pub const SIDEHOPPER_HOPPING_IDLE: u16 = 0xd90f;
    /// $A9:D910: Sidehopper corpse function - start idling.
    pub const SIDEHOPPER_START_IDLING: u16 = 0xd910;
    /// $A9:D91D: Sidehopper corpse function - idling.
    pub const SIDEHOPPER_IDLING: u16 = 0xd91d;
    /// $A9:DA08: Sidehopper corpse function - being drained.
    pub const SIDEHOPPER_BEING_DRAINED: u16 = 0xda08;
    /// $A9:DA63: RTS.
    pub const CORPSE_ROTTED: u16 = 0xda63;
    /// $A9:DA64: Sidehopper corpse function - dead - wait for Samus collision.
    pub const SIDEHOPPER_DEAD_WAIT_FOR_SAMUS_COLLISION: u16 = 0xda64;
    /// $A9:DA69: Zoomer corpse function - wait for Samus collision.
    pub const GEEMER_WAIT_FOR_SAMUS_COLLISION: u16 = 0xda69;
    /// $A9:DA6E: Skree corpse function - wait for Samus collision.
    pub const SKREE_WAIT_FOR_SAMUS_COLLISION: u16 = 0xda6e;
    /// $A9:DA73: Ripper corpse function - wait for Samus collision.
    pub const RIPPER_WAIT_FOR_SAMUS_COLLISION: u16 = 0xda73;
    /// $A9:DA8F: Sidehopper corpse function - pre-rot delay.
    pub const SIDEHOPPER_PRE_ROT_DELAY: u16 = 0xda8f;
    /// $A9:DA94: Zoomer corpse function - pre-rot delay.
    pub const GEEMER_PRE_ROT_DELAY: u16 = 0xda94;
    /// $A9:DA99: Ripper corpse function - pre-rot delay.
    pub const RIPPER_PRE_ROT_DELAY: u16 = 0xda99;
    /// $A9:DA9E: Skree corpse function - pre-rot delay.
    pub const SKREE_PRE_ROT_DELAY: u16 = 0xda9e;
    /// $A9:DABA: Sidehopper corpse function - rotting.
    pub const SIDEHOPPER_ROTTING: u16 = 0xdaba;
    /// $A9:DAD0: Zoomer corpse function - rotting.
    pub const GEEMER_ROTTING: u16 = 0xdad0;
    /// $A9:DAE6: Ripper corpse function - rotting.
    pub const RIPPER_ROTTING: u16 = 0xdae6;
    /// $A9:DAFC: Skree corpse function - rotting.
    pub const SKREE_ROTTING: u16 = 0xdafc;
}

// Each named function is the one its dispatch entry runs.
const _: () = {
    use function as f;
    const fn at(word: u16) -> u32 {
        0xa9_0000 | word as u32
    }
    assert!(at(f::TORIZO_WAIT_FOR_SAMUS_COLLISION) == 11129773);
    assert!(at(f::TORIZO_ROTTED) == 11129799);
    assert!(at(f::TORIZO_PRE_ROT_DELAY) == 11129800);
    assert!(at(f::TORIZO_ROTTING) == 11129830);
    assert!(at(f::SIDEHOPPER_ALIVE_WAITING_FOR_ACTIVATION) == 11131106);
    assert!(at(f::SIDEHOPPER_HOPPING) == 11131121);
    assert!(at(f::SIDEHOPPER_HOPPING_IDLE) == 11131151);
    assert!(at(f::SIDEHOPPER_START_IDLING) == 11131152);
    assert!(at(f::SIDEHOPPER_IDLING) == 11131165);
    assert!(at(f::SIDEHOPPER_BEING_DRAINED) == 11131400);
    assert!(at(f::CORPSE_ROTTED) == 11131491);
    assert!(at(f::SIDEHOPPER_DEAD_WAIT_FOR_SAMUS_COLLISION) == 11131492);
    assert!(at(f::GEEMER_WAIT_FOR_SAMUS_COLLISION) == 11131497);
    assert!(at(f::SKREE_WAIT_FOR_SAMUS_COLLISION) == 11131502);
    assert!(at(f::RIPPER_WAIT_FOR_SAMUS_COLLISION) == 11131507);
    assert!(at(f::SIDEHOPPER_PRE_ROT_DELAY) == 11131535);
    assert!(at(f::GEEMER_PRE_ROT_DELAY) == 11131540);
    assert!(at(f::RIPPER_PRE_ROT_DELAY) == 11131545);
    assert!(at(f::SKREE_PRE_ROT_DELAY) == 11131550);
    assert!(at(f::SIDEHOPPER_ROTTING) == 11131578);
    assert!(at(f::GEEMER_ROTTING) == 11131600);
    assert!(at(f::RIPPER_ROTTING) == 11131622);
    assert!(at(f::SKREE_ROTTING) == 11131644);
};

part_view! {
    /// A corpse's view of its slot: the dead sidehopper's hop and drain, the
    /// pre-rot delay and the corpse rotting state at $7E:8802-$7E:8837. Slot
    /// 0's $7E:8802-$7E:8813 hold the working copy for the corpse rotting.
    Corpse: EnemyLayout72Late {
        words: [
            function, pre_rot_delay, var_c, var_d, var_e, function_timer, parameter_1,
            parameter_2,
        ],
        pad1: [u8; 26696],
        words: [
            var_00, var_01, var_02, var_03, var_04, var_05,
            // $7E:780C: the sidehopper's hop index.
            hop,
            // $7E:780E and $7E:7810: the drained palette timer and index (8
            // and over: drained).
            drained_palette_timer, drained_palette_index, var_09,
            // $7E:7814 and $7E:7816: the hop's velocities.
            x_velocity, y_velocity,
            var_0c, var_0d, var_0e, var_0f, var_10, var_11,
            var_12, var_13, var_14, var_15, var_16, var_17,
            var_18, var_19, var_1a, var_1b, var_1c, var_1d,
            var_1e, var_1f,
        ],
        pad3: [u8; 4032],
        words: [
            var_40,
            // $7E:8802: the rot entry being copied or moved.
            rot_entry,
            // $7E:8804-$7E:8813 (slot 0): the working copy of $7E:8828-$7E:8837.
            active_copy_function, active_move_function, active_row_offsets, active_height,
            active_last_row, active_second_last_row, active_next_row_offset,
            active_finished_hook,
            var_4a, var_4b, var_4c, var_4d, var_4e, var_4f, var_50, var_51,
            // $7E:8824: the rot table pointer; $7E:8826: the VRAM transfers.
            rot_table, vram_transfers,
            // $7E:8828 and $7E:882A: the rot entry copy and move functions.
            copy_function, move_function,
            // $7E:882C: the tile data row offsets; $7E:882E: the sprite height
            // and $7E:8830/$7E:8832 that less one and two.
            row_offsets, height, last_row, second_last_row,
            // $7E:8834: from pixel row 6 of a tile row to row 0 of the next.
            next_row_offset,
            // $7E:8836: the rot entry finished hook.
            finished_hook,
            var_5c, var_5d, var_5e, var_5f,
        ],
    }
}

part_view! {
    /// The dead Torizo's view of slot 0.
    DeadTorizo: EnemyLayout40 {
        words: [
            function, x_velocity, y_velocity, var_d, var_e, var_f, parameter_1, parameter_2,
        ],
        pad1: [u8; 26696],
        words: [
            // $7E:7800: odd and even frames alternate the tile transfers.
            tile_frame, var_01,
            // $7E:7804: sand heap lines left to copy; $7E:7806: the rot timer.
            sand_heap_lines, rot_timer,
            // $7E:7808: frames before rotting starts.
            pre_rot_delay,
            var_05, var_06, var_07, var_08, var_09, var_0a, var_0b,
            var_0c, var_0d, var_0e, var_0f, var_10, var_11,
            var_12, var_13, var_14, var_15, var_16, var_17,
            var_18, var_19, var_1a, var_1b, var_1c, var_1d,
            var_1e, var_1f,
        ],
    }
}

part_view! {
    /// Slot 0's sprite tile transfer entry at $7E:8004, which the corpses
    /// share with Mother Brain.
    CorpseTiles: EnemyLayout40Late {
        words: [var_a, var_b, var_c, var_d, var_e, var_f, parameter_1, parameter_2],
        pad2: [u8; 28744],
        words: [
            var_20, var_21, sprite_tile_transfer, var_23, var_24, var_25,
            var_26, var_27, var_28, var_29, var_2a, var_2b,
            var_2c, var_2d, var_2e, var_2f, var_30, var_31,
            var_32, var_33, var_34, var_35, var_36, var_37,
            var_38, var_39, var_3a, var_3b, var_3c, var_3d,
            var_3e, var_3f,
        ],
    }
}

/// $B7:A800, $B7:C000 and $B7:CE00: the dead Torizo's, the dead monsters' and
/// Mother Brain's corpse tile data.
const TORIZO_CORPSE_TILES: u16 = 0xa800;
const DEAD_MONSTERS_TILES: u16 = 0xc000;
const MOTHER_BRAIN_CORPSE_TILES: u16 = 0xce00;

/// $A9:D39A: the graphics drawn hook that draws the dead Torizo's sand heap
/// spritemap; the id `actor_species_draw_hook` dispatches on.
pub const DRAW_DEAD_TORIZO_HOOK: u32 = 0xa9_d39a;
const _: () = assert!(DRAW_DEAD_TORIZO_HOOK == 11129754);

/// A corpse's tile rows for its rot entry copy and move functions
/// ($A9:E272-$A9:EB0B): the tile buffer, the rot entry below which a row is
/// copied, each row's first rot entry (0: from the start), and whether the
/// rows are indexed by byte rather than by word.
#[derive(Clone, Copy)]
struct RotTiles {
    base: usize,
    copy_limit: u16,
    row_gates: &'static [u16],
    byte_indices: bool,
}

const TORIZO_ROT_TILES: RotTiles = RotTiles {
    base: 0x2000,
    copy_limit: 94,
    row_gates: &[80, 72, 16, 0, 0, 0, 0, 0, 0, 16],
    byte_indices: false,
};
const SIDEHOPPER_0_ROT_TILES: RotTiles = RotTiles {
    base: 0x2000,
    copy_limit: 38,
    row_gates: &[8, 8, 0, 0, 0],
    byte_indices: false,
};
const SIDEHOPPER_2_ROT_TILES: RotTiles = RotTiles {
    base: 0x2320,
    copy_limit: 38,
    row_gates: &[0, 0, 8, 8, 8],
    byte_indices: false,
};
const MOTHER_BRAIN_ROT_TILES: RotTiles = RotTiles {
    base: 0x9000,
    copy_limit: 46,
    row_gates: &[16, 8, 0, 0, 0, 8, 32],
    byte_indices: true,
};

/// A three-row corpse (zoomers and rippers) at `base`.
const fn small_rot_tiles(base: usize) -> RotTiles {
    RotTiles {
        base,
        copy_limit: 14,
        row_gates: &[0, 0, 0],
        byte_indices: false,
    }
}
/// A two-row skree corpse at `base`.
const fn skree_rot_tiles(base: usize) -> RotTiles {
    RotTiles {
        base,
        copy_limit: 30,
        row_gates: &[0, 0],
        byte_indices: false,
    }
}

/// Run the corpse rotting rot entry move (clearing the source) or copy
/// function `ea` for rot entry `k` to `j`.
pub fn run_rot_entry_copy_or_move(ea: u32, k: u16, j: u16, game: &mut impl Lease) {
    let (tiles, clear_source) = match ea {
        11133554 => (TORIZO_ROT_TILES, true),
        11133835 => (TORIZO_ROT_TILES, false),
        11134056 => (SIDEHOPPER_0_ROT_TILES, true),
        11134197 => (SIDEHOPPER_0_ROT_TILES, false),
        11134308 => (SIDEHOPPER_2_ROT_TILES, true),
        11134454 => (SIDEHOPPER_2_ROT_TILES, false),
        11134570 => (small_rot_tiles(0x2940), true),
        11134649 => (small_rot_tiles(0x2940), false),
        11134710 => (small_rot_tiles(0x2a00), true),
        11134789 => (small_rot_tiles(0x2a00), false),
        11134850 => (small_rot_tiles(0x2ac0), true),
        11134929 => (small_rot_tiles(0x2ac0), false),
        11134990 => (small_rot_tiles(0x2b80), true),
        11135069 => (small_rot_tiles(0x2b80), false),
        11135130 => (small_rot_tiles(0x2c40), true),
        11135209 => (small_rot_tiles(0x2c40), false),
        11135270 => (skree_rot_tiles(0x2640), true),
        11135323 => (skree_rot_tiles(0x2640), false),
        11135364 => (skree_rot_tiles(0x2740), true),
        11135417 => (skree_rot_tiles(0x2740), false),
        11135458 => (skree_rot_tiles(0x2840), true),
        11135511 => (skree_rot_tiles(0x2840), false),
        11135552 => (MOTHER_BRAIN_ROT_TILES, true),
        11135755 => (MOTHER_BRAIN_ROT_TILES, false),
        _ => {
            bank_unreachable(game);
            return;
        }
    };
    transfer_corpse_tiles(k, j, &tiles, clear_source, game);
}
pub fn run_corpse_rotting_initialization(ea: u32, game: &mut impl Lease) {
    match ea {
        11132440 => initialize_corpse_tiles(
            TORIZO_CORPSE_TILES,
            0x2000,
            &[
                (0x60, 0x120, 0xc0),
                (0x1a0, 0x320, 0xc0),
                (0x2c0, 0x500, 0x100),
                (0x400, 0x700, 0x100),
                (0x540, 0x900, 0x100),
                (0x680, 0xb00, 0x100),
                (0x7c0, 0xd00, 0x100),
                (0x900, 0xf00, 0x100),
                (0xa40, 0x1100, 0x100),
                (0xb60, 0x12e0, 0x120),
                (0xc80, 0x14c0, 0x140),
                (0xdc0, 0x16c0, 0x140),
            ],
            game,
        ),
        11132609 => initialize_corpse_tiles(
            DEAD_MONSTERS_TILES,
            0x2000,
            &[
                (0x40, 0x40, 0x60),
                (0xa0, 0x200, 0xa0),
                (0x140, 0x400, 0xa0),
                (0x1e0, 0x600, 0xa0),
                (0x280, 0x800, 0xa0),
            ],
            game,
        ),
        11132680 => initialize_corpse_tiles(
            DEAD_MONSTERS_TILES,
            0x2000,
            &[
                (0x320, 0x120, 0x40),
                (0x3c0, 0x320, 0xa0),
                (0x460, 0x520, 0xa0),
                (0x500, 0x720, 0xa0),
                (0x5a0, 0x920, 0xa0),
            ],
            game,
        ),
        11132751 => initialize_corpse_tiles(
            DEAD_MONSTERS_TILES,
            0x2000,
            &[(0x940, 0xa60, 0x60), (0x9a0, 0xc60, 0x60)],
            game,
        ),
        11132780 => initialize_corpse_tiles(
            DEAD_MONSTERS_TILES,
            0x2000,
            &[(0xa00, 0xac0, 0x60), (0xa60, 0xcc0, 0x60)],
            game,
        ),
        11132809 => initialize_corpse_tiles(
            DEAD_MONSTERS_TILES,
            0x2000,
            &[(0xac0, 0xb20, 0x60), (0xb20, 0xd20, 0x60)],
            game,
        ),
        11132838 => initialize_corpse_tiles(
            DEAD_MONSTERS_TILES,
            0x2000,
            &[(0xb80, 0xa00, 0x60), (0xbe0, 0xc00, 0x60)],
            game,
        ),
        11132867 => initialize_corpse_tiles(
            DEAD_MONSTERS_TILES,
            0x2000,
            &[(0xc40, 0xb80, 0x60), (0xca0, 0xd80, 0x60)],
            game,
        ),
        11132896 => initialize_corpse_tiles(
            DEAD_MONSTERS_TILES,
            0x2000,
            &[
                (0x640, 0x2a0, 0x40),
                (0x680, 0x4a0, 0x40),
                (0x6c0, 0x6a0, 0x40),
                (0x700, 0x8a0, 0x40),
            ],
            game,
        ),
        11132953 => initialize_corpse_tiles(
            DEAD_MONSTERS_TILES,
            0x2000,
            &[
                (0x740, 0xe0, 0x40),
                (0x780, 0x2e0, 0x40),
                (0x7c0, 0x4e0, 0x40),
                (0x800, 0x6e0, 0x40),
            ],
            game,
        ),
        11133010 => initialize_corpse_tiles(
            DEAD_MONSTERS_TILES,
            0x2000,
            &[
                (0x840, 0x1c0, 0x40),
                (0x880, 0x3c0, 0x40),
                (0x8c0, 0x5c0, 0x40),
                (0x900, 0x7c0, 0x40),
            ],
            game,
        ),
        11133067 => initialize_corpse_tiles(
            MOTHER_BRAIN_CORPSE_TILES,
            0x9000,
            &[
                (0x0, 0xc0, 0xc0),
                (0xe0, 0x2c0, 0xc0),
                (0x1c0, 0x4c0, 0xc0),
                (0x2a0, 0x6c0, 0xc0),
                (0x380, 0x8c0, 0xe0),
                (0x460, 0xac0, 0xe0),
            ],
            game,
        ),
        _ => {
            bank_unreachable(game);
        }
    }
}
pub fn run_rot_entry_finished_hook(ea: u32, game: &mut impl Lease) {
    match ea {
        11130301 => torizo_corpse_rotting_finished(game),
        11131912 => normal_corpse_rotting_finished(game),
        11121187 => corpse_rotting_finished(game),
        _ => {
            bank_unreachable(game);
        }
    }
}
crate::dispatch::instruction_callbacks! {
    pub fn (game: &mut impl Lease);
    @prefix {};
    torizo_corpse_rotting_finished [] {
        {
            let (x, y) = (((word!(game.RANDOM_NUMBER) as i32 & 0x1f) + 272) as u16, 188);
            game.foreign(|game| crate::actors::host::BossEffects::set_projectile_spawn_point(game, x, y));
        }
        spawn_room_projectile(ADDR_K_EPROJ_DUST_CLOUD_EXPLOSION as u16, 0xa, game);
        if (word!(game.RANDOM_ENEMY_COUNTER) & 7) == 0 {
            queue_sfx2_max6(sound::LAVA_ACID_DAMAGE, game);
        }
    };
    normal_corpse_rotting_finished [] {
        let v0: i32 = game.current_enemy() as i32;
        {
            let (x, y) = ((session_slot!(game, Corpse[v0].base.x_pos) as i32 + (word!(game.RANDOM_NUMBER) as i32 & 0x1a) - 14i32) as u16, ((session_slot!(game, Corpse[v0].base.y_pos)).wrapping_add(16)));
            game.foreign(|game| crate::actors::host::BossEffects::set_projectile_spawn_point(game, x, y));
        }
        spawn_room_projectile(ADDR_K_EPROJ_DUST_CLOUD_EXPLOSION as u16, 0xa, game);
        if (word!(game.RANDOM_ENEMY_COUNTER) & 7) == 0 {
            queue_sfx2_max3(sound::LAVA_ACID_DAMAGE, game);
        }
    };
}
crate::dispatch::callback_dispatch! {
    pub fn run_torizo_corpse_function(ea: u32, game: &mut impl Lease) => ea;
    call (game); default { bank_unreachable(game); };
    @body checked;
    @functions [pub fn (game: &mut impl Lease)];
    [11129800] torizo_corpse_pre_rot_delay [] {
        let e: i32 = 0;
        let v1: u16 = (session_slot!(game, DeadTorizo[e].pre_rot_delay)).wrapping_add(1);
        session_slot!(game, DeadTorizo[e].pre_rot_delay = v1);
        if v1 >= 0x10 {
            session_slot!(game, DeadTorizo[e].base.properties = (session_slot!(game, DeadTorizo[e].base.properties)) | ((K_ENEMY_PROPS_TANGIBLE) as u16));
            session_slot!(game, DeadTorizo[e].function = function::TORIZO_ROTTING);
            torizo_corpse_rotting(game);
        }
    };
    [11129830] torizo_corpse_rotting [] {
        let e: i32 = 0;
        let v1: u16 = (session_slot!(game, DeadTorizo[e].rot_timer)).wrapping_add(1);
        session_slot!(game, DeadTorizo[e].rot_timer = v1);
        if v1 >= 0xf {
            session_slot!(game, DeadTorizo[e].rot_timer = 0);
            let sand_heap_lines: u16 = session_slot!(game, DeadTorizo[e].sand_heap_lines);
            if sand_heap_lines != 0 {
                copy_line_of_sand_heap_tile_data(sand_heap_lines, game);
                session_slot!(game, DeadTorizo[e].sand_heap_lines = session_slot!(game, DeadTorizo[e].sand_heap_lines).wrapping_sub(1));
            }
        }
        session_slot!(game, DeadTorizo[e].y_velocity = session_slot!(game, DeadTorizo[e].y_velocity).wrapping_add(1));
        move_enemy_with_velocity(game);
        if process_corpse_rotting(0, game) & 1 == 0 {
            session_slot!(game, DeadTorizo[e].function = function::TORIZO_ROTTED);
        }
    };
    @external {
        11129773 => [{
                    torizo_corpse_wait_for_samus_collision(game.current_enemy(), game);
                }];
        11129799 => [()];
    }
}
pub fn torizo_corpse_graphics_drawn_hook(game: &mut impl Lease) {
    add_spritemap_to_oam(ADDR_K_DEAD_TORIZO_SPRMAP_D761 as u16, 296, 187, 0, game);
}
/// Whether Samus collided with enemy `k` from any direction this frame.
fn collided_with_samus(k: u16, game: &impl State) -> bool {
    (0..4).any(|direction| k == word!(game.ENEMY_INDEX_COLLIDING_DIRS[direction]))
}
pub fn torizo_corpse_wait_for_samus_collision(k: u16, game: &mut impl State) {
    if collided_with_samus(k, game) {
        session_slot!(
            game,
            DeadTorizo[0i32].function = function::TORIZO_PRE_ROT_DELAY
        );
    }
}
macro_rules! queue_dead_torizo_tile_transfers {
    ($game:ident, $frame:expr, $tail:ident) => {
        let mut v8: u16 = 0;
        let mut v9: u16 = torizo_corpse_tables::transfer_size($game, $frame, 0);
        loop {
            let entry = usize::try_from(0xd0i32 + $tail as i32).expect("VRAM entry outside RAM");
            SessionRam::write_word_native($game, v9, entry);
            let transfer = torizo_corpse_tables::transfer($game, $frame, (v8 >> 3) as isize);
            // The source address overwrites the bank word's low byte, as in the original.
            for (offset, value) in [
                (3, transfer.source_bank),
                (2, transfer.source_address),
                (5, transfer.vram_address),
            ] {
                SessionRam::write_word_native($game, value, entry + offset);
            }
            $tail = ($tail).wrapping_add(7);
            v8 = (v8).wrapping_add(8);
            v9 = torizo_corpse_tables::transfer_size($game, $frame, (v8 >> 3) as isize);
            if (v9 == 0) {
                break;
            }
        }
    };
}

pub fn process_torizo_corpse_rotting_vram_transfers(game: &mut impl State) {
    let e: i32 = 0;
    let v1: u16 = (session_slot!(game, DeadTorizo[e].tile_frame)).wrapping_add(1);
    session_slot!(game, DeadTorizo[e].tile_frame = v1);
    let mut v3: u16 = word!(game.VRAM_WRITE_QUEUE_TAIL);
    if v1 & 1 != 0 {
        queue_dead_torizo_tile_transfers!(game, torizo_corpse_tables::Frame::Odd, v3);
    } else {
        queue_dead_torizo_tile_transfers!(game, torizo_corpse_tables::Frame::Even, v3);
    }
    session_slot!(game, CorpseTiles[0i32].sprite_tile_transfer = 0);
    word!(game.VRAM_WRITE_QUEUE_TAIL = v3);
}
pub fn copy_line_of_sand_heap_tile_data(a: u16, game: &mut impl State) {
    let line = torizo_corpse_tables::sand_heap_line(game, a as isize);
    let v1: u16 = line.destination;
    let v2: i32 = line.source as i32 >> 1;
    // 18 words, eight words apart in the heap and 16 bytes apart in the buffer.
    for step in 0..18 {
        let value = crate::sand_heap::word(game, (v2 + 8 * step) as usize);
        let at = 0x9500 + 0x10 * step as usize + usize::from(v1);
        SessionRam::write_word_native(game, value, at);
    }
}
pub fn initialize_alive_sidehopper_corpse(game: &mut impl Lease) {
    let e: i32 = game.current_enemy() as i32;
    session_slot!(
        game,
        Corpse[e].base.properties = (session_slot!(game, Corpse[e].base.properties) as i32 & 0x77ff
            | K_ENEMY_PROPS_PROCESSED_OFFSCREEN as i32) as u16
    );
    if (session_slot!(game, Corpse[0i32].base.properties) as i32 & K_ENEMY_PROPS_INVISIBLE as i32)
        != 0
    {
        session_slot!(
            game,
            Corpse[e].base.properties =
                (session_slot!(game, Corpse[e].base.properties)) | ((K_ENEMY_PROPS_DELETED) as u16)
        );
    }
    session_slot! { game, Corpse[e]; drained_palette_index = 0, x_velocity = 96, y_velocity = 256 }
    session_slot!(game, Corpse[e].base.x_pos = 488);
    session_slot!(game, Corpse[e].base.y_pos = 184);
    session_slot!(
        game,
        Corpse[e].function = function::SIDEHOPPER_ALIVE_WAITING_FOR_ACTIVATION
    );
    session_slot!(game, Corpse[e].base.palette_index = 512);
    session_slot!(game, Corpse[e].base.y_height = 21);
    set_enemy_instruction_list(
        game.current_enemy(),
        ADDR_K_DEAD_MONSTERS_ILIST_ECE3 as u16,
        game,
    );
    initialize_enemy_corpse_rotting(game.current_enemy(), ADDR_STRU_A9DD68 as u16, game);
}
pub fn initialize_dead_sidehopper_corpse(game: &mut impl Lease) {
    let e: i32 = game.current_enemy() as i32;
    session_slot!(game, Corpse[e].drained_palette_index = -1i32 as u16);
    session_slot!(
        game,
        Corpse[e].function = function::SIDEHOPPER_DEAD_WAIT_FOR_SAMUS_COLLISION
    );
    session_slot!(game, Corpse[e].base.palette_index = 3584);
    set_enemy_instruction_list(
        game.current_enemy(),
        ADDR_K_DEAD_MONSTERS_ILIST_ECEF as u16,
        game,
    );
    initialize_enemy_corpse_rotting(game.current_enemy(), ADDR_STRU_A9DD78 as u16, game);
}

macro_rules! initialize_rotting_enemy_corpse {
    (
        $enemy:ident, $function:expr, $game:ident, $corpse:expr $(,)?
    ) => {
        session_slot!($game, Corpse[$enemy].base.palette_index = 3584);
        session_slot!($game, Corpse[$enemy].function = $function);
        let v2: i32 = session_slot!($game, Corpse[$enemy].parameter_1) as i32 >> 1;
        set_enemy_instruction_list(
            $game.current_enemy(),
            corpse_tables::variant($game, $corpse, v2 as isize).instruction_list,
            $game,
        );
        initialize_enemy_corpse_rotting(
            $game.current_enemy(),
            corpse_tables::variant($game, $corpse, v2 as isize).rotting,
            $game,
        );
    };
}

crate::dispatch::callback_dispatch! {
    pub fn run_dead_monster_function(ea: u32, k: u16, game: &mut impl Lease) => ea;
    call (k, game); default { bank_unreachable(game); };
    @body checked;
    @functions [pub fn (k: u16, game: &mut impl Lease)];
    [11131106] sidehopper_corpse_wait_for_activation [] {
        if (game.camera().layer1[0] as i32 - 513i32) & 0x8000 != 0 {
            session_slot!(game, Corpse[k as i32].function = function::SIDEHOPPER_HOPPING);
            sidehopper_corpse_hopping(k, game);
        }
    };
    [11131121] sidehopper_corpse_hopping [] {
        if sidehopper_corpse_movement(k, game) & 1 != 0 {
            let e: i32 = k as i32;
            session_slot!(game, Corpse[e].hop = ((session_slot!(game, Corpse[e].hop) as i32 + 1) & 3) as u16);
            set_enemy_instruction_list(k, ADDR_K_DEAD_MONSTERS_ILIST_ECAC as u16, game);
            session_slot!(game, Corpse[e].function = function::SIDEHOPPER_HOPPING_IDLE);
        }
    };
    [11131152] sidehopper_corpse_start_idling [] {
        let e: i32 = k as i32;
        session_slot! { game, Corpse[e]; function = function::SIDEHOPPER_IDLING, function_timer = 64 }
    };
    [11131165] sidehopper_corpse_idling [] {
        let e: i32 = k as i32;
        session_slot!(game, Corpse[e].function_timer = session_slot!(game, Corpse[e].function_timer).wrapping_sub(1));
        if (session_slot!(game, Corpse[e].function_timer) & 0x8000) != 0 {
            if session_slot!(game, Corpse[e].drained_palette_index) != 0 {
                session_slot!(game, Corpse[e].function = function::SIDEHOPPER_BEING_DRAINED);
            } else {
                session_slot!(game, Corpse[e].function = function::SIDEHOPPER_HOPPING);
                set_enemy_instruction_list(k, ADDR_K_DEAD_MONSTERS_ILIST_ECE3 as u16, game);
                let v3: i32 = session_slot!(game, Corpse[e].hop) as i32;
                let hop = corpse_tables::hop(game, v3 as isize);
                session_slot! { game, Corpse[e]; y_velocity = hop.y_velocity, x_velocity = hop.x_velocity }
            }
        }
    };
    [11131400] sidehopper_corpse_being_drained [] {
        let e: i32 = k as i32;
        let v2: u16 = (session_slot!(game, Corpse[e].drained_palette_timer)).wrapping_add(1);
        session_slot!(game, Corpse[e].drained_palette_timer = v2);
        if v2 >= 8 {
            session_slot!(game, Corpse[e].drained_palette_timer = 0);
            copy_rom_colors_to_palette(
                cgram::SPRITE_PALETTE_1,
                BANK,
                (32 * (session_slot!(game, Corpse[e].drained_palette_index) as i32 - 1i32) - 0x1434i32) as u16,
                0xf,
                game,
            );
            let v3: u16 = game.current_enemy();
            let v5: u16 = (session_slot!(game, Corpse[e].drained_palette_index)).wrapping_add(1);
            session_slot!(game, Corpse[e].drained_palette_index = v5);
            if v5 >= 8 {
                set_enemy_instruction_list(v3, ADDR_K_DEAD_MONSTERS_ILIST_ECE9 as u16, game);
                session_slot!(game, Corpse[e].function = function::SIDEHOPPER_DEAD_WAIT_FOR_SAMUS_COLLISION);
                session_slot!(game, Corpse[e].base.properties = (session_slot!(game, Corpse[e].base.properties)) | (0x8000u16));
                session_slot!(game, Corpse[e].base.y_height = 12);
            }
        }
    };
    [11131578] sidehopper_corpse_rotting [] {
        let v1: u8 = process_corpse_rotting(k, game);
        if v1 & 1 == 0 {
            session_slot!(game, Corpse[game.current_enemy() as i32].function = function::SIDEHOPPER_DEAD_WAIT_FOR_SAMUS_COLLISION);
        }
        let vram_transfers: u16 = session_slot!(game, Corpse[game.current_enemy() as i32].vram_transfers);
        process_corpse_rotting_vram_transfers(vram_transfers, game);
    };
    [11131600] geemer_rotting [] {
        let v1: u8 = process_corpse_rotting(k, game);
        if v1 & 1 == 0 {
            session_slot!(game, Corpse[game.current_enemy() as i32].function = function::CORPSE_ROTTED);
        }
        let vram_transfers: u16 = session_slot!(game, Corpse[game.current_enemy() as i32].vram_transfers);
        process_corpse_rotting_vram_transfers(vram_transfers, game);
    };
    @forward [pub fn (k: u16, game: &mut impl State)];
    [11131492]
    sidehopper_corpse_wait_for_samus_collision => corpse_wait_for_samus_collision(k, function::SIDEHOPPER_PRE_ROT_DELAY, game);
    [11131497]
    geemer_wait_for_samus_collision => corpse_wait_for_samus_collision(k, function::GEEMER_PRE_ROT_DELAY, game);
    [11131502]
    skree_corpse_wait_for_samus_collision => corpse_wait_for_samus_collision(k, function::SKREE_PRE_ROT_DELAY, game);
    [11131507]
    ripper_corpse_wait_for_samus_collision => corpse_wait_for_samus_collision(k, function::RIPPER_PRE_ROT_DELAY, game);
    @forward [pub fn (k: u16, game: &mut impl State)];
    [11131535]
    sidehopper_corpse_pre_rot_delay => corpse_pre_rot_delay(k, function::SIDEHOPPER_ROTTING, game);
    [11131540]
    geemer_pre_rot_delay => corpse_pre_rot_delay(k, function::GEEMER_ROTTING, game);
    [11131545]
    ripper_corpse_pre_rot_delay => corpse_pre_rot_delay(k, function::RIPPER_ROTTING, game);
    [11131550]
    skree_corpse_pre_rot_delay => corpse_pre_rot_delay(k, function::SKREE_ROTTING, game);
    @external {
        11131644 => skree_corpse_rotting; 11131622 => ripper_corpse_rotting;
        11131151 => [()]; 11131491 => [()];
    }
}
pub fn sidehopper_corpse_movement(k: u16, game: &mut impl Lease) -> u8 {
    let e: i32 = k as i32;
    sidehopper_corpse_horizontal_movement(k, session_slot!(game, Corpse[e].x_velocity), game);
    let mut v2: u16 = 32;
    if (session_slot!(game, Corpse[e].y_velocity) & 0x8000) == 0 {
        v2 = 128;
    }
    let v3: u16 = (session_slot!(game, Corpse[e].y_velocity)).wrapping_add(v2);
    session_slot!(game, Corpse[e].y_velocity = v3);
    if (session_slot!(game, Corpse[e].base.x_pos) as i32 - 544i32) & 0x8000 != 0 {
        let full: i32 = session_slot!(byte game, Corpse[e].base.y_subpos[1]) as i32
            + v3.to_ne_bytes()[0] as i32;
        session_slot!(byte game, Corpse[e].base.y_subpos[1] = full as u8);
        let v12: u16 = (session_slot!(game, Corpse[e].base.y_pos) as i32
            + (v3 >> 8) as i8 as i32
            + (full >> 8)) as u16;
        session_slot!(game, Corpse[e].base.y_pos = v12);
        (((v12 as i32 - 184i32) & 0x8000) == 0) as u8
    } else {
        {
            let (actor, amount) = (k, (v3 as i16 as i32) << 8);
            game.foreign(|game| game.move_vertical(actor, amount)) as u8
        }
    }
}
pub fn sidehopper_corpse_horizontal_movement(k: u16, a: u16, game: &mut impl Lease) {
    let e: i32 = k as i32;
    if (session_slot!(game, Corpse[e].base.x_pos) as i32 - 544i32) & 0x8000 != 0 {
        let full: i32 =
            session_slot!(byte game, Corpse[e].base.x_subpos[1]) as i32 + a.to_ne_bytes()[0] as i32;
        session_slot!(byte game, Corpse[e].base.x_subpos[1] = full as u8);
        session_slot!(
            game,
            Corpse[e].base.x_pos = (session_slot!(game, Corpse[e].base.x_pos) as i32
                + ((a >> 8) as i8 as i32 + (full >> 8))) as u16
        );
    } else {
        {
            let (actor, amount) = (k, (a as i16 as i32) << 8);
            game.foreign(|game| game.move_horizontal(actor, amount, Slopes::Ignore)) as u8
        };
    };
}
pub fn corpse_wait_for_samus_collision(k: u16, j: u16, game: &mut impl State) {
    if collided_with_samus(k, game) {
        session_slot!(game, Corpse[k as i32].function = j);
    }
}
pub fn corpse_pre_rot_delay(k: u16, j: u16, game: &mut impl State) {
    let e: i32 = k as i32;
    session_slot!(
        game,
        Corpse[e].pre_rot_delay = session_slot!(game, Corpse[e].pre_rot_delay).wrapping_add(1)
    );
    if session_slot!(game, Corpse[e].pre_rot_delay) >= 0x10 {
        session_slot!(game, Corpse[e].function = j);
        session_slot!(
            game,
            Corpse[e].base.properties = (session_slot!(game, Corpse[e].base.properties))
                | ((K_ENEMY_PROPS_TANGIBLE) as u16)
        );
    }
}
pub use self::geemer_rotting as ripper_corpse_rotting;
pub use self::geemer_rotting as skree_corpse_rotting;
pub fn process_corpse_rotting(k: u16, game: &mut impl Lease) -> u8 {
    let ek: i32 = k as i32;
    let mut e0: i32 = 0;
    session_slot! { game, Corpse[e0]; active_height = session_slot!(game, Corpse[ek].height), active_last_row = session_slot!(game, Corpse[ek].last_row), active_second_last_row = session_slot!(game, Corpse[ek].second_last_row), active_next_row_offset = session_slot!(game, Corpse[ek].next_row_offset), active_finished_hook = session_slot!(game, Corpse[ek].finished_hook), active_row_offsets = session_slot!(game, Corpse[ek].row_offsets), active_copy_function = session_slot!(game, Corpse[ek].copy_function), active_move_function = session_slot!(game, Corpse[ek].move_function) };
    let mut rot_entry_at: u16 = session_slot!(game, Corpse[ek].rot_table);
    let mut v4: u16 = 0;
    loop {
        let v15: u16 = v4;
        let v5: usize = rot_entry_at as usize;
        let v6 = v5;
        if (native_word(v5, game) as i16) >= 0 {
            let v7: i16 = native_word(v5 + 2, game) as i16;
            if v7 != 0 {
                let v8: i16 = (v7).wrapping_sub(1);
                set_native_word(v5 + 2, v8 as u16, game);
                if (v8 as u16) < 4 {
                    e0 = 0;
                    let mut r18: u16 = session_slot!(game, Corpse[e0].active_copy_function);
                    if v15 >= session_slot!(game, Corpse[e0].active_second_last_row) {
                        r18 = session_slot!(game, Corpse[e0].active_move_function);
                    }
                    copy_move_corpse_rotting_rot_entry(
                        native_record(
                            BANK_BASE
                                | u32::from(session_slot!(game, Corpse[e0].active_row_offsets)),
                            game,
                        ),
                        native_word(v6, game),
                        r18,
                        game,
                    );
                }
            } else {
                let e: i32 = 0;
                let r18_0: u16 = session_slot!(game, Corpse[e].active_move_function);
                let v11: usize = rot_entry_at as usize;
                copy_move_corpse_rotting_rot_entry(
                    native_record(
                        BANK_BASE | u32::from(session_slot!(game, Corpse[e].active_row_offsets)),
                        game,
                    ),
                    native_word(v11, game),
                    r18_0,
                    game,
                );
                let v12: u16 = (native_word(v11, game)).wrapping_add(2);
                if v12 < session_slot!(game, Corpse[e].active_last_row) {
                    set_native_word(v11, v12, game);
                } else {
                    run_rot_entry_finished_hook(
                        ((session_slot!(game, Corpse[e].active_finished_hook)) as u32)
                            | (BANK_BASE),
                        game,
                    );
                    if v15 >= session_slot!(game, Corpse[0i32].active_last_row) {
                        break;
                    }
                    set_native_word(v11, -1i32 as u16, game);
                }
            }
        }
        rot_entry_at = (rot_entry_at).wrapping_add(4);
        v4 = (v15).wrapping_add(1);
        e0 = 0;
        if ((v15 as i32 + 1 - session_slot!(game, Corpse[e0].active_height) as i32) as i16) >= 0 {
            return ((v15).wrapping_add(1) >= session_slot!(game, Corpse[e0].active_height)) as u8;
        }
    }
    0
}
pub fn copy_move_corpse_rotting_rot_entry(r20: RomRecord, a: u16, r18: u16, game: &mut impl Lease) {
    let e: i32 = 0;
    session_slot!(game, Corpse[e].rot_entry = a);
    let v1: u16 = ((a) & (0xfff8u16)) >> 2;
    let v2: u16 = (a) & (7u16);
    let v6: u16 = (r20.table_word((v1 >> 1) as isize) as i32 + 2 * v2 as i32) as u16;
    if v2 >= 6 {
        run_rot_entry_copy_or_move(
            ((r18) as u32) | (BANK_BASE),
            (session_slot!(game, Corpse[e].active_next_row_offset)).wrapping_add(v6),
            v6,
            game,
        );
    } else {
        run_rot_entry_copy_or_move(((r18) as u32) | (BANK_BASE), v6, v6, game);
    };
}
pub fn initialize_corpse_rotting_data_table(mut table: usize, a: u16, game: &mut impl Lease) {
    let mut v2: i16 = ((a) as i16).wrapping_sub(1);
    let mut r18: u16 = 0;
    do_until! { v2 < 0 ;
        set_native_word(table, v2 as u16, game);
        set_native_word(table + 2, r18, game);
        table += 4;
        r18 = (r18).wrapping_add(2);
        v2 -= 1;
    }
}
pub fn initialize_enemy_corpse_rotting(k: u16, j: u16, game: &mut impl Lease) {
    let v2 = native_record(BANK_BASE | u32::from(j), game);
    let e: i32 = k as i32;
    session_slot! { game, Corpse[e]; rot_table = v2.word(0), vram_transfers = v2.word(2), copy_function = v2.word(4), move_function = v2.word(6), finished_hook = v2.word(14) };
    let v4: u16 = v2.word(12);
    session_slot!(game, Corpse[e].row_offsets = v4);
    session_slot!(
        game,
        Corpse[e].next_row_offset = native_record(BANK_BASE | u32::from(v4), game)
            .word(2)
            .wrapping_sub(12)
    );
    let v5 = v2;
    let mut v6: u16 = v5.word(8);
    let fresh5 = v6;
    v6 = v6.wrapping_sub(1);
    session_slot! { game, Corpse[e]; height = fresh5, last_row = v6 }
    session_slot!(game, Corpse[e].second_last_row = (v6).wrapping_sub(1));
    initialize_corpse_rotting_data_table(v5.word(0) as usize, v5.word(8), game);
    run_corpse_rotting_initialization(((v5.word(10)) as u32) | (BANK_BASE), game);
}
pub fn process_corpse_rotting_vram_transfers(k: u16, game: &mut impl Lease) {
    let p = native_record(BANK_BASE | u32::from(k), game);
    let mut offset = 0;
    let mut v1: u16 = word!(game.VRAM_WRITE_QUEUE_TAIL);
    let mut v2: u16 = p.word(offset);
    do_until! { v2 == 0 ;
        let v3: usize = usize::try_from(0xd0i32 + v1 as i32).expect("VRAM entry outside RAM");
        set_native_word(v3, v2, game);
        set_native_word(v3 + 3, p.word(offset + 2), game);
        set_native_word(v3 + 2, p.word(offset + 4), game);
        set_native_word(v3 + 5, p.word(offset + 6), game);
        v1 = (v1).wrapping_add(7);
        offset += 8;
        v2 = p.word(offset);
    }
    session_slot!(game, CorpseTiles[0i32].sprite_tile_transfer = 0);
    word!(game.VRAM_WRITE_QUEUE_TAIL = v1);
}
pub fn sidehopper_corpse_rottable_contact(game: &mut impl State) {
    corpse_contact_reaction(function::SIDEHOPPER_ROTTING, game);
}
pub fn corpse_contact_reaction(a: u16, game: &mut impl State) {
    let e: i32 = game.current_enemy() as i32;
    session_slot!(game, Corpse[e].function = a);
    session_slot!(
        game,
        Corpse[e].base.properties = (session_slot!(game, Corpse[e].base.properties) as i32
            | (K_ENEMY_PROPS_PROCESSED_OFFSCREEN as i32 | K_ENEMY_PROPS_TANGIBLE as i32))
            as u16
    );
}
fn initialize_corpse_tiles(
    rom_address: u16,
    tile_base: usize,
    segments: &[(usize, usize, i32)],
    game: &mut impl Lease,
) {
    let source = native_record(0xb70000 | u32::from(rom_address), game);
    for &(destination_offset, source_offset, length) in segments {
        // Dispatch descriptors contain positive spans of at most 320 bytes.
        let mut bytes = [0; 320];
        for (offset, byte) in bytes[..length as usize].iter_mut().enumerate() {
            *byte = source.byte(source_offset + offset);
        }
        game.write_slice(tile_base + destination_offset, &bytes[..length as usize]);
    }
}

// Each descriptor lists row gates in transfer order. A move copies each row
// before clearing its source; a copy leaves the source intact.
fn transfer_corpse_tiles(
    j: u16,
    k: u16,
    tiles: &RotTiles,
    clear_source: bool,
    game: &mut impl Lease,
) {
    let RotTiles {
        base: tile_base,
        copy_limit,
        row_gates,
        byte_indices,
    } = *tiles;
    let destination = if byte_indices {
        j as usize
    } else {
        (j as usize >> 1) * 2
    };
    let source = if byte_indices {
        k as usize
    } else {
        (k as usize >> 1) * 2
    };
    let progress_offset = 0xf78 + core::mem::offset_of!(Corpse, rot_entry);
    for (row, &minimum) in row_gates.iter().enumerate() {
        let progress = || crate::ram::checked_word!(game, progress_offset;);
        if minimum != 0 && progress() < minimum {
            continue;
        }
        let row_base = tile_base + row * 32;
        // Preserve the original bit-15 subtraction test, including high values.
        if (progress() as i32 - copy_limit as i32) & 0x8000 != 0 {
            for plane in [0, 16] {
                let at = row_base + source + plane;
                let bytes = [SessionRam::byte(game, at), SessionRam::byte(game, at + 1)];
                game.write_slice(row_base + destination + plane + 2, &bytes);
            }
        }
        if clear_source {
            for plane in [0, 16] {
                game.write_slice(row_base + source + plane, &0u16.to_ne_bytes());
            }
        }
    }
}
pub fn initialize_torizo_corpse(game: &mut impl Lease) {
    word_transfers!(game, descending, 2048; le(TILEMAP_STUFF, 2047) <- value(0));
    let e: i32 = 0;
    session_slot!(
        game,
        DeadTorizo[e].function = function::TORIZO_WAIT_FOR_SAMUS_COLLISION
    );
    session_slot!(
        game,
        DeadTorizo[e].base.properties =
            (session_slot!(game, DeadTorizo[e].base.properties)) | (0xa000u16)
    );
    session_slot! { game, DeadTorizo[e].base; current_instruction = ADDR_K_DEAD_TORIZO_ILIST_D6DC as u16, instruction_timer = 1 }
    session_slot!(game, DeadTorizo[e].base.timer = 0);
    session_slot!(game, DeadTorizo[e].base.palette_index = 512);
    session_slot! { game, DeadTorizo[e]; x_velocity = 0, y_velocity = 8, pre_rot_delay = 0, sand_heap_lines = 15, rot_timer = 0 };
    initialize_enemy_corpse_rotting(0, ADDR_STRU_A9DD58 as u16, game);
}
/// Main AI - Torizo corpse: the frame runs on one lease.
pub fn torizo_corpse_main_ai(game: &mut impl Host) {
    torizo_corpse_frame(&mut game.lease());
}
fn torizo_corpse_frame(game: &mut impl Lease) {
    let e: i32 = 0;
    if (session_slot!(game, DeadTorizo[e].base.properties) as i32 & K_ENEMY_PROPS_TANGIBLE as i32)
        == 0
        && game.foreign(|game| game.dead_torizo_contact())
    {
        session_slot!(
            game,
            DeadTorizo[e].base.properties = (session_slot!(game, DeadTorizo[e].base.properties))
                | ((K_ENEMY_PROPS_TANGIBLE) as u16)
        );
        session_slot!(game, DeadTorizo[e].function = function::TORIZO_ROTTING);
    }
    set_graphics_drawn_hook(DRAW_DEAD_TORIZO_HOOK, game);
    run_torizo_corpse_function(
        ((session_slot!(game, DeadTorizo[e].function)) as u32) | (BANK_BASE),
        game,
    );
    process_torizo_corpse_rotting_vram_transfers(game);
}
pub fn torizo_corpse_power_bomb_reaction(game: &mut impl State) {
    if (session_slot!(game, DeadTorizo[0i32].base.properties) as i32
        & K_ENEMY_PROPS_TANGIBLE as i32)
        == 0
    {
        torizo_corpse_touch_or_shot(game);
    }
}
pub fn torizo_corpse_touch_or_shot(game: &mut impl State) {
    let e: i32 = 0;
    session_slot!(
        game,
        DeadTorizo[e].base.properties = (session_slot!(game, DeadTorizo[e].base.properties))
            | ((K_ENEMY_PROPS_TANGIBLE) as u16)
    );
    session_slot!(game, DeadTorizo[e].function = function::TORIZO_ROTTING);
}
pub fn initialize_sidehopper_corpse(game: &mut impl Lease) {
    let dsr_parameter_1: u16 = session_slot!(game, Corpse[game.current_enemy() as i32].parameter_1);
    if dsr_parameter_1 != 0 {
        if dsr_parameter_1 != 2 {
            bank_unreachable(game);
        }
        initialize_dead_sidehopper_corpse(game);
    } else {
        initialize_alive_sidehopper_corpse(game);
    };
}
pub fn initialize_geemer_corpse(game: &mut impl Lease) {
    let v1: i32 = game.current_enemy() as i32;
    initialize_rotting_enemy_corpse!(
        v1,
        function::GEEMER_WAIT_FOR_SAMUS_COLLISION,
        game,
        corpse_tables::Corpse::Geemer
    );
}
pub fn initialize_ripper_corpse(game: &mut impl Lease) {
    let v1: i32 = game.current_enemy() as i32;
    initialize_rotting_enemy_corpse!(
        v1,
        function::RIPPER_WAIT_FOR_SAMUS_COLLISION,
        game,
        corpse_tables::Corpse::Ripper
    );
}
pub fn initialize_skree_corpse(game: &mut impl Lease) {
    let v1: i32 = game.current_enemy() as i32;
    initialize_rotting_enemy_corpse!(
        v1,
        function::SKREE_WAIT_FOR_SAMUS_COLLISION,
        game,
        corpse_tables::Corpse::Skree
    );
}
pub fn sidehopper_corpse_power_bomb_reaction(game: &mut impl Lease) {
    if session_slot!(
        game,
        Corpse[game.current_enemy() as i32].drained_palette_index
    ) >= 8
    {
        sidehopper_corpse_shot(game);
    } else {
        corpse_main_ai(game);
    };
}
pub fn corpse_main_ai(game: &mut impl Lease) {
    let e: i32 = game.current_enemy() as i32;
    run_dead_monster_function(
        ((session_slot!(game, Corpse[e].function)) as u32) | (BANK_BASE),
        game.current_enemy(),
        game,
    );
}
pub fn power_bomb_reaction_geemer_corpse(game: &mut impl State) {
    if (session_slot!(game, Corpse[game.current_enemy() as i32].base.properties) as i32
        & K_ENEMY_PROPS_TANGIBLE as i32)
        == 0
    {
        touch_or_shot_geemer_corpse(game);
    }
}
pub fn touch_or_shot_geemer_corpse(game: &mut impl State) {
    corpse_contact_reaction(function::GEEMER_ROTTING, game);
}
pub fn ripper_corpse_power_bomb_reaction(game: &mut impl State) {
    if (session_slot!(game, Corpse[game.current_enemy() as i32].base.properties) as i32
        & K_ENEMY_PROPS_TANGIBLE as i32)
        == 0
    {
        ripper_corpse_touch_or_shot(game);
    }
}
pub fn ripper_corpse_touch_or_shot(game: &mut impl State) {
    corpse_contact_reaction(function::RIPPER_ROTTING, game);
}
pub fn skree_corpse_power_bomb_reaction(game: &mut impl State) {
    if (session_slot!(game, Corpse[game.current_enemy() as i32].base.properties) as i32
        & K_ENEMY_PROPS_TANGIBLE as i32)
        == 0
    {
        skree_corpse_touch_or_shot(game);
    }
}
pub fn skree_corpse_touch_or_shot(game: &mut impl State) {
    corpse_contact_reaction(function::SKREE_ROTTING, game);
}
pub fn sidehopper_corpse_shot(game: &mut impl State) {
    let e: i32 = game.current_enemy() as i32;
    if !((session_slot!(game, Corpse[e].base.properties) as i32 & K_ENEMY_PROPS_TANGIBLE as i32)
        != 0
        || session_slot!(game, Corpse[e].drained_palette_index) < 8)
    {
        sidehopper_corpse_rottable_contact(game);
    }
}
pub fn sidehopper_corpse_touch(game: &mut impl Lease) {
    if session_slot!(
        game,
        Corpse[game.current_enemy() as i32].drained_palette_index
    ) < 8
    {
        game.foreign(|game| game.contact_reaction(Death::LeavePending));
    } else {
        sidehopper_corpse_rottable_contact(game);
    };
}
pub fn sidehopper_corpse_end_hop(k: u16, jp: EnemyScript, game: &mut impl State) -> EnemyScript {
    let mut v2: u16 = function::SIDEHOPPER_START_IDLING;
    let e: i32 = k as i32;
    if session_slot!(game, Corpse[e].drained_palette_index) != 0 {
        v2 = function::SIDEHOPPER_BEING_DRAINED;
    }
    session_slot!(game, Corpse[e].function = v2);
    jp
}
