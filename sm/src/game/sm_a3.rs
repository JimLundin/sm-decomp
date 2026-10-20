use crate::c_library::printf;
use crate::cpu_infra::Call;
use crate::game::callback_tables::callback_table;
pub use crate::game::constants::bank_86::*;
pub use crate::game::constants::bank_b4::*;
pub use crate::game::constants::enemy_properties::*;
use crate::game::enemy_instruction;
use crate::game::sm_80::{
    Multiply16x16, NextRandom, QueueSfx1_Max6, QueueSfx2_Max3, QueueSfx2_Max6, QueueSfx3_Max6,
};
use crate::game::sm_86::SpawnEprojWithGfx;
use crate::game::sm_90::{CallSomeSamusCode, ProjectileReflection, ResetProjectileData};
use crate::game::sm_91::Samus_DealDamage;
use crate::game::sm_a0::{
    Abs16, CalculateAngleOfSamusFromEnemy, CalculateBlockContainingPixelPos, CallEnemyPreInstr,
    CheckIfEnemyIsOnScreen, CheckIfEnemyTouchesSamus, ConvertAngleToXy, CosineMult8bit,
    CosineMult8bitFull, DetermineDirectionOfSamusFromEnemy, EnemyDeathAnimation, EnemyFunc_B691,
    EnemyFunc_BBBF, EnemyFunc_BC76, EnemyFunc_BF8A, EnemyFunc_C8AD, EnemyGrappleDeath,
    Enemy_ItemDrop_Metroid, Enemy_MoveDown, Enemy_MoveRight_IgnoreSlopes,
    Enemy_MoveRight_ProcessSlopes, Enemy_SwitchToFrozenAi, GetSamusEnemyDelta_X,
    GetSamusEnemyDelta_Y, IsSamusWithinEnemy_X, IsSamusWithinEnemy_Y, NormalEnemyFrozenAI,
    NormalEnemyPowerBombAi, NormalEnemyShotAi, NormalEnemyShotAiSkipDeathAnim_CurEnemy,
    NormalEnemyTouchAi, SignExtend8, SineMult8bit, SineMult8bitFull, SwitchEnemyAiToMainAi,
};
use crate::game::sm_b4::CreateSpriteAtPos;
pub use crate::game::waver::{Enemy_Waver, Waver_Func_1, Waver_Init, Waver_Instr_1, Waver_Main};
use crate::game_state::Game;
use crate::ram_map::ram_ptr;
use crate::ram_map::word;
use crate::rtl::abs16;
use crate::rtl::words;
use crate::rtl::{AddToHiLo, SetHiLo};
use crate::rtl::{RomPtr, SnesDivide, Unreachable};
use crate::set_fields;
pub use crate::types::{
    EnemyBase, EnemyData, EnemyProps, EnemySpawnData, Func_V, Func_Y_V, LongPtr, VoidP,
};
use ::core::ffi::c_char;
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct MaridiaSnailData2 {
    pub field_0: VoidP,
    pub field_2: u16,
    pub field_4: VoidP,
    pub field_6: u16,
}
pub use crate::types::Consts_A3;
pub use crate::types::{Consts_86, EnemyDef};
crate::game::constants::typed_constants! { Consts_A3;
    addr_kMetroid_Ilist_EA25 = 59941, addr_kMetroid_Ilist_E9CF = 59855,
    addr_locret_A3E08A = 57482, addr_kWreckedShipOrangeZoomer_Ilist_E01F = 57375,
    addr_kWreckedShipOrangeZoomer_Ilist_E003 = 57347, addr_kWreckedShipOrangeZoomer_Ilist_DFE7 = 57319,
    addr_kWreckedShipOrangeZoomer_Ilist_DFCB = 57291, addr_stru_A3CD3A = 52538,
    addr_stru_A3CD32 = 52530, addr_stru_A3CD2A = 52522,
    addr_stru_A3CD22 = 52514, addr_stru_A3CD1A = 52506,
    addr_stru_A3CD12 = 52498, addr_stru_A3CD0A = 52490,
    addr_stru_A3CD02 = 52482, addr_stru_A3CCFA = 52474,
    addr_stru_A3CCF2 = 52466, addr_stru_A3CCEA = 52458,
    addr_stru_A3CCE2 = 52450, addr_kSkree_Ilist_C65E = 50782,
    addr_kBang_Ilist_B75E = 46942, addr_kMaridiaRefillCandy_Ilist_B3C1 = 46017,
    addr_kMochtroid_Ilist_A759 = 42841, addr_kMochtroid_Ilist_A745 = 42821,
    addr_kPlatformThatFallsWithSamus_Sprmap_A015 = 40981, addr_kPlatformThatFallsWithSamus_Sprmap_A009 = 40969,
    addr_kPlatformThatFallsWithSamus_Ilist_9C55 = 40021, addr_kPlatformThatFallsWithSamus_Ilist_9C3F = 39999,
    addr_kPlatformThatFallsWithSamus_Ilist_9C29 = 39977, addr_kPlatformThatFallsWithSamus_Ilist_9C13 = 39955,
    addr_kPlatformThatFallsWithSamus_Ilist_9BFD = 39933, addr_kPlatformThatFallsWithSamus_Ilist_9BE7 = 39911,
    addr_kPlatformThatFallsWithSamus_Ilist_9BD1 = 39889, addr_kPlatformThatFallsWithSamus_Ilist_9BBB = 39867,
    addr_kElevator_Ilist_94D6 = 38102, addr_kMaridiaFish_Ilist_9072 = 36978,
    addr_kMaridiaFish_Ilist_9060 = 36960, addr_kMaridiaFish_Ilist_903C = 36924,
    addr_kMaridiaFish_Ilist_902A = 36906, addr_kFireflea_Ilist_8C2F = 35887,
    addr_kMetalee_Ilist_8910 = 35088, addr_kWaver_Ilist_86A7 = 34471,
    addr_kSpritemap_Nothing_A3 = 32845,
}
pub use crate::types::Consts_B4;
pub use crate::types::Point32;
pub type Enemy_Bang = crate::game::enemy_layouts::EnemyLayout72;

pub type Enemy_MaridiaFish = crate::game::enemy_layouts::EnemyLayout40;

pub type Enemy_MaridiaRefillCandy = crate::game::enemy_layouts::EnemyLayout40;

pub type Enemy_MaridiaSnail = crate::game::enemy_layouts::EnemyLayout72;

pub type Enemy_Metalee = crate::game::enemy_layouts::EnemyLayout8;

pub type Enemy_PlatformThatFallsWithSamus = crate::game::enemy_layouts::EnemyLayout40;

pub type Enemy_Reflec = crate::game::enemy_layouts::EnemyLayout8;

pub type Enemy_Sidehopper = crate::game::enemy_layouts::EnemyLayout40;

pub type Enemy_Skree = crate::game::enemy_layouts::EnemyLayout40;

pub type Enemy_WreckedShipOrangeZoomer = crate::game::enemy_layouts::EnemyLayout40;

pub type Enemy_StoneZoomer = crate::game::enemy_layouts::EnemyLayout8;

pub type Enemy_Elevator = crate::game::enemy_layouts::EnemyLayout8;

pub type Enemy_FireZoomer = crate::game::enemy_layouts::EnemyLayout40;

pub type Enemy_FireFlea = crate::game::enemy_layouts::EnemyLayout40;

pub type Enemy_Metroid = crate::game::enemy_layouts::EnemyLayout40;

pub type Enemy_Mochtroid = crate::game::enemy_layouts::EnemyLayout72;

pub type Enemy_Roach = crate::game::enemy_layouts::EnemyLayout72;

const RomPtr_A3: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0xa3>;
#[inline]
unsafe fn get_EnemyDef_A2(a: u16, game: &mut Game) -> *mut EnemyDef {
    RomPtr((0xa00000u32) | ((a) as u32), game) as *mut EnemyDef
}
crate::game::constants::typed_constants! { i32;
    fnBang_Func_6 = 0xa3bca5,
    fnBang_Func_7 = 0xa3bcc1, fnBang_Func_8 = 0xa3bcc5,
    fnBang_Func_10 = 0xa3bd1c, fnBang_Func_11 = 0xa3bd2c,
    fnnullsub_215 = 0xa3cf5f, fnMaridiaSnail_Func_15 = 0xa3d1b3,
    fnReflec_Func_1 = 0xa3db0c, fnnullsub_304 = 0xa3e6c1,
    fnFireZoomer_Func_1 = 0xa3e6c8, fnFireZoomer_Func_2 = 0xa3e785,
}
crate::ram_map::ram_pointer_functions! {
    gEnemyData(o: i32) -> EnemyData = [0xf78, o as isize];
    gEnemySpawnData(o: i32) -> EnemySpawnData = [0x7000, o as isize];
    Get_Bang(o: i32) -> Enemy_Bang = [0xf78, o as isize];
    Get_Elevator(o: i32) -> Enemy_Elevator = [0xf78, o as isize];
    Get_FireFlea(o: i32) -> Enemy_FireFlea = [0xf78, o as isize];
    Get_FireZoomer(o: i32) -> Enemy_FireZoomer = [0xf78, o as isize];
    Get_MaridiaFish(o: i32) -> Enemy_MaridiaFish = [0xf78, o as isize];
    Get_MaridiaRefillCandy(o: i32) -> Enemy_MaridiaRefillCandy = [0xf78, o as isize];
    Get_MaridiaSnail(o: i32) -> Enemy_MaridiaSnail = [0xf78, o as isize];
    Get_Metalee(o: i32) -> Enemy_Metalee = [0xf78, o as isize];
    Get_Metroid(o: i32) -> Enemy_Metroid = [0xf78, o as isize];
    Get_Mochtroid(o: i32) -> Enemy_Mochtroid = [0xf78, o as isize];
    Get_PlatformThatFallsWithSamus(o: i32) -> Enemy_PlatformThatFallsWithSamus = [0xf78, o as isize];
    Get_Reflec(o: i32) -> Enemy_Reflec = [0xf78, o as isize];
    Get_Roach(o: i32) -> Enemy_Roach = [0xf78, o as isize];
    Get_Sidehopper(o: i32) -> Enemy_Sidehopper = [0xf78, o as isize];
    Get_Skree(o: i32) -> Enemy_Skree = [0xf78, o as isize];
    Get_StoneZoomer(o: i32) -> Enemy_StoneZoomer = [0xf78, o as isize];
    Get_WreckedShipOrangeZoomer(o: i32) -> Enemy_WreckedShipOrangeZoomer = [0xf78, o as isize];
}
static g_word_A3A76D: [i16; 4] = [2, 0, -2, 0];
static g_word_A3A775: [i16; 4] = [0, -2, 0, 2];
crate::forward_calls! { pub unsafe fn (game: &mut Game);
    Enemy_GrappleReact_NoInteract_A3 => SwitchEnemyAiToMainAi(game);
    Enemy_GrappleReact_KillEnemy_A3 => EnemyGrappleDeath(game);
    Enemy_GrappleReact_CancelBeam_A3 => Enemy_SwitchToFrozenAi(game);
    Enemy_NormalTouchAI_A3 => NormalEnemyTouchAi(game);
    Enemy_NormalShotAI_A3 => NormalEnemyShotAi(game);
    Enemy_NormalShotAI_SkipSomeParts_A3 => NormalEnemyShotAiSkipDeathAnim_CurEnemy(game);
    Enemy_NormalPowerBombAI_A3 => NormalEnemyPowerBombAi(game);
    Enemy_NormalFrozenAI_A3 => NormalEnemyFrozenAI(game);
}
pub unsafe fn Metalee_Instr_1(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_Metalee(word!(game.cur_enemy_index) as i32, game)).var_E = 1;
    jp
}
pub unsafe fn Metalee_Init(game: &mut Game) {
    let E = Get_Metalee(word!(game.cur_enemy_index) as i32, game);
    set_fields! {[(*E)], 0; var_C, var_D, var_E}
    (*E).base.current_instruction = addr_kMetalee_Ilist_8910 as u16;
    (*E).var_B = ((0xa38987i32) as u16);
}
crate::game::address_dispatch! {
    pub unsafe fn CallMetaleeFunc(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        10717575 => Metalee_Func_1;
        10717652 => [{
                    Metalee_Func_3(word!(game.cur_enemy_index), game);
                }];
        10717683 => Metalee_Func_4; 10717788 => Metalee_Func_5;
    }
}
pub unsafe fn Metalee_Main(game: &mut Game) {
    let E = Get_Metalee(word!(game.cur_enemy_index) as i32, game);
    CallMetaleeFunc((((*E).var_B) as u32) | (0xa30000u32), game);
}
pub unsafe fn Metalee_Func_1(game: &mut Game) {
    let E = Get_Metalee(word!(game.cur_enemy_index) as i32, game);
    if abs16(((*E).base.x_pos).wrapping_sub(word!(game.samus_x_pos))) < 0x48 {
        Metalee_Func_2(word!(game.cur_enemy_index), game);
        (*E).var_C = (*E).var_C.wrapping_add(1);
        Metalee_Func_6(game);
        (*E).var_B = ((0xa389d4i32) as u16);
    }
}
pub unsafe fn Metalee_Func_2(k: u16, game: &mut Game) {
    let E = Get_Metalee(k as i32, game);
    let div: u16 =
        SnesDivide((word!(game.samus_y_pos)).wrapping_sub((*E).base.y_pos), 24) as u8 as u16;
    (*E).var_F = (div).wrapping_add(4);
}
pub unsafe fn Metalee_Func_3(k: u16, game: &mut Game) {
    let E = Get_Metalee(k as i32, game);
    if (*E).var_E != 0 {
        (*E).var_E = 0;
        (*E).var_C = (*E).var_C.wrapping_add(1);
        Metalee_Func_6(game);
        (*E).var_B = ((0xa389f3i32) as u16);
        QueueSfx2_Max6(0x5b, game);
    }
}
pub unsafe fn Metalee_Func_4(game: &mut Game) {
    let E = Get_Metalee(word!(game.cur_enemy_index) as i32, game);
    (*E).var_A = 21;
    let v1: u16 = ((*E).base.properties) | (3u16);
    (*E).base.properties = v1;
    if EnemyFunc_BF8A(
        word!(game.cur_enemy_index),
        v1,
        ((*E).var_F as i16 as i32) << 16,
        game,
    ) & 1
        != 0
    {
        (*E).base.instruction_timer = 1;
        (*E).base.timer = 0;
        (*E).var_B = ((0xa38a5ci32) as u16);
        QueueSfx2_Max6(0x5c, game);
    } else {
        (*E).base.y_pos = ((*E).base.y_pos).wrapping_add((*E).var_F);
        (*E).base.x_pos = ((*E).base.x_pos as i32
            + if (((*E).base.x_pos) as i16).wrapping_sub((word!(game.samus_x_pos)) as i16) >= 0 {
                -2i32
            } else {
                2
            }) as u16;
    };
}
pub unsafe fn Metalee_Func_5(game: &mut Game) {
    let E = Get_Metalee(word!(game.cur_enemy_index) as i32, game);
    crate::update_place! { previous fresh8 = (*E).var_A => (*E).var_A.wrapping_sub(1); }
    if fresh8 == 1 {
        (*gEnemySpawnData(word!(game.cur_enemy_index) as i32, game)).vram_tiles_index =
            ((*E).base.vram_tiles_index) | ((*E).base.palette_index);
        (*E).base.palette_index = 2560;
        (*E).base.vram_tiles_index = 0;
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
    } else {
        if (*E).var_A == 8 {
            let v2: u16 = word!(game.cur_enemy_index);
            let mut parameter = SpawnEprojWithGfx(8, v2, addr_stru_868BFA as u16, game);
            parameter = SpawnEprojWithGfx(parameter, v2, addr_stru_868C08 as u16, game);
            parameter = SpawnEprojWithGfx(parameter, v2, addr_stru_868C16 as u16, game);
            SpawnEprojWithGfx(parameter, v2, addr_stru_868C24 as u16, game);
        }
        (*E).base.y_pos = (*E).base.y_pos.wrapping_add(1);
    };
}
pub unsafe fn Metalee_Func_6(game: &mut Game) {
    let E = Get_Metalee(word!(game.cur_enemy_index) as i32, game);
    let metalee_var_C: u16 = (*E).var_C;
    if metalee_var_C != (*E).var_D {
        (*E).var_D = metalee_var_C;
        enemy_instruction! {[(*E).base]; instruction = crate::rtl::rom_table_word!(game, 0xa3894e, metalee_var_C as isize), timer = 1}
        (*E).base.timer = 0;
    }
}
pub unsafe fn Metalee_Shot(game: &mut Game) {
    let E = Get_Metalee(word!(game.cur_enemy_index) as i32, game);
    let varE2A: u16 = (*E).base.vram_tiles_index;
    let varE2C: u16 = (*E).base.palette_index;
    Enemy_NormalShotAI_A3(game);
    if (*Get_Metalee(word!(game.cur_enemy_index) as i32, game))
        .base
        .health
        == 0
    {
        (*E).base.vram_tiles_index = varE2A;
        (*E).base.palette_index = varE2C;
        let v2: u16 = word!(game.cur_enemy_index);
        SpawnEprojWithGfx(
            (*E).var_A,
            word!(game.cur_enemy_index),
            addr_stru_868BFA as u16,
            game,
        );
        SpawnEprojWithGfx(0, v2, addr_stru_868C08 as u16, game);
        SpawnEprojWithGfx(0, v2, addr_stru_868C16 as u16, game);
        SpawnEprojWithGfx(0, v2, addr_stru_868C24 as u16, game);
        set_fields! {[(*E).base], 0; vram_tiles_index, palette_index}
    }
}
pub unsafe fn Fireflea_Init(game: &mut Game) {
    let E = Get_FireFlea(word!(game.cur_enemy_index) as i32, game);
    (*E).base.current_instruction = addr_kFireflea_Ilist_8C2F as u16;
    if ((*E).parameter_1 & 2) != 0 {
        Fireflea_Func_1(word!(game.cur_enemy_index), game);
        Fireflea_Func_2(word!(game.cur_enemy_index), game);
        Fireflea_Func_3(word!(game.cur_enemy_index), game);
        Fireflea_Func_4(word!(game.cur_enemy_index), game);
        Fireflea_Func_5(word!(game.cur_enemy_index), game);
    } else {
        Fireflea_Func_3(word!(game.cur_enemy_index), game);
        Fireflea_Func_4(word!(game.cur_enemy_index), game);
        Fireflea_Func_6(word!(game.cur_enemy_index), game);
    };
}
pub unsafe fn Fireflea_Func_1(k: u16, game: &mut Game) {
    let E = Get_FireFlea(k as i32, game);
    (*E).var_E = (*E).base.x_pos;
    (*E).var_F = (*E).base.y_pos;
}
pub unsafe fn Fireflea_Func_2(k: u16, game: &mut Game) {
    let E = Get_FireFlea(k as i32, game);
    (*E).var_D = ((byte!((*E).parameter_1; 1) as i32) << 8) as u16;
}
pub unsafe fn Fireflea_Func_3(k: u16, game: &mut Game) {
    let E = Get_FireFlea(k as i32, game);
    let mut v2: u16 = (8u16).wrapping_mul((byte!((*E).parameter_2; 0)) as u16);
    if ((*E).parameter_1 & 1) == 0 {
        v2 = (v2).wrapping_add(4);
    }
    (*E).var_02 = v2;
    let v3: i32 = v2 as i32 >> 1;
    (*E).var_B = crate::rtl::rom_table_word!(game, 0xa28187, v3 as isize);
    (*E).var_A = crate::rtl::rom_table_word!(game, 0xa28187, (v3 + 1) as isize);
}
pub unsafe fn Fireflea_Func_4(k: u16, game: &mut Game) {
    let E = Get_FireFlea(k as i32, game);
    (*E).var_C = byte!(@pointer (game.g_rom.offset(crate::rtl::lorom_offset(0xa38d1d)) as *mut u16)
        .offset(*(&raw mut (*E).parameter_2 as *mut u8).offset(1isize) as isize); 0)
        as u16;
}
pub unsafe fn Fireflea_Func_5(k: u16, game: &mut Game) {
    let E = Get_FireFlea(k as i32, game);
    (*E).base.x_pos = ((*E).var_E).wrapping_add(CosineMult8bit((*E).var_D, (*E).var_C, game));
    (*E).base.y_pos = ((*E).var_F).wrapping_add(SineMult8bit((*E).var_D, (*E).var_C, game));
}
pub unsafe fn Fireflea_Func_6(k: u16, game: &mut Game) {
    let E = Get_FireFlea(k as i32, game);
    (*E).var_00 = ((*E).base.y_pos).wrapping_sub((*E).var_C);
    (*E).var_01 = ((*E).var_C).wrapping_add((*E).base.y_pos);
}
pub unsafe fn Fireflea_Main(game: &mut Game) {
    let E = Get_FireFlea(word!(game.cur_enemy_index) as i32, game);
    if ((*E).parameter_1 & 2) != 0 {
        (*E).base.x_pos = ((*E).var_E).wrapping_add(CosineMult8bit(
            byte!((*E).var_D; 1) as u16,
            (*E).var_C,
            game,
        ));
        (*E).base.y_pos =
            ((*E).var_F).wrapping_add(SineMult8bit(byte!((*E).var_D; 1) as u16, (*E).var_C, game));
        (*E).var_D =
            ((*E).var_D).wrapping_add(*((&raw mut (*E).var_A as *mut u8).offset(1) as *mut u16));
    } else {
        let v2: i32 = (*E).var_02 as i32 >> 1;
        words!(
            (*E).base,
            y_pos,
            y_subpos += words!((
                crate::rtl::rom_table_word!(game, 0xa28187, v2 as isize),
                crate::rtl::rom_table_word!(game, 0xa28187, (v2 + 1) as isize)
            ))
        );
        if (((*E).base.y_pos) as i16).wrapping_sub(((*E).var_00) as i16) < 0
            || (((*E).base.y_pos) as i16).wrapping_sub(((*E).var_01) as i16) >= 0
        {
            (*E).var_02 = ((*E).var_02) ^ (4u16);
        }
    };
}
pub unsafe fn Fireflea_Touch(k: u16, game: &mut Game) {
    let v1: u16 = 0;
    Enemy_NormalTouchAI_A3(game);
    EnemyDeathAnimation(k, v1, game);
    if word!(game.fireflea_darkness_level) as i32 - 12i32 & 0x8000 != 0 {
        word!(game.fireflea_darkness_level, wrapping_add(2));
    }
}
pub unsafe fn Fireflea_Powerbomb(game: &mut Game) {
    Enemy_NormalPowerBombAI_A3(game);
    Fireflea_Common(game);
}
pub unsafe fn Fireflea_Shot(game: &mut Game) {
    Enemy_NormalShotAI_A3(game);
    Fireflea_Common(game);
}
pub unsafe fn Fireflea_Common(game: &mut Game) {
    if (*Get_FireFlea(word!(game.cur_enemy_index) as i32, game))
        .base
        .health
        == 0
        && word!(game.fireflea_darkness_level) as i32 - 12i32 & 0x8000 != 0
    {
        word!(game.fireflea_darkness_level, wrapping_add(2));
    }
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16;
    MaridiaFish_Instr_3 [jp] {
        (*Get_MaridiaFish(word!(game.cur_enemy_index) as i32, game))
            .base
            .layer = 6;
    };
    MaridiaFish_Instr_1 [jp] {
        (*Get_MaridiaFish(word!(game.cur_enemy_index) as i32, game))
            .base
            .layer = 2;
    };
    MaridiaFish_Instr_2 [jp] {
        (*Get_MaridiaFish(word!(game.cur_enemy_index) as i32, game)).var_01 = 1;
    };
}
pub unsafe fn MaridiaFish_Init(game: &mut Game) {
    let E = Get_MaridiaFish(word!(game.cur_enemy_index) as i32, game);
    (*E).base.current_instruction = addr_kMaridiaFish_Ilist_902A as u16;
    (*E).var_A = ((0xa39132i32) as u16);
    if byte!((*E).parameter_1; 1) == 0 {
        (*E).base.current_instruction = addr_kMaridiaFish_Ilist_9060 as u16;
        (*E).var_A = ((0xa391abi32) as u16);
    }
    let v1: i32 = 8 * byte!((*E).parameter_1; 0) as i32 >> 1;
    (*E).var_C = crate::rtl::rom_table_word!(game, 0xa28187, v1 as isize);
    (*E).var_B = crate::rtl::rom_table_word!(game, 0xa28187, (v1 + 1) as isize);
    (*E).var_E = crate::rtl::rom_table_word!(game, 0xa28187, (v1 + 2) as isize);
    (*E).var_D = crate::rtl::rom_table_word!(game, 0xa28187, (v1 + 3) as isize);
    (*E).var_00 = byte!((*E).parameter_2; 0) as u16;
    (*E).var_02 = byte!((*E).parameter_2; 1) as u16;
    set_fields! {[(*E)], 0; var_F, var_01}
    (*E).var_03 = SineMult8bit((*E).var_F, (*E).var_00, game);
}

