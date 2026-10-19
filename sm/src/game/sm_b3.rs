pub use crate::game::constants::bank_86::*;
pub use crate::game::constants::enemy_properties::*;
use crate::game::enemy_layouts::enemy_layout;
use crate::game::sm_80::{
    CheckEventHappened, NextRandom, QueueMusic_Delayed8, QueueSfx2_Max6, SetBossBitForCurArea,
};
use crate::game::sm_84::SpawnHardcodedPlm;
use crate::game::sm_86::{SpawnEprojWithGfx, SpawnEprojWithRoomGfx};
use crate::game::sm_a0::{
    CalculateAngleFromXY, CalculateAngleOfSamusFromEnemy, CallEnemyPreInstr,
    CheckIfEnemyIsOnScreen, ConvertAngleToXy, EnemyFunc_B691, EnemyGrappleDeath,
    Enemy_ItemDrop_Botwoon, Enemy_MoveDown, Enemy_MoveRight_IgnoreSlopes, Enemy_SwitchToFrozenAi,
    GetSamusEnemyDelta_X, GetSamusEnemyDelta_Y, IsEnemyLeavingScreen, IsSamusWithinEnemy_X,
    IsSamusWithinEnemy_Y, NormalEnemyFrozenAI, NormalEnemyPowerBombAiSkipDeathAnim_CurEnemy,
    NormalEnemyShotAi, NormalEnemyShotAiSkipDeathAnim_CurEnemy, NormalEnemyTouchAi,
    NormalEnemyTouchAiSkipDeathAnim_CurEnemy, SignExtend8, SwitchEnemyAiToMainAi,
};
use crate::game::sm_b4::CreateSpriteAtPos;
use crate::game_state::Game;
use crate::ram_map::{ram_word, set_ram_word, update_ram_word};
use crate::rtl::abs16;
use crate::rtl::AddToHiLo;
use crate::rtl::{RomPtr, Unreachable};

pub use crate::types::Consts_86;
pub use crate::types::Consts_B3;
pub use crate::types::{
    __int16_t, __int32_t, __int8_t, __uint16_t, __uint32_t, __uint8_t, int16, int16_t, int32,
    int32_t, int8, int8_t, uint16, uint16_t, uint32, uint32_t, uint8, uint8_t, uintptr_t,
    EnemyBase, EnemyData, EnemyProps, Point16U, VoidP,
};
pub const addr_kEscapeDachora_Ilist_E964: Consts_B3 = 59748;
pub const addr_kEscapeEtecoon_Ilist_E5DA: Consts_B3 = 58842;
pub const addr_kEscapeEtecoon_Ilist_E5AE: Consts_B3 = 58798;
pub const addr_kEscapeEtecoon_Ilist_E582: Consts_B3 = 58754;
pub const addr_kEscapeEtecoon_Ilist_E556: Consts_B3 = 58710;
pub const addr_loc_B3E28C: Consts_B3 = 57996;
pub const addr_stru_B3E150: Consts_B3 = 57680;
pub const addr_kBotwoon_Ilist_9389: Consts_B3 = 37769;
pub const addr_kBrinstarYellowPipeBug_Ilist_8F38: Consts_B3 = 36664;
pub const addr_kBrinstarYellowPipeBug_Ilist_8F24: Consts_B3 = 36644;
pub const addr_kBrinstarYellowPipeBug_Ilist_8F10: Consts_B3 = 36624;
pub const addr_kBrinstarYellowPipeBug_Ilist_8EFC: Consts_B3 = 36604;
pub const addr_kNorfairPipeBug_Ilist_8B45: Consts_B3 = 35653;
pub const addr_kNorfairPipeBug_Ilist_8B21: Consts_B3 = 35617;
pub const addr_kNorfairPipeBug_Ilist_8B05: Consts_B3 = 35589;
pub const addr_kNorfairPipeBug_Ilist_8AE1: Consts_B3 = 35553;
pub const addr_kBrinstarPipeBug_Ilist_8A1D: Consts_B3 = 35357;
pub const addr_kBrinstarPipeBug_Ilist_87AB: Consts_B3 = 34731;
pub const addr_kUnusedSpinningTurtleEye_Ilist_86A7: Consts_B3 = 34471;
pub const addr_kSpritemap_Nothing_B3: Consts_B3 = 32845;
pub use crate::types::{Enemy_Botwoon, Point32, SpawnHardcodedPlmArgs};
enemy_layout! {
    Enemy_EscapeDachora {
        base: EnemyBase,
        words: [
            eda_var_A, eda_var_B, eda_var_C, eda_var_D, eda_var_E, eda_var_F,
            eda_parameter_1, eda_parameter_2,
        ],
    }
}

enemy_layout! {
    Enemy_EscapeEtecoon {
        base: EnemyBase,
        words: [
            een_var_A, een_var_B, een_var_C, een_var_D, een_var_E, een_var_F,
            een_parameter_1, een_parameter_2,
        ],
    }
}

enemy_layout! {
    Enemy_PipeBug {
        base: EnemyBase,
        words: [
            pbg_var_A, pbg_var_B, pbg_var_C, pbg_var_D, pbg_var_E, pbg_var_F,
            pbg_parameter_1, pbg_parameter_2,
        ],
        pad1: [uint8; 26696],
        words: [
            pbg_var_00, pbg_var_01, pbg_var_02, pbg_var_03, pbg_var_04, pbg_var_05,
            pbg_var_06, pbg_var_07, pbg_var_08, pbg_var_09, pbg_var_0A, pbg_var_0B,
            pbg_var_0C, pbg_var_0D, pbg_var_0E, pbg_var_0F, pbg_var_10, pbg_var_11,
            pbg_var_12, pbg_var_13, pbg_var_14, pbg_var_15, pbg_var_16, pbg_var_17,
            pbg_var_18, pbg_var_19, pbg_var_1A, pbg_var_1B, pbg_var_1C, pbg_var_1D,
            pbg_var_1E, pbg_var_1F,
        ],
        pad2: [uint8; 1984],
        words: [
            pbg_var_20, pbg_var_21, pbg_var_22, pbg_var_23, pbg_var_24, pbg_var_25,
            pbg_var_26, pbg_var_27, pbg_var_28, pbg_var_29, pbg_var_2A, pbg_var_2B,
            pbg_var_2C, pbg_var_2D, pbg_var_2E, pbg_var_2F, pbg_var_30, pbg_var_31,
            pbg_var_32, pbg_var_33, pbg_var_34, pbg_var_35, pbg_var_36, pbg_var_37,
            pbg_var_38, pbg_var_39, pbg_var_3A, pbg_var_3B, pbg_var_3C, pbg_var_3D,
            pbg_var_3E, pbg_var_3F,
        ],
    }
}

