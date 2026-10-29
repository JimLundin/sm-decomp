use crate::game::actors;
use crate::game::actors::access::ActorKey as Key;
use crate::game::actors::api as actor_api;
use crate::game::actors::mother_brain_effects::{
    MotherBrain_GenerateExplosions, MotherBrain_HandleRainbowBeamPalette, MotherBrain_WritePalette,
};
use crate::game::display_refresh::{native_word, set_native_word};
use crate::game::enemy_script::EnemyScript;
use crate::game::menu_tilemap::native_record;
use crate::game::music::QueueMusic_Delayed8;
use crate::game::palette_copies::{WriteColorsToPalette, WriteColorsToTargetPalette};
use crate::game::player_update::minimap;
pub use crate::game::prelude::*;
use crate::game::rom_vram_entries::ProcessSpriteTilesTransfers;
use crate::game::session_runtime::Runtime;
use crate::rom::RomRecord;

use crate::game::beam_windows::calculate_rainbow_window;

pub use crate::game::constants::bank_8d::*;
pub use crate::game::constants::bank_86::*;
pub use crate::game::constants::bank_a6::*;
pub use crate::game::constants::bank_a9::*;

use crate::game::enemy_projectiles::{
    Math_MultByCos, Math_MultBySin, SpawnEprojWithGfx, SpawnEprojWithRoomGfx,
};
use crate::game::gameplay_flag_runtime::{
    CheckEventHappened, SetBossBitForCurArea, SetEventHappened,
};
use crate::game::mother_brain_samus_vitals as vitals;
use crate::game::mother_brain_visuals::{self, Fade};
use crate::game::mother_brain_visuals::{spawn_rainbow_hdma, spawn_rising_hdma};
use crate::game::music::{
    QueueSfx1_Max6, QueueSfx2_Max3, QueueSfx2_Max6, QueueSfx3_Max3, QueueSfx3_Max6,
};
use crate::game::palette_effects;
use crate::game::projectile_geometry::SINE_COSINE as kSinCosTable8bit_Sext;
use crate::game::room_mechanisms::spawn_hardcoded;
use crate::game::room_state::load_fx_entry as LoadFxEntry;
use crate::game::session_runtime::player_command as CallSomeSamusCode;
use crate::game::session_runtime::player_mother_brain;

use crate::game::escape_typewriter::{HandleTypewriterText_Ext, SetupZebesEscapeTypewriter};
use crate::game::sm_a6::{HomingReversal, Ridley_Func_98, homing_velocity};

use crate::ram_map::slot;

pub use crate::types::Consts_B7;

crate::game::constants::typed_constants! { Consts_B7;
    addr_kMotherBrain_Misc_TileData = 52736, addr_kDeadMonsters_TileData = 49152,
    addr_kDeadTorizo_TileData = 43008,
}

crate::typed_struct! {
    #[derive(Copy, Clone)]
    #[repr(C)]
    pub struct MotherBrainInstrExecState {
        pub ip, timer: u16;
    }
}

crate::type_aliases! {
    pub Enemy_Shitroid, Enemy_DeadTorizo, Enemy_DeadSidehopper =
        crate::game::enemy_layouts::EnemyLayout40;
}

pub type Enemy_DeadMonsters = crate::game::enemy_layouts::EnemyLayout72Late;

pub type Enemy_ShitroidInCutscene = crate::game::enemy_layouts::EnemyLayout72;

pub type Enemy_DeadEnemy = crate::game::enemy_layouts::EnemyLayout40Late;

crate::type_aliases! {
    pub Enemy_MotherBomb, Enemy_MotherBrainBody, Enemy_MotherBrainsTubesFalling =
        crate::game::enemy_layouts::EnemyLayout8;
}

crate::game::constants::typed_constants! { i32;
    kCoroutineNone = 0,
    fnEproj_Init_0x8bde_SkreeDownLeft = 0x868b15, fnMotherBrainsBody_UnpauseHook = 0xa98763,
    fnMotherBrainsBody_FirstPhase_DoubleRet = 0xa987e1,
}
const RomPtr_A9: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0xa9>;
crate::game::constants::typed_constants! { u32;
    fnMotherBrainBody_FakeDeath_Descent_0_Pause = 11110429,
}
static g_word_A98B5D: [u16; 5] = [0x10, 0x10, 8, 8, 0x10];
static g_word_A98B67: [u16; 5] = [0x20, 0x20, 0x18, 0x18, 0x20];
static g_word_A98B71: [u16; 5] = [0xf8, 0xf8, 0xf0, 0xf0, 0xf6];
static g_word_A98C61: [u16; 4] = [0xfff8, 2, 0xfffc, 6];
static g_byte_A98F7D: [u8; 2] = [9, 0x12];
static g_word_A98F7F: [u16; 8] = [0x3d, 0x54, 0x20, 0x35, 0x5a, 0x43, 0x67, 0x29];
static g_word_A993BB: [i16; 4] = [0, -1, 0, 1];
static g_word_A993C3: [i16; 4] = [0, 1, -1, 1];
static g_word_A99E0F: [u16; 13] = [
    0x6f, 0x6f, 0x6f, 0x7e, 0x6f, 0x6f, 0x7e, 0x6f, 0x6f, 0x7e, 0x7e, 0x6f, 0x6f,
];
static g_word_A9B393: [u16; 8] = [8, 0x6c, 0x18, 0x80, 9, 0x90, 0x18, 0x74];
static g_byte_A9B546: [u8; 8] = [0, 1, 1, 0, 0, 0, 0, 0];
static g_byte_A9B5A1: [u8; 8] = [2, 1, 1, 0, 0, 0, 0, 0];
static g_word_A9B72C: [i16; 28] = [
    -1, -17, 0, 0, -1, -1, 0, -1, 0, 0, -1, 0, 0, 0, -17, -1, -1, -1, 0, 0, 0, -17, -1, -1, -1, -1,
    -1, -1,
];
static g_word_A9BCA6: [i16; 8] = [-8, 6, -4, 2, 3, -6, 8, 0];
static g_word_A9BCB6: [i16; 8] = [-7, 2, 5, -4, 6, -2, -6, 7];
static g_byte_A9BEEE: [u8; 16] = [2, 0, 2, 0, 6, 0, 6, 0, 8, 0, 8, 0, 10, 0, 10, 0];
static g_word_A9BEFE: [u16; 8] = [0x500, 0x500, 0x200, 0x200, 0xc0, 0xc0, 0x40, 0x40];
static g_word_A9C049: [i16; 8] = [0x10, 0x10, 0x20, 0x20, 0x30, 0x30, 0x40, 0x40];
static g_byte_A9F56A: [i8; 16] = [
    0x10, 0xf, 0xe, 0xd, 0xc, 0xb, 0xa, 9, 8, 7, 6, 5, 4, 3, 2, 1,
];
pub fn CallCorpseRottingMove(ea: u32, k: u16, j: u16, game: &mut Game) {
    match ea {
        11133554 => transfer_corpse_tiles(
            k,
            j,
            0x2000,
            94,
            &[80, 72, 16, 0, 0, 0, 0, 0, 0, 16],
            true,
            false,
            game,
        ),
        11133835 => transfer_corpse_tiles(
            k,
            j,
            0x2000,
            94,
            &[80, 72, 16, 0, 0, 0, 0, 0, 0, 16],
            false,
            false,
            game,
        ),
        11134056 => transfer_corpse_tiles(k, j, 0x2000, 38, &[8, 8, 0, 0, 0], true, false, game),
        11134197 => transfer_corpse_tiles(k, j, 0x2000, 38, &[8, 8, 0, 0, 0], false, false, game),
        11134308 => transfer_corpse_tiles(k, j, 0x2320, 38, &[0, 0, 8, 8, 8], true, false, game),
        11134454 => transfer_corpse_tiles(k, j, 0x2320, 38, &[0, 0, 8, 8, 8], false, false, game),
        11134570 => transfer_corpse_tiles(k, j, 0x2940, 14, &[0, 0, 0], true, false, game),
        11134649 => transfer_corpse_tiles(k, j, 0x2940, 14, &[0, 0, 0], false, false, game),
        11134710 => transfer_corpse_tiles(k, j, 0x2a00, 14, &[0, 0, 0], true, false, game),
        11134789 => transfer_corpse_tiles(k, j, 0x2a00, 14, &[0, 0, 0], false, false, game),
        11134850 => transfer_corpse_tiles(k, j, 0x2ac0, 14, &[0, 0, 0], true, false, game),
        11134929 => transfer_corpse_tiles(k, j, 0x2ac0, 14, &[0, 0, 0], false, false, game),
        11134990 => transfer_corpse_tiles(k, j, 0x2b80, 14, &[0, 0, 0], true, false, game),
        11135069 => transfer_corpse_tiles(k, j, 0x2b80, 14, &[0, 0, 0], false, false, game),
        11135130 => transfer_corpse_tiles(k, j, 0x2c40, 14, &[0, 0, 0], true, false, game),
        11135209 => transfer_corpse_tiles(k, j, 0x2c40, 14, &[0, 0, 0], false, false, game),
        11135270 => transfer_corpse_tiles(k, j, 0x2640, 30, &[0, 0], true, false, game),
        11135323 => transfer_corpse_tiles(k, j, 0x2640, 30, &[0, 0], false, false, game),
        11135364 => transfer_corpse_tiles(k, j, 0x2740, 30, &[0, 0], true, false, game),
        11135417 => transfer_corpse_tiles(k, j, 0x2740, 30, &[0, 0], false, false, game),
        11135458 => transfer_corpse_tiles(k, j, 0x2840, 30, &[0, 0], true, false, game),
        11135511 => transfer_corpse_tiles(k, j, 0x2840, 30, &[0, 0], false, false, game),
        11135552 => {
            transfer_corpse_tiles(k, j, 0x9000, 46, &[16, 8, 0, 0, 0, 8, 32], true, true, game)
        }
        11135755 => transfer_corpse_tiles(
            k,
            j,
            0x9000,
            46,
            &[16, 8, 0, 0, 0, 8, 32],
            false,
            true,
            game,
        ),
        _ => {
            Unreachable(game);
        }
    }
}
pub fn CallCorpseRottingInit(ea: u32, game: &mut Game) {
    match ea {
        11132440 => initialize_corpse_tiles(
            addr_kDeadTorizo_TileData as u16,
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
            addr_kDeadMonsters_TileData as u16,
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
            addr_kDeadMonsters_TileData as u16,
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
            addr_kDeadMonsters_TileData as u16,
            0x2000,
            &[(0x940, 0xa60, 0x60), (0x9a0, 0xc60, 0x60)],
            game,
        ),
        11132780 => initialize_corpse_tiles(
            addr_kDeadMonsters_TileData as u16,
            0x2000,
            &[(0xa00, 0xac0, 0x60), (0xa60, 0xcc0, 0x60)],
            game,
        ),
        11132809 => initialize_corpse_tiles(
            addr_kDeadMonsters_TileData as u16,
            0x2000,
            &[(0xac0, 0xb20, 0x60), (0xb20, 0xd20, 0x60)],
            game,
        ),
        11132838 => initialize_corpse_tiles(
            addr_kDeadMonsters_TileData as u16,
            0x2000,
            &[(0xb80, 0xa00, 0x60), (0xbe0, 0xc00, 0x60)],
            game,
        ),
        11132867 => initialize_corpse_tiles(
            addr_kDeadMonsters_TileData as u16,
            0x2000,
            &[(0xc40, 0xb80, 0x60), (0xca0, 0xd80, 0x60)],
            game,
        ),
        11132896 => initialize_corpse_tiles(
            addr_kDeadMonsters_TileData as u16,
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
            addr_kDeadMonsters_TileData as u16,
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
            addr_kDeadMonsters_TileData as u16,
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
            addr_kMotherBrain_Misc_TileData as u16,
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
            Unreachable(game);
        }
    }
}
pub fn CallCorpseRottingFinish(ea: u32, game: &mut Game) {
    match ea {
        11130301 => DeadTorizo_CorpseRottingFinished(game),
        11131912 => CorpseRottingRotEntryFinishedHook(game),
        11121187 => MotherBrain_CorpseRottingFinished(game),
        _ => {
            Unreachable(game);
        }
    }
}
crate::instruction_callbacks! {
    pub fn (game: &mut Game);
    @prefix {};
    DeadTorizo_CorpseRottingFinished [] {
        game.eproj_spawn_pt = Point16U {
            x: ((word!(game.random_number) as i32 & 0x1f) + 272) as u16,
            y: 188,
        };
        SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 0xa, game);
        if (word!(game.random_enemy_counter) & 7) == 0 {
            QueueSfx2_Max6(0x10, game);
        }
    };
    CorpseRottingRotEntryFinishedHook [] {
        let v0: i32 = word!(game.cur_enemy_index) as i32;
        game.eproj_spawn_pt = Point16U {
            x: (slot!(game, EnemyData[v0].x_pos) as i32 + (word!(game.random_number) as i32 & 0x1a) - 14i32) as u16,
            y: ((slot!(game, EnemyData[v0].y_pos)).wrapping_add(16)),
        };
        SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 0xa, game);
        if (word!(game.random_enemy_counter) & 7) == 0 {
            QueueSfx2_Max3(0x10, game);
        }
    };
}
pub fn MotherBrain_CorpseRottingFinished(game: &mut Game) {
    let E1: i32 = 0x40;
    game.eproj_spawn_pt = Point16U {
        x: (slot!(game, Enemy_MotherBrain[E1].base.x_pos) as i32
            + (word!(game.random_number) as i32 & 0x1f)
            - 16i32) as u16,
        y: ((slot!(game, Enemy_MotherBrain[E1].base.y_pos)).wrapping_add(16)),
    };
    SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 0xa, game);
    if (word!(game.random_enemy_counter) & 7) == 0 {
        QueueSfx2_Max3(0x10, game);
    }
}

