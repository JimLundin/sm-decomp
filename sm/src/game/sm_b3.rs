pub use crate::game::constants::bank_86::*;
pub use crate::game::constants::enemy_properties::*;
use crate::game::enemy_instruction;
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
use crate::ram_map::ram_ptr;
use crate::ram_map::{ram_word, set_ram_word, update_ram_word};
use crate::rtl::abs16;
use crate::rtl::AddToHiLo;
use crate::rtl::{RomPtr, Unreachable};
use crate::set_fields;

pub use crate::types::Consts_86;
pub use crate::types::Consts_B3;
pub use crate::types::{EnemyBase, EnemyData, EnemyProps, Point16U, VoidP};
crate::game::constants::typed_constants! { Consts_B3;
    addr_kEscapeDachora_Ilist_E964 = 59748, addr_kEscapeEtecoon_Ilist_E5DA = 58842,
    addr_kEscapeEtecoon_Ilist_E5AE = 58798, addr_kEscapeEtecoon_Ilist_E582 = 58754,
    addr_kEscapeEtecoon_Ilist_E556 = 58710, addr_loc_B3E28C = 57996,
    addr_stru_B3E150 = 57680, addr_kBotwoon_Ilist_9389 = 37769,
    addr_kBrinstarYellowPipeBug_Ilist_8F38 = 36664, addr_kBrinstarYellowPipeBug_Ilist_8F24 = 36644,
    addr_kBrinstarYellowPipeBug_Ilist_8F10 = 36624, addr_kBrinstarYellowPipeBug_Ilist_8EFC = 36604,
    addr_kNorfairPipeBug_Ilist_8B45 = 35653, addr_kNorfairPipeBug_Ilist_8B21 = 35617,
    addr_kNorfairPipeBug_Ilist_8B05 = 35589, addr_kNorfairPipeBug_Ilist_8AE1 = 35553,
    addr_kBrinstarPipeBug_Ilist_8A1D = 35357, addr_kBrinstarPipeBug_Ilist_87AB = 34731,
    addr_kUnusedSpinningTurtleEye_Ilist_86A7 = 34471, addr_kSpritemap_Nothing_B3 = 32845,
}
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
        pad1: [u8; 26696],
        words: [
            pbg_var_00, pbg_var_01, pbg_var_02, pbg_var_03, pbg_var_04, pbg_var_05,
            pbg_var_06, pbg_var_07, pbg_var_08, pbg_var_09, pbg_var_0A, pbg_var_0B,
            pbg_var_0C, pbg_var_0D, pbg_var_0E, pbg_var_0F, pbg_var_10, pbg_var_11,
            pbg_var_12, pbg_var_13, pbg_var_14, pbg_var_15, pbg_var_16, pbg_var_17,
            pbg_var_18, pbg_var_19, pbg_var_1A, pbg_var_1B, pbg_var_1C, pbg_var_1D,
            pbg_var_1E, pbg_var_1F,
        ],
        pad2: [u8; 1984],
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

