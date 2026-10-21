use crate::game::beam_windows::calculate_xray_window;
use crate::game::callback_tables::callback_table;
pub use crate::game::constants::bank_86::*;
pub use crate::game::constants::bank_87::*;
pub use crate::game::constants::bank_8f::*;
pub use crate::game::constants::samus_poses::*;
use crate::game::explosion_window::{self, Phase};
use crate::game::sm_80::{
    CheckBossBitForCurArea, CheckEventHappened, HandleMusicQueue, QueueMusic_Delayed8,
    QueueSfx1_Max15, QueueSfx1_Max6, QueueSfx2_Max6, SetEventHappened,
};
use crate::game::sm_84::SpawnHardcodedPlm;
use crate::game::sm_86::SpawnEprojWithRoomGfx;
use crate::game::sm_87::SpawnAnimtiles;
use crate::game::sm_8b::ConfigureTitleSequenceGradientHDMA;
use crate::game::sm_90::CallSomeSamusCode;
use crate::game::sm_91::{
    CanXrayShowBlocks, HdmaobjPreInstr_XraySetup, InitializeSuitPickupHdma,
    ResponsibleForXrayStandupGlitch, Samus_LoadSuitPalette, Xray_SetupStage1_FreezeTimeBackup,
    Xray_SetupStage2_ReadBg1_2ndScreen, Xray_SetupStage3_ReadBg1_1stScreen, Xray_SetupStage4,
    Xray_SetupStage5, Xray_SetupStage6, Xray_SetupStage7, Xray_SetupStage8_SetBackdropColor,
};
use crate::game::sm_94::kSinCosTable8bit_Sext;
use crate::game::sm_a9::MotherBrain_CalculateRainbowBeamHdma;
use crate::game_state::Game;
use crate::ram_map::ram_ptr;
use crate::ram_map::word;
use crate::rtl::rom_ptr;
use crate::rtl::words;
use crate::rtl::{mov24, Load24, Mult8x8, RomPtr, Unreachable, WriteReg, WriteRegWord};
use crate::rtl::{AddToHiLo, SetHiLo};
use crate::set_fields;
pub use crate::snes::registers::*;
use ::core::ffi::{c_long, c_void};
use ::core::ptr::{null, null_mut};

