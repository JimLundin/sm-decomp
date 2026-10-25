use crate::game::liquid_effects;
use crate::game::music::{HandleMusicQueue, QueueMusic_Delayed8};
use crate::game::player_transitions::xray::BeamPhase;
pub use crate::game::prelude::*;
use crate::game::scrolling_sky;
use crate::game::session_runtime::Runtime;
use crate::game::visual_runtime::{self, spawn_hdma, spawn_hdma_last};
use crate::game::visual_state::{EyeBeamPhase, HazePhase, HdmaWave, Weather};

pub use crate::game::constants::bank_8f::*;
pub use crate::game::constants::bank_86::*;
pub use crate::game::constants::bank_87::*;

use crate::game::animated_tiles;
use crate::game::enemy_projectiles::SpawnEprojWithRoomGfx;
use crate::game::explosion_window::{self, Phase};
use crate::game::gameplay_flag_runtime::{
    CheckBossBitForCurArea, CheckEventHappened, SetEventHappened,
};
use crate::game::room_mechanisms::spawn_hardcoded;
use crate::game::sm_8b::ConfigureTitleSequenceGradientHDMA;
use crate::game::sm_80::QueueSfx2_Max6;
use crate::game::sm_a9::MotherBrain_CalculateRainbowBeamHdma;
use crate::game::xray_background::{self, finish_restore, restore_left, restore_right};

pub use crate::types::Consts_88;

crate::game::constants::typed_constants! { Consts_88;
    addr_locret_88858A = 34186,
}

