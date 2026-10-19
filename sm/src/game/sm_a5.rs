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
use crate::ram_map::{ram_word, set_ram_word, update_ram_word};
use crate::rtl::{AddToHiLo, SetHiLo};
use crate::rtl::{RomPtr, Unreachable};
use ::core::ffi::{c_char, c_void};
use ::core::ptr::null_mut;
extern "C" {
    fn memcpy(__dest: *mut c_void, __src: *const c_void, __n: size_t) -> *mut c_void;
    fn printf(__format: *const c_char, ...) -> i32;

}
pub use crate::types::Consts_A5;
pub use crate::types::{
    size_t, EnemyBase, EnemyData, EnemyProps, LongPtr, Point16U, VoidP, VramWriteEntry,
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
        pad1: [u8; 26696],
        words: [
            draygon_var_00, draygon_var_01, draygon_var_02, draygon_var_03, draygon_var_04,
            draygon_var_05, draygon_var_06, draygon_var_07, draygon_var_08, draygon_var_09,
            draygon_var_0A, draygon_var_0B, draygon_var_0C, draygon_var_0D, draygon_var_0E,
            draygon_var_0F, draygon_var_10, draygon_var_11, draygon_var_12, draygon_var_13,
            draygon_var_14, draygon_var_15, draygon_var_16, draygon_var_17, draygon_var_18,
            draygon_var_19, draygon_var_1A, draygon_var_1B, draygon_var_1C, draygon_var_1D,
            draygon_var_1E, draygon_var_1F,
        ],
        pad2: [u8; 1984],
        words: [
            draygon_var_20, draygon_var_21, draygon_var_22, draygon_var_23, draygon_var_24,
            draygon_var_25, draygon_var_26, draygon_var_27, draygon_var_28, draygon_var_29,
            draygon_var_2A, draygon_var_2B, draygon_var_2C, draygon_var_2D, draygon_var_2E,
            draygon_var_2F, draygon_var_30, draygon_var_31, draygon_var_32, draygon_var_33,
            draygon_var_34, draygon_var_35, draygon_var_36, draygon_var_37, draygon_var_38,
            draygon_var_39, draygon_var_3A, draygon_var_3B, draygon_var_3C, draygon_var_3D,
            draygon_var_3E, draygon_var_3F,
        ],
        pad3: [u8; 1984],
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
        pad1: [u8; 26696],
        words: [
            ssn_var_00, ssn_var_01, ssn_var_02, ssn_var_03, ssn_var_04, ssn_var_05,
            ssn_var_06, ssn_var_07, ssn_var_08, ssn_var_09, ssn_var_0A, ssn_var_0B,
            ssn_var_0C, ssn_var_0D, ssn_var_0E, ssn_var_0F, ssn_var_10, ssn_var_11,
            ssn_var_12, ssn_var_13, ssn_var_14, ssn_var_15, ssn_var_16, ssn_var_17,
            ssn_var_18, ssn_var_19, ssn_var_1A, ssn_var_1B, ssn_var_1C, ssn_var_1D,
            ssn_var_1E, ssn_var_1F,
        ],
        pad2: [u8; 1984],
        words: [
            ssn_var_20, ssn_var_21, ssn_var_22, ssn_var_23, ssn_var_24, ssn_var_25,
            ssn_var_26, ssn_var_27, ssn_var_28, ssn_var_29, ssn_var_2A, ssn_var_2B,
            ssn_var_2C, ssn_var_2D, ssn_var_2E, ssn_var_2F, ssn_var_30, ssn_var_31,
            ssn_var_32, ssn_var_33, ssn_var_34, ssn_var_35, ssn_var_36, ssn_var_37,
            ssn_var_38, ssn_var_39, ssn_var_3A, ssn_var_3B, ssn_var_3C, ssn_var_3D,
            ssn_var_3E, ssn_var_3F,
        ],
        pad3: [u8; 1984],
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
unsafe fn RomPtr_A0(addr: u16, game: &mut Game) -> *const u8 {
    RomPtr((0xa00000u32) | ((addr) as u32), game)
}
#[inline]
unsafe fn RomPtr_A5(addr: u16, game: &mut Game) -> *const u8 {
    RomPtr((0xa50000u32) | ((addr) as u32), game)
}
#[inline]
unsafe fn gVramWriteEntry(a: i32, game: &mut Game) -> *mut VramWriteEntry {
    (&raw mut game.g_ram as *mut u8).offset((0xd0i32 + a) as isize) as *mut u8
        as *mut VramWriteEntry
}
#[inline]
unsafe fn get_EnemyDef_A2(a: u16, game: &mut Game) -> *mut EnemyDef {
    RomPtr((0xa00000u32) | ((a) as u32), game) as *mut EnemyDef
}
pub const fnSamus_ScrollFinishedHook_SporeSpawnFight: i32 = 0x909589;
pub const fnGrappleBeam_Func2: i32 = 0x9bc8c5;
pub const fnnullsub_169_A5: i32 = 0xa5804b;
pub const fnDraygon_Func_1: i32 = 0xa5871b;
pub const fnDraygon_Func_2: i32 = 0xa5878b;
pub const fnDraygon_Func_4: i32 = 0xa587f4;
pub const fnDraygon_Func_6: i32 = 0xa588b1;
pub const fnDraygon_Func_8: i32 = 0xa58922;
pub const fnDraygon_Func_9: i32 = 0xa58951;
pub const fnDraygon_Func_10: i32 = 0xa589b3;
pub const fnDraygon_Func_11: i32 = 0xa58a00;
pub const fnDraygon_Func_12: i32 = 0xa58a50;
pub const fnDraygon_Func_13: i32 = 0xa58a90;
pub const fnDraygon_Func_14: i32 = 0xa58b0a;
pub const fnDraygon_Func_15: i32 = 0xa58b52;
pub const fnDraygon_Func_16: i32 = 0xa58bae;
pub const fnDraygon_Func_17: i32 = 0xa58c33;
pub const fnDraygon_Func_18: i32 = 0xa58c8e;
pub const fnDraygon_Func_19: i32 = 0xa58cd4;
pub const fnDraygon_Func_20: i32 = 0xa58d30;
pub const fnDraygon_Func_21: i32 = 0xa58db2;
pub const fnDraygon_Func_22: i32 = 0xa58e19;
pub const fnDraygon_Func_23: i32 = 0xa58f10;
pub const fnDraygon_Func_24: i32 = 0xa58f1e;
pub const fnDraygon_Func_25: i32 = 0xa58fd6;
pub const fnDraygon_Func_26: i32 = 0xa590d4;
pub const fnDraygon_Func_27: i32 = 0xa59105;
pub const fnDraygon_Func_28: i32 = 0xa59124;
pub const fnDraygon_Func_29: i32 = 0xa59128;
pub const fnDraygon_Func_30: i32 = 0xa59154;
pub const fnDraygon_Func_31: i32 = 0xa59185;
pub const fnDraygon_Func_32: i32 = 0xa59294;
pub const fnDraygon_Func_33: i32 = 0xa592ab;
pub const fnDraygon_Func_36: i32 = 0xa59342;
pub const fnDraygon_Func_48: u32 = 10863757;
pub const fnDraygon_Func_49: u32 = 10863891;
pub const fnnullsub_223: i32 = 0xa5eb1a;
pub const fnSporeSpawn_Func_1: u32 = 10873627;
pub const fnSporeSpawn_Func_2: i32 = 0xa5eb52;
pub const fnSporeSpawn_Func_3: u32 = 10873755;
pub const fnSporeSpawn_Func_4: u32 = 10873838;
#[inline]
unsafe fn gEnemyData(o: i32, game: &mut Game) -> *mut EnemyData {
    (&raw mut game.g_ram as *mut u8)
        .offset(0xf78)
        .offset(o as isize) as *mut EnemyData
}
#[inline]
unsafe fn Get_Draygon(o: i32, game: &mut Game) -> *mut Enemy_Draygon {
    (&raw mut game.g_ram as *mut u8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_Draygon
}
#[inline]
unsafe fn Get_SporeSpawn(o: i32, game: &mut Game) -> *mut Enemy_SporeSpawn {
    (&raw mut game.g_ram as *mut u8)
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
    let mut j: u16 = 0;
    let mut i: i32 = 48i32;
    while i >= 0i32 {
        set_ram_word!(
            game.target_palettes[((i >> 1i32) + 144i32)] =
                crate::rtl::rom_table_word!(game, 0xa5a217, (i >> 1i32) as isize)
        );
        i -= 2i32;
    }
    j = 4094u16;
    while (j as i32 & 0x8000i32) == 0i32 {
        set_ram_word!(game.tilemap_stuff[(j as i32 >> 1i32)] = 824u16);
        j = (j).wrapping_sub(2u16);
    }
    (*Get_Draygon(ram_word!(game, cur_enemy_index) as i32, game))
        .base
        .palette_index = 0xe00u16;
    *(&raw mut (*((&raw mut game.g_ram as *mut u8).offset(0x178c) as *mut LongPtr)).bank
        as *mut u16) = 165u16;
    (*((&raw mut game.g_ram as *mut u8).offset(0x178c) as *mut LongPtr)).addr =
        ((0xa59342i32) as u16);
    set_ram_word!(game.flag_process_all_enemies = -(1i32) as u16);
    (*Get_Draygon(0i32, game)).base.current_instruction = addr_kDraygon_Ilist_9889 as u16;
    (*Get_Draygon(0x40i32, game)).base.current_instruction = addr_kDraygon_Ilist_9944 as u16;
    (*Get_Draygon(0x80i32, game)).base.current_instruction = addr_kDraygon_Ilist_99FC as u16;
    (*Get_Draygon(0xc0i32, game)).base.current_instruction = addr_kDraygon_Ilist_9813 as u16;
    set_ram_word!(game.enemy_bg2_tilemap_size = 1024u16);
    (*Get_Draygon(ram_word!(game, cur_enemy_index) as i32, game)).draygon_var_A =
        ((0xa5871bi32) as u16);
    sub_88DF34(game);
    set_ram_word!(game.room_loading_irq_handler = 12u16);
    SpawnBG3ScrollHdmaObject(game);
    (*Get_Draygon(0i32, game)).draygon_var_45 = 1u16;
    DisableMinimapAndMarkBossRoomAsExplored(game);
}
pub unsafe fn CallDraygonFunc(ea: u32, k: u16, game: &mut Game) {
    match ea {
        10846283 => (),
        10848027 => Draygon_Func_1(game),
        10848139 => Draygon_Func_2(game),
        10848244 => Draygon_Func_4(game),
        10848433 => Draygon_Func_6(game),
        10848546 => Draygon_Func_8(game),
        10848593 => Draygon_Func_9(game),
        10848691 => Draygon_Func_10(game),
        10848768 => Draygon_Func_11(game),
        10848848 => Draygon_Func_12(game),
        10848912 => Draygon_Func_13(game),
        10849034 => Draygon_Func_14(game),
        10849106 => Draygon_Func_15(game),
        10849198 => Draygon_Func_16(game),
        10849331 => Draygon_Func_17(k, game),
        10849422 => Draygon_Func_18(game),
        10849492 => Draygon_Func_19(game),
        10849584 => Draygon_Func_20(game),
        10849714 => Draygon_Func_21(k, game),
        10849817 => Draygon_Func_22(game),
        10850064 => Draygon_Func_23(game),
        10850078 => Draygon_Func_24(k, game),
        10850262 => Draygon_Func_25(k, game),
        10850516 => Draygon_Func_26(k, game),
        10850565 => Draygon_Func_27(k, game),
        10850596 => Draygon_Func_28(k, game),
        10850600 => Draygon_Func_29(game),
        10850644 => Draygon_Func_30(game),
        10850693 => Draygon_Func_31(game),
        10850964 => Draygon_Func_32(game),
        10850987 => Draygon_Func_33(game),
        10863757 => Draygon_Func_48(k, game),
        10863891 => Draygon_Func_49(k, game),
        10873626 => (),
        10873627 => SporeSpawn_Func_1(game),
        10873682 => SporeSpawn_Func_2(k, game),
        10873755 => SporeSpawn_Func_3(game),
        10873838 => SporeSpawn_Func_4(game),
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn Draygon_Main(game: &mut Game) {
    let EK: *mut Enemy_Draygon = Get_Draygon(ram_word!(game, cur_enemy_index) as i32, game);
    CallDraygonFunc(
        (((*EK).draygon_var_A) as u32) | (0xa50000u32),
        ram_word!(game, cur_enemy_index),
        game,
    );
    let E0: *mut Enemy_Draygon = Get_Draygon(0i32, game);
    let x_pos: u16 = (*E0).base.x_pos;
    let E1: *mut Enemy_Draygon = Get_Draygon(0x40i32, game);
    (*E1).base.x_pos = x_pos;
    let E2: *mut Enemy_Draygon = Get_Draygon(0x80i32, game);
    (*E2).base.x_pos = x_pos;
    let E3: *mut Enemy_Draygon = Get_Draygon(0xc0i32, game);
    (*E3).base.x_pos = x_pos;
    let y_pos: u16 = (*E0).base.y_pos;
    (*E1).base.y_pos = y_pos;
    (*E2).base.y_pos = y_pos;
    (*E3).base.y_pos = y_pos;
}
pub unsafe fn Draygon_Func_1(game: &mut Game) {
    let mut v2: *mut VramWriteEntry = null_mut::<VramWriteEntry>();
    Draygon_Func_3(game);
    let E: *mut Enemy_Draygon = Get_Draygon(0i32, game);
    if (*E).draygon_var_B as i32 - 256i32 & 0x8000i32 != 0 {
        if (*E).draygon_var_B == 0 {
            v2 = gVramWriteEntry(ram_word!(game, vram_write_queue_tail) as i32, game);
            (*v2).size = 1536u16;
            (*v2).src.addr = -(27648i32) as VoidP;
            *(&raw mut (*v2).src.bank as *mut u16) = 177u16;
            (*v2).vram_dst = 27904u16;
            update_ram_word!(game.vram_write_queue_tail, wrapping_add(7u16));
            Draygon_Func_45(game);
        }
        (*E).draygon_var_B = (*E).draygon_var_B.wrapping_add(1);
    } else {
        (*E).draygon_var_A = ((0xa5878bi32) as u16);
        (*E).draygon_var_B = 0u16;
        let x_pos: u16 = (*E).base.x_pos;
        (*E).draygon_var_00 = x_pos;
        (*E).draygon_var_02 = (x_pos).wrapping_add(672u16);
        (*E).draygon_var_01 = (*E).base.y_pos;
        (*E).base.x_pos = (*E).draygon_var_00;
        (*E).base.y_pos = (*E).draygon_var_01;
        (*E).draygon_var_0F = 24u16;
    };
}
pub unsafe fn Draygon_Func_2(game: &mut Game) {
    Draygon_Func_3(game);
    let E: *mut Enemy_Draygon = Get_Draygon(0i32, game);
    if (*E).draygon_var_B as i32 - 1232i32 & 0x8000i32 != 0 {
        Draygon_Func_46(game);
        (*E).draygon_var_B = (*E).draygon_var_B.wrapping_add(1);
    } else {
        (*E).draygon_var_A = ((0xa587f4i32) as u16);
        (*E).draygon_var_B = 0u16;
    };
}
pub unsafe fn Draygon_Func_3(game: &mut Game) {
    let mut Random: i8 = 0;
    if (ram_word!(game, nmi_frame_counter_word) as i32 & 0x3fi32) == 0i32 {
        Random = NextRandom(game) as i8;
        let result: u16 = (2i32 * ((Random as i32 & 3i32) + 2i32)) as u16;
        if (*Get_Draygon(result as i32, game)).draygon_var_40 == 0 {
            let v2: i32 = (4i32 * ((Random as i32 & 3i32) + 2i32)) as u16 as i32 >> 1i32;
            game.eproj_spawn_pt = Point16U {
                x: crate::rtl::rom_table_word!(game, 0xa587dc, v2 as isize),
                y: crate::rtl::rom_table_word!(game, 0xa587dc, (v2 + 1i32) as isize),
            };
            SpawnEprojWithGfx(3u16, result, addr_stru_868E5E as u16, game);
        }
    }
}
pub unsafe fn Draygon_Func_4(game: &mut Game) {
    Draygon_Func_3(game);
    Draygon_Func_5(game);
    let E: *mut Enemy_Draygon = Get_Draygon(0i32, game);
    (*E).draygon_var_A = ((0xa588b1i32) as u16);
    (*E).base.current_instruction = addr_kDraygon_Ilist_97D1 as u16;
    (*E).base.instruction_timer = 1u16;
    (*E).draygon_var_20 = 1u16;
}
pub unsafe fn Draygon_Func_5(game: &mut Game) {
    let E: *mut Enemy_Draygon = Get_Draygon(0i32, game);
    let mut varE24: u16 = 384u16;
    let mut varE22: u16 = 0u16;
    let mut varE26: u16 = 0u16;
    loop {
        varE22 = (varE22).wrapping_add((*E).draygon_var_0F);
        let v1: u16 = (varE24).wrapping_sub((*(&raw mut varE22 as *mut u8).offset(1isize)) as u16);
        if ((v1) as i16).wrapping_sub(((*E).draygon_var_01) as i16) < 0i16 {
            break;
        }
        varE24 = (varE24).wrapping_sub((*(&raw mut varE22 as *mut u8).offset(1isize)) as u16);
        let fresh4 = varE26;
        varE26 = varE26.wrapping_add(1);
        *(((&raw mut game.g_ram as *mut u8).offset(0x9002) as *mut u16 as *mut u8)
            .offset(fresh4 as i32 as isize) as *mut u16) = v1;
        varE26 = (varE26).wrapping_add(2u16);
        varE26 = varE26.wrapping_add(1);
        if varE26 as i32 - 2048i32 & 0x8000i32 == 0 {
            Unreachable(game);
        }
    }
    let t: u32 = EnemyFunc_Divide(
        (varE26 as i32 >> 2i32) as u32,
        ((Abs16(((*E).draygon_var_00).wrapping_sub(ram_word!(game, samus_x_pos))) as i32) << 16i32)
            as u32,
    );
    SetHiLo(&raw mut (*E).draygon_var_D, &raw mut (*E).draygon_var_E, t);
    let v6: u16 = varE26;
    *(((&raw mut game.g_ram as *mut u8).offset(0x9002) as *mut u16 as *mut u8)
        .offset(varE26 as i32 as isize) as *mut u16) = (*E).base.y_pos;
    (*E).draygon_var_B = v6;
    (*E).draygon_var_C = v6;
}
pub unsafe fn Draygon_Func_6(game: &mut Game) {
    Draygon_Func_3(game);
    Draygon_Func_35(game);
    let E: *mut Enemy_Draygon = Get_Draygon(0i32, game);
    let draygon_var_B: u16 = (*E).draygon_var_B;
    if draygon_var_B == 104u16 {
        let E3: *mut Enemy_Draygon = Get_Draygon(0xc0i32, game);
        (*E3).base.current_instruction = addr_kDraygon_Ilist_9C06 as u16;
        (*E3).base.instruction_timer = 1u16;
    }
    (*Get_Draygon(ram_word!(game, cur_enemy_index) as i32, game))
        .base
        .y_pos = *(((&raw mut game.g_ram as *mut u8).offset(0x9002) as *mut u16 as *mut u8)
        .offset(draygon_var_B as i32 as isize) as *mut u16);
    (*E).draygon_var_B = ((*E).draygon_var_B).wrapping_sub(4u16);
    if (*E).draygon_var_B != 0 {
        AddToHiLo(
            &raw mut (*E).base.x_pos,
            &raw mut (*E).base.x_subpos,
            ((*E).draygon_var_D as u32) << 16i32 | (*E).draygon_var_E as u32,
        );
    } else {
        (*E).draygon_var_A = ((0xa58922i32) as u16);
    };
}
pub unsafe fn Draygon_Func_7(game: &mut Game) {
    let E: *mut Enemy_Draygon = Get_Draygon(0i32, game);
    if (ram_word!(game, nmi_frame_counter_word) as i32 & 0xfi32) == 0i32 {
        (*E).base.current_instruction = addr_kDraygon_Ilist_98FE as u16;
        if (*E).draygon_var_20 != 0 {
            (*E).base.current_instruction = addr_kDraygon_Ilist_9C90 as u16;
        }
        (*E).base.instruction_timer = 1u16;
    }
}
pub unsafe fn Draygon_Func_8(game: &mut Game) {
    Draygon_Func_3(game);
    let E: *mut Enemy_Draygon = Get_Draygon(0i32, game);
    let t: u32 = EnemyFunc_Divide(
        ((*E).draygon_var_C as i32 >> 2i32) as u32,
        ((672i32 - (*E).base.x_pos as i32) << 16i32) as u32,
    );
    SetHiLo(&raw mut (*E).draygon_var_D, &raw mut (*E).draygon_var_E, t);
    (*E).draygon_var_A = ((0xa58951i32) as u16);
}
pub unsafe fn Draygon_Func_9(game: &mut Game) {
    Draygon_Func_3(game);
    let E: *mut Enemy_Draygon = Get_Draygon(0i32, game);
    let draygon_var_B: u16 = (*E).draygon_var_B;
    if draygon_var_B == 104u16 {
        let E3: *mut Enemy_Draygon = Get_Draygon(0xc0i32, game);
        (*E3).base.current_instruction = addr_kDraygon_Ilist_9BDA as u16;
        (*E3).base.instruction_timer = 1u16;
    }
    (*Get_Draygon(ram_word!(game, cur_enemy_index) as i32, game))
        .base
        .y_pos = *(((&raw mut game.g_ram as *mut u8).offset(0x9002) as *mut u16 as *mut u8)
        .offset(draygon_var_B as i32 as isize) as *mut u16);
    (*E).draygon_var_B = ((*E).draygon_var_B).wrapping_add(4u16);
    if (*E).draygon_var_B == (*E).draygon_var_C {
        (*E).draygon_var_B = 0u16;
        if (ram_word!(game, random_number) as i32 & 1i32) != 0i32 {
            (*E).draygon_var_A = ((0xa589b3i32) as u16);
        } else {
            (*E).draygon_var_A = ((0xa58c8ei32) as u16);
        }
    } else {
        AddToHiLo(
            &raw mut (*E).base.x_pos,
            &raw mut (*E).base.x_subpos,
            ((*E).draygon_var_D as u32) << 16i32 | (*E).draygon_var_E as u32,
        );
    };
}
pub unsafe fn Draygon_Func_10(game: &mut Game) {
    Draygon_Func_3(game);
    let E: *mut Enemy_Draygon = Get_Draygon(0i32, game);
    printf(b"Wtf is varE26?\n\0" as *const u8 as *const c_char);
    let varE26: u16 = 0u16;
    let t: u32 = EnemyFunc_Divide(
        (varE26 as i32 >> 2i32) as u32,
        ((Abs16(((*E).draygon_var_02).wrapping_sub(ram_word!(game, samus_x_pos))) as i32) << 16i32)
            as u32,
    );
    SetHiLo(&raw mut (*E).draygon_var_D, &raw mut (*E).draygon_var_E, t);
    (*E).draygon_var_A = ((0xa58a00i32) as u16);
    (*E).base.current_instruction = addr_kDraygon_Ilist_97BB as u16;
    (*E).base.instruction_timer = 1u16;
    (*E).draygon_var_20 = 0u16;
    (*E).draygon_var_B = (*E).draygon_var_C;
}
pub unsafe fn Draygon_Func_11(game: &mut Game) {
    Draygon_Func_3(game);
    Draygon_Func_35(game);
    let E: *mut Enemy_Draygon = Get_Draygon(0i32, game);
    let draygon_var_B: u16 = (*E).draygon_var_B;
    if draygon_var_B == 104u16 {
        let E3: *mut Enemy_Draygon = Get_Draygon(0xc0i32, game);
        (*E3).base.current_instruction = addr_kDraygon_Ilist_9813 as u16;
        (*E3).base.instruction_timer = 1u16;
    }
    (*Get_Draygon(ram_word!(game, cur_enemy_index) as i32, game))
        .base
        .y_pos = *(((&raw mut game.g_ram as *mut u8).offset(0x9002) as *mut u16 as *mut u8)
        .offset(draygon_var_B as i32 as isize) as *mut u16);
    (*E).draygon_var_B = ((*E).draygon_var_B).wrapping_sub(4u16);
    if (*E).draygon_var_B != 0 {
        AddToHiLo(
            &raw mut (*E).base.x_pos,
            &raw mut (*E).base.x_subpos,
            -(((*E).draygon_var_D as i32) << 16i32 | (*E).draygon_var_E as i32) as u32,
        );
    } else {
        (*E).draygon_var_A = ((0xa58a50i32) as u16);
    };
}
pub unsafe fn Draygon_Func_12(game: &mut Game) {
    Draygon_Func_3(game);
    let E: *mut Enemy_Draygon = Get_Draygon(0i32, game);
    let t: u32 = EnemyFunc_Divide(
        ((*E).draygon_var_C as i32 >> 2i32) as u32,
        (((*E).base.x_pos as i32 - (*E).draygon_var_00 as i32) << 16i32) as u32,
    );
    (*E).draygon_var_D = (t >> 16i32) as u16;
    (*E).draygon_var_E = t as u16;
    (*E).draygon_var_A = ((0xa58a90i32) as u16);
}
pub unsafe fn Draygon_Func_13(game: &mut Game) {
    Draygon_Func_3(game);
    Draygon_Func_35(game);
    let E: *mut Enemy_Draygon = Get_Draygon(0i32, game);
    let draygon_var_B: u16 = (*E).draygon_var_B;
    if draygon_var_B == 104u16 {
        let E3: *mut Enemy_Draygon = Get_Draygon(0xc0i32, game);
        (*E3).base.current_instruction = addr_kDraygon_Ilist_97E7 as u16;
        (*E3).base.instruction_timer = 1u16;
    }
    (*Get_Draygon(ram_word!(game, cur_enemy_index) as i32, game))
        .base
        .y_pos = *(((&raw mut game.g_ram as *mut u8).offset(0x9002) as *mut u16 as *mut u8)
        .offset(draygon_var_B as i32 as isize) as *mut u16);
    (*E).draygon_var_B = ((*E).draygon_var_B).wrapping_add(4u16);
    if (*E).draygon_var_B == (*E).draygon_var_C {
        if (ram_word!(game, random_number) as i32 & 1i32) != 0i32 {
            (*E).draygon_var_A = ((0xa587f4i32) as u16);
            (*E).draygon_var_B = 0u16;
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
            -(((*E).draygon_var_D as i32) << 16i32 | (*E).draygon_var_E as i32) as u32,
        );
    };
}
pub unsafe fn Draygon_Func_14(game: &mut Game) {
    Draygon_Func_3(game);
    let E: *mut Enemy_Draygon = Get_Draygon(0i32, game);
    (*E).base.x_pos = -(80i32) as u16;
    (*E).base.y_pos = 384u16;
    (*E).draygon_var_E = 0u16;
    (*E).draygon_var_D = 1u16;
    (*E).draygon_var_0D = 0u16;
    let E3: *mut Enemy_Draygon = Get_Draygon(0xc0i32, game);
    (*E3).base.current_instruction = addr_kDraygon_Ilist_9C06 as u16;
    (*E3).base.instruction_timer = 1u16;
    (*E).draygon_var_A = ((0xa58b52i32) as u16);
    (*E).base.current_instruction = addr_kDraygon_Ilist_97D1 as u16;
    (*E).base.instruction_timer = 1u16;
    (*E).draygon_var_20 = 1u16;
}
pub unsafe fn Draygon_Func_15(game: &mut Game) {
    Draygon_Func_3(game);
    Draygon_Func_35(game);
    let E: *mut Enemy_Draygon = Get_Draygon(0i32, game);
    let v2: u16 = Abs16(((*E).base.x_pos).wrapping_sub(ram_word!(game, samus_x_pos)));
    if v2 as i32 - 208i32 & 0x8000i32 != 0 {
        (*E).draygon_var_A = ((0xa58baei32) as u16);
        (*E).draygon_var_03 = 16u16;
    } else {
        (*E).base.y_pos = (CosineMult8bit((*E).draygon_var_0D, 32u16, game)).wrapping_add(384u16);
        (*E).draygon_var_0D = ((((*E).draygon_var_0D) as u8).wrapping_add(1u8)) as u16;
        AddToHiLo(
            &raw mut (*E).base.x_pos,
            &raw mut (*E).base.x_subpos,
            ((*E).draygon_var_D as u32) << 16i32 | (*E).draygon_var_E as u32,
        );
    };
}
pub unsafe fn Draygon_Func_16(game: &mut Game) {
    let mut current_block: u64;
    let E: *mut Enemy_Draygon = Get_Draygon(0i32, game);
    Draygon_Func_3(game);
    if ram_word!(game, samus_x_speed_divisor) != 0 {
        current_block = 1399980599923904196;
    } else {
        if (ram_word!(game, random_number) as i32 & 0xfi32) == 0i32 {
            let v1: u16 = ((*E).draygon_var_03).wrapping_sub(1u16);
            (*E).draygon_var_03 = v1;
            if v1 == 0 {
                current_block = 11361156962102030361;
            } else {
                (*E).base.current_instruction = addr_kDraygon_Ilist_9C90 as u16;
                (*E).base.instruction_timer = 1u16;
                current_block = 6483416627284290920;
            }
        } else {
            current_block = 6483416627284290920;
        }
        if current_block == 6483416627284290920 {
            (*E).base.y_pos =
                (CosineMult8bit((*E).draygon_var_0D, 32u16, game)).wrapping_add(384u16);
            (*E).draygon_var_0D = ((((*E).draygon_var_0D) as u8).wrapping_add(1u8)) as u16;
            AddToHiLo(
                &raw mut (*E).base.x_pos,
                &raw mut (*E).base.x_subpos,
                ((*E).draygon_var_D as u32) << 16i32 | (*E).draygon_var_E as u32,
            );
            if ((*E).base.x_pos as i16) >= 0i16 && (*E).base.x_pos as i32 - 672i32 & 0x8000i32 == 0
            {
                current_block = 11361156962102030361;
            } else {
                current_block = 7976072742316086414;
            }
        }
        if current_block == 7976072742316086414 {
        } else {
            if ram_word!(game, samus_x_speed_divisor) == 0 {
                let E3: *mut Enemy_Draygon = Get_Draygon(0xc0i32, game);
                (*E3).base.current_instruction = addr_kDraygon_Ilist_9BDA as u16;
                (*E3).base.instruction_timer = 1u16;
                (*E).draygon_var_A = ((0xa58c33i32) as u16);
                return;
            }
            current_block = 1399980599923904196;
        }
    }
    if current_block == 1399980599923904196 {
        (*E).draygon_var_A = ((0xa58e19i32) as u16);
    };
}
pub unsafe fn Draygon_Func_17(k: u16, game: &mut Game) {
    Draygon_Func_35(game);
    let E: *mut Enemy_Draygon = Get_Draygon(0i32, game);
    (*E).base.y_pos = (CosineMult8bit((*E).draygon_var_0D, 32u16, game)).wrapping_add(384u16);
    (*E).draygon_var_0D = ((((*E).draygon_var_0D) as u8).wrapping_add(1u8)) as u16;
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        ((*E).draygon_var_D as u32) << 16i32 | (*E).draygon_var_E as u32,
    );
    if ((*E).base.x_pos as i16) >= 0i16 && (*E).base.x_pos as i32 - 672i32 & 0x8000i32 == 0 {
        (*E).draygon_var_A = ((0xa589b3i32) as u16);
        (*E).base.x_pos = 592u16;
        (*E).base.y_pos = -(80i32) as u16;
        (*E).draygon_var_B = (*E).draygon_var_C;
    }
}
pub unsafe fn Draygon_Func_18(game: &mut Game) {
    let E: *mut Enemy_Draygon = Get_Draygon(0i32, game);
    (*E).base.x_pos = (*E).draygon_var_02;
    (*E).base.y_pos = 384u16;
    (*E).draygon_var_E = 0u16;
    (*E).draygon_var_D = 1u16;
    (*E).draygon_var_0D = 0u16;
    let E3: *mut Enemy_Draygon = Get_Draygon(0xc0i32, game);
    (*E3).base.current_instruction = addr_kDraygon_Ilist_9C06 as u16;
    (*E3).base.instruction_timer = 1u16;
    (*E).draygon_var_A = ((0xa58cd4i32) as u16);
    (*E).base.current_instruction = addr_kDraygon_Ilist_97BB as u16;
    (*E).base.instruction_timer = 1u16;
    (*E).draygon_var_20 = 0u16;
}
pub unsafe fn Draygon_Func_19(game: &mut Game) {
    Draygon_Func_3(game);
    Draygon_Func_35(game);
    let E: *mut Enemy_Draygon = Get_Draygon(0i32, game);
    let v2: u16 = Abs16(((*E).base.x_pos).wrapping_sub(ram_word!(game, samus_x_pos)));
    if v2 as i32 - 208i32 & 0x8000i32 != 0 {
        (*E).draygon_var_A = ((0xa58d30i32) as u16);
        (*E).draygon_var_03 = 16u16;
    } else {
        (*E).base.y_pos = (CosineMult8bit((*E).draygon_var_0D, 32u16, game)).wrapping_add(384u16);
        (*E).draygon_var_0D = ((((*E).draygon_var_0D) as u8).wrapping_add(1u8)) as u16;
        AddToHiLo(
            &raw mut (*E).base.x_pos,
            &raw mut (*E).base.x_subpos,
            -(((*E).draygon_var_D as i32) << 16i32 | (*E).draygon_var_E as i32) as u32,
        );
    };
}
pub unsafe fn Draygon_Func_20(game: &mut Game) {
    let mut current_block: u64;
    let E: *mut Enemy_Draygon = Get_Draygon(0i32, game);
    if ram_word!(game, samus_x_speed_divisor) != 0 {
        current_block = 11384386367536015713;
    } else {
        if (ram_word!(game, random_number) as i32 & 0xfi32) == 0i32 {
            let v1: u16 = ((*E).draygon_var_03).wrapping_sub(1u16);
            (*E).draygon_var_03 = v1;
            if v1 == 0 {
                current_block = 17278387905106781498;
            } else {
                (*E).base.current_instruction = addr_kDraygon_Ilist_98FE as u16;
                (*E).base.instruction_timer = 1u16;
                current_block = 6873731126896040597;
            }
        } else {
            current_block = 6873731126896040597;
        }
        if current_block == 6873731126896040597 {
            (*E).base.y_pos =
                (CosineMult8bit((*E).draygon_var_0D, 32u16, game)).wrapping_add(384u16);
            (*E).draygon_var_0D = ((((*E).draygon_var_0D) as u8).wrapping_add(1u8)) as u16;
            AddToHiLo(
                &raw mut (*E).base.x_pos,
                &raw mut (*E).base.x_subpos,
                -(((*E).draygon_var_D as i32) << 16i32 | (*E).draygon_var_E as i32) as u32,
            );
            if (*E).base.x_pos < 0u16 && (*E).base.x_pos as i32 + 80i32 & 0x8000i32 != 0 {
                current_block = 17278387905106781498;
            } else {
                current_block = 10048703153582371463;
            }
        }
        if current_block == 10048703153582371463 {
        } else {
            if ram_word!(game, samus_x_speed_divisor) == 0 {
                let E3: *mut Enemy_Draygon = Get_Draygon(0xc0i32, game);
                (*E3).base.current_instruction = addr_kDraygon_Ilist_97E7 as u16;
                (*E3).base.instruction_timer = 1u16;
                (*Get_Draygon(0i32, game)).draygon_var_A = ((0xa58db2i32) as u16);
                return;
            }
            current_block = 11384386367536015713;
        }
    }
    if current_block == 11384386367536015713 {
        (*Get_Draygon(0i32, game)).draygon_var_A = ((0xa58e19i32) as u16);
    };
}
pub unsafe fn Draygon_Func_21(k: u16, game: &mut Game) {
    let E: *mut Enemy_Draygon = Get_Draygon(0i32, game);
    Draygon_Func_35(game);
    if ram_word!(game, samus_x_speed_divisor) != 0 {
        (*E).draygon_var_A = ((0xa58e19i32) as u16);
    } else {
        (*E).base.y_pos = (CosineMult8bit((*E).draygon_var_0D, 32u16, game)).wrapping_add(384u16);
        (*E).draygon_var_0D = ((((*E).draygon_var_0D) as u8).wrapping_add(1u8)) as u16;
        AddToHiLo(
            &raw mut (*E).base.x_pos,
            &raw mut (*E).base.x_subpos,
            -(((*E).draygon_var_D as i32) << 16i32 | (*E).draygon_var_E as i32) as u32,
        );
        if ((*E).base.x_pos as i16) < 0i16 && (*E).base.x_pos as i32 + 80i32 & 0x8000i32 != 0 {
            (*E).draygon_var_A = ((0xa587f4i32) as u16);
            (*E).base.x_pos = -(80i32) as u16;
            (*E).base.y_pos = -(80i32) as u16;
            (*E).draygon_var_B = (*E).draygon_var_C;
        }
    };
}
pub unsafe fn Draygon_Func_22(game: &mut Game) {
    let E: *mut Enemy_Draygon = Get_Draygon(0i32, game);
    let mut v2: i16 = 0;
    let mut v4: u16 = 0;
    Draygon_Func_3(game);
    if ram_word!(game, samus_x_speed_divisor) != 0 {
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
        v2 = 8i16;
        if (*E).draygon_var_20 == 0 {
            v2 = -(8i32) as i16;
        }
        let v3: u16 = Abs16(
            ((*E).base.x_pos as i32 + v2 as i32 - ram_word!(game, samus_x_pos) as i32) as u16,
        );
        if v3 as i32 - 8i32 & 0x8000i32 != 0 && {
            v4 = Abs16(((*E).base.y_pos).wrapping_sub(ram_word!(game, samus_y_pos)));
            v4 as i32 - 8i32 & 0x8000i32 != 0
        } {
            let mut v5: u16 = addr_kDraygon_Ilist_9845 as u16;
            if (*E).draygon_var_20 != 0 {
                v5 = addr_kDraygon_Ilist_9C38 as u16;
            }
            let E3: *mut Enemy_Draygon = Get_Draygon(0xc0i32, game);
            (*E3).base.current_instruction = v5;
            (*E3).base.instruction_timer = 1u16;
            if CallSomeSamusCode(0xdu16, game) != 0 {
                (*E).draygon_var_A = ((0xa58f10i32) as u16);
            } else {
                Samus_SetGrabbedByDraygonPose(((*E).draygon_var_20 != 0u16) as u16, game);
                (*E).draygon_var_06 = 256u16;
                (*E).draygon_var_07 = 384u16;
                (*E).draygon_var_05 = 0u16;
                (*E).draygon_var_09 = 0u16;
                (*E).draygon_var_08 = 192u16;
                (*E).draygon_var_0B = 2048u16;
                (*E).draygon_var_A = ((0xa58f1ei32) as u16);
            }
        } else {
            let varE20: u16 = ((64u8).wrapping_sub(
                (CalculateAngleFromXY(
                    (ram_word!(game, samus_x_pos)).wrapping_sub((*E).base.x_pos),
                    (ram_word!(game, samus_y_pos)).wrapping_sub((*E).base.y_pos),
                    game,
                )) as u8,
            )) as u16;
            let pt: Point32 = ConvertAngleToXy(varE20, 2u16, game);
            EnemyFunc_B691(varE20, pt, game);
        }
    } else {
        (*E).draygon_var_A = ((0xa59154i32) as u16);
    };
}
pub unsafe fn Draygon_Func_23(game: &mut Game) {
    set_ram_word!(game.grapple_beam_function = ((0x9bc8c5i32) as u16));
    (*Get_Draygon(0i32, game)).draygon_var_A = ((0xa59128i32) as u16);
}
pub unsafe fn Draygon_Func_24(k: u16, game: &mut Game) {
    let E: *mut Enemy_Draygon = Get_Draygon(0i32, game);
    let mut v5: u16 = 0;
    if (ram_word!(game, samus_grapple_flags) as i32 & 1i32) != 0i32 {
        (*E).base.flash_timer = (((*get_EnemyDef_A2((*E).base.enemy_ptr, game)).hurt_ai_time)
            as u16)
            .wrapping_add(8u16);
        (*E).base.ai_handler_bits = ((*E).base.ai_handler_bits) | (2u16);
    } else {
        let v4: u16 = Abs16(((*E).base.x_pos).wrapping_sub(256u16));
        if v4 as i32 - 2i32 & 0x8000i32 != 0 && {
            v5 = Abs16(((*E).base.y_pos).wrapping_sub(384u16));
            v5 as i32 - 2i32 & 0x8000i32 != 0
        } {
            (*E).draygon_var_A = ((0xa58fd6i32) as u16);
            let mut v6: u16 = addr_kDraygon_Ilist_9922 as u16;
            if (*E).draygon_var_20 != 0 {
                v6 = addr_kDraygon_Ilist_9CB4 as u16;
            }
            let EK: *mut Enemy_Draygon = Get_Draygon(k as i32, game);
            (*EK).base.current_instruction = v6;
            (*EK).base.instruction_timer = 1u16;
            (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
        } else {
            let varE20: u16 = ((64u8).wrapping_sub(
                (CalculateAngleFromXY(
                    (256u16).wrapping_sub((*E).base.x_pos),
                    (384u16).wrapping_sub((*E).base.y_pos),
                    game,
                )) as u8,
            )) as u16;
            let pt: Point32 = ConvertAngleToXy(varE20, 2u16, game);
            EnemyFunc_B691(varE20, pt, game);
            Draygon_Func_40(k, game);
        }
    };
}
pub unsafe fn Draygon_Func_25(k: u16, game: &mut Game) {
    let E: *mut Enemy_Draygon = Get_Draygon(0i32, game);
    let mut v6: u16 = 0;
    if (ram_word!(game, samus_grapple_flags) as i32 & 1i32) != 0i32 {
        (*E).base.flash_timer = (((*get_EnemyDef_A2((*E).base.enemy_ptr, game)).hurt_ai_time)
            as u16)
            .wrapping_add(8u16);
        (*E).base.ai_handler_bits = ((*E).base.ai_handler_bits) | (2u16);
    } else if ram_word!(game, random_number) as u8 != 0 {
        (*E).base.x_pos = ((*E).draygon_var_06).wrapping_add(CosineMult8bit(
            (*E).draygon_var_08,
            (*E).draygon_var_05,
            game,
        ));
        (*E).base.y_pos = ((*E).draygon_var_07).wrapping_add(SineMult8bit(
            (*E).draygon_var_08,
            ((*E).draygon_var_05 as i32 >> 2i32) as u16,
            game,
        ));
        if ((*Get_Draygon(k as i32, game)).base.frame_counter as i32 & 7i32) == 0i32 {
            let x: u16 = (if (*E).draygon_var_20 != 0u16 {
                (*E).base.x_pos as i32 + 32i32
            } else {
                (*E).base.x_pos as i32 - 32i32
            }) as u16;
            let y: u16 = ((*E).base.y_pos).wrapping_sub(16u16);
            CreateSpriteAtPos(x, y, 61u16, 0u16, game);
        }
        AddToHiLo(
            &raw mut (*E).draygon_var_05,
            &raw mut (*E).draygon_var_09,
            0x2000u32,
        );
        let mut draygon_var_0A: u16 = 0;
        let mut v8: u16 = 0;
        if (*E).draygon_var_05 as i32 - 160i32 & 0x8000i32 == 0 || {
            v6 = ((*E).draygon_var_0B).wrapping_sub(1u16);
            (*E).draygon_var_0B = v6;
            (*E).draygon_var_08 = ((*(&raw mut (*E).draygon_var_08 as *mut u8).offset(0))
                .wrapping_add(*(&raw mut v6 as *mut u8).offset(1isize)))
                as u16;
            draygon_var_0A = (*E).draygon_var_0A;
            (*E).draygon_var_0A = (draygon_var_0A).wrapping_sub(0x4000u16);
            v8 = ((((*E).draygon_var_07 as u32) << 16i32 | draygon_var_0A as u32)
                .wrapping_sub(0x4000u32)
                >> 16i32) as u16;
            (*E).draygon_var_07 = v8;
            v8 as i32 - 64i32 & 0x8000i32 != 0
        } {
            (*E).draygon_var_A = ((0xa59105i32) as u16);
        } else {
            Draygon_Func_40(k, game);
        }
    } else {
        (*E).draygon_var_0C = 64u16;
        (*E).draygon_var_A = ((0xa590d4i32) as u16);
    };
}
pub unsafe fn Draygon_Func_26(k: u16, game: &mut Game) {
    let v0: u16 = k;
    let mut v3: u16 = 0;
    Draygon_Func_40(v0, game);
    let E: *mut Enemy_Draygon = Get_Draygon(0i32, game);
    let v2: u16 = ((*E).draygon_var_0C).wrapping_sub(1u16);
    (*E).draygon_var_0C = v2;
    if v2 != 0 {
        if v2 == 63u16 {
            v3 = addr_kDraygon_Ilist_9AE8 as u16;
            if (*E).draygon_var_20 != 0 {
                v3 = addr_kDraygon_Ilist_9EA1 as u16;
            }
            let E2: *mut Enemy_Draygon = Get_Draygon(0x80i32, game);
            (*E2).base.current_instruction = v3;
            (*E2).base.instruction_timer = 1u16;
        }
    } else {
        (*E).draygon_var_A = ((0xa58fd6i32) as u16);
    };
}
pub unsafe fn Draygon_Func_27(k: u16, game: &mut Game) {
    Draygon_Func_40(k, game);
    let mut v1: u16 = addr_kDraygon_Ilist_9A68 as u16;
    if (*Get_Draygon(k as i32, game)).draygon_var_20 != 0 {
        v1 = addr_kDraygon_Ilist_9E21 as u16;
    }
    let E: *mut Enemy_Draygon = Get_Draygon(0x80i32, game);
    (*E).base.current_instruction = v1;
    (*E).base.instruction_timer = 1u16;
    (*Get_Draygon(0i32, game)).draygon_var_A = ((0xa59124i32) as u16);
}
pub unsafe fn Draygon_Func_28(k: u16, game: &mut Game) {
    Draygon_Func_40(k, game);
}
pub unsafe fn Draygon_Func_29(game: &mut Game) {
    Samus_ReleaseFromDraygon_(game);
    set_ram_word!(game.samus_grapple_flags = 0u16);
    let E: *mut Enemy_Draygon = Get_Draygon(0i32, game);
    (*E).draygon_var_A = ((0xa59154i32) as u16);
    (*E).base.properties = ((*E).base.properties as i32 & !(kEnemyProps_Tangible as i32)) as u16;
    let mut v1: u16 = addr_kDraygon_Ilist_9B5A as u16;
    if (*E).draygon_var_20 != 0 {
        v1 = addr_kDraygon_Ilist_9F15 as u16;
    }
    let E2: *mut Enemy_Draygon = Get_Draygon(0x80i32, game);
    (*E2).base.current_instruction = v1;
    (*E2).base.instruction_timer = 1u16;
}
pub unsafe fn Draygon_Func_30(game: &mut Game) {
    let mut v1: i16 = 0;
    Draygon_Func_3(game);
    let E: *mut Enemy_Draygon = Get_Draygon(0i32, game);
    v1 = (((*E).base.y_pos) as i16).wrapping_sub(4i16);
    (*E).base.y_pos = v1 as u16;
    if v1 < 0i16 {
        (*E).base.properties =
            ((*E).base.properties as i32 & !(kEnemyProps_Tangible as i32)) as u16;
        (*E).draygon_var_A = ((0xa587f4i32) as u16);
        (*E).draygon_var_B = 0u16;
        (*E).base.x_pos = (*E).draygon_var_00;
        (*E).base.y_pos = (*E).draygon_var_01;
    }
}
pub unsafe fn Draygon_Func_31(game: &mut Game) {
    let E: *mut Enemy_Draygon = Get_Draygon(0i32, game);
    if ((*Get_Draygon(ram_word!(game, cur_enemy_index) as i32, game))
        .base
        .frame_counter as i32
        & 0xfi32)
        == 0i32
    {
        let x: u16 = (if (*E).draygon_var_20 != 0u16 {
            (*E).base.x_pos as i32 + 32i32
        } else {
            (*E).base.x_pos as i32 - 32i32
        }) as u16;
        let y: u16 = ((*E).base.y_pos).wrapping_sub(16u16);
        CreateSpriteAtPos(x, y, 61u16, 0u16, game);
    }
    (*E).draygon_var_43 = ((64u8).wrapping_sub(
        (CalculateAngleFromXY(
            (64i32 - ((*E).base.x_pos as i32 >> 2i32)) as u16,
            (120i32 - ((*E).base.y_pos as i32 >> 2i32)) as u16,
            game,
        )) as u8,
    )) as u16;
    let pt: Point32 = ConvertAngleToXy(
        *(&raw mut (*E).draygon_var_43 as *mut u8).offset(0) as u16,
        1u16,
        game,
    );
    EnemyFunc_B691(
        *(&raw mut (*E).draygon_var_43 as *mut u8).offset(0) as u16,
        pt,
        game,
    );
    let v3: u16 = Abs16(((*E).base.x_pos).wrapping_sub(256u16));
    if v3 as i32 - 4i32 & 0x8000i32 != 0 {
        let v4: u16 = Abs16(((*E).base.y_pos).wrapping_sub(480u16));
        if v4 as i32 - 4i32 & 0x8000i32 != 0 {
            Draygon_Func_43(game);
            (*E).draygon_var_A = ((0xa59294i32) as u16);
            QueueMusic_Delayed8(3u16, game);
            (*E).draygon_var_B = 416u16;
            (*E).base.current_instruction = addr_kDraygon_Ilist_97B9 as u16;
            (*E).base.instruction_timer = 1u16;
            let v5: u16 = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
            (*Get_Draygon(0x80i32, game)).base.properties = v5;
            (*Get_Draygon(0xc0i32, game)).base.properties = v5;
            let mut v6: u16 = addr_kDraygon_Ilist_999C as u16;
            if (*E).draygon_var_20 != 0 {
                v6 = addr_kDraygon_Ilist_9D3E as u16;
            }
            let E1: *mut Enemy_Draygon = Get_Draygon(0x40i32, game);
            (*E1).base.current_instruction = v6;
            (*E1).base.instruction_timer = 1u16;
            (*E1).draygon_var_A = ((0xa5804bi32) as u16);
        }
    }
}
pub unsafe fn Draygon_Func_32(game: &mut Game) {
    Draygon_Func_34(game);
    let E: *mut Enemy_Draygon = Get_Draygon(0i32, game);
    (*E).draygon_var_B = (*E).draygon_var_B.wrapping_sub(1);
    if (*E).draygon_var_B != 0 {
        Draygon_Func_42(0u16, game);
    } else {
        (*E).draygon_var_A = ((0xa592abi32) as u16);
    };
}
pub unsafe fn Draygon_Func_33(game: &mut Game) {
    Draygon_Func_34(game);
    Draygon_Func_42(0u16, game);
    let E: *mut Enemy_Draygon = Get_Draygon(0i32, game);
    (*E).base.y_pos = (*E).base.y_pos.wrapping_add(1);
    if (*E).base.y_pos as i32 - 576i32 & 0x8000i32 == 0 {
        (*E).base.current_instruction = addr_kDraygon_Ilist_98ED as u16;
        (*E).base.instruction_timer = 1u16;
        let v1: u16 = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
        (*E).base.properties = v1;
        (*Get_Draygon(0x40i32, game)).base.properties = v1;
        let v2: u16 = ram_word!(game, area_index);
        let ref mut fresh3 = *((&raw mut game.g_ram as *mut u8)
            .offset(0xd828)
            .offset(ram_word!(game, area_index) as isize) as *mut u8
            as *mut u16);
        *fresh3 = (*fresh3) | (1u16);
        Enemy_ItemDrop_Draygon(v2, game);
        Draygon_Func_44(game);
    }
}
pub unsafe fn Draygon_Func_34(game: &mut Game) {
    if (ram_word!(game, nmi_frame_counter_word) as i32 & 7i32) == 0i32 {
        let x: u16 = ((NextRandom(game) as i32 & 0x7fi32) + 192i32) as u16;
        let y: u16 =
            ((((ram_word!(game, random_number)) & (0x3f00u16)) as i32 >> 8i32) + 400i32) as u16;
        CreateSpriteAtPos(x, y, 21u16, 0u16, game);
    }
}
pub unsafe fn Draygon_Func_35(game: &mut Game) {
    let mut Draygon: *mut Enemy_Draygon =
        Get_Draygon(ram_word!(game, cur_enemy_index) as i32, game);
    if ((*Draygon).base.frame_counter as i32 & 0x7fi32) == 0i32 {
        Draygon = Get_Draygon(0i32, game);
        CreateSpriteAtPos(
            ((*Draygon).base.x_pos).wrapping_sub(16u16),
            ((*Draygon).base.y_pos).wrapping_sub(16u16),
            24u16,
            0u16,
            game,
        );
    }
}
pub unsafe fn Draygon_Func_36(game: &mut Game) {
    let E: *mut Enemy_Draygon = Get_Draygon(0i32, game);
    set_ram_word!(
        game.reg_BG2HOFS = ((*E).draygon_var_5E as i32 + ram_word!(game, layer1_x_pos) as i32
            - (*E).base.x_pos as i32
            - 450i32) as u16
    );
    set_ram_word!(
        game.reg_BG2VOFS = ((*E).draygon_var_5F as i32 + ram_word!(game, layer1_y_pos) as i32
            - (*E).base.y_pos as i32
            - 192i32) as u16
    );
}
pub unsafe fn Draygon_Func_37(game: &mut Game) {
    let E: *mut Enemy_Draygon = Get_Draygon(0i32, game);
    if (ram_word!(game, joypad2_last) as i32 & 0x40i32) != 0i32 {
        Draygon_Func_39(game);
    } else {
        Draygon_Func_38(game);
    }
    if (ram_word!(game, joypad2_new_keys) as i32 & 0x4000i32) != 0i32 {
        let mut v0: u16 = addr_kDraygon_Ilist_98FE as u16;
        if (*E).draygon_var_20 != 0 {
            v0 = addr_kDraygon_Ilist_9C90 as u16;
        }
        (*E).base.current_instruction = v0;
        (*E).base.instruction_timer = 1u16;
    } else if (ram_word!(game, joypad2_new_keys) as i32 & 0x8000i32) == 0i32 {
        if (ram_word!(game, joypad2_new_keys) as i32 & 0x80i32) != 0i32 {
            let mut v4: u16 = addr_kDraygon_Ilist_9825 as u16;
            if (*E).draygon_var_20 != 0 {
                v4 = addr_kDraygon_Ilist_9C18 as u16;
            }
            let E3: *mut Enemy_Draygon = Get_Draygon(0xc0i32, game);
            (*E3).base.current_instruction = v4;
            (*E3).base.instruction_timer = 1u16;
        }
    } else {
        let mut v2: u16 = addr_kDraygon_Ilist_99FC as u16;
        if (*E).draygon_var_20 != 0 {
            v2 = addr_kDraygon_Ilist_9D9E as u16;
        }
        let E2: *mut Enemy_Draygon = Get_Draygon(0x80i32, game);
        (*E2).base.current_instruction = v2;
        (*E2).base.instruction_timer = 1u16;
    };
}
pub unsafe fn Draygon_Func_38(game: &mut Game) {
    let current_block: u64;
    let E: *mut Enemy_Draygon = Get_Draygon(0i32, game);
    let v0: u16 = ram_word!(game, joypad2_last);
    let mut v2: u16 = 0;
    if (ram_word!(game, joypad2_last) as i32 & 0x200i32) == 0i32 || {
        (*E).base.x_pos = (*E).base.x_pos.wrapping_sub(1);
        v0 == ram_word!(game, debug_enemy_population_pointer)
    } {
        if (v0 as i32 & 0x100i32) == 0i32 {
            current_block = 1196177745739367891;
        } else {
            (*E).base.x_pos = (*E).base.x_pos.wrapping_add(1);
            if v0 == ram_word!(game, debug_enemy_population_pointer) {
                current_block = 1196177745739367891;
            } else {
                set_ram_word!(game.debug_enemy_population_pointer = v0);
                v2 = addr_kDraygon_Ilist_97D1 as u16;
                (*E).draygon_var_20 = 1u16;
                current_block = 3276175668257526147;
            }
        }
    } else {
        set_ram_word!(game.debug_enemy_population_pointer = v0);
        v2 = addr_kDraygon_Ilist_97BB as u16;
        (*E).draygon_var_20 = 0u16;
        current_block = 3276175668257526147;
    }
    if current_block == 3276175668257526147 {
        (*E).base.current_instruction = v2;
        (*E).base.instruction_timer = 1u16;
    }
    if (ram_word!(game, joypad2_last) as i32 & 0x800i32) != 0i32 {
        (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
    } else if (ram_word!(game, joypad2_last) as i32 & 0x400i32) != 0i32 {
        (*E).base.y_pos = (*E).base.y_pos.wrapping_add(1);
    }
}
pub unsafe fn Draygon_Func_39(game: &mut Game) {
    let current_block: u64;
    let E: *mut Enemy_Draygon = Get_Draygon(0i32, game);
    let mut v0: u16 = ram_word!(game, joypad2_last);
    let mut v2: u16 = 0;
    if (ram_word!(game, joypad2_last) as i32 & 0x200i32) == 0i32 || {
        (*E).base.x_pos = ((*E).base.x_pos).wrapping_sub(4u16);
        v0 = ram_word!(game, joypad2_last);
        ram_word!(game, joypad2_last) == ram_word!(game, debug_enemy_population_pointer)
    } {
        if (v0 as i32 & 0x100i32) == 0i32 {
            current_block = 18201098939790280272;
        } else {
            (*E).base.x_pos = ((*E).base.x_pos).wrapping_add(4u16);
            if ram_word!(game, joypad2_last) == ram_word!(game, debug_enemy_population_pointer) {
                current_block = 18201098939790280272;
            } else {
                set_ram_word!(game.debug_enemy_population_pointer = ram_word!(game, joypad2_last));
                v2 = addr_kDraygon_Ilist_97D1 as u16;
                (*E).draygon_var_20 = 1u16;
                current_block = 3276175668257526147;
            }
        }
    } else {
        set_ram_word!(game.debug_enemy_population_pointer = ram_word!(game, joypad2_last));
        v2 = addr_kDraygon_Ilist_97BB as u16;
        (*E).draygon_var_20 = 0u16;
        current_block = 3276175668257526147;
    }
    if current_block == 3276175668257526147 {
        (*E).base.current_instruction = v2;
        (*E).base.instruction_timer = 1u16;
    }
    if (ram_word!(game, joypad2_last) as i32 & 0x800i32) != 0i32 {
        (*E).base.y_pos = ((*E).base.y_pos).wrapping_sub(4u16);
    } else if (ram_word!(game, joypad2_last) as i32 & 0x400i32) != 0i32 {
        (*E).base.y_pos = (*E).base.y_pos.wrapping_add(1);
        (*E).base.y_pos = ((*E).base.y_pos).wrapping_add(4u16);
    }
}
pub unsafe fn Draygon_Func_40(k: u16, game: &mut Game) {
    let mut v1: i16 = 0;
    v1 = 8i16;
    if (*Get_Draygon(k as i32, game)).draygon_var_20 == 0 {
        v1 = -(8i32) as i16;
    }
    let E: *mut Enemy_Draygon = Get_Draygon(0i32, game);
    set_ram_word!(game.samus_x_pos = ((*E).base.x_pos).wrapping_add((v1) as u16));
    set_ram_word!(game.samus_y_pos = ((*E).base.y_pos).wrapping_add(40u16));
    EnemyFunc_B7A1(game);
    if (ram_word!(game, samus_grapple_flags) as i32 & 2i32) != 0i32 {
        set_ram_word!(game.samus_grapple_flags = 0u16);
        (*E).draygon_var_A = ((0xa59154i32) as u16);
    }
}
pub unsafe fn Draygon_Instr_1(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_Draygon(0i32, game)).base.current_instruction = *jp.offset(0);
    (*Get_Draygon(0x40i32, game)).base.current_instruction = *jp.offset(1);
    (*Get_Draygon(0x80i32, game)).base.current_instruction = *jp.offset(2);
    (*Get_Draygon(0xc0i32, game)).base.current_instruction = *jp.offset(3);
    (*Get_Draygon(0i32, game)).base.instruction_timer = 1u16;
    (*Get_Draygon(0x40i32, game)).base.instruction_timer = 1u16;
    (*Get_Draygon(0x80i32, game)).base.instruction_timer = 1u16;
    (*Get_Draygon(0xc0i32, game)).base.instruction_timer = 1u16;
    jp.offset(4)
}
pub unsafe fn Draygon_Hurt(game: &mut Game) {
    let mut v0: u16 = addr_kDraygon_MorePalettes2 as u16;
    if ((*Get_Draygon(ram_word!(game, cur_enemy_index) as i32, game))
        .base
        .flash_timer as i32
        & 2i32)
        != 0i32
    {
        v0 = addr_word_A5A297 as u16;
    }
    memcpy(
        (&raw mut game.g_ram as *mut u8)
            .offset(addr_kDraygon_BigSprmap_C08F as i32 as isize)
            .offset(17) as *mut c_void,
        RomPtr_A5(v0, game) as *const c_void,
        32 as size_t,
    );
    if ((*Get_Draygon(ram_word!(game, cur_enemy_index) as i32, game))
        .base
        .flash_timer as i32
        & 2i32)
        == 0i32
    {
        let mut v3: u16 = (4u16).wrapping_mul((*Get_Draygon(0i32, game)).draygon_var_0E);
        let mut i: i32 = 0i32;
        while i != 8i32 {
            set_ram_word!(
                game.palette_buffer[((i >> 1i32) + 89i32)] =
                    crate::rtl::rom_table_word!(game, 0xa596af, (v3 as i32 >> 1i32) as isize)
            );
            v3 = (v3).wrapping_add(2u16);
            i += 2i32;
        }
    }
    let mut v5: u16 = addr_kDraygon_Palette as u16;
    if ((*Get_Draygon(ram_word!(game, cur_enemy_index) as i32, game))
        .base
        .flash_timer as i32
        & 2i32)
        != 0i32
    {
        v5 = addr_word_A5A297 as u16;
    }
    memcpy(
        (&raw mut game.g_ram as *mut u8)
            .offset(addr_kDraygon_BigSprmap_C11B as i32 as isize)
            .offset(197) as *mut c_void,
        RomPtr_A5(v5, game) as *const c_void,
        32 as size_t,
    );
    if (ram_word!(game, samus_grapple_flags) as i32 & 1i32) != 0i32 {
        let E: *mut Enemy_Draygon = Get_Draygon(0i32, game);
        if ((*E).base.frame_counter as i32 & 7i32) == 0i32 {
            let mut v9: i16 = (((*E).base.health) as i16).wrapping_sub(256i16);
            if v9 < 0i16 {
                v9 = 0i16;
            }
            (*E).base.health = v9 as u16;
            sub_A5960D(game);
        }
    }
}
pub unsafe fn Draygon_Touch(game: &mut Game) {
    NormalEnemyTouchAiSkipDeathAnim_CurEnemy(game);
    sub_A5960D(game);
}
pub unsafe fn Draygon_Shot(game: &mut Game) {
    let E: *mut Enemy_Draygon = Get_Draygon(0i32, game);
    if (*E).draygon_var_0F as i32 - 152i32 & 0x8000i32 != 0 {
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
    if (*Get_Draygon(ram_word!(game, cur_enemy_index) as i32, game))
        .base
        .health
        != 0
    {
        Draygon_Func_41(game);
    } else {
        set_ram_word!(game.grapple_beam_function = ((0x9bc8c5i32) as u16));
        let mut v0: u16 = addr_kDraygon_Ilist_9867 as u16;
        let E: *mut Enemy_Draygon = Get_Draygon(0i32, game);
        if (*E).draygon_var_20 != 0 {
            v0 = addr_kDraygon_Ilist_9C5A as u16;
        }
        (*E).base.current_instruction = v0;
        (*E).base.instruction_timer = 1u16;
        let mut v3: u16 = addr_kDraygon_Ilist_997A as u16;
        if (*E).draygon_var_20 != 0 {
            v3 = addr_kDraygon_Ilist_9D1C as u16;
        }
        let E1: *mut Enemy_Draygon = Get_Draygon(0x40i32, game);
        (*E1).base.current_instruction = v3;
        (*E1).base.instruction_timer = 1u16;
        (*E1).draygon_var_A = ((0xa5804bi32) as u16);
        (*E).draygon_var_A = ((0xa59185i32) as u16);
        Samus_ReleaseFromDraygon_(game);
        set_ram_word!(game.samus_grapple_flags = 0u16);
        (*E).draygon_var_43 = ((64u8).wrapping_sub(
            (CalculateAngleFromXY(
                (256u16).wrapping_sub((*E).base.x_pos),
                (480u16).wrapping_sub((*E).base.y_pos),
                game,
            )) as u8,
        )) as u16;
        let pt: Point32 = ConvertAngleToXy(
            *(&raw mut (*E).draygon_var_43 as *mut u8).offset(0) as u16,
            1u16,
            game,
        );
        (*E).draygon_var_28 = (pt.x >> 16i32) as u16;
        (*E).draygon_var_29 = pt.x as u16;
        (*E).draygon_var_2A = (pt.y >> 16i32) as u16;
        (*E).draygon_var_2B = pt.y as u16;
    };
}
pub unsafe fn Draygon_Func_41(game: &mut Game) {
    let mut i: i32 = 0;
    let mut j: u16 = 0;
    let E: *mut Enemy_Draygon = Get_Draygon(0i32, game);
    i = 0i32;
    while ((((*E).base.health) as i16)
        .wrapping_sub((crate::rtl::rom_table_word!(game, 0xa596ef, (i >> 1i32) as isize)) as i16)
        < 0i16)
    {
        i += 2i32;
    }
    if i != (*E).draygon_var_0E as i32 {
        (*E).draygon_var_0E = i as u16;
        let mut v2: u16 = (4u16).wrapping_mul((*E).draygon_var_0E);
        j = 0u16;
        while j != 8u16 {
            *((&raw mut game.g_ram as *mut u8).offset(0xc000) as *mut u16)
                .offset(((j as i32 >> 1i32) + 89i32) as isize) =
                crate::rtl::rom_table_word!(game, 0xa596af, (v2 as i32 >> 1i32) as isize);
            v2 = (v2).wrapping_add(2u16);
            j = (j).wrapping_add(2u16);
        }
    }
}
pub unsafe fn Draygon_Instr_13(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_Draygon(k as i32, game)).draygon_var_A = *jp.offset(0);
    jp.offset(1)
}
unsafe fn Draygon_Func_47(game: &mut Game) -> Point16U {
    NextRandom(game);
    let E: *mut Enemy_Draygon = Get_Draygon(0i32, game);
    let r18: u16 = ((ram_word!(game, random_number) as i32 & 0x7fi32) - 64i32) as u16;
    let r20: u16 = (((ram_word!(game, random_number) as i32 & 0x7f00i32) >> 8i32) - 64i32) as u16;
    Point16U {
        x: ((r18).wrapping_add((*E).base.x_pos)),
        y: ((r20).wrapping_add((*E).base.y_pos)),
    }
}
pub unsafe fn Draygon_Instr_8(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let pt: Point16U = Draygon_Func_47(game);
    CreateSpriteAtPos(pt.x, pt.y, 21u16, 0u16, game);
    jp
}
pub unsafe fn Draygon_Instr_7(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let pt: Point16U = Draygon_Func_47(game);
    CreateSpriteAtPos(pt.x, pt.y, 3u16, 0u16, game);
    jp
}
pub unsafe fn Draygon_Instr_6(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let pt: Point16U = Draygon_Func_47(game);
    CreateSpriteAtPos(pt.x, pt.y, 29u16, 0u16, game);
    jp
}
pub unsafe fn Draygon_Instr_9(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let pt: Point16U = Draygon_Func_47(game);
    CreateSpriteAtPos(pt.x, pt.y, 24u16, 0u16, game);
    jp
}
pub unsafe fn Draygon_Instr_2(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    set_ram_word!(game.room_loading_irq_handler = 12u16);
    jp
}
pub unsafe fn Draygon_Instr_11(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E2: *mut Enemy_Draygon = Get_Draygon(0x80i32, game);
    (*E2).base.instruction_timer = 1u16;
    (*E2).base.current_instruction = addr_kDraygon_Ilist_97B9 as u16;
    let E3: *mut Enemy_Draygon = Get_Draygon(0xc0i32, game);
    (*E3).base.instruction_timer = 1u16;
    (*E3).base.current_instruction = addr_kDraygon_Ilist_97B9 as u16;
    jp
}
pub unsafe fn Draygon_Instr_5(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E: *mut Enemy_Draygon = Get_Draygon(0i32, game);
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
    jp
}
pub unsafe fn Draygon_Instr_15(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E: *mut Enemy_Draygon = Get_Draygon(0i32, game);
    (*E).draygon_var_0F = 24u16;
    let v3: *const u8 = RomPtr_A0((*E).base.enemy_ptr, game);
    Samus_DealDamage(SuitDamageDivision(*(v3.offset(6) as *mut u16), game), game);
    set_ram_word!(game.earthquake_timer = 32u16);
    set_ram_word!(game.earthquake_type = 7u16);
    CreateSpriteAtPos(
        ram_word!(game, samus_x_pos),
        (ram_word!(game, samus_y_pos)).wrapping_add(16u16),
        21u16,
        0u16,
        game,
    );
    jp
}
pub unsafe fn Draygon_Instr_17(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    set_ram_word!(game.room_loading_irq_handler = 12u16);
    jp
}
pub unsafe fn Draygon_Instr_14(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E: *mut Enemy_Draygon = Get_Draygon(0i32, game);
    (*E).draygon_var_5E = *jp.offset(0);
    (*E).draygon_var_5F = *jp.offset(1);
    jp.offset(2)
}
pub unsafe fn Draygon_Instr_16(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_Draygon(0i32, game)).draygon_var_A = *jp.offset(0);
    jp.offset(1)
}
pub unsafe fn Draygon_Instr_10(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    QueueSfx2_Max6(*jp.offset(0), game);
    jp.offset(1)
}
pub unsafe fn Draygon_Instr_4(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    QueueSfx3_Max6(*jp.offset(0), game);
    jp.offset(1)
}
pub unsafe fn Draygon_Instr_12(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E: *mut Enemy_Draygon = Get_Draygon(0i32, game);
    game.eproj_spawn_pt = Point16U {
        x: (((*E).base.x_pos).wrapping_sub(28u16)),
        y: (((*E).base.y_pos).wrapping_sub(16u16)),
    };
    set_ram_word!(game.cinematic_var9 = ((NextRandom(game) as i32 & 0x3fi32) + 128i32) as u16);
    SpawnEprojWithGfx(
        2u16,
        ram_word!(game, cur_enemy_index),
        addr_stru_868E50 as u16,
        game,
    );
    jp
}
pub unsafe fn Draygon_Instr_18(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E: *mut Enemy_Draygon = Get_Draygon(0i32, game);
    game.eproj_spawn_pt = Point16U {
        x: (((*E).base.x_pos).wrapping_add(24u16)),
        y: (((*E).base.y_pos).wrapping_sub(16u16)),
    };
    set_ram_word!(game.cinematic_var9 = ((NextRandom(game) as i32 & 0x3fi32) + 192i32) as u16);
    SpawnEprojWithGfx(
        2u16,
        ram_word!(game, cur_enemy_index),
        addr_stru_868E50 as u16,
        game,
    );
    jp
}
pub unsafe fn Draygon_Func_42(varE24: u16, game: &mut Game) {
    let mut v0: u16 = 62u16;
    let mut i: i32 = 20i32;
    while i >= 0i32 {
        let v2: i32 = i >> 1i32;
        let v3: i32 = v0 as i32 >> 1i32;
        if (crate::rtl::rom_table_word!(game, 0xa5a1df, v2 as isize) as i32 + 64i32 & 0x80i32)
            != 0i32
        {
            AddToHiLo(
                ((&raw mut game.g_ram as *mut u8).offset(0xf0f8) as *mut u16).offset(v3 as isize)
                    as *mut u16,
                ((&raw mut game.g_ram as *mut u8).offset(0xf178) as *mut u16).offset(v3 as isize)
                    as *mut u16,
                crate::rtl::rom_table_word!(game, 0xa5a1af, v2 as isize) as u32,
            );
        } else {
            AddToHiLo(
                ((&raw mut game.g_ram as *mut u8).offset(0xf0f8) as *mut u16).offset(v3 as isize)
                    as *mut u16,
                ((&raw mut game.g_ram as *mut u8).offset(0xf178) as *mut u16).offset(v3 as isize)
                    as *mut u16,
                (-(crate::rtl::rom_table_word!(game, 0xa5a1af, v2 as isize) as i32)
                    - ((varE24 as i32) << 16i32)) as u32,
            );
        }
        let v8: u16 = ram_word!(game, sprite_y_subpos[v3]);
        if (crate::rtl::rom_table_word!(game, 0xa5a1df, v2 as isize) as i32 + 128i32 & 0x80i32)
            != 0i32
        {
            AddToHiLo(
                ((&raw mut game.g_ram as *mut u8).offset(0xf1f8) as *mut u16).offset(v3 as isize)
                    as *mut u16,
                ((&raw mut game.g_ram as *mut u8).offset(0xf278) as *mut u16).offset(v3 as isize)
                    as *mut u16,
                crate::rtl::rom_table_word!(game, 0xa5a1af, (v2 + 1i32) as isize) as u32,
            );
        } else {
            AddToHiLo(
                ((&raw mut game.g_ram as *mut u8).offset(0xf1f8) as *mut u16).offset(v3 as isize)
                    as *mut u16,
                ((&raw mut game.g_ram as *mut u8).offset(0xf278) as *mut u16).offset(v3 as isize)
                    as *mut u16,
                -(crate::rtl::rom_table_word!(game, 0xa5a1af, (v2 + 1i32) as isize) as i32) as u32,
            );
        }
        v0 = (v0).wrapping_sub(2u16);
        i -= 4i32;
    }
}
pub unsafe fn Draygon_Func_43(game: &mut Game) {
    let mut v1: i16 = 0;
    let mut j: i16 = 0;
    let mut i: i32 = 62i32;
    while i >= 0i32 {
        set_ram_word!(game.sprite_instr_list_ptrs[(i >> 1i32)] = 0u16);
        i -= 2i32;
    }
    v1 = 2i16;
    let mut v2: u16 = 20u16;
    loop {
        let v3: i32 = v2 as i32 >> 1i32;
        CreateSpriteAtPos(
            crate::rtl::rom_table_word!(game, 0xa5a1c7, v3 as isize),
            crate::rtl::rom_table_word!(game, 0xa5a1c7, (v3 + 1i32) as isize),
            59u16,
            0xe00u16,
            game,
        );
        v2 = (v2).wrapping_sub(4u16);
        v1 -= 1;
        if (v1 < 0i16) {
            break;
        }
    }
    j = 2i16;
    while j >= 0i16 {
        let v5: i32 = v2 as i32 >> 1i32;
        CreateSpriteAtPos(
            crate::rtl::rom_table_word!(game, 0xa5a1c7, v5 as isize),
            crate::rtl::rom_table_word!(game, 0xa5a1c7, (v5 + 1i32) as isize),
            60u16,
            0xe00u16,
            game,
        );
        v2 = (v2).wrapping_sub(4u16);
        j -= 1;
    }
}
pub unsafe fn Draygon_Func_44(game: &mut Game) {
    let mut i: i32 = 62i32;
    while i >= 0i32 {
        set_ram_word!(game.sprite_instr_list_ptrs[(i >> 1i32)] = 0u16);
        i -= 2i32;
    }
}
pub unsafe fn Draygon_Func_45(game: &mut Game) {
    CreateSpriteAtPos(16u16, 384u16, 59u16, 0xe00u16, game);
    CreateSpriteAtPos(16u16, 384u16, 59u16, 0xe00u16, game);
    CreateSpriteAtPos(16u16, 384u16, 59u16, 0xe00u16, game);
    CreateSpriteAtPos(16u16, 384u16, 59u16, 0xe00u16, game);
}
pub unsafe fn Draygon_Func_46(game: &mut Game) {
    let mut v0: u16 = 62u16;
    loop {
        let v1: u16 =
            ((*Get_Draygon(0i32, game)).draygon_var_46).wrapping_add(crate::rtl::rom_table_word!(
                game,
                0xa5a19f,
                (((v0).wrapping_sub(56u16)) as i32 >> 1i32) as isize
            ));
        if (v1 as i32 & 0x8000i32) == 0i32 {
            let v2: u16 = v1;
            if *((game.g_rom.offset(crate::rtl::lorom_offset(0xa5ce07)) as *const u8 as *mut u8)
                .offset(v1 as isize) as *mut u8 as *mut u16)
                == 0x8080u16
            {
                set_ram_word!(game.sprite_instr_list_ptrs[(v0 as i32 >> 1i32)] = 0u16);
            } else {
                let v3: i32 = v0 as i32 >> 1i32;
                update_ram_word!(
                    game.sprite_x_pos[v3],
                    wrapping_add(SignExtend8(
                        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa5ce07)) as *const u8
                            as *mut u8)
                            .offset(v1 as isize) as u16,
                    ))
                );
                update_ram_word!(
                    game.sprite_y_pos[v3],
                    wrapping_add(SignExtend8(
                        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa5ce07)) as *const u8
                            as *mut u8)
                            .offset((v2 as i32 + 1i32) as isize) as u16,
                    ))
                );
            }
        }
        v0 = (v0).wrapping_sub(2u16);
        if (((v0) as i16).wrapping_sub(56i16) < 0i16) {
            break;
        }
    }
    let E: *mut Enemy_Draygon = Get_Draygon(0i32, game);
    (*E).draygon_var_46 = ((*E).draygon_var_46).wrapping_add(4u16);
}
pub unsafe fn DraygonsEye_Init(game: &mut Game) {
    let E: *mut Enemy_Draygon = Get_Draygon(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.current_instruction = addr_kDraygon_Ilist_9944 as u16;
    (*E).draygon_var_A = ((0xa5804bi32) as u16);
}
pub unsafe fn Draygon_Instr_3(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_Draygon(0x40i32, game)).draygon_var_A = *jp.offset(0);
    jp.offset(1)
}
pub unsafe fn DraygonsEye_Main(game: &mut Game) {
    let E: *mut Enemy_Draygon = Get_Draygon(ram_word!(game, cur_enemy_index) as i32, game);
    CallDraygonFunc(
        (((*E).draygon_var_A) as u32) | (0xa50000u32),
        ram_word!(game, cur_enemy_index),
        game,
    );
}
pub unsafe fn Draygon_Func_48(k: u16, game: &mut Game) {
    if ((*Get_Draygon(k as i32, game)).base.frame_counter as i32 & 0x7fi32) == 0i32 {
        let E1: *mut Enemy_Draygon = Get_Draygon(0x40i32, game);
        game.eproj_spawn_pt = Point16U {
            x: (((*E1).base.x_pos).wrapping_sub(24u16)),
            y: (((*E1).base.y_pos).wrapping_sub(32u16)),
        };
        SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 0x18u16, game);
    }
    let E0: *mut Enemy_Draygon = Get_Draygon(0i32, game);
    let v4: u16 = CalculateAngleFromXY(
        (ram_word!(game, samus_x_pos) as i32 - ((*E0).base.x_pos as i32 - 24i32)) as u16,
        (ram_word!(game, samus_y_pos) as i32 - ((*E0).base.y_pos as i32 - 32i32)) as u16,
        game,
    );
    let E: *mut Enemy_Draygon = Get_Draygon(ram_word!(game, cur_enemy_index) as i32, game);
    if v4 != (*E).draygon_var_F {
        let mut v6: u16 = addr_kDraygon_Ilist_99BA as u16;
        if v4 as i32 - 32i32 & 0x8000i32 == 0 {
            v6 = addr_kDraygon_Ilist_99B4 as u16;
            if v4 as i32 - 96i32 & 0x8000i32 == 0 {
                v6 = addr_kDraygon_Ilist_99C0 as u16;
                if v4 as i32 - 160i32 & 0x8000i32 == 0 {
                    v6 = addr_kDraygon_Ilist_99AE as u16;
                    if v4 as i32 - 224i32 & 0x8000i32 == 0 {
                        v6 = addr_kDraygon_Ilist_99BA as u16;
                    }
                }
            }
        }
        (*E).draygon_var_F = v4;
        (*E).base.current_instruction = v6;
        (*E).base.instruction_timer = 1u16;
        (*E).base.timer = 0u16;
    }
}
pub unsafe fn Draygon_Func_49(k: u16, game: &mut Game) {
    let E0: *mut Enemy_Draygon = Get_Draygon(0i32, game);
    if ((*Get_Draygon(k as i32, game)).base.frame_counter as i32 & 0x7fi32) == 0i32 {
        game.eproj_spawn_pt = Point16U {
            x: (((*E0).base.x_pos).wrapping_add(24u16)),
            y: (((*E0).base.y_pos).wrapping_sub(32u16)),
        };
        SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 0x18u16, game);
    }
    let v4: u16 = CalculateAngleFromXY(
        (ram_word!(game, samus_x_pos) as i32 - ((*E0).base.x_pos as i32 + 24i32)) as u16,
        (ram_word!(game, samus_y_pos) as i32 - ((*E0).base.y_pos as i32 - 32i32)) as u16,
        game,
    );
    let E: *mut Enemy_Draygon = Get_Draygon(ram_word!(game, cur_enemy_index) as i32, game);
    if v4 != (*E).draygon_var_F {
        let mut v6: u16 = addr_kDraygon_Ilist_9D5C as u16;
        if v4 as i32 - 32i32 & 0x8000i32 == 0 {
            v6 = addr_kDraygon_Ilist_9D50 as u16;
            if v4 as i32 - 96i32 & 0x8000i32 == 0 {
                v6 = addr_kDraygon_Ilist_9D62 as u16;
                if v4 as i32 - 160i32 & 0x8000i32 == 0 {
                    v6 = addr_kDraygon_Ilist_9D56 as u16;
                    if v4 as i32 - 224i32 & 0x8000i32 == 0 {
                        v6 = addr_kDraygon_Ilist_9D5C as u16;
                    }
                }
            }
        }
        (*E).draygon_var_F = v4;
        (*E).base.current_instruction = v6;
        (*E).base.instruction_timer = 1u16;
        (*E).base.timer = 0u16;
    }
}
pub unsafe fn DraygonsTail_Init(game: &mut Game) {
    let v0: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    (*v0).current_instruction = addr_kDraygon_Ilist_99FC as u16;
    (*v0).palette_index = 0xe00u16;
}
pub unsafe fn DraygonsArms_Init(game: &mut Game) {
    let v0: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    (*v0).current_instruction = addr_kDraygon_Ilist_97E7 as u16;
    (*v0).palette_index = 0xe00u16;
    (*v0).layer = 2u16;
}
pub unsafe fn Draygon_Instr_25(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E: *mut Enemy_Draygon = Get_Draygon(0i32, game);
    if (*E).draygon_var_0B as i32 - 40i32 & 0x8000i32 != 0 {
        (*E).draygon_var_0B = ((*E).draygon_var_0B).wrapping_add(8u16);
    }
    jp
}
pub unsafe fn Draygon_Instr_24(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_Draygon(0i32, game)).draygon_var_2F = 0u16;
    jp
}
pub unsafe fn Draygon_Func_50(j: u16, game: &mut Game) {
    let v1: *const u8 = RomPtr_A5(j, game);
    let E: *mut Enemy_Draygon = Get_Draygon(0i32, game);
    (*E).draygon_var_0B = *(v1 as *mut u16);
    (*E).draygon_var_0C = *(v1.offset(2) as *mut u16);
    (*E).draygon_var_0A = *(v1.offset(4) as *mut u16);
}
pub unsafe fn Draygon_Instr_21(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E: *mut Enemy_Draygon = Get_Draygon(0i32, game);
    (*E).draygon_var_0B = *jp.offset(0);
    (*E).draygon_var_0C = *jp.offset(1);
    jp.offset(2)
}
pub unsafe fn Draygon_Func_51(j: u16, game: &mut Game) {
    let v1: u16 = *(RomPtr_A5(j, game) as *mut u16);
    (*Get_Draygon(0i32, game)).draygon_var_0B = v1;
}
pub unsafe fn Draygon_Func_52(j: u16, game: &mut Game) {
    let v1: u16 = *(RomPtr_A5(j, game) as *mut u16);
    (*Get_Draygon(0i32, game)).draygon_var_0C = v1;
}
pub unsafe fn Draygon_Instr_53(k: u16, j: u16, game: &mut Game) -> u16 {
    let E: *mut Enemy_Draygon = Get_Draygon(0i32, game);
    (*E).draygon_var_0B = ((*E).draygon_var_0B).wrapping_add(*(RomPtr_A5(j, game) as *mut u16));
    (j).wrapping_add(2u16)
}
pub unsafe fn Draygon_Instr_22(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    set_ram_word!(game.kraid_unk9000 = *jp.offset(0));
    jp.offset(1)
}
pub unsafe fn Draygon_Instr_27(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E: *mut Enemy_Draygon = Get_Draygon(0i32, game);
    (*E).base.x_pos = 128u16;
    (*E).base.y_pos = 624u16;
    (*E).base.properties = ((*E).base.properties as i32 & 0x5bffi32 | 0xa000i32) as u16;
    jp
}
pub unsafe fn Draygon_Instr_23(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    QueueSfx2_Max6(*jp.offset(0), game);
    jp.offset(1)
}
pub unsafe fn Draygon_Instr_30(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    Enemy_ItemDrop_SporeSpawn(k, game);
    jp
}
pub unsafe fn Draygon_Instr_20(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_Draygon(ram_word!(game, cur_enemy_index) as i32, game)).draygon_var_A = *jp.offset(0);
    jp.offset(1)
}
pub unsafe fn Draygon_Instr_29(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let v2: u16 = *jp.offset(0);
    let mut i: i32 = 0i32;
    while i != 32i32 {
        set_ram_word!(
            game.palette_buffer[((i >> 1i32) + 144i32)] =
                crate::rtl::rom_table_word!(game, 0xa5e3f9, (v2 as i32 + i >> 1i32) as isize)
        );
        i += 2i32;
    }
    let mut i_0: i32 = 0i32;
    while i_0 != 32i32 {
        set_ram_word!(
            game.palette_buffer[((i_0 >> 1i32) + 64i32)] =
                crate::rtl::rom_table_word!(game, 0xa5e4f9, (v2 as i32 + i_0 >> 1i32) as isize)
        );
        i_0 += 2i32;
    }
    let mut i_1: i32 = 0i32;
    while i_1 != 32i32 {
        set_ram_word!(
            game.palette_buffer[((i_1 >> 1i32) + 112i32)] =
                crate::rtl::rom_table_word!(game, 0xa5e5d9, (v2 as i32 + i_1 >> 1i32) as isize)
        );
        i_1 += 2i32;
    }
    jp.offset(1)
}
pub unsafe fn Draygon_Instr_19(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let v2: u16 = *jp.offset(0);
    let mut i: i32 = 0i32;
    while i != 32i32 {
        set_ram_word!(
            game.target_palettes[((i >> 1i32) + 144i32)] =
                crate::rtl::rom_table_word!(game, 0xa5e3f9, (v2 as i32 + i >> 1i32) as isize)
        );
        i += 2i32;
    }
    let mut i_0: i32 = 0i32;
    while i_0 != 32i32 {
        set_ram_word!(
            game.target_palettes[((i_0 >> 1i32) + 64i32)] =
                crate::rtl::rom_table_word!(game, 0xa5e4f9, (v2 as i32 + i_0 >> 1i32) as isize)
        );
        i_0 += 2i32;
    }
    let mut i_1: i32 = 0i32;
    while i_1 != 32i32 {
        set_ram_word!(
            game.target_palettes[((i_1 >> 1i32) + 112i32)] =
                crate::rtl::rom_table_word!(game, 0xa5e5d9, (v2 as i32 + i_1 >> 1i32) as isize)
        );
        i_1 += 2i32;
    }
    jp.offset(1)
}
pub unsafe fn Draygon_Instr_28(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    NextRandom(game);
    let mut r18: u16 = ((ram_word!(game, random_number) as i32 & 0x7fi32) - 64i32) as u16;
    let E: *mut Enemy_Draygon = Get_Draygon(0i32, game);
    r18 = (r18).wrapping_add((*E).base.x_pos);
    let mut r20: u16 =
        ((((ram_word!(game, random_number)) & (0x7f00u16)) as i32 >> 8i32) - 64i32) as u16;
    r20 = (r20).wrapping_add((*E).base.y_pos);
    game.eproj_spawn_pt = Point16U { x: r18, y: r20 };
    SpawnEprojWithRoomGfx(0xe509u16, 0x15u16, game);
    QueueSfx2_Max6(0x29u16, game);
    jp
}
pub unsafe fn Draygon_Instr_26(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    NextRandom(game);
    let E: *mut Enemy_Draygon = Get_Draygon(0i32, game);
    let x: u16 =
        ((*E).base.x_pos as i32 + (ram_word!(game, random_number) as i32 & 0x7fi32) - 64i32) as u16;
    let y: u16 = ((*E).base.y_pos as i32
        + (((ram_word!(game, random_number)) & (0x3f00u16)) as i32 >> 8i32)
        - 32i32) as u16;
    CreateSpriteAtPos(x, y, 3u16, 0u16, game);
    QueueSfx2_Max6(0x25u16, game);
    jp
}
pub unsafe fn sub_A5E9F5(game: &mut Game) {
    if (ram_word!(game, nmi_frame_counter_word) as i32 & 0xfi32) == 0i32 {
        NextRandom(game);
        let x: u16 = ((ram_word!(game, random_number) as i32 & 0x3fi32) + 96i32) as u16;
        let y: u16 =
            ((((ram_word!(game, random_number)) & (0xf00u16)) as i32 >> 8i32) + 480i32) as u16;
        CreateSpriteAtPos(x, y, 21u16, 0u16, game);
    }
}
pub unsafe fn SporeSpawn_Init(game: &mut Game) {
    let mut v0: u16 = 0u16;
    let mut i: i32 = 0i32;
    while i != 32i32 {
        set_ram_word!(
            game.target_palettes[((i >> 1i32) + 240i32)] =
                crate::rtl::rom_table_word!(game, 0xa5e359, (v0 as i32 >> 1i32) as isize)
        );
        v0 = (v0).wrapping_add(2u16);
        i += 2i32;
    }
    SpawnEprojWithGfx(
        0u16,
        ram_word!(game, cur_enemy_index),
        addr_kEproj_SporeSpawnsStalk as u16,
        game,
    );
    SpawnEprojWithGfx(
        1u16,
        ram_word!(game, cur_enemy_index),
        addr_kEproj_SporeSpawnsStalk as u16,
        game,
    );
    SpawnEprojWithGfx(
        2u16,
        ram_word!(game, cur_enemy_index),
        addr_kEproj_SporeSpawnsStalk as u16,
        game,
    );
    SpawnEprojWithGfx(
        3u16,
        ram_word!(game, cur_enemy_index),
        addr_kEproj_SporeSpawnsStalk as u16,
        game,
    );
    let E: *mut Enemy_SporeSpawn = Get_SporeSpawn(ram_word!(game, cur_enemy_index) as i32, game);
    let E0: *mut Enemy_SporeSpawn = Get_SporeSpawn(0i32, game);
    (*E).ssn_var_04 = (*E).base.x_pos;
    (*E).ssn_var_05 = ((*E).base.y_pos).wrapping_sub(72u16);
    (*E).ssn_var_C = (*E).base.x_pos;
    (*E).ssn_var_D = (*E).base.y_pos;
    (*E).ssn_var_F = 0u16;
    if (*(&raw mut game.g_ram as *mut u8)
        .offset(0xd828)
        .offset(ram_word!(game, area_index) as isize) as i32
        & 2i32)
        != 0i32
    {
        (*E0).base.current_instruction = addr_kDraygon_Ilist_E6B9 as u16;
        (*E0).ssn_var_A = ((0xa5eb1ai32) as u16);
        (*E0).base.properties = ((*E0).base.properties) | (0x8000u16);
        SporeSpawn_Func_5(game);
        SpawnHardcodedPlm(
            SpawnHardcodedPlmArgs::new(0x7u8, 0x1eu8, 0xb793 as VoidP),
            game,
        );
        set_ram_word!(game.scrolling_finished_hook = 0u16);
    } else {
        (*E0).base.current_instruction = addr_kDraygon_Ilist_E6C7 as u16;
        set_ram_word!(game.flag_process_all_enemies = -(1i32) as u16);
        let v5: u16 = ram_word!(game, cur_enemy_index);
        (*E).ssn_var_A = ((0xa5eb1ai32) as u16);
        set_ram_word!(game.scrolling_finished_hook = ((0x909589i32) as u16));
        (*E).base.y_pos = ((*E).base.y_pos).wrapping_sub(128u16);
        SpawnEprojWithGfx(0u16, v5, addr_kEproj_SporeSpawners as u16, game);
        SpawnEprojWithGfx(1u16, v5, addr_kEproj_SporeSpawners as u16, game);
        SpawnEprojWithGfx(2u16, v5, addr_kEproj_SporeSpawners as u16, game);
        SpawnEprojWithGfx(3u16, v5, addr_kEproj_SporeSpawners as u16, game);
        SporeSpawn_Func_5(game);
    };
}
pub unsafe fn CallSporeSpawnFunc(ea: u32, game: &mut Game) {
    match ea {
        10873626 => (),
        10873627 => SporeSpawn_Func_1(game),
        10873682 => {
            SporeSpawn_Func_2(ram_word!(game, cur_enemy_index), game);
        }
        10873755 => SporeSpawn_Func_3(game),
        10873838 => SporeSpawn_Func_4(game),
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn SporeSpawn_Main(game: &mut Game) {
    let E: *mut Enemy_SporeSpawn = Get_SporeSpawn(ram_word!(game, cur_enemy_index) as i32, game);
    CallSporeSpawnFunc((((*E).ssn_var_A) as u32) | (0xa50000u32), game);
}
pub unsafe fn SporeSpawn_Func_1(game: &mut Game) {
    SporeSpawn_Func_5(game);
    let E: *mut Enemy_SporeSpawn = Get_SporeSpawn(ram_word!(game, cur_enemy_index) as i32, game);
    let v1: u16 = ((*E).base.y_pos).wrapping_add(1u16);
    (*E).base.y_pos = v1;
    if v1 as i32 - 624i32 & 0x8000i32 == 0 {
        (*E).base.current_instruction = addr_kDraygon_Ilist_E6D5 as u16;
        (*E).base.instruction_timer = 1u16;
    }
    let E0: *mut Enemy_SporeSpawn = Get_SporeSpawn(0i32, game);
    (*E0).ssn_var_0B = 48u16;
    (*E0).ssn_var_0C = 1u16;
    (*E0).ssn_var_0A = 192u16;
}
pub unsafe fn SporeSpawn_Func_2(k: u16, game: &mut Game) {
    SporeSpawn_Func_5(game);
    let E0: *mut Enemy_SporeSpawn = Get_SporeSpawn(0i32, game);
    let E: *mut Enemy_SporeSpawn = Get_SporeSpawn(k as i32, game);
    (*E).base.x_pos =
        ((*E).ssn_var_C).wrapping_add(CosineMult8bit((*E0).ssn_var_0A, (*E0).ssn_var_0B, game));
    (*E).base.y_pos = ((*E).ssn_var_D).wrapping_add(SineMult8bit(
        (2i32 * ((*E0).ssn_var_0A as i32 - 64i32)) as u16,
        ((*E0).ssn_var_0B).wrapping_sub(16u16),
        game,
    ));
    (*E0).ssn_var_0A = ((*(&raw mut (*E0).ssn_var_0C as *mut u8).offset(0))
        .wrapping_add(((*E0).ssn_var_0A) as u8)) as u16;
}
pub unsafe fn SporeSpawn_Func_3(game: &mut Game) {
    let E0: *mut Enemy_SporeSpawn = Get_SporeSpawn(0i32, game);
    (*E0).ssn_var_43 = ((64u8).wrapping_sub(
        (CalculateAngleFromXY(
            (128u16).wrapping_sub((*E0).base.x_pos),
            (624u16).wrapping_sub((*E0).base.y_pos),
            game,
        )) as u8,
    )) as u16;
    let E: *mut Enemy_SporeSpawn = Get_SporeSpawn(ram_word!(game, cur_enemy_index) as i32, game);
    let pt: Point32 = ConvertAngleToXy(
        *(&raw mut (*E0).ssn_var_43 as *mut u8).offset(0) as u16,
        1u16,
        game,
    );
    (*E).ssn_var_28 = (pt.x >> 16i32) as u16;
    (*E).ssn_var_29 = pt.x as u16;
    (*E).ssn_var_2A = (pt.y >> 16i32) as u16;
    (*E).ssn_var_2B = pt.y as u16;
}
pub unsafe fn SporeSpawn_Func_4(game: &mut Game) {
    let E: *mut Enemy_SporeSpawn = Get_SporeSpawn(ram_word!(game, cur_enemy_index) as i32, game);
    let E0: *mut Enemy_SporeSpawn = Get_SporeSpawn(0i32, game);
    let varE20: u16 = *(&raw mut (*E0).ssn_var_43 as *mut u8).offset(0) as u16;
    EnemyFunc_B691(
        varE20,
        Point32 {
            x: ((*E).ssn_var_28 as u32) << 16i32 | (*E).ssn_var_29 as u32,
            y: ((*E).ssn_var_2A as u32) << 16i32 | (*E).ssn_var_2B as u32,
        },
        game,
    );
    let v2: u16 = Abs16(((*E0).base.x_pos).wrapping_sub(128u16));
    if v2 as i32 - 8i32 & 0x8000i32 != 0 {
        let v3: u16 = Abs16(((*E0).base.y_pos).wrapping_sub(624u16));
        if v3 as i32 - 8i32 & 0x8000i32 != 0 {
            (*E0).ssn_var_A = ((0xa5eb1ai32) as u16);
        }
    }
    SporeSpawn_Func_5(game);
    sub_A5E9F5(game);
}
pub unsafe fn SporeSpawn_Func_5(game: &mut Game) {
    let E: *mut Enemy_SporeSpawn = Get_SporeSpawn(0i32, game);
    let v1: u16 = ((*E).base.x_pos).wrapping_sub((*E).ssn_var_04);
    let mut v2: u16 = 0;
    let mut r18: u16 = 0;
    let mut r20: u16 = 0;
    let mut r22: u16 = 0;
    if (v1 as i32 & 0x8000i32) == 0i32 {
        r18 = (v1 as i32 >> 1i32) as u16;
        r20 = (v1 as i32 >> 2i32) as u16;
        r22 = ((v1 as i32 >> 1i32) + (v1 as i32 >> 2i32)) as u16;
        set_ram_word!(game.eproj_x_pos[14] = 128u16);
        set_ram_word!(game.eproj_x_pos[15] = ((*E).ssn_var_04 as i32 + (v1 as i32 >> 2i32)) as u16);
        (*E).ssn_var_20 = ram_word!(game, eproj_x_pos[15]);
        set_ram_word!(game.eproj_x_pos[16] = ((*E).ssn_var_04).wrapping_add(r18));
        (*E).ssn_var_21 = ram_word!(game, eproj_x_pos[16]);
        v2 = ((*E).ssn_var_04).wrapping_add(r22);
    } else {
        r18 = ((((*E).ssn_var_04).wrapping_sub((*E).base.x_pos)) as i32 >> 1i32) as u16;
        r20 = (-(v1 as i32) as u16 as i32 >> 2i32) as u16;
        r22 = (r18).wrapping_add(r20);
        set_ram_word!(game.eproj_x_pos[14] = 128u16);
        set_ram_word!(game.eproj_x_pos[15] = ((*E).ssn_var_04).wrapping_sub(r20));
        (*E).ssn_var_20 = ram_word!(game, eproj_x_pos[15]);
        set_ram_word!(game.eproj_x_pos[16] = ((*E).ssn_var_04).wrapping_sub(r18));
        (*E).ssn_var_21 = ram_word!(game, eproj_x_pos[16]);
        v2 = ((*E).ssn_var_04).wrapping_sub(r22);
    }
    set_ram_word!(game.eproj_x_pos[17] = v2);
    (*E).ssn_var_22 = v2;
    let v3: u16 = ((*E).base.y_pos as i32 - 40i32 - (*E).ssn_var_05 as i32) as u16;
    let mut v4: u16 = 0;
    if (v3 as i32 & 0x8000i32) == 0i32 {
        r18 = (v3 as i32 >> 1i32) as u16;
        r20 = (v3 as i32 >> 2i32) as u16;
        r22 = ((v3 as i32 >> 1i32) + (v3 as i32 >> 2i32)) as u16;
        set_ram_word!(game.eproj_y_pos[14] = 560u16);
        set_ram_word!(game.eproj_y_pos[15] = ((*E).ssn_var_05 as i32 + (v3 as i32 >> 2i32)) as u16);
        (*E).ssn_var_23 = ram_word!(game, eproj_y_pos[15]);
        set_ram_word!(game.eproj_y_pos[16] = ((*E).ssn_var_05).wrapping_add(r18));
        (*E).ssn_var_24 = ram_word!(game, eproj_y_pos[16]);
        v4 = ((*E).ssn_var_05).wrapping_add(r22);
    } else {
        r18 = (((*E).ssn_var_05 as i32 - ((*E).base.y_pos as i32 - 40i32)) as u16 as i32 >> 1i32)
            as u16;
        r20 = (-(v3 as i32) as u16 as i32 >> 2i32) as u16;
        r22 = (r18).wrapping_add(r20);
        set_ram_word!(game.eproj_y_pos[14] = 560u16);
        set_ram_word!(game.eproj_y_pos[15] = ((*E).ssn_var_05).wrapping_sub(r20));
        (*E).ssn_var_23 = ram_word!(game, eproj_y_pos[15]);
        set_ram_word!(game.eproj_y_pos[16] = ((*E).ssn_var_05).wrapping_sub(r18));
        (*E).ssn_var_24 = ram_word!(game, eproj_y_pos[16]);
        v4 = ((*E).ssn_var_05).wrapping_sub(r22);
    }
    set_ram_word!(game.eproj_y_pos[17] = v4);
    (*E).ssn_var_25 = v4;
}
pub unsafe fn SporeSpawn_Shot(game: &mut Game) {
    let mut v4: i16 = 0;
    let mut v0: u16 = (2u16).wrapping_mul(ram_word!(game, collision_detection_index));
    let v1: u16 = ram_word!(
        game,
        projectile_type[ram_word!(game, collision_detection_index)]
    );
    if (v1 as i32 & 0x700i32) != 0i32 || (v1 as i32 & 0x10i32) != 0i32 {
        EnemyFunc_A6B4_UsedBySporeSpawn(game);
        v0 = ram_word!(game, cur_enemy_index);
        let EK: *mut Enemy_SporeSpawn = Get_SporeSpawn(v0 as i32, game);
        if (*EK).base.flash_timer != 0 {
            let E: *mut Enemy_SporeSpawn = Get_SporeSpawn(0i32, game);
            (*EK).ssn_var_A = ((0xa5eb52i32) as u16);
            v4 = 2i16;
            if (*EK).base.health as i32 - 400i32 & 0x8000i32 != 0 {
                if ((*E).ssn_var_0C as i32 & 0x8000i32) != 0i32 {
                    v4 = -(2i32) as i16;
                }
                (*E).ssn_var_0C = v4 as u16;
            }
            if (*E).ssn_var_2F == 0 {
                (*E).ssn_var_0C = -((*E).ssn_var_0C as i32) as u16;
                (*E).ssn_var_2F = 1u16;
                (*EK).base.current_instruction = addr_stru_A5E729 as u16;
                (*EK).base.instruction_timer = 1u16;
                let mut v7: u16 = 96u16;
                let health: u16 = (*EK).base.health;
                if health as i32 - 70i32 & 0x8000i32 == 0 {
                    v7 = 64u16;
                    if health as i32 - 410i32 & 0x8000i32 == 0 {
                        v7 = 32u16;
                        if health as i32 - 770i32 & 0x8000i32 == 0 {
                            v7 = 0u16;
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
    if (*Get_SporeSpawn(ram_word!(game, cur_enemy_index) as i32, game))
        .base
        .health
        == 0
    {
        let E: *mut Enemy_SporeSpawn =
            Get_SporeSpawn(ram_word!(game, cur_enemy_index) as i32, game);
        let E0: *mut Enemy_SporeSpawn = Get_SporeSpawn(0i32, game);
        (*E0).ssn_var_0E = 0u16;
        (*E).base.invincibility_timer = 0u16;
        (*E).base.flash_timer = 0u16;
        (*E).base.ai_handler_bits = 0u16;
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
        let mut i: i32 = 26i32;
        while i >= 0i32 {
            *(((&raw mut game.g_ram as *mut u8).offset(0x1997) as *mut u16 as *mut u8)
                .offset(i as u16 as i32 as isize) as *mut u16) = 0u16;
            i -= 2i32;
        }
        (*E0).base.current_instruction = addr_kDraygon_Ilist_E77D as u16;
        (*E0).base.instruction_timer = 1u16;
        let ref mut fresh5 = *((&raw mut game.g_ram as *mut u8)
            .offset(0xd828)
            .offset(ram_word!(game, area_index) as isize) as *mut u8
            as *mut u16);
        *fresh5 = (*fresh5) | (2u16);
        set_ram_word!(game.scrolling_finished_hook = 0u16);
        SpawnHardcodedPlm(
            SpawnHardcodedPlmArgs::new(0x7u8, 0x1eu8, 0xb78f as VoidP),
            game,
        );
    }
}
pub unsafe fn SporeSpawn_Func_7(a: u16, game: &mut Game) {
    let mut v1: u16 = a;
    let mut i: i32 = 0i32;
    while i != 32i32 {
        set_ram_word!(
            game.palette_buffer[((i >> 1i32) + 144i32)] =
                crate::rtl::rom_table_word!(game, 0xa5e379, (v1 as i32 >> 1i32) as isize)
        );
        v1 = (v1).wrapping_add(2u16);
        i += 2i32;
    }
}