pub use crate::types::EnemyData;
pub use crate::types::{
    Func_U8, Func_V, Func_Y_V, Func_Y_Y, LongPtr, SnesRegs, VoidP, VramWriteEntry,
};
crate::typed_struct! {
    #[derive(Copy, Clone)]
    #[repr(C, packed)]
    pub struct HdmaScrollEntry {
        pub top_pos, scroll_subspeed, scroll_speed: u16;
        pub hdma_data_table_entry: VoidP;
    }
}
pub use crate::types::Consts_87;
pub use crate::types::Consts_88;
pub use crate::types::{Consts_86, ExpandingSquareTransitionHdma, Ram4000, Ram4000_Backups};
pub use crate::types::{SamusPose, SpawnHdmaObject_Args};
crate::game::constants::typed_constants! { Consts_88;
    addr_word_88DF71 = 57201, addr_word_88DF6B = 57195,
    addr_word_88DF65 = 57189, addr_word_88DF5F = 57183,
    addr_off_88ADA6 = 44454, addr_off_88AD9C = 44444,
    addr_word_88A8E8 = 43240,
    addr_loc_889180 = 37248,
    addr_locret_88858A = 34186,
}
pub use crate::types::Consts_89;
pub use crate::types::Consts_8F;
pub use crate::types::SpawnHardcodedPlmArgs;
pub const NULL: *mut c_void = null_mut::<c_void>();
crate::game::constants::typed_constants! { i32;
    true_0 = 1, false_0 = 0,
}
const RomPtr_88: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x88>;
#[inline]
unsafe fn RomBankBase(bank: u8, game: &mut Game) -> *const u8 {
    RomPtr((((bank as i32) << 16i32) + 0x8000i32) as u32, game).offset(-(0x8000i32 as isize))
}
crate::ram_map::ram_pointer_functions! {
    gVramWriteEntry(a: i32) -> VramWriteEntry = [(0xd0 + a) as isize];
}
crate::game::constants::typed_constants! { i32;
    fnDoorTransitionFunction_FadeOutScreen = 0x82e2db, fnDoorTransition_FadeInScreenAndFinish = 0x82e737,
    fnPauseHook_Empty = 0x8883e1,
}
crate::game::constants::typed_constants! { u32;
    fnHdmaobjInstr_InitMorphBallEyeBeamHdma = 0x88e917,
    fnHdmaobjPreInstr_MorphBallEyeBeamWidening = 0x88e9e6, fnHdmaobjPreInstr_MorphBallEyeBeamFull = 0x88ea3c,
    fnHdmaobjPreInstr_MorphBallEyeBeamFading = 0x88eacb,
}
crate::game::constants::typed_constants! { i32;
    fnCinematicFunctionOpening = 0x8b9b68,
}
pub fn nullsub_9(j: u16, game: &mut Game) -> u16 {
    j
}
pub unsafe fn InitializeLayerBlending(game: &mut Game) {
    *ram_ptr!(game; 0x60) = 0;
    *ram_ptr!(game; 0x61) = 0;
    *ram_ptr!(game; 0x62) = 0;
    *ram_ptr!(game; 0x69) = 19;
    *ram_ptr!(game; 0x6b) = 4;
    *ram_ptr!(game; 0x6c) = 0;
    *ram_ptr!(game; 0x6d) = 0;
    *ram_ptr!(game; 0x6e) = 2;
    *ram_ptr!(game; 0x71) = 51;
}
pub unsafe fn LayerBlendFunc_4_PhantoonIntro(j: u16, game: &mut Game) -> u16 {
    *ram_ptr!(game; 0x69) = 17;
    *ram_ptr!(game; 0x6b) = 4;
    j
}
pub unsafe fn LayerBlendFunc_6(j: u16, game: &mut Game) -> u16 {
    *ram_ptr!(game; 0x6b) = 20;
    *ram_ptr!(game; 0x71) = 39;
    j
}
pub unsafe fn LayerBlendFunc_8(j: u16, game: &mut Game) -> u16 {
    *ram_ptr!(game; 0x6b) = 20;
    *ram_ptr!(game; 0x71) = 34;
    j
}
pub unsafe fn LayerBlendFunc_A(j: u16, game: &mut Game) -> u16 {
    *ram_ptr!(game; 0x71) = 50;
    j
}
pub unsafe fn LayerBlendFunc_C(j: u16, game: &mut Game) -> u16 {
    *ram_ptr!(game; 0x6e) = 0;
    *ram_ptr!(game; 0x71) = -94i32 as u8;
    j
}
pub use self::nullsub_9 as nullsub_10;
pub unsafe fn LayerBlendFunc_10(j: u16, game: &mut Game) -> u16 {
    *ram_ptr!(game; 0x61) = 2;
    *ram_ptr!(game; 0x62) = 32;
    *ram_ptr!(game; 0x6d) = 4;
    j
}
pub unsafe fn LayerBlendFunc_14(j: u16, game: &mut Game) -> u16 {
    *ram_ptr!(game; 0x71) = -77i32 as u8;
    j
}
pub unsafe fn LayerBlendFunc_16(mut j: u16, game: &mut Game) -> u16 {
    byte!(j; 0) = 4;
    *ram_ptr!(game; 0x69) = 17;
    *ram_ptr!(game; 0x6b) = 6;
    *ram_ptr!(game; 0x71) = -79i32 as u8;
    j
}
pub unsafe fn LayerBlendFunc_1A(mut j: u16, game: &mut Game) -> u16 {
    byte!(j; 0) = 4;
    *ram_ptr!(game; 0x69) = 21;
    *ram_ptr!(game; 0x6b) = 2;
    *ram_ptr!(game; 0x71) = 53;
    j
}
pub unsafe fn LayerBlendFunc_1C(j: u16, game: &mut Game) -> u16 {
    *ram_ptr!(game; 0x69) = 21;
    *ram_ptr!(game; 0x6b) = 2;
    *ram_ptr!(game; 0x71) = 85;
    j
}
pub unsafe fn LayerBlendFunc_18(mut j: u16, game: &mut Game) -> u16 {
    j = 2;
    if (*ram_ptr!(game; 0x84) & 0x30) == 0x30 {
        *ram_ptr!(game; 0x71) = 36;
        *ram_ptr!(game; 0x6b) = 19;
        *ram_ptr!(game; 0x69) = 4;
    }
    j
}
pub use self::nullsub_9 as nullsub_11;
pub unsafe fn LayerBlendFunc_26(j: u16, game: &mut Game) -> u16 {
    *ram_ptr!(game; 0x71) = 119;
    j
}
pub unsafe fn LayerBlendFunc_28(j: u16, game: &mut Game) -> u16 {
    *ram_ptr!(game; 0x6e) = 0;
    *ram_ptr!(game; 0x71) = -77i32 as u8;
    if (word!(game.fx_layer_blending_config_c) & 0x8000) == 0 {
        *ram_ptr!(game; 0x74, 0) = 37;
        *ram_ptr!(game; 0x74, 1) = 64;
        *ram_ptr!(game; 0x74, 2) = 0x80;
    }
    j
}
pub unsafe fn LayerBlendFunc_2A(j: u16, game: &mut Game) -> u16 {
    *ram_ptr!(game; 0x6e) = 0;
    *ram_ptr!(game; 0x71) = -77i32 as u8;
    if (word!(game.fx_layer_blending_config_c) & 0x8000) == 0 {
        *ram_ptr!(game; 0x74, 0) = 38;
        *ram_ptr!(game; 0x74, 1) = 66;
        *ram_ptr!(game; 0x74, 2) = 0x80;
    }
    j
}
pub unsafe fn LayerBlendFunc_2C(j: u16, game: &mut Game) -> u16 {
    *ram_ptr!(game; 0x6e) = 0;
    j
}
pub use self::LayerBlendFunc_14 as LayerBlendFunc_2E;
pub unsafe fn LayerBlendFunc_32(j: u16, game: &mut Game) -> u16 {
    *ram_ptr!(game; 0x6b) = 68;
    *ram_ptr!(game; 0x71) = -78i32 as u8;
    j
}
pub unsafe fn LayerBlendFunc_34(mut j: u16, game: &mut Game) -> u16 {
    j = 6;
    j
}
pub unsafe fn LayerBlendFunc_24(j: u16, game: &mut Game) -> u16 {
    *ram_ptr!(game; 0x60) = 0;
    *ram_ptr!(game; 0x61) = 2;
    *ram_ptr!(game; 0x62) = 32;
    *ram_ptr!(game; 0x69) = 19;
    *ram_ptr!(game; 0x6b) = 4;
    *ram_ptr!(game; 0x6c) = 19;
    *ram_ptr!(game; 0x6d) = 4;
    *ram_ptr!(game; 0x6e) = 16;
    *ram_ptr!(game; 0x71) = 51;
    j
}
pub unsafe fn HandleLayerBlendingXrayCanShowBlocks(game: &mut Game) {
    *ram_ptr!(game; 0x60) = -56i32 as u8;
    *ram_ptr!(game; 0x61) = 8;
    *ram_ptr!(game; 0x62) = 0x80;
    *ram_ptr!(game; 0x69) = 19;
    *ram_ptr!(game; 0x6b) = 4;
    *ram_ptr!(game; 0x6c) = 3;
    *ram_ptr!(game; 0x6d) = 4;
    *ram_ptr!(game; 0x6e) = 34;
    *ram_ptr!(game; 0x71) = (*ram_ptr!(game; 0x71) & 0x80 | 0x73);
}
pub unsafe fn HandleLayerBlendingXrayCantShowBlocks(game: &mut Game) {
    *ram_ptr!(game; 0x60) = 0;
    *ram_ptr!(game; 0x61) = 8;
    *ram_ptr!(game; 0x62) = 0x80;
    *ram_ptr!(game; 0x69) = 19;
    *ram_ptr!(game; 0x6b) = 4;
    *ram_ptr!(game; 0x6c) = 3;
    *ram_ptr!(game; 0x6d) = 4;
    *ram_ptr!(game; 0x6e) = 34;
    *ram_ptr!(game; 0x71) = (*ram_ptr!(game; 0x71) & 0x80 | 0x61);
    if word!(game.room_ptr) as i32 == addr_kRoom_cefb as i32 {
        *ram_ptr!(game; 0x69) = 17;
    }
}
pub unsafe fn HandleLayerBlendingXrayFirefleaRoom(game: &mut Game) {
    *ram_ptr!(game; 0x60) = 0;
    *ram_ptr!(game; 0x61) = 8;
    *ram_ptr!(game; 0x62) = 0x80;
    *ram_ptr!(game; 0x69) = 19;
    *ram_ptr!(game; 0x6b) = 0;
    *ram_ptr!(game; 0x6c) = 3;
    *ram_ptr!(game; 0x6d) = 4;
    *ram_ptr!(game; 0x6e) = 32;
    *ram_ptr!(game; 0x71) = -77i32 as u8;
}
pub unsafe fn LayerBlendPowerBombFunc_0(game: &mut Game) {
    *ram_ptr!(game; 0x60) = 0;
    *ram_ptr!(game; 0x61) = 8;
    *ram_ptr!(game; 0x62) = 0x80;
    *ram_ptr!(game; 0x6e) = 2;
    *ram_ptr!(game; 0x71) = 55;
    *ram_ptr!(game; 0x6c) = 0;
    *ram_ptr!(game; 0x6d) = 4;
    *ram_ptr!(game; 0x69) = 19;
    *ram_ptr!(game; 0x6b) = 4;
}
pub unsafe fn LayerBlendPowerBombFunc_4(game: &mut Game) {
    *ram_ptr!(game; 0x60) = 0x80;
    *ram_ptr!(game; 0x61) = 8;
    *ram_ptr!(game; 0x62) = 0x80;
    *ram_ptr!(game; 0x6e) = 2;
    *ram_ptr!(game; 0x71) = 55;
    *ram_ptr!(game; 0x6c) = 0;
    *ram_ptr!(game; 0x6d) = 6;
    *ram_ptr!(game; 0x69) = 17;
    *ram_ptr!(game; 0x6b) = 6;
}
pub unsafe fn LayerBlendPowerBombFunc_6(game: &mut Game) {
    *ram_ptr!(game; 0x60) = 0;
    *ram_ptr!(game; 0x61) = 8;
    *ram_ptr!(game; 0x62) = 0x80;
    *ram_ptr!(game; 0x6e) = 2;
    *ram_ptr!(game; 0x71) = 49;
    *ram_ptr!(game; 0x6c) = 0;
    *ram_ptr!(game; 0x6d) = 4;
    *ram_ptr!(game; 0x69) = 19;
    *ram_ptr!(game; 0x6b) = 4;
}
static kLayerBlendFuncTable: [Func_Y_Y; 27] = callback_table! { nonnull Func_Y_Y;
    nullsub_9, nullsub_9, LayerBlendFunc_4_PhantoonIntro, LayerBlendFunc_6, LayerBlendFunc_8,
    LayerBlendFunc_A, LayerBlendFunc_C, nullsub_10, LayerBlendFunc_10, LayerBlendFunc_10,
    LayerBlendFunc_14, LayerBlendFunc_16, LayerBlendFunc_18, LayerBlendFunc_1A,
    LayerBlendFunc_1C, LayerBlendFunc_18, nullsub_11, LayerBlendFunc_14, LayerBlendFunc_24,
    LayerBlendFunc_26, LayerBlendFunc_28, LayerBlendFunc_2A, LayerBlendFunc_2C,
    LayerBlendFunc_2E, LayerBlendFunc_18, LayerBlendFunc_32, LayerBlendFunc_34,
};
pub unsafe fn LayerBlendingHandler(game: &mut Game) {
    let mut v0: u16 = 0;
    let v1: u8 = word!(game.fx_layer_blending_config_c) as u8;
    if word!(game.fx_layer_blending_config_c) as u8 != 0 {
        InitializeLayerBlending(game);
        v0 = kLayerBlendFuncTable[(v1 >> 1) as usize](v0, game);
    }
    if (word!(game.fx_layer_blending_config_c) & 0x8000) == 0 {
        if (word!(game.fx_layer_blending_config_c) & 0x4000) != 0 {
            HandleLayerBlendingXrayCanShowBlocks(game);
        } else if (word!(game.fx_layer_blending_config_c) & 0x2000) != 0 {
            HandleLayerBlendingXrayCantShowBlocks(game);
        } else if (word!(game.fx_layer_blending_config_c) & 0x1000) != 0 {
            HandleLayerBlendingXrayFirefleaRoom(game);
        }
    } else {
        HandleLayerBlendingPowerBomb(v0, game);
    };
}
static kLayerBlendPowerBombFuncs: [Func_V; 4] = callback_table! { nonnull Func_V;
    LayerBlendPowerBombFunc_0, LayerBlendPowerBombFunc_0, LayerBlendPowerBombFunc_4,
    LayerBlendPowerBombFunc_6,
};
pub unsafe fn HandleLayerBlendingPowerBomb(mut j: u16, game: &mut Game) {
    if word!(game.room_ptr) as i32 == addr_kRoom_a66a as i32 {
        j = 6;
    }
    kLayerBlendPowerBombFuncs[(j >> 1) as usize](game);
}
pub unsafe fn EnableHdmaObjects(game: &mut Game) {
    word!(game.hdma_objects_enable_flag, |fresh31| (fresh31)
        | (0x8000u16));
}
pub unsafe fn DisableHdmaObjects(game: &mut Game) {
    word!(game.hdma_objects_enable_flag, |fresh30| (fresh30
        & !(0x8000)));
}
pub unsafe fn WaitUntilEndOfVblankAndClearHdma(game: &mut Game) {
    crate::rtl::register_writes! { game;
        byte(MDMAEN as u16, 0); byte(HDMAEN as u16, 0);
    }
    sub_8882AC(game);
}
pub unsafe fn sub_8882AC(game: &mut Game) {
    *ram_ptr!(game; 0x85) = 0;
    word_transfers!(game, descending, 6; le(hdma_object_channels_bitmask, 5) <- value(0));
}
pub unsafe fn InitializeSpecialEffectsForNewRoom(game: &mut Game) {
    word! { game, 0; earthquake_sfx_index, earthquake_sfx_timer }
    if word!(game.room_ptr) as i32 == addr_kRoom_9804 as i32
        || word!(game.room_ptr) as i32 == addr_kRoom_96ba as i32
        || word!(game.room_ptr) as i32 == addr_kRoom_b32e as i32
        || word!(game.room_ptr) as i32 == addr_kRoom_b457 as i32
        || word!(game.room_ptr) as i32 == addr_kRoom_dd58 as i32
        || word!(game.room_ptr) as i32 == addr_kRoom_dede as i32
    {
        word!(game.earthquake_sfx_timer = -1i32 as u16);
    }
    word!(game.debug_disable_minimap = 0);
    for i in (32i32..128).step_by(16) {
        crate::rtl::register_writes! { game;
            byte((i + 17152) as SnesRegs as u16, 0); byte((i + 17153) as SnesRegs as u16, 0x13);
            byte((i + 17154) as SnesRegs as u16, 0); byte((i + 17155) as SnesRegs as u16, 0);
            byte((i + 17156) as SnesRegs as u16, 0); byte((i + 17157) as SnesRegs as u16, 0);
            byte((i + 17158) as SnesRegs as u16, 0); byte((i + 17160) as SnesRegs as u16, 0);
            byte((i + 17161) as SnesRegs as u16, 0);
        }
    }
    word! {game; fx_y_subpos = 0; fx_y_pos = -1i32 as u16; lava_acid_y_subpos = 0;
    lava_acid_y_pos = -1i32 as u16; }
    word! { game, 0;
        hud_bg3_xpos, hud_bg3_ypos, bg3_xpos, bg3_ypos, irq_enable_mode7, camera_distance_index,
        tourian_entrance_statue_animstate, tourian_entrance_statue_finished, earthquake_timer,
        phantom_related_layer_flag, power_bomb_explosion_status, power_bomb_flag,
        power_bomb_explosion_radius, fx_tilemap_ptr, fx_type,
    }
    word!(game.fx_base_y_subpos = 0x8000);
    word! { game, 0;
        fx_base_y_pos, fx_target_y_pos, fx_y_vel, fx_liquid_options, fx_timer, tide_phase,
        fx_y_suboffset, fx_y_offset,
    }
    word!(game.fx_layer_blending_config_a = 2);
    word! { game, 0;
        reg_BG3HOFS, reg_BG3VOFS, layer2_x_pos, layer2_y_pos, room_loading_irq_handler,
    }
    *((&raw mut (*(ram_ptr!(game, LongPtr; 0x601))).addr as *mut u8).offset(1) as *mut VoidP) =
        -30720i32 as VoidP;
    *((&raw mut (*(ram_ptr!(game, LongPtr; 0x604))).addr as *mut u8).offset(1) as *mut VoidP) =
        -30720i32 as VoidP;
    (*(ram_ptr!(game, LongPtr; 0x601))).addr = ((0x8883e1i32) as u16);
    (*(ram_ptr!(game, LongPtr; 0x604))).addr = ((0x8883e1i32) as u16);
    crate::rtl::register_writes! { game;
        byte(WMADDL as u16, 0xf0); byte(WMADDM as u16, 0xff);
        byte(WMADDH as u16, 1);
    }
    *ram_ptr!(game; 0x85) = 0;
    *ram_ptr!(game; 0x74, 0) = 32;
    *ram_ptr!(game; 0x74, 1) = 64;
    *ram_ptr!(game; 0x74, 2) = 0x80;
    *ram_ptr!(game; 0x57) = 0;
    *ram_ptr!(game; 0x69) = 19;
    *ram_ptr!(game; 0x6f) = 0;
    *ram_ptr!(game; 0x72) = 0;
    *ram_ptr!(game; 0x5d) = 0;
    *ram_ptr!(game; 0x5e) = 4;
    *ram_ptr!(game; 0x59) = 73;
    *ram_ptr!(game; 0x5a) = 90;
    *ram_ptr!(game; 0x5b) = 90;
}
unsafe fn SpawnHdmaObjectInner(
    k: u16,
    p: *mut u16,
    r18: u16,
    r20: u16,
    r24: u16,
    game: &mut Game,
) -> u16 {
    let v2: i32 = k as i32 >> 1;
    word!(game.hdma_object_pre_instructions[v2] = ((0x8884b8i32) as u16));
    word!(game.hdma_object_pre_instruction_bank[v2] = 136);
    word!(game.hdma_object_instruction_list_pointers[v2] = *p.offset(1));
    word!(game.hdma_object_instruction_timers[v2] = 1);
    word! { game, 0;
        hdma_object_timers[v2], hdma_object_A[v2], hdma_object_B[v2], hdma_object_C[v2],
        hdma_object_D[v2],
    }
    word!(game.hdma_object_channels_bitmask[v2] = u16::swap_bytes(r18));
    word!(game.hdma_object_bank_slot[v2] = (r24) | (r20));
    WriteRegWord((r20 as i32 + DMAP0 as i32) as SnesRegs as u16, *p, game);
    k
}
pub unsafe fn SpawnHdmaObject(db: u8, p: *const c_void, game: &mut Game) -> u16 {
    let mut r18: u16 = 1024;
    let mut r20: u16 = 32;
    let mut v3: u16 = 0;
    loop {
        if word!(game.hdma_object_channels_bitmask[(v3 >> 1)]) == 0 {
            return SpawnHdmaObjectInner(
                v3,
                p as *mut u16,
                r18,
                r20,
                ((db as i32) << 8) as u16,
                game,
            );
        }
        let v4: bool = r18 as i32 >> 15 != 0;
        r18 = (r18).wrapping_mul(2);
        if v4 {
            return -1i32 as u16;
        }
        r20 = (r20).wrapping_add(16);
        v3 = (v3).wrapping_add(2);
        if v3 == 12 {
            return -1i32 as u16;
        }
    }
}
pub unsafe fn SpawnHdmaObjectToSlot0xA(db: u8, p: *const c_void, game: &mut Game) {
    SpawnHdmaObjectInner(
        0xa,
        p as *mut u16,
        0x8000,
        0x70,
        ((db as i32) << 8) as u16,
        game,
    );
}
pub unsafe fn HdmaObjectHandler(game: &mut Game) {
    HandleMusicQueue(game);
    if word!(game.time_is_frozen_flag) == 0
        && (word!(game.power_bomb_explosion_status) & 0x4000) != 0
    {
        word!(game.power_bomb_explosion_status = 0x8000);
        static unk_8884D5: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0x40, 0x28, 0x8ace as VoidP);
        static unk_8884DD: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0x40, 0x29, 0x8b80 as VoidP);
        SpawnHdmaObject(0x88, &raw const unk_8884D5 as *const c_void, game);
        SpawnHdmaObject(0x88, &raw const unk_8884DD as *const c_void, game);
    }
    word!(game.fx_layer_blending_config_c = word!(game.fx_layer_blending_config_a));
    if (word!(game.hdma_objects_enable_flag) & 0x8000) != 0 {
        *ram_ptr!(game; 0x85) = 0;
        let mut i: i32 = 0;
        do_until! { (i == 12);
            word!(game.hdma_object_index = i as u16);
            let mut v1: i8 = byte!(@pointer ram_ptr!(game, u16; 0x18b4); i as isize) as i8;
            if v1 != 0 {
                crate::update_place! { reference fresh58 = *ram_ptr!(game; 0x85) => (*fresh58) | ((v1) as u8); }
                HdmaobjInstructionHandler(i as u8, game);
                i = word!(game.hdma_object_index) as i32;
            }
            i += 2;
        }
        LayerBlendingHandler(game);
    }
}
pub unsafe fn HdmaobjInstructionHandler(k: u8, game: &mut Game) {
    let kh: i32 = k as i32 >> 1;
    CallHdmaobjPreInstr(
        (words!(
            i32,
            (
                word!(game.hdma_object_pre_instruction_bank[kh]),
                word!(game.hdma_object_pre_instructions[kh])
            )
        )) as u32,
        k as u16,
        game,
    );
    crate::update_place! { unaligned_word fresh59 [fresh60] = &raw mut (*(ram_ptr!(game, u16; 0x18e4)).offset(kh as isize)) => (u16::from_le(fresh59.read_unaligned())).wrapping_sub(1); }
    if fresh60 == 1 {
        let base = RomBankBase(
            *(ram_ptr!(game, u16; 0x18c0) as *mut u8)
                .offset(k as i32 as isize)
                .offset(1),
            game,
        );
        let mut p =
            base.offset(word!(game.hdma_object_instruction_list_pointers[kh]) as i32 as isize);
        while *(p as *mut u16) & 0x8000 != 0 {
            p = CallHdmaobjInstr(
                ((*(p as *mut u16)) as u32) | (0x880000u32),
                k as u16,
                p.offset(2),
                game,
            );
            if (p as usize) < 0x10000i32 as usize {
                if p.is_null() {
                    return;
                }
                p = base.offset(p as usize as isize);
            }
        }
        word!(game.hdma_object_instruction_timers[kh] = *(p as *mut u16));
        word!(game.hdma_object_table_pointers[kh] = *(p.offset(2) as *mut u16));
        word!(
            game.hdma_object_instruction_list_pointers[kh] =
                p.offset(4).offset_from(base) as c_long as u16
        );
    }
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, hdp: *const u8, game: &mut Game) -> *const u8;
    HdmaobjInstr_Delete [null::<u8>()] {
        word!(game.hdma_object_channels_bitmask[(k >> 1)] = 0);
    };
    HdmaobjInstr_SetPreInstr [hdp.offset(3)] {
        word!(game.hdma_object_pre_instructions[(k >> 1)] = *(hdp as *mut u16));
        *(ram_ptr!(game, u16; 0x18fc) as *mut u8).offset(k as i32 as isize) = *hdp.offset(2);
    };
    HdmaobjInstr_ClearPreInstr [hdp] {
        word!(game.hdma_object_pre_instructions[(k >> 1)] = addr_locret_88858A as u16);
    };
}
crate::game::callback_dispatch! {
    pub unsafe fn CallHdmaobjInstrFunc(ea: u32, k: u16, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [8972135] InitializeRainbowBeam [] {
        *ram_ptr!(game; 0x74, 0) = 32;
        *ram_ptr!(game; 0x74, 1) = 71;
        *ram_ptr!(game; 0x74, 2) = 0x8f;
        word!(game.fx_layer_blending_config_c = 36);
        *ram_ptr!(game; 0x9c00, 0) = 100;
        *(ram_ptr!(game, u16; 0x9c00, 1)) = -25344i32 as u16;
        *(ram_ptr!(game, u16; 0x9c00, 3)) = 0;
        word! {game; hdma_table_2[0] = 0; hdma_table_2[1] = -32736i32 as u16;
            hdma_table_2[2] = 0; }
        MotherBrain_CalculateRainbowBeamHdma(game);
    };
    @external {
        8948500 => explosion_window::begin_pre_flash; 8948530 => explosion_window::begin_prescaled_pre_flash;
        8948537 => explosion_window::begin_explosion; 8948551 => explosion_window::begin_prescaled_explosion;
        8948558 => [explosion_window::finish_power_bomb(k, game)]; 8954596 => explosion_window::begin_crystal_flash;
        8954633 => explosion_window::begin_explosion; 8954647 => [explosion_window::finish_crystal_flash(k, game)];
        8970129 => [()]; 8970130 => [()]; 8972070 => [()];
        9554681 => [Xray_SetupStage1_FreezeTimeBackup(k, game)];
        9554716 => Xray_SetupStage2_ReadBg1_2ndScreen;
        9554775 => Xray_SetupStage3_ReadBg1_1stScreen; 9554830 => Xray_SetupStage4;
        9556179 => Xray_SetupStage5; 9556339 => Xray_SetupStage6; 9556384 => Xray_SetupStage7;
        9556668 => Xray_SetupStage8_SetBackdropColor; 9557650 => InitializeSuitPickupHdma;
    }
}
pub unsafe fn HdmaobjInstr_CallFarFunc(k: u16, hdp: *const u8, game: &mut Game) -> *const u8 {
    CallHdmaobjInstrFunc(Load24(hdp as *mut LongPtr), k, game);
    hdp.offset(3)
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, hdp: *const u8) -> *const u8;
    HdmaobjInstr_Goto [*(hdp as *mut u16) as usize as *const u8] { };
    HdmaobjInstr_GotoRel [hdp.offset(*hdp as i8 as i32 as isize)] { };
}
pub unsafe fn HdmaobjInstr_DecrementAndGoto(k: u16, hdp: *const u8, game: &mut Game) -> *const u8 {
    let v2: i32 = k as i32 >> 1;
    crate::update_place! { unaligned_word fresh63 [fresh64] = &raw mut (*(ram_ptr!(game, u16; 0x1908)).offset(v2 as isize)) => (u16::from_le(fresh63.read_unaligned())).wrapping_sub(1); }
    if fresh64 == 1 {
        hdp.offset(2)
    } else {
        *(hdp as *mut u16) as usize as *const u8
    }
}
pub unsafe fn HdmaobjInstr_DecrementAndGotoRel(
    k: u16,
    hdp: *const u8,
    game: &mut Game,
) -> *const u8 {
    let v2: i32 = k as i32 >> 1;
    crate::update_place! { unaligned_word fresh61 [fresh62] = &raw mut (*(ram_ptr!(game, u16; 0x1908)).offset(v2 as isize)) => (u16::from_le(fresh61.read_unaligned())).wrapping_sub(1); }
    if fresh62 == 1 {
        hdp.offset(1)
    } else {
        hdp.offset(*hdp as i8 as i32 as isize)
    }
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, hdp: *const u8, game: &mut Game) -> *const u8;
    HdmaobjInstr_SetTimer [hdp.offset(1)] {
        *(ram_ptr!(game, u16; 0x1908) as *mut u8).offset(k as i32 as isize) = *hdp;
    };
    HdmaobjInstr_SetHdmaControl [hdp.offset(1)] {
        WriteReg(
            (*((ram_ptr!(game, u16; 0x18c0)).offset((k >> 1) as isize) as *mut u8) as i32
                + DMAP0 as i32) as SnesRegs as u16,
            *hdp,
            game,
        );
    };
    HdmaobjInstr_SetHdmaTarget [hdp.offset(1)] {
        WriteReg(
            (*((ram_ptr!(game, u16; 0x18c0)).offset((k >> 1) as isize) as *mut u8) as i32
                + BBAD0 as i32) as SnesRegs as u16,
            *hdp,
            game,
        );
    };
    HdmaobjInstr_SetHdmaTablePtr [hdp.offset(2)] {
        word!(game.hdma_object_table_pointers[(k >> 1)] = *(hdp as *mut u16));
    };
    HdmaobjInstr_SetHdmaTableBank [hdp.offset(1)] {
        WriteReg(
            (*((ram_ptr!(game, u16; 0x18c0)).offset((k >> 1) as isize) as *mut u8) as i32
                + A1B0 as i32) as SnesRegs as u16,
            *hdp,
            game,
        );
    };
}
pub unsafe fn HdmaobjInstr_SetIndirectHdmaDataBank(
    k: u16,
    hdp: *const u8,
    game: &mut Game,
) -> *const u8 {
    WriteReg(
        (byte!(@pointer (ram_ptr!(game, u16; 0x18c0)).offset((k >> 1) as isize); 0) as i32
            + DAS00 as i32) as SnesRegs as u16,
        *hdp,
        game,
    );
    hdp.offset(1)
}
pub unsafe fn HdmaobjInstr_Sleep(k: u16, hdp: *const u8, game: &mut Game) -> *const u8 {
    let base = RomBankBase(
        *(ram_ptr!(game, u16; 0x18c0) as *mut u8)
            .offset(k as i32 as isize)
            .offset(1),
        game,
    );
    word!(
        game.hdma_object_instruction_list_pointers[(k >> 1)] =
            (hdp.offset_from(base) as c_long - 2 as c_long) as u16
    );
    null::<u8>()
}
pub unsafe fn RaiseOrLowerFx(game: &mut Game) -> u8 {
    if (word!(game.fx_target_y_pos) & 0x8000) != 0 {
        return 1;
    }
    let r24_r22: u32 = ((word!(game.fx_y_vel) as i16 as i32) << 8) as u32;
    if (word!(game.fx_y_vel) & 0x8000) == 0 {
        let mut v2: u16 =
            (r24_r22.wrapping_add(words!(game.fx_base_y_pos, fx_base_y_subpos)) >> 16) as u16;
        word!(
            game.fx_base_y_subpos,
            |fresh13| (fresh13 as u32).wrapping_add(r24_r22) as u16
        );
        if (v2 & 0x8000) != 0 {
            v2 = -1i32 as u16;
        }
        word!(game.fx_base_y_pos = v2);
        if word!(game.fx_target_y_pos) >= v2 {
            0
        } else {
            word! {game; fx_base_y_pos = word!(game.fx_target_y_pos); fx_base_y_subpos = 0; }
            1
        }
    } else {
        let mut v0: u16 =
            (r24_r22.wrapping_add(words!(game.fx_base_y_pos, fx_base_y_subpos)) >> 16) as u16;
        word!(
            game.fx_base_y_subpos,
            |fresh14| (fresh14 as u32).wrapping_add(r24_r22) as u16
        );
        if (v0 & 0x8000) != 0 {
            v0 = 0;
        }
        word!(game.fx_base_y_pos = v0);
        if word!(game.fx_target_y_pos) >= v0 {
            word! {game; fx_base_y_pos = word!(game.fx_target_y_pos); fx_base_y_subpos = 0; }
        }
        (word!(game.fx_target_y_pos) >= v0) as u8
    }
}
pub unsafe fn HdmaobjPreInstr_XrayFunc0_NoBeam(k: u16, game: &mut Game) {
    if (word!(game.button_config_run_b) & word!(game.joypad1_lastkeys)) != 0 {
        CalculateXrayHdmaTable(game);
        word!(game.demo_input_pre_instr, wrapping_add(1));
    } else {
        word!(game.demo_input_pre_instr = 3);
    };
}
pub unsafe fn HdmaobjPreInstr_XrayFunc1_BeamWidening(k: u16, game: &mut Game) {
    if (word!(game.button_config_run_b) & word!(game.joypad1_lastkeys)) != 0 {
        words!(game.demo_input_instr_timer, demo_input_instr_ptr += 2048);
        words!(
            game.demo_input,
            demo_input_new += words!(game.demo_input_instr_timer, demo_input_instr_ptr)
        );
        if word!(game.demo_input) as i32 - 11i32 & 0x8000 == 0 {
            word! {game; demo_input_new = 0; demo_input = 10; }
            word!(game.demo_input_pre_instr, wrapping_add(1));
        }
        CalculateXrayHdmaTable(game);
    } else {
        word!(game.demo_input_pre_instr = 3);
    };
}
pub unsafe fn HdmaobjPreInstr_XrayFunc2_FullBeam(k: u16, game: &mut Game) {
    if (word!(game.button_config_run_b) & word!(game.joypad1_lastkeys)) != 0 {
        HandleMovingXrayUpDown(game);
        CalculateXrayHdmaTable(game);
    } else {
        word!(game.demo_input_pre_instr, wrapping_add(1));
    };
}
pub unsafe fn HandleMovingXrayUpDown(game: &mut Game) {
    if (word!(game.button_config_up) & word!(game.joypad1_lastkeys)) != 0 {
        MoveXrayUp(game);
    } else if (word!(game.button_config_down) & word!(game.joypad1_lastkeys)) != 0 {
        MoveXrayDown(game);
    }
}
pub unsafe fn MoveXrayUp(game: &mut Game) {
    if word!(game.xray_angle) as i32 - 128i32 & 0x8000 != 0 {
        if word!(game.xray_angle) != word!(game.demo_input)
            && (((word!(game.xray_angle)) as i16).wrapping_sub((word!(game.demo_input)) as i16) < 0
                || {
                    let mut v0: bool = word!(game.xray_angle) != 0;
                    word!(game.xray_angle, wrapping_sub(1));
                    ((word!(game.xray_angle) as i32 - (!v0 as i32 + word!(game.demo_input) as i32))
                        as i16)
                        < 0
                })
        {
            word!(game.xray_angle = word!(game.demo_input));
        }
    } else if (word!(game.demo_input) as i32 + word!(game.xray_angle) as i32) != 256
        && (((word!(game.demo_input) as i32 + word!(game.xray_angle) as i32 - 256i32) as i16) >= 0
            || {
                let mut v1: i16 = ((words!(game.demo_input, xray_angle))
                    .wrapping_add(words!((word!(game.xray_angle), 1u16)))
                    >> 16) as i16;
                word!(game.xray_angle, wrapping_add(1));
                v1 != 256 && (v1).wrapping_sub(256) >= 0
            })
    {
        word!(game.xray_angle = (256u16).wrapping_sub(word!(game.demo_input)));
    }
}
pub unsafe fn MoveXrayDown(game: &mut Game) {
    if word!(game.xray_angle) as i32 - 128i32 & 0x8000 != 0 {
        if (word!(game.demo_input) as i32 + word!(game.xray_angle) as i32) != 128
            && (((word!(game.demo_input) as i32 + word!(game.xray_angle) as i32 - 128i32) as i16)
                >= 0
                || {
                    let mut v0: i16 = ((words!(game.demo_input, xray_angle))
                        .wrapping_add(words!((word!(game.xray_angle), 1u16)))
                        >> 16) as i16;
                    word!(game.xray_angle, wrapping_add(1));
                    v0 != 128 && (v0).wrapping_sub(128) >= 0
                })
        {
            word!(game.xray_angle = (128u16).wrapping_sub(word!(game.demo_input)));
        }
    } else if (word!(game.xray_angle) as i32 - word!(game.demo_input) as i32) != 128
        && (((word!(game.xray_angle) as i32 - word!(game.demo_input) as i32 - 128i32) as i16) < 0
            || {
                let mut v1: bool = word!(game.xray_angle) != 0;
                word!(game.xray_angle, wrapping_sub(1));
                word!(game.xray_angle) as i32 - (!v1 as i32 + word!(game.demo_input) as i32) != 128
                    && ((word!(game.xray_angle) as i32
                        - (!v1 as i32 + word!(game.demo_input) as i32)
                        - 128i32) as i16)
                        < 0
            })
    {
        word!(game.xray_angle = (word!(game.demo_input)).wrapping_add(128));
    }
}
pub unsafe fn CalculateXrayHdmaTable(game: &mut Game) {
    let mut v0: i16 = if *ram_ptr!(game; 0xa1e) == 4 {
        (word!(game.samus_x_pos) as i32 - word!(game.layer1_x_pos) as i32 - 3i32) as i16
    } else {
        (word!(game.samus_x_pos) as i32 - word!(game.layer1_x_pos) as i32 + 3) as i16
    };
    let mut v1: u16 = if *ram_ptr!(game; 0xa1f) == 5 {
        (word!(game.samus_y_pos) as i32 - word!(game.layer1_y_pos) as i32 - 12i32) as u16
    } else {
        (word!(game.samus_y_pos) as i32 - word!(game.layer1_y_pos) as i32 - 16i32) as u16
    };

    let skip_xray_table: bool = if v0 < 0 {
        !(*ram_ptr!(game; 0xa1e) != 4)
    } else {
        if (v0).wrapping_sub(256) < 0 {
            calculate_xray_window(
                v0 as u16,
                v1,
                word!(game.xray_angle),
                word!(game.demo_input),
                false_0 != 0,
                ram_ptr!(game, u16; 0x9800),
                game,
            );
            return;
        }
        !(*ram_ptr!(game; 0xa1e) != 8)
    };
    if skip_xray_table {
    } else {
        calculate_xray_window(
            v0 as u16,
            v1,
            word!(game.xray_angle),
            word!(game.demo_input),
            true_0 != 0,
            ram_ptr!(game, u16; 0x9800),
            game,
        );
        return;
    }
    word_transfers!(game, descending, 256; le(hdma_table_1, 255) <- value(255));
}
pub unsafe fn HdmaobjPreInstr_XrayFunc3_DeactivateBeam(k: u16, game: &mut Game) {
    mov24(ram_ptr!(game, LongPtr; 0xa88), 0x980001);
    *((ram_ptr!(game, u16; 0xa8a) as *mut u8).offset(1) as *mut u16) = 0;
    word! { game, 0;
        demo_input_prev, demo_input_prev_new, demo_backup_prev_controller_input,
    }
    word!(game.hdma_table_1[0] = 255);
    let mut v1: i16 = 4096;
    if word!(game.fx_type) != 36 {
        v1 = 0x2000;
        if CanXrayShowBlocks(game) {
            v1 = 0x4000;
        }
    }
    word!(game.fx_layer_blending_config_c, |fresh133| (fresh133)
        | ((v1) as u16));
    word!(game.palette_buffer[0] = 0);
    let v2: i32 = word!(game.hdma_object_index) as i32 >> 1;
    word!(game.reg_BG2HOFS = word!(game.hdma_object_A[v2]));
    word!(game.reg_BG2VOFS = word!(game.hdma_object_B[v2]));
    *ram_ptr!(game; 0x59) =
        byte!(@pointer ram_ptr!(game, u16; 0x192c); word!(game.hdma_object_index) as i32 as isize);
    let v3: u16 = word!(game.vram_write_queue_tail);
    if ((word!(game.vram_write_queue_tail)) as i16).wrapping_sub(240) < 0 {
        let mut v4: *mut VramWriteEntry =
            gVramWriteEntry(word!(game.vram_write_queue_tail) as i32, game);
        (*v4).size = 2048;
        (*v4).src.addr = ((ram_ptr!(game, Ram4000; 0x4000) as *mut u8)
            .offset_from(&raw mut game.g_ram as *mut u8) as c_long
            + 4096 as c_long) as VoidP;
        *(&raw mut (*v4).src.bank as *mut u16) = 126;
        (*v4).vram_dst = ((*ram_ptr!(game; 0x59) as i32 & 0xfc) << 8) as u16;
        word!(game.vram_write_queue_tail = (v3).wrapping_add(7));
        word!(game.demo_input_pre_instr, wrapping_add(1));
    }
}
pub unsafe fn HdmaobjPreInstr_XrayFunc4_DeactivateBeam(k: u16, game: &mut Game) {
    let mut v1: i16 = 4096;
    if word!(game.fx_type) != 36 {
        v1 = 0x2000;
        if CanXrayShowBlocks(game) {
            v1 = 0x4000;
        }
    }
    word!(game.fx_layer_blending_config_c, |fresh131| (fresh131)
        | ((v1) as u16));
    let v2: u16 = word!(game.vram_write_queue_tail);
    if ((word!(game.vram_write_queue_tail)) as i16).wrapping_sub(240) < 0 {
        let mut v3: *mut VramWriteEntry =
            gVramWriteEntry(word!(game.vram_write_queue_tail) as i32, game);
        (*v3).size = 2048;
        (*v3).src.addr = ((ram_ptr!(game, Ram4000; 0x4000) as *mut u8)
            .offset_from(&raw mut game.g_ram as *mut u8) as c_long
            + 6144 as c_long) as VoidP;
        *(&raw mut (*v3).src.bank as *mut u16) = 126;
        (*v3).vram_dst = (((*ram_ptr!(game; 0x59) as i32 & 0xfc) << 8) + 1024) as u16;
        word!(game.vram_write_queue_tail = (v2).wrapping_add(7));
        word!(game.demo_input_pre_instr, wrapping_add(1));
    }
}
pub unsafe fn HdmaobjPreInstr_XrayFunc5_DeactivateBeam(k: u16, game: &mut Game) {
    let mut v1: i16 = 4096;
    if word!(game.fx_type) != 36 {
        v1 = 0x2000;
        if CanXrayShowBlocks(game) {
            v1 = 0x4000;
        }
    }
    word!(game.fx_layer_blending_config_c, |fresh130| (fresh130)
        | ((v1) as u16));
    if word!(game.time_is_frozen_flag) != 0 {
        word! { game, 0;
            time_is_frozen_flag, demo_input_pre_instr, demo_input_instr_timer, demo_input_instr_ptr,
            xray_angle, demo_input, demo_input_new,
        }
        mov24(ram_ptr!(game, LongPtr; 0xa88), 0x980001);
        *((ram_ptr!(game, u16; 0xa8a) as *mut u8).offset(1) as *mut u16) = 0;
        word! { game, 0;
            demo_input_prev, demo_input_prev_new, demo_backup_prev_controller_input,
        }
        ResponsibleForXrayStandupGlitch(game);
        *(ram_ptr!(game, u16; 0x18b4)).offset((word!(game.hdma_object_index) >> 1) as isize) = 0;
        QueueSfx1_Max6(0xa, game);
        if (word!(game.fx_type) as u8) != 36 {
            *ram_ptr!(game; 0x74, 2) = 0x80;
            *ram_ptr!(game; 0x74, 1) = 64;
            *ram_ptr!(game; 0x74, 0) = 32;
        }
        word_transfers!(game, descending, 256; le(hdma_table_1, 255) <- value(0xff));
        if word!(game.samus_auto_cancel_hud_item_index) != 0 {
            word! { game, 0; hud_item_index, samus_auto_cancel_hud_item_index }
        }
    }
}
static kHdmaobjPreInstr_XrayFuncs: [Func_Y_V; 6] = callback_table! { nonnull Func_Y_V;
    HdmaobjPreInstr_XrayFunc0_NoBeam, HdmaobjPreInstr_XrayFunc1_BeamWidening,
    HdmaobjPreInstr_XrayFunc2_FullBeam, HdmaobjPreInstr_XrayFunc3_DeactivateBeam,
    HdmaobjPreInstr_XrayFunc4_DeactivateBeam, HdmaobjPreInstr_XrayFunc5_DeactivateBeam,
};
pub unsafe fn HdmaobjPreInstr_Xray(k: u16, game: &mut Game) {
    let mut v1: i16 = 4096;
    if word!(game.fx_type) != 36 {
        v1 = 0x2000;
        if CanXrayShowBlocks(game) {
            v1 = 0x4000;
            *(ram_ptr!(game, u16; 0x74, 0)) = 0x27;
            *(ram_ptr!(game, u16; 0x74, 1)) = 0x47;
            *(ram_ptr!(game, u16; 0x74, 2)) = 0x87;
        }
    }
    word!(game.fx_layer_blending_config_c, |fresh138| (fresh138)
        | ((v1) as u16));
    kHdmaobjPreInstr_XrayFuncs[word!(game.demo_input_pre_instr) as usize](
        (2u16).wrapping_mul(word!(game.demo_input_pre_instr)),
        game,
    );
}
pub unsafe fn FxTypeFunc_22_ScrollingSky(game: &mut Game) {
    word! {game; fx_y_pos = 1248; fx_type = 6; }
    static unk_88A62E: SpawnHdmaObject_Args = SpawnHdmaObject_Args::new(0x2, 0x12, 0xad63 as VoidP);
    static unk_88A636: SpawnHdmaObject_Args =
        SpawnHdmaObject_Args::new(0x42, 0x11, 0xad4e as VoidP);
    static unk_88A63E: SpawnHdmaObject_Args = SpawnHdmaObject_Args::new(0x42, 0xd, 0xad39 as VoidP);
    SpawnHdmaObject(0x88, &raw const unk_88A62E as *const c_void, game);
    SpawnHdmaObject(0x88, &raw const unk_88A636 as *const c_void, game);
    SpawnHdmaObject(0x88, &raw const unk_88A63E as *const c_void, game);
    HdmaobjPreInstr_FxType22_BG3Yscroll(0, game);
}
unsafe fn SetupSomeHdmaTablesBG3(
    r24: u16,
    r0: u16,
    r3: u16,
    r6: u16,
    r9: u16,
    game: &mut Game,
) -> u16 {
    word!(game.g_word_7E0598 = 32);
    *(ram_ptr!(game, u16; 0x9c00)) = 31;
    *(ram_ptr!(game, u16; 0x9c00, 1)) = 0;
    let mut v0: u16 = 3;
    let mut v1: i16 = ((word!(game.layer1_y_pos)) as i16).wrapping_add(32);
    let mut r18: u16 = (word!(game.layer1_y_pos)).wrapping_add(32);
    let mut v2: u16 = 0;
    while (v1).wrapping_sub(
        (*((RomPtr_88 as unsafe fn(u16, &mut Game) -> *const u8)(r0, game).offset(v2 as isize)
            as *mut u16)) as i16,
    ) < 0
        || (v1).wrapping_sub(
            (*((RomPtr_88 as unsafe fn(u16, &mut Game) -> *const u8)(r9, game).offset(v2 as isize)
                as *mut u16)) as i16,
        ) >= 0
    {
        v2 = (v2).wrapping_add(6);
        if ((v2) as i16).wrapping_sub((r24) as i16) >= 0 {
            return v0;
        }
    }
    let mut v3: u16 = if r18 as i32 - 1248i32 & 0x8000 != 0 {
        (r18) & (0xfu16)
    } else {
        (r18) & (0x1fu16)
    };
    let R22_: u16 = v3;
    *(ram_ptr!(game, u16; 0x9c00, 3)) =
        (*((RomPtr_88 as unsafe fn(u16, &mut Game) -> *const u8)(r3, game).offset(v2 as isize)
            as *mut u16))
            .wrapping_sub(v3);
    *(ram_ptr!(game, u16; 0x9c00, 4)) = (R22_ as i32
        + *((RomPtr_88 as unsafe fn(u16, &mut Game) -> *const u8)(r6, game).offset(v2 as isize)
            as *mut u16) as i32
        - word!(game.g_word_7E0598) as i32) as u16;
    loop {
        let r20: u16 = *ram_ptr!(game; 0x9c00, v0 as isize) as u16;
        r18 = (r18).wrapping_add(r20);
        v0 = (v0).wrapping_add(3);
        word!(game.g_word_7E0598, wrapping_add(r20));
        if word!(game.g_word_7E0598) as i32 - 224i32 & 0x8000 == 0 {
            break;
        }
        while ((r18) as i16).wrapping_sub(
            (*((RomPtr_88 as unsafe fn(u16, &mut Game) -> *const u8)(r0, game).offset(v2 as isize)
                as *mut u16)) as i16,
        ) < 0
            || ((r18) as i16).wrapping_sub(
                (*((RomPtr_88 as unsafe fn(u16, &mut Game) -> *const u8)(r9, game)
                    .offset(v2 as isize) as *mut u16)) as i16,
            ) >= 0
        {
            v2 = (v2).wrapping_add(6);
            if ((v2) as i16).wrapping_sub((r24) as i16) >= 0 {
                return v0;
            }
        }
        *(ram_ptr!(game, u16; 0x9c00, v0 as isize)) =
            *((RomPtr_88 as unsafe fn(u16, &mut Game) -> *const u8)(r3, game).offset(v2 as isize)
                as *mut u16);
        *(ram_ptr!(game, u16; 0x9c00, (v0 as i32 + 1) as isize)) =
            (*((RomPtr_88 as unsafe fn(u16, &mut Game) -> *const u8)(r6, game).offset(v2 as isize)
                as *mut u16))
                .wrapping_sub(word!(game.g_word_7E0598));
    }
    v0
}
pub unsafe fn HdmaobjPreInstr_FxType22_BG3Yscroll(mut k: u16, game: &mut Game) {
    DamageSamusInTopRow(game);
    k = SetupSomeHdmaTablesBG3(
        78,
        addr_word_88A8E8 as u16,
        (addr_word_88A8E8 as i32 + 2) as u16,
        (addr_word_88A8E8 as i32 + 4) as u16,
        (addr_word_88A8E8 as i32 + 6) as u16,
        game,
    );
    *(ram_ptr!(game, u16; 0x9c00, k as isize)) = 0;
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, hdp: *const u8, game: &mut Game) -> *const u8;
    HdmaobjInstr_SetFlagB [hdp] { word!(game.hdma_object_B[(k >> 1)] = 1); };
}
static g_word_88C46E: [i16; 16] = [0, 1, 1, 0, 0, -1, -1, 0, 0, 1, 1, 0, 0, -1, -1, 0];
pub unsafe fn HdmaobjPreInstr_BG3Xscroll(k: u16, game: &mut Game) {
    if word!(game.layer1_y_pos) as i32 - 1024i32 & 0x8000 != 0 {
        if (*ram_ptr!(game; 0x5b5) & 1) == 0 {
            word!(game.g_word_7E0596 = (word!(game.g_word_7E0596) as i32 + 2 & 0x1e) as u16);
        }
        let mut v1: u16 = word!(game.g_word_7E0596);
        let mut v2: u16 = 0;
        do_until! { (((v1) as i16).wrapping_sub(32) >= 0);
            word!(
                game.g_word_7E9E80[(v2 >> 1)] =
                    crate::rtl::rom_table_word!(game, 0x88a938, (v1 >> 1) as isize)
            );
            v2 = (v2).wrapping_add(2);
            v1 = (v1).wrapping_add(2);
        }
    } else {
        let v3: i32 = k as i32 >> 1;
        crate::update_place! { unaligned_word fresh104 [fresh105] = &raw mut (*(ram_ptr!(game, u16; 0x1920)).offset(v3 as isize)) => (u16::from_le(fresh104.read_unaligned())).wrapping_sub(1); }
        if fresh105 == 1 {
            word! {game; hdma_object_B[v3] = 6;
            hdma_object_A[v3] = (word!(game.hdma_object_A[v3]) as i32 + 2 & 0x1f) as u16; }
        }
        let mut v5: u16 = word!(game.hdma_object_A[v3]);
        for i in (0i32..=30).rev().step_by(2) {
            word!(
                game.g_word_7E9E80[(v5 >> 1)] = ((g_word_88C46E[(i >> 1) as usize]) as u16)
                    .wrapping_add(word!(game.reg_BG1HOFS))
            );
            v5 = (v5 as i32 - 2i32 & 0x1f) as u16;
        }
    }
    *(&raw mut (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00))).field_0 as *mut u16) = 0;
    *(ram_ptr!(game, u16; 0x9f00)) = 0;
    *(&raw mut (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00))).field_2 as *mut u16) = 31;
    *(ram_ptr!(game, u16; 0x9f00, 2)) = 31;
    let mut r18: u16 = 31;
    *(&raw mut (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00)).offset(1)).field_0
        as *mut u16) = -25088i32 as u16;
    *(ram_ptr!(game, u16; 0x9f00, 3)) = -25088i32 as u16;
    if word!(game.layer1_y_pos) as i32 - 1024i32 & 0x8000 != 0 {
        *(ram_ptr!(game, u16; 0x9f00, 2)) = 0;
    } else {
        r18 = HdmaFunc_A786(0x105, r18, 177, 0, game);
    }
    let mut v7: u16 = 5;
    r18 = HdmaFunc_A786(5, r18, -24960i32 as u16, 128, game);
    let mut v8: i16 = (224i16).wrapping_sub((r18) as i16);
    while (v8).wrapping_sub(16) >= 0 {
        v8 = (v8).wrapping_sub(16);
        *((&raw mut (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00))).field_0)
            .offset(v7 as i32 as isize) as *mut u16) = 144;
        *(ram_ptr!(game, u16; 0x9f00, v7 as isize)) = 144;
        *((&raw mut (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00))).field_1)
            .offset(v7 as i32 as isize) as *mut u16) = -24960i32 as u16;
        *(ram_ptr!(game, u16; 0x9f00, (v7 as i32 + 1) as isize)) = -24960i32 as u16;
        v7 = (v7).wrapping_add(3);
    }
    let mut v9: i16 = v8;
    if v8 != 0 {
        v9 = (v8).wrapping_add(128);
    }
    *((&raw mut (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00))).field_0)
        .offset(v7 as i32 as isize) as *mut u16) = v9 as u16;
    *(ram_ptr!(game, u16; 0x9f00, v7 as isize)) = v9 as u16;
    *((&raw mut (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00))).field_1)
        .offset(v7 as i32 as isize) as *mut u16) = -24960i32 as u16;
    *(ram_ptr!(game, u16; 0x9f00, (v7 as i32 + 1) as isize)) = -24960i32 as u16;
    *((&raw mut (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00)).offset(1)).field_0)
        .offset(v7 as i32 as isize) as *mut u16) = 0;
    *(ram_ptr!(game, u16; 0x9f00, (v7 as i32 + 3) as isize)) = 0;
}
pub unsafe fn HdmaFunc_A786(k: u16, mut r18: u16, r20: u16, r22: u16, game: &mut Game) -> u16 {
    let mut v0: u16 = k;
    let mut v1: u16 = (1216u16).wrapping_sub(word!(game.layer1_y_pos));
    let mut r24: u16 = (1216u16).wrapping_sub(word!(game.layer1_y_pos));
    if (1216i16).wrapping_sub((word!(game.layer1_y_pos)) as i16) >= 0
        && word!(game.layer1_y_pos) != 1216
    {
        if v1 as i32 - 128i32 & 0x8000 != 0 {
            r18 = (1216u16).wrapping_sub(word!(game.layer1_y_pos));
        } else {
            r18 = (1216u16).wrapping_sub(word!(game.layer1_y_pos));
            if v1 as i32 - 193i32 & 0x8000 == 0 {
                v1 = 193;
            }
            r18 = v1;
            r24 = v1;
            loop {
                let mut v2: i16 = ((v1) as i16).wrapping_sub(16);
                if v2 < 0 {
                    break;
                }
                r24 = v2 as u16;
                *((&raw mut (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00))).field_0)
                    .offset(v0 as i32 as isize) as *mut u16) = (r22) | (0x10u16);
                *((&raw mut (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00))).field_1)
                    .offset(v0 as i32 as isize) as *mut u16) = r20;
                v1 = r24;
                v0 = (v0).wrapping_add(3);
            }
        }
        *((&raw mut (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00))).field_0)
            .offset(v0 as i32 as isize) as *mut u16) = (r22) | (r24);
        *((&raw mut (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00))).field_1)
            .offset(v0 as i32 as isize) as *mut u16) = r20;
    }
    r18
}
static unk_88A7EF: SpawnHdmaObject_Args = SpawnHdmaObject_Args::new(0x42, 0xf, 0xad76 as VoidP);
static unk_88A80B: SpawnHdmaObject_Args = SpawnHdmaObject_Args::new(0x42, 0xf, 0xad89 as VoidP);
pub unsafe fn FxTypeFunc_20(game: &mut Game) {
    crate::update_place! { reference fresh47 = *ram_ptr!(game; 0x91b) => (*fresh47) | (1u8); }
    crate::update_place! { reference fresh48 = *ram_ptr!(game; 0x91c) => (*fresh48) | (1u8); }
    SpawnHdmaObject(0x88, &raw const unk_88A7EF as *const c_void, game);
    word! {game; set_to_e0_by_scrolling_sky = 224; UNUSED_word_7E059C = 0; }
}
pub unsafe fn RoomSetupAsm_ScrollingSkyOcean(game: &mut Game) {
    *ram_ptr!(game; 0x59) = 74;
    SpawnHdmaObject(0x88, &raw const unk_88A80B as *const c_void, game);
    word! {game; set_to_e0_by_scrolling_sky = 224; UNUSED_word_7E059C = 0; }
}
pub unsafe fn DamageSamusInTopRow(game: &mut Game) {
    if ((word!(game.samus_y_pos)) as i16).wrapping_sub((word!(game.samus_y_radius)) as i16) < 0
        || word!(game.samus_y_pos) as i32 - word!(game.samus_y_radius) as i32 - 17i32 & 0x8000 != 0
    {
        word!(game.samus_periodic_damage = 8);
    }
}
pub unsafe fn HdmaobjPreInstr_SkyLandBG2XscrollInner(k: u16, game: &mut Game) {
    let mut r24: u16 = 0;
    *ram_ptr!(game; 0x59) = 74;
    for v1 in 0u16..23 {
        let v3: *mut u16 = ram_ptr!(game, u16; (*(rom_ptr!(game, HdmaScrollEntry; 0x88aec1))
                .offset(v1 as isize))
            .hdma_data_table_entry as isize,);
        AddToHiLo(
            v3.offset(1),
            v3,
            words!((
                (*(rom_ptr!(game, HdmaScrollEntry; 0x88aec1)).offset(v1 as isize)).scroll_speed,
                (*(rom_ptr!(game, HdmaScrollEntry; 0x88aec1)).offset(v1 as isize)).scroll_subspeed
            )),
        );
    }
    word! { game, 0; scrolling_sky_bg2_hdma_data[44], scrolling_sky_bg2_hdma_data[45] }
    *(ram_ptr!(game, u16; 0x9f00)) = 31;
    *(ram_ptr!(game, u16; 0x9f00, 1)) = 1438;
    let mut r18: u16 = (word!(game.layer1_y_pos)).wrapping_add(32);
    let r20: u16 = (word!(game.layer1_y_pos)).wrapping_add(224);
    let mut v6: u16 = 3;
    for v5 in 0u16..23 {
        while ((r18) as i16).wrapping_sub(
            ((*(rom_ptr!(game, HdmaScrollEntry; 0x88aec1)).offset(v5 as isize)).top_pos) as i16,
        ) >= 0
            && ((r18) as i16).wrapping_sub(
                ((*(rom_ptr!(game, HdmaScrollEntry; 0x88aec1)).offset((v5 as i32 + 1i32) as isize))
                    .top_pos) as i16,
            ) < 0
        {
            let mut v8: u16 = ((*(rom_ptr!(game, HdmaScrollEntry; 0x88aec1))
                .offset((v5 as i32 + 1i32) as isize))
            .top_pos)
                .wrapping_sub(r18);
            r24 = v8;
            if v8 as i32 - 128i32 & 0x8000 == 0 {
                *(ram_ptr!(game, u16; 0x9f00, v6 as isize)) = 127;
                *(ram_ptr!(game, u16; 0x9f00, (v6 as i32 + 1) as isize)) =
                    ((*(rom_ptr!(game, HdmaScrollEntry; 0x88aec1)).offset(v5 as isize))
                        .hdma_data_table_entry)
                        .wrapping_add(2);
                v6 = (v6).wrapping_add(3);
                v8 = (r24).wrapping_sub(127);
            }
            *(ram_ptr!(game, u16; 0x9f00, v6 as isize)) = v8;
            *(ram_ptr!(game, u16; 0x9f00, (v6 as i32 + 1) as isize)) =
                ((*(rom_ptr!(game, HdmaScrollEntry; 0x88aec1)).offset(v5 as isize))
                    .hdma_data_table_entry)
                    .wrapping_add(2);
            r18 = (r18).wrapping_add(r24);
            v6 = (v6).wrapping_add(3);
            if r18 as i32 - r20 as i32 & 0x8000 == 0 {
                *(ram_ptr!(game, u16; 0x9f00, (v6 as i32 + 3) as isize)) = 0;
                return;
            }
        }
    }
    let mut i: u16 = (1535u16).wrapping_sub(r18);
    loop {
        r24 = i;
        if i as i32 - 128i32 & 0x8000 != 0 {
            break;
        }
        *(ram_ptr!(game, u16; 0x9f00, v6 as isize)) = 127;
        *(ram_ptr!(game, u16; 0x9f00, (v6 as i32 + 1) as isize)) = 181;
        v6 = (v6).wrapping_add(3);
        i = (r24).wrapping_sub(127);
    }
    *(ram_ptr!(game, u16; 0x9f00, v6 as isize)) = i;
    *(ram_ptr!(game, u16; 0x9f00, (v6 as i32 + 1) as isize)) = 181;
    *(ram_ptr!(game, u16; 0x9f00, (v6 as i32 + 3) as isize)) = 0;
}
pub unsafe fn HdmaobjPreInstr_SkyLandBG2Xscroll(k: u16, game: &mut Game) {
    if word!(game.time_is_frozen_flag) == 0 {
        HdmaobjPreInstr_SkyLandBG2XscrollInner(k, game);
    }
}
pub use self::HdmaobjPreInstr_SkyLandBG2Xscroll as HdmaobjPreInstr_SkyLandBG2Xscroll2;
pub unsafe fn RoomMainAsm_ScrollingSky(src: *const u16, game: &mut Game) {
    if word!(game.time_is_frozen_flag) != 0 {
        *(ram_ptr!(game, u16; 0x9f00, 0)) = 0;
    } else {
        word!(game.reg_BG2VOFS = word!(game.layer1_y_pos));
        let v0: u16 = word!(game.vram_write_queue_tail);
        let v1 = gVramWriteEntry(word!(game.vram_write_queue_tail) as i32, game);
        (*v1).size = 64;
        (*v1.offset(1)).size = 64;
        (*v1.offset(2)).size = 64;
        (*v1.offset(3)).size = 64;
        let mut tt: i32 = (word!(game.layer1_y_pos) as i32 & 0x7f8) - 16i32;
        if tt & 0x8000 != 0 {
            tt = 0;
        }
        let v2: i32 = 8 * tt as u8 as i32;
        let y1: i32 = (tt >> 8) as u8 as i32;
        let y2: i32 = ((tt >> 8) + 1) as u8 as i32;
        let v3: VoidP = (*src.offset(y1 as isize) as i32 + v2) as VoidP;
        (*v1).src.addr = v3;
        (*v1.offset(1)).src.addr = (v3).wrapping_add(64);
        let v5: VoidP = (*src.offset(y2 as isize) as i32 + v2) as VoidP;
        (*v1.offset(2)).src.addr = v5;
        (*v1.offset(3)).src.addr = (v5).wrapping_add(64);
        (*v1).src.bank = 0x8a;
        (*v1.offset(1)).src.bank = 0x8a;
        (*v1.offset(2)).src.bank = 0x8a;
        (*v1.offset(3)).src.bank = 0x8a;
        let t: i32 = (*ram_ptr!(game; 0x59) as i32 & 0xfc) << 8;
        let v6: u16 = (t + 4 * (word!(game.layer1_y_pos) as i32 - 16i32 & 0x1f8)) as u16;
        (*v1).vram_dst = v6;
        (*v1.offset(1)).vram_dst = (v6).wrapping_add(32);
        let v7: u16 = (t + 4 * (word!(game.layer1_y_pos) as i32 + 240 & 0x1f8)) as u16;
        (*v1.offset(2)).vram_dst = v7;
        (*v1.offset(3)).vram_dst = (v7).wrapping_add(32);
        word!(game.vram_write_queue_tail = (v0).wrapping_add(28));
    };
}
crate::forward_calls! { pub unsafe fn (game: &mut Game);
    RoomCode_ScrollingSkyLand =>
        RoomMainAsm_ScrollingSky(RomPtr_88(addr_off_88AD9C as u16, game) as *const u16, game);
    RoomMainAsm_ScrollingSkyOcean =>
        RoomMainAsm_ScrollingSky(RomPtr_88(addr_off_88ADA6 as u16, game) as *const u16, game);
}
static unk_88B08C: SpawnHdmaObject_Args = SpawnHdmaObject_Args::new(0x42, 0x11, 0xb0ac as VoidP);
static kFirefleaFlashingShades: [u16; 12] = [
    0, 0x100, 0x200, 0x300, 0x400, 0x500, 0x600, 0x500, 0x400, 0x300, 0x200, 0x100,
];
static kFirefleaDarknessShades: [u16; 6] = [0, 0x600, 0xc00, 0x1200, 0x1800, 0x1900];
pub unsafe fn FxTypeFunc_24(game: &mut Game) {
    word! {game; fireflea_flashing_timer = 6; fireflea_flashing_index = 0; }
    SpawnHdmaObject(0x88, &raw const unk_88B08C as *const c_void, game);
    *(&raw mut (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00))).field_0 as *mut u16) = 0;
    word! { game, 0; UNUSED_word_7E177C, fireflea_darkness_level }
    word!(game.UNUSED_word_7E1780 = 24);
    word!(game.UNUSED_word_7E1782 = kFirefleaFlashingShades[0i32 as usize]);
}
pub unsafe fn HdmaobjPreInstr_FirefleaBG3XScroll(k: u16, game: &mut Game) {
    let mut v1: u16 = 0;
    word!(
        game.fx_layer_blending_config_c = (word!(game.fx_layer_blending_config_c) & 0xff00 | 0xc)
    );
    if word!(game.time_is_frozen_flag) == 0 {
        if word!(game.fireflea_flashing_timer, wrapping_sub(1), reread) == 0 {
            word!(game.fireflea_flashing_timer = 6);
            if word!(game.fireflea_darkness_level) as i32 - 10i32 & 0x8000 != 0 {
                v1 = (word!(game.fireflea_flashing_index)).wrapping_add(1);
                if (word!(game.fireflea_flashing_index)).wrapping_add(1) >= 0xc {
                    v1 = 0;
                }
            } else {
                v1 = 6;
            }
            word!(game.fireflea_flashing_index = v1);
        }
        let mut v2: i8 = (((kFirefleaDarknessShades
            [(word!(game.fireflea_darkness_level) >> 1) as usize])
            .wrapping_add(kFirefleaFlashingShades[word!(game.fireflea_flashing_index) as usize]))
            >> 8) as i8;
        *ram_ptr!(game; 0x74, 1) = ((v2) as u8) | (0x80u8);
        *ram_ptr!(game; 0x74, 2) = ((v2) as u8) | (0x40u8);
        *ram_ptr!(game; 0x74, 0) = ((v2) as u8) | (0x20u8);
    }
}
static g_word_88B256: [u16; 17] = [
    0x46, 1, 0x46, 3, 0x46, 2, 0x46, 1, 0x46, 1, 0x46, 2, 0x46, 2, 0x46, 1, 0x8000,
];
pub unsafe fn HandleEarthquakeSoundEffect(game: &mut Game) {
    if (word!(game.earthquake_sfx_timer) & 0x8000) == 0 && {
        (word!(game.earthquake_sfx_timer, wrapping_sub(1), reread) & 0x8000) != 0
    } {
        let mut v0: u16 = word!(game.earthquake_sfx_index);
        let mut v1: i16 = g_word_88B256[(word!(game.earthquake_sfx_index) >> 1) as usize] as i16;
        if v1 < 0 {
            v0 = 0;
            v1 = g_word_88B256[0i32 as usize] as i16;
        }
        QueueSfx2_Max6(v1 as u16, game);
        word! { game;
            earthquake_sfx_timer = (g_word_88B256[((v0 as i32 >> 1) + 1) as usize] as i32
                + (v0 as i32 & 1)
                + (word!(game.random_number) as i32 & 3)) as u16;
            earthquake_sfx_index = (v0).wrapping_add(4);
        }
    }
}
static unk_88B289: SpawnHdmaObject_Args = SpawnHdmaObject_Args::new(0x42, 0x12, 0xc3e1 as VoidP);
static unk_88B291: SpawnHdmaObject_Args = SpawnHdmaObject_Args::new(0x42, 0x10, 0xc3f0 as VoidP);
static unk_88B2B1: SpawnHdmaObject_Args = SpawnHdmaObject_Args::new(0x42, 0x12, 0xc3e1 as VoidP);
static unk_88B2B9: SpawnHdmaObject_Args = SpawnHdmaObject_Args::new(0x42, 0x10, 0xc3f0 as VoidP);
pub unsafe fn FxTypeFunc_2_Lava(game: &mut Game) {
    word!(game.fx_rising_function_bank_88 = ((0x88b343i32) as u16));
    word!(game.lava_acid_y_pos = word!(game.fx_base_y_pos));
    SpawnHdmaObject(0x88, &raw const unk_88B289 as *const c_void, game);
    SpawnHdmaObject(0x88, &raw const unk_88B291 as *const c_void, game);
    SpawnBG3ScrollHdmaObject(game);
    SpawnAnimtiles(addr_kAnimtiles_Lava as u16, game);
}
pub unsafe fn FxTypeFunc_4_Acid(game: &mut Game) {
    word!(game.fx_rising_function_bank_88 = ((0x88b343i32) as u16));
    word!(game.lava_acid_y_pos = word!(game.fx_base_y_pos));
    SpawnHdmaObject(0x88, &raw const unk_88B2B1 as *const c_void, game);
    SpawnHdmaObject(0x88, &raw const unk_88B2B9 as *const c_void, game);
    SpawnBG3ScrollHdmaObject(game);
    SpawnAnimtiles(addr_kAnimtiles_Acid as u16, game);
}

