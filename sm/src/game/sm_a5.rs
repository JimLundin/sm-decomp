pub use crate::game::constants::bank_86::*;
pub use crate::game::constants::enemy_properties::*;
use crate::game::enemy_layouts::enemy_layout;
use crate::game::sm_80::{NextRandom, QueueMusic_Delayed8, QueueSfx2_Max6, QueueSfx3_Max6};
use crate::game::sm_84::SpawnHardcodedPlm;
use crate::game::sm_86::{SpawnEprojWithGfx, SpawnEprojWithRoomGfx};
use crate::game::sm_88::{sub_88DF34, SpawnBG3ScrollHdmaObject};
use crate::game::sm_90::{
    CallSomeSamusCode, DisableMinimapAndMarkBossRoomAsExplored, Samus_ReleaseFromDraygon_,
    Samus_SetGrabbedByDraygonPose,
};
use crate::game::sm_91::Samus_DealDamage;
use crate::game::sm_a0::{
    Abs16, CalculateAngleFromXY, ConvertAngleToXy, CosineMult8bit, EnemyFunc_A6B4_UsedBySporeSpawn,
    EnemyFunc_B691, EnemyFunc_B7A1, EnemyFunc_Divide, Enemy_ItemDrop_Draygon,
    Enemy_ItemDrop_SporeSpawn, Enemy_SwitchToFrozenAi, NormalEnemyFrozenAI,
    NormalEnemyPowerBombAiSkipDeathAnim_CurEnemy, NormalEnemyShotAi,
    NormalEnemyShotAiSkipDeathAnim_CurEnemy, NormalEnemyTouchAi,
    NormalEnemyTouchAiSkipDeathAnim_CurEnemy, SignExtend8, SineMult8bit, SuitDamageDivision,
    SwitchEnemyAiToMainAi,
};
use crate::game::sm_b4::CreateSpriteAtPos;
use crate::game_state::Game;
use crate::rtl::{AddToHiLo, SetHiLo};
use crate::rtl::{RomPtr, Unreachable};
extern "C" {
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn printf(__format: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;

}
pub use crate::types::Consts_A5;
pub use crate::types::{
    __int16_t, __int32_t, __int8_t, __uint16_t, __uint32_t, __uint8_t, int16, int16_t, int32,
    int32_t, int8, int8_t, size_t, uint16, uint16_t, uint32, uint32_t, uint8, uint8_t, EnemyBase,
    EnemyData, EnemyProps, LongPtr, Point16U, VoidP, VramWriteEntry,
};
pub use crate::types::{Consts_86, EnemyDef};
pub const addr_kDraygon_Ilist_E77D: Consts_A5 = 59261;
pub const addr_stru_A5E729: Consts_A5 = 59177;
pub const addr_kDraygon_Ilist_E6D5: Consts_A5 = 59093;
pub const addr_kDraygon_Ilist_E6C7: Consts_A5 = 59079;
pub const addr_kDraygon_Ilist_E6B9: Consts_A5 = 59065;
pub const addr_kDraygon_BigSprmap_C11B: Consts_A5 = 49435;
pub const addr_kDraygon_BigSprmap_C08F: Consts_A5 = 49295;
pub const addr_word_A5A297: Consts_A5 = 41623;
pub const addr_kDraygon_MorePalettes2: Consts_A5 = 41591;
pub const addr_kDraygon_Palette: Consts_A5 = 41463;
pub const addr_kDraygon_Ilist_9F15: Consts_A5 = 40725;
pub const addr_kDraygon_Ilist_9EA1: Consts_A5 = 40609;
pub const addr_kDraygon_Ilist_9E21: Consts_A5 = 40481;
pub const addr_kDraygon_Ilist_9D9E: Consts_A5 = 40350;
pub const addr_kDraygon_Ilist_9D62: Consts_A5 = 40290;
pub const addr_kDraygon_Ilist_9D5C: Consts_A5 = 40284;
pub const addr_kDraygon_Ilist_9D56: Consts_A5 = 40278;
pub const addr_kDraygon_Ilist_9D50: Consts_A5 = 40272;
pub const addr_kDraygon_Ilist_9D3E: Consts_A5 = 40254;
pub const addr_kDraygon_Ilist_9D1C: Consts_A5 = 40220;
pub const addr_kDraygon_Ilist_9CB4: Consts_A5 = 40116;
pub const addr_kDraygon_Ilist_9C90: Consts_A5 = 40080;
pub const addr_kDraygon_Ilist_9C5A: Consts_A5 = 40026;
pub const addr_kDraygon_Ilist_9C38: Consts_A5 = 39992;
pub const addr_kDraygon_Ilist_9C18: Consts_A5 = 39960;
pub const addr_kDraygon_Ilist_9C06: Consts_A5 = 39942;
pub const addr_kDraygon_Ilist_9BDA: Consts_A5 = 39898;
pub const addr_kDraygon_Ilist_9B5A: Consts_A5 = 39770;
pub const addr_kDraygon_Ilist_9AE8: Consts_A5 = 39656;
pub const addr_kDraygon_Ilist_9A68: Consts_A5 = 39528;
pub const addr_kDraygon_Ilist_99FC: Consts_A5 = 39420;
pub const addr_kDraygon_Ilist_99C0: Consts_A5 = 39360;
pub const addr_kDraygon_Ilist_99BA: Consts_A5 = 39354;
pub const addr_kDraygon_Ilist_99B4: Consts_A5 = 39348;
pub const addr_kDraygon_Ilist_99AE: Consts_A5 = 39342;
pub const addr_kDraygon_Ilist_999C: Consts_A5 = 39324;
pub const addr_kDraygon_Ilist_997A: Consts_A5 = 39290;
pub const addr_kDraygon_Ilist_9944: Consts_A5 = 39236;
pub const addr_kDraygon_Ilist_9922: Consts_A5 = 39202;
pub const addr_kDraygon_Ilist_98FE: Consts_A5 = 39166;
pub const addr_kDraygon_Ilist_98ED: Consts_A5 = 39149;
pub const addr_kDraygon_Ilist_9889: Consts_A5 = 39049;
pub const addr_kDraygon_Ilist_9867: Consts_A5 = 39015;
pub const addr_kDraygon_Ilist_9845: Consts_A5 = 38981;
pub const addr_kDraygon_Ilist_9825: Consts_A5 = 38949;
pub const addr_kDraygon_Ilist_9813: Consts_A5 = 38931;
pub const addr_kDraygon_Ilist_97E7: Consts_A5 = 38887;
pub const addr_kDraygon_Ilist_97D1: Consts_A5 = 38865;
pub const addr_kDraygon_Ilist_97BB: Consts_A5 = 38843;
pub const addr_kDraygon_Ilist_97B9: Consts_A5 = 38841;
pub use crate::types::{Point32, SpawnHardcodedPlmArgs};
enemy_layout! {
    Enemy_Draygon {
        base: EnemyBase,
        words: [
            draygon_var_A, draygon_var_B, draygon_var_C, draygon_var_D, draygon_var_E,
            draygon_var_F, draygon_parameter_1, draygon_parameter_2,
        ],
        pad1: [uint8; 26696],
        words: [
            draygon_var_00, draygon_var_01, draygon_var_02, draygon_var_03, draygon_var_04,
            draygon_var_05, draygon_var_06, draygon_var_07, draygon_var_08, draygon_var_09,
            draygon_var_0A, draygon_var_0B, draygon_var_0C, draygon_var_0D, draygon_var_0E,
            draygon_var_0F, draygon_var_10, draygon_var_11, draygon_var_12, draygon_var_13,
            draygon_var_14, draygon_var_15, draygon_var_16, draygon_var_17, draygon_var_18,
            draygon_var_19, draygon_var_1A, draygon_var_1B, draygon_var_1C, draygon_var_1D,
            draygon_var_1E, draygon_var_1F,
        ],
        pad2: [uint8; 1984],
        words: [
            draygon_var_20, draygon_var_21, draygon_var_22, draygon_var_23, draygon_var_24,
            draygon_var_25, draygon_var_26, draygon_var_27, draygon_var_28, draygon_var_29,
            draygon_var_2A, draygon_var_2B, draygon_var_2C, draygon_var_2D, draygon_var_2E,
            draygon_var_2F, draygon_var_30, draygon_var_31, draygon_var_32, draygon_var_33,
            draygon_var_34, draygon_var_35, draygon_var_36, draygon_var_37, draygon_var_38,
            draygon_var_39, draygon_var_3A, draygon_var_3B, draygon_var_3C, draygon_var_3D,
            draygon_var_3E, draygon_var_3F,
        ],
        pad3: [uint8; 1984],
        words: [
            draygon_var_40, draygon_var_41, draygon_var_42, draygon_var_43, draygon_var_44,
            draygon_var_45, draygon_var_46, draygon_var_47, draygon_var_48, draygon_var_49,
            draygon_var_4A, draygon_var_4B, draygon_var_4C, draygon_var_4D, draygon_var_4E,
            draygon_var_4F, draygon_var_50, draygon_var_51, draygon_var_52, draygon_var_53,
            draygon_var_54, draygon_var_55, draygon_var_56, draygon_var_57, draygon_var_58,
            draygon_var_59, draygon_var_5A, draygon_var_5B, draygon_var_5C, draygon_var_5D,
            draygon_var_5E, draygon_var_5F,
        ],
    }
}

enemy_layout! {
    Enemy_SporeSpawn {
        base: EnemyBase,
        words: [
            ssn_var_A, ssn_var_B, ssn_var_C, ssn_var_D, ssn_var_E, ssn_var_F,
            ssn_parameter_1, ssn_parameter_2,
        ],
        pad1: [uint8; 26696],
        words: [
            ssn_var_00, ssn_var_01, ssn_var_02, ssn_var_03, ssn_var_04, ssn_var_05,
            ssn_var_06, ssn_var_07, ssn_var_08, ssn_var_09, ssn_var_0A, ssn_var_0B,
            ssn_var_0C, ssn_var_0D, ssn_var_0E, ssn_var_0F, ssn_var_10, ssn_var_11,
            ssn_var_12, ssn_var_13, ssn_var_14, ssn_var_15, ssn_var_16, ssn_var_17,
            ssn_var_18, ssn_var_19, ssn_var_1A, ssn_var_1B, ssn_var_1C, ssn_var_1D,
            ssn_var_1E, ssn_var_1F,
        ],
        pad2: [uint8; 1984],
        words: [
            ssn_var_20, ssn_var_21, ssn_var_22, ssn_var_23, ssn_var_24, ssn_var_25,
            ssn_var_26, ssn_var_27, ssn_var_28, ssn_var_29, ssn_var_2A, ssn_var_2B,
            ssn_var_2C, ssn_var_2D, ssn_var_2E, ssn_var_2F, ssn_var_30, ssn_var_31,
            ssn_var_32, ssn_var_33, ssn_var_34, ssn_var_35, ssn_var_36, ssn_var_37,
            ssn_var_38, ssn_var_39, ssn_var_3A, ssn_var_3B, ssn_var_3C, ssn_var_3D,
            ssn_var_3E, ssn_var_3F,
        ],
        pad3: [uint8; 1984],
        words: [
            ssn_var_40, ssn_var_41, ssn_var_42, ssn_var_43, ssn_var_44, ssn_var_45,
            ssn_var_46, ssn_var_47, ssn_var_48, ssn_var_49, ssn_var_4A, ssn_var_4B,
            ssn_var_4C, ssn_var_4D, ssn_var_4E, ssn_var_4F, ssn_var_50, ssn_var_51,
            ssn_var_52, ssn_var_53, ssn_var_54, ssn_var_55, ssn_var_56, ssn_var_57,
            ssn_var_58, ssn_var_59, ssn_var_5A, ssn_var_5B, ssn_var_5C, ssn_var_5D,
            ssn_var_5E, ssn_var_5F,
        ],
    }
}

#[inline]
unsafe fn RomPtr_A0(addr: uint16_t, game: &mut Game) -> *const uint8 {
    RomPtr((0xa00000u32) | ((addr) as u32), game)
}
#[inline]
unsafe fn RomPtr_A5(addr: uint16_t, game: &mut Game) -> *const uint8 {
    RomPtr((0xa50000u32) | ((addr) as u32), game)
}
#[inline]
unsafe fn gVramWriteEntry(a: ::core::ffi::c_int, game: &mut Game) -> *mut VramWriteEntry {
    (&raw mut game.g_ram as *mut uint8).offset((0xd0 as ::core::ffi::c_int + a) as isize)
        as *mut uint8 as *mut VramWriteEntry
}
#[inline]
unsafe fn get_EnemyDef_A2(a: uint16, game: &mut Game) -> *mut EnemyDef {
    RomPtr((0xa00000u32) | ((a) as u32), game) as *mut EnemyDef
}
pub const fnSamus_ScrollFinishedHook_SporeSpawnFight: ::core::ffi::c_int = 0x909589;
pub const fnGrappleBeam_Func2: ::core::ffi::c_int = 0x9bc8c5;
pub const fnnullsub_169_A5: ::core::ffi::c_int = 0xa5804b;
pub const fnDraygon_Func_1: ::core::ffi::c_int = 0xa5871b;
pub const fnDraygon_Func_2: ::core::ffi::c_int = 0xa5878b;
pub const fnDraygon_Func_4: ::core::ffi::c_int = 0xa587f4;
pub const fnDraygon_Func_6: ::core::ffi::c_int = 0xa588b1;
pub const fnDraygon_Func_8: ::core::ffi::c_int = 0xa58922;
pub const fnDraygon_Func_9: ::core::ffi::c_int = 0xa58951;
pub const fnDraygon_Func_10: ::core::ffi::c_int = 0xa589b3;
pub const fnDraygon_Func_11: ::core::ffi::c_int = 0xa58a00;
pub const fnDraygon_Func_12: ::core::ffi::c_int = 0xa58a50;
pub const fnDraygon_Func_13: ::core::ffi::c_int = 0xa58a90;
pub const fnDraygon_Func_14: ::core::ffi::c_int = 0xa58b0a;
pub const fnDraygon_Func_15: ::core::ffi::c_int = 0xa58b52;
pub const fnDraygon_Func_16: ::core::ffi::c_int = 0xa58bae;
pub const fnDraygon_Func_17: ::core::ffi::c_int = 0xa58c33;
pub const fnDraygon_Func_18: ::core::ffi::c_int = 0xa58c8e;
pub const fnDraygon_Func_19: ::core::ffi::c_int = 0xa58cd4;
pub const fnDraygon_Func_20: ::core::ffi::c_int = 0xa58d30;
pub const fnDraygon_Func_21: ::core::ffi::c_int = 0xa58db2;
pub const fnDraygon_Func_22: ::core::ffi::c_int = 0xa58e19;
pub const fnDraygon_Func_23: ::core::ffi::c_int = 0xa58f10;
pub const fnDraygon_Func_24: ::core::ffi::c_int = 0xa58f1e;
pub const fnDraygon_Func_25: ::core::ffi::c_int = 0xa58fd6;
pub const fnDraygon_Func_26: ::core::ffi::c_int = 0xa590d4;
pub const fnDraygon_Func_27: ::core::ffi::c_int = 0xa59105;
pub const fnDraygon_Func_28: ::core::ffi::c_int = 0xa59124;
pub const fnDraygon_Func_29: ::core::ffi::c_int = 0xa59128;
pub const fnDraygon_Func_30: ::core::ffi::c_int = 0xa59154;
pub const fnDraygon_Func_31: ::core::ffi::c_int = 0xa59185;
pub const fnDraygon_Func_32: ::core::ffi::c_int = 0xa59294;
pub const fnDraygon_Func_33: ::core::ffi::c_int = 0xa592ab;
pub const fnDraygon_Func_36: ::core::ffi::c_int = 0xa59342;
pub const fnDraygon_Func_48: uint32 = 10863757;
pub const fnDraygon_Func_49: uint32 = 10863891;
pub const fnnullsub_223: ::core::ffi::c_int = 0xa5eb1a;
pub const fnSporeSpawn_Func_1: uint32 = 10873627;
pub const fnSporeSpawn_Func_2: ::core::ffi::c_int = 0xa5eb52;
pub const fnSporeSpawn_Func_3: uint32 = 10873755;
pub const fnSporeSpawn_Func_4: uint32 = 10873838;
#[inline]
unsafe fn gEnemyData(o: ::core::ffi::c_int, game: &mut Game) -> *mut EnemyData {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut EnemyData
}
#[inline]
unsafe fn Get_Draygon(o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_Draygon {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_Draygon
}
#[inline]
unsafe fn Get_SporeSpawn(o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_SporeSpawn {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_SporeSpawn
}
pub unsafe fn Enemy_GrappleReact_NoInteract_A5(game: &mut Game) {
    SwitchEnemyAiToMainAi(game);
}
pub unsafe fn Enemy_GrappleReact_CancelBeam_A5(game: &mut Game) {
    Enemy_SwitchToFrozenAi(game);
}
pub unsafe fn Enemy_NormalTouchAI_A5(game: &mut Game) {
    NormalEnemyTouchAi(game);
}
pub unsafe fn Enemy_NormalShotAI_A5(game: &mut Game) {
    NormalEnemyShotAi(game);
}
pub unsafe fn Enemy_NormalFrozenAI_A5(game: &mut Game) {
    NormalEnemyFrozenAI(game);
}
pub unsafe fn Draygon_Init(game: &mut Game) {
    let mut j: uint16 = 0;
    let mut i: ::core::ffi::c_int = 48 as ::core::ffi::c_int;
    while i >= 0i32 {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
            .offset(((i >> 1 as ::core::ffi::c_int) + 144 as ::core::ffi::c_int) as isize) =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xa5a217)) as *const uint8 as *mut uint16)
                .offset((i >> 1 as ::core::ffi::c_int) as isize);
        i -= 2 as ::core::ffi::c_int;
    }
    j = 4094 as uint16;
    while (j as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 824 as uint16;
        j = (j).wrapping_sub(2u16);
    }
    (*Get_Draygon(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .base
    .palette_index = 0xe00 as uint16;
    *(&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x178c) as *mut LongPtr)).bank
        as *mut uint16) = 165 as uint16;
    (*((&raw mut game.g_ram as *mut uint8).offset(0x178c) as *mut LongPtr)).addr =
        ((0xa59342i32) as u16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x183c) as *mut uint16) =
        -(1 as ::core::ffi::c_int) as uint16;
    (*Get_Draygon(0 as ::core::ffi::c_int, game))
        .base
        .current_instruction = addr_kDraygon_Ilist_9889 as uint16;
    (*Get_Draygon(0x40 as ::core::ffi::c_int, game))
        .base
        .current_instruction = addr_kDraygon_Ilist_9944 as uint16;
    (*Get_Draygon(0x80 as ::core::ffi::c_int, game))
        .base
        .current_instruction = addr_kDraygon_Ilist_99FC as uint16;
    (*Get_Draygon(0xc0 as ::core::ffi::c_int, game))
        .base
        .current_instruction = addr_kDraygon_Ilist_9813 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x179a) as *mut uint16) = 1024 as uint16;
    (*Get_Draygon(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .draygon_var_A = ((0xa5871bi32) as u16);
    sub_88DF34(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa9) as *mut uint16) = 12 as uint16;
    SpawnBG3ScrollHdmaObject(game);
    (*Get_Draygon(0 as ::core::ffi::c_int, game)).draygon_var_45 = 1 as uint16;
    DisableMinimapAndMarkBossRoomAsExplored(game);
}
pub unsafe fn CallDraygonFunc(ea: uint32, k: uint16, game: &mut Game) {
    match ea {
        10846283 => (),
        10848027 => {
            Draygon_Func_1(game);
        }
        10848139 => {
            Draygon_Func_2(game);
        }
        10848244 => {
            Draygon_Func_4(game);
        }
        10848433 => {
            Draygon_Func_6(game);
        }
        10848546 => {
            Draygon_Func_8(game);
        }
        10848593 => {
            Draygon_Func_9(game);
        }
        10848691 => {
            Draygon_Func_10(game);
        }
        10848768 => {
            Draygon_Func_11(game);
        }
        10848848 => {
            Draygon_Func_12(game);
        }
        10848912 => {
            Draygon_Func_13(game);
        }
        10849034 => {
            Draygon_Func_14(game);
        }
        10849106 => {
            Draygon_Func_15(game);
        }
        10849198 => {
            Draygon_Func_16(game);
        }
        10849331 => {
            Draygon_Func_17(k, game);
        }
        10849422 => {
            Draygon_Func_18(game);
        }
        10849492 => {
            Draygon_Func_19(game);
        }
        10849584 => {
            Draygon_Func_20(game);
        }
        10849714 => {
            Draygon_Func_21(k, game);
        }
        10849817 => {
            Draygon_Func_22(game);
        }
        10850064 => {
            Draygon_Func_23(game);
        }
        10850078 => {
            Draygon_Func_24(k, game);
        }
        10850262 => {
            Draygon_Func_25(k, game);
        }
        10850516 => {
            Draygon_Func_26(k, game);
        }
        10850565 => {
            Draygon_Func_27(k, game);
        }
        10850596 => {
            Draygon_Func_28(k, game);
        }
        10850600 => {
            Draygon_Func_29(game);
        }
        10850644 => {
            Draygon_Func_30(game);
        }
        10850693 => {
            Draygon_Func_31(game);
        }
        10850964 => {
            Draygon_Func_32(game);
        }
        10850987 => {
            Draygon_Func_33(game);
        }
        10863757 => {
            Draygon_Func_48(k, game);
        }
        10863891 => {
            Draygon_Func_49(k, game);
        }
        10873626 => (),
        10873627 => {
            SporeSpawn_Func_1(game);
        }
        10873682 => {
            SporeSpawn_Func_2(k, game);
        }
        10873755 => {
            SporeSpawn_Func_3(game);
        }
        10873838 => {
            SporeSpawn_Func_4(game);
        }
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn Draygon_Main(game: &mut Game) {
    let EK: *mut Enemy_Draygon = Get_Draygon(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallDraygonFunc(
        (((*EK).draygon_var_A) as u32) | (0xa50000u32),
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
    let E0: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
    let x_pos: uint16 = (*E0).base.x_pos;
    let E1: *mut Enemy_Draygon = Get_Draygon(0x40 as ::core::ffi::c_int, game);
    (*E1).base.x_pos = x_pos;
    let E2: *mut Enemy_Draygon = Get_Draygon(0x80 as ::core::ffi::c_int, game);
    (*E2).base.x_pos = x_pos;
    let E3: *mut Enemy_Draygon = Get_Draygon(0xc0 as ::core::ffi::c_int, game);
    (*E3).base.x_pos = x_pos;
    let y_pos: uint16 = (*E0).base.y_pos;
    (*E1).base.y_pos = y_pos;
    (*E2).base.y_pos = y_pos;
    (*E3).base.y_pos = y_pos;
}
pub unsafe fn Draygon_Func_1(game: &mut Game) {
    let mut v2: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    Draygon_Func_3(game);
    let E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
    if (*E).draygon_var_B as ::core::ffi::c_int - 256 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        if (*E).draygon_var_B == 0 {
            v2 = gVramWriteEntry(
                *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16)
                    as ::core::ffi::c_int,
                game,
            );
            (*v2).size = 1536 as uint16;
            (*v2).src.addr = -(27648 as ::core::ffi::c_int) as VoidP;
            *(&raw mut (*v2).src.bank as *mut uint16) = 177 as uint16;
            (*v2).vram_dst = 27904 as uint16;
            let ref mut fresh0 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
            *fresh0 = (*fresh0).wrapping_add(7u16);
            Draygon_Func_45(game);
        }
        (*E).draygon_var_B = (*E).draygon_var_B.wrapping_add(1);
    } else {
        (*E).draygon_var_A = ((0xa5878bi32) as u16);
        (*E).draygon_var_B = 0 as uint16;
        let x_pos: uint16 = (*E).base.x_pos;
        (*E).draygon_var_00 = x_pos;
        (*E).draygon_var_02 = (x_pos).wrapping_add(672u16);
        (*E).draygon_var_01 = (*E).base.y_pos;
        (*E).base.x_pos = (*E).draygon_var_00;
        (*E).base.y_pos = (*E).draygon_var_01;
        (*E).draygon_var_0F = 24 as uint16;
    };
}
pub unsafe fn Draygon_Func_2(game: &mut Game) {
    Draygon_Func_3(game);
    let E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
    if (*E).draygon_var_B as ::core::ffi::c_int - 1232 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        Draygon_Func_46(game);
        (*E).draygon_var_B = (*E).draygon_var_B.wrapping_add(1);
    } else {
        (*E).draygon_var_A = ((0xa587f4i32) as u16);
        (*E).draygon_var_B = 0 as uint16;
    };
}
pub unsafe fn Draygon_Func_3(game: &mut Game) {
    let mut Random: int8 = 0;
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x5b6) as *mut uint16) as ::core::ffi::c_int
        & 0x3f as ::core::ffi::c_int)
        == 0i32
    {
        Random = NextRandom(game) as int8;
        let result: uint16 = (2 as ::core::ffi::c_int
            * ((Random as ::core::ffi::c_int & 3 as ::core::ffi::c_int) + 2 as ::core::ffi::c_int))
            as uint16;
        if (*Get_Draygon(result as ::core::ffi::c_int, game)).draygon_var_40 == 0 {
            let v2: ::core::ffi::c_int = (4 as ::core::ffi::c_int
                * ((Random as ::core::ffi::c_int & 3 as ::core::ffi::c_int)
                    + 2 as ::core::ffi::c_int)) as uint16
                as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int;
            game.eproj_spawn_pt = Point16U {
                x: *(game.g_rom.offset(crate::rtl::lorom_offset(0xa587dc)) as *const uint8
                    as *mut uint16)
                    .offset(v2 as isize),
                y: *(game.g_rom.offset(crate::rtl::lorom_offset(0xa587dc)) as *const uint8
                    as *mut uint16)
                    .offset((v2 + 1 as ::core::ffi::c_int) as isize),
            };
            SpawnEprojWithGfx(3 as uint16, result, addr_stru_868E5E as uint16, game);
        }
    }
}
pub unsafe fn Draygon_Func_4(game: &mut Game) {
    Draygon_Func_3(game);
    Draygon_Func_5(game);
    let E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
    (*E).draygon_var_A = ((0xa588b1i32) as u16);
    (*E).base.current_instruction = addr_kDraygon_Ilist_97D1 as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).draygon_var_20 = 1 as uint16;
}
pub unsafe fn Draygon_Func_5(game: &mut Game) {
    let E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
    let mut varE24: uint16 = 384 as uint16;
    let mut varE22: uint16 = 0 as uint16;
    let mut varE26: uint16 = 0 as uint16;
    loop {
        varE22 = (varE22).wrapping_add((*E).draygon_var_0F);
        let v1: uint16 = (varE24).wrapping_sub(
            (*(&raw mut varE22 as *mut uint8).offset(
                (::core::mem::size_of::<uint16>())
                    .wrapping_div(::core::mem::size_of::<uint8>())
                    .wrapping_sub(1_usize) as isize,
            )) as u16,
        );
        if ((v1) as i16).wrapping_sub(((*E).draygon_var_01) as i16) < 0i16 {
            break;
        }
        varE24 = (varE24).wrapping_sub(
            (*(&raw mut varE22 as *mut uint8).offset(
                (::core::mem::size_of::<uint16>())
                    .wrapping_div(::core::mem::size_of::<uint8>())
                    .wrapping_sub(1_usize) as isize,
            )) as u16,
        );
        let fresh4 = varE26;
        varE26 = varE26.wrapping_add(1);
        *(((&raw mut game.g_ram as *mut uint8).offset(0x9002) as *mut uint16 as *mut uint8)
            .offset(fresh4 as ::core::ffi::c_int as isize) as *mut uint16) = v1;
        varE26 = (varE26).wrapping_add(2u16);
        varE26 = varE26.wrapping_add(1);
        if varE26 as ::core::ffi::c_int - 2048 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            == 0
        {
            Unreachable(game);
        }
    }
    let t: uint32 = EnemyFunc_Divide(
        (varE26 as ::core::ffi::c_int >> 2 as ::core::ffi::c_int) as uint32,
        ((Abs16(
            ((*E).draygon_var_00)
                .wrapping_sub(*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)),
        ) as ::core::ffi::c_int)
            << 16 as ::core::ffi::c_int) as uint32,
    );
    SetHiLo(&raw mut (*E).draygon_var_D, &raw mut (*E).draygon_var_E, t);
    let v6: uint16 = varE26;
    *(((&raw mut game.g_ram as *mut uint8).offset(0x9002) as *mut uint16 as *mut uint8)
        .offset(varE26 as ::core::ffi::c_int as isize) as *mut uint16) = (*E).base.y_pos;
    (*E).draygon_var_B = v6;
    (*E).draygon_var_C = v6;
}
pub unsafe fn Draygon_Func_6(game: &mut Game) {
    Draygon_Func_3(game);
    Draygon_Func_35(game);
    let E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
    let draygon_var_B: uint16 = (*E).draygon_var_B;
    if draygon_var_B == 104u16 {
        let E3: *mut Enemy_Draygon = Get_Draygon(0xc0 as ::core::ffi::c_int, game);
        (*E3).base.current_instruction = addr_kDraygon_Ilist_9C06 as uint16;
        (*E3).base.instruction_timer = 1 as uint16;
    }
    (*Get_Draygon(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .base
    .y_pos = *(((&raw mut game.g_ram as *mut uint8).offset(0x9002) as *mut uint16 as *mut uint8)
        .offset(draygon_var_B as ::core::ffi::c_int as isize) as *mut uint16);
    (*E).draygon_var_B = ((*E).draygon_var_B).wrapping_sub(4u16);
    if (*E).draygon_var_B != 0 {
        AddToHiLo(
            &raw mut (*E).base.x_pos,
            &raw mut (*E).base.x_subpos,
            ((*E).draygon_var_D as uint32) << 16 as ::core::ffi::c_int
                | (*E).draygon_var_E as uint32,
        );
    } else {
        (*E).draygon_var_A = ((0xa58922i32) as u16);
    };
}
pub unsafe fn Draygon_Func_7(game: &mut Game) {
    let E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x5b6) as *mut uint16) as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int)
        == 0i32
    {
        (*E).base.current_instruction = addr_kDraygon_Ilist_98FE as uint16;
        if (*E).draygon_var_20 != 0 {
            (*E).base.current_instruction = addr_kDraygon_Ilist_9C90 as uint16;
        }
        (*E).base.instruction_timer = 1 as uint16;
    }
}
pub unsafe fn Draygon_Func_8(game: &mut Game) {
    Draygon_Func_3(game);
    let E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
    let t: uint32 = EnemyFunc_Divide(
        ((*E).draygon_var_C as ::core::ffi::c_int >> 2 as ::core::ffi::c_int) as uint32,
        ((672 as ::core::ffi::c_int - (*E).base.x_pos as ::core::ffi::c_int)
            << 16 as ::core::ffi::c_int) as uint32,
    );
    SetHiLo(&raw mut (*E).draygon_var_D, &raw mut (*E).draygon_var_E, t);
    (*E).draygon_var_A = ((0xa58951i32) as u16);
}
pub unsafe fn Draygon_Func_9(game: &mut Game) {
    Draygon_Func_3(game);
    let E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
    let draygon_var_B: uint16 = (*E).draygon_var_B;
    if draygon_var_B == 104u16 {
        let E3: *mut Enemy_Draygon = Get_Draygon(0xc0 as ::core::ffi::c_int, game);
        (*E3).base.current_instruction = addr_kDraygon_Ilist_9BDA as uint16;
        (*E3).base.instruction_timer = 1 as uint16;
    }
    (*Get_Draygon(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .base
    .y_pos = *(((&raw mut game.g_ram as *mut uint8).offset(0x9002) as *mut uint16 as *mut uint8)
        .offset(draygon_var_B as ::core::ffi::c_int as isize) as *mut uint16);
    (*E).draygon_var_B = ((*E).draygon_var_B).wrapping_add(4u16);
    if (*E).draygon_var_B == (*E).draygon_var_C {
        (*E).draygon_var_B = 0 as uint16;
        if (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
            as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int)
            != 0i32
        {
            (*E).draygon_var_A = ((0xa589b3i32) as u16);
        } else {
            (*E).draygon_var_A = ((0xa58c8ei32) as u16);
        }
    } else {
        AddToHiLo(
            &raw mut (*E).base.x_pos,
            &raw mut (*E).base.x_subpos,
            ((*E).draygon_var_D as uint32) << 16 as ::core::ffi::c_int
                | (*E).draygon_var_E as uint32,
        );
    };
}
pub unsafe fn Draygon_Func_10(game: &mut Game) {
    Draygon_Func_3(game);
    let E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
    printf(b"Wtf is varE26?\n\0" as *const u8 as *const ::core::ffi::c_char);
    let varE26: uint16 = 0 as uint16;
    let t: uint32 = EnemyFunc_Divide(
        (varE26 as ::core::ffi::c_int >> 2 as ::core::ffi::c_int) as uint32,
        ((Abs16(
            ((*E).draygon_var_02)
                .wrapping_sub(*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)),
        ) as ::core::ffi::c_int)
            << 16 as ::core::ffi::c_int) as uint32,
    );
    SetHiLo(&raw mut (*E).draygon_var_D, &raw mut (*E).draygon_var_E, t);
    (*E).draygon_var_A = ((0xa58a00i32) as u16);
    (*E).base.current_instruction = addr_kDraygon_Ilist_97BB as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).draygon_var_20 = 0 as uint16;
    (*E).draygon_var_B = (*E).draygon_var_C;
}
pub unsafe fn Draygon_Func_11(game: &mut Game) {
    Draygon_Func_3(game);
    Draygon_Func_35(game);
    let E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
    let draygon_var_B: uint16 = (*E).draygon_var_B;
    if draygon_var_B == 104u16 {
        let E3: *mut Enemy_Draygon = Get_Draygon(0xc0 as ::core::ffi::c_int, game);
        (*E3).base.current_instruction = addr_kDraygon_Ilist_9813 as uint16;
        (*E3).base.instruction_timer = 1 as uint16;
    }
    (*Get_Draygon(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .base
    .y_pos = *(((&raw mut game.g_ram as *mut uint8).offset(0x9002) as *mut uint16 as *mut uint8)
        .offset(draygon_var_B as ::core::ffi::c_int as isize) as *mut uint16);
    (*E).draygon_var_B = ((*E).draygon_var_B).wrapping_sub(4u16);
    if (*E).draygon_var_B != 0 {
        AddToHiLo(
            &raw mut (*E).base.x_pos,
            &raw mut (*E).base.x_subpos,
            -(((*E).draygon_var_D as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
                | (*E).draygon_var_E as ::core::ffi::c_int) as uint32,
        );
    } else {
        (*E).draygon_var_A = ((0xa58a50i32) as u16);
    };
}
pub unsafe fn Draygon_Func_12(game: &mut Game) {
    Draygon_Func_3(game);
    let E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
    let t: uint32 = EnemyFunc_Divide(
        ((*E).draygon_var_C as ::core::ffi::c_int >> 2 as ::core::ffi::c_int) as uint32,
        (((*E).base.x_pos as ::core::ffi::c_int - (*E).draygon_var_00 as ::core::ffi::c_int)
            << 16 as ::core::ffi::c_int) as uint32,
    );
    (*E).draygon_var_D = (t >> 16 as ::core::ffi::c_int) as uint16;
    (*E).draygon_var_E = t as uint16;
    (*E).draygon_var_A = ((0xa58a90i32) as u16);
}
pub unsafe fn Draygon_Func_13(game: &mut Game) {
    Draygon_Func_3(game);
    Draygon_Func_35(game);
    let E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
    let draygon_var_B: uint16 = (*E).draygon_var_B;
    if draygon_var_B == 104u16 {
        let E3: *mut Enemy_Draygon = Get_Draygon(0xc0 as ::core::ffi::c_int, game);
        (*E3).base.current_instruction = addr_kDraygon_Ilist_97E7 as uint16;
        (*E3).base.instruction_timer = 1 as uint16;
    }
    (*Get_Draygon(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .base
    .y_pos = *(((&raw mut game.g_ram as *mut uint8).offset(0x9002) as *mut uint16 as *mut uint8)
        .offset(draygon_var_B as ::core::ffi::c_int as isize) as *mut uint16);
    (*E).draygon_var_B = ((*E).draygon_var_B).wrapping_add(4u16);
    if (*E).draygon_var_B == (*E).draygon_var_C {
        if (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
            as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int)
            != 0i32
        {
            (*E).draygon_var_A = ((0xa587f4i32) as u16);
            (*E).draygon_var_B = 0 as uint16;
            (*E).base.x_pos = (*E).draygon_var_00;
            (*E).base.y_pos = (*E).draygon_var_01;
        } else {
            (*E).draygon_var_A = ((0xa58b0ai32) as u16);
            (*E).base.x_pos = (*E).draygon_var_00;
        }
    } else {
        AddToHiLo(
            &raw mut (*E).base.x_pos,
            &raw mut (*E).base.x_subpos,
            -(((*E).draygon_var_D as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
                | (*E).draygon_var_E as ::core::ffi::c_int) as uint32,
        );
    };
}
pub unsafe fn Draygon_Func_14(game: &mut Game) {
    Draygon_Func_3(game);
    let E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
    (*E).base.x_pos = -(80 as ::core::ffi::c_int) as uint16;
    (*E).base.y_pos = 384 as uint16;
    (*E).draygon_var_E = 0 as uint16;
    (*E).draygon_var_D = 1 as uint16;
    (*E).draygon_var_0D = 0 as uint16;
    let E3: *mut Enemy_Draygon = Get_Draygon(0xc0 as ::core::ffi::c_int, game);
    (*E3).base.current_instruction = addr_kDraygon_Ilist_9C06 as uint16;
    (*E3).base.instruction_timer = 1 as uint16;
    (*E).draygon_var_A = ((0xa58b52i32) as u16);
    (*E).base.current_instruction = addr_kDraygon_Ilist_97D1 as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).draygon_var_20 = 1 as uint16;
}
pub unsafe fn Draygon_Func_15(game: &mut Game) {
    Draygon_Func_3(game);
    Draygon_Func_35(game);
    let E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
    let v2: uint16 = Abs16(
        ((*E).base.x_pos)
            .wrapping_sub(*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)),
    );
    if v2 as ::core::ffi::c_int - 208 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        (*E).draygon_var_A = ((0xa58baei32) as u16);
        (*E).draygon_var_03 = 16 as uint16;
    } else {
        (*E).base.y_pos =
            (CosineMult8bit((*E).draygon_var_0D, 32 as uint16, game)).wrapping_add(384u16);
        (*E).draygon_var_0D = ((((*E).draygon_var_0D) as u8).wrapping_add(1u8)) as uint16;
        AddToHiLo(
            &raw mut (*E).base.x_pos,
            &raw mut (*E).base.x_subpos,
            ((*E).draygon_var_D as uint32) << 16 as ::core::ffi::c_int
                | (*E).draygon_var_E as uint32,
        );
    };
}
pub unsafe fn Draygon_Func_16(game: &mut Game) {
    let mut current_block: u64;
    let E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
    Draygon_Func_3(game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa66) as *mut uint16) != 0 {
        current_block = 1399980599923904196;
    } else {
        if (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
            as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int)
            == 0i32
        {
            let v1: uint16 = ((*E).draygon_var_03).wrapping_sub(1u16);
            (*E).draygon_var_03 = v1;
            if v1 == 0 {
                current_block = 11361156962102030361;
            } else {
                (*E).base.current_instruction = addr_kDraygon_Ilist_9C90 as uint16;
                (*E).base.instruction_timer = 1 as uint16;
                current_block = 6483416627284290920;
            }
        } else {
            current_block = 6483416627284290920;
        }
        match current_block {
            6483416627284290920 => {
                (*E).base.y_pos =
                    (CosineMult8bit((*E).draygon_var_0D, 32 as uint16, game)).wrapping_add(384u16);
                (*E).draygon_var_0D = ((((*E).draygon_var_0D) as u8).wrapping_add(1u8)) as uint16;
                AddToHiLo(
                    &raw mut (*E).base.x_pos,
                    &raw mut (*E).base.x_subpos,
                    ((*E).draygon_var_D as uint32) << 16 as ::core::ffi::c_int
                        | (*E).draygon_var_E as uint32,
                );
                if ((*E).base.x_pos as int16) >= 0i16
                    && (*E).base.x_pos as ::core::ffi::c_int - 672 as ::core::ffi::c_int
                        & 0x8000 as ::core::ffi::c_int
                        == 0
                {
                    current_block = 11361156962102030361;
                } else {
                    current_block = 7976072742316086414;
                }
            }
            _ => {}
        }
        match current_block {
            7976072742316086414 => {}
            _ => {
                if *((&raw mut game.g_ram as *mut uint8).offset(0xa66) as *mut uint16) == 0 {
                    let E3: *mut Enemy_Draygon = Get_Draygon(0xc0 as ::core::ffi::c_int, game);
                    (*E3).base.current_instruction = addr_kDraygon_Ilist_9BDA as uint16;
                    (*E3).base.instruction_timer = 1 as uint16;
                    (*E).draygon_var_A = ((0xa58c33i32) as u16);
                    return;
                }
                current_block = 1399980599923904196;
            }
        }
    }
    match current_block {
        1399980599923904196 => {
            (*E).draygon_var_A = ((0xa58e19i32) as u16);
        }
        _ => {}
    };
}
pub unsafe fn Draygon_Func_17(k: uint16, game: &mut Game) {
    Draygon_Func_35(game);
    let E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
    (*E).base.y_pos =
        (CosineMult8bit((*E).draygon_var_0D, 32 as uint16, game)).wrapping_add(384u16);
    (*E).draygon_var_0D = ((((*E).draygon_var_0D) as u8).wrapping_add(1u8)) as uint16;
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        ((*E).draygon_var_D as uint32) << 16 as ::core::ffi::c_int | (*E).draygon_var_E as uint32,
    );
    if ((*E).base.x_pos as int16) >= 0i16
        && (*E).base.x_pos as ::core::ffi::c_int - 672 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
    {
        (*E).draygon_var_A = ((0xa589b3i32) as u16);
        (*E).base.x_pos = 592 as uint16;
        (*E).base.y_pos = -(80 as ::core::ffi::c_int) as uint16;
        (*E).draygon_var_B = (*E).draygon_var_C;
    }
}
pub unsafe fn Draygon_Func_18(game: &mut Game) {
    let E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
    (*E).base.x_pos = (*E).draygon_var_02;
    (*E).base.y_pos = 384 as uint16;
    (*E).draygon_var_E = 0 as uint16;
    (*E).draygon_var_D = 1 as uint16;
    (*E).draygon_var_0D = 0 as uint16;
    let E3: *mut Enemy_Draygon = Get_Draygon(0xc0 as ::core::ffi::c_int, game);
    (*E3).base.current_instruction = addr_kDraygon_Ilist_9C06 as uint16;
    (*E3).base.instruction_timer = 1 as uint16;
    (*E).draygon_var_A = ((0xa58cd4i32) as u16);
    (*E).base.current_instruction = addr_kDraygon_Ilist_97BB as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).draygon_var_20 = 0 as uint16;
}
pub unsafe fn Draygon_Func_19(game: &mut Game) {
    Draygon_Func_3(game);
    Draygon_Func_35(game);
    let E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
    let v2: uint16 = Abs16(
        ((*E).base.x_pos)
            .wrapping_sub(*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)),
    );
    if v2 as ::core::ffi::c_int - 208 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        (*E).draygon_var_A = ((0xa58d30i32) as u16);
        (*E).draygon_var_03 = 16 as uint16;
    } else {
        (*E).base.y_pos =
            (CosineMult8bit((*E).draygon_var_0D, 32 as uint16, game)).wrapping_add(384u16);
        (*E).draygon_var_0D = ((((*E).draygon_var_0D) as u8).wrapping_add(1u8)) as uint16;
        AddToHiLo(
            &raw mut (*E).base.x_pos,
            &raw mut (*E).base.x_subpos,
            -(((*E).draygon_var_D as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
                | (*E).draygon_var_E as ::core::ffi::c_int) as uint32,
        );
    };
}
pub unsafe fn Draygon_Func_20(game: &mut Game) {
    let mut current_block: u64;
    let E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa66) as *mut uint16) != 0 {
        current_block = 11384386367536015713;
    } else {
        if (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
            as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int)
            == 0i32
        {
            let v1: uint16 = ((*E).draygon_var_03).wrapping_sub(1u16);
            (*E).draygon_var_03 = v1;
            if v1 == 0 {
                current_block = 17278387905106781498;
            } else {
                (*E).base.current_instruction = addr_kDraygon_Ilist_98FE as uint16;
                (*E).base.instruction_timer = 1 as uint16;
                current_block = 6873731126896040597;
            }
        } else {
            current_block = 6873731126896040597;
        }
        match current_block {
            6873731126896040597 => {
                (*E).base.y_pos =
                    (CosineMult8bit((*E).draygon_var_0D, 32 as uint16, game)).wrapping_add(384u16);
                (*E).draygon_var_0D = ((((*E).draygon_var_0D) as u8).wrapping_add(1u8)) as uint16;
                AddToHiLo(
                    &raw mut (*E).base.x_pos,
                    &raw mut (*E).base.x_subpos,
                    -(((*E).draygon_var_D as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
                        | (*E).draygon_var_E as ::core::ffi::c_int) as uint32,
                );
                if (*E).base.x_pos < 0u16
                    && (*E).base.x_pos as ::core::ffi::c_int + 80 as ::core::ffi::c_int
                        & 0x8000 as ::core::ffi::c_int
                        != 0
                {
                    current_block = 17278387905106781498;
                } else {
                    current_block = 10048703153582371463;
                }
            }
            _ => {}
        }
        match current_block {
            10048703153582371463 => {}
            _ => {
                if *((&raw mut game.g_ram as *mut uint8).offset(0xa66) as *mut uint16) == 0 {
                    let E3: *mut Enemy_Draygon = Get_Draygon(0xc0 as ::core::ffi::c_int, game);
                    (*E3).base.current_instruction = addr_kDraygon_Ilist_97E7 as uint16;
                    (*E3).base.instruction_timer = 1 as uint16;
                    (*Get_Draygon(0 as ::core::ffi::c_int, game)).draygon_var_A =
                        ((0xa58db2i32) as u16);
                    return;
                }
                current_block = 11384386367536015713;
            }
        }
    }
    match current_block {
        11384386367536015713 => {
            (*Get_Draygon(0 as ::core::ffi::c_int, game)).draygon_var_A = ((0xa58e19i32) as u16);
        }
        _ => {}
    };
}
pub unsafe fn Draygon_Func_21(k: uint16, game: &mut Game) {
    let E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
    Draygon_Func_35(game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa66) as *mut uint16) != 0 {
        (*E).draygon_var_A = ((0xa58e19i32) as u16);
    } else {
        (*E).base.y_pos =
            (CosineMult8bit((*E).draygon_var_0D, 32 as uint16, game)).wrapping_add(384u16);
        (*E).draygon_var_0D = ((((*E).draygon_var_0D) as u8).wrapping_add(1u8)) as uint16;
        AddToHiLo(
            &raw mut (*E).base.x_pos,
            &raw mut (*E).base.x_subpos,
            -(((*E).draygon_var_D as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
                | (*E).draygon_var_E as ::core::ffi::c_int) as uint32,
        );
        if ((*E).base.x_pos as int16) < 0i16
            && (*E).base.x_pos as ::core::ffi::c_int + 80 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
        {
            (*E).draygon_var_A = ((0xa587f4i32) as u16);
            (*E).base.x_pos = -(80 as ::core::ffi::c_int) as uint16;
            (*E).base.y_pos = -(80 as ::core::ffi::c_int) as uint16;
            (*E).draygon_var_B = (*E).draygon_var_C;
        }
    };
}
pub unsafe fn Draygon_Func_22(game: &mut Game) {
    let E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
    let mut v2: int16 = 0;
    let mut v4: uint16 = 0;
    Draygon_Func_3(game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa66) as *mut uint16) != 0 {
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
        v2 = 8 as int16;
        if (*E).draygon_var_20 == 0 {
            v2 = -(8 as ::core::ffi::c_int) as int16;
        }
        let v3: uint16 = Abs16(
            ((*E).base.x_pos as ::core::ffi::c_int + v2 as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                    as ::core::ffi::c_int) as uint16,
        );
        if v3 as ::core::ffi::c_int - 8 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0
            && {
                v4 = Abs16(((*E).base.y_pos).wrapping_sub(
                    *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16),
                ));
                v4 as ::core::ffi::c_int - 8 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                    != 0
            }
        {
            let mut v5: uint16 = addr_kDraygon_Ilist_9845 as uint16;
            if (*E).draygon_var_20 != 0 {
                v5 = addr_kDraygon_Ilist_9C38 as uint16;
            }
            let E3: *mut Enemy_Draygon = Get_Draygon(0xc0 as ::core::ffi::c_int, game);
            (*E3).base.current_instruction = v5;
            (*E3).base.instruction_timer = 1 as uint16;
            if CallSomeSamusCode(0xd as uint16, game) != 0 {
                (*E).draygon_var_A = ((0xa58f10i32) as u16);
            } else {
                Samus_SetGrabbedByDraygonPose(((*E).draygon_var_20 != 0u16) as uint16, game);
                (*E).draygon_var_06 = 256 as uint16;
                (*E).draygon_var_07 = 384 as uint16;
                (*E).draygon_var_05 = 0 as uint16;
                (*E).draygon_var_09 = 0 as uint16;
                (*E).draygon_var_08 = 192 as uint16;
                (*E).draygon_var_0B = 2048 as uint16;
                (*E).draygon_var_A = ((0xa58f1ei32) as u16);
            }
        } else {
            let varE20: uint16 = ((64u8).wrapping_sub(
                (CalculateAngleFromXY(
                    (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16))
                        .wrapping_sub((*E).base.x_pos),
                    (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16))
                        .wrapping_sub((*E).base.y_pos),
                    game,
                )) as u8,
            )) as uint16;
            let pt: Point32 = ConvertAngleToXy(varE20, 2 as uint16, game);
            EnemyFunc_B691(varE20, pt, game);
        }
    } else {
        (*E).draygon_var_A = ((0xa59154i32) as u16);
    };
}
pub unsafe fn Draygon_Func_23(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xd32) as *mut uint16) = ((0x9bc8c5i32) as u16);
    (*Get_Draygon(0 as ::core::ffi::c_int, game)).draygon_var_A = ((0xa59128i32) as u16);
}
pub unsafe fn Draygon_Func_24(k: uint16, game: &mut Game) {
    let E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
    let mut v5: uint16 = 0;
    if (*((&raw mut game.g_ram as *mut uint8).offset(0xa64) as *mut uint16) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int)
        != 0i32
    {
        (*E).base.flash_timer = (((*get_EnemyDef_A2((*E).base.enemy_ptr, game)).hurt_ai_time)
            as u16)
            .wrapping_add(8u16);
        (*E).base.ai_handler_bits = ((*E).base.ai_handler_bits) | (2u16);
    } else {
        let v4: uint16 = Abs16(((*E).base.x_pos).wrapping_sub(256u16));
        if v4 as ::core::ffi::c_int - 2 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0
            && {
                v5 = Abs16(((*E).base.y_pos).wrapping_sub(384u16));
                v5 as ::core::ffi::c_int - 2 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                    != 0
            }
        {
            (*E).draygon_var_A = ((0xa58fd6i32) as u16);
            let mut v6: uint16 = addr_kDraygon_Ilist_9922 as uint16;
            if (*E).draygon_var_20 != 0 {
                v6 = addr_kDraygon_Ilist_9CB4 as uint16;
            }
            let EK: *mut Enemy_Draygon = Get_Draygon(k as ::core::ffi::c_int, game);
            (*EK).base.current_instruction = v6;
            (*EK).base.instruction_timer = 1 as uint16;
            (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
        } else {
            let varE20: uint16 = ((64u8).wrapping_sub(
                (CalculateAngleFromXY(
                    (256u16).wrapping_sub((*E).base.x_pos),
                    (384u16).wrapping_sub((*E).base.y_pos),
                    game,
                )) as u8,
            )) as uint16;
            let pt: Point32 = ConvertAngleToXy(varE20, 2 as uint16, game);
            EnemyFunc_B691(varE20, pt, game);
            Draygon_Func_40(k, game);
        }
    };
}
pub unsafe fn Draygon_Func_25(k: uint16, game: &mut Game) {
    let E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
    let mut v6: uint16 = 0;
    if (*((&raw mut game.g_ram as *mut uint8).offset(0xa64) as *mut uint16) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int)
        != 0i32
    {
        (*E).base.flash_timer = (((*get_EnemyDef_A2((*E).base.enemy_ptr, game)).hurt_ai_time)
            as u16)
            .wrapping_add(8u16);
        (*E).base.ai_handler_bits = ((*E).base.ai_handler_bits) | (2u16);
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) as uint8 != 0 {
        (*E).base.x_pos = ((*E).draygon_var_06).wrapping_add(CosineMult8bit(
            (*E).draygon_var_08,
            (*E).draygon_var_05,
            game,
        ));
        (*E).base.y_pos = ((*E).draygon_var_07).wrapping_add(SineMult8bit(
            (*E).draygon_var_08,
            ((*E).draygon_var_05 as ::core::ffi::c_int >> 2 as ::core::ffi::c_int) as uint16,
            game,
        ));
        if ((*Get_Draygon(k as ::core::ffi::c_int, game))
            .base
            .frame_counter as ::core::ffi::c_int
            & 7 as ::core::ffi::c_int)
            == 0i32
        {
            let x: uint16 = (if (*E).draygon_var_20 != 0u16 {
                (*E).base.x_pos as ::core::ffi::c_int + 32 as ::core::ffi::c_int
            } else {
                (*E).base.x_pos as ::core::ffi::c_int - 32 as ::core::ffi::c_int
            }) as uint16;
            let y: uint16 = ((*E).base.y_pos).wrapping_sub(16u16);
            CreateSpriteAtPos(x, y, 61 as uint16, 0 as uint16, game);
        }
        AddToHiLo(
            &raw mut (*E).draygon_var_05,
            &raw mut (*E).draygon_var_09,
            0x2000 as uint32,
        );
        let mut draygon_var_0A: uint16 = 0;
        let mut v8: uint16 = 0;
        if (*E).draygon_var_05 as ::core::ffi::c_int - 160 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
            || {
                v6 = ((*E).draygon_var_0B).wrapping_sub(1u16);
                (*E).draygon_var_0B = v6;
                (*E).draygon_var_08 = ((*(&raw mut (*E).draygon_var_08 as *mut uint8).offset(0))
                    .wrapping_add(
                        *(&raw mut v6 as *mut uint8).offset(
                            (::core::mem::size_of::<uint16>())
                                .wrapping_div(::core::mem::size_of::<uint8>())
                                .wrapping_sub(1_usize) as isize,
                        ),
                    )) as uint16;
                draygon_var_0A = (*E).draygon_var_0A;
                (*E).draygon_var_0A = (draygon_var_0A).wrapping_sub(0x4000u16);
                v8 = ((((*E).draygon_var_07 as uint32) << 16 as ::core::ffi::c_int
                    | draygon_var_0A as uint32)
                    .wrapping_sub(0x4000 as uint32)
                    >> 16 as ::core::ffi::c_int) as uint16;
                (*E).draygon_var_07 = v8;
                v8 as ::core::ffi::c_int - 64 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                    != 0
            }
        {
            (*E).draygon_var_A = ((0xa59105i32) as u16);
        } else {
            Draygon_Func_40(k, game);
        }
    } else {
        (*E).draygon_var_0C = 64 as uint16;
        (*E).draygon_var_A = ((0xa590d4i32) as u16);
    };
}
pub unsafe fn Draygon_Func_26(k: uint16, game: &mut Game) {
    let v0: uint16 = k;
    let mut v3: uint16 = 0;
    Draygon_Func_40(v0, game);
    let E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
    let v2: uint16 = ((*E).draygon_var_0C).wrapping_sub(1u16);
    (*E).draygon_var_0C = v2;
    if v2 != 0 {
        if v2 == 63u16 {
            v3 = addr_kDraygon_Ilist_9AE8 as uint16;
            if (*E).draygon_var_20 != 0 {
                v3 = addr_kDraygon_Ilist_9EA1 as uint16;
            }
            let E2: *mut Enemy_Draygon = Get_Draygon(0x80 as ::core::ffi::c_int, game);
            (*E2).base.current_instruction = v3;
            (*E2).base.instruction_timer = 1 as uint16;
        }
    } else {
        (*E).draygon_var_A = ((0xa58fd6i32) as u16);
    };
}
pub unsafe fn Draygon_Func_27(k: uint16, game: &mut Game) {
    Draygon_Func_40(k, game);
    let mut v1: uint16 = addr_kDraygon_Ilist_9A68 as uint16;
    if (*Get_Draygon(k as ::core::ffi::c_int, game)).draygon_var_20 != 0 {
        v1 = addr_kDraygon_Ilist_9E21 as uint16;
    }
    let E: *mut Enemy_Draygon = Get_Draygon(0x80 as ::core::ffi::c_int, game);
    (*E).base.current_instruction = v1;
    (*E).base.instruction_timer = 1 as uint16;
    (*Get_Draygon(0 as ::core::ffi::c_int, game)).draygon_var_A = ((0xa59124i32) as u16);
}
pub unsafe fn Draygon_Func_28(k: uint16, game: &mut Game) {
    Draygon_Func_40(k, game);
}
pub unsafe fn Draygon_Func_29(game: &mut Game) {
    Samus_ReleaseFromDraygon_(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa64) as *mut uint16) = 0 as uint16;
    let E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
    (*E).draygon_var_A = ((0xa59154i32) as u16);
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        & !(kEnemyProps_Tangible as ::core::ffi::c_int)) as uint16;
    let mut v1: uint16 = addr_kDraygon_Ilist_9B5A as uint16;
    if (*E).draygon_var_20 != 0 {
        v1 = addr_kDraygon_Ilist_9F15 as uint16;
    }
    let E2: *mut Enemy_Draygon = Get_Draygon(0x80 as ::core::ffi::c_int, game);
    (*E2).base.current_instruction = v1;
    (*E2).base.instruction_timer = 1 as uint16;
}
pub unsafe fn Draygon_Func_30(game: &mut Game) {
    let mut v1: int16 = 0;
    Draygon_Func_3(game);
    let E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
    v1 = (((*E).base.y_pos) as i16).wrapping_sub(4i16);
    (*E).base.y_pos = v1 as uint16;
    if v1 < 0i16 {
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            & !(kEnemyProps_Tangible as ::core::ffi::c_int))
            as uint16;
        (*E).draygon_var_A = ((0xa587f4i32) as u16);
        (*E).draygon_var_B = 0 as uint16;
        (*E).base.x_pos = (*E).draygon_var_00;
        (*E).base.y_pos = (*E).draygon_var_01;
    }
}
pub unsafe fn Draygon_Func_31(game: &mut Game) {
    let E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
    if ((*Get_Draygon(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .base
    .frame_counter as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int)
        == 0i32
    {
        let x: uint16 = (if (*E).draygon_var_20 != 0u16 {
            (*E).base.x_pos as ::core::ffi::c_int + 32 as ::core::ffi::c_int
        } else {
            (*E).base.x_pos as ::core::ffi::c_int - 32 as ::core::ffi::c_int
        }) as uint16;
        let y: uint16 = ((*E).base.y_pos).wrapping_sub(16u16);
        CreateSpriteAtPos(x, y, 61 as uint16, 0 as uint16, game);
    }
    (*E).draygon_var_43 = ((64u8).wrapping_sub(
        (CalculateAngleFromXY(
            (64 as ::core::ffi::c_int
                - ((*E).base.x_pos as ::core::ffi::c_int >> 2 as ::core::ffi::c_int))
                as uint16,
            (120 as ::core::ffi::c_int
                - ((*E).base.y_pos as ::core::ffi::c_int >> 2 as ::core::ffi::c_int))
                as uint16,
            game,
        )) as u8,
    )) as uint16;
    let pt: Point32 = ConvertAngleToXy(
        *(&raw mut (*E).draygon_var_43 as *mut uint8).offset(0) as uint16,
        1 as uint16,
        game,
    );
    EnemyFunc_B691(
        *(&raw mut (*E).draygon_var_43 as *mut uint8).offset(0) as uint16,
        pt,
        game,
    );
    let v3: uint16 = Abs16(((*E).base.x_pos).wrapping_sub(256u16));
    if v3 as ::core::ffi::c_int - 4 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        let v4: uint16 = Abs16(((*E).base.y_pos).wrapping_sub(480u16));
        if v4 as ::core::ffi::c_int - 4 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            Draygon_Func_43(game);
            (*E).draygon_var_A = ((0xa59294i32) as u16);
            QueueMusic_Delayed8(3 as uint16, game);
            (*E).draygon_var_B = 416 as uint16;
            (*E).base.current_instruction = addr_kDraygon_Ilist_97B9 as uint16;
            (*E).base.instruction_timer = 1 as uint16;
            let v5: uint16 = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
            (*Get_Draygon(0x80 as ::core::ffi::c_int, game))
                .base
                .properties = v5;
            (*Get_Draygon(0xc0 as ::core::ffi::c_int, game))
                .base
                .properties = v5;
            let mut v6: uint16 = addr_kDraygon_Ilist_999C as uint16;
            if (*E).draygon_var_20 != 0 {
                v6 = addr_kDraygon_Ilist_9D3E as uint16;
            }
            let E1: *mut Enemy_Draygon = Get_Draygon(0x40 as ::core::ffi::c_int, game);
            (*E1).base.current_instruction = v6;
            (*E1).base.instruction_timer = 1 as uint16;
            (*E1).draygon_var_A = ((0xa5804bi32) as u16);
        }
    }
}
pub unsafe fn Draygon_Func_32(game: &mut Game) {
    Draygon_Func_34(game);
    let E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
    (*E).draygon_var_B = (*E).draygon_var_B.wrapping_sub(1);
    if (*E).draygon_var_B != 0 {
        Draygon_Func_42(0 as uint16, game);
    } else {
        (*E).draygon_var_A = ((0xa592abi32) as u16);
    };
}
pub unsafe fn Draygon_Func_33(game: &mut Game) {
    Draygon_Func_34(game);
    Draygon_Func_42(0 as uint16, game);
    let E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
    (*E).base.y_pos = (*E).base.y_pos.wrapping_add(1);
    if (*E).base.y_pos as ::core::ffi::c_int - 576 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        (*E).base.current_instruction = addr_kDraygon_Ilist_98ED as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        let v1: uint16 = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
        (*E).base.properties = v1;
        (*Get_Draygon(0x40 as ::core::ffi::c_int, game))
            .base
            .properties = v1;
        let v2: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16);
        let ref mut fresh3 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xd828)
            .offset(*((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) as isize)
            as *mut uint8 as *mut uint16);
        *fresh3 = (*fresh3) | (1u16);
        Enemy_ItemDrop_Draygon(v2, game);
        Draygon_Func_44(game);
    }
}
pub unsafe fn Draygon_Func_34(game: &mut Game) {
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x5b6) as *mut uint16) as ::core::ffi::c_int
        & 7 as ::core::ffi::c_int)
        == 0i32
    {
        let x: uint16 = ((NextRandom(game) as ::core::ffi::c_int & 0x7f as ::core::ffi::c_int)
            + 192 as ::core::ffi::c_int) as uint16;
        let y: uint16 = ((((*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16))
            & (0x3f00u16)) as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int)
            + 400 as ::core::ffi::c_int) as uint16;
        CreateSpriteAtPos(x, y, 21 as uint16, 0 as uint16, game);
    }
}
pub unsafe fn Draygon_Func_35(game: &mut Game) {
    let mut Draygon: *mut Enemy_Draygon = Get_Draygon(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if ((*Draygon).base.frame_counter as ::core::ffi::c_int & 0x7f as ::core::ffi::c_int) == 0i32 {
        Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
        CreateSpriteAtPos(
            ((*Draygon).base.x_pos).wrapping_sub(16u16),
            ((*Draygon).base.y_pos).wrapping_sub(16u16),
            24 as uint16,
            0 as uint16,
            game,
        );
    }
}
pub unsafe fn Draygon_Func_36(game: &mut Game) {
    let E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0xb5) as *mut uint16) = ((*E).draygon_var_5E
        as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16) as ::core::ffi::c_int
        - (*E).base.x_pos as ::core::ffi::c_int
        - 450 as ::core::ffi::c_int)
        as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb7) as *mut uint16) = ((*E).draygon_var_5F
        as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16) as ::core::ffi::c_int
        - (*E).base.y_pos as ::core::ffi::c_int
        - 192 as ::core::ffi::c_int)
        as uint16;
}
pub unsafe fn Draygon_Func_37(game: &mut Game) {
    let E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x8d) as *mut uint16) as ::core::ffi::c_int
        & 0x40 as ::core::ffi::c_int)
        != 0i32
    {
        Draygon_Func_39(game);
    } else {
        Draygon_Func_38(game);
    }
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x91) as *mut uint16) as ::core::ffi::c_int
        & 0x4000 as ::core::ffi::c_int)
        != 0i32
    {
        let mut v0: uint16 = addr_kDraygon_Ilist_98FE as uint16;
        if (*E).draygon_var_20 != 0 {
            v0 = addr_kDraygon_Ilist_9C90 as uint16;
        }
        (*E).base.current_instruction = v0;
        (*E).base.instruction_timer = 1 as uint16;
    } else if (*((&raw mut game.g_ram as *mut uint8).offset(0x91) as *mut uint16)
        as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        == 0i32
    {
        if (*((&raw mut game.g_ram as *mut uint8).offset(0x91) as *mut uint16)
            as ::core::ffi::c_int
            & 0x80 as ::core::ffi::c_int)
            != 0i32
        {
            let mut v4: uint16 = addr_kDraygon_Ilist_9825 as uint16;
            if (*E).draygon_var_20 != 0 {
                v4 = addr_kDraygon_Ilist_9C18 as uint16;
            }
            let E3: *mut Enemy_Draygon = Get_Draygon(0xc0 as ::core::ffi::c_int, game);
            (*E3).base.current_instruction = v4;
            (*E3).base.instruction_timer = 1 as uint16;
        }
    } else {
        let mut v2: uint16 = addr_kDraygon_Ilist_99FC as uint16;
        if (*E).draygon_var_20 != 0 {
            v2 = addr_kDraygon_Ilist_9D9E as uint16;
        }
        let E2: *mut Enemy_Draygon = Get_Draygon(0x80 as ::core::ffi::c_int, game);
        (*E2).base.current_instruction = v2;
        (*E2).base.instruction_timer = 1 as uint16;
    };
}
pub unsafe fn Draygon_Func_38(game: &mut Game) {
    let current_block: u64;
    let E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
    let v0: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x8d) as *mut uint16);
    let mut v2: uint16 = 0;
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x8d) as *mut uint16) as ::core::ffi::c_int
        & 0x200 as ::core::ffi::c_int)
        == 0i32
        || {
            (*E).base.x_pos = (*E).base.x_pos.wrapping_sub(1);
            v0 == *((&raw mut game.g_ram as *mut uint8).offset(0x1866) as *mut uint16)
        }
    {
        if (v0 as ::core::ffi::c_int & 0x100 as ::core::ffi::c_int) == 0i32 {
            current_block = 1196177745739367891;
        } else {
            (*E).base.x_pos = (*E).base.x_pos.wrapping_add(1);
            if v0 == *((&raw mut game.g_ram as *mut uint8).offset(0x1866) as *mut uint16) {
                current_block = 1196177745739367891;
            } else {
                *((&raw mut game.g_ram as *mut uint8).offset(0x1866) as *mut uint16) = v0;
                v2 = addr_kDraygon_Ilist_97D1 as uint16;
                (*E).draygon_var_20 = 1 as uint16;
                current_block = 3276175668257526147;
            }
        }
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1866) as *mut uint16) = v0;
        v2 = addr_kDraygon_Ilist_97BB as uint16;
        (*E).draygon_var_20 = 0 as uint16;
        current_block = 3276175668257526147;
    }
    match current_block {
        3276175668257526147 => {
            (*E).base.current_instruction = v2;
            (*E).base.instruction_timer = 1 as uint16;
        }
        _ => {}
    }
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x8d) as *mut uint16) as ::core::ffi::c_int
        & 0x800 as ::core::ffi::c_int)
        != 0i32
    {
        (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
    } else if (*((&raw mut game.g_ram as *mut uint8).offset(0x8d) as *mut uint16)
        as ::core::ffi::c_int
        & 0x400 as ::core::ffi::c_int)
        != 0i32
    {
        (*E).base.y_pos = (*E).base.y_pos.wrapping_add(1);
    }
}
pub unsafe fn Draygon_Func_39(game: &mut Game) {
    let current_block: u64;
    let E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
    let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x8d) as *mut uint16);
    let mut v2: uint16 = 0;
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x8d) as *mut uint16) as ::core::ffi::c_int
        & 0x200 as ::core::ffi::c_int)
        == 0i32
        || {
            (*E).base.x_pos = ((*E).base.x_pos).wrapping_sub(4u16);
            v0 = *((&raw mut game.g_ram as *mut uint8).offset(0x8d) as *mut uint16);
            *((&raw mut game.g_ram as *mut uint8).offset(0x8d) as *mut uint16)
                == *((&raw mut game.g_ram as *mut uint8).offset(0x1866) as *mut uint16)
        }
    {
        if (v0 as ::core::ffi::c_int & 0x100 as ::core::ffi::c_int) == 0i32 {
            current_block = 18201098939790280272;
        } else {
            (*E).base.x_pos = ((*E).base.x_pos).wrapping_add(4u16);
            if *((&raw mut game.g_ram as *mut uint8).offset(0x8d) as *mut uint16)
                == *((&raw mut game.g_ram as *mut uint8).offset(0x1866) as *mut uint16)
            {
                current_block = 18201098939790280272;
            } else {
                *((&raw mut game.g_ram as *mut uint8).offset(0x1866) as *mut uint16) =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x8d) as *mut uint16);
                v2 = addr_kDraygon_Ilist_97D1 as uint16;
                (*E).draygon_var_20 = 1 as uint16;
                current_block = 3276175668257526147;
            }
        }
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1866) as *mut uint16) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x8d) as *mut uint16);
        v2 = addr_kDraygon_Ilist_97BB as uint16;
        (*E).draygon_var_20 = 0 as uint16;
        current_block = 3276175668257526147;
    }
    match current_block {
        3276175668257526147 => {
            (*E).base.current_instruction = v2;
            (*E).base.instruction_timer = 1 as uint16;
        }
        _ => {}
    }
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x8d) as *mut uint16) as ::core::ffi::c_int
        & 0x800 as ::core::ffi::c_int)
        != 0i32
    {
        (*E).base.y_pos = ((*E).base.y_pos).wrapping_sub(4u16);
    } else if (*((&raw mut game.g_ram as *mut uint8).offset(0x8d) as *mut uint16)
        as ::core::ffi::c_int
        & 0x400 as ::core::ffi::c_int)
        != 0i32
    {
        (*E).base.y_pos = (*E).base.y_pos.wrapping_add(1);
        (*E).base.y_pos = ((*E).base.y_pos).wrapping_add(4u16);
    }
}
pub unsafe fn Draygon_Func_40(k: uint16, game: &mut Game) {
    let mut v1: int16 = 0;
    v1 = 8 as int16;
    if (*Get_Draygon(k as ::core::ffi::c_int, game)).draygon_var_20 == 0 {
        v1 = -(8 as ::core::ffi::c_int) as int16;
    }
    let E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) =
        ((*E).base.x_pos).wrapping_add((v1) as u16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) =
        ((*E).base.y_pos).wrapping_add(40u16);
    EnemyFunc_B7A1(game);
    if (*((&raw mut game.g_ram as *mut uint8).offset(0xa64) as *mut uint16) as ::core::ffi::c_int
        & 2 as ::core::ffi::c_int)
        != 0i32
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa64) as *mut uint16) = 0 as uint16;
        (*E).draygon_var_A = ((0xa59154i32) as u16);
    }
}
pub unsafe fn Draygon_Instr_1(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    (*Get_Draygon(0 as ::core::ffi::c_int, game))
        .base
        .current_instruction = *jp.offset(0);
    (*Get_Draygon(0x40 as ::core::ffi::c_int, game))
        .base
        .current_instruction = *jp.offset(1);
    (*Get_Draygon(0x80 as ::core::ffi::c_int, game))
        .base
        .current_instruction = *jp.offset(2);
    (*Get_Draygon(0xc0 as ::core::ffi::c_int, game))
        .base
        .current_instruction = *jp.offset(3);
    (*Get_Draygon(0 as ::core::ffi::c_int, game))
        .base
        .instruction_timer = 1 as uint16;
    (*Get_Draygon(0x40 as ::core::ffi::c_int, game))
        .base
        .instruction_timer = 1 as uint16;
    (*Get_Draygon(0x80 as ::core::ffi::c_int, game))
        .base
        .instruction_timer = 1 as uint16;
    (*Get_Draygon(0xc0 as ::core::ffi::c_int, game))
        .base
        .instruction_timer = 1 as uint16;
    jp.offset(4)
}
pub unsafe fn Draygon_Hurt(game: &mut Game) {
    let mut v0: uint16 = addr_kDraygon_MorePalettes2 as uint16;
    if ((*Get_Draygon(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .base
    .flash_timer as ::core::ffi::c_int
        & 2 as ::core::ffi::c_int)
        != 0i32
    {
        v0 = addr_word_A5A297 as uint16;
    }
    memcpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset(addr_kDraygon_BigSprmap_C08F as ::core::ffi::c_int as isize)
            .offset(17) as *mut ::core::ffi::c_void,
        RomPtr_A5(v0, game) as *const ::core::ffi::c_void,
        32 as size_t,
    );
    if ((*Get_Draygon(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .base
    .flash_timer as ::core::ffi::c_int
        & 2 as ::core::ffi::c_int)
        == 0i32
    {
        let mut v3: uint16 =
            (4u16).wrapping_mul((*Get_Draygon(0 as ::core::ffi::c_int, game)).draygon_var_0E);
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i != 8i32 {
            *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                .offset(((i >> 1 as ::core::ffi::c_int) + 89 as ::core::ffi::c_int) as isize) =
                *(game.g_rom.offset(crate::rtl::lorom_offset(0xa596af)) as *const uint8
                    as *mut uint16)
                    .offset((v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
            v3 = (v3).wrapping_add(2u16);
            i += 2 as ::core::ffi::c_int;
        }
    }
    let mut v5: uint16 = addr_kDraygon_Palette as uint16;
    if ((*Get_Draygon(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .base
    .flash_timer as ::core::ffi::c_int
        & 2 as ::core::ffi::c_int)
        != 0i32
    {
        v5 = addr_word_A5A297 as uint16;
    }
    memcpy(
        (&raw mut game.g_ram as *mut uint8)
            .offset(addr_kDraygon_BigSprmap_C11B as ::core::ffi::c_int as isize)
            .offset(197) as *mut ::core::ffi::c_void,
        RomPtr_A5(v5, game) as *const ::core::ffi::c_void,
        32 as size_t,
    );
    if (*((&raw mut game.g_ram as *mut uint8).offset(0xa64) as *mut uint16) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int)
        != 0i32
    {
        let E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
        if ((*E).base.frame_counter as ::core::ffi::c_int & 7 as ::core::ffi::c_int) == 0i32 {
            let mut v9: int16 = (((*E).base.health) as i16).wrapping_sub(256i16);
            if v9 < 0i16 {
                v9 = 0 as int16;
            }
            (*E).base.health = v9 as uint16;
            sub_A5960D(game);
        }
    }
}
pub unsafe fn Draygon_Touch(game: &mut Game) {
    NormalEnemyTouchAiSkipDeathAnim_CurEnemy(game);
    sub_A5960D(game);
}
pub unsafe fn Draygon_Shot(game: &mut Game) {
    let E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
    if (*E).draygon_var_0F as ::core::ffi::c_int - 152 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        (*E).draygon_var_0F = ((*E).draygon_var_0F).wrapping_add(8u16);
    }
    NormalEnemyShotAiSkipDeathAnim_CurEnemy(game);
    sub_A5960D(game);
}
pub unsafe fn Draygon_Powerbomb(game: &mut Game) {
    NormalEnemyPowerBombAiSkipDeathAnim_CurEnemy(game);
    sub_A5960D(game);
}
pub unsafe fn sub_A5960D(game: &mut Game) {
    if (*Get_Draygon(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .base
    .health
        != 0
    {
        Draygon_Func_41(game);
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xd32) as *mut uint16) =
            ((0x9bc8c5i32) as u16);
        let mut v0: uint16 = addr_kDraygon_Ilist_9867 as uint16;
        let E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
        if (*E).draygon_var_20 != 0 {
            v0 = addr_kDraygon_Ilist_9C5A as uint16;
        }
        (*E).base.current_instruction = v0;
        (*E).base.instruction_timer = 1 as uint16;
        let mut v3: uint16 = addr_kDraygon_Ilist_997A as uint16;
        if (*E).draygon_var_20 != 0 {
            v3 = addr_kDraygon_Ilist_9D1C as uint16;
        }
        let E1: *mut Enemy_Draygon = Get_Draygon(0x40 as ::core::ffi::c_int, game);
        (*E1).base.current_instruction = v3;
        (*E1).base.instruction_timer = 1 as uint16;
        (*E1).draygon_var_A = ((0xa5804bi32) as u16);
        (*E).draygon_var_A = ((0xa59185i32) as u16);
        Samus_ReleaseFromDraygon_(game);
        *((&raw mut game.g_ram as *mut uint8).offset(0xa64) as *mut uint16) = 0 as uint16;
        (*E).draygon_var_43 = ((64u8).wrapping_sub(
            (CalculateAngleFromXY(
                (256u16).wrapping_sub((*E).base.x_pos),
                (480u16).wrapping_sub((*E).base.y_pos),
                game,
            )) as u8,
        )) as uint16;
        let pt: Point32 = ConvertAngleToXy(
            *(&raw mut (*E).draygon_var_43 as *mut uint8).offset(0) as uint16,
            1 as uint16,
            game,
        );
        (*E).draygon_var_28 = (pt.x >> 16 as ::core::ffi::c_int) as uint16;
        (*E).draygon_var_29 = pt.x as uint16;
        (*E).draygon_var_2A = (pt.y >> 16 as ::core::ffi::c_int) as uint16;
        (*E).draygon_var_2B = pt.y as uint16;
    };
}
pub unsafe fn Draygon_Func_41(game: &mut Game) {
    let mut i: ::core::ffi::c_int = 0;
    let mut j: uint16 = 0;
    let E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
    i = 0 as ::core::ffi::c_int;
    while !((((*E).base.health) as i16).wrapping_sub(
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa596ef)) as *const uint8 as *mut uint16)
            .offset((i >> 1 as ::core::ffi::c_int) as isize)) as i16,
    ) >= 0i16)
    {
        i += 2 as ::core::ffi::c_int;
    }
    if i != (*E).draygon_var_0E as ::core::ffi::c_int {
        (*E).draygon_var_0E = i as uint16;
        let mut v2: uint16 = (4u16).wrapping_mul((*E).draygon_var_0E);
        j = 0 as uint16;
        while j != 8u16 {
            *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(
                ((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 89 as ::core::ffi::c_int)
                    as isize,
            ) = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa596af)) as *const uint8
                as *mut uint16)
                .offset((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
            v2 = (v2).wrapping_add(2u16);
            j = (j).wrapping_add(2u16);
        }
    }
}
pub unsafe fn Draygon_Instr_13(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    (*Get_Draygon(k as ::core::ffi::c_int, game)).draygon_var_A = *jp.offset(0);
    jp.offset(1)
}
unsafe fn Draygon_Func_47(game: &mut Game) -> Point16U {
    NextRandom(game);
    let E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
    let r18: uint16 = ((*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
        as ::core::ffi::c_int
        & 0x7f as ::core::ffi::c_int)
        - 64 as ::core::ffi::c_int) as uint16;
    let r20: uint16 = (((*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
        as ::core::ffi::c_int
        & 0x7f00 as ::core::ffi::c_int)
        >> 8 as ::core::ffi::c_int)
        - 64 as ::core::ffi::c_int) as uint16;
    Point16U {
        x: ((r18).wrapping_add((*E).base.x_pos)),
        y: ((r20).wrapping_add((*E).base.y_pos)),
    }
}
pub unsafe fn Draygon_Instr_8(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let pt: Point16U = Draygon_Func_47(game);
    CreateSpriteAtPos(pt.x, pt.y, 21 as uint16, 0 as uint16, game);
    jp
}
pub unsafe fn Draygon_Instr_7(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let pt: Point16U = Draygon_Func_47(game);
    CreateSpriteAtPos(pt.x, pt.y, 3 as uint16, 0 as uint16, game);
    jp
}
pub unsafe fn Draygon_Instr_6(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let pt: Point16U = Draygon_Func_47(game);
    CreateSpriteAtPos(pt.x, pt.y, 29 as uint16, 0 as uint16, game);
    jp
}
pub unsafe fn Draygon_Instr_9(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let pt: Point16U = Draygon_Func_47(game);
    CreateSpriteAtPos(pt.x, pt.y, 24 as uint16, 0 as uint16, game);
    jp
}
pub unsafe fn Draygon_Instr_2(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0xa9) as *mut uint16) = 12 as uint16;
    jp
}
pub unsafe fn Draygon_Instr_11(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let E2: *mut Enemy_Draygon = Get_Draygon(0x80 as ::core::ffi::c_int, game);
    (*E2).base.instruction_timer = 1 as uint16;
    (*E2).base.current_instruction = addr_kDraygon_Ilist_97B9 as uint16;
    let E3: *mut Enemy_Draygon = Get_Draygon(0xc0 as ::core::ffi::c_int, game);
    (*E3).base.instruction_timer = 1 as uint16;
    (*E3).base.current_instruction = addr_kDraygon_Ilist_97B9 as uint16;
    jp
}
pub unsafe fn Draygon_Instr_5(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
    jp
}
pub unsafe fn Draygon_Instr_15(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
    (*E).draygon_var_0F = 24 as uint16;
    let v3: *const uint8 = RomPtr_A0((*E).base.enemy_ptr, game);
    Samus_DealDamage(
        SuitDamageDivision(*(v3.offset(6) as *mut uint16), game),
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1840) as *mut uint16) = 32 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x183e) as *mut uint16) = 7 as uint16;
    CreateSpriteAtPos(
        *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16),
        (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)).wrapping_add(16u16),
        21 as uint16,
        0 as uint16,
        game,
    );
    jp
}
pub unsafe fn Draygon_Instr_17(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0xa9) as *mut uint16) = 12 as uint16;
    jp
}
pub unsafe fn Draygon_Instr_14(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
    (*E).draygon_var_5E = *jp.offset(0);
    (*E).draygon_var_5F = *jp.offset(1);
    jp.offset(2)
}
pub unsafe fn Draygon_Instr_16(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    (*Get_Draygon(0 as ::core::ffi::c_int, game)).draygon_var_A = *jp.offset(0);
    jp.offset(1)
}
pub unsafe fn Draygon_Instr_10(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    QueueSfx2_Max6(*jp.offset(0), game);
    jp.offset(1)
}
pub unsafe fn Draygon_Instr_4(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    QueueSfx3_Max6(*jp.offset(0), game);
    jp.offset(1)
}
pub unsafe fn Draygon_Instr_12(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
    game.eproj_spawn_pt = Point16U {
        x: (((*E).base.x_pos).wrapping_sub(28u16)),
        y: (((*E).base.y_pos).wrapping_sub(16u16)),
    };
    *((&raw mut game.g_ram as *mut uint8).offset(0x1995) as *mut uint16) =
        ((NextRandom(game) as ::core::ffi::c_int & 0x3f as ::core::ffi::c_int)
            + 128 as ::core::ffi::c_int) as uint16;
    SpawnEprojWithGfx(
        2 as uint16,
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        addr_stru_868E50 as uint16,
        game,
    );
    jp
}
pub unsafe fn Draygon_Instr_18(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
    game.eproj_spawn_pt = Point16U {
        x: (((*E).base.x_pos).wrapping_add(24u16)),
        y: (((*E).base.y_pos).wrapping_sub(16u16)),
    };
    *((&raw mut game.g_ram as *mut uint8).offset(0x1995) as *mut uint16) =
        ((NextRandom(game) as ::core::ffi::c_int & 0x3f as ::core::ffi::c_int)
            + 192 as ::core::ffi::c_int) as uint16;
    SpawnEprojWithGfx(
        2 as uint16,
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        addr_stru_868E50 as uint16,
        game,
    );
    jp
}
pub unsafe fn Draygon_Func_42(varE24: uint16, game: &mut Game) {
    let mut v0: uint16 = 62 as uint16;
    let mut i: ::core::ffi::c_int = 20 as ::core::ffi::c_int;
    while i >= 0i32 {
        let v2: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
        let v3: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        if (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa5a1df)) as *const uint8 as *mut uint16)
            .offset(v2 as isize) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int
            & 0x80 as ::core::ffi::c_int)
            != 0i32
        {
            AddToHiLo(
                ((&raw mut game.g_ram as *mut uint8).offset(0xf0f8) as *mut uint16)
                    .offset(v3 as isize) as *mut uint16,
                ((&raw mut game.g_ram as *mut uint8).offset(0xf178) as *mut uint16)
                    .offset(v3 as isize) as *mut uint16,
                *(game.g_rom.offset(crate::rtl::lorom_offset(0xa5a1af)) as *const uint8
                    as *mut uint16)
                    .offset(v2 as isize) as uint32,
            );
        } else {
            AddToHiLo(
                ((&raw mut game.g_ram as *mut uint8).offset(0xf0f8) as *mut uint16)
                    .offset(v3 as isize) as *mut uint16,
                ((&raw mut game.g_ram as *mut uint8).offset(0xf178) as *mut uint16)
                    .offset(v3 as isize) as *mut uint16,
                (-(*(game.g_rom.offset(crate::rtl::lorom_offset(0xa5a1af)) as *const uint8
                    as *mut uint16)
                    .offset(v2 as isize) as ::core::ffi::c_int)
                    - ((varE24 as ::core::ffi::c_int) << 16 as ::core::ffi::c_int))
                    as uint32,
            );
        }
        let v8: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xf278) as *mut uint16)
            .offset(v3 as isize);
        if (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa5a1df)) as *const uint8 as *mut uint16)
            .offset(v2 as isize) as ::core::ffi::c_int
            + 128 as ::core::ffi::c_int
            & 0x80 as ::core::ffi::c_int)
            != 0i32
        {
            AddToHiLo(
                ((&raw mut game.g_ram as *mut uint8).offset(0xf1f8) as *mut uint16)
                    .offset(v3 as isize) as *mut uint16,
                ((&raw mut game.g_ram as *mut uint8).offset(0xf278) as *mut uint16)
                    .offset(v3 as isize) as *mut uint16,
                *(game.g_rom.offset(crate::rtl::lorom_offset(0xa5a1af)) as *const uint8
                    as *mut uint16)
                    .offset((v2 + 1 as ::core::ffi::c_int) as isize) as uint32,
            );
        } else {
            AddToHiLo(
                ((&raw mut game.g_ram as *mut uint8).offset(0xf1f8) as *mut uint16)
                    .offset(v3 as isize) as *mut uint16,
                ((&raw mut game.g_ram as *mut uint8).offset(0xf278) as *mut uint16)
                    .offset(v3 as isize) as *mut uint16,
                -(*(game.g_rom.offset(crate::rtl::lorom_offset(0xa5a1af)) as *const uint8
                    as *mut uint16)
                    .offset((v2 + 1 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int) as uint32,
            );
        }
        v0 = (v0).wrapping_sub(2u16);
        i -= 4 as ::core::ffi::c_int;
    }
}
pub unsafe fn Draygon_Func_43(game: &mut Game) {
    let mut v1: int16 = 0;
    let mut j: int16 = 0;
    let mut i: ::core::ffi::c_int = 62 as ::core::ffi::c_int;
    while i >= 0i32 {
        *((&raw mut game.g_ram as *mut uint8).offset(0xef78) as *mut uint16)
            .offset((i >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
    v1 = 2 as int16;
    let mut v2: uint16 = 20 as uint16;
    loop {
        let v3: ::core::ffi::c_int = v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        CreateSpriteAtPos(
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xa5a1c7)) as *const uint8 as *mut uint16)
                .offset(v3 as isize),
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xa5a1c7)) as *const uint8 as *mut uint16)
                .offset((v3 + 1 as ::core::ffi::c_int) as isize),
            59 as uint16,
            0xe00 as uint16,
            game,
        );
        v2 = (v2).wrapping_sub(4u16);
        v1 -= 1;
        if !(v1 >= 0i16) {
            break;
        }
    }
    j = 2 as int16;
    while j >= 0i16 {
        let v5: ::core::ffi::c_int = v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        CreateSpriteAtPos(
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xa5a1c7)) as *const uint8 as *mut uint16)
                .offset(v5 as isize),
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xa5a1c7)) as *const uint8 as *mut uint16)
                .offset((v5 + 1 as ::core::ffi::c_int) as isize),
            60 as uint16,
            0xe00 as uint16,
            game,
        );
        v2 = (v2).wrapping_sub(4u16);
        j -= 1;
    }
}
pub unsafe fn Draygon_Func_44(game: &mut Game) {
    let mut i: ::core::ffi::c_int = 62 as ::core::ffi::c_int;
    while i >= 0i32 {
        *((&raw mut game.g_ram as *mut uint8).offset(0xef78) as *mut uint16)
            .offset((i >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
}
pub unsafe fn Draygon_Func_45(game: &mut Game) {
    CreateSpriteAtPos(
        16 as uint16,
        384 as uint16,
        59 as uint16,
        0xe00 as uint16,
        game,
    );
    CreateSpriteAtPos(
        16 as uint16,
        384 as uint16,
        59 as uint16,
        0xe00 as uint16,
        game,
    );
    CreateSpriteAtPos(
        16 as uint16,
        384 as uint16,
        59 as uint16,
        0xe00 as uint16,
        game,
    );
    CreateSpriteAtPos(
        16 as uint16,
        384 as uint16,
        59 as uint16,
        0xe00 as uint16,
        game,
    );
}
pub unsafe fn Draygon_Func_46(game: &mut Game) {
    let mut v0: uint16 = 62 as uint16;
    loop {
        let v1: uint16 = ((*Get_Draygon(0 as ::core::ffi::c_int, game)).draygon_var_46)
            .wrapping_add(
                *(game.g_rom.offset(crate::rtl::lorom_offset(0xa5a19f)) as *const uint8
                    as *mut uint16)
                    .offset(
                        (((v0).wrapping_sub(56u16)) as ::core::ffi::c_int
                            >> 1 as ::core::ffi::c_int) as isize,
                    ),
            );
        if (v1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32 {
            let v2: uint16 = v1;
            if *((game.g_rom.offset(crate::rtl::lorom_offset(0xa5ce07)) as *const uint8
                as *mut uint8)
                .offset(v1 as isize) as *mut uint8 as *mut uint16)
                == 0x8080u16
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0xef78) as *mut uint16)
                    .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                    0 as uint16;
            } else {
                let v3: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
                let ref mut fresh1 = *((&raw mut game.g_ram as *mut uint8).offset(0xf0f8)
                    as *mut uint16)
                    .offset(v3 as isize);
                *fresh1 = (*fresh1).wrapping_add(SignExtend8(
                    *(game.g_rom.offset(crate::rtl::lorom_offset(0xa5ce07)) as *const uint8
                        as *mut uint8)
                        .offset(v1 as isize) as uint16,
                ));
                let ref mut fresh2 = *((&raw mut game.g_ram as *mut uint8).offset(0xf1f8)
                    as *mut uint16)
                    .offset(v3 as isize);
                *fresh2 = (*fresh2).wrapping_add(SignExtend8(
                    *(game.g_rom.offset(crate::rtl::lorom_offset(0xa5ce07)) as *const uint8
                        as *mut uint8)
                        .offset((v2 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize)
                        as uint16,
                ));
            }
        }
        v0 = (v0).wrapping_sub(2u16);
        if !(((v0) as i16).wrapping_sub(56i16) >= 0i16) {
            break;
        }
    }
    let E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
    (*E).draygon_var_46 = ((*E).draygon_var_46).wrapping_add(4u16);
}
pub unsafe fn DraygonsEye_Init(game: &mut Game) {
    let E: *mut Enemy_Draygon = Get_Draygon(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.current_instruction = addr_kDraygon_Ilist_9944 as uint16;
    (*E).draygon_var_A = ((0xa5804bi32) as u16);
}
pub unsafe fn Draygon_Instr_3(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    (*Get_Draygon(0x40 as ::core::ffi::c_int, game)).draygon_var_A = *jp.offset(0);
    jp.offset(1)
}
pub unsafe fn DraygonsEye_Main(game: &mut Game) {
    let E: *mut Enemy_Draygon = Get_Draygon(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallDraygonFunc(
        (((*E).draygon_var_A) as u32) | (0xa50000u32),
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
}
pub unsafe fn Draygon_Func_48(k: uint16, game: &mut Game) {
    if ((*Get_Draygon(k as ::core::ffi::c_int, game))
        .base
        .frame_counter as ::core::ffi::c_int
        & 0x7f as ::core::ffi::c_int)
        == 0i32
    {
        let E1: *mut Enemy_Draygon = Get_Draygon(0x40 as ::core::ffi::c_int, game);
        game.eproj_spawn_pt = Point16U {
            x: (((*E1).base.x_pos).wrapping_sub(24u16)),
            y: (((*E1).base.y_pos).wrapping_sub(32u16)),
        };
        SpawnEprojWithRoomGfx(
            addr_kEproj_DustCloudExplosion as uint16,
            0x18 as uint16,
            game,
        );
    }
    let E0: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
    let v4: uint16 = CalculateAngleFromXY(
        (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) as ::core::ffi::c_int
            - ((*E0).base.x_pos as ::core::ffi::c_int - 24 as ::core::ffi::c_int))
            as uint16,
        (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) as ::core::ffi::c_int
            - ((*E0).base.y_pos as ::core::ffi::c_int - 32 as ::core::ffi::c_int))
            as uint16,
        game,
    );
    let E: *mut Enemy_Draygon = Get_Draygon(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if v4 != (*E).draygon_var_F {
        let mut v6: uint16 = addr_kDraygon_Ilist_99BA as uint16;
        if v4 as ::core::ffi::c_int - 32 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
            v6 = addr_kDraygon_Ilist_99B4 as uint16;
            if v4 as ::core::ffi::c_int - 96 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == 0
            {
                v6 = addr_kDraygon_Ilist_99C0 as uint16;
                if v4 as ::core::ffi::c_int - 160 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    == 0
                {
                    v6 = addr_kDraygon_Ilist_99AE as uint16;
                    if v4 as ::core::ffi::c_int - 224 as ::core::ffi::c_int
                        & 0x8000 as ::core::ffi::c_int
                        == 0
                    {
                        v6 = addr_kDraygon_Ilist_99BA as uint16;
                    }
                }
            }
        }
        (*E).draygon_var_F = v4;
        (*E).base.current_instruction = v6;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
    }
}
pub unsafe fn Draygon_Func_49(k: uint16, game: &mut Game) {
    let E0: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
    if ((*Get_Draygon(k as ::core::ffi::c_int, game))
        .base
        .frame_counter as ::core::ffi::c_int
        & 0x7f as ::core::ffi::c_int)
        == 0i32
    {
        game.eproj_spawn_pt = Point16U {
            x: (((*E0).base.x_pos).wrapping_add(24u16)),
            y: (((*E0).base.y_pos).wrapping_sub(32u16)),
        };
        SpawnEprojWithRoomGfx(
            addr_kEproj_DustCloudExplosion as uint16,
            0x18 as uint16,
            game,
        );
    }
    let v4: uint16 = CalculateAngleFromXY(
        (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) as ::core::ffi::c_int
            - ((*E0).base.x_pos as ::core::ffi::c_int + 24 as ::core::ffi::c_int))
            as uint16,
        (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) as ::core::ffi::c_int
            - ((*E0).base.y_pos as ::core::ffi::c_int - 32 as ::core::ffi::c_int))
            as uint16,
        game,
    );
    let E: *mut Enemy_Draygon = Get_Draygon(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if v4 != (*E).draygon_var_F {
        let mut v6: uint16 = addr_kDraygon_Ilist_9D5C as uint16;
        if v4 as ::core::ffi::c_int - 32 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
            v6 = addr_kDraygon_Ilist_9D50 as uint16;
            if v4 as ::core::ffi::c_int - 96 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == 0
            {
                v6 = addr_kDraygon_Ilist_9D62 as uint16;
                if v4 as ::core::ffi::c_int - 160 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    == 0
                {
                    v6 = addr_kDraygon_Ilist_9D56 as uint16;
                    if v4 as ::core::ffi::c_int - 224 as ::core::ffi::c_int
                        & 0x8000 as ::core::ffi::c_int
                        == 0
                    {
                        v6 = addr_kDraygon_Ilist_9D5C as uint16;
                    }
                }
            }
        }
        (*E).draygon_var_F = v4;
        (*E).base.current_instruction = v6;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
    }
}
pub unsafe fn DraygonsTail_Init(game: &mut Game) {
    let v0: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*v0).current_instruction = addr_kDraygon_Ilist_99FC as uint16;
    (*v0).palette_index = 0xe00 as uint16;
}
pub unsafe fn DraygonsArms_Init(game: &mut Game) {
    let v0: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*v0).current_instruction = addr_kDraygon_Ilist_97E7 as uint16;
    (*v0).palette_index = 0xe00 as uint16;
    (*v0).layer = 2 as uint16;
}
pub unsafe fn Draygon_Instr_25(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
    if (*E).draygon_var_0B as ::core::ffi::c_int - 40 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        (*E).draygon_var_0B = ((*E).draygon_var_0B).wrapping_add(8u16);
    }
    jp
}
pub unsafe fn Draygon_Instr_24(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    (*Get_Draygon(0 as ::core::ffi::c_int, game)).draygon_var_2F = 0 as uint16;
    jp
}
pub unsafe fn Draygon_Func_50(j: uint16, game: &mut Game) {
    let v1: *const uint8 = RomPtr_A5(j, game);
    let E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
    (*E).draygon_var_0B = *(v1 as *mut uint16);
    (*E).draygon_var_0C = *(v1.offset(2) as *mut uint16);
    (*E).draygon_var_0A = *(v1.offset(4) as *mut uint16);
}
pub unsafe fn Draygon_Instr_21(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
    (*E).draygon_var_0B = *jp.offset(0);
    (*E).draygon_var_0C = *jp.offset(1);
    jp.offset(2)
}
pub unsafe fn Draygon_Func_51(j: uint16, game: &mut Game) {
    let v1: uint16 = *(RomPtr_A5(j, game) as *mut uint16);
    (*Get_Draygon(0 as ::core::ffi::c_int, game)).draygon_var_0B = v1;
}
pub unsafe fn Draygon_Func_52(j: uint16, game: &mut Game) {
    let v1: uint16 = *(RomPtr_A5(j, game) as *mut uint16);
    (*Get_Draygon(0 as ::core::ffi::c_int, game)).draygon_var_0C = v1;
}
pub unsafe fn Draygon_Instr_53(k: uint16, j: uint16, game: &mut Game) -> uint16 {
    let E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
    (*E).draygon_var_0B = ((*E).draygon_var_0B).wrapping_add(*(RomPtr_A5(j, game) as *mut uint16));
    (j).wrapping_add(2u16)
}
pub unsafe fn Draygon_Instr_22(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x9000) as *mut uint16) = *jp.offset(0);
    jp.offset(1)
}
pub unsafe fn Draygon_Instr_27(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
    (*E).base.x_pos = 128 as uint16;
    (*E).base.y_pos = 624 as uint16;
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        & 0x5bff as ::core::ffi::c_int
        | 0xa000 as ::core::ffi::c_int) as uint16;
    jp
}
pub unsafe fn Draygon_Instr_23(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    QueueSfx2_Max6(*jp.offset(0), game);
    jp.offset(1)
}
pub unsafe fn Draygon_Instr_30(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    Enemy_ItemDrop_SporeSpawn(k, game);
    jp
}
pub unsafe fn Draygon_Instr_20(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    (*Get_Draygon(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .draygon_var_A = *jp.offset(0);
    jp.offset(1)
}
pub unsafe fn Draygon_Instr_29(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let v2: uint16 = *jp.offset(0);
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i != 32i32 {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
            .offset(((i >> 1 as ::core::ffi::c_int) + 144 as ::core::ffi::c_int) as isize) =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xa5e3f9)) as *const uint8 as *mut uint16)
                .offset((v2 as ::core::ffi::c_int + i >> 1 as ::core::ffi::c_int) as isize);
        i += 2 as ::core::ffi::c_int;
    }
    let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_0 != 32i32 {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
            .offset(((i_0 >> 1 as ::core::ffi::c_int) + 64 as ::core::ffi::c_int) as isize) =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xa5e4f9)) as *const uint8 as *mut uint16)
                .offset((v2 as ::core::ffi::c_int + i_0 >> 1 as ::core::ffi::c_int) as isize);
        i_0 += 2 as ::core::ffi::c_int;
    }
    let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_1 != 32i32 {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
            .offset(((i_1 >> 1 as ::core::ffi::c_int) + 112 as ::core::ffi::c_int) as isize) =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xa5e5d9)) as *const uint8 as *mut uint16)
                .offset((v2 as ::core::ffi::c_int + i_1 >> 1 as ::core::ffi::c_int) as isize);
        i_1 += 2 as ::core::ffi::c_int;
    }
    jp.offset(1)
}
pub unsafe fn Draygon_Instr_19(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let v2: uint16 = *jp.offset(0);
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i != 32i32 {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
            .offset(((i >> 1 as ::core::ffi::c_int) + 144 as ::core::ffi::c_int) as isize) =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xa5e3f9)) as *const uint8 as *mut uint16)
                .offset((v2 as ::core::ffi::c_int + i >> 1 as ::core::ffi::c_int) as isize);
        i += 2 as ::core::ffi::c_int;
    }
    let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_0 != 32i32 {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
            .offset(((i_0 >> 1 as ::core::ffi::c_int) + 64 as ::core::ffi::c_int) as isize) =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xa5e4f9)) as *const uint8 as *mut uint16)
                .offset((v2 as ::core::ffi::c_int + i_0 >> 1 as ::core::ffi::c_int) as isize);
        i_0 += 2 as ::core::ffi::c_int;
    }
    let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_1 != 32i32 {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
            .offset(((i_1 >> 1 as ::core::ffi::c_int) + 112 as ::core::ffi::c_int) as isize) =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xa5e5d9)) as *const uint8 as *mut uint16)
                .offset((v2 as ::core::ffi::c_int + i_1 >> 1 as ::core::ffi::c_int) as isize);
        i_1 += 2 as ::core::ffi::c_int;
    }
    jp.offset(1)
}
pub unsafe fn Draygon_Instr_28(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    NextRandom(game);
    let mut r18: uint16 = ((*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
        as ::core::ffi::c_int
        & 0x7f as ::core::ffi::c_int)
        - 64 as ::core::ffi::c_int) as uint16;
    let E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
    r18 = (r18).wrapping_add((*E).base.x_pos);
    let mut r20: uint16 = ((((*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16))
        & (0x7f00u16)) as ::core::ffi::c_int
        >> 8 as ::core::ffi::c_int)
        - 64 as ::core::ffi::c_int) as uint16;
    r20 = (r20).wrapping_add((*E).base.y_pos);
    game.eproj_spawn_pt = Point16U { x: r18, y: r20 };
    SpawnEprojWithRoomGfx(0xe509 as uint16, 0x15 as uint16, game);
    QueueSfx2_Max6(0x29 as uint16, game);
    jp
}
pub unsafe fn Draygon_Instr_26(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    NextRandom(game);
    let E: *mut Enemy_Draygon = Get_Draygon(0 as ::core::ffi::c_int, game);
    let x: uint16 = ((*E).base.x_pos as ::core::ffi::c_int
        + (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
            as ::core::ffi::c_int
            & 0x7f as ::core::ffi::c_int)
        - 64 as ::core::ffi::c_int) as uint16;
    let y: uint16 = ((*E).base.y_pos as ::core::ffi::c_int
        + (((*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)) & (0x3f00u16))
            as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int)
        - 32 as ::core::ffi::c_int) as uint16;
    CreateSpriteAtPos(x, y, 3 as uint16, 0 as uint16, game);
    QueueSfx2_Max6(0x25 as uint16, game);
    jp
}
pub unsafe fn sub_A5E9F5(game: &mut Game) {
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x5b6) as *mut uint16) as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int)
        == 0i32
    {
        NextRandom(game);
        let x: uint16 = ((*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
            as ::core::ffi::c_int
            & 0x3f as ::core::ffi::c_int)
            + 96 as ::core::ffi::c_int) as uint16;
        let y: uint16 = ((((*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16))
            & (0xf00u16)) as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int)
            + 480 as ::core::ffi::c_int) as uint16;
        CreateSpriteAtPos(x, y, 21 as uint16, 0 as uint16, game);
    }
}
pub unsafe fn SporeSpawn_Init(game: &mut Game) {
    let mut v0: uint16 = 0 as uint16;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i != 32i32 {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
            .offset(((i >> 1 as ::core::ffi::c_int) + 240 as ::core::ffi::c_int) as isize) =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xa5e359)) as *const uint8 as *mut uint16)
                .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        v0 = (v0).wrapping_add(2u16);
        i += 2 as ::core::ffi::c_int;
    }
    SpawnEprojWithGfx(
        0 as uint16,
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        addr_kEproj_SporeSpawnsStalk as uint16,
        game,
    );
    SpawnEprojWithGfx(
        1 as uint16,
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        addr_kEproj_SporeSpawnsStalk as uint16,
        game,
    );
    SpawnEprojWithGfx(
        2 as uint16,
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        addr_kEproj_SporeSpawnsStalk as uint16,
        game,
    );
    SpawnEprojWithGfx(
        3 as uint16,
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        addr_kEproj_SporeSpawnsStalk as uint16,
        game,
    );
    let E: *mut Enemy_SporeSpawn = Get_SporeSpawn(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let E0: *mut Enemy_SporeSpawn = Get_SporeSpawn(0 as ::core::ffi::c_int, game);
    (*E).ssn_var_04 = (*E).base.x_pos;
    (*E).ssn_var_05 = ((*E).base.y_pos).wrapping_sub(72u16);
    (*E).ssn_var_C = (*E).base.x_pos;
    (*E).ssn_var_D = (*E).base.y_pos;
    (*E).ssn_var_F = 0 as uint16;
    if (*(&raw mut game.g_ram as *mut uint8)
        .offset(0xd828)
        .offset(*((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) as isize)
        as ::core::ffi::c_int
        & 2 as ::core::ffi::c_int)
        != 0i32
    {
        (*E0).base.current_instruction = addr_kDraygon_Ilist_E6B9 as uint16;
        (*E0).ssn_var_A = ((0xa5eb1ai32) as u16);
        (*E0).base.properties = ((*E0).base.properties) | (0x8000u16);
        SporeSpawn_Func_5(game);
        SpawnHardcodedPlm(
            SpawnHardcodedPlmArgs {
                field_0: 0x7 as uint8,
                field_1: 0x1e as uint8,
                field_2: 0xb793 as VoidP,
            },
            game,
        );
        *((&raw mut game.g_ram as *mut uint8).offset(0x7e9) as *mut uint16) = 0 as uint16;
    } else {
        (*E0).base.current_instruction = addr_kDraygon_Ilist_E6C7 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x183c) as *mut uint16) =
            -(1 as ::core::ffi::c_int) as uint16;
        let v5: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16);
        (*E).ssn_var_A = ((0xa5eb1ai32) as u16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x7e9) as *mut uint16) =
            ((0x909589i32) as u16);
        (*E).base.y_pos = ((*E).base.y_pos).wrapping_sub(128u16);
        SpawnEprojWithGfx(0 as uint16, v5, addr_kEproj_SporeSpawners as uint16, game);
        SpawnEprojWithGfx(1 as uint16, v5, addr_kEproj_SporeSpawners as uint16, game);
        SpawnEprojWithGfx(2 as uint16, v5, addr_kEproj_SporeSpawners as uint16, game);
        SpawnEprojWithGfx(3 as uint16, v5, addr_kEproj_SporeSpawners as uint16, game);
        SporeSpawn_Func_5(game);
    };
}
pub unsafe fn CallSporeSpawnFunc(ea: uint32, game: &mut Game) {
    match ea {
        10873626 => (),
        10873627 => {
            SporeSpawn_Func_1(game);
        }
        10873682 => {
            SporeSpawn_Func_2(
                *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
                game,
            );
        }
        10873755 => {
            SporeSpawn_Func_3(game);
        }
        10873838 => {
            SporeSpawn_Func_4(game);
        }
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn SporeSpawn_Main(game: &mut Game) {
    let E: *mut Enemy_SporeSpawn = Get_SporeSpawn(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallSporeSpawnFunc((((*E).ssn_var_A) as u32) | (0xa50000u32), game);
}
pub unsafe fn SporeSpawn_Func_1(game: &mut Game) {
    SporeSpawn_Func_5(game);
    let E: *mut Enemy_SporeSpawn = Get_SporeSpawn(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let v1: uint16 = ((*E).base.y_pos).wrapping_add(1u16);
    (*E).base.y_pos = v1;
    if v1 as ::core::ffi::c_int - 624 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
        (*E).base.current_instruction = addr_kDraygon_Ilist_E6D5 as uint16;
        (*E).base.instruction_timer = 1 as uint16;
    }
    let E0: *mut Enemy_SporeSpawn = Get_SporeSpawn(0 as ::core::ffi::c_int, game);
    (*E0).ssn_var_0B = 48 as uint16;
    (*E0).ssn_var_0C = 1 as uint16;
    (*E0).ssn_var_0A = 192 as uint16;
}
pub unsafe fn SporeSpawn_Func_2(k: uint16, game: &mut Game) {
    SporeSpawn_Func_5(game);
    let E0: *mut Enemy_SporeSpawn = Get_SporeSpawn(0 as ::core::ffi::c_int, game);
    let E: *mut Enemy_SporeSpawn = Get_SporeSpawn(k as ::core::ffi::c_int, game);
    (*E).base.x_pos =
        ((*E).ssn_var_C).wrapping_add(CosineMult8bit((*E0).ssn_var_0A, (*E0).ssn_var_0B, game));
    (*E).base.y_pos = ((*E).ssn_var_D).wrapping_add(SineMult8bit(
        (2 as ::core::ffi::c_int
            * ((*E0).ssn_var_0A as ::core::ffi::c_int - 64 as ::core::ffi::c_int))
            as uint16,
        ((*E0).ssn_var_0B).wrapping_sub(16u16),
        game,
    ));
    (*E0).ssn_var_0A = ((*(&raw mut (*E0).ssn_var_0C as *mut uint8).offset(0))
        .wrapping_add(((*E0).ssn_var_0A) as u8)) as uint16;
}
pub unsafe fn SporeSpawn_Func_3(game: &mut Game) {
    let E0: *mut Enemy_SporeSpawn = Get_SporeSpawn(0 as ::core::ffi::c_int, game);
    (*E0).ssn_var_43 = ((64u8).wrapping_sub(
        (CalculateAngleFromXY(
            (128u16).wrapping_sub((*E0).base.x_pos),
            (624u16).wrapping_sub((*E0).base.y_pos),
            game,
        )) as u8,
    )) as uint16;
    let E: *mut Enemy_SporeSpawn = Get_SporeSpawn(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let pt: Point32 = ConvertAngleToXy(
        *(&raw mut (*E0).ssn_var_43 as *mut uint8).offset(0) as uint16,
        1 as uint16,
        game,
    );
    (*E).ssn_var_28 = (pt.x >> 16 as ::core::ffi::c_int) as uint16;
    (*E).ssn_var_29 = pt.x as uint16;
    (*E).ssn_var_2A = (pt.y >> 16 as ::core::ffi::c_int) as uint16;
    (*E).ssn_var_2B = pt.y as uint16;
}
pub unsafe fn SporeSpawn_Func_4(game: &mut Game) {
    let E: *mut Enemy_SporeSpawn = Get_SporeSpawn(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let E0: *mut Enemy_SporeSpawn = Get_SporeSpawn(0 as ::core::ffi::c_int, game);
    let varE20: uint16 = *(&raw mut (*E0).ssn_var_43 as *mut uint8).offset(0) as uint16;
    EnemyFunc_B691(
        varE20,
        Point32 {
            x: ((*E).ssn_var_28 as uint32) << 16 as ::core::ffi::c_int | (*E).ssn_var_29 as uint32,
            y: ((*E).ssn_var_2A as uint32) << 16 as ::core::ffi::c_int | (*E).ssn_var_2B as uint32,
        },
        game,
    );
    let v2: uint16 = Abs16(((*E0).base.x_pos).wrapping_sub(128u16));
    if v2 as ::core::ffi::c_int - 8 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        let v3: uint16 = Abs16(((*E0).base.y_pos).wrapping_sub(624u16));
        if v3 as ::core::ffi::c_int - 8 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            (*E0).ssn_var_A = ((0xa5eb1ai32) as u16);
        }
    }
    SporeSpawn_Func_5(game);
    sub_A5E9F5(game);
}
pub unsafe fn SporeSpawn_Func_5(game: &mut Game) {
    let E: *mut Enemy_SporeSpawn = Get_SporeSpawn(0 as ::core::ffi::c_int, game);
    let v1: uint16 = ((*E).base.x_pos).wrapping_sub((*E).ssn_var_04);
    let mut v2: uint16 = 0;
    let mut r18: uint16 = 0;
    let mut r20: uint16 = 0;
    let mut r22: uint16 = 0;
    if (v1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32 {
        r18 = (v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
        r20 = (v1 as ::core::ffi::c_int >> 2 as ::core::ffi::c_int) as uint16;
        r22 = ((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
            + (v1 as ::core::ffi::c_int >> 2 as ::core::ffi::c_int)) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(14) =
            128 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(15) =
            ((*E).ssn_var_04 as ::core::ffi::c_int
                + (v1 as ::core::ffi::c_int >> 2 as ::core::ffi::c_int)) as uint16;
        (*E).ssn_var_20 =
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(15);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(16) =
            ((*E).ssn_var_04).wrapping_add(r18);
        (*E).ssn_var_21 =
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(16);
        v2 = ((*E).ssn_var_04).wrapping_add(r22);
    } else {
        r18 = ((((*E).ssn_var_04).wrapping_sub((*E).base.x_pos)) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int) as uint16;
        r20 = (-(v1 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
            >> 2 as ::core::ffi::c_int) as uint16;
        r22 = (r18).wrapping_add(r20);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(14) =
            128 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(15) =
            ((*E).ssn_var_04).wrapping_sub(r20);
        (*E).ssn_var_20 =
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(15);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(16) =
            ((*E).ssn_var_04).wrapping_sub(r18);
        (*E).ssn_var_21 =
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(16);
        v2 = ((*E).ssn_var_04).wrapping_sub(r22);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(17) = v2;
    (*E).ssn_var_22 = v2;
    let v3: uint16 = ((*E).base.y_pos as ::core::ffi::c_int
        - 40 as ::core::ffi::c_int
        - (*E).ssn_var_05 as ::core::ffi::c_int) as uint16;
    let mut v4: uint16 = 0;
    if (v3 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32 {
        r18 = (v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
        r20 = (v3 as ::core::ffi::c_int >> 2 as ::core::ffi::c_int) as uint16;
        r22 = ((v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
            + (v3 as ::core::ffi::c_int >> 2 as ::core::ffi::c_int)) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(14) =
            560 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(15) =
            ((*E).ssn_var_05 as ::core::ffi::c_int
                + (v3 as ::core::ffi::c_int >> 2 as ::core::ffi::c_int)) as uint16;
        (*E).ssn_var_23 =
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(15);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(16) =
            ((*E).ssn_var_05).wrapping_add(r18);
        (*E).ssn_var_24 =
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(16);
        v4 = ((*E).ssn_var_05).wrapping_add(r22);
    } else {
        r18 = (((*E).ssn_var_05 as ::core::ffi::c_int
            - ((*E).base.y_pos as ::core::ffi::c_int - 40 as ::core::ffi::c_int))
            as uint16 as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int) as uint16;
        r20 = (-(v3 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
            >> 2 as ::core::ffi::c_int) as uint16;
        r22 = (r18).wrapping_add(r20);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(14) =
            560 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(15) =
            ((*E).ssn_var_05).wrapping_sub(r20);
        (*E).ssn_var_23 =
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(15);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(16) =
            ((*E).ssn_var_05).wrapping_sub(r18);
        (*E).ssn_var_24 =
            *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(16);
        v4 = ((*E).ssn_var_05).wrapping_sub(r22);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(17) = v4;
    (*E).ssn_var_25 = v4;
}
pub unsafe fn SporeSpawn_Shot(game: &mut Game) {
    let mut v4: int16 = 0;
    let mut v0: uint16 =
        (2u16).wrapping_mul(*((&raw mut game.g_ram as *mut uint8).offset(0x18a6) as *mut uint16));
    let v1: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xc18) as *mut uint16)
        .offset(*((&raw mut game.g_ram as *mut uint8).offset(0x18a6) as *mut uint16) as isize);
    if (v1 as ::core::ffi::c_int & 0x700 as ::core::ffi::c_int) != 0i32
        || (v1 as ::core::ffi::c_int & 0x10 as ::core::ffi::c_int) != 0i32
    {
        EnemyFunc_A6B4_UsedBySporeSpawn(game);
        v0 = *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16);
        let EK: *mut Enemy_SporeSpawn = Get_SporeSpawn(v0 as ::core::ffi::c_int, game);
        if (*EK).base.flash_timer != 0 {
            let E: *mut Enemy_SporeSpawn = Get_SporeSpawn(0 as ::core::ffi::c_int, game);
            (*EK).ssn_var_A = ((0xa5eb52i32) as u16);
            v4 = 2 as int16;
            if (*EK).base.health as ::core::ffi::c_int - 400 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
            {
                if ((*E).ssn_var_0C as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
                    v4 = -(2 as ::core::ffi::c_int) as int16;
                }
                (*E).ssn_var_0C = v4 as uint16;
            }
            if (*E).ssn_var_2F == 0 {
                (*E).ssn_var_0C = -((*E).ssn_var_0C as ::core::ffi::c_int) as uint16;
                (*E).ssn_var_2F = 1 as uint16;
                (*EK).base.current_instruction = addr_stru_A5E729 as uint16;
                (*EK).base.instruction_timer = 1 as uint16;
                let mut v7: uint16 = 96 as uint16;
                let health: uint16 = (*EK).base.health;
                if health as ::core::ffi::c_int - 70 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    == 0
                {
                    v7 = 64 as uint16;
                    if health as ::core::ffi::c_int - 410 as ::core::ffi::c_int
                        & 0x8000 as ::core::ffi::c_int
                        == 0
                    {
                        v7 = 32 as uint16;
                        if health as ::core::ffi::c_int - 770 as ::core::ffi::c_int
                            & 0x8000 as ::core::ffi::c_int
                            == 0
                        {
                            v7 = 0 as uint16;
                        }
                    }
                }
                if health != (*E).ssn_var_40 {
                    (*E).ssn_var_40 = health;
                    SporeSpawn_Func_7(v7, game);
                }
            }
        }
        SporeSpawn_Func_6(game);
    }
}
pub unsafe fn SporeSpawn_Touch(game: &mut Game) {
    NormalEnemyTouchAiSkipDeathAnim_CurEnemy(game);
    SporeSpawn_Func_6(game);
}
pub unsafe fn SporeSpawn_Func_6(game: &mut Game) {
    if (*Get_SporeSpawn(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .base
    .health
        == 0
    {
        let E: *mut Enemy_SporeSpawn = Get_SporeSpawn(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        );
        let E0: *mut Enemy_SporeSpawn = Get_SporeSpawn(0 as ::core::ffi::c_int, game);
        (*E0).ssn_var_0E = 0 as uint16;
        (*E).base.invincibility_timer = 0 as uint16;
        (*E).base.flash_timer = 0 as uint16;
        (*E).base.ai_handler_bits = 0 as uint16;
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
        let mut i: ::core::ffi::c_int = 26 as ::core::ffi::c_int;
        while i >= 0i32 {
            *(((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16 as *mut uint8)
                .offset(i as uint16 as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
            i -= 2 as ::core::ffi::c_int;
        }
        (*E0).base.current_instruction = addr_kDraygon_Ilist_E77D as uint16;
        (*E0).base.instruction_timer = 1 as uint16;
        let ref mut fresh5 = *((&raw mut game.g_ram as *mut uint8)
            .offset(0xd828)
            .offset(*((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) as isize)
            as *mut uint8 as *mut uint16);
        *fresh5 = (*fresh5) | (2u16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x7e9) as *mut uint16) = 0 as uint16;
        SpawnHardcodedPlm(
            SpawnHardcodedPlmArgs {
                field_0: 0x7 as uint8,
                field_1: 0x1e as uint8,
                field_2: 0xb78f as VoidP,
            },
            game,
        );
    }
}
pub unsafe fn SporeSpawn_Func_7(a: uint16, game: &mut Game) {
    let mut v1: uint16 = a;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i != 32i32 {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
            .offset(((i >> 1 as ::core::ffi::c_int) + 144 as ::core::ffi::c_int) as isize) =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xa5e379)) as *const uint8 as *mut uint16)
                .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        v1 = (v1).wrapping_add(2u16);
        i += 2 as ::core::ffi::c_int;
    }
}
