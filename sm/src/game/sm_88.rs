use crate::game::actors::access::{AccessMode::LittleEndian as Le, ActorAccess, ActorKey as Key};
use crate::game::beam_windows::calculate_xray_window;
use crate::game::liquid_effects;
use crate::game::music::{HandleMusicQueue, QueueMusic_Delayed8};
use crate::game::player_transitions as pt;
pub use crate::game::prelude::*;
use crate::game::scrolling_sky;
use crate::game::session_runtime::Runtime;

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
use crate::game::sm_90::CallSomeSamusCode;
use crate::game::sm_91::{InitializeSuitPickupHdma, Samus_LoadSuitPalette};
use crate::game::sm_94::kSinCosTable8bit_Sext;
use crate::game::sm_a9::MotherBrain_CalculateRainbowBeamHdma;
use crate::game::xray_background::{self, finish_restore, restore_left, restore_right};

pub use crate::types::Consts_88;

crate::game::constants::typed_constants! { Consts_88;
    addr_word_88DF71 = 57201, addr_word_88DF6B = 57195,
    addr_word_88DF65 = 57189, addr_word_88DF5F = 57183,
    addr_loc_889180 = 37248,
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
unsafe fn SpawnHdmaObjectInner(
    k: u16,
    p: *mut u16,
    r18: u16,
    r20: u16,
    r24: u16,
    game: &mut Game,
) -> u16 {
    unsafe {
        let v2: i32 = k as i32 >> 1;
        word!(game.hdma_object_pre_instructions[v2] = (0x8884b8i32) as u16);
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
}
pub unsafe fn SpawnHdmaObject(db: u8, p: *const c_void, game: &mut Game) -> u16 {
    unsafe {
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
}
pub unsafe fn SpawnHdmaObjectToSlot0xA(db: u8, p: *const c_void, game: &mut Game) {
    unsafe {
        SpawnHdmaObjectInner(
            0xa,
            p as *mut u16,
            0x8000,
            0x70,
            ((db as i32) << 8) as u16,
            game,
        );
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
            SpawnHdmaObject(0x88, &raw const unk_8884D5 as *const c_void, game);
            SpawnHdmaObject(0x88, &raw const unk_8884DD as *const c_void, game);
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
        9556668 => xray_background::set_backdrop; 9557650 => InitializeSuitPickupHdma;
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
pub unsafe fn HdmaobjPreInstr_XrayFunc0_NoBeam(k: u16, game: &mut Game) {
    unsafe {
        if (word!(game.button_config_run_b) & word!(game.joypad1_lastkeys)) != 0 {
            CalculateXrayHdmaTable(game);
            word!(game.demo_input_pre_instr += 1);
        } else {
            word!(game.demo_input_pre_instr = 3);
        };
    }
}
pub unsafe fn HdmaobjPreInstr_XrayFunc1_BeamWidening(k: u16, game: &mut Game) {
    unsafe {
        if (word!(game.button_config_run_b) & word!(game.joypad1_lastkeys)) != 0 {
            words!(game.demo_input_instr_timer, demo_input_instr_ptr += 2048);
            words!(
                game.demo_input,
                demo_input_new += words!(game.demo_input_instr_timer, demo_input_instr_ptr)
            );
            if word!(game.demo_input) as i32 - 11i32 & 0x8000 == 0 {
                word! {game; demo_input_new = 0; demo_input = 10; }
                word!(game.demo_input_pre_instr += 1);
            }
            CalculateXrayHdmaTable(game);
        } else {
            word!(game.demo_input_pre_instr = 3);
        };
    }
}
pub unsafe fn HdmaobjPreInstr_XrayFunc2_FullBeam(k: u16, game: &mut Game) {
    unsafe {
        if (word!(game.button_config_run_b) & word!(game.joypad1_lastkeys)) != 0 {
            HandleMovingXrayUpDown(game);
            CalculateXrayHdmaTable(game);
        } else {
            word!(game.demo_input_pre_instr += 1);
        };
    }
}
pub unsafe fn HandleMovingXrayUpDown(game: &mut Game) {
    unsafe {
        if (word!(game.button_config_up) & word!(game.joypad1_lastkeys)) != 0 {
            MoveXrayUp(game);
        } else if (word!(game.button_config_down) & word!(game.joypad1_lastkeys)) != 0 {
            MoveXrayDown(game);
        }
    }
}
pub unsafe fn MoveXrayUp(game: &mut Game) {
    unsafe {
        if word!(game.xray_angle) as i32 - 128i32 & 0x8000 != 0 {
            if word!(game.xray_angle) != word!(game.demo_input)
                && (((word!(game.xray_angle)) as i16).wrapping_sub((word!(game.demo_input)) as i16)
                    < 0
                    || {
                        let mut v0: bool = word!(game.xray_angle) != 0;
                        word!(game.xray_angle -= 1);
                        ((word!(game.xray_angle) as i32
                            - (!v0 as i32 + word!(game.demo_input) as i32))
                            as i16)
                            < 0
                    })
            {
                word!(game.xray_angle = word!(game.demo_input));
            }
        } else if (word!(game.demo_input) as i32 + word!(game.xray_angle) as i32) != 256
            && (((word!(game.demo_input) as i32 + word!(game.xray_angle) as i32 - 256i32) as i16)
                >= 0
                || {
                    let mut v1: i16 = ((words!(game.demo_input, xray_angle))
                        .wrapping_add(words!((word!(game.xray_angle), 1u16)))
                        >> 16) as i16;
                    word!(game.xray_angle += 1);
                    v1 != 256 && (v1).wrapping_sub(256) >= 0
                })
        {
            word!(game.xray_angle = (256u16).wrapping_sub(word!(game.demo_input)));
        }
    }
}
pub unsafe fn MoveXrayDown(game: &mut Game) {
    unsafe {
        if word!(game.xray_angle) as i32 - 128i32 & 0x8000 != 0 {
            if (word!(game.demo_input) as i32 + word!(game.xray_angle) as i32) != 128
                && (((word!(game.demo_input) as i32 + word!(game.xray_angle) as i32 - 128i32)
                    as i16)
                    >= 0
                    || {
                        let mut v0: i16 = ((words!(game.demo_input, xray_angle))
                            .wrapping_add(words!((word!(game.xray_angle), 1u16)))
                            >> 16) as i16;
                        word!(game.xray_angle += 1);
                        v0 != 128 && (v0).wrapping_sub(128) >= 0
                    })
            {
                word!(game.xray_angle = (128u16).wrapping_sub(word!(game.demo_input)));
            }
        } else if (word!(game.xray_angle) as i32 - word!(game.demo_input) as i32) != 128
            && (((word!(game.xray_angle) as i32 - word!(game.demo_input) as i32 - 128i32) as i16)
                < 0
                || {
                    let mut v1: bool = word!(game.xray_angle) != 0;
                    word!(game.xray_angle -= 1);
                    word!(game.xray_angle) as i32 - (!v1 as i32 + word!(game.demo_input) as i32)
                        != 128
                        && ((word!(game.xray_angle) as i32
                            - (!v1 as i32 + word!(game.demo_input) as i32)
                            - 128i32) as i16)
                            < 0
                })
        {
            word!(game.xray_angle = (word!(game.demo_input)).wrapping_add(128));
        }
    }
}
pub unsafe fn CalculateXrayHdmaTable(game: &mut Game) {
    unsafe {
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
}
static kHdmaobjPreInstr_XrayFuncs: [Func_Y_V; 6] = callback_table! { nonnull Func_Y_V;
    HdmaobjPreInstr_XrayFunc0_NoBeam, HdmaobjPreInstr_XrayFunc1_BeamWidening,
    HdmaobjPreInstr_XrayFunc2_FullBeam, restore_left,
    restore_right, finish_restore,
};
pub unsafe fn HdmaobjPreInstr_Xray(k: u16, game: &mut Game) {
    unsafe {
        let mut v1: i16 = 4096;
        if word!(game.fx_type) != 36 {
            v1 = 0x2000;
            if xray_background::can_reveal_blocks(game) {
                v1 = 0x4000;
                *(ram_ptr!(game, u16; 0x74, 0)) = 0x27;
                *(ram_ptr!(game, u16; 0x74, 1)) = 0x47;
                *(ram_ptr!(game, u16; 0x74, 2)) = 0x87;
            }
        }
        word!(game.fx_layer_blending_config_c |= ((v1) as u16));
        kHdmaobjPreInstr_XrayFuncs[word!(game.demo_input_pre_instr) as usize](
            (2u16).wrapping_mul(word!(game.demo_input_pre_instr)),
            game,
        );
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
        SpawnHdmaObject(0x88, &raw const unk_88B08C as *const c_void, game);
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
        SpawnHdmaObjectToSlot0xA(0x88, &raw const unk_88D869 as *const c_void, game);
    }
}
pub unsafe fn FxTypeFunc_28_CeresRidley(game: &mut Game) {
    unsafe {
        static unk_88D8E2: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0x40, 0x5, 0xd906 as VoidP);
        static unk_88D8EA: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0, 0x2c, 0xd91d as VoidP);
        SpawnHdmaObject(0x88, &raw const unk_88D8E2 as *const c_void, game);
        SpawnHdmaObject(0x88, &raw const unk_88D8EA as *const c_void, game);
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
        SpawnHdmaObject(0x88, &raw const unk_88D92C as *const c_void, game);
    }
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, hdp: *const u8, game: &mut Game) -> *const u8;
    HdmaobjInstr_SetVideoMode1 [hdp] { word!(game.hdma_data_table_in_ceres = 9); };
}
pub unsafe fn FxTypeFunc_A_Rain(game: &mut Game) {
    unsafe {
        static unk_88D95C: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0x43, 0x11, 0xd96c as VoidP);
        *ram_ptr!(game; 0x5b) = 92;
        SpawnHdmaObject(0x88, &raw const unk_88D95C as *const c_void, game);
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
pub unsafe fn HdmaobjPreInstr_RainBg3Scroll(k: u16, game: &mut Game) {
    unsafe {
        *ram_ptr!(game; 0x5b) = 92;
        word!(game.fx_layer_blending_config_c = word!(game.fx_layer_blending_config_b));
        if word!(game.time_is_frozen_flag) == 0 {
            let v1: i32 = k as u8 as i32 >> 1;
            word!(
                game.bg3_ypos = (word!(game.hdma_object_E[v1]) as i32
                    - word!(game.layer1_y_pos) as i32
                    + *((ram_ptr!(game, u16; 0x1914)).offset(v1 as isize) as *mut u8).offset(1,)
                        as i8 as i32) as u16
            );
            word!(game.hdma_object_A[v1] -= 1536);
            word!(game.hdma_object_E[v1] = word!(game.layer1_y_pos));
            word!(
                game.bg3_xpos = (word!(game.hdma_object_F[v1]) as i32
                    - word!(game.layer1_x_pos) as i32
                    + *((ram_ptr!(game, u16; 0x1920)).offset(v1 as isize) as *mut u8).offset(1,)
                        as i8 as i32) as u16
            );
            word!(game.hdma_object_B[v1] += word!(game.hdma_object_D[v1]));
            word!(game.hdma_object_F[v1] = word!(game.layer1_x_pos));
        }
    }
}
pub unsafe fn FxTypeFunc_8_Spores(game: &mut Game) {
    unsafe {
        *ram_ptr!(game; 0x5b) = 92;
        static unk_88DA1D: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0x42, 0x11, 0xda2d as VoidP);
        SpawnHdmaObject(0x88, &raw const unk_88DA1D as *const c_void, game);
        SpawnBG3ScrollHdmaObject(game);
        animated_tiles::spawn(addr_kAnimtiles_Spores as u16, game);
    }
}
pub unsafe fn HdmaobjPreInstr_SporesBG3Xscroll(k: u16, game: &mut Game) {
    unsafe {
        *ram_ptr!(game; 0x5b) = 92;
        word!(game.fx_layer_blending_config_c = word!(game.fx_layer_blending_config_b));
        if word!(game.time_is_frozen_flag) == 0 {
            let v1: i32 = k as u8 as i32 >> 1;
            word!(
                game.bg3_ypos = (word!(game.layer1_y_pos)).wrapping_add(
                    (*((ram_ptr!(game, u16; 0x192c)).offset(v1 as isize) as *mut u8).offset(1,)
                        as i8) as u16,
                )
            );
            let v3: u16 = (word!(game.hdma_object_C[v1])).wrapping_sub(64);
            word!(game.hdma_object_C[v1] = v3);
            word!(
                game.bg3_xpos = (word!(game.layer1_x_pos)).wrapping_add(
                    (*((ram_ptr!(game, u16; 0x1938)).offset(v1 as isize) as *mut u8).offset(1,)
                        as i8) as u16,
                )
            );
            word!(game.hdma_object_D[v1] = word!(game.hdma_object_D[v1]));
        }
    }
}
pub unsafe fn FxTypeFunc_C(game: &mut Game) {
    unsafe {
        *ram_ptr!(game; 0x5b) = 92;
        static unk_88DB14: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0x43, 0x11, 0xdb19 as VoidP);
        SpawnHdmaObject(0x88, &raw const unk_88DB14 as *const c_void, game);
    }
}
pub unsafe fn HdmaobjPreInstr_FogBG3Scroll(k: u16, game: &mut Game) {
    unsafe {
        *ram_ptr!(game; 0x5b) = 92;
        word!(game.fx_layer_blending_config_c = word!(game.fx_layer_blending_config_b));
        if word!(game.time_is_frozen_flag) == 0 {
            let v1: i32 = k as u8 as i32 >> 1;
            word!(
                game.bg3_ypos = (word!(game.layer1_y_pos)).wrapping_add(
                    (*((ram_ptr!(game, u16; 0x1914)).offset(v1 as isize) as *mut u8).offset(1,)
                        as i8) as u16,
                )
            );
            word!(game.hdma_object_A[v1] -= 64);
            word!(
                game.bg3_xpos = (word!(game.layer1_x_pos)).wrapping_add(
                    (*((ram_ptr!(game, u16; 0x1920)).offset(v1 as isize) as *mut u8).offset(1,)
                        as i8) as u16,
                )
            );
            word!(game.hdma_object_B[v1] += 80);
        }
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
        SpawnHdmaObject(0x88, &raw const unk_88DBBA as *const c_void, game);
        SpawnHdmaObject(0x88, &raw const unk_88DBC2 as *const c_void, game);
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
        SpawnHdmaObject(0x88, &raw const unk_88DD36 as *const c_void, game);
        SpawnHdmaObject(0x88, &raw const unk_88DD3E as *const c_void, game);
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
            SpawnHdmaObject(0x88, &raw const unk_88DDDD as *const c_void, game);
        } else {
            SpawnHdmaObject(0x88, &raw const unk_88DDD4 as *const c_void, game);
        };
    }
}
crate::forward_calls! { pub unsafe fn (k: u16, game: &mut Game);
    HdmaobjPreInstr_HazeColorMathSubscreen_CeresRidleyAlive => sub_88DE18(k, 0x80, game);
    HdmaobjPreInstr_HazeColorMathSubscreen_CeresRidleyDead => sub_88DE18(k, 0x20, game);
}
pub unsafe fn sub_88DE18(k: u16, a: u16, game: &mut Game) {
    unsafe {
        let v2: i32 = k as u8 as i32 >> 1;
        word! {game; hdma_object_B[v2] = a; hdma_object_A[v2] = 0; }
        if word!(game.door_transition_function) as i32 == 0x82e737 & 0xffff {
            word!(game.hdma_object_pre_instructions[v2] = (0x88de2di32) as u16);
            HdmaobjPreInstr_HazeColorMathSubscreen_FadingIn(k, game);
        }
    }
}
pub unsafe fn HdmaobjPreInstr_HazeColorMathSubscreen_FadingIn(k: u16, game: &mut Game) {
    unsafe {
        *ram_ptr!(game; 0x74, 0) = 32;
        *ram_ptr!(game; 0x74, 1) = 64;
        *ram_ptr!(game; 0x74, 2) = 0x80;
        word!(game.fx_layer_blending_config_c = 44);
        if word!(game.hdma_object_A[(k as u8 >> 1)]) == 16 {
            word!(game.hdma_object_pre_instructions[(k as u8 >> 1)] = (0x88de74i32) as u16);
        } else {
            let v3: u8 = k as u8;
            let r20: u8 = byte!(@pointer ram_ptr!(game, u16; 0x1920); k as u8 as i32 as isize);
            let mut v1: i8 =
                byte!(@pointer ram_ptr!(game, u16; 0x1914); k as u8 as i32 as isize) as i8;
            for i in (0i32..=15).rev() {
                let r18: u8 = v1 as u8;
                byte!(@pointer ram_ptr!(game, u16; 0x9d00); i as u8 as i32 as isize) =
                    (r20) | ((v1) as u8);
                v1 = ((r18) as i8).wrapping_sub(1);
                if ((r18) as i8).wrapping_sub(1) < 0 {
                    v1 = 0;
                }
            }
            word!(game.hdma_object_A[(v3 >> 1)] += 1);
        };
    }
}
pub unsafe fn HdmaobjPreInstr_HazeColorMathSubscreen_FadedIn(k: u16, game: &mut Game) {
    unsafe {
        *ram_ptr!(game; 0x74, 0) = 32;
        *ram_ptr!(game; 0x74, 1) = 64;
        *ram_ptr!(game; 0x74, 2) = 0x80;
        word!(game.fx_layer_blending_config_c = 44);
        if word!(game.door_transition_function) as i32 == 0x82e2db & 0xffff {
            word!(game.hdma_object_pre_instructions[(k as u8 >> 1)] = (0x88de96i32) as u16);
        }
    }
}
pub unsafe fn HdmaobjPreInstr_HazeColorMathSubscreen_FadingOut(k: u16, game: &mut Game) {
    unsafe {
        *ram_ptr!(game; 0x74, 0) = 32;
        *ram_ptr!(game; 0x74, 1) = 64;
        *ram_ptr!(game; 0x74, 2) = 0x80;
        word!(game.fx_layer_blending_config_c = 44);
        if word!(game.hdma_object_A[(k as u8 >> 1)]) != 0 {
            let v3: u8 = k as u8;
            let r20: u8 = byte!(@pointer ram_ptr!(game, u16; 0x1920); k as u8 as i32 as isize);
            let mut v1: i8 =
                byte!(@pointer ram_ptr!(game, u16; 0x1914); k as u8 as i32 as isize) as i8;
            for i in (0i32..=15).rev() {
                let r18: u8 = v1 as u8;
                byte!(@pointer ram_ptr!(game, u16; 0x9d00); i as u8 as i32 as isize) =
                    (r20) | ((v1) as u8);
                v1 = ((r18) as i8).wrapping_sub(1);
                if ((r18) as i8).wrapping_sub(1) < 0 {
                    v1 = 0;
                }
            }
            word!(game.hdma_object_A[(v3 >> 1)] -= 1);
        }
    }
}
static unk_88DF38: SpawnHdmaObject_Args = SpawnHdmaObject_Args::new(0, 0x2c, 0xdf4f as VoidP);
crate::forward_calls! { pub unsafe fn (game: &mut Game);
    sub_88DF34 => SpawnHdmaObject(0x88, &raw const unk_88DF38 as *const c_void, game);
}
pub unsafe fn HdmaobjPreInstr_DF94(v0: u16, game: &mut Game) {
    unsafe {
        let mut v1: i16 = 0;
        if ((*(ram_ptr!(game, EnemyData; 0xf78))).properties & 0x200) == 0
            && (((*(ram_ptr!(game, EnemyData; 0xf78))).x_pos as i32
                - word!(game.layer1_x_pos) as i32
                + 64) as i16)
                >= 0
            && (*(ram_ptr!(game, EnemyData; 0xf78))).x_pos as i32
                - word!(game.layer1_x_pos) as i32
                - 320i32
                & 0x8000
                != 0
            && (((*(ram_ptr!(game, EnemyData; 0xf78))).y_pos as i32
                - word!(game.layer1_y_pos) as i32
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
}
static off_88E04E: [Func_U8; 7] = callback_table! { nonnull Func_U8;
    VariaSuitPickup_0_LightBeamAppears, VariaSuitPickup_1, VariaSuitPickup_2_LightBeamWidens,
    VariaSuitPickup_3, VariaSuitPickup_4_LightBeamShrinks,
    VariaSuitPickup_5_LightBeamDissipates, VariaSuitPickup_6,
};
pub unsafe fn HdmaobjPreInstr_VariaSuitPickup(k: u16, game: &mut Game) {
    unsafe {
        if off_88E04E[word!(game.substate) as usize](game) & 1 != 0 {
            *ram_ptr!(game; 0x74, 0) = *ram_ptr!(game; 0xdf0);
            *ram_ptr!(game; 0x74, 1) = *ram_ptr!(game; 0xdf1);
            *ram_ptr!(game; 0x74, 2) = *ram_ptr!(game; 0xdf2);
            word!(game.fx_layer_blending_config_c = 18);
        }
    }
}
static off_88E084: [Func_U8; 7] = callback_table! { nonnull Func_U8;
    VariaSuitPickup_0_LightBeamAppears, VariaSuitPickup_1, VariaSuitPickup_2_LightBeamWidens,
    GravitySuitPickup_3, VariaSuitPickup_4_LightBeamShrinks,
    VariaSuitPickup_5_LightBeamDissipates, GravitySuitPickup_6,
};
pub unsafe fn HdmaobjPreInstr_GravitySuitPickup(k: u16, game: &mut Game) {
    unsafe {
        if off_88E084[word!(game.substate) as usize](game) & 1 != 0 {
            *ram_ptr!(game; 0x74, 0) = *ram_ptr!(game; 0xdf0);
            *ram_ptr!(game; 0x74, 1) = *ram_ptr!(game; 0xdf1);
            *ram_ptr!(game; 0x74, 2) = *ram_ptr!(game; 0xdf2);
            word!(game.fx_layer_blending_config_c = 18);
        }
    }
}
static g_word_88E0D5: u16 = 0x7878;
pub unsafe fn VariaSuitPickup_0_LightBeamAppears(game: &mut Game) -> u8 {
    unsafe {
        word!(game.suit_pickup_light_beam_pos += 8);
        let mut v0: i16 = word!(game.suit_pickup_light_beam_pos) as i16;
        let mut v1: u16 = 0;
        do_until! { v0 <= 0 ;
            word!(game.hdma_table_1[(v1 >> 1)] = g_word_88E0D5);
            v1 = (v1).wrapping_add(2);
            v0 -= 1;
        }
        let mut v2: u16 = 510;
        let mut v3: i16 = word!(game.suit_pickup_light_beam_pos) as i16;
        do_until! { v3 <= 0 ;
            word!(game.hdma_table_1[(v2 >> 1)] = g_word_88E0D5);
            v2 = (v2).wrapping_sub(2);
            v3 -= 1;
        }
        if word!(game.suit_pickup_light_beam_pos) as i32 - 128i32 & 0x8000 == 0 {
            word!(game.substate += 1);
            word!(game.suit_pickup_light_beam_pos = 30840);
        }
        1
    }
}
pub unsafe fn VariaSuitPickup_1(game: &mut Game) -> u8 {
    unsafe {
        byte!(@pointer ram_ptr!(game, u16; 0xdee); 0) = ((word!(game.suit_pickup_light_beam_pos))
            as u8)
            .wrapping_sub(byte!(@pointer ram_ptr!(game, u16; 0xddc); 1));
        crate::update_place! { reference fresh17 = *ram_ptr!(game; 0xdee, 1) => (*fresh17).wrapping_add(*ram_ptr!(game; 0xddc, 1)); }
        for i in (0i32..=510).rev().step_by(2) {
            word!(game.hdma_table_1[(i >> 1)] = word!(game.suit_pickup_light_beam_pos));
        }
        if word!(game.suit_pickup_light_beam_pos) as u8 as i32 - 97i32 & 0x8000 != 0 {
            word!(game.substate += 1);
            word!(game.suit_pickup_light_beam_pos = 0x846c);
        }
        1
    }
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
    unsafe {
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
        do_until! { v2 >= 0x100 ;
            widen_suit_pickup_beam_row!(game, v3, v2);
            v3 = v3.wrapping_add(1);
        }
        let mut v7: u16 = (v3).wrapping_sub(1);
        do_until! { v2 >= 0x200 ;
            widen_suit_pickup_beam_row!(game, v7, v2);
            v7 = v7.wrapping_sub(1);
        }
        word!(game.suit_pickup_light_beam_widening_speed += 96);
        if word!(game.suit_pickup_light_beam_pos) == 0xff00 {
            word!(game.substate += 1);
            word!(game.suit_pickup_light_beam_widening_speed >>= 1);
            word!(game.suit_pickup_light_beam_pos = 0);
        }
        1
    }
}
pub unsafe fn VariaSuitPickup_4_LightBeamShrinks(game: &mut Game) -> u8 {
    unsafe {
        if *ram_ptr!(game; 0xdf3) != 0 {
            AdvanceSuitPickupColorMathToBlue(game);
        } else {
            AdvanceSuitPickupColorMathToOrange(game);
        }
        word!(
            checked_i32,
            game.suit_pickup_light_beam_pos +=
                ((word!(game.suit_pickup_light_beam_widening_speed) as i32 & 0xff00) >> 8)
        );
        let mut v0: i16 = word!(game.suit_pickup_light_beam_pos) as i16;
        let mut v1: u16 = 0;
        do_until! { v0 <= 0 ;
            word!(game.hdma_table_1[(v1 >> 1)] = 255);
            v1 = (v1).wrapping_add(2);
            v0 -= 1;
        }
        let mut v2: u16 = 510;
        let mut v3: i16 = ((word!(game.suit_pickup_light_beam_pos)) as i16).wrapping_sub(1);
        do_until! { v3 <= 0 ;
            word!(game.hdma_table_1[(v2 >> 1)] = 255);
            v2 = (v2).wrapping_sub(2);
            v3 -= 1;
        }
        word!(game.suit_pickup_light_beam_widening_speed -= 32);
        if word!(game.suit_pickup_light_beam_widening_speed) as i32 - 256i32 & 0x8000 != 0 {
            word!(game.suit_pickup_light_beam_widening_speed = 256);
        }
        if word!(game.suit_pickup_light_beam_pos) as i32 - 128i32 & 0x8000 == 0 {
            word!(game.substate += 1);
            word!(game.suit_pickup_light_beam_pos = -1793i32 as u16);
        }
        1
    }
}
pub unsafe fn VariaSuitPickup_5_LightBeamDissipates(game: &mut Game) -> u8 {
    unsafe {
        byte!(@pointer ram_ptr!(game, u16; 0xdee); 0) =
            ((word!(game.suit_pickup_light_beam_pos)) as u8).wrapping_add(8);
        crate::update_place! { reference fresh28 = *ram_ptr!(game; 0xdee, 1) => (*fresh28).wrapping_sub(8); }
        word!(game.hdma_table_1[128] = word!(game.suit_pickup_light_beam_pos));
        if word!(game.suit_pickup_light_beam_pos) as u8 as i32 - 112i32 & 0x8000 == 0 {
            word!(game.substate += 1);
        }
        1
    }
}
pub unsafe fn VariaSuitPickup_6(game: &mut Game) -> u8 {
    unsafe {
        QueueMusic_Delayed8(3, game);
        GravitySuitPickup_6(game)
    }
}
pub unsafe fn GravitySuitPickup_6(game: &mut Game) -> u8 {
    unsafe {
        *ram_ptr!(game; 0x74, 2) = 0x80;
        *ram_ptr!(game; 0x74, 1) = 64;
        *ram_ptr!(game; 0x74, 0) = 32;
        mov24(ram_ptr!(game, LongPtr; 0xa88), 0x980001);
        *(ram_ptr!(game, u16; 0xa8a, 1)) = 0;
        word! { game, 0;
            demo_input_prev, demo_input_prev_new, demo_backup_prev_controller_input,
        }
        word!(game.hdma_table_1[0] = 255);
        word! { game, 0; substate, suit_pickup_light_beam_pos }
        *(ram_ptr!(game, u16; 0xdf0)) = 0;
        *(ram_ptr!(game, u16; 0xdf2)) = 0;
        let v0: i32 = word!(game.hdma_object_index) as i32 >> 1;
        word!(game.hdma_object_instruction_list_pointers[v0] += 2);
        word!(game.hdma_object_instruction_timers[v0] = 1);
        CallSomeSamusCode(0xb, game);
        0
    }
}
pub unsafe fn AdvanceSuitPickupColorMathSubscreenToWhite(game: &mut Game) -> u8 {
    unsafe {
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
}
pub unsafe fn AdvanceSuitPickupColorMathToOrange(game: &mut Game) -> u8 {
    unsafe {
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
}
pub unsafe fn VariaSuitPickup_3(game: &mut Game) -> u8 {
    unsafe {
        word!(game.equipped_items |= (1u16));
        word!(game.collected_items |= (1u16));
        word!(game.samus_pose = kPose_9B_FaceF_VariaGravitySuit as u16);
        pt::finish_pose_change(&mut Runtime::new(game));
        Samus_LoadSuitPalette(game);
        word!(game.substate += 1);
        1
    }
}
pub unsafe fn GravitySuitPickup_3(game: &mut Game) -> u8 {
    unsafe {
        word!(game.equipped_items |= (0x20u16));
        word!(game.collected_items |= (0x20u16));
        word!(game.samus_pose = kPose_9B_FaceF_VariaGravitySuit as u16);
        pt::finish_pose_change(&mut Runtime::new(game));
        Samus_LoadSuitPalette(game);
        word!(game.substate += 1);
        1
    }
}
pub unsafe fn AdvanceSuitPickupColorMathToBlue(game: &mut Game) -> u8 {
    unsafe {
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
        SpawnHdmaObject(0x88, &raw const unk_88E4A0 as *const c_void, game);
    }
}
pub unsafe fn HdmaobjInstr_E4BD(k: u16, hdp: *const u8, game: &mut Game) -> *const u8 {
    unsafe {
        (*(ram_ptr!(game, EnemyData; 0xf78)).offset(1)).parameter_1 =
            (*(ram_ptr!(game, EnemyData; 0xf78)).offset(2)).parameter_1;
        if ((*(ram_ptr!(game, EnemyData; 0xf78)).offset(2)).parameter_1 & 1) != 0 {
            word! { game, 192; kraid_unk9000, g_word_7E9006 }
            let mut v4: u32 = 12620032i32 as u32;
            *(ram_ptr!(game, u16; 0x9002, 1)) = *(&raw mut v4 as *mut u16).offset(1);
            *(ram_ptr!(game, u16; 0x9000, 1)) = v4 as u16;
            word!(game.g_word_7E9004 = -28288i32 as u16);
            let mut v5: u32 = 12620032i32 as u32;
            *(ram_ptr!(game, u16; 0x9008, 1)) = *(&raw mut v5 as *mut u16).offset(1);
            *(ram_ptr!(game, u16; 0x9006, 1)) = v5 as u16;
            word! {game; g_word_7E900A = addr_loc_889180 as u16; g_word_7E900C = 0; }
        } else {
            word! { game, 160;
                kraid_unk9000, g_word_7E9006, g_word_7E900C, g_word_7E900F, g_word_7E9012,
                g_word_7E9015,
            }
            let mut v2: u32 = 10522880i32 as u32;
            *(ram_ptr!(game, u16; 0x9002, 1)) = *(&raw mut v2 as *mut u16).offset(1);
            *(ram_ptr!(game, u16; 0x9000, 1)) = v2 as u16;
            let mut v3: u32 = 10522880i32 as u32;
            *(ram_ptr!(game, u16; 0x9008, 1)) = *(&raw mut v3 as *mut u16).offset(1);
            *(ram_ptr!(game, u16; 0x9006, 1)) = v3 as u16;
            *(ram_ptr!(game, u16; 0x900c, 1)) = -28416i32 as u16;
            *(ram_ptr!(game, u16; 0x9012, 1)) = -28416i32 as u16;
            word! {game; g_word_7E9004 = -28352i32 as u16; g_word_7E900A = -28352i32 as u16; }
            *(ram_ptr!(game, u16; 0x900f, 1)) = -28352i32 as u16;
            *(ram_ptr!(game, u16; 0x9015, 1)) = -28352i32 as u16;
            word!(game.g_word_7E9018 = 0);
        }
        let v6: i32 = k as i32 >> 1;
        word! {game; hdma_object_A[v6] = -2i32 as u16; hdma_object_B[v6] = 1; }
        word! { game, 0; hdma_object_C[v6], hdma_object_D[v6] }
        hdp
    }
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
    unsafe {
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
            do_until! { ((v4) as i16).wrapping_sub((r30) as i16) >= 0 ;
                let v5: u16 = r20;
                let v6: i32 = r20 as i32 >> 1;
                let sample = kSinCosTable8bit_Sext[(v6 + 64) as usize];
                let offset = hdma_wave_offset(
                    sample,
                    (*(ram_ptr!(game, EnemyData; 0xf78)).offset(3)).ai_var_D as u8,
                    byte!((*(ram_ptr!(game, EnemyData; 0xf78)).offset(3)).ai_var_D; 1),
                );
                r20 = (r28 as i32 + v5 as i32 & 0x1ff) as u16;
                *(ram_ptr!(game, u16; 0x9100, v4 as i32 as isize)) =
                    word!(game.reg_BG2HOFS).wrapping_add(offset);
                v4 = v4.wrapping_add(2);
            }
            if ((*(ram_ptr!(game, EnemyData; 0xf78)).offset(1)).parameter_1 & 1) != 0 {
                for i in (0i32..=126).rev().step_by(2) {
                    *(ram_ptr!(game, u16; 0x9180, i as isize)) = (word!(game.reg_BG2HOFS) as i32
                        + word!(game.reg_BG2HOFS) as i32
                        - *(ram_ptr!(game, u16; 0x9100, i as isize)) as i32)
                        as u16;
                }
            } else {
                for j in (0u16..=62).rev().step_by(2) {
                    *(ram_ptr!(game, u16; 0x9140, j as i32 as isize)) =
                        (word!(game.reg_BG2HOFS) as i32 + word!(game.reg_BG2HOFS) as i32
                            - *(ram_ptr!(game, u16; 0x9100, j as i32 as isize)) as i32)
                            as u16;
                }
            }
        } else {
            let v1: i32 = word!(game.hdma_object_index) as i32 >> 1;
            word!(game.hdma_object_instruction_list_pointers[v1] += 2);
            word!(game.hdma_object_instruction_timers[v1] = 1);
        };
    }
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
        SpawnHdmaObject(0x88, &raw const unk_88E8E4 as *const c_void, game);
    }
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
    Runtime::new(game).edit_actor(Key::from_legacy(0x40), Le, |actor| actor.variables[2] = 1);
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
    unsafe {
        // The beam's origin and angle belong to enemy slot 1.
        const BEAM_ENEMY: u16 = 0x40;
        let angle = Runtime::new(game)
            .resolve_actor(Key::from_legacy(BEAM_ENEMY), Le)
            .variables[3];
        // HDMA object C uses a word index; the colors below keep the byte index.
        let width_offset = 0x192c + usize::from(k & !1);
        let width = u16::from_le_bytes([game.g_ram[width_offset], game.g_ram[width_offset + 1]]);
        let origin_y = Runtime::new(game)
            .resolve_actor(Key::from_legacy(BEAM_ENEMY), Le)
            .y
            .whole;
        let relative_y = origin_y.wrapping_sub(game.layer1_y_pos());
        let origin_x = Runtime::new(game)
            .resolve_actor(Key::from_legacy(BEAM_ENEMY), Le)
            .x
            .whole;
        let relative_x = origin_x.wrapping_sub(game.layer1_x_pos());
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
}
pub unsafe fn HdmaobjPreInstr_MorphBallEyeBeamWidening(k: u16, game: &mut Game) {
    unsafe {
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
}
pub unsafe fn HdmaobjPreInstr_MorphBallEyeBeamFull(k: u16, game: &mut Game) {
    unsafe {
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
}
pub unsafe fn HdmaobjPreInstr_MorphBallEyeBeamFading(k: u16, game: &mut Game) {
    unsafe {
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
}
pub unsafe fn SpawnTitleScreenGradientObjs(game: &mut Game) {
    unsafe {
        static unk_88EB6B: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0, 0x31, 0xeb85 as VoidP);
        static unk_88EB63: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0, 0x32, 0xeb73 as VoidP);
        SpawnHdmaObject(0x88, &raw const unk_88EB63 as *const c_void, game);
        SpawnHdmaObject(0x88, &raw const unk_88EB6B as *const c_void, game);
    }
}
pub unsafe fn HdmaobjInsr_ConfigTitleSequenceGradientHDMA(
    k: u16,
    hdp: *const u8,
    game: &mut Game,
) -> *const u8 {
    unsafe {
        *ram_ptr!(game; 0x6f) = 0;
        ConfigureTitleSequenceGradientHDMA(game);
        hdp
    }
}
pub unsafe fn HdmaobjPreInstr_Backdrop_TitleSequenceGradient(k: u16, game: &mut Game) {
    unsafe {
        ConfigureTitleSequenceGradientHDMA(game);
        if word!(game.cinematic_function) as i32 == 0x8b9b68 & 0xffff {
            let v1: i32 = word!(game.hdma_object_index) as i32 >> 1;
            word!(game.hdma_object_instruction_list_pointers[v1] += 2);
            word!(game.hdma_object_instruction_timers[v1] = 1);
        }
    }
}
pub unsafe fn HdmaobjPreInstr_ColorMathControlB_TitleGradient(k: u16, game: &mut Game) {
    unsafe {
        if word!(game.cinematic_function) as i32 == 0x8b9b68 & 0xffff {
            let v1: i32 = word!(game.hdma_object_index) as i32 >> 1;
            word!(game.hdma_object_instruction_list_pointers[v1] += 2);
            word!(game.hdma_object_instruction_timers[v1] = 1);
        }
    }
}
pub unsafe fn SpawnIntroCutsceneCrossfadeHdma(game: &mut Game) {
    unsafe {
        static unk_88EBFB: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0, 0x31, 0xec03 as VoidP);
        SpawnHdmaObject(0x88, &raw const unk_88EBFB as *const c_void, game);
    }
}
pub unsafe fn HdmaobjPreInstr_IntroCutsceneCrossfade(k: u16, game: &mut Game) {
    unsafe {
        if word!(game.eproj_x_pos[0]) == 1 {
            let v1: i32 = word!(game.hdma_object_index) as i32 >> 1;
            word!(game.hdma_object_instruction_list_pointers[v1] += 2);
            word!(game.hdma_object_instruction_timers[v1] = 1);
        }
    }
}
pub unsafe fn CinematicFunction_Intro_Func133(game: &mut Game) {
    unsafe {
        word!(game.g_word_7E0D9C = 1);
        *(ram_ptr!(game, u16; 0xd9e)) = 0x4000;
        word!(game.loop_counter_transfer_enemies_to_vram = 8);
        word! { game, 192; button_config_shoot_x_saved, button_config_itemcancel_y_saved }
        let mut v0: u32 = 12621824i32 as u32;
        *(ram_ptr!(game, u16; 0xd84, 1)) = *(&raw mut v0 as *mut u16).offset(1);
        *(ram_ptr!(game, u16; 0xd82, 1)) = v0 as u16;
        word!(game.button_config_run_b_saved = -26496i32 as u16);
        let mut v1: u32 = 12621824i32 as u32;
        *(ram_ptr!(game, u16; 0xd8a, 1)) = *(&raw mut v1 as *mut u16).offset(1);
        *(ram_ptr!(game, u16; 0xd88, 1)) = v1 as u16;
        word!(game.button_config_aim_down_L_saved = -26496i32 as u16);
        word!(game.button_config_aim_up_R_saved = 0);
        static unk_88EC82: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0x42, 0x11, 0xec8a as VoidP);
        SpawnHdmaObject(0x88, &raw const unk_88EC82 as *const c_void, game);
    }
}
pub unsafe fn HdmaobjInstr_EC9F_ClearVars(k: u16, hdp: *const u8, game: &mut Game) -> *const u8 {
    unsafe {
        let v2: i32 = k as i32 >> 1;
        word! {game; hdma_object_A[v2] = -2i32 as u16; hdma_object_B[v2] = 1; }
        word! { game, 0; hdma_object_C[v2], hdma_object_D[v2] }
        hdp
    }
}
pub unsafe fn HdmaobjPreInstr_ECB6(k: u16, game: &mut Game) {
    unsafe {
        if word!(game.g_word_7E0D9C) != 0 {
            let r28: u16 = 4;
            let n: i32 = 128;
            let v3: i32 = k as i32 >> 1;
            word!(
                game.hdma_object_A[v3] = (2 * word!(game.loop_counter_transfer_enemies_to_vram)
                    as i32
                    + word!(game.hdma_object_A[v3]) as i32
                    & 0x1ff) as u16
            );
            let mut r20: u16 = word!(game.hdma_object_A[v3]);
            let mut v4: u16 = 0;
            do_until! { ((v4 as i32 - n) as i16) >= 0 ;
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
                    game.hdma_table_1[((i >> 1) + 64)] = (word!(game.reg_BG3HOFS) as i32
                        + word!(game.reg_BG3HOFS) as i32
                        - word!(game.hdma_table_1[(i >> 1)]) as i32)
                        as u16
                );
            }
        } else {
            let v2: i32 = word!(game.hdma_object_index) as i32 >> 1;
            word!(game.hdma_object_instruction_list_pointers[v2] += 2);
            word!(game.hdma_object_instruction_timers[v2] = 1);
        };
    }
}
crate::game::address_dispatch! {
    pub unsafe fn CallHdmaobjPreInstr(ea: u32, k: u16, game: &mut Game) => ea;
        call (k, game); default { Unreachable(game); };
    {
        8491739 => [()]; 8946872 => [()]; 10785852 => [()]; 9556607 => xray_background::setup_blending;
        8947439 => HdmaobjPreInstr_Xray; 8947506 => HdmaobjPreInstr_XrayFunc0_NoBeam;
        8947540 => HdmaobjPreInstr_XrayFunc1_BeamWidening;
        8947627 => HdmaobjPreInstr_XrayFunc2_FullBeam;
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
        8963638 => liquid_effects::water_flat; 8968609 => HdmaobjPreInstr_RainBg3Scroll;
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
        8947330 => HdmaobjInstr_Sleep; 8955500 => scrolling_sky::start_wave;
        8959182 => liquid_effects::start_wave; 8963175 => liquid_effects::start_wave;
        8963458 => liquid_effects::start_wave; 8968521 => HdmaobjInstr_SetVideoMode1;
        8968577 => HdmaobjInstr_1938_RandomNumber; 8969419 => HdmaobjInstr_GotoIfEventHappened;
        8971453 => HdmaobjInstr_E4BD;
        fnHdmaobjInstr_InitMorphBallEyeBeamHdma => [{
                    HdmaobjInstr_InitMorphBallEyeBeamHdma(k, j, game)
                }];
        8973471 => HdmaobjInstr_EC9F_ClearVars; 8958889 => liquid_effects::start_lava_sound;
        8973215 => HdmaobjInsr_ConfigTitleSequenceGradientHDMA;
        8968470 => [{
                    sub_88D916(game);
                    j
                }];
    }
}