macro_rules! advance_liquid_tide {
    (
        $game:ident, $amplitude:literal, $sign_mask:literal, $phase:ident,
        $nonnegative_sine_step:literal, $negative_sine_step:literal $(,)?
    ) => {
        word! { $game, 0; fx_y_suboffset, fx_y_offset }
        let v0: i32 = byte!(@pointer ram_ptr!($game, u16; 0x1974); 1) as i32;
        let v1: u16 =
            ($amplitude).wrapping_mul((kSinCosTable8bit_Sext[v0 as usize]) as u16);
        if (kSinCosTable8bit_Sext[v0 as usize] as u16 & $sign_mask) != 0 {
            word!($game.fx_y_offset, wrapping_sub(1));
        }
        *((ram_ptr!($game, u16; 0x1970) as *mut u8).offset(1) as *mut u16) = v1;
        if (kSinCosTable8bit_Sext[v0 as usize] as u16 & 0x8000) == 0 {
            $phase = (word!($game.tide_phase)).wrapping_add($nonnegative_sine_step);
        } else {
            $phase = (word!($game.tide_phase)).wrapping_add($negative_sine_step);
        }
        word!($game.tide_phase = $phase);
    };
}

pub unsafe fn FxHandleTide(game: &mut Game) {
    let mut v2: u16 = 0;
    let mut v5: u16 = 0;
    if *((ram_ptr!(game, u16; 0x197c) as *mut u8).offset(1) as *mut i16) < 0 {
        advance_liquid_tide!(game, 8u16, 0x1000, v2, 288, 192);
    } else if (*((ram_ptr!(game, u16; 0x197c) as *mut u8).offset(1) as *mut u16) & 0x4000) != 0 {
        advance_liquid_tide!(game, 32u16, 0x400, v5, 224, 128);
    }
}