const RomPtr_B3: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0xb3>;
crate::game::constants::typed_constants! { i32;
    fnnullsub_171_B3 = 0xb3807b, fnBrinstarPipeBug_PreInstr_1 = 0xb38880,
    fnBrinstarPipeBug_PreInstr_2 = 0xb38890, fnBrinstarPipeBug_PreInstr_3 = 0xb388e3,
    fnBrinstarPipeBug_PreInstr_4 = 0xb3891c, fnBrinstarPipeBug_PreInstr_5 = 0xb3897e,
    fnNorfairPipeBug_Func_1 = 0xb38bcd, fnNorfairPipeBug_Func_2 = 0xb38bff,
    fnNorfairPipeBug_Func_4 = 0xb38ca6, fnNorfairPipeBug_Func_5 = 0xb38cff,
    fnNorfairPipeBug_Func_6 = 0xb38d0c, fnNorfairPipeBug_Func_7 = 0xb38d4e,
    fnNorfairPipeBug_Func_8 = 0xb38d90, fnNorfairPipeBug_Func_9 = 0xb38dd2,
    fnNorfairPipeBug_Func_10 = 0xb38e14, fnNorfairPipeBug_Func_11 = 0xb38e35,
    fnNorfairPipeBug_Func_12 = 0xb38e5a, fnBrinstarYellowPipeBug_Func_1 = 0xb38fb5,
    fnBrinstarYellowPipeBug_Func_2 = 0xb38ff5, fnBrinstarYellowPipeBug_Func_3 = 0xb39028,
    fnBrinstarYellowPipeBug_Func_5 = 0xb390bd, fnBrinstarYellowPipeBug_Func_7 = 0xb3915a,
    fnBrinstarYellowPipeBug_Func_8 = 0xb391d8, fnBotwoon_Func_6 = 0xb39878,
    fnBotwoon_Func_7 = 0xb3989d, fnBotwoon_Func_12 = 0xb399a4,
    fnBotwoon_Func_13 = 0xb399e4, fnBotwoon_Func_14 = 0xb39a46,
    fnBotwoon_Func_15 = 0xb39a5e, fnBotwoon_Func_16 = 0xb39aca,
    fnBotwoon_Func_18 = 0xb39af9, fnBotwoon_Func_19 = 0xb39bb7,
    fnBotwoon_Func_26 = 0xb39dc0, fnBotwoon_Func_27 = 0xb39e7d,
    fnBotwoon_Func_28 = 0xb39ee0, fnBotwoon_Func_29 = 0xb39f34,
    fnBotwoon_Func_30 = 0xb39f7a, fnBotwoon_Func_32 = 0xb3e250,
}
crate::game::constants::typed_constants! { u32;
    fnBotwoon_Func_33 = 11788940,
}
crate::ram_map::ram_pointer_functions! {
    gEnemyData(o: i32) -> EnemyData = [0xf78, o as isize];
    Get_Botwoon(o: i32) -> Enemy_Botwoon = [0xf78, o as isize];
    Get_EscapeDachora(o: i32) -> Enemy_EscapeDachora = [0xf78, o as isize];
    Get_EscapeEtecoon(o: i32) -> Enemy_EscapeEtecoon = [0xf78, o as isize];
    Get_PipeBug(o: i32) -> Enemy_PipeBug = [0xf78, o as isize];
}
static g_word_B39E77: [i16; 3] = [2, 3, 4];
crate::forward_calls! { pub unsafe fn (game: &mut Game);
    Enemy_GrappleReact_NoInteract_B3 => SwitchEnemyAiToMainAi(game);
    Enemy_GrappleReact_KillEnemy_B3 => EnemyGrappleDeath(game);
    Enemy_GrappleReact_CancelBeam_B3 => Enemy_SwitchToFrozenAi(game);
    Enemy_NormalTouchAI_B3 => NormalEnemyTouchAi(game);
    Enemy_NormalTouchAI_SkipDeathAnim_B3 => NormalEnemyTouchAiSkipDeathAnim_CurEnemy(game);
    Enemy_NormalShotAI_B3 => NormalEnemyShotAi(game);
    Enemy_NormalShotAI_SkipSomeParts_B3 => NormalEnemyShotAiSkipDeathAnim_CurEnemy(game);
    Enemy_NormalPowerBombAI_SkipDeathAnim_B3 => NormalEnemyPowerBombAiSkipDeathAnim_CurEnemy(game);
    Enemy_NormalFrozenAI_B3 => NormalEnemyFrozenAI(game);
}
pub unsafe fn Enemy_SetAiPreInstr_B3(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*gEnemyData(k as i32, game)).ai_preinstr = *jp.offset(0);
    jp.offset(1)
}
pub unsafe fn Enemy_ClearAiPreInstr_B3(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*gEnemyData(k as i32, game)).ai_preinstr = ((0xb3807bi32) as u16);
    jp
}
pub unsafe fn UnusedSpinningTurtleEye_Init(game: &mut Game) {
    let v0 = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    (*v0).properties = ((*v0).properties) | ((kEnemyProps_DisableSamusColl) as u16);
    (*v0).current_instruction = addr_kUnusedSpinningTurtleEye_Ilist_86A7 as u16;
}
pub fn UnusedSpinningTurtleEye_Main() {}
pub unsafe fn BrinstarPipeBug_Init(game: &mut Game) {
    let PipeBug = Get_PipeBug(ram_word!(game, cur_enemy_index) as i32, game);
    (*PipeBug).pbg_var_B = (*PipeBug).base.x_pos;
    let y_pos: u16 = (*PipeBug).base.y_pos;
    (*PipeBug).pbg_var_C = y_pos;
    (*PipeBug).pbg_var_00 = (y_pos).wrapping_sub(16u16);
    (*PipeBug).pbg_var_F = ((0xb38880i32) as u16);
    (*PipeBug).pbg_var_D = 48u16;
    set_fields! {[(*PipeBug)], 0u16; pbg_var_E, pbg_var_01}
    (*PipeBug).base.current_instruction = addr_kBrinstarPipeBug_Ilist_87AB as u16;
    if (*PipeBug).pbg_parameter_1 != 0 {
        (*PipeBug).base.current_instruction = addr_kBrinstarPipeBug_Ilist_8A1D as u16;
    }
}
pub unsafe fn BrinstarPipeBug_Main(game: &mut Game) {
    let PipeBug = Get_PipeBug(ram_word!(game, cur_enemy_index) as i32, game);
    CallEnemyPreInstr((((*PipeBug).pbg_var_F) as u32) | (0xb30000u32), game);
}
pub unsafe fn BrinstarPipeBug_PreInstr_1(k: u16, game: &mut Game) {
    if CheckIfEnemyIsOnScreen(game) == 0 {
        (*Get_PipeBug(ram_word!(game, cur_enemy_index) as i32, game)).pbg_var_F =
            ((0xb38890i32) as u16);
    }
}
pub unsafe fn BrinstarPipeBug_PreInstr_2(k: u16, game: &mut Game) {
    let mut SamusEnemyDelta_Y: i16 = GetSamusEnemyDelta_Y(k, game) as i16;
    if SamusEnemyDelta_Y < 0i16 && SamusEnemyDelta_Y as i32 + 96i32 & 0x8000i32 == 0 {
        let PipeBug = Get_PipeBug(k as i32, game);
        let a: u16 = GetSamusEnemyDelta_X(k, game);
        (*PipeBug).pbg_var_A = ((*PipeBug).pbg_var_A & 0x7fff | a & 0x8000);
        if abs16(((*PipeBug).base.x_pos).wrapping_sub(ram_word!(game, samus_x_pos))) < 0x40u16 {
            (*PipeBug).pbg_var_F = ((0xb388e3i32) as u16);
            (*PipeBug).base.properties =
                ((*PipeBug).base.properties as i32 & !(kEnemyProps_Invisible as i32)) as u16;
            (*PipeBug).base.timer = 0u16;
            let mut v4: u16 = 0;
            if ((*PipeBug).pbg_var_A & 0x8000) != 0 {
                v4 = 0u16;
            } else {
                v4 = 2u16;
            }
            (*PipeBug).pbg_var_E = v4;
            BrinstarPipeBug_Func_1(game);
        }
    }
}
pub unsafe fn BrinstarPipeBug_PreInstr_3(k: u16, game: &mut Game) {
    let PipeBug = Get_PipeBug(k as i32, game);
    let fresh7 = (*PipeBug).base.y_subpos;
    (*PipeBug).base.y_subpos = (*PipeBug).base.y_subpos.wrapping_sub(1);
    let v2: bool = fresh7 != 0u16;
    (*PipeBug).base.y_pos = ((*PipeBug).base.y_pos as i32 + (v2 as i32 - 2i32)) as u16;
    if (((*PipeBug).pbg_var_00) as i16).wrapping_sub(((*PipeBug).base.y_pos) as i16) >= 0i16
        && (*PipeBug).base.y_pos < ram_word!(game, samus_y_pos)
    {
        (*PipeBug).pbg_var_E = ((*PipeBug).pbg_var_E) | (1u16);
        BrinstarPipeBug_Func_1(game);
        (*PipeBug).pbg_var_F = ((0xb3891ci32) as u16);
    }
}
pub unsafe fn BrinstarPipeBug_PreInstr_4(k: u16, game: &mut Game) {
    let PipeBug = Get_PipeBug(k as i32, game);
    if ((*PipeBug).pbg_var_A & 0x8000) == 0 {
        (*PipeBug).base.x_subpos = (*PipeBug).base.x_subpos;
        (*PipeBug).base.x_pos = ((*PipeBug).base.x_pos).wrapping_add(2u16);
    } else {
        (*PipeBug).base.x_subpos = (*PipeBug).base.x_subpos;
        (*PipeBug).base.x_pos = ((*PipeBug).base.x_pos).wrapping_sub(2u16);
    }
    if IsEnemyLeavingScreen(k, game) & 1 != 0 {
        (*PipeBug).base.x_pos = (*PipeBug).pbg_var_B;
        (*PipeBug).base.x_subpos = 0u16;
        let pbg_var_C: u16 = (*PipeBug).pbg_var_C;
        (*PipeBug).base.y_pos = pbg_var_C;
        (*PipeBug).base.y_subpos = pbg_var_C;
        (*PipeBug).pbg_var_E = 0u16;
        BrinstarPipeBug_Func_1(game);
        (*PipeBug).base.properties =
            ((*PipeBug).base.properties) | ((kEnemyProps_Invisible) as u16);
        (*PipeBug).pbg_var_D = 48u16;
        (*PipeBug).pbg_var_F = ((0xb3897ei32) as u16);
    }
}
pub unsafe fn BrinstarPipeBug_PreInstr_5(k: u16, game: &mut Game) {
    let E = Get_PipeBug(k as i32, game);
    let fresh8 = (*E).pbg_var_D;
    (*E).pbg_var_D = (*E).pbg_var_D.wrapping_sub(1);
    if fresh8 == 1u16 {
        (*E).pbg_var_F = ((0xb38890i32) as u16);
    }
}
pub unsafe fn BrinstarPipeBug_Func_1(game: &mut Game) {
    let PipeBug = Get_PipeBug(ram_word!(game, cur_enemy_index) as i32, game);
    let pbg_var_E: u16 = (*PipeBug).pbg_var_E;
    if pbg_var_E != (*PipeBug).pbg_var_01 {
        (*PipeBug).pbg_var_01 = pbg_var_E;
        enemy_instruction! {[(*PipeBug).base]; instruction = (if (*PipeBug).pbg_parameter_1 != 0u16 {
            crate::rtl::rom_table_word!(game, 0xb38833, pbg_var_E as isize) as i32
        } else {
            crate::rtl::rom_table_word!(game, 0xb3882b, pbg_var_E as isize) as i32
        }) as u16, timer = 1u16}
        (*PipeBug).base.timer = 0u16;
    }
}
pub unsafe fn NorfairPipeBug_Init(game: &mut Game) {
    let PipeBug = Get_PipeBug(ram_word!(game, cur_enemy_index) as i32, game);
    (*PipeBug).pbg_var_D = (*PipeBug).base.x_pos;
    (*PipeBug).pbg_var_E = (*PipeBug).base.y_pos;
    (*PipeBug).base.properties = ((*PipeBug).base.properties) | ((kEnemyProps_Invisible) as u16);
    (*PipeBug).pbg_var_B = (8u16)
        .wrapping_mul((*(&raw mut (*PipeBug).pbg_parameter_2 as *mut u8).offset(1isize)) as u16);
    (*PipeBug).pbg_var_C = 0u16;
    (*PipeBug).base.instruction_timer = 1u16;
    (*PipeBug).base.timer = 0u16;
    (*PipeBug).base.current_instruction = addr_kNorfairPipeBug_Ilist_8AE1 as u16;
    (*PipeBug).pbg_var_A = ((0xb38bcdi32) as u16);
}
crate::game::address_dispatch! {
    pub unsafe fn CallBugInstr(ea: u32, k: u16, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        11766733 => NorfairPipeBug_Func_1; 11766783 => NorfairPipeBug_Func_2;
        11766950 => NorfairPipeBug_Func_4; 11767039 => [NorfairPipeBug_Func_5(k, game)];
        11767052 => NorfairPipeBug_Func_6; 11767118 => NorfairPipeBug_Func_7;
        11767184 => NorfairPipeBug_Func_8; 11767250 => NorfairPipeBug_Func_9;
        11767316 => NorfairPipeBug_Func_10; 11767349 => NorfairPipeBug_Func_11;
        11767386 => NorfairPipeBug_Func_12; 11767733 => BrinstarYellowPipeBug_Func_1;
        11767797 => [{
                    BrinstarYellowPipeBug_Func_2(ram_word!(game, cur_enemy_index), game);
                }];
        11767848 => BrinstarYellowPipeBug_Func_3; 11767997 => BrinstarYellowPipeBug_Func_5;
        11768154 => BrinstarYellowPipeBug_Func_7; 11768280 => BrinstarYellowPipeBug_Func_8;
    }
}
pub unsafe fn NorfairPipeBug_Main(game: &mut Game) {
    let PipeBug = Get_PipeBug(ram_word!(game, cur_enemy_index) as i32, game);
    CallBugInstr(
        (((*PipeBug).pbg_var_A) as u32) | (0xb30000u32),
        ram_word!(game, cur_enemy_index),
        game,
    );
    NorfairPipeBug_8BA8(game);
}
pub unsafe fn NorfairPipeBug_8BA8(game: &mut Game) {
    if CheckIfEnemyIsOnScreen(game) != 0 {
        let PipeBug = Get_PipeBug(ram_word!(game, cur_enemy_index) as i32, game);
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
    .pbg_parameter_2 as *mut u8)
        .offset(0)
        != 0u8
        && (*Get_PipeBug(ram_word!(game, cur_enemy_index) as i32 + 64i32, game)).pbg_var_A as i32
            == 0xb38bcdi32 & 0xffffi32
        && (*Get_PipeBug(ram_word!(game, cur_enemy_index) as i32 + 128i32, game)).pbg_var_A as i32
            == 0xb38bcdi32 & 0xffffi32
        && (*Get_PipeBug(ram_word!(game, cur_enemy_index) as i32 + 192i32, game)).pbg_var_A as i32
            == 0xb38bcdi32 & 0xffffi32
        && (*Get_PipeBug(ram_word!(game, cur_enemy_index) as i32 + 256i32, game)).pbg_var_A as i32
            == 0xb38bcdi32 & 0xffffi32
    {
        (*Get_PipeBug(ram_word!(game, cur_enemy_index) as i32, game)).pbg_var_A =
            ((0xb38bffi32) as u16);
    }
}
pub unsafe fn NorfairPipeBug_Func_2(game: &mut Game) {
    let k: i32 = ram_word!(game, cur_enemy_index) as i32;
    let PipeBug = Get_PipeBug(ram_word!(game, cur_enemy_index) as i32, game);
    if IsSamusWithinEnemy_X(
        ram_word!(game, cur_enemy_index),
        *(&raw mut (*PipeBug).pbg_parameter_2 as *mut u8).offset(0) as u16,
        game,
    ) != 0u16
        && (((*PipeBug).base.y_pos) as i16).wrapping_sub((ram_word!(game, samus_y_pos)) as i16)
            >= 0i16
    {
        (*PipeBug).pbg_var_C = (*PipeBug).pbg_var_C.wrapping_add(1);
        (*PipeBug).base.instruction_timer = 1u16;
        (*PipeBug).base.timer = 0u16;
        if (GetSamusEnemyDelta_X(k as u16, game) & 0x8000) == 0 {
            (*Get_PipeBug(k, game)).base.current_instruction =
                addr_kNorfairPipeBug_Ilist_8B21 as u16;
            (*Get_PipeBug(k + 64i32, game)).base.current_instruction =
                addr_kNorfairPipeBug_Ilist_8B21 as u16;
            (*Get_PipeBug(k + 128i32, game)).base.current_instruction =
                addr_kNorfairPipeBug_Ilist_8B21 as u16;
            (*Get_PipeBug(k + 192i32, game)).base.current_instruction =
                addr_kNorfairPipeBug_Ilist_8B21 as u16;
            (*Get_PipeBug(k + 256i32, game)).base.current_instruction =
                addr_kNorfairPipeBug_Ilist_8B21 as u16;
        } else {
            (*PipeBug).base.current_instruction = addr_kNorfairPipeBug_Ilist_8AE1 as u16;
            (*Get_PipeBug(k + 64i32, game)).base.current_instruction =
                addr_kNorfairPipeBug_Ilist_8AE1 as u16;
            (*Get_PipeBug(k + 128i32, game)).base.current_instruction =
                addr_kNorfairPipeBug_Ilist_8AE1 as u16;
            (*Get_PipeBug(k + 192i32, game)).base.current_instruction =
                addr_kNorfairPipeBug_Ilist_8AE1 as u16;
            (*Get_PipeBug(k + 256i32, game)).base.current_instruction =
                addr_kNorfairPipeBug_Ilist_8AE1 as u16;
        }
        NorfairPipeBug_Func_3(k as u16, game);
    }
}
pub unsafe fn NorfairPipeBug_Func_3(k: u16, game: &mut Game) {
    (*Get_PipeBug(k as i32, game)).pbg_var_A = ((0xb38ca6i32) as u16);
    (*Get_PipeBug(k as i32 + 64i32, game)).pbg_var_A = ((0xb38ca6i32) as u16);
    (*Get_PipeBug(k as i32 + 128i32, game)).pbg_var_A = ((0xb38ca6i32) as u16);
    (*Get_PipeBug(k as i32 + 192i32, game)).pbg_var_A = ((0xb38ca6i32) as u16);
    (*Get_PipeBug(k as i32 + 256i32, game)).pbg_var_A = ((0xb38ca6i32) as u16);
    (*Get_PipeBug(k as i32, game)).pbg_var_02 = 104u16;
    (*Get_PipeBug(k as i32 + 64i32, game)).pbg_var_02 = 96u16;
    (*Get_PipeBug(k as i32 + 128i32, game)).pbg_var_02 = 88u16;
    (*Get_PipeBug(k as i32 + 192i32, game)).pbg_var_02 = 112u16;
    (*Get_PipeBug(k as i32 + 256i32, game)).pbg_var_02 = 120u16;
    (*Get_PipeBug(k as i32, game)).pbg_var_F = ((0xb38cffi32) as u16);
    (*Get_PipeBug(k as i32 + 64i32, game)).pbg_var_F = ((0xb38d0ci32) as u16);
    (*Get_PipeBug(k as i32 + 128i32, game)).pbg_var_F = ((0xb38d4ei32) as u16);
    (*Get_PipeBug(k as i32 + 192i32, game)).pbg_var_F = ((0xb38d90i32) as u16);
    (*Get_PipeBug(k as i32 + 256i32, game)).pbg_var_F = ((0xb38dd2i32) as u16);
}
pub unsafe fn NorfairPipeBug_Func_4(game: &mut Game) {
    let E = Get_PipeBug(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.properties = ((*E).base.properties as i32 & !(kEnemyProps_Invisible as i32)) as u16;
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        (crate::rtl::rom_table_word!(game, 0xa28187, 66) as u32) << 16i32
            | crate::rtl::rom_table_word!(game, 0xa28187, 67) as u32,
    );
    if (((*E).base.y_pos) as i16).wrapping_sub((ram_word!(game, samus_y_pos)) as i16) < 0i16 {
        (*E).pbg_var_A = (*E).pbg_var_F;
        (*E).pbg_var_01 = (*E).base.y_pos;
        (*E).base.instruction_timer = 1u16;
        (*E).base.timer = 0u16;
        if (GetSamusEnemyDelta_X(ram_word!(game, cur_enemy_index), game) & 0x8000) == 0 {
            (*E).base.current_instruction = addr_kNorfairPipeBug_Ilist_8B21 as u16;
        } else {
            (*E).base.current_instruction = addr_kNorfairPipeBug_Ilist_8AE1 as u16;
        }
    }
}
pub unsafe fn NorfairPipeBug_Func_5(k: u16, game: &mut Game) {
    let E = Get_PipeBug(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).pbg_var_C = (*E).pbg_var_C.wrapping_add(1);
    (*E).pbg_var_A = ((0xb38e5ai32) as u16);
}
pub unsafe fn NorfairPipeBug_Func_6(game: &mut Game) {
    let E = Get_PipeBug(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).pbg_var_C = (*E).pbg_var_C.wrapping_add(1);
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        (crate::rtl::rom_table_word!(game, 0xa28187, 66) as u32) << 16i32
            | crate::rtl::rom_table_word!(game, 0xa28187, 67) as u32,
    );
    if (((*E).base.y_pos as i32 + 16i32 - (*E).pbg_var_01 as i32) as i16) < 0i16 {
        (*E).base.y_pos = ((*E).pbg_var_01).wrapping_sub(16u16);
        (*E).base.y_subpos = 0u16;
        (*E).pbg_var_A = ((0xb38e5ai32) as u16);
    }
}
pub unsafe fn NorfairPipeBug_Func_7(game: &mut Game) {
    let E = Get_PipeBug(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).pbg_var_C = (*E).pbg_var_C.wrapping_add(1);
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        (crate::rtl::rom_table_word!(game, 0xa28187, 66) as u32) << 16i32
            | crate::rtl::rom_table_word!(game, 0xa28187, 67) as u32,
    );
    if (((*E).base.y_pos as i32 + 32i32 - (*E).pbg_var_01 as i32) as i16) < 0i16 {
        (*E).base.y_pos = ((*E).pbg_var_01).wrapping_sub(32u16);
        (*E).base.y_subpos = 0u16;
        (*E).pbg_var_A = ((0xb38e5ai32) as u16);
    }
}
pub unsafe fn NorfairPipeBug_Func_8(game: &mut Game) {
    let E = Get_PipeBug(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).pbg_var_C = (*E).pbg_var_C.wrapping_add(1);
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        (crate::rtl::rom_table_word!(game, 0xa28187, 64) as u32) << 16i32
            | crate::rtl::rom_table_word!(game, 0xa28187, 65) as u32,
    );
    if (((*E).base.y_pos as i32 - 16i32 - (*E).pbg_var_01 as i32) as i16) >= 0i16 {
        (*E).base.y_pos = ((*E).pbg_var_01).wrapping_add(16u16);
        (*E).base.y_subpos = 0u16;
        (*E).pbg_var_A = ((0xb38e5ai32) as u16);
    }
}
pub unsafe fn NorfairPipeBug_Func_9(game: &mut Game) {
    let E = Get_PipeBug(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).pbg_var_C = (*E).pbg_var_C.wrapping_add(1);
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        (crate::rtl::rom_table_word!(game, 0xa28187, 64) as u32) << 16i32
            | crate::rtl::rom_table_word!(game, 0xa28187, 65) as u32,
    );
    if (((*E).base.y_pos as i32 - 32i32 - (*E).pbg_var_01 as i32) as i16) >= 0i16 {
        (*E).base.y_pos = ((*E).pbg_var_01).wrapping_add(32u16);
        (*E).base.y_subpos = 0u16;
        (*E).pbg_var_A = ((0xb38e5ai32) as u16);
    }
}
pub unsafe fn NorfairPipeBug_Func_10(game: &mut Game) {
    let E = Get_PipeBug(ram_word!(game, cur_enemy_index) as i32, game);
    let v2: i32 = (*E).pbg_var_B as i32 >> 1i32;
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        (crate::rtl::rom_table_word!(game, 0xa28187, (v2 + 2i32) as isize) as u32) << 16i32
            | crate::rtl::rom_table_word!(game, 0xa28187, (v2 + 3i32) as isize) as u32,
    );
}
pub unsafe fn NorfairPipeBug_Func_11(game: &mut Game) {
    let E = Get_PipeBug(ram_word!(game, cur_enemy_index) as i32, game);
    let v2: i32 = (*E).pbg_var_B as i32 >> 1i32;
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        (crate::rtl::rom_table_word!(game, 0xa28187, v2 as isize) as u32) << 16i32
            | crate::rtl::rom_table_word!(game, 0xa28187, (v2 + 1i32) as isize) as u32,
    );
}
pub fn sub_B38E56() {}
pub unsafe fn NorfairPipeBug_Func_12(game: &mut Game) {
    let E = Get_PipeBug(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).pbg_var_C = (*E).pbg_var_C.wrapping_add(1);
    if (((*E).pbg_var_C) as i16).wrapping_sub(((*E).pbg_var_02) as i16) >= 0i16 {
        (*E).pbg_var_C = 0u16;
        (*E).base.instruction_timer = 1u16;
        (*E).base.timer = 0u16;
        (*E).pbg_var_A = ((0xb38e14i32) as u16);
        (*E).base.current_instruction = addr_kNorfairPipeBug_Ilist_8B05 as u16;
        if (GetSamusEnemyDelta_X(ram_word!(game, cur_enemy_index), game) & 0x8000) == 0 {
            (*E).pbg_var_A = ((0xb38e35i32) as u16);
            (*E).base.current_instruction = addr_kNorfairPipeBug_Ilist_8B45 as u16;
        }
    }
}
pub unsafe fn BrinstarYellowPipeBug_Init(game: &mut Game) {
    let E = Get_PipeBug(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).pbg_var_07 = (*E).base.x_pos;
    (*E).pbg_var_08 = (*E).base.y_pos;
    set_fields! {[(*E).base], 0u16; x_subpos, y_subpos}
    (*E).base.instruction_timer = 1u16;
    (*E).base.timer = 0u16;
    (*E).base.current_instruction = addr_kBrinstarYellowPipeBug_Ilist_8EFC as u16;
    if (*E).pbg_parameter_1 == 0 {
        (*E).base.current_instruction = addr_kBrinstarYellowPipeBug_Ilist_8F24 as u16;
    }
    let v1: i32 = 8i32 * (*E).pbg_parameter_2 as i32 >> 1i32;
    (*E).pbg_var_01 = crate::rtl::rom_table_word!(game, 0xa28187, v1 as isize);
    (*E).pbg_var_00 = crate::rtl::rom_table_word!(game, 0xa28187, (v1 + 1i32) as isize);
    (*E).pbg_var_03 = crate::rtl::rom_table_word!(game, 0xa28187, (v1 + 2i32) as isize);
    (*E).pbg_var_02 = crate::rtl::rom_table_word!(game, 0xa28187, (v1 + 3i32) as isize);
    (*E).pbg_var_A = ((0xb38fb5i32) as u16);
    (*E).pbg_var_06 = 0u16;
}
pub unsafe fn BrinstarYellowPipeBug_Main(game: &mut Game) {
    let E = Get_PipeBug(ram_word!(game, cur_enemy_index) as i32, game);
    CallBugInstr(
        (((*E).pbg_var_A) as u32) | (0xb30000u32),
        ram_word!(game, cur_enemy_index),
        game,
    );
}
pub unsafe fn BrinstarYellowPipeBug_Func_1(game: &mut Game) {
    let E = Get_PipeBug(ram_word!(game, cur_enemy_index) as i32, game);

    let samus_in_x_range: bool = if (*E).pbg_parameter_1 != 0 {
        let mut SamusEnemyDelta_X: i16 =
            GetSamusEnemyDelta_X(ram_word!(game, cur_enemy_index), game) as i16;
        (SamusEnemyDelta_X < 0i16 && SamusEnemyDelta_X as i32 + 192i32 & 0x8000i32 == 0)
    } else {
        let mut v2: i16 = GetSamusEnemyDelta_X(ram_word!(game, cur_enemy_index), game) as i16;
        (v2 >= 0i16 && v2 as i32 - 192i32 & 0x8000i32 != 0)
    };
    if samus_in_x_range
        && IsSamusWithinEnemy_Y(ram_word!(game, cur_enemy_index), 0x30u16, game) != 0
    {
        (*E).base.properties =
            ((*E).base.properties as i32 & !(kEnemyProps_Invisible as i32)) as u16;
        (*E).pbg_var_20 = 24u16;
        (*E).pbg_var_A = ((0xb38ff5i32) as u16);
    };
}
pub unsafe fn BrinstarYellowPipeBug_Func_2(k: u16, game: &mut Game) {
    let E = Get_PipeBug(k as i32, game);
    let v2: u16 = ((*E).pbg_var_20).wrapping_sub(1u16);
    (*E).pbg_var_20 = v2;
    if v2 == 0 {
        (*E).base.instruction_timer = 1u16;
        (*E).base.timer = 0u16;
        (*E).base.current_instruction = addr_kBrinstarYellowPipeBug_Ilist_8EFC as u16;
        (*E).pbg_var_A = ((0xb39028i32) as u16);
        if (*E).pbg_parameter_1 == 0 {
            (*E).base.current_instruction = addr_kBrinstarYellowPipeBug_Ilist_8F24 as u16;
            (*E).pbg_var_A = ((0xb390bdi32) as u16);
        }
    }
}
pub unsafe fn BrinstarYellowPipeBug_Func_3(game: &mut Game) {
    let E = Get_PipeBug(ram_word!(game, cur_enemy_index) as i32, game);
    BrinstarYellowPipeBug_Func_4(ram_word!(game, cur_enemy_index), game);
    if CheckIfEnemyIsOnScreen(game) != 0 {
        (*E).base.x_pos = (*E).pbg_var_07;
        (*E).base.y_pos = (*E).pbg_var_08;
        set_fields! {[(*E).base], 0u16; x_subpos, y_subpos}
        (*E).pbg_var_A = ((0xb38fb5i32) as u16);
        (*E).pbg_var_06 = 0u16;
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
    } else if (*E).pbg_var_06 == 0
        && (*E).base.x_pos as i32 - ram_word!(game, samus_x_pos) as i32 - 48i32 & 0x8000i32 != 0
    {
        (*E).pbg_var_A = ((0xb3915ai32) as u16);
        set_fields! {[(*E)], 0u16; pbg_var_C, pbg_var_D}
        (*E).pbg_var_E = 40u16;
        (*E).pbg_var_F = 1u16;
        (*E).pbg_var_04 = (*E).base.x_pos;
        (*E).base.instruction_timer = 1u16;
        (*E).base.timer = 0u16;
        (*E).base.current_instruction = addr_kBrinstarYellowPipeBug_Ilist_8F10 as u16;
    }
}
pub unsafe fn BrinstarYellowPipeBug_Func_4(k: u16, game: &mut Game) {
    let E = Get_PipeBug(k as i32, game);
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        ((*E).pbg_var_03 as u32) << 16i32 | (*E).pbg_var_02 as u32,
    );
}
pub unsafe fn BrinstarYellowPipeBug_Func_5(game: &mut Game) {
    let E = Get_PipeBug(ram_word!(game, cur_enemy_index) as i32, game);
    BrinstarYellowPipeBug_Func_6(ram_word!(game, cur_enemy_index), game);
    if CheckIfEnemyIsOnScreen(game) != 0 {
        (*E).base.x_pos = (*E).pbg_var_07;
        (*E).base.y_pos = (*E).pbg_var_08;
        set_fields! {[(*E).base], 0u16; x_subpos, y_subpos}
        (*E).pbg_var_A = ((0xb38fb5i32) as u16);
        (*E).pbg_var_06 = 0u16;
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
    } else if (*E).pbg_var_06 == 0
        && ram_word!(game, samus_x_pos) as i32 - (*E).base.x_pos as i32 - 48i32 & 0x8000i32 != 0
    {
        (*E).pbg_var_A = ((0xb391d8i32) as u16);
        set_fields! {[(*E)], 0u16; pbg_var_C, pbg_var_D}
        (*E).pbg_var_E = 40u16;
        (*E).pbg_var_F = 1u16;
        (*E).pbg_var_04 = (*E).base.x_pos;
        (*E).base.instruction_timer = 1u16;
        (*E).base.timer = 0u16;
        (*E).base.current_instruction = addr_kBrinstarYellowPipeBug_Ilist_8F38 as u16;
    }
}
pub unsafe fn BrinstarYellowPipeBug_Func_6(k: u16, game: &mut Game) {
    let E = Get_PipeBug(k as i32, game);
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        ((*E).pbg_var_01 as u32) << 16i32 | (*E).pbg_var_00 as u32,
    );
}
pub unsafe fn BrinstarYellowPipeBug_Func_7(game: &mut Game) {
    let E = Get_PipeBug(ram_word!(game, cur_enemy_index) as i32, game);
    if CheckIfEnemyIsOnScreen(game) != 0 {
        (*E).base.x_pos = (*E).pbg_var_07;
        (*E).base.y_pos = (*E).pbg_var_08;
        set_fields! {[(*E).base], 0u16; x_subpos, y_subpos}
        (*E).pbg_var_A = ((0xb38fb5i32) as u16);
        (*E).pbg_var_06 = 0u16;
        (*E).base.instruction_timer = 1u16;
        (*E).base.timer = 0u16;
        (*E).base.current_instruction = addr_kBrinstarYellowPipeBug_Ilist_8EFC as u16;
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
    } else {
        BrinstarYellowPipeBug_Func_4(ram_word!(game, cur_enemy_index), game);
        if (*E).pbg_var_F != 0 {
            BrinstarYellowPipeBug_Func_10(ram_word!(game, cur_enemy_index), game);
        } else {
            BrinstarYellowPipeBug_Func_9(ram_word!(game, cur_enemy_index), game);
            if (((*E).base.y_pos) as i16).wrapping_sub(((*E).pbg_var_08) as i16) < 0i16 {
                set_fields! {[(*E)], 1u16; pbg_var_06, pbg_var_F}
                (*E).pbg_var_A = ((0xb39028i32) as u16);
                (*E).base.instruction_timer = 1u16;
                (*E).base.timer = 0u16;
                (*E).base.current_instruction = addr_kBrinstarYellowPipeBug_Ilist_8EFC as u16;
            }
        }
    };
}
pub unsafe fn BrinstarYellowPipeBug_Func_8(game: &mut Game) {
    let E = Get_PipeBug(ram_word!(game, cur_enemy_index) as i32, game);
    if CheckIfEnemyIsOnScreen(game) != 0 {
        (*E).base.x_pos = (*E).pbg_var_07;
        (*E).base.y_pos = (*E).pbg_var_08;
        set_fields! {[(*E).base], 0u16; x_subpos, y_subpos}
        (*E).pbg_var_A = ((0xb38fb5i32) as u16);
        (*E).pbg_var_06 = 0u16;
        (*E).base.instruction_timer = 1u16;
        (*E).base.timer = 0u16;
        (*E).base.current_instruction = addr_kBrinstarYellowPipeBug_Ilist_8F24 as u16;
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
    } else {
        BrinstarYellowPipeBug_Func_6(ram_word!(game, cur_enemy_index), game);
        if (*E).pbg_var_F != 0 {
            BrinstarYellowPipeBug_Func_10(ram_word!(game, cur_enemy_index), game);
        } else {
            BrinstarYellowPipeBug_Func_9(ram_word!(game, cur_enemy_index), game);
            if (((*E).pbg_var_08) as i16).wrapping_sub(((*E).base.y_pos) as i16) >= 0i16 {
                set_fields! {[(*E)], 1u16; pbg_var_06, pbg_var_F}
                (*E).base.instruction_timer = 1u16;
                (*E).base.timer = 0u16;
                (*E).base.current_instruction = addr_kBrinstarYellowPipeBug_Ilist_8F24 as u16;
                (*E).pbg_var_A = ((0xb390bdi32) as u16);
            }
        }
    };
}
pub unsafe fn BrinstarYellowPipeBug_Func_9(k: u16, game: &mut Game) {
    let E = Get_PipeBug(k as i32, game);
    (*E).pbg_var_E = (*E).pbg_var_E.wrapping_add(1);
    let v3: i32 = 8i32 * (*E).pbg_var_E as i32 >> 1i32;
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        (crate::rtl::rom_table_word!(game, 0xa2838f, (v3 + 3i32) as isize) as u32) << 16i32
            | crate::rtl::rom_table_word!(game, 0xa2838f, (v3 + 2i32) as isize) as u32,
    );
}
pub unsafe fn BrinstarYellowPipeBug_Func_10(k: u16, game: &mut Game) {
    let E = Get_PipeBug(k as i32, game);
    (*E).pbg_var_E = (*E).pbg_var_E.wrapping_sub(1);
    if ((*E).pbg_var_E & 0x8000) != 0 {
        set_fields! {[(*E)], 0u16; pbg_var_E, pbg_var_F}
    } else {
        let v4: i32 = 8i32 * (*E).pbg_var_E as i32 >> 1i32;
        AddToHiLo(
            &raw mut (*E).base.y_pos,
            &raw mut (*E).base.y_subpos,
            (crate::rtl::rom_table_word!(game, 0xa2838f, (v4 + 1i32) as isize) as u32) << 16i32
                | crate::rtl::rom_table_word!(game, 0xa2838f, v4 as isize) as u32,
        );
    };
}
pub unsafe fn Botwoon_Instr_1(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.x_width = 8u16;
    (*E).base.y_height = 16u16;
    jp
}
pub unsafe fn Botwoon_Instr_2(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    set_fields! {[(*E).base], 12u16; x_width, y_height}
    jp
}
pub unsafe fn Botwoon_Instr_3(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.x_width = 16u16;
    (*E).base.y_height = 8u16;
    jp
}
pub unsafe fn Botwoon_Instr_4(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    set_fields! {[(*E).base], 12u16; x_width, y_height}
    jp
}
pub unsafe fn Botwoon_Instr_5(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.x_width = 8u16;
    (*E).base.y_height = 16u16;
    jp
}
pub unsafe fn Botwoon_Instr_6(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.x_width = 8u16;
    (*E).base.y_height = 16u16;
    jp
}
pub unsafe fn Botwoon_Instr_7(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    set_fields! {[(*E).base], 12u16; x_width, y_height}
    jp
}
pub unsafe fn Botwoon_Instr_8(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.x_width = 16u16;
    (*E).base.y_height = 8u16;
    jp
}
pub unsafe fn Botwoon_Instr_9(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    set_fields! {[(*E).base], 12u16; x_width, y_height}
    jp
}
pub unsafe fn Botwoon_Instr_10(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.x_width = 8u16;
    (*E).base.y_height = 16u16;
    jp
}
pub unsafe fn Botwoon_Instr_SetSpitting(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game)).botwoon_var_5A = 1u16;
    jp
}
pub unsafe fn Botwoon_Instr_QueueSpitSfx(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    QueueSfx2_Max6(0x7cu16, game);
    jp
}
pub unsafe fn Botwoon_QueueExplosionSfx(game: &mut Game) {
    QueueSfx2_Max6(0x24u16, game);
}
pub unsafe fn Botwoon_Init(game: &mut Game) {
    let E = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    if (*ram_ptr!(game; 0xd828, 4) & 2) != 0 {
        SpawnHardcodedPlm(
            SpawnHardcodedPlmArgs::new(0xfu8, 0x4u8, 0xb797 as VoidP),
            game,
        );
        *(ram_ptr!(game, u16; 0xcd20)) = 257u16;
        (*E).base.current_instruction = addr_kBotwoon_Ilist_9389 as u16;
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
    } else {
        let v3: u16 = 24u16;
        (*E).botwoon_var_A = 24u16;
        loop {
            SpawnEprojWithGfx(
                v3,
                ram_word!(game, cur_enemy_index),
                addr_kEproj_BotwoonsBody as u16,
                game,
            );
            (*E).botwoon_var_A = (*E).botwoon_var_A.wrapping_sub(1);
            let mut v5: bool = (((*E).botwoon_var_A) as i16).wrapping_sub(1i16) < 0i16;
            (*E).botwoon_var_A = (*E).botwoon_var_A.wrapping_sub(1);
            if v5 {
                break;
            }
        }
        (*E).base.current_instruction = addr_kBotwoon_Ilist_9389 as u16;
        (*E).botwoon_var_3B = addr_kBotwoon_Ilist_9389 as u16;
        (*E).botwoon_var_D = ((0xb39878i32) as u16);
        (*E).botwoon_var_E = ((0xb39bb7i32) as u16);
        (*E).botwoon_var_F = ((0xb39dc0i32) as u16);
        (*E).botwoon_var_20 = 256u16;
        (*E).botwoon_var_38 = crate::rtl::rom_table_word!(game, 0xb394bb, 0);
        (*E).botwoon_var_C = crate::rtl::rom_table_word!(game, 0xb394bb, 1);
        set_fields! {[(*E)], 1u16; botwoon_var_33, botwoon_var_34, botwoon_var_59}
        (*E).botwoon_var_36 = -1i32 as u16;
        (*E).botwoon_var_37 = 0u16;
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
        let mut health: u16 = (*E).base.health;
        (*E).botwoon_var_5B = health;
        health = (health as i32 >> 1i32) as u16;
        (*E).botwoon_var_5C = health;
        (*E).botwoon_var_5D = (health as i32 >> 1i32) as u16;
        let x_pos: u16 = (*E).base.x_pos;
        (*E).botwoon_var_50 = x_pos;
        (*E).botwoon_var_52 = x_pos;
        (*E).botwoon_var_54 = x_pos;
        (*E).botwoon_var_56 = x_pos;
        let y_pos: u16 = (*E).base.y_pos;
        (*E).botwoon_var_51 = y_pos;
        (*E).botwoon_var_53 = y_pos;
        (*E).botwoon_var_55 = y_pos;
        (*E).botwoon_var_57 = y_pos;
        (*E).botwoon_var_4E = (((*E).base.palette_index as i32 >> 4i32) + 256i32) as u16;
    };
}
crate::game::address_dispatch! {
    pub unsafe fn CallBotwoonFuncVarD(ea: u32, k: u16, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        11769976 => Botwoon_Func_6; 11770013 => Botwoon_Func_7; 11770276 => Botwoon_Func_12;
        11770340 => Botwoon_Func_13; 11770438 => Botwoon_Func_14; 11770462 => Botwoon_Func_15;
        11770570 => Botwoon_Func_16; 11770617 => [Botwoon_Func_18(k, game)];
    }
    pub unsafe fn CallBotwoonFuncVarE(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        11770807 => Botwoon_Func_19; 11788880 => Botwoon_Func_32; 11788940 => Botwoon_Func_33;
    }
}
pub unsafe fn Botwoon_Main(game: &mut Game) {
    Botwoon_Func_3(game);
    let E = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    CallBotwoonFuncVarD(
        (((*E).botwoon_var_D) as u32) | (0xb30000u32),
        ram_word!(game, cur_enemy_index),
        game,
    );
    Botwoon_HealthBasedPalHandling(game);
}
pub unsafe fn Botwoon_Func_1(k: u16, game: &mut Game) {
    let E = Get_Botwoon(k as i32, game);
    if (crate::rtl::rom_table_word!(game, 0xb39675, (*E).botwoon_var_3F as isize)
        & NextRandom(game))
        == 0
    {
        (*E).botwoon_var_2E = 1u16;
    }
}
pub unsafe fn Botwoon_Func_2(game: &mut Game) {
    let E = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).botwoon_var_2F == 0
        && (*E).botwoon_var_2E != 0u16
        && (*E).botwoon_var_33 == 0
        && (*E).botwoon_var_21 == 0
    {
        (*E).botwoon_var_F = ((0xb39e7di32) as u16);
        (*E).botwoon_var_21 = 24u16;
    }
    (*E).botwoon_var_2E = 0u16;
}
pub unsafe fn Botwoon_Func_3(game: &mut Game) {
    let E = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).botwoon_var_2F != 0 && (*E).botwoon_var_3E != 0 {
        (*E).botwoon_var_30 = 1u16;
        (*E).botwoon_var_D = ((0xb39a46i32) as u16);
        (*E).botwoon_var_22 = 240u16;
        set_fields! {[(*E)], 0u16; botwoon_var_2F, botwoon_var_3E}
    }
}
pub unsafe fn Botwoon_Func_4(game: &mut Game) {
    let E = Get_Botwoon(0i32, game);
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
}
pub unsafe fn Botwoon_Func_5(game: &mut Game) {
    let mut v0: u16 = 34u16;
    loop {
        update_ram_word!(
            game.eproj_properties[(v0 as i32 >> 1i32)],
            |fresh4| (fresh4) | (0x2000u16)
        );
        set_ram_word!(game.eproj_flags[(v0 as i32 >> 1i32)] = 2u16);
        v0 = (v0).wrapping_sub(2u16);
        if (((v0) as i16).wrapping_sub(10i16) < 0i16) {
            break;
        }
    }
}
pub unsafe fn Botwoon_HealthBasedPalHandling(game: &mut Game) {
    let E = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).botwoon_var_4F != 16u16
        && (((*E).base.health) as i16).wrapping_sub(
            (crate::rtl::rom_table_word!(
                game,
                0xb3981b,
                ((*E).botwoon_var_4F as i32 >> 1i32) as isize
            )) as i16,
        ) < 0i16
    {
        let mut r18: u16 = (16u16).wrapping_mul((*E).botwoon_var_4F);
        let mut r20: u16 = (*E).botwoon_var_4E;
        loop {
            let fresh5 = r18;
            r18 = r18.wrapping_add(1);
            set_ram_word!(
                game.palette_buffer[(r20 as i32 >> 1i32)] =
                    crate::rtl::rom_table_word!(game, 0xb3971b, (fresh5 as i32 >> 1i32) as isize)
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
    let E = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    let v1: u16 = ((*E).botwoon_var_20).wrapping_sub(1u16);
    (*E).botwoon_var_20 = v1;
    if v1 == 0 {
        (*E).botwoon_var_D = ((0xb3989di32) as u16);
    }
}
pub unsafe fn Botwoon_Func_7(game: &mut Game) {
    let E = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).botwoon_var_3C != 0 {
        (*E).botwoon_var_3C = 0u16;
        let mut r18: u8 = 0u8;
        if (*E).botwoon_var_33 == 0 && (*E).botwoon_var_59 == 0 && (*E).botwoon_var_3F == 0 {
            r18 = ((NextRandom(game)) as u8) & (0xeu8);
        }
        (*E).botwoon_var_59 = 0u16;
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
    let E = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).botwoon_var_D = ((0xb399a4i32) as u16);
    (*E).botwoon_var_E = ((0xb3e250i32) as u16);
    (*E).botwoon_var_21 = 0u16;
    (*E).botwoon_var_F = ((0xb39dc0i32) as u16);
    (*E).botwoon_var_4B = 0u16;
    Botwoon_Func_10(ram_word!(game, cur_enemy_index), game);
}
pub unsafe fn Botwoon_Func_9(game: &mut Game) {
    let E = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).botwoon_var_D = ((0xb399e4i32) as u16);
    (*E).botwoon_var_F = ((0xb39e7di32) as u16);
    (*E).botwoon_var_21 = 48u16;
    (*E).base.properties = ((*E).base.properties as i32 & !(kEnemyProps_Tangible as i32)) as u16;
}
pub unsafe fn Botwoon_Func_10(k: u16, game: &mut Game) {
    Botwoon_Func_11(k, game);
    let E = Get_Botwoon(k as i32, game);
    let r18: u16 = (if (*E).botwoon_var_33 != 0u16 {
        128i32
    } else {
        0i32
    }) as u16;
    (*E).botwoon_var_40 = ((NextRandom(game) as i32 & 0x18i32)
        + r18 as i32
        + 4i32 * (*E).botwoon_var_37 as i32) as u16;
}
pub unsafe fn Botwoon_Func_11(k: u16, game: &mut Game) {
    let E = Get_Botwoon(k as i32, game);
    if (*E).botwoon_var_33 == 0 {
        (*E).botwoon_var_3F = 0u16;
        let mut health: i16 = (*E).base.health as i16;
        if health > 0i16 && (health).wrapping_sub(((*E).botwoon_var_5C) as i16) < 0i16 {
            if (health).wrapping_sub(((*E).botwoon_var_5D) as i16) < 0i16 {
                (*E).botwoon_var_3F = 2u16;
            } else {
                (*E).botwoon_var_3F = 1u16;
            }
            let v3: i32 = ((4u16).wrapping_mul((*E).botwoon_var_3F)) as i32 >> 1i32;
            (*E).botwoon_var_38 = crate::rtl::rom_table_word!(game, 0xb394bb, v3 as isize);
            (*E).botwoon_var_C = crate::rtl::rom_table_word!(game, 0xb394bb, (v3 + 1i32) as isize);
        }
    }
}
pub unsafe fn Botwoon_Func_12(game: &mut Game) {
    let E = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).botwoon_var_3C != 0 {
        (*E).botwoon_var_3C = 0u16;
        (*E).botwoon_var_D = ((0xb3989di32) as u16);
        (*E).botwoon_var_E = ((0xb39bb7i32) as u16);
        if (*E).botwoon_var_33 != 0 {
            (*E).botwoon_var_35 = 0u16;
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
    let E = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).botwoon_var_21 != 0 {
        (*E).botwoon_var_2E = 0u16;
        CallEnemyPreInstr((((*E).botwoon_var_F) as u32) | (0xb30000u32), game);
    } else {
        (*E).botwoon_var_3C = 0u16;
        (*E).botwoon_var_D = ((0xb399a4i32) as u16);
        (*E).botwoon_var_E = ((0xb3e250i32) as u16);
        (*E).botwoon_var_F = ((0xb39dc0i32) as u16);
        if (*E).botwoon_var_2F != 0u16 || {
            let mut v3: u16 = (NextRandom(game)) & (1u16);
            (*E).botwoon_var_33 = v3;
            (*E).botwoon_var_34 = v3;
            (*E).botwoon_var_34 == 0u16
        } {
            (*E).botwoon_var_4B = 0u16;
        } else {
            (*E).botwoon_var_35 = 0u16;
            (*E).botwoon_var_36 = -1i32 as u16;
        }
        Botwoon_Func_10(ram_word!(game, cur_enemy_index), game);
    };
}
pub unsafe fn Botwoon_Func_14(game: &mut Game) {
    let E = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    let v1: u16 = ((*E).botwoon_var_22).wrapping_add(1u16);
    (*E).botwoon_var_22 = v1;
    if v1 as i32 - 256i32 & 0x8000i32 == 0 {
        (*E).botwoon_var_D = ((0xb39a5ei32) as u16);
    }
}
pub unsafe fn Botwoon_Func_15(game: &mut Game) {
    let E = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    let v2: i32 = 8i32 * *(&raw mut (*E).botwoon_var_28 as *mut u8).offset(1isize) as i32 >> 1i32;
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        (crate::rtl::rom_table_word!(game, 0xa2838f, (v2 + 1i32) as isize) as u32) << 16i32
            | crate::rtl::rom_table_word!(game, 0xa2838f, v2 as isize) as u32,
    );
    if (*E).base.y_pos as i32 - 200i32 & 0x8000i32 != 0 {
        (*E).botwoon_var_28 = ((*E).botwoon_var_28).wrapping_add(192u16);
    } else {
        (*E).base.y_pos = 200u16;
        (*E).botwoon_var_D = ((0xb39acai32) as u16);
        game.eproj_spawn_pt = Point16U {
            x: (*E).base.x_pos,
            y: (*E).base.y_pos,
        };
        SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 0x1du16, game);
        Botwoon_QueueExplosionSfx(game);
        (*E).base.properties = ((*E).base.properties) | (0x500u16);
    };
}
pub unsafe fn Botwoon_Func_16(game: &mut Game) {
    let E = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).botwoon_var_5F != 0 {
        (*E).botwoon_var_D = ((0xb39af9i32) as u16);
        Botwoon_Func_17(ram_word!(game, cur_enemy_index), game);
    }
}
pub unsafe fn Botwoon_Func_17(k: u16, game: &mut Game) {
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0xfu8, 0x4u8, 0xb79b as VoidP),
        game,
    );
    Enemy_ItemDrop_Botwoon(k, game);
    let E = Get_Botwoon(k as i32, game);
    set_fields! {[(*E)], 0u16; botwoon_var_23, botwoon_var_24, botwoon_var_25}
}
pub unsafe fn Botwoon_Func_18(k: u16, game: &mut Game) {
    let E = Get_Botwoon(k as i32, game);
    if (*E).botwoon_var_23 as i32 - 192i32 & 0x8000i32 != 0 {
        if (*E).botwoon_var_23 as i32 - 64i32 & 0x8000i32 == 0 {
            let v2: i16 = (((*E).botwoon_var_24) as i16).wrapping_sub(1i16);
            (*E).botwoon_var_24 = v2 as u16;
            if v2 < 0i16 {
                (*E).botwoon_var_24 = 12u16;
                let x: u16 = ((NextRandom(game) as i32 & 0x1fi32) + 232i32) as u16;
                let y: u16 = ((*E).botwoon_var_23 as i32 + (NextRandom(game) as i32 & 0x1fi32)
                    - 8i32) as u16;
                CreateSpriteAtPos(x, y, 29u16, 2560u16, game);
                Botwoon_QueueExplosionSfx(game);
            }
            let v3: i16 = (((*E).botwoon_var_25) as i16).wrapping_sub(1i16);
            (*E).botwoon_var_25 = v3 as u16;
            if v3 < 0i16 {
                (*E).botwoon_var_25 = 4u16;
                for _ in 0i32..2i32 {
                    let x_0: u16 = ((NextRandom(game) as i32 & 0x3fi32) + 224i32) as u16;
                    let y_0: u16 = ((*Get_Botwoon(k as i32, game)).botwoon_var_23 as i32
                        + ((NextRandom(game) as i32 & 0x1fi32) - 8i32))
                        as u16;
                    CreateSpriteAtPos(x_0, y_0, 9u16, 2560u16, game);
                }
            }
        }
        (*E).botwoon_var_23 = (*E).botwoon_var_23.wrapping_add(1);
    } else {
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
        SetBossBitForCurArea(2u16, game);
        QueueMusic_Delayed8(3u16, game);
    };
}
pub unsafe fn Botwoon_Func_19(game: &mut Game) {
    let pt: Point16U = Botwoon_Func_20(ram_word!(game, cur_enemy_index), game);
    let v1: u16 = CalculateAngleFromXY(pt.x, pt.y, game);
    let E = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).botwoon_var_39 = v1;
    (*E).botwoon_var_3A = ((64u8).wrapping_sub((v1) as u8)) as u16;
    if (*E).botwoon_var_33 == (*E).botwoon_var_34 {
        Botwoon_Func_21(game);
    } else {
        (*E).botwoon_var_34 = (*E).botwoon_var_33;
        (*E).botwoon_var_3C = 1u16;
    };
}
pub unsafe fn Botwoon_Func_20(k: u16, game: &mut Game) -> Point16U {
    let E = Get_Botwoon(k as i32, game);
    let v2: i32 = (*E).botwoon_var_37 as i32 >> 1i32;
    let v3: u16 = (crate::rtl::rom_table_word!(game, 0xb3949b, v2 as isize) as i32 + 4i32
        - (*E).base.x_pos as i32) as u16;
    let mut r18: u16 = v3;
    if v3 as i32 - 256i32 & 0x8000i32 != 0 {
        if v3 as i32 + 256i32 & 0x8000i32 != 0 {
            r18 = -255i32 as u16;
        }
    } else {
        r18 = 255u16;
    }
    let v4: u16 = (crate::rtl::rom_table_word!(game, 0xb3949b, (v2 + 2i32) as isize) as i32 + 4i32
        - (*E).base.y_pos as i32) as u16;
    let mut r20: u16 = v4;
    if v4 as i32 - 256i32 & 0x8000i32 != 0 {
        if v4 as i32 + 256i32 & 0x8000i32 != 0 {
            r20 = -255i32 as u16;
        }
    } else {
        r20 = 255u16;
    }
    Point16U { x: r18, y: r20 }
}
pub unsafe fn Botwoon_Func_21(game: &mut Game) {
    let E = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    let pt: Point32 = ConvertAngleToXy((*E).botwoon_var_3A, (*E).botwoon_var_38, game);
    EnemyFunc_B691((*E).botwoon_var_3A, pt, game);
}
pub unsafe fn Botwoon_Func_22(game: &mut Game) {
    let E = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    let botwoon_var_B: u16 = (*E).botwoon_var_B;
    *((ram_ptr!(game, u16; 0x9000) as *mut u8).offset(botwoon_var_B as i32 as isize) as *mut u16) =
        (*E).base.x_pos;
    *((ram_ptr!(game, u16; 0x9002) as *mut u8).offset(botwoon_var_B as i32 as isize) as *mut u16) =
        (*E).base.y_pos;
}
pub unsafe fn Botwoon_Func_23(game: &mut Game) {
    let E = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    let mut r18: u16 = ((*E).botwoon_var_B as i32 - (*E).botwoon_var_C as i32 & 0x3ffi32) as u16;
    let mut n: i32 = 24i32;
    loop {
        let ET = Get_Botwoon(n, game);
        let botwoo_var_00: u16 = (*ET).botwoon_var_00;
        if (*E).botwoon_var_36 == r18 {
            let v2: u16 = n as u16;
            let v4: bool = (*ET).botwoon_var_10 == 1u16;
            (*ET).botwoon_var_10 = ((*ET).botwoon_var_10) ^ (1u16);
            if v4 {
                update_ram_word!(
                    game.eproj_properties[(botwoo_var_00 as i32 >> 1i32)],
                    |fresh0| (fresh0 & !(0x2000))
                );
                *(ram_ptr!(game, u16; 0xf380)).offset((botwoo_var_00 as i32 >> 1i32) as isize) =
                    1u16;
            } else {
                update_ram_word!(
                    game.eproj_properties[(botwoo_var_00 as i32 >> 1i32)],
                    |fresh1| (fresh1) | (0x2000u16)
                );
                *(ram_ptr!(game, u16; 0xf380)).offset((botwoo_var_00 as i32 >> 1i32) as isize) =
                    2u16;
            }
            if v2 == 0 {
                set_fields! {[(*E)], 0u16; botwoon_var_35, botwoon_var_3E}
                (*E).botwoon_var_36 = -1i32 as u16;
                if (*E).botwoon_var_33 == 0 {
                    (*E).botwoon_var_3E = 1u16;
                }
            }
        }
        let v7: i32 = botwoo_var_00 as i32 >> 1i32;
        set_ram_word!(
            game.eproj_x_pos[v7] =
                *((ram_ptr!(game, u16; 0x9000) as *mut u8).offset(r18 as i32 as isize) as *mut u16)
        );
        set_ram_word!(
            game.eproj_y_pos[v7] =
                *((ram_ptr!(game, u16; 0x9002) as *mut u8).offset(r18 as i32 as isize) as *mut u16)
        );
        r18 = (r18 as i32 - (*E).botwoon_var_C as i32 & 0x3ffi32) as u16;
        n -= 2i32;
        if ((n & 0x8000i32) != 0i32) {
            break;
        }
    }
}
pub unsafe fn Botwoon_Func_24(game: &mut Game) {
    let E = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).botwoon_var_B = ((*E).botwoon_var_B as i32 + 4i32 & 0x3ffi32) as u16;
}
pub unsafe fn Botwoon_Func_25(game: &mut Game) {
    let E = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    let mut n: i32 = 24i32;
    loop {
        let mut r24: u16 = 0u16;
        let ERR = Get_Botwoon(n, game);
        let botwoo_var_00: u16 = (*ERR).botwoon_var_00;
        if (*ERR).botwoon_var_10 != 0 {
            r24 = 256u16;
        }
        let mut x: u16 = 0;
        let mut y: u16 = 0;
        if n == 24i32 {
            let v4: i32 = botwoo_var_00 as i32 >> 1i32;
            x = ((*E).base.x_pos).wrapping_sub(ram_word!(game, eproj_x_pos[v4]));
            y = ((*E).base.y_pos).wrapping_sub(ram_word!(game, eproj_y_pos[v4]));
        } else {
            if n == 0 {
                r24 = (r24).wrapping_add(512u16);
            }
            let v2: i32 = botwoo_var_00 as i32 >> 1i32;
            x = (ram_word!(game, eproj_x_pos[(v2 + 1i32)]))
                .wrapping_sub(ram_word!(game, eproj_x_pos[v2]));
            y = (ram_word!(game, eproj_y_pos[(v2 + 1i32)]))
                .wrapping_sub(ram_word!(game, eproj_y_pos[v2]));
        }
        set_ram_word!(
            game.eproj_E[(botwoo_var_00 as i32 >> 1i32)] = (2i32
                * (((r24).wrapping_add(CalculateAngleFromXY(x, y, game))) as i32 >> 5i32))
                as u16
        );
        n -= 2i32;
        if (n < 0i32) {
            break;
        }
    }
}
pub unsafe fn Botwoon_Func_26(k: u16, game: &mut Game) {
    let E = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    let x: u16 = ((*E).base.x_pos).wrapping_sub((*E).botwoon_var_56);
    let y: u16 = ((*E).base.y_pos).wrapping_sub((*E).botwoon_var_57);
    let r22: u16 = CalculateAngleFromXY(x, y, game);
    if x != 0u16 || y != 0u16 {
        let mut v1: u16 = 0;
        if (*E).botwoon_var_33 != 0 {
            (*E).base.layer = 7u16;
            (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
            v1 = addr_kBotwoon_Ilist_9389 as u16;
        } else {
            (*E).base.layer = 2u16;
            (*E).base.properties =
                ((*E).base.properties as i32 & !(kEnemyProps_Tangible as i32)) as u16;
            (*E).botwoon_var_45 = ((*E).botwoon_var_45 as i32
                + (((r22) as i8).wrapping_sub(((*E).botwoon_var_45) as i8)) as i32 * 3i32 / 4i32)
                as u8 as u16;
            v1 = crate::rtl::rom_table_word!(
                game,
                0xb3946b,
                ((*E).botwoon_var_45 as i32 >> 5i32) as isize
            );
        }
        if v1 != (*E).botwoon_var_3B {
            (*E).base.current_instruction = v1;
            (*E).botwoon_var_3B = v1;
            (*E).base.instruction_timer = 1u16;
            (*E).base.timer = 0u16;
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
pub unsafe fn Botwoon_Func_27(k: u16, game: &mut Game) {
    let E = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.layer = 2u16;
    let v2: u16 = CalculateAngleOfSamusFromEnemy(ram_word!(game, cur_enemy_index), game);
    (*E).botwoon_var_3D = v2;
    let v3: u16 = crate::rtl::rom_table_word!(
        game,
        0xb3948b,
        ((((v2) as u8).wrapping_add(16u8)) as i32 >> 5i32) as isize
    );
    (*E).base.current_instruction = v3;
    (*E).botwoon_var_3B = v3;
    (*E).base.instruction_timer = 1u16;
    (*E).base.timer = 0u16;
    (*E).botwoon_var_3D = ((64u8).wrapping_sub(((*E).botwoon_var_3D) as u8)) as u16;
    (*E).botwoon_var_F = ((0xb39ee0i32) as u16);
    if (*E).botwoon_var_D != 0x99e4u16 {
        (*E).botwoon_var_F = ((0xb39f34i32) as u16);
    }
    CallEnemyPreInstr((((*E).botwoon_var_F) as u32) | (0xb30000u32), game);
}
pub unsafe fn Botwoon_Func_28(k: u16, game: &mut Game) {
    let E = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).botwoon_var_5A != 0 {
        *ram_ptr!(game; 0x60b) = (((*E).botwoon_var_3D) as u8).wrapping_sub(32u8);
        let varE32: u16 = g_word_B39E77[(*E).botwoon_var_3F as usize] as u16;
        for _ in 0i32..5i32 {
            SpawnEprojWithGfx(
                varE32,
                ram_word!(game, cur_enemy_index),
                addr_kEproj_BotwoonsSpit as u16,
                game,
            );
            let ref mut fresh2 = *ram_ptr!(game; 0x60b);
            *fresh2 = (*fresh2).wrapping_add(16u8);
        }
        (*E).botwoon_var_5A = 0u16;
        (*E).botwoon_var_F = ((0xb39f7ai32) as u16);
    }
}
pub unsafe fn Botwoon_Func_29(k: u16, game: &mut Game) {
    let E = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    *ram_ptr!(game; 0x60b) = (((*E).botwoon_var_3D) as u8).wrapping_sub(16u8);
    let varE32: u16 = g_word_B39E77[(*E).botwoon_var_3F as usize] as u16;
    for _ in 0i32..3i32 {
        SpawnEprojWithGfx(
            varE32,
            ram_word!(game, cur_enemy_index),
            addr_kEproj_BotwoonsSpit as u16,
            game,
        );
        let ref mut fresh3 = *ram_ptr!(game; 0x60b);
        *fresh3 = (*fresh3).wrapping_add(16u8);
    }
    (*Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game)).botwoon_var_F =
        ((0xb39f7ai32) as u16);
}
pub unsafe fn Botwoon_Func_30(k: u16, game: &mut Game) {
    let E = Get_Botwoon(k as i32, game);
    let v2: i16 = (((*E).botwoon_var_21) as i16).wrapping_sub(1i16);
    (*E).botwoon_var_21 = v2 as u16;
    if v2 < 0i16 {
        (*E).botwoon_var_21 = 0u16;
        (*E).botwoon_var_F = ((0xb39dc0i32) as u16);
    }
}
pub unsafe fn Botwoon_Func_31(k: u16, game: &mut Game) {
    let E = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    if (*Get_Botwoon(k as i32, game)).botwoon_var_35 == 0 {
        let mut n: i32 = 24i32;
        loop {
            let v2: i32 = n >> 1i32;
            if (((*E).base.x_pos) as i16)
                .wrapping_sub((crate::rtl::rom_table_word!(game, 0xb3949b, v2 as isize)) as i16)
                >= 0i16
                && (((*E).base.x_pos) as i16).wrapping_sub(
                    (crate::rtl::rom_table_word!(game, 0xb3949b, (v2 + 1i32) as isize)) as i16,
                ) < 0i16
                && (((*E).base.y_pos) as i16).wrapping_sub(
                    (crate::rtl::rom_table_word!(game, 0xb3949b, (v2 + 2i32) as isize)) as i16,
                ) >= 0i16
                && (((*E).base.y_pos) as i16).wrapping_sub(
                    (crate::rtl::rom_table_word!(game, 0xb3949b, (v2 + 3i32) as isize)) as i16,
                ) < 0i16
            {
                (*E).botwoon_var_35 = 1u16;
                (*E).botwoon_var_33 = ((*E).botwoon_var_33) ^ (1u16);
                (*E).botwoon_var_36 = (*E).botwoon_var_B;
                (*E).botwoon_var_26 = (*E).botwoon_var_26.wrapping_sub(1);
                break;
            } else {
                (*E).botwoon_var_35 = 0u16;
                n -= 8i32;
                if (n < 0i32) {
                    break;
                }
            }
        }
    }
}
pub unsafe fn Botwoon_Touch(game: &mut Game) {
    Enemy_NormalTouchAI_SkipDeathAnim_B3(game);
    let E = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).base.health == 0 {
        (*E).botwoon_var_2F = 1u16;
        Botwoon_Func_4(game);
    }
}
pub unsafe fn Botwoon_Shot(game: &mut Game) {
    let E = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).botwoon_var_4C = (*E).base.health;
    Enemy_NormalShotAI_SkipSomeParts_B3(game);
    if (*E).base.health == 0 {
        (*E).botwoon_var_2F = 1u16;
        Botwoon_Func_4(game);
    }
}
pub unsafe fn Botwoon_Powerbomb(game: &mut Game) {
    Enemy_NormalPowerBombAI_SkipDeathAnim_B3(game);
    let E = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).base.health == 0 {
        (*E).botwoon_var_2F = 1u16;
        Botwoon_Func_4(game);
    }
}
pub unsafe fn Botwoon_Func_32(game: &mut Game) {
    let E = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).botwoon_var_E = addr_loc_B3E28C as u16;
    (*E).botwoon_var_3C = 0u16;
    let v1 = RomPtr_B3(
        ((*E).botwoon_var_40 as i32 + addr_stru_B3E150 as i32) as u16,
        game,
    );
    (*E).botwoon_var_42 = *(v1 as *mut u16);
    (*E).botwoon_var_37 = *(v1.offset(4) as *mut u16);
    let v2: i16 = *(v1.offset(2) as *mut u16) as i16;
    (*E).botwoon_var_44 = v2 as u16;
    if v2 < 0i16 {
        (*E).botwoon_var_42 = ((*E).botwoon_var_42).wrapping_sub(4u16);
    }
    Botwoon_Func_33(game);
}
pub unsafe fn Botwoon_Func_33(game: &mut Game) {
    let mut x: u16 = 0u16;
    let mut y: u16 = 0u16;
    let E = Get_Botwoon(ram_word!(game, cur_enemy_index) as i32, game);
    let mut n: i32 = (*E).botwoon_var_38 as i32;
    let step: i32 = if ((*E).botwoon_var_44 & 0x8000) == 0 {
        2i32
    } else {
        -2i32
    };
    loop {
        let v6 = RomPtr_B3((*E).botwoon_var_42, game);
        let v7: u16 = SignExtend8(*v6 as u16);
        let mut v8: u16 = 0;
        if v7 == 0xff80u16 || {
            x = (x).wrapping_add(v7);
            v8 = SignExtend8(*v6.offset(1) as u16);
            v8 == 0xff80u16
        } {
            (*E).botwoon_var_41 = 0u16;
            (*E).botwoon_var_3C = 1u16;
            return;
        }
        y = (y).wrapping_add(v8);
        (*E).botwoon_var_42 = ((*E).botwoon_var_42 as i32 + step) as u16;
        n -= 1;
        if (n == 0) {
            break;
        }
    }
    if ((*E).botwoon_var_44 & 0x8000) != 0 {
        x = -(x as i32) as u16;
        y = -(y as i32) as u16;
    }
    (*E).base.x_pos = ((*E).base.x_pos).wrapping_add(x);
    (*E).base.y_pos = ((*E).base.y_pos).wrapping_add(y);
}
pub unsafe fn EscapeEtecoon_Instr_1(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    if ram_word!(game, lava_acid_y_pos) >= 0xceu16 {
        jp.offset(1)
    } else {
        *jp.offset(0) as usize as *const u16
    }
}
pub unsafe fn EscapeEtecoon_Instr_2(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E = Get_EscapeEtecoon(k as i32, game);
    (*E).base.x_pos = ((*E).base.x_pos).wrapping_add(*jp.offset(0));
    jp.offset(1)
}
pub unsafe fn EscapeEtecoon_Main(game: &mut Game) {
    let E = Get_EscapeEtecoon(ram_word!(game, cur_enemy_index) as i32, game);
    CallEnemyPreInstr((((*E).een_var_F) as u32) | (0xb30000u32), game);
}
pub unsafe fn EscapeEtecoon_E65C(k: u16, game: &mut Game) {
    let E = Get_EscapeEtecoon(k as i32, game);
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        0x38000u32,
    );
}
pub unsafe fn EscapeEtecoon_E670(k: u16, game: &mut Game) {
    if CheckEventHappened(0xfu16, game) != 0 {
        (*Get_EscapeEtecoon(k as i32, game))
            .base
            .current_instruction = addr_kEscapeEtecoon_Ilist_E5DA as u16;
    }
}
pub unsafe fn EscapeEtecoon_E680(k: u16, game: &mut Game) {
    let E = Get_EscapeEtecoon(k as i32, game);
    if Enemy_MoveRight_IgnoreSlopes(k, ((*E).een_var_A as i16 as i32) << 8i32, game) != 0 {
        (*E).base.instruction_timer = 1u16;
        let v3: bool = (-((*E).een_var_A as i32) & 0x8000i32) != 0i32;
        (*E).een_var_A = -((*E).een_var_A as i32) as u16;
        (*E).base.current_instruction = (if v3 {
            addr_kEscapeEtecoon_Ilist_E556 as i32
        } else {
            addr_kEscapeEtecoon_Ilist_E582 as i32
        }) as u16;
        if CheckEventHappened(0xfu16, game) != 0 {
            (*E).base.current_instruction = addr_kEscapeEtecoon_Ilist_E5AE as u16;
        }
    }
    Enemy_MoveDown(k, (1i16 as i32) << 16i32, game);
}
pub unsafe fn EscapeEtecoon_Init(game: &mut Game) {
    let E = Get_EscapeEtecoon(ram_word!(game, cur_enemy_index) as i32, game);
    if CheckEventHappened(0xfu16, game) != 0 {
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
    } else {
        (*E).base.properties = ((*E).base.properties as i32
            | (kEnemyProps_DisableSamusColl as i32 | kEnemyProps_Tangible as i32 | 0x8000i32))
            as u16;
        (*E).base.instruction_timer = 1u16;
        set_fields! {[(*E).base], 0u16; timer, palette_index}
        let v3: i32 = (*E).een_parameter_1 as i32 >> 1i32;
        (*E).base.x_pos = crate::rtl::rom_table_word!(game, 0xb3e718, v3 as isize);
        (*E).base.y_pos = crate::rtl::rom_table_word!(game, 0xb3e71e, v3 as isize);
        (*E).een_var_F = crate::rtl::rom_table_word!(game, 0xb3e724, v3 as isize);
        (*E).base.current_instruction = crate::rtl::rom_table_word!(game, 0xb3e72a, v3 as isize);
        (*E).een_var_A = crate::rtl::rom_table_word!(game, 0xb3e730, v3 as isize);
    };
}
pub unsafe fn EscapeDachora_Instr_2(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    if ram_word!(game, lava_acid_y_pos) >= 0xceu16 {
        jp.offset(1)
    } else {
        *jp.offset(0) as usize as *const u16
    }
}
pub unsafe fn EscapeDachora_Instr_3(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    if CheckEventHappened(0xfu16, game) != 0 {
        *jp.offset(0) as usize as *const u16
    } else {
        jp.offset(1)
    }
}
pub unsafe fn EscapeDachora_Instr_1(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E = Get_EscapeDachora(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.x_pos = ((*E).base.x_pos).wrapping_sub(6u16);
    jp
}
pub unsafe fn EscapeDachora_Instr_4(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E = Get_EscapeDachora(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.x_pos = ((*E).base.x_pos).wrapping_add(6u16);
    jp
}
pub unsafe fn EscapeDachora_Init(game: &mut Game) {
    let E = Get_EscapeDachora(ram_word!(game, cur_enemy_index) as i32, game);
    if CheckEventHappened(0xfu16, game) != 0 {
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
    } else {
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_DisableSamusColl) as u16);
        (*E).base.spritemap_pointer = addr_kSpritemap_Nothing_B3 as u16;
        (*E).base.instruction_timer = 1u16;
        (*E).base.timer = 0u16;
        (*E).base.current_instruction = addr_kEscapeDachora_Ilist_E964 as u16;
    };
}