macro_rules! advance_maridia_fish_sine_motion {
    ($enemy:ident, $game:ident, $collision_state:literal, $collision_instruction:ident) => {
        let v2: u16 = SineMult8bit((*$enemy).var_F, (*$enemy).var_00, $game);
        (*$enemy).var_04 = v2;
        if Enemy_MoveDown(
            word!($game.cur_enemy_index),
            ((((v2) as i16).wrapping_sub(((*$enemy).var_03) as i16)) as i32) << 16,
            $game,
        ) != 0
        {
            (*$enemy).var_A = (($collision_state) as u16);
            (*$enemy).base.instruction_timer = 1;
            (*$enemy).base.timer = 0;
            (*$enemy).base.current_instruction = $collision_instruction as u16;
        } else {
            (*$enemy).var_F = ((byte!((*$enemy).var_02; 0))
                .wrapping_add(((*$enemy).var_F) as u8))
                as u16;
        }
    };
}

pub unsafe fn MaridiaFish_Func_1(game: &mut Game) {
    let E = Get_MaridiaFish(word!(game.cur_enemy_index) as i32, game);
    if Enemy_MoveRight_IgnoreSlopes(
        word!(game.cur_enemy_index),
        (words!((*E), var_E, var_D)) as i32,
        game,
    ) != 0
    {
        (*E).var_A = ((0xa39224i32) as u16);
        (*E).base.instruction_timer = 1;
        (*E).base.timer = 0;
        (*E).base.current_instruction = addr_kMaridiaFish_Ilist_903C as u16;
    } else {
        advance_maridia_fish_sine_motion!(E, game, 0xa39224i32, addr_kMaridiaFish_Ilist_903C);
    }
    (*E).var_03 = (*E).var_04;
}
pub unsafe fn MaridiaFish_Func_2(game: &mut Game) {
    let E = Get_MaridiaFish(word!(game.cur_enemy_index) as i32, game);
    if Enemy_MoveRight_IgnoreSlopes(
        word!(game.cur_enemy_index),
        (words!((*E), var_C, var_B)) as i32,
        game,
    ) != 0
    {
        (*E).var_A = ((0xa39256i32) as u16);
        (*E).base.instruction_timer = 1;
        (*E).base.timer = 0;
        (*E).base.current_instruction = addr_kMaridiaFish_Ilist_9072 as u16;
    } else {
        advance_maridia_fish_sine_motion!(E, game, 0xa39256i32, addr_kMaridiaFish_Ilist_9072);
    }
    (*E).var_03 = (*E).var_04;
}
pub unsafe fn MaridiaFish_Func_3(game: &mut Game) {
    let E = Get_MaridiaFish(word!(game.cur_enemy_index) as i32, game);
    if (*E).var_01 != 0 {
        (*E).var_01 = 0;
        (*E).var_A = ((0xa391abi32) as u16);
        (*E).var_02 = -((*E).var_02 as i32) as u16;
        (*E).base.instruction_timer = 1;
        (*E).base.timer = 0;
        (*E).base.current_instruction = addr_kMaridiaFish_Ilist_9060 as u16;
    }
}
pub unsafe fn MaridiaFish_Func_4(game: &mut Game) {
    let E = Get_MaridiaFish(word!(game.cur_enemy_index) as i32, game);
    if (*E).var_01 != 0 {
        (*E).var_01 = 0;
        (*E).var_A = ((0xa39132i32) as u16);
        (*E).var_02 = -((*E).var_02 as i32) as u16;
        (*E).base.instruction_timer = 1;
        (*E).base.timer = 0;
        (*E).base.current_instruction = addr_kMaridiaFish_Ilist_902A as u16;
    }
}
crate::game::address_dispatch! {
    pub unsafe fn CallMaridiaFishFunc(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        10719538 => MaridiaFish_Func_1; 10719659 => MaridiaFish_Func_2;
        10719780 => MaridiaFish_Func_3; 10719830 => MaridiaFish_Func_4;
    }
}
pub unsafe fn MaridiaFish_Main(game: &mut Game) {
    let E = Get_MaridiaFish(word!(game.cur_enemy_index) as i32, game);
    CallMaridiaFishFunc((((*E).var_A) as u32) | (0xa30000u32), game);
}
pub unsafe fn Elevator_Init(game: &mut Game) {
    let E = Get_Elevator(word!(game.cur_enemy_index) as i32, game);
    (*E).base.spritemap_pointer = addr_kSpritemap_Nothing_A3 as u16;
    (*E).base.instruction_timer = 1;
    (*E).base.timer = 0;
    (*E).base.current_instruction = addr_kElevator_Ilist_94D6 as u16;
    (*E).parameter_1 = ((*E).parameter_1).wrapping_mul(2);
    (*E).var_A = (*E).base.y_pos;
    if word!(game.elevator_status) != 2 {
        crate::update_place! { reference fresh5 = *(ram_ptr!(game, u16; 0xe18)) => 0; }
        word!(game.elevator_flags = *fresh5);
    }
    if words!(game.elevator_status, elevator_flags) != 0 {
        (*E).base.y_pos = (*E).parameter_2;
        Elevator_Func_4(game);
    }
}
static off_A39540: [Option<Func_V>; 4] = callback_table! { unsafe Func_V;
    Elevator_Func_1, Elevator_Func_2, Elevator_Func_3, Elevator_Func3b,
};
pub unsafe fn Elevator_Frozen(game: &mut Game) {
    if word!(game.door_transition_flag_elevator_zebetites) == 0
        && words!(game.elevator_status, elevator_flags) != 0
    {
        off_A39540[word!(game.elevator_status) as usize].expect("non-null function pointer")(game);
    }
}
pub unsafe fn Elevator_Func_1(game: &mut Game) {
    if (crate::rtl::rom_table_word!(
        game,
        0xa394e2,
        ((*Get_Elevator(word!(game.cur_enemy_index) as i32, game)).parameter_1 >> 1) as isize,
    ) & word!(game.joypad1_newkeys))
        != 0
    {
        QueueSfx3_Max6(0xb, game);
        QueueSfx1_Max6(0x32, game);
        CallSomeSamusCode(7, game);
        ResetProjectileData(game);
        Elevator_Func_4(game);
        word!(game.elevator_status, wrapping_add(1));
    } else {
        word!(game.elevator_flags = 0);
    };
}
pub unsafe fn Elevator_Func_2(game: &mut Game) {
    let E = Get_Elevator(word!(game.cur_enemy_index) as i32, game);
    if (*E).parameter_1 != 0 {
        word!(game.elevator_direction = 0x8000);
        words!((*E).base, y_pos, y_subpos += -0x18000i32 as u32);
    } else {
        word!(game.elevator_direction = 0);
        words!((*E).base, y_pos, y_subpos += 0x18000);
    }
    Elevator_Func_4(game);
}
pub unsafe fn Elevator_Func_3(game: &mut Game) {
    word!(game.elevator_status, wrapping_add(1));
    Elevator_Func3b(game);
}
pub unsafe fn Elevator_Func3b(game: &mut Game) {
    let E = Get_Elevator(word!(game.cur_enemy_index) as i32, game);
    if (*E).parameter_1 != 0 {
        words!((*E).base, y_pos, y_subpos += 0x18000);
        if (*E).base.y_pos < (*E).var_A {
            Elevator_Func_4(game);
            return;
        }
    } else {
        words!((*E).base, y_pos, y_subpos += -0x18000i32 as u32);
        if (*E).base.y_pos >= (*E).var_A {
            Elevator_Func_4(game);
            return;
        }
    }
    word! { game, 0; elevator_status, elevator_flags }
    QueueSfx3_Max6(0x25, game);
    (*E).base.y_pos = (*E).var_A;
    CallSomeSamusCode(0xb, game);
    Elevator_Func_4(game);
}
pub unsafe fn Elevator_Func_4(game: &mut Game) {
    let E = Get_Elevator(word!(game.cur_enemy_index) as i32, game);
    word!(game.samus_y_pos = ((*E).base.y_pos).wrapping_sub(26));
    word!(game.samus_y_subpos = 0);
    word!(game.samus_x_pos = (*E).base.x_pos);
    word! { game, 0; samus_y_speed, samus_y_subspeed }
}
pub unsafe fn Crab_Init(game: &mut Game) {
    let v1 = gEnemyData(word!(game.cur_enemy_index) as i32, game);
    (*v1).parameter_2 = 8;
    (*v1).current_instruction = crate::rtl::rom_table_word!(
        game,
        0xa396db,
        ((*v1).current_instruction as i32 & 3) as isize
    );
    StoneZoomer_E67A(word!(game.cur_enemy_index), game);
}
pub unsafe fn Slug_Init(game: &mut Game) {
    let v1 = gEnemyData(word!(game.cur_enemy_index) as i32, game);
    (*v1).parameter_2 = 10;
    (*v1).current_instruction = crate::rtl::rom_table_word!(
        game,
        0xa3992b,
        ((*v1).current_instruction as i32 & 3) as isize
    );
    StoneZoomer_E67A(word!(game.cur_enemy_index), game);
}
crate::instruction_callbacks! { pub unsafe fn (
    k: u16,
    jp: *const u16,
    game: &mut Game,
) -> *const u16;
    PlatformThatFallsWithSamus_Instr_3 [jp] {
        (*Get_PlatformThatFallsWithSamus(word!(game.cur_enemy_index) as i32, game)).var_02 = 0;
    };
    PlatformThatFallsWithSamus_Instr_4 [jp] {
        (*Get_PlatformThatFallsWithSamus(word!(game.cur_enemy_index) as i32, game)).var_02 = 1;
    };
    PlatformThatFallsWithSamus_Instr_1 [jp] {
        (*Get_PlatformThatFallsWithSamus(word!(game.cur_enemy_index) as i32, game)).var_02 = 0;
    };
    PlatformThatFallsWithSamus_Instr_2 [jp] {
        (*Get_PlatformThatFallsWithSamus(word!(game.cur_enemy_index) as i32, game)).var_02 = 1;
    };
}
pub unsafe fn PlatformThatFallsWithSamus_Init(game: &mut Game) {
    let E = Get_PlatformThatFallsWithSamus(word!(game.cur_enemy_index) as i32, game);
    (*E).var_1F = -1i32 as u16;
    let ptfwss_parameter_1: u16 = (*E).parameter_1;
    (*E).var_02 = ptfwss_parameter_1;
    if ptfwss_parameter_1 != 0 {
        PlatformThatFalls_Init(
            word!(game.cur_enemy_index),
            addr_kPlatformThatFallsWithSamus_Ilist_9BFD as u16,
            game,
        );
    } else {
        PlatformThatFalls_Init(
            word!(game.cur_enemy_index),
            addr_kPlatformThatFallsWithSamus_Ilist_9BE7 as u16,
            game,
        );
    };
}
pub unsafe fn FastMovingSlowSinkingPlatform_Init(game: &mut Game) {
    let E = Get_PlatformThatFallsWithSamus(word!(game.cur_enemy_index) as i32, game);
    let ptfwss_parameter_1: u16 = (*E).parameter_1;
    (*E).var_02 = ptfwss_parameter_1;
    if ptfwss_parameter_1 != 0 {
        PlatformThatFalls_Init(
            word!(game.cur_enemy_index),
            addr_kPlatformThatFallsWithSamus_Ilist_9C55 as u16,
            game,
        );
    } else {
        PlatformThatFalls_Init(
            word!(game.cur_enemy_index),
            addr_kPlatformThatFallsWithSamus_Ilist_9C3F as u16,
            game,
        );
    };
}
pub unsafe fn PlatformThatFalls_Init(k: u16, j: u16, game: &mut Game) {
    let E = Get_PlatformThatFallsWithSamus(k as i32, game);
    (*E).base.current_instruction = j;
    let v3: i32 = 8 * byte!((*E).parameter_2; 0) as i32 >> 1;
    (*E).var_C = crate::rtl::rom_table_word!(game, 0xa28187, v3 as isize);
    (*E).var_B = crate::rtl::rom_table_word!(game, 0xa28187, (v3 + 1) as isize);
    (*E).var_E = crate::rtl::rom_table_word!(game, 0xa28187, (v3 + 2) as isize);
    (*E).var_D = crate::rtl::rom_table_word!(game, 0xa28187, (v3 + 3) as isize);
    set_fields! {[(*E)], 0; var_00, var_03, var_04}
    (*E).var_A = ((*E).base.y_pos).wrapping_add(1);
    (*E).var_F = 0;
    (*E).var_05 = byte!((*E).parameter_2; 1) as u16;
}
static off_A39C97: [Option<Func_V>; 2] = callback_table! { unsafe Func_V;
    PlatformThatFallsWithSamus_Func_1, PlatformThatFallsWithSamus_Func_2,
};
static off_A39C9B: [Option<Func_V>; 2] = callback_table! { unsafe Func_V;
    PlatformThatFallsWithSamus_Func_3, PlatformThatFallsWithSamus_Func_4,
};
pub unsafe fn PlatformThatFallsWithSamus_Main(game: &mut Game) {
    let E = Get_PlatformThatFallsWithSamus(word!(game.cur_enemy_index) as i32, game);
    (*E).var_00 = 0;
    if CheckIfEnemyTouchesSamus(word!(game.cur_enemy_index), game) != 0 {
        (*E).var_00 = 1;
    }
    off_A39C97[(*E).var_02 as usize].expect("non-null function pointer")(game);
    let v3: i32 =
        (*Get_PlatformThatFallsWithSamus(word!(game.cur_enemy_index) as i32, game)).var_00 as i32;
    off_A39C9B[v3 as usize].expect("non-null function pointer")(game);
    if (*E).var_00 != (*E).var_06 {
        (*E).var_F = 0;
    }
    (*E).var_06 = (*E).var_00;
}
pub unsafe fn PlatformThatFallsWithSamus_Func_1(game: &mut Game) {
    let E = Get_PlatformThatFallsWithSamus(word!(game.cur_enemy_index) as i32, game);
    (*E).var_01 = (*E).base.x_pos;
    if Enemy_MoveRight_IgnoreSlopes(
        word!(game.cur_enemy_index),
        (words!((*E), var_E, var_D)) as i32,
        game,
    ) != 0
    {
        (*E).var_02 = 1;
        PlatformThatFallsWithSamus_Func_8(game);
    }
}
pub unsafe fn PlatformThatFallsWithSamus_Func_2(game: &mut Game) {
    let E = Get_PlatformThatFallsWithSamus(word!(game.cur_enemy_index) as i32, game);
    (*E).var_01 = (*E).base.x_pos;
    if Enemy_MoveRight_IgnoreSlopes(
        word!(game.cur_enemy_index),
        (words!((*E), var_C, var_B)) as i32,
        game,
    ) != 0
    {
        (*E).var_02 = 0;
        PlatformThatFallsWithSamus_Func_7(game);
    }
}
pub unsafe fn PlatformThatFallsWithSamus_Func_3(game: &mut Game) {
    let E = Get_PlatformThatFallsWithSamus(word!(game.cur_enemy_index) as i32, game);
    let stop_platform: bool = ((((*E).base.y_pos) as i16).wrapping_sub(((*E).var_A) as i16) < 0)
        || ({
            PlatformThatFallsWithSamus_Func_9(game);
            let mut ptfwss_var_05: u16 = 0;
            (*E).var_F = (*E).var_F.wrapping_add(1);
            ptfwss_var_05 = (*E).var_F;
            if ((ptfwss_var_05) as i16).wrapping_sub(((*E).var_05) as i16) >= 0 {
                ptfwss_var_05 = (*E).var_05;
                (*E).var_F = ptfwss_var_05;
            }
            let mut v3: i32 = 0;
            v3 = 8 * ptfwss_var_05 as i32 >> 1;
            (Enemy_MoveDown(
                word!(game.cur_enemy_index),
                (words!((
                    crate::rtl::rom_table_word!(game, 0xa2838f, (v3 + 3) as isize),
                    crate::rtl::rom_table_word!(game, 0xa2838f, (v3 + 2) as isize)
                ))) as i32,
                game,
            ) != 0)
        });
    if stop_platform {
        (*E).var_F = 0;
        PlatformThatFallsWithSamus_Func_10(game);
    };
}
pub unsafe fn PlatformThatFallsWithSamus_Func_4(game: &mut Game) {
    let E = Get_PlatformThatFallsWithSamus(word!(game.cur_enemy_index) as i32, game);
    (*E).var_F = (*E).var_F.wrapping_add(1);
    if (((*E).var_F) as i16).wrapping_sub(((*E).var_05) as i16) >= 0 {
        (*E).var_F = (*E).var_05;
    }
    word!(game.extra_samus_x_displacement, |fresh13| (fresh13 as i32
        + ((*E).base.x_pos as i32 - (*E).var_01 as i32))
        as u16);
    (*E).var_01 = (*E).base.y_pos;
    let v2: i32 = 8 * (*E).var_F as i32 >> 1;
    if Enemy_MoveDown(
        word!(game.cur_enemy_index),
        (words!((
            crate::rtl::rom_table_word!(game, 0xa2838f, (v2 + 1) as isize),
            crate::rtl::rom_table_word!(game, 0xa2838f, v2 as isize)
        ))) as i32,
        game,
    ) != 0
    {
        (*E).var_F = 0;
        PlatformThatFallsWithSamus_Func_10(game);
    }
    word!(game.extra_samus_y_displacement, |fresh14| (fresh14 as i32
        + ((*E).base.y_pos as i32 - (*E).var_01 as i32))
        as u16);
}
pub unsafe fn PlatformThatFallsWithSamus_Func_5(game: &mut Game) {
    let E = Get_PlatformThatFallsWithSamus(word!(game.cur_enemy_index) as i32, game);
    let mut v0: u16 = addr_kPlatformThatFallsWithSamus_Ilist_9C13 as u16;
    if ((*E).var_1F & 0x8000) != 0 {
        v0 = addr_kPlatformThatFallsWithSamus_Ilist_9BBB as u16;
    }
    enemy_instruction! {[(*E).base]; instruction = v0, timer = 1}
    (*E).base.timer = 0;
}
pub unsafe fn PlatformThatFallsWithSamus_Func_6(game: &mut Game) {
    let E = Get_PlatformThatFallsWithSamus(word!(game.cur_enemy_index) as i32, game);
    let mut v0: u16 = addr_kPlatformThatFallsWithSamus_Ilist_9C29 as u16;
    if ((*E).var_1F & 0x8000) != 0 {
        v0 = addr_kPlatformThatFallsWithSamus_Ilist_9BD1 as u16;
    }
    enemy_instruction! {[(*E).base]; instruction = v0, timer = 1}
    (*E).base.timer = 0;
}
pub unsafe fn PlatformThatFallsWithSamus_Func_7(game: &mut Game) {
    let E = Get_PlatformThatFallsWithSamus(word!(game.cur_enemy_index) as i32, game);
    let mut v0: u16 = addr_kPlatformThatFallsWithSamus_Ilist_9C3F as u16;
    if ((*E).var_1F & 0x8000) != 0 {
        v0 = addr_kPlatformThatFallsWithSamus_Ilist_9BE7 as u16;
    }
    enemy_instruction! {[(*E).base]; instruction = v0, timer = 1}
    (*E).base.timer = 0;
}
pub unsafe fn PlatformThatFallsWithSamus_Func_8(game: &mut Game) {
    let E = Get_PlatformThatFallsWithSamus(word!(game.cur_enemy_index) as i32, game);
    let mut v0: u16 = addr_kPlatformThatFallsWithSamus_Ilist_9C55 as u16;
    if ((*E).var_1F & 0x8000) != 0 {
        v0 = addr_kPlatformThatFallsWithSamus_Ilist_9BFD as u16;
    }
    enemy_instruction! {[(*E).base]; instruction = v0, timer = 1}
    (*E).base.timer = 0;
}
pub unsafe fn PlatformThatFallsWithSamus_Func_9(game: &mut Game) {
    let E = Get_PlatformThatFallsWithSamus(word!(game.cur_enemy_index) as i32, game);
    if (*E).var_03 == 0 {
        (*E).var_03 = 1;
        if (*E).var_02 != 0 {
            PlatformThatFallsWithSamus_Func_6(game);
        } else {
            PlatformThatFallsWithSamus_Func_5(game);
        }
    }
    (*E).var_04 = 0;
}
pub unsafe fn PlatformThatFallsWithSamus_Func_10(game: &mut Game) {
    let E = Get_PlatformThatFallsWithSamus(word!(game.cur_enemy_index) as i32, game);
    if (*E).var_04 == 0 {
        (*E).var_04 = 1;
        if (*E).var_02 != 0 {
            PlatformThatFallsWithSamus_Func_8(game);
        } else {
            PlatformThatFallsWithSamus_Func_7(game);
        }
    }
    (*E).var_03 = 0;
}
pub unsafe fn FastMovingSlowSinkingPlatform_Shot(game: &mut Game) {
    Enemy_NormalShotAI_A3(game);
    let E = Get_PlatformThatFallsWithSamus(word!(game.cur_enemy_index) as i32, game);
    if (*E).base.frozen_timer != 0 {
        if (*E).var_02 != 0 {
            (*E).base.spritemap_pointer = addr_kPlatformThatFallsWithSamus_Sprmap_A015 as u16;
        } else {
            (*E).base.spritemap_pointer = addr_kPlatformThatFallsWithSamus_Sprmap_A009 as u16;
        }
    }
}
pub unsafe fn Roach_Func_1(game: &mut Game) {
    let E = Get_Roach(word!(game.cur_enemy_index) as i32, game);
    let roach_var_24: u16 = (*E).var_24;
    if roach_var_24 != (*E).var_25 {
        (*E).var_25 = roach_var_24;
        enemy_instruction! {[(*E).base]; instruction = roach_var_24, timer = 1}
        (*E).base.timer = 0;
    }
}
pub unsafe fn Roach_Init(game: &mut Game) {
    let E = Get_Roach(word!(game.cur_enemy_index) as i32, game);
    set_fields! {[(*E)], 0; var_25, var_24}
    Roach_Func_2(word!(game.cur_enemy_index), game);
    Roach_Func_4(word!(game.cur_enemy_index), game);
    Roach_Func_5(word!(game.cur_enemy_index), game);
    Roach_Func_6(word!(game.cur_enemy_index), game);
    Roach_Func_7(word!(game.cur_enemy_index), game);
    Roach_Func_8(word!(game.cur_enemy_index), game);
    (*E).var_24 = crate::rtl::rom_table_word!(game, 0xa3a111, ((*E).var_20 >> 1) as isize);
    Roach_Func_1(game);
    (*E).var_B = ((0xa3a2d7i32) as u16);
}
pub unsafe fn Roach_Func_2(k: u16, game: &mut Game) {
    let E = Get_Roach(k as i32, game);
    let pt: Point32 = ConvertAngleToXy(
        byte!((*E).parameter_1; 1) as u16,
        byte!((*E).parameter_1; 0) as u16,
        game,
    );
    words!((*E), var_01, var_00 = pt.x);
    words!((*E), var_03, var_02 = pt.y);
}