crate::game::callback_dispatch! {
    pub unsafe fn CallMotherBrainFunc(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [11110306] MotherBrainsBrain_SetupBrainAndNeckToDraw [] {
        if word!(game.time_is_frozen_flag) == 0 {
            MotherBrain_HandleNeck(game);
        }
        let E1: i32 = 0x40;
        slot!(game, Enemy_MotherBrain[E1].base.x_pos = slot!(game, Enemy_MotherBrain[E1].var_2E));
        slot!(game, Enemy_MotherBrain[E1].base.y_pos = (slot!(game, Enemy_MotherBrain[E1].var_2F)).wrapping_sub(21));
        { let value: u32 = 0xa987c9; { let value: u16 = value as u16; let at: usize = 0x178c; let bytes = value.to_ne_bytes(); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); }; game.ram().write_byte(0x178c + 2, (value >> 16) as u8); };
    };
    [11110429] MotherBrainBody_FakeDeath_Descent_0_Pause [] {
        let E: i32 = 0;
        slot!(game, Enemy_MotherBrainBody[E].var_A = (0xa98829i32) as u16);
        slot!(game, Enemy_MotherBrainBody[E].var_F = 64);
        MotherBrainBody_FakeDeath_Descent_1(game);
    };
    [11110611] MotherBrainBody_FakeDeath_Descent_6 [] {
        let E: i32 = 0x40;
        CallMotherBrainFunc(((slot!(game, Enemy_MotherBrainBody[E].var_E)) as u32) | (0xa90000u32), game);
        MotherBrain_HandleFakeDeathExplosions(game);
    };
    [11110729] MotherBrainBody_0_Wait [] {
        let mut v1: i16 = 0;
        for v0 in (0u16..36).step_by(2) {
            if { let at: usize = 0x1997 + (v0 as i32 as isize ) as usize; u16::from_ne_bytes([game.ram().read_byte(at), game.ram().read_byte(at + 1)]) }
                == 0
            {
                v1 += 1;
            }
        }
        if (v1).wrapping_sub(4) >= 0 {
            ({ let (bank, population) = (0xa9, addr_stru_A98AE5 as u16); actors::population::spawn_legacy(&mut Runtime::new(game), bank, population) });
            slot!(game, Enemy_MotherBrainBody[0x40i32].var_E = (0xa9896ei32) as u16);
        }
    };
    [11110766] MotherBrainBody_1_ClearBottomLeftTube [] {
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x5, 0x9, 0xb6c3 as VoidP), game);
        let E: i32 = 0x40;
        slot!(game, Enemy_MotherBrainBody[E].var_E = (0xa98983i32) as u16);
        slot!(game, Enemy_MotherBrainBody[E].var_F = 32);
    };
    [11110816] MotherBrainBody_3_ClearCeilingBlock9 [] {
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x9, 0x2, 0xb6b3 as VoidP), game);
        let E: i32 = 0x40;
        slot!(game, Enemy_MotherBrainBody[E].var_E = (0xa989b5i32) as u16);
        slot!(game, Enemy_MotherBrainBody[E].var_F = 32);
    };
    [11110866] MotherBrainBody_4_ClearCeilingBlock6 [] {
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x6, 0x2, 0xb6b3 as VoidP), game);
        let E: i32 = 0x40;
        slot!(game, Enemy_MotherBrainBody[E].var_E = (0xa989e7i32) as u16);
        slot!(game, Enemy_MotherBrainBody[E].var_F = 32);
    };
    [11110906] MotherBrainBody_6_ClearBottomRightTube [] {
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0xa, 0x9, 0xb6c7 as VoidP), game);
        let E: i32 = 0x40;
        slot!(game, Enemy_MotherBrainBody[E].var_E = (0xa98a0fi32) as u16);
        slot!(game, Enemy_MotherBrainBody[E].var_F = 32);
    };
    [11110946] MotherBrainBody_8_ClearBottomMiddleLeftTube [] {
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x6, 0xa, 0xb6bb as VoidP), game);
        let E: i32 = 0x40;
        slot!(game, Enemy_MotherBrainBody[E].var_E = (0xa98a37i32) as u16);
        slot!(game, Enemy_MotherBrainBody[E].var_F = 32);
    };
    [11110996] MotherBrainBody_10_ClearCeilingTubeColumn7 [] {
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x7, 0x2, 0xb6b7 as VoidP), game);
        let E: i32 = 0x40;
        slot!(game, Enemy_MotherBrainBody[E].var_E = (0xa98a69i32) as u16);
        slot!(game, Enemy_MotherBrainBody[E].var_F = 32);
    };
    [11111046] MotherBrainBody_12_ClearCeilingTubeColumn8 [] {
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x8, 0x2, 0xb6b7 as VoidP), game);
        let E: i32 = 0x40;
        slot!(game, Enemy_MotherBrainBody[E].var_E = (0xa98a9bi32) as u16);
        slot!(game, Enemy_MotherBrainBody[E].var_F = 32);
    };
    [11111086] MotherBrainBody_14_ClearBottomMiddleRightTube [] {
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x9, 0xa, 0xb6bb as VoidP), game);
        let E: i32 = 0x40;
        slot!(game, Enemy_MotherBrainBody[E].var_E = (0xa98ac3i32) as u16);
        slot!(game, Enemy_MotherBrainBody[E].var_F = 2);
    };
    [11111126] MotherBrainBody_16_ClearBottomMiddleTubes [] {
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x7, 0x7, 0xb6bf as VoidP), game);
        slot!(game, Enemy_MotherBrainBody[0x40i32].var_E = addr_locret_A98AE4 as u16);
    };
    [11111559] MotherBrainBody_FakeDeath_Ascent_0_DrawBG1Row23 [] {
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x2, 0x2, 0xb67b as VoidP), game);
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x2, 0x3, 0xb67f as VoidP), game);
        slot!(game, Enemy_MotherBrainBody[0i32].var_A = (0xa98c9ei32) as u16);
    };
    [11111582] MotherBrainBody_FakeDeath_Ascent_1_DrawBG1Row45 [] {
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x2, 0x4, 0xb683 as VoidP), game);
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x2, 0x5, 0xb687 as VoidP), game);
        slot!(game, Enemy_MotherBrainBody[0i32].var_A = (0xa98cb5i32) as u16);
    };
    [11111605] MotherBrainBody_FakeDeath_Ascent_2_DrawBG1Row67 [] {
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x2, 0x6, 0xb68b as VoidP), game);
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x2, 0x7, 0xb68f as VoidP), game);
        slot!(game, Enemy_MotherBrainBody[0i32].var_A = (0xa98ccci32) as u16);
    };
    [11111628] MotherBrainBody_FakeDeath_Ascent_3_DrawBG1Row89 [] {
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x2, 0x8, 0xb693 as VoidP), game);
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x2, 0x9, 0xb697 as VoidP), game);
        slot!(game, Enemy_MotherBrainBody[0i32].var_A = (0xa98ce3i32) as u16);
    };
    [11111651] MotherBrainBody_FakeDeath_Ascent_4_DrawBG1RowAB [] {
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x2, 0xa, 0xb69b as VoidP), game);
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x2, 0xb, 0xb69f as VoidP), game);
        slot!(game, Enemy_MotherBrainBody[0i32].var_A = (0xa98cfai32) as u16);
    };
    [11111674] MotherBrainBody_FakeDeath_Ascent_5_DrawBG1RowCD [] {
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x2, 0xc, 0xb6a3 as VoidP), game);
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x2, 0xd, 0xb6a7 as VoidP), game);
        slot!(game, Enemy_MotherBrainBody[0i32].var_A = (0xa98d11i32) as u16);
    };
    [11111697] MotherBrainBody_FakeDeath_Ascent_6_SetupPhase2Gfx [] {
        { let value: u16 = 257; let at: usize = 0x91b; let bytes = value.to_ne_bytes(); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); };
        { let at: usize = 0x59; let bytes: [u8; 2] = ((u16::from_le_bytes([game.ram().read_byte(at), game.ram().read_byte(at + 1)])) & (0xfffcu16)).to_le_bytes(); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); }
        WriteColorsToPalette(
            0x142,
            0xa9,
            (addr_kMotherBrainPalette_1 as i32 + 2) as u16,
            0xf,
            game,
        );
        WriteColorsToPalette(
            0x162,
            0xa9,
            (addr_kMotherBrainPalette_0 as i32 + 2) as u16,
            0xf,
            game,
        );
        slot!(game, Enemy_MotherBrain[0i32].var_A = (0xa98d49i32) as u16);
        word!(game.nmi_flag_bg2_enemy_vram_transfer = 1);
        slot!(game, Enemy_MotherBrain[0x40i32].var_02 = 1);
    };
    [11111753] MotherBrainBody_FakeDeath_Ascent_7_SetupPhase2Brain [] {
        word!(game.fx_layer_blending_config_a = 52);
        let E: i32 = 0x40;
        slot!(game, Enemy_MotherBrainBody[E].var_A = (0xa987a2i32) as u16);
        let E0: i32 = 0;
        slot!(game, Enemy_MotherBrainBody[E0].base.properties = (slot!(game, Enemy_MotherBrainBody[E0].base.properties) as i32 & !(kEnemyProps_Tangible as i32)) as u16);
        slot!(game, Enemy_MotherBrainBody[E].base.properties = slot!(game, Enemy_MotherBrainBody[E].base.properties) & !(0x400));
        slot!(game, Enemy_MotherBrainBody[E].base.health = 18000);
        slot!(game, Enemy_MotherBrainBody[E0].var_A = (0xa98d79i32) as u16);
        slot!(game, Enemy_MotherBrainBody[E0].var_F = 128);
        MotherBrainBody_FakeDeath_Ascent_8_Pause(game);
    };
    [11111860] MotherBrainBody_FakeDeath_Ascent_10_LoadLegTiles [] {
        if ProcessSpriteTilesTransfers(0xa9, addr_stru_A98F8F as u16, game) != 0 {
            slot!(game, Enemy_MotherBrainBody[0i32].var_A = (0xa98dc3i32) as u16);
            MotherBrainBody_FakeDeath_Ascent_11_ContinuePause(game);
        }
    };
    [11111916] MotherBrainBody_FakeDeath_Ascent_12_StartMusic [] {
        MotherBrain_SetBodyInstrs(addr_kMotherBrain_Ilist_9A02 as u16, game);
        let E1: i32 = 0x40;
        slot!(game, Enemy_MotherBrain[E1].base.instruction_timer = 1);
        let E0: i32 = 0;
        slot!(game, Enemy_MotherBrain[E0].base.properties = (slot!(game, Enemy_MotherBrain[E0].base.properties) as i32 & !(kEnemyProps_Invisible as i32)) as u16);
        slot!(game, Enemy_MotherBrain[E0].base.x_pos = 59);
        slot!(game, Enemy_MotherBrain[E0].base.y_pos = 279);
        word! {game; reg_BG2HOFS = -27i32 as u16; reg_BG2VOFS = -217i32 as u16; }
        QueueMusic_Delayed8(5, game);
        word! {game; earthquake_type = 2; earthquake_timer = 256; }
        slot! { game, Enemy_MotherBrain[E1]; var_34 = 80, var_31 = 1, var_32 = 8, var_33 = 6 };
        slot!(game, Enemy_MotherBrain[E0].var_A = (0xa98e4di32) as u16);
    };
    [11120417] MotherBrain_Phase3_Death_2 [] {
        MotherBrain_GenerateSmokyExplosions(game);
        if MotherBrain_MakeWalkForwards(2, 0x60, game) & 1 != 0 {
            MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9C39 as u16, game);
            let E1: i32 = 0x40;
            slot! { game, Enemy_MotherBrain[E1]; var_32 = 6, var_33 = 6 }
            slot!(game, Enemy_MotherBrain[E1].var_34 = 1280);
            let E: i32 = 0;
            slot!(game, Enemy_MotherBrain[E].var_A = (0xa9af54i32) as u16);
            slot!(game, Enemy_MotherBrain[E].var_F = 32);
        }
    };
    [11120566] MotherBrain_Phase3_Death_5 [] {
        mother_brain_visuals::flicker_body(game);
        MotherBrain_GenerateMixedExplosions(game);
        let E: i32 = 0;
        slot!(game, Enemy_MotherBrain[E].var_F = slot!(game, Enemy_MotherBrain[E].var_F).wrapping_sub(1));
        if (slot!(game, Enemy_MotherBrain[E].var_F) & 0x8000) != 0 {
            slot!(game, Enemy_MotherBrain[E].var_F = 16);
            let v2: u16 = (slot!(game, Enemy_MotherBrain[E].var_37)).wrapping_add(1);
            slot!(game, Enemy_MotherBrain[E].var_37 = v2);
            if mother_brain_visuals::fade(Fade::ToBlack, (v2).wrapping_sub(1), game) & 1 != 0 {
                word!(game.enemy_bg2_tilemap_size = 710);
                word_transfers!(game, descending, 356; le(tilemap_stuff, 355) <- value(824));
                word!(game.nmi_flag_bg2_enemy_vram_transfer = 1);
                slot!(game, Enemy_MotherBrain[E].base.properties = slot!(game, Enemy_MotherBrain[E].base.properties) & 0xdeff | 0x100);
                slot!(game, Enemy_MotherBrain[E].base.extra_properties = 0);
                slot!(game, Enemy_MotherBrain[E].var_A = (0xa9b013i32) as u16);
                slot!(game, Enemy_MotherBrain[E].var_F = 16);
            }
        }
    };
    [11120917] MotherBrain_Phase3_Death_7 [] {
        MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9C29 as u16, game);
        slot!(game, Enemy_MotherBrain[0x40i32].var_A = (0xa987d0i32) as u16);
        let E: i32 = 0;
        slot!(game, Enemy_MotherBrain[E].var_F = 0);
        slot!(game, Enemy_MotherBrain[E].var_A = (0xa9b12di32) as u16);
        MotherBrain_Phase3_Death_8(game);
    };
    [11120990] MotherBrain_Phase3_Death_9 [] {
        if ProcessSpriteTilesTransfers(0xa9, addr_stru_A99003 as u16, game) != 0 {
            let E: i32 = 0;
            slot!(game, Enemy_MotherBrain[E].var_A = (0xa9b173i32) as u16);
            slot!(game, Enemy_MotherBrain[E].var_F = 32);
        }
    };
    [11121109] MotherBrain_Phase3_Death_13 [] {
        if ProcessCorpseRotting(0x40, game) & 1 != 0 {
            let dms_var_53: u16 = slot!(game, Enemy_DeadMonsters[0x40i32].var_53);
            ProcessCorpseRottingVramTransfers(dms_var_53, game);
        } else {
            let mut E1: i32 = 0x40;
            slot!(game, Enemy_DeadMonsters[E1].base.properties = slot!(game, Enemy_DeadMonsters[E1].base.properties) & 0xdeff | 0x100);
            slot!(game, Enemy_DeadMonsters[E1].base.extra_properties = 0);
            QueueMusic_Delayed8(0, game);
            QueueMusic_Delayed8(0xff24, game);
            E1 = 0;
            slot!(game, Enemy_DeadMonsters[E1].var_A = (0xa9b211i32) as u16);
            slot!(game, Enemy_DeadMonsters[E1].var_F = 20);
            MotherBrain_Phase3_Death_14_20framedelay(game);
        };
    };
    [11121240] MotherBrain_Phase3_Death_15_LoadEscapeTimerTiles [] {
        if ProcessSpriteTilesTransfers(0xa6, addr_stru_A6C4CB as u16, game) != 0 {
            slot!(game, Enemy_MotherBrain[0i32].var_A = (0xa9b26di32) as u16);
            MotherBrain_Phase3_Death_16_StartEscape(game);
        }
    };
    [11121261] MotherBrain_Phase3_Death_16_StartEscape [] {
        if ProcessSpriteTilesTransfers(0xa9, addr_stru_A9902F as u16, game) != 0 {
            WriteColorsToPalette(
                0x122,
                0xa9,
                (addr_kMotherBrainPalette_5 as i32 + 2) as u16,
                0xe,
                game,
            );
            QueueMusic_Delayed8(7, game);
            word! {game; earthquake_type = 5; earthquake_timer = -1i32 as u16; }
            palette_effects::spawn(addr_kPalfx_FFC9 as u16, game);
            palette_effects::spawn(addr_kPalfx_FFCD as u16, game);
            palette_effects::spawn(addr_kPalfx_FFD1 as u16, game);
            palette_effects::spawn(addr_kPalfx_FFD5 as u16, game);
            slot!(game, Enemy_MotherBrain[0x40i32].var_02 = 0);
            SetupZebesEscapeTypewriter(game);
            let E: i32 = 0;
            slot!(game, Enemy_MotherBrain[E].var_F = 32);
            let mut v1: u16 = (0xa9b2d1i32) as u16;
            if word!(game.japanese_text_flag) == 0 {
                v1 = (0xa9b2e3i32) as u16;
            }
            slot!(game, Enemy_MotherBrain[E].var_A = v1);
        }
    };
    [11121379] MotherBrain_Phase3_Death_18_TypesZebesText [] {
        if HandleTypewriterText_Ext(0x2610, game) & 1 != 0 {
            let E: i32 = 0;
            slot!(game, Enemy_MotherBrain[E].var_A = (0xa9b2f9i32) as u16);
            slot!(game, Enemy_MotherBrain[E].var_F = 32);
        }
    };
    [11121401] MotherBrain_Phase3_Death_19_EscapeDoorExploding [] {
        MotherBrain_GenerateEscapeDoorExploding(game);
        let E: i32 = 0;
        slot!(game, Enemy_MotherBrain[E].var_F = slot!(game, Enemy_MotherBrain[E].var_F).wrapping_sub(1));
        if (slot!(game, Enemy_MotherBrain[E].var_F) & 0x8000) != 0 {
            CallSomeSamusCode(0xf, game);
            word!(game.timer_status = 2);
            SetBossBitForCurArea(2, game);
            SetEventHappened(0xe, game);
            slot!(game, Enemy_MotherBrain[E].var_A = (0xa9b32ai32) as u16);
            let E1: i32 = 0x40;
            slot! { game, Enemy_MotherBrain[E1]; var_E = 0, var_F = 0 }
        }
    };
    [11121450] MotherBrain_Phase3_Death_20_BlowUpEscapeDoor [] {
        MotherBrain_ExplodeEscapeDoor(game);
        slot!(game, Enemy_MotherBrain[0i32].var_A = (0xa9b33ci32) as u16);
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0, 0x6, 0xb677 as VoidP), game);
    };
    [11121468] MotherBrain_Phase3_Death_21_KeepEarthquakeGoing [] {
        if word!(game.earthquake_timer) == 0 {
            word!(game.earthquake_timer -= 1);
        }
    };
    [11122561] MotherBrain_FiringBomb_DecideOnWalking [] {
        if word!(game.random_number) >= 0xff80 {
            MotherBrain_FiringBomb_DecideOnCrouching(game);
            return;
        }
        let mut v0: u16 = 64;
        if word!(game.random_number) < 0x6000 {
            v0 = 96;
        }
        let E: i32 = 0;
        if ((v0) as i16).wrapping_sub((slot!(game, Enemy_MotherBrain[E].base.x_pos)) as i16) >= 0 || {
            slot!(game, Enemy_MotherBrain[E].var_F = v0);
            MotherBrain_MakeWalkBackwards(v0, 6, game) & 1 != 0
        } {
            MotherBrain_FiringBomb_DecideOnCrouching(game);
        } else {
            slot!(game, Enemy_MotherBrain[E].var_A = (0xa9b7aci32) as u16);
        };
    };
    [11122630] MotherBrain_FiringBomb_Crouch [] {
        if MotherBrain_MakeHerCrouch(game) & 1 != 0 {
            MotherBrain_B7CB(game);
        }
    };
    [11122680] MotherBrain_FiringBomb_Standup [] {
        if MotherBrain_MakeHerStandUp(game) & 1 != 0 {
            MotherBrain_FiringBomb_Finish(game);
        }
    };
    [11122702] MotherBomb_FiringLaser_PositionHead [] {
        let mut v0: i16 = 8;
        let E1: i32 = 0x40;
        if ((slot!(game, Enemy_MotherBrain[E1].base.y_pos)) as i16).wrapping_sub((word!(game.samus_y_pos)) as i16) < 0 {
            v0 = 6;
        }
        slot!(game, Enemy_MotherBrain[E1].var_32 = v0 as u16);
        slot!(game, Enemy_MotherBrain[E1].var_33 = v0 as u16);
        slot!(game, Enemy_MotherBrain[E1].var_34 = 512);
        let E: i32 = 0;
        slot!(game, Enemy_MotherBrain[E].var_A = (0xa9b839i32) as u16);
        slot!(game, Enemy_MotherBrain[E].var_F = 4);
    };
    [11122813] MotherBomb_FiringDeathBeam [] {
        let v0: i32 = slot!(game, Enemy_MotherBrain[0i32].var_17) as i32;
        off_A9B887[v0 as usize](game);
    };
    [11122923] MotherBomb_FiringRainbowBeam_0 [] {
        MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9C87 as u16, game);
        let E1: i32 = 0x40;
        slot! { game, Enemy_MotherBrain[E1]; var_34 = 64, var_31 = 1, var_32 = 2, var_33 = 4 };
        let E: i32 = 0;
        slot!(game, Enemy_MotherBrain[E].var_A = (0xa9b91ai32) as u16);
        slot!(game, Enemy_MotherBrain[E].var_F = 256);
    };
    [11122987] MotherBomb_FiringRainbowBeam_2_RetractNeck [] {
        if MotherBrain_WalkBackwardsSlowlyAndRetractHead(0x28, game) & 1 != 0 {
            let E: i32 = 0;
            slot!(game, Enemy_MotherBomb[E].var_A = (0xa9b93fi32) as u16);
            slot!(game, Enemy_MotherBomb[E].var_F = 256);
            MotherBomb_FiringRainbowBeam_3_Wait(game);
        }
    };
    [11123025] MotherBomb_FiringRainbowBeam_4_ExtendNeckDown [] {
        word!(game.cooldown_timer = 8);
        let E1: i32 = 0x40;
        slot! { game, Enemy_MotherBrain[E1]; var_32 = 6, var_33 = 6 }
        slot!(game, Enemy_MotherBrain[E1].var_34 = 1280);
        let E: i32 = 0;
        slot!(game, Enemy_MotherBrain[E].var_A = (0xa9b975i32) as u16);
        slot!(game, Enemy_MotherBrain[E].var_F = 16);
        MotherBomb_FiringRainbowBeam_5_StartFiring(game);
    };
    [11123061] MotherBomb_FiringRainbowBeam_5_StartFiring [] {
        MotherBrain_AimBeamAndIncrWidth(game);
        if word!(game.power_bomb_flag) == 0 {
            let E: i32 = 0;
            slot!(game, Enemy_MotherBrain[E].var_F = slot!(game, Enemy_MotherBrain[E].var_F).wrapping_sub(1));
            if (slot!(game, Enemy_MotherBrain[E].var_F) & 0x8000) != 0 && word!(game.power_bomb_flag) == 0 {
                word!(game.cooldown_timer = 0);
                MotherBrain_BodyRainbowBeamPalAnimIndex0(game);
                MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9C77 as u16, game);
                slot!(game, Enemy_MotherBrain[E].var_33 = 512);
                slot!(game, Enemy_MotherBrain[E].var_09 = spawn_rainbow_hdma(game));
                let E1: i32 = 0x40;
                slot! { game, Enemy_MotherBrain[E1]; var_34 = 64, var_31 = 1, var_32 = 2, var_33 = 4 };
                slot! { game, Enemy_MotherBrain[E]; parameter_1 = 0, parameter_2 = 0 }
                let mut v4: u16 = 5;
                if ((word!(game.samus_health)) as i16).wrapping_sub(700) < 0 {
                    v4 = 24;
                }
                CallSomeSamusCode(v4, game);
                slot!(game, Enemy_MotherBrain[E].var_15 = 6);
                slot!(game, Enemy_MotherBrain[E].var_A = (0xa9b9e5i32) as u16);
            }
        }
    };
    [11123173] MotherBomb_FiringRainbowBeam_6_MoveSamusToWall [] {
        MotherBrain_PlayRainbowBeamSfx(game);
        MotherBrain_HandleRainbowBeamPalette(game);
        MotherBrain_AimBeamAndIncrWidth(game);
        MotherBrain_HandleRainbowBeamExplosions(game);
        if MotherBrain_MoveSamusTowardsWallDueToBeam(game) & 1 != 0 {
            let E: i32 = 0;
            slot!(game, Enemy_MotherBomb[E].var_A = (0xa9ba00i32) as u16);
            slot!(game, Enemy_MotherBomb[E].var_F = 0);
        }
    };
    [11123200] MotherBomb_FiringRainbowBeam_7_DelayFrame [] {
        MotherBrain_PlayRainbowBeamSfx(game);
        MotherBrain_HandleRainbowBeamPalette(game);
        MotherBrain_AimBeamAndIncrWidth(game);
        MotherBrain_HandleRainbowBeamExplosions(game);
        MotherBrain_MoveSamusTowardsWallDueToBeam(game);
        let E: i32 = 0;
        slot!(game, Enemy_MotherBomb[E].var_F = slot!(game, Enemy_MotherBomb[E].var_F).wrapping_sub(1));
        if (slot!(game, Enemy_MotherBomb[E].var_F) & 0x8000) != 0 {
            word! { game, 8; earthquake_type, earthquake_timer }
            slot!(game, Enemy_MotherBomb[E].var_A = (0xa9ba27i32) as u16);
        }
    };
    [11123239] MotherBomb_FiringRainbowBeam_8_StartDrainSamus [] {
        let E: i32 = 0;
        slot!(game, Enemy_MotherBomb[E].var_A = (0xa9ba3ci32) as u16);
        slot!(game, Enemy_MotherBomb[E].var_F = 299);
        word! {game; earthquake_timer = 299; earthquake_type = 8; }
        MotherBomb_FiringRainbowBeam_9_DrainingSamus(game);
    };
    [11123260] MotherBomb_FiringRainbowBeam_9_DrainingSamus [] {
        MotherBrain_PlayRainbowBeamSfx(game);
        MotherBrain_HandleRainbowBeamPalette(game);
        MotherBrain_AimBeamAndIncrWidth(game);
        MotherBrain_HandleRainbowBeamExplosions(game);
        vitals::drain_health_by_rainbow_beam(game);
        vitals::drain_ammo_by_rainbow_beam(game);
        MotherBrain_MoveSamusTowardsMiddleOfWall(game);
        let E: i32 = 0;
        slot!(game, Enemy_MotherBomb[E].var_F = slot!(game, Enemy_MotherBomb[E].var_F).wrapping_sub(1));
        if (slot!(game, Enemy_MotherBomb[E].var_F) & 0x8000) != 0 {
            slot!(game, Enemy_MotherBomb[E].var_A = (0xa9ba5ei32) as u16);
        }
    };
    [11123294] MotherBomb_FiringRainbowBeam_10_FinishFiringRainbow [] {
        MotherBrain_PlayRainbowBeamSfx(game);
        MotherBrain_HandleRainbowBeamPalette(game);
        MotherBrain_AimBeam(game);
        MotherBrain_HandleRainbowBeamExplosions(game);
        let E: i32 = 0;
        let v1: u16 = (slot!(game, Enemy_MotherBrain[E].var_33)).wrapping_sub(384);
        slot!(game, Enemy_MotherBrain[E].var_33 = v1);
        if v1 as i32 - 512i32 & 0x8000 != 0 {
            slot!(game, Enemy_MotherBrain[E].var_33 = 512);
            slot!(game, Enemy_MotherBrain[E].parameter_1 = -256i32 as u16);
            slot!(game, Enemy_MotherBrain[E].parameter_2 = 0);
            word! { game, 0;
                hdma_object_channels_bitmask[(slot!(game, Enemy_MotherBrain[E].var_09) as i32 >> 1)], earthquake_timer,
            }
            MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9C87 as u16, game);
            MotherBrain_SetupBrainNormalPal(game);
            MotherBrain_WriteDefaultPalette(game);
            QueueSfx1_Max6(2, game);
            slot!(game, Enemy_MotherBrain[E].var_16 = 0);
            CallSomeSamusCode(1, game);
            word!(game.cooldown_timer = 8);
            slot!(game, Enemy_MotherBrain[E].var_A = (0xa9bac4i32) as u16);
        }
    };
    [11123396] MotherBomb_FiringRainbowBeam_11_LetSamusFall [] {
        player_mother_brain(0, game);
        slot!(game, Enemy_MotherBomb[0i32].var_A = (0xa9bad1i32) as u16);
        MotherBomb_FiringRainbowBeam_12_WaitForSamusHitGround(game);
    };
    [11123409] MotherBomb_FiringRainbowBeam_12_WaitForSamusHitGround [] {
        if MotherBrain_MoveSamusForFallingAfterBeam(game) & 1 != 0 {
            slot!(game, Enemy_MotherBomb[0i32].var_A = (0xa9baddi32) as u16);
        }
    };
    [11124037] MotherBrain_Phase2Cut_0 [] {
        let v0: u16 = { let amount = 0x50; actors::combat::suit_damage(&mut Runtime::new(game), amount) };
        if ((4 * v0 as i32 + 20 - word!(game.samus_health) as i32) as i16) >= 0 {
            slot!(game, Enemy_MotherBrain[0i32].var_A = (0xa9bd98i32) as u16);
        } else if (word!(game.random_number) & 0xfff) < 0xfa0 {
            if (slot!(game, Enemy_MotherBrain[0i32].base.frame_counter) & 0x1f) == 0 {
                MotherBrain_MaybeStandupOrLeanDown(game);
            }
        } else {
            let mut v1: u16 = 0;
            if ((({ let amount = 0xa0; actors::combat::suit_damage(&mut Runtime::new(game), amount) }) as i32 + 20 - word!(game.samus_health) as i32)
                as i16)
                >= 0
                || {
                    v1 = addr_kMotherBrain_Ilist_9ECC as u16;
                    (word!(game.random_number) & 0xfff) < 0xff0
                }
            {
                v1 = addr_kMotherBrain_Ilist_9D7F as u16;
            }
            MotherBrain_SetBrainInstrs(v1, game);
        };
    };
    [11124120] MotherBrain_Phase2Cut_1 [] {
        if MotherBrain_MakeHerStandUp(game) & 1 != 0 {
            let E: i32 = 0;
            slot!(game, Enemy_MotherBrain[E].var_A = (0xa9bda9i32) as u16);
            slot!(game, Enemy_MotherBrain[E].var_F = 16);
            MotherBrain_Phase2Cut_2(game);
        }
    };
    [11124178] MotherBrain_Phase2Cut_4 [] {
        if ProcessSpriteTilesTransfers(0xa9, addr_stru_A98FE5 as u16, game) != 0 {
            MotherBrain_RetractHead(game);
            MotherBrain_SpawnShitroidInCutscene(game);
            let E: i32 = 0;
            slot!(game, Enemy_MotherBrain[E].var_A = (0xa9bdedi32) as u16);
            slot!(game, Enemy_MotherBrain[E].var_F = 256);
        }
    };
    [11124317] MotherBrain_DrainedByShitroid_1 [] {
        MotherBrain_HandleRainbowBeamPalette(game);
        let E: i32 = 0;
        slot!(game, Enemy_MotherBrain[E].var_F = slot!(game, Enemy_MotherBrain[E].var_F).wrapping_sub(1));
        if (slot!(game, Enemy_MotherBrain[E].var_F) & 0x8000) != 0 {
            slot!(game, Enemy_MotherBrain[E].var_A = (0xa9be96i32) as u16);
            let E1: i32 = 0x40;
            slot!(game, Enemy_MotherBrain[E1].var_08 = (0xa9bfd0i32) as u16);
            slot!(game, Enemy_MotherBrain[E1].var_06 = 0);
            slot! { game, Enemy_MotherBrain[E1]; var_07 = 2, var_32 = 2 }
            slot!(game, Enemy_MotherBrain[E1].var_33 = 4);
        }
    };
    [11124545] MotherBrain_DrainedByShitroid_4 [] {
        if MotherBrain_MakeWalkBackwards(0x28, 0, game) & 1 != 0 {
            slot!(game, Enemy_MotherBrain[0i32].var_A = (0xa9bf56i32) as u16);
            slot!(game, Enemy_MotherBrain[0x40i32].var_33 = 0);
            MotherBrain_DrainedByShitroid_5(game);
        }
    };
    [11124922] MotherBrain_Phase2_Revive_4 [] {
        if MotherBrain_MakeHerStandUp(game) & 1 != 0 {
            let E1: i32 = 0x40;
            slot! { game, Enemy_MotherBrain[E1]; var_32 = 6, var_33 = 6 }
            slot!(game, Enemy_MotherBrain[E1].var_34 = 1280);
            slot!(game, Enemy_MotherBrain[E1].var_31 = 1);
            let E: i32 = 0;
            slot!(game, Enemy_MotherBrain[E].var_A = (0xa9c0e4i32) as u16);
            slot!(game, Enemy_MotherBrain[E].var_F = 16);
            MotherBrain_Phase2_Revive_5(game);
        }
    };
    [11125063] MotherBrain_Phase2_Revive_8 [] {
        if MotherBrain_MakeHerStandUp(game) & 1 != 0 {
            slot!(game, Enemy_MotherBrain[0i32].var_A = (0xa9c15ci32) as u16);
            MotherBrain_MakeWalkForwards(0xa, 0x50, game);
            MotherBrain_Phase2_MurderShitroid_1(game);
        }
    };
    [11125084] MotherBrain_Phase2_MurderShitroid_1 [] {
        MotherBrain_MaybeStandupOrLeanDown(game);
        if (word!(game.random_number) & 0x8000) != 0 {
            let mut v0: u16 = addr_kMotherBrain_Ilist_9DBB as u16;
            if slot!(game, Enemy_MotherBrain[0x40i32].var_0A) != 0 {
                v0 = addr_kMotherBrain_Ilist_9DB1 as u16;
            }
            MotherBrain_SetBrainInstrs(v0, game);
            let E: i32 = 0;
            slot!(game, Enemy_MotherBrain[E].var_A = (0xa9c182i32) as u16);
            slot!(game, Enemy_MotherBrain[E].var_F = 64);
        }
    };
    [11125134] MotherBrain_Phase2_PrepareForFinalShitroid [] {
        MotherBrain_MakeHerStandUp(game);
        MotherBrain_MakeWalkBackwards(0x40, 4, game);
    };
    [11125146] MotherBrain_Phase2_ExecuteFinalkShitroid [] {
        MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9DB1 as u16, game);
        slot!(game, Enemy_MotherBrain[0i32].var_A = (0xa9c1a6i32) as u16);
    };
    [11125257] MotherBrain_Phase3_Fighting_Main [] {
        if slot!(game, Enemy_MotherBrain[0x40i32].base.health) != 0 {
            MotherBrain_Phase3_NeckHandler(game);
            MotherBrain_Phase3_WalkHandler(game);
            let E: i32 = 0;
            if slot!(game, Enemy_MotherBrain[E].var_02) == 0 && slot!(game, Enemy_MotherBrain[E].var_06) == 0 && (word!(game.random_number) & 0x8000) != 0 {
                let mut v1: u16 = addr_kMotherBrain_Ilist_9F00 as u16;
                if (word!(game.random_number) as u8) >= 0x80 {
                    v1 = addr_kMotherBrain_Ilist_9DBB as u16;
                }
                MotherBrain_SetBrainInstrs(v1, game);
                slot!(game, Enemy_MotherBrain[E].var_A = (0xa9c24ei32) as u16);
                slot!(game, Enemy_MotherBrain[E].var_F = 64);
            }
        } else {
            slot!(game, Enemy_MotherBrain[0i32].var_A = (0xa9aee1i32) as u16);
        };
    };
    @functions [pub unsafe fn (game: &mut Game)];
    @prefix { let E1: i32 = 0x40; };
    [11123421] MotherBomb_FiringRainbowBeam_13_LowerHead [] {
        slot! { game, Enemy_MotherBrain[E1]; var_34 = 64, var_31 = 1, var_32 = 2, var_33 = 4 };
        let E: i32 = 0;
        slot!(game, Enemy_MotherBrain[E].var_A = (0xa9bb06i32) as u16);
        slot!(game, Enemy_MotherBrain[E].var_F = 128);
    };
    [11124374] MotherBrain_DrainedByShitroid_2 [] {
        if slot!(game, Enemy_MotherBrain[E1].var_00) == 0 {
            slot!(game, Enemy_MotherBrain[E1].var_00 = 50);
        }
        MotherBrain_HandleRainbowBeamPalette(game);
        CallMotherBrainPainfulWalk(((slot!(game, Enemy_MotherBrain[E1].var_08)) as u32) | (0xa90000u32), game);
        let v2: u16 = (2u16).wrapping_mul(slot!(game, Enemy_MotherBrain[E1].var_06));
        slot!(game, Enemy_MotherBrain[E1].var_07 = g_byte_A9BEEE[v2 as usize] as u16);
        slot!(game, Enemy_MotherBrain[E1].var_34 = g_word_A9BEFE[(v2 >> 1) as usize]);
        if slot!(game, Enemy_MotherBrain[E1].var_06) == 6 {
            let E: i32 = 0;
            slot!(game, Enemy_MotherBrain[E].var_16 = 0);
            slot!(game, Enemy_MotherBrain[E1].var_10 = 0);
            MotherBrain_WritePhase2DeathPalette(game);
            QueueSfx1_Max6(2, game);
            slot!(game, Enemy_MotherBrain[E].var_A = (0xa9bf0ei32) as u16);
        }
    };
    [11124494] MotherBrain_DrainedByShitroid_3 [] {
        CallMotherBrainPainfulWalk(((slot!(game, Enemy_MotherBrain[E1].var_08)) as u32) | (0xa90000u32), game);
        if slot!(game, Enemy_MotherBrain[E1].var_06) as i32 - 8i32 & 0x8000 == 0 {
            slot!(game, Enemy_MotherBrain[E1].var_34 = 64);
            slot! { game, Enemy_MotherBrain[E1]; var_32 = 8, var_33 = 8 }
            MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9C39 as u16, game);
            slot!(game, Enemy_MotherBrain[0i32].var_A = (0xa9bf41i32) as u16);
            MotherBrain_DrainedByShitroid_4(game);
        }
    };
    [11124566] MotherBrain_DrainedByShitroid_5 [] {
        if { let at = slot!(@at Enemy_MotherBrain, E1, var_32); u32::from_ne_bytes([game.ram().read_byte(at), game.ram().read_byte(at + 1), game.ram().read_byte(at + 2), game.ram().read_byte(at + 3)]) } == 0 {
            slot!(game, Enemy_MotherBrain[E1].var_12 = (slot!(game, Enemy_MotherBrain[E1].var_33)) | (slot!(game, Enemy_MotherBrain[E1].var_32)));
            let E: i32 = 0;
            if slot!(game, Enemy_MotherBrain[E].var_02) == 0 {
                MotherBrain_SetBodyInstrs(addr_kMotherBrain_Ilist_9A26 as u16, game);
                slot!(game, Enemy_MotherBrain[E].var_A = (0xa9bf7di32) as u16);
                slot!(game, Enemy_MotherBrain[E].var_F = 64);
            }
        }
    };
    @functions [pub unsafe fn (game: &mut Game)];
    @prefix { let E: i32 = 0; };
    [11110578] MotherBrainBody_FakeDeath_Descent_5 [] {
        slot!(game, Enemy_MotherBrain[E].var_F = slot!(game, Enemy_MotherBrain[E].var_F).wrapping_sub(1));
        if (slot!(game, Enemy_MotherBrain[E].var_F) & 0x8000) != 0 {
            slot!(game, Enemy_MotherBrain[E].var_F = 8);
            let v2: u16 = (slot!(game, Enemy_MotherBrain[E].var_37)).wrapping_add(1);
            slot!(game, Enemy_MotherBrain[E].var_37 = v2);
            if mother_brain_visuals::fade(Fade::FakeDeathToGray, (v2).wrapping_sub(1), game) & 1 != 0 {
                slot!(game, Enemy_MotherBrain[E].var_A = (0xa988d3i32) as u16);
            }
        }
        MotherBrainBody_FakeDeath_Descent_6(game);
    };
    MotherBrain_HandleFakeDeathExplosions [] {
        let mut v1: i16 = ((slot!(game, Enemy_MotherBrain[E].var_38)) as i16).wrapping_sub(1);
        if v1 < 0 {
            slot!(game, Enemy_MotherBrain[E].var_38 = 8);
            let mut v2: i16 = ((slot!(game, Enemy_MotherBrain[E].var_39)) as i16).wrapping_sub(1);
            if v2 < 0 {
                v2 = 7;
            }
            slot!(game, Enemy_MotherBrain[E].var_39 = v2 as u16);
            let v3: i32 = ((4u16).wrapping_mul((v2) as u16)) as i32 >> 1;
            game.eproj_spawn_pt = Point16U {
                x: crate::rtl::rom_table_word!(game, 0xa98929, v3 as isize),
                y: crate::rtl::rom_table_word!(game, 0xa98929, (v3 + 1) as isize),
            };
            let v4: u16 = (if word!(game.random_number) < 0x4000 {
                12
            } else {
                3
            }) as u16;
            SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, v4, game);
            QueueSfx2_Max3(0x24, game);
        } else {
            slot!(game, Enemy_MotherBrain[E].var_38 = v1 as u16);
        };
    };
    [11112013] MotherBrainBody_FakeDeath_Ascent_13_Raise [] {
        if !((word!(game.nmi_frame_counter_word) & 3) != 0 || {
            MotherBrain_SpawnDustCloudsForAscent(game);
            word!(game.reg_BG2VOFS += 2);
            let mut v1: u16 = (slot!(game, Enemy_MotherBrain[E].base.y_pos)).wrapping_sub(2);
            slot!(game, Enemy_MotherBrain[E].base.y_pos = v1);
            v1 >= 0xbd
        }) {
            word!(game.enemy_bg2_tilemap_size = 320);
            slot!(game, Enemy_MotherBrain[E].base.y_pos = 188);
            word! { game, 0;
                earthquake_timer, hdma_object_channels_bitmask[(slot!(game, Enemy_MotherBrain[E].var_09) as i32 >> 1)],
            }
            MotherBrain_SetBodyInstrs(addr_kMotherBrain_Ilist_99AA as u16, game);
            slot!(game, Enemy_MotherBrain[E].var_A = (0xa98e95i32) as u16);
            MotherBrainBody_FakeDeath_Ascent_14_WaitForUncouching(game);
        }
    };
    [11112085] MotherBrainBody_FakeDeath_Ascent_14_WaitForUncouching [] {
        if slot!(game, Enemy_MotherBrain[E].var_02) == 0 {
            slot!(game, Enemy_MotherBrain[E].var_37 = 0);
            slot!(game, Enemy_MotherBrain[E].var_A = (0xa98eaai32) as u16);
            slot!(game, Enemy_MotherBrain[E].var_F = 0);
        }
    };
    MotherBrain_HandleNeck [] {
        slot!(game, Enemy_MotherBrain[E].var_0A = (slot!(game, Enemy_MotherBrain[E].base.x_pos)).wrapping_sub(80));
        slot!(game, Enemy_MotherBrain[E].var_0B = (slot!(game, Enemy_MotherBrain[E].base.y_pos)).wrapping_add(46));
        let E1: i32 = 0x40;
        if slot!(game, Enemy_MotherBrain[E1].var_31) != 0 {
            MotherBrain_HandleNeckLower(game);
            MotherBrain_HandleNeckUpper(game);
        }
        let mut r18: u16 = slot!(byte game, Enemy_MotherBrain[E1].var_20[1]) as u16;
        slot!(game, Enemy_MotherBrain[E1].var_22 = (slot!(game, Enemy_MotherBrain[E].var_0A) as i32 + ComputeSinMult(slot!(game, Enemy_MotherBrain[E1].var_24), r18, game) as i32 + 112) as u16);
        slot!(game, Enemy_MotherBrain[E1].var_23 = (slot!(game, Enemy_MotherBrain[E].var_0B) as i32 + ComputeCosMult(slot!(game, Enemy_MotherBrain[E1].var_24), r18, game) as i32 - 96i32) as u16);
        slot!(game, Enemy_MotherBrain[E1].var_25 = (slot!(game, Enemy_MotherBrain[E].var_0A) as i32 + ComputeSinMult(slot!(game, Enemy_MotherBrain[E1].var_27), r18, game) as i32 + 112) as u16);
        slot!(game, Enemy_MotherBrain[E1].var_26 = (slot!(game, Enemy_MotherBrain[E].var_0B) as i32 + ComputeCosMult(slot!(game, Enemy_MotherBrain[E1].var_27), r18, game) as i32 - 96i32) as u16);
        slot!(game, Enemy_MotherBrain[E1].var_28 = (slot!(game, Enemy_MotherBrain[E].var_0A) as i32 + ComputeSinMult(slot!(game, Enemy_MotherBrain[E1].var_2A), r18, game) as i32 + 112) as u16);
        slot!(game, Enemy_MotherBrain[E1].var_29 = (slot!(game, Enemy_MotherBrain[E].var_0B) as i32 + ComputeCosMult(slot!(game, Enemy_MotherBrain[E1].var_2A), r18, game) as i32 - 96i32) as u16);
        r18 = slot!(byte game, Enemy_MotherBrain[E1].var_21[1]) as u16;
        slot!(game, Enemy_MotherBrain[E1].var_2B = (slot!(game, Enemy_MotherBrain[E1].var_28)).wrapping_add(ComputeSinMult(slot!(game, Enemy_MotherBrain[E1].var_2D), r18, game)));
        slot!(game, Enemy_MotherBrain[E1].var_2C = (slot!(game, Enemy_MotherBrain[E1].var_29)).wrapping_add(ComputeCosMult(slot!(game, Enemy_MotherBrain[E1].var_2D), r18, game)));
        slot!(game, Enemy_MotherBrain[E1].var_2E = (slot!(game, Enemy_MotherBrain[E1].var_28)).wrapping_add(ComputeSinMult(slot!(game, Enemy_MotherBrain[E1].var_30), r18, game)));
        slot!(game, Enemy_MotherBrain[E1].var_2F = (slot!(game, Enemy_MotherBrain[E1].var_29)).wrapping_add(ComputeCosMult(slot!(game, Enemy_MotherBrain[E1].var_30), r18, game)));
    };
    [11120353] MotherBrain_Phase3_Death_0 [] {
        slot!(game, Enemy_MotherBrain[E].base.properties = (slot!(game, Enemy_MotherBrain[E].base.properties)) | ((kEnemyProps_Tangible) as u16));
        let E1: i32 = 0x40;
        slot!(game, Enemy_MotherBrain[E1].base.properties = (slot!(game, Enemy_MotherBrain[E1].base.properties)) | (0x400u16));
        slot!(game, Enemy_MotherBrain[E].var_04 = 0);
        if MotherBrain_MakeWalkBackwards(0x28, 6, game) & 1 != 0 {
            slot!(game, Enemy_MotherBrain[E].var_A = (0xa9af12i32) as u16);
            slot!(game, Enemy_MotherBrain[E].var_F = 128);
            MotherBrain_Phase3_Death_1(game);
        }
    };
    [11120941] MotherBrain_Phase3_Death_8 [] {
        let mut v1: u16 = (slot!(game, Enemy_MotherBrain[E].var_F)).wrapping_add(32);
        slot!(game, Enemy_MotherBrain[E].var_F = v1);
        let E1: i32 = 0x40;
        let mut v3: i16 = ((slot!(game, Enemy_MotherBrain[E1].base.y_pos)) as i16).wrapping_add((byte!(v1; 1)) as i16);
        if (v3 as u16) >= 0xc4 {
            mother_brain_visuals::start_earthquake(2, game);
            slot!(game, Enemy_MotherBrain[E].var_A = (0xa9b15ei32) as u16);
            slot!(game, Enemy_MotherBrain[E].var_F = 256);
            v3 = 196;
        }
        slot!(game, Enemy_MotherBrain[E1].base.y_pos = v3 as u16);
    };
    [11121361] MotherBrain_Phase3_Death_17_SpawnTimeBomb [] {
        slot!(game, Enemy_MotherBrain[E].var_F = slot!(game, Enemy_MotherBrain[E].var_F).wrapping_sub(1));
        if (slot!(game, Enemy_MotherBrain[E].var_F) & 0x8000) != 0 {
            slot!(game, Enemy_MotherBrain[E].var_A = (0xa9b2e3i32) as u16);
            SpawnEprojWithRoomGfx(
                addr_kEproj_TimeBombSetJapaneseText as u16,
                (0xa9b2e3i32) as u16,
                game,
            );
        }
        MotherBrain_Phase3_Death_18_TypesZebesText(game);
    };
    MotherBrainsBrain_Shot [] {
        if slot!(game, Enemy_MotherBrain[E].var_00) != 0 {
            MotherBrain_Phase23_ShotReaction(game);
            if slot!(game, Enemy_MotherBrain[E].var_00) == 1 {
                actors::combat::create_dud(&mut Runtime::new(game));
            } else {
                actors::combat::shot_reaction(&mut Runtime::new(game), actors::api::ShotHandling::ReactionOnly);
            }
        } else {
            let v1: u16 = ((game.ram().read_byte(0xc18 + 2 * word!(game.collision_detection_index) as usize + 1))
                as u16)
                & (7u16);
            if g_byte_A9B546[v1 as usize] != 0 {
                crate::game::room_mechanisms::add_glass_hits(
                    39,
                    u16::from(g_byte_A9B546[v1 as usize]),
                    game,
                );
                QueueSfx2_Max6(0x6e, game);
                let mut v2: i16 = 13;
                let E1: i32 = 0x40;
                let flash_timer: u16 = slot!(game, Enemy_MotherBrain[E1].base.flash_timer);
                if flash_timer != 0 && flash_timer & 1 != 0 {
                    v2 = 14;
                }
                slot!(game, Enemy_MotherBrain[E1].base.flash_timer = v2 as u16);
                actors::combat::shot_reaction(&mut Runtime::new(game), actors::api::ShotHandling::ReactionOnly);
            }
        };
    };
    MotherBrain_Phase3_BeamShotReaction [] {
        let mut v1: i16 = ((slot!(game, Enemy_MotherBrain[E].var_07)) as i16).wrapping_sub(266);
        if v1 < 0 {
            slot!(game, Enemy_MotherBrain[0x40i32].var_18 = (0xa9c395i32) as u16);
            v1 = 0;
            slot!(game, Enemy_MotherBrain[E].var_F = 0);
        }
        slot!(game, Enemy_MotherBrain[E].var_07 = v1 as u16);
    };
    [11122181] MotherBrain_Body_Phase2_Thinking [] {
        let E1: i32 = 0x40;
        if slot!(game, Enemy_MotherBrain[E1].base.health) == 0 {
            slot!(game, Enemy_MotherBrain[E].var_A = (0xa9b8ebi32) as u16);
            MotherBomb_FiringRainbowBeam_0(game);
            return;
        }
        if slot!(game, Enemy_MotherBrain[E].var_02) == 0 {
            let keep_walking: bool = if slot!(game, Enemy_MotherBrain[E1].base.health) as i32 - 4500i32 & 0x8000 == 0 {
                word!(game.random_number) >= 0x1000
            } else if word!(game.random_number) < 0x2000 {
                true
            } else if word!(game.random_number) >= 0xa000 {
                false
            } else {
                slot!(game, Enemy_MotherBrain[E].var_A = (0xa9b87di32) as u16);
                return;
            };
            if keep_walking {
                MotherBrain_HandleWalking(game)
            } else {
                slot!(game, Enemy_MotherBrain[E].var_A = (0xa9b64bi32) as u16);
            }
        }
    };
    [11122251] MotherBrain_Body_Phase2_TryAttack [] {
        let mut v0: i16 = (2i16).wrapping_mul((slot!(game, Enemy_MotherBrain[E].var_18)) as i16);
        if v0 != 0 {
            if v0 == 2 {
                MotherBrain_Phase2_Attack_Cooldown(game);
            } else {
                if v0 != 4 {
                    Unreachable(game);
                    loop {}
                }
                MotherBrain_Phase2_Attack_End(game);
            }
        } else {
            slot!(game, Enemy_MotherBrain[E].parameter_1 = 64);
            slot!(game, Enemy_MotherBrain[E].var_18 = slot!(game, Enemy_MotherBrain[E].var_18).wrapping_add(1));
            if MotherBrain_Phase2_DecideAttackStrategy_DoubleRet(game) != 0 {
                return;
            }
            let mut v2: u16 = addr_byte_A9B6DC as u16;
            let E1: i32 = 0x40;
            if abs16((slot!(game, Enemy_MotherBrain[E1].base.y_pos) as i32 + 4 - word!(game.samus_y_pos) as i32) as u16) < 0x20 {
                v2 = addr_byte_A9B6DF as u16;
            }
            let mut v4: u16 = 0;
            let v5 = RomPtr_A9(v2, game);
            if (word!(game.random_number) as u8) >= *v5 {
                v4 = 2;
                if (word!(game.random_number) as u8) >= *v5.offset(1) {
                    v4 = 4;
                    if (word!(game.random_number) as u8) >= *v5.offset(2) {
                        v4 = 6;
                    }
                }
            }
            let v6: u16 = crate::rtl::rom_table_word!(game, 0xa9b6d4, (v4 >> 1) as isize);
            if v6 as i32 == addr_kMotherBrain_Ilist_9ECC as i32 {
                if slot!(game, Enemy_MotherBrain[E1].var_05) as i32 - 1i32 & 0x8000 != 0 {
                    slot!(game, Enemy_MotherBrain[E].var_A = (0xa9b781i32) as u16);
                    MotherBrain_FiringBomb_DecideOnWalking(game);
                }
            } else if v6 as i32 == addr_kMotherBrain_Ilist_9F34 as i32 {
                slot!(game, Enemy_MotherBrain[E].var_A = (0xa9b80ei32) as u16);
                MotherBomb_FiringLaser_PositionHead(game);
            } else {
                MotherBrain_SetBrainInstrs(v6, game);
            }
        };
    };
    MotherBrain_Phase2_Attack_Cooldown [] {
        let fresh9: u16 = slot!(game, Enemy_MotherBrain[E].parameter_1);
        slot!(game, Enemy_MotherBrain[E].parameter_1 = slot!(game, Enemy_MotherBrain[E].parameter_1).wrapping_sub(1));
        if fresh9 == 1 {
            slot!(game, Enemy_MotherBrain[E].var_18 = slot!(game, Enemy_MotherBrain[E].var_18).wrapping_add(1));
        }
    };
    MotherBrain_Phase2_Attack_End [] {
        slot!(game, Enemy_MotherBrain[E].var_18 = 0);
        slot!(game, Enemy_MotherBrain[E].var_A = (0xa9b605i32) as u16);
    };
    [11122604] MotherBrain_FiringBomb_WalkingBackwards [] {
        if MotherBrain_MakeWalkBackwards(slot!(game, Enemy_MotherBrain[E].var_F), 6, game) & 1 != 0 {
            MotherBrain_FiringBomb_DecideOnCrouching(game);
        }
    };
    MotherBrain_PlayRainbowBeamSfx [] {
        let mut mbn_var_15: i16 = slot!(game, Enemy_MotherBrain[E].var_15) as i16;
        if mbn_var_15 >= 0 {
            slot!(game, Enemy_MotherBrain[E].var_15 = ((mbn_var_15) as u16).wrapping_sub(1));
            QueueSfx1_Max6(0x40, game);
            slot!(game, Enemy_MotherBrain[E].var_16 = 1);
        }
    };
    MotherBrain_AimBeamAndIncrWidth [] {
        let mut v1: i16 = ((slot!(game, Enemy_MotherBrain[E].var_33)) as i16).wrapping_add(384);
        if slot!(game, Enemy_MotherBrain[E].var_33) as i32 - 2688i32 & 0x8000 == 0 {
            v1 = 3072;
        }
        slot!(game, Enemy_MotherBrain[E].var_33 = v1 as u16);
        MotherBrain_AimBeam(game);
    };
    MotherBrain_HandleRainbowBeamExplosions [] {
        slot!(game, Enemy_MotherBrain[E].parameter_2 = slot!(game, Enemy_MotherBrain[E].parameter_2).wrapping_sub(1));
        if (slot!(game, Enemy_MotherBrain[E].parameter_2) & 0x8000) != 0 {
            slot!(game, Enemy_MotherBrain[E].parameter_2 = 8);
            slot!(game, Enemy_MotherBrain[E].parameter_1 = slot!(game, Enemy_MotherBrain[E].parameter_1).wrapping_add(1));
            let v2: i32 = slot!(game, Enemy_MotherBrain[E].parameter_1) as i32 & 7;
            game.eproj_spawn_pt = Point16U {
                x: g_word_A9BCA6[v2 as usize] as u16,
                y: g_word_A9BCB6[v2 as usize] as u16,
            };
            SpawnEprojWithRoomGfx(
                addr_kEproj_MotherBrainRainbowBeamExplosion as u16,
                game.eproj_spawn_pt.y,
                game,
            );
            QueueSfx2_Max6(0x24, game);
        }
    };
    [11124280] MotherBrain_DrainedByShitroid_0 [] {
        slot!(game, Enemy_MotherBrain[E].var_00 = 3);
        let E1: i32 = 0x40;
        slot! { game, Enemy_MotherBrain[E1]; var_32 = 8, var_33 = 8 }
        slot!(game, Enemy_MotherBrain[E1].var_34 = 1792);
        slot!(game, Enemy_MotherBrain[E].var_A = (0xa9be5di32) as u16);
        slot!(game, Enemy_MotherBrain[E].var_F = 48);
        MotherBrain_DrainedByShitroid_1(game);
    };
    [11124629] MotherBrain_DrainedByShitroid_7 [] {
        slot!(game, Enemy_MotherBrain[E].var_F = slot!(game, Enemy_MotherBrain[E].var_F).wrapping_sub(1));
        let v2: bool = (slot!(game, Enemy_MotherBrain[E].var_F) & 0x8000) != 0;
        if v2 && {
            slot!(game, Enemy_MotherBrain[E].var_F = 16);
            let mut v3: u16 = (slot!(game, Enemy_MotherBrain[E].var_37)).wrapping_add(1);
            slot!(game, Enemy_MotherBrain[E].var_37 = v3);
            mother_brain_visuals::fade(Fade::DrainedToGray, (v3).wrapping_sub(1), game) & 1 != 0
        } {
            let E1: i32 = 0x40;
            slot!(game, Enemy_MotherBrain[E1].base.health = -29536i32 as u16);
            slot!(game, Enemy_MotherBrain[E].var_1F = 1);
            slot!(game, Enemy_MotherBrain[E1].var_14 = 0);
            slot!(game, Enemy_MotherBrain[E].var_00 = 2);
            slot!(game, Enemy_MotherBrain[E].var_A = (0xa9c059i32) as u16);
        }
    };
    [11124825] MotherBrain_Phase2_Revive_0 [] {
        slot!(game, Enemy_MotherBrain[E].var_A = (0xa9c066i32) as u16);
        slot!(game, Enemy_MotherBrain[E].var_F = 768);
    };
    [11124866] MotherBrain_Phase2_Revive_2 [] {
        slot!(game, Enemy_MotherBrain[E].var_A = (0xa9c08fi32) as u16);
        slot!(game, Enemy_MotherBrain[E].var_37 = 0);
        MotherBrain_Phase2_Revive_3(game);
    };
    [11124879] MotherBrain_Phase2_Revive_3 [] {
        slot!(game, Enemy_MotherBrain[E].var_F = slot!(game, Enemy_MotherBrain[E].var_F).wrapping_sub(1));
        let v1: bool = (slot!(game, Enemy_MotherBrain[E].var_F) & 0x8000) != 0;
        if v1 && {
            slot!(game, Enemy_MotherBrain[E].var_F = 16);
            let mut v2: u16 = (slot!(game, Enemy_MotherBrain[E].var_37)).wrapping_add(1);
            slot!(game, Enemy_MotherBrain[E].var_37 = v2);
            mother_brain_visuals::fade(Fade::DrainedFromGray, (v2).wrapping_sub(1), game) & 1 != 0
        } {
            slot!(game, Enemy_MotherBrain[E].var_A = (0xa9c0bai32) as u16);
            slot!(game, Enemy_MotherBrain[0x40i32].var_10 = 1);
            MotherBrain_SetupBrainNormalPal(game);
            MotherBrain_Phase2_Revive_4(game);
        }
    };
    [11124987] MotherBrain_Phase2_Revive_6 [] {
        slot!(game, Enemy_MotherBrain[E].var_F = slot!(game, Enemy_MotherBrain[E].var_F).wrapping_sub(1));
        let v1: bool = (slot!(game, Enemy_MotherBrain[E].var_F) & 0x8000) != 0;
        if v1 && MotherBrain_MakeWalkForwards(4, 0x50, game) & 1 != 0 {
            slot!(game, Enemy_MotherBrain[E].var_A = (0xa9c11ei32) as u16);
            slot!(game, Enemy_MotherBrain[E].var_1F = 2);
            slot!(game, Enemy_MotherBrain[0x40i32].var_11 = 1);
        }
    };
    [11125022] MotherBrain_Phase2_Revive_7 [] {
        slot!(game, Enemy_MotherBrain[E].var_13 = 0);
        let E1: i32 = 0x40;
        slot! { game, Enemy_MotherBrain[E1]; var_31 = 1, var_32 = 2, var_33 = 4, var_34 = 64 };
        slot!(game, Enemy_MotherBrain[E].var_A = (0xa9c147i32) as u16);
        MotherBrain_Phase2_Revive_8(game);
    };
    [11125199] MotherBrain_Phase3_Recover_MakeDistance [] {
        slot!(game, Enemy_MotherBrain[E].var_00 = 4);
        slot!(game, Enemy_MotherBrain[E].var_A = (0xa9c1f0i32) as u16);
        slot!(game, Enemy_MotherBrain[E].var_F = 32);
        MotherBrain_MakeWalkBackwards((slot!(game, Enemy_MotherBrain[E].base.x_pos)).wrapping_sub(14), 2, game);
    };
    MotherBrain_Phase3_Walk_TryToInchForward [] {
        let mbn_var_07: u16 = slot!(game, Enemy_MotherBrain[E].var_07);
        if mbn_var_07 != 0 {
            let v3: u16 = (mbn_var_07).wrapping_add(32);
            slot!(game, Enemy_MotherBrain[E].var_07 = v3);
            if v3 >= 0x100 {
                let v4: u16 = (slot!(game, Enemy_MotherBrain[E].base.x_pos)).wrapping_add(1);
                let E1: i32 = 0x40;
                slot!(game, Enemy_MotherBrain[E1].var_1B = v4);
                if MotherBrain_MakeWalkForwards(
                    ((word!(game.random_number) as i32 & 2) + 4) as u16,
                    slot!(game, Enemy_MotherBrain[E1].var_1B),
                    game,
                ) & 1
                    != 0
                {
                    slot!(game, Enemy_MotherBrain[E].var_07 = 128);
                }
            }
        } else {
            let v6: u16 = (slot!(game, Enemy_MotherBrain[E].base.x_pos)).wrapping_sub(14);
            let E1_0: i32 = 0x40;
            slot!(game, Enemy_MotherBrain[E1_0].var_1B = v6);
            slot!(game, Enemy_MotherBrain[E1_0].var_1A = (0xa9c2b3i32) as u16);
            MotherBrain_Phase3_Walk_RetreatQuickly(game);
        };
    };
    MotherBrain_HandleWalking [] {
        if slot!(game, Enemy_MotherBrain[E].var_02) == 0 {
            let mbn_var_07: u16 = slot!(game, Enemy_MotherBrain[E].var_07);
            let can_pause_walk: bool = if mbn_var_07 != 0 {
                let mut v2: u16 = (mbn_var_07).wrapping_add(6);
                slot!(game, Enemy_MotherBrain[E].var_07 = v2);
                !(v2 >= 0x100)
                    && ({
                        if slot!(game, Enemy_MotherBrain[E].base.x_pos) as i32 - 48i32 & 0x8000 == 0 {
                            return;
                        }
                        true
                    })
            } else {
                slot!(game, Enemy_MotherBrain[E].var_07 = 1);
                if slot!(game, Enemy_MotherBrain[E].base.x_pos) as i32 - 48i32 & 0x8000 == 0 {
                    MotherBrain_SetBodyInstrs(addr_kMotherBrain_Ilist_98C6 as u16, game);
                    return;
                }
                true
            };
            if can_pause_walk && (word!(game.random_number) as i32 & 0xfff) - 4032i32 & 0x8000 != 0 {
                return;
            }
            slot!(game, Enemy_MotherBrain[E].var_07 = 128);
            if slot!(game, Enemy_MotherBrain[E].base.x_pos) as i32 - 128i32 & 0x8000 != 0 {
                MotherBrain_SetBodyInstrs(addr_kMotherBrain_Ilist_97A4 as u16, game);
            }
        }
    };
    MotherBrain_Pal_BeginScreenFlash [] {
        slot!(game, Enemy_MotherBrain[E].var_0E = addr_stru_A9D046 as u16);
        slot!(game, Enemy_MotherBrain[E].var_0F = 1);
    };
    MotherBrain_Pal_EndScreenFlash [] {
        slot! { game, Enemy_MotherBrain[E]; var_0E = 0, var_0F = 0 }
        MotherBrain_Pal_WriteRoomPal(addr_word_A9D082 as u16, game);
    };
    MotherBrain_HandleBrainPal [] {
        if slot!(game, Enemy_MotherBrain[E].var_00) != 1 {
            let mut mbn_var_D_high: i8 = slot!(byte game, Enemy_MotherBrain[E].var_D[1]) as i8;
            if mbn_var_D_high != 0 {
                slot!(byte game, Enemy_MotherBrain[E].var_D[1] = ((mbn_var_D_high) as u8).wrapping_sub(1));
            } else {
                let r18: u16 = crate::rtl::rom_table_word!(
                    game,
                    0xa9d260,
                    (slot!(byte game, Enemy_MotherBrain[E].var_E[1]) >> 1) as isize,
                );
                slot!(byte game, Enemy_MotherBrain[E].var_D[1] = slot!(game, Enemy_MotherBrain[E].var_E) as u8);
                let mut mbn_var_D: i8 = slot!(game, Enemy_MotherBrain[E].var_D) as i8;
                if mbn_var_D != 0 || slot!(game, Enemy_MotherBrain[0x40i32].var_A) as i32 == 0xa987a2 & 0xffff
                {
                    let v4: u8 = (mbn_var_D as i32 + 1 & 7) as u8;
                    slot!(byte game, Enemy_MotherBrain[E].var_D[0] = v4);
                    let mut v5: u16 = 290;
                    if slot!(game, Enemy_MotherBrain[E].var_0D) != 512 {
                        v5 = 482;
                    }
                    WriteColorsToPalette(v5, 0xa9, (r18 as i32 + 8 * v4 as i32) as u16, 3, game);
                }
            }
        }
    };
    @forward [pub unsafe fn (game: &mut Game)];
    [11110352]
    MotherBrainsBrain_SetupBrainToDraw => { let value: u32 = 0xa987dd; { let value: u16 = value as u16; let at: usize = 0x178c; let bytes = value.to_ne_bytes(); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); }; game.ram().write_byte(0x178c + 2, (value >> 16) as u8); };
    MotherBrainsBrain_GfxDrawHook => MotherBrain_DrawBrain(game);
    @countdown [pub unsafe fn (game: &mut Game)];
    before {
        let E: i32 = 0x40;
        slot!(game, Enemy_MotherBrainBody[E].var_F = slot!(game, Enemy_MotherBrainBody[E].var_F).wrapping_sub(1));
    }
    when (slot!(game, Enemy_MotherBrain[E].var_F) & 0x8000) != 0;
    [11110787] MotherBrainBody_2_SpawnTopRightTubeFalling {
        game.eproj_spawn_pt = Point16U { x: 152, y: 47 };
        SpawnEprojWithRoomGfx(0xcc5b, 0x2f, game);
        slot!(game, Enemy_MotherBrain[E].var_E = -30304i32 as u16);
    };
    [11110837] MotherBrainBody_4_SpawnTopLeftTubeFalling {
        game.eproj_spawn_pt = Point16U { x: 104, y: 47 };
        SpawnEprojWithRoomGfx(
            addr_kEproj_MotherBrainTubeFalling_TopLeft as u16,
            0x2f,
            game,
        );
        slot!(game, Enemy_MotherBrain[E].var_E = (0xa989d2i32) as u16);
    };
    [11110887] MotherBrainBody_5_SpawnTubeFallingEnemy1 {
        ({ let (bank, population) = (0xa9, addr_stru_A98AF5 as u16); actors::population::spawn_legacy(&mut Runtime::new(game), bank, population) });
        slot!(game, Enemy_MotherBrain[E].var_E = (0xa989fai32) as u16);
    };
    [11110927] MotherBrainBody_7_SpawnTubeFallingEnemy2 {
        ({ let (bank, population) = (0xa9, addr_stru_A98B05 as u16); actors::population::spawn_legacy(&mut Runtime::new(game), bank, population) });
        slot!(game, Enemy_MotherBrain[E].var_E = (0xa98a22i32) as u16);
    };
    [11110967] MotherBrainBody_9_SpawnTopMiddleLeftFalling {
        game.eproj_spawn_pt = Point16U { x: 120, y: 59 };
        SpawnEprojWithRoomGfx(
            addr_kEproj_MotherBrainTubeFalling_TopMiddleLeft as u16,
            0x3b,
            game,
        );
        slot!(game, Enemy_MotherBrain[E].var_E = (0xa98a54i32) as u16);
    };
    [11111017] MotherBrainBody_11_SpawnTopMiddleRightFalling {
        game.eproj_spawn_pt = Point16U { x: 136, y: 59 };
        SpawnEprojWithRoomGfx(
            addr_kEproj_MotherBrainTubeFalling_TopMiddleRight as u16,
            0x3b,
            game,
        );
        slot!(game, Enemy_MotherBrain[E].var_E = (0xa98a86i32) as u16);
    };
    [11111067] MotherBrainBody_13_SpawnTubeFallingEnemy3 {
        ({ let (bank, population) = (0xa9, ((0x868b15i32) as u16)); actors::population::spawn_legacy(&mut Runtime::new(game), bank, population) });
        slot!(game, Enemy_MotherBrain[E].var_E = (0xa98aaei32) as u16);
    };
    [11111107] MotherBrainBody_15_SpawnTubeFallingEnemy4 {
        ({ let (bank, population) = (0xa9, addr_stru_A98B25 as u16); actors::population::spawn_legacy(&mut Runtime::new(game), bank, population) });
        slot!(game, Enemy_MotherBrain[E].var_E = (0xa98ad6i32) as u16);
    };
    @countdown [pub unsafe fn (game: &mut Game)];
    before {
        let E: i32 = 0;
        slot!(game, Enemy_MotherBrainBody[E].var_F = slot!(game, Enemy_MotherBrainBody[E].var_F).wrapping_sub(1));
    }
    when (slot!(game, Enemy_MotherBrain[E].var_F) & 0x8000) != 0;
    [11110441] MotherBrainBody_FakeDeath_Descent_1 {
        CallSomeSamusCode(0, game);
        { let value: u16 = game.ram().read_byte(0xcd20 + 0) as u16; let at: usize = 0xcd20; let bytes = value.to_ne_bytes(); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); };
        slot!(game, Enemy_MotherBrain[E].var_A = (0xa9884di32) as u16);
        slot!(game, Enemy_MotherBrain[E].var_F = 32);
        MotherBrainBody_FakeDeath_Descent_2(game);
    };
    [11110477] MotherBrainBody_FakeDeath_Descent_2 {
        QueueMusic_Delayed8(0, game);
        QueueMusic_Delayed8(0xff21, game);
        slot!(game, Enemy_MotherBrain[E].var_A = (0xa9886ci32) as u16);
        slot!(game, Enemy_MotherBrain[E].var_F = 12);
        MotherBrainBody_FakeDeath_Descent_3(game);
    };
    [11110508] MotherBrainBody_FakeDeath_Descent_3 {
        CallSomeSamusCode(1, game);
        slot!(game, Enemy_MotherBrain[E].var_A = (0xa98884i32) as u16);
        slot!(game, Enemy_MotherBrain[E].var_F = 8);
        MotherBrainBody_FakeDeath_Descent_4(game);
    };
    [11111801] MotherBrainBody_FakeDeath_Ascent_8_Pause {
        slot!(game, Enemy_MotherBrain[E].var_A = (0xa98d8bi32) as u16);
        slot!(game, Enemy_MotherBrain[E].var_F = 32);
        MotherBrainBody_FakeDeath_Ascent_9_PrepareRise(game);
    };
    @countdown [pub unsafe fn (game: &mut Game)];
    before {
        MotherBrain_GenerateSmokyExplosions(game);
        let E: i32 = 0;
        slot!(game, Enemy_MotherBrain[E].var_F = slot!(game, Enemy_MotherBrain[E].var_F).wrapping_sub(1));
    }
    when (slot!(game, Enemy_MotherBrain[E].var_F) & 0x8000) != 0;
    [11120402] MotherBrain_Phase3_Death_1 {
        slot!(game, Enemy_MotherBrain[E].var_A = (0xa9af21i32) as u16);
    };
    [11120468] MotherBrain_Phase3_Death_3 {
        let E1: i32 = 0x40;
        slot! { game, Enemy_MotherBrain[E1]; var_32 = 0, var_33 = 0, var_12 = 0, var_14 = 0, var_10 = 0, var_11 = 0 }
        word_transfers!(game, descending, 15; le(palette_buffer, 255) <- le(palette_buffer, 159));
        mother_brain_visuals::update_health_palette(game);
        slot!(game, Enemy_MotherBrain[E].var_0D = 3584);
        slot! { game, Enemy_MotherBrain[E1]; var_E = 0, var_F = 0 }
        slot!(game, Enemy_MotherBrain[E].var_A = (0xa9af9di32) as u16);
        MotherBrain_Phase3_Death_4(game);
    };
    @countdown [pub unsafe fn (game: &mut Game)];
    before {
        MotherBrain_GenerateMixedExplosions(game);
        let E: i32 = 0;
        slot!(game, Enemy_MotherBrain[E].var_F = slot!(game, Enemy_MotherBrain[E].var_F).wrapping_sub(1));
    }
    when (slot!(game, Enemy_MotherBrain[E].var_F) & 0x8000) != 0;
    [11120541] MotherBrain_Phase3_Death_4 {
        slot!(game, Enemy_MotherBrain[E].var_37 = 0);
        slot!(game, Enemy_MotherBrain[E].var_A = (0xa9afb6i32) as u16);
        slot!(game, Enemy_MotherBrain[E].var_F = 0);
        MotherBrain_Phase3_Death_5(game);
    };
    [11120659] MotherBrain_Phase3_Death_6 {
        slot!(game, Enemy_MotherBrain[E].var_A = (0xa9b115i32) as u16);
    };
    @countdown [pub unsafe fn (game: &mut Game)];
    before {
        let E: i32 = 0;
        slot!(game, Enemy_MotherBomb[E].var_F = slot!(game, Enemy_MotherBomb[E].var_F).wrapping_sub(1));
    }
    when (slot!(game, Enemy_MotherBrain[E].var_F) & 0x8000) != 0;
    [11122970] MotherBomb_FiringRainbowBeam_1_StartCharge {
        MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9F6C as u16, game);
        slot!(game, Enemy_MotherBrain[E].var_A = (0xa9b92bi32) as u16);
        MotherBomb_FiringRainbowBeam_2_RetractNeck(game);
    };
    [11123007] MotherBomb_FiringRainbowBeam_3_Wait {
        QueueSfx2_Max6(0x71, game);
        slot!(game, Enemy_MotherBrain[E].var_A = (0xa9b951i32) as u16);
        MotherBomb_FiringRainbowBeam_4_ExtendNeckDown(game);
    };
    [11123462] MotherBomb_FiringRainbowBeam_14_DecideNextAction {
        if word!(game.samus_health) as i32 - 400i32 & 0x8000 != 0 {
            MotherBrain_MakeWalkForwards(0xa, (slot!(game, Enemy_MotherBrain[E].base.x_pos)).wrapping_add(16), game);
            slot!(game, Enemy_MotherBrain[E].var_A = (0xa9bd45i32) as u16);
        } else {
            slot!(game, Enemy_MotherBrain[E].var_A = (0xa9b8ebi32) as u16);
        }
    };
    @countdown [pub unsafe fn (game: &mut Game)];
    before {
        let E: i32 = 0;
        slot!(game, Enemy_MotherBrain[E].var_F = slot!(game, Enemy_MotherBrain[E].var_F).wrapping_sub(1));
    }
    when (slot!(game, Enemy_MotherBrain[E].var_F) & 0x8000) != 0;
    [11110532] MotherBrainBody_FakeDeath_Descent_4 {
        MotherBrain_Pal_BeginScreenFlash(game);
        LoadFxEntry(2, game);
        slot!(game, Enemy_MotherBrain[0x40i32].var_E = (0xa98949i32) as u16);
        slot!(game, Enemy_MotherBrain[E].var_A = (0xa988b2i32) as u16);
        slot! { game, Enemy_MotherBrain[E]; var_F = 0, var_37 = 0 }
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0xe, 0x2, 0xb6b3 as VoidP), game);
    };
    [11111819] MotherBrainBody_FakeDeath_Ascent_9_PrepareRise {
        slot!(game, Enemy_MotherBrain[E].var_09 = spawn_rising_hdma(game));
        let E1: i32 = 0x40;
        slot!(game, Enemy_MotherBrain[E1].base.properties = (slot!(game, Enemy_MotherBrain[E1].base.properties)) | (0x100u16));
        MotherBrain_SetBrainInstrs(addr_stru_A99C21 as u16, game);
        slot!(game, Enemy_MotherBrain[E].var_A = (0xa98db4i32) as u16);
        slot!(game, Enemy_MotherBrain[E].var_F = 256);
        MotherBrainBody_FakeDeath_Ascent_10_LoadLegTiles(game);
    };
    [11111875] MotherBrainBody_FakeDeath_Ascent_11_ContinuePause {
        slot!(game, Enemy_MotherBrain[E].base.x_pos = 59);
        slot!(game, Enemy_MotherBrain[E].base.y_pos = 279);
        word! {game; reg_BG2HOFS = -27i32 as u16; reg_BG2VOFS = -217i32 as u16; }
        slot!(game, Enemy_MotherBrain[E].var_04 = 7);
        slot!(game, Enemy_MotherBrain[E].var_A = (0xa98deci32) as u16);
    };
    [11112106] MotherBrainBody_FakeDeath_Ascent_15_TransitionFromGrey {
        slot!(game, Enemy_MotherBrain[E].var_F = 4);
        let v3: u16 = (slot!(game, Enemy_MotherBrain[E].var_37)).wrapping_add(1);
        slot!(game, Enemy_MotherBrain[E].var_37 = v3);
        if mother_brain_visuals::fade(Fade::FakeDeathFromGray, (v3).wrapping_sub(1), game) & 1 != 0 {
            let E1: i32 = 0x40;
            slot!(game, Enemy_MotherBrain[E1].var_10 = 1);
            slot!(game, Enemy_MotherBrain[E].var_00 = 2);
            slot!(game, Enemy_MotherBrain[E1].var_12 = 1);
            slot! { game, Enemy_MotherBrain[E1]; var_32 = 6, var_33 = 6 }
            slot!(game, Enemy_MotherBrain[E1].var_34 = 1280);
            slot!(game, Enemy_MotherBrain[E].var_A = (0xa98ef5i32) as u16);
            slot!(game, Enemy_MotherBrain[E].var_F = 23);
        }
    };
    [11112181] MotherBrainBody_2ndphase_16_ShakeHeadMenacingly {
        MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9B7F as u16, game);
        slot!(game, Enemy_MotherBrain[E].var_A = (0xa98f14i32) as u16);
        slot!(game, Enemy_MotherBrain[0x40i32].var_34 = 64);
        slot!(game, Enemy_MotherBrain[E].var_F = 256);
        MotherBrainBody_2ndphase_17_BringHeadBackUp(game);
    };
    [11112212] MotherBrainBody_2ndphase_17_BringHeadBackUp {
        let E1: i32 = 0x40;
        slot!(game, Enemy_MotherBrain[E1].var_32 = 2);
        slot!(game, Enemy_MotherBrain[E1].var_33 = 4);
        slot!(game, Enemy_MotherBrain[E].var_A = (0xa98f33i32) as u16);
        slot!(game, Enemy_MotherBrain[E].var_F = 64);
        MotherBrainBody_2ndphase_18_FinishStretching(game);
    };
    [11112243] MotherBrainBody_2ndphase_18_FinishStretching {
        slot!(game, Enemy_MotherBrain[0x40i32].var_14 = 1);
        slot!(game, Enemy_MotherBrain[E].var_A = (0xa9b605i32) as u16);
    };
    [11121011] MotherBrain_Phase3_Death_10 {
        slot!(game, Enemy_MotherBrain[E].var_37 = 0);
        slot!(game, Enemy_MotherBrain[E].var_A = (0xa9b189i32) as u16);
        slot!(game, Enemy_MotherBrain[E].var_F = 0);
    };
    [11121033] MotherBrain_Phase3_Death_11 {
        let v2: u16 = (slot!(game, Enemy_MotherBrain[E].var_37)).wrapping_add(1);
        slot!(game, Enemy_MotherBrain[E].var_37 = v2);
        if mother_brain_visuals::fade(Fade::RealDeathToGray, (v2).wrapping_sub(1), game) & 1 != 0 {
            MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9D25 as u16, game);
            slot!(game, Enemy_MotherBrain[E].var_A = (0xa9b1b8i32) as u16);
            slot!(game, Enemy_MotherBrain[E].var_F = 256);
        } else {
            slot!(game, Enemy_MotherBrain[E].var_F = 16);
        }
    };
    [11121080] MotherBrain_Phase3_Death_12 {
        slot!(game, Enemy_MotherBrain[E].var_A = (0xa9b1d5i32) as u16);
        let E1: i32 = 0x40;
        slot!(game, Enemy_MotherBrain[E1].base.properties = (slot!(game, Enemy_MotherBrain[E1].base.properties)) | (0x400u16));
        slot!(game, Enemy_MotherBrain[E].var_04 = 0);
    };
    [11121169] MotherBrain_Phase3_Death_14_20framedelay {
        let E1: i32 = 0x40;
        slot! { game, Enemy_MotherBrain[E1].base; x_pos = 0, y_pos = 0 }
        slot!(game, Enemy_MotherBrain[E].var_A = (0xa9b258i32) as u16);
    };
    [11122664] MotherBrain_FiringBomb_Fired {
        if MotherBrain_MakeHerStandUp(game) & 1 != 0 {
            MotherBrain_FiringBomb_Finish(game);
        } else {
            slot!(game, Enemy_MotherBrain[E].var_A = (0xa9b7f8i32) as u16);
            MotherBrain_FiringBomb_Standup(game);
        }
    };
    [11122745] MotherBomb_FiringLaser_PositionHeadSlowlyFire {
        let mut v2: i16 = 256;
        let E1: i32 = 0x40;
        if (slot!(game, Enemy_MotherBrain[E1].var_34) & 0x8000) != 0 {
            v2 = -256i16;
        }
        slot!(game, Enemy_MotherBrain[E1].var_34 = v2 as u16);
        MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9F34 as u16, game);
        slot!(game, Enemy_MotherBrain[E].var_A = (0xa9b863i32) as u16);
        slot!(game, Enemy_MotherBrain[E].var_F = 16);
    };
    [11122787] MotherBomb_FiringLaser_FinishAttack {
        let E1: i32 = 0x40;
        slot! { game, Enemy_MotherBrain[E1]; var_32 = 4, var_33 = 4 }
        slot!(game, Enemy_MotherBrain[E].var_A = (0xa9b605i32) as u16);
        MotherBrain_Body_Phase2_Thinking(game);
    };
    [11124137] MotherBrain_Phase2Cut_2 {
        MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9B7F as u16, game);
        slot!(game, Enemy_MotherBrain[E].var_A = (0xa9bdc1i32) as u16);
        slot!(game, Enemy_MotherBrain[E].var_F = 256);
    };
    [11124161] MotherBrain_Phase2Cut_3 {
        MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9F6C as u16, game);
        slot!(game, Enemy_MotherBrain[E].var_A = (0xa9bdd2i32) as u16);
        MotherBrain_Phase2Cut_4(game);
    };
    [11124205] MotherBrain_Phase2Cut_5 {
        MotherBrain_BodyRainbowBeamPalAnimIndex0(game);
        MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9C77 as u16, game);
        let E1: i32 = 0x40;
        slot! { game, Enemy_MotherBrain[E1]; var_32 = 6, var_33 = 6 }
        slot!(game, Enemy_MotherBrain[E1].var_34 = 1280);
        QueueSfx2_Max6(0x71, game);
        slot!(game, Enemy_MotherBrain[E].var_A = (0xa9be1ai32) as u16);
    };
    [11124605] MotherBrain_DrainedByShitroid_6 {
        slot!(game, Enemy_MotherBrain[E].var_37 = 0);
        slot!(game, Enemy_MotherBrain[E].var_A = (0xa9bf95i32) as u16);
        slot!(game, Enemy_MotherBrain[E].var_F = 16);
        MotherBrain_DrainedByShitroid_7(game);
    };
    [11124838] MotherBrain_Phase2_Revive_1 {
        let E1: i32 = 0x40;
        slot! { game, Enemy_MotherBrain[E1]; var_14 = 1, var_12 = 1 }
        slot!(game, Enemy_MotherBrain[E].var_A = (0xa9c082i32) as u16);
        slot!(game, Enemy_MotherBrain[E].var_F = 224);
        MotherBrain_Phase2_Revive_2(game);
    };
    [11124964] MotherBrain_Phase2_Revive_5 {
        MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9BB3 as u16, game);
        slot!(game, Enemy_MotherBrain[E].var_A = (0xa9c0fbi32) as u16);
        slot!(game, Enemy_MotherBrain[E].var_F = 128);
        MotherBrain_Phase2_Revive_6(game);
    };
    [11125122] MotherBrain_Phase2_MurderShitroid_2 {
        slot!(game, Enemy_MotherBrain[E].var_A = (0xa9c15ci32) as u16);
    };
    [11125232] MotherBrain_Phase3_Recover_SetupForFight {
        slot!(game, Enemy_MotherBrain[E].var_A = (0xa9c209i32) as u16);
        let E1: i32 = 0x40;
        slot!(game, Enemy_MotherBrain[E1].var_18 = (0xa9c330i32) as u16);
        slot!(game, Enemy_MotherBrain[E1].var_1A = (0xa9c26ai32) as u16);
        MotherBrain_Phase3_Fighting_Main(game);
    };
    [11125326] MotherBrain_Phase3_Fighting_Cooldown {
        slot!(game, Enemy_MotherBrain[E].var_A = (0xa9c209i32) as u16);
    };
    @external {
        11110369 => [{
                    Unreachable(game);
                }];
        11124250 => [()];
        11125158 => [()];
        11111140 => [()]; 11125133 => [()];
    }
}
pub unsafe fn MotherBrainsBody_UnpauseHook(game: &mut Game) -> CoroutineRet {
    let E: i32 = 0;
    if slot!(game, Enemy_MotherBrain[E].var_16) != 0 {
        QueueSfx1_Max6(0x40, game);
    }
    if (slot!(game, Enemy_MotherBrain[E].base.extra_properties) & 4) != 0 {
        word! {game; enemy_bg2_tilemap_size = 2048; nmi_flag_bg2_enemy_vram_transfer = 1; }
    }
    kCoroutineNone as CoroutineRet
}
pub unsafe fn MotherBrain_DrawBrainNeck_EnemyGfxDrawHook(game: &mut Game) {
    unsafe {
        MotherBrain_DrawBrain(game);
        MotherBrain_DrawNeck(game);
    }
}
pub unsafe fn MotherBrainsBody_FirstPhase_DoubleRet(game: &mut Game) {
    unsafe {
        if CheckEventHappened(2, game) != 0 {
            let E1: i32 = 0x40;
            slot!(
                game,
                Enemy_MotherBrain[E1].var_00 = word!(game.earthquake_timer)
            );
            if word!(game.samus_x_pos) as i32 - 236i32 & 0x8000 != 0
                && slot!(game, Enemy_MotherBrain[E1].base.health) == 0
            {
                let E: i32 = 0;
                slot! { game, Enemy_MotherBrain[E]; var_1D = 1, var_00 = 1 }
                minimap::disable_for_boss(&mut crate::game::session_runtime::Runtime::new(game));
                QueueMusic_Delayed8(6, game);
                mother_brain_visuals::seal_wall(game);
            }
        }
        MotherBrain_SamusCollDetect(game);
    }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallMotherBrainsTubesFalling(ea: u32, k: u16, game: &mut Game) => ea;
    call (k, game); default { Unreachable(game); };
    @functions [pub unsafe fn (k: u16, game: &mut Game)];
    @prefix { let E: i32 = k as i32; };
    MotherBrainsTubesFalling_Main [] {
        CallMotherBrainsTubesFalling(((slot!(game, Enemy_MotherBrainsTubesFalling[E].var_A)) as u32) | (0xa90000u32), k, game);
    };
    [11111304] MotherBrainsTubesFalling_Main_NonMain [] {
        let v2: u16 = (slot!(game, Enemy_MotherBrainsTubesFalling[E].var_C)).wrapping_add(6);
        slot!(game, Enemy_MotherBrainsTubesFalling[E].var_C = v2);
        Enemy_IncreaseYpos(k, v2, game);
        if ((slot!(game, Enemy_MotherBrainsTubesFalling[E].base.y_pos)) as i16).wrapping_sub((slot!(game, Enemy_MotherBrainsTubesFalling[E].var_B)) as i16) < 0 {
            MotherBrainsTubesFalling_HandleSmoke(k, game);
        } else {
            MotherBrainsTubesFalling_Explode(k, game);
        };
    };
    MotherBrainsTubesFalling_HandleSmoke [] {
        slot!(game, Enemy_MotherBrainsTubesFalling[E].var_D = slot!(game, Enemy_MotherBrainsTubesFalling[E].var_D).wrapping_sub(1));
        if (slot!(game, Enemy_MotherBrainsTubesFalling[E].var_D) & 0x8000) != 0 {
            MotherBrainsTubesFalling_SpawnSmoke(k, game);
        }
    };
    MotherBrainsTubesFalling_Explode [] {
        slot!(game, Enemy_MotherBrainsTubesFalling[E].base.properties = (slot!(game, Enemy_MotherBrainsTubesFalling[E].base.properties)) | ((kEnemyProps_Deleted) as u16));
        game.eproj_spawn_pt = Point16U {
            x: slot!(game, Enemy_MotherBrainsTubesFalling[E].base.x_pos),
            y: slot!(game, Enemy_MotherBrainsTubesFalling[E].base.y_pos),
        };
        SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 3, game);
        QueueSfx2_Max3(0x24, game);
    };
    [11111371] MotherBrainsTubesFalling_WaitToFall [] {
        slot!(game, Enemy_MotherBrainsTubesFalling[E].parameter_2 = slot!(game, Enemy_MotherBrainsTubesFalling[E].parameter_2).wrapping_sub(1));
        if (slot!(game, Enemy_MotherBrainsTubesFalling[E].parameter_2) & 0x8000) != 0 {
            slot!(game, Enemy_MotherBrainsTubesFalling[E].var_A = (0xa98bd6i32) as u16);
            MotherBrainsTubesFalling_Falling(k, game);
        }
    };
    [11111382] MotherBrainsTubesFalling_Falling [] {
        let v2: u16 = (slot!(game, Enemy_MotherBrainsTubesFalling[E].var_C)).wrapping_add(6);
        slot!(game, Enemy_MotherBrainsTubesFalling[E].var_C = v2);
        Enemy_IncreaseYpos(k, v2, game);
        let ypos: u16 = slot!(game, Enemy_MotherBrainsTubesFalling[E].base.y_pos);
        if ypos as i32 - 244i32 & 0x8000 == 0 {
            slot!(game, Enemy_MotherBrainsTubesFalling[E].base.properties = (slot!(game, Enemy_MotherBrainsTubesFalling[E].base.properties)) | ((kEnemyProps_Invisible) as u16));
        }
        let E1: i32 = 0x40;
        slot!(game, Enemy_MotherBrainsTubesFalling[E1].base.y_pos = (ypos).wrapping_sub(56));
        if slot!(game, Enemy_MotherBrainsTubesFalling[E1].base.y_pos) as i32 - 0xc4i32 & 0x8000 != 0 {
            MotherBrainsTubesFalling_HandleSmoke(k, game);
        } else {
            MotherBrain_Pal_EndScreenFlash(game);
            mother_brain_visuals::start_earthquake(0x19, game);
            word! { game, 0;
                hdma_object_channels_bitmask[0], hdma_object_channels_bitmask[1],
            }
            slot!(game, Enemy_MotherBrainsTubesFalling[E1].base.y_pos = 196);
            let E0: i32 = 0;
            slot!(game, Enemy_MotherBrainsTubesFalling[E0].base.x_pos = 59);
            slot!(game, Enemy_MotherBrainsTubesFalling[E0].base.y_pos = 279);
            MotherBrain_SetupNeckForFakeAscent(game);
            slot!(game, Enemy_MotherBrainsTubesFalling[E0].var_A = (0xa98c87i32) as u16);
            MotherBrainsTubesFalling_Explode(word!(game.cur_enemy_index), game);
        };
    };
    MotherBrainsTubesFalling_SpawnSmoke [] {
        slot!(game, Enemy_MotherBrainsTubesFalling[E].var_D = 8);
        let v2: u16 = (slot!(game, Enemy_MotherBrainsTubesFalling[E].var_E) as i32 + 1 & 3) as u16;
        slot!(game, Enemy_MotherBrainsTubesFalling[E].var_E = v2);
        game.eproj_spawn_pt = Point16U {
            x: ((slot!(game, Enemy_MotherBrainsTubesFalling[E].base.x_pos)).wrapping_add(g_word_A98C61[v2 as usize])),
            y: 208,
        };
        SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 9, game);
    };
}
pub unsafe fn MotherBrain_HandleNeckLower(game: &mut Game) {
    unsafe {
        let mbn_var_32: u16 = slot!(game, Enemy_MotherBrain[0x40i32].var_32);
        if mbn_var_32 != 0 {
            match mbn_var_32 as i32 {
                2 => MotherBrain_HandleNeckLower_2_BobDown(game),
                4 => MotherBrain_HandleNeckLower_4_BobUp(game),
                6 => MotherBrain_HandleNeckLower_6_Lower(game),
                8 => MotherBrain_HandleNeckLower_8_Raise(game),
                _ => {
                    Unreachable(game);
                    loop {}
                }
            }
        } else {
            MotherBrain_HandleNeckLower_0();
        };
    }
}
pub fn MotherBrain_HandleNeckLower_0() {}
crate::game::callback_dispatch! {
    pub unsafe fn CallMotherBrainInstr(ea: u32, k: u16, game: &mut Game) -> u16 => ea;
    call (k, game); default Unreachable(game) as u16;
    @functions [pub unsafe fn (k: u16, game: &mut Game) -> u16];
    [11115279] MotherBrain_Instr_Goto [native_record(0xa90000 | u32::from(k), game).word(0)] { };
    [11115284] MotherBrain_Instr_EnableNeckMovementGoto [native_record(0xa90000 | u32::from(k), game).word(0)] {
        slot!(game, Enemy_MotherBrain[0x40].var_31 = 1);
    };
    [11115296]
    MotherBrain_Instr_DisableNeckMovement [k] { slot!(game, Enemy_MotherBrain[0x40].var_31 = 0); };
    @functions [pub unsafe fn (k: u16, game: &mut Game) -> u16];
    @prefix { let v2 = native_record(0xa90000 | u32::from(k), game); };
    [11115304] MotherBrain_Instr_QueueSfx2 [(k).wrapping_add(2)] {  QueueSfx2_Max6(v2.word(0), game); };
    [11115314] MotherBrain_Instr_QueueSfx3 [(k).wrapping_add(2)] {  QueueSfx3_Max6(v2.word(0), game); };
    @functions [pub unsafe fn (k: u16, game: &mut Game) -> u16];
    [11115324] MotherBrain_Instr_SpawnDroolEproj [k] {
        let E: i32 = 0x40;
        if slot!(game, Enemy_MotherBrain[E].var_12) != 0 {
            let mut v2: u16 = (slot!(game, Enemy_MotherBrain[E].var_13)).wrapping_add(1);
            if slot!(game, Enemy_MotherBrain[E].var_13) as i32 - 5i32 & 0x8000 == 0 {
                v2 = 0;
            }
            slot!(game, Enemy_MotherBrain[E].var_13 = v2);
            let v3: u16 = (if slot!(game, Enemy_MotherBrain[E].var_34) as i32 - 128i32 & 0x8000 != 0 {
                addr_kEproj_MotherBrainDrool as i32
            } else {
                addr_kEproj_MotherBrainDyingDrool as i32
            }) as u16;
            SpawnEprojWithRoomGfx(v3, slot!(game, Enemy_MotherBrain[E].var_13), game);
        }
    };
    [11115373] MotherBrain_Instr_SpawnPurpleBreath [k] {
        SpawnEprojWithRoomGfx(addr_kEproj_MotherBrainPurpleBreathBig as u16, 0x9b6d, game);
    };
    [11115383]
    MotherBrain_Instr_SetMainShakeTimer50 [k] { slot!(game, Enemy_MotherBrain[0x40].var_00 = 50); };
    @functions [pub unsafe fn (_k: u16, game: &mut Game) -> u16];
    [11115621] MotherBrain_Instr_GotoEitherOr [result] {
        let result: u16 = addr_stru_A99C5F as u16;
        if (word!(game.random_number) & 0xfff) >= 0xfe0 {
            return addr_stru_A99C47 as u16;
        }
    };
    @functions [pub unsafe fn (k: u16, game: &mut Game) -> u16];
    [11115693] MotherBrain_Instr_MaybeGoto [k] {
        if word!(game.random_number) < 0xf000 {
            return addr_stru_A99C9F as u16;
        }
    };
    [11115789] MotherBrain_Instr_MaybeGoto2 [k] {
        if (word!(game.random_number) & 0xfff) < 0xec0 {
            return MotherBrain_Instr_Goto2(k);
        }
    };
    [11116023] MotherBrain_Instr_QueueShitroidAttackSfx [k] {
        if slot!(game, Enemy_MotherBrain[0].var_13) != 11 {
            QueueSfx2_Max6(g_word_A99E0F[0i32 as usize], game);
        }
    };
    @functions [pub unsafe fn (k: u16, game: &mut Game) -> u16];
    [11116073] MotherBrain_Instr_SpawnBlueRingEproj [k] {
        let E: i32 = 0;
        SpawnEprojWithRoomGfx(
            addr_kEproj_MotherBrainBlueRingLasers as u16,
            slot!(game, Enemy_MotherBrain[E].var_1A),
            game,
        );
    };
    [11116087] MotherBrain_Instr_AimRingsAtShitroid [k] {
        let E: i32 = 0x40;
        let E1: i32 = slot!(game, Enemy_MotherBrain[E].var_0A) as i32;
        MotherBrain_Instr_AimRings(
            (slot!(game, Enemy_MotherBrain[E1].base.x_pos) as i32 - slot!(game, Enemy_MotherBrain[E].base.x_pos) as i32 - 10i32) as u16,
            (slot!(game, Enemy_MotherBrain[E1].base.y_pos) as i32 - slot!(game, Enemy_MotherBrain[E].base.y_pos) as i32 - 16i32) as u16,
            game,
        );
    };
    [11116123] MotherBrain_Instr_AimRingsAtSamus [k] {
        let E: i32 = 0x40;
        MotherBrain_Instr_AimRings(
            (word!(game.samus_x_pos) as i32 - slot!(game, Enemy_MotherBrain[E].base.x_pos) as i32 - 10i32) as u16,
            (word!(game.samus_y_pos) as i32 - slot!(game, Enemy_MotherBrain[E].base.y_pos) as i32 - 16i32) as u16,
            game,
        );
    };
    @functions [pub unsafe fn (k: u16, game: &mut Game) -> u16];
    [11116195] MotherBrain_Instr_IncrShitroidAttackCtr [k] {
        let E: i32 = 0;
        let mut v3: i16 = ((slot!(game, Enemy_MotherBrain[E].var_13)) as i16).wrapping_add(1);
        if (v3 as u16) >= 0xc {
            v3 = 12;
        }
        slot!(game, Enemy_MotherBrain[E].var_13 = v3 as u16);
    };
    [11116213]
    MotherBrain_Instr_SetShitroidAttackCtr0 [k] { slot!(game, Enemy_MotherBrain[0].var_13 = 0); };
    [11116221] MotherBrain_Instr_SpawnBombEproj [(k).wrapping_add(2)] {
        let v2 = native_record(0xa90000 | u32::from(k), game);
        SpawnEprojWithRoomGfx(addr_kEproj_MotherBrainBomb as u16, v2.word(0), game);
    };
    [11116358] MotherBrain_Instr_SpawnLaserEproj [k] {
        let E: i32 = 0x40;
        slot!(game, Enemy_MotherBrain[E].var_31 = 0);
        game.eproj_spawn_r22 = 1;
        game.eproj_spawn_pt = Point16U {
            x: ((slot!(game, Enemy_MotherBrain[E].base.x_pos)).wrapping_add(16)),
            y: ((slot!(game, Enemy_MotherBrain[E].base.y_pos)).wrapping_add(4)),
        };
        SpawnEprojWithRoomGfx(addr_stru_86A17B as u16, 1, game);
    };
    [11116420] MotherBrain_Instr_SpawnRainbowEproj [k] {
        SpawnEprojWithRoomGfx(
            addr_kEproj_MotherBrainRainbowBeamCharging as u16,
            0x9f84,
            game,
        );
    };
    [11116430] MotherBrain_Instr_SetupFxForRainbowBeam [k] {
        slot!(game, Enemy_MotherBrain[0x40].var_14 = 0);
        MotherBrain_SetupBrainPalForLaser(game);
        QueueSfx2_Max6(0x7f, game);
    };
    @external {
        11115809 => [MotherBrain_Instr_Goto2(k)];
    }
}
pub unsafe fn MotherBrain_Func_1_DoubleRet(game: &mut Game) -> i32 {
    unsafe {
        let E: i32 = 0;
        let mut pc: u16 = slot!(game, Enemy_MotherBrain[E].var_21);
        if pc & 0x8000 == 0 {
            return -1i32;
        }
        let mut p = native_record(0xa90000 | u32::from(pc), game);
        if word!(game.time_is_frozen_flag) != 0 {
            return p.word(2) as i32;
        }
        let mut v5: u16 = p.word(0);
        let mut execute_instruction: bool = (v5 & 0x8000 != 0)
            || ({
                if ((v5) as i16).wrapping_sub((slot!(game, Enemy_MotherBrain[E].var_20)) as i16)
                    >= 0
                {
                    slot!(
                        game,
                        Enemy_MotherBrain[E].var_20 =
                            slot!(game, Enemy_MotherBrain[E].var_20).wrapping_add(1)
                    );
                    return p.word(2) as i32;
                }
                pc = (pc).wrapping_add(4);
                false
            });
        loop {
            if execute_instruction {
                pc =
                    CallMotherBrainInstr(((v5) as u32) | (0xa90000u32), (pc).wrapping_add(2), game);
                execute_instruction = false;
            } else {
                p = native_record(0xa90000 | u32::from(pc), game);
                v5 = p.word(0);
                if v5 & 0x8000 == 0 {
                    break;
                } else {
                    execute_instruction = true;
                }
            }
        }
        slot!(game, Enemy_MotherBrain[E].var_20 = 1);
        slot!(game, Enemy_MotherBrain[E].var_21 = pc);
        p.word(2) as i32
    }
}
pub unsafe fn MotherBrain_DrawNeck(game: &mut Game) {
    unsafe {
        if (slot!(game, Enemy_MotherBrain[0i32].base.properties) as i32
            & kEnemyProps_Invisible as i32)
            == 0
        {
            let E: i32 = 0x40;
            MotherBrain_DrawNeckSegment(
                slot!(game, Enemy_MotherBrain[E].var_2E),
                slot!(game, Enemy_MotherBrain[E].var_2F),
                game,
            );
            MotherBrain_DrawNeckSegment(
                slot!(game, Enemy_MotherBrain[E].var_2B),
                slot!(game, Enemy_MotherBrain[E].var_2C),
                game,
            );
            MotherBrain_DrawNeckSegment(
                slot!(game, Enemy_MotherBrain[E].var_28),
                slot!(game, Enemy_MotherBrain[E].var_29),
                game,
            );
            MotherBrain_DrawNeckSegment(
                slot!(game, Enemy_MotherBrain[E].var_25),
                slot!(game, Enemy_MotherBrain[E].var_26),
                game,
            );
            MotherBrain_DrawNeckSegment(
                slot!(game, Enemy_MotherBrain[E].var_22),
                slot!(game, Enemy_MotherBrain[E].var_23),
                game,
            );
        }
    }
}
pub unsafe fn MotherBrain_DrawNeckSegment(x: u16, y: u16, game: &mut Game) {
    unsafe {
        let v0: i32 = (slot!(game, Enemy_MotherBrain[0x40i32].base.flash_timer) as i32 & 6) >> 1;
        MotherBrain_AddSpritemapToOam(
            addr_kMotherBrain_Sprmap_A694 as u16,
            (x).wrapping_add((g_word_A993BB[v0 as usize]) as u16),
            (y).wrapping_add((g_word_A993C3[v0 as usize]) as u16),
            slot!(game, Enemy_MotherBrain[0i32].var_0C),
            game,
        );
    }
}
pub unsafe fn MotherBrain_AddSpritemapToOam(j: u16, r18: u16, r20: u16, r22: u16, game: &mut Game) {
    let source = native_record(0xa90000 | u32::from(j), game);
    let mut n: i32 = source.word(0) as i32;
    let mut offset: usize = 2;
    let mut idx: i32 = word!(game.oam_next_ptr) as i32;
    do_until! { n == 0 ;
        let y: i16 =
            (r20 as i32 + source.byte(offset + 2) as i8 as i32 - word!(game.layer1_y_pos) as i32) as i16;
        if y >= 0 {
            let oam: usize = usize::try_from(0x370i32 + idx).expect("OAM entry outside RAM");
            let x: u16 =
                (r18 as i32 + source.word(offset) as i32 - word!(game.layer1_x_pos) as i32) as u16;
            game.ram().write_byte(oam, x as u8);
            game.ram().write_byte(oam + 1, y as u8);
            set_native_word(oam + 2, r22 | source.word(offset + 3), game);
            word!(bits_i32, game.oam_ext[(idx >> 5)] |= ((x as i32 & 0x100) >> 8 | ((source.word(offset) as i16) < 0) as i32 * 2)
                    << 2 * (idx >> 2 & 7));
            idx = idx + 4 & 0x1ff;
        }
        offset += 5;
        n -= 1;
    }
    word!(game.oam_next_ptr = idx as u16);
}
pub unsafe fn MotherBrain_CalculateRainbowBeamHdma(game: &mut Game) {
    unsafe {
        calculate_rainbow_window(game);
    }
}
pub fn MotherBrain_Instr_Goto2(_k: u16) -> u16 {
    addr_stru_A99CD1 as u16
}
pub unsafe fn MotherBrain_Instr_AimRings(x: u16, y: u16, game: &mut Game) {
    unsafe {
        let mut v0: u16 = -(actor_api::angle_from_delta(x, y) as i32 + 0x80i32) as u8 as u16;
        let clamp_ring_angle: bool = if v0 as i32 - 16i32 & 0x80 == 0 {
            !((v0 as u8) < 0x48)
        } else {
            (v0 as i32 + 64 & 0x80 != 0)
                || ({
                    byte!(v0; 0) = 16;
                    false
                })
        };
        if clamp_ring_angle {
            byte!(v0; 0) = 72;
        }
        slot!(game, Enemy_MotherBrain[0i32].var_1A = v0);
    }
}
crate::forward_calls! { pub unsafe fn (game: &mut Game);
    MotherBrain_GenerateSmokyExplosions =>
        MotherBrain_GenerateExplosions(0x10, addr_word_A9B10F as u16, 2, game);
    MotherBrain_GenerateMixedExplosions =>
        MotherBrain_GenerateExplosions(8, addr_word_A9B109 as u16, 4, game);
}
pub unsafe fn MotherBrain_ExplodeEscapeDoor(game: &mut Game) {
    {
        for v0 in 0u16..8 {
            SpawnEprojWithRoomGfx(addr_kEproj_MotherBrainExplodedEscapeDoor as u16, v0, game);
        }
    }
}
pub unsafe fn MotherBrain_SamusCollDetect(game: &mut Game) {
    unsafe {
        let E0: i32 = 0;
        let mut R26: u16 = slot!(game, Enemy_MotherBrain[E0].var_04);
        let mut v1: i8 = ((R26) as i8) & (1i8);
        R26 = R26 >> 1;
        if v1 == 0
            || MotherBrain_SamusCollDetectPart(
                addr_word_A9B427 as u16,
                slot!(game, Enemy_MotherBrain[E0].base.x_pos),
                slot!(game, Enemy_MotherBrain[E0].base.y_pos),
                game,
            ) & 1
                == 0
        {
            let E: i32 = 0x40;
            v1 = ((R26) as i8) & (1i8);
            R26 = R26 >> 1;
            if v1 == 0
                || MotherBrain_SamusCollDetectPart(
                    0xb439,
                    slot!(game, Enemy_MotherBrain[E].base.x_pos),
                    slot!(game, Enemy_MotherBrain[E].base.y_pos),
                    game,
                ) & 1
                    == 0
            {
                v1 = ((R26) as i8) & (1i8);
                R26 = R26 >> 1;
                if v1 != 0
                    && MotherBrain_SamusCollDetectPart(
                        0xb44b,
                        slot!(game, Enemy_MotherBrain[E].var_25),
                        slot!(game, Enemy_MotherBrain[E].var_26),
                        game,
                    ) & 1
                        == 0
                    && MotherBrain_SamusCollDetectPart(
                        0xb44b,
                        slot!(game, Enemy_MotherBrain[E].var_28),
                        slot!(game, Enemy_MotherBrain[E].var_29),
                        game,
                    ) & 1
                        == 0
                {
                    MotherBrain_SamusCollDetectPart(
                        0xb44b,
                        slot!(game, Enemy_MotherBrain[E].var_2B),
                        slot!(game, Enemy_MotherBrain[E].var_2C),
                        game,
                    );
                }
            }
        }
    }
}
pub unsafe fn MotherBrain_SamusCollDetectPart(k: u16, r18: u16, r20: u16, game: &mut Game) -> u8 {
    use actors::part_hitboxes::{Contact, samus_contact};
    match samus_contact(k, r18, r20, game) {
        Contact::None => 0,
        Contact::Pushed => 1,
        Contact::Hurt => {
            unsafe { MotherBrain_HurtSamus(game) };
            1
        }
    }
}
pub unsafe fn MotherBrain_Phase23_ShotReaction(game: &mut Game) {
    unsafe {
        let mut v1: i16 = 0;
        let E: i32 = 0;
        if slot!(game, Enemy_MotherBrain[E].var_00) == 4
            && MotherBrain_DetermineShotReactionType(game) == 2
        {
            MotherBrain_Phase3_BeamShotReaction(game);
        } else {
            if MotherBrain_DetermineShotReactionType(game) == 1 || {
                v1 = ((slot!(game, Enemy_MotherBrain[E].var_07)) as i16).wrapping_sub(256);
                v1 < 0
            } {
                v1 = 0;
            }
            slot!(game, Enemy_MotherBrain[E].var_07 = v1 as u16);
        };
    }
}
pub unsafe fn MotherBrain_DetermineShotReactionType(game: &mut Game) -> u16 {
    g_byte_A9B5A1[(game
        .ram()
        .read_byte(0xc18 + 2 * word!(game.collision_detection_index) as usize + 1)
        as i32
        & 7) as usize] as u16
}
pub unsafe fn MotherBrain_HurtSamus(game: &mut Game) {
    unsafe {
        Ridley_Func_98(game);
        word!(game.samus_invincibility_timer = 96);
        word!(game.samus_knockback_timer = 5);
        word!(
            game.knockback_x_dir = (((word!(game.samus_x_pos)) as i16).wrapping_sub(
                (slot!(
                    game,
                    Enemy_MotherBrain[word!(game.cur_enemy_index) as i32]
                        .base
                        .x_pos
                )) as i16,
            ) >= 0) as u16
        );
    }
}
pub unsafe fn MotherBrain_Phase2_DecideAttackStrategy_DoubleRet(game: &mut Game) -> u8 {
    unsafe {
        if g_word_A9B72C[game.ram().read_byte(0xa1f) as usize] != 0 {
            if word!(game.random_number) as u8 as i32 - 128i32 & 0x8000 != 0 {
                0
            } else {
                let E1: i32 = 0x40;
                if slot!(game, Enemy_MotherBrain[E1].var_05) as i32 - 1i32 & 0x8000 != 0 {
                    slot!(game, Enemy_MotherBrain[0i32].var_A = (0xa9b781i32) as u16);
                    MotherBrain_FiringBomb_DecideOnWalking(game);
                    return 1;
                }
                0
            }
        } else {
            if word!(game.random_number) as u8 as i32 - 128i32 & 0x8000 != 0 {
                slot!(game, Enemy_MotherBrain[0i32].var_A = (0xa9b80ei32) as u16);
                MotherBomb_FiringLaser_PositionHead(game);
            } else {
                MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9D3D as u16, game);
            }
            1
        }
    }
}
pub unsafe fn MotherBrain_FiringBomb_DecideOnCrouching(game: &mut Game) {
    unsafe {
        if game.next_random() >= 0x8000 {
            slot!(game, Enemy_MotherBrain[0i32].var_A = (0xa9b7c6i32) as u16);
            MotherBrain_FiringBomb_Crouch(game);
        } else {
            MotherBrain_B7CB(game);
        };
    }
}
pub unsafe fn MotherBrain_B7CB(game: &mut Game) {
    unsafe {
        let mut v0: u16 = addr_kMotherBrain_Ilist_9ECC as u16;
        let E: i32 = 0;
        if slot!(game, Enemy_MotherBrain[E].var_1F) != 0 {
            v0 = addr_kMotherBrain_Ilist_9F00 as u16;
        }
        MotherBrain_SetBrainInstrs(v0, game);
        slot!(game, Enemy_MotherBrain[E].var_A = (0xa9b7e8i32) as u16);
        slot!(game, Enemy_MotherBrain[E].var_F = 44);
    }
}
pub unsafe fn MotherBrain_FiringBomb_Finish(game: &mut Game) {
    let mut v0: u16 = (0xa9b605i32) as u16;
    let E: i32 = 0;
    if slot!(game, Enemy_MotherBrain[E].var_1F) != 0 {
        v0 = (0xa9c15ci32) as u16;
    }
    slot!(game, Enemy_MotherBrain[E].var_A = v0);
}
static off_A9B887: [Func_V; 4] = callback_table! { nonnull Func_V;
    MotherBomb_FiringDeathBeam_0, MotherBomb_FiringDeathBeam_1, MotherBomb_FiringDeathBeam_2,
    MotherBomb_FiringDeathBeam_3,
};
pub unsafe fn MotherBomb_FiringDeathBeam_0(game: &mut Game) {
    unsafe {
        if MotherBrain_MakeWalkBackwards(0x28, 8, game) & 1 != 0 {
            let E1: i32 = 0x40;
            slot!(game, Enemy_MotherBrain[E1].var_32 = 8);
            slot!(game, Enemy_MotherBrain[E1].var_33 = 6);
            let E: i32 = 0;
            slot!(
                game,
                Enemy_MotherBrain[E].var_17 =
                    slot!(game, Enemy_MotherBrain[E].var_17).wrapping_add(1)
            );
        }
    }
}
pub unsafe fn MotherBomb_FiringDeathBeam_1(game: &mut Game) {
    unsafe {
        if slot!(game, Enemy_MotherBrain[0x40i32].var_05) == 0 {
            MotherBrain_SetBodyInstrs(addr_kMotherBrain_Ilist_9A42 as u16, game);
            let E: i32 = 0;
            slot!(
                game,
                Enemy_MotherBrain[E].var_17 =
                    slot!(game, Enemy_MotherBrain[E].var_17).wrapping_add(1)
            );
        }
    }
}
pub fn MotherBomb_FiringDeathBeam_2(_game: &mut Game) {}
pub unsafe fn MotherBomb_FiringDeathBeam_3(game: &mut Game) {
    unsafe {
        MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9C87 as u16, game);
        let E1: i32 = 0x40;
        slot!(game, Enemy_MotherBrain[E1].var_32 = 2);
        slot!(game, Enemy_MotherBrain[E1].var_33 = 4);
        let E: i32 = 0;
        slot!(game, Enemy_MotherBrain[E].var_17 = 0);
        slot!(game, Enemy_MotherBrain[E].var_A = (0xa9b605i32) as u16);
    }
}
pub unsafe fn MotherBrain_WalkBackwardsSlowlyAndRetractHead(a: u16, game: &mut Game) -> u8 {
    unsafe {
        let rv: u8 = MotherBrain_MakeWalkBackwards(a, 0xa, game);
        if rv != 0 {
            MotherBrain_RetractHead(game);
        }
        rv
    }
}
pub unsafe fn MotherBrain_MoveSamusTowardsWallDueToBeam(game: &mut Game) -> u8 {
    unsafe {
        let v0: u8 = MotherBrain_MoveSamusHorizTowardsWall(0x1000, game);
        if v0 == 0 {
            let v1: u16 = Math_MultByCos(0x1000, slot!(game, Enemy_MotherBrain[0i32].var_31));
            MotherBrain_MoveSamusVerticallyTowardsCeilingFloor(v1, game);
        }
        v0
    }
}
pub unsafe fn MotherBrain_MoveSamusTowardsMiddleOfWall(game: &mut Game) {
    unsafe {
        let mut v0: u16 = 64;
        if (124i16).wrapping_sub((word!(game.samus_y_pos)) as i16) < 0 {
            v0 = -64i32 as u16;
        }
        MotherBrain_MoveSamusVerticallyTowardsCeilingFloor(v0, game);
    }
}
pub unsafe fn MotherBrain_MoveSamusForFallingAfterBeam(game: &mut Game) -> u8 {
    unsafe {
        let E: i32 = 0;
        let mut v1: i16 = ((slot!(game, Enemy_MotherBrain[E].parameter_1)) as i16).wrapping_add(2);
        if v1 >= 0 {
            v1 = 0;
        }
        slot!(game, Enemy_MotherBrain[E].parameter_1 = v1 as u16);
        MotherBrain_MoveSamusHorizTowardsWall(v1 as u16, game);
        let v2: u16 = (slot!(game, Enemy_MotherBrain[E].parameter_2)).wrapping_add(24);
        slot!(game, Enemy_MotherBrain[E].parameter_2 = v2);
        (MotherBrain_MoveSamusVerticallyTowardsCeilingFloor(v2, game)) & (1u8)
    }
}
pub unsafe fn MotherBrain_MoveSamusVerticallyTowardsCeilingFloor(
    mut a: u16,
    game: &mut Game,
) -> u8 {
    unsafe {
        let carry: i32 = game.ram().read_byte(0xafc + 1) as i32 + byte!(a; 0) as i32;
        {
            let value: u8 = carry as u8;
            let at: usize = 0xafc + 1;
            game.ram().write_byte(at, value);
        };
        {
            let value: u8 = carry as u8;
            let at: usize = 0xb16 + 1;
            game.ram().write_byte(at, value);
        };
        let v4: u16 =
            (word!(game.samus_y_pos) as i32 + (a >> 8) as i8 as i32 + (carry >> 8)) as u16;
        let mut v6: u16 = if v4 as i32 - 48i32 & 0x8000 != 0 {
            48
        } else {
            if v4 as i32 - 192i32 & 0x8000 != 0 {
                {
                    let at: usize = 0xafa;
                    let bytes: [u8; 2] = (v4).to_le_bytes();
                    game.ram().write_byte(at, bytes[0]);
                    game.ram().write_byte(at + 1, bytes[1]);
                }
                word!(
                    game.samus_prev_y_pos = u16::from_le_bytes([
                        game.ram().read_byte(0xafa),
                        game.ram().read_byte(0xafb)
                    ])
                );
                return 0;
            }
            192
        };
        {
            let at: usize = 0xafa;
            let bytes: [u8; 2] = (v6).to_le_bytes();
            game.ram().write_byte(at, bytes[0]);
            game.ram().write_byte(at + 1, bytes[1]);
        }
        word!(
            game.samus_prev_y_pos =
                u16::from_le_bytes([game.ram().read_byte(0xafa), game.ram().read_byte(0xafb)])
        );
        word! { game, 0; samus_y_subpos, samus_prev_y_subpos }
        1
    }
}
pub unsafe fn MotherBrain_MoveSamusHorizTowardsWall(mut a: u16, game: &mut Game) -> u8 {
    unsafe {
        let carry: i32 = game.ram().read_byte(0xaf8 + 1) as i32 + byte!(a; 0) as i32;
        {
            let value: u8 = carry as u8;
            let at: usize = 0xaf8 + 1;
            game.ram().write_byte(at, value);
        };
        {
            let value: u8 = carry as u8;
            let at: usize = 0xb12 + 1;
            game.ram().write_byte(at, value);
        };
        let v4: u16 =
            (word!(game.samus_x_pos) as i32 + (a >> 8) as i8 as i32 + (carry >> 8)) as u16;
        if v4 as i32 - 235i32 & 0x8000 != 0 {
            word! {game; samus_x_pos = v4; samus_prev_x_pos = v4; }
            0
        } else {
            word! { game, 235; samus_x_pos, samus_prev_x_pos }
            word! { game, 0; samus_x_subpos, samus_prev_x_subpos }
            1
        }
    }
}
pub unsafe fn MotherBrain_BodyRainbowBeamPalAnimIndex0(game: &mut Game) {
    slot!(game, Enemy_MotherBrain[0x40i32].var_01 = 0);
}
pub unsafe fn MotherBrain_WriteDefaultPalette(game: &mut Game) {
    {
        WriteColorsToPalette(
            0x82,
            0xa9,
            (addr_kMotherBrainsBrain_Palette as i32 + 2) as u16,
            0xf,
            game,
        );
        WriteColorsToPalette(
            0x122,
            0xa9,
            (addr_kMotherBrainsBrain_Palette as i32 + 2) as u16,
            0xf,
            game,
        );
        WriteColorsToPalette(
            0x162,
            0xa9,
            (addr_kMotherBrainPalette_0 as i32 + 2) as u16,
            0xf,
            game,
        );
    }
}
pub unsafe fn MotherBrain_WritePhase2DeathPalette(game: &mut Game) {
    {
        MotherBrain_WritePalette(crate::rtl::rom_table_word!(game, 0xade434, 6), game);
    }
}
pub unsafe fn MotherBrain_SpawnShitroidInCutscene(game: &mut Game) {
    slot!(
        game,
        Enemy_MotherBrain[0x40i32].var_0A = {
            let (bank, population) = (0xa9, addr_stru_A9BE28 as u16);
            actors::population::spawn_legacy(&mut Runtime::new(game), bank, population)
        }
    );
}
crate::game::callback_dispatch! {
    pub unsafe fn CallMotherBrainPainfulWalk(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @countdown [pub unsafe fn (game: &mut Game)];
    before {
        let E: i32 = 0x40;
        let v1: u16 = (slot!(game, Enemy_MotherBrain[E].var_09)).wrapping_sub(1);
        slot!(game, Enemy_MotherBrain[E].var_09 = v1);
    }
    when v1 == 0;
    [11124712] MotherBrain_PainfulWalkingForwards {
        slot!(game, Enemy_MotherBrain[E].var_06 = slot!(game, Enemy_MotherBrain[E].var_06).wrapping_add(1));
        slot!(game, Enemy_MotherBrain[E].var_08 = (0xa9c004i32) as u16);
    };
    [11124764] MotherBrain_PainfulWalkingBackwards {
        slot!(game, Enemy_MotherBrain[E].var_06 = slot!(game, Enemy_MotherBrain[E].var_06).wrapping_add(1));
        slot!(game, Enemy_MotherBrain[E].var_08 = (0xa9bfd0i32) as u16);
    };
    @external {
        11124688 => MotherBrain_PainfulWalkForwards;
        11124740 => MotherBrain_PainfulWalkBackwards;
    }
}
pub unsafe fn MotherBrain_MaybeStandupOrLeanDown(game: &mut Game) {
    unsafe {
        let mbn_var_02: u16 = slot!(game, Enemy_MotherBrain[0i32].var_02);
        if mbn_var_02 != 0 {
            if mbn_var_02 == 6 && (word!(game.random_number) as u8) >= 0xc0 {
                MotherBrain_MakeHerStandUp(game);
            }
        } else if (word!(game.random_number) as u8) >= 0xc0 {
            MotherBrain_MakeHerLeanDown(game);
        }
    }
}
crate::game::address_dispatch! {
    pub unsafe fn CallMotherBrainWallkFunc1A(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        11125354 => MotherBrain_Phase3_Walk_TryToInchForward;
        11125427 => MotherBrain_Phase3_Walk_RetreatQuickly;
        11125458 => MotherBrain_Phase3_Walk_RetreatSlowly;
    }
}
pub unsafe fn MotherBrain_Phase3_WalkHandler(game: &mut Game) {
    unsafe {
        if slot!(game, Enemy_MotherBrain[0i32].var_02) == 0 {
            CallMotherBrainWallkFunc1A(
                ((slot!(game, Enemy_MotherBrain[0x40i32].var_1A)) as u32) | (0xa90000u32),
                game,
            );
        }
    }
}
pub unsafe fn MotherBrain_SetToTryToInchForward(a: u16, game: &mut Game) {
    let E: i32 = 0;
    slot!(game, Enemy_MotherBrain[E].var_07 = a);
    let E1: i32 = 0x40;
    slot!(game, Enemy_MotherBrain[E1].var_1A = (0xa9c26ai32) as u16);
    slot!(
        game,
        Enemy_MotherBrain[E1].var_1B =
            (slot!(game, Enemy_MotherBrain[E].base.x_pos)).wrapping_add(1)
    );
}
crate::game::callback_dispatch! {
    pub unsafe fn CallMotherBrainNeckFunc(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    @prefix { let E: i32 = 0x40; };
    MotherBrain_SpawnDustCloudsForAscent [] {
        slot!(game, Enemy_MotherBrain[E].var_F = slot!(game, Enemy_MotherBrain[E].var_F).wrapping_sub(1));
        if (slot!(game, Enemy_MotherBrain[E].var_F) & 0x8000) != 0 {
            slot!(game, Enemy_MotherBrain[E].var_F = 7);
        }
        game.eproj_spawn_pt = Point16U {
            x: g_word_A98F7F[slot!(game, Enemy_MotherBrain[E].var_F) as usize],
            y: 212,
        };
        SpawnEprojWithRoomGfx(
            addr_kEproj_DustCloudExplosion as u16,
            g_byte_A98F7D[(((word!(game.random_number)) & (0x100u16)) >> 8) as usize] as u16,
            game,
        );
        QueueSfx2_Max3(0x29, game);
    };
    MotherBrain_SetupNeckForFakeAscent [] {
        slot!(game, Enemy_MotherBrain[E].var_24 = 2);
        slot! { game, Enemy_MotherBrain[E]; var_27 = 10, var_2D = 10 }
        slot! { game, Enemy_MotherBrain[E]; var_2A = 20, var_30 = 20 }
        slot!(game, Enemy_MotherBrain[E].var_20 = 18432);
        slot!(game, Enemy_MotherBrain[E].var_21 = 20480);
        slot!(game, Enemy_MotherBrain[E].var_34 = 256);
    };
    MotherBrain_HandleNeckLower_2_BobDown [] {
        let mut v1: i16 = ((slot!(game, Enemy_MotherBrain[E].var_20)) as i16).wrapping_sub((slot!(game, Enemy_MotherBrain[E].var_34)) as i16);
        if (v1 as u16) < 0x2800 {
            slot!(game, Enemy_MotherBrain[E].var_32 = 4);
            v1 = 10240;
        }
        slot!(game, Enemy_MotherBrain[E].var_20 = v1 as u16);
    };
    MotherBrain_HandleNeckLower_4_BobUp [] {
        if slot!(game, Enemy_MotherBrain[E].base.y_pos) as i32 - 60i32 & 0x8000 != 0 {
            slot!(game, Enemy_MotherBrain[E].var_32 = 2);
        } else {
            let mut v1: i16 = ((slot!(game, Enemy_MotherBrain[E].var_34)) as i16).wrapping_add((slot!(game, Enemy_MotherBrain[E].var_20)) as i16);
            if (v1 as u16) >= 0x9000 {
                slot!(game, Enemy_MotherBrain[E].var_32 = 2);
                v1 = -28672i16;
            }
            slot!(game, Enemy_MotherBrain[E].var_20 = v1 as u16);
        };
    };
    MotherBrain_HandleNeckLower_6_Lower [] {
        let mut v1: i16 = ((slot!(game, Enemy_MotherBrain[E].var_20)) as i16).wrapping_sub((slot!(game, Enemy_MotherBrain[E].var_34)) as i16);
        if (v1 as u16) < 0x3000 {
            slot!(game, Enemy_MotherBrain[E].var_32 = 0);
            v1 = 12288;
        }
        slot!(game, Enemy_MotherBrain[E].var_20 = v1 as u16);
    };
    MotherBrain_HandleNeckLower_8_Raise [] {
        let mut v1: i16 = ((slot!(game, Enemy_MotherBrain[E].var_34)) as i16).wrapping_add((slot!(game, Enemy_MotherBrain[E].var_20)) as i16);
        if (v1 as u16) >= 0x9000 {
            slot!(game, Enemy_MotherBrain[E].var_32 = 0);
            v1 = -28672i16;
        }
        slot!(game, Enemy_MotherBrain[E].var_20 = v1 as u16);
    };
    MotherBrain_HandleNeckUpper [] {
        let mbn_var_33: u16 = slot!(game, Enemy_MotherBrain[E].var_33);
        if mbn_var_33 != 0 {
            match mbn_var_33 as i32 {
                2 => {
                    if ((slot!(game, Enemy_MotherBrain[E].base.y_pos) as i32 + 4 - word!(game.samus_y_pos) as i32) as i16) < 0 {
                        let mut v2: i16 = ((slot!(game, Enemy_MotherBrain[E].var_21)) as i16).wrapping_sub((slot!(game, Enemy_MotherBrain[E].var_34)) as i16);
                        if (v2 as u16) < 0x2000 {
                            slot!(game, Enemy_MotherBrain[E].var_33 = 4);
                            v2 = 0x2000;
                        }
                        slot!(game, Enemy_MotherBrain[E].var_21 = v2 as u16);
                    } else {
                        slot! { game, Enemy_MotherBrain[E]; var_32 = 4, var_33 = 4 }
                    }
                }
                4 => {
                    let r18: u16 = (slot!(game, Enemy_MotherBrain[E].var_20)).wrapping_add(2048);
                    let mut v4: u16 = (slot!(game, Enemy_MotherBrain[E].var_34)).wrapping_add(slot!(game, Enemy_MotherBrain[E].var_21));
                    if v4 >= r18 {
                        slot!(game, Enemy_MotherBrain[E].var_33 = 2);
                        v4 = r18;
                    }
                    slot!(game, Enemy_MotherBrain[E].var_21 = v4);
                }
                6 => {
                    let mut v6: i16 = ((slot!(game, Enemy_MotherBrain[E].var_21)) as i16).wrapping_sub((slot!(game, Enemy_MotherBrain[E].var_34)) as i16);
                    if (v6 as u16) < 0x2000 {
                        slot!(game, Enemy_MotherBrain[E].var_33 = 0);
                        v6 = 0x2000;
                    }
                    slot!(game, Enemy_MotherBrain[E].var_21 = v6 as u16);
                }
                8 => {
                    let r18_0: u16 = (slot!(game, Enemy_MotherBrain[E].var_20)).wrapping_add(2048);
                    let mut v8: u16 = (slot!(game, Enemy_MotherBrain[E].var_34)).wrapping_add(slot!(game, Enemy_MotherBrain[E].var_21));
                    if v8 >= r18_0 {
                        slot!(game, Enemy_MotherBrain[E].var_33 = 0);
                        v8 = r18_0;
                    }
                    slot!(game, Enemy_MotherBrain[E].var_21 = v8);
                }
                _ => {
                    Unreachable(game);
                    loop {}
                }
            }
        }
    };
    MotherBrain_DrawBrain [] {
        if slot!(game, Enemy_MotherBrain[E].var_14) != 0 && slot!(game, Enemy_MotherBrain[E].var_15) == 0 && (word!(game.random_number) & 0x8000) == 0 {
            SpawnEprojWithRoomGfx(
                addr_kEproj_MotherBrainPurpleBreathSmall as u16,
                word!(game.random_number),
                game,
            );
        }
        let tt: i32 = MotherBrain_Func_1_DoubleRet(game);
        if tt < 0 {
            return;
        }
        let v2: u16 = tt as u16;
        let mut mbn_var_0D: u16 = slot!(game, Enemy_MotherBrain[0i32].var_0D);
        if slot!(game, Enemy_MotherBrain[E].base.flash_timer) & 1 != 0 {
            mbn_var_0D = 0;
        }
        let r22: u16 = mbn_var_0D;
        let mbn_var_00: u16 = slot!(game, Enemy_MotherBrain[E].var_00);
        let mut flash_timer: u16 = 0;
        if mbn_var_00 != 0 {
            flash_timer = (mbn_var_00).wrapping_sub(1);
            slot!(game, Enemy_MotherBrain[E].var_00 = flash_timer);
        } else {
            flash_timer = slot!(game, Enemy_MotherBrain[E].base.flash_timer);
            if flash_timer == 0 {
                flash_timer = slot!(game, Enemy_MotherBrain[E].base.shake_timer);
            }
        }
        let v6: i32 = (flash_timer as i32 & 6) >> 1;
        let r18: u16 = (slot!(game, Enemy_MotherBrain[E].base.x_pos)).wrapping_add((g_word_A993BB[v6 as usize]) as u16);
        if ((r18 as i32 + 32 - word!(game.layer1_x_pos) as i32) as i16) >= 0 {
            let r20: u16 = (slot!(game, Enemy_MotherBrain[E].base.y_pos)).wrapping_add((g_word_A993C3[v6 as usize]) as u16);
            MotherBrain_AddSpritemapToOam(v2, r18, r20, r22, game);
        }
    };
    MotherBrain_GenerateEscapeDoorExploding [] {
        let mut v1: i16 = ((slot!(game, Enemy_MotherBrain[E].var_E)) as i16).wrapping_sub(1);
        slot!(game, Enemy_MotherBrain[E].var_E = v1 as u16);
        if v1 < 0 {
            slot!(game, Enemy_MotherBrain[E].var_E = 4);
            let mut v2: i16 = ((slot!(game, Enemy_MotherBrain[E].var_F)) as i16).wrapping_sub(1);
            slot!(game, Enemy_MotherBrain[E].var_F = v2 as u16);
            if v2 < 0 {
                slot!(game, Enemy_MotherBrain[E].var_F = 3);
            }
            let v3: i32 = ((4u16).wrapping_mul(slot!(game, Enemy_MotherBrain[E].var_F))) as i32 >> 1;
            game.eproj_spawn_pt = Point16U {
                x: g_word_A9B393[v3 as usize],
                y: g_word_A9B393[(v3 + 1) as usize],
            };
            let mut v4: u16 = 3;
            if game.next_random() < 0x4000 {
                v4 = 12;
            }
            SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, v4, game);
            QueueSfx2_Max3(0x24, game);
        }
    };
    MotherBrain_RetractHead [] {
        slot!(game, Enemy_MotherBrain[E].var_34 = 80);
        slot!(game, Enemy_MotherBrain[E].var_31 = 1);
        slot!(game, Enemy_MotherBrain[E].var_32 = 8);
        slot!(game, Enemy_MotherBrain[E].var_33 = 6);
    };
    MotherBrain_AimBeam [] {
        slot!(game, Enemy_MotherBrain[0i32].var_31 = -(actor_api::angle_from_delta((word!(game.samus_x_pos) as i32 - slot!(game, Enemy_MotherBrain[E].base.x_pos) as i32 - 16i32) as u16, (word!(game.samus_y_pos) as i32 - slot!(game, Enemy_MotherBrain[E].base.y_pos) as i32 - 4i32) as u16) as i32
            + 0x80i32) as u8 as u16);
    };
    MotherBrain_PainfulWalkForwards [] {
        if MotherBrain_MakeWalkForwards(slot!(game, Enemy_MotherBrain[E].var_07), 0x48, game) & 1 != 0 {
            slot!(game, Enemy_MotherBrain[E].var_08 = (0xa9bfe8i32) as u16);
            MotherBrain_SetPainfulWalkingTimer(game);
        }
    };
    MotherBrain_PainfulWalkBackwards [] {
        if MotherBrain_MakeWalkBackwards(0x28, slot!(game, Enemy_MotherBrain[E].var_07), game) & 1 != 0 {
            slot!(game, Enemy_MotherBrain[E].var_08 = (0xa9c01ci32) as u16);
            MotherBrain_SetPainfulWalkingTimer(game);
        }
    };
    MotherBrain_SetPainfulWalkingTimer [] {
        slot!(game, Enemy_MotherBrain[E].var_09 = byte!(@pointer (&raw const g_word_A9C049 as *const i16).offset(slot!(game, Enemy_MotherBrain[E].var_06) as isize); 0)
            as u16);
    };
    MotherBrain_Phase3_Walk_RetreatQuickly [] {
        if MotherBrain_MakeWalkBackwards(slot!(game, Enemy_MotherBrain[E].var_1B), 2, game) & 1 != 0 {
            slot!(game, Enemy_MotherBrain[E].var_1B = (slot!(game, Enemy_MotherBrain[0i32].base.x_pos)).wrapping_sub(14));
            slot!(game, Enemy_MotherBrain[E].var_1A = (0xa9c2d2i32) as u16);
        }
    };
    MotherBrain_Phase3_Walk_RetreatSlowly [] {
        if MotherBrain_MakeWalkBackwards(slot!(game, Enemy_MotherBrain[E].var_1B), 4, game) & 1 != 0 {
            MotherBrain_SetToTryToInchForward(0x40, game);
        }
    };
    [11125552] MotherBrain_Phase3_Neck_Normal [] {
        slot! { game, Enemy_MotherBrain[E]; var_32 = 1, var_34 = 128, var_32 = 2, var_33 = 4, var_18 = addr_locret_A9C353 as u16 };
    };
    [11125588] MotherBrain_Phase3_Neck_SetupRecoilRecovery [] {
        slot!(game, Enemy_MotherBrain[E].var_31 = 1);
        slot!(game, Enemy_MotherBrain[E].var_34 = 1280);
        slot! { game, Enemy_MotherBrain[E]; var_32 = 6, var_33 = 6 }
        slot!(game, Enemy_MotherBrain[E].var_18 = (0xa9c37bi32) as u16);
        slot!(game, Enemy_MotherBrain[E].var_19 = 16);
        MotherBrain_Phase3_Neck_RecoilRecovery(game);
    };
    [11125627] MotherBrain_Phase3_Neck_RecoilRecovery [] {
        let mut v1: i16 = ((slot!(game, Enemy_MotherBrain[E].var_19)) as i16).wrapping_sub(1);
        if v1 < 0 {
            MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9DBB as u16, game);
            slot!(game, Enemy_MotherBrain[E].var_18 = (0xa9c330i32) as u16);
        } else {
            slot!(game, Enemy_MotherBrain[E].var_19 = v1 as u16);
        };
    };
    [11125653] MotherBrain_Phase3_Neck_SetupHyperBeamRecoil [] {
        slot!(game, Enemy_MotherBrain[E].var_31 = 1);
        slot!(game, Enemy_MotherBrain[0i32].var_06 = 1);
        MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9BE7 as u16, game);
        slot!(game, Enemy_MotherBrain[E].var_00 = 50);
        slot!(game, Enemy_MotherBrain[E].var_34 = 2304);
        slot! { game, Enemy_MotherBrain[E]; var_32 = 8, var_33 = 8 }
        slot!(game, Enemy_MotherBrain[E].var_18 = (0xa9c3cdi32) as u16);
        slot!(game, Enemy_MotherBrain[E].var_19 = 11);
        MotherBrain_Phase3_Neck_HyperBeamRecoil(game);
    };
    [11125709] MotherBrain_Phase3_Neck_HyperBeamRecoil [] {
        let mut v2: i16 = ((slot!(game, Enemy_MotherBrain[E].var_19)) as i16).wrapping_sub(1);
        if v2 < 0 {
            slot!(game, Enemy_MotherBrain[E].var_34 = 128);
            slot!(game, Enemy_MotherBrain[0i32].var_06 = 0);
            slot!(game, Enemy_MotherBrain[E].var_18 = (0xa9c354i32) as u16);
        } else {
            slot!(game, Enemy_MotherBrain[E].var_19 = v2 as u16);
        };
    };
    MotherBrain_HandlePalette [] {
        if slot!(game, Enemy_MotherBrain[E].var_10) != 0 {
            MotherBrain_HandleBrainPal(game);
        }
        if slot!(game, Enemy_MotherBrain[E].var_11) != 0 {
            mother_brain_visuals::update_health_palette(game);
        }
    };
    @external {
        11125587 => [()];
    }
}
pub unsafe fn MotherBrain_Phase3_NeckHandler(game: &mut Game) {
    unsafe {
        CallMotherBrainNeckFunc(
            ((slot!(game, Enemy_MotherBrain[0x40i32].var_18)) as u32) | (0xa90000u32),
            game,
        );
    }
}
pub unsafe fn MoveEnemyWithVelocity(game: &mut Game) {
    unsafe {
        let v0: i32 = word!(game.cur_enemy_index) as i32;
        let mut ai_var_B: u16 = slot!(game, EnemyData[v0].ai_var_B);
        let mut carry: i32 =
            slot!(byte game, EnemyData[v0].x_subpos[1]) as i32 + byte!(ai_var_B; 0) as i32;
        slot!(byte game, EnemyData[v0].x_subpos[1] = carry as u8);
        slot!(
            game,
            EnemyData[v0].x_pos = (slot!(game, EnemyData[v0].x_pos) as i32
                + ((ai_var_B >> 8) as i8 as i32 + (carry >> 8)))
                as u16
        );
        let mut ai_var_C: u16 = slot!(game, EnemyData[v0].ai_var_C);
        carry = slot!(byte game, EnemyData[v0].y_subpos[1]) as i32 + byte!(ai_var_C; 0) as i32;
        slot!(byte game, EnemyData[v0].y_subpos[1] = carry as u8);
        slot!(
            game,
            EnemyData[v0].y_pos = (slot!(game, EnemyData[v0].y_pos) as i32
                + ((ai_var_C >> 8) as i8 as i32 + (carry >> 8)))
                as u16
        );
    }
}
pub unsafe fn MotherBrain_SetBodyInstrs(a: u16, game: &mut Game) {
    let E: i32 = 0;
    slot! { game, Enemy_MotherBrain[E].base; current_instruction = a, instruction_timer = 1 }
    slot!(game, Enemy_MotherBrain[E].base.timer = 0);
}
pub unsafe fn MotherBrain_SetBrainUnusedInstrs(a: u16, game: &mut Game) {
    let E: i32 = 0x40;
    slot! { game, Enemy_MotherBrain[E].base; current_instruction = a, instruction_timer = 1 }
    slot!(game, Enemy_MotherBrain[E].base.timer = 0);
}
pub unsafe fn MotherBrain_SetBrainInstrs(a: u16, game: &mut Game) {
    let E: i32 = 0;
    slot!(game, Enemy_MotherBrain[E].var_21 = a);
    slot!(game, Enemy_MotherBrain[E].var_20 = 1);
}
pub fn Enemy_SetInstrList(k: u16, a: u16, game: &mut Game) {
    let v2: i32 = k as i32;
    slot! { game, EnemyData[v2]; current_instruction = a, instruction_timer = 1 }
    slot!(game, EnemyData[v2].timer = 0);
}
crate::forward_calls! { pub unsafe fn (a: u16, r18: u16, game: &mut Game) -> u16;
    ComputeSinMult => sub_A9C46C(r18, a, game);
    ComputeCosMult => sub_A9C46C((r18).wrapping_add(64), a, game);
}
pub unsafe fn sub_A9C46C(a: u16, j: u16, game: &mut Game) -> u16 {
    unsafe {
        let mut v2: u16 = kSinCosTable8bit_Sext[((a as i32 & 0xff) + 64) as usize] as u16;
        crate::rtl::register_writes! { game;
            byte(M7A as u16, v2 as u8); byte(M7A as u16, *(&raw mut v2 as *mut u8).offset(1));
            byte(M7B as u16, j as u8);
        }
        ReadRegWord(MPYM as u16, game)
    }
}
pub unsafe fn Enemy_IncreaseYpos(k: u16, mut a: u16, game: &mut Game) {
    unsafe {
        let v2: i32 = k as i32;
        let carry: i32 = slot!(byte game, EnemyData[v2].y_subpos[1]) as i32 + byte!(a; 0) as i32;
        slot!(byte game, EnemyData[v2].y_subpos[1] = carry as u8);
        slot!(
            game,
            EnemyData[v2].y_pos = (slot!(game, EnemyData[v2].y_pos) as i32
                + ((a >> 8) as i8 as i32 + (carry >> 8))) as u16
        );
    }
}
pub unsafe fn MotherBrain_MakeWalkForwards(j: u16, a: u16, game: &mut Game) -> u8 {
    unsafe {
        let E: i32 = 0;
        if ((a) as i16).wrapping_sub((slot!(game, Enemy_MotherBrain[E].base.x_pos)) as i16) >= 0 {
            if slot!(game, Enemy_MotherBrain[E].var_02) != 0 {
                return 0;
            }
            if slot!(game, Enemy_MotherBrain[E].base.x_pos) as i32 - 128i32 & 0x8000 != 0 {
                MotherBrain_SetBodyInstrs(
                    crate::rtl::rom_table_word!(game, 0xa9c61e, (j >> 1) as isize),
                    game,
                );
                return 0;
            }
        }
        1
    }
}
pub unsafe fn MotherBrain_MakeWalkBackwards(a: u16, j: u16, game: &mut Game) -> u8 {
    unsafe {
        let E: i32 = 0;
        if ((a) as i16).wrapping_sub((slot!(game, Enemy_MotherBrain[E].base.x_pos)) as i16) < 0 {
            if slot!(game, Enemy_MotherBrain[E].var_02) != 0 {
                return 0;
            }
            if slot!(game, Enemy_MotherBrain[E].base.x_pos) as i32 - 48i32 & 0x8000 == 0 {
                MotherBrain_SetBodyInstrs(
                    crate::rtl::rom_table_word!(game, 0xa9c664, (j >> 1) as isize),
                    game,
                );
                return 0;
            }
        }
        1
    }
}
pub unsafe fn MotherBrain_MakeHerStandUp(game: &mut Game) -> u8 {
    unsafe {
        let mbn_var_02: u16 = slot!(game, Enemy_MotherBrain[0i32].var_02);
        if mbn_var_02 != 0 {
            let mut v1: u16 = addr_kMotherBrain_Ilist_99C6 as u16;
            if mbn_var_02 != 3 {
                if mbn_var_02 != 6 {
                    return 0;
                }
                v1 = addr_kMotherBrain_Ilist_99E2 as u16;
            }
            MotherBrain_SetBodyInstrs(v1, game);
            return 0;
        }
        1
    }
}
pub unsafe fn MotherBrain_MakeHerCrouch(game: &mut Game) -> u8 {
    unsafe {
        let mbn_var_02: u16 = slot!(game, Enemy_MotherBrain[0i32].var_02);
        if mbn_var_02 != 0 {
            if mbn_var_02 == 3 {
                return 1;
            }
        } else {
            MotherBrain_SetBodyInstrs(addr_kMotherBrain_Ilist_9A0A as u16, game);
        }
        0
    }
}
pub unsafe fn MotherBrain_MakeHerLeanDown(game: &mut Game) -> u8 {
    unsafe {
        let mbn_var_02: u16 = slot!(game, Enemy_MotherBrain[0i32].var_02);
        if mbn_var_02 != 0 {
            if mbn_var_02 == 6 {
                return 1;
            }
        } else {
            MotherBrain_SetBodyInstrs(addr_kMotherBrain_Ilist_99F2 as u16, game);
        }
        0
    }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallShitroidCutsceneFuncA(ea: u32, k: u16, game: &mut Game) => ea;
    call (k, game); default { Unreachable(game); };
    @functions [pub unsafe fn (k: u16, game: &mut Game)];
    [11126764] ShitroidInCutscene_CurveTowardsBrain [] {
        ShitroidInCutscene_UpdateSpeedAndAngle(k, -384i32 as u16, -20480i32 as u16, 2560, game);
        let E: i32 = k as i32;
        slot!(game, Enemy_ShitroidInCutscene[E].var_F = slot!(game, Enemy_ShitroidInCutscene[E].var_F).wrapping_sub(1));
        if (slot!(game, Enemy_ShitroidInCutscene[E].var_F) & 0x8000) != 0 {
            slot!(game, Enemy_ShitroidInCutscene[E].var_A = (0xa9c811i32) as u16);
            slot!(game, Enemy_ShitroidInCutscene[E].var_F = 9);
        }
    };
    [11126801] ShitroidInCutscene_GetIntoFace [] {
        ShitroidInCutscene_UpdateSpeedAndAngle(k, -1536i32 as u16, -32256i32 as u16, 3584, game);
        let mut E: i32 = 0x40;
        let rect: Rect16U = Rect16U {
            x: slot!(game, Enemy_ShitroidInCutscene[E].base.x_pos),
            y: slot!(game, Enemy_ShitroidInCutscene[E].base.y_pos),
            w: 4,
            h: 4,
        };
        if Shitroid_Func_2(k, rect, game) == 0 || {
            E = k as i32;
            let mut v3: bool = ((slot!(game, Enemy_ShitroidInCutscene[E].var_F)) as i16).wrapping_sub(1) < 0;
            slot!(game, Enemy_ShitroidInCutscene[E].var_F = slot!(game, Enemy_ShitroidInCutscene[E].var_F).wrapping_sub(1));
            v3 as i32 != 0
        } {
            slot!(game, Enemy_ShitroidInCutscene[k as i32].var_A = (0xa9c851i32) as u16);
            player_mother_brain(1, game);
        }
    };
    [11126865] ShitroidInCutscene_LatchOntoBrain [] {
        let E: i32 = 0x40;
        let rect: Rect16U = Rect16U {
            x: slot!(game, Enemy_ShitroidInCutscene[E].base.x_pos),
            y: ((slot!(game, Enemy_ShitroidInCutscene[E].base.y_pos)).wrapping_sub(24)),
            w: 8,
            h: 8,
        };
        Shitroid_Func_GraduallyAccelerateTowards0x400(k, 0, rect.x, rect.y, game);
        if Shitroid_Func_2(k, rect, game) == 0 {
            slot!(game, Enemy_ShitroidInCutscene[k as i32].var_A = (0xa9c879i32) as u16);
        }
    };
    [11126905] ShitroidInCutscene_SetMotherBrainToStumbleBack [] {
        let E: i32 = 0;
        MotherBrain_MakeWalkBackwards((slot!(game, Enemy_ShitroidInCutscene[E].base.x_pos)).wrapping_sub(1), 2, game);
        slot!(game, Enemy_ShitroidInCutscene[k as i32].var_A = (0xa9c889i32) as u16);
        ShitroidInCutscene_ActivateRainbowBeam(k, game);
    };
    [11126921] ShitroidInCutscene_ActivateRainbowBeam [] {
        let E1: i32 = 0x40;
        if Shitroid_AccelerateTowardsPoint(
            k,
            0x200,
            slot!(game, Enemy_ShitroidInCutscene[E1].base.x_pos),
            (slot!(game, Enemy_ShitroidInCutscene[E1].base.y_pos)).wrapping_sub(24),
            game,
        ) & 1
            != 0
        {
            let E: i32 = k as i32;
            slot! { game, Enemy_ShitroidInCutscene[E]; var_B = 0, var_C = 0 }
            slot!(game, Enemy_ShitroidInCutscene[E].base.x_pos = slot!(game, Enemy_ShitroidInCutscene[E1].base.x_pos));
            slot!(game, Enemy_ShitroidInCutscene[E].base.y_pos = (slot!(game, Enemy_ShitroidInCutscene[E1].base.y_pos)).wrapping_sub(24));
            Enemy_SetInstrList(k, addr_kShitroid_Ilist_CFB8 as u16, game);
            slot!(game, Enemy_ShitroidInCutscene[E].var_A = (0xa9c8e2i32) as u16);
            slot!(game, Enemy_ShitroidInCutscene[E].var_E = 1);
            let E0: i32 = 0;
            slot!(game, Enemy_ShitroidInCutscene[E0].var_A = (0xa9be38i32) as u16);
            QueueSfx1_Max6(0x40, game);
            slot!(game, Enemy_ShitroidInCutscene[E0].var_16 = 1);
        }
    };
    [11127061] ShitroidInCutscene_StopDraining [] {
        let E: i32 = k as i32;
        let E1: i32 = 0x40;
        slot!(game, Enemy_ShitroidInCutscene[E].base.x_pos = slot!(game, Enemy_ShitroidInCutscene[E1].base.x_pos));
        slot!(game, Enemy_ShitroidInCutscene[E].base.y_pos = (slot!(game, Enemy_ShitroidInCutscene[E1].base.y_pos)).wrapping_sub(24));
        slot!(game, Enemy_ShitroidInCutscene[E].var_F = slot!(game, Enemy_ShitroidInCutscene[E].var_F).wrapping_sub(1));
        if (slot!(game, Enemy_ShitroidInCutscene[E].var_F) & 0x8000) != 0 {
            Enemy_SetInstrList(k, addr_kShitroid_Ilist_CFA2 as u16, game);
            slot!(game, Enemy_ShitroidInCutscene[E].var_E = 10);
            slot!(game, Enemy_ShitroidInCutscene[E].var_A = (0xa9c94bi32) as u16);
            slot!(game, Enemy_ShitroidInCutscene[E].var_F = 32);
            slot! { game, Enemy_ShitroidInCutscene[E]; var_B = 0, var_C = 0 }
        }
    };
    [11127129] ShitroidInCutscene_MoveUpToCeiling [] {
        let rect: Rect16U = Rect16U {
            x: slot!(game, Enemy_ShitroidInCutscene[0x40i32].base.x_pos),
            y: 0,
            w: 4,
            h: 4,
        };
        Shitroid_Func_GraduallyAccelerateTowards0x400(k, 0, rect.x, rect.y, game);
        if Shitroid_Func_2(k, rect, game) == 0 {
            player_mother_brain(4, game);
            let E: i32 = k as i32;
            slot!(game, Enemy_ShitroidInCutscene[E].var_A = (0xa9c9c3i32) as u16);
            slot!(game, Enemy_ShitroidInCutscene[E].var_0E = addr_stru_A9CA24 as u16);
        }
    };
    [11127398] ShitroidInCutscene_LatchOntoSamus [] {
        Shitroid_Func_GraduallyAccelerateTowards0x400(
            k,
            0,
            word!(game.samus_x_pos),
            (word!(game.samus_y_pos)).wrapping_sub(20),
            game,
        );
    };
    [11127485] ShitroidInCutscene_IdleUntilToNoHealth [] {
        ShitroidInCutscene_HandleCry(game);
        let E: i32 = k as i32;
        let sice_var_06: u16 = slot!(game, Enemy_ShitroidInCutscene[E].var_06);
        if sice_var_06 != 0 {
            let v3: i32 = (sice_var_06 as i32 & 6) >> 1;
            slot!(game, Enemy_ShitroidInCutscene[E].base.x_pos = (word!(game.samus_x_pos) as i32 + 2 * g_word_A993BB[v3 as usize] as i32) as u16);
            slot!(game, Enemy_ShitroidInCutscene[E].base.y_pos = (word!(game.samus_y_pos) as i32
                + 2 * g_word_A993C3[v3 as usize] as i32
                - 20i32) as u16);
        }
        if slot!(game, Enemy_ShitroidInCutscene[E].base.health) == 0 {
            slot!(game, Enemy_ShitroidInCutscene[E].base.health = 320);
            slot! { game, Enemy_ShitroidInCutscene[E]; var_A = (0xa9cb13i32) as u16, var_E = 10, var_D = 0, var_0F = (0xa9f68fi32) as u16, var_04 = 1, var_09 = 0 };
        }
    };
    [11127571] ShitroidInCutscene_ReleaseSamus [] {
        QueueSfx2_Max6(0x72, game);
        slot!(game, Enemy_ShitroidInCutscene[0x40i32].var_0B = 1);
        slot!(game, Enemy_ShitroidInCutscene[k as i32].var_A = (0xa9cb2di32) as u16);
        slot!(game, Enemy_ShitroidInCutscene[0i32].var_A = (0xa9c18ei32) as u16);
        ShitroidInCutscene_StareDownMotherBrain(k, game);
    };
    [11127597] ShitroidInCutscene_StareDownMotherBrain [] {
        let rect: Rect16U = Rect16U {
            x: ((word!(game.samus_x_pos)).wrapping_sub(4)),
            y: 96,
            w: 4,
            h: 4,
        };
        Shitroid_GraduallyAccelerateTowards0x10(k, 0, rect.x, rect.y, game);
        if Shitroid_Func_2(k, rect, game) == 0 {
            slot!(game, Enemy_ShitroidInCutscene[k as i32].var_A = (0xa9cb56i32) as u16);
        }
    };
    [11127638] ShitroidInCutscene_FlyOffScreen [] {
        let rect: Rect16U = Rect16U {
            x: 272,
            y: 64,
            w: 4,
            h: 4,
        };
        Shitroid_GraduallyAccelerateTowards0x10(k, 0, rect.x, rect.y, game);
        if Shitroid_Func_2(k, rect, game) == 0 {
            slot!(game, Enemy_ShitroidInCutscene[k as i32].var_A = (0xa9cb7bi32) as u16);
        }
    };
    [11127675] ShitroidInCutscene_MoveToFinalChargeStart [] {
        let rect: Rect16U = Rect16U {
            x: 305,
            y: 160,
            w: 4,
            h: 4,
        };
        Shitroid_GraduallyAccelerateTowards0x10(k, 0, rect.x, rect.y, game);
        if Shitroid_Func_2(k, rect, game) == 0 {
            let E: i32 = k as i32;
            slot!(game, Enemy_ShitroidInCutscene[E].base.health = 79);
            slot!(game, Enemy_ShitroidInCutscene[0x40i32].var_0B = 0);
            slot!(game, Enemy_ShitroidInCutscene[0i32].var_A = (0xa9c19ai32) as u16);
            slot!(game, Enemy_ShitroidInCutscene[E].var_A = (0xa9cbb3i32) as u16);
        }
    };
    [11127731] ShitroidInCutscene_InitiateFinalCharge [] {
        let rect: Rect16U = Rect16U {
            x: 290,
            y: 128,
            w: 4,
            h: 4,
        };
        Shitroid_Func_GraduallyAccelerateTowards0x400(k, 0xa, rect.x, rect.y, game);
        if Shitroid_Func_2(k, rect, game) == 0 {
            slot!(game, Enemy_ShitroidInCutscene[k as i32].var_A = (0xa9cbd8i32) as u16);
        }
    };
    [11127768] ShitroidInCutscene_FinalCharge [] {
        let E1: i32 = 0x40;
        Shitroid_Func_GraduallyAccelerateTowards0x400(
            k,
            0xc,
            slot!(game, Enemy_ShitroidInCutscene[E1].base.x_pos),
            (slot!(game, Enemy_ShitroidInCutscene[E1].base.y_pos)).wrapping_sub(32),
            game,
        );
        let E: i32 = k as i32;
        if slot!(game, Enemy_ShitroidInCutscene[E].base.health) == 0 {
            slot!(game, Enemy_ShitroidInCutscene[E].var_0F = (0xa9cc98i32) as u16);
            slot!(game, Enemy_ShitroidInCutscene[E].base.vram_tiles_index = 4256);
            QueueSfx3_Max6(0x19, game);
            mother_brain_visuals::lights_off(game);
            Enemy_SetInstrList(k, addr_kShitroid_Ilist_CFCE as u16, game);
            slot! { game, Enemy_ShitroidInCutscene[E]; var_B = 0, var_C = 0 }
            slot!(game, Enemy_ShitroidInCutscene[0i32].var_A = addr_locret_A9C18D as u16);
            QueueMusic_Delayed8(0, game);
            slot!(game, Enemy_ShitroidInCutscene[E].var_A = (0xa9cc3ei32) as u16);
            slot!(game, Enemy_ShitroidInCutscene[E].var_F = 16);
            slot!(game, Enemy_ShitroidInCutscene[E].var_10 = slot!(game, Enemy_ShitroidInCutscene[E].base.x_pos));
            slot!(game, Enemy_ShitroidInCutscene[E].var_11 = slot!(game, Enemy_ShitroidInCutscene[E].base.y_pos));
            ShitroidInCutscene_ShitroidFinalBelow(k, game);
        }
    };
    [11127870] ShitroidInCutscene_ShitroidFinalBelow [] {
        ShitroidInCutscene_Shake(k, game);
        let E: i32 = k as i32;
        slot!(game, Enemy_ShitroidInCutscene[E].var_F = slot!(game, Enemy_ShitroidInCutscene[E].var_F).wrapping_sub(1));
        if (slot!(game, Enemy_ShitroidInCutscene[E].var_F) & 0x8000) != 0 {
            slot!(game, Enemy_ShitroidInCutscene[E].base.x_pos = slot!(game, Enemy_ShitroidInCutscene[E].var_10));
            slot!(game, Enemy_ShitroidInCutscene[E].base.y_pos = slot!(game, Enemy_ShitroidInCutscene[E].var_11));
            slot!(game, Enemy_ShitroidInCutscene[E].var_A = (0xa9cc60i32) as u16);
            slot!(game, Enemy_ShitroidInCutscene[E].var_F = 56);
            ShitroidInCutscene_PlaySamusTheme(k, game);
        }
    };
    [11127904] ShitroidInCutscene_PlaySamusTheme [] {
        let E: i32 = k as i32;
        slot!(game, Enemy_ShitroidInCutscene[E].var_F = slot!(game, Enemy_ShitroidInCutscene[E].var_F).wrapping_sub(1));
        if (slot!(game, Enemy_ShitroidInCutscene[E].var_F) & 0x8000) != 0 {
            QueueMusic_Delayed8(0xff48, game);
            QueueMusic_Delayed8(5, game);
            slot!(game, Enemy_ShitroidInCutscene[E].var_A = (0xa9cc7fi32) as u16);
            slot!(game, Enemy_ShitroidInCutscene[E].var_F = 12);
            ShitroidInCutscene_PrepareSamusHyperbeam(k, game);
        }
    };
    [11127961] ShitroidInCutscene_DeathSequence [] {
        HandleSamusRainbowPaletteAnimation(k, game);
        ShitroidInCutscene_AccelerateDownwards(k, game);
        if ShitroidInCutscene_FadeShitroidToBlack(k, game) & 1 != 0 {
            let E: i32 = k as i32;
            slot!(game, Enemy_ShitroidInCutscene[E].base.properties = (slot!(game, Enemy_ShitroidInCutscene[E].base.properties)) | ((kEnemyProps_Invisible) as u16));
            slot!(game, Enemy_ShitroidInCutscene[E].var_A = (0xa9ccc0i32) as u16);
            slot!(game, Enemy_ShitroidInCutscene[E].var_F = 128);
        } else {
            ShitroidInCutscene_HandleShitroidDeathExplosions(k, game);
            ShitroidInCutscene_HandleEnemyBlinking(k, game);
        };
    };
    [11128000] ShitroidInCutscene_UnloadShitroid [] {
        HandleSamusRainbowPaletteAnimation(k, game);
        let E: i32 = k as i32;
        slot!(game, Enemy_ShitroidInCutscene[E].var_F = slot!(game, Enemy_ShitroidInCutscene[E].var_F).wrapping_sub(1));
        let v2: bool = (slot!(game, Enemy_ShitroidInCutscene[E].var_F) & 0x8000) != 0;
        if v2 && ProcessSpriteTilesTransfers(0xa9, addr_stru_A98FC7 as u16, game) != 0 {
            slot!(game, Enemy_ShitroidInCutscene[E].var_A = (0xa9ccdei32) as u16);
            slot!(game, Enemy_ShitroidInCutscene[E].var_F = 176);
            ShitroidInCutscene_LetSamusRainbowMore(k, game);
        }
    };
    [11128048] ShitroidInCutscene_FinishCutscene [] {
        let E: i32 = 0;
        let v2: u16 = (slot!(game, Enemy_ShitroidInCutscene[E].var_37)).wrapping_add(1);
        slot!(game, Enemy_ShitroidInCutscene[E].var_37 = v2);
        if mother_brain_visuals::fade(Fade::LightsOn, (v2).wrapping_sub(1), game) & 1 != 0 {
            slot!(game, Enemy_ShitroidInCutscene[E].var_A = (0xa9c1cfi32) as u16);
            CallSomeSamusCode(0x17, game);
            player_mother_brain(3, game);
            let fresh12: i32 = k as i32;
            slot!(game, Enemy_ShitroidInCutscene[fresh12].base.properties, |old| old | (kEnemyProps_Deleted as u16));
            slot!(game, Enemy_ShitroidInCutscene[0x40i32].var_0A = 0);
        }
    };
    @functions [pub unsafe fn (k: u16, game: &mut Game)];
    @prefix { let E: i32 = k as i32; };
    [11127010] ShitroidInCutscene_BrainTurnsToCorpse [] {
        let E1: i32 = 0x40;
        let v2: i32 = (slot!(game, Enemy_ShitroidInCutscene[E].base.frame_counter) as i32 & 6) >> 1;
        slot!(game, Enemy_ShitroidInCutscene[E].base.x_pos = (slot!(game, Enemy_ShitroidInCutscene[E1].base.x_pos)).wrapping_add((g_word_A993BB[v2 as usize]) as u16));
        slot!(game, Enemy_ShitroidInCutscene[E].base.y_pos = (slot!(game, Enemy_ShitroidInCutscene[E1].base.y_pos) as i32 + g_word_A993C3[v2 as usize] as i32 - 24i32) as u16);
        if slot!(game, Enemy_ShitroidInCutscene[0i32].var_1F) != 0 {
            slot!(game, Enemy_ShitroidInCutscene[E].var_A = (0xa9c915i32) as u16);
            slot!(game, Enemy_ShitroidInCutscene[E].var_F = 64);
        }
    };
    [11127115] ShitroidInCutscene_LetGoAndSpawnDust [] {
        slot!(game, Enemy_ShitroidInCutscene[E].var_F = slot!(game, Enemy_ShitroidInCutscene[E].var_F).wrapping_sub(1));
        if (slot!(game, Enemy_ShitroidInCutscene[E].var_F) & 0x8000) != 0 {
            ShitroidInCutscene_SpawnThreeDustClouds(game);
            slot!(game, Enemy_ShitroidInCutscene[E].var_A = addr_loc_A9C959 as u16);
        }
        ShitroidInCutscene_MoveUpToCeiling(k, game);
    };
    [11127235] ShitroidInCutscene_MoveToSamus [] {
        slot!(game, Enemy_ShitroidInCutscene[E].var_04 = 0);
        slot!(game, Enemy_ShitroidInCutscene[E].var_09 = 1);
        if (word!(game.random_number) & 0xfff) >= 0xfa0 {
            QueueSfx2_Max6(0x52, game);
        }
        let sice_var_0E: u16 = slot!(game, Enemy_ShitroidInCutscene[E].var_0E);
        let v3 = RomPtr_A9(sice_var_0E, game) as *const u16;
        let rect: Rect16U = Rect16U {
            x: *v3,
            y: *v3.offset(1),
            w: 4,
            h: 4,
        };
        CallShitroidMoveFunc(
            ((*v3.offset(3)) as u32) | (0xa90000u32),
            k,
            *v3.offset(2),
            rect.x,
            rect.y,
            game,
        );
        if Shitroid_Func_2(k, rect, game) == 0 {
            let mut v4: i16 = *(RomPtr_A9(sice_var_0E, game) as *mut u16).offset(4) as i16;
            if v4 < 0 {
                slot!(game, Enemy_ShitroidInCutscene[E].var_A = v4 as u16);
            } else {
                slot!(game, Enemy_ShitroidInCutscene[E].var_0E = (slot!(game, Enemy_ShitroidInCutscene[E].var_0E)).wrapping_add(8));
            }
        }
    };
    [11127418] ShitroidInCutscene_HealSamusToFullHealth [] {
        slot!(game, Enemy_ShitroidInCutscene[E].var_04 = 0);
        ShitroidInCutscene_HandleCry(game);
        let v2: i32 = (slot!(game, Enemy_ShitroidInCutscene[E].base.frame_counter) as i32 & 6) >> 1;
        slot!(game, Enemy_ShitroidInCutscene[E].base.x_pos = (word!(game.samus_x_pos)).wrapping_add((g_word_A993BB[v2 as usize]) as u16));
        slot!(game, Enemy_ShitroidInCutscene[E].base.y_pos = (word!(game.samus_y_pos) as i32 + g_word_A993C3[v2 as usize] as i32 - 20i32) as u16);
        if !vitals::heal_by_baby_metroid(game) {
            word!(game.samus_reserve_health = word!(game.samus_max_reserve_health));
            slot!(game, Enemy_ShitroidInCutscene[E].var_A = (0xa9cabdi32) as u16);
            slot!(game, Enemy_ShitroidInCutscene[E].var_0F = (0xa9cabci32) as u16);
        }
    };
    ShitroidInCutscene_HandleShitroidDeathExplosions [] {
        let mut v2: i16 = ((slot!(game, Enemy_ShitroidInCutscene[E].var_08)) as i16).wrapping_sub(1);
        if v2 < 0 {
            slot!(game, Enemy_ShitroidInCutscene[E].var_08 = 4);
            let mut v3: u16 = (slot!(game, Enemy_ShitroidInCutscene[E].var_07)).wrapping_add(1);
            if slot!(game, Enemy_ShitroidInCutscene[E].var_07) as i32 - 9i32 & 0x8000 == 0 {
                v3 = 0;
            }
            slot!(game, Enemy_ShitroidInCutscene[E].var_07 = v3);
            let v4: i32 = ((4u16).wrapping_mul(v3)) as i32 >> 1;
            crate::game::sm_a6::set_explosion_spawn_offset!(game, E, 0xa9cdfc, v4);
            SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 3, game);
            QueueSfx3_Max3(0x13, game);
        } else {
            slot!(game, Enemy_ShitroidInCutscene[E].var_08 = v2 as u16);
        };
    };
    ShitroidInCutscene_HandleHealthBasedPalette [] {
        if slot!(game, Enemy_ShitroidInCutscene[E].var_09) != 0 {
            let health: u16 = slot!(game, Enemy_ShitroidInCutscene[E].base.health);
            if health as i32 - 2560i32 & 0x8000 != 0 {
                let mut v3: u16 = 2;
                if health as i32 - 2240i32 & 0x8000 != 0 {
                    v3 = 4;
                    if health as i32 - 1920i32 & 0x8000 != 0 {
                        v3 = 6;
                        if health as i32 - 1600i32 & 0x8000 != 0 {
                            v3 = 8;
                            if health as i32 - 1280i32 & 0x8000 != 0 {
                                v3 = 10;
                                if health as i32 - 960i32 & 0x8000 != 0 {
                                    v3 = 12;
                                    if health as i32 - 640i32 & 0x8000 != 0 {
                                        v3 = 14;
                                    }
                                }
                            }
                        }
                    }
                }
                let v5: u16 = v3;
                let v4: u16 = crate::rtl::rom_table_word!(game, 0xade7e2, (v3 >> 1) as isize);
                WriteColorsToPalette(0x1e2, 0xad, v4, 4, game);
                WriteColorsToPalette(0x1f4, 0xad, (v4).wrapping_add(8), 5, game);
                WriteColorsToPalette(
                    0x1ea,
                    0xad,
                    crate::rtl::rom_table_word!(game, 0xade882, (v5 >> 1) as isize),
                    5,
                    game,
                );
            }
        }
    };
    ShitroidInCutscene_Shake [] {
        slot!(game, Enemy_ShitroidInCutscene[E].var_C = slot!(game, Enemy_ShitroidInCutscene[E].var_C).wrapping_sub(1));
        let v2: i32 = (slot!(game, Enemy_ShitroidInCutscene[E].base.frame_counter) as i32 & 6) >> 1;
        slot!(game, Enemy_ShitroidInCutscene[E].base.x_pos = (slot!(game, Enemy_ShitroidInCutscene[E].var_10)).wrapping_add((g_word_A993BB[v2 as usize]) as u16));
        slot!(game, Enemy_ShitroidInCutscene[E].base.y_pos = (slot!(game, Enemy_ShitroidInCutscene[E].var_11)).wrapping_add((g_word_A993C3[v2 as usize]) as u16));
    };
    @countdown [pub unsafe fn (k: u16, game: &mut Game)];
    before {
        let E: i32 = k as i32;
        slot!(game, Enemy_ShitroidInCutscene[E].var_F = slot!(game, Enemy_ShitroidInCutscene[E].var_F).wrapping_sub(1));
    }
    when (slot!(game, Enemy_ShitroidInCutscene[E].var_F) & 0x8000) != 0;
    [11126732] ShitroidInCutscene_DashOntoScreen {
        slot!(game, Enemy_ShitroidInCutscene[E].var_0A = -10240i32 as u16);
        slot!(game, Enemy_ShitroidInCutscene[E].var_0B = 2560);
        slot!(game, Enemy_ShitroidInCutscene[E].var_A = (0xa9c7eci32) as u16);
        slot!(game, Enemy_ShitroidInCutscene[E].var_F = 10);
        ShitroidInCutscene_CurveTowardsBrain(k, game);
    };
    [11127935] ShitroidInCutscene_PrepareSamusHyperbeam {
        CallSomeSamusCode(0x19, game);
        slot!(game, Enemy_ShitroidInCutscene[0x40i32].var_16 = (0xa9cd30i32) as u16);
        slot!(game, Enemy_ShitroidInCutscene[E].var_A = (0xa9cc99i32) as u16);
    };
    [11128030] ShitroidInCutscene_LetSamusRainbowMore {
        slot!(game, Enemy_ShitroidInCutscene[E].var_A = (0xa9ccf0i32) as u16);
        slot!(game, Enemy_ShitroidInCutscene[0i32].var_37 = 0);
        ShitroidInCutscene_FinishCutscene(k, game);
    };
    @external {
        11125133 => [()];
    }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallShitroidCutsceneFunc0F(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @forward [pub unsafe fn (game: &mut Game)];
    Shitroid_HandleNormalPalette => Shitroid_HandleCutscenePalette_Common(addr_word_A9F6D1 as u16, 330, game);
    [11138691]
    Shitroid_HandleCutscenePalette => Shitroid_HandleCutscenePalette_Common(addr_word_A9F6D1 as u16, 490, game);
    [11138703]
    Shitroid_HandleCutscenePalette_LowHealth => Shitroid_HandleCutscenePalette_Common(addr_word_A9F711 as u16, 490, game);
    @external {
        11127484 => [()]; 11127960 => [()];
    }
}
pub unsafe fn ShitroidInCutscene_Flashing(k: u16, game: &mut Game) {
    let mut v1: i16 = 3584;
    let E: i32 = k as i32;
    let sice_var_06: u16 = slot!(game, Enemy_ShitroidInCutscene[E].var_06);
    if sice_var_06 != 0 {
        let v4: u16 = (sice_var_06).wrapping_sub(1);
        slot!(game, Enemy_ShitroidInCutscene[E].var_06 = v4);
        if (v4 & 2) != 0 {
            v1 = 0;
        }
    }
    slot!(
        game,
        Enemy_ShitroidInCutscene[E].base.palette_index = v1 as u16
    );
}
pub unsafe fn ShitroidInCutscene_HandleCry(game: &mut Game) {
    let E: i32 = 0;
    if slot!(game, Enemy_ShitroidInCutscene[E].var_14) != 0 {
        slot!(game, Enemy_ShitroidInCutscene[E].var_14 = 0);
        QueueSfx2_Max6(0x72, game);
    }
}
pub unsafe fn ShitroidInCutscene_SpawnOneDustCloudAt(xd: u16, yd: u16, game: &mut Game) {
    let E: i32 = 0x40;
    game.eproj_spawn_pt = Point16U {
        x: ((slot!(game, Enemy_ShitroidInCutscene[E].base.x_pos)).wrapping_add(xd)),
        y: ((slot!(game, Enemy_ShitroidInCutscene[E].base.y_pos)).wrapping_add(yd)),
    };
    SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 9, game);
}
pub unsafe fn ShitroidInCutscene_SpawnThreeDustClouds(game: &mut Game) {
    unsafe {
        ShitroidInCutscene_SpawnOneDustCloudAt(-16i32 as u16, -8i32 as u16, game);
        ShitroidInCutscene_SpawnOneDustCloudAt(0, -16i32 as u16, game);
        ShitroidInCutscene_SpawnOneDustCloudAt(16, -8i32 as u16, game);
    }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallShitroidMoveFunc(ea: u32, k: u16, j: u16, r18: u16, r20: u16, game: &mut Game) => ea;
    call (k, j, r18, r20, game); default { Unreachable(game); };
    @forward [pub unsafe fn ( k: u16, j: u16, r18: u16, r20: u16, game: &mut Game, )];
    Shitroid_Func_GraduallyAccelerateTowards0x400 => Shitroid_GraduallyAccelerateTowardsPt(k, j, r18, r20, 1024, game);
    Shitroid_GraduallyAccelerateTowards0x4 => Shitroid_GraduallyAccelerateTowardsPt(k, j, r18, r20, 4, game);
    [11138143]
    Shitroid_GraduallyAccelerateTowards0x8 => Shitroid_GraduallyAccelerateTowardsPt(k, j, r18, r20, 8, game);
    [11138150]
    Shitroid_GraduallyAccelerateTowards0x10 => Shitroid_GraduallyAccelerateTowardsPt(k, j, r18, r20, 16, game);
}
crate::game::callback_dispatch! {
    pub unsafe fn CallSamusRainbowPaletteFunc(ea: u32, k: u16, game: &mut Game) => ea;
    call (k, game); default { Unreachable(game); };
    @functions [pub unsafe fn (k: u16, game: &mut Game)];
    [11128112] SamusRainbowPaletteFunc_ActivateWhenEnemyLow [] {
        if ((slot!(game, Enemy_ShitroidInCutscene[k as i32].base.y_pos) as i32 + 16
            - word!(game.samus_y_pos) as i32) as i16)
            >= 0
        {
            CallSomeSamusCode(0x16, game);
            slot!(game, Enemy_ShitroidInCutscene[0x40i32].var_16 = (0xa9cd4bi32) as u16);
        }
    };
    @external {
        11128139 => [SamusRainbowPaletteFunc_GraduallySlowDown(game)];
    }
}
pub unsafe fn HandleSamusRainbowPaletteAnimation(k: u16, game: &mut Game) {
    unsafe {
        let r18: u16 = slot!(game, Enemy_ShitroidInCutscene[0x40i32].var_16);
        CallSamusRainbowPaletteFunc(((r18) as u32) | (0xa90000u32), k, game);
    }
}
pub unsafe fn SamusRainbowPaletteFunc_GraduallySlowDown(game: &mut Game) {
    let E: i32 = 0x40;
    let sice_var_1D: u16 = slot!(game, Enemy_ShitroidInCutscene[E].var_1D);
    slot!(
        game,
        Enemy_ShitroidInCutscene[E].var_1D = (sice_var_1D).wrapping_add(768)
    );
    if sice_var_1D >= 0xfd00 {
        let mut v2: u16 = (word!(game.special_samus_palette_frame)).wrapping_add(1);
        if word!(game.special_samus_palette_frame) as i32 - 9i32 & 0x8000 == 0 {
            v2 = 10;
        }
        word!(game.special_samus_palette_frame = v2);
    }
}
pub unsafe fn ShitroidInCutscene_FadeShitroidToBlack(k: u16, game: &mut Game) -> u8 {
    {
        let E: i32 = k as i32;
        if slot!(game, Enemy_ShitroidInCutscene[E].base.y_pos) as i32 - 128i32 & 0x8000 != 0 {
            return 0;
        }
        let mut v2: i16 =
            ((slot!(game, Enemy_ShitroidInCutscene[E].var_0C)) as i16).wrapping_sub(1);
        if v2 >= 0 {
            slot!(game, Enemy_ShitroidInCutscene[E].var_0C = v2 as u16);
            return 0;
        }
        slot!(game, Enemy_ShitroidInCutscene[E].var_0C = 8);
        let v4: u16 = (slot!(game, Enemy_ShitroidInCutscene[E].var_0D)).wrapping_add(1);
        if slot!(game, Enemy_ShitroidInCutscene[E].var_0D) as i32 - 6i32 & 0x8000 == 0 {
            return 1;
        }
        slot!(game, Enemy_ShitroidInCutscene[E].var_0D = v4);
        WriteColorsToPalette(
            0x1e2,
            0xad,
            crate::rtl::rom_table_word!(game, 0xade8e2, v4 as isize),
            0xe,
            game,
        );
        0
    }
}
pub unsafe fn ShitroidInCutscene_HandleEnemyBlinking(k: u16, game: &mut Game) {
    let E: i32 = k as i32;
    if (slot!(game, Enemy_ShitroidInCutscene[E].base.frame_counter) & 1) != 0 {
        slot!(
            game,
            Enemy_ShitroidInCutscene[E].base.properties =
                (slot!(game, Enemy_ShitroidInCutscene[E].base.properties) as i32
                    & !(kEnemyProps_Invisible as i32)) as u16
        );
    } else {
        slot!(
            game,
            Enemy_ShitroidInCutscene[E].base.properties =
                (slot!(game, Enemy_ShitroidInCutscene[E].base.properties))
                    | ((kEnemyProps_Invisible) as u16)
        );
    };
}
pub unsafe fn ShitroidInCutscene_AccelerateDownwards(k: u16, game: &mut Game) {
    let E: i32 = k as i32;
    let mut v2: i16 =
        ((abs16(slot!(game, Enemy_ShitroidInCutscene[E].var_B))) as i16).wrapping_sub(32);
    if v2 < 0 {
        v2 = 0;
    }
    slot!(
        game,
        Enemy_ShitroidInCutscene[E].var_B =
            (if slot!(game, Enemy_ShitroidInCutscene[E].var_B) & 0x8000 != 0 {
                -(v2 as i32)
            } else {
                v2 as i32
            }) as u16
    );
    slot!(
        game,
        Enemy_ShitroidInCutscene[E].var_C =
            (slot!(game, Enemy_ShitroidInCutscene[E].var_C)).wrapping_add(2)
    );
}
pub unsafe fn ShitroidInCutscene_UpdateSpeedAndAngle(
    k: u16,
    mut r18: u16,
    r20: u16,
    r22: u16,
    game: &mut Game,
) {
    unsafe {
        let E: i32 = k as i32;
        crate::game::sm_a6::steer_enemy_speed_and_angle!(
            game,
            Enemy_ShitroidInCutscene,
            r22,
            E,
            r18,
            r20
        );
    }
}
crate::instruction_callbacks! { pub unsafe fn (_k: u16, _jp: EnemyScript) -> EnemyScript;
    Shitroid_Instr_1 [EnemyScript::jump(addr_kShitroid_Ilist_CFA2 as u16)] { };
    Shitroid_Instr_2 [EnemyScript::jump(addr_kShitroid_Ilist_CFB8 as u16)] { };
}
pub unsafe fn MotherBrain_Pal_ProcessInvincibility(game: &mut Game) {
    if slot!(game, Enemy_MotherBrain[0i32].var_00) == 4 {
        let flash_timer: u16 = slot!(game, Enemy_MotherBrain[0x40i32].base.flash_timer);
        if flash_timer as i32 >> 1 != 0 && flash_timer & 1 == 0 {
            for i in (0i32..=28).rev().step_by(2) {
                let v2: i32 = i >> 1;
                let v3: u16 = word!(game.palette_buffer[(v2 + 129)]);
                word! {game; palette_buffer[(v2 + 145)] = v3; palette_buffer[(v2 + 177)] = v3;
                palette_buffer[(v2 + 65)] = v3; }
            }
        }
    }
}
pub unsafe fn MotherBrain_Pal_HandleRoomPal(game: &mut Game) {
    unsafe {
        let j: u16 = HandleMotherBrainInstructionList(0x781c, game);
        if j != 0 {
            MotherBrain_Pal_WriteRoomPal(j, game);
        }
    }
}
pub unsafe fn MotherBrain_Pal_WriteRoomPal(j: u16, game: &mut Game) {
    {
        WriteColorsToPalette(0x68, 0xa9, j, 12, game);
        WriteColorsToPalette(0xa6, 0xa9, (j).wrapping_add(24), 12, game);
        WriteColorsToPalette(0xe6, 0xa9, (j).wrapping_add(24), 12, game);
    }
}
pub unsafe fn HandleMotherBrainInstructionList(a: u16, game: &mut Game) -> u16 {
    unsafe {
        let st: usize = a as usize;
        if native_word(st, game) & 0x8000 == 0 {
            return 0;
        }
        let mut v2: u16 = native_word(st, game);
        let mut v3 = native_record(0xa90000 | u32::from(v2), game);
        let mut execute_instruction: bool = (v3.word(0) & 0x8000 != 0)
            || ({
                if native_word(st + 2, game) != v3.word(0) {
                    set_native_word(st + 2, native_word(st + 2, game).wrapping_add(1), game);
                    return v3.word(2);
                }
                v2 = v2.wrapping_add(4);
                false
            });
        loop {
            if execute_instruction {
                v2 = CallMotherBrainInstr(
                    u32::from(v3.word(0)) | 0xa90000,
                    v2.wrapping_add(2),
                    game,
                );
                execute_instruction = false;
            } else {
                v3 = native_record(0xa90000 | u32::from(v2), game);
                if v3.word(0) & 0x8000 == 0 {
                    break;
                } else {
                    execute_instruction = true;
                }
            }
        }
        if v3.word(0) == 0 {
            set_native_word(st, 0, game);
            set_native_word(st + 2, 0, game);
            0
        } else {
            set_native_word(st + 2, 1, game);
            set_native_word(st, v2, game);
            v3.word(2)
        }
    }
}
pub unsafe fn MotherBrain_SetupBrainNormalPal(game: &mut Game) {
    slot!(game, Enemy_MotherBrain[0i32].var_E = 10);
}
pub unsafe fn MotherBrain_SetupBrainPalForLaser(game: &mut Game) {
    slot!(game, Enemy_MotherBrain[0i32].var_E = 514);
}
crate::game::callback_dispatch! {
    pub unsafe fn CallDeadTorizoFuncA(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [11129800] DeadTorizo_PreRotDelay [] {
        let E: i32 = 0;
        let v1: u16 = (slot!(game, Enemy_DeadTorizo[E].var_04)).wrapping_add(1);
        slot!(game, Enemy_DeadTorizo[E].var_04 = v1);
        if v1 >= 0x10 {
            slot!(game, Enemy_DeadTorizo[E].base.properties = (slot!(game, Enemy_DeadTorizo[E].base.properties)) | ((kEnemyProps_Tangible) as u16));
            slot!(game, Enemy_DeadTorizo[E].var_A = (0xa9d3e6i32) as u16);
            DeadTorizo_Rotting(game);
        }
    };
    [11129830] DeadTorizo_Rotting [] {
        let E: i32 = 0;
        let v1: u16 = (slot!(game, Enemy_DeadTorizo[E].var_03)).wrapping_add(1);
        slot!(game, Enemy_DeadTorizo[E].var_03 = v1);
        if v1 >= 0xf {
            slot!(game, Enemy_DeadTorizo[E].var_03 = 0);
            let dto_var_02: u16 = slot!(game, Enemy_DeadTorizo[E].var_02);
            if dto_var_02 != 0 {
                DeadTorizo_CopyLineOfSandHeapTileData(dto_var_02, game);
                slot!(game, Enemy_DeadTorizo[E].var_02 = slot!(game, Enemy_DeadTorizo[E].var_02).wrapping_sub(1));
            }
        }
        slot!(game, Enemy_DeadTorizo[E].var_C = slot!(game, Enemy_DeadTorizo[E].var_C).wrapping_add(1));
        MoveEnemyWithVelocity(game);
        if ProcessCorpseRotting(0, game) & 1 == 0 {
            slot!(game, Enemy_DeadTorizo[E].var_A = (0xa9d3c7i32) as u16);
        }
    };
    @external {
        11129773 => [{
                    DeadTorizo_WaitForSamusColl(word!(game.cur_enemy_index), game);
                }];
        11129799 => [()];
    }
}
pub unsafe fn DeadTorizo_MainGfxHook(game: &mut Game) {
    unsafe {
        MotherBrain_AddSpritemapToOam(addr_kDeadTorizo_Sprmap_D761 as u16, 296, 187, 0, game);
    }
}
pub unsafe fn DeadTorizo_WaitForSamusColl(k: u16, game: &mut Game) {
    if k == word!(game.enemy_index_colliding_dirs[0])
        || k == word!(game.enemy_index_colliding_dirs[1])
        || k == word!(game.enemy_index_colliding_dirs[2])
        || k == word!(game.enemy_index_colliding_dirs[3])
    {
        slot!(game, Enemy_DeadTorizo[0i32].var_A = (0xa9d3c8i32) as u16);
    }
}
macro_rules! queue_dead_torizo_tile_transfers {
    ($game:ident, $transfer_table:literal, $entry:ident, $tail:ident) => {
        let mut v8: u16 = 0;
        let mut v9: u16 = crate::rtl::rom_table_word!($game, $transfer_table, 0);
        loop {
            $entry = usize::try_from(0xd0i32 + $tail as i32).expect("VRAM entry outside RAM");
            {
                let value: u16 = v9;
                let at: usize = $entry;
                let bytes = value.to_ne_bytes();
                $game.ram().write_byte(at, bytes[0]);
                $game.ram().write_byte(at + 1, bytes[1]);
            };
            let v11: i32 = v8 as i32 >> 1;
            {
                let value: u16 =
                    crate::rtl::rom_table_word!($game, $transfer_table, (v11 + 1) as isize);
                let at: usize = $entry + 3;
                let bytes = value.to_ne_bytes();
                $game.ram().write_byte(at, bytes[0]);
                $game.ram().write_byte(at + 1, bytes[1]);
            };
            {
                let value: u16 =
                    crate::rtl::rom_table_word!($game, $transfer_table, (v11 + 2) as isize);
                let at: usize = $entry + 2;
                let bytes = value.to_ne_bytes();
                $game.ram().write_byte(at, bytes[0]);
                $game.ram().write_byte(at + 1, bytes[1]);
            };
            {
                let value: u16 =
                    crate::rtl::rom_table_word!($game, $transfer_table, (v11 + 3) as isize);
                let at: usize = $entry + 5;
                let bytes = value.to_ne_bytes();
                $game.ram().write_byte(at, bytes[0]);
                $game.ram().write_byte(at + 1, bytes[1]);
            };
            $tail = ($tail).wrapping_add(7);
            v8 = (v8).wrapping_add(8);
            v9 = crate::rtl::rom_table_word!($game, $transfer_table, (v8 >> 1) as isize);
            if (v9 == 0) {
                break;
            }
        }
    };
}

pub unsafe fn DeadTorizo_Func_1(game: &mut Game) {
    let mut v5 = 0usize;
    let mut v10 = 0usize;
    let E: i32 = 0;
    let v1: u16 = (slot!(game, Enemy_DeadTorizo[E].var_00)).wrapping_add(1);
    slot!(game, Enemy_DeadTorizo[E].var_00 = v1);
    let mut v3: u16 = word!(game.vram_write_queue_tail);
    if v1 & 1 != 0 {
        queue_dead_torizo_tile_transfers!(game, 0xa9d583, v10, v3);
    } else {
        queue_dead_torizo_tile_transfers!(game, 0xa9d549, v5, v3);
    }
    slot!(game, Enemy_DeadEnemy[0i32].var_22 = 0);
    word!(game.vram_write_queue_tail = v3);
}
pub unsafe fn DeadTorizo_CopyLineOfSandHeapTileData(a: u16, game: &mut Game) {
    let v1: u16 = crate::rtl::rom_table_word!(game, 0xa9d67c, a as isize);
    let v2: i32 = crate::rtl::rom_table_word!(game, 0xa9d69c, a as isize) as i32 >> 1;
    {
        let value: u16 = crate::rtl::rom_table_word!(game, 0xb7a800, v2 as isize);
        let at: usize = 0x9500 + (v1 as i32 as isize) as usize;
        let bytes = value.to_ne_bytes();
        game.ram().write_byte(at, bytes[0]);
        game.ram().write_byte(at + 1, bytes[1]);
    };
    {
        let value: u16 = crate::rtl::rom_table_word!(game, 0xb7a800, (v2 + 8) as isize);
        let at: usize = 0x9510 + (v1 as i32 as isize) as usize;
        let bytes = value.to_ne_bytes();
        game.ram().write_byte(at, bytes[0]);
        game.ram().write_byte(at + 1, bytes[1]);
    };
    {
        let value: u16 = crate::rtl::rom_table_word!(game, 0xb7a800, (v2 + 16) as isize);
        let at: usize = 0x9520 + (v1 as i32 as isize) as usize;
        let bytes = value.to_ne_bytes();
        game.ram().write_byte(at, bytes[0]);
        game.ram().write_byte(at + 1, bytes[1]);
    };
    {
        let value: u16 = crate::rtl::rom_table_word!(game, 0xb7a800, (v2 + 24) as isize);
        let at: usize = 0x9530 + (v1 as i32 as isize) as usize;
        let bytes = value.to_ne_bytes();
        game.ram().write_byte(at, bytes[0]);
        game.ram().write_byte(at + 1, bytes[1]);
    };
    {
        let value: u16 = crate::rtl::rom_table_word!(game, 0xb7a800, (v2 + 32) as isize);
        let at: usize = 0x9540 + (v1 as i32 as isize) as usize;
        let bytes = value.to_ne_bytes();
        game.ram().write_byte(at, bytes[0]);
        game.ram().write_byte(at + 1, bytes[1]);
    };
    {
        let value: u16 = crate::rtl::rom_table_word!(game, 0xb7a800, (v2 + 40) as isize);
        let at: usize = 0x9550 + (v1 as i32 as isize) as usize;
        let bytes = value.to_ne_bytes();
        game.ram().write_byte(at, bytes[0]);
        game.ram().write_byte(at + 1, bytes[1]);
    };
    {
        let value: u16 = crate::rtl::rom_table_word!(game, 0xb7a800, (v2 + 48) as isize);
        let at: usize = 0x9560 + (v1 as i32 as isize) as usize;
        let bytes = value.to_ne_bytes();
        game.ram().write_byte(at, bytes[0]);
        game.ram().write_byte(at + 1, bytes[1]);
    };
    {
        let value: u16 = crate::rtl::rom_table_word!(game, 0xb7a800, (v2 + 56) as isize);
        let at: usize = 0x9570 + (v1 as i32 as isize) as usize;
        let bytes = value.to_ne_bytes();
        game.ram().write_byte(at, bytes[0]);
        game.ram().write_byte(at + 1, bytes[1]);
    };
    {
        let value: u16 = crate::rtl::rom_table_word!(game, 0xb7a800, (v2 + 64) as isize);
        let at: usize = 0x9580 + (v1 as i32 as isize) as usize;
        let bytes = value.to_ne_bytes();
        game.ram().write_byte(at, bytes[0]);
        game.ram().write_byte(at + 1, bytes[1]);
    };
    {
        let value: u16 = crate::rtl::rom_table_word!(game, 0xb7a800, (v2 + 72) as isize);
        let at: usize = 0x9590 + (v1 as i32 as isize) as usize;
        let bytes = value.to_ne_bytes();
        game.ram().write_byte(at, bytes[0]);
        game.ram().write_byte(at + 1, bytes[1]);
    };
    {
        let value: u16 = crate::rtl::rom_table_word!(game, 0xb7a800, (v2 + 80) as isize);
        let at: usize = 0x95a0 + (v1 as i32 as isize) as usize;
        let bytes = value.to_ne_bytes();
        game.ram().write_byte(at, bytes[0]);
        game.ram().write_byte(at + 1, bytes[1]);
    };
    {
        let value: u16 = crate::rtl::rom_table_word!(game, 0xb7a800, (v2 + 88) as isize);
        let at: usize = 0x95b0 + (v1 as i32 as isize) as usize;
        let bytes = value.to_ne_bytes();
        game.ram().write_byte(at, bytes[0]);
        game.ram().write_byte(at + 1, bytes[1]);
    };
    {
        let value: u16 = crate::rtl::rom_table_word!(game, 0xb7a800, (v2 + 96) as isize);
        let at: usize = 0x95c0 + (v1 as i32 as isize) as usize;
        let bytes = value.to_ne_bytes();
        game.ram().write_byte(at, bytes[0]);
        game.ram().write_byte(at + 1, bytes[1]);
    };
    {
        let value: u16 = crate::rtl::rom_table_word!(game, 0xb7a800, (v2 + 104) as isize);
        let at: usize = 0x95d0 + (v1 as i32 as isize) as usize;
        let bytes = value.to_ne_bytes();
        game.ram().write_byte(at, bytes[0]);
        game.ram().write_byte(at + 1, bytes[1]);
    };
    {
        let value: u16 = crate::rtl::rom_table_word!(game, 0xb7a800, (v2 + 112) as isize);
        let at: usize = 0x95e0 + (v1 as i32 as isize) as usize;
        let bytes = value.to_ne_bytes();
        game.ram().write_byte(at, bytes[0]);
        game.ram().write_byte(at + 1, bytes[1]);
    };
    {
        let value: u16 = crate::rtl::rom_table_word!(game, 0xb7a800, (v2 + 120) as isize);
        let at: usize = 0x95f0 + (v1 as i32 as isize) as usize;
        let bytes = value.to_ne_bytes();
        game.ram().write_byte(at, bytes[0]);
        game.ram().write_byte(at + 1, bytes[1]);
    };
    {
        let value: u16 = crate::rtl::rom_table_word!(game, 0xb7a800, (v2 + 128) as isize);
        let at: usize = 0x9600 + (v1 as i32 as isize) as usize;
        let bytes = value.to_ne_bytes();
        game.ram().write_byte(at, bytes[0]);
        game.ram().write_byte(at + 1, bytes[1]);
    };
    {
        let value: u16 = crate::rtl::rom_table_word!(game, 0xb7a800, (v2 + 136) as isize);
        let at: usize = 0x9610 + (v1 as i32 as isize) as usize;
        let bytes = value.to_ne_bytes();
        game.ram().write_byte(at, bytes[0]);
        game.ram().write_byte(at + 1, bytes[1]);
    };
}
pub fn DeadSidehopper_Init_0(game: &mut Game) {
    let E: i32 = word!(game.cur_enemy_index) as i32;
    slot!(
        game,
        Enemy_DeadSidehopper[E].base.properties =
            (slot!(game, Enemy_DeadSidehopper[E].base.properties) as i32 & 0x77ff
                | kEnemyProps_ProcessedOffscreen as i32) as u16
    );
    if (slot!(game, Enemy_DeadSidehopper[0i32].base.properties) as i32
        & kEnemyProps_Invisible as i32)
        != 0
    {
        slot!(
            game,
            Enemy_DeadSidehopper[E].base.properties =
                (slot!(game, Enemy_DeadSidehopper[E].base.properties))
                    | ((kEnemyProps_Deleted) as u16)
        );
    }
    slot!(game, Enemy_DeadSidehopper[E].var_08 = 0);
    slot!(game, Enemy_DeadSidehopper[E].var_0A = 96);
    slot!(game, Enemy_DeadSidehopper[E].var_0B = 256);
    slot!(game, Enemy_DeadSidehopper[E].base.x_pos = 488);
    slot!(game, Enemy_DeadSidehopper[E].base.y_pos = 184);
    slot!(game, Enemy_DeadSidehopper[E].var_A = (0xa9d8e2i32) as u16);
    slot!(game, Enemy_DeadSidehopper[E].base.palette_index = 512);
    slot!(game, Enemy_DeadSidehopper[E].base.y_height = 21);
    Enemy_SetInstrList(
        word!(game.cur_enemy_index),
        addr_kDeadMonsters_Ilist_ECE3 as u16,
        game,
    );
    InitializeEnemyCorpseRotting(word!(game.cur_enemy_index), addr_stru_A9DD68 as u16, game);
}
pub fn DeadSidehopper_Init_1(game: &mut Game) {
    let E: i32 = word!(game.cur_enemy_index) as i32;
    slot!(game, Enemy_DeadSidehopper[E].var_08 = -1i32 as u16);
    slot!(game, Enemy_DeadSidehopper[E].var_A = (0xa9da64i32) as u16);
    slot!(game, Enemy_DeadSidehopper[E].base.palette_index = 3584);
    Enemy_SetInstrList(
        word!(game.cur_enemy_index),
        addr_kDeadMonsters_Ilist_ECEF as u16,
        game,
    );
    InitializeEnemyCorpseRotting(word!(game.cur_enemy_index), addr_stru_A9DD78 as u16, game);
}

macro_rules! initialize_rotting_enemy_corpse {
    (
        $enemy:ident, $state_address:literal, $game:ident, $instruction_table:literal,
        $corpse_table:literal $(,)?
    ) => {
        slot!($game, EnemyData[$enemy].palette_index = 3584);
        slot!(
            $game,
            EnemyData[$enemy].ai_var_A = (($state_address) as u16)
        );
        let v2: i32 = slot!($game, EnemyData[$enemy].parameter_1) as i32 >> 1;
        Enemy_SetInstrList(
            word!($game.cur_enemy_index),
            crate::rtl::rom_table_word!($game, $instruction_table, v2 as isize),
            $game,
        );
        InitializeEnemyCorpseRotting(
            word!($game.cur_enemy_index),
            crate::rtl::rom_table_word!($game, $corpse_table, v2 as isize),
            $game,
        );
    };
}

crate::game::callback_dispatch! {
    pub unsafe fn CallDeadSidehopperFunc(ea: u32, k: u16, game: &mut Game) => ea;
    call (k, game); default { Unreachable(game); };
    @functions [pub unsafe fn (k: u16, game: &mut Game)];
    [11131106] DeadSidehopper_Alive_WaitForActivate [] {
        if word!(game.layer1_x_pos) as i32 - 513i32 & 0x8000 != 0 {
            slot!(game, Enemy_DeadSidehopper[k as i32].var_A = (0xa9d8f1i32) as u16);
            DeadSidehopper_Activated(k, game);
        }
    };
    [11131121] DeadSidehopper_Activated [] {
        if DeadMonsters_Func_3(k, game) & 1 != 0 {
            let E: i32 = k as i32;
            slot!(game, Enemy_DeadSidehopper[E].var_06 = (slot!(game, Enemy_DeadSidehopper[E].var_06) as i32 + 1 & 3) as u16);
            Enemy_SetInstrList(k, addr_kDeadMonsters_Ilist_ECAC as u16, game);
            slot!(game, Enemy_DeadSidehopper[E].var_A = (0xa9d90fi32) as u16);
        }
    };
    [11131152] DeadMonsters_Func_1 [] {
        let E: i32 = k as i32;
        slot!(game, Enemy_DeadMonsters[E].var_A = (0xa9d91di32) as u16);
        slot!(game, Enemy_DeadMonsters[E].var_F = 64);
    };
    [11131165] DeadMonsters_Func_2 [] {
        let E: i32 = k as i32;
        slot!(game, Enemy_DeadMonsters[E].var_F = slot!(game, Enemy_DeadMonsters[E].var_F).wrapping_sub(1));
        if (slot!(game, Enemy_DeadMonsters[E].var_F) & 0x8000) != 0 {
            if slot!(game, Enemy_DeadMonsters[E].var_08) != 0 {
                slot!(game, Enemy_DeadMonsters[E].var_A = (0xa9da08i32) as u16);
            } else {
                slot!(game, Enemy_DeadMonsters[E].var_A = (0xa9d8f1i32) as u16);
                Enemy_SetInstrList(k, addr_kDeadMonsters_Ilist_ECE3 as u16, game);
                let v3: i32 = slot!(game, Enemy_DeadMonsters[E].var_06) as i32;
                slot!(game, Enemy_DeadMonsters[E].var_0B = crate::rtl::rom_table_word!(game, 0xa9d951, v3 as isize));
                slot!(game, Enemy_DeadMonsters[E].var_0A = crate::rtl::rom_table_word!(game, 0xa9d959, v3 as isize));
            }
        }
    };
    [11131400] DeadMonsters_Func_5 [] {
        let E: i32 = k as i32;
        let v2: u16 = (slot!(game, Enemy_DeadMonsters[E].var_07)).wrapping_add(1);
        slot!(game, Enemy_DeadMonsters[E].var_07 = v2);
        if v2 >= 8 {
            slot!(game, Enemy_DeadMonsters[E].var_07 = 0);
            WriteColorsToPalette(
                0x122,
                0xa9,
                (32 * (slot!(game, Enemy_DeadMonsters[E].var_08) as i32 - 1i32) - 0x1434i32) as u16,
                0xf,
                game,
            );
            let v3: u16 = word!(game.cur_enemy_index);
            let v5: u16 = (slot!(game, Enemy_DeadMonsters[E].var_08)).wrapping_add(1);
            slot!(game, Enemy_DeadMonsters[E].var_08 = v5);
            if v5 >= 8 {
                Enemy_SetInstrList(v3, addr_kDeadMonsters_Ilist_ECE9 as u16, game);
                slot!(game, Enemy_DeadMonsters[E].var_A = (0xa9da64i32) as u16);
                slot!(game, Enemy_DeadMonsters[E].base.properties = (slot!(game, Enemy_DeadMonsters[E].base.properties)) | (0x8000u16));
                slot!(game, Enemy_DeadMonsters[E].base.y_height = 12);
            }
        }
    };
    @functions [pub fn (k: u16, game: &mut Game)];
    [11131578] DeadSidehopper_Rotting [] {
        let v1: u8 = ProcessCorpseRotting(k, game);
        if v1 & 1 == 0 {
            slot!(game, Enemy_DeadMonsters[word!(game.cur_enemy_index) as i32].var_A = (0xa9da64i32) as u16);
        }
        let dms_var_53: u16 = slot!(game, Enemy_DeadMonsters[word!(game.cur_enemy_index) as i32].var_53);
        ProcessCorpseRottingVramTransfers(dms_var_53, game);
    };
    [11131600] DeadZoomer_Rotting [] {
        let v1: u8 = ProcessCorpseRotting(k, game);
        if v1 & 1 == 0 {
            slot!(game, Enemy_DeadMonsters[word!(game.cur_enemy_index) as i32].var_A = (0xa9da63i32) as u16);
        }
        let dms_var_53: u16 = slot!(game, Enemy_DeadMonsters[word!(game.cur_enemy_index) as i32].var_53);
        ProcessCorpseRottingVramTransfers(dms_var_53, game);
    };
    @forward [pub unsafe fn (k: u16, game: &mut Game)];
    [11131492]
    DeadSidehopper_WaitForSamusColl => DeadMonsters_WaitForSamusColl(k, (0xa9da8fi32) as u16, game);
    [11131497]
    DeadZoomer_WaitForSamusColl => DeadMonsters_WaitForSamusColl(k, (0xa9da94i32) as u16, game);
    [11131502]
    DeadSkree_WaitForSamusColl => DeadMonsters_WaitForSamusColl(k, (0xa9da9ei32) as u16, game);
    [11131507]
    DeadRipper_WaitForSamusColl => DeadMonsters_WaitForSamusColl(k, (0xa9da99i32) as u16, game);
    @forward [pub unsafe fn (k: u16, game: &mut Game)];
    [11131535]
    DeadSidehopper_PreRotDelay => DeadMonsters_PreRotDelay_Common(k, (0xa9dabai32) as u16, game);
    [11131540]
    DeadZoomer_PreRotDelay => DeadMonsters_PreRotDelay_Common(k, (0xa9dad0i32) as u16, game);
    [11131545]
    DeadRipper_PreRotDelay => DeadMonsters_PreRotDelay_Common(k, (0xa9dae6i32) as u16, game);
    [11131550]
    DeadSkree_PreRotDelay => DeadMonsters_PreRotDelay_Common(k, (0xa9dafci32) as u16, game);
    @external {
        11131644 => DeadSkree_Rotting; 11131622 => DeadRipper_Rotting;
        11131151 => [()]; 11131491 => [()];
    }
}
pub unsafe fn DeadMonsters_Func_3(k: u16, game: &mut Game) -> u8 {
    unsafe {
        let E: i32 = k as i32;
        DeadMonsters_Func_4(k, slot!(game, Enemy_DeadMonsters[E].var_0A), game);
        let mut v2: u16 = 32;
        if (slot!(game, Enemy_DeadMonsters[E].var_0B) & 0x8000) == 0 {
            v2 = 128;
        }
        let mut v3: u16 = (slot!(game, Enemy_DeadMonsters[E].var_0B)).wrapping_add(v2);
        slot!(game, Enemy_DeadMonsters[E].var_0B = v3);
        if slot!(game, Enemy_DeadMonsters[E].base.x_pos) as i32 - 544i32 & 0x8000 != 0 {
            let full: i32 = slot!(byte game, Enemy_DeadMonsters[E].base.y_subpos[1]) as i32
                + byte!(v3; 0) as i32;
            slot!(byte game, Enemy_DeadMonsters[E].base.y_subpos[1] = full as u8);
            let v12: u16 = (slot!(game, Enemy_DeadMonsters[E].base.y_pos) as i32
                + (v3 >> 8) as i8 as i32
                + (full >> 8)) as u16;
            slot!(game, Enemy_DeadMonsters[E].base.y_pos = v12);
            ((v12 as i32 - 184i32 & 0x8000) == 0) as u8
        } else {
            {
                let (actor, amount) = (k, (v3 as i16 as i32) << 8);
                (actor_api::move_vertical(&mut Runtime::new(game), Key::from_legacy(actor), amount)
                    == actor_api::MovementResult::Blocked) as u8
            }
        }
    }
}
pub unsafe fn DeadMonsters_Func_4(k: u16, mut a: u16, game: &mut Game) {
    unsafe {
        let E: i32 = k as i32;
        if slot!(game, Enemy_DeadMonsters[E].base.x_pos) as i32 - 544i32 & 0x8000 != 0 {
            let full: i32 = slot!(byte game, Enemy_DeadMonsters[E].base.x_subpos[1]) as i32
                + byte!(a; 0) as i32;
            slot!(byte game, Enemy_DeadMonsters[E].base.x_subpos[1] = full as u8);
            slot!(
                game,
                Enemy_DeadMonsters[E].base.x_pos =
                    (slot!(game, Enemy_DeadMonsters[E].base.x_pos) as i32
                        + ((a >> 8) as i8 as i32 + (full >> 8))) as u16
            );
        } else {
            {
                let (actor, amount) = (k, (a as i16 as i32) << 8);
                (actor_api::move_horizontal(
                    &mut Runtime::new(game),
                    Key::from_legacy(actor),
                    amount,
                    actor_api::SlopePolicy::Ignore,
                ) == actor_api::MovementResult::Blocked) as u8
            };
        };
    }
}
pub unsafe fn DeadMonsters_WaitForSamusColl(k: u16, j: u16, game: &mut Game) {
    if k == word!(game.enemy_index_colliding_dirs[0])
        || k == word!(game.enemy_index_colliding_dirs[1])
        || k == word!(game.enemy_index_colliding_dirs[2])
        || k == word!(game.enemy_index_colliding_dirs[3])
    {
        slot!(game, Enemy_DeadMonsters[k as i32].var_A = j);
    }
}
pub unsafe fn DeadMonsters_PreRotDelay_Common(k: u16, j: u16, game: &mut Game) {
    let E: i32 = k as i32;
    slot!(
        game,
        Enemy_DeadMonsters[E].var_B = slot!(game, Enemy_DeadMonsters[E].var_B).wrapping_add(1)
    );
    if slot!(game, Enemy_DeadMonsters[E].var_B) >= 0x10 {
        slot!(game, Enemy_DeadMonsters[E].var_A = j);
        slot!(
            game,
            Enemy_DeadMonsters[E].base.properties =
                (slot!(game, Enemy_DeadMonsters[E].base.properties))
                    | ((kEnemyProps_Tangible) as u16)
        );
    }
}
pub use self::DeadZoomer_Rotting as DeadRipper_Rotting;
pub use self::DeadZoomer_Rotting as DeadSkree_Rotting;
pub fn ProcessCorpseRotting(k: u16, game: &mut Game) -> u8 {
    let EK: i32 = k as i32;
    let mut E0: i32 = 0;
    slot! { game, Enemy_DeadMonsters[E0]; var_45 = slot!(game, Enemy_DeadMonsters[EK].var_57), var_46 = slot!(game, Enemy_DeadMonsters[EK].var_58), var_47 = slot!(game, Enemy_DeadMonsters[EK].var_59), var_48 = slot!(game, Enemy_DeadMonsters[EK].var_5A), var_49 = slot!(game, Enemy_DeadMonsters[EK].var_5B), var_44 = slot!(game, Enemy_DeadMonsters[EK].var_56), var_42 = slot!(game, Enemy_DeadMonsters[EK].var_54), var_43 = slot!(game, Enemy_DeadMonsters[EK].var_55) };
    let mut dms_var_52: u16 = slot!(game, Enemy_DeadMonsters[EK].var_52);
    let mut v4: u16 = 0;
    loop {
        let mut v15: u16 = v4;
        let v5: usize = dms_var_52 as usize;
        let v6 = v5;
        if (native_word(v5, game) as i16) >= 0 {
            let mut v7: i16 = native_word(v5 + 2, game) as i16;
            if v7 != 0 {
                let mut v8: i16 = (v7).wrapping_sub(1);
                set_native_word(v5 + 2, v8 as u16, game);
                if (v8 as u16) < 4 {
                    E0 = 0;
                    let mut r18: u16 = slot!(game, Enemy_DeadMonsters[E0].var_42);
                    if v15 >= slot!(game, Enemy_DeadMonsters[E0].var_47) {
                        r18 = slot!(game, Enemy_DeadMonsters[E0].var_43);
                    }
                    CopyMoveCorpseRottingRotEntry(
                        native_record(
                            0xa90000 | u32::from(slot!(game, Enemy_DeadMonsters[E0].var_44)),
                            game,
                        ),
                        native_word(v6, game),
                        r18,
                        game,
                    );
                }
            } else {
                let mut E: i32 = 0;
                let mut r18_0: u16 = slot!(game, Enemy_DeadMonsters[E].var_43);
                let v11: usize = dms_var_52 as usize;
                CopyMoveCorpseRottingRotEntry(
                    native_record(
                        0xa90000 | u32::from(slot!(game, Enemy_DeadMonsters[E].var_44)),
                        game,
                    ),
                    native_word(v11, game),
                    r18_0,
                    game,
                );
                let mut v12: u16 = (native_word(v11, game)).wrapping_add(2);
                if v12 < slot!(game, Enemy_DeadMonsters[E].var_46) {
                    set_native_word(v11, v12, game);
                } else {
                    CallCorpseRottingFinish(
                        ((slot!(game, Enemy_DeadMonsters[E].var_49)) as u32) | (0xa90000u32),
                        game,
                    );
                    if v15 >= slot!(game, Enemy_DeadMonsters[0i32].var_46) {
                        break;
                    }
                    set_native_word(v11, -1i32 as u16, game);
                }
            }
        }
        dms_var_52 = (dms_var_52).wrapping_add(4);
        v4 = (v15).wrapping_add(1);
        E0 = 0;
        if ((v15 as i32 + 1 - slot!(game, Enemy_DeadMonsters[E0].var_45) as i32) as i16) >= 0 {
            return ((v15).wrapping_add(1) >= slot!(game, Enemy_DeadMonsters[E0].var_45)) as u8;
        }
    }
    0
}
pub(crate) fn CopyMoveCorpseRottingRotEntry(r20: RomRecord, a: u16, r18: u16, game: &mut Game) {
    let E: i32 = 0;
    slot!(game, Enemy_DeadMonsters[E].var_41 = a);
    let v1: u16 = ((a) & (0xfff8u16)) >> 2;
    let v2: u16 = (a) & (7u16);
    let v6: u16 = (r20.table_word((v1 >> 1) as isize) as i32 + 2 * v2 as i32) as u16;
    if v2 >= 6 {
        CallCorpseRottingMove(
            ((r18) as u32) | (0xa90000u32),
            (slot!(game, Enemy_DeadMonsters[E].var_48)).wrapping_add(v6),
            v6,
            game,
        );
    } else {
        CallCorpseRottingMove(((r18) as u32) | (0xa90000u32), v6, v6, game);
    };
}
pub fn InitializeCorpseRottingDataTable(mut table: usize, a: u16, game: &mut Game) {
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
pub fn InitializeEnemyCorpseRotting(k: u16, j: u16, game: &mut Game) {
    let v2 = native_record(0xa90000 | u32::from(j), game);
    let E: i32 = k as i32;
    slot! { game, Enemy_DeadMonsters[E]; var_52 = v2.word(0), var_53 = v2.word(2), var_54 = v2.word(4), var_55 = v2.word(6), var_5B = v2.word(14) };
    let v4: u16 = v2.word(12);
    slot!(game, Enemy_DeadMonsters[E].var_56 = v4);
    slot!(
        game,
        Enemy_DeadMonsters[E].var_5A = native_record(0xa90000 | u32::from(v4), game)
            .word(2)
            .wrapping_sub(12)
    );
    let v5 = v2;
    let mut v6: u16 = v5.word(8);
    crate::update_place! { previous fresh5 = v6 => v6.wrapping_sub(1); }
    slot!(game, Enemy_DeadMonsters[E].var_57 = fresh5);
    slot!(game, Enemy_DeadMonsters[E].var_58 = v6);
    slot!(game, Enemy_DeadMonsters[E].var_59 = (v6).wrapping_sub(1));
    InitializeCorpseRottingDataTable(v5.word(0) as usize, v5.word(8), game);
    CallCorpseRottingInit(((v5.word(10)) as u32) | (0xa90000u32), game);
}
pub fn ProcessCorpseRottingVramTransfers(k: u16, game: &mut Game) {
    let p = native_record(0xa90000 | u32::from(k), game);
    let mut offset = 0;
    let mut v1: u16 = word!(game.vram_write_queue_tail);
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
    slot!(game, Enemy_DeadEnemy[0i32].var_22 = 0);
    word!(game.vram_write_queue_tail = v1);
}
pub unsafe fn DeadSidehopper_DD31(game: &mut Game) {
    unsafe {
        DeadSidehopper_DD34((0xa9dabai32) as u16, game);
    }
}
pub unsafe fn DeadSidehopper_DD34(a: u16, game: &mut Game) {
    let E: i32 = word!(game.cur_enemy_index) as i32;
    slot!(game, Enemy_DeadSidehopper[E].var_A = a);
    slot!(
        game,
        Enemy_DeadSidehopper[E].base.properties =
            (slot!(game, Enemy_DeadSidehopper[E].base.properties) as i32
                | (kEnemyProps_ProcessedOffscreen as i32 | kEnemyProps_Tangible as i32))
                as u16
    );
}
fn initialize_corpse_tiles(
    rom_address: u16,
    tile_base: usize,
    segments: &[(usize, usize, i32)],
    game: &mut Game,
) {
    let source = native_record(0xb70000 | u32::from(rom_address), game);
    for &(destination_offset, source_offset, length) in segments {
        // Dispatch descriptors contain positive spans of at most 320 bytes.
        let mut bytes = [0; 320];
        for (offset, byte) in bytes[..length as usize].iter_mut().enumerate() {
            *byte = source.byte(source_offset + offset);
        }
        game.ram()
            .copy_in(tile_base + destination_offset, &bytes[..length as usize]);
    }
}

// Each descriptor lists row gates in transfer order. A move copies each row
// before clearing its source; a copy leaves the source intact.
fn transfer_corpse_tiles(
    j: u16,
    k: u16,
    tile_base: usize,
    copy_limit: u16,
    row_gates: &[u16],
    clear_source: bool,
    byte_indices: bool,
    game: &mut Game,
) {
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
    let progress_offset = 0xf78 + core::mem::offset_of!(Enemy_DeadMonsters, var_41);
    for (row, &minimum) in row_gates.iter().enumerate() {
        let progress = || crate::ram_map::checked_word!(game, progress_offset;);
        if minimum != 0 && progress() < minimum {
            continue;
        }
        let row_base = tile_base + row * 32;
        // Preserve the original bit-15 subtraction test, including high values.
        if (progress() as i32 - copy_limit as i32) & 0x8000 != 0 {
            for plane in [0, 16] {
                let mut bytes = [0; 2];
                game.ram().copy_out(row_base + source + plane, &mut bytes);
                game.ram()
                    .copy_in(row_base + destination + plane + 2, &bytes);
            }
        }
        if clear_source {
            for plane in [0, 16] {
                game.ram()
                    .copy_in(row_base + source + plane, &0u16.to_ne_bytes());
            }
        }
    }
}

pub unsafe fn Shitroid_Func_1(k: u16, j: u16, game: &mut Game) -> u8 {
    let E: i32 = j as i32;
    let G: i32 = k as i32;
    let mut r18: u16 = (slot!(game, Enemy_Shitroid[G].base.x_width) as i32
        + slot!(game, Enemy_Shitroid[E].base.x_width) as i32
        + 1) as u16;
    let v4: u16 = abs16(
        (slot!(game, Enemy_Shitroid[E].base.x_pos))
            .wrapping_sub(slot!(game, Enemy_Shitroid[G].base.x_pos)),
    );
    let v5: bool = v4 >= r18;
    if v4 < r18 {
        r18 = (slot!(game, Enemy_Shitroid[G].base.y_height) as i32
            + slot!(game, Enemy_Shitroid[E].base.y_height) as i32
            + 1) as u16;
        return (abs16(
            (slot!(game, Enemy_Shitroid[E].base.y_pos))
                .wrapping_sub(slot!(game, Enemy_Shitroid[G].base.y_pos)),
        ) >= r18) as u8;
    }
    v5 as u8
}
pub unsafe fn Shitroid_Func_2(k: u16, rect: Rect16U, game: &mut Game) -> u8 {
    let E: i32 = k as i32;
    let mut R26: u16 =
        (slot!(game, Enemy_Shitroid[E].base.x_width) as i32 + rect.w as i32 + 1) as u16;
    let v2: u16 = abs16((rect.x).wrapping_sub(slot!(game, Enemy_Shitroid[E].base.x_pos)));
    let v3: bool = v2 >= R26;
    if v2 < R26 {
        R26 = (slot!(game, Enemy_Shitroid[E].base.y_height) as i32 + rect.h as i32 + 1) as u16;
        return (abs16((rect.y).wrapping_sub(slot!(game, Enemy_Shitroid[E].base.y_pos))) >= R26)
            as u8;
    }
    v3 as u8
}
crate::game::callback_dispatch! {
    pub unsafe fn CallShitroidFunc(ea: u32, k: u16, game: &mut Game) => ea;
    call (k, game); default { Unreachable(game); };
    @functions [pub unsafe fn (k: u16, game: &mut Game)];
    [11137097] Shitroid_Func_7 [] {
        let rect: Rect16U = Rect16U {
            x: 584,
            y: 74,
            w: 1,
            h: 1,
        };
        Shitroid_Func_GraduallyAccelerateTowards0x400(k, 0xf, rect.x, rect.y, game);
        if Shitroid_Func_2(k, rect, game) == 0 {
            slot!(game, Enemy_Shitroid[k as i32].var_A = (0xa9f06di32) as u16);
        }
    };
    [11137133] Shitroid_Func_8 [] {
        let E: i32 = k as i32 + 64;
        Shitroid_Func_GraduallyAccelerateTowards0x400(
            k,
            0xf,
            slot!(game, Enemy_Shitroid[E].base.x_pos),
            (slot!(game, Enemy_Shitroid[E].base.y_pos)).wrapping_sub(32),
            game,
        );
        if Shitroid_Func_1(k, (k).wrapping_add(64), game) & 1 == 0 {
            slot!(game, Enemy_Shitroid[k as i32].var_A = (0xa9f094i32) as u16);
        }
    };
    [11137172] Shitroid_Func_9 [] {
        let E1: i32 = k as i32 + 64;
        if Shitroid_AccelerateTowardsPoint(
            k,
            0x200,
            slot!(game, Enemy_Shitroid[E1].base.x_pos),
            (slot!(game, Enemy_Shitroid[E1].base.y_pos)).wrapping_sub(32),
            game,
        ) & 1
            != 0
        {
            let E: i32 = k as i32;
            slot! { game, Enemy_Shitroid[E]; var_B = 0, var_C = 0 }
            slot!(game, Enemy_Shitroid[E].base.x_pos = slot!(game, Enemy_Shitroid[E1].base.x_pos));
            slot!(game, Enemy_Shitroid[E].base.y_pos = (slot!(game, Enemy_Shitroid[E1].base.y_pos)).wrapping_sub(32));
            slot! { game, Enemy_Shitroid[E].base; current_instruction = addr_kShitroid_Ilist_F924 as u16, instruction_timer = 1 }
            slot!(game, Enemy_Shitroid[E].base.timer = 0);
            slot!(game, Enemy_Shitroid[E].var_A = (0xa9f0e6i32) as u16);
            slot!(game, Enemy_Shitroid[E].var_E = 1);
            slot!(game, Enemy_Shitroid[E].parameter_2 = 0);
            slot!(game, Enemy_Shitroid[E].var_F = 320);
        }
    };
    [11137317] Shitroid_Func_11 [] {
        slot!(game, Enemy_Shitroid[0x40i32].var_08 = 1);
        let E: i32 = k as i32;
        slot!(game, Enemy_Shitroid[E].var_A = (0xa9f138i32) as u16);
        slot!(game, Enemy_Shitroid[E].var_F = 192);
        Shitroid_Func_12(k, game);
    };
    [11137530] Shitroid_Func_14 [] {
        Shitroid_Func_GraduallyAccelerateTowards0x400(
            k,
            0xf,
            word!(game.samus_x_pos),
            (word!(game.samus_y_pos)).wrapping_sub(32),
            game,
        );
    };
    [11137550] Shitroid_Func_15 [] {
        CallSomeSamusCode(0x12, game);
        slot!(game, Enemy_Shitroid[0i32].var_A = (0xa9f21bi32) as u16);
        Shitroid_Func_16(k, game);
    };
    @functions [pub unsafe fn (_k: u16, game: &mut Game)];
    [11137728] Shitroid_Func_19 [] {
        Shitroid_Func_GraduallyAccelerateTowards0x400(
            word!(game.cur_enemy_index),
            0,
            word!(game.samus_x_pos),
            104,
            game,
        );
        let E: i32 = word!(game.cur_enemy_index) as i32;
        slot!(game, Enemy_Shitroid[E].var_F = slot!(game, Enemy_Shitroid[E].var_F).wrapping_sub(1));
        if (slot!(game, Enemy_Shitroid[E].var_F) & 0x8000) != 0 {
            QueueSfx2_Max6(0x7d, game);
            slot!(game, Enemy_Shitroid[E].var_A = (0xa9f2fbi32) as u16);
            slot!(game, Enemy_Shitroid[E].var_F = 88);
            slot! { game, Enemy_Shitroid[E].base; current_instruction = addr_kShitroid_Ilist_F924 as u16, instruction_timer = 1 }
            slot!(game, Enemy_Shitroid[E].base.timer = 0);
            Shitroid_Func_20(game);
        }
    };
    @functions [pub unsafe fn (k: u16, game: &mut Game)];
    [11137888] Shitroid_Func_22 [] {
        QueueSfx2_Max6(0x52, game);
        slot!(game, Enemy_Shitroid[k as i32].var_A = (0xa9f36di32) as u16);
        Shitroid_Func_23(game);
    };
    [11137982] Shitroid_Func_25 [] {
        if Shitroid_Func_27(k, game) & 1 != 0 {
            Shitroid_F3C4(k, game);
        }
    };
    @functions [pub unsafe fn (k: u16, game: &mut Game)];
    @prefix { let E: i32 = k as i32; };
    [11136991] Shitroid_Func_3 [] {
        slot! { game, Enemy_Shitroid[E]; var_B = 0, var_C = 0 }
    };
    [11137067] Shitroid_Func_5 [] {
        slot!(game, Enemy_Shitroid[E].var_A = (0xa9f037i32) as u16);
        slot!(game, Enemy_Shitroid[E].var_F = 464);
        Shitroid_Func_6(k, game);
    };
    [11137254] Shitroid_Func_10 [] {
        let v2: i32 = (slot!(game, Enemy_Shitroid[E].base.frame_counter) as i32 & 6) >> 1;
        let E1: i32 = k as i32 + 64;
        slot!(game, Enemy_Shitroid[E].base.x_pos = (slot!(game, Enemy_Shitroid[E1].base.x_pos)).wrapping_add((g_word_A993BB[v2 as usize]) as u16));
        slot!(game, Enemy_Shitroid[E].base.y_pos = (slot!(game, Enemy_Shitroid[E1].base.y_pos) as i32 + g_word_A993C3[v2 as usize] as i32 - 32i32) as u16);
        let fresh13: u16 = slot!(game, Enemy_Shitroid[E].var_F);
        slot!(game, Enemy_Shitroid[E].var_F = slot!(game, Enemy_Shitroid[E].var_F).wrapping_sub(1));
        if fresh13 == 1 {
            slot!(game, Enemy_Shitroid[E].var_A = (0xa9f125i32) as u16);
            slot! { game, Enemy_Shitroid[E].base; current_instruction = addr_kShitroid_Ilist_F906 as u16, instruction_timer = 1 }
            slot!(game, Enemy_Shitroid[E].base.timer = 0);
            slot!(game, Enemy_Shitroid[E].var_E = 10);
        }
    };
    [11137336] Shitroid_Func_12 [] {
        Shitroid_Func_GraduallyAccelerateTowards0x400(k, 0, slot!(game, Enemy_Shitroid[E].base.x_pos), 104, game);
        slot!(game, Enemy_Shitroid[E].var_F = slot!(game, Enemy_Shitroid[E].var_F).wrapping_sub(1));
        if (slot!(game, Enemy_Shitroid[E].var_F) & 0x8000) != 0 {
            slot!(game, Enemy_Shitroid[E].var_A = (0xa9f180i32) as u16);
            slot!(game, Enemy_Shitroid[E].parameter_2 = 1);
            { let at: usize = 0xcd20; let bytes: [u8; 2] = ((u16::from_le_bytes([game.ram().read_byte(at), game.ram().read_byte(at + 1)])) | (0x100u16)).to_le_bytes(); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); }
            { let at: usize = 0xcd20 + (2 ) as usize; let bytes: [u8; 2] = ((u16::from_le_bytes([game.ram().read_byte(at), game.ram().read_byte(at + 1)])) | (0x100u16)).to_le_bytes(); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); }
            spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x30, 0x3, 0xb763 as VoidP), game);
            spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x1f, 0x3, 0xb763 as VoidP), game);
        }
    };
    [11137408] Shitroid_Func_13 [] {
        let v2: u16 = abs16((slot!(game, Enemy_Shitroid[E].base.x_pos)).wrapping_sub(word!(game.samus_x_pos)));
        let mut shitro_var_02: u16 = 0;
        if v2 >= 8 {
            shitro_var_02 = slot!(game, Enemy_Shitroid[E].var_02);
            if shitro_var_02 != 0 {
                shitro_var_02 = shitro_var_02.wrapping_sub(1);
                if (shitro_var_02 & 0x8000) != 0 {
                    shitro_var_02 = 0;
                }
            }
        } else {
            shitro_var_02 = ((v2 >= 8) as i32 + slot!(game, Enemy_Shitroid[E].var_02) as i32 + 2) as u16;
        }
        slot!(game, Enemy_Shitroid[E].var_02 = shitro_var_02);
        if shitro_var_02 >= 0x100 || word!(game.samus_x_pos) as i32 - 512i32 & 0x8000 != 0 {
            slot!(game, Enemy_Shitroid[word!(game.cur_enemy_index) as i32].var_A = (0xa9f1fai32) as u16);
        } else {
            let shitro_var_01: u16 = slot!(game, Enemy_Shitroid[E].var_01);
            let mut v5: u16 = 0;
            if shitro_var_01 != 0 {
                slot!(game, Enemy_Shitroid[E].var_01 = (shitro_var_01).wrapping_sub(1));
                v5 = word!(game.samus_y_pos);
            } else {
                v5 = 80;
                if (word!(game.random_number) & 0xfff) >= 0xfe0 {
                    slot!(game, Enemy_Shitroid[E].var_01 = 32);
                }
            }
            Shitroid_Func_GraduallyAccelerateTowards0x400(
                word!(game.cur_enemy_index),
                0xa,
                word!(game.samus_x_pos),
                v5,
                game,
            );
        };
    };
    [11137563] Shitroid_Func_16 [] {
        if word!(game.samus_health) < 2 {
            word! { game, 0; samus_x_speed_divisor, bomb_counter }
            slot!(game, Enemy_Shitroid[E].var_A = (0xa9f2a2i32) as u16);
            slot! { game, Enemy_Shitroid[E]; var_B = 0, var_C = 0 }
            Enemy_SetInstrList(k, addr_kShitroid_Ilist_F906 as u16, game);
            slot!(game, Enemy_Shitroid[E].var_E = 10);
            CallSomeSamusCode(0x13, game);
            player_mother_brain(0, game);
            slot!(game, Enemy_Shitroid[E].var_04 = 0);
            QueueMusic_Delayed8(7, game);
        } else {
            word! {game; cooldown_timer = 8; bomb_counter = 5; samus_x_speed_divisor = 2; }
            if word!(game.samus_y_speed) as i32 - 4i32 & 0x8000 == 0 {
                word!(game.samus_y_speed = 2);
            }
            let v2: i32 = (slot!(game, Enemy_Shitroid[E].base.frame_counter) as i32 & 6) >> 1;
            slot!(game, Enemy_Shitroid[E].base.x_pos = (word!(game.samus_x_pos)).wrapping_add((g_word_A993BB[v2 as usize]) as u16));
            slot!(game, Enemy_Shitroid[E].base.y_pos = (word!(game.samus_y_pos) as i32 + g_word_A993C3[v2 as usize] as i32 - 20i32) as u16);
            vitals::drain_health_by_baby_metroid(game);
        };
    };
    [11137698] Shitroid_Func_17 [] {
        slot!(game, Enemy_Shitroid[E].var_A = (0xa9f2aei32) as u16);
        slot!(game, Enemy_Shitroid[E].var_F = 120);
        Shitroid_Func_18(k, game);
    };
    [11137955] Shitroid_Func_24 [] {
        slot!(game, Enemy_Shitroid[E].var_F = slot!(game, Enemy_Shitroid[E].var_F).wrapping_sub(1));
        if (slot!(game, Enemy_Shitroid[E].var_F) & 0x8000) != 0 {
            player_mother_brain(2, game);
            slot!(game, Enemy_Shitroid[E].parameter_2 = 1);
            slot!(game, Enemy_Shitroid[E].var_A = (0xa9f3bei32) as u16);
            Shitroid_Func_25(k, game);
        } else {
            Shitroid_Func_27(k, game);
        };
    };
    @countdown [pub unsafe fn (k: u16, game: &mut Game)];
    before {
        let E: i32 = k as i32;
        slot!(game, Enemy_Shitroid[E].var_F = slot!(game, Enemy_Shitroid[E].var_F).wrapping_sub(1));
    }
    when (slot!(game, Enemy_Shitroid[E].var_F) & 0x8000) != 0;
    [11137079] Shitroid_Func_6 {
        QueueMusic_Delayed8(5, game);
        slot!(game, Enemy_Shitroid[E].var_A = (0xa9f049i32) as u16);
        Shitroid_Func_7(k, game);
    };
    [11137710] Shitroid_Func_18 {
        slot!(game, Enemy_Shitroid[E].var_A = (0xa9f2c0i32) as u16);
        slot!(game, Enemy_Shitroid[E].var_F = 192);
        Shitroid_Func_19(k, game);
    };
    @external {
        11136998 => [Shitroid_Func_4(game)];
        11137787 => [Shitroid_Func_20(game)]; 11137828 => [Shitroid_Func_21(game)];
        11137901 => [Shitroid_Func_23(game)];
    }
}
pub unsafe fn Shitroid_Func_4(game: &mut Game) {
    if word!(game.layer1_x_pos) as i32 - 513i32 & 0x8000 != 0 {
        word!(game.layer1_x_pos = 512);
        {
            let value: u16 = game.ram().read_byte(0xcd20 + 0) as u16;
            let at: usize = 0xcd20;
            let bytes = value.to_ne_bytes();
            game.ram().write_byte(at, bytes[0]);
            game.ram().write_byte(at + 1, bytes[1]);
        };
        {
            let value: u16 = game.ram().read_byte(0xcd20 + 2) as u16;
            let at: usize = 0xcd20 + (2) as usize;
            let bytes = value.to_ne_bytes();
            game.ram().write_byte(at, bytes[0]);
            game.ram().write_byte(at + 1, bytes[1]);
        };
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x30, 0x3, 0xb767 as VoidP), game);
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x1f, 0x3, 0xb767 as VoidP), game);
        let E: i32 = word!(game.cur_enemy_index) as i32;
        slot!(game, Enemy_Shitroid[E].var_A = (0xa9f02bi32) as u16);
        slot!(game, Enemy_Shitroid[E].var_04 = 1);
    }
}
pub unsafe fn Shitroid_Func_20(game: &mut Game) {
    unsafe {
        Shitroid_Func_GraduallyAccelerateTowards0x400(
            word!(game.cur_enemy_index),
            0,
            (word!(game.samus_x_pos)).wrapping_sub(64),
            100,
            game,
        );
        let E: i32 = word!(game.cur_enemy_index) as i32;
        slot!(
            game,
            Enemy_Shitroid[E].var_F = slot!(game, Enemy_Shitroid[E].var_F).wrapping_sub(1)
        );
        if (slot!(game, Enemy_Shitroid[E].var_F) & 0x8000) != 0 {
            slot!(game, Enemy_Shitroid[E].var_A = (0xa9f324i32) as u16);
            slot!(game, Enemy_Shitroid[E].var_F = 88);
            Shitroid_Func_21(game);
        }
    }
}
pub unsafe fn Shitroid_Func_21(game: &mut Game) {
    unsafe {
        Shitroid_Func_GraduallyAccelerateTowards0x400(
            word!(game.cur_enemy_index),
            0,
            (word!(game.samus_x_pos)).wrapping_add(96),
            104,
            game,
        );
        let E: i32 = word!(game.cur_enemy_index) as i32;
        slot!(
            game,
            Enemy_Shitroid[E].var_F = slot!(game, Enemy_Shitroid[E].var_F).wrapping_sub(1)
        );
        if (slot!(game, Enemy_Shitroid[E].var_F) & 0x8000) != 0 {
            slot!(game, Enemy_Shitroid[E].var_A = (0xa9f3a3i32) as u16);
            slot!(game, Enemy_Shitroid[E].var_F = 256);
            slot! { game, Enemy_Shitroid[E].base; current_instruction = addr_kShitroid_Ilist_F93A as u16, instruction_timer = 1 }
            slot!(game, Enemy_Shitroid[E].base.timer = 0);
        }
    }
}
pub unsafe fn Shitroid_Func_23(game: &mut Game) {
    unsafe {
        let rect: Rect16U = Rect16U {
            x: -128i32 as u16,
            y: 64,
            w: 8,
            h: 8,
        };
        Shitroid_Func_GraduallyAccelerateTowards0x400(
            word!(game.cur_enemy_index),
            0,
            rect.x,
            rect.y,
            game,
        );
        if Shitroid_Func_2(word!(game.cur_enemy_index), rect, game) == 0 {
            let E: i32 = word!(game.cur_enemy_index) as i32;
            slot! { game, Enemy_Shitroid[E]; var_B = 0, var_C = 0 }
            slot!(
                game,
                Enemy_Shitroid[E].base.properties = (slot!(game, Enemy_Shitroid[E].base.properties)
                    as i32
                    & !(kEnemyProps_DisableSamusColl as i32 | kEnemyProps_Invisible as i32))
                    as u16
            );
            slot!(game, Enemy_Shitroid[E].var_A = (0xa9efdfi32) as u16);
        }
    }
}
pub unsafe fn Shitroid_F3C4(k: u16, game: &mut Game) {
    slot!(game, Enemy_Shitroid[k as i32].var_A = (0xa9f360i32) as u16);
}
pub unsafe fn Shitroid_Func_26(k: u16, game: &mut Game) {
    unsafe {
        if slot!(game, Enemy_Shitroid[k as i32].var_A) as i32 == 0xa9f3be & 0xffff {
            Shitroid_F3C4(k, game);
        }
    }
}
pub unsafe fn Shitroid_Func_27(k: u16, game: &mut Game) -> u8 {
    unsafe {
        let E: i32 = k as i32;
        let v2: u16 = abs16(
            (slot!(game, Enemy_Shitroid[E].base.x_pos)).wrapping_sub(word!(game.samus_x_pos)),
        );
        let mut shitro_var_02: u16 = 0;
        if v2 >= 2 {
            shitro_var_02 = slot!(game, Enemy_Shitroid[E].var_02);
            if shitro_var_02 != 0 {
                shitro_var_02 = shitro_var_02.wrapping_sub(1);
                if (shitro_var_02 & 0x8000) != 0 {
                    shitro_var_02 = 0;
                }
            }
        } else {
            shitro_var_02 =
                ((v2 >= 2) as i32 + slot!(game, Enemy_Shitroid[E].var_02) as i32 + 2) as u16;
        }
        slot!(game, Enemy_Shitroid[E].var_02 = shitro_var_02);
        let shitro_var_01: u16 = slot!(game, Enemy_Shitroid[E].var_01);
        let mut v5: u16 = 0;
        if shitro_var_01 != 0 {
            slot!(
                game,
                Enemy_Shitroid[E].var_01 = (shitro_var_01).wrapping_sub(1)
            );
            v5 = (word!(game.samus_y_pos)).wrapping_sub(18);
        } else {
            v5 = 80;
            if (word!(game.random_number) & 0xfff) >= 0xfe0 {
                slot!(game, Enemy_Shitroid[E].var_01 = 32);
            }
        }
        Shitroid_Func_GraduallyAccelerateTowards0x400(
            word!(game.cur_enemy_index),
            8,
            word!(game.samus_x_pos),
            v5,
            game,
        );
        (slot!(
            game,
            Enemy_Shitroid[word!(game.cur_enemy_index) as i32].var_02
        ) >= 0x400
            || word!(game.samus_x_pos) as i32 - 128i32 & 0x8000 != 0) as u8
    }
}
pub unsafe fn Shitroid_GraduallyAccelerateTowardsPt(
    k: u16,
    j: u16,
    r18: u16,
    r20: u16,
    r26: u16,
    game: &mut Game,
) {
    unsafe {
        crate::verify_probe::hit("homing.metroid");
        let divisor = g_byte_A9F56A[j as usize];
        Shitroid_GraduallyAccelerateHoriz(k, r18, divisor as u16, r26, game);
        let E: i32 = k as i32;
        let distance = (slot!(game, Enemy_Shitroid[E].base.y_pos) as i16).wrapping_sub(r20 as i16);
        if distance != 0 {
            slot!(
                game,
                Enemy_Shitroid[E].var_C = homing_velocity(
                    distance,
                    slot!(game, Enemy_Shitroid[E].var_C),
                    divisor as u8,
                    HomingReversal::ExtraForce(8),
                    1280,
                )
            );
        }
    }
}
pub unsafe fn Shitroid_GraduallyAccelerateHoriz(
    k: u16,
    r18: u16,
    r24: u16,
    r26: u16,
    game: &mut Game,
) {
    unsafe {
        let E: i32 = k as i32;
        let distance = (slot!(game, Enemy_Shitroid[E].base.x_pos) as i16).wrapping_sub(r18 as i16);
        if distance != 0 {
            // Capture velocity before the screen check, as in both original branches.
            let velocity = slot!(game, Enemy_Shitroid[E].var_B);
            let reversing = (velocity as i16 >= 0) == (distance > 0);
            let force = if reversing && Shitroid_CheckIfOnScreen(k, game) & 1 != 0 {
                r26.wrapping_add(8)
            } else {
                8
            };
            slot!(
                game,
                Enemy_Shitroid[E].var_B = homing_velocity(
                    distance,
                    velocity,
                    r24 as u8,
                    HomingReversal::ExtraForce(force),
                    2048,
                )
            );
        }
    }
}
pub unsafe fn Shitroid_CheckIfOnScreen(k: u16, game: &mut Game) -> u8 {
    let E: i32 = k as i32;
    let mut y_pos: i16 = slot!(game, Enemy_Shitroid[E].base.y_pos) as i16;
    let result: u8 = 1;
    if y_pos >= 0 {
        let mut v3: i16 = (y_pos as i32 + 96 - word!(game.layer1_y_pos) as i32) as i16;
        if v3 >= 0 && v3 as i32 - 416i32 & 0x8000 != 0 {
            let mut x_pos: i16 = slot!(game, Enemy_Shitroid[E].base.x_pos) as i16;
            if x_pos >= 0 {
                let mut v5: i16 = (x_pos as i32 + 16 - word!(game.layer1_x_pos) as i32) as i16;
                if v5 >= 0 && v5 as i32 - 288i32 & 0x8000 != 0 {
                    return 0;
                }
            }
        }
    }
    result
}
pub unsafe fn Shitroid_AccelerateTowardsPoint(
    k: u16,
    a: u16,
    r18: u16,
    r20: u16,
    game: &mut Game,
) -> u8 {
    unsafe {
        let mut R28: u16 = 0;
        R28 = (R28).wrapping_add(Shitroid_AccelerateTowardsX(k, r18, a, game));
        R28 = (R28).wrapping_add(Shitroid_AccelerateTowardsY(k, r20, a, game));
        R28 = R28 >> 1;
        ((R28) as u8) & (1u8)
    }
}
pub unsafe fn Shitroid_AccelerateTowardsY(k: u16, r20: u16, r22: u16, game: &mut Game) -> u16 {
    let E: i32 = k as i32;
    let mut result: u16 = 0;
    let v2: i16 = ((slot!(game, Enemy_Shitroid[E].base.y_pos)) as i16).wrapping_sub((r20) as i16);
    if v2 == 0 {
        result = result.wrapping_add(1);
    } else if v2 >= 0 {
        let mut v7: u16 = (slot!(game, Enemy_Shitroid[E].var_C)).wrapping_sub(r22);
        if v7 as i32 + 1280 & 0x8000 != 0 {
            v7 = -1280i32 as u16;
        }
        slot!(game, Enemy_Shitroid[E].var_C = v7);
        if ((slot!(game, Enemy_Shitroid[E].base.y_pos) as i32 + (v7 >> 8) as i8 as i32 - r20 as i32)
            as i16)
            <= 0
        {
            slot!(game, Enemy_Shitroid[E].var_C = 0);
            result = result.wrapping_add(1);
        }
    } else {
        let mut v3: u16 = (r22).wrapping_add(slot!(game, Enemy_Shitroid[E].var_C));
        if v3 as i32 - 1280i32 & 0x8000 == 0 {
            v3 = 1280;
        }
        slot!(game, Enemy_Shitroid[E].var_C = v3);
        if ((slot!(game, Enemy_Shitroid[E].base.y_pos) as i32 + (v3 >> 8) as i8 as i32 - r20 as i32)
            as i16)
            >= 0
        {
            slot!(game, Enemy_Shitroid[E].var_C = 0);
            result = result.wrapping_add(1);
        }
    }
    result
}
pub unsafe fn Shitroid_AccelerateTowardsX(k: u16, r18: u16, r22: u16, game: &mut Game) -> u16 {
    let E: i32 = k as i32;
    let mut result: u16 = 0;
    if ((slot!(game, Enemy_Shitroid[E].base.x_pos)) as i16).wrapping_sub((r18) as i16) >= 0 {
        let mut v6: u16 = (slot!(game, Enemy_Shitroid[E].var_B)).wrapping_sub(r22);
        if v6 as i32 + 1280 & 0x8000 != 0 {
            v6 = -1280i32 as u16;
        }
        slot!(game, Enemy_Shitroid[E].var_B = v6);
        if ((slot!(game, Enemy_Shitroid[E].base.x_pos) as i32 + (v6 >> 8) as i8 as i32 - r18 as i32)
            as i16)
            <= 0
        {
            slot!(game, Enemy_Shitroid[E].var_B = 0);
            result = result.wrapping_add(1);
        }
    } else {
        let mut v2: u16 = (r22).wrapping_add(slot!(game, Enemy_Shitroid[E].var_B));
        if v2 as i32 - 1280i32 & 0x8000 == 0 {
            v2 = 1280;
        }
        slot!(game, Enemy_Shitroid[E].var_B = v2);
        if ((slot!(game, Enemy_Shitroid[E].base.x_pos) as i32 + (v2 >> 8) as i8 as i32 - r18 as i32)
            as i16)
            >= 0
        {
            slot!(game, Enemy_Shitroid[E].var_B = 0);
            result = result.wrapping_add(1);
        }
    }
    result
}
pub unsafe fn Shitroid_HandleCutscenePalette_Common(r22: u16, r18: u16, game: &mut Game) {
    unsafe {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        let mut shitro_var_D_high: i8 = slot!(byte game, Enemy_Shitroid[E].var_D[1]) as i8;
        if shitro_var_D_high != 0 {
            slot!(byte game, Enemy_Shitroid[E].var_D[1] = ((shitro_var_D_high) as u8).wrapping_sub(1));
        } else {
            slot!(byte game, Enemy_Shitroid[E].var_D[1] = slot!(game, Enemy_Shitroid[E].var_E) as u8);
            let v3: u8 = (slot!(byte game, Enemy_Shitroid[E].var_D[0]) as i32 + 1 & 7) as u8;
            slot!(byte game, Enemy_Shitroid[E].var_D[0] = v3);
            let v4: u16 =
                Shitroid_HandleCrySoundEffect(word!(game.cur_enemy_index), v3 as u16, game);
            WriteColorsToPalette(r18, 0xa9, (r22 as i32 + 8 * v4 as i32) as u16, 4, game);
        };
    }
}
pub unsafe fn Shitroid_HandleCrySoundEffect(k: u16, a: u16, game: &mut Game) -> u16 {
    if a == 5 {
        let E: i32 = k as i32;
        if slot!(game, Enemy_Shitroid[E].var_04) != 0 {
            let v3: u16 = (slot!(game, Enemy_Shitroid[E].var_05)).wrapping_add(1);
            slot!(game, Enemy_Shitroid[E].var_05 = v3);
            if v3 >= 4 {
                slot!(game, Enemy_Shitroid[E].var_05 = 0);
                let mut v4: u16 = 114;
                if slot!(game, Enemy_Shitroid[E].var_E) < 0xa {
                    v4 = 120;
                }
                QueueSfx2_Max6(v4, game);
            }
        }
        return 5;
    }
    a
}
crate::instruction_callbacks! { pub unsafe fn (_k: u16, _jp: EnemyScript) -> EnemyScript;
    Shitroid_Instr_3 [EnemyScript::jump(addr_kShitroid_Ilist_F90E as u16)] { };
    Shitroid_Instr_4 [EnemyScript::jump(addr_kShitroid_Ilist_F924 as u16)] { };
    Shitroid_Instr_6 [EnemyScript::jump(addr_kShitroid_Ilist_F93A as u16)] { };
}