macro_rules! start_liquid_motion_toward_target {
    ($game:ident, $motion_state:literal) => {
        if (word!($game.fx_y_vel) & 0x8000) == 0 {
            if word!($game.fx_target_y_pos) < word!($game.fx_base_y_pos)
                || word!($game.fx_target_y_pos) == word!($game.fx_base_y_pos)
            {
                return;
            }
        } else if !(word!($game.fx_target_y_pos) != word!($game.fx_base_y_pos)
            && word!($game.fx_target_y_pos) < word!($game.fx_base_y_pos))
        {
            return;
        }
        word!($game.fx_rising_function_bank_88 = (($motion_state) as u16));
    };
}

crate::instruction_callbacks! { pub unsafe fn (k: u16, hdp: *const u8, game: &mut Game) -> *const u8;
    HdmaobjInstr_B3A9 [hdp] { word!(game.hdma_object_C[(k >> 1)] = 112); };
}
crate::game::callback_dispatch! {
    pub unsafe fn CallFxRisingFunc(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [8958787] FxRisingFunction_LavaAcid [] {
        if word!(game.fx_y_vel) != 0 {
            start_liquid_motion_toward_target!(game, 0x88b367i32);
        }
    };
    [8958823] FxRisingFunction_LavaAcid_WaitToRise [] {
        HandleEarthquakeSoundEffect(game);
        word!(game.earthquake_type = 21);
        word!(game.earthquake_timer, |fresh44| (fresh44) | (0x20u16));
        if word!(game.fx_timer, wrapping_sub(1), reread) == 0 {
            word!(game.fx_rising_function_bank_88 = ((0x88b382i32) as u16));
        }
    };
    [8958850] FxRisingFunction_LavaAcid_Raising [] {
        HandleEarthquakeSoundEffect(game);
        word!(game.earthquake_type = 21);
        word!(game.earthquake_timer, |fresh42| (fresh42) | (0x20u16));
        if RaiseOrLowerFx(game) & 1 != 0 {
            word! {game; fx_y_vel = 0; fx_rising_function_bank_88 = ((0x88b343i32) as u16); }
        }
    };
    [8963112] FxRisingFunction_C428_WaterNormal [] {
        if word!(game.fx_y_vel) != 0 {
            start_liquid_motion_toward_target!(game, 0x88c44ci32);
        }
    };
    [8963148] FxRisingFunction_WaterWaitToRise [] {
        if word!(game.fx_timer, wrapping_sub(1), reread) == 0 {
            word!(game.fx_rising_function_bank_88 = ((0x88c458i32) as u16));
        }
    };
    [8963160] FxRisingFunction_WaterRising [] {
        if RaiseOrLowerFx(game) & 1 != 0 {
            word! {game; fx_rising_function_bank_88 = ((0x88c428i32) as u16); fx_y_vel = 0; }
        }
    };
}
static kLavaSoundEffects: [u8; 8] = [0x12, 0x13, 0x14, 0x12, 0x13, 0x14, 0x12, 0x13];
pub unsafe fn HdmaobjPreInstr_LavaAcidBG3YScroll(k: u16, game: &mut Game) {
    word!(game.fx_layer_blending_config_c = word!(game.fx_layer_blending_config_b));
    if word!(game.time_is_frozen_flag) == 0 {
        CallFxRisingFunc(
            ((word!(game.fx_rising_function_bank_88)) as u32) | (0x880000u32),
            game,
        );
        FxHandleTide(game);
        words!(
            game.lava_acid_y_pos,
            lava_acid_y_subpos = (words!(game.fx_y_offset, fx_y_suboffset))
                .wrapping_add(words!(game.fx_base_y_pos, fx_base_y_subpos),)
        );
        word!(game.bg3_xpos = word!(game.reg_BG1HOFS));
        *(ram_ptr!(game, u16; 0x9c00)) = 0;
        word!(game.bg3_ypos = 0);
        let mut v2: i16 = ((words!(game.fx_y_offset, fx_y_suboffset))
            .wrapping_add(words!(game.fx_base_y_pos, fx_base_y_subpos))
            >> 16) as i16;
        let v4 = if v2 >= 0 {
            let v3: i16 = (v2).wrapping_sub((word!(game.layer1_y_pos)) as i16);
            if v3 <= 0 {
                ((v3 as i32 ^ 0x1f) & 0x1f | 0x100) as i16
            } else if (v3 as u16) < 0x100 {
                !(v3 as i32) as u8 as i16
            } else {
                0
            }
        } else {
            0
        };
        *(ram_ptr!(game, u16; 0x9c00, 2)) = v4 as u16;
        if word!(game.fx_type) == 2 && (word!(game.lava_acid_y_pos) & 0x8000) == 0 {
            let v5: i32 = word!(game.hdma_object_index) as u8 as i32 >> 1;
            crate::update_place! { unaligned_word fresh101 [fresh102] = &raw mut (*(ram_ptr!(game, u16; 0x192c)).offset(v5 as isize)) => (u16::from_le(fresh101.read_unaligned())).wrapping_sub(1); }
            if fresh102 == 1 {
                word!(game.hdma_object_C[v5] = 112);
                QueueSfx2_Max6(
                    kLavaSoundEffects[(word!(game.random_number) as i32 & 7) as usize] as u16,
                    game,
                );
            }
        }
        word!(game.random_number = u16::swap_bytes(word!(game.random_number)));
        let v8 = if (word!(game.lava_acid_y_pos) & 0x8000) == 0 {
            let v8 =
                (word!(game.lava_acid_y_pos) as i32 - word!(game.layer1_y_pos) as i32 + 256) as i16;
            if v8 < 0 {
                255
            } else if (v8 as u16) < 0x200 {
                v8
            } else {
                511
            }
        } else {
            511
        };
        *(ram_ptr!(game, u16; 0x18d8))
            .offset((word!(game.hdma_object_index) as u8 >> 1) as isize) =
            (3 * ((v8 as i32 ^ 0x1ff) + 1 & 0x3ff) + 0xb62a) as u16;
    } else {
    }
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, hdp: *const u8, game: &mut Game) -> *const u8;
    HdmaobjInstr_SetFlagB_Copy [hdp] { word!(game.hdma_object_B[(k >> 1)] = 1); };
}
pub unsafe fn HdmaobjPreInstr_LavaAcidBG2YScroll(k: u16, game: &mut Game) {
    word!(game.g_word_7E9C44 = word!(game.reg_BG2VOFS));
    if word!(game.time_is_frozen_flag) == 0 && (word!(game.fx_liquid_options) & 6) != 0 {
        if (word!(game.fx_liquid_options) & 2) != 0 {
            Handle_LavaAcidBG2YScroll_Func3(k, game);
        } else {
            Handle_LavaAcidBG2YScroll_Func2(k, game);
        }
    } else {
        Handle_LavaAcidBG2YScroll_Func1(k, game);
    }
    *(ram_ptr!(game, u16; 0x18d8)).offset((word!(game.hdma_object_index) as u8 >> 1) as isize) =
        (3 * (word!(game.reg_BG2VOFS) as i32 & 0xf) + 0xc0b1) as u16;
}
pub unsafe fn Handle_LavaAcidBG2YScroll_Func1(v0: u16, game: &mut Game) {
    WriteReg(
        (((byte!(@pointer ram_ptr!(game, u16; 0x18c0); v0 as i32 as isize)) as u32)
            .wrapping_add(17153)) as u16,
        0x10,
        game,
    );
    let mut v1: u8 = 30;
    let v2: u16 = (word!(game.reg_BG2VOFS)) & (0x1ffu16);
    word_transfers!(game, descending, 16; le(g_word_7E9C46, 15) <- value(v2));
    v1 = 254;
}
pub unsafe fn Handle_LavaAcidBG2YScroll_Func2(v0: u16, game: &mut Game) {
    WriteReg(
        (((byte!(@pointer ram_ptr!(game, u16; 0x18c0); v0 as i32 as isize)) as u32)
            .wrapping_add(17153)) as u16,
        0xf,
        game,
    );
    let v1: i32 = v0 as i32 >> 1;
    crate::update_place! { unaligned_word fresh54 [fresh55] = &raw mut (*(ram_ptr!(game, u16; 0x1920)).offset(v1 as isize)) => (u16::from_le(fresh54.read_unaligned())).wrapping_sub(1); }
    if fresh55 == 1 {
        word!(game.hdma_object_B[v1] = 6);
        word!(game.hdma_object_A[v1] = (word!(game.hdma_object_A[v1]) as i32 - 2i32 & 0x1e) as u16);
    }
    let mut v3: u8 = byte!(@pointer ram_ptr!(game, u16; 0x1914); v0 as i32 as isize);
    let mut v4: u8 = 30;
    for _ in 0..=15 {
        word!(
            game.g_word_7E9C46[(v4 >> 1)] =
                (crate::rtl::rom_table_word!(game, 0x88b589, (v3 >> 1) as isize) as i32
                    + word!(game.reg_BG2HOFS) as i32
                    & 0x1ff) as u16
        );
        v3 = (v3 as i32 - 2i32 & 0x1e) as u8;
        v4 = (v4 as i32 - 2i32 & 0x1e) as u8;
    }
}
pub unsafe fn Handle_LavaAcidBG2YScroll_Func3(v0: u16, game: &mut Game) {
    WriteReg(
        (byte!(@pointer ram_ptr!(game, u16; 0x18c0); v0 as i32 as isize) as i32 + BBAD0 as i32)
            as SnesRegs as u16,
        0x10,
        game,
    );
    let v1: i32 = v0 as i32 >> 1;
    crate::update_place! { unaligned_word fresh56 [fresh57] = &raw mut (*(ram_ptr!(game, u16; 0x1920)).offset(v1 as isize)) => (u16::from_le(fresh56.read_unaligned())).wrapping_sub(1); }
    if fresh57 == 1 {
        word!(game.hdma_object_B[v1] = 4);
        word!(game.hdma_object_A[v1] = (word!(game.hdma_object_A[v1]) as i32 - 2i32 & 0x1e) as u16);
    }
    let mut v3: u8 = (byte!(@pointer (ram_ptr!(game, u16; 0x1914)).offset(v1 as isize); 0) as i32
        + 2 * (word!(game.reg_BG2VOFS) as i32 & 0xf)
        & 0x1e) as u8;
    let mut v4: u8 = (2 * (word!(game.reg_BG2VOFS) as i32 & 0xf) + 30 & 0x1e) as u8;
    for _ in 0..=15 {
        word!(
            game.g_word_7E9C46[(v4 >> 1)] =
                (crate::rtl::rom_table_word!(game, 0x88b60a, (v3 >> 1) as isize) as i32
                    + word!(game.reg_BG2VOFS) as i32
                    & 0x1ff) as u16
        );
        v4 = (v4 as i32 - 2i32 & 0x1e) as u8;
        v3 = (v3 as i32 - 2i32 & 0x1e) as u8;
    }
}
static unk_88C40F: SpawnHdmaObject_Args = SpawnHdmaObject_Args::new(0x42, 0x11, 0xd856 as VoidP);
static unk_88C41F: SpawnHdmaObject_Args = SpawnHdmaObject_Args::new(0x42, 0xf, 0xd847 as VoidP);
pub unsafe fn FxTypeFunc_6_Water(game: &mut Game) {
    word!(game.fx_rising_function_bank_88 = ((0x88c428i32) as u16));
    word!(game.fx_y_pos = word!(game.fx_base_y_pos));
    SpawnHdmaObject(0x88, &raw const unk_88C40F as *const c_void, game);
    if (word!(game.fx_liquid_options) & 2) != 0 {
        SpawnHdmaObject(0x88, &raw const unk_88C41F as *const c_void, game);
    }
    SpawnBG3ScrollHdmaObject(game);
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, hdp: *const u8, game: &mut Game) -> *const u8;
    HdmaobjInstr_SetFlagB_Copy2 [hdp] { word!(game.hdma_object_B[(k >> 1)] = 1); };
}
crate::game::address_dispatch! {
    pub unsafe fn CallFxRisingFunction(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        8958787 => FxRisingFunction_LavaAcid; 8958823 => FxRisingFunction_LavaAcid_WaitToRise;
        8958850 => FxRisingFunction_LavaAcid_Raising; 8963112 => FxRisingFunction_C428_WaterNormal;
        8963148 => FxRisingFunction_WaterWaitToRise; 8963160 => FxRisingFunction_WaterRising;
    }
}
pub unsafe fn HdmaobjPreInstr_WaterBG3XScroll(k: u16, game: &mut Game) {
    word!(game.fx_layer_blending_config_c = word!(game.fx_layer_blending_config_b));
    if word!(game.time_is_frozen_flag) != 0 {
        return;
    }
    CallFxRisingFunction(
        ((word!(game.fx_rising_function_bank_88)) as u32) | (0x880000u32),
        game,
    );
    FxHandleTide(game);
    words!(
        game.fx_y_pos,
        fx_y_subpos = (words!(game.fx_y_offset, fx_y_suboffset))
            .wrapping_add(words!(game.fx_base_y_pos, fx_base_y_subpos),)
    );
    let v3: i16 = ((word!(game.fx_y_pos)) as i16).wrapping_sub((word!(game.layer1_y_pos)) as i16);
    if (word!(game.fx_y_pos) as i16) < 0 {
        word!(game.bg3_ypos = 0);
    } else if v3 <= 0 {
        word!(game.bg3_ypos = ((v3 as u16 ^ 0x1f) & 0x1f | 0x100));
    } else if (v3 as u16) < 0x100 {
        word!(game.bg3_ypos = !(v3 as i32) as u8 as u16);
    } else {
        word!(game.bg3_ypos = 0);
    }
    let v5: i32 = k as i32 >> 1;
    let r20: u16 = (word!(game.layer1_x_pos)).wrapping_add(
        (byte!(@pointer (ram_ptr!(game, u16; 0x192c)).offset(v5 as isize); 1) as i8) as u16,
    );
    crate::update_place! { unaligned_word fresh98 [fresh99] = &raw mut (*(ram_ptr!(game, u16; 0x1920)).offset(v5 as isize)) => (u16::from_le(fresh98.read_unaligned())).wrapping_sub(1); }
    if fresh99 == 1 {
        word!(game.hdma_object_B[v5] = 10);
        word!(game.hdma_object_A[v5] = (word!(game.hdma_object_A[v5]) as i32 + 2 & 0x1e) as u16);
    }
    let mut v7: u8 = word!(game.hdma_object_A[v5]) as u8;
    for i in (0i32..=30).rev().step_by(2) {
        *(ram_ptr!(game, u16; 0x9c00, (v7 as i32 + 4) as isize)) =
            ((g_word_88C46E[(i >> 1i32) as usize]) as u16).wrapping_add(r20);
        v7 = (v7 as i32 - 2i32 & 0x1e) as u8;
    }
    if (word!(game.fx_liquid_options) & 1) != 0 {
        word!(game.hdma_object_C[(k >> 1)], wrapping_add(64));
    }
    let mut v9: i16 = (word!(game.fx_y_pos) as i32 - word!(game.layer1_y_pos) as i32 + 256) as i16;
    if (word!(game.fx_y_pos) as i16) < 0 {
        v9 = 511;
    } else if v9 >= 0 {
        if (v9 as u16) >= 0x200 {
            v9 = 511;
        }
    } else {
        v9 = (v9 as i32 & 0xf | 0x100) as i16;
    }
    word!(
        game.hdma_object_table_pointers[(k >> 1)] =
            (3 * ((v9 as i32 ^ 0x1ff) + 1 & 0x3ff) + 0xc645) as u16
    );
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, hdp: *const u8, game: &mut Game) -> *const u8;
    HdmaobjInstr_SetFlagB_Copy3 [hdp] { word!(game.hdma_object_B[(k >> 1)] = 1); };
}
pub unsafe fn HdmaobjPreInstr_WaterBG2XScroll(k: u16, game: &mut Game) {
    word!(game.g_word_7E9C44 = word!(game.reg_BG2HOFS));
    if word!(game.time_is_frozen_flag) != 0 {
        HdmaobjPreInstr_WaterBG2XScroll_Func1(k, game);
    } else if (word!(game.fx_liquid_options) & 2) == 0 {
        word!(game.hdma_object_channels_bitmask[(k as u8 >> 1)] = 0);
        HdmaobjPreInstr_WaterBG2XScroll_Func1(k, game);
    } else {
        HdmaobjPreInstr_WaterBG2XScroll_Func2(k, game);
    }
    let mut v1: i16 = (word!(game.fx_y_pos) as i32 - word!(game.layer1_y_pos) as i32 + 256) as i16;
    if (word!(game.fx_y_pos) as i16) < 0 {
        v1 = 511;
    } else if v1 >= 0 {
        if (v1 as u16) >= 0x200 {
            v1 = 511;
        }
    } else {
        v1 = (word!(game.fx_y_pos) as i32 - word!(game.layer1_y_pos) as i32 & 0xf | 0x100) as i16;
    }
    word!(
        game.hdma_object_table_pointers[(k as u8 >> 1)] =
            (3 * ((v1 as i32 ^ 0x1ff) & 0x3ff) + 0xcf46) as u16
    );
}
pub unsafe fn HdmaobjPreInstr_WaterBG2XScroll_Func2(k: u16, game: &mut Game) {
    let v1: i32 = k as u8 as i32 >> 1;
    crate::update_place! { unaligned_word fresh96 [fresh97] = &raw mut (*(ram_ptr!(game, u16; 0x1920)).offset(v1 as isize)) => (u16::from_le(fresh96.read_unaligned())).wrapping_sub(1); }
    if fresh97 == 1 {
        word!(game.hdma_object_B[v1] = 6);
        word!(game.hdma_object_A[v1] = (word!(game.hdma_object_A[v1]) as i32 + 2 & 0x1e) as u16);
    }
    let mut v3: u8 = (byte!(@pointer (ram_ptr!(game, u16; 0x1914)).offset(v1 as isize); 0) as i32
        + 2 * (word!(game.reg_BG2VOFS) as i32 & 0xf)
        & 0x1e) as u8;
    let mut v4: u8 = (2 * (word!(game.reg_BG2VOFS) as i32 & 0xf) + 30 & 0x1e) as u8;
    for _ in 0..=15 {
        *(ram_ptr!(game, u16; 0x9c46)).offset(((v4 as i32 >> 1) + 1) as isize) =
            ((g_word_88C46E[(v3 >> 1) as usize]) as u16).wrapping_add(word!(game.reg_BG2HOFS));
        v4 = (v4 as i32 - 2i32 & 0x1e) as u8;
        v3 = (v3 as i32 - 2i32 & 0x1e) as u8;
    }
}
pub unsafe fn HdmaobjPreInstr_WaterBG2XScroll_Func1(k: u16, game: &mut Game) {
    let mut v1: u8 = 30;
    let v2: u16 = word!(game.reg_BG2HOFS);
    word_transfers!(game, descending, 16; native(0x9c66) <- value(v2));
    v1 = 254;
}
pub unsafe fn SpawnBG3ScrollHdmaObject(game: &mut Game) {
    static unk_88D869: SpawnHdmaObject_Args =
        SpawnHdmaObject_Args::new(0x43, 0x11, 0xd8d0 as VoidP);
    SpawnHdmaObjectToSlot0xA(0x88, &raw const unk_88D869 as *const c_void, game);
}
pub unsafe fn FxTypeFunc_28_CeresRidley(game: &mut Game) {
    static unk_88D8E2: SpawnHdmaObject_Args = SpawnHdmaObject_Args::new(0x40, 0x5, 0xd906 as VoidP);
    static unk_88D8EA: SpawnHdmaObject_Args = SpawnHdmaObject_Args::new(0, 0x2c, 0xd91d as VoidP);
    SpawnHdmaObject(0x88, &raw const unk_88D8E2 as *const c_void, game);
    SpawnHdmaObject(0x88, &raw const unk_88D8EA as *const c_void, game);
}
pub unsafe fn sub_88D916(game: &mut Game) {
    word!(game.hdma_data_table_in_ceres = 9);
}
pub unsafe fn FxTypeFunc_CeresElevator(game: &mut Game) {
    static unk_88D92C: SpawnHdmaObject_Args = SpawnHdmaObject_Args::new(0x40, 0x5, 0xd939 as VoidP);
    SpawnHdmaObject(0x88, &raw const unk_88D92C as *const c_void, game);
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, hdp: *const u8, game: &mut Game) -> *const u8;
    HdmaobjInstr_SetVideoMode1 [hdp] { word!(game.hdma_data_table_in_ceres = 9); };
}
pub unsafe fn FxTypeFunc_A_Rain(game: &mut Game) {
    static unk_88D95C: SpawnHdmaObject_Args =
        SpawnHdmaObject_Args::new(0x43, 0x11, 0xd96c as VoidP);
    *ram_ptr!(game; 0x5b) = 92;
    SpawnHdmaObject(0x88, &raw const unk_88D95C as *const c_void, game);
    SpawnBG3ScrollHdmaObject(game);
    SpawnAnimtiles(addr_kAnimtiles_Rain as u16, game);
}
pub unsafe fn HdmaobjInstr_1938_RandomNumber(k: u16, hdp: *const u8, game: &mut Game) -> *const u8 {
    word!(
        game.hdma_object_D[(k >> 1)] = crate::rtl::rom_table_word!(
            game,
            0x88d992,
            ((word!(game.random_number) as i32 >> 1 & 6) as u16 >> 1) as isize,
        )
    );
    hdp
}
pub unsafe fn HdmaobjPreInstr_RainBg3Scroll(k: u16, game: &mut Game) {
    *ram_ptr!(game; 0x5b) = 92;
    word!(game.fx_layer_blending_config_c = word!(game.fx_layer_blending_config_b));
    if word!(game.time_is_frozen_flag) == 0 {
        let v1: i32 = k as u8 as i32 >> 1;
        word!(
            game.bg3_ypos = (word!(game.hdma_object_E[v1]) as i32 - word!(game.layer1_y_pos) as i32
                + *((ram_ptr!(game, u16; 0x1914)).offset(v1 as isize) as *mut u8).offset(1,) as i8
                    as i32) as u16
        );
        word!(game.hdma_object_A[v1], wrapping_sub(1536));
        word!(game.hdma_object_E[v1] = word!(game.layer1_y_pos));
        word!(
            game.bg3_xpos = (word!(game.hdma_object_F[v1]) as i32 - word!(game.layer1_x_pos) as i32
                + *((ram_ptr!(game, u16; 0x1920)).offset(v1 as isize) as *mut u8).offset(1,) as i8
                    as i32) as u16
        );
        word!(
            game.hdma_object_B[v1],
            wrapping_add(word!(game.hdma_object_D[v1]))
        );
        word!(game.hdma_object_F[v1] = word!(game.layer1_x_pos));
    }
}
pub unsafe fn FxTypeFunc_8_Spores(game: &mut Game) {
    *ram_ptr!(game; 0x5b) = 92;
    static unk_88DA1D: SpawnHdmaObject_Args =
        SpawnHdmaObject_Args::new(0x42, 0x11, 0xda2d as VoidP);
    SpawnHdmaObject(0x88, &raw const unk_88DA1D as *const c_void, game);
    SpawnBG3ScrollHdmaObject(game);
    SpawnAnimtiles(addr_kAnimtiles_Spores as u16, game);
}
pub unsafe fn HdmaobjPreInstr_SporesBG3Xscroll(k: u16, game: &mut Game) {
    *ram_ptr!(game; 0x5b) = 92;
    word!(game.fx_layer_blending_config_c = word!(game.fx_layer_blending_config_b));
    if word!(game.time_is_frozen_flag) == 0 {
        let v1: i32 = k as u8 as i32 >> 1;
        word!(
            game.bg3_ypos = (word!(game.layer1_y_pos)).wrapping_add(
                (*((ram_ptr!(game, u16; 0x192c)).offset(v1 as isize) as *mut u8).offset(1,) as i8)
                    as u16,
            )
        );
        let v3: u16 = (word!(game.hdma_object_C[v1])).wrapping_sub(64);
        word!(game.hdma_object_C[v1] = v3);
        word!(
            game.bg3_xpos = (word!(game.layer1_x_pos)).wrapping_add(
                (*((ram_ptr!(game, u16; 0x1938)).offset(v1 as isize) as *mut u8).offset(1,) as i8)
                    as u16,
            )
        );
        word!(game.hdma_object_D[v1] = word!(game.hdma_object_D[v1]));
    }
}
pub unsafe fn FxTypeFunc_C(game: &mut Game) {
    *ram_ptr!(game; 0x5b) = 92;
    static unk_88DB14: SpawnHdmaObject_Args =
        SpawnHdmaObject_Args::new(0x43, 0x11, 0xdb19 as VoidP);
    SpawnHdmaObject(0x88, &raw const unk_88DB14 as *const c_void, game);
}
pub unsafe fn HdmaobjPreInstr_FogBG3Scroll(k: u16, game: &mut Game) {
    *ram_ptr!(game; 0x5b) = 92;
    word!(game.fx_layer_blending_config_c = word!(game.fx_layer_blending_config_b));
    if word!(game.time_is_frozen_flag) == 0 {
        let v1: i32 = k as u8 as i32 >> 1;
        word!(
            game.bg3_ypos = (word!(game.layer1_y_pos)).wrapping_add(
                (*((ram_ptr!(game, u16; 0x1914)).offset(v1 as isize) as *mut u8).offset(1,) as i8)
                    as u16,
            )
        );
        word!(game.hdma_object_A[v1], wrapping_sub(64));
        word!(
            game.bg3_xpos = (word!(game.layer1_x_pos)).wrapping_add(
                (*((ram_ptr!(game, u16; 0x1920)).offset(v1 as isize) as *mut u8).offset(1,) as i8)
                    as u16,
            )
        );
        word!(game.hdma_object_B[v1], wrapping_add(80));
    }
}
pub unsafe fn FxTypeFunc_26_TourianEntranceStatue(game: &mut Game) {
    static unk_88DBBA: SpawnHdmaObject_Args =
        SpawnHdmaObject_Args::new(0x42, 0x11, 0xd856 as VoidP);
    static unk_88DBC2: SpawnHdmaObject_Args =
        SpawnHdmaObject_Args::new(0x42, 0x10, 0xdcfa as VoidP);
    if CheckEventHappened(0xa, game) & 1 != 0 {
        SpawnHardcodedPlm(SpawnHardcodedPlmArgs::new(0x6, 0xc, 0xb777 as VoidP), game);
        *(ram_ptr!(game, u16; 0xcd20)) = 514;
    }
    *ram_ptr!(game; 0x59) = 74;
    word!(game.fx_rising_function_bank_88 = ((0x88c428i32) as u16));
    word!(game.fx_y_pos = word!(game.fx_base_y_pos));
    SpawnHdmaObject(0x88, &raw const unk_88DBBA as *const c_void, game);
    SpawnHdmaObject(0x88, &raw const unk_88DBC2 as *const c_void, game);
    SpawnBG3ScrollHdmaObject(game);
}
pub unsafe fn sub_88DBCB(k: u16, game: &mut Game) {
    *(&raw mut (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00))).field_0 as *mut u16) =
        (word!(game.layer1_y_pos)).wrapping_add(word!(game.hdma_object_B[(k >> 1)]));
}
pub unsafe fn HdmaobjPreInstr_CheckLotsOfEventsHappened(mut v0: u16, game: &mut Game) {
    if CheckEventHappened(6, game) & 1 != 0
        && CheckEventHappened(7, game) & 1 != 0
        && CheckEventHappened(8, game) & 1 != 0
        && CheckEventHappened(9, game) & 1 != 0
    {
        word!(game.tourian_entrance_statue_animstate, |fresh90| (fresh90)
            | (0x10u16));
        if (word!(game.tourian_entrance_statue_animstate) & 0x8000) == 0 {
            word!(game.hdma_object_C[(v0 >> 1)] = 300);
            v0 = word!(game.hdma_object_index);
            let v1: i32 = word!(game.hdma_object_index) as i32 >> 1;
            word!(game.hdma_object_instruction_timers[v1] = 1);
            word!(
                game.hdma_object_instruction_list_pointers[v1],
                wrapping_add(2)
            );
        }
    }
    sub_88DBCB(v0, game);
}
pub unsafe fn HdmaobjPreInstr_DC23(k: u16, game: &mut Game) {
    let v0: u16 = k;
    HandleEarthquakeSoundEffect(game);
    word!(game.earthquake_type = 13);
    word!(game.earthquake_timer, |fresh87| (fresh87) | (0x20u16));
    let v1: i32 = v0 as u8 as i32 >> 1;
    if (word!(game.hdma_object_C[v1], wrapping_sub(1), reread) & 0x8000) != 0 {
        SpawnEprojWithRoomGfx(addr_kEproj_TourianStatueDustClouds as u16, 0, game);
        SpawnEprojWithRoomGfx(addr_kEproj_TourianStatueDustClouds as u16, 0, game);
        SpawnEprojWithRoomGfx(addr_kEproj_TourianStatueDustClouds as u16, 0, game);
        SpawnEprojWithRoomGfx(addr_kEproj_TourianStatueDustClouds as u16, 0, game);
        word!(game.hdma_object_instruction_timers[v1] = 1);
        word!(
            game.hdma_object_instruction_list_pointers[v1],
            wrapping_add(2)
        );
    }
    sub_88DBCB(v0, game);
}
pub unsafe fn HdmaobjPreInstr_DC69(k: u16, game: &mut Game) {
    HandleEarthquakeSoundEffect(game);
    word!(game.earthquake_type = 13);
    word!(game.earthquake_timer, |fresh85| (fresh85) | (0x20u16));
    if word!(game.time_is_frozen_flag) == 0 {
        let v1: i32 = k as i32 >> 1;
        AddToHiLo(
            (ram_ptr!(game, u16; 0x1920)).offset(v1 as isize),
            (ram_ptr!(game, u16; 0x1914)).offset(v1 as isize),
            -0x4000i32 as u32,
        );
        if word!(game.hdma_object_B[v1]) == 0xff10 {
            SpawnHardcodedPlm(SpawnHardcodedPlmArgs::new(0x6, 0xc, 0xb773 as VoidP), game);
            SetEventHappened(0xa, game);
            word!(game.hdma_object_instruction_timers[v1] = 1);
            word!(
                game.hdma_object_instruction_list_pointers[v1],
                wrapping_add(2)
            );
        }
        sub_88DBCB(k, game);
    }
}
pub unsafe fn HdmaobjPreInstr_DCBA(v0: u16, game: &mut Game) {
    word!(game.tourian_entrance_statue_finished = 0x8000);
    *(ram_ptr!(game, u16; 0xcd20)) = 514;
    sub_88DBCB(v0, game);
}
pub unsafe fn HdmaobjInstr_GotoIfEventHappened(
    k: u16,
    hdp: *const u8,
    game: &mut Game,
) -> *const u8 {
    let v2: i32 = k as i32 >> 1;
    word! { game, 0; hdma_object_C[v2], hdma_object_A[v2] }
    if CheckEventHappened(0xa, game) != 0 {
        word!(game.hdma_object_B[v2] = -240i32 as u16);
        *(&raw mut (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00))).field_0
            as *mut u16) = -240i32 as u16;
        *(hdp as *mut u16) as usize as *const u8
    } else {
        word!(game.hdma_object_B[v2] = 0);
        *(&raw mut (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00))).field_0
            as *mut u16) = 0;
        *(ram_ptr!(game, u16; 0xcd20)) = 1;
        hdp.offset(2)
    }
}
pub unsafe fn SpawnBombTorizoHaze(game: &mut Game) {
    static unk_88DD36: SpawnHdmaObject_Args = SpawnHdmaObject_Args::new(0x2, 0x32, 0xdd4a as VoidP);
    static unk_88DD3E: SpawnHdmaObject_Args = SpawnHdmaObject_Args::new(0, 0x32, 0xdd62 as VoidP);
    SpawnHdmaObject(0x88, &raw const unk_88DD36 as *const c_void, game);
    SpawnHdmaObject(0x88, &raw const unk_88DD3E as *const c_void, game);
}
pub unsafe fn HdmaobjPreInstr_BombTorizoHazeColorMathBgColor(k: u16, game: &mut Game) {
    word!(game.fx_layer_blending_config_c = 44);
}
pub unsafe fn FxTypeFunc_2C_Haze(game: &mut Game) {
    static unk_88DDD4: SpawnHdmaObject_Args =
        SpawnHdmaObject_Args::new(0x40, 0x32, 0xded3 as VoidP);
    static unk_88DDDD: SpawnHdmaObject_Args =
        SpawnHdmaObject_Args::new(0x40, 0x32, 0xdeeb as VoidP);
    if CheckBossBitForCurArea(1, game) & 1 != 0 {
        SpawnHdmaObject(0x88, &raw const unk_88DDDD as *const c_void, game);
    } else {
        SpawnHdmaObject(0x88, &raw const unk_88DDD4 as *const c_void, game);
    };
}
crate::forward_calls! { pub unsafe fn (k: u16, game: &mut Game);
    HdmaobjPreInstr_HazeColorMathSubscreen_CeresRidleyAlive => sub_88DE18(k, 0x80, game);
    HdmaobjPreInstr_HazeColorMathSubscreen_CeresRidleyDead => sub_88DE18(k, 0x20, game);
}
pub unsafe fn sub_88DE18(k: u16, a: u16, game: &mut Game) {
    let v2: i32 = k as u8 as i32 >> 1;
    word! {game; hdma_object_B[v2] = a; hdma_object_A[v2] = 0; }
    if word!(game.door_transition_function) as i32 == 0x82e737 & 0xffff {
        word!(game.hdma_object_pre_instructions[v2] = ((0x88de2di32) as u16));
        HdmaobjPreInstr_HazeColorMathSubscreen_FadingIn(k, game);
    }
}
pub unsafe fn HdmaobjPreInstr_HazeColorMathSubscreen_FadingIn(k: u16, game: &mut Game) {
    *ram_ptr!(game; 0x74, 0) = 32;
    *ram_ptr!(game; 0x74, 1) = 64;
    *ram_ptr!(game; 0x74, 2) = 0x80;
    word!(game.fx_layer_blending_config_c = 44);
    if word!(game.hdma_object_A[(k as u8 >> 1)]) == 16 {
        word!(game.hdma_object_pre_instructions[(k as u8 >> 1)] = ((0x88de74i32) as u16));
    } else {
        let v3: u8 = k as u8;
        let r20: u8 = byte!(@pointer ram_ptr!(game, u16; 0x1920); k as u8 as i32 as isize);
        let mut v1: i8 = byte!(@pointer ram_ptr!(game, u16; 0x1914); k as u8 as i32 as isize) as i8;
        for i in (0i32..=15).rev() {
            let r18: u8 = v1 as u8;
            byte!(@pointer ram_ptr!(game, u16; 0x9d00); i as u8 as i32 as isize) =
                (r20) | ((v1) as u8);
            v1 = ((r18) as i8).wrapping_sub(1);
            if ((r18) as i8).wrapping_sub(1) < 0 {
                v1 = 0;
            }
        }
        word!(game.hdma_object_A[(v3 >> 1)], wrapping_add(1));
    };
}
pub unsafe fn HdmaobjPreInstr_HazeColorMathSubscreen_FadedIn(k: u16, game: &mut Game) {
    *ram_ptr!(game; 0x74, 0) = 32;
    *ram_ptr!(game; 0x74, 1) = 64;
    *ram_ptr!(game; 0x74, 2) = 0x80;
    word!(game.fx_layer_blending_config_c = 44);
    if word!(game.door_transition_function) as i32 == 0x82e2db & 0xffff {
        word!(game.hdma_object_pre_instructions[(k as u8 >> 1)] = ((0x88de96i32) as u16));
    }
}
pub unsafe fn HdmaobjPreInstr_HazeColorMathSubscreen_FadingOut(k: u16, game: &mut Game) {
    *ram_ptr!(game; 0x74, 0) = 32;
    *ram_ptr!(game; 0x74, 1) = 64;
    *ram_ptr!(game; 0x74, 2) = 0x80;
    word!(game.fx_layer_blending_config_c = 44);
    if word!(game.hdma_object_A[(k as u8 >> 1)]) != 0 {
        let v3: u8 = k as u8;
        let r20: u8 = byte!(@pointer ram_ptr!(game, u16; 0x1920); k as u8 as i32 as isize);
        let mut v1: i8 = byte!(@pointer ram_ptr!(game, u16; 0x1914); k as u8 as i32 as isize) as i8;
        for i in (0i32..=15).rev() {
            let r18: u8 = v1 as u8;
            byte!(@pointer ram_ptr!(game, u16; 0x9d00); i as u8 as i32 as isize) =
                (r20) | ((v1) as u8);
            v1 = ((r18) as i8).wrapping_sub(1);
            if ((r18) as i8).wrapping_sub(1) < 0 {
                v1 = 0;
            }
        }
        word!(game.hdma_object_A[(v3 >> 1)], wrapping_sub(1));
    }
}
static unk_88DF38: SpawnHdmaObject_Args = SpawnHdmaObject_Args::new(0, 0x2c, 0xdf4f as VoidP);
static unk_88DF41: SpawnHdmaObject_Args = SpawnHdmaObject_Args::new(0, 0x2c, 0xdf6b as VoidP);
static unk_88DF4A: SpawnHdmaObject_Args = SpawnHdmaObject_Args::new(0x2, 0x12, 0xdf77 as VoidP);
crate::forward_calls! { pub unsafe fn (game: &mut Game);
    sub_88DF34 => SpawnHdmaObject(0x88, &raw const unk_88DF38 as *const c_void, game);
    sub_88DF3D => SpawnHdmaObject(0x88, &raw const unk_88DF41 as *const c_void, game);
    sub_88DF46 => SpawnHdmaObject(0x88, &raw const unk_88DF4A as *const c_void, game);
}
pub unsafe fn HdmaobjPreInstr_DF94(v0: u16, game: &mut Game) {
    let mut v1: i16 = 0;
    if ((*(ram_ptr!(game, EnemyData; 0xf78))).properties & 0x200) == 0
        && (((*(ram_ptr!(game, EnemyData; 0xf78))).x_pos as i32 - word!(game.layer1_x_pos) as i32
            + 64) as i16)
            >= 0
        && (*(ram_ptr!(game, EnemyData; 0xf78))).x_pos as i32
            - word!(game.layer1_x_pos) as i32
            - 320i32
            & 0x8000
            != 0
        && (((*(ram_ptr!(game, EnemyData; 0xf78))).y_pos as i32 - word!(game.layer1_y_pos) as i32
            + 16) as i16)
            >= 0
        && {
            v1 = (((*(ram_ptr!(game, EnemyData; 0xf78))).y_pos) as i16)
                .wrapping_sub((word!(game.layer1_y_pos)) as i16);
            (*(ram_ptr!(game, EnemyData; 0xf78))).y_pos as i32
                - word!(game.layer1_y_pos) as i32
                - 304i32
                & 0x8000
                != 0
        }
    {
        if v1 as i32 - 40i32 & 0x8000 != 0 {
            let v2: i32 = v0 as i32 >> 1;
            word!(game.hdma_object_instruction_timers[v2] = 1);
            word!(game.hdma_object_instruction_list_pointers[v2] = addr_word_88DF6B as u16);
        } else {
            let v4: i32 = v0 as i32 >> 1;
            word!(game.hdma_object_instruction_timers[v4] = 1);
            if v1 as i32 - 192i32 & 0x8000 != 0 {
                word!(game.hdma_object_instruction_list_pointers[v4] = addr_word_88DF5F as u16);
            } else {
                word!(game.hdma_object_instruction_list_pointers[v4] = addr_word_88DF65 as u16);
            }
        }
    } else {
        let v3: i32 = v0 as i32 >> 1;
        word!(game.hdma_object_instruction_timers[v3] = 1);
        word!(game.hdma_object_instruction_list_pointers[v3] = addr_word_88DF71 as u16);
    };
}
static off_88E04E: [Func_U8; 7] = callback_table! { nonnull Func_U8;
    VariaSuitPickup_0_LightBeamAppears, VariaSuitPickup_1, VariaSuitPickup_2_LightBeamWidens,
    VariaSuitPickup_3, VariaSuitPickup_4_LightBeamShrinks,
    VariaSuitPickup_5_LightBeamDissipates, VariaSuitPickup_6,
};
pub unsafe fn HdmaobjPreInstr_VariaSuitPickup(k: u16, game: &mut Game) {
    if off_88E04E[word!(game.substate) as usize](game) & 1 != 0 {
        *ram_ptr!(game; 0x74, 0) = *ram_ptr!(game; 0xdf0);
        *ram_ptr!(game; 0x74, 1) = *ram_ptr!(game; 0xdf1);
        *ram_ptr!(game; 0x74, 2) = *ram_ptr!(game; 0xdf2);
        word!(game.fx_layer_blending_config_c = 18);
    }
}
static off_88E084: [Func_U8; 7] = callback_table! { nonnull Func_U8;
    VariaSuitPickup_0_LightBeamAppears, VariaSuitPickup_1, VariaSuitPickup_2_LightBeamWidens,
    GravitySuitPickup_3, VariaSuitPickup_4_LightBeamShrinks,
    VariaSuitPickup_5_LightBeamDissipates, GravitySuitPickup_6,
};
pub unsafe fn HdmaobjPreInstr_GravitySuitPickup(k: u16, game: &mut Game) {
    if off_88E084[word!(game.substate) as usize](game) & 1 != 0 {
        *ram_ptr!(game; 0x74, 0) = *ram_ptr!(game; 0xdf0);
        *ram_ptr!(game; 0x74, 1) = *ram_ptr!(game; 0xdf1);
        *ram_ptr!(game; 0x74, 2) = *ram_ptr!(game; 0xdf2);
        word!(game.fx_layer_blending_config_c = 18);
    }
}
static g_word_88E0D5: u16 = 0x7878;
pub unsafe fn VariaSuitPickup_0_LightBeamAppears(game: &mut Game) -> u8 {
    word!(game.suit_pickup_light_beam_pos, wrapping_add(8));
    let mut v0: i16 = word!(game.suit_pickup_light_beam_pos) as i16;
    let mut v1: u16 = 0;
    do_until! { (v0 <= 0);
        word!(game.hdma_table_1[(v1 >> 1)] = g_word_88E0D5);
        v1 = (v1).wrapping_add(2);
        v0 -= 1;
    }
    let mut v2: u16 = 510;
    let mut v3: i16 = word!(game.suit_pickup_light_beam_pos) as i16;
    do_until! { (v3 <= 0);
        word!(game.hdma_table_1[(v2 >> 1)] = g_word_88E0D5);
        v2 = (v2).wrapping_sub(2);
        v3 -= 1;
    }
    if word!(game.suit_pickup_light_beam_pos) as i32 - 128i32 & 0x8000 == 0 {
        word!(game.substate, wrapping_add(1));
        word!(game.suit_pickup_light_beam_pos = 30840);
    }
    1
}
pub unsafe fn VariaSuitPickup_1(game: &mut Game) -> u8 {
    byte!(@pointer ram_ptr!(game, u16; 0xdee); 0) = ((word!(game.suit_pickup_light_beam_pos))
        as u8)
        .wrapping_sub(byte!(@pointer ram_ptr!(game, u16; 0xddc); 1));
    crate::update_place! { reference fresh17 = *(ram_ptr!(game, u16; 0xdee) as *mut u8).offset(1) => (*fresh17).wrapping_add(*(ram_ptr!(game, u16; 0xddc) as *mut u8).offset(1)); }
    for i in (0i32..=510).rev().step_by(2) {
        word!(game.hdma_table_1[(i >> 1)] = word!(game.suit_pickup_light_beam_pos));
    }
    if word!(game.suit_pickup_light_beam_pos) as u8 as i32 - 97i32 & 0x8000 != 0 {
        word!(game.substate, wrapping_add(1));
        word!(game.suit_pickup_light_beam_pos = 0x846c);
    }
    1
}