macro_rules! set_roach_trigonometric_velocity {
    (
        $enemy:ident, $cosine_high:ident, $cosine_low:ident, $angle_adjust:ident, $game:ident,
        $sine_high:ident, $sine_low:ident $(,)?
    ) => {
        words!(
            (*$enemy),
            $cosine_high,
            $cosine_low = CosineMult8bitFull(
                ((byte!((*$enemy).parameter_1; 1isize)).$angle_adjust(32)) as u16,
                byte!((*$enemy).parameter_1; 0) as u16,
                $game,
            )
        );
        words!(
            (*$enemy),
            $sine_high,
            $sine_low = SineMult8bitFull(
                ((byte!((*$enemy).parameter_1; 1isize)).$angle_adjust(32)) as u16,
                byte!((*$enemy).parameter_1; 0) as u16,
                $game,
            )
        );
    };
}

pub unsafe fn Roach_Func_4(k: u16, game: &mut Game) {
    let E = Get_Roach(k as i32, game);
    set_roach_trigonometric_velocity!(E, var_05, var_04, wrapping_sub, game, var_07, var_06);
}
pub unsafe fn Roach_Func_5(k: u16, game: &mut Game) {
    let E = Get_Roach(k as i32, game);
    set_roach_trigonometric_velocity!(E, var_09, var_08, wrapping_add, game, var_0B, var_0A);
}
pub unsafe fn Roach_Func_6(k: u16, game: &mut Game) {
    let E = Get_Roach(k as i32, game);
    (*E).var_20 = (2 * (((byte!((*E).parameter_1; 1isize)).wrapping_sub(48)) as i32 >> 5)) as u16;
}
pub unsafe fn Roach_Func_7(k: u16, game: &mut Game) {
    let E = Get_Roach(k as i32, game);
    (*E).var_21 = (2 * (((byte!((*E).parameter_1; 1isize)).wrapping_sub(80)) as i32 >> 5)) as u16;
}
pub unsafe fn Roach_Func_8(k: u16, game: &mut Game) {
    let E = Get_Roach(k as i32, game);
    (*E).var_22 = (2 * ((byte!((*E).parameter_1; 1) as i32 - 48i32 + 32) as u8 as i32 >> 5)) as u16;
}
crate::game::address_dispatch! {
    pub unsafe fn CallRoachFunc(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        10724055 => Roach_Func_9; 10724097 => Roach_Func_10; 10724107 => Roach_Func_11;
        10724117 => Roach_Func_12; 10724133 => Roach_Func_13; 10724155 => Roach_Func_14;
        10724171 => Roach_Func_15; 10724224 => Roach_Func_16; 10724359 => Roach_Func_19;
        10724366 => Roach_Func_20; 10724416 => Roach_Func_21; 10724423 => Roach_Func_22;
        10724430 => Roach_Func_23; 10724450 => Roach_Func_24; 10724470 => Roach_Func_25;
        10724534 => Roach_Func_26; 10724592 => Roach_Func_27;
    }
}
pub unsafe fn Roach_Main(game: &mut Game) {
    let E = Get_Roach(word!(game.cur_enemy_index) as i32, game);
    CallRoachFunc((((*E).var_B) as u32) | (0xa30000u32), game);
}
pub unsafe fn Roach_Func_9(game: &mut Game) {
    let E = Get_Roach(word!(game.cur_enemy_index) as i32, game);
    if IsSamusWithinEnemy_X(
        word!(game.cur_enemy_index),
        byte!((*E).parameter_2; 0) as u16,
        game,
    ) != 0
        && IsSamusWithinEnemy_Y(
            word!(game.cur_enemy_index),
            byte!((*E).parameter_2; 0) as u16,
            game,
        ) != 0
    {
        (*E).var_B = crate::rtl::rom_table_word!(
            game,
            0xa3a121,
            *(&raw mut (*E).parameter_2 as *mut u8).offset(1) as isize
        );
    }
}
pub unsafe fn Roach_Func_10(game: &mut Game) {
    (*Get_Roach(word!(game.cur_enemy_index) as i32, game)).var_B = ((0xa3a407i32) as u16);
}
pub unsafe fn Roach_Func_11(game: &mut Game) {
    (*Get_Roach(word!(game.cur_enemy_index) as i32, game)).var_B = ((0xa3a40ei32) as u16);
}
pub unsafe fn Roach_Func_12(game: &mut Game) {
    word!(game.random_number = 11);
    (*Get_Roach(word!(game.cur_enemy_index) as i32, game)).var_B = ((0xa3a4f0i32) as u16);
}
pub unsafe fn Roach_Func_13(game: &mut Game) {
    let E = Get_Roach(word!(game.cur_enemy_index) as i32, game);
    (*E).var_A = 512;
    word!(game.random_number = 11);
    (*E).var_B = ((0xa3a4b6i32) as u16);
}
pub unsafe fn Roach_Func_14(game: &mut Game) {
    let E = Get_Roach(word!(game.cur_enemy_index) as i32, game);
    (*E).var_F = 32;
    (*E).var_B = ((0xa3a44ei32) as u16);
}
pub unsafe fn Roach_Func_15(game: &mut Game) {
    let v1: u8 = (64u8)
        .wrapping_sub((CalculateAngleOfSamusFromEnemy(word!(game.cur_enemy_index), game)) as u8);
    let E = Get_Roach(word!(game.cur_enemy_index) as i32, game);
    (*E).var_C = v1 as u16;
    Roach_Func_18(word!(game.cur_enemy_index), game);
    Roach_Func_17(word!(game.cur_enemy_index), game);
    (*E).var_24 = crate::rtl::rom_table_word!(game, 0xa3a111, ((*E).var_23 >> 1) as isize);
    Roach_Func_1(game);
    (*E).var_B = ((0xa3a440i32) as u16);
}
pub unsafe fn Roach_Func_16(game: &mut Game) {
    let v1: u16 = (64 - CalculateAngleOfSamusFromEnemy(word!(game.cur_enemy_index), game) as i32
        + 0x80) as u8 as u16;
    let E = Get_Roach(word!(game.cur_enemy_index) as i32, game);
    (*E).var_C = v1;
    Roach_Func_18(word!(game.cur_enemy_index), game);
    Roach_Func_17(word!(game.cur_enemy_index), game);
    (*E).var_24 = crate::rtl::rom_table_word!(game, 0xa3a111, ((*E).var_23 >> 1) as isize);
    Roach_Func_1(game);
    (*E).var_B = ((0xa3a447i32) as u16);
}
pub unsafe fn Roach_Func_17(k: u16, game: &mut Game) {
    let E = Get_Roach(k as i32, game);
    (*E).var_23 = (2 * (((((*E).var_C) as u8).wrapping_sub(48)) as i32 >> 5)) as u16;
}
pub unsafe fn Roach_Func_18(k: u16, game: &mut Game) {
    let E = Get_Roach(k as i32, game);
    words!(
        (*E),
        var_0D,
        var_0C = CosineMult8bitFull((*E).var_C, byte!((*E).parameter_1; 0) as u16, game)
    );
    words!(
        (*E),
        var_0F,
        var_0E = SineMult8bitFull((*E).var_C, byte!((*E).parameter_1; 0) as u16, game)
    );
}
pub unsafe fn Roach_Func_19(game: &mut Game) {
    Roach_Func_29(word!(game.cur_enemy_index), game);
}
pub unsafe fn Roach_Func_20(game: &mut Game) {
    let E = Get_Roach(word!(game.cur_enemy_index) as i32, game);
    if ((*E).base.frame_counter & 0x10) != 0 {
        (*E).var_24 = crate::rtl::rom_table_word!(game, 0xa3a111, ((*E).var_22 >> 1) as isize);
        Roach_Func_1(game);
        Roach_Func_31(word!(game.cur_enemy_index), game);
    } else {
        (*E).var_24 = crate::rtl::rom_table_word!(game, 0xa3a111, ((*E).var_21 >> 1) as isize);
        Roach_Func_1(game);
        Roach_Func_30(word!(game.cur_enemy_index), game);
    };
}
crate::forward_calls! { pub unsafe fn (game: &mut Game);
    Roach_Func_21 => Roach_Func_32(word!(game.cur_enemy_index), game);
    Roach_Func_22 => Roach_Func_32(word!(game.cur_enemy_index), game);
}
pub unsafe fn Roach_Func_23(game: &mut Game) {
    let E = Get_Roach(word!(game.cur_enemy_index) as i32, game);
    (*E).var_F = (*E).var_F.wrapping_sub(1);
    if ((*E).var_F & 0x8000) != 0 {
        (*E).var_B = ((0xa3a2d7i32) as u16);
    } else {
        Roach_Func_29(word!(game.cur_enemy_index), game);
    };
}
pub unsafe fn Roach_Func_24(game: &mut Game) {
    let E = Get_Roach(word!(game.cur_enemy_index) as i32, game);
    (*E).var_F = (*E).var_F.wrapping_sub(1);
    if ((*E).var_F & 0x8000) != 0 {
        (*E).var_B = ((0xa3a4f0i32) as u16);
    } else {
        Roach_Func_33(word!(game.cur_enemy_index), game);
    };
}
pub unsafe fn Roach_Func_25(game: &mut Game) {
    let E = Get_Roach(word!(game.cur_enemy_index) as i32, game);
    (*E).var_A = (*E).var_A.wrapping_sub(1);
    let mut v2: bool = ((*E).var_A & 0x8000) != 0;
    if !v2 {
        (*E).var_F = (*E).var_F.wrapping_sub(1);
        v2 = ((*E).var_F & 0x8000) != 0;
        if !v2 {
            let v3: u16 = Abs16(((*E).base.x_pos).wrapping_sub(word!(game.samus_x_pos)));
            if v3 as i32 - 96i32 & 0x8000 == 0 {
                let v4: u16 = Abs16(((*E).base.y_pos).wrapping_sub(word!(game.samus_y_pos)));
                if v4 as i32 - 96i32 & 0x8000 == 0 {
                    Roach_Func_28(game);
                }
            }
        } else {
            (*E).var_B = ((0xa3a4b6i32) as u16);
            return;
        }
    }
    Roach_Func_32(word!(game.cur_enemy_index), game);
}
pub unsafe fn Roach_Func_26(game: &mut Game) {
    let v1: u8 = ((NextRandom(game)) as u8).wrapping_sub(64);
    let E = Get_Roach(word!(game.cur_enemy_index) as i32, game);
    (*E).var_C = ((*E).var_C).wrapping_add((v1) as u16);
    Roach_Func_18(word!(game.cur_enemy_index), game);
    Roach_Func_17(word!(game.cur_enemy_index), game);
    (*E).var_24 = crate::rtl::rom_table_word!(game, 0xa3a111, ((*E).var_23 >> 1) as isize);
    Roach_Func_1(game);
    (*E).var_F = 32;
    (*E).var_B = ((0xa3a476i32) as u16);
}
pub unsafe fn Roach_Func_27(game: &mut Game) {
    let v1: u8 = ((NextRandom(game)) as u8).wrapping_sub(64);
    let E = Get_Roach(word!(game.cur_enemy_index) as i32, game);
    (*E).var_C = ((*E).var_C).wrapping_add((v1) as u16);
    Roach_Func_18(word!(game.cur_enemy_index), game);
    Roach_Func_17(word!(game.cur_enemy_index), game);
    (*E).var_24 = crate::rtl::rom_table_word!(game, 0xa3a111, ((*E).var_23 >> 1) as isize);
    Roach_Func_1(game);
    (*E).var_F = 32;
    (*E).var_B = ((0xa3a462i32) as u16);
}
pub unsafe fn Roach_Func_28(game: &mut Game) {
    let E = Get_Roach(word!(game.cur_enemy_index) as i32, game);
    (*E).var_0D = -((*E).var_0D as i32) as u16;
    (*E).var_0C = -((*E).var_0C as i32) as u16;
    (*E).var_0F = -((*E).var_0F as i32) as u16;
    (*E).var_0E = -((*E).var_0E as i32) as u16;
    let v1: u16 = ((*E).var_23 as i32 + 4 & 7) as u16;
    (*E).var_23 = v1;
    (*E).var_24 = crate::rtl::rom_table_word!(game, 0xa3a111, (v1 >> 1) as isize);
    Roach_Func_1(game);
}
pub unsafe fn Roach_Func_29(k: u16, game: &mut Game) {
    let E = Get_Roach(k as i32, game);
    let varE20: u16 = byte!((*E).parameter_1; 1) as u16;
    EnemyFunc_B691(
        varE20,
        Point32 {
            x: words!((*E), var_01, var_00),
            y: words!((*E), var_03, var_02),
        },
        game,
    );
}
pub unsafe fn Roach_Func_30(k: u16, game: &mut Game) {
    let E = Get_Roach(k as i32, game);
    words!((*E).base, x_pos, x_subpos += words!((*E), var_05, var_04));
    words!((*E).base, y_pos, y_subpos += words!((*E), var_07, var_06));
}
pub unsafe fn Roach_Func_31(k: u16, game: &mut Game) {
    let E = Get_Roach(k as i32, game);
    words!((*E).base, x_pos, x_subpos += words!((*E), var_09, var_08));
    words!((*E).base, y_pos, y_subpos += words!((*E), var_0B, var_0A));
}
pub unsafe fn Roach_Func_32(k: u16, game: &mut Game) {
    let E = Get_Roach(k as i32, game);
    words!((*E).base, x_pos, x_subpos += words!((*E), var_0D, var_0C));
    words!((*E).base, y_pos, y_subpos += words!((*E), var_0F, var_0E));
}
pub unsafe fn Roach_Func_33(k: u16, game: &mut Game) {
    let E = Get_Roach(k as i32, game);
    if Enemy_MoveRight_IgnoreSlopes(k, (words!((*E), var_0D, var_0C)) as i32, game) != 0 {
        (*E).var_B = ((0xa3a2d7i32) as u16);
    } else if Enemy_MoveDown(k, (words!((*E), var_0F, var_0E)) as i32, game) != 0 {
        (*E).var_B = ((0xa3a2d7i32) as u16);
    }
}
pub unsafe fn Mochtroid_Init(game: &mut Game) {
    let E = Get_Mochtroid(word!(game.cur_enemy_index) as i32, game);
    (*E).base.layer = 2;
    Mochtroid_Func_4(
        word!(game.cur_enemy_index),
        addr_kMochtroid_Ilist_A745 as u16,
        game,
    );
    (*E).var_F = 0;
}
static off_A3A7A4: [Option<Func_V>; 3] = callback_table! { unsafe Func_V;
    Mochtroid_Func_1, Mochtroid_Func_3, Mochtroid_Func_2,
};
pub unsafe fn Mochtroid_Main(game: &mut Game) {
    let v0: u16 =
        (2u16).wrapping_mul((*Get_Mochtroid(word!(game.cur_enemy_index) as i32, game)).var_F);
    (*Get_Mochtroid(word!(game.cur_enemy_index) as i32, game)).var_F = 0;
    off_A3A7A4[(v0 >> 1) as usize].expect("non-null function pointer")(game);
}