crate::game::callback_dispatch! {
    pub(crate) unsafe fn DispatchEnemyAi(ea: u32, game: &mut Game) => ea;
    call (game); default { crate::rtl::Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [11110023] MotherBrainsBody_Init [] {
        word_transfers!(game, descending, 2048; le(tilemap_stuff, 2047) <- value(824));
        MotherBrain_SetBodyInstrs(addr_kMotherBrain_Ilist_9C13 as u16, game);
        let E: i32 = 0;
        slot!(game, Enemy_MotherBrain[E].base.vram_tiles_index = 0);
        slot!(game, Enemy_MotherBrain[E].base.properties = (slot!(game, Enemy_MotherBrain[E].base.properties) as i32
            | (kEnemyProps_BlockPlasmaBeam as i32
                | kEnemyProps_Tangible as i32
                | kEnemyProps_Invisible as i32)) as u16);
        slot!(game, Enemy_MotherBrain[E].base.palette_index = 0);
        WriteColorsToTargetPalette(
            0xa9,
            0x162,
            (addr_kMotherBrainPalette_4 as i32 + 2) as u16,
            0xf,
            game,
        );
        WriteColorsToTargetPalette(
            0xa9,
            0x1e2,
            (addr_kMotherBrainPalette_3 as i32 + 2) as u16,
            0xf,
            game,
        );
        slot!(game, Enemy_MotherBrain[E].var_00 = 0);
        let E1: i32 = 0x40;
        slot!(game, Enemy_MotherBrain[E1].var_02 = 0);
        slot!(game, Enemy_MotherBrain[E].var_04 = 2);
        slot!(game, Enemy_MotherBrain[E1].var_A = (0xa987d0i32) as u16);
        slot!(game, Enemy_MotherBrain[E].var_A = (0xa987e1i32) as u16);
        LoadFxEntry(1, game);
        for v3 in 0u16..12 {
            SpawnEprojWithRoomGfx(addr_kEproj_MotherBrainRoomTurrets as u16, v3, game);
        }
    };
    [11110149] MotherBrainsBrain_Init [] {
        InitializeEnemyCorpseRotting(0x40, addr_stru_A9DE08 as u16, game);
        let E1: i32 = 0x40;
        slot!(game, Enemy_MotherBrain[E1].base.health = 3000);
        MotherBrain_SetBrainUnusedInstrs(addr_kMotherBrain_Ilist_9C13 as u16, game);
        slot! { game, Enemy_MotherBrain[E1].base; vram_tiles_index = 0, properties = (slot!(game, Enemy_MotherBrain[E1].base.properties)) | (0x1100u16), palette_index = 512 };
        let E: i32 = 0;
        slot! { game, Enemy_MotherBrain[E]; var_0C = 512, var_0D = 512 }
        MotherBrain_SetBrainInstrs(addr_stru_A99C21 as u16, game);
        MotherBrain_SetupBrainNormalPal(game);
    };
    [11110206] MotherBrainsBody_Hurt [] {
        MotherBrain_Pal_HandleRoomPal(game);
        let E: i32 = 0;
        if slot!(game, Enemy_MotherBrain[E].var_A) == (fnMotherBrainsBody_FirstPhase_DoubleRet as u16) {
            MotherBrainsBody_FirstPhase_DoubleRet(game);
            return;
        }
        CallMotherBrainFunc(((slot!(game, Enemy_MotherBrain[E].var_A)) as u32) | (0xa90000u32), game);
        MotherBrain_HandlePalette(game);
        MotherBrain_SamusCollDetect(game);
        MotherBrain_Pal_ProcessInvincibility(game);
        if slot!(game, Enemy_MotherBrain[0x40i32].var_02) != 0 {
            { let value: u32 = fnMotherBrainsBody_UnpauseHook as u32; { let value: u16 = value as u16; let at: usize = 0x604; let bytes = value.to_ne_bytes(); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); }; game.ram().write_byte(0x604 + 2, (value >> 16) as u8); };
        }
    };
    [11110279] MotherBrainsBody_Powerbomb [] {
        actors::combat::power_bomb_reaction(&mut Runtime::new(game), actors::api::DeathHandling::LeavePending);
        MotherBrainsBrain_Hurt(game);
    };
    [11110283] MotherBrainsBrain_Hurt [] {
        { let value: u32 = 0xa98786; { let value: u16 = value as u16; let at: usize = 0x178c; let bytes = value.to_ne_bytes(); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); }; game.ram().write_byte(0x178c + 2, (value >> 16) as u8); };
        let v0: i32 = 0;
        if (slot!(game, EnemyData[v0 + core::mem::size_of::<EnemyData>() as i32].properties) & 0x100) != 0 {
            CallMotherBrainFunc(((slot!(game, EnemyData[v0 + core::mem::size_of::<EnemyData>() as i32].ai_var_A)) as u32) | (0xa90000u32), game);
        }
    };
    [11111221] MotherBrainsTubesFalling_Init [] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        let v1: i32 = slot!(game, Enemy_MotherBrainsTubesFalling[E].parameter_1) as i32 >> 1;
        slot!(game, Enemy_MotherBrainsTubesFalling[E].base.x_width = g_word_A98B5D[v1 as usize]);
        slot!(game, Enemy_MotherBrainsTubesFalling[E].base.y_height = g_word_A98B67[v1 as usize]);
        slot!(game, Enemy_MotherBrainsTubesFalling[E].var_B = g_word_A98B71[v1 as usize]);
        slot! { game, Enemy_MotherBrainsTubesFalling[E]; var_D = 0, var_E = 0, var_C = 0 }
        slot!(game, Enemy_MotherBrainsTubesFalling[E].var_A = crate::rtl::rom_table_word!(game, 0xa98b7b, v1 as isize));
    };
    [11121923] MotherBrainsBody_Shot [] {
        actors::combat::create_dud(&mut Runtime::new(game));
    };
    [11122118] MotherBrainsBrain_Touch [] {
        if game.ram().read_byte(0xa1f) == 3 {
            let mut v0: i16 = 13;
            let v1: i32 = 0;
            let flash_timer: u16 = slot!(game, EnemyData[v1 + core::mem::size_of::<EnemyData>() as i32].flash_timer);
            if flash_timer != 0 && flash_timer & 1 != 0 {
                v0 = 14;
            }
            slot!(game, EnemyData[v1 + core::mem::size_of::<EnemyData>() as i32].flash_timer = v0 as u16);
        }
    };
    [11126544] ShitroidInCutscene_Init [] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        slot!(game, Enemy_ShitroidInCutscene[E].base.properties = (slot!(game, Enemy_ShitroidInCutscene[E].base.properties) as i32
            | (kEnemyProps_DisableSamusColl as i32 | kEnemyProps_BlockPlasmaBeam as i32))
            as u16);
        slot!(game, Enemy_ShitroidInCutscene[E].base.palette_index = 3584);
        slot! { game, Enemy_ShitroidInCutscene[E].base; current_instruction = addr_kShitroid_Ilist_CFA2 as u16, instruction_timer = 1 }
        slot!(game, Enemy_ShitroidInCutscene[E].var_04 = 1);
        slot!(game, Enemy_ShitroidInCutscene[E].base.timer = 0);
        slot!(game, Enemy_ShitroidInCutscene[E].var_E = 10);
        slot! { game, Enemy_ShitroidInCutscene[E].base; vram_tiles_index = 160, x_pos = 320, y_pos = 96 };
        slot! { game, Enemy_ShitroidInCutscene[E]; var_B = 0, var_C = 0, var_09 = 0 }
        slot!(game, Enemy_ShitroidInCutscene[E].var_A = (0xa9c7cci32) as u16);
        slot!(game, Enemy_ShitroidInCutscene[E].var_F = 248);
        slot!(game, Enemy_ShitroidInCutscene[E].var_0F = (0xa9f683i32) as u16);
        WriteColorsToPalette(
            0x1e2,
            0xa9,
            (addr_kMotherBrainPalette_2 as i32 + 2) as u16,
            0xf,
            game,
        );
    };
    [11126649] ShitroidInCutscene_Main [] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        slot!(game, Enemy_ShitroidInCutscene[E].base.shake_timer = 0);
        CallShitroidCutsceneFuncA(
            ((slot!(game, Enemy_ShitroidInCutscene[E].var_A)) as u32) | (0xa90000u32),
            word!(game.cur_enemy_index),
            game,
        );
        MoveEnemyWithVelocity(game);
        ShitroidInCutscene_Flashing(word!(game.cur_enemy_index), game);
        ShitroidInCutscene_HandleHealthBasedPalette(word!(game.cur_enemy_index), game);
        let r18: u16 = slot!(game, Enemy_ShitroidInCutscene[word!(game.cur_enemy_index) as i32].var_0F);
        CallShitroidCutsceneFunc0F(((r18) as u32) | (0xa90000u32), game);
    };
    [11128579] ShitroidInCutscene_Touch [] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        if slot!(game, Enemy_ShitroidInCutscene[E].var_A) as i32 == 0xa9ca66 & 0xffff
            && Shitroid_AccelerateTowardsPoint(
                word!(game.cur_enemy_index),
                0x10,
                word!(game.samus_x_pos),
                (word!(game.samus_y_pos)).wrapping_sub(20),
                game,
            ) & 1
                != 0
        {
            slot! { game, Enemy_ShitroidInCutscene[E]; var_B = 0, var_C = 0 }
            slot!(game, Enemy_ShitroidInCutscene[E].var_A = (0xa9ca7ai32) as u16);
        }
    };
    [11129608] DeadTorizo_Init [] {
        word_transfers!(game, descending, 2048; le(tilemap_stuff, 2047) <- value(0));
        let E: i32 = 0;
        slot!(game, Enemy_DeadTorizo[E].var_A = (0xa9d3adi32) as u16);
        slot!(game, Enemy_DeadTorizo[E].base.properties = (slot!(game, Enemy_DeadTorizo[E].base.properties)) | (0xa000u16));
        slot! { game, Enemy_DeadTorizo[E].base; current_instruction = addr_kDeadTorizo_Ilist_D6DC as u16, instruction_timer = 1 }
        slot!(game, Enemy_DeadTorizo[E].base.timer = 0);
        slot!(game, Enemy_DeadTorizo[E].base.palette_index = 512);
        slot! { game, Enemy_DeadTorizo[E]; var_B = 0, var_C = 8, var_04 = 0, var_02 = 15, var_03 = 0 };
        InitializeEnemyCorpseRotting(0, addr_stru_A9DD58 as u16, game);
    };
    [11129704] DeadTorizo_Main [] {
        let E: i32 = 0;
        if (slot!(game, Enemy_DeadTorizo[E].base.properties) as i32 & kEnemyProps_Tangible as i32) == 0
            && actors::part_hitboxes::dead_torizo_contact(game)
        {
            slot!(game, Enemy_DeadTorizo[E].base.properties = (slot!(game, Enemy_DeadTorizo[E].base.properties)) | ((kEnemyProps_Tangible) as u16));
            slot!(game, Enemy_DeadTorizo[E].var_A = (0xa9d3e6i32) as u16);
        }
        { let value: u32 = 0xa9d39a; { let value: u16 = value as u16; let at: usize = 0x178c; let bytes = value.to_ne_bytes(); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); }; game.ram().write_byte(0x178c + 2, (value >> 16) as u8); };
        CallDeadTorizoFuncA(((slot!(game, Enemy_DeadTorizo[E].var_A)) as u32) | (0xa90000u32), game);
        DeadTorizo_Func_1(game);
    };
    [11129898] DeadTorizo_Powerbomb [] {
        if (slot!(game, Enemy_DeadTorizo[0i32].base.properties) as i32 & kEnemyProps_Tangible as i32) == 0
        {
            DeadTorizo_Shot(game);
        }
    };
    [11129907] DeadTorizo_Shot [] {
        let E: i32 = 0;
        slot!(game, Enemy_DeadTorizo[E].base.properties = (slot!(game, Enemy_DeadTorizo[E].base.properties)) | ((kEnemyProps_Tangible) as u16));
        slot!(game, Enemy_DeadTorizo[E].var_A = (0xa9d3e6i32) as u16);
    };
    [11130806] DeadSidehopper_Init [] {
        let dsr_parameter_1: u16 =
            slot!(game, Enemy_DeadSidehopper[word!(game.cur_enemy_index) as i32].parameter_1);
        if dsr_parameter_1 != 0 {
            if dsr_parameter_1 != 2 {
                Unreachable(game);
            }
            DeadSidehopper_Init_1(game);
        } else {
            DeadSidehopper_Init_0(game);
        };
    };
    [11130953] DeadZoomer_Init [] {
        let v1: i32 = word!(game.cur_enemy_index) as i32;
        initialize_rotting_enemy_corpse!(v1, 0xa9da69i32, game, 0xa9d86a, 0xa9d870);
    };
    [11130998] DeadRipper_Init [] {
        let v1: i32 = word!(game.cur_enemy_index) as i32;
        initialize_rotting_enemy_corpse!(v1, 0xa9da73i32, game, 0xa9d897, 0xa9d89b);
    };
    [11131039] DeadSkree_Init [] {
        let v1: i32 = word!(game.cur_enemy_index) as i32;
        initialize_rotting_enemy_corpse!(v1, 0xa9da6ei32, game, 0xa9d8c0, 0xa9d8c6);
    };
    [11131084] DeadSidehopper_Powerbomb [] {
        if slot!(game, Enemy_DeadSidehopper[word!(game.cur_enemy_index) as i32].var_08) >= 8 {
            DeadSidehopper_Shot(game);
        } else {
            DeadSidehopper_Main(game);
        };
    };
    [11131099] DeadSidehopper_Main [] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        CallDeadSidehopperFunc(
            ((slot!(game, Enemy_DeadSidehopper[E].var_A)) as u32) | (0xa90000u32),
            word!(game.cur_enemy_index),
            game,
        );
    };
    [11132141] DeadZoomer_Powerbomb [] {
        if (slot!(game, EnemyData[word!(game.cur_enemy_index) as i32].properties) as i32
            & kEnemyProps_Tangible as i32)
            == 0
        {
            DeadZoomer_Shot(game);
        }
    };
    [11132152] DeadZoomer_Shot [] {
        DeadSidehopper_DD34((0xa9dad0i32) as u16, game);
    };
    [11132157] DeadRipper_Powerbomb [] {
        if (slot!(game, EnemyData[word!(game.cur_enemy_index) as i32].properties) as i32
            & kEnemyProps_Tangible as i32)
            == 0
        {
            DeadRipper_Shot(game);
        }
    };
    [11132168] DeadRipper_Shot [] {
        DeadSidehopper_DD34((0xa9dae6i32) as u16, game);
    };
    [11132173] DeadSkree_Powerbomb [] {
        if (slot!(game, EnemyData[word!(game.cur_enemy_index) as i32].properties) as i32
            & kEnemyProps_Tangible as i32)
            == 0
        {
            DeadSkree_Shot(game);
        }
    };
    [11132184] DeadSkree_Shot [] {
        DeadSidehopper_DD34((0xa9dafci32) as u16, game);
    };
    [11132189] DeadSidehopper_Shot [] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        if !((slot!(game, Enemy_DeadSidehopper[E].base.properties) as i32 & kEnemyProps_Tangible as i32) != 0 || slot!(game, Enemy_DeadSidehopper[E].var_08) < 8) {
            DeadSidehopper_DD31(game);
        }
    };
    [11132228] DeadSidehopper_Touch [] {
        if slot!(game, Enemy_DeadSidehopper[word!(game.cur_enemy_index) as i32].var_08) < 8 {
            actors::combat::contact_reaction(&mut Runtime::new(game), actors::api::DeathHandling::LeavePending);
        } else {
            DeadSidehopper_DD31(game);
        };
    };
    [11136823] Shitroid_Init [] {
        word_transfers!(game, descending, 2048; le(tilemap_stuff, 2047) <- value(0));
        let E: i32 = word!(game.cur_enemy_index) as i32;
        slot!(game, Enemy_Shitroid[E].base.properties = (slot!(game, Enemy_Shitroid[E].base.properties) as i32
            | (kEnemyProps_DisableSamusColl as i32 | kEnemyProps_BlockPlasmaBeam as i32))
            as u16);
        slot!(game, Enemy_Shitroid[E].base.palette_index = 1024);
        slot! { game, Enemy_Shitroid[E].base; current_instruction = addr_kShitroid_Ilist_F90E as u16, instruction_timer = 1 }
        slot!(game, Enemy_Shitroid[E].base.timer = 0);
        let mut v2: u16 = (0xa9efe6i32) as u16;
        if (word!(game.layer1_x_pos) & 0x8000) != 0 {
            slot!(game, Enemy_Shitroid[E].base.properties = (slot!(game, Enemy_Shitroid[E].base.properties) as i32
                | (kEnemyProps_Tangible as i32 | kEnemyProps_Invisible as i32))
                as u16);
            v2 = (0xa9efdfi32) as u16;
        }
        slot!(game, Enemy_Shitroid[E].var_A = v2);
        slot! { game, Enemy_Shitroid[E]; var_B = 0, var_C = 0 }
        slot!(game, Enemy_Shitroid[E].var_E = 10);
        slot!(game, Enemy_Shitroid[E].parameter_2 = 0);
        WriteColorsToTargetPalette(0xa9, 0x120, addr_word_A9F8C6 as u16, 0x10, game);
        WriteColorsToTargetPalette(
            0xa9,
            0x140,
            addr_kShitroidInCutscene_Palette as u16,
            0x10,
            game,
        );
        WriteColorsToTargetPalette(
            0xa9,
            0x1e0,
            addr_kDeadSidehopper_Palette_0 as u16,
            0x10,
            game,
        );
    };
    [11136954] Shitroid_Powerbomb [] {
        if slot!(game, Enemy_Shitroid[word!(game.cur_enemy_index) as i32].parameter_2) != 0 {
            Shitroid_Func_26(word!(game.cur_enemy_index), game);
        }
        Shitroid_Main(game);
    };
    [11136965] Shitroid_Main [] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        slot!(game, Enemy_Shitroid[E].base.health = 0x7fff);
        CallShitroidFunc(
            ((slot!(game, Enemy_Shitroid[E].var_A)) as u32) | (0xa90000u32),
            word!(game.cur_enemy_index),
            game,
        );
        MoveEnemyWithVelocity(game);
        if word!(game.palette_change_num) == 0 {
            Shitroid_HandleNormalPalette(game);
        }
    };
    [11138953] Shitroid_Touch [] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        if slot!(game, Enemy_Shitroid[E].parameter_2) != 0 {
            Shitroid_Func_26(word!(game.cur_enemy_index), game);
            if game.ram().read_byte(0xa1f) == 3 && word!(game.samus_x_pos) as i32 - 512i32 & 0x8000 == 0
            {
                let r18: u16 = (0x80
                    - actor_api::angle_from_delta((word!(game.samus_x_pos)).wrapping_sub(slot!(game, Enemy_Shitroid[E].base.x_pos)), (word!(game.samus_y_pos)).wrapping_sub(slot!(game, Enemy_Shitroid[E].base.y_pos))) as i32
                    + 0x80) as u8 as u16;
                slot!(game, Enemy_Shitroid[E].var_B = (slot!(game, Enemy_Shitroid[E].var_B)).wrapping_add(Math_MultBySin(0x40, r18)));
                slot!(game, Enemy_Shitroid[E].var_C = (slot!(game, Enemy_Shitroid[E].var_C)).wrapping_add(Math_MultByCos(0x40, r18)));
            } else if slot!(game, Enemy_Shitroid[E].var_A) as i32 == 0xa9f1fa & 0xffff {
                if Shitroid_AccelerateTowardsPoint(
                    word!(game.cur_enemy_index),
                    0x200,
                    word!(game.samus_x_pos),
                    (word!(game.samus_y_pos)).wrapping_sub(32),
                    game,
                ) & 1
                    != 0
                {
                    slot! { game, Enemy_Shitroid[E].base; current_instruction = addr_kShitroid_Ilist_F924 as u16, instruction_timer = 1 }
                    slot!(game, Enemy_Shitroid[E].base.timer = 0);
                    slot!(game, Enemy_Shitroid[E].var_E = 1);
                    slot! { game, Enemy_Shitroid[E]; parameter_2 = 0, var_B = 0, var_C = 0 }
                    slot!(game, Enemy_Shitroid[E].var_A = (0xa9f20ei32) as u16);
                }
            } else if slot!(game, Enemy_Shitroid[E].var_A) as i32 == 0xa9f180 & 0xffff {
                slot!(game, Enemy_Shitroid[E].var_A = (0xa9f1fai32) as u16);
            }
        }
    };
    [11139138] Shitroid_Shot [] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        if slot!(game, Enemy_Shitroid[E].parameter_2) != 0 {
            Shitroid_Func_26(word!(game.cur_enemy_index), game);
            let r18: u16 = -(actor_api::angle_from_delta((word!(game.projectile_x_pos[0])).wrapping_sub(slot!(game, Enemy_Shitroid[E].base.x_pos)), (word!(game.projectile_y_pos[0])).wrapping_sub(slot!(game, Enemy_Shitroid[E].base.y_pos))) as i32) as u8 as u16;
            let mut v1: u16 = (8u16).wrapping_mul(word!(
                game.projectile_damage[word!(game.collision_detection_index)]
            ));
            if v1 >= 0xf0 {
                v1 = 240;
            }
            slot!(game, Enemy_Shitroid[E].var_B = (slot!(game, Enemy_Shitroid[E].var_B)).wrapping_add(Math_MultBySin(v1, r18)));
            slot!(game, Enemy_Shitroid[E].var_C = (slot!(game, Enemy_Shitroid[E].var_C)).wrapping_add(Math_MultByCos(v1, r18)));
        }
    };
    @forward [pub unsafe fn (game: &mut Game)];
    [11108367] Enemy_GrappleReact_CancelBeam_A9 => actors::combat::select_frozen_ai(&mut Runtime::new(game));
    [11108417] Enemy_NormalFrozenAI_A9 => actors::scheduling::update_frozen_enemy(&mut Runtime::new(game));
}

