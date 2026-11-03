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
use crate::actors::mother_brain_fight::OwnedWord;
use crate::actors::mother_brain_fight::random_number;
use crate::actors::mother_brain_fight::{BANK, BANK_BASE, Host as _, Lease, State, cgram, sound};
use crate::actors::mother_brain_fight::{
    add_spritemap_to_oam, bank_unreachable, corpse_rotting_finished, move_enemy_with_velocity,
    queue_sfx2_max3, queue_sfx2_max6, set_enemy_instruction_list, set_graphics_drawn_hook,
    set_native_word, spawn_room_projectile,
};
use crate::actors::slot_view::{SlotState, slot_view};
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

slot_view! {
    /// A corpse's rotting state, at $7E:8802-$7E:8837: the rot entry being
    /// copied or moved; slot 0's working copy of the rot entry copy and move
    /// functions, row offsets, height, last and second last rows, next row
    /// offset and finished hook; the rot table pointer and VRAM transfers; the
    /// rot entry copy and move functions; the tile data row offsets; the sprite
    /// height and that less one and two; the offset from pixel row 6 of a tile
    /// row to row 0 of the next; the rot entry finished hook; and slot 0's
    /// sprite tile transfer entry at $7E:8004, which the corpses share with
    /// Mother Brain.
    Corpse, CorpseEdit {
        record: [],
        extended: [
            sprite_tile_transfer = var_22, rot_entry = var_41,
            active_copy_function = var_42, active_move_function = var_43,
            active_row_offsets = var_44, active_height = var_45, active_last_row = var_46,
            active_second_last_row = var_47, active_next_row_offset = var_48,
            active_finished_hook = var_49, rot_table = var_52, vram_transfers = var_53,
            copy_function = var_54, move_function = var_55, row_offsets = var_56,
            height = var_57, last_row = var_58, second_last_row = var_59,
            next_row_offset = var_5a, finished_hook = var_5b,
        ],
    }
}

slot_view! {
    /// A dead monster's slot (the corpse-only fields of [`Corpse`]): its
    /// function, pre-rot delay and function timer, the sidehopper's hop index,
    /// drained palette timer and index (8 and over: drained) and hop
    /// velocities, and the corpse rotting's VRAM transfers.
    DeadMonster, DeadMonsterEdit {
        record: [
            function = var_a, pre_rot_delay = var_b, function_timer = var_f,
            parameter_1 = parameter_1,
        ],
        extended: [
            hop = var_06, drained_palette_timer = var_07, drained_palette_index = var_08,
            x_velocity = var_0a, y_velocity = var_0b, vram_transfers = var_53,
        ],
    }
}

slot_view! {
    /// The dead Torizo's slot 0: its function and velocities; odd and even
    /// frames alternating the tile transfers ($7E:7800); the sand heap lines
    /// left to copy ($7E:7804), the rot timer ($7E:7806) and the frames before
    /// rotting starts ($7E:7808); and the sprite tile transfer entry it shares
    /// with Mother Brain ($7E:8004).
    DeadTorizo, DeadTorizoEdit {
        record: [function = var_a, x_velocity = var_b, y_velocity = var_c],
        extended: [
            tile_frame = var_00, sand_heap_lines = var_02, rot_timer = var_03,
            pre_rot_delay = var_04, sprite_tile_transfer = var_22,
        ],
    }
}