macro_rules! accelerate_and_clamp_mochtroid_axis {
    (
        $enemy:ident, $speed_high:ident, $speed_low:ident, $acceleration:ident,
        $high_field:ident, $clamped_speed:ident, $low_field:ident $(,)?
    ) => {
        words!((*$enemy), $speed_high, $speed_low += -$acceleration as u32);

        let update_vertical_speed: bool = if ((*$enemy).$high_field as i16) < 0 {
            ((*$enemy).$high_field < 0xfffd)
                && ({
                    $clamped_speed = -3i16;
                    true
                })
        } else {
            ((*$enemy).$high_field >= 3)
                && ({
                    $clamped_speed = 3;
                    true
                })
        };
        if update_vertical_speed {
            (*$enemy).$high_field = $clamped_speed as u16;
            (*$enemy).$low_field = 0;
        }
    };
}

pub unsafe fn Mochtroid_Func_1(game: &mut Game) {
    let mut v5: i16 = 0;
    let mut v10: i16 = 0;
    let E = Get_Mochtroid(word!(game.cur_enemy_index) as i32, game);
    let mut t: i32 = ((((((*E).base.y_pos) as i16).wrapping_sub((word!(game.samus_y_pos)) as i16))
        >> 2) as i32)
        << 8;
    accelerate_and_clamp_mochtroid_axis!(E, var_D, var_C, t, var_D, v5, var_C);
    if Enemy_MoveDown(
        word!(game.cur_enemy_index),
        (words!((*E), var_D, var_C)) as i32,
        game,
    ) != 0
    {
        set_fields! {[(*E)], 0; var_C, var_D}
    }
    t = ((((((*E).base.x_pos) as i16).wrapping_sub((word!(game.samus_x_pos)) as i16)) >> 2) as i32)
        << 8;
    accelerate_and_clamp_mochtroid_axis!(E, var_B, var_A, t, var_B, v10, var_A);
    if Enemy_MoveRight_IgnoreSlopes(
        word!(game.cur_enemy_index),
        (words!((*E), var_B, var_A)) as i32,
        game,
    ) != 0
    {
        set_fields! {[(*E)], 0; var_A, var_B}
    }
    Mochtroid_Func_4(
        word!(game.cur_enemy_index),
        addr_kMochtroid_Ilist_A745 as u16,
        game,
    );
}
pub unsafe fn Mochtroid_Func_2(game: &mut Game) {
    let E = Get_Mochtroid(word!(game.cur_enemy_index) as i32, game);
    let v2: i32 = ((*E).var_E as i32 & 6) >> 1;
    (*E).base.x_pos = ((*E).base.x_pos).wrapping_add((g_word_A3A76D[v2 as usize]) as u16);
    (*E).base.y_pos = ((*E).base.y_pos).wrapping_add((g_word_A3A775[v2 as usize]) as u16);
    set_fields! {[(*E)], 0; var_A, var_B, var_C, var_D}
    crate::update_place! { previous fresh12 = (*E).var_E => (*E).var_E.wrapping_sub(1); }
    if fresh12 == 1 {
        (*E).var_F = 0;
    }
    Mochtroid_Func_4(
        word!(game.cur_enemy_index),
        addr_kMochtroid_Ilist_A745 as u16,
        game,
    );
}
pub unsafe fn Mochtroid_Func_3(game: &mut Game) {
    let E = Get_Mochtroid(word!(game.cur_enemy_index) as i32, game);
    let x_pos: u16 = (*E).base.x_pos;
    if x_pos == word!(game.samus_x_pos) {
        set_fields! {[(*E)], 0; var_A, var_B}
    } else if ((x_pos) as i16).wrapping_sub((word!(game.samus_x_pos)) as i16) >= 0 {
        (*E).var_A = 0;
        (*E).var_B = -1i32 as u16;
    } else {
        (*E).var_A = 0;
        (*E).var_B = 1;
    }
    Enemy_MoveRight_IgnoreSlopes(
        word!(game.cur_enemy_index),
        (words!((*E), var_B, var_A)) as i32,
        game,
    );
    let y_pos: u16 = (*E).base.y_pos;
    if y_pos == word!(game.samus_y_pos) {
        set_fields! {[(*E)], 0; var_C, var_D}
    } else if ((y_pos) as i16).wrapping_sub((word!(game.samus_y_pos)) as i16) >= 0 {
        (*E).var_C = 0;
        (*E).var_D = -1i32 as u16;
    } else {
        (*E).var_C = 0;
        (*E).var_D = 1;
    }
    Enemy_MoveDown(
        word!(game.cur_enemy_index),
        (words!((*E), var_D, var_C)) as i32,
        game,
    );
}
pub unsafe fn Mochtroid_Func_4(k: u16, a: u16, game: &mut Game) {
    let E = Get_Mochtroid(k as i32, game);
    if a != (*E).var_01 {
        (*E).var_01 = a;
        enemy_instruction! {[(*E).base]; instruction = a, timer = 1}
        (*E).base.timer = 0;
    }
}
pub unsafe fn Mochtroid_Touch(game: &mut Game) {
    let E = Get_Mochtroid(word!(game.cur_enemy_index) as i32, game);
    (*E).var_F = 1;
    Mochtroid_Func_4(
        word!(game.cur_enemy_index),
        addr_kMochtroid_Ilist_A759 as u16,
        game,
    );
    (*E).var_20 = (*E).var_20.wrapping_add(1);
    let apply_touch_damage: bool = (word!(game.samus_contact_damage_index) != 0)
        || ({
            if (word!(game.random_enemy_counter) & 7) == 7
                && word!(game.samus_health) as i32 - 30i32 & 0x8000 == 0
            {
                QueueSfx3_Max6(0x2d, game);
            }
            ((*E).var_20 as i32 - 80i32 & 0x8000 == 0)
                && ({
                    (*E).var_20 = 0;
                    true
                })
        });
    if apply_touch_damage {
        Enemy_NormalTouchAI_A3(game);
        word! { game, 0; samus_invincibility_timer, samus_knockback_timer }
    };
}
pub unsafe fn Mochtroid_Shot(game: &mut Game) {
    Enemy_NormalShotAI_A3(game);
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16;
    Sidehopper_Func_1 [jp.offset(1)] { QueueSfx2_Max3(*jp, game); };
    Sidehopper_Instr_1 [jp] {
        (*Get_Sidehopper(word!(game.cur_enemy_index) as i32, game)).var_04 = 1;
    };
}
pub unsafe fn Sidehopper_Init(game: &mut Game) {
    word!(game.random_number = 37);
    NextRandom(game);
    let E = Get_Sidehopper(word!(game.cur_enemy_index) as i32, game);
    set_fields! {[(*E)], 0; var_03, var_04, var_05}
    let v2: u16 = (2u16).wrapping_mul((*get_EnemyDef_A2((*E).base.enemy_ptr, game)).field_2A);
    (*E).var_06 = v2;
    let mut v4: u16 = 0;
    if (*E).parameter_1 != 0 {
        v4 = crate::rtl::rom_table_word!(game, 0xa3aaca, ((*E).var_06 >> 1) as isize);
    } else {
        v4 = crate::rtl::rom_table_word!(game, 0xa3aac2, ((*E).var_06 >> 1) as isize);
    }
    (*E).var_00 = v4;
    Sidehopper_Func_3(game);
    if (*get_EnemyDef_A2((*E).base.enemy_ptr, game)).field_2A != 0 {
        (*E).var_05 = 2;
    }
    let v6: i32 = (*E).var_05 as i32 >> 1;
    (*E).var_01 = Sidehopper_Func_2(
        crate::rtl::rom_table_word!(game, 0xa3aaee, v6 as isize),
        crate::rtl::rom_table_word!(game, 0xa3aae6, v6 as isize),
        game,
    );
    let v8: i32 = (*E).var_05 as i32 >> 1;
    (*E).var_02 = Sidehopper_Func_2(
        crate::rtl::rom_table_word!(game, 0xa3aafa, v8 as isize),
        crate::rtl::rom_table_word!(game, 0xa3aaf2, v8 as isize),
        game,
    );
    (*E).var_B = ((0xa3abd6i32) as u16);
}

pub unsafe fn Sidehopper_Func_2(r22: u16, r24: u16, game: &mut Game) -> u16 {
    crate::verify_probe::hit("hop.Sidehopper_Func_2");
    let mut r18: u16 = 0;
    let mut r20: u16 = 0;
    loop {
        r18 = (r18).wrapping_add(r24);
        r20 = (r20).wrapping_add(
            *((game.g_rom.offset(crate::rtl::lorom_offset(0xa2838f)) as *mut u16 as *mut u8)
                .offset((8i32 * r18 as i32) as isize)
                .offset(1) as *mut u16),
        );
        if (r20 as i32 - r22 as i32 & 0x8000 == 0) {
            break;
        }
    }
    r18
}
pub unsafe fn Sidehopper_Func_3(game: &mut Game) {
    let E = Get_Sidehopper(word!(game.cur_enemy_index) as i32, game);
    enemy_instruction! {[(*E).base]; instruction = (*E).var_00, timer = 1}
    (*E).base.timer = 0;
}
pub unsafe fn Sidehopper_Main(game: &mut Game) {
    let E = Get_Sidehopper(word!(game.cur_enemy_index) as i32, game);
    CallSidehopperFunc((((*E).var_B) as u32) | (0xa30000u32), game);
}
pub unsafe fn Sidehopper_Func_4(k: u16, game: &mut Game) {
    let v1: u16 =
        crate::rtl::rom_table_word!(game, 0xa3aae2, (NextRandom(game) as i32 & 1) as isize);
    (*Get_Sidehopper(k as i32, game)).var_B = v1;
}
pub unsafe fn Sidehopper_Func_5(game: &mut Game) {
    let E = Get_Sidehopper(word!(game.cur_enemy_index) as i32, game);
    let v1: i32 = (*E).var_05 as i32 >> 1;
    (*E).var_E = crate::rtl::rom_table_word!(game, 0xa3aae6, v1 as isize);
    (*E).var_D = crate::rtl::rom_table_word!(game, 0xa3aaea, v1 as isize);
    (*E).var_C = (*E).var_01;
    (*E).var_B = ((0xa3ac40i32) as u16);
    if (*E).parameter_1 != 0 {
        (*E).var_B = ((0xa3ac56i32) as u16);
    }
}
pub unsafe fn Sidehopper_Func_6(game: &mut Game) {
    let E = Get_Sidehopper(word!(game.cur_enemy_index) as i32, game);
    let v1: i32 = (*E).var_05 as i32 >> 1;
    (*E).var_E = crate::rtl::rom_table_word!(game, 0xa3aaf2, v1 as isize);
    (*E).var_D = crate::rtl::rom_table_word!(game, 0xa3aaf6, v1 as isize);
    (*E).var_C = (*E).var_02;
    (*E).var_B = ((0xa3ac40i32) as u16);
    if (*E).parameter_1 != 0 {
        (*E).var_B = ((0xa3ac56i32) as u16);
    }
}
pub unsafe fn Sidehopper_Func_7(game: &mut Game) {
    let E = Get_Sidehopper(word!(game.cur_enemy_index) as i32, game);
    (*E).var_B = ((0xa3ac6ci32) as u16);
    if (GetSamusEnemyDelta_X(word!(game.cur_enemy_index), game) & 0x8000) == 0 {
        (*E).var_B = ((0xa3ac8fi32) as u16);
    }
}
pub unsafe fn Sidehopper_Func_8(game: &mut Game) {
    let E = Get_Sidehopper(word!(game.cur_enemy_index) as i32, game);
    (*E).var_B = ((0xa3aca8i32) as u16);
    if (GetSamusEnemyDelta_X(word!(game.cur_enemy_index), game) & 0x8000) == 0 {
        (*E).var_B = ((0xa3accbi32) as u16);
    }
}
pub unsafe fn Sidehopper_Func_9(game: &mut Game) {
    let E = Get_Sidehopper(word!(game.cur_enemy_index) as i32, game);
    (*E).var_D = -((*E).var_D as i32) as u16;
    (*E).var_00 = crate::rtl::rom_table_word!(game, 0xa3aad2, ((*E).var_06 >> 1) as isize);
    Sidehopper_Func_3(game);
    (*E).var_B = ((0xa3ad0ei32) as u16);
}
pub unsafe fn Sidehopper_Func_10(game: &mut Game) {
    let E = Get_Sidehopper(word!(game.cur_enemy_index) as i32, game);
    (*E).var_00 = crate::rtl::rom_table_word!(game, 0xa3aad2, ((*E).var_06 >> 1) as isize);
    Sidehopper_Func_3(game);
    (*E).var_B = ((0xa3ad20i32) as u16);
}
pub unsafe fn Sidehopper_Func_11(game: &mut Game) {
    let E = Get_Sidehopper(word!(game.cur_enemy_index) as i32, game);
    (*E).var_D = -((*E).var_D as i32) as u16;
    (*E).var_00 = crate::rtl::rom_table_word!(game, 0xa3aada, ((*E).var_06 >> 1) as isize);
    Sidehopper_Func_3(game);
    (*E).var_B = ((0xa3ad32i32) as u16);
}
pub unsafe fn Sidehopper_Func_12(game: &mut Game) {
    let E = Get_Sidehopper(word!(game.cur_enemy_index) as i32, game);
    (*E).var_00 = crate::rtl::rom_table_word!(game, 0xa3aada, ((*E).var_06 >> 1) as isize);
    Sidehopper_Func_3(game);
    (*E).var_B = ((0xa3ad44i32) as u16);
}
pub unsafe fn Sidehopper_Func_13(game: &mut Game) {
    let E = Get_Sidehopper(word!(game.cur_enemy_index) as i32, game);
    (*E).var_00 = crate::rtl::rom_table_word!(game, 0xa3aac2, ((*E).var_06 >> 1) as isize);
    if (*E).parameter_1 != 0 {
        (*E).var_00 = crate::rtl::rom_table_word!(game, 0xa3aaca, ((*E).var_06 >> 1) as isize);
    }
    Sidehopper_Func_3(game);
    (*E).var_B = ((0xa3ad56i32) as u16);
}
pub unsafe fn Sidehopper_Func_14(game: &mut Game) {
    if (*Get_Sidehopper(word!(game.cur_enemy_index) as i32, game)).var_03 != 0 {
        Sidehopper_Func_20(game);
    } else {
        Sidehopper_Func_19(game);
    };
}
pub use self::Sidehopper_Func_14 as Sidehopper_Func_15;
pub unsafe fn Sidehopper_Func_16(game: &mut Game) {
    if (*Get_Sidehopper(word!(game.cur_enemy_index) as i32, game)).var_03 != 0 {
        Sidehopper_Func_22(game);
    } else {
        Sidehopper_Func_21(game);
    };
}
pub use self::Sidehopper_Func_16 as Sidehopper_Func_17;
pub unsafe fn Sidehopper_Func_18(game: &mut Game) {
    let E = Get_Sidehopper(word!(game.cur_enemy_index) as i32, game);
    if (*E).var_04 != 0 {
        (*E).var_04 = 0;
        (*E).var_B = ((0xa3abd6i32) as u16);
    }
}
crate::game::address_dispatch! {
    unsafe fn CallSidehopperFunc(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        10726358 => [{
                    Sidehopper_Func_4(word!(game.cur_enemy_index), game);
                }];
        10726374 => Sidehopper_Func_5; 10726419 => Sidehopper_Func_6; 10726464 => Sidehopper_Func_7;
        10726486 => Sidehopper_Func_8; 10726508 => Sidehopper_Func_9;
        10726543 => Sidehopper_Func_10; 10726568 => Sidehopper_Func_11;
        10726603 => Sidehopper_Func_12; 10726628 => Sidehopper_Func_13;
        10726670 => Sidehopper_Func_14; 10726688 => Sidehopper_Func_15;
        10726706 => Sidehopper_Func_16; 10726724 => Sidehopper_Func_17;
        10726742 => Sidehopper_Func_18;
    }
}
// Sidehoppers keep separate SNES state entry points for floor and ceiling hops.
unsafe fn step_sidehopper_jump<const FALLING: bool, const CEILING: bool>(game: &mut Game) {
    let E = Get_Sidehopper(word!(game.cur_enemy_index) as i32, game);
    let index: i32 = (8 * (*E).var_C as i32 >> 1) + if FALLING == CEILING { 2 } else { 0 };
    if Enemy_MoveDown(
        word!(game.cur_enemy_index),
        words!((
            crate::rtl::rom_table_word!(game, 0xa2838f, (index + 1) as isize),
            crate::rtl::rom_table_word!(game, 0xa2838f, index as isize)
        )) as i32,
        game,
    ) != 0
    {
        if FALLING {
            (*E).var_03 = 0;
            (*E).var_B = ((0xa3ace4i32) as u16);
        } else {
            (*E).var_D = -((*E).var_D as i32) as u16;
            (*E).var_03 = 1;
        }
    } else if FALLING {
        if Enemy_MoveRight_IgnoreSlopes(
            word!(game.cur_enemy_index),
            ((*E).var_D as i16 as i32) << 16,
            game,
        ) != 0
        {
            (*E).var_D = -((*E).var_D as i32) as u16;
        }
        let mut next: i16 = ((*E).var_E as i16).wrapping_add((*E).var_C as i16);
        if next as i32 - 64i32 & 0x8000 == 0 {
            next = 64;
        }
        (*E).var_C = next as u16;
    } else if Enemy_MoveRight_IgnoreSlopes(
        word!(game.cur_enemy_index),
        ((*E).var_D as i16 as i32) << 16,
        game,
    ) != 0
    {
        (*E).var_D = -((*E).var_D as i32) as u16;
        (*E).var_03 = 1;
    } else {
        let next = ((*E).var_C as i16).wrapping_sub((*E).var_E as i16);
        (*E).var_C = next as u16;
        if next < 0 {
            (*E).var_03 = 1;
            (*E).var_C = 0;
        }
    }
}
crate::forward_calls! { pub unsafe fn (game: &mut Game);
    Sidehopper_Func_19 => { crate::verify_probe::hit("hop.Sidehopper_Func_19"); step_sidehopper_jump::<false, false>(game) };
    Sidehopper_Func_20 => { crate::verify_probe::hit("hop.Sidehopper_Func_20"); step_sidehopper_jump::<true, false>(game) };
    Sidehopper_Func_21 => { crate::verify_probe::hit("hop.Sidehopper_Func_21"); step_sidehopper_jump::<false, true>(game) };
    Sidehopper_Func_22 => { crate::verify_probe::hit("hop.Sidehopper_Func_22"); step_sidehopper_jump::<true, true>(game) };
}