macro_rules! widen_suit_pickup_beam_row {
    ($game:ident, $width_index:ident, $output_offset:ident) => {
        let mut v4: i8 = ((word!($game.suit_pickup_light_beam_pos)) as i8).wrapping_sub(
            (byte!(
                @pointer $crate::rtl::rom_ptr!($game; 0x88e3c9);
                $width_index as isize
            )) as i8,
        );
        if v4 < 0 {
            v4 = 0;
        }
        byte!(@pointer ram_ptr!($game, u16; 0x9800); $output_offset as u16 as i32 as isize) =
            v4 as u8;
        let v5: u16 = (($output_offset) as u16).wrapping_add(1);
        let mut v6: i8 = ((byte!(
            @pointer $crate::rtl::rom_ptr!($game; 0x88e3c9);
            $width_index as isize
        )) as i8)
            .wrapping_add((byte!(@pointer ram_ptr!($game, u16; 0xdee); 1)) as i8);
        if u8::overflowing_add(
            byte!(
                @pointer $crate::rtl::rom_ptr!($game; 0x88e3c9);
                $width_index as isize
            ),
            byte!(@pointer ram_ptr!($game, u16; 0xdee); 1isize),
        )
        .1
        {
            v6 = -1i8;
        }
        byte!(@pointer ram_ptr!($game, u16; 0x9800); v5 as i32 as isize) = v6 as u8;
        $output_offset = ((v5) as i16).wrapping_add(1);
    };
}