/// A lease the corpse-only routines run on: its slots are the Session's.
pub trait Corpses: Lease + SlotState {}
impl<T: Lease + SlotState + ?Sized> Corpses for T {}
/// The state the corpse-only reactions run on: its slots are the Session's.
pub trait CorpseState: State + SlotState {}
impl<T: State + SlotState + ?Sized> CorpseState for T {}

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
            let (x, y) = (((random_number(game) as i32 & 0x1f) + 272) as u16, 188);
            game.foreign(|game| crate::actors::host::BossEffects::set_projectile_spawn_point(game, x, y));
        }
        spawn_room_projectile(ADDR_K_EPROJ_DUST_CLOUD_EXPLOSION as u16, 0xa, game);
        if (game.owned(OwnedWord::RandomEnemyCounter) & 7) == 0 {
            queue_sfx2_max6(sound::LAVA_ACID_DAMAGE, game);
        }
    };
    normal_corpse_rotting_finished [] {
        let k = game.current_enemy();
        {
            let x_pos = Corpse::of(game, k).actor().x.whole;
            let x = (x_pos as i32 + (random_number(game) as i32 & 0x1a) - 14i32) as u16;
            let y = Corpse::of(game, k).actor().y.whole.wrapping_add(16);
            game.foreign(|game| crate::actors::host::BossEffects::set_projectile_spawn_point(game, x, y));
        }
        spawn_room_projectile(ADDR_K_EPROJ_DUST_CLOUD_EXPLOSION as u16, 0xa, game);
        if (game.owned(OwnedWord::RandomEnemyCounter) & 7) == 0 {
            queue_sfx2_max3(sound::LAVA_ACID_DAMAGE, game);
        }
    };
}
crate::dispatch::callback_dispatch! {
    pub fn run_torizo_corpse_function(ea: u32, game: &mut impl Corpses) => ea;
    call (game); default { bank_unreachable(game); };
    @body checked;
    @functions [pub fn (game: &mut impl Corpses)];
    [11129800] torizo_corpse_pre_rot_delay [] {
        let delay = DeadTorizo::of(game, 0).pre_rot_delay().wrapping_add(1);
        *DeadTorizoEdit::of(game, 0).pre_rot_delay() = delay;
        if delay >= 0x10 {
            {
                let mut torizo = DeadTorizoEdit::of(game, 0);
                torizo.actor().properties |= K_ENEMY_PROPS_TANGIBLE as u16;
                *torizo.function() = function::TORIZO_ROTTING;
            }
            torizo_corpse_rotting(game);
        }
    };
    [11129830] torizo_corpse_rotting [] {
        let timer = DeadTorizo::of(game, 0).rot_timer().wrapping_add(1);
        *DeadTorizoEdit::of(game, 0).rot_timer() = timer;
        if timer >= 0xf {
            *DeadTorizoEdit::of(game, 0).rot_timer() = 0;
            let sand_heap_lines = DeadTorizo::of(game, 0).sand_heap_lines();
            if sand_heap_lines != 0 {
                copy_line_of_sand_heap_tile_data(sand_heap_lines, game);
                let mut torizo = DeadTorizoEdit::of(game, 0);
                let lines = torizo.sand_heap_lines();
                *lines = lines.wrapping_sub(1);
            }
        }
        {
            let mut torizo = DeadTorizoEdit::of(game, 0);
            let speed = torizo.y_velocity();
            *speed = speed.wrapping_add(1);
        }
        move_enemy_with_velocity(game);
        if process_corpse_rotting(0, game) & 1 == 0 {
            *DeadTorizoEdit::of(game, 0).function() = function::TORIZO_ROTTED;
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
    (0..4).any(|direction| k == game.owned(OwnedWord::CollidingEnemy(direction)))
}
pub fn torizo_corpse_wait_for_samus_collision(k: u16, game: &mut impl CorpseState) {
    if collided_with_samus(k, game) {
        *DeadTorizoEdit::of(game, 0).function() = function::TORIZO_PRE_ROT_DELAY;
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

pub fn process_torizo_corpse_rotting_vram_transfers(game: &mut impl CorpseState) {
    let v1: u16 = DeadTorizo::of(game, 0).tile_frame().wrapping_add(1);
    *DeadTorizoEdit::of(game, 0).tile_frame() = v1;
    let mut v3: u16 = word!(game.VRAM_WRITE_QUEUE_TAIL);
    if v1 & 1 != 0 {
        queue_dead_torizo_tile_transfers!(game, torizo_corpse_tables::Frame::Odd, v3);
    } else {
        queue_dead_torizo_tile_transfers!(game, torizo_corpse_tables::Frame::Even, v3);
    }
    *DeadTorizoEdit::of(game, 0).sprite_tile_transfer() = 0;
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
pub fn initialize_alive_sidehopper_corpse(game: &mut impl Corpses) {
    let e = game.current_enemy();
    {
        let mut corpse = DeadMonsterEdit::of(game, e);
        let actor = corpse.actor();
        actor.properties = (actor.properties & 0x77ff) | K_ENEMY_PROPS_PROCESSED_OFFSCREEN as u16;
    }
    if DeadMonster::of(game, 0).actor().properties & K_ENEMY_PROPS_INVISIBLE as u16 != 0 {
        DeadMonsterEdit::of(game, e).actor().properties |= K_ENEMY_PROPS_DELETED as u16;
    }
    {
        let mut corpse = DeadMonsterEdit::of(game, e);
        *corpse.drained_palette_index() = 0;
        *corpse.x_velocity() = 96;
        *corpse.y_velocity() = 256;
        let actor = corpse.actor();
        actor.x.whole = 488;
        actor.y.whole = 184;
        *corpse.function() = function::SIDEHOPPER_ALIVE_WAITING_FOR_ACTIVATION;
        let actor = corpse.actor();
        actor.palette = 512;
        actor.radius.y = 21;
    }
    set_enemy_instruction_list(
        game.current_enemy(),
        ADDR_K_DEAD_MONSTERS_ILIST_ECE3 as u16,
        game,
    );
    initialize_enemy_corpse_rotting(game.current_enemy(), ADDR_STRU_A9DD68 as u16, game);
}
pub fn initialize_dead_sidehopper_corpse(game: &mut impl Corpses) {
    let e = game.current_enemy();
    {
        let mut corpse = DeadMonsterEdit::of(game, e);
        *corpse.drained_palette_index() = u16::MAX;
        *corpse.function() = function::SIDEHOPPER_DEAD_WAIT_FOR_SAMUS_COLLISION;
        corpse.actor().palette = 3584;
    }
    set_enemy_instruction_list(
        game.current_enemy(),
        ADDR_K_DEAD_MONSTERS_ILIST_ECEF as u16,
        game,
    );
    initialize_enemy_corpse_rotting(game.current_enemy(), ADDR_STRU_A9DD78 as u16, game);
}

/// Set up the rotting corpse `enemy` of `corpse`'s variant its first
/// parameter names, waiting in `function`.
fn initialize_rotting_enemy_corpse(
    enemy: u16,
    function: u16,
    corpse: corpse_tables::Corpse,
    game: &mut impl Corpses,
) {
    {
        let mut edit = DeadMonsterEdit::of(game, enemy);
        edit.actor().palette = 3584;
        *edit.function() = function;
    }
    let variant = (DeadMonster::of(game, enemy).parameter_1() >> 1) as isize;
    set_enemy_instruction_list(
        game.current_enemy(),
        corpse_tables::variant(game, corpse, variant).instruction_list,
        game,
    );
    initialize_enemy_corpse_rotting(
        game.current_enemy(),
        corpse_tables::variant(game, corpse, variant).rotting,
        game,
    );
}

crate::dispatch::callback_dispatch! {
    pub fn run_dead_monster_function(ea: u32, k: u16, game: &mut impl Corpses) => ea;
    call (k, game); default { bank_unreachable(game); };
    @body checked;
    @functions [pub fn (k: u16, game: &mut impl Corpses)];
    [11131106] sidehopper_corpse_wait_for_activation [] {
        if (game.camera().layer1[0] as i32 - 513i32) & 0x8000 != 0 {
            *DeadMonsterEdit::of(game, k).function() = function::SIDEHOPPER_HOPPING;
            sidehopper_corpse_hopping(k, game);
        }
    };
    [11131121] sidehopper_corpse_hopping [] {
        if sidehopper_corpse_movement(k, game) & 1 != 0 {
            {
                let mut corpse = DeadMonsterEdit::of(game, k);
                let hop = corpse.hop();
                *hop = hop.wrapping_add(1) & 3;
            }
            set_enemy_instruction_list(k, ADDR_K_DEAD_MONSTERS_ILIST_ECAC as u16, game);
            *DeadMonsterEdit::of(game, k).function() = function::SIDEHOPPER_HOPPING_IDLE;
        }
    };
    [11131152] sidehopper_corpse_start_idling [] {
        let mut corpse = DeadMonsterEdit::of(game, k);
        *corpse.function() = function::SIDEHOPPER_IDLING;
        *corpse.function_timer() = 64;
    };
    [11131165] sidehopper_corpse_idling [] {
        let timer = {
            let mut corpse = DeadMonsterEdit::of(game, k);
            let timer = corpse.function_timer();
            *timer = timer.wrapping_sub(1);
            *timer
        };
        if (timer & 0x8000) != 0 {
            if DeadMonster::of(game, k).drained_palette_index() != 0 {
                *DeadMonsterEdit::of(game, k).function() = function::SIDEHOPPER_BEING_DRAINED;
            } else {
                *DeadMonsterEdit::of(game, k).function() = function::SIDEHOPPER_HOPPING;
                set_enemy_instruction_list(k, ADDR_K_DEAD_MONSTERS_ILIST_ECE3 as u16, game);
                let index = DeadMonster::of(game, k).hop();
                let hop = corpse_tables::hop(game, index as isize);
                let mut corpse = DeadMonsterEdit::of(game, k);
                *corpse.y_velocity() = hop.y_velocity;
                *corpse.x_velocity() = hop.x_velocity;
            }
        }
    };
    [11131400] sidehopper_corpse_being_drained [] {
        let v2: u16 = DeadMonster::of(game, k).drained_palette_timer().wrapping_add(1);
        *DeadMonsterEdit::of(game, k).drained_palette_timer() = v2;
        if v2 >= 8 {
            *DeadMonsterEdit::of(game, k).drained_palette_timer() = 0;
            let index = DeadMonster::of(game, k).drained_palette_index();
            copy_rom_colors_to_palette(
                cgram::SPRITE_PALETTE_1,
                BANK,
                (32 * (index as i32 - 1i32) - 0x1434i32) as u16,
                0xf,
                game,
            );
            let v3: u16 = game.current_enemy();
            let v5: u16 = DeadMonster::of(game, k).drained_palette_index().wrapping_add(1);
            *DeadMonsterEdit::of(game, k).drained_palette_index() = v5;
            if v5 >= 8 {
                set_enemy_instruction_list(v3, ADDR_K_DEAD_MONSTERS_ILIST_ECE9 as u16, game);
                let mut corpse = DeadMonsterEdit::of(game, k);
                *corpse.function() = function::SIDEHOPPER_DEAD_WAIT_FOR_SAMUS_COLLISION;
                let actor = corpse.actor();
                actor.properties |= 0x8000;
                actor.radius.y = 12;
            }
        }
    };
    [11131578] sidehopper_corpse_rotting [] {
        rot_dead_monster(k, function::SIDEHOPPER_DEAD_WAIT_FOR_SAMUS_COLLISION, game);
    };
    [11131600] geemer_rotting [] {
        rot_dead_monster(k, function::CORPSE_ROTTED, game);
    };
    @forward [pub fn (k: u16, game: &mut impl CorpseState)];
    [11131492]
    sidehopper_corpse_wait_for_samus_collision => corpse_wait_for_samus_collision(k, function::SIDEHOPPER_PRE_ROT_DELAY, game);
    [11131497]
    geemer_wait_for_samus_collision => corpse_wait_for_samus_collision(k, function::GEEMER_PRE_ROT_DELAY, game);
    [11131502]
    skree_corpse_wait_for_samus_collision => corpse_wait_for_samus_collision(k, function::SKREE_PRE_ROT_DELAY, game);
    [11131507]
    ripper_corpse_wait_for_samus_collision => corpse_wait_for_samus_collision(k, function::RIPPER_PRE_ROT_DELAY, game);
    @forward [pub fn (k: u16, game: &mut impl CorpseState)];
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
/// The corpse rotting of dead monster `k`: once rotted, its function becomes
/// `rotted`; then its VRAM transfers are queued.
fn rot_dead_monster(k: u16, rotted: u16, game: &mut impl Corpses) {
    if process_corpse_rotting(k, game) & 1 == 0 {
        let current = game.current_enemy();
        *DeadMonsterEdit::of(game, current).function() = rotted;
    }
    let vram_transfers = DeadMonster::of(game, game.current_enemy()).vram_transfers();
    process_corpse_rotting_vram_transfers(vram_transfers, game);
}
pub fn sidehopper_corpse_movement(k: u16, game: &mut impl Corpses) -> u8 {
    let x_velocity = DeadMonster::of(game, k).x_velocity();
    sidehopper_corpse_horizontal_movement(k, x_velocity, game);
    let mut v2: u16 = 32;
    if (DeadMonster::of(game, k).y_velocity() & 0x8000) == 0 {
        v2 = 128;
    }
    let v3: u16 = DeadMonster::of(game, k).y_velocity().wrapping_add(v2);
    *DeadMonsterEdit::of(game, k).y_velocity() = v3;
    if (DeadMonster::of(game, k).actor().x.whole as i32 - 544i32) & 0x8000 != 0 {
        let mut corpse = DeadMonsterEdit::of(game, k);
        let y = &mut corpse.actor().y;
        // The 8.8 move: the low byte into the fraction's high byte, carrying.
        y.advance_8_8(v3 as i16);
        (((y.whole as i32 - 184i32) & 0x8000) == 0) as u8
    } else {
        {
            let (actor, amount) = (k, (v3 as i16 as i32) << 8);
            game.foreign(|game| game.move_vertical(actor, amount)) as u8
        }
    }
}
pub fn sidehopper_corpse_horizontal_movement(k: u16, a: u16, game: &mut impl Corpses) {
    if (DeadMonster::of(game, k).actor().x.whole as i32 - 544i32) & 0x8000 != 0 {
        DeadMonsterEdit::of(game, k).actor().x.advance_8_8(a as i16);
    } else {
        {
            let (actor, amount) = (k, (a as i16 as i32) << 8);
            game.foreign(|game| game.move_horizontal(actor, amount, Slopes::Ignore)) as u8
        };
    };
}
pub fn corpse_wait_for_samus_collision(k: u16, j: u16, game: &mut impl CorpseState) {
    if collided_with_samus(k, game) {
        *DeadMonsterEdit::of(game, k).function() = j;
    }
}
pub fn corpse_pre_rot_delay(k: u16, j: u16, game: &mut impl CorpseState) {
    let mut corpse = DeadMonsterEdit::of(game, k);
    let delay = corpse.pre_rot_delay();
    *delay = delay.wrapping_add(1);
    if *delay >= 0x10 {
        *corpse.function() = j;
        corpse.actor().properties |= K_ENEMY_PROPS_TANGIBLE as u16;
    }
}
pub use self::geemer_rotting as ripper_corpse_rotting;
pub use self::geemer_rotting as skree_corpse_rotting;
/// The rot tables' home: every corpse definition ($A9:DD58..$A9:DE17) puts
/// its table in the actors' fourth extended block ($7E:9000..$7E:97DB), at
/// $9000..$93FF or $9700..$97BF.
const ROT_TABLES: usize = 0x9000;
const ROT_TABLE_WORDS: usize = (0x97dc - ROT_TABLES) / 2;

/// The rot table word at `at`, read in native order: the row's entry, then
/// its delay two bytes on.
fn rot_word(at: usize, game: &mut impl Lease) -> u16 {
    let index = at.checked_sub(ROT_TABLES).filter(|offset| offset % 2 == 0);
    match index.map(|offset| offset / 2) {
        Some(index) if index < ROT_TABLE_WORDS => {
            u16::from_ne_bytes(game.session().actors.extended.blocks[3][index].to_le_bytes())
        }
        _ => {
            // LEGACY(original-overflow): a rot table cursor outside the
            // block reads whatever RAM lies there, on the game as before.
            game.foreign(|game| crate::legacy_overflow::ram_native_word(game, at))
        }
    }
}
pub fn process_corpse_rotting(k: u16, game: &mut impl Lease) -> u8 {
    *CorpseEdit::of(game, 0).active_height() = Corpse::of(game, k).height();
    *CorpseEdit::of(game, 0).active_last_row() = Corpse::of(game, k).last_row();
    *CorpseEdit::of(game, 0).active_second_last_row() = Corpse::of(game, k).second_last_row();
    *CorpseEdit::of(game, 0).active_next_row_offset() = Corpse::of(game, k).next_row_offset();
    *CorpseEdit::of(game, 0).active_finished_hook() = Corpse::of(game, k).finished_hook();
    *CorpseEdit::of(game, 0).active_row_offsets() = Corpse::of(game, k).row_offsets();
    *CorpseEdit::of(game, 0).active_copy_function() = Corpse::of(game, k).copy_function();
    *CorpseEdit::of(game, 0).active_move_function() = Corpse::of(game, k).move_function();
    let mut rot_entry_at: u16 = Corpse::of(game, k).rot_table();
    let mut v4: u16 = 0;
    loop {
        let v15: u16 = v4;
        let v5: usize = rot_entry_at as usize;
        let v6 = v5;
        if (rot_word(v5, game) as i16) >= 0 {
            let v7: i16 = rot_word(v5 + 2, game) as i16;
            if v7 != 0 {
                let v8: i16 = (v7).wrapping_sub(1);
                set_native_word(v5 + 2, v8 as u16, game);
                if (v8 as u16) < 4 {
                    let mut r18: u16 = Corpse::of(game, 0).active_copy_function();
                    if v15 >= Corpse::of(game, 0).active_second_last_row() {
                        r18 = Corpse::of(game, 0).active_move_function();
                    }
                    copy_move_corpse_rotting_rot_entry(
                        native_record(
                            BANK_BASE | u32::from(Corpse::of(game, 0).active_row_offsets()),
                            game,
                        ),
                        rot_word(v6, game),
                        r18,
                        game,
                    );
                }
            } else {
                let r18_0: u16 = Corpse::of(game, 0).active_move_function();
                let v11: usize = rot_entry_at as usize;
                copy_move_corpse_rotting_rot_entry(
                    native_record(
                        BANK_BASE | u32::from(Corpse::of(game, 0).active_row_offsets()),
                        game,
                    ),
                    rot_word(v11, game),
                    r18_0,
                    game,
                );
                let v12: u16 = (rot_word(v11, game)).wrapping_add(2);
                if v12 < Corpse::of(game, 0).active_last_row() {
                    set_native_word(v11, v12, game);
                } else {
                    run_rot_entry_finished_hook(
                        u32::from(Corpse::of(game, 0).active_finished_hook()) | BANK_BASE,
                        game,
                    );
                    if v15 >= Corpse::of(game, 0).active_last_row() {
                        break;
                    }
                    set_native_word(v11, -1i32 as u16, game);
                }
            }
        }
        rot_entry_at = (rot_entry_at).wrapping_add(4);
        v4 = (v15).wrapping_add(1);
        if ((v15 as i32 + 1 - Corpse::of(game, 0).active_height() as i32) as i16) >= 0 {
            return ((v15).wrapping_add(1) >= Corpse::of(game, 0).active_height()) as u8;
        }
    }
    0
}
pub fn copy_move_corpse_rotting_rot_entry(r20: RomRecord, a: u16, r18: u16, game: &mut impl Lease) {
    *CorpseEdit::of(game, 0).rot_entry() = a;
    let v1: u16 = ((a) & (0xfff8u16)) >> 2;
    let v2: u16 = (a) & (7u16);
    let v6: u16 = (r20.table_word((v1 >> 1) as isize) as i32 + 2 * v2 as i32) as u16;
    if v2 >= 6 {
        run_rot_entry_copy_or_move(
            ((r18) as u32) | (BANK_BASE),
            Corpse::of(game, 0)
                .active_next_row_offset()
                .wrapping_add(v6),
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
    let v4: u16 = v2.word(12);
    {
        let mut corpse = CorpseEdit::of(game, k);
        *corpse.rot_table() = v2.word(0);
        *corpse.vram_transfers() = v2.word(2);
        *corpse.copy_function() = v2.word(4);
        *corpse.move_function() = v2.word(6);
        *corpse.finished_hook() = v2.word(14);
        *corpse.row_offsets() = v4;
    }
    let next_row_offset = native_record(BANK_BASE | u32::from(v4), game)
        .word(2)
        .wrapping_sub(12);
    *CorpseEdit::of(game, k).next_row_offset() = next_row_offset;
    let v5 = v2;
    let mut v6: u16 = v5.word(8);
    let fresh5 = v6;
    v6 = v6.wrapping_sub(1);
    {
        let mut corpse = CorpseEdit::of(game, k);
        *corpse.height() = fresh5;
        *corpse.last_row() = v6;
        *corpse.second_last_row() = v6.wrapping_sub(1);
    }
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
    *CorpseEdit::of(game, 0).sprite_tile_transfer() = 0;
    word!(game.VRAM_WRITE_QUEUE_TAIL = v1);
}
pub fn sidehopper_corpse_rottable_contact(game: &mut impl CorpseState) {
    corpse_contact_reaction(function::SIDEHOPPER_ROTTING, game);
}
pub fn corpse_contact_reaction(a: u16, game: &mut impl CorpseState) {
    let e = game.current_enemy();
    let mut corpse = DeadMonsterEdit::of(game, e);
    *corpse.function() = a;
    corpse.actor().properties |=
        (K_ENEMY_PROPS_PROCESSED_OFFSCREEN | K_ENEMY_PROPS_TANGIBLE) as u16;
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
    for (row, &minimum) in row_gates.iter().enumerate() {
        let progress = || Corpse::of(game, 0).rot_entry();
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
pub fn initialize_torizo_corpse(game: &mut impl Corpses) {
    word_transfers!(game, descending, 2048; le(TILEMAP_STUFF, 2047) <- value(0));
    {
        let mut torizo = DeadTorizoEdit::of(game, 0);
        *torizo.function() = function::TORIZO_WAIT_FOR_SAMUS_COLLISION;
        let actor = torizo.actor();
        actor.properties |= 0xa000;
        actor.instruction = ADDR_K_DEAD_TORIZO_ILIST_D6DC as u16;
        actor.instruction_delay = 1;
        actor.timer = 0;
        actor.palette = 512;
        *torizo.x_velocity() = 0;
        *torizo.y_velocity() = 8;
        *torizo.pre_rot_delay() = 0;
        *torizo.sand_heap_lines() = 15;
        *torizo.rot_timer() = 0;
    }
    initialize_enemy_corpse_rotting(0, ADDR_STRU_A9DD58 as u16, game);
}
/// Main AI - Torizo corpse.
pub fn torizo_corpse_main_ai(game: &mut impl Corpses) {
    if DeadTorizo::of(game, 0).actor().properties & K_ENEMY_PROPS_TANGIBLE as u16 == 0
        && game.foreign(|game| game.dead_torizo_contact())
    {
        torizo_corpse_touch_or_shot(game);
    }
    set_graphics_drawn_hook(DRAW_DEAD_TORIZO_HOOK, game);
    let function = DeadTorizo::of(game, 0).function();
    run_torizo_corpse_function(u32::from(function) | BANK_BASE, game);
    process_torizo_corpse_rotting_vram_transfers(game);
}
pub fn torizo_corpse_power_bomb_reaction(game: &mut impl CorpseState) {
    if DeadTorizo::of(game, 0).actor().properties & K_ENEMY_PROPS_TANGIBLE as u16 == 0 {
        torizo_corpse_touch_or_shot(game);
    }
}
pub fn torizo_corpse_touch_or_shot(game: &mut impl CorpseState) {
    let mut torizo = DeadTorizoEdit::of(game, 0);
    torizo.actor().properties |= K_ENEMY_PROPS_TANGIBLE as u16;
    *torizo.function() = function::TORIZO_ROTTING;
}
pub fn initialize_sidehopper_corpse(game: &mut impl Corpses) {
    let dsr_parameter_1 = DeadMonster::of(game, game.current_enemy()).parameter_1();
    if dsr_parameter_1 != 0 {
        if dsr_parameter_1 != 2 {
            bank_unreachable(game);
        }
        initialize_dead_sidehopper_corpse(game);
    } else {
        initialize_alive_sidehopper_corpse(game);
    };
}
pub fn initialize_geemer_corpse(game: &mut impl Corpses) {
    let enemy = game.current_enemy();
    initialize_rotting_enemy_corpse(
        enemy,
        function::GEEMER_WAIT_FOR_SAMUS_COLLISION,
        corpse_tables::Corpse::Geemer,
        game,
    );
}
pub fn initialize_ripper_corpse(game: &mut impl Corpses) {
    let enemy = game.current_enemy();
    initialize_rotting_enemy_corpse(
        enemy,
        function::RIPPER_WAIT_FOR_SAMUS_COLLISION,
        corpse_tables::Corpse::Ripper,
        game,
    );
}
pub fn initialize_skree_corpse(game: &mut impl Corpses) {
    let enemy = game.current_enemy();
    initialize_rotting_enemy_corpse(
        enemy,
        function::SKREE_WAIT_FOR_SAMUS_COLLISION,
        corpse_tables::Corpse::Skree,
        game,
    );
}
pub fn sidehopper_corpse_power_bomb_reaction(game: &mut impl Corpses) {
    if DeadMonster::of(game, game.current_enemy()).drained_palette_index() >= 8 {
        sidehopper_corpse_shot(game);
    } else {
        corpse_main_ai(game);
    };
}
pub fn corpse_main_ai(game: &mut impl Corpses) {
    let function = DeadMonster::of(game, game.current_enemy()).function();
    run_dead_monster_function(u32::from(function) | BANK_BASE, game.current_enemy(), game);
}
pub fn power_bomb_reaction_geemer_corpse(game: &mut impl CorpseState) {
    if DeadMonster::of(game, game.current_enemy())
        .actor()
        .properties
        & K_ENEMY_PROPS_TANGIBLE as u16
        == 0
    {
        touch_or_shot_geemer_corpse(game);
    }
}
pub fn touch_or_shot_geemer_corpse(game: &mut impl CorpseState) {
    corpse_contact_reaction(function::GEEMER_ROTTING, game);
}
pub fn ripper_corpse_power_bomb_reaction(game: &mut impl CorpseState) {
    if DeadMonster::of(game, game.current_enemy())
        .actor()
        .properties
        & K_ENEMY_PROPS_TANGIBLE as u16
        == 0
    {
        ripper_corpse_touch_or_shot(game);
    }
}
pub fn ripper_corpse_touch_or_shot(game: &mut impl CorpseState) {
    corpse_contact_reaction(function::RIPPER_ROTTING, game);
}
pub fn skree_corpse_power_bomb_reaction(game: &mut impl CorpseState) {
    if DeadMonster::of(game, game.current_enemy())
        .actor()
        .properties
        & K_ENEMY_PROPS_TANGIBLE as u16
        == 0
    {
        skree_corpse_touch_or_shot(game);
    }
}
pub fn skree_corpse_touch_or_shot(game: &mut impl CorpseState) {
    corpse_contact_reaction(function::SKREE_ROTTING, game);
}
pub fn sidehopper_corpse_shot(game: &mut impl CorpseState) {
    let corpse = DeadMonster::of(game, game.current_enemy());
    if !((corpse.actor().properties & K_ENEMY_PROPS_TANGIBLE as u16) != 0
        || corpse.drained_palette_index() < 8)
    {
        sidehopper_corpse_rottable_contact(game);
    }
}
pub fn sidehopper_corpse_touch(game: &mut impl Corpses) {
    if DeadMonster::of(game, game.current_enemy()).drained_palette_index() < 8 {
        game.foreign(|game| game.contact_reaction(Death::LeavePending));
    } else {
        sidehopper_corpse_rottable_contact(game);
    };
}
pub fn sidehopper_corpse_end_hop(
    k: u16,
    jp: EnemyScript,
    game: &mut impl CorpseState,
) -> EnemyScript {
    let mut v2: u16 = function::SIDEHOPPER_START_IDLING;
    if DeadMonster::of(game, k).drained_palette_index() != 0 {
        v2 = function::SIDEHOPPER_BEING_DRAINED;
    }
    *DeadMonsterEdit::of(game, k).function() = v2;
    jp
}