crate::instruction_callbacks! { pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16;
    MaridiaRefillCandy_Instr_1 [jp] {
        (*Get_MaridiaRefillCandy(word!(game.cur_enemy_index) as i32, game)).var_00 = 4;
    };
    MaridiaRefillCandy_Instr_2 [jp] {
        (*Get_MaridiaRefillCandy(word!(game.cur_enemy_index) as i32, game)).var_00 = 8;
    };
    MaridiaRefillCandy_Instr_3 [jp] {
        (*Get_MaridiaRefillCandy(word!(game.cur_enemy_index) as i32, game)).var_00 = 12;
    };
}
pub unsafe fn MaridiaRefillCandy_Init(game: &mut Game) {
    let E = Get_MaridiaRefillCandy(word!(game.cur_enemy_index) as i32, game);
    (*E).parameter_1 = ((0xa3b482i32) as u16);
    set_fields! {[(*E)], 0; var_D, var_E, var_00}
    (*E).base.current_instruction = addr_kMaridiaRefillCandy_Ilist_B3C1 as u16;
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
    (*E).var_B = (*E).base.x_pos;
    (*E).var_C = (*E).base.y_pos;
}
crate::game::address_dispatch! {
    pub unsafe fn CallMaridiaRefillCandyFunc(ea: u32, k: u16, game: &mut Game) => ea;
        call (k, game); default { Unreachable(game); };
    {
        10728578 => [MaridiaRefillCandy_Func_1(game)]; 10728616 => MaridiaRefillCandy_Func_2;
        10728662 => MaridiaRefillCandy_Func_3;
    }
}
pub unsafe fn MaridiaRefillCandy_Main(game: &mut Game) {
    let E = Get_MaridiaRefillCandy(word!(game.cur_enemy_index) as i32, game);
    CallMaridiaRefillCandyFunc(
        (((*E).parameter_1) as u32) | (0xa30000u32),
        word!(game.cur_enemy_index),
        game,
    );
}
pub unsafe fn MaridiaRefillCandy_Func_1(game: &mut Game) {
    if IsSamusWithinEnemy_X(word!(game.cur_enemy_index), 0x80, game) != 0 {
        let mut v1: i16 = 1;
        if (GetSamusEnemyDelta_X(word!(game.cur_enemy_index), game) & 0x8000) == 0 {
            v1 = 3;
        }
        let E = Get_MaridiaRefillCandy(word!(game.cur_enemy_index) as i32, game);
        (*E).var_D = v1 as u16;
        MaridiaRefillCandy_Func_4(game);
        (*E).parameter_1 = ((0xa3b4a8i32) as u16);
    }
}
pub unsafe fn MaridiaRefillCandy_Func_2(k: u16, game: &mut Game) {
    let E = Get_MaridiaRefillCandy(k as i32, game);
    (*E).base.properties = ((*E).base.properties as i32 & !(kEnemyProps_Invisible as i32)) as u16;
    if (GetSamusEnemyDelta_Y(k, game) & 0x8000) != 0 {
        words!((*E).base, y_pos, y_subpos += -0x8000i32 as u32);
    } else {
        (*E).var_D = (*E).var_D.wrapping_sub(1);
        MaridiaRefillCandy_Func_4(game);
        (*E).var_F = 0;
        (*E).parameter_1 = ((0xa3b4d6i32) as u16);
    };
}
pub unsafe fn MaridiaRefillCandy_Func_3(k: u16, game: &mut Game) {
    let E = Get_MaridiaRefillCandy(k as i32, game);
    let v2: i32 = (*E).var_00 as i32 >> 1;
    let v: i32 = (words!((
        crate::rtl::rom_table_word!(game, 0xa3b415, v2 as isize),
        crate::rtl::rom_table_word!(game, 0xa3b415, (v2 + 1) as isize)
    ))) as i32;
    words!(
        (*E).base,
        x_pos,
        x_subpos += (if (*E).var_D != 0 { v } else { -v }) as u32
    );
    if CheckIfEnemyIsOnScreen(game) == 0 {
        MaridiaRefillCandy_Func_4(game);
        return;
    }
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
    (*E).base.x_pos = (*E).var_B;
    (*E).base.y_pos = (*E).var_C;
    (*E).var_D = 0;
    MaridiaRefillCandy_Func_4(game);
    (*E).parameter_1 = ((0xa3b482i32) as u16);
}
pub unsafe fn MaridiaRefillCandy_Func_4(game: &mut Game) {
    let E = Get_MaridiaRefillCandy(word!(game.cur_enemy_index) as i32, game);
    let mrcy_var_D: u16 = (*E).var_D;
    if mrcy_var_D != (*E).var_E {
        (*E).var_E = mrcy_var_D;
        enemy_instruction! {[(*E).base]; instruction = crate::rtl::rom_table_word!(game, 0xa3b40d, mrcy_var_D as isize), timer = 1}
        (*E).base.timer = 0;
    }
}
pub unsafe fn NorfairSlowFireball_Init(game: &mut Game) {
    let v1 = gEnemyData(word!(game.cur_enemy_index) as i32, game);
    (*v1).parameter_2 = 6;
    (*v1).current_instruction =
        crate::rtl::rom_table_word!(game, 0xa3b667, ((*v1).properties as i32 & 3) as isize);
    StoneZoomer_E67A(word!(game.cur_enemy_index), game);
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16;
    Bang_Instr_1 [jp] { QueueSfx2_Max6(0x56, game); };
    Bang_Instr_2 [jp] { (*Get_Bang(word!(game.cur_enemy_index) as i32, game)).var_22 = 1; };
}
pub unsafe fn Bang_Init(game: &mut Game) {
    let E = Get_Bang(word!(game.cur_enemy_index) as i32, game);
    (*E).var_B = (*E).base.palette_index;
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_BlockPlasmaBeam) as u16);
    (*E).var_F = ((0xa3bca5i32) as u16);
    (*E).var_00 = 16;
    set_fields! {[(*E)], 0; var_01, var_02, var_20, var_21, var_22}
    (*E).var_0B = crate::rtl::rom_table_word!(
        game,
        0xa3ba84,
        *(&raw mut (*E).parameter_2 as *mut u8).offset(0) as isize
    );
    let v1: i32 = 2 * byte!((*E).parameter_2; 1) as i32;
    let v2: u16 = crate::rtl::rom_table_word!(game, 0xa3bc6a, v1 as isize);
    (*E).var_0C = v2;
    (*E).var_0D = v2;
    (*E).var_0E = crate::rtl::rom_table_word!(game, 0xa3bc6a, (v1 + 1) as isize);
    if (*E).base.current_instruction == 0 {
        (*E).var_F = ((0xa3bcc1i32) as u16);
    }
    (*E).base.current_instruction = addr_kBang_Ilist_B75E as u16;
}
pub unsafe fn Bang_Main(game: &mut Game) {
    let E = Get_Bang(word!(game.cur_enemy_index) as i32, game);
    Call((((*E).parameter_1) as u32) | (0xa30000u32), game);
}
pub unsafe fn Bang_Func_1(game: &mut Game) {
    let v0: u16 =
        ((*Get_Bang(word!(game.cur_enemy_index) as i32 + 64i32, game)).var_20).wrapping_add(10);
    (*Get_Bang(word!(game.cur_enemy_index) as i32, game)).var_20 = v0;
    Bang_Func_18(game);
    let E = Get_Bang(word!(game.cur_enemy_index) as i32, game);
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
}
pub unsafe fn Bang_Func_2(game: &mut Game) {
    let v0: u16 =
        ((*Get_Bang(word!(game.cur_enemy_index) as i32 + 1984i32, game)).var_00).wrapping_add(20);
    let E = Get_Bang(word!(game.cur_enemy_index) as i32, game);
    (*E).var_20 = v0;
    Bang_Func_18(game);
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
}
pub unsafe fn Bang_Func_3(game: &mut Game) {
    Bang_Func_5(game);
    let E0 = Get_Bang(word!(game.cur_enemy_index) as i32, game);
    let x_pos: u16 = (*E0).base.x_pos;
    let v3: i32 = word!(game.cur_enemy_index) as i32 >> 1;
    word!(game.enemy_drawing_queue[(v3 + 91)] = x_pos);
    let E1 = Get_Bang(word!(game.cur_enemy_index) as i32 + 64, game);
    (*E1).base.x_pos = x_pos;
    let y_pos: u16 = (*E0).base.y_pos;
    word!(game.enemy_drawing_queue[(v3 + 93)] = y_pos);
    (*E1).base.y_pos = y_pos;
    let mut bang_var_B: i16 = (*E0).var_B as i16;
    if ((*E0).var_20 & 1) != 0 {
        bang_var_B = 3072;
    }
    (*E0).base.palette_index = bang_var_B as u16;
    Bang_Func_18(game);
    if (*E0).var_22 != 0 {
        (*E0).var_22 = 0;
        if (*E0).var_20 == 9 {
            (*E0).base.invincibility_timer = 16;
            (*E0).base.properties = ((*E0).base.properties) | ((kEnemyProps_Tangible) as u16);
            let v7: u16 = DetermineDirectionOfSamusFromEnemy(game);
            let v8: u16 = Bang_Func_4(v7, game);
            EnemyDeathAnimation(word!(game.cur_enemy_index), v8, game);
            let v9: u16 = word!(game.cur_enemy_index);
            let E = Get_Bang(word!(game.cur_enemy_index) as i32 + 64, game);
            (*E).base.properties = ((*E).base.properties) | (0x200u16);
            word!(
                game.enemy_drawing_queue[((v9 as i32 >> 1) + 97)],
                |fresh1| (fresh1) | (0x200u16)
            );
        } else {
            (*E0).var_20 = (*E0).var_20.wrapping_add(1);
            Bang_Func_18(game);
        }
    }
}
pub unsafe fn Bang_Func_4(a: u16, game: &mut Game) -> u16 {
    if word!(game.projectile_counter) as i32 - 5i32 & 0x8000 == 0 {
        return 1;
    }
    let mut i: i32 = 0;
    while word!(game.projectile_damage[(i >> 1)]) != 0 {
        i += 2;
    }
    let E = Get_Bang(word!(game.cur_enemy_index) as i32, game);
    let v3: i32 = i >> 1;
    word!(game.projectile_x_pos[v3] = (*E).base.x_pos);
    word!(game.projectile_y_pos[v3] = (*E).base.y_pos);
    word!(game.projectile_dir[v3] = a);
    word!(game.projectile_type[v3] = (word!(game.equipped_beams) & 0xf | 0x10));
    word!(game.projectile_counter, wrapping_add(1));
    ProjectileReflection(i as u16, game);
    word!(game.projectile_damage[v3] = (*E).var_E);
    QueueSfx1_Max6(
        crate::rtl::rom_table_word!(
            game,
            0xa3bc4a,
            (word!(game.projectile_type[v3]) as i32 & 0xf) as isize
        ),
        game,
    );
    0
}
pub unsafe fn Bang_Func_5(game: &mut Game) {
    let E = Get_Bang(word!(game.cur_enemy_index) as i32, game);
    CallEnemyPreInstr((((*E).var_F) as u32) | (0xa30000u32), game);
}
pub unsafe fn Bang_Func_6(k: u16, game: &mut Game) {
    let E = Get_Bang(word!(game.cur_enemy_index) as i32, game);
    let v1: u16 = ((*E).var_00).wrapping_sub(1);
    (*E).var_00 = v1;
    if v1 == 0 {
        (*E).var_00 = 16;
        (*E).var_F = ((0xa3bcc5i32) as u16);
    }
}
pub fn Bang_Func_7(k: u16) {}
pub unsafe fn Bang_Func_8(k: u16, game: &mut Game) {
    let v1: u8 = ((CalculateAngleOfSamusFromEnemy(word!(game.cur_enemy_index), game)) as u8)
        .wrapping_sub(64);
    let E = Get_Bang(word!(game.cur_enemy_index) as i32, game);
    (*E).var_01 = v1 as u16;
    (*E).var_F = ((0xa3bd1ci32) as u16);
    set_fields! {[(*E)], 0; var_07, var_08, var_09, var_0A}
}
pub unsafe fn Bang_Func_9(game: &mut Game) {
    let v1: u8 = ((CalculateAngleOfSamusFromEnemy(word!(game.cur_enemy_index), game)) as u8)
        .wrapping_sub(64);
    let E = Get_Bang(word!(game.cur_enemy_index) as i32, game);
    (*E).var_02 = v1 as u16;
    let v3: u16 = SignExtend8(((v1) as u16).wrapping_sub((*E).var_01));
    let v4: u16 = Abs16(v3);
    if v4 as i32 - 48i32 & 0x8000 == 0 {
        (*E).var_F = ((0xa3bd2ci32) as u16);
    }
}
pub unsafe fn Bang_Func_10(k: u16, game: &mut Game) {
    Bang_Func_14(game);
    Bang_Func_15(game);
    Bang_Func_12(game);
    Bang_Func_9(game);
}
pub unsafe fn Bang_Func_11(k: u16, game: &mut Game) {
    Bang_Func_14(game);
    Bang_Func_15(game);
    Bang_Func_13(game);
    let E = Get_Bang(word!(game.cur_enemy_index) as i32, game);
    if (*E).var_0A != 0 || ((*E).var_09 as i16) <= 0 {
        (*E).var_F = ((0xa3bca5i32) as u16);
    }
}
pub unsafe fn Bang_Func_12(game: &mut Game) {
    let E = Get_Bang(word!(game.cur_enemy_index) as i32, game);
    let v1: i16 = (((*E).var_0C) as i16).wrapping_sub(1);
    (*E).var_0C = v1 as u16;
    if v1 < 0 {
        (*E).var_0C = (*E).var_0D;
        if (((*E).var_0A) as i16).wrapping_sub(((*E).var_0B) as i16) < 0 {
            (*E).var_09 = ((*E).var_09).wrapping_add(22);
            (*E).var_0A = ((*E).var_0A).wrapping_add((*E).var_09);
        }
    }
}
pub unsafe fn Bang_Func_13(game: &mut Game) {
    let E = Get_Bang(word!(game.cur_enemy_index) as i32, game);
    let v1: i16 = (((*E).var_0C) as i16).wrapping_sub(1);
    (*E).var_0C = v1 as u16;
    if v1 < 0 {
        (*E).var_0C = (*E).var_0E;
        (*E).var_09 = ((*E).var_09).wrapping_sub(22);
        (*E).var_0A = ((*E).var_0A).wrapping_sub((*E).var_09);
    }
}
pub unsafe fn Bang_Func_14(game: &mut Game) {
    let E = Get_Bang(word!(game.cur_enemy_index) as i32, game);
    let r18: u16 = crate::rtl::rom_table_word!(
        game,
        0xa0b1c3,
        ((((*E).var_01) as u8).wrapping_add(64)) as isize
    );
    if (r18 & 0x8000) != 0 {
        (*E).var_07 = 1;
    }
    let mut t: u32 = ((Abs16(r18) as i32 >> 8) * (*E).var_0A as i32) as u32;
    if (*E).var_07 != 0 {
        t = -(t as i32) as u32;
    }
    t = t.wrapping_add((*E).base.x_subpos as u32);
    (*E).base.x_subpos = t as u16;
    (*E).base.x_pos = (*E).base.x_pos.wrapping_add((t >> 16) as u16);
}
pub unsafe fn Bang_Func_15(game: &mut Game) {
    let E = Get_Bang(word!(game.cur_enemy_index) as i32, game);
    let r18: u16 = crate::rtl::rom_table_word!(game, 0xa0b1c3, (*E).var_01 as u8 as isize);
    if (r18 & 0x8000) != 0 {
        (*E).var_08 = 1;
    }
    let mut t: u32 = ((Abs16(r18) as i32 >> 8) * (*E).var_0A as i32) as u32;
    if (*E).var_08 != 0 {
        t = -(t as i32) as u32;
    }
    t = t.wrapping_add((*E).base.y_subpos as u32);
    (*E).base.y_subpos = t as u16;
    (*E).base.y_pos = (*E).base.y_pos.wrapping_add((t >> 16) as u16);
}
pub unsafe fn Bang_Func_18(game: &mut Game) {
    let E = Get_Bang(word!(game.cur_enemy_index) as i32, game);
    let bang_var_20: u16 = (*E).var_20;
    if bang_var_20 != (*E).var_21 {
        (*E).var_21 = bang_var_20;
        enemy_instruction! {[(*E).base]; instruction = crate::rtl::rom_table_word!(game, 0xa3b722, bang_var_20 as isize), timer = 1}
        (*E).base.timer = 0;
    }
}
pub unsafe fn Bang_Shot(game: &mut Game) {
    let E = Get_Bang(word!(game.cur_enemy_index) as i32, game);
    if (*E).var_F != 0xbcc1 {
        (*E).var_01 = crate::rtl::rom_table_word!(
            game,
            0xa3ba94,
            (word!(game.projectile_dir[word!(game.collision_detection_index)]) as i32 & 0xf)
                as isize,
        );
        (*E).var_F = ((0xa3bd2ci32) as u16);
        set_fields! {[(*E)], 0; var_07, var_08}
        (*E).var_09 = 256;
        (*E).var_0A = 1536;
    }
    if (*E).var_20 != 9 {
        (*E).var_20 = (*E).var_20.wrapping_add(1);
        Bang_Func_18(game);
        let v1: i32 = word!(game.collision_detection_index) as i32;
        (*E).var_E = ((*E).var_E).wrapping_add(word!(game.projectile_damage[v1]));
        word!(game.projectile_dir[v1], |fresh2| (fresh2) | (0x10u16));
        if (*E).var_20 == 9 {
            (*E).var_D = 1;
        }
    }
}
pub unsafe fn Skree_Instr_1(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_Skree(word!(game.cur_enemy_index) as i32, game)).var_E = 1;
    jp
}
pub unsafe fn Skree_Init(game: &mut Game) {
    let E = Get_Skree(word!(game.cur_enemy_index) as i32, game);
    set_fields! {[(*E)], 0; var_C, var_D, var_E}
    (*E).base.current_instruction = addr_kSkree_Ilist_C65E as u16;
    (*E).var_B = ((0xa3c6d5i32) as u16);
}
crate::game::address_dispatch! {
    pub unsafe fn CallSkreeFunc(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        10733269 => Skree_Func_1;
        10733303 => [{
                    Skree_Func_2(word!(game.cur_enemy_index), game);
                }];
        10733334 => Skree_Func_3; 10733439 => Skree_Func_4;
    }
}
pub unsafe fn Skree_Main(game: &mut Game) {
    let E = Get_Skree(word!(game.cur_enemy_index) as i32, game);
    CallSkreeFunc((((*E).var_B) as u32) | (0xa30000u32), game);
}
pub unsafe fn Skree_Func_1(game: &mut Game) {
    let E = Get_Skree(word!(game.cur_enemy_index) as i32, game);
    if abs16(((*E).base.x_pos).wrapping_sub(word!(game.samus_x_pos))) < 0x30 {
        (*E).var_C = (*E).var_C.wrapping_add(1);
        Skree_Func_5(game);
        (*E).var_B = ((0xa3c6f7i32) as u16);
    }
}
pub unsafe fn Skree_Func_2(k: u16, game: &mut Game) {
    let E = Get_Skree(k as i32, game);
    if (*E).var_E != 0 {
        (*E).var_E = 0;
        (*E).var_C = (*E).var_C.wrapping_add(1);
        Skree_Func_5(game);
        (*E).var_B = ((0xa3c716i32) as u16);
        QueueSfx2_Max6(0x5b, game);
    }
}
pub unsafe fn Skree_Func_3(game: &mut Game) {
    let E = Get_Skree(word!(game.cur_enemy_index) as i32, game);
    (*E).var_A = 21;
    let v1: u16 = ((*E).base.properties) | (3u16);
    (*E).base.properties = v1;
    if EnemyFunc_BF8A(word!(game.cur_enemy_index), v1, (6i16 as i32) << 16, game) & 1 != 0 {
        (*E).base.instruction_timer = 1;
        (*E).base.timer = 0;
        (*E).var_B = ((0xa3c77fi32) as u16);
        QueueSfx2_Max6(0x5c, game);
    } else {
        (*E).base.y_pos = ((*E).base.y_pos).wrapping_add(6);
        (*E).base.x_pos = ((*E).base.x_pos as i32
            + if (((*E).base.x_pos) as i16).wrapping_sub((word!(game.samus_x_pos)) as i16) >= 0 {
                -1i32
            } else {
                1
            }) as u16;
    };
}
pub unsafe fn Skree_Func_4(game: &mut Game) {
    let E = Get_Skree(word!(game.cur_enemy_index) as i32, game);
    crate::update_place! { previous fresh18 = (*E).var_A => (*E).var_A.wrapping_sub(1); }
    if fresh18 == 1 {
        (*gEnemySpawnData(word!(game.cur_enemy_index) as i32, game)).vram_tiles_index =
            ((*E).base.vram_tiles_index) | ((*E).base.palette_index);
        (*E).base.palette_index = 2560;
        (*E).base.vram_tiles_index = 0;
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
    } else {
        if (*E).var_A == 8 {
            let v2: u16 = word!(game.cur_enemy_index);
            let mut parameter = SpawnEprojWithGfx(
                8,
                word!(game.cur_enemy_index),
                addr_stru_868BC2 as u16,
                game,
            );
            parameter = SpawnEprojWithGfx(parameter, v2, addr_stru_868BD0 as u16, game);
            parameter = SpawnEprojWithGfx(parameter, v2, addr_stru_868BDE as u16, game);
            SpawnEprojWithGfx(parameter, v2, addr_stru_868BEC as u16, game);
        }
        (*E).base.y_pos = (*E).base.y_pos.wrapping_add(1);
    };
}
pub unsafe fn Skree_Func_5(game: &mut Game) {
    let E = Get_Skree(word!(game.cur_enemy_index) as i32, game);
    let skree_var_C: u16 = (*E).var_C;
    if skree_var_C != (*E).var_D {
        (*E).var_D = skree_var_C;
        enemy_instruction! {[(*E).base]; instruction = crate::rtl::rom_table_word!(game, 0xa3c69c, skree_var_C as isize), timer = 1}
        (*E).base.timer = 0;
    }
}
pub unsafe fn Skree_Shot(game: &mut Game) {
    Enemy_NormalShotAI_SkipSomeParts_A3(game);
    let E = Get_Skree(word!(game.cur_enemy_index) as i32, game);
    if (*E).base.health == 0 {
        let v1: u16 = word!(game.cur_enemy_index);
        let mut parameter = SpawnEprojWithGfx(
            (*E).var_A,
            word!(game.cur_enemy_index),
            addr_stru_868BC2 as u16,
            game,
        );
        parameter = SpawnEprojWithGfx(parameter, v1, addr_stru_868BD0 as u16, game);
        parameter = SpawnEprojWithGfx(parameter, v1, addr_stru_868BDE as u16, game);
        SpawnEprojWithGfx(parameter, v1, addr_stru_868BEC as u16, game);
        let mut v5: u16 = 2;
        if (word!(game.projectile_type[word!(game.collision_detection_index)]) & 0xf00) != 512 {
            v5 = 0;
        }
        EnemyDeathAnimation(
            (2u16).wrapping_mul(word!(game.collision_detection_index)),
            v5,
            game,
        );
    }
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16;
    MaridiaSnail_Instr_1 [jp.offset(1)] { (*Get_MaridiaSnail(k as i32, game)).var_F = *jp; };
    MaridiaSnail_Instr_2 [jp.offset(1)] { (*Get_MaridiaSnail(k as i32, game)).var_D = *jp; };
    MaridiaSnail_Instr_4 [jp.offset(1)] {
        let E = Get_MaridiaSnail(k as i32, game);
        (*E).var_07 = *jp.offset(0);
        (*E).var_C = *((&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa3cd42))
            as *mut MaridiaSnailData2)
            .offset(0))
        .field_6 as *mut u8)
            .offset((8 * (*E).var_07 as i32) as isize) as *mut u16);
    };
    MaridiaSnail_Instr_3 [jp.offset(2)] {
        let E = Get_MaridiaSnail(k as i32, game);
        (*E).base.x_pos = ((*E).base.x_pos).wrapping_add(*jp.offset(0));
        (*E).base.y_pos = ((*E).base.y_pos).wrapping_add(*jp.offset(1));
    };
}
pub unsafe fn MaridiaSnail_Instr_5(k: u16, mut jp: *const u16, game: &mut Game) -> *const u16 {
    if (*Get_MaridiaSnail(k as i32, game)).var_08 == 2 || (NextRandom(game) & 1) != 0 {
        jp = jp.offset(-(3i32 as isize));
    }
    jp
}
pub unsafe fn MaridiaSnail_Init(game: &mut Game) {
    let E = Get_MaridiaSnail(word!(game.cur_enemy_index) as i32, game);
    (*E).var_F = ((0xa3cf5fi32) as u16);
    (*E).base.spritemap_pointer = addr_kSpritemap_Nothing_A3 as u16;
    (*E).base.instruction_timer = 1;
    crate::update_place! { previous current_instruction: u16 = (*E).base.current_instruction => (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa3cd42))
        as *mut MaridiaSnailData2)
        .offset(current_instruction as isize))
    .field_0; }
    (*E).base.properties = ((*E).base.properties)
        | ((*(game.g_rom.offset(crate::rtl::lorom_offset(0xa3cd42)) as *mut MaridiaSnailData2)
            .offset(current_instruction as isize))
        .field_2);
    (*E).var_D = (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa3cd42))
        as *mut MaridiaSnailData2)
        .offset(current_instruction as isize))
    .field_4;
    (*E).var_C = (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa3cd42))
        as *mut MaridiaSnailData2)
        .offset(current_instruction as isize))
    .field_6;
    (*E).var_08 = 0;
    (*E).var_06 = (*E).parameter_1;
    MaridiaSnail_Func_2(
        word!(game.cur_enemy_index),
        (current_instruction).wrapping_mul(8),
        game,
    );
}
pub unsafe fn MaridiaSnail_Func_2(k: u16, j: u16, game: &mut Game) {
    let v2: i32 = j as i32 >> 1;
    let r18: u16 = crate::rtl::rom_table_word!(game, 0xa3cd82, v2 as isize);
    let r20: u16 = crate::rtl::rom_table_word!(game, 0xa3cd82, (v2 + 1) as isize);
    let r22: u16 = crate::rtl::rom_table_word!(game, 0xa3cd82, (v2 + 2) as isize);
    let r24: u16 = crate::rtl::rom_table_word!(game, 0xa3cd82, (v2 + 3) as isize);
    let E = Get_MaridiaSnail(k as i32, game);
    (*E).var_A = (r20 as i32
        + (r18 as i32
            ^ crate::rtl::rom_table_word!(game, 0xa3cca2, (*E).parameter_1 as isize) as i32))
        as u16;
    (*E).var_B = (r24 as i32
        + (r22 as i32
            ^ crate::rtl::rom_table_word!(game, 0xa3cca2, (*E).parameter_1 as isize) as i32))
        as u16;
}
pub unsafe fn MaridiaSnail_Func_3(k: u16, game: &mut Game) {
    let E = Get_MaridiaSnail(k as i32, game);
    (*E).var_F = crate::rtl::rom_table_word!(game, 0xa3cdd2, (*E).var_07 as isize);
}
pub unsafe fn MaridiaSnail_Main(game: &mut Game) {
    MaridiaSnail_Func_4(word!(game.cur_enemy_index), game);
    MaridiaSnail_Func_5(word!(game.cur_enemy_index), game);
    MaridiaSnail_Func_6(word!(game.cur_enemy_index), game);
    let E = Get_MaridiaSnail(word!(game.cur_enemy_index) as i32, game);
    CallEnemyPreInstr((((*E).var_F) as u32) | (0xa30000u32), game);
}
pub unsafe fn MaridiaSnail_Func_4(k: u16, game: &mut Game) {
    let msl_var_08: u16 = (*Get_MaridiaSnail(k as i32, game)).var_08;
    if msl_var_08 != 3
        && msl_var_08 != 4
        && msl_var_08 != 5
        && word!(game.earthquake_timer) == 30
        && word!(game.earthquake_type) == 20
    {
        MaridiaSnail_Func_14(k, game);
    }
}
pub unsafe fn MaridiaSnail_Func_5(k: u16, game: &mut Game) {
    let skip_instruction_change: bool;
    let E = Get_MaridiaSnail(k as i32, game);
    let msl_var_08: u16 = (*E).var_08;
    if msl_var_08 != 1 && msl_var_08 != 3 && msl_var_08 != 4 && msl_var_08 != 5 {
        if (*E).base.y_pos as i32 - word!(game.samus_y_pos) as i32 + 96 & 0x8000 == 0 {
            skip_instruction_change =
                if (((*E).base.x_pos) as i16).wrapping_sub((word!(game.samus_x_pos)) as i16) < 0 {
                    (*ram_ptr!(game; 0xa1e) != 4)
                } else {
                    (*ram_ptr!(game; 0xa1e) != 8)
                };
            if skip_instruction_change {
            } else {
                if (*E).var_08 == 2 {
                    return;
                }
                if (*E).var_D as i32 != 0xa3cf5f & 0xffff {
                    (*E).base.current_instruction = (*E).var_D;
                    (*E).base.spritemap_pointer = addr_kSpritemap_Nothing_A3 as u16;
                    (*E).base.instruction_timer = 1;
                    (*E).base.timer = 0;
                    (*E).var_08 = 2;
                    return;
                }
            }
        }
        (*E).var_08 = 0;
    }
}
pub unsafe fn MaridiaSnail_Func_6(k: u16, game: &mut Game) {
    let keep_properties: bool;
    let E = Get_MaridiaSnail(k as i32, game);
    if ((*E).var_F as i32 != 0xa3d1b3 & 0xffff) {
        keep_properties = if (*E).var_08 == 1 {
            (word!(game.samus_has_momentum_flag) != 0 || (*E).var_F as i32 != 0xa3cf5f & 0xffff)
        } else {
            !(word!(game.samus_has_momentum_flag) == 0)
                || ({
                    let msl_var_08: u16 = (*E).var_08;
                    !(msl_var_08 == 2 || msl_var_08 != 3 && msl_var_08 != 5)
                })
        };
        if keep_properties {
        } else {
            (*E).base.properties = ((*E).base.properties) | (0x8000u16);
            return;
        }
    }
    (*E).base.properties = ((*E).base.properties) & (0x7fffu16);
}
unsafe fn Shift8AddMagn(a: i16, s: i32) -> u32 {
    let mut r: i32 = (a as i32) << 8;
    r = r + if r < 0 { -(s << 16i32) } else { s << 16 };
    r as u32
}
pub unsafe fn MaridiaSnail_Func_7(k: u16, game: &mut Game) {
    let E = Get_MaridiaSnail(k as i32, game);
    if (*E).var_07 >= 4 {
        if EnemyFunc_BC76(k, Shift8AddMagn((*E).var_B as i16, 7) as i32, game) != 0 {
            return;
        }
    } else if EnemyFunc_BBBF(k, Shift8AddMagn((*E).var_A as i16, 7) as i32, game) != 0 {
        return;
    }
    MaridiaSnail_Func_14(k, game);
}
pub unsafe fn MaridiaSnail_Func_9(k: u16, game: &mut Game) {
    let v1: u16 = (if (*Get_MaridiaSnail(k as i32, game)).var_05 != 0 {
        addr_stru_A3CD22 as i32
    } else {
        addr_stru_A3CCE2 as i32
    }) as u16;
    MaridiaSnail_D07E(k, RomPtr_A3(v1, game), game);
}
pub unsafe fn MaridiaSnail_CFB7(k: u16, game: &mut Game) {
    MaridiaSnail_D002(k, RomPtr_A3(addr_stru_A3CCEA as u16, game), game);
}
pub unsafe fn MaridiaSnail_CFBD(k: u16, game: &mut Game) {
    let v1: u16 = (if (*Get_MaridiaSnail(k as i32, game)).var_05 != 0 {
        addr_stru_A3CD2A as i32
    } else {
        addr_stru_A3CCF2 as i32
    }) as u16;
    MaridiaSnail_D07E(k, RomPtr_A3(v1, game), game);
}
pub unsafe fn MaridiaSnail_CFCE(k: u16, game: &mut Game) {
    MaridiaSnail_D002(k, RomPtr_A3(addr_stru_A3CCFA as u16, game), game);
}
pub unsafe fn MaridiaSnail_CFD4(k: u16, game: &mut Game) {
    let v1: u16 = (if (*Get_MaridiaSnail(k as i32, game)).var_05 != 0 {
        addr_stru_A3CD32 as i32
    } else {
        addr_stru_A3CD02 as i32
    }) as u16;
    MaridiaSnail_D07E(k, RomPtr_A3(v1, game), game);
}
pub unsafe fn MaridiaSnail_CFE5(k: u16, game: &mut Game) {
    MaridiaSnail_D002(k, RomPtr_A3(addr_stru_A3CD0A as u16, game), game);
}
pub unsafe fn MaridiaSnail_CFEB(k: u16, game: &mut Game) {
    let mut v1: u16 = 0;
    if (*Get_MaridiaSnail(k as i32, game)).var_05 != 0 {
        v1 = addr_stru_A3CD3A as u16;
    } else {
        v1 = addr_stru_A3CD12 as u16;
    }
    MaridiaSnail_D07E(k, RomPtr_A3(v1, game), game);
}
pub unsafe fn MaridiaSnail_CFFC(k: u16, game: &mut Game) {
    MaridiaSnail_D002(k, RomPtr_A3(addr_stru_A3CD1A as u16, game), game);
}
pub unsafe fn MaridiaSnail_D002(k: u16, j: *const u8, game: &mut Game) {
    MaridiaSnail_Func_10(k, j, game);
    let E = Get_MaridiaSnail(k as i32, game);
    let rv: u8 = Enemy_MoveRight_ProcessSlopes(k, Shift8AddMagn((*E).var_A as i16, 1) as i32, game);
    MaridiaSnail_Func_11(k, j, game);
    if rv != 0 {
        (*E).var_E = 0;
        let carry: u8 = EnemyFunc_C8AD(k, game);
        MaridiaSnail_Func_12(k, carry as u16, game);
        if Enemy_MoveDown(k, ((*E).var_B as i16 as i32) << 8, game) != 0 {
            (*E).var_A = -((*E).var_A as i32) as u16;
            MaridiaSnail_Func_13(k, *(j.offset(6) as *mut u16), game);
        }
    } else {
        let v7: u16 = ((*E).var_E).wrapping_add(1);
        (*E).var_E = v7;
        if v7 as i32 - 4i32 & 0x8000 != 0 {
            (*E).var_B = -((*E).var_B as i32) as u16;
            MaridiaSnail_Func_13(k, *(j.offset(4) as *mut u16), game);
        } else {
            MaridiaSnail_Func_14(k, game);
        }
    };
}
pub unsafe fn MaridiaSnail_D07E(k: u16, j: *const u8, game: &mut Game) {
    MaridiaSnail_Func_10(k, j, game);
    let E = Get_MaridiaSnail(k as i32, game);
    let rv: u8 = Enemy_MoveDown(k, Shift8AddMagn((*E).var_B as i16, 1) as i32, game);
    MaridiaSnail_Func_11(k, j, game);
    if rv != 0 {
        (*E).var_E = 0;
        if Enemy_MoveRight_ProcessSlopes(k, ((*E).var_A as i16 as i32) << 8, game) != 0 {
            (*E).var_B = -((*E).var_B as i32) as u16;
            MaridiaSnail_Func_13(k, *(j.offset(6) as *mut u16), game);
        } else {
            let carry: u8 = EnemyFunc_C8AD(k, game);
            MaridiaSnail_Func_12(k, carry as u16, game);
        }
    } else {
        let v7: u16 = ((*E).var_E).wrapping_add(1);
        (*E).var_E = v7;
        if v7 as i32 - 4i32 & 0x8000 != 0 {
            (*E).var_A = -((*E).var_A as i32) as u16;
            MaridiaSnail_Func_13(k, *(j.offset(4) as *mut u16), game);
        } else {
            MaridiaSnail_Func_14(k, game);
        }
    };
}
pub unsafe fn MaridiaSnail_Func_10(k: u16, j: *const u8, game: &mut Game) {
    let E = Get_MaridiaSnail(k as i32, game);
    (*E).base.x_pos = ((*E).base.x_pos).wrapping_add(*(j as *mut u16));
    (*E).base.y_pos = ((*E).base.y_pos).wrapping_add(*(j.offset(2) as *mut u16));
}
pub unsafe fn MaridiaSnail_Func_11(k: u16, j: *const u8, game: &mut Game) {
    let E = Get_MaridiaSnail(k as i32, game);
    (*E).base.x_pos = ((*E).base.x_pos).wrapping_sub(*(j as *mut u16));
    (*E).base.y_pos = ((*E).base.y_pos).wrapping_sub(*(j.offset(2) as *mut u16));
}
pub unsafe fn MaridiaSnail_Func_12(k: u16, carry: u16, game: &mut Game) {
    let E = Get_MaridiaSnail(k as i32, game);
    if carry == 0 {
        if ((*E).var_04).wrapping_add(1) >= 0x10 {
            (*E).var_05 = 0;
        } else {
            (*E).var_04 = (*E).var_04.wrapping_add(1);
        }
    } else {
        (*E).var_05 = 1;
        (*E).var_04 = 0;
    };
}
pub unsafe fn MaridiaSnail_Func_13(k: u16, a: u16, game: &mut Game) {
    let E = Get_MaridiaSnail(k as i32, game);
    enemy_instruction! {[(*E).base]; instruction = a, timer = 1}
    (*E).var_05 = 1;
    (*E).var_04 = 0;
}
pub unsafe fn MaridiaSnail_Func_14(k: u16, game: &mut Game) {
    let E = Get_MaridiaSnail(k as i32, game);
    if (*E).var_08 != 3 {
        (*E).var_08 = 3;
        (*E).var_F = ((0xa3d1b3i32) as u16);
        let v2: i32 = 2 * (*E).var_C as i32;
        (*E).base.current_instruction = crate::rtl::rom_table_word!(game, 0xa3d1ab, v2 as isize);
        (*E).var_D = crate::rtl::rom_table_word!(game, 0xa3d1ab, (v2 + 1) as isize);
        (*E).base.instruction_timer = 1;
        (*E).base.timer = 0;
        set_fields! {[(*E)], 0; var_02, var_03, var_00, var_01}
    }
}
pub unsafe fn MaridiaSnail_Func_15(k: u16, game: &mut Game) {
    let E = Get_MaridiaSnail(k as i32, game);
    if (*E).var_08 != 3 {
        if Enemy_MoveRight_IgnoreSlopes(k, (words!((*E), var_03, var_02)) as i32, game) != 0 {
            (*E).var_02 = -((*E).var_02 as i32) as u16;
            (*E).var_03 = -((*E).var_03 as i32) as u16;
            (*E).var_20 = 1;
            QueueSfx2_Max3(0x70, game);
        } else {
            let mut delta: u32 = 0;
            if ((*E).var_03 & 0x8000) != 0 {
                delta = 4096;
            } else {
                delta = -4096i32 as u32;
            }
            let t: u32 = (words!((*E), var_03, var_02)).wrapping_add(delta);
            (*E).var_02 = t as u16;
            if t >> 16 != 0 {
                (*E).var_03 = (t >> 16) as u16;
            }
        }
    }
    if Enemy_MoveDown(k, (words!((*E), var_01, var_00)) as i32, game) != 0 {
        let msl_var_01: i16 = (*E).var_01 as i16;
        if msl_var_01 >= 0 && msl_var_01 as i32 - 3i32 & 0x8000 != 0 {
            set_fields! {[(*E)], 0; var_03, var_02, var_01, var_00, var_E, var_04}
            (*E).var_05 = 1;
            if (*E).var_08 == 3 {
                (*E).var_08 = 0;
            } else {
                (*E).var_08 = 1;
                (*E).parameter_1 = 8;
                MaridiaSnail_Func_18(k, game);
                MaridiaSnail_Func_16(k, game);
            }
            (*E).base.properties = ((*E).var_C | (*E).base.properties & !(3));
            StoneZoomer_E67A(k, game);
            (*E).var_F = ((0xa3cf5fi32) as u16);
            enemy_instruction! {[(*E).base]; instruction = (*E).var_D, timer = 1}
            (*E).base.timer = 0;
        } else {
            (*E).var_00 = -((*E).var_00 as i32) as u16;
            (*E).var_01 = -((*E).var_01 as i32) as u16;
            (*E).var_20 = 0;
        }
    } else {
        let t_0: u32 = (words!((*E), var_01, var_00)).wrapping_add(0x2000);
        (*E).var_00 = t_0 as u16;
        if (t_0 >> 16i32).wrapping_sub(4) & 0x8000 != 0 {
            (*E).var_01 = (t_0 >> 16) as u16;
        }
    };
}
pub unsafe fn MaridiaSnail_Func_16(k: u16, game: &mut Game) {
    let E = Get_MaridiaSnail(k as i32, game);
    let v2: i32 = ((4u16).wrapping_mul((*E).var_C)) as i32 >> 1;
    (*E).base.current_instruction = crate::rtl::rom_table_word!(game, 0xa3d30d, v2 as isize);
    (*E).var_D = crate::rtl::rom_table_word!(game, 0xa3d30d, (v2 + 1) as isize);
}
pub unsafe fn MaridiaSnail_Func_18(k: u16, game: &mut Game) -> u8 {
    let E = Get_MaridiaSnail(k as i32, game);
    if (*E).var_C != 0 {
        ((if (*E).base.x_pos >= word!(game.samus_x_pos) {
            MaridiaSnail_Func_19(k, game) as i32
        } else {
            0
        }) as u8)
    } else {
        ((if (*E).base.x_pos < word!(game.samus_x_pos) {
            MaridiaSnail_Func_19(k, game) as i32
        } else {
            0
        }) as u8)
    }
}
pub unsafe fn MaridiaSnail_Func_19(k: u16, game: &mut Game) -> u8 {
    let E = Get_MaridiaSnail(k as i32, game);
    if (*E).var_08 == 2 || (*E).var_F == 0xcf5f {
        return 0;
    }
    let v2: u16 = crate::rtl::rom_table_word!(game, 0xa3cdc2, (*E).var_07 as isize);
    (*E).var_07 = v2;
    (*E).base.current_instruction =
        *((&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa3cd42))
            as *mut MaridiaSnailData2)
            .offset(0))
        .field_0 as *mut u8)
            .offset((8 * v2 as i32) as isize) as *mut VoidP);
    (*E).base.properties = (*((&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa3cd42))
        as *mut MaridiaSnailData2)
        .offset(0))
    .field_2 as *mut u8)
        .offset((8 * v2 as i32) as isize) as *mut u16)
        | (*E).base.properties & !(3));
    (*E).var_D = *((&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa3cd42))
        as *mut MaridiaSnailData2)
        .offset(0))
    .field_4 as *mut u8)
        .offset((8 * v2 as i32) as isize) as *mut VoidP);
    (*E).var_C = *((&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa3cd42))
        as *mut MaridiaSnailData2)
        .offset(0))
    .field_6 as *mut u8)
        .offset((8 * v2 as i32) as isize) as *mut u16);
    MaridiaSnail_Func_2(k, (8u16).wrapping_mul(v2), game);
    MaridiaSnail_Func_3(k, game);
    (*E).var_05 = 1;
    (*E).var_04 = 0;
    1
}
pub unsafe fn MaridiaSnail_Touch(game: &mut Game) {
    let E = Get_MaridiaSnail(word!(game.cur_enemy_index) as i32, game);
    if ((*E).var_08 != 1
        || (*E).var_F == 0xcf5f
        || MaridiaSnail_Func_20(word!(game.cur_enemy_index), game) & 1 == 0)
        && ((*E).var_F == 0xd1b3 || word!(game.samus_has_momentum_flag) != 0)
    {
        MaridiaSnail_Func_22(word!(game.cur_enemy_index), game);
        if (*E).var_F == 0xd1b3 {
            QueueSfx2_Max3(0x70, game);
        }
    } else if (*E).var_F != 0xcf5f && (*E).var_08 != 4 && (*E).var_08 != 3 {
        Enemy_NormalTouchAI_A3(game);
        (*E).parameter_1 = (*E).var_06;
        if (*E).var_08 != 0 {
            MaridiaSnail_Func_19(word!(game.cur_enemy_index), game);
        }
        (*E).var_08 = 0;
    }
}
pub unsafe fn MaridiaSnail_Func_20(k: u16, game: &mut Game) -> u8 {
    let r20: u16 = ((*Get_MaridiaSnail(k as i32, game)).var_C) & (1u16);
    if (((word!(game.joypad1_lastkeys)) & (0x300u16)) as i32 >> 8) == 1 {
        if r20 != 0 {
            return 0;
        }
    } else if r20 == 0 {
        return 0;
    }
    1
}
pub unsafe fn MaridiaSnail_Shot(game: &mut Game) {
    let v0: u16 =
        (word!(game.projectile_type[word!(game.collision_detection_index)])) & (0xff00u16);
    if v0 == 768 || v0 == 1280 {
        Enemy_NormalShotAI_A3(game);
    } else {
        let msl_var_08: u16 = (*Get_MaridiaSnail(word!(game.cur_enemy_index) as i32, game)).var_08;
        if msl_var_08 != 3 && msl_var_08 != 4 {
            MaridiaSnail_Func_23(word!(game.cur_enemy_index), game);
        }
        QueueSfx2_Max3(0x70, game);
    };
}
pub unsafe fn MaridiaSnail_Func_22(k: u16, game: &mut Game) {
    let E = Get_MaridiaSnail(k as i32, game);
    (*E).var_08 = 4;
    (*E).var_F = ((0xa3d1b3i32) as u16);
    let v2: i32 = ((4u16).wrapping_mul((*E).var_C)) as i32 >> 1;
    (*E).base.current_instruction = crate::rtl::rom_table_word!(game, 0xa3d50f, v2 as isize);
    (*E).var_D = crate::rtl::rom_table_word!(game, 0xa3d50f, (v2 + 1) as isize);
    (*E).base.instruction_timer = 1;
    (*E).base.timer = 0;
    (*E).var_02 = word!(game.absolute_moved_last_frame_x_fract);
    let mut v3: u16 = word!(game.absolute_moved_last_frame_x);
    (*E).var_03 = word!(game.absolute_moved_last_frame_x);
    if v3 >= 0x10 {
        v3 = 15;
    }
    let v4: i32 = ((4u16).wrapping_mul(v3)) as i32 >> 1;
    (*E).var_00 = crate::rtl::rom_table_word!(game, 0xa3d517, v4 as isize);
    (*E).var_01 = crate::rtl::rom_table_word!(game, 0xa3d517, (v4 + 1) as isize);
    if (*ram_ptr!(game; 0xa1e) & 4) != 0 {
        (*E).var_02 = -((*E).var_02 as i32) as u16;
        (*E).var_03 = -((*E).var_03 as i32) as u16;
    }
}
pub unsafe fn MaridiaSnail_Func_23(k: u16, game: &mut Game) {
    let E = Get_MaridiaSnail(k as i32, game);
    (*E).var_08 = 5;
    (*E).var_F = ((0xa3d1b3i32) as u16);
    let v2: i32 = ((4u16).wrapping_mul((*E).var_C)) as i32 >> 1;
    (*E).base.current_instruction = crate::rtl::rom_table_word!(game, 0xa3d5a4, v2 as isize);
    (*E).var_D = crate::rtl::rom_table_word!(game, 0xa3d5a4, (v2 + 1) as isize);
    (*E).base.instruction_timer = 1;
    (*E).base.timer = 0;
    (*E).var_01 = -1i32 as u16;
    (*E).var_03 = (if *ram_ptr!(game; 0xa1e) == 4 {
        -1i32
    } else {
        1
    }) as u16;
}
pub unsafe fn Reflec_Func_1(game: &mut Game) {
    if word!(game.door_transition_flag_enemies) == 0 && {
        word!(
            game.variables_for_enemy_graphics_drawn_hook[2],
            wrapping_sub(1),
            reread
        ) == 0
    } {
        word!(game.variables_for_enemy_graphics_drawn_hook[2] = 16);
        let mut v0: u16 = word!(game.variables_for_enemy_graphics_drawn_hook[0]);
        let mut v1: u16 =
            (8u16).wrapping_mul(word!(game.variables_for_enemy_graphics_drawn_hook[1]));
        for _ in 0..4 {
            *(ram_ptr!(game, u16; 0xc000)).offset(((v0 as i32 >> 1) + 137) as isize) =
                crate::rtl::rom_table_word!(game, 0xa3dabc, (v1 >> 1) as isize);
            v1 = (v1).wrapping_add(2);
            v0 = (v0).wrapping_add(2);
        }
        word!(
            game.variables_for_enemy_graphics_drawn_hook[1] =
                (*((ram_ptr!(game, u16; 0x1794)).offset(1) as *mut u8).offset(0) as i32 + 1 & 7)
                    as u16
        );
    }
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16;
    Reflec_Instr_1 [jp.offset(1)] { (*Get_Reflec(k as i32, game)).parameter_2 = *jp.offset(0); };
}
pub unsafe fn Reflec_Init(game: &mut Game) {
    let E = Get_Reflec(word!(game.cur_enemy_index) as i32, game);
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_BlockPlasmaBeam) as u16);
    (*E).base.current_instruction =
        crate::rtl::rom_table_word!(game, 0xa3dc0b, (*E).parameter_1 as isize);
    (*(ram_ptr!(game, LongPtr; 0x178c))).addr = ((0xa3db0ci32) as u16);
    *(&raw mut (*(ram_ptr!(game, LongPtr; 0x178c))).bank as *mut u16) = 163;
    word!(
        game.variables_for_enemy_graphics_drawn_hook[0] =
            ((16 * (*E).base.palette_index as i32 & 0xff00) >> 8) as u16
    );
    word!(game.variables_for_enemy_graphics_drawn_hook[2] = 16);
}
pub unsafe fn Reflec_Shot(game: &mut Game) {
    let v0: u16 = (2u16).wrapping_mul(word!(game.collision_detection_index));
    let EK = Get_Reflec(word!(game.cur_enemy_index) as i32, game);
    (*EK).base.invincibility_timer = 10;
    let v2: u16 = (32 * (*EK).parameter_2 as i32
        + 2 * (word!(game.projectile_dir[(v0 >> 1)]) as i32 & 0xf)) as u16;
    let varE32: u16 = v2;
    let v3: i32 = v2 as i32 >> 1;
    if crate::rtl::rom_table_word!(game, 0xa3dcae, v3 as isize) == 0x8000 {
        word!(game.projectile_dir[(v0 >> 1)], |fresh16| (fresh16)
            | (0x10u16));
        printf(b"Possible bug. What is X?\n\0" as *const u8 as *const c_char);
        let ET = Get_Reflec(v2 as i32, game);
        if (*ET).base.health != 0 {
            Enemy_NormalShotAI_SkipSomeParts_A3(game);
            if (*ET).base.health == 0 {
                enemy_instruction! {[(*ET).base]; instruction = crate::rtl::rom_table_word!(game, 0xa3dca6, (*ET).parameter_2 as isize), timer = 1}
                (*ET).base.timer = 0;
            }
        }
    } else {
        let mut v4: i16 = crate::rtl::rom_table_word!(game, 0xa3dcae, v3 as isize) as i16;
        if v4 < 0 {
            v4 = -(crate::rtl::rom_table_word!(game, 0xa3dcae, (varE32 >> 1) as isize) as i32)
                as i16;
        }
        let v5: i32 = v0 as i32 >> 1;
        word!(game.projectile_dir[v5] = v4 as u16);
        word!(game.projectile_type[v5], |fresh17| (fresh17 & !(0x8000)));
        ProjectileReflection(v0, game);
        QueueSfx2_Max6(0x57, game);
    };
}
pub unsafe fn WreckedShipOrangeZoomer_Func_1(
    k: u16,
    jp: *const u16,
    game: &mut Game,
) -> *const u16 {
    (*Get_WreckedShipOrangeZoomer(k as i32, game)).var_F = *jp.offset(0);
    jp.offset(1)
}
pub unsafe fn WreckedShipOrangeZoomer_Init(game: &mut Game) {
    let E = Get_WreckedShipOrangeZoomer(word!(game.cur_enemy_index) as i32, game);
    (*E).base.current_instruction = crate::rtl::rom_table_word!(
        game,
        0xa3e03b,
        ((*E).base.current_instruction as i32 & 3) as isize
    );
    (*E).var_F = addr_locret_A3E08A as u16;
    let v1: u16 = crate::rtl::rom_table_word!(game, 0xa3e5f0, (*E).parameter_1 as isize);
    (*E).var_A = v1;
    (*E).var_B = v1;
    if ((*E).base.properties & 3) != 0 {
        if ((*E).base.properties & 3) == 2 {
            (*E).var_B = -((*E).var_B as i32) as u16;
        }
    } else {
        (*E).var_A = -((*E).var_A as i32) as u16;
    };
}
pub unsafe fn WreckedShipOrangeZoomer_Main(game: &mut Game) {
    let E = Get_WreckedShipOrangeZoomer(word!(game.cur_enemy_index) as i32, game);
    CallEnemyPreInstr((((*E).var_F) as u32) | (0xa30000u32), game);
}
pub unsafe fn WreckedShipOrangeZoomer_Func_2(k: u16, game: &mut Game) {
    let mut v7: i16 = 0;
    let E = Get_WreckedShipOrangeZoomer(k as i32, game);
    if word!(game.earthquake_timer) == 30 && word!(game.earthquake_type) == 20 {
        (*E).var_03 = (*E).var_F;
        (*E).var_F = ((0xa3e785i32) as u16);
    }
    if Enemy_MoveRight_ProcessSlopes(k, Shift8AddMagn((*E).var_A as i16, 1) as i32, game) != 0 {
        (*E).var_04 = 0;
        EnemyFunc_C8AD(k, game);
        if Enemy_MoveDown(k, ((*E).var_B as i16 as i32) << 8, game) == 0 {
            let negate_displacement: bool =
                if ((word!(game.samus_y_pos)) as i16).wrapping_sub(((*E).base.y_pos) as i16) >= 0 {
                    v7 = (*E).var_B as i16;
                    (v7 < 0)
                } else {
                    v7 = (*E).var_B as i16;
                    (v7 >= 0)
                };
            if negate_displacement {
                v7 = -(v7 as i32) as i16;
            }
            (*E).var_B = v7 as u16;
            return;
        }
        (*E).var_A = -((*E).var_A as i32) as u16;
        let mut v9: u16 = addr_kWreckedShipOrangeZoomer_Ilist_E01F as u16;
        if ((*E).var_B & 0x8000) != 0 {
            v9 = addr_kWreckedShipOrangeZoomer_Ilist_E003 as u16;
        }
        enemy_instruction! {[(*E).base]; instruction = v9, timer = 1}
    } else {
        let v11: u16 = ((*E).var_04).wrapping_add(1);
        (*E).var_04 = v11;
        if v11 as i32 - 4i32 & 0x8000 != 0 {
            (*E).var_B = -((*E).var_B as i32) as u16;
            let mut v12: u16 = addr_kWreckedShipOrangeZoomer_Ilist_E01F as u16;
            if ((*E).var_B & 0x8000) != 0 {
                v12 = addr_kWreckedShipOrangeZoomer_Ilist_E003 as u16;
            }
            enemy_instruction! {[(*E).base]; instruction = v12, timer = 1}
        } else {
            (*E).var_03 = (*E).var_F;
            (*E).var_F = ((0xa3e785i32) as u16);
        }
    };
}
pub unsafe fn sub_A3E168(k: u16, game: &mut Game) {
    let mut v7: i16 = 0;
    let E = Get_WreckedShipOrangeZoomer(k as i32, game);
    if word!(game.earthquake_timer) == 30 && word!(game.earthquake_type) == 20 {
        (*E).var_03 = (*E).var_F;
        (*E).var_F = ((0xa3e785i32) as u16);
    }
    if Enemy_MoveDown(k, Shift8AddMagn((*E).var_B as i16, 1) as i32, game) != 0 {
        (*E).var_04 = 0;
        if Enemy_MoveRight_ProcessSlopes(k, ((*E).var_A as i16 as i32) << 8, game) == 0 {
            EnemyFunc_C8AD(k, game);
            if ((word!(game.samus_x_pos)) as i16).wrapping_sub(((*E).base.x_pos) as i16) >= 0 {
                v7 = (*E).var_A as i16;
                if v7 < 0 {
                    v7 = -(v7 as i32) as i16;
                }
            } else {
                v7 = (*E).var_A as i16;
                if v7 >= 0 {
                    v7 = -(v7 as i32) as i16;
                }
            }
            (*E).var_A = v7 as u16;
            return;
        }
        (*E).var_B = -((*E).var_B as i32) as u16;
        let mut v9: u16 = addr_kWreckedShipOrangeZoomer_Ilist_DFE7 as u16;
        if ((*E).var_A & 0x8000) != 0 {
            v9 = addr_kWreckedShipOrangeZoomer_Ilist_DFCB as u16;
        }
        enemy_instruction! {[(*E).base]; instruction = v9, timer = 1}
    } else {
        let v11: u16 = ((*E).var_04).wrapping_add(1);
        (*E).var_04 = v11;
        if v11 as i32 - 4i32 & 0x8000 != 0 {
            (*E).var_A = -((*E).var_A as i32) as u16;
            let mut v12: u16 = addr_kWreckedShipOrangeZoomer_Ilist_DFE7 as u16;
            if ((*E).var_A & 0x8000) != 0 {
                v12 = addr_kWreckedShipOrangeZoomer_Ilist_DFCB as u16;
            }
            enemy_instruction! {[(*E).base]; instruction = v12, timer = 1}
        } else {
            (*E).var_03 = (*E).var_F;
            (*E).var_F = ((0xa3e785i32) as u16);
        }
    };
}
pub unsafe fn BigEyeBugs_Init(game: &mut Game) {
    let v1 = gEnemyData(word!(game.cur_enemy_index) as i32, game);
    (*v1).current_instruction = crate::rtl::rom_table_word!(
        game,
        0xa3e2cc,
        ((*v1).current_instruction as i32 & 3) as isize
    );
    StoneZoomer_E67A(word!(game.cur_enemy_index), game);
}
pub unsafe fn FireZoomer_Init(game: &mut Game) {
    let E = Get_FireZoomer(word!(game.cur_enemy_index) as i32, game);
    (*E).base.current_instruction = crate::rtl::rom_table_word!(
        game,
        0xa3e2cc,
        ((*E).base.current_instruction as i32 & 3) as isize
    );
    StoneZoomer_E67A(word!(game.cur_enemy_index), game);
}
pub unsafe fn Zoomer_Instr_SetPreinstr(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*gEnemyData(k as i32, game)).ai_preinstr = *jp.offset(0);
    jp.offset(1)
}
pub unsafe fn StoneZoomer_Init(game: &mut Game) {
    let E = Get_StoneZoomer(word!(game.cur_enemy_index) as i32, game);
    (*E).base.current_instruction = crate::rtl::rom_table_word!(
        game,
        0xa3e2cc,
        ((*E).base.current_instruction as i32 & 3) as isize
    );
    StoneZoomer_E67A(word!(game.cur_enemy_index), game);
}
pub unsafe fn StoneZoomer_E67A(k: u16, game: &mut Game) {
    let E = Get_StoneZoomer(k as i32, game);
    (*E).base.spritemap_pointer = addr_kSpritemap_Nothing_A3 as u16;
    (*E).base.instruction_timer = 1;
    (*E).var_F = ((0xa3e6c1i32) as u16);
    if (*E).parameter_1 != 255 {
        let v2: u16 = crate::rtl::rom_table_word!(game, 0xa3e5f0, (*E).parameter_1 as isize);
        (*E).var_A = v2;
        (*E).var_B = v2;
    }
    if ((*E).base.properties & 3) != 0 {
        if ((*E).base.properties & 3) == 2 {
            (*E).var_B = -((*E).var_B as i32) as u16;
        }
    } else {
        (*E).var_A = -((*E).var_A as i32) as u16;
    };
}
pub unsafe fn StoneZoomer_Main(game: &mut Game) {
    let E = Get_StoneZoomer(word!(game.cur_enemy_index) as i32, game);
    CallEnemyPreInstr((((*E).var_F) as u32) | (0xa30000u32), game);
}
pub unsafe fn FireZoomer_Func_1(k: u16, game: &mut Game) {
    let E = Get_FireZoomer(k as i32, game);
    if word!(game.earthquake_timer) == 30 && word!(game.earthquake_type) == 20 {
        (*E).var_03 = (*E).var_F;
        (*E).var_F = ((0xa3e785i32) as u16);
    }
    if Enemy_MoveRight_IgnoreSlopes(k, Shift8AddMagn((*E).var_A as i16, 1) as i32, game) != 0 {
        (*E).var_04 = 0;
        EnemyFunc_C8AD(k, game);
        if Enemy_MoveDown(k, ((*E).var_B as i16 as i32) << 8, game) != 0 {
            (*E).var_A = -((*E).var_A as i32) as u16;
            let fzr_parameter_2: u16 = (*E).parameter_2;
            let mut v7: u16 = 0;
            if ((*E).var_B & 0x8000) == 0 {
                v7 = crate::rtl::rom_table_word!(game, 0xa3e63c, (fzr_parameter_2 >> 1) as isize);
            } else {
                v7 = crate::rtl::rom_table_word!(game, 0xa3e630, (fzr_parameter_2 >> 1) as isize);
            }
            enemy_instruction! {[(*E).base]; instruction = v7, timer = 1}
        }
    } else {
        let v8: u16 = ((*E).var_04).wrapping_add(1);
        (*E).var_04 = v8;
        if v8 as i32 - 4i32 & 0x8000 != 0 {
            (*E).var_B = -((*E).var_B as i32) as u16;
            let v9: u16 = (*E).parameter_2;
            let mut v10: u16 = 0;
            if ((*E).var_B & 0x8000) == 0 {
                v10 = crate::rtl::rom_table_word!(game, 0xa3e63c, (v9 >> 1) as isize);
            } else {
                v10 = crate::rtl::rom_table_word!(game, 0xa3e630, (v9 >> 1) as isize);
            }
            enemy_instruction! {[(*E).base]; instruction = v10, timer = 1}
        } else {
            (*E).var_03 = (*E).var_F;
            (*E).var_F = ((0xa3e785i32) as u16);
        }
    };
}
pub unsafe fn FireZoomer_Func_2(k: u16, game: &mut Game) {
    let E = Get_FireZoomer(k as i32, game);
    if Enemy_MoveDown(k, (words!((*E), var_02, var_01)) as i32, game) != 0 {
        if (*E).parameter_1 == 255 {
            set_fields! {[(*E)], 128; var_A, var_B}
        }
        set_fields! {[(*E)], 0; var_01, var_02, var_04}
        (*E).var_F = (*E).var_03;
    } else {
        if (*E).var_02 as i32 - 4i32 & 0x8000 != 0 {
            words!((*E), var_02, var_01 += 0x8000);
        }
        if (*E).var_01 == 0 && (*E).var_02 == 0 {
            (*E).var_F = ((0xa3e6c8i32) as u16);
        }
    };
}
pub unsafe fn FireZoomer_Func_3(k: u16, game: &mut Game) {
    let E = Get_FireZoomer(k as i32, game);
    if word!(game.earthquake_timer) == 30 && word!(game.earthquake_type) == 20 {
        (*E).var_03 = (*E).var_F;
        (*E).var_F = ((0xa3e785i32) as u16);
    }
    if Enemy_MoveDown(k, Shift8AddMagn((*E).var_B as i16, 1) as i32, game) != 0 {
        (*E).var_04 = 0;
        let amt: i32 = FireZoomer_E8A5(k, game);
        if Enemy_MoveRight_IgnoreSlopes(k, amt, game) != 0 {
            (*E).var_B = -((*E).var_B as i32) as u16;
            let fzr_parameter_2: u16 = (*E).parameter_2;
            let mut v6: u16 = 0;
            if ((*E).var_A & 0x8000) == 0 {
                v6 = crate::rtl::rom_table_word!(game, 0xa3e654, (fzr_parameter_2 >> 1) as isize);
            } else {
                v6 = crate::rtl::rom_table_word!(game, 0xa3e648, (fzr_parameter_2 >> 1) as isize);
            }
            enemy_instruction! {[(*E).base]; instruction = v6, timer = 1}
        } else {
            EnemyFunc_C8AD(k, game);
        }
    } else {
        let v7: u16 = ((*E).var_04).wrapping_add(1);
        (*E).var_04 = v7;
        if v7 as i32 - 4i32 & 0x8000 != 0 {
            (*E).var_A = -((*E).var_A as i32) as u16;
            let v8: u16 = (*E).parameter_2;
            let mut v9: u16 = 0;
            if ((*E).var_A & 0x8000) == 0 {
                v9 = crate::rtl::rom_table_word!(game, 0xa3e654, (v8 >> 1) as isize);
            } else {
                v9 = crate::rtl::rom_table_word!(game, 0xa3e648, (v8 >> 1) as isize);
            }
            enemy_instruction! {[(*E).base]; instruction = v9, timer = 1}
        } else {
            (*E).var_03 = (*E).var_F;
            (*E).var_F = ((0xa3e785i32) as u16);
        }
    };
}
pub unsafe fn FireZoomer_E8A5(k: u16, game: &mut Game) -> i32 {
    let E = Get_FireZoomer(word!(game.cur_enemy_index) as i32, game);
    let xpos: u16 = (*E).base.x_pos;
    let ypos: u16 = (if (*E).var_B & 0x8000 != 0 {
        (*E).base.y_pos as i32 - (*E).base.y_height as i32
    } else {
        (*E).base.y_pos as i32 + (*E).base.y_height as i32 - 1i32
    }) as u16;
    CalculateBlockContainingPixelPos(xpos, ypos, game);
    if (word!(game.level_data[word!(game.cur_block_index)]) & 0xf000) == 4096
        && (*ram_ptr!(game; 0x16402, word!(game.cur_block_index) as isize) & 0x1f) >= 5
    {
        let v1: u16 = crate::rtl::rom_table_word!(
            game,
            0xa3e931,
            ((4 * (*ram_ptr!(game; 0x16402, word!(game.cur_block_index) as isize) as i32 & 0x1f))
                as u16
                >> 1) as isize,
        );
        if ((*E).var_A as i16) >= 0 {
            Multiply16x16((*E).var_A, v1, game) as i32
        } else {
            -(Multiply16x16(-((*E).var_A as i32) as u16, v1, game) as i32)
        }
    } else {
        ((*E).var_A as i16 as i32) << 8
    }
}
pub unsafe fn Metroid_Init(game: &mut Game) {
    let E = Get_Metroid(word!(game.cur_enemy_index) as i32, game);
    (*E).base.current_instruction = addr_kMetroid_Ilist_E9CF as u16;
    (*E).var_00 = CreateSpriteAtPos(
        (*E).base.x_pos,
        (*E).base.y_pos,
        50,
        ((*E).base.vram_tiles_index) | ((*E).base.palette_index),
        game,
    );
    (*E).var_01 = CreateSpriteAtPos(
        (*E).base.x_pos,
        (*E).base.y_pos,
        52,
        ((*E).base.vram_tiles_index) | ((*E).base.palette_index),
        game,
    );
    (*E).var_02 = 0;
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16;
    Metroid_Instr_2 [jp] { QueueSfx2_Max6(0x50, game); };
    Metroid_Instr_1 [jp] {
        let v2: i32 = NextRandom(game) as i32 & 7;
        QueueSfx2_Max6(
            crate::rtl::rom_table_word!(game, 0xa3ead6, v2 as isize),
            game,
        );
    };
}
pub unsafe fn Metroid_Frozen(game: &mut Game) {
    Enemy_NormalFrozenAI_A3(game);
    let E = Get_Metroid(word!(game.cur_enemy_index) as i32, game);
    if (*E).var_E != 0 {
        (*E).var_E = (*E).var_E.wrapping_sub(1);
        (*E).base.flash_timer = 2;
    }
    let v1: i32 = (*E).var_00 as i32 >> 1;
    word!(game.sprite_palettes[v1] = 3072);
    word!(game.sprite_disable_flag[v1] = 1);
    word!(game.sprite_instr_list_ptrs[v1] = addr_kSpriteObject_Ilist_C3BA as u16);
    let v2: i32 = (*E).var_01 as i32 >> 1;
    word!(game.sprite_palettes[v2] = 3072);
    word!(game.sprite_disable_flag[v2] = 1);
    word!(game.sprite_instr_list_ptrs[v2] = addr_kSpriteObject_Ilist_C4B6 as u16);
}
pub unsafe fn Metroid_Hurt(game: &mut Game) {
    let E = Get_Metroid(word!(game.cur_enemy_index) as i32, game);
    if ((*E).base.flash_timer & 2) != 0 {
        let r18: u16 = (*E).base.palette_index;
        let v1: i32 = (*E).var_00 as i32 >> 1;
        word!(game.sprite_palettes[v1] = (r18 | word!(game.sprite_palettes[v1]) & 0xf1ff));
        let v2: i32 = (*E).var_01 as i32 >> 1;
        word!(game.sprite_palettes[v2] = (r18 | word!(game.sprite_palettes[v2]) & 0xf1ff));
    } else {
        let v3: i32 = (*E).var_00 as i32 >> 1;
        word!(game.sprite_palettes[v3], |fresh10| (fresh10) & (0xf1ffu16));
        let metroid_var_01: u16 = (*E).var_01;
        word!(
            game.sprite_palettes[(metroid_var_01 >> 1)],
            |fresh11| (fresh11) & (0xf1ffu16)
        );
    };
}
static off_A3EC09: [Option<Func_Y_V>; 4] = callback_table! { unsafe Func_Y_V;
    Metroid_Func_1, Metroid_Func_2, Metroid_Func_3, Metroid_Func_4,
};
pub unsafe fn Metroid_Main(game: &mut Game) {
    let E = Get_Metroid(word!(game.cur_enemy_index) as i32, game);
    off_A3EC09[(*E).var_F as usize].expect("non-null function pointer")(
        (word!(game.samus_y_pos)).wrapping_sub(8),
        game,
    );
    let r24: u16 = ((*E).base.vram_tiles_index) | ((*E).base.palette_index);
    let v2: i32 = (*E).var_00 as i32 >> 1;
    word!(game.sprite_x_pos[v2] = (*E).base.x_pos);
    word!(game.sprite_y_pos[v2] = (*E).base.y_pos);
    word!(game.sprite_palettes[v2] = r24);
    word!(game.sprite_disable_flag[v2] = 0);
    let v4: i32 = (*E).var_01 as i32 >> 1;
    word!(game.sprite_x_pos[v4] = (*E).base.x_pos);
    word!(game.sprite_y_pos[v4] = (*E).base.y_pos);
    word!(game.sprite_palettes[v4] = r24);
    word!(game.sprite_disable_flag[v4] = 0);
}
pub unsafe fn Metroid_Func_1(varE32: u16, game: &mut Game) {
    let E = Get_Metroid(word!(game.cur_enemy_index) as i32, game);
    let mut t: i32 =
        ((((((*E).base.y_pos) as i16).wrapping_sub((varE32) as i16)) >> 2) as i32) << 8;
    let mut r18: u16 = t as u16;
    let mut r20: u16 = (t >> 16) as u16;
    let metroid_var_C: u16 = (*E).var_C;
    let mut v2: bool = metroid_var_C < r18;
    (*E).var_C = (metroid_var_C).wrapping_sub(r18);
    let mut v3: i16 = ((*E).var_D as i32 - (v2 as i32 + r20 as i32)) as i16;
    (*E).var_D = v3 as u16;
    // The positive boundary also clears the fractional velocity.
    if v3 < -3 || v3 >= 3 {
        (*E).var_D = v3.clamp(-3, 3) as u16;
        (*E).var_C = 0;
    }
    if Enemy_MoveDown(
        word!(game.cur_enemy_index),
        (words!((*E), var_D, var_C)) as i32,
        game,
    ) != 0
    {
        set_fields! {[(*E)], 0; var_C, var_D}
    }
    t = ((((((*E).base.x_pos) as i16).wrapping_sub((word!(game.samus_x_pos)) as i16)) >> 2) as i32)
        << 8;
    r18 = t as u16;
    r20 = (t >> 16) as u16;
    let metroid_var_A: u16 = (*E).var_A;
    v2 = metroid_var_A < r18;
    (*E).var_A = (metroid_var_A).wrapping_sub(r18);
    let mut v8: i16 = ((*E).var_B as i32 - (v2 as i32 + r20 as i32)) as i16;
    (*E).var_B = v8 as u16;
    // The positive boundary also clears the fractional velocity.
    if v8 < -3 || v8 >= 3 {
        (*E).var_B = v8.clamp(-3, 3) as u16;
        (*E).var_A = 0;
    }
    if Enemy_MoveRight_IgnoreSlopes(
        word!(game.cur_enemy_index),
        (words!((*E), var_B, var_A)) as i32,
        game,
    ) != 0
    {
        set_fields! {[(*E)], 0; var_A, var_B}
    }
}
pub unsafe fn Metroid_Func_2(varE32: u16, game: &mut Game) {
    let mut v1: i16 = 0;
    let mut v5: i16 = 0;
    let mut r18: u16 = 0;
    let E = Get_Metroid(word!(game.cur_enemy_index) as i32, game);
    byte!(v1; 0) = (((varE32).wrapping_sub((*E).base.y_pos)) >> 8) as u8;
    byte!(v1; 1) = ((varE32) as u8).wrapping_sub(byte!((*E).base.y_pos; 0));
    let mut r20: u16 = ((((v1) as u16) & (0xff00u16)) >> 3);
    if (r20 & 0x1000) != 0 {
        r20 = (r20) | (0xe000u16);
    }
    if (r20 as i16) < -3 || (r20 as i16) >= 3 {
        r20 = (r20 as i16).clamp(-3, 3) as u16;
        r18 = 0;
    }
    (*E).var_C = r18;
    (*E).var_D = r20;
    if Enemy_MoveDown(
        word!(game.cur_enemy_index),
        (words!((r20, r18))) as i32,
        game,
    ) != 0
    {
        set_fields! {[(*E)], 0; var_C, var_D}
    }
    r18 = 0;
    r20 = 0;
    byte!(v5; 0) = (((word!(game.samus_x_pos)).wrapping_sub((*E).base.x_pos)) >> 8) as u8;
    byte!(v5; 1) = ((word!(game.samus_x_pos)) as u8).wrapping_sub(byte!((*E).base.x_pos; 0));
    r20 = ((((v5) as u16) & (0xff00u16)) >> 3);
    if (r20 & 0x1000) != 0 {
        r20 = (r20) | (0xe000u16);
    }
    if (r20 as i16) < -3 || (r20 as i16) >= 3 {
        r20 = (r20 as i16).clamp(-3, 3) as u16;
        r18 = 0;
    }
    (*E).var_A = r18;
    (*E).var_B = r20;
    if Enemy_MoveRight_IgnoreSlopes(
        word!(game.cur_enemy_index),
        (words!((r20, r18))) as i32,
        game,
    ) != 0
    {
        set_fields! {[(*E)], 0; var_A, var_B}
    }
}
pub unsafe fn Metroid_Func_3(varE32: u16, game: &mut Game) {
    let E = Get_Metroid(word!(game.cur_enemy_index) as i32, game);
    (*E).base.x_pos = word!(game.samus_x_pos);
    (*E).base.y_pos = varE32;
    set_fields! {[(*E)], 0; var_A, var_B, var_C, var_D}
}
pub unsafe fn Metroid_Func_4(varE32: u16, game: &mut Game) {
    let E = Get_Metroid(word!(game.cur_enemy_index) as i32, game);
    let v1: i32 = (*E).var_E as i32 & 3;
    (*E).base.x_pos =
        ((*E).base.x_pos).wrapping_add(crate::rtl::rom_table_word!(game, 0xa3ea3f, v1 as isize));
    (*E).base.y_pos = ((*E).base.y_pos).wrapping_add(crate::rtl::rom_table_word!(
        game,
        0xa3ea3f,
        (v1 + 4) as isize
    ));
    set_fields! {[(*E)], 0; var_A, var_B, var_C, var_D}
    crate::update_place! { previous fresh9 = (*E).var_E => (*E).var_E.wrapping_sub(1); }
    if fresh9 == 1 {
        (*E).var_F = 0;
        enemy_instruction! {[(*E).base]; instruction = addr_kMetroid_Ilist_E9CF as u16, timer = 1}
    }
}
pub unsafe fn Metroid_Touch(game: &mut Game) {
    let E = Get_Metroid(word!(game.cur_enemy_index) as i32, game);
    let varE32: u16 = (word!(game.samus_y_pos)).wrapping_sub(8);
    if word!(game.samus_contact_damage_index) != 0 {
        if (*E).var_F != 2 {
            set_fields! {[(*E)], 0; var_A, var_C}
            let v2: i16 = (((*E).base.x_pos) as i16).wrapping_sub((word!(game.samus_x_pos)) as i16);
            (*E).var_B = (if v2 < 0 { -256i32 } else { 0 }) as u16;
            *((&raw mut (*E).var_A as *mut u8).offset(1) as *mut u16) = ((v2 as i32) << 6) as u16;
            let v4: i16 = (((*E).base.y_pos) as i16).wrapping_sub((varE32) as i16);
            (*E).var_D = (if v4 < 0 { -256i32 } else { 0 }) as u16;
            *((&raw mut (*E).var_C as *mut u8).offset(1) as *mut u16) = ((v4 as i32) << 6) as u16;
            (*E).var_F = 0;
            enemy_instruction! {[(*E).base]; instruction = addr_kMetroid_Ilist_E9CF as u16, timer = 1}
        }
    } else {
        let v5: u16 = word!(game.cur_enemy_index);
        if (*E).var_F != 3 {
            if (word!(game.random_enemy_counter) & 7) == 7
                && word!(game.samus_health) as i32 - 30i32 & 0x8000 == 0
            {
                QueueSfx3_Max6(0x2d, game);
            }
            Metroid_Func_5(v5, game);
        }
        if (*E).var_F < 2 {
            let mut v7: u16 = 1;
            if abs16(((*E).base.x_pos).wrapping_sub(word!(game.samus_x_pos))) < 8
                && abs16(((*E).base.y_pos).wrapping_sub(varE32)) < 8
            {
                v7 = 2;
                CallSomeSamusCode(0x12, game);
            }
            (*E).var_F = v7;
            if v7 == 2 {
                enemy_instruction! {[(*E).base]; instruction = addr_kMetroid_Ilist_EA25 as u16, timer = 1}
            }
        }
    };
}
pub unsafe fn Metroid_Func_5(k: u16, game: &mut Game) {
    let mut v1: u16 = 0;
    let varE32: u16 = (word!(game.samus_y_pos)).wrapping_sub(8);
    if (word!(game.equipped_items) & 0x20) != 0 {
        v1 = 12288;
    } else if word!(game.equipped_items) & 1 != 0 {
        v1 = 24576;
    } else {
        v1 = -16384i32 as u16;
    }
    let r18: u16 = v1;
    let E = Get_Metroid(k as i32, game);
    let metroid_var_02: u16 = (*E).var_02;
    let v4: bool = metroid_var_02 < r18;
    (*E).var_02 = (metroid_var_02).wrapping_sub(r18);
    if v4 {
        Samus_DealDamage(1, game);
    }
}
pub unsafe fn Metroid_Shot(game: &mut Game) {
    let v0: u16 = (2u16).wrapping_mul(word!(game.collision_detection_index));
    let E = Get_Metroid(word!(game.cur_enemy_index) as i32, game);
    if (*E).base.frozen_timer != 0 {
        let v3: u16 = (word!(game.projectile_type[(v0 >> 1)])) & (0xf00u16);
        if v3 == 256 || v3 == 512 {
            word!(game.special_death_item_drop_x_origin_pos = (*E).base.x_pos);
            word!(game.special_death_item_drop_y_origin_pos = (*E).base.y_pos);
            Enemy_NormalShotAI_SkipSomeParts_A3(game);
            if (*E).base.health == 0 {
                (*E).var_B = 0;
                EnemyDeathAnimation(word!(game.cur_enemy_index), 4, game);
                CallSomeSamusCode(0x13, game);
                *(ram_ptr!(game, u16; 0xef78)).offset(((*E).var_00 >> 1) as isize) = 0;
                let metroid_var_01: u16 = (*E).var_01;
                *(ram_ptr!(game, u16; 0xef78)).offset((metroid_var_01 >> 1) as isize) = 0;
                Enemy_ItemDrop_Metroid(metroid_var_01, game);
            }
        }
    } else if (*E).var_F == 2 {
        if (word!(game.projectile_type[(v0 >> 1)]) & 0xf00) == 1280 {
            (*E).var_E = 4;
            (*E).var_F = 3;
            enemy_instruction! {[(*E).base]; instruction = addr_kMetroid_Ilist_E9CF as u16, timer = 1}
            CallSomeSamusCode(0x13, game);
        }
    } else {
        set_fields! {[(*E)], 0; var_A, var_C}
        let v7: i16 =
            (((*E).base.x_pos) as i16).wrapping_sub((word!(game.projectile_x_pos[0])) as i16);
        (*E).var_B = (if v7 < 0 { -256i32 } else { 0 }) as u16;
        *((&raw mut (*E).var_A as *mut u8).offset(1) as *mut u16) =
            (32u16).wrapping_mul((v7) as u16);
        let v9: i16 =
            (((*E).base.y_pos) as i16).wrapping_sub((word!(game.projectile_y_pos[0])) as i16);
        (*E).var_D = (if v9 < 0 { -256i32 } else { 0 }) as u16;
        *((&raw mut (*E).var_C as *mut u8).offset(1) as *mut u16) =
            (32u16).wrapping_mul((v9) as u16);
        (*E).var_F = 0;
        enemy_instruction! {[(*E).base]; instruction = addr_kMetroid_Ilist_E9CF as u16, timer = 1}
        let v10: i32 = word!(game.collision_detection_index) as i32;
        if (word!(game.projectile_type[v10]) & 2) != 0 {
            QueueSfx3_Max6(0xa, game);
            let r18: u16 = word!(game.projectile_damage[v10]);
            (*E).base.flash_timer = 4;
            let metroid_parameter_2: u16 = (*E).parameter_2;
            let mut v13: bool = metroid_parameter_2 < r18;
            let v12: u16 = (metroid_parameter_2).wrapping_sub(r18);
            v13 = !v13;
            if v12 == 0 || !v13 {
                (*E).parameter_2 = 0;
                (*E).base.frozen_timer = 400;
                (*E).base.ai_handler_bits = ((*E).base.ai_handler_bits) | (4u16);
                return;
            }
            (*E).parameter_2 = v12;
        }
        QueueSfx2_Max6(0x5a, game);
    };
}
pub unsafe fn Metroid_Powerbomb(k: u16, game: &mut Game) {
    NormalEnemyPowerBombAi(game);
    if (*Get_Metroid(k as i32, game)).base.health == 0 {
        CallSomeSamusCode(0x13, game);
        *(ram_ptr!(game, u16; 0xef78)).offset(
            ((*Get_Metroid(word!(game.cur_enemy_index) as i32, game)).var_00 >> 1) as isize,
        ) = 0;
        *(ram_ptr!(game, u16; 0xef78)).offset(
            ((*Get_Metroid(word!(game.cur_enemy_index) as i32, game)).var_01 >> 1) as isize,
        ) = 0;
    }
}