#[inline]
unsafe fn RomPtr_B3(addr: uint16_t, game: &mut Game) -> *const uint8 {
    RomPtr((0xb30000u32) | ((addr) as u32), game)
}
pub const fnnullsub_171_B3: i32 = 0xb3807b;
pub const fnBrinstarPipeBug_PreInstr_1: i32 = 0xb38880;
pub const fnBrinstarPipeBug_PreInstr_2: i32 = 0xb38890;
pub const fnBrinstarPipeBug_PreInstr_3: i32 = 0xb388e3;
pub const fnBrinstarPipeBug_PreInstr_4: i32 = 0xb3891c;
pub const fnBrinstarPipeBug_PreInstr_5: i32 = 0xb3897e;
pub const fnNorfairPipeBug_Func_1: i32 = 0xb38bcd;
pub const fnNorfairPipeBug_Func_2: i32 = 0xb38bff;
pub const fnNorfairPipeBug_Func_4: i32 = 0xb38ca6;
pub const fnNorfairPipeBug_Func_5: i32 = 0xb38cff;
pub const fnNorfairPipeBug_Func_6: i32 = 0xb38d0c;
pub const fnNorfairPipeBug_Func_7: i32 = 0xb38d4e;
pub const fnNorfairPipeBug_Func_8: i32 = 0xb38d90;
pub const fnNorfairPipeBug_Func_9: i32 = 0xb38dd2;
pub const fnNorfairPipeBug_Func_10: i32 = 0xb38e14;
pub const fnNorfairPipeBug_Func_11: i32 = 0xb38e35;
pub const fnNorfairPipeBug_Func_12: i32 = 0xb38e5a;
pub const fnBrinstarYellowPipeBug_Func_1: i32 = 0xb38fb5;
pub const fnBrinstarYellowPipeBug_Func_2: i32 = 0xb38ff5;
pub const fnBrinstarYellowPipeBug_Func_3: i32 = 0xb39028;
pub const fnBrinstarYellowPipeBug_Func_5: i32 = 0xb390bd;
pub const fnBrinstarYellowPipeBug_Func_7: i32 = 0xb3915a;
pub const fnBrinstarYellowPipeBug_Func_8: i32 = 0xb391d8;
pub const fnBotwoon_Func_6: i32 = 0xb39878;
pub const fnBotwoon_Func_7: i32 = 0xb3989d;
pub const fnBotwoon_Func_12: i32 = 0xb399a4;
pub const fnBotwoon_Func_13: i32 = 0xb399e4;
pub const fnBotwoon_Func_14: i32 = 0xb39a46;
pub const fnBotwoon_Func_15: i32 = 0xb39a5e;
pub const fnBotwoon_Func_16: i32 = 0xb39aca;
pub const fnBotwoon_Func_18: i32 = 0xb39af9;
pub const fnBotwoon_Func_19: i32 = 0xb39bb7;
pub const fnBotwoon_Func_26: i32 = 0xb39dc0;
pub const fnBotwoon_Func_27: i32 = 0xb39e7d;
pub const fnBotwoon_Func_28: i32 = 0xb39ee0;
pub const fnBotwoon_Func_29: i32 = 0xb39f34;
pub const fnBotwoon_Func_30: i32 = 0xb39f7a;
pub const fnBotwoon_Func_32: i32 = 0xb3e250;
pub const fnBotwoon_Func_33: uint32 = 11788940;
#[inline]
unsafe fn gEnemyData(o: i32, game: &mut Game) -> *mut EnemyData {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut EnemyData
}
#[inline]
unsafe fn Get_Botwoon(o: i32, game: &mut Game) -> *mut Enemy_Botwoon {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_Botwoon
}
#[inline]
unsafe fn Get_EscapeDachora(o: i32, game: &mut Game) -> *mut Enemy_EscapeDachora {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_EscapeDachora
}
#[inline]
unsafe fn Get_EscapeEtecoon(o: i32, game: &mut Game) -> *mut Enemy_EscapeEtecoon {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_EscapeEtecoon
}
#[inline]
unsafe fn Get_PipeBug(o: i32, game: &mut Game) -> *mut Enemy_PipeBug {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_PipeBug
}
static g_word_B39E77: [int16; 3] = [2, 3, 4];
pub unsafe fn Enemy_GrappleReact_NoInteract_B3(game: &mut Game) {
    SwitchEnemyAiToMainAi(game);
}
pub unsafe fn Enemy_GrappleReact_KillEnemy_B3(game: &mut Game) {
    EnemyGrappleDeath(game);
}
pub unsafe fn Enemy_GrappleReact_CancelBeam_B3(game: &mut Game) {
    Enemy_SwitchToFrozenAi(game);
}
pub unsafe fn Enemy_NormalTouchAI_B3(game: &mut Game) {
    NormalEnemyTouchAi(game);
}
pub unsafe fn Enemy_NormalTouchAI_SkipDeathAnim_B3(game: &mut Game) {
    NormalEnemyTouchAiSkipDeathAnim_CurEnemy(game);
}
pub unsafe fn Enemy_NormalShotAI_B3(game: &mut Game) {
    NormalEnemyShotAi(game);
}
pub unsafe fn Enemy_NormalShotAI_SkipSomeParts_B3(game: &mut Game) {
    NormalEnemyShotAiSkipDeathAnim_CurEnemy(game);
}
pub unsafe fn Enemy_NormalPowerBombAI_SkipDeathAnim_B3(game: &mut Game) {
    NormalEnemyPowerBombAiSkipDeathAnim_CurEnemy(game);
}
pub unsafe fn Enemy_NormalFrozenAI_B3(game: &mut Game) {
    NormalEnemyFrozenAI(game);
}
pub unsafe fn Enemy_SetAiPreInstr_B3(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*gEnemyData(k as i32, game)).ai_preinstr = *jp.offset(0);
    jp.offset(1)
}
pub unsafe fn Enemy_ClearAiPreInstr_B3(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*gEnemyData(k as i32, game)).ai_preinstr = ((0xb3807bi32) as u16);
    jp
}
pub unsafe fn UnusedSpinningTurtleEye_Init(game: &mut Game) {
    let v0: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    (*v0).properties = ((*v0).properties) | ((kEnemyProps_DisableSamusColl) as u16);
    (*v0).current_instruction = addr_kUnusedSpinningTurtleEye_Ilist_86A7 as uint16;
}
pub fn UnusedSpinningTurtleEye_Main() {}
pub unsafe fn BrinstarPipeBug_Init(game: &mut Game) {
    let PipeBug: *mut Enemy_PipeBug = Get_PipeBug(ram_word!(game, cur_enemy_index) as i32, game);
    (*PipeBug).pbg_var_B = (*PipeBug).base.x_pos;
    let y_pos: uint16 = (*PipeBug).base.y_pos;
    (*PipeBug).pbg_var_C = y_pos;
    (*PipeBug).pbg_var_00 = (y_pos).wrapping_sub(16u16);
    (*PipeBug).pbg_var_F = ((0xb38880i32) as u16);
    (*PipeBug).pbg_var_D = 48 as uint16;
    (*PipeBug).pbg_var_E = 0 as uint16;
    (*PipeBug).pbg_var_01 = 0 as uint16;
    (*PipeBug).base.current_instruction = addr_kBrinstarPipeBug_Ilist_87AB as uint16;
    if (*PipeBug).pbg_parameter_1 != 0 {
        (*PipeBug).base.current_instruction = addr_kBrinstarPipeBug_Ilist_8A1D as uint16;
    }
}
pub unsafe fn BrinstarPipeBug_Main(game: &mut Game) {
    let PipeBug: *mut Enemy_PipeBug = Get_PipeBug(ram_word!(game, cur_enemy_index) as i32, game);
    CallEnemyPreInstr((((*PipeBug).pbg_var_F) as u32) | (0xb30000u32), game);
}
pub unsafe fn BrinstarPipeBug_PreInstr_1(k: uint16, game: &mut Game) {
    if CheckIfEnemyIsOnScreen(game) == 0 {
        (*Get_PipeBug(ram_word!(game, cur_enemy_index) as i32, game)).pbg_var_F =
            ((0xb38890i32) as u16);
    }
}
pub unsafe fn BrinstarPipeBug_PreInstr_2(k: uint16, game: &mut Game) {
    let mut SamusEnemyDelta_Y: int16 = 0;
    SamusEnemyDelta_Y = GetSamusEnemyDelta_Y(k, game) as int16;
    if SamusEnemyDelta_Y < 0i16 && SamusEnemyDelta_Y as i32 + 96 as i32 & 0x8000 as i32 == 0 {
        let PipeBug: *mut Enemy_PipeBug = Get_PipeBug(k as i32, game);
        let a: uint16 = GetSamusEnemyDelta_X(k, game);
        (*PipeBug).pbg_var_A =
            ((*PipeBug).pbg_var_A as i32 & 0x7fff as i32 | a as i32 & 0x8000 as i32) as uint16;
        if abs16(((*PipeBug).base.x_pos).wrapping_sub(ram_word!(game, samus_x_pos))) < 0x40u16 {
            (*PipeBug).pbg_var_F = ((0xb388e3i32) as u16);
            (*PipeBug).base.properties =
                ((*PipeBug).base.properties as i32 & !(kEnemyProps_Invisible as i32)) as uint16;
            (*PipeBug).base.timer = 0 as uint16;
            let mut v4: uint16 = 0;
            if ((*PipeBug).pbg_var_A as i32 & 0x8000 as i32) != 0i32 {
                v4 = 0 as uint16;
            } else {
                v4 = 2 as uint16;
            }
            (*PipeBug).pbg_var_E = v4;
            BrinstarPipeBug_Func_1(game);
        }
    }
}
pub unsafe fn BrinstarPipeBug_PreInstr_3(k: uint16, game: &mut Game) {
    let PipeBug: *mut Enemy_PipeBug = Get_PipeBug(k as i32, game);
    let fresh7 = (*PipeBug).base.y_subpos;
    (*PipeBug).base.y_subpos = (*PipeBug).base.y_subpos.wrapping_sub(1);
    let v2: bool = fresh7 != 0u16;
    (*PipeBug).base.y_pos = ((*PipeBug).base.y_pos as i32 + (v2 as i32 - 2 as i32)) as uint16;
    if (((*PipeBug).pbg_var_00) as i16).wrapping_sub(((*PipeBug).base.y_pos) as i16) >= 0i16
        && (*PipeBug).base.y_pos < ram_word!(game, samus_y_pos)
    {
        (*PipeBug).pbg_var_E = ((*PipeBug).pbg_var_E) | (1u16);
        BrinstarPipeBug_Func_1(game);
        (*PipeBug).pbg_var_F = ((0xb3891ci32) as u16);
    }
}
pub unsafe fn BrinstarPipeBug_PreInstr_4(k: uint16, game: &mut Game) {
    let PipeBug: *mut Enemy_PipeBug = Get_PipeBug(k as i32, game);
    if ((*PipeBug).pbg_var_A as i32 & 0x8000 as i32) == 0i32 {
        (*PipeBug).base.x_subpos = (*PipeBug).base.x_subpos;
        (*PipeBug).base.x_pos = ((*PipeBug).base.x_pos).wrapping_add(2u16);
    } else {
        (*PipeBug).base.x_subpos = (*PipeBug).base.x_subpos;
        (*PipeBug).base.x_pos = ((*PipeBug).base.x_pos).wrapping_sub(2u16);
    }
    if IsEnemyLeavingScreen(k, game) as i32 & 1 as i32 != 0 {
        (*PipeBug).base.x_pos = (*PipeBug).pbg_var_B;
        (*PipeBug).base.x_subpos = 0 as uint16;
        let pbg_var_C: uint16 = (*PipeBug).pbg_var_C;
        (*PipeBug).base.y_pos = pbg_var_C;
        (*PipeBug).base.y_subpos = pbg_var_C;
        (*PipeBug).pbg_var_E = 0 as uint16;
        BrinstarPipeBug_Func_1(game);
        (*PipeBug).base.properties =
            ((*PipeBug).base.properties) | ((kEnemyProps_Invisible) as u16);
        (*PipeBug).pbg_var_D = 48 as uint16;
        (*PipeBug).pbg_var_F = ((0xb3897ei32) as u16);
    }
}
pub unsafe fn BrinstarPipeBug_PreInstr_5(k: uint16, game: &mut Game) {
    let E: *mut Enemy_PipeBug = Get_PipeBug(k as i32, game);
    let fresh8 = (*E).pbg_var_D;
    (*E).pbg_var_D = (*E).pbg_var_D.wrapping_sub(1);
    if fresh8 == 1u16 {
        (*E).pbg_var_F = ((0xb38890i32) as u16);
    }
}
pub unsafe fn BrinstarPipeBug_Func_1(game: &mut Game) {
    let PipeBug: *mut Enemy_PipeBug = Get_PipeBug(ram_word!(game, cur_enemy_index) as i32, game);
    let pbg_var_E: uint16 = (*PipeBug).pbg_var_E;
    if pbg_var_E != (*PipeBug).pbg_var_01 {
        (*PipeBug).pbg_var_01 = pbg_var_E;
        (*PipeBug).base.current_instruction = (if (*PipeBug).pbg_parameter_1 != 0u16 {
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xb38833)) as *const uint8 as *mut uint16)
                .offset(pbg_var_E as isize) as i32
        } else {
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xb3882b)) as *const uint8 as *mut uint16)
                .offset(pbg_var_E as isize) as i32
        }) as uint16;
        (*PipeBug).base.instruction_timer = 1 as uint16;
        (*PipeBug).base.timer = 0 as uint16;
    }
}
pub unsafe fn NorfairPipeBug_Init(game: &mut Game) {
    let PipeBug: *mut Enemy_PipeBug = Get_PipeBug(ram_word!(game, cur_enemy_index) as i32, game);
    (*PipeBug).pbg_var_D = (*PipeBug).base.x_pos;
    (*PipeBug).pbg_var_E = (*PipeBug).base.y_pos;
    (*PipeBug).base.properties = ((*PipeBug).base.properties) | ((kEnemyProps_Invisible) as u16);
    (*PipeBug).pbg_var_B = (8u16).wrapping_mul(
        (*(&raw mut (*PipeBug).pbg_parameter_2 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1_usize) as isize,
        )) as u16,
    );
    (*PipeBug).pbg_var_C = 0 as uint16;
    (*PipeBug).base.instruction_timer = 1 as uint16;
    (*PipeBug).base.timer = 0 as uint16;
    (*PipeBug).base.current_instruction = addr_kNorfairPipeBug_Ilist_8AE1 as uint16;
    (*PipeBug).pbg_var_A = ((0xb38bcdi32) as u16);
}
pub unsafe fn CallBugInstr(ea: uint32, k: uint16, game: &mut Game) {
    match ea {
        11766733 => NorfairPipeBug_Func_1(game),
        11766783 => NorfairPipeBug_Func_2(game),
        11766950 => NorfairPipeBug_Func_4(game),
        11767039 => NorfairPipeBug_Func_5(k, game),
        11767052 => NorfairPipeBug_Func_6(game),
        11767118 => NorfairPipeBug_Func_7(game),
        11767184 => NorfairPipeBug_Func_8(game),
        11767250 => NorfairPipeBug_Func_9(game),
        11767316 => NorfairPipeBug_Func_10(game),
        11767349 => NorfairPipeBug_Func_11(game),
        11767386 => NorfairPipeBug_Func_12(game),
        11767733 => BrinstarYellowPipeBug_Func_1(game),
        11767797 => {
            BrinstarYellowPipeBug_Func_2(ram_word!(game, cur_enemy_index), game);
        }
        11767848 => BrinstarYellowPipeBug_Func_3(game),
        11767997 => BrinstarYellowPipeBug_Func_5(game),
        11768154 => BrinstarYellowPipeBug_Func_7(game),
        11768280 => BrinstarYellowPipeBug_Func_8(game),
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn NorfairPipeBug_Main(game: &mut Game) {
    let PipeBug: *mut Enemy_PipeBug = Get_PipeBug(ram_word!(game, cur_enemy_index) as i32, game);
    CallBugInstr(
        (((*PipeBug).pbg_var_A) as u32) | (0xb30000u32),
        ram_word!(game, cur_enemy_index),
        game,
    );
    NorfairPipeBug_8BA8(game);
}
pub unsafe fn NorfairPipeBug_8BA8(game: &mut Game) {
    if CheckIfEnemyIsOnScreen(game) != 0 {
        let PipeBug: *mut Enemy_PipeBug =
            Get_PipeBug(ram_word!(game, cur_enemy_index) as i32, game);
        (*PipeBug).base.properties =
            ((*PipeBug).base.properties) | ((kEnemyProps_Invisible) as u16);
        (*PipeBug).pbg_var_A = ((0xb38bcdi32) as u16);
        (*PipeBug).base.x_pos = (*PipeBug).pbg_var_D;
        (*PipeBug).base.y_pos = (*PipeBug).pbg_var_E;
    }
}
pub unsafe fn NorfairPipeBug_Func_1(game: &mut Game) {
    if *(&raw mut (*(Get_PipeBug as unsafe fn(i32, &mut Game) -> *mut Enemy_PipeBug)(
        ram_word!(game, cur_enemy_index) as i32,
        game,
    ))
    .pbg_parameter_2 as *mut uint8)
        .offset(0)
        != 0u8
        && (*Get_PipeBug(ram_word!(game, cur_enemy_index) as i32 + 64 as i32, game)).pbg_var_A
            as i32
            == 0xb38bcd as i32 & 0xffff as i32
        && (*Get_PipeBug(ram_word!(game, cur_enemy_index) as i32 + 128 as i32, game)).pbg_var_A
            as i32
            == 0xb38bcd as i32 & 0xffff as i32
        && (*Get_PipeBug(ram_word!(game, cur_enemy_index) as i32 + 192 as i32, game)).pbg_var_A
            as i32
            == 0xb38bcd as i32 & 0xffff as i32
        && (*Get_PipeBug(ram_word!(game, cur_enemy_index) as i32 + 256 as i32, game)).pbg_var_A
            as i32
            == 0xb38bcd as i32 & 0xffff as i32
    {
        (*Get_PipeBug(ram_word!(game, cur_enemy_index) as i32, game)).pbg_var_A =
            ((0xb38bffi32) as u16);
    }
}
pub unsafe fn NorfairPipeBug_Func_2(game: &mut Game) {
    let k: i32 = ram_word!(game, cur_enemy_index) as i32;
    let PipeBug: *mut Enemy_PipeBug = Get_PipeBug(ram_word!(game, cur_enemy_index) as i32, game);
    if IsSamusWithinEnemy_X(
        ram_word!(game, cur_enemy_index),
        *(&raw mut (*PipeBug).pbg_parameter_2 as *mut uint8).offset(0) as uint16,
        game,
    ) != 0u16
        && (((*PipeBug).base.y_pos) as i16).wrapping_sub((ram_word!(game, samus_y_pos)) as i16)
            >= 0i16
    {
        (*PipeBug).pbg_var_C = (*PipeBug).pbg_var_C.wrapping_add(1);
        (*PipeBug).base.instruction_timer = 1 as uint16;
        (*PipeBug).base.timer = 0 as uint16;
        if (GetSamusEnemyDelta_X(k as uint16, game) as i32 & 0x8000 as i32) == 0i32 {
            (*Get_PipeBug(k, game)).base.current_instruction =
                addr_kNorfairPipeBug_Ilist_8B21 as uint16;
            (*Get_PipeBug(k + 64 as i32, game)).base.current_instruction =
                addr_kNorfairPipeBug_Ilist_8B21 as uint16;
            (*Get_PipeBug(k + 128 as i32, game))
                .base
                .current_instruction = addr_kNorfairPipeBug_Ilist_8B21 as uint16;
            (*Get_PipeBug(k + 192 as i32, game))
                .base
                .current_instruction = addr_kNorfairPipeBug_Ilist_8B21 as uint16;
            (*Get_PipeBug(k + 256 as i32, game))
                .base
                .current_instruction = addr_kNorfairPipeBug_Ilist_8B21 as uint16;
        } else {
            (*PipeBug).base.current_instruction = addr_kNorfairPipeBug_Ilist_8AE1 as uint16;
            (*Get_PipeBug(k + 64 as i32, game)).base.current_instruction =
                addr_kNorfairPipeBug_Ilist_8AE1 as uint16;
            (*Get_PipeBug(k + 128 as i32, game))
                .base
                .current_instruction = addr_kNorfairPipeBug_Ilist_8AE1 as uint16;
            (*Get_PipeBug(k + 192 as i32, game))
                .base
                .current_instruction = addr_kNorfairPipeBug_Ilist_8AE1 as uint16;
            (*Get_PipeBug(k + 256 as i32, game))
                .base
                .current_instruction = addr_kNorfairPipeBug_Ilist_8AE1 as uint16;
        }
        NorfairPipeBug_Func_3(k as uint16, game);
    }
}
pub unsafe fn NorfairPipeBug_Func_3(k: uint16, game: &mut Game) {
    (*Get_PipeBug(k as i32, game)).pbg_var_A = ((0xb38ca6i32) as u16);
    (*Get_PipeBug(k as i32 + 64 as i32, game)).pbg_var_A = ((0xb38ca6i32) as u16);
    (*Get_PipeBug(k as i32 + 128 as i32, game)).pbg_var_A = ((0xb38ca6i32) as u16);
    (*Get_PipeBug(k as i32 + 192 as i32, game)).pbg_var_A = ((0xb38ca6i32) as u16);
    (*Get_PipeBug(k as i32 + 256 as i32, game)).pbg_var_A = ((0xb38ca6i32) as u16);
    (*Get_PipeBug(k as i32, game)).pbg_var_02 = 104 as uint16;
    (*Get_PipeBug(k as i32 + 64 as i32, game)).pbg_var_02 = 96 as uint16;
    (*Get_PipeBug(k as i32 + 128 as i32, game)).pbg_var_02 = 88 as uint16;
    (*Get_PipeBug(k as i32 + 192 as i32, game)).pbg_var_02 = 112 as uint16;
    (*Get_PipeBug(k as i32 + 256 as i32, game)).pbg_var_02 = 120 as uint16;
    (*Get_PipeBug(k as i32, game)).pbg_var_F = ((0xb38cffi32) as u16);
    (*Get_PipeBug(k as i32 + 64 as i32, game)).pbg_var_F = ((0xb38d0ci32) as u16);
    (*Get_PipeBug(k as i32 + 128 as i32, game)).pbg_var_F = ((0xb38d4ei32) as u16);
    (*Get_PipeBug(k as i32 + 192 as i32, game)).pbg_var_F = ((0xb38d90i32) as u16);
    (*Get_PipeBug(k as i32 + 256 as i32, game)).pbg_var_F = ((0xb38dd2i32) as u16);
}
pub unsafe fn NorfairPipeBug_Func_4(game: &mut Game) {
    let E: *mut Enemy_PipeBug = Get_PipeBug(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.properties =
        ((*E).base.properties as i32 & !(kEnemyProps_Invisible as i32)) as uint16;
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8 as *mut uint16)
            .offset(66) as uint32)
            << 16 as i32
            | *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
                as *mut uint16)
                .offset(67) as uint32,
    );
    if (((*E).base.y_pos) as i16).wrapping_sub((ram_word!(game, samus_y_pos)) as i16) < 0i16 {
        (*E).pbg_var_A = (*E).pbg_var_F;
        (*E).pbg_var_01 = (*E).base.y_pos;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
        if (GetSamusEnemyDelta_X(ram_word!(game, cur_enemy_index), game) as i32 & 0x8000 as i32)
            == 0i32
        {
            (*E).base.current_instruction = addr_kNorfairPipeBug_Ilist_8B21 as uint16;
        } else {
            (*E).base.current_instruction = addr_kNorfairPipeBug_Ilist_8AE1 as uint16;
        }
    }
}
pub unsafe fn NorfairPipeBug_Func_5(k: uint16, game: &mut Game) {
    let E: *mut Enemy_PipeBug = Get_PipeBug(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).pbg_var_C = (*E).pbg_var_C.wrapping_add(1);
    (*E).pbg_var_A = ((0xb38e5ai32) as u16);
}
pub unsafe fn NorfairPipeBug_Func_6(game: &mut Game) {
    let E: *mut Enemy_PipeBug = Get_PipeBug(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).pbg_var_C = (*E).pbg_var_C.wrapping_add(1);
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8 as *mut uint16)
            .offset(66) as uint32)
            << 16 as i32
            | *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
                as *mut uint16)
                .offset(67) as uint32,
    );
    if (((*E).base.y_pos as i32 + 16 as i32 - (*E).pbg_var_01 as i32) as int16) < 0i16 {
        (*E).base.y_pos = ((*E).pbg_var_01).wrapping_sub(16u16);
        (*E).base.y_subpos = 0 as uint16;
        (*E).pbg_var_A = ((0xb38e5ai32) as u16);
    }
}
pub unsafe fn NorfairPipeBug_Func_7(game: &mut Game) {
    let E: *mut Enemy_PipeBug = Get_PipeBug(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).pbg_var_C = (*E).pbg_var_C.wrapping_add(1);
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8 as *mut uint16)
            .offset(66) as uint32)
            << 16 as i32
            | *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
                as *mut uint16)
                .offset(67) as uint32,
    );
    if (((*E).base.y_pos as i32 + 32 as i32 - (*E).pbg_var_01 as i32) as int16) < 0i16 {
        (*E).base.y_pos = ((*E).pbg_var_01).wrapping_sub(32u16);
        (*E).base.y_subpos = 0 as uint16;
        (*E).pbg_var_A = ((0xb38e5ai32) as u16);
    }
}
pub unsafe fn NorfairPipeBug_Func_8(game: &mut Game) {
    let E: *mut Enemy_PipeBug = Get_PipeBug(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).pbg_var_C = (*E).pbg_var_C.wrapping_add(1);
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8 as *mut uint16)
            .offset(64) as uint32)
            << 16 as i32
            | *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
                as *mut uint16)
                .offset(65) as uint32,
    );
    if (((*E).base.y_pos as i32 - 16 as i32 - (*E).pbg_var_01 as i32) as int16) >= 0i16 {
        (*E).base.y_pos = ((*E).pbg_var_01).wrapping_add(16u16);
        (*E).base.y_subpos = 0 as uint16;
        (*E).pbg_var_A = ((0xb38e5ai32) as u16);
    }
}
pub unsafe fn NorfairPipeBug_Func_9(game: &mut Game) {
    let E: *mut Enemy_PipeBug = Get_PipeBug(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).pbg_var_C = (*E).pbg_var_C.wrapping_add(1);
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8 as *mut uint16)
            .offset(64) as uint32)
            << 16 as i32
            | *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
                as *mut uint16)
                .offset(65) as uint32,
    );
    if (((*E).base.y_pos as i32 - 32 as i32 - (*E).pbg_var_01 as i32) as int16) >= 0i16 {
        (*E).base.y_pos = ((*E).pbg_var_01).wrapping_add(32u16);
        (*E).base.y_subpos = 0 as uint16;
        (*E).pbg_var_A = ((0xb38e5ai32) as u16);
    }
}
pub unsafe fn NorfairPipeBug_Func_10(game: &mut Game) {
    let E: *mut Enemy_PipeBug = Get_PipeBug(ram_word!(game, cur_enemy_index) as i32, game);
    let v2: i32 = (*E).pbg_var_B as i32 >> 1 as i32;
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8 as *mut uint16)
            .offset((v2 + 2 as i32) as isize) as uint32)
            << 16 as i32
            | *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
                as *mut uint16)
                .offset((v2 + 3 as i32) as isize) as uint32,
    );
}
pub unsafe fn NorfairPipeBug_Func_11(game: &mut Game) {
    let E: *mut Enemy_PipeBug = Get_PipeBug(ram_word!(game, cur_enemy_index) as i32, game);
    let v2: i32 = (*E).pbg_var_B as i32 >> 1 as i32;
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8 as *mut uint16)
            .offset(v2 as isize) as uint32)
            << 16 as i32
            | *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
                as *mut uint16)
                .offset((v2 + 1 as i32) as isize) as uint32,
    );
}
pub fn sub_B38E56() {}
pub unsafe fn NorfairPipeBug_Func_12(game: &mut Game) {
    let E: *mut Enemy_PipeBug = Get_PipeBug(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).pbg_var_C = (*E).pbg_var_C.wrapping_add(1);
    if (((*E).pbg_var_C) as i16).wrapping_sub(((*E).pbg_var_02) as i16) >= 0i16 {
        (*E).pbg_var_C = 0 as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
        (*E).pbg_var_A = ((0xb38e14i32) as u16);
        (*E).base.current_instruction = addr_kNorfairPipeBug_Ilist_8B05 as uint16;
        if (GetSamusEnemyDelta_X(ram_word!(game, cur_enemy_index), game) as i32 & 0x8000 as i32)
            == 0i32
        {
            (*E).pbg_var_A = ((0xb38e35i32) as u16);
            (*E).base.current_instruction = addr_kNorfairPipeBug_Ilist_8B45 as uint16;
        }
    }
}
pub unsafe fn BrinstarYellowPipeBug_Init(game: &mut Game) {
    let E: *mut Enemy_PipeBug = Get_PipeBug(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).pbg_var_07 = (*E).base.x_pos;
    (*E).pbg_var_08 = (*E).base.y_pos;
    (*E).base.x_subpos = 0 as uint16;
    (*E).base.y_subpos = 0 as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    (*E).base.current_instruction = addr_kBrinstarYellowPipeBug_Ilist_8EFC as uint16;
    if (*E).pbg_parameter_1 == 0 {
        (*E).base.current_instruction = addr_kBrinstarYellowPipeBug_Ilist_8F24 as uint16;
    }
    let v1: i32 = 8 as i32 * (*E).pbg_parameter_2 as i32 >> 1 as i32;
    (*E).pbg_var_01 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
        as *mut uint16)
        .offset(v1 as isize);
    (*E).pbg_var_00 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
        as *mut uint16)
        .offset((v1 + 1 as i32) as isize);
    (*E).pbg_var_03 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
        as *mut uint16)
        .offset((v1 + 2 as i32) as isize);
    (*E).pbg_var_02 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
        as *mut uint16)
        .offset((v1 + 3 as i32) as isize);
    (*E).pbg_var_A = ((0xb38fb5i32) as u16);
    (*E).pbg_var_06 = 0 as uint16;
}
pub unsafe fn BrinstarYellowPipeBug_Main(game: &mut Game) {
    let E: *mut Enemy_PipeBug = Get_PipeBug(ram_word!(game, cur_enemy_index) as i32, game);
    CallBugInstr(
        (((*E).pbg_var_A) as u32) | (0xb30000u32),
        ram_word!(game, cur_enemy_index),
        game,
    );
}
pub unsafe fn BrinstarYellowPipeBug_Func_1(game: &mut Game) {
    let mut v2: int16 = 0;
    let mut SamusEnemyDelta_X: int16 = 0;
    let E: *mut Enemy_PipeBug = Get_PipeBug(ram_word!(game, cur_enemy_index) as i32, game);
    let current_block_6: u64;
    if (*E).pbg_parameter_1 != 0 {
        SamusEnemyDelta_X = GetSamusEnemyDelta_X(ram_word!(game, cur_enemy_index), game) as int16;
        if SamusEnemyDelta_X < 0i16 && SamusEnemyDelta_X as i32 + 192 as i32 & 0x8000 as i32 == 0 {
            current_block_6 = 1628911311613519252;
        } else {
            current_block_6 = 2868539653012386629;
        }
    } else {
        v2 = GetSamusEnemyDelta_X(ram_word!(game, cur_enemy_index), game) as int16;
        if v2 >= 0i16 && v2 as i32 - 192 as i32 & 0x8000 as i32 != 0 {
            current_block_6 = 1628911311613519252;
        } else {
            current_block_6 = 2868539653012386629;
        }
    }
    if current_block_6 == 1628911311613519252
        && IsSamusWithinEnemy_Y(ram_word!(game, cur_enemy_index), 0x30 as uint16, game) != 0
    {
        (*E).base.properties =
            ((*E).base.properties as i32 & !(kEnemyProps_Invisible as i32)) as uint16;
        (*E).pbg_var_20 = 24 as uint16;
        (*E).pbg_var_A = ((0xb38ff5i32) as u16);
    };
}
pub unsafe fn BrinstarYellowPipeBug_Func_2(k: uint16, game: &mut Game) {
    let E: *mut Enemy_PipeBug = Get_PipeBug(k as i32, game);
    let v2: uint16 = ((*E).pbg_var_20).wrapping_sub(1u16);
    (*E).pbg_var_20 = v2;
    if v2 == 0 {
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
        (*E).base.current_instruction = addr_kBrinstarYellowPipeBug_Ilist_8EFC as uint16;
        (*E).pbg_var_A = ((0xb39028i32) as u16);
        if (*E).pbg_parameter_1 == 0 {
            (*E).base.current_instruction = addr_kBrinstarYellowPipeBug_Ilist_8F24 as uint16;
            (*E).pbg_var_A = ((0xb390bdi32) as u16);
        }
    }
}
pub unsafe fn BrinstarYellowPipeBug_Func_3(game: &mut Game) {
    let E: *mut Enemy_PipeBug = Get_PipeBug(ram_word!(game, cur_enemy_index) as i32, game);
    BrinstarYellowPipeBug_Func_4(ram_word!(game, cur_enemy_index), game);
    if CheckIfEnemyIsOnScreen(game) != 0 {
        (*E).base.x_pos = (*E).pbg_var_07;
        (*E).base.y_pos = (*E).pbg_var_08;
        (*E).base.x_subpos = 0 as uint16;
        (*E).base.y_subpos = 0 as uint16;
        (*E).pbg_var_A = ((0xb38fb5i32) as u16);
        (*E).pbg_var_06 = 0 as uint16;
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
    } else if (*E).pbg_var_06 == 0
        && (*E).base.x_pos as i32 - ram_word!(game, samus_x_pos) as i32 - 48 as i32 & 0x8000 as i32
            != 0
    {
        (*E).pbg_var_A = ((0xb3915ai32) as u16);
        (*E).pbg_var_C = 0 as uint16;
        (*E).pbg_var_D = 0 as uint16;
        (*E).pbg_var_E = 40 as uint16;
        (*E).pbg_var_F = 1 as uint16;
        (*E).pbg_var_04 = (*E).base.x_pos;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
        (*E).base.current_instruction = addr_kBrinstarYellowPipeBug_Ilist_8F10 as uint16;
    }
}
pub unsafe fn BrinstarYellowPipeBug_Func_4(k: uint16, game: &mut Game) {
    let E: *mut Enemy_PipeBug = Get_PipeBug(k as i32, game);
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        ((*E).pbg_var_03 as uint32) << 16 as i32 | (*E).pbg_var_02 as uint32,
    );
}
pub unsafe fn BrinstarYellowPipeBug_Func_5(game: &mut Game) {
    let E: *mut Enemy_PipeBug = Get_PipeBug(ram_word!(game, cur_enemy_index) as i32, game);
    BrinstarYellowPipeBug_Func_6(ram_word!(game, cur_enemy_index), game);
    if CheckIfEnemyIsOnScreen(game) != 0 {
        (*E).base.x_pos = (*E).pbg_var_07;
        (*E).base.y_pos = (*E).pbg_var_08;
        (*E).base.x_subpos = 0 as uint16;
        (*E).base.y_subpos = 0 as uint16;
        (*E).pbg_var_A = ((0xb38fb5i32) as u16);
        (*E).pbg_var_06 = 0 as uint16;
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
    } else if (*E).pbg_var_06 == 0
        && ram_word!(game, samus_x_pos) as i32 - (*E).base.x_pos as i32 - 48 as i32 & 0x8000 as i32
            != 0
    {
        (*E).pbg_var_A = ((0xb391d8i32) as u16);
        (*E).pbg_var_C = 0 as uint16;
        (*E).pbg_var_D = 0 as uint16;
        (*E).pbg_var_E = 40 as uint16;
        (*E).pbg_var_F = 1 as uint16;
        (*E).pbg_var_04 = (*E).base.x_pos;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
        (*E).base.current_instruction = addr_kBrinstarYellowPipeBug_Ilist_8F38 as uint16;
    }
}
pub unsafe fn BrinstarYellowPipeBug_Func_6(k: uint16, game: &mut Game) {
    let E: *mut Enemy_PipeBug = Get_PipeBug(k as i32, game);
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        ((*E).pbg_var_01 as uint32) << 16 as i32 | (*E).pbg_var_00 as uint32,
    );
}
pub unsafe fn BrinstarYellowPipeBug_Func_7(game: &mut Game) {
    let E: *mut Enemy_PipeBug = Get_PipeBug(ram_word!(game, cur_enemy_index) as i32, game);
    if CheckIfEnemyIsOnScreen(game) != 0 {
        (*E).base.x_pos = (*E).pbg_var_07;
        (*E).base.y_pos = (*E).pbg_var_08;
        (*E).base.x_subpos = 0 as uint16;
        (*E).base.y_subpos = 0 as uint16;
        (*E).pbg_var_A = ((0xb38fb5i32) as u16);
        (*E).pbg_var_06 = 0 as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
        (*E).base.current_instruction = addr_kBrinstarYellowPipeBug_Ilist_8EFC as uint16;
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
    } else {
        BrinstarYellowPipeBug_Func_4(ram_word!(game, cur_enemy_index), game);
        if (*E).pbg_var_F != 0 {
            BrinstarYellowPipeBug_Func_10(ram_word!(game, cur_enemy_index), game);
        } else {
            BrinstarYellowPipeBug_Func_9(ram_word!(game, cur_enemy_index), game);
            if (((*E).base.y_pos) as i16).wrapping_sub(((*E).pbg_var_08) as i16) < 0i16 {
                (*E).pbg_var_06 = 1 as uint16;
                (*E).pbg_var_F = 1 as uint16;
                (*E).pbg_var_A = ((0xb39028i32) as u16);
                (*E).base.instruction_timer = 1 as uint16;
                (*E).base.timer = 0 as uint16;
                (*E).base.current_instruction = addr_kBrinstarYellowPipeBug_Ilist_8EFC as uint16;
            }
        }
    };
}
pub unsafe fn BrinstarYellowPipeBug_Func_8(game: &mut Game) {
    let E: *mut Enemy_PipeBug = Get_PipeBug(ram_word!(game, cur_enemy_index) as i32, game);
    if CheckIfEnemyIsOnScreen(game) != 0 {
        (*E).base.x_pos = (*E).pbg_var_07;
        (*E).base.y_pos = (*E).pbg_var_08;
        (*E).base.x_subpos = 0 as uint16;
        (*E).base.y_subpos = 0 as uint16;
        (*E).pbg_var_A = ((0xb38fb5i32) as u16);
        (*E).pbg_var_06 = 0 as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
        (*E).base.current_instruction = addr_kBrinstarYellowPipeBug_Ilist_8F24 as uint16;
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
    } else {
        BrinstarYellowPipeBug_Func_6(ram_word!(game, cur_enemy_index), game);
        if (*E).pbg_var_F != 0 {
            BrinstarYellowPipeBug_Func_10(ram_word!(game, cur_enemy_index), game);
        } else {
            BrinstarYellowPipeBug_Func_9(ram_word!(game, cur_enemy_index), game);
            if (((*E).pbg_var_08) as i16).wrapping_sub(((*E).base.y_pos) as i16) >= 0i16 {
                (*E).pbg_var_06 = 1 as uint16;
                (*E).pbg_var_F = 1 as uint16;
                (*E).base.instruction_timer = 1 as uint16;
                (*E).base.timer = 0 as uint16;
                (*E).base.current_instruction = addr_kBrinstarYellowPipeBug_Ilist_8F24 as uint16;
                (*E).pbg_var_A = ((0xb390bdi32) as u16);
            }
        }
    };
}
pub unsafe fn BrinstarYellowPipeBug_Func_9(k: uint16, game: &mut Game) {
    let E: *mut Enemy_PipeBug = Get_PipeBug(k as i32, game);
    (*E).pbg_var_E = (*E).pbg_var_E.wrapping_add(1);
    let v3: i32 = 8 as i32 * (*E).pbg_var_E as i32 >> 1 as i32;
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa2838f)) as *const uint8 as *mut uint16)
            .offset((v3 + 3 as i32) as isize) as uint32)
            << 16 as i32
            | *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2838f)) as *const uint8
                as *mut uint16)
                .offset((v3 + 2 as i32) as isize) as uint32,
    );
}
pub unsafe fn BrinstarYellowPipeBug_Func_10(k: uint16, game: &mut Game) {
    let E: *mut Enemy_PipeBug = Get_PipeBug(k as i32, game);
    (*E).pbg_var_E = (*E).pbg_var_E.wrapping_sub(1);
    if ((*E).pbg_var_E as i32 & 0x8000 as i32) != 0i32 {
        (*E).pbg_var_E = 0 as uint16;
        (*E).pbg_var_F = 0 as uint16;
    } else {
        let v4: i32 = 8 as i32 * (*E).pbg_var_E as i32 >> 1 as i32;
        AddToHiLo(
            &raw mut (*E).base.y_pos,
            &raw mut (*E).base.y_subpos,
            (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa2838f)) as *const uint8 as *mut uint16)
                .offset((v4 + 1 as i32) as isize) as uint32)
                << 16 as i32
                | *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2838f)) as *const uint8
                    as *mut uint16)
                    .offset(v4 as isize) as uint32,
        );
    };
}
pub unsafe fn Botwoon_Instr_1(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let E: *mut Enemy_Botwoon = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.x_width = 8 as uint16;
    (*E).base.y_height = 16 as uint16;
    jp
}
pub unsafe fn Botwoon_Instr_2(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let E: *mut Enemy_Botwoon = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.x_width = 12 as uint16;
    (*E).base.y_height = 12 as uint16;
    jp
}
pub unsafe fn Botwoon_Instr_3(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let E: *mut Enemy_Botwoon = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.x_width = 16 as uint16;
    (*E).base.y_height = 8 as uint16;
    jp
}
pub unsafe fn Botwoon_Instr_4(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let E: *mut Enemy_Botwoon = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.x_width = 12 as uint16;
    (*E).base.y_height = 12 as uint16;
    jp
}
pub unsafe fn Botwoon_Instr_5(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let E: *mut Enemy_Botwoon = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.x_width = 8 as uint16;
    (*E).base.y_height = 16 as uint16;
    jp
}
pub unsafe fn Botwoon_Instr_6(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let E: *mut Enemy_Botwoon = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.x_width = 8 as uint16;
    (*E).base.y_height = 16 as uint16;
    jp
}
pub unsafe fn Botwoon_Instr_7(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let E: *mut Enemy_Botwoon = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.x_width = 12 as uint16;
    (*E).base.y_height = 12 as uint16;
    jp
}
pub unsafe fn Botwoon_Instr_8(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let E: *mut Enemy_Botwoon = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.x_width = 16 as uint16;
    (*E).base.y_height = 8 as uint16;
    jp
}
pub unsafe fn Botwoon_Instr_9(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let E: *mut Enemy_Botwoon = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.x_width = 12 as uint16;
    (*E).base.y_height = 12 as uint16;
    jp
}
pub unsafe fn Botwoon_Instr_10(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let E: *mut Enemy_Botwoon = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.x_width = 8 as uint16;
    (*E).base.y_height = 16 as uint16;
    jp
}
pub unsafe fn Botwoon_Instr_SetSpitting(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game)).botwoon_var_5A = 1 as uint16;
    jp
}
pub unsafe fn Botwoon_Instr_QueueSpitSfx(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    QueueSfx2_Max6(0x7c as uint16, game);
    jp
}
pub unsafe fn Botwoon_QueueExplosionSfx(game: &mut Game) {
    QueueSfx2_Max6(0x24 as uint16, game);
}
pub unsafe fn Botwoon_Init(game: &mut Game) {
    let E: *mut Enemy_Botwoon = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    if (*(&raw mut game.g_ram as *mut uint8).offset(0xd828).offset(4) as i32 & 2 as i32) != 0i32 {
        SpawnHardcodedPlm(
            SpawnHardcodedPlmArgs::new(0xf as uint8, 0x4 as uint8, 0xb797 as VoidP),
            game,
        );
        *((&raw mut game.g_ram as *mut uint8).offset(0xcd20) as *mut uint16) = 257 as uint16;
        (*E).base.current_instruction = addr_kBotwoon_Ilist_9389 as uint16;
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
    } else {
        let v3: uint16 = 24 as uint16;
        (*E).botwoon_var_A = 24 as uint16;
        let mut v5: bool = false;
        loop {
            SpawnEprojWithGfx(
                v3,
                ram_word!(game, cur_enemy_index),
                addr_kEproj_BotwoonsBody as uint16,
                game,
            );
            (*E).botwoon_var_A = (*E).botwoon_var_A.wrapping_sub(1);
            v5 = (((*E).botwoon_var_A) as i16).wrapping_sub(1i16) < 0i16;
            (*E).botwoon_var_A = (*E).botwoon_var_A.wrapping_sub(1);
            if v5 {
                break;
            }
        }
        (*E).base.current_instruction = addr_kBotwoon_Ilist_9389 as uint16;
        (*E).botwoon_var_3B = addr_kBotwoon_Ilist_9389 as uint16;
        (*E).botwoon_var_D = ((0xb39878i32) as u16);
        (*E).botwoon_var_E = ((0xb39bb7i32) as u16);
        (*E).botwoon_var_F = ((0xb39dc0i32) as u16);
        (*E).botwoon_var_20 = 256 as uint16;
        (*E).botwoon_var_38 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xb394bb))
            as *const uint8 as *mut uint16)
            .offset(0);
        (*E).botwoon_var_C =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xb394bb)) as *const uint8 as *mut uint16)
                .offset(1);
        (*E).botwoon_var_33 = 1 as uint16;
        (*E).botwoon_var_34 = 1 as uint16;
        (*E).botwoon_var_59 = 1 as uint16;
        (*E).botwoon_var_36 = -(1 as i32) as uint16;
        (*E).botwoon_var_37 = 0 as uint16;
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
        let mut health: uint16 = (*E).base.health;
        (*E).botwoon_var_5B = health;
        health = (health as i32 >> 1 as i32) as uint16;
        (*E).botwoon_var_5C = health;
        (*E).botwoon_var_5D = (health as i32 >> 1 as i32) as uint16;
        let x_pos: uint16 = (*E).base.x_pos;
        (*E).botwoon_var_50 = x_pos;
        (*E).botwoon_var_52 = x_pos;
        (*E).botwoon_var_54 = x_pos;
        (*E).botwoon_var_56 = x_pos;
        let y_pos: uint16 = (*E).base.y_pos;
        (*E).botwoon_var_51 = y_pos;
        (*E).botwoon_var_53 = y_pos;
        (*E).botwoon_var_55 = y_pos;
        (*E).botwoon_var_57 = y_pos;
        (*E).botwoon_var_4E = (((*E).base.palette_index as i32 >> 4 as i32) + 256 as i32) as uint16;
    };
}
pub unsafe fn CallBotwoonFuncVarD(ea: uint32, k: uint16, game: &mut Game) {
    match ea {
        11769976 => Botwoon_Func_6(game),
        11770013 => Botwoon_Func_7(game),
        11770276 => Botwoon_Func_12(game),
        11770340 => Botwoon_Func_13(game),
        11770438 => Botwoon_Func_14(game),
        11770462 => Botwoon_Func_15(game),
        11770570 => Botwoon_Func_16(game),
        11770617 => Botwoon_Func_18(k, game),
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn CallBotwoonFuncVarE(ea: uint32, game: &mut Game) {
    match ea {
        11770807 => Botwoon_Func_19(game),
        11788880 => Botwoon_Func_32(game),
        11788940 => Botwoon_Func_33(game),
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn Botwoon_Main(game: &mut Game) {
    Botwoon_Func_3(game);
    let E: *mut Enemy_Botwoon = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    CallBotwoonFuncVarD(
        (((*E).botwoon_var_D) as u32) | (0xb30000u32),
        ram_word!(game, cur_enemy_index),
        game,
    );
    Botwoon_HealthBasedPalHandling(game);
}
pub unsafe fn Botwoon_Func_1(k: uint16, game: &mut Game) {
    let E: *mut Enemy_Botwoon = Get_Botwoon(k as i32, game);
    if (*(game.g_rom.offset(crate::rtl::lorom_offset(0xb39675)) as *const uint8 as *mut uint16)
        .offset((*E).botwoon_var_3F as isize) as i32
        & NextRandom(game) as i32)
        == 0i32
    {
        (*E).botwoon_var_2E = 1 as uint16;
    }
}
pub unsafe fn Botwoon_Func_2(game: &mut Game) {
    let E: *mut Enemy_Botwoon = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).botwoon_var_2F == 0
        && (*E).botwoon_var_2E != 0u16
        && (*E).botwoon_var_33 == 0
        && (*E).botwoon_var_21 == 0
    {
        (*E).botwoon_var_F = ((0xb39e7di32) as u16);
        (*E).botwoon_var_21 = 24 as uint16;
    }
    (*E).botwoon_var_2E = 0 as uint16;
}
pub unsafe fn Botwoon_Func_3(game: &mut Game) {
    let E: *mut Enemy_Botwoon = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).botwoon_var_2F != 0 && (*E).botwoon_var_3E != 0 {
        (*E).botwoon_var_30 = 1 as uint16;
        (*E).botwoon_var_D = ((0xb39a46i32) as u16);
        (*E).botwoon_var_22 = 240 as uint16;
        (*E).botwoon_var_2F = 0 as uint16;
        (*E).botwoon_var_3E = 0 as uint16;
    }
}
pub unsafe fn Botwoon_Func_4(game: &mut Game) {
    let E: *mut Enemy_Botwoon = Get_Botwoon(0 as i32, game);
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
}
pub unsafe fn Botwoon_Func_5(game: &mut Game) {
    let mut v0: uint16 = 34 as uint16;
    loop {
        update_ram_word!(
            game.eproj_properties[(v0 as i32 >> 1 as i32)],
            |fresh4| (fresh4) | (0x2000u16)
        );
        set_ram_word!(game.eproj_flags[(v0 as i32 >> 1i32)] = 2 as uint16);
        v0 = (v0).wrapping_sub(2u16);
        if (((v0) as i16).wrapping_sub(10i16) < 0i16) {
            break;
        }
    }
}
pub unsafe fn Botwoon_HealthBasedPalHandling(game: &mut Game) {
    let E: *mut Enemy_Botwoon = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).botwoon_var_4F != 16u16
        && (((*E).base.health) as i16).wrapping_sub(
            (*(game.g_rom.offset(crate::rtl::lorom_offset(0xb3981b)) as *const uint8
                as *mut uint16)
                .offset(((*E).botwoon_var_4F as i32 >> 1 as i32) as isize)) as i16,
        ) < 0i16
    {
        let mut r18: uint16 = (16u16).wrapping_mul((*E).botwoon_var_4F);
        let mut r20: uint16 = (*E).botwoon_var_4E;
        loop {
            let fresh5 = r18;
            r18 = r18.wrapping_add(1);
            set_ram_word!(
                game.palette_buffer[(r20 as i32 >> 1i32)] =
                    *(game.g_rom.offset(crate::rtl::lorom_offset(0xb3971b)) as *const uint8
                        as *mut uint16)
                        .offset((fresh5 as i32 >> 1 as i32) as isize)
            );
            r18 = r18.wrapping_add(1);
            r20 = (r20).wrapping_add(2u16);
            if (r20 == 512u16) {
                break;
            }
        }
        (*E).botwoon_var_4F = ((*E).botwoon_var_4F).wrapping_add(2u16);
    }
}
pub unsafe fn Botwoon_Func_6(game: &mut Game) {
    let E: *mut Enemy_Botwoon = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    let v1: uint16 = ((*E).botwoon_var_20).wrapping_sub(1u16);
    (*E).botwoon_var_20 = v1;
    if v1 == 0 {
        (*E).botwoon_var_D = ((0xb3989di32) as u16);
    }
}
pub unsafe fn Botwoon_Func_7(game: &mut Game) {
    let E: *mut Enemy_Botwoon = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).botwoon_var_3C != 0 {
        (*E).botwoon_var_3C = 0 as uint16;
        let mut r18: uint8 = 0 as uint8;
        if (*E).botwoon_var_33 == 0 && (*E).botwoon_var_59 == 0 && (*E).botwoon_var_3F == 0 {
            r18 = ((NextRandom(game)) as u8) & (0xeu8);
        }
        (*E).botwoon_var_59 = 0 as uint16;
        if r18 == 0 || r18 == 2u8 || r18 == 4u8 {
            Botwoon_Func_8(game);
        } else {
            if r18 != 6u8 && r18 != 8u8 && r18 != 10u8 && r18 != 12u8 && r18 != 14u8 {
                Unreachable(game);
            }
            Botwoon_Func_9(game);
        }
    } else {
        CallBotwoonFuncVarE((((*E).botwoon_var_E) as u32) | (0xb30000u32), game);
        Botwoon_Func_22(game);
        Botwoon_Func_23(game);
        CallEnemyPreInstr((((*E).botwoon_var_F) as u32) | (0xb30000u32), game);
        Botwoon_Func_25(game);
        Botwoon_Func_24(game);
        Botwoon_Func_31(ram_word!(game, cur_enemy_index), game);
    };
}
pub unsafe fn Botwoon_Func_8(game: &mut Game) {
    let E: *mut Enemy_Botwoon = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).botwoon_var_D = ((0xb399a4i32) as u16);
    (*E).botwoon_var_E = ((0xb3e250i32) as u16);
    (*E).botwoon_var_21 = 0 as uint16;
    (*E).botwoon_var_F = ((0xb39dc0i32) as u16);
    (*E).botwoon_var_4B = 0 as uint16;
    Botwoon_Func_10(ram_word!(game, cur_enemy_index), game);
}
pub unsafe fn Botwoon_Func_9(game: &mut Game) {
    let E: *mut Enemy_Botwoon = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).botwoon_var_D = ((0xb399e4i32) as u16);
    (*E).botwoon_var_F = ((0xb39e7di32) as u16);
    (*E).botwoon_var_21 = 48 as uint16;
    (*E).base.properties = ((*E).base.properties as i32 & !(kEnemyProps_Tangible as i32)) as uint16;
}
pub unsafe fn Botwoon_Func_10(k: uint16, game: &mut Game) {
    Botwoon_Func_11(k, game);
    let E: *mut Enemy_Botwoon = Get_Botwoon(k as i32, game);
    let r18: uint16 = (if (*E).botwoon_var_33 != 0u16 {
        128 as i32
    } else {
        0 as i32
    }) as uint16;
    (*E).botwoon_var_40 = ((NextRandom(game) as i32 & 0x18 as i32)
        + r18 as i32
        + 4 as i32 * (*E).botwoon_var_37 as i32) as uint16;
}
pub unsafe fn Botwoon_Func_11(k: uint16, game: &mut Game) {
    let mut health: int16 = 0;
    let E: *mut Enemy_Botwoon = Get_Botwoon(k as i32, game);
    if (*E).botwoon_var_33 == 0 {
        (*E).botwoon_var_3F = 0 as uint16;
        health = (*E).base.health as int16;
        if health > 0i16 && (health).wrapping_sub(((*E).botwoon_var_5C) as i16) < 0i16 {
            if (health).wrapping_sub(((*E).botwoon_var_5D) as i16) < 0i16 {
                (*E).botwoon_var_3F = 2 as uint16;
            } else {
                (*E).botwoon_var_3F = 1 as uint16;
            }
            let v3: i32 = ((4u16).wrapping_mul((*E).botwoon_var_3F)) as i32 >> 1 as i32;
            (*E).botwoon_var_38 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xb394bb))
                as *const uint8 as *mut uint16)
                .offset(v3 as isize);
            (*E).botwoon_var_C = *(game.g_rom.offset(crate::rtl::lorom_offset(0xb394bb))
                as *const uint8 as *mut uint16)
                .offset((v3 + 1 as i32) as isize);
        }
    }
}
pub unsafe fn Botwoon_Func_12(game: &mut Game) {
    let E: *mut Enemy_Botwoon = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).botwoon_var_3C != 0 {
        (*E).botwoon_var_3C = 0 as uint16;
        (*E).botwoon_var_D = ((0xb3989di32) as u16);
        (*E).botwoon_var_E = ((0xb39bb7i32) as u16);
        if (*E).botwoon_var_33 != 0 {
            (*E).botwoon_var_35 = 0 as uint16;
        }
    } else {
        CallBotwoonFuncVarE((((*E).botwoon_var_E) as u32) | (0xb30000u32), game);
        Botwoon_Func_22(game);
        Botwoon_Func_23(game);
        CallEnemyPreInstr((((*E).botwoon_var_F) as u32) | (0xb30000u32), game);
        Botwoon_Func_25(game);
        Botwoon_Func_24(game);
    };
}
pub unsafe fn Botwoon_Func_13(game: &mut Game) {
    let mut v3: uint16 = 0;
    let E: *mut Enemy_Botwoon = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).botwoon_var_21 != 0 {
        (*E).botwoon_var_2E = 0 as uint16;
        CallEnemyPreInstr((((*E).botwoon_var_F) as u32) | (0xb30000u32), game);
    } else {
        (*E).botwoon_var_3C = 0 as uint16;
        (*E).botwoon_var_D = ((0xb399a4i32) as u16);
        (*E).botwoon_var_E = ((0xb3e250i32) as u16);
        (*E).botwoon_var_F = ((0xb39dc0i32) as u16);
        if (*E).botwoon_var_2F != 0u16 || {
            v3 = (NextRandom(game)) & (1u16);
            (*E).botwoon_var_33 = v3;
            (*E).botwoon_var_34 = v3;
            (*E).botwoon_var_34 == 0u16
        } {
            (*E).botwoon_var_4B = 0 as uint16;
        } else {
            (*E).botwoon_var_35 = 0 as uint16;
            (*E).botwoon_var_36 = -(1 as i32) as uint16;
        }
        Botwoon_Func_10(ram_word!(game, cur_enemy_index), game);
    };
}
pub unsafe fn Botwoon_Func_14(game: &mut Game) {
    let E: *mut Enemy_Botwoon = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    let v1: uint16 = ((*E).botwoon_var_22).wrapping_add(1u16);
    (*E).botwoon_var_22 = v1;
    if v1 as i32 - 256 as i32 & 0x8000 as i32 == 0 {
        (*E).botwoon_var_D = ((0xb39a5ei32) as u16);
    }
}
pub unsafe fn Botwoon_Func_15(game: &mut Game) {
    let E: *mut Enemy_Botwoon = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    let v2: i32 = 8 as i32
        * *(&raw mut (*E).botwoon_var_28 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1_usize) as isize,
        ) as i32
        >> 1 as i32;
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa2838f)) as *const uint8 as *mut uint16)
            .offset((v2 + 1 as i32) as isize) as uint32)
            << 16 as i32
            | *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2838f)) as *const uint8
                as *mut uint16)
                .offset(v2 as isize) as uint32,
    );
    if (*E).base.y_pos as i32 - 200 as i32 & 0x8000 as i32 != 0 {
        (*E).botwoon_var_28 = ((*E).botwoon_var_28).wrapping_add(192u16);
    } else {
        (*E).base.y_pos = 200 as uint16;
        (*E).botwoon_var_D = ((0xb39acai32) as u16);
        game.eproj_spawn_pt = Point16U {
            x: (*E).base.x_pos,
            y: (*E).base.y_pos,
        };
        SpawnEprojWithRoomGfx(
            addr_kEproj_DustCloudExplosion as uint16,
            0x1d as uint16,
            game,
        );
        Botwoon_QueueExplosionSfx(game);
        (*E).base.properties = ((*E).base.properties) | (0x500u16);
    };
}
pub unsafe fn Botwoon_Func_16(game: &mut Game) {
    let E: *mut Enemy_Botwoon = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).botwoon_var_5F != 0 {
        (*E).botwoon_var_D = ((0xb39af9i32) as u16);
        Botwoon_Func_17(ram_word!(game, cur_enemy_index), game);
    }
}
pub unsafe fn Botwoon_Func_17(k: uint16, game: &mut Game) {
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0xf as uint8, 0x4 as uint8, 0xb79b as VoidP),
        game,
    );
    Enemy_ItemDrop_Botwoon(k, game);
    let E: *mut Enemy_Botwoon = Get_Botwoon(k as i32, game);
    (*E).botwoon_var_23 = 0 as uint16;
    (*E).botwoon_var_24 = 0 as uint16;
    (*E).botwoon_var_25 = 0 as uint16;
}
pub unsafe fn Botwoon_Func_18(k: uint16, game: &mut Game) {
    let E: *mut Enemy_Botwoon = Get_Botwoon(k as i32, game);
    if (*E).botwoon_var_23 as i32 - 192 as i32 & 0x8000 as i32 != 0 {
        if (*E).botwoon_var_23 as i32 - 64 as i32 & 0x8000 as i32 == 0 {
            let v2: int16 = (((*E).botwoon_var_24) as i16).wrapping_sub(1i16);
            (*E).botwoon_var_24 = v2 as uint16;
            if v2 < 0i16 {
                (*E).botwoon_var_24 = 12 as uint16;
                let x: uint16 = ((NextRandom(game) as i32 & 0x1f as i32) + 232 as i32) as uint16;
                let y: uint16 = ((*E).botwoon_var_23 as i32
                    + (NextRandom(game) as i32 & 0x1f as i32)
                    - 8 as i32) as uint16;
                CreateSpriteAtPos(x, y, 29 as uint16, 2560 as uint16, game);
                Botwoon_QueueExplosionSfx(game);
            }
            let v3: int16 = (((*E).botwoon_var_25) as i16).wrapping_sub(1i16);
            (*E).botwoon_var_25 = v3 as uint16;
            if v3 < 0i16 {
                (*E).botwoon_var_25 = 4 as uint16;
                let mut n: i32 = 2 as i32;
                loop {
                    let x_0: uint16 =
                        ((NextRandom(game) as i32 & 0x3f as i32) + 224 as i32) as uint16;
                    let y_0: uint16 = ((*Get_Botwoon(k as i32, game)).botwoon_var_23 as i32
                        + ((NextRandom(game) as i32 & 0x1f as i32) - 8 as i32))
                        as uint16;
                    CreateSpriteAtPos(x_0, y_0, 9 as uint16, 2560 as uint16, game);
                    n -= 1;
                    if (n == 0) {
                        break;
                    }
                }
            }
        }
        (*E).botwoon_var_23 = (*E).botwoon_var_23.wrapping_add(1);
    } else {
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
        SetBossBitForCurArea(2 as uint16, game);
        QueueMusic_Delayed8(3 as uint16, game);
    };
}
pub unsafe fn Botwoon_Func_19(game: &mut Game) {
    let pt: Point16U = Botwoon_Func_20(ram_word!(game, cur_enemy_index), game);
    let v1: uint16 = CalculateAngleFromXY(pt.x, pt.y, game);
    let E: *mut Enemy_Botwoon = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).botwoon_var_39 = v1;
    (*E).botwoon_var_3A = ((64u8).wrapping_sub((v1) as u8)) as uint16;
    if (*E).botwoon_var_33 == (*E).botwoon_var_34 {
        Botwoon_Func_21(game);
    } else {
        (*E).botwoon_var_34 = (*E).botwoon_var_33;
        (*E).botwoon_var_3C = 1 as uint16;
    };
}
pub unsafe fn Botwoon_Func_20(k: uint16, game: &mut Game) -> Point16U {
    let E: *mut Enemy_Botwoon = Get_Botwoon(k as i32, game);
    let v2: i32 = (*E).botwoon_var_37 as i32 >> 1 as i32;
    let v3: uint16 = (*(game.g_rom.offset(crate::rtl::lorom_offset(0xb3949b)) as *const uint8
        as *mut uint16)
        .offset(v2 as isize) as i32
        + 4 as i32
        - (*E).base.x_pos as i32) as uint16;
    let mut r18: uint16 = v3;
    if v3 as i32 - 256 as i32 & 0x8000 as i32 != 0 {
        if v3 as i32 + 256 as i32 & 0x8000 as i32 != 0 {
            r18 = -(255 as i32) as uint16;
        }
    } else {
        r18 = 255 as uint16;
    }
    let v4: uint16 = (*(game.g_rom.offset(crate::rtl::lorom_offset(0xb3949b)) as *const uint8
        as *mut uint16)
        .offset((v2 + 2 as i32) as isize) as i32
        + 4 as i32
        - (*E).base.y_pos as i32) as uint16;
    let mut r20: uint16 = v4;
    if v4 as i32 - 256 as i32 & 0x8000 as i32 != 0 {
        if v4 as i32 + 256 as i32 & 0x8000 as i32 != 0 {
            r20 = -(255 as i32) as uint16;
        }
    } else {
        r20 = 255 as uint16;
    }
    Point16U { x: r18, y: r20 }
}
pub unsafe fn Botwoon_Func_21(game: &mut Game) {
    let E: *mut Enemy_Botwoon = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    let pt: Point32 = ConvertAngleToXy((*E).botwoon_var_3A, (*E).botwoon_var_38, game);
    EnemyFunc_B691((*E).botwoon_var_3A, pt, game);
}
pub unsafe fn Botwoon_Func_22(game: &mut Game) {
    let E: *mut Enemy_Botwoon = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    let botwoon_var_B: uint16 = (*E).botwoon_var_B;
    *(((&raw mut game.g_ram as *mut uint8).offset(0x9000) as *mut uint16 as *mut uint8)
        .offset(botwoon_var_B as i32 as isize) as *mut uint16) = (*E).base.x_pos;
    *(((&raw mut game.g_ram as *mut uint8).offset(0x9002) as *mut uint16 as *mut uint8)
        .offset(botwoon_var_B as i32 as isize) as *mut uint16) = (*E).base.y_pos;
}
pub unsafe fn Botwoon_Func_23(game: &mut Game) {
    let E: *mut Enemy_Botwoon = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    let mut r18: uint16 =
        ((*E).botwoon_var_B as i32 - (*E).botwoon_var_C as i32 & 0x3ff as i32) as uint16;
    let mut n: i32 = 24 as i32;
    loop {
        let ET: *mut Enemy_Botwoon = Get_Botwoon(n, game);
        let botwoo_var_00: uint16 = (*ET).botwoon_var_00;
        if (*E).botwoon_var_36 == r18 {
            let v2: uint16 = n as uint16;
            let v4: bool = (*ET).botwoon_var_10 == 1u16;
            (*ET).botwoon_var_10 = ((*ET).botwoon_var_10) ^ (1u16);
            if v4 {
                update_ram_word!(
                    game.eproj_properties[(botwoo_var_00 as i32 >> 1 as i32)],
                    |fresh0| (fresh0 as i32 & !(0x2000 as i32)) as uint16
                );
                *((&raw mut game.g_ram as *mut uint8).offset(0xf380) as *mut uint16)
                    .offset((botwoo_var_00 as i32 >> 1 as i32) as isize) = 1 as uint16;
            } else {
                update_ram_word!(
                    game.eproj_properties[(botwoo_var_00 as i32 >> 1 as i32)],
                    |fresh1| (fresh1) | (0x2000u16)
                );
                *((&raw mut game.g_ram as *mut uint8).offset(0xf380) as *mut uint16)
                    .offset((botwoo_var_00 as i32 >> 1 as i32) as isize) = 2 as uint16;
            }
            if v2 == 0 {
                (*E).botwoon_var_35 = 0 as uint16;
                (*E).botwoon_var_3E = 0 as uint16;
                (*E).botwoon_var_36 = -(1 as i32) as uint16;
                if (*E).botwoon_var_33 == 0 {
                    (*E).botwoon_var_3E = 1 as uint16;
                }
            }
        }
        let v7: i32 = botwoo_var_00 as i32 >> 1 as i32;
        set_ram_word!(
            game.eproj_x_pos[v7] = *(((&raw mut game.g_ram as *mut uint8).offset(0x9000)
                as *mut uint16 as *mut uint8)
                .offset(r18 as i32 as isize) as *mut uint16)
        );
        set_ram_word!(
            game.eproj_y_pos[v7] = *(((&raw mut game.g_ram as *mut uint8).offset(0x9002)
                as *mut uint16 as *mut uint8)
                .offset(r18 as i32 as isize) as *mut uint16)
        );
        r18 = (r18 as i32 - (*E).botwoon_var_C as i32 & 0x3ff as i32) as uint16;
        n -= 2 as i32;
        if ((n & 0x8000 as i32) != 0i32) {
            break;
        }
    }
}
pub unsafe fn Botwoon_Func_24(game: &mut Game) {
    let E: *mut Enemy_Botwoon = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).botwoon_var_B = ((*E).botwoon_var_B as i32 + 4 as i32 & 0x3ff as i32) as uint16;
}
pub unsafe fn Botwoon_Func_25(game: &mut Game) {
    let E: *mut Enemy_Botwoon = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    let mut n: i32 = 24 as i32;
    loop {
        let mut r24: uint16 = 0 as uint16;
        let ERR: *mut Enemy_Botwoon = Get_Botwoon(n, game);
        let botwoo_var_00: uint16 = (*ERR).botwoon_var_00;
        if (*ERR).botwoon_var_10 != 0 {
            r24 = 256 as uint16;
        }
        let mut x: uint16 = 0;
        let mut y: uint16 = 0;
        if n == 24i32 {
            let v4: i32 = botwoo_var_00 as i32 >> 1 as i32;
            x = ((*E).base.x_pos).wrapping_sub(ram_word!(game, eproj_x_pos[v4]));
            y = ((*E).base.y_pos).wrapping_sub(ram_word!(game, eproj_y_pos[v4]));
        } else {
            if n == 0 {
                r24 = (r24).wrapping_add(512u16);
            }
            let v2: i32 = botwoo_var_00 as i32 >> 1 as i32;
            x = (ram_word!(game, eproj_x_pos[(v2 + 1i32)]))
                .wrapping_sub(ram_word!(game, eproj_x_pos[v2]));
            y = (ram_word!(game, eproj_y_pos[(v2 + 1i32)]))
                .wrapping_sub(ram_word!(game, eproj_y_pos[v2]));
        }
        set_ram_word!(
            game.eproj_E[(botwoo_var_00 as i32 >> 1i32)] = (2 as i32
                * (((r24).wrapping_add(CalculateAngleFromXY(x, y, game))) as i32 >> 5 as i32))
                as uint16
        );
        n -= 2 as i32;
        if (n < 0i32) {
            break;
        }
    }
}
pub unsafe fn Botwoon_Func_26(k: uint16, game: &mut Game) {
    let E: *mut Enemy_Botwoon = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    let x: uint16 = ((*E).base.x_pos).wrapping_sub((*E).botwoon_var_56);
    let y: uint16 = ((*E).base.y_pos).wrapping_sub((*E).botwoon_var_57);
    let r22: uint16 = CalculateAngleFromXY(x, y, game);
    if x != 0u16 || y != 0u16 {
        let mut v1: uint16 = 0;
        if (*E).botwoon_var_33 != 0 {
            (*E).base.layer = 7 as uint16;
            (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
            v1 = addr_kBotwoon_Ilist_9389 as uint16;
        } else {
            (*E).base.layer = 2 as uint16;
            (*E).base.properties =
                ((*E).base.properties as i32 & !(kEnemyProps_Tangible as i32)) as uint16;
            (*E).botwoon_var_45 = ((*E).botwoon_var_45 as i32
                + (((r22) as i8).wrapping_sub(((*E).botwoon_var_45) as i8)) as i32 * 3 as i32
                    / 4 as i32) as uint8 as uint16;
            v1 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xb3946b)) as *const uint8
                as *mut uint16)
                .offset(((*E).botwoon_var_45 as i32 >> 5 as i32) as isize);
        }
        if v1 != (*E).botwoon_var_3B {
            (*E).base.current_instruction = v1;
            (*E).botwoon_var_3B = v1;
            (*E).base.instruction_timer = 1 as uint16;
            (*E).base.timer = 0 as uint16;
        }
    }
    (*E).botwoon_var_56 = (*E).botwoon_var_54;
    (*E).botwoon_var_57 = (*E).botwoon_var_55;
    (*E).botwoon_var_54 = (*E).botwoon_var_52;
    (*E).botwoon_var_55 = (*E).botwoon_var_53;
    (*E).botwoon_var_52 = (*E).botwoon_var_50;
    (*E).botwoon_var_53 = (*E).botwoon_var_51;
    (*E).botwoon_var_50 = (*E).base.x_pos;
    (*E).botwoon_var_51 = (*E).base.y_pos;
}
pub unsafe fn Botwoon_Func_27(k: uint16, game: &mut Game) {
    let E: *mut Enemy_Botwoon = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.layer = 2 as uint16;
    let v2: uint16 = CalculateAngleOfSamusFromEnemy(ram_word!(game, cur_enemy_index), game);
    (*E).botwoon_var_3D = v2;
    let v3: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xb3948b)) as *const uint8
        as *mut uint16)
        .offset(((((v2) as u8).wrapping_add(16u8)) as i32 >> 5 as i32) as isize);
    (*E).base.current_instruction = v3;
    (*E).botwoon_var_3B = v3;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    (*E).botwoon_var_3D = ((64u8).wrapping_sub(((*E).botwoon_var_3D) as u8)) as uint16;
    (*E).botwoon_var_F = ((0xb39ee0i32) as u16);
    if (*E).botwoon_var_D != 0x99e4u16 {
        (*E).botwoon_var_F = ((0xb39f34i32) as u16);
    }
    CallEnemyPreInstr((((*E).botwoon_var_F) as u32) | (0xb30000u32), game);
}
pub unsafe fn Botwoon_Func_28(k: uint16, game: &mut Game) {
    let E: *mut Enemy_Botwoon = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).botwoon_var_5A != 0 {
        *(&raw mut game.g_ram as *mut uint8).offset(0x60b) =
            (((*E).botwoon_var_3D) as u8).wrapping_sub(32u8);
        let mut n: i32 = 5 as i32;
        let varE32: uint16 = g_word_B39E77[(*E).botwoon_var_3F as usize] as uint16;
        loop {
            SpawnEprojWithGfx(
                varE32,
                ram_word!(game, cur_enemy_index),
                addr_kEproj_BotwoonsSpit as uint16,
                game,
            );
            let ref mut fresh2 = *(&raw mut game.g_ram as *mut uint8).offset(0x60b);
            *fresh2 = (*fresh2).wrapping_add(16u8);
            n -= 1;
            if (n == 0) {
                break;
            }
        }
        (*E).botwoon_var_5A = 0 as uint16;
        (*E).botwoon_var_F = ((0xb39f7ai32) as u16);
    }
}
pub unsafe fn Botwoon_Func_29(k: uint16, game: &mut Game) {
    let E: *mut Enemy_Botwoon = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x60b) =
        (((*E).botwoon_var_3D) as u8).wrapping_sub(16u8);
    let mut n: i32 = 3 as i32;
    let varE32: uint16 = g_word_B39E77[(*E).botwoon_var_3F as usize] as uint16;
    loop {
        SpawnEprojWithGfx(
            varE32,
            ram_word!(game, cur_enemy_index),
            addr_kEproj_BotwoonsSpit as uint16,
            game,
        );
        let ref mut fresh3 = *(&raw mut game.g_ram as *mut uint8).offset(0x60b);
        *fresh3 = (*fresh3).wrapping_add(16u8);
        n -= 1;
        if (n == 0) {
            break;
        }
    }
    (*Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game)).botwoon_var_F =
        ((0xb39f7ai32) as u16);
}
pub unsafe fn Botwoon_Func_30(k: uint16, game: &mut Game) {
    let E: *mut Enemy_Botwoon = Get_Botwoon(k as i32, game);
    let v2: int16 = (((*E).botwoon_var_21) as i16).wrapping_sub(1i16);
    (*E).botwoon_var_21 = v2 as uint16;
    if v2 < 0i16 {
        (*E).botwoon_var_21 = 0 as uint16;
        (*E).botwoon_var_F = ((0xb39dc0i32) as u16);
    }
}
pub unsafe fn Botwoon_Func_31(k: uint16, game: &mut Game) {
    let E: *mut Enemy_Botwoon = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    if (*Get_Botwoon(k as i32, game)).botwoon_var_35 == 0 {
        let mut n: i32 = 24 as i32;
        loop {
            let v2: i32 = n >> 1 as i32;
            if (((*E).base.x_pos) as i16).wrapping_sub(
                (*(game.g_rom.offset(crate::rtl::lorom_offset(0xb3949b)) as *const uint8
                    as *mut uint16)
                    .offset(v2 as isize)) as i16,
            ) >= 0i16
                && (((*E).base.x_pos) as i16).wrapping_sub(
                    (*(game.g_rom.offset(crate::rtl::lorom_offset(0xb3949b)) as *const uint8
                        as *mut uint16)
                        .offset((v2 + 1 as i32) as isize)) as i16,
                ) < 0i16
                && (((*E).base.y_pos) as i16).wrapping_sub(
                    (*(game.g_rom.offset(crate::rtl::lorom_offset(0xb3949b)) as *const uint8
                        as *mut uint16)
                        .offset((v2 + 2 as i32) as isize)) as i16,
                ) >= 0i16
                && (((*E).base.y_pos) as i16).wrapping_sub(
                    (*(game.g_rom.offset(crate::rtl::lorom_offset(0xb3949b)) as *const uint8
                        as *mut uint16)
                        .offset((v2 + 3 as i32) as isize)) as i16,
                ) < 0i16
            {
                (*E).botwoon_var_35 = 1 as uint16;
                (*E).botwoon_var_33 = ((*E).botwoon_var_33) ^ (1u16);
                (*E).botwoon_var_36 = (*E).botwoon_var_B;
                (*E).botwoon_var_26 = (*E).botwoon_var_26.wrapping_sub(1);
                break;
            } else {
                (*E).botwoon_var_35 = 0 as uint16;
                n -= 8 as i32;
                if (n < 0i32) {
                    break;
                }
            }
        }
    }
}
pub unsafe fn Botwoon_Touch(game: &mut Game) {
    Enemy_NormalTouchAI_SkipDeathAnim_B3(game);
    let E: *mut Enemy_Botwoon = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).base.health == 0 {
        (*E).botwoon_var_2F = 1 as uint16;
        Botwoon_Func_4(game);
    }
}
pub unsafe fn Botwoon_Shot(game: &mut Game) {
    let E: *mut Enemy_Botwoon = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).botwoon_var_4C = (*E).base.health;
    Enemy_NormalShotAI_SkipSomeParts_B3(game);
    if (*E).base.health == 0 {
        (*E).botwoon_var_2F = 1 as uint16;
        Botwoon_Func_4(game);
    }
}
pub unsafe fn Botwoon_Powerbomb(game: &mut Game) {
    Enemy_NormalPowerBombAI_SkipDeathAnim_B3(game);
    let E: *mut Enemy_Botwoon = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).base.health == 0 {
        (*E).botwoon_var_2F = 1 as uint16;
        Botwoon_Func_4(game);
    }
}
pub unsafe fn Botwoon_Func_32(game: &mut Game) {
    let E: *mut Enemy_Botwoon = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).botwoon_var_E = addr_loc_B3E28C as uint16;
    (*E).botwoon_var_3C = 0 as uint16;
    let v1: *const uint8 = RomPtr_B3(
        ((*E).botwoon_var_40 as i32 + addr_stru_B3E150 as i32) as uint16_t,
        game,
    );
    (*E).botwoon_var_42 = *(v1 as *mut uint16);
    (*E).botwoon_var_37 = *(v1.offset(4) as *mut uint16);
    let v2: int16 = *(v1.offset(2) as *mut uint16) as int16;
    (*E).botwoon_var_44 = v2 as uint16;
    if v2 < 0i16 {
        (*E).botwoon_var_42 = ((*E).botwoon_var_42).wrapping_sub(4u16);
    }
    Botwoon_Func_33(game);
}
pub unsafe fn Botwoon_Func_33(game: &mut Game) {
    let mut x: uint16 = 0 as uint16;
    let mut y: uint16 = 0 as uint16;
    let E: *mut Enemy_Botwoon = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    let mut n: i32 = (*E).botwoon_var_38 as i32;
    let step: i32 = if ((*E).botwoon_var_44 as i32 & 0x8000 as i32) == 0i32 {
        2 as i32
    } else {
        -(2 as i32)
    };
    loop {
        let v6: *const uint8 = RomPtr_B3((*E).botwoon_var_42, game);
        let v7: uint16 = SignExtend8(*v6 as uint16);
        let mut v8: uint16 = 0;
        if v7 == 0xff80u16 || {
            x = (x).wrapping_add(v7);
            v8 = SignExtend8(*v6.offset(1) as uint16);
            v8 == 0xff80u16
        } {
            (*E).botwoon_var_41 = 0 as uint16;
            (*E).botwoon_var_3C = 1 as uint16;
            return;
        }
        y = (y).wrapping_add(v8);
        (*E).botwoon_var_42 = ((*E).botwoon_var_42 as i32 + step) as uint16;
        n -= 1;
        if (n == 0) {
            break;
        }
    }
    if ((*E).botwoon_var_44 as i32 & 0x8000 as i32) != 0i32 {
        x = -(x as i32) as uint16;
        y = -(y as i32) as uint16;
    }
    (*E).base.x_pos = ((*E).base.x_pos).wrapping_add(x);
    (*E).base.y_pos = ((*E).base.y_pos).wrapping_add(y);
}
pub unsafe fn EscapeEtecoon_Instr_1(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    if ram_word!(game, lava_acid_y_pos) >= 0xceu16 {
        jp.offset(1)
    } else {
        *jp.offset(0) as uintptr_t as *const uint16
    }
}
pub unsafe fn EscapeEtecoon_Instr_2(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let E: *mut Enemy_EscapeEtecoon = Get_EscapeEtecoon(k as i32, game);
    (*E).base.x_pos = ((*E).base.x_pos).wrapping_add(*jp.offset(0));
    jp.offset(1)
}
pub unsafe fn EscapeEtecoon_Main(game: &mut Game) {
    let E: *mut Enemy_EscapeEtecoon =
        Get_EscapeEtecoon(ram_word!(game, cur_enemy_index) as i32, game);
    CallEnemyPreInstr((((*E).een_var_F) as u32) | (0xb30000u32), game);
}
pub unsafe fn EscapeEtecoon_E65C(k: uint16, game: &mut Game) {
    let E: *mut Enemy_EscapeEtecoon = Get_EscapeEtecoon(k as i32, game);
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        0x38000 as uint32,
    );
}
pub unsafe fn EscapeEtecoon_E670(k: uint16, game: &mut Game) {
    if CheckEventHappened(0xf as uint16, game) != 0 {
        (*Get_EscapeEtecoon(k as i32, game))
            .base
            .current_instruction = addr_kEscapeEtecoon_Ilist_E5DA as uint16;
    }
}
pub unsafe fn EscapeEtecoon_E680(k: uint16, game: &mut Game) {
    let E: *mut Enemy_EscapeEtecoon = Get_EscapeEtecoon(k as i32, game);
    if Enemy_MoveRight_IgnoreSlopes(k, ((*E).een_var_A as int16 as int32) << 8 as i32, game) != 0 {
        (*E).base.instruction_timer = 1 as uint16;
        let v3: bool = (-((*E).een_var_A as i32) & 0x8000 as i32) != 0i32;
        (*E).een_var_A = -((*E).een_var_A as i32) as uint16;
        (*E).base.current_instruction = (if v3 {
            addr_kEscapeEtecoon_Ilist_E556 as i32
        } else {
            addr_kEscapeEtecoon_Ilist_E582 as i32
        }) as uint16;
        if CheckEventHappened(0xf as uint16, game) != 0 {
            (*E).base.current_instruction = addr_kEscapeEtecoon_Ilist_E5AE as uint16;
        }
    }
    Enemy_MoveDown(k, (1 as int16 as int32) << 16 as i32, game);
}
pub unsafe fn EscapeEtecoon_Init(game: &mut Game) {
    let E: *mut Enemy_EscapeEtecoon =
        Get_EscapeEtecoon(ram_word!(game, cur_enemy_index) as i32, game);
    if CheckEventHappened(0xf as uint16, game) != 0 {
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
    } else {
        (*E).base.properties = ((*E).base.properties as i32
            | (kEnemyProps_DisableSamusColl as i32 | kEnemyProps_Tangible as i32 | 0x8000 as i32))
            as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
        (*E).base.palette_index = 0 as uint16;
        let v3: i32 = (*E).een_parameter_1 as i32 >> 1 as i32;
        (*E).base.x_pos = *(game.g_rom.offset(crate::rtl::lorom_offset(0xb3e718)) as *const uint8
            as *mut uint16)
            .offset(v3 as isize);
        (*E).base.y_pos = *(game.g_rom.offset(crate::rtl::lorom_offset(0xb3e71e)) as *const uint8
            as *mut uint16)
            .offset(v3 as isize);
        (*E).een_var_F = *(game.g_rom.offset(crate::rtl::lorom_offset(0xb3e724)) as *const uint8
            as *mut uint16)
            .offset(v3 as isize);
        (*E).base.current_instruction = *(game.g_rom.offset(crate::rtl::lorom_offset(0xb3e72a))
            as *const uint8 as *mut uint16)
            .offset(v3 as isize);
        (*E).een_var_A = *(game.g_rom.offset(crate::rtl::lorom_offset(0xb3e730)) as *const uint8
            as *mut uint16)
            .offset(v3 as isize);
    };
}
pub unsafe fn EscapeDachora_Instr_2(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    if ram_word!(game, lava_acid_y_pos) >= 0xceu16 {
        jp.offset(1)
    } else {
        *jp.offset(0) as uintptr_t as *const uint16
    }
}
pub unsafe fn EscapeDachora_Instr_3(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    if CheckEventHappened(0xf as uint16, game) != 0 {
        *jp.offset(0) as uintptr_t as *const uint16
    } else {
        jp.offset(1)
    }
}
pub unsafe fn EscapeDachora_Instr_1(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let E: *mut Enemy_EscapeDachora =
        Get_EscapeDachora(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.x_pos = ((*E).base.x_pos).wrapping_sub(6u16);
    jp
}
pub unsafe fn EscapeDachora_Instr_4(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let E: *mut Enemy_EscapeDachora =
        Get_EscapeDachora(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.x_pos = ((*E).base.x_pos).wrapping_add(6u16);
    jp
}
pub unsafe fn EscapeDachora_Init(game: &mut Game) {
    let E: *mut Enemy_EscapeDachora =
        Get_EscapeDachora(ram_word!(game, cur_enemy_index) as i32, game);
    if CheckEventHappened(0xf as uint16, game) != 0 {
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
    } else {
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_DisableSamusColl) as u16);
        (*E).base.spritemap_pointer = addr_kSpritemap_Nothing_B3 as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
        (*E).base.current_instruction = addr_kEscapeDachora_Ilist_E964 as uint16;
    };
}