pub unsafe fn VariaSuitPickup_2_LightBeamWidens(game: &mut Game) -> u8 {
    AdvanceSuitPickupColorMathSubscreenToWhite(game);
    let v0: bool = ((word!(game.suit_pickup_light_beam_pos)) as i8)
        .wrapping_sub((byte!(@pointer ram_ptr!(game, u16; 0xddc); 1)) as i8)
        < 0;
    byte!(@pointer ram_ptr!(game, u16; 0xdee); 0) = ((word!(game.suit_pickup_light_beam_pos))
        as u8)
        .wrapping_sub(byte!(@pointer ram_ptr!(game, u16; 0xddc); 1));
    if word!(game.suit_pickup_light_beam_pos) as u8 == 0 || v0 {
        word!(game.suit_pickup_light_beam_pos = -256i32 as u16);
    } else {
        let mut v1: i8 = ((byte!(@pointer ram_ptr!(game, u16; 0xddc); 1isize)) as i8)
            .wrapping_add((byte!(@pointer ram_ptr!(game, u16; 0xdee); 1)) as i8);
        if u8::overflowing_add(
            byte!(@pointer ram_ptr!(game, u16; 0xddc); 1isize),
            byte!(@pointer ram_ptr!(game, u16; 0xdee); 1isize),
        )
        .1
        {
            v1 = -1i8;
        }
        byte!(@pointer ram_ptr!(game, u16; 0xdee); 1) = v1 as u8;
    }
    let mut v2: i16 = 0;
    let mut v3: u16 = 0;
    do_until! { (v2 >= 0x100);
        widen_suit_pickup_beam_row!(game, v3, v2);
        v3 = v3.wrapping_add(1);
    }
    let mut v7: u16 = (v3).wrapping_sub(1);
    do_until! { (v2 >= 0x200);
        widen_suit_pickup_beam_row!(game, v7, v2);
        v7 = v7.wrapping_sub(1);
    }
    word!(game.suit_pickup_light_beam_widening_speed, wrapping_add(96));
    if word!(game.suit_pickup_light_beam_pos) == 0xff00 {
        word!(game.substate, wrapping_add(1));
        word!(
            game.suit_pickup_light_beam_widening_speed,
            |fresh21| (fresh21 >> 1)
        );
        word!(game.suit_pickup_light_beam_pos = 0);
    }
    1
}
pub unsafe fn VariaSuitPickup_4_LightBeamShrinks(game: &mut Game) -> u8 {
    if *ram_ptr!(game; 0xdf3) != 0 {
        AdvanceSuitPickupColorMathToBlue(game);
    } else {
        AdvanceSuitPickupColorMathToOrange(game);
    }
    word!(game.suit_pickup_light_beam_pos, |fresh25| (fresh25 as i32
        + ((word!(game.suit_pickup_light_beam_widening_speed) as i32 & 0xff00) >> 8))
        as u16);
    let mut v0: i16 = word!(game.suit_pickup_light_beam_pos) as i16;
    let mut v1: u16 = 0;
    do_until! { (v0 <= 0);
        word!(game.hdma_table_1[(v1 >> 1)] = 255);
        v1 = (v1).wrapping_add(2);
        v0 -= 1;
    }
    let mut v2: u16 = 510;
    let mut v3: i16 = ((word!(game.suit_pickup_light_beam_pos)) as i16).wrapping_sub(1);
    do_until! { (v3 <= 0);
        word!(game.hdma_table_1[(v2 >> 1)] = 255);
        v2 = (v2).wrapping_sub(2);
        v3 -= 1;
    }
    word!(game.suit_pickup_light_beam_widening_speed, wrapping_sub(32));
    if word!(game.suit_pickup_light_beam_widening_speed) as i32 - 256i32 & 0x8000 != 0 {
        word!(game.suit_pickup_light_beam_widening_speed = 256);
    }
    if word!(game.suit_pickup_light_beam_pos) as i32 - 128i32 & 0x8000 == 0 {
        word!(game.substate, wrapping_add(1));
        word!(game.suit_pickup_light_beam_pos = -1793i32 as u16);
    }
    1
}
pub unsafe fn VariaSuitPickup_5_LightBeamDissipates(game: &mut Game) -> u8 {
    byte!(@pointer ram_ptr!(game, u16; 0xdee); 0) =
        ((word!(game.suit_pickup_light_beam_pos)) as u8).wrapping_add(8);
    crate::update_place! { reference fresh28 = *(ram_ptr!(game, u16; 0xdee) as *mut u8).offset(1) => (*fresh28).wrapping_sub(8); }
    word!(game.hdma_table_1[128] = word!(game.suit_pickup_light_beam_pos));
    if word!(game.suit_pickup_light_beam_pos) as u8 as i32 - 112i32 & 0x8000 == 0 {
        word!(game.substate, wrapping_add(1));
    }
    1
}
pub unsafe fn VariaSuitPickup_6(game: &mut Game) -> u8 {
    QueueMusic_Delayed8(3, game);
    GravitySuitPickup_6(game)
}
pub unsafe fn GravitySuitPickup_6(game: &mut Game) -> u8 {
    *ram_ptr!(game; 0x74, 2) = 0x80;
    *ram_ptr!(game; 0x74, 1) = 64;
    *ram_ptr!(game; 0x74, 0) = 32;
    mov24(ram_ptr!(game, LongPtr; 0xa88), 0x980001);
    *((ram_ptr!(game, u16; 0xa8a) as *mut u8).offset(1) as *mut u16) = 0;
    word! { game, 0;
        demo_input_prev, demo_input_prev_new, demo_backup_prev_controller_input,
    }
    word!(game.hdma_table_1[0] = 255);
    word! { game, 0; substate, suit_pickup_light_beam_pos }
    *(ram_ptr!(game, u16; 0xdf0)) = 0;
    *(ram_ptr!(game, u16; 0xdf2)) = 0;
    let v0: i32 = word!(game.hdma_object_index) as i32 >> 1;
    word!(
        game.hdma_object_instruction_list_pointers[v0],
        wrapping_add(2)
    );
    word!(game.hdma_object_instruction_timers[v0] = 1);
    CallSomeSamusCode(0xb, game);
    0
}
pub unsafe fn AdvanceSuitPickupColorMathSubscreenToWhite(game: &mut Game) -> u8 {
    if *ram_ptr!(game; 0xdf0) != 63 {
        crate::update_place! { reference fresh0 = *ram_ptr!(game; 0xdf0) => (*fresh0).wrapping_add(2); }
        if *ram_ptr!(game; 0xdf0) as i32 - 64i32 & 0x80 == 0 {
            *ram_ptr!(game; 0xdf0) = 63;
        }
    }
    if *ram_ptr!(game; 0xdf1) != 95 {
        crate::update_place! { reference fresh1 = *ram_ptr!(game; 0xdf1) => (*fresh1).wrapping_add(2); }
        if *ram_ptr!(game; 0xdf1) as i32 - 96i32 & 0x80 == 0 {
            *ram_ptr!(game; 0xdf1) = 95;
        }
    }
    if *ram_ptr!(game; 0xdf2) != 0x9f {
        crate::update_place! { reference fresh2 = *ram_ptr!(game; 0xdf2) => (*fresh2).wrapping_add(2); }
        if *ram_ptr!(game; 0xdf2) as i32 + 96 & 0x80 == 0 {
            *ram_ptr!(game; 0xdf2) = -97i32 as u8;
        }
    }
    1
}
pub unsafe fn AdvanceSuitPickupColorMathToOrange(game: &mut Game) -> u8 {
    if *ram_ptr!(game; 0xdf0) != 63 {
        crate::update_place! { reference fresh6 = *ram_ptr!(game; 0xdf0) => (*fresh6).wrapping_sub(1); }
    }
    if *ram_ptr!(game; 0xdf1) != 77 {
        crate::update_place! { reference fresh7 = *ram_ptr!(game; 0xdf1) => (*fresh7).wrapping_sub(1); }
    }
    if *ram_ptr!(game; 0xdf2) != 0x83 {
        crate::update_place! { reference fresh8 = *ram_ptr!(game; 0xdf2) => (*fresh8).wrapping_sub(1); }
    }
    1
}
pub unsafe fn VariaSuitPickup_3(game: &mut Game) -> u8 {
    word!(game.equipped_items, |fresh22| (fresh22) | (1u16));
    word!(game.collected_items, |fresh23| (fresh23) | (1u16));
    word!(game.samus_pose = kPose_9B_FaceF_VariaGravitySuit as u16);
    crate::game::samus_pose::finish_pose_change(game);
    Samus_LoadSuitPalette(game);
    word!(game.substate, wrapping_add(1));
    1
}
pub unsafe fn GravitySuitPickup_3(game: &mut Game) -> u8 {
    word!(game.equipped_items, |fresh9| (fresh9) | (0x20u16));
    word!(game.collected_items, |fresh10| (fresh10) | (0x20u16));
    word!(game.samus_pose = kPose_9B_FaceF_VariaGravitySuit as u16);
    crate::game::samus_pose::finish_pose_change(game);
    Samus_LoadSuitPalette(game);
    word!(game.substate, wrapping_add(1));
    1
}
pub unsafe fn AdvanceSuitPickupColorMathToBlue(game: &mut Game) -> u8 {
    if *ram_ptr!(game; 0xdf0) != 48 {
        crate::update_place! { reference fresh3 = *ram_ptr!(game; 0xdf0) => (*fresh3).wrapping_sub(1); }
    }
    if *ram_ptr!(game; 0xdf1) != 73 {
        crate::update_place! { reference fresh4 = *ram_ptr!(game; 0xdf1) => (*fresh4).wrapping_sub(1); }
    }
    if *ram_ptr!(game; 0xdf2) != 0x90 {
        crate::update_place! { reference fresh5 = *ram_ptr!(game; 0xdf2) => (*fresh5).wrapping_sub(1); }
    }
    1
}
pub unsafe fn HdmaobjPreInstr_E449(k: u16, game: &mut Game) {
    if (word!(game.phantom_related_layer_flag) & 0x4000) != 0 {
        word!(game.fx_layer_blending_config_c = 26);
    } else if byte!((*(ram_ptr!(game, EnemyData; 0xf78)).offset(3)).parameter_1; 0) != 0 {
        if byte!((*(ram_ptr!(game, EnemyData; 0xf78)).offset(3)).parameter_1; 0) == 0xff {
            word!(game.fx_layer_blending_config_c = 4);
            let v0: i32 = word!(game.hdma_object_index) as i32 >> 1;
            word!(
                game.hdma_object_instruction_list_pointers[v0],
                wrapping_add(2)
            );
            word!(game.hdma_object_instruction_timers[v0] = 1);
        }
    } else {
        word!(game.fx_layer_blending_config_c = 4);
    };
}
static unk_88E4A0: SpawnHdmaObject_Args = SpawnHdmaObject_Args::new(0x42, 0xf, 0xe4a8 as VoidP);
pub unsafe fn sub_88E487(v0: u16, r22: u16, game: &mut Game) {
    (*(ram_ptr!(game, EnemyData; 0xf78)).offset(2)).parameter_1 = v0;
    set_fields! {[(*(ram_ptr!(game, EnemyData; 0xf78)).offset(3))], 0; ai_var_D, ai_var_E}
    (*(ram_ptr!(game, EnemyData; 0xf78)).offset(3)).ai_preinstr = r22;
    SpawnHdmaObject(0x88, &raw const unk_88E4A0 as *const c_void, game);
}
pub unsafe fn HdmaobjInstr_E4BD(k: u16, hdp: *const u8, game: &mut Game) -> *const u8 {
    (*(ram_ptr!(game, EnemyData; 0xf78)).offset(1)).parameter_1 =
        (*(ram_ptr!(game, EnemyData; 0xf78)).offset(2)).parameter_1;
    if ((*(ram_ptr!(game, EnemyData; 0xf78)).offset(2)).parameter_1 & 1) != 0 {
        word! { game, 192; kraid_unk9000, g_word_7E9006 }
        let mut v4: u32 = 12620032i32 as u32;
        *((ram_ptr!(game, u16; 0x9002) as *mut u8).offset(1) as *mut u16) =
            *(&raw mut v4 as *mut u16).offset(1);
        *((ram_ptr!(game, u16; 0x9000) as *mut u8).offset(1) as *mut u16) = v4 as u16;
        word!(game.g_word_7E9004 = -28288i32 as u16);
        let mut v5: u32 = 12620032i32 as u32;
        *((ram_ptr!(game, u16; 0x9008) as *mut u8).offset(1) as *mut u16) =
            *(&raw mut v5 as *mut u16).offset(1);
        *((ram_ptr!(game, u16; 0x9006) as *mut u8).offset(1) as *mut u16) = v5 as u16;
        word! {game; g_word_7E900A = addr_loc_889180 as u16; g_word_7E900C = 0; }
    } else {
        word! { game, 160;
            kraid_unk9000, g_word_7E9006, g_word_7E900C, g_word_7E900F, g_word_7E9012,
            g_word_7E9015,
        }
        let mut v2: u32 = 10522880i32 as u32;
        *((ram_ptr!(game, u16; 0x9002) as *mut u8).offset(1) as *mut u16) =
            *(&raw mut v2 as *mut u16).offset(1);
        *((ram_ptr!(game, u16; 0x9000) as *mut u8).offset(1) as *mut u16) = v2 as u16;
        let mut v3: u32 = 10522880i32 as u32;
        *((ram_ptr!(game, u16; 0x9008) as *mut u8).offset(1) as *mut u16) =
            *(&raw mut v3 as *mut u16).offset(1);
        *((ram_ptr!(game, u16; 0x9006) as *mut u8).offset(1) as *mut u16) = v3 as u16;
        *((ram_ptr!(game, u16; 0x900c) as *mut u8).offset(1) as *mut u16) = -28416i32 as u16;
        *((ram_ptr!(game, u16; 0x9012) as *mut u8).offset(1) as *mut u16) = -28416i32 as u16;
        word! {game; g_word_7E9004 = -28352i32 as u16; g_word_7E900A = -28352i32 as u16; }
        *((ram_ptr!(game, u16; 0x900f) as *mut u8).offset(1) as *mut u16) = -28352i32 as u16;
        *((ram_ptr!(game, u16; 0x9015) as *mut u8).offset(1) as *mut u16) = -28352i32 as u16;
        word!(game.g_word_7E9018 = 0);
    }
    let v6: i32 = k as i32 >> 1;
    word! {game; hdma_object_A[v6] = -2i32 as u16; hdma_object_B[v6] = 1; }
    word! { game, 0; hdma_object_C[v6], hdma_object_D[v6] }
    hdp
}
// Keep bits 16..23 of the magnitude product before restoring the sine sign.
fn hdma_wave_offset(sample: i16, amplitude_low: u8, amplitude_high: u8) -> u16 {
    let amplitude = u16::from_le_bytes([amplitude_low, amplitude_high]);
    let product = u32::from(sample.unsigned_abs()) * u32::from(amplitude);
    let offset = (product >> 16) as u8 as u16;
    if sample < 0 {
        offset.wrapping_neg()
    } else {
        offset
    }
}