pub const NULL: *mut c_void = null_mut::<c_void>();
crate::game::constants::typed_constants! { i32;
    true_0 = 1, false_0 = 0,
    fnDoorTransitionFunction_FadeOutScreen = 0x82e2db, fnDoorTransition_FadeInScreenAndFinish = 0x82e737,
    fnPauseHook_Empty = 0x8883e1,
    fnCinematicFunctionOpening = 0x8b9b68,
}
#[inline]
unsafe fn RomBankBase(bank: u8, game: &mut Game) -> *const u8 {
    unsafe {
        RomPtr((((bank as i32) << 16i32) + 0x8000i32) as u32, game).offset(-(0x8000i32 as isize))
    }
}
crate::game::constants::typed_constants! { u32;
    fnHdmaobjInstr_InitMorphBallEyeBeamHdma = 0x88e917,
    fnHdmaobjPreInstr_MorphBallEyeBeamWidening = 0x88e9e6, fnHdmaobjPreInstr_MorphBallEyeBeamFull = 0x88ea3c,
    fnHdmaobjPreInstr_MorphBallEyeBeamFading = 0x88eacb,
}
pub unsafe fn EnableHdmaObjects(game: &mut Game) {
    unsafe {
        word!(game.hdma_objects_enable_flag |= (0x8000u16));
    }
}
pub unsafe fn DisableHdmaObjects(game: &mut Game) {
    unsafe {
        word!(game.hdma_objects_enable_flag &= !(0x8000));
    }
}
pub unsafe fn WaitUntilEndOfVblankAndClearHdma(game: &mut Game) {
    unsafe {
        crate::rtl::register_writes! { game;
            byte(MDMAEN as u16, 0); byte(HDMAEN as u16, 0);
        }
        sub_8882AC(game);
    }
}
pub unsafe fn sub_8882AC(game: &mut Game) {
    unsafe {
        *ram_ptr!(game; 0x85) = 0;
        word_transfers!(game, descending, 6; le(hdma_object_channels_bitmask, 5) <- value(0));
    }
}
pub unsafe fn InitializeSpecialEffectsForNewRoom(game: &mut Game) {
    unsafe {
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
        (*(ram_ptr!(game, LongPtr; 0x601))).addr = (0x8883e1i32) as u16;
        (*(ram_ptr!(game, LongPtr; 0x604))).addr = (0x8883e1i32) as u16;
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
}
pub unsafe fn HdmaObjectHandler(game: &mut Game) {
    unsafe {
        HandleMusicQueue(game);
        if word!(game.time_is_frozen_flag) == 0
            && (word!(game.power_bomb_explosion_status) & 0x4000) != 0
        {
            word!(game.power_bomb_explosion_status = 0x8000);
            static unk_8884D5: SpawnHdmaObject_Args =
                SpawnHdmaObject_Args::new(0x40, 0x28, 0x8ace as VoidP);
            static unk_8884DD: SpawnHdmaObject_Args =
                SpawnHdmaObject_Args::new(0x40, 0x29, 0x8b80 as VoidP);
            spawn_hdma(0x88, &unk_8884D5, game);
            spawn_hdma(0x88, &unk_8884DD, game);
        }
        word!(game.fx_layer_blending_config_c = word!(game.fx_layer_blending_config_a));
        if (word!(game.hdma_objects_enable_flag) & 0x8000) != 0 {
            *ram_ptr!(game; 0x85) = 0;
            let mut i: i32 = 0;
            do_until! { i == 12 ;
                word!(game.hdma_object_index = i as u16);
                let mut v1: i8 = byte!(@pointer ram_ptr!(game, u16; 0x18b4); i as isize) as i8;
                if v1 != 0 {
                    crate::update_place! { reference fresh58 = *ram_ptr!(game; 0x85) => (*fresh58) | ((v1) as u8); }
                    HdmaobjInstructionHandler(i as u8, game);
                    i = word!(game.hdma_object_index) as i32;
                }
                i += 2;
            }
            crate::game::layer_blending::update(game);
        }
    }
}
pub unsafe fn HdmaobjInstructionHandler(k: u8, game: &mut Game) {
    unsafe {
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
            let base = RomBankBase(*ram_ptr!(game; 0x18c0, k as i32 as isize, 1), game);
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
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, hdp: *const u8, game: &mut Game) -> *const u8;
    HdmaobjInstr_Delete [null::<u8>()] {
        word!(game.hdma_object_channels_bitmask[(k >> 1)] = 0);
    };
    HdmaobjInstr_SetPreInstr [hdp.offset(3)] {
        word!(game.hdma_object_pre_instructions[(k >> 1)] = *(hdp as *mut u16));
        *ram_ptr!(game; 0x18fc, k as i32 as isize) = *hdp.offset(2);
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
        9554681 => [xray_background::freeze_and_backup(k, game)];
        9554716 => xray_background::capture_bg1_right;
        9554775 => xray_background::capture_bg1_left; 9554830 => xray_background::build_overlay;
        9556179 => xray_background::capture_bg2_right; 9556339 => xray_background::upload_left; 9556384 => xray_background::upload_right;
        9556668 => xray_background::set_backdrop; 9557650 => [Runtime::new(game).initialize_suit_pickup_hdma()];
    }
}
pub unsafe fn HdmaobjInstr_CallFarFunc(k: u16, hdp: *const u8, game: &mut Game) -> *const u8 {
    unsafe {
        CallHdmaobjInstrFunc(Load24(hdp as *mut LongPtr), k, game);
        hdp.offset(3)
    }
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, hdp: *const u8) -> *const u8;
    HdmaobjInstr_Goto [*(hdp as *mut u16) as usize as *const u8] { };
    HdmaobjInstr_GotoRel [hdp.offset(*hdp as i8 as i32 as isize)] { };
}
pub unsafe fn HdmaobjInstr_DecrementAndGoto(k: u16, hdp: *const u8, game: &mut Game) -> *const u8 {
    unsafe {
        let v2: i32 = k as i32 >> 1;
        crate::update_place! { unaligned_word fresh63 [fresh64] = &raw mut (*(ram_ptr!(game, u16; 0x1908)).offset(v2 as isize)) => (u16::from_le(fresh63.read_unaligned())).wrapping_sub(1); }
        if fresh64 == 1 {
            hdp.offset(2)
        } else {
            *(hdp as *mut u16) as usize as *const u8
        }
    }
}
pub unsafe fn HdmaobjInstr_DecrementAndGotoRel(
    k: u16,
    hdp: *const u8,
    game: &mut Game,
) -> *const u8 {
    unsafe {
        let v2: i32 = k as i32 >> 1;
        crate::update_place! { unaligned_word fresh61 [fresh62] = &raw mut (*(ram_ptr!(game, u16; 0x1908)).offset(v2 as isize)) => (u16::from_le(fresh61.read_unaligned())).wrapping_sub(1); }
        if fresh62 == 1 {
            hdp.offset(1)
        } else {
            hdp.offset(*hdp as i8 as i32 as isize)
        }
    }
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, hdp: *const u8, game: &mut Game) -> *const u8;
    HdmaobjInstr_SetTimer [hdp.offset(1)] {
        *ram_ptr!(game; 0x1908, k as i32 as isize) = *hdp;
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
    unsafe {
        WriteReg(
            (byte!(@pointer (ram_ptr!(game, u16; 0x18c0)).offset((k >> 1) as isize); 0) as i32
                + DAS00 as i32) as SnesRegs as u16,
            *hdp,
            game,
        );
        hdp.offset(1)
    }
}
pub unsafe fn HdmaobjInstr_Sleep(k: u16, hdp: *const u8, game: &mut Game) -> *const u8 {
    unsafe {
        let base = RomBankBase(*ram_ptr!(game; 0x18c0, k as i32 as isize, 1), game);
        word!(
            game.hdma_object_instruction_list_pointers[(k >> 1)] =
                (hdp.offset_from(base) as c_long - 2 as c_long) as u16
        );
        null::<u8>()
    }
}
static unk_88B08C: SpawnHdmaObject_Args = SpawnHdmaObject_Args::new(0x42, 0x11, 0xb0ac as VoidP);
static kFirefleaFlashingShades: [u16; 12] = [
    0, 0x100, 0x200, 0x300, 0x400, 0x500, 0x600, 0x500, 0x400, 0x300, 0x200, 0x100,
];
static kFirefleaDarknessShades: [u16; 6] = [0, 0x600, 0xc00, 0x1200, 0x1800, 0x1900];
pub unsafe fn FxTypeFunc_24(game: &mut Game) {
    unsafe {
        word! {game; fireflea_flashing_timer = 6; fireflea_flashing_index = 0; }
        spawn_hdma(0x88, &unk_88B08C, game);
        *(&raw mut (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00))).field_0
            as *mut u16) = 0;
        word! { game, 0; UNUSED_word_7E177C, fireflea_darkness_level }
        word!(game.UNUSED_word_7E1780 = 24);
        word!(game.UNUSED_word_7E1782 = kFirefleaFlashingShades[0i32 as usize]);
    }
}
pub unsafe fn HdmaobjPreInstr_FirefleaBG3XScroll(k: u16, game: &mut Game) {
    unsafe {
        let mut v1: u16 = 0;
        word!(
            game.fx_layer_blending_config_c = word!(game.fx_layer_blending_config_c) & 0xff00 | 0xc
        );
        if word!(game.time_is_frozen_flag) == 0 {
            if word!(game.fireflea_flashing_timer -= 1, reread) == 0 {
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
                .wrapping_add(
                    kFirefleaFlashingShades[word!(game.fireflea_flashing_index) as usize],
                ))
                >> 8) as i8;
            *ram_ptr!(game; 0x74, 1) = ((v2) as u8) | (0x80u8);
            *ram_ptr!(game; 0x74, 2) = ((v2) as u8) | (0x40u8);
            *ram_ptr!(game; 0x74, 0) = ((v2) as u8) | (0x20u8);
        }
    }
}
static g_word_88B256: [u16; 17] = [
    0x46, 1, 0x46, 3, 0x46, 2, 0x46, 1, 0x46, 1, 0x46, 2, 0x46, 2, 0x46, 1, 0x8000,
];
pub unsafe fn HandleEarthquakeSoundEffect(game: &mut Game) {
    unsafe {
        if (word!(game.earthquake_sfx_timer) & 0x8000) == 0 && {
            (word!(game.earthquake_sfx_timer -= 1, reread) & 0x8000) != 0
        } {
            let mut v0: u16 = word!(game.earthquake_sfx_index);
            let mut v1: i16 =
                g_word_88B256[(word!(game.earthquake_sfx_index) >> 1) as usize] as i16;
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
}
pub unsafe fn SpawnBG3ScrollHdmaObject(game: &mut Game) {
    unsafe {
        static unk_88D869: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0x43, 0x11, 0xd8d0 as VoidP);
        spawn_hdma_last(0x88, &unk_88D869, game);
    }
}
pub unsafe fn FxTypeFunc_28_CeresRidley(game: &mut Game) {
    unsafe {
        static unk_88D8E2: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0x40, 0x5, 0xd906 as VoidP);
        static unk_88D8EA: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0, 0x2c, 0xd91d as VoidP);
        spawn_hdma(0x88, &unk_88D8E2, game);
        spawn_hdma(0x88, &unk_88D8EA, game);
    }
}
pub unsafe fn sub_88D916(game: &mut Game) {
    unsafe {
        word!(game.hdma_data_table_in_ceres = 9);
    }
}
pub unsafe fn FxTypeFunc_CeresElevator(game: &mut Game) {
    unsafe {
        static unk_88D92C: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0x40, 0x5, 0xd939 as VoidP);
        spawn_hdma(0x88, &unk_88D92C, game);
    }
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, hdp: *const u8, game: &mut Game) -> *const u8;
    HdmaobjInstr_SetVideoMode1 [hdp] { word!(game.hdma_data_table_in_ceres = 9); };
}
pub unsafe fn FxTypeFunc_A_Rain(game: &mut Game) {
    unsafe {
        static unk_88D95C: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0x43, 0x11, 0xd96c as VoidP);
        game.g_ram[0x5b] = 92;
        spawn_hdma(0x88, &unk_88D95C, game);
        SpawnBG3ScrollHdmaObject(game);
        animated_tiles::spawn(addr_kAnimtiles_Rain as u16, game);
    }
}
pub unsafe fn HdmaobjInstr_1938_RandomNumber(k: u16, hdp: *const u8, game: &mut Game) -> *const u8 {
    unsafe {
        word!(
            game.hdma_object_D[(k >> 1)] = crate::rtl::rom_table_word!(
                game,
                0x88d992,
                ((word!(game.random_number) as i32 >> 1 & 6) as u16 >> 1) as isize,
            )
        );
        hdp
    }
}
pub unsafe fn FxTypeFunc_8_Spores(game: &mut Game) {
    unsafe {
        game.g_ram[0x5b] = 92;
        static unk_88DA1D: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0x42, 0x11, 0xda2d as VoidP);
        spawn_hdma(0x88, &unk_88DA1D, game);
        SpawnBG3ScrollHdmaObject(game);
        animated_tiles::spawn(addr_kAnimtiles_Spores as u16, game);
    }
}
pub unsafe fn FxTypeFunc_C(game: &mut Game) {
    unsafe {
        game.g_ram[0x5b] = 92;
        static unk_88DB14: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0x43, 0x11, 0xdb19 as VoidP);
        spawn_hdma(0x88, &unk_88DB14, game);
    }
}
pub unsafe fn FxTypeFunc_26_TourianEntranceStatue(game: &mut Game) {
    unsafe {
        static unk_88DBBA: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0x42, 0x11, 0xd856 as VoidP);
        static unk_88DBC2: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0x42, 0x10, 0xdcfa as VoidP);
        if CheckEventHappened(0xa, game) & 1 != 0 {
            spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x6, 0xc, 0xb777 as VoidP), game);
            *(ram_ptr!(game, u16; 0xcd20)) = 514;
        }
        *ram_ptr!(game; 0x59) = 74;
        word!(game.fx_rising_function_bank_88 = (0x88c428i32) as u16);
        word!(game.fx_y_pos = word!(game.fx_base_y_pos));
        spawn_hdma(0x88, &unk_88DBBA, game);
        spawn_hdma(0x88, &unk_88DBC2, game);
        SpawnBG3ScrollHdmaObject(game);
    }
}
pub unsafe fn sub_88DBCB(k: u16, game: &mut Game) {
    unsafe {
        *(&raw mut (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00))).field_0
            as *mut u16) =
            (word!(game.layer1_y_pos)).wrapping_add(word!(game.hdma_object_B[(k >> 1)]));
    }
}
pub unsafe fn HdmaobjPreInstr_CheckLotsOfEventsHappened(mut v0: u16, game: &mut Game) {
    unsafe {
        if CheckEventHappened(6, game) & 1 != 0
            && CheckEventHappened(7, game) & 1 != 0
            && CheckEventHappened(8, game) & 1 != 0
            && CheckEventHappened(9, game) & 1 != 0
        {
            word!(game.tourian_entrance_statue_animstate |= (0x10u16));
            if (word!(game.tourian_entrance_statue_animstate) & 0x8000) == 0 {
                word!(game.hdma_object_C[(v0 >> 1)] = 300);
                v0 = word!(game.hdma_object_index);
                let v1: i32 = word!(game.hdma_object_index) as i32 >> 1;
                word!(game.hdma_object_instruction_timers[v1] = 1);
                word!(game.hdma_object_instruction_list_pointers[v1] += 2);
            }
        }
        sub_88DBCB(v0, game);
    }
}
pub unsafe fn HdmaobjPreInstr_DC23(k: u16, game: &mut Game) {
    unsafe {
        let v0: u16 = k;
        HandleEarthquakeSoundEffect(game);
        word!(game.earthquake_type = 13);
        word!(game.earthquake_timer |= (0x20u16));
        let v1: i32 = v0 as u8 as i32 >> 1;
        if (word!(game.hdma_object_C[v1] -= 1, reread) & 0x8000) != 0 {
            SpawnEprojWithRoomGfx(addr_kEproj_TourianStatueDustClouds as u16, 0, game);
            SpawnEprojWithRoomGfx(addr_kEproj_TourianStatueDustClouds as u16, 0, game);
            SpawnEprojWithRoomGfx(addr_kEproj_TourianStatueDustClouds as u16, 0, game);
            SpawnEprojWithRoomGfx(addr_kEproj_TourianStatueDustClouds as u16, 0, game);
            word!(game.hdma_object_instruction_timers[v1] = 1);
            word!(game.hdma_object_instruction_list_pointers[v1] += 2);
        }
        sub_88DBCB(v0, game);
    }
}
pub unsafe fn HdmaobjPreInstr_DC69(k: u16, game: &mut Game) {
    unsafe {
        HandleEarthquakeSoundEffect(game);
        word!(game.earthquake_type = 13);
        word!(game.earthquake_timer |= (0x20u16));
        if word!(game.time_is_frozen_flag) == 0 {
            let v1: i32 = k as i32 >> 1;
            AddToHiLo(
                (ram_ptr!(game, u16; 0x1920)).offset(v1 as isize),
                (ram_ptr!(game, u16; 0x1914)).offset(v1 as isize),
                -0x4000i32 as u32,
            );
            if word!(game.hdma_object_B[v1]) == 0xff10 {
                spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x6, 0xc, 0xb773 as VoidP), game);
                SetEventHappened(0xa, game);
                word!(game.hdma_object_instruction_timers[v1] = 1);
                word!(game.hdma_object_instruction_list_pointers[v1] += 2);
            }
            sub_88DBCB(k, game);
        }
    }
}
pub unsafe fn HdmaobjPreInstr_DCBA(v0: u16, game: &mut Game) {
    unsafe {
        word!(game.tourian_entrance_statue_finished = 0x8000);
        *(ram_ptr!(game, u16; 0xcd20)) = 514;
        sub_88DBCB(v0, game);
    }
}
pub unsafe fn HdmaobjInstr_GotoIfEventHappened(
    k: u16,
    hdp: *const u8,
    game: &mut Game,
) -> *const u8 {
    unsafe {
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
}
pub unsafe fn SpawnBombTorizoHaze(game: &mut Game) {
    unsafe {
        static unk_88DD36: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0x2, 0x32, 0xdd4a as VoidP);
        static unk_88DD3E: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0, 0x32, 0xdd62 as VoidP);
        spawn_hdma(0x88, &unk_88DD36, game);
        spawn_hdma(0x88, &unk_88DD3E, game);
    }
}
pub unsafe fn HdmaobjPreInstr_BombTorizoHazeColorMathBgColor(k: u16, game: &mut Game) {
    unsafe {
        word!(game.fx_layer_blending_config_c = 44);
    }
}
pub unsafe fn FxTypeFunc_2C_Haze(game: &mut Game) {
    unsafe {
        static unk_88DDD4: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0x40, 0x32, 0xded3 as VoidP);
        static unk_88DDDD: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0x40, 0x32, 0xdeeb as VoidP);
        if CheckBossBitForCurArea(1, game) & 1 != 0 {
            spawn_hdma(0x88, &unk_88DDDD, game);
        } else {
            spawn_hdma(0x88, &unk_88DDD4, game);
        };
    }
}
static unk_88DF38: SpawnHdmaObject_Args = SpawnHdmaObject_Args::new(0, 0x2c, 0xdf4f as VoidP);
crate::forward_calls! { pub unsafe fn (game: &mut Game);
    sub_88DF34 => spawn_hdma(0x88, &unk_88DF38, game);
}
pub unsafe fn HdmaobjPreInstr_E449(k: u16, game: &mut Game) {
    unsafe {
        if (word!(game.phantom_related_layer_flag) & 0x4000) != 0 {
            word!(game.fx_layer_blending_config_c = 26);
        } else if byte!((*(ram_ptr!(game, EnemyData; 0xf78)).offset(3)).parameter_1; 0) != 0 {
            if byte!((*(ram_ptr!(game, EnemyData; 0xf78)).offset(3)).parameter_1; 0) == 0xff {
                word!(game.fx_layer_blending_config_c = 4);
                let v0: i32 = word!(game.hdma_object_index) as i32 >> 1;
                word!(game.hdma_object_instruction_list_pointers[v0] += 2);
                word!(game.hdma_object_instruction_timers[v0] = 1);
            }
        } else {
            word!(game.fx_layer_blending_config_c = 4);
        };
    }
}
static unk_88E4A0: SpawnHdmaObject_Args = SpawnHdmaObject_Args::new(0x42, 0xf, 0xe4a8 as VoidP);
pub unsafe fn sub_88E487(v0: u16, r22: u16, game: &mut Game) {
    unsafe {
        (*(ram_ptr!(game, EnemyData; 0xf78)).offset(2)).parameter_1 = v0;
        set_fields! {[(*(ram_ptr!(game, EnemyData; 0xf78)).offset(3))], 0; ai_var_D, ai_var_E}
        (*(ram_ptr!(game, EnemyData; 0xf78)).offset(3)).ai_preinstr = r22;
        spawn_hdma(0x88, &unk_88E4A0, game);
    }
}
static unk_88E722: SpawnHdmaObject_Args = SpawnHdmaObject_Args::new(0, 0x2c, 0xe727 as VoidP);
static unk_88E74C: SpawnHdmaObject_Args = SpawnHdmaObject_Args::new(0x41, 0x26, 0xe751 as VoidP);
crate::forward_calls! { pub unsafe fn (game: &mut Game) -> u16;
    MotherBrainRisingHdmaObject =>
        spawn_hdma(0x88, &unk_88E722, game);
    SpawnMotherBrainRainbowBeamHdma =>
        spawn_hdma(0x88, &unk_88E74C, game);
}
pub unsafe fn HdmaobjPreInstr_E7BC(k: u16, game: &mut Game) {
    unsafe {
        if word!(game.game_state) == 19 {
            *(ram_ptr!(game, u16; 0x18b4)).offset((word!(game.hdma_object_index) >> 1) as isize) =
                0;
        } else {
            word!(game.fx_layer_blending_config_c = 36);
            if word!(game.game_state) != 27 {
                MotherBrain_CalculateRainbowBeamHdma(game);
                sub_88E7ED(game);
            }
        };
    }
}
pub unsafe fn sub_88E7ED(game: &mut Game) {
    unsafe {
        let mut v0: u16 = crate::rtl::rom_table_word!(
            game,
            0x88e833,
            (word!(game.hdma_object_A[0]) >> 1) as isize
        );
        if (v0 & 0x8000) == 0 {
            word!(game.hdma_object_A[0] += 1);
            word!(game.hdma_object_A[0] += 1);
            word!(game.hdma_object_A[0] += 1);
            word!(game.hdma_object_A[0] += 1);
        } else {
            word!(game.hdma_object_A[0] = 0);
            v0 = crate::rtl::rom_table_word!(game, 0x88e833, 0);
        }
        *ram_ptr!(game; 0x74, 0) = v0 as u8 & 0x1f | 0x20;
        *ram_ptr!(game; 0x74, 1) = (v0 as i32 >> 5 & 0x1f | 0x40) as u8;
        *ram_ptr!(game; 0x74, 2) = ((v0 >> 2) as i32 >> 8 & 0x1f | 0x80) as u8;
    }
}
pub unsafe fn SpawnMorphBallEyeBeamHdma(game: &mut Game) {
    unsafe {
        static unk_88E8E4: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0x41, 0x26, 0xe8ec as VoidP);
        spawn_hdma(0x88, &unk_88E8E4, game);
    }
}
pub unsafe fn SpawnTitleScreenGradientObjs(game: &mut Game) {
    unsafe {
        static unk_88EB6B: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0, 0x31, 0xeb85 as VoidP);
        static unk_88EB63: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0, 0x32, 0xeb73 as VoidP);
        spawn_hdma(0x88, &unk_88EB63, game);
        spawn_hdma(0x88, &unk_88EB6B, game);
    }
}
pub unsafe fn SpawnIntroCutsceneCrossfadeHdma(game: &mut Game) {
    unsafe {
        static unk_88EBFB: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0, 0x31, 0xec03 as VoidP);
        spawn_hdma(0x88, &unk_88EBFB, game);
    }
}
pub unsafe fn CinematicFunction_Intro_Func133(game: &mut Game) {
    unsafe {
        word!(game.g_word_7E0D9C = 1);
        *(ram_ptr!(game, u16; 0xd9e)) = 0x4000;
        word!(game.loop_counter_transfer_enemies_to_vram = 8);
        Runtime::new(game).initialize_wave_pair(0xd82, 0x9800);
        static unk_88EC82: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0x42, 0x11, 0xec8a as VoidP);
        spawn_hdma(0x88, &unk_88EC82, game);
    }
}
crate::game::address_dispatch! {
    pub unsafe fn CallHdmaobjPreInstr(ea: u32, k: u16, game: &mut Game) => ea;
        call (k, game); default { Unreachable(game); };
    {
        8491739 => [()]; 8946872 => [()]; 10785852 => [()]; 9556607 => xray_background::setup_blending;
        8947439 => [Runtime::new(game).advance_xray_background()]; 8947506 => [Runtime::new(game).advance_xray_beam(BeamPhase::Initial)];
        8947540 => [Runtime::new(game).advance_xray_beam(BeamPhase::Widening)];
        8947627 => [Runtime::new(game).advance_xray_beam(BeamPhase::Full)];
        8948020 => restore_left;
        8948154 => restore_right;
        8948232 => finish_restore;
        8948623 => explosion_window::set_power_bomb_blending;
        8948632 => explosion_window::power_bomb_afterglow;
        8949225 => [explosion_window::update_scaled(k, Phase::PowerBomb, game)];
        8949426 => [explosion_window::update_prescaled(k, Phase::PowerBomb, game)];
        8949983 => [explosion_window::update_scaled(k, Phase::PreFlash, game)];
        8950184 => [explosion_window::update_prescaled(k, Phase::PreFlash, game)];
        8954681 => explosion_window::set_crystal_flash_blending;
        8954717 => explosion_window::crystal_flash_afterglow;
        8955218 => [explosion_window::update_scaled(k, Phase::CrystalFlash, game)];
        8955459 => scrolling_sky::update_vertical_scroll; 8955507 => scrolling_sky::update_horizontal_scroll;
        8957362 => scrolling_sky::update_parallax; 8957370 => scrolling_sky::update_parallax;
        8957378 => scrolling_sky::advance_parallax;
        8958140 => HdmaobjPreInstr_FirefleaBG3XScroll;
        8958896 => liquid_effects::lava_surface;
        8959189 => liquid_effects::lava_background; 8963214 => liquid_effects::water_surface;
        8963465 => liquid_effects::water_background;
        8963556 => liquid_effects::water_wave;
        8963638 => liquid_effects::water_flat;
        8968609 => [visual_runtime::scroll_weather(k, Weather::Rain, game)];
        8968775 => [visual_runtime::scroll_weather(k, Weather::Spores, game)];
        8969014 => [visual_runtime::scroll_weather(k, Weather::Fog, game)];
        8969175 => HdmaobjPreInstr_CheckLotsOfEventsHappened; 8969251 => HdmaobjPreInstr_DC23;
        8969321 => HdmaobjPreInstr_DC69; 8969402 => HdmaobjPreInstr_DCBA;
        8969539 => HdmaobjPreInstr_BombTorizoHazeColorMathBgColor;
        8969744 => [Runtime::new(game).begin_haze(k, 0x80)];
        8969749 => [Runtime::new(game).begin_haze(k, 0x20)];
        8969773 => [Runtime::new(game).advance_haze(k, HazePhase::FadingIn)];
        8969844 => [Runtime::new(game).advance_haze(k, HazePhase::FadedIn)];
        8969878 => [Runtime::new(game).advance_haze(k, HazePhase::FadingOut)];
        8970132 => [Runtime::new(game).select_draygon_layer(k)]; 8970278 => [Runtime::new(game).advance_suit_pickup(false)];
        8970332 => [Runtime::new(game).advance_suit_pickup(true)]; 8971337 => HdmaobjPreInstr_E449;
        8971623 => [Runtime::new(game).advance_hdma_wave(k, HdmaWave::Phantoon)]; 8972220 => HdmaobjPreInstr_E7BC;
        fnHdmaobjPreInstr_MorphBallEyeBeamWidening => [{
                    Runtime::new(game).advance_eye_beam(k, EyeBeamPhase::Widening);
                }];
        fnHdmaobjPreInstr_MorphBallEyeBeamFull => [{
                    Runtime::new(game).advance_eye_beam(k, EyeBeamPhase::Full);
                }];
        fnHdmaobjPreInstr_MorphBallEyeBeamFading => [{
                    Runtime::new(game).advance_eye_beam(k, EyeBeamPhase::Fading);
                }];
        8973232 => [visual_runtime::advance_title_gradient(true, game)];
        8973266 => [visual_runtime::advance_title_gradient(false, game)];
        8973341 => [visual_runtime::advance_intro_crossfade(game)]; 8972070 => [()];
        8973494 => [Runtime::new(game).advance_hdma_wave(k, HdmaWave::Intro)];
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
        8947330 => HdmaobjInstr_Sleep; 8955500 => scrolling_sky::start_wave;
        8959182 => liquid_effects::start_wave; 8963175 => liquid_effects::start_wave;
        8963458 => liquid_effects::start_wave; 8968521 => HdmaobjInstr_SetVideoMode1;
        8968577 => HdmaobjInstr_1938_RandomNumber; 8969419 => HdmaobjInstr_GotoIfEventHappened;
        8971453 => [{ Runtime::new(game).initialize_phantoon_wave(k); j }];
        fnHdmaobjInstr_InitMorphBallEyeBeamHdma => [{
                    Runtime::new(game).initialize_eye_beam(k);
                    j
                }];
        8973471 => [{ Runtime::new(game).reset_hdma_wave(k); j }]; 8958889 => liquid_effects::start_lava_sound;
        8973215 => [{
                    game.g_ram[0x6f] = 0;
                    ConfigureTitleSequenceGradientHDMA(game);
                    j
                }];
        8968470 => [{
                    sub_88D916(game);
                    j
                }];
    }
}