crate::game::callback_dispatch! {
    pub(crate) unsafe fn DispatchEnemyInstr(ea: u32, k: u16, j: EnemyScript, game: &mut Game) -> EnemyScript => ea;
    call (k, j, game); default { match crate::game::mother_brain_body::dispatch_instruction(ea, j, game) { Some(script) => script, None => { crate::rtl::Unreachable(game); EnemyScript::Stop } } };
    @functions [pub unsafe fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [11136208] sub_A9ECD0 [] {
        let mut v2: u16 = (0xa9d910i32) as u16;
        let E: i32 = k as i32;
        if slot!(game, Enemy_DeadMonsters[E].var_08) != 0 {
            v2 = (0xa9da08i32) as u16;
        }
        slot!(game, Enemy_DeadMonsters[E].var_A = v2);
        jp
    };
    @functions [pub unsafe fn (_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [11139476] Shitroid_Instr_5 [] {
        if (word!(game.random_number) & 0x8000) == 0 {
            return EnemyScript::jump(jp.word(0));
        }
        QueueSfx2_Max6(0x52, game);
        jp.offset(1)
    };
    @functions [pub unsafe fn (_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [11115208] MotherBrain_Instr_SpawnEprojToOffset [jp.offset(3)] {
        let E: i32 = 0;
        game.eproj_spawn_pt = Point16U {
            x: ((slot!(game, Enemy_MotherBrain[E].base.x_pos)).wrapping_add(jp.word(0))),
            y: ((slot!(game, Enemy_MotherBrain[E].base.y_pos)).wrapping_add(jp.word(2))),
        };
        SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, jp.word(4), game);
    };
    [11115247] MotherBrain_Instr_SpawnDeathBeamEproj [jp] {
        let spawn_parameter = crate::game::music::queue_sfx(1, 0x63, 6, game);
        SpawnEprojWithGfx(
            spawn_parameter,
            0x40,
            addr_kEproj_MotherBrainDeathBeamCharging as u16,
            game,
        );
    };
    [11115269] MotherBrain_Instr_IncrBeamAttackPhase [jp] {
        let E: i32 = 0;
        slot!(game, Enemy_MotherBrain[E].var_17 = slot!(game, Enemy_MotherBrain[E].var_17).wrapping_add(1));
    };
}