pub unsafe fn HdmaobjPreInstr_E567(v0: u16, game: &mut Game) {
    let mut r28: u16 = 0;
    let mut r30: u16 = 0;
    if (*(ram_ptr!(game, EnemyData; 0xf78)).offset(1)).parameter_1 != 0 {
        if ((*(ram_ptr!(game, EnemyData; 0xf78)).offset(1)).parameter_1 & 1) != 0 {
            r28 = 4;
            r30 = 128;
        } else {
            r28 = 8;
            r30 = 64;
        }
        let v2: i32 = v0 as i32 >> 1;
        crate::update_place! { unaligned_word fresh79 [fresh80] = &raw mut (*(ram_ptr!(game, u16; 0x1920)).offset(v2 as isize)) => (u16::from_le(fresh79.read_unaligned())).wrapping_sub(1); }
        if fresh80 == 1 {
            word!(game.hdma_object_B[v2] = 1);
            word!(
                game.hdma_object_A[v2] = (2
                    * (*(ram_ptr!(game, EnemyData; 0xf78)).offset(3)).ai_preinstr as i32
                    + word!(game.hdma_object_A[v2]) as i32
                    & 0x1ff) as u16
            );
        }
        let mut r20: u16 = word!(game.hdma_object_A[v2]);
        let mut v4: u16 = 0;
        do_until! { (((v4) as i16).wrapping_sub((r30) as i16) >= 0);
            let v5: u16 = r20;
            let v6: i32 = r20 as i32 >> 1;
            let sample = kSinCosTable8bit_Sext[(v6 + 64) as usize];
            let offset = hdma_wave_offset(
                sample,
                (*(ram_ptr!(game, EnemyData; 0xf78)).offset(3)).ai_var_D as u8,
                byte!((*(ram_ptr!(game, EnemyData; 0xf78)).offset(3)).ai_var_D; 1),
            );
            r20 = (r28 as i32 + v5 as i32 & 0x1ff) as u16;
            *((ram_ptr!(game, u16; 0x9100) as *mut u8).offset(v4 as i32 as isize) as *mut u16) =
                word!(game.reg_BG2HOFS).wrapping_add(offset);
            v4 = v4.wrapping_add(2);
        }
        if ((*(ram_ptr!(game, EnemyData; 0xf78)).offset(1)).parameter_1 & 1) != 0 {
            for i in (0i32..=126).rev().step_by(2) {
                *((ram_ptr!(game, u16; 0x9180) as *mut u8).offset(i as isize) as *mut u16) =
                    (word!(game.reg_BG2HOFS) as i32 + word!(game.reg_BG2HOFS) as i32
                        - *((ram_ptr!(game, u16; 0x9100) as *mut u8).offset(i as isize) as *mut u16)
                            as i32) as u16;
            }
        } else {
            for j in (0u16..=62).rev().step_by(2) {
                *((ram_ptr!(game, u16; 0x9140) as *mut u8).offset(j as i32 as isize) as *mut u16) =
                    (word!(game.reg_BG2HOFS) as i32 + word!(game.reg_BG2HOFS) as i32
                        - *((ram_ptr!(game, u16; 0x9100) as *mut u8).offset(j as i32 as isize)
                            as *mut u16) as i32) as u16;
            }
        }
    } else {
        let v1: i32 = word!(game.hdma_object_index) as i32 >> 1;
        word!(
            game.hdma_object_instruction_list_pointers[v1],
            wrapping_add(2)
        );
        word!(game.hdma_object_instruction_timers[v1] = 1);
    };
}
static unk_88E722: SpawnHdmaObject_Args = SpawnHdmaObject_Args::new(0, 0x2c, 0xe727 as VoidP);
static unk_88E74C: SpawnHdmaObject_Args = SpawnHdmaObject_Args::new(0x41, 0x26, 0xe751 as VoidP);
crate::forward_calls! { pub unsafe fn (game: &mut Game) -> u16;
    MotherBrainRisingHdmaObject =>
        SpawnHdmaObject(0x88, &raw const unk_88E722 as *const c_void, game);
    SpawnMotherBrainRainbowBeamHdma =>
        SpawnHdmaObject(0x88, &raw const unk_88E74C as *const c_void, game);
}
pub unsafe fn HdmaobjPreInstr_E7BC(k: u16, game: &mut Game) {
    if word!(game.game_state) == 19 {
        *(ram_ptr!(game, u16; 0x18b4)).offset((word!(game.hdma_object_index) >> 1) as isize) = 0;
    } else {
        word!(game.fx_layer_blending_config_c = 36);
        if word!(game.game_state) != 27 {
            MotherBrain_CalculateRainbowBeamHdma(game);
            sub_88E7ED(game);
        }
    };
}
pub unsafe fn sub_88E7ED(game: &mut Game) {
    let mut v0: u16 =
        crate::rtl::rom_table_word!(game, 0x88e833, (word!(game.hdma_object_A[0]) >> 1) as isize);
    if (v0 & 0x8000) == 0 {
        word!(game.hdma_object_A[0], wrapping_add(1));
        word!(game.hdma_object_A[0], wrapping_add(1));
        word!(game.hdma_object_A[0], wrapping_add(1));
        word!(game.hdma_object_A[0], wrapping_add(1));
    } else {
        word!(game.hdma_object_A[0] = 0);
        v0 = crate::rtl::rom_table_word!(game, 0x88e833, 0);
    }
    *ram_ptr!(game; 0x74, 0) = (v0 as u8 & 0x1f | 0x20);
    *ram_ptr!(game; 0x74, 1) = (v0 as i32 >> 5 & 0x1f | 0x40) as u8;
    *ram_ptr!(game; 0x74, 2) = ((v0 >> 2) as i32 >> 8 & 0x1f | 0x80) as u8;
}
pub unsafe fn SpawnMorphBallEyeBeamHdma(game: &mut Game) {
    static unk_88E8E4: SpawnHdmaObject_Args =
        SpawnHdmaObject_Args::new(0x41, 0x26, 0xe8ec as VoidP);
    SpawnHdmaObject(0x88, &raw const unk_88E8E4 as *const c_void, game);
}
pub fn HdmaobjInstr_InitMorphBallEyeBeamHdma(k: u16, hdp: *const u8, game: &mut Game) -> *const u8 {
    let color_slot = usize::from(k);
    game.g_ram[0x1914 + color_slot] = 0x30;
    game.g_ram[0x74] = 0x30;
    game.g_ram[0x1915 + color_slot] = 0x50;
    game.g_ram[0x75] = 0x50;
    game.g_ram[0x1920 + color_slot] = 0x80;
    game.g_ram[0x76] = 0x80;
    game.g_ram[0x1986..0x1988].copy_from_slice(&16u16.to_le_bytes());
    // The original stores also clear the byte after the HDMA terminator.
    game.g_ram[0x9000..0x900b].copy_from_slice(&[
        0xe4, 0x00, 0x91, 0xe4, 0xc8, 0x91, 0x98, 0x90, 0x92, 0x00, 0x00,
    ]);
    let active_offset = 0xf78 + 0x40 + core::mem::offset_of!(EnemyData, ai_var_C);
    game.g_ram[active_offset..active_offset + 2].copy_from_slice(&1u16.to_le_bytes());
    let slot = usize::from(k & !1);
    let width_offset = 0x192c + slot;
    game.g_ram[width_offset..width_offset + 2].copy_from_slice(&0u16.to_le_bytes());
    let width_fraction_offset = 0x1938 + slot;
    game.g_ram[width_fraction_offset..width_fraction_offset + 2]
        .copy_from_slice(&0u16.to_le_bytes());
    game.g_ram[0x9080..0x9084].fill(0);
    game.g_ram[0x9090..0x9092].fill(0);
    hdp
}
pub unsafe fn UpdateMorphBallEyeBeamHdma(k: u16, game: &mut Game) {
    // The beam's origin and angle belong to enemy slot 1.
    const BEAM_ENEMY: u16 = 0x40;
    let angle_offset = 0xf78 + usize::from(BEAM_ENEMY) + core::mem::offset_of!(EnemyData, ai_var_D);
    let angle = u16::from_le_bytes([game.g_ram[angle_offset], game.g_ram[angle_offset + 1]]);
    // HDMA object C uses a word index; the colors below keep the byte index.
    let width_offset = 0x192c + usize::from(k & !1);
    let width = u16::from_le_bytes([game.g_ram[width_offset], game.g_ram[width_offset + 1]]);
    let relative_y = game
        .enemy_coordinates(BEAM_ENEMY)
        .y_pos()
        .wrapping_sub(game.layer1_y_pos());
    let relative_x = game
        .enemy_coordinates(BEAM_ENEMY)
        .x_pos()
        .wrapping_sub(game.layer1_x_pos());
    calculate_xray_window(
        relative_x,
        relative_y,
        angle,
        width,
        relative_x >= 256,
        game.g_ram.as_mut_ptr().add(0x9100).cast(),
        game,
    );
    // Color sources can overlap the table, so read them after it is updated.
    let color_offset = usize::from(k);
    game.g_ram[0x74] = game.g_ram[0x1914 + color_offset];
    game.g_ram[0x75] = game.g_ram[0x1915 + color_offset];
    game.g_ram[0x76] = game.g_ram[0x1920 + color_offset];
}
pub unsafe fn HdmaobjPreInstr_MorphBallEyeBeamWidening(k: u16, game: &mut Game) {
    fn add_fixed_point(ram: &mut [u8], whole: usize, fraction: usize, increment: u32) -> u32 {
        let value =
            u32::from_le_bytes([ram[fraction], ram[fraction + 1], ram[whole], ram[whole + 1]])
                .wrapping_add(increment);
        let bytes = value.to_le_bytes();
        ram[fraction..fraction + 2].copy_from_slice(&bytes[..2]);
        ram[whole..whole + 2].copy_from_slice(&bytes[2..]);
        value
    }

    // Select the eye-beam layer blending configuration.
    game.g_ram[0x1986..0x1988].copy_from_slice(&16u16.to_le_bytes());
    let widening_speed = add_fixed_point(&mut game.g_ram, 0x9080, 0x9082, 0x4000);
    let slot = usize::from(k & !1);
    let width_offset = 0x192c + slot;
    let fraction_offset = 0x1938 + slot;
    let width = add_fixed_point(
        &mut game.g_ram,
        width_offset,
        fraction_offset,
        widening_speed,
    );
    let whole_width = (width >> 16) as u16;
    if (whole_width.wrapping_sub(4) as i16) >= 0 {
        // Keep the fractional width when the integer part reaches the limit.
        game.g_ram[width_offset..width_offset + 2].copy_from_slice(&4u16.to_le_bytes());
        let current_slot = u16::from_le_bytes([game.g_ram[0x18b2], game.g_ram[0x18b3]]);
        let current_offset = usize::from(current_slot & !1);
        let instruction_offset = 0x18cc + current_offset;
        let next_instruction = u16::from_le_bytes([
            game.g_ram[instruction_offset],
            game.g_ram[instruction_offset + 1],
        ])
        .wrapping_add(2);
        game.g_ram[instruction_offset..instruction_offset + 2]
            .copy_from_slice(&next_instruction.to_le_bytes());
        let timer_offset = 0x18e4 + current_offset;
        game.g_ram[timer_offset..timer_offset + 2].copy_from_slice(&1u16.to_le_bytes());
    }
    UpdateMorphBallEyeBeamHdma(k, game);
}
pub unsafe fn HdmaobjPreInstr_MorphBallEyeBeamFull(k: u16, game: &mut Game) {
    game.g_ram[0x1986..0x1988].copy_from_slice(&16u16.to_le_bytes());
    // Enemy slot 1 signals whether the eye is still active.
    let active_offset = 0xf78 + 0x40 + core::mem::offset_of!(EnemyData, ai_var_C);
    let active = u16::from_le_bytes([game.g_ram[active_offset], game.g_ram[active_offset + 1]]);
    if active != 0 {
        UpdateMorphBallEyeBeamHdma(k, game);
        let color_index = u16::from_le_bytes([game.g_ram[0x9090], game.g_ram[0x9091]]);
        let color_offset = usize::from(color_index.wrapping_mul(4));
        let slot = usize::from(k);
        game.g_ram[0x1914 + slot] = game.read_rom_byte(0x88ea8b, color_offset);
        game.g_ram[0x1915 + slot] = game.read_rom_byte(0x88ea8b, color_offset + 1);
        game.g_ram[0x1920 + slot] = game.read_rom_byte(0x88ea8b, color_offset + 2);
        // Color destinations can overlap the index, so advance its live value.
        let next_color_index =
            u16::from_le_bytes([game.g_ram[0x9090], game.g_ram[0x9091]]).wrapping_add(1) & 0xf;
        game.g_ram[0x9090..0x9092].copy_from_slice(&next_color_index.to_le_bytes());
    } else {
        let current_slot = u16::from_le_bytes([game.g_ram[0x18b2], game.g_ram[0x18b3]]);
        let current_offset = usize::from(current_slot & !1);
        let instruction_offset = 0x18cc + current_offset;
        let next_instruction = u16::from_le_bytes([
            game.g_ram[instruction_offset],
            game.g_ram[instruction_offset + 1],
        ])
        .wrapping_add(2);
        game.g_ram[instruction_offset..instruction_offset + 2]
            .copy_from_slice(&next_instruction.to_le_bytes());
        let timer_offset = 0x18e4 + current_offset;
        game.g_ram[timer_offset..timer_offset + 2].copy_from_slice(&1u16.to_le_bytes());
    }
}
pub unsafe fn HdmaobjPreInstr_MorphBallEyeBeamFading(k: u16, game: &mut Game) {
    game.g_ram[0x1986..0x1988].copy_from_slice(&16u16.to_le_bytes());
    if game.g_ram[0x1915 + usize::from(k)] == 0x40 {
        game.g_ram[0x74..0x77].copy_from_slice(&[0x20, 0x40, 0x80]);
        let slot = usize::from(k & !1);
        let width_fraction_offset = 0x1938 + slot;
        game.g_ram[width_fraction_offset..width_fraction_offset + 2]
            .copy_from_slice(&0u16.to_le_bytes());
        let width_offset = 0x192c + slot;
        game.g_ram[width_offset..width_offset + 2].copy_from_slice(&0u16.to_le_bytes());
        for entry in game.g_ram[0x9100..0x9300].chunks_exact_mut(2).rev() {
            entry.copy_from_slice(&0xffu16.to_le_bytes());
        }
        game.g_ram[0x9000..0x900a].fill(0);

        let current_slot = u16::from_le_bytes([game.g_ram[0x18b2], game.g_ram[0x18b3]]);
        let current_offset = usize::from(current_slot & !1);
        let instruction_offset = 0x18cc + current_offset;
        let next_instruction = u16::from_le_bytes([
            game.g_ram[instruction_offset],
            game.g_ram[instruction_offset + 1],
        ])
        .wrapping_add(2);
        game.g_ram[instruction_offset..instruction_offset + 2]
            .copy_from_slice(&next_instruction.to_le_bytes());
        let timer_offset = 0x18e4 + current_offset;
        game.g_ram[timer_offset..timer_offset + 2].copy_from_slice(&1u16.to_le_bytes());
    } else {
        UpdateMorphBallEyeBeamHdma(k, game);
        let current_slot = u16::from_le_bytes([game.g_ram[0x18b2], game.g_ram[0x18b3]]);
        // Colors use byte slots, including odd current object indices.
        for (color_offset, black) in [(0x1914, 0x20), (0x1915, 0x40), (0x1920, 0x80)] {
            let offset = color_offset + usize::from(current_slot);
            if game.g_ram[offset] != black {
                game.g_ram[offset] = game.g_ram[offset].wrapping_sub(1);
            }
        }
    }
}
pub unsafe fn SpawnTitleScreenGradientObjs(game: &mut Game) {
    static unk_88EB6B: SpawnHdmaObject_Args = SpawnHdmaObject_Args::new(0, 0x31, 0xeb85 as VoidP);
    static unk_88EB63: SpawnHdmaObject_Args = SpawnHdmaObject_Args::new(0, 0x32, 0xeb73 as VoidP);
    SpawnHdmaObject(0x88, &raw const unk_88EB63 as *const c_void, game);
    SpawnHdmaObject(0x88, &raw const unk_88EB6B as *const c_void, game);
}
pub unsafe fn HdmaobjInsr_ConfigTitleSequenceGradientHDMA(
    k: u16,
    hdp: *const u8,
    game: &mut Game,
) -> *const u8 {
    *ram_ptr!(game; 0x6f) = 0;
    ConfigureTitleSequenceGradientHDMA(game);
    hdp
}
pub unsafe fn HdmaobjPreInstr_Backdrop_TitleSequenceGradient(k: u16, game: &mut Game) {
    ConfigureTitleSequenceGradientHDMA(game);
    if word!(game.cinematic_function) as i32 == 0x8b9b68 & 0xffff {
        let v1: i32 = word!(game.hdma_object_index) as i32 >> 1;
        word!(
            game.hdma_object_instruction_list_pointers[v1],
            wrapping_add(2)
        );
        word!(game.hdma_object_instruction_timers[v1] = 1);
    }
}
pub unsafe fn HdmaobjPreInstr_ColorMathControlB_TitleGradient(k: u16, game: &mut Game) {
    if word!(game.cinematic_function) as i32 == 0x8b9b68 & 0xffff {
        let v1: i32 = word!(game.hdma_object_index) as i32 >> 1;
        word!(
            game.hdma_object_instruction_list_pointers[v1],
            wrapping_add(2)
        );
        word!(game.hdma_object_instruction_timers[v1] = 1);
    }
}
pub unsafe fn SpawnIntroCutsceneCrossfadeHdma(game: &mut Game) {
    static unk_88EBFB: SpawnHdmaObject_Args = SpawnHdmaObject_Args::new(0, 0x31, 0xec03 as VoidP);
    SpawnHdmaObject(0x88, &raw const unk_88EBFB as *const c_void, game);
}
pub unsafe fn HdmaobjPreInstr_IntroCutsceneCrossfade(k: u16, game: &mut Game) {
    if word!(game.eproj_x_pos[0]) == 1 {
        let v1: i32 = word!(game.hdma_object_index) as i32 >> 1;
        word!(
            game.hdma_object_instruction_list_pointers[v1],
            wrapping_add(2)
        );
        word!(game.hdma_object_instruction_timers[v1] = 1);
    }
}
pub unsafe fn CinematicFunction_Intro_Func133(game: &mut Game) {
    word!(game.g_word_7E0D9C = 1);
    *(ram_ptr!(game, u16; 0xd9e)) = 0x4000;
    word!(game.loop_counter_transfer_enemies_to_vram = 8);
    word! { game, 192; button_config_shoot_x_saved, button_config_itemcancel_y_saved }
    let mut v0: u32 = 12621824i32 as u32;
    *((ram_ptr!(game, u16; 0xd84) as *mut u8).offset(1) as *mut u16) =
        *(&raw mut v0 as *mut u16).offset(1);
    *((ram_ptr!(game, u16; 0xd82) as *mut u8).offset(1) as *mut u16) = v0 as u16;
    word!(game.button_config_run_b_saved = -26496i32 as u16);
    let mut v1: u32 = 12621824i32 as u32;
    *((ram_ptr!(game, u16; 0xd8a) as *mut u8).offset(1) as *mut u16) =
        *(&raw mut v1 as *mut u16).offset(1);
    *((ram_ptr!(game, u16; 0xd88) as *mut u8).offset(1) as *mut u16) = v1 as u16;
    word!(game.button_config_aim_down_L_saved = -26496i32 as u16);
    word!(game.button_config_aim_up_R_saved = 0);
    static unk_88EC82: SpawnHdmaObject_Args =
        SpawnHdmaObject_Args::new(0x42, 0x11, 0xec8a as VoidP);
    SpawnHdmaObject(0x88, &raw const unk_88EC82 as *const c_void, game);
}
pub unsafe fn HdmaobjInstr_EC9F_ClearVars(k: u16, hdp: *const u8, game: &mut Game) -> *const u8 {
    let v2: i32 = k as i32 >> 1;
    word! {game; hdma_object_A[v2] = -2i32 as u16; hdma_object_B[v2] = 1; }
    word! { game, 0; hdma_object_C[v2], hdma_object_D[v2] }
    hdp
}
pub unsafe fn HdmaobjPreInstr_ECB6(k: u16, game: &mut Game) {
    if word!(game.g_word_7E0D9C) != 0 {
        let r28: u16 = 4;
        let n: i32 = 128;
        let v3: i32 = k as i32 >> 1;
        word!(
            game.hdma_object_A[v3] = (2 * word!(game.loop_counter_transfer_enemies_to_vram) as i32
                + word!(game.hdma_object_A[v3]) as i32
                & 0x1ff) as u16
        );
        let mut r20: u16 = word!(game.hdma_object_A[v3]);
        let mut v4: u16 = 0;
        do_until! { (((v4 as i32 - n) as i16) >= 0);
            let v5: u16 = r20;
            let v6: i32 = r20 as i32 >> 1;
            let sample = kSinCosTable8bit_Sext[(v6 + 64) as usize];
            let offset = hdma_wave_offset(sample, *ram_ptr!(game; 0xd9e), *ram_ptr!(game; 0xd9f));
            r20 = (r28 as i32 + v5 as i32 & 0x1ff) as u16;
            word!(game.hdma_table_1[(v4 >> 1)] = word!(game.reg_BG3HOFS).wrapping_add(offset));
            v4 = v4.wrapping_add(2);
        }
        for i in (0i32..=126).rev().step_by(2) {
            word!(
                game.hdma_table_1[((i >> 1) + 64)] =
                    (word!(game.reg_BG3HOFS) as i32 + word!(game.reg_BG3HOFS) as i32
                        - word!(game.hdma_table_1[(i >> 1)]) as i32) as u16
            );
        }
    } else {
        let v2: i32 = word!(game.hdma_object_index) as i32 >> 1;
        word!(
            game.hdma_object_instruction_list_pointers[v2],
            wrapping_add(2)
        );
        word!(game.hdma_object_instruction_timers[v2] = 1);
    };
}
crate::game::address_dispatch! {
    pub unsafe fn CallHdmaobjPreInstr(ea: u32, k: u16, game: &mut Game) => ea;
        call (k, game); default { Unreachable(game); };
    {
        8491739 => [()]; 8946872 => [()]; 10785852 => [()]; 9556607 => HdmaobjPreInstr_XraySetup;
        8947439 => HdmaobjPreInstr_Xray; 8947506 => HdmaobjPreInstr_XrayFunc0_NoBeam;
        8947540 => HdmaobjPreInstr_XrayFunc1_BeamWidening;
        8947627 => HdmaobjPreInstr_XrayFunc2_FullBeam;
        8948020 => HdmaobjPreInstr_XrayFunc3_DeactivateBeam;
        8948154 => HdmaobjPreInstr_XrayFunc4_DeactivateBeam;
        8948232 => HdmaobjPreInstr_XrayFunc5_DeactivateBeam;
        8948623 => explosion_window::set_power_bomb_blending;
        8948632 => explosion_window::power_bomb_afterglow;
        8949225 => [explosion_window::update_scaled(k, Phase::PowerBomb, game)];
        8949426 => [explosion_window::update_prescaled(k, Phase::PowerBomb, game)];
        8949983 => [explosion_window::update_scaled(k, Phase::PreFlash, game)];
        8950184 => [explosion_window::update_prescaled(k, Phase::PreFlash, game)];
        8954681 => explosion_window::set_crystal_flash_blending;
        8954717 => explosion_window::crystal_flash_afterglow;
        8955218 => [explosion_window::update_scaled(k, Phase::CrystalFlash, game)];
        8955459 => HdmaobjPreInstr_FxType22_BG3Yscroll; 8955507 => HdmaobjPreInstr_BG3Xscroll;
        8957362 => HdmaobjPreInstr_SkyLandBG2Xscroll; 8957370 => HdmaobjPreInstr_SkyLandBG2Xscroll2;
        8957378 => HdmaobjPreInstr_SkyLandBG2XscrollInner;
        8958140 => HdmaobjPreInstr_FirefleaBG3XScroll;
        8958896 => HdmaobjPreInstr_LavaAcidBG3YScroll;
        8959189 => HdmaobjPreInstr_LavaAcidBG2YScroll; 8963214 => HdmaobjPreInstr_WaterBG3XScroll;
        8963465 => HdmaobjPreInstr_WaterBG2XScroll;
        8963556 => HdmaobjPreInstr_WaterBG2XScroll_Func2;
        8963638 => HdmaobjPreInstr_WaterBG2XScroll_Func1; 8968609 => HdmaobjPreInstr_RainBg3Scroll;
        8968775 => HdmaobjPreInstr_SporesBG3Xscroll; 8969014 => HdmaobjPreInstr_FogBG3Scroll;
        8969175 => HdmaobjPreInstr_CheckLotsOfEventsHappened; 8969251 => HdmaobjPreInstr_DC23;
        8969321 => HdmaobjPreInstr_DC69; 8969402 => HdmaobjPreInstr_DCBA;
        8969539 => HdmaobjPreInstr_BombTorizoHazeColorMathBgColor;
        8969744 => HdmaobjPreInstr_HazeColorMathSubscreen_CeresRidleyAlive;
        8969749 => HdmaobjPreInstr_HazeColorMathSubscreen_CeresRidleyDead;
        8969773 => HdmaobjPreInstr_HazeColorMathSubscreen_FadingIn;
        8969844 => HdmaobjPreInstr_HazeColorMathSubscreen_FadedIn;
        8969878 => HdmaobjPreInstr_HazeColorMathSubscreen_FadingOut;
        8970132 => HdmaobjPreInstr_DF94; 8970278 => HdmaobjPreInstr_VariaSuitPickup;
        8970332 => HdmaobjPreInstr_GravitySuitPickup; 8971337 => HdmaobjPreInstr_E449;
        8971623 => HdmaobjPreInstr_E567; 8972220 => HdmaobjPreInstr_E7BC;
        fnHdmaobjPreInstr_MorphBallEyeBeamWidening => [{
                    HdmaobjPreInstr_MorphBallEyeBeamWidening(k, game);
                }];
        fnHdmaobjPreInstr_MorphBallEyeBeamFull => [{
                    HdmaobjPreInstr_MorphBallEyeBeamFull(k, game);
                }];
        fnHdmaobjPreInstr_MorphBallEyeBeamFading => [{
                    HdmaobjPreInstr_MorphBallEyeBeamFading(k, game);
                }];
        8973232 => HdmaobjPreInstr_Backdrop_TitleSequenceGradient;
        8973266 => HdmaobjPreInstr_ColorMathControlB_TitleGradient;
        8973341 => HdmaobjPreInstr_IntroCutsceneCrossfade; 8972070 => [()];
        8973494 => HdmaobjPreInstr_ECB6;
    }
}
crate::game::address_dispatch! {
    pub unsafe fn CallHdmaobjInstr(ea: u32, k: u16, j: *const u8, game: &mut Game) -> *const u8 => ea;
        call (k, j, game); default {
            Unreachable(game);
            null::<u8>()
        };
    {
        8969006 => [j]; 8947049 => HdmaobjInstr_Delete; 8947056 => HdmaobjInstr_SetPreInstr;
        8947076 => HdmaobjInstr_ClearPreInstr; 8947124 => HdmaobjInstr_CallFarFunc;
        8947180 => [HdmaobjInstr_Goto(k, j)]; 8947185 => [HdmaobjInstr_GotoRel(k, j)];
        8947207 => HdmaobjInstr_DecrementAndGoto; 8947215 => HdmaobjInstr_DecrementAndGotoRel;
        8947222 => HdmaobjInstr_SetTimer; 8947234 => HdmaobjInstr_SetHdmaControl;
        8947255 => HdmaobjInstr_SetHdmaTarget; 8947276 => HdmaobjInstr_SetHdmaTablePtr;
        8947285 => HdmaobjInstr_SetHdmaTableBank; 8947306 => HdmaobjInstr_SetIndirectHdmaDataBank;
        8947330 => HdmaobjInstr_Sleep; 8955500 => HdmaobjInstr_SetFlagB;
        8959182 => HdmaobjInstr_SetFlagB_Copy; 8963175 => HdmaobjInstr_SetFlagB_Copy2;
        8963458 => HdmaobjInstr_SetFlagB_Copy3; 8968521 => HdmaobjInstr_SetVideoMode1;
        8968577 => HdmaobjInstr_1938_RandomNumber; 8969419 => HdmaobjInstr_GotoIfEventHappened;
        8971453 => HdmaobjInstr_E4BD;
        fnHdmaobjInstr_InitMorphBallEyeBeamHdma => [{
                    HdmaobjInstr_InitMorphBallEyeBeamHdma(k, j, game)
                }];
        8973471 => HdmaobjInstr_EC9F_ClearVars; 8958889 => HdmaobjInstr_B3A9;
        8973215 => HdmaobjInsr_ConfigTitleSequenceGradientHDMA;
        8968470 => [{
                    sub_88D916(game);
                    j
                }];
    }
}
