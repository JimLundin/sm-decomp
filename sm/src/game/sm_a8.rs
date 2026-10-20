use crate::c_library::printf;
use crate::game::callback_tables::callback_table;
pub use crate::game::constants::bank_86::*;
pub use crate::game::constants::enemy_properties::*;
use crate::game::enemy_instruction;
use crate::game::sm_80::{NextRandom, QueueSfx2_Max6, QueueSfx2_Max9, QueueSfx3_Max6};
use crate::game::sm_86::SpawnEprojWithGfx;
use crate::game::sm_88::SpawnMorphBallEyeBeamHdma;
use crate::game::sm_90::CallSomeSamusCode;
use crate::game::sm_a0::{
    Abs16, CalculateAngleFromXY, CalculateAngleOfSamusFromEnemy, CallEnemyPreInstr,
    CheckIfEnemyIsOnScreen, CheckIfEnemyTouchesSamus, CosineMult8bit, CosineMult8bitFull,
    EnemyDeathAnimation, EnemyFunc_ACA8, EnemyFunc_ADA3, EnemyFunc_B7A1, EnemyFunc_C8AD,
    EnemyGrappleDeath, Enemy_MoveDown, Enemy_MoveRight_IgnoreSlopes, Enemy_SwitchToFrozenAi,
    GetSamusEnemyDelta_X, GetSamusEnemyDelta_Y, IsSamusWithinEnemy_X, IsSamusWithinEnemy_Y,
    NormalEnemyFrozenAI, NormalEnemyPowerBombAi, NormalEnemyShotAi,
    NormalEnemyShotAiSkipDeathAnim_CurEnemy, NormalEnemyTouchAi, SamusHurtFromGrapple,
    SamusLatchesOnWithGrappleNoInvinc, SignExtend8, SineMult8bit, SineMult8bitFull,
    SwitchEnemyAiToMainAi,
};
use crate::game::sm_b4::CreateSpriteAtPos;
use crate::game_state::Game;
use crate::ram_map::ram_ptr;
use crate::ram_map::set_ram_words;
use crate::ram_map::{ram_word, set_ram_word, update_ram_word};
use crate::rtl::abs16;
use crate::rtl::{AddToHiLo, SetHiLo};
use crate::rtl::{Mult8x8, RomPtr, Unreachable, WriteReg};
use crate::set_fields;
pub use crate::snes::registers::*;
pub use crate::types::Consts_86;
pub use crate::types::Consts_A8;
pub use crate::types::{EnemyBase, EnemyData, EnemyProps, EnemySpawnData};
pub use crate::types::{Func_V, LongPtr, PairU16, Point16U, SnesRegs, VoidP, VramWriteEntry};
use ::core::ffi::c_char;
use ::core::ptr::null_mut;
crate::game::constants::typed_constants! { Consts_A8;
    addr_locret_A8F5E3 = 62947, addr_kKiHunter_Ilist_EB10 = 60176,
    addr_kKiHunter_Ilist_EAF2 = 60146, addr_kKiHunter_Ilist_EADA = 60122,
    addr_kKiHunter_Ilist_EAC2 = 60098, addr_kKiHunter_Ilist_EAA6 = 60070,
    addr_kKiHunter_Ilist_EA8A = 60042, addr_kKiHunter_Ilist_EA7E = 60030,
    addr_kKiHunter_Ilist_EA5E = 59998, addr_kKiHunter_Ilist_EA4E = 59982,
    addr_kKiHunter_Ilist_EA32 = 59954, addr_kKiHunter_Ilist_EA24 = 59940,
    addr_kKiHunter_Ilist_EA08 = 59912, addr_kKiHunter_Ilist_E9FA = 59898,
    addr_kBlueBrinstarFaceBlock_Ilist_E828 = 59432, addr_kBlueBrinstarFaceBlock_Ilist_E81A = 59418,
    addr_kBlueBrinstarFaceBlock_Ilist_E80C = 59404, addr_kWreckedShipSpark_Ilist_E5E5 = 58853,
    addr_kWreckedShipSpark_Ilist_E5A7 = 58791, addr_kWalkingLavaSeahorse_Ilist_DCC1 = 56513,
    addr_stru_A8DCBB = 56507, addr_kWalkingLavaSeahorse_Ilist_DC73 = 56435,
    addr_stru_A8DC59 = 56409, addr_kWalkingLavaSeahorse_Ilist_DC57 = 56407,
    addr_kWalkingLavaSeahorse_Ilist_DC51 = 56401, addr_stru_A8DC4B = 56395,
    addr_kWalkingLavaSeahorse_Ilist_DC03 = 56323, addr_stru_A8DBE9 = 56297,
    addr_kWalkingLavaSeahorse_Ilist_DBE7 = 56295, addr_kMaridiaPuffer_Ilist_D855 = 55381,
    addr_kMaridiaPuffer_Ilist_D841 = 55361, addr_loc_A8D02E = 53294,
    addr_loc_A8CF26 = 53030, addr_loc_A8CF18 = 53016,
    addr_kWreckedShipRobot_Ilist_CB65 = 52069, addr_kWreckedShipRobot_Ilist_CB1D = 51997,
    addr_kWreckedShipRobot_Ilist_CB09 = 51977, addr_kWreckedShipRobot_Ilist_CAFD = 51965,
    addr_kWreckedShipRobot_Ilist_CA7D = 51837, addr_kWreckedShipRobot_Ilist_CA01 = 51713,
    addr_kWreckedShipRobot_Ilist_C985 = 51589, addr_kWreckedShipRobot_Ilist_C92D = 51501,
    addr_kWreckedShipRobot_Ilist_C91B = 51483, addr_kWreckedShipRobot_Ilist_C8D1 = 51409,
    addr_kWreckedShipRobot_Ilist_C8BD = 51389, addr_kWreckedShipRobot_Ilist_C8B1 = 51377,
    addr_kWreckedShipRobot_Ilist_C833 = 51251, addr_kWreckedShipRobot_Ilist_C7BB = 51131,
    addr_kWreckedShipRobot_Ilist_C73F = 51007, addr_stru_A8C6E9 = 50921,
    addr_kWreckedShipRobot_Ilist_C6E5 = 50917, addr_stru_A8C199 = 49561,
    addr_kMaridiaFloater_Ilist_C191 = 49553, addr_kMaridiaFloater_Ilist_C183 = 49539,
    addr_kMaridiaFloater_Ilist_C173 = 49523, addr_kMaridiaFloater_Ilist_C163 = 49507,
    addr_kBeetom_Ilist_B728 = 46888, addr_kBeetom_Ilist_B708 = 46856,
    addr_kBeetom_Ilist_B6F2 = 46834, addr_kBeetom_Ilist_B6CC = 46796,
    addr_kBeetom_Ilist_B6AC = 46764, addr_kBeetom_Ilist_B696 = 46742,
    addr_kNorfairLavaMan_Ilist_AE0C = 44556, addr_kNorfairLavaMan_Ilist_ADDC = 44508,
    addr_kNorfairLavaMan_Ilist_ADAC = 44460, addr_kNorfairLavaMan_Ilist_AD7E = 44414,
    addr_kNorfairLavaMan_Ilist_AD50 = 44368, addr_kNorfairLavaMan_Ilist_AD3C = 44348,
    addr_kNorfairLavaMan_Ilist_AD0C = 44300, addr_kNorfairLavaMan_Ilist_ACDE = 44254,
    addr_kNorfairLavaMan_Ilist_ACB0 = 44208, addr_kNorfairLavaMan_Ilist_AC9C = 44188,
    addr_kKago_Ilist_AB32 = 43826, addr_kKago_Ilist_AB1E = 43806,
    addr_kYappingMaw_Ilist_A079 = 41081, addr_off_A8A061 = 41057,
    addr_kYappingMaw_Ilist_A05B = 41051, addr_kYappingMaw_Ilist_A03D = 41021,
    addr_off_A8A025 = 40997, addr_kYappingMaw_Ilist_A01F = 40991,
    addr_kYappingMaw_Ilist_9FC7 = 40903, addr_kYappingMaw_Ilist_9F6F = 40815,
    addr_loc_A89E90 = 40592, addr_kWreckedShipGhost_Ilist_9A8C = 39564,
    addr_off_A896DF = 38623, addr_off_A896D7 = 38615,
    addr_kMorphBallEye_Ilist_9026 = 36902, addr_kMorphBallEye_Ilist_9014 = 36884,
    addr_stru_A8900E = 36878, addr_kMorphBallEye_Ilist_9002 = 36866,
    addr_stru_A88FFC = 36860, addr_kMorphBallEye_Ilist_8FF0 = 36848,
    addr_kMiniDraygon_Ilist_8775 = 34677, addr_kMiniDraygon_Ilist_876F = 34671,
    addr_kMiniDraygon_Ilist_8727 = 34599, addr_kMiniDraygon_Ilist_870B = 34571,
    addr_kMiniDraygon_Ilist_86C3 = 34499, addr_kMiniDraygon_Ilist_86A7 = 34471,
    addr_kSpritemap_Nothing_A8 = 32845,
}
pub type Enemy_Fune = crate::game::enemy_layouts::EnemyLayout40;

pub type Enemy_KiHunter = crate::game::enemy_layouts::EnemyLayout40;

pub type Enemy_MiniDraygon = crate::game::enemy_layouts::EnemyLayout40;

pub type Enemy_NorfairLavaMan = crate::game::enemy_layouts::EnemyLayout40;

pub type Enemy_WalkingLavaSeahorse = crate::game::enemy_layouts::EnemyLayout40;

pub type Enemy_WreckedShipGhost = crate::game::enemy_layouts::EnemyLayout40;

pub type Enemy_WreckedShipRobot = crate::game::enemy_layouts::EnemyLayout8;

pub type Enemy_WreckedShipSpark = crate::game::enemy_layouts::EnemyLayout8;

pub type Enemy_YappingMaw = crate::game::enemy_layouts::EnemyLayout104;

pub type Enemy_Beetom = crate::game::enemy_layouts::EnemyLayout40;

pub type Enemy_Kago = crate::game::enemy_layouts::EnemyLayout40;

pub type Enemy_MaridiaFloater = crate::game::enemy_layouts::EnemyLayout8;

pub type Enemy_MaridiaPuffer = crate::game::enemy_layouts::EnemyLayout72Late;

pub type Enemy_MorphBallEye = crate::game::enemy_layouts::EnemyLayout8;

pub type Enemy_WreckedShipOrbs = crate::game::enemy_layouts::EnemyLayout40;

const RomPtr_A8: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0xa8>;
crate::ram_map::ram_pointer_functions! {
    gVramWriteEntry(a: i32) -> VramWriteEntry = [(0xd0i32 + a) as isize];
}
crate::game::constants::typed_constants! { i32;
    fnnullsub_152 = 0x90e90e, fnSamus_InputHandler_E913 = 0x90e913,
    fnSamus_InputHandler_E91D = 0x90e91d, fnnullsub_170_A8 = 0xa8804c,
    fnMorphBallEye_Func_1 = 0xa890f1, fnMorphBallEye_Func_2 = 0xa8912e,
    fnMorphBallEye_Func_3 = 0xa89160, fnMorphBallEye_Func_4 = 0xa891ce,
    fnnullsub_244 = 0xa891dc,
    fnNorfairLavaMan_Func_6 = 0xa8b0b2,
    fnNorfairLavaMan_Func_7 = 0xa8b11a, fnNorfairLavaMan_Func_8 = 0xa8b175,
    fnsub_A8B193 = 0xa8b193, fnNorfairLavaMan_Func_9 = 0xa8b1b8,
    fnNorfairLavaMan_Func_10 = 0xa8b1dd, fnNorfairLavaMan_Func_11 = 0xa8b204,
    fnsub_A8B291 = 0xa8b291, fnNorfairLavaMan_Func_13 = 0xa8b295,
    fnNorfairLavaMan_Func_15 = 0xa8b30d, fnNorfairLavaMan_Func_16 = 0xa8b31f,
    fnNorfairLavaMan_Func_17 = 0xa8b356, fnNorfairLavaMan_Func_18 = 0xa8b3a7,
}
crate::game::constants::typed_constants! { i32;
    fnMaridiaFloater_Func_3 = 0xa8c283, fnMaridiaFloater_Func_4 = 0xa8c2a6,
    fnMaridiaFloater_Func_5 = 0xa8c2cf, fnMaridiaFloater_Func_6 = 0xa8c36b,
    fnMaridiaFloater_Func_7 = 0xa8c3e1, fnMaridiaFloater_Func_8 = 0xa8c469,
    fnMaridiaFloater_Func_9 = 0xa8c4dc, fnMaridiaFloater_Func_10 = 0xa8c500,
    fnMaridiaFloater_Func_11 = 0xa8c51d, fnnullsub_256 = 0xa8c568,
    fnMaridiaFloater_Func_12 = 0xa8c569, fnMaridiaFloater_Func_13 = 0xa8c59f,
    fnWreckedShipRobot_Func_1 = 0xa8cc67,
}
crate::game::constants::typed_constants! { i32;
    fnBlueBrinstarFaceBlock_Func_1 = 0xa8e86e,
}
crate::ram_map::ram_pointer_functions! {
    gEnemyData(o: i32) -> EnemyData = [0xf78, o as isize];
    gEnemySpawnData(o: i32) -> EnemySpawnData = [0x7000, o as isize];
    Get_Beetom(o: i32) -> Enemy_Beetom = [0xf78, o as isize];
    Get_Fune(o: i32) -> Enemy_Fune = [0xf78, o as isize];
    Get_Kago(o: i32) -> Enemy_Kago = [0xf78, o as isize];
    Get_KiHunter(o: i32) -> Enemy_KiHunter = [0xf78, o as isize];
    Get_MaridiaFloater(o: i32) -> Enemy_MaridiaFloater = [0xf78, o as isize];
    Get_MaridiaPuffer(o: i32) -> Enemy_MaridiaPuffer = [0xf78, o as isize];
    Get_MiniDraygon(o: i32) -> Enemy_MiniDraygon = [0xf78, o as isize];
    Get_MorphBallEye(o: i32) -> Enemy_MorphBallEye = [0xf78, o as isize];
    Get_NorfairLavaMan(o: i32) -> Enemy_NorfairLavaMan = [0xf78, o as isize];
    Get_WalkingLavaSeahorse(o: i32) -> Enemy_WalkingLavaSeahorse = [0xf78, o as isize];
    Get_WreckedShipGhost(o: i32) -> Enemy_WreckedShipGhost = [0xf78, o as isize];
    Get_WreckedShipOrbs(o: i32) -> Enemy_WreckedShipOrbs = [0xf78, o as isize];
    Get_WreckedShipRobot(o: i32) -> Enemy_WreckedShipRobot = [0xf78, o as isize];
    Get_WreckedShipSpark(o: i32) -> Enemy_WreckedShipSpark = [0xf78, o as isize];
    Get_YappingMaw(o: i32) -> Enemy_YappingMaw = [0xf78, o as isize];
}
static g_word_A890CA: [i16; 4] = [-8, 8, 0, 0];
static g_word_A890D2: [i16; 4] = [0, 0, -8, 8];
static g_word_A89050: u16 = 0x80u16;
static g_word_A89052: u16 = 0xb0u16;
static g_word_A89054: u16 = 0x80u16;
static g_word_A89056: u16 = 0x80u16;
static g_word_A89A9C: u16 = 0x10u16;
static g_word_A89A9E: u16 = 0x40u16;
static g_word_A89AA0: u16 = 0x1800u16;
static g_word_A89AA2: u16 = 1u16;
static g_word_A89AA4: u16 = 0x78u16;
static g_word_A89AA6: u16 = 0x78u16;
static g_word_A89AA8: [u16; 18] = [
    0xffc0, 0xffc0, 0, 0xffc0, 0x40, 0, 0xffc0, 0, 0, 0, 0x40, 0, 0xffc0, 0x40, 0, 0x40, 0x40, 0x40,
];
static g_word_A89ACC: [u16; 17] = [1, 8, 1, 8, 1, 7, 1, 7, 2, 6, 2, 6, 3, 5, 3, 5, 0xffff];
static kWreckedShipGhost_Palette: [u16; 16] = [
    0x3800, 0x57ff, 0x42f7, 0x929, 0xa5, 0x4f5a, 0x36b5, 0x2610, 0x1dce, 0x1df, 0x1f, 0x18, 0xa,
    0x6b9, 0xea, 0x45,
];
static g_word_A89D32: u16 = 1u16;
static g_word_A89D34: u16 = 1u16;
static g_word_A8C277: [i16; 3] = [-12, -16, -20];
static g_word_A8C27D: [i16; 3] = [-20, -16, -12];
static g_word_A8C19F: u16 = 0x40u16;
static g_word_A8C1A1: [u16; 12] = [0, 1, 2, 3, 2, 1, 0, 0xffff, 0xfffe, 0xfffd, 0xfffe, 0xffff];
static g_word_A8C1B9: u16 = 0u16;
static g_word_A8C1BB: u16 = 0u16;
static g_word_A8C1BD: u16 = 0u16;
static g_word_A8C1BF: u16 = 0u16;
static g_word_A8C1C1: u16 = 1u16;
static g_word_A8C1C3: u16 = 0u16;
static g_word_A8C1C5: u16 = 0xffffu16;
static g_word_A8C1C7: u16 = 0x8000u16;
static g_word_A8CCC1: [i16; 31] = [
    0x1f, 0x18, 0xf, 8, 0x40, 0x18, 0xf, 8, 0x1f, 0x10, 0xf, 8, 0x1f, 0x18, 0x10, 8, 0x1f, 0x18,
    0xf, 0x40, 0xf, 8, 0x1f, 0x18, 0x10, 0x18, 0xf, 8, 0x1f, 0x10, -1,
];
static g_word_A8D871: [i16; 10] = [0xc0, 0xe0, 0, 0x20, 0x40, 0x40, 0x60, 0x80, 0xa0, 0xc0];
static g_word_A8D885: [i16; 8] = [0x3ff, 0x4ff, 0x5ff, 0x6ff, 0x7ff, 0x8ff, 0x9ff, 0xaff];
static g_word_A8D895: [i16; 26] = [
    3, 1, 4, 1, 5, 2, 6, 2, 7, 2, 8, 3, 9, 3, 0xa, 4, 0xb, 4, 0xc, 5, 0xd, 5, 0xe, 6, 0xf, 6,
];
static g_word_A8DCC7: u16 = 0x50u16;
static g_word_A8DCCB: u16 = 0x70u16;
static g_word_A8F180: u16 = 0x60u16;
static g_word_A8F182: u16 = 0xe000u16;
static g_word_A8F184: u16 = 0u16;
static g_byte_A8F186: u8 = 0x30u8;
crate::forward_calls! { pub unsafe fn (game: &mut Game);
    Enemy_GrappleReact_NoInteract_A8 => SwitchEnemyAiToMainAi(game);
    Enemy_GrappleReact_KillEnemy_A8 => EnemyGrappleDeath(game);
    Enemy_GrappleReact_CancelBeam_A8 => Enemy_SwitchToFrozenAi(game);
    Enemy_GrappleReact_SamusLatchesNoInvinc_A8 => SamusLatchesOnWithGrappleNoInvinc(game);
    Enemy_GrappleReact_HurtSamus_A8 => SamusHurtFromGrapple(game);
    Enemy_NormalTouchAI_A8 => NormalEnemyTouchAi(game);
    Enemy_NormalShotAI_A8 => NormalEnemyShotAi(game);
    Enemy_NormalPowerBombAI_A8 => NormalEnemyPowerBombAi(game);
    Enemy_NormalFrozenAI_A8 => NormalEnemyFrozenAI(game);
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16;
    MiniDraygon_Instr_2 [jp] { QueueSfx2_Max6(0x5eu16, game); };
    MiniDraygon_Instr_1 [jp] {
        if (*gEnemySpawnData(ram_word!(game, cur_enemy_index) as i32, game).offset(30)).cause_of_death
            as i32
            == addr_kMiniDraygon_Ilist_870B as i32
        {
            (*Get_MiniDraygon(ram_word!(game, cur_enemy_index) as i32, game)).var_F = -8i32 as u16;
        } else {
            (*Get_MiniDraygon(ram_word!(game, cur_enemy_index) as i32, game)).var_F = 8u16;
        }
    };
    MiniDraygon_Instr_3 [jp] {
        let E = Get_MiniDraygon(ram_word!(game, cur_enemy_index) as i32, game);
        if (*gEnemySpawnData(ram_word!(game, cur_enemy_index) as i32, game).offset(30)).cause_of_death
            as i32
            == addr_kMiniDraygon_Ilist_870B as i32
        {
            (*E).var_F = (*E).var_F.wrapping_add(1);
        } else {
            (*E).var_F = (*E).var_F.wrapping_sub(1);
        }
    };
    MiniDraygon_Instr_4 [jp] {
        let E = Get_MiniDraygon(ram_word!(game, cur_enemy_index) as i32, game);
        set_fields! {[(*E)], 0u16; var_0C, var_0B}
        (*E).var_C = ((0xa88a34i32) as u16);
    };
}
pub unsafe fn MiniDraygon_Init(game: &mut Game) {
    let E = Get_MiniDraygon(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).parameter_1 != 0 {
        MiniDraygon_Func_2(game);
        (*E).base.layer = 4u16;
    } else {
        MiniDraygon_Func_1(game);
        let v2: u16 =
            (8u16).wrapping_mul((*(&raw mut (*E).parameter_2 as *mut u8).offset(0)) as u16);
        let v3: i32 = v2 as i32 >> 1i32;
        (*E).var_04 = crate::rtl::rom_table_word!(game, 0xa28187, v3 as isize);
        (*E).var_03 = crate::rtl::rom_table_word!(game, 0xa28187, (v3 + 1i32) as isize);
        (*E).var_06 = crate::rtl::rom_table_word!(game, 0xa28187, (v3 + 2i32) as isize);
        (*E).var_05 = crate::rtl::rom_table_word!(game, 0xa28187, (v3 + 3i32) as isize);
        (*E).var_E = (*(&raw mut (*(Get_MiniDraygon
            as unsafe fn(i32, &mut Game) -> *mut Enemy_MiniDraygon)(
            v2 as i32, game
        ))
        .parameter_2 as *mut u8)
            .offset(1isize)
            >> 1) as u16;
    }
    set_fields! {[(*E)], 0u16; var_00, var_01}
    (*E).var_C = ((0xa88922i32) as u16);
}
pub unsafe fn MiniDraygon_Func_1(game: &mut Game) {
    (*Get_MiniDraygon(ram_word!(game, cur_enemy_index) as i32, game)).var_B =
        ((GetSamusEnemyDelta_X(ram_word!(game, cur_enemy_index), game) & 0x8000) == 0) as u16;
    let E = Get_MiniDraygon(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).var_02 = addr_kMiniDraygon_Ilist_86A7 as u16;
    if (*E).var_B != 0 {
        (*E).var_02 = addr_kMiniDraygon_Ilist_870B as u16;
    }
    MiniDraygon_Func_12(game);
}
pub unsafe fn MiniDraygon_Func_2(game: &mut Game) {
    let v0: i32 = ram_word!(game, cur_enemy_index) as i32 >> 1i32;
    let v1: u16 = ram_word!(game, enemy_drawing_queue_sizes[(v0 + 1i32)]);
    let E = Get_MiniDraygon(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).var_B = v1;
    if v1 != 0 {
        (*E).base.x_pos = (ram_word!(game, enemy_drawing_queue[(v0 + 91i32)])).wrapping_add(4u16);
        (*E).base.y_pos = (ram_word!(game, enemy_drawing_queue[(v0 + 93i32)])).wrapping_add(10u16);
        (*E).var_02 = addr_kMiniDraygon_Ilist_8727 as u16;
    } else {
        (*E).base.x_pos = (ram_word!(game, enemy_drawing_queue[(v0 + 91i32)])).wrapping_sub(4u16);
        (*E).base.y_pos = (ram_word!(game, enemy_drawing_queue[(v0 + 93i32)])).wrapping_add(10u16);
        (*E).var_02 = addr_kMiniDraygon_Ilist_86C3 as u16;
    }
    MiniDraygon_Func_12(game);
}
pub unsafe fn EvirProjectile_Init(game: &mut Game) {
    let E = Get_MiniDraygon(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).var_02 = addr_kMiniDraygon_Ilist_876F as u16;
    MiniDraygon_Func_12(game);
    let v2: i32 = ram_word!(game, cur_enemy_index) as i32 >> 1i32;
    (*E).base.palette_index = ram_word!(game, enemy_drawing_queue[(v2 + 73i32)]);
    (*E).base.vram_tiles_index = ram_word!(game, enemy_drawing_queue[(v2 + 74i32)]);
    MiniDraygon_Func_3(game);
    set_fields! {[(*E)], 0u16; var_01, var_0C, var_0B, var_F}
    (*E).var_C = ((0xa88a34i32) as u16);
}
pub unsafe fn MiniDraygon_Func_3(game: &mut Game) {
    let v0: i32 = ram_word!(game, cur_enemy_index) as i32 >> 1i32;
    let v1: u16 = ram_word!(game, enemy_drawing_queue[(v0 + 83i32)]);
    let E = Get_MiniDraygon(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).var_B = v1;
    if v1 != 0 {
        (*E).base.x_pos = (ram_word!(game, enemy_drawing_queue[(v0 + 59i32)])).wrapping_add(4u16);
    } else {
        (*E).base.x_pos = (ram_word!(game, enemy_drawing_queue[(v0 + 59i32)])).wrapping_sub(4u16);
    }
    (*E).base.y_pos = (ram_word!(game, enemy_drawing_queue[(v0 + 61i32)])).wrapping_add(18u16);
}
crate::game::address_dispatch! {
    pub unsafe fn CallMiniDraygonFunc(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        11045154 => MiniDraygon_Func_4; 11045428 => MiniDraygon_Func_8;
        11045435 => MiniDraygon_Func_9; 11045496 => MiniDraygon_Func_10;
    }
}
pub unsafe fn MiniDraygon_Main(game: &mut Game) {
    let E = Get_MiniDraygon(ram_word!(game, cur_enemy_index) as i32, game);
    CallMiniDraygonFunc((((*E).var_C) as u32) | (0xa80000u32), game);
}
pub unsafe fn MiniDraygon_Func_4(game: &mut Game) {
    if (*Get_MiniDraygon(ram_word!(game, cur_enemy_index) as i32, game)).parameter_1 != 0 {
        MiniDraygon_Func_6(game);
    } else {
        MiniDraygon_Func_5(game);
    };
}
pub unsafe fn MiniDraygon_Func_5(game: &mut Game) {
    if (*Get_MiniDraygon(ram_word!(game, cur_enemy_index) as i32 + 128i32, game)).var_0C == 0 {
        MiniDraygon_Func_1(game);
    }
    let E = Get_MiniDraygon(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).var_00 != 0 {
        AddToHiLo(
            &raw mut (*E).base.y_pos,
            &raw mut (*E).base.y_subpos,
            ((*E).var_04 as u32) << 16i32 | (*E).var_03 as u32,
        );
    } else {
        AddToHiLo(
            &raw mut (*E).base.y_pos,
            &raw mut (*E).base.y_subpos,
            ((*E).var_06 as u32) << 16i32 | (*E).var_05 as u32,
        );
    }
    (*E).var_E = (*E).var_E.wrapping_sub(1);
    if ((*E).var_E & 0x8000) != 0 {
        (*E).var_E = *(&raw mut (*E).parameter_2 as *mut u8).offset(1isize) as u16;
        (*E).var_00 = ((*E).var_00) ^ (1u16);
    }
}
pub unsafe fn MiniDraygon_Func_6(game: &mut Game) {
    MiniDraygon_Func_2(game);
}
pub unsafe fn EvirProjectile_Main(game: &mut Game) {
    let E = Get_MiniDraygon(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).base.frozen_timer == 0 {
        if (*E).var_0B != 0 {
            (*E).var_02 = addr_kMiniDraygon_Ilist_876F as u16;
            MiniDraygon_Func_12(game);
        } else if (*E).var_0C != 0 {
            (*E).var_02 = addr_kMiniDraygon_Ilist_8775 as u16;
            MiniDraygon_Func_12(game);
        } else {
            MiniDraygon_Func_7(game);
        }
    }
    CallMiniDraygonFunc((((*E).var_C) as u32) | (0xa80000u32), game);
}
pub unsafe fn MiniDraygon_Func_7(game: &mut Game) {
    if IsSamusWithinEnemy_X(
        (ram_word!(game, cur_enemy_index)).wrapping_sub(128u16),
        0x80u16,
        game,
    ) != 0
    {
        let r22: u16 = -((((CalculateAngleOfSamusFromEnemy(ram_word!(game, cur_enemy_index), game))
            as u8)
            .wrapping_sub(64u8)) as i32) as u16;
        let E = Get_MiniDraygon(ram_word!(game, cur_enemy_index) as i32, game);
        SetHiLo(
            &raw mut (*E).var_07,
            &raw mut (*E).var_08,
            CosineMult8bitFull(r22, 4u16, game),
        );
        SetHiLo(
            &raw mut (*E).var_09,
            &raw mut (*E).var_0A,
            SineMult8bitFull(r22, 4u16, game),
        );
        (*E).var_02 = addr_kMiniDraygon_Ilist_876F as u16;
        MiniDraygon_Func_12(game);
        (*E).var_0B = 1u16;
        (*E).var_C = ((0xa88a3bi32) as u16);
    }
}
pub unsafe fn MiniDraygon_Func_8(game: &mut Game) {
    MiniDraygon_Func_3(game);
}
pub unsafe fn MiniDraygon_Func_9(game: &mut Game) {
    MiniDraygon_Func_11(game);
    let E = Get_MiniDraygon(ram_word!(game, cur_enemy_index) as i32, game);
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        ((*E).var_07 as u32) << 16i32 | (*E).var_08 as u32,
    );
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        ((*E).var_09 as u32) << 16i32 | (*E).var_0A as u32,
    );
}
pub unsafe fn MiniDraygon_Func_10(game: &mut Game) {
    if *(ram_ptr!(game, u16; 0xe84))
        .offset(((ram_word!(game, cur_enemy_index) as i32 >> 1i32) + 77i32) as isize)
        == 0
    {
        let E = Get_MiniDraygon(ram_word!(game, cur_enemy_index) as i32, game);
        if (*E).var_0C != 0 {
            MiniDraygon_Func_3(game);
            (*E).base.x_pos = ((*E).base.x_pos).wrapping_add((*E).var_F);
        } else {
            (*E).var_02 = addr_kMiniDraygon_Ilist_876F as u16;
            MiniDraygon_Func_12(game);
            set_fields! {[(*E)], 0u16; var_0C, var_0B}
            (*E).var_C = ((0xa88a34i32) as u16);
        }
    }
}
pub unsafe fn MiniDraygon_Func_11(game: &mut Game) {
    if EnemyFunc_ADA3(0x100u16, game) != 0
        && *(ram_ptr!(game, u16; 0xe84))
            .offset(((ram_word!(game, cur_enemy_index) as i32 >> 1i32) + 77i32) as isize)
            == 0
    {
        let E = Get_MiniDraygon(ram_word!(game, cur_enemy_index) as i32, game);
        (*E).var_0B = 0u16;
        (*E).var_0C = 1u16;
        (*E).var_C = ((0xa88a78i32) as u16);
        (*E).var_0C = 1u16;
        (*E).var_02 = addr_kMiniDraygon_Ilist_8775 as u16;
        MiniDraygon_Func_12(game);
    }
}
pub unsafe fn MiniDraygon_Func_12(game: &mut Game) {
    let E = Get_MiniDraygon(ram_word!(game, cur_enemy_index) as i32, game);
    let mdn_var_02: u16 = (*E).var_02;
    if mdn_var_02 != (*E).var_01 {
        (*E).base.current_instruction = mdn_var_02;
        (*E).var_01 = mdn_var_02;
        (*E).base.instruction_timer = 1u16;
        (*E).base.timer = 0u16;
    }
}
pub unsafe fn MiniDraygon_Touch(game: &mut Game) {
    Enemy_NormalTouchAI_A8(game);
    MiniDraygon_Func_13(game);
}
pub unsafe fn MiniDraygon_Powerbomb(game: &mut Game) {
    Enemy_NormalPowerBombAI_A8(game);
    MiniDraygon_Func_13(game);
}
pub unsafe fn MiniDraygon_Shot(game: &mut Game) {
    NormalEnemyShotAi(game);
    MiniDraygon_Func_13(game);
}
pub unsafe fn MiniDraygon_Func_13(game: &mut Game) {
    let E = Get_MiniDraygon(ram_word!(game, cur_enemy_index) as i32, game);
    let E1 = Get_MiniDraygon(ram_word!(game, cur_enemy_index) as i32 + 64i32, game);
    let E2 = Get_MiniDraygon(ram_word!(game, cur_enemy_index) as i32 + 128i32, game);
    if (*E).base.health == 0 {
        (*E1).base.properties = ((*E1).base.properties) | (0x200u16);
        (*E2).base.properties = ((*E2).base.properties) | (0x200u16);
    }
    let frozen_timer: u16 = (*E).base.frozen_timer;
    if frozen_timer != 0 {
        (*E1).base.frozen_timer = frozen_timer;
        (*E1).base.ai_handler_bits = ((*E1).base.ai_handler_bits) | (4u16);
        if (*E2).var_C as i32 != 0xa88a3bi32 & 0xffffi32 {
            (*E2).base.ai_handler_bits = ((*E2).base.ai_handler_bits) | (4u16);
            (*E2).base.frozen_timer = (*E).base.frozen_timer;
        }
    }
}
pub unsafe fn MorphBallEye_Init(game: &mut Game) {
    let E = Get_MorphBallEye(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_DisableSamusColl) as u16);
    (*E).base.spritemap_pointer = addr_kSpritemap_Nothing_A8 as u16;
    (*E).base.instruction_timer = 1u16;
    (*E).base.timer = 0u16;
    if ((*E).parameter_2 & 0x8000) != 0 {
        let v1: i32 = (*E).parameter_2 as i32 & 0xfi32;
        (*E).base.x_pos = ((*E).base.x_pos).wrapping_add((g_word_A890CA[v1 as usize]) as u16);
        (*E).base.y_pos = ((*E).base.y_pos).wrapping_add((g_word_A890D2[v1 as usize]) as u16);
        (*E).var_F = ((0xa891dci32) as u16);
        (*E).base.current_instruction = crate::rtl::rom_table_word!(game, 0xa890da, v1 as isize);
        let mut i: i32 = 510i32;
        while i >= 0i32 {
            *((ram_ptr!(game, u16; 0x9100) as *mut u8).offset(i as u16 as i32 as isize)
                as *mut u16) = 255u16;
            i -= 2i32;
        }
    } else {
        (*E).var_F = ((0xa890f1i32) as u16);
        if ((*E).parameter_1 & 1) != 0 {
            (*E).base.current_instruction = addr_stru_A8900E as u16;
        } else {
            (*E).base.current_instruction = addr_stru_A88FFC as u16;
        }
    };
}
pub unsafe fn MorphBallEye_Main(game: &mut Game) {
    if (ram_word!(game, collected_items) & 4) != 0 {
        let E = Get_MorphBallEye(ram_word!(game, cur_enemy_index) as i32, game);
        CallEnemyPreInstr((((*E).var_F) as u32) | (0xa80000u32), game);
    }
}
pub unsafe fn MorphBallEye_Func_1(k: u16, game: &mut Game) {
    if IsSamusWithinEnemy_Y(k, g_word_A89054, game) != 0u16
        && IsSamusWithinEnemy_X(k, g_word_A89050, game) != 0u16
    {
        let E = Get_MorphBallEye(k as i32, game);
        (*E).var_E = 32u16;
        (*E).base.instruction_timer = 1u16;
        if ((*E).parameter_1 & 1) != 0 {
            (*E).base.current_instruction = addr_kMorphBallEye_Ilist_9026 as u16;
        } else {
            (*E).base.current_instruction = addr_kMorphBallEye_Ilist_9014 as u16;
        }
        (*E).var_F = ((0xa8912ei32) as u16);
    }
}
pub unsafe fn MorphBallEye_Func_2(k: u16, game: &mut Game) {
    let E = Get_MorphBallEye(k as i32, game);
    let v2: bool = (*E).var_E == 1u16;
    (*E).var_E = (*E).var_E.wrapping_sub(1);
    let v3: bool = ((*E).var_E & 0x8000) != 0;
    if v2 || v3 {
        QueueSfx2_Max6(0x17u16, game);
        SpawnMorphBallEyeBeamHdma(game);
        (*E).var_F = ((0xa89160i32) as u16);
        (*E).var_D = CalculateAngleFromXY(
            (ram_word!(game, samus_x_pos)).wrapping_sub((*E).base.x_pos),
            (ram_word!(game, samus_y_pos)).wrapping_sub((*E).base.y_pos),
            game,
        );
    }
}
pub unsafe fn MorphBallEye_Func_3(k: u16, game: &mut Game) {
    let E = Get_MorphBallEye(k as i32, game);
    if IsSamusWithinEnemy_Y(k, g_word_A89056, game) != 0u16
        && IsSamusWithinEnemy_X(k, g_word_A89052, game) != 0u16
    {
        let v3: u16 = CalculateAngleFromXY(
            (ram_word!(game, samus_x_pos)).wrapping_sub((*E).base.x_pos),
            (ram_word!(game, samus_y_pos)).wrapping_sub((*E).base.y_pos),
            game,
        );
        (*E).var_D = v3;
        (*E).base.current_instruction = (((v3 as i32 & 0xf0i32) >> 2i32) - 28756i32) as u16;
    } else {
        QueueSfx2_Max6(0x71u16, game);
        (*E).var_C = 0u16;
        (*E).var_E = 32u16;
        if ((*E).parameter_1 & 1) != 0 {
            (*E).base.current_instruction = addr_kMorphBallEye_Ilist_9002 as u16;
        } else {
            (*E).base.current_instruction = addr_kMorphBallEye_Ilist_8FF0 as u16;
        }
        (*E).var_F = ((0xa891cei32) as u16);
    }
    (*E).base.instruction_timer = 1u16;
}
pub unsafe fn MorphBallEye_Func_4(k: u16, game: &mut Game) {
    let E = Get_MorphBallEye(k as i32, game);
    let v2: bool = (*E).var_E == 1u16;
    (*E).var_E = (*E).var_E.wrapping_sub(1);
    let v3: bool = ((*E).var_E & 0x8000) != 0;
    if v2 || v3 {
        (*E).var_F = ((0xa890f1i32) as u16);
    }
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16;
    Fune_Instr_2 [jp] { QueueSfx2_Max9(0x1fu16, game); };
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16;
    @prefix {
        set_ram_word!(
            game.cinematic_var9 = *(&raw mut (*(Get_Fune
                as unsafe fn(i32, &mut Game) -> *mut Enemy_Fune)(
                ram_word!(game, cur_enemy_index) as i32,
                game,
            ))
            .parameter_2 as *mut u8)
                .offset(0) as u16
        );
    };
    Fune_Instr_6 [jp] {

        SpawnEprojWithGfx(
            0u16,
            ram_word!(game, cur_enemy_index),
            addr_stru_86DFBC as u16,
            game,
        );
    };
    Fune_Instr_7 [jp] {

        SpawnEprojWithGfx(
            1u16,
            ram_word!(game, cur_enemy_index),
            addr_stru_86DFBC as u16,
            game,
        );
    };
    Fune_Instr_1 [jp] {

        SpawnEprojWithGfx(
            0u16,
            ram_word!(game, cur_enemy_index),
            addr_stru_86DFCA as u16,
            game,
        );
    };
    Fune_Instr_4 [jp] {

        SpawnEprojWithGfx(
            1u16,
            ram_word!(game, cur_enemy_index),
            addr_stru_86DFCA as u16,
            game,
        );
    };
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16;
    Fune_Instr_3 [jp] {
        let E = Get_Fune(ram_word!(game, cur_enemy_index) as i32, game);
        (*E).var_A = ((*E).var_A).wrapping_add(4u16);
        (*E).var_B = ((0xa89737i32) as u16);
        if (*E).var_D != 0 {
            (*E).var_B = ((0xa8975ci32) as u16);
        }
    };
    Fune_Instr_5 [jp] {
        let E = Get_Fune(ram_word!(game, cur_enemy_index) as i32, game);
        (*E).var_A = ((*E).var_A).wrapping_add(4u16);
        (*E).var_B = ((0xa89737i32) as u16);
        if (*E).var_D != 0 {
            (*E).var_B = ((0xa8975ci32) as u16);
        }
    };
}
pub unsafe fn Fune_Init(game: &mut Game) {
    let E = Get_Fune(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).var_A = addr_off_A896D7 as u16;
    (*E).var_B = ((0xa89737i32) as u16);
    let v1: u16 = ((*E).parameter_1) & (0xfu16);
    (*E).var_D = v1;
    if v1 != 0 {
        (*E).var_A = addr_off_A896DF as u16;
        (*E).var_B = ((0xa8975ci32) as u16);
    }
    if ((*E).parameter_1 & 0xf0) != 0 {
        (*E).var_A = ((*E).var_A).wrapping_add(2u16);
    }
    Fune_Func_4(game);
    (*E).var_C = *(&raw mut (*E).parameter_2 as *mut u8).offset(1isize) as u16;
    (*E).var_F = *(&raw mut (*E).parameter_1 as *mut u8).offset(1isize) as u16;
    (*E).var_E = 0u16;
}
crate::game::address_dispatch! {
    pub unsafe fn CallFuneFunc(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        11048759 => Fune_Func_1; 11048796 => Fune_Func_2; 11048846 => [()]; 11048847 => [()];
    }
}
pub unsafe fn Fune_Main(game: &mut Game) {
    let E = Get_Fune(ram_word!(game, cur_enemy_index) as i32, game);
    CallFuneFunc((((*E).var_B) as u32) | (0xa80000u32), game);
}
pub unsafe fn Fune_Func_1(game: &mut Game) {
    let E = Get_Fune(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).var_E = (*E).var_E.wrapping_add(1);
    if (((*E).var_E) as i16).wrapping_sub(((*E).var_F) as i16) >= 0i16 {
        (*E).var_A = ((*E).var_A).wrapping_sub(4u16);
        Fune_Func_4(game);
        (*E).var_B = ((0xa8978ei32) as u16);
        (*E).var_E = 0u16;
    }
}
pub unsafe fn Fune_Func_2(game: &mut Game) {
    let E = Get_Fune(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).var_00 = (*E).var_A;
    if Fune_Func_3(game) != 0 {
        let mut fune_var_00: u16 = (*E).var_00;
        (*E).var_A = fune_var_00;
        fune_var_00 = (fune_var_00).wrapping_sub(4u16);
        (*E).var_00 = fune_var_00;
        (*E).var_A = fune_var_00;
        Fune_Func_4(game);
        (*E).var_B = ((0xa8978fi32) as u16);
    }
    (*E).var_A = (*E).var_00;
}
pub unsafe fn Fune_Func_3(game: &mut Game) -> u16 {
    let E = Get_Fune(ram_word!(game, cur_enemy_index) as i32, game);
    IsSamusWithinEnemy_Y(ram_word!(game, cur_enemy_index), (*E).var_C, game)
}
pub unsafe fn Fune_Func_4(game: &mut Game) {
    let E = Get_Fune(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.instruction_timer = 1u16;
    (*E).base.timer = 0u16;
    (*E).base.current_instruction = *(RomPtr_A8((*E).var_A, game) as *mut u16);
}
pub unsafe fn WreckedShipGhost_Init(game: &mut Game) {
    let E = Get_WreckedShipGhost(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.properties = ((*E).base.properties as i32
        | (kEnemyProps_DisableSamusColl as i32
            | kEnemyProps_Tangible as i32
            | kEnemyProps_Invisible as i32)) as u16;
    (*E).base.instruction_timer = 1u16;
    (*E).base.timer = 0u16;
    (*E).base.current_instruction = addr_kWreckedShipGhost_Ilist_9A8C as u16;
    (*E).var_A = ((0xa89d13i32) as u16);
    (*E).var_B = (g_word_A89AA4).wrapping_add(160u16);
    let v2: u16 = u16::swap_bytes((*E).base.palette_index);
    let mut v3: u16 = (16i32 * v2 as i32 + 256i32) as u16;
    for _ in 0i32..=16i32 {
        set_ram_word!(game.target_palettes[(v3 >> 1)] = 0u16);
        v3 = (v3).wrapping_add(2u16);
    }
}
crate::game::address_dispatch! {
    pub unsafe fn CallWreckedShipGhost(ea: u32, k: u16, game: &mut Game) => ea;
        call (k, game); default { Unreachable(game); };
    {
        11049794 => WreckedShipGhost_Func_1; 11049901 => WreckedShipGhost_Func_2;
        11050089 => WreckedShipGhost_Func_4; 11050122 => WreckedShipGhost_Func_5;
        11050259 => WreckedShipGhost_Func_6; 11050294 => WreckedShipGhost_Func_7;
    }
}
pub unsafe fn WreckedShipGhost_Main(game: &mut Game) {
    let E = Get_WreckedShipGhost(ram_word!(game, cur_enemy_index) as i32, game);
    CallWreckedShipGhost(
        (((*E).var_A) as u32) | (0xa80000u32),
        ram_word!(game, cur_enemy_index),
        game,
    );
}
pub unsafe fn WreckedShipGhost_Func_1(k: u16, game: &mut Game) {
    WreckedShipGhost_Func_3(k, game);
    let E = Get_WreckedShipGhost(ram_word!(game, cur_enemy_index) as i32, game);
    let v3: u16 = u16::swap_bytes((*E).base.palette_index);
    let mut v4: u16 = (16i32 * v3 as i32 + 256i32) as u16;
    let mut v1: i32 = 16i32;
    for _ in 0i32..16i32 {
        let v5: i32 = v4 as i32 >> 1i32;
        if (ram_word!(game, palette_buffer[v5]) as i32 & 0x1fi32) - 31i32 & 0x8000i32 != 0 {
            update_ram_word!(game.palette_buffer[v5], wrapping_add(1057u16));
            v1 -= 1;
        }
        v4 = (v4).wrapping_add(2u16);
    }
    if ((v1 - 16i32) as i16) >= 0i16 {
        (*E).var_A = ((0xa89badi32) as u16);
        let v7: u16 = u16::swap_bytes((*E).base.palette_index);
        let mut v8: u16 = (16i32 * v7 as i32 + 256i32) as u16;
        let mut v9: u16 = 0u16;
        loop {
            set_ram_word!(
                game.target_palettes[(v8 >> 1)] = kWreckedShipGhost_Palette[(v9 >> 1) as usize]
            );
            v8 = (v8).wrapping_add(2u16);
            v9 = (v9).wrapping_add(2u16);
            if (((v9) as i16).wrapping_sub(32i16) >= 0i16) {
                break;
            }
        }
    }
}
pub unsafe fn WreckedShipGhost_Func_2(k: u16, game: &mut Game) {
    let v4: u16 = WreckedShipGhost_Func_8(game);
    WreckedShipGhost_Func_3(ram_word!(game, cur_enemy_index), game);
    let E = Get_WreckedShipGhost(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).var_B | v4 == 0 {
        (*E).base.properties = ((*E).base.properties) & (0xfaffu16);
        (*E).var_A = ((0xa89c8ai32) as u16);
        (*E).var_00 = (*E).base.y_pos;
        (*E).var_B = 1u16;
        (*E).var_C = 2u16;
        (*E).var_01 = 0u16;
        (*E).var_02 = g_word_A89AA2;
        (*E).var_B = g_word_A89AA6;
        (*E).var_05 = 4u16;
        let v2: u16 = ram_word!(game, samus_x_pos);
        (*E).var_06 = ram_word!(game, samus_x_pos);
        (*E).var_07 = v2;
        (*E).var_08 = v2;
        (*E).var_09 = 12u16;
        let v3: u16 = *((ram_ptr!(game, u16; 0xafa) as *mut u8)
            .offset(ram_word!(game, cur_enemy_index) as i32 as isize)
            as *mut u16);
        (*E).var_0A = v3;
        (*E).var_0B = v3;
        (*E).var_0C = v3;
        (*E).var_0D = g_word_A89A9E;
        (*E).var_0E = g_word_A89A9C;
    }
}
pub unsafe fn WreckedShipGhost_Func_3(k: u16, game: &mut Game) {
    let E = Get_WreckedShipGhost(k as i32, game);
    let wsgt_var_B: u16 = (*E).var_B;
    let make_visible: bool = (wsgt_var_B == 0)
        || ({
            let mut v3: u16 = 0;
            v3 = (wsgt_var_B).wrapping_sub(1u16);
            (*E).var_B = v3;
            (v3 == 0)
                && ({
                    let mut wsgt_var_C: u16 = 0;
                    wsgt_var_C = (*E).var_C;
                    let mut v5: i16 = g_word_A89ACC[(wsgt_var_C >> 1) as usize] as i16;
                    if v5 < 0i16 {
                        set_fields! {[(*E)], 0u16; var_B, var_C}
                        return;
                    }
                    (*E).var_B = v5 as u16;
                    (*E).var_C = (wsgt_var_C).wrapping_add(2u16);
                    ((wsgt_var_C & 2) == 0)
                })
        });
    if make_visible {
        (*E).base.properties =
            ((*E).base.properties as i32 & !(kEnemyProps_Invisible as i32)) as u16;
    };
}
pub unsafe fn WreckedShipGhost_Func_4(k: u16, game: &mut Game) {
    if WreckedShipGhost_Func_8(game) == 0 {
        let E = Get_WreckedShipGhost(ram_word!(game, cur_enemy_index) as i32, game);
        (*E).var_A = ((0xa89d13i32) as u16);
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
        (*E).var_B = g_word_A89AA4;
    }
}
pub unsafe fn WreckedShipGhost_Func_5(k: u16, game: &mut Game) {
    let E = Get_WreckedShipGhost(k as i32, game);
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        ((*E).var_02 as u32) << 16i32 | (*E).var_01 as u32,
    );
    AddToHiLo(
        &raw mut (*E).var_02,
        &raw mut (*E).var_01,
        (if (((*E).base.y_pos) as i16).wrapping_sub(((*E).var_00) as i16) < 0i16 {
            g_word_A89AA0 as i32
        } else {
            -(g_word_A89AA0 as i32)
        }) as u32,
    );
    (*E).var_B = (*E).var_B.wrapping_sub(1);
    if (*E).var_B == 0u16 {
        (*E).var_A = ((0xa89c69i32) as u16);
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
        let t: i32 = u16::swap_bytes((*E).base.palette_index) as i32;
        let mut v9: u16 = (16i32 * t + 256i32) as u16;
        for _ in 0i32..=16i32 {
            set_ram_word!(game.target_palettes[(v9 >> 1)] = 0x7fffu16);
            v9 = (v9).wrapping_add(2u16);
        }
    }
}
pub unsafe fn WreckedShipGhost_Func_6(k: u16, game: &mut Game) {
    let E = Get_WreckedShipGhost(k as i32, game);
    let wsgt_var_B: u16 = (*E).var_B;
    if wsgt_var_B == 0 || {
        let mut v3: u16 = (wsgt_var_B).wrapping_sub(1u16);
        (*E).var_B = v3;
        (*E).var_B == 0u16
    } {
        (*E).var_B = 1u16;
        (*E).var_C = 2u16;
        (*E).var_A = ((0xa89d36i32) as u16);
    }
}
pub unsafe fn WreckedShipGhost_Func_7(k: u16, game: &mut Game) {
    let mut v2: i16 = 0;
    let E = Get_WreckedShipGhost(k as i32, game);

    let skip_reposition: bool =
        if ((ram_word!(game, samus_x_pos)) as i16).wrapping_sub(((*E).var_07) as i16) < 0i16
            || ((ram_word!(game, samus_x_pos)) as i16).wrapping_sub(((*E).var_08) as i16) >= 0i16
            || {
                v2 = *((ram_ptr!(game, u16; 0xafa) as *mut u8).offset(k as i32 as isize)
                    as *mut u16) as i16;
                (v2).wrapping_sub(((*E).var_0B) as i16) < 0i16
            }
            || (v2).wrapping_sub(((*E).var_0C) as i16) >= 0i16
        {
            (*E).var_0D = g_word_A89A9E;
            let mut v4: i16 = 0i16;
            if ((ram_word!(game, samus_x_pos)) as i16).wrapping_sub(((*E).var_06) as i16) >= 0i16 {
                v4 = (if ram_word!(game, samus_x_pos) == (*E).var_06 {
                    4i32
                } else {
                    8i32
                }) as i16;
            }
            if v4 as i32 == (*E).var_05 as i32 {
                let mut v7: i16 = 0i16;
                let mut v9: i16 =
                    ((ram_word!(game, samus_y_pos)) as i16).wrapping_sub(((*E).var_0A) as i16);
                if v9 >= 0i16 {
                    v7 = (if v9 != 0i16 { 24i32 } else { 12i32 }) as i16;
                }
                if v7 as i32 == (*E).var_09 as i32 {
                    let v10: u16 = ((*E).var_0E).wrapping_sub(1u16);
                    (*E).var_0E = v10;
                    !(v10 == 0)
                } else {
                    (*E).var_09 = v7 as u16;
                    (*E).var_0E = g_word_A89A9C;
                    true
                }
            } else {
                (*E).var_05 = v4 as u16;
                (*E).var_0E = g_word_A89A9C;
                true
            }
        } else {
            let mut v3: u16 = 0;
            v3 = ((*E).var_0D).wrapping_sub(1u16);
            (*E).var_0D = v3;
            !(v3 == 0)
                || ({
                    (*E).var_05 = 4u16;
                    (*E).var_09 = 12u16;
                    false
                })
        };
    if skip_reposition {
    } else {
        (*E).var_A = ((0xa89b42i32) as u16);
        (*E).var_0D = g_word_A89A9E;
        (*E).var_0E = g_word_A89A9C;
        let v6: i32 = (((*E).var_09).wrapping_add((*E).var_05)) as i32 >> 1i32;
        (*E).base.x_pos = (g_word_A89AA8[v6 as usize]).wrapping_add(
            *((ram_ptr!(game, u16; 0xaf6) as *mut u8).offset(k as i32 as isize) as *mut u16),
        );
        (*E).base.y_pos = (g_word_A89AA8[(v6 + 1i32) as usize]).wrapping_add(
            *((ram_ptr!(game, u16; 0xafa) as *mut u8).offset(k as i32 as isize) as *mut u16),
        );
        return;
    }
    let v11: u16 = ram_word!(game, samus_x_pos);
    (*E).var_06 = ram_word!(game, samus_x_pos);
    (*E).var_07 = (v11).wrapping_sub(g_word_A89D32);
    (*E).var_08 = (g_word_A89D32).wrapping_add(ram_word!(game, samus_x_pos));
    let v13: u16 = ram_word!(game, samus_y_pos);
    (*E).var_0A = ram_word!(game, samus_y_pos);
    (*E).var_0B = (v13).wrapping_sub(g_word_A89D34);
    (*E).var_0C = (g_word_A89D34).wrapping_add(ram_word!(game, samus_y_pos));
}
pub unsafe fn WreckedShipGhost_Func_8(game: &mut Game) -> u16 {
    let mut v0: u16 = 0u16;
    if ram_word!(game, door_transition_flag_enemies) == 0 {
        let E = Get_WreckedShipGhost(ram_word!(game, cur_enemy_index) as i32, game);
        let v2: u16 = u16::swap_bytes((*E).base.palette_index);
        let mut v3: u16 = (16i32 * v2 as i32 + 256i32) as u16;
        let r20: u16 = (16i32 * v2 as i32 + 288i32) as u16;
        loop {
            let v4: i32 = v3 as i32 >> 1i32;
            if ram_word!(game, target_palettes[v4]) != ram_word!(game, palette_buffer[v4]) {
                let mut r18: u16 = (ram_word!(game, target_palettes[v4])) & (0x1fu16);
                let v5: u16 = (ram_word!(game, palette_buffer[v4])) & (0x1fu16);
                if v5 != r18 {
                    let mut v6: u16 = 0;
                    if ((v5) as i16).wrapping_sub((r18) as i16) >= 0i16 {
                        v6 = (v5).wrapping_sub(1u16);
                    } else {
                        v6 = (v5).wrapping_add(1u16);
                    }
                    r18 = v6;
                    set_ram_word!(
                        game.palette_buffer[v4] =
                            (v6 | ram_word!(game, palette_buffer[v4]) & 0xffe0)
                    );
                    v0 = v0.wrapping_add(1);
                }
                r18 = (ram_word!(game, target_palettes[v4])) & (0x3e0u16);
                let v7: u16 = (ram_word!(game, palette_buffer[v4])) & (0x3e0u16);
                if v7 != r18 {
                    let mut v8: u16 = 0;
                    if ((v7) as i16).wrapping_sub((r18) as i16) >= 0i16 {
                        v8 = (v7).wrapping_sub(32u16);
                    } else {
                        v8 = (v7).wrapping_add(32u16);
                    }
                    set_ram_word!(
                        game.palette_buffer[v4] =
                            (v8 | ram_word!(game, palette_buffer[v4]) & 0xfc1f)
                    );
                    v0 = v0.wrapping_add(1);
                }
                r18 = (ram_word!(game, target_palettes[v4])) & (0x7c00u16);
                let v9: u16 = (ram_word!(game, palette_buffer[v4])) & (0x7c00u16);
                if v9 != r18 {
                    let mut v10: u16 = 0;
                    if ((v9) as i16).wrapping_sub((r18) as i16) >= 0i16 {
                        v10 = (v9).wrapping_sub(1024u16);
                    } else {
                        v10 = (v9).wrapping_add(1024u16);
                    }
                    r18 = v10;
                    set_ram_word!(
                        game.palette_buffer[v4] =
                            (v10 | ram_word!(game, palette_buffer[v4]) & 0x83ff)
                    );
                    v0 = v0.wrapping_add(1);
                }
            }
            v3 = (v3).wrapping_add(2u16);
            if (((v3) as i16).wrapping_sub((r20) as i16) >= 0i16) {
                break;
            }
        }
    }
    v0
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16;
    YappingMaw_Instr_2 [jp] {
        let E = Get_YappingMaw(ram_word!(game, cur_enemy_index) as i32, game);
        (*E).var_32 = crate::rtl::rom_table_word!(game, 0xa8a0ab, 0isize);
        (*E).var_33 = crate::rtl::rom_table_word!(game, 0xa8a0ad, 0isize);
    };
    YappingMaw_Instr_4 [jp] {
        let E = Get_YappingMaw(ram_word!(game, cur_enemy_index) as i32, game);
        (*E).var_32 = crate::rtl::rom_table_word!(game, 0xa8a0c3, 0isize);
        (*E).var_33 = crate::rtl::rom_table_word!(game, 0xa8a0c5, 0isize);
    };
    YappingMaw_Instr_5 [jp] {
        let E = Get_YappingMaw(ram_word!(game, cur_enemy_index) as i32, game);
        (*E).var_32 = crate::rtl::rom_table_word!(game, 0xa8a0b3, 0isize);
        (*E).var_33 = crate::rtl::rom_table_word!(game, 0xa8a0b5, 0isize);
    };
    YappingMaw_Instr_7 [jp] {
        let E = Get_YappingMaw(ram_word!(game, cur_enemy_index) as i32, game);
        (*E).var_32 = crate::rtl::rom_table_word!(game, 0xa8a0bb, 0isize);
        (*E).var_33 = crate::rtl::rom_table_word!(game, 0xa8a0bd, 0isize);
    };
    YappingMaw_Instr_3 [jp] {
        let E = Get_YappingMaw(ram_word!(game, cur_enemy_index) as i32, game);
        (*E).var_32 = crate::rtl::rom_table_word!(game, 0xa8a0a7, 0isize);
        (*E).var_33 = crate::rtl::rom_table_word!(game, 0xa8a0a9, 0isize);
    };
    YappingMaw_Instr_6 [jp] {
        let E = Get_YappingMaw(ram_word!(game, cur_enemy_index) as i32, game);
        (*E).var_32 = crate::rtl::rom_table_word!(game, 0xa8a0b7, 0isize);
        (*E).var_33 = crate::rtl::rom_table_word!(game, 0xa8a0b9, 0isize);
    };
    YappingMaw_Instr_1 [jp] {
        if (*Get_YappingMaw(ram_word!(game, cur_enemy_index) as i32, game)).var_36 == 0 {
            QueueSfx2_Max6(0x2fu16, game);
        }
    };
}
pub unsafe fn YappingMaw_Init(game: &mut Game) {
    let E = Get_YappingMaw(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).var_0C = (*E).base.x_pos;
    (*E).var_0D = (*E).base.y_pos;
    set_fields! {[(*E)], 0u16; var_00, var_01, var_02, var_03, var_04, var_05, var_06, var_07, var_30}
    (*E).var_F = (*E).parameter_1;
    (*E).var_E = 64u16;
    (*E).base.current_instruction = addr_kYappingMaw_Ilist_9F6F as u16;
    let mut R36: u16 = 57u16;
    let mut R34: u16 = 8u16;
    if (*E).parameter_2 == 0 {
        (*E).base.current_instruction = addr_kYappingMaw_Ilist_9FC7 as u16;
        R36 = 56u16;
        R34 = -8i32 as u16;
    }
    (*E).base.instruction_timer = 1u16;
    (*E).base.timer = 0u16;
    (*E).var_A = ((0xa8a235i32) as u16);
    (*E).var_45 = ((*E).base.palette_index) & (0xe00u16);
    let mut v1: i16 = 3i16;
    (*E).var_44 = 3u16;
    loop {
        SpawnEprojWithGfx(
            v1 as u16,
            ram_word!(game, cur_enemy_index),
            addr_kEproj_YappingMawsBody as u16,
            game,
        );
        v1 = (((*E).var_44) as i16).wrapping_sub(1i16);
        (*E).var_44 = v1 as u16;
        if (v1 < 0i16) {
            break;
        }
    }
    let v3: u16 = ((*E).base.vram_tiles_index) | ((*E).base.palette_index);
    (*E).var_47 = v3;
    (*E).var_46 = CreateSpriteAtPos(
        (*E).base.x_pos,
        (R34).wrapping_add((*E).base.y_pos),
        R36,
        v3,
        game,
    );
}
crate::game::address_dispatch! {
    pub unsafe fn CallYappingMawFunc(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        11051573 => YappingMaw_Func_1; 11051660 => YappingMaw_Func_2; 11052101 => YappingMaw_Func_8;
        11052682 => YappingMaw_Func_11;
    }
}
pub unsafe fn YappingMaw_Main(game: &mut Game) {
    let E = Get_YappingMaw(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).var_35 = (*E).var_35.wrapping_sub(1);
    (*E).var_36 = CheckIfEnemyIsOnScreen(game);
    CallYappingMawFunc((((*E).var_A) as u32) | (0xa80000u32), game);
    YappingMaw_Func_15(game);
    YappingMaw_Func_14(game);
    YappingMaw_Func_13(game);
    YappingMaw_Func_12(game);
}
pub unsafe fn YappingMaw_Func_1(game: &mut Game) {
    let E = Get_YappingMaw(ram_word!(game, cur_enemy_index) as i32, game);
    let pair: PairU16 = EnemyFunc_ACA8(
        Point16U {
            x: (*E).base.x_pos,
            y: (*E).base.y_pos,
        },
        Point16U {
            x: ram_word!(game, samus_x_pos),
            y: ram_word!(game, samus_y_pos),
        },
        game,
    );
    let v2: u16 = Abs16(pair.k);
    (*E).var_08 = v2;
    if v2 as i32 - 32i32 & 0x8000i32 != 0 {
        (*E).var_35 = 48u16;
    } else if ((v2) as i16).wrapping_sub(((*E).var_F) as i16) < 0i16 {
        if (*E).var_08 as i32 - 64i32 & 0x8000i32 == 0 {
            (*E).var_08 = 64u16;
        }
        (*E).var_0A = pair.j;
        (*E).var_A = ((0xa8a28ci32) as u16);
    }
}
pub unsafe fn YappingMaw_Func_2(game: &mut Game) {
    let E = Get_YappingMaw(ram_word!(game, cur_enemy_index) as i32, game);
    set_fields! {[(*E)], 0u16; var_B, var_C, var_D}
    (*E).var_09 = ((*E).var_08 >> 1);
    let v1: u16 = ((64u8).wrapping_sub(((*E).var_0A) as u8)) as u16;
    (*E).var_0B = v1;
    if v1 as i32 - 128i32 & 0x8000i32 != 0 {
        (*E).var_2F = 0u16;
    } else {
        (*E).var_2F = 1u16;
    }
    YappingMaw_Func_3(game);
    let v2: u16 = (2i32 * (((((*E).var_0A) as u8).wrapping_add(16u8)) as i32 >> 5i32)) as u16;
    (*E).var_34 = v2;
    enemy_instruction! {[(*E).base]; instruction = crate::rtl::rom_table_word!(game, 0xa8a097, (v2 >> 1) as isize), timer = 1u16}
    (*E).base.timer = 0u16;
    (*E).var_32 = *((game.g_rom.offset(crate::rtl::lorom_offset(0xa8a0a7)) as *mut u16 as *mut u8)
        .offset(((2u16).wrapping_mul(v2)) as i32 as isize) as *mut u16);
    (*E).var_33 = *((game.g_rom.offset(crate::rtl::lorom_offset(0xa8a0a9)) as *mut u16 as *mut u8)
        .offset(((2u16).wrapping_mul(v2)) as i32 as isize) as *mut u16);
    (*E).var_A = ((0xa8a445i32) as u16);
}
pub unsafe fn YappingMaw_Func_3(game: &mut Game) {
    let E = Get_YappingMaw(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).var_0E = YappingMaw_Func_16(0x80u16, (*E).var_09, game);
    (*E).var_0F = YappingMaw_Func_17(0x80u16, ((*E).var_09 >> 1), game);
}
pub unsafe fn YappingMaw_Func_4(game: &mut Game) {
    let E = Get_YappingMaw(ram_word!(game, cur_enemy_index) as i32, game);
    let varE32: u16 = (*E).var_00;
    (*E).var_10 = YappingMaw_Func_16(0u16, varE32, game);
    (*E).var_11 = YappingMaw_Func_17(0u16, varE32, game);
    (*E).var_27 = (YappingMaw_Func_16((*E).var_0B, varE32, game)).wrapping_sub((*E).var_10);
    (*E).var_28 = (YappingMaw_Func_17((*E).var_0B, varE32, game)).wrapping_sub((*E).var_11);
}
pub unsafe fn YappingMaw_Func_5(game: &mut Game) {
    let E = Get_YappingMaw(ram_word!(game, cur_enemy_index) as i32, game);
    let varE32: u16 = (*E).var_02;
    (*E).var_10 = YappingMaw_Func_16(0u16, varE32, game);
    (*E).var_11 = YappingMaw_Func_17(0u16, varE32, game);
    (*E).var_29 = (YappingMaw_Func_16((*E).var_0B, varE32, game)).wrapping_sub((*E).var_10);
    (*E).var_2A = (YappingMaw_Func_17((*E).var_0B, varE32, game)).wrapping_sub((*E).var_11);
}
pub unsafe fn YappingMaw_Func_6(game: &mut Game) {
    let E = Get_YappingMaw(ram_word!(game, cur_enemy_index) as i32, game);
    let varE32: u16 = (*E).var_04;
    (*E).var_10 = YappingMaw_Func_16(0u16, varE32, game);
    (*E).var_11 = YappingMaw_Func_17(0u16, varE32, game);
    (*E).var_2B = (YappingMaw_Func_16((*E).var_0B, varE32, game)).wrapping_sub((*E).var_10);
    (*E).var_2C = (YappingMaw_Func_17((*E).var_0B, varE32, game)).wrapping_sub((*E).var_11);
}
pub unsafe fn YappingMaw_Func_7(game: &mut Game) {
    let E = Get_YappingMaw(ram_word!(game, cur_enemy_index) as i32, game);
    let varE32: u16 = (*E).var_06;
    (*E).var_10 = YappingMaw_Func_16(0u16, varE32, game);
    (*E).var_11 = YappingMaw_Func_17(0u16, varE32, game);
    (*E).var_2D = (YappingMaw_Func_16((*E).var_0B, varE32, game)).wrapping_sub((*E).var_10);
    (*E).var_2E = (YappingMaw_Func_17((*E).var_0B, varE32, game)).wrapping_sub((*E).var_11);
}
pub unsafe fn YappingMaw_Func_8(game: &mut Game) {
    let E = Get_YappingMaw(ram_word!(game, cur_enemy_index) as i32, game);
    let r20: u16 = ((*E).var_B >> 2);
    if (*E).var_2F != 0 {
        (*E).var_21 = (128u16).wrapping_add(r20);
        (*E).var_22 = (128i32 + r20 as i32 * 2i32) as u16;
        (*E).var_23 = (128i32 + r20 as i32 * 3i32) as u16;
        (*E).var_24 = (128i32 + r20 as i32 * 4i32) as u16;
    } else {
        (*E).var_21 = (128u16).wrapping_sub(r20);
        (*E).var_22 = (128i32 - r20 as i32 * 2i32) as u16;
        (*E).var_23 = (128i32 - r20 as i32 * 3i32) as u16;
        (*E).var_24 = (128i32 - r20 as i32 * 4i32) as u16;
    }
    let mut varE32: u16 = (*E).var_09;
    (*E).var_00 = (YappingMaw_Func_16((*E).var_21, varE32, game)).wrapping_sub((*E).var_0E);
    (*E).var_02 = (YappingMaw_Func_16((*E).var_22, varE32, game)).wrapping_sub((*E).var_0E);
    (*E).var_04 = (YappingMaw_Func_16((*E).var_23, varE32, game)).wrapping_sub((*E).var_0E);
    (*E).var_06 = (YappingMaw_Func_16((*E).var_24, varE32, game)).wrapping_sub((*E).var_0E);
    varE32 = ((*E).var_09 >> 1);
    (*E).var_01 = (YappingMaw_Func_17((*E).var_21, varE32, game)).wrapping_sub((*E).var_0F);
    (*E).var_03 = (YappingMaw_Func_17((*E).var_22, varE32, game)).wrapping_sub((*E).var_0F);
    (*E).var_05 = (YappingMaw_Func_17((*E).var_23, varE32, game)).wrapping_sub((*E).var_0F);
    (*E).var_07 = (YappingMaw_Func_17((*E).var_24, varE32, game)).wrapping_sub((*E).var_0F);
    YappingMaw_Func_4(game);
    YappingMaw_Func_5(game);
    YappingMaw_Func_6(game);
    YappingMaw_Func_7(game);
    (*E).var_00 = ((*E).var_00).wrapping_add((*E).var_27);
    (*E).var_01 = ((*E).var_01).wrapping_add((*E).var_28);
    (*E).var_02 = ((*E).var_02).wrapping_add((*E).var_29);
    (*E).var_03 = ((*E).var_03).wrapping_add((*E).var_2A);
    (*E).var_04 = ((*E).var_04).wrapping_add((*E).var_2B);
    (*E).var_05 = ((*E).var_05).wrapping_add((*E).var_2C);
    (*E).var_06 = ((*E).var_06).wrapping_add((*E).var_2D);
    (*E).var_07 = ((*E).var_07).wrapping_add((*E).var_2E);
    (*E).base.x_pos = ((*E).var_06).wrapping_add((*E).var_0C);
    (*E).base.y_pos = ((*E).var_07).wrapping_add((*E).var_0D);
    YappingMaw_Func_9(ram_word!(game, cur_enemy_index), game);
    if (*E).var_B & 0x8000 == 0 {
        if (*E).var_B as i32 - 128i32 & 0x8000i32 == 0 {
            (*E).var_B = 128u16;
            (*E).var_C = 0u16;
            (*E).var_D = ((*E).var_D).wrapping_add(4u16);
        }
    } else {
        (*E).var_A = ((0xa8a68ai32) as u16);
        (*E).var_35 = 48u16;
        if (*E).var_34 == 4u16 {
            (*E).base.current_instruction = addr_kYappingMaw_Ilist_A01F as u16;
        } else if (*E).var_34 == 12u16 {
            (*E).base.current_instruction = addr_kYappingMaw_Ilist_A03D as u16;
        } else {
            (*E).base.current_instruction = addr_off_A8A025 as u16;
        }
        if (*E).parameter_2 == 0 {
            if (*E).var_34 == 4u16 {
                (*E).base.current_instruction = addr_kYappingMaw_Ilist_A05B as u16;
            } else {
                if (*E).var_34 != 12u16 {
                    (*E).base.current_instruction = addr_off_A8A061 as u16;
                    return;
                }
                (*E).base.current_instruction = addr_kYappingMaw_Ilist_A079 as u16;
            }
        } else {
            return;
        }
    }
    if (*E).var_30 != 0 {
        YappingMaw_Func_10(game);
    }
}
pub unsafe fn YappingMaw_Func_9(k: u16, game: &mut Game) {
    let E = Get_YappingMaw(k as i32, game);
    let v2: i32 = (*E).var_D as i32 >> 1i32;
    AddToHiLo(
        &raw mut (*E).var_B,
        &raw mut (*E).var_C,
        (crate::rtl::rom_table_word!(game, 0xa2838f, (v2 + 1i32) as isize) as u32) << 16i32
            | crate::rtl::rom_table_word!(game, 0xa2838f, v2 as isize) as u32,
    );
    (*E).var_D = ((*E).var_D).wrapping_add(8u16);
}
pub unsafe fn YappingMaw_Func_10(game: &mut Game) {
    CallSomeSamusCode(3u16, game);
    let E = Get_YappingMaw(ram_word!(game, cur_enemy_index) as i32, game);
    set_ram_word!(game.samus_x_pos = ((*E).var_32).wrapping_add((*E).base.x_pos));
    set_ram_word!(game.samus_y_pos = ((*E).var_33).wrapping_add((*E).base.y_pos));
    EnemyFunc_B7A1(game);
}
pub unsafe fn YappingMaw_Func_11(game: &mut Game) {
    let E = Get_YappingMaw(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).var_30 != 0 {
        YappingMaw_Func_10(game);
    }
    (*E).var_E = (*E).var_E.wrapping_sub(1);
    if ((*E).var_E & 0x8000) != 0
        && ram_word!(game, samus_input_handler) as i32 != 0x90e91di32 & 0xffffi32
    {
        set_ram_word!(game.samus_input_handler = ((0x90e913i32) as u16));
        (*E).var_30 = 0u16;
        (*E).var_35 = 48u16;
        (*E).var_E = 64u16;
        (*E).var_A = ((0xa8a235i32) as u16);
    }
}
pub unsafe fn YappingMaw_Func_12(game: &mut Game) {
    let E = Get_YappingMaw(ram_word!(game, cur_enemy_index) as i32, game);
    let v1: i32 = (*E).var_40 as i32 >> 1i32;
    set_ram_word!(game.eproj_x_pos[v1] = (*E).var_0C);
    set_ram_word!(game.eproj_y_pos[v1] = (*E).var_0D);
}
pub unsafe fn YappingMaw_Func_13(game: &mut Game) {
    let E = Get_YappingMaw(ram_word!(game, cur_enemy_index) as i32, game);
    let v1: i32 = (*E).var_41 as i32 >> 1i32;
    set_ram_word!(game.eproj_x_pos[v1] = ((*E).var_00).wrapping_add((*E).var_0C));
    set_ram_word!(game.eproj_y_pos[v1] = ((*E).var_01).wrapping_add((*E).var_0D));
}
pub unsafe fn YappingMaw_Func_14(game: &mut Game) {
    let E = Get_YappingMaw(ram_word!(game, cur_enemy_index) as i32, game);
    let v1: i32 = (*E).var_42 as i32 >> 1i32;
    set_ram_word!(game.eproj_x_pos[v1] = ((*E).var_02).wrapping_add((*E).var_0C));
    set_ram_word!(game.eproj_y_pos[v1] = ((*E).var_03).wrapping_add((*E).var_0D));
}
pub unsafe fn YappingMaw_Func_15(game: &mut Game) {
    let E = Get_YappingMaw(ram_word!(game, cur_enemy_index) as i32, game);
    let v1: i32 = (*E).var_43 as i32 >> 1i32;
    set_ram_word!(game.eproj_x_pos[v1] = ((*E).var_04).wrapping_add((*E).var_0C));
    set_ram_word!(game.eproj_y_pos[v1] = ((*E).var_05).wrapping_add((*E).var_0D));
}
pub unsafe fn YappingMaw_Func_16(a: u16, varE32: u16, game: &mut Game) -> u16 {
    YappingMaw_Func_17((a).wrapping_sub(64u16), varE32, game)
}
pub unsafe fn YappingMaw_Func_17(a: u16, varE32: u16, game: &mut Game) -> u16 {
    let mut r20: u16 = 0u16;
    let mut r22: u16 = 0u16;
    let mut v1: i16 =
        crate::rtl::rom_table_word!(game, 0xa0b1c3, -(a as i32) as u8 as isize) as i16;
    if v1 < 0i16 {
        v1 = -(v1 as i32) as i16;
        r20 = r20.wrapping_add(1);
    }
    WriteReg(
        WRMPYA as u16,
        *(&raw mut v1 as *mut u8).offset(1isize),
        game,
    );
    if varE32 as u8 != 0 {
        let prod: u16 = Mult8x8(*(&raw mut v1 as *mut u8).offset(1isize), varE32 as u8);
        r22 = (2i32 * ((prod as i32 & 0xff00i32) >> 8i32)) as u16;
        if r20 != 0 {
            r22 = (2i32 * (-(prod as i32) as u16 as i32 >> 8i32) | 0xff00i32) as u16;
        }
    }
    r22
}
pub unsafe fn YappingMaw_Touch(game: &mut Game) {
    let E = Get_YappingMaw(ram_word!(game, cur_enemy_index) as i32, game);
    if ((*E).var_35 & 0x8000) != 0 && (*E).var_30 == 0 {
        (*E).var_35 = 0u16;
        (*E).var_30 = 1u16;
        set_ram_word!(game.samus_input_handler = ((0x90e90ei32) as u16));
    }
}
pub unsafe fn YappingMaw_Shot(game: &mut Game) {
    NormalEnemyShotAi(game);
    let E = Get_YappingMaw(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).base.health != 0 {
        if (*E).base.frozen_timer != 0 {
            if ram_word!(game, samus_input_handler) as i32 != 0x90e91di32 & 0xffffi32 {
                set_ram_word!(game.samus_input_handler = ((0x90e913i32) as u16));
            }
            (*E).var_30 = 0u16;
        }
    } else {
        *((ram_ptr!(game, u16; 0x1997) as *mut u8).offset((*E).var_40 as i32 as isize)
            as *mut u16) = 0u16;
        *((ram_ptr!(game, u16; 0x1997) as *mut u8).offset((*E).var_41 as i32 as isize)
            as *mut u16) = 0u16;
        *((ram_ptr!(game, u16; 0x1997) as *mut u8).offset((*E).var_42 as i32 as isize)
            as *mut u16) = 0u16;
        *((ram_ptr!(game, u16; 0x1997) as *mut u8).offset((*E).var_43 as i32 as isize)
            as *mut u16) = 0u16;
        set_ram_word!(game.sprite_instr_list_ptrs[((*E).var_46 >> 1)] = 0u16);
        if ram_word!(game, samus_input_handler) as i32 != 0x90e91di32 & 0xffffi32 {
            set_ram_word!(game.samus_input_handler = ((0x90e913i32) as u16));
        }
        (*Get_YappingMaw(ram_word!(game, cur_enemy_index) as i32, game)).var_30 = 0u16;
    };
}
pub unsafe fn YappingMaw_Frozen(game: &mut Game) {
    Enemy_NormalFrozenAI_A8(game);
    let E = Get_YappingMaw(ram_word!(game, cur_enemy_index) as i32, game);
    YappingMaw_Func_18((*E).var_40, game);
    YappingMaw_Func_18((*E).var_41, game);
    YappingMaw_Func_18((*E).var_42, game);
    YappingMaw_Func_18((*E).var_43, game);
    YappingMaw_Func_19(game);
}
pub unsafe fn YappingMaw_Func_18(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    let E = Get_YappingMaw(ram_word!(game, cur_enemy_index) as i32, game);
    set_ram_word!(
        game.eproj_gfx_idx[v1] = ((*E).var_45 | ram_word!(game, eproj_gfx_idx[v1]) & 0xf1ff)
    );
    if (*E).base.frozen_timer != 0 {
        set_ram_word!(
            game.eproj_gfx_idx[v1] = (ram_word!(game, eproj_gfx_idx[v1]) & 0xf1ff | 0xc00)
        );
        let frozen_timer: u16 = (*E).base.frozen_timer;
        if frozen_timer as i32 - 90i32 & 0x8000i32 != 0 && (frozen_timer & 2) == 0 {
            set_ram_word!(
                game.eproj_gfx_idx[v1] =
                    ((*E).var_45 | ram_word!(game, eproj_gfx_idx[v1]) & 0xf1ff)
            );
        }
    }
}
pub unsafe fn YappingMaw_Func_19(game: &mut Game) {
    let E = Get_YappingMaw(ram_word!(game, cur_enemy_index) as i32, game);
    let r18: u16 = (*E).var_45;
    let r20: u16 = (*E).var_46;
    set_ram_word!(
        game.sprite_palettes[(r20 >> 1)] =
            (r18 | ram_word!(game, sprite_palettes[(r20 >> 1)]) & 0xf1ff)
    );
    if (*E).base.frozen_timer != 0 {
        set_ram_word!(
            game.sprite_palettes[(r20 >> 1)] =
                (ram_word!(game, sprite_palettes[(r20 >> 1)]) & 0xf1ff | 0xc00)
        );
        let frozen_timer: u16 = (*E).base.frozen_timer;
        if frozen_timer as i32 - 90i32 & 0x8000i32 != 0 && (frozen_timer & 2) == 0 {
            set_ram_word!(
                game.sprite_palettes[(r20 >> 1)] =
                    (r18 | ram_word!(game, sprite_palettes[(r20 >> 1)]) & 0xf1ff)
            );
        }
    }
}
pub unsafe fn Kago_Init(game: &mut Game) {
    let E = Get_Kago(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_DisableSamusColl) as u16);
    (*E).base.instruction_timer = 1u16;
    (*E).base.timer = 0u16;
    (*E).var_E = 0u16;
    (*E).base.current_instruction = addr_kKago_Ilist_AB1E as u16;
    (*E).var_A = ((0xa8ab7bi32) as u16);
    (*E).var_F = 0u16;
    (*E).var_04 = (*E).parameter_1;
}
crate::game::address_dispatch! {
    pub unsafe fn CallKagoFunc(ea: u32, k: u16, game: &mut Game) => ea;
        call (k, game); default { Unreachable(game); };
    {
        11053947 => Kago_Func_1; 11053953 => [()];
    }
}
pub unsafe fn Kago_Main(game: &mut Game) {
    let E = Get_Kago(ram_word!(game, cur_enemy_index) as i32, game);
    CallKagoFunc(
        (((*E).var_A) as u32) | (0xa80000u32),
        ram_word!(game, cur_enemy_index),
        game,
    );
}
pub unsafe fn Kago_Func_1(k: u16, game: &mut Game) {
    (*Get_Kago(k as i32, game)).var_A = ((0xa8ab81i32) as u16);
}
pub unsafe fn Kago_Shot(game: &mut Game) {
    NormalEnemyShotAi(game);
    set_ram_word!(game.earthquake_type = 2u16);
    set_ram_word!(game.earthquake_timer = 16u16);
    let E = Get_Kago(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).var_B == 0 {
        (*E).var_B = 1u16;
        enemy_instruction! {[(*E).base]; instruction = addr_kKago_Ilist_AB32 as u16, timer = 1u16}
    }
    let mut v2: i16 = (((*E).var_04) as i16).wrapping_sub(1i16);
    (*E).var_04 = v2 as u16;
    if v2 < 0i16 {
        EnemyDeathAnimation(ram_word!(game, cur_enemy_index), 4u16, game);
        (*E).var_F = 1u16;
    }
    SpawnEprojWithGfx(
        (*E).base.y_pos,
        ram_word!(game, cur_enemy_index),
        addr_loc_A8D02E as u16,
        game,
    );
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16;
    NorfairLavaMan_Instr_1 [jp.offset(1)] {
        if CheckIfEnemyIsOnScreen(game) == 0 {
            QueueSfx2_Max6(*jp.offset(0), game);
        }
    };
    NorfairLavaMan_Instr_8 [jp] {
        let E = Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as i32, game);
        (*E).base.y_pos = (*E).base.y_pos.wrapping_add(1);
        (*E).base.y_pos = (*E).base.y_pos.wrapping_add(1);
    };
    NorfairLavaMan_Instr_14 [jp] {
        let E = Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as i32, game);
        (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
        (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
    };
    NorfairLavaMan_Instr_2 [jp] {
        (*Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as i32, game)).var_01 = 1u16;
    };
    NorfairLavaMan_Instr_7 [jp] {
        (*Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as i32, game)).var_01 = 0u16;
    };
    NorfairLavaMan_Instr_10 [jp] {
        let E = Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as i32, game);
        (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
        let E1 = Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as i32 + 64i32, game);
        (*E1).base.y_pos = (*E1).base.y_pos.wrapping_sub(1);
    };
    NorfairLavaMan_Instr_12 [jp] {
        let E = Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as i32, game);
        (*E).base.y_pos = (*E).base.y_pos.wrapping_add(1);
        let E1 = Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as i32 + 64i32, game);
        (*E1).base.y_pos = (*E1).base.y_pos.wrapping_add(1);
    };
    NorfairLavaMan_Instr_9 [jp] {
        let E = Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as i32, game);
        let v4: u16 = ((*E).var_03).wrapping_add(24u16);
        (*E).base.y_pos = v4;
        let E1 = Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as i32 + 64i32, game);
        (*E1).base.y_pos = v4;
        (*E1).base.properties = ((*E1).base.properties & !(0x100));
        let E2 = Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as i32 + 128i32, game);
        (*E2).base.properties = ((*E2).base.properties & !(0x100));
    };
    NorfairLavaMan_Instr_11 [jp] {
        let E = Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as i32, game);
        let nlmn_var_03: u16 = (*E).var_03;
        (*E).base.y_pos = nlmn_var_03;
        (*Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as i32 + 64i32, game))
            .base
            .y_pos = nlmn_var_03;
    };
    NorfairLavaMan_Instr_13 [jp] {
        let E = Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as i32, game);
        let v4: u16 = ((*E).var_03).wrapping_add(4u16);
        (*E).base.y_pos = v4;
        let E1 = Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as i32 + 64i32, game);
        (*E1).base.y_pos = v4;
        (*E1).base.properties = ((*E1).base.properties) | (0x100u16);
        let E2 = Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as i32 + 128i32, game);
        (*E2).base.properties = ((*E2).base.properties) | (0x100u16);
    };
    NorfairLavaMan_Instr_5 [jp] {
        let E = Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as i32, game);
        SpawnEprojWithGfx(
            (*E).var_B,
            ram_word!(game, cur_enemy_index),
            addr_kEproj_LavaThrownByLavaman as u16,
            game,
        );
    };
    NorfairLavaMan_Instr_15 [jp] {
        let E = Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as i32, game);
        (*E).base.x_pos = ((*E).var_12).wrapping_add(8u16);
        (*E).base.y_pos = ((*E).var_13).wrapping_sub(4u16);
    };
    NorfairLavaMan_Instr_4 [jp] {
        let E = Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as i32, game);
        (*E).base.x_pos = ((*E).var_12).wrapping_sub(8u16);
        (*E).base.y_pos = ((*E).var_13).wrapping_sub(4u16);
    };
    NorfairLavaMan_Instr_16 [jp] {
        let E = Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as i32, game);
        (*E).base.x_pos = ((*E).var_12).wrapping_add(8u16);
        (*E).base.y_pos = ((*E).var_13).wrapping_sub(8u16);
    };
    NorfairLavaMan_Instr_6 [jp] {
        let E = Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as i32, game);
        (*E).base.x_pos = ((*E).var_12).wrapping_sub(8u16);
        (*E).base.y_pos = ((*E).var_13).wrapping_sub(4u16);
    };
    NorfairLavaMan_Instr_3 [jp] {
        (*Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as i32, game)).var_04 = 256u16;
    };
}
static g_off_A8AF4F: [Option<Func_V>; 3] = callback_table! { unsafe Func_V;
    NorfairLavaMan_Func_1, NorfairLavaMan_Func_2, NorfairLavaMan_Func_3,
};
pub unsafe fn NorfairLavaMan_Init(game: &mut Game) {
    let v0: u16 = (2u16).wrapping_mul(
        (*Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as i32, game)).parameter_1,
    );
    g_off_A8AF4F[(v0 >> 1) as usize].expect("non-null function pointer")(game);
    NorfairLavaMan_Func_4(game);
    NorfairLavaMan_Func_5(ram_word!(game, cur_enemy_index), game);
}
pub unsafe fn NorfairLavaMan_Func_1(game: &mut Game) {
    let E = Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as i32, game);
    set_fields! {[(*E)], 0u16; var_C, var_00, var_02}
    (*E).var_03 = (*E).base.y_pos;
    if (GetSamusEnemyDelta_X(ram_word!(game, cur_enemy_index), game) & 0x8000) != 0 {
        (*E).var_00 = 1u16;
    }
    (*E).var_D = addr_kNorfairLavaMan_Ilist_AC9C as u16;
    if (*E).var_00 == 0 {
        (*E).var_D = addr_kNorfairLavaMan_Ilist_AD3C as u16;
    }
    NorfairLavaMan_Func_20(game);
    (*E).var_F = ((0xa8b11ai32) as u16);
}
pub unsafe fn NorfairLavaMan_Func_2(game: &mut Game) {
    let E = Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).var_03 = (*E).base.y_pos;
    set_fields! {[(*E)], 0u16; var_C, var_0C}
    (*E).var_0D = 1u16;
    (*E).var_D = addr_kNorfairLavaMan_Ilist_ADDC as u16;
    NorfairLavaMan_Func_20(game);
    (*E).base.y_pos = ((*E).base.y_pos).wrapping_add(32u16);
    (*E).var_F = ((0xa8b1ddi32) as u16);
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
}
pub unsafe fn NorfairLavaMan_Func_3(game: &mut Game) {
    let E = Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).var_03 = (*E).base.y_pos;
    (*E).var_11 = (*E).base.x_pos;
    set_fields! {[(*E)], 0u16; var_C, var_04}
    (*E).var_D = addr_kNorfairLavaMan_Ilist_AE0C as u16;
    NorfairLavaMan_Func_20(game);
    (*E).base.y_pos = ((*E).base.y_pos).wrapping_add(32u16);
    (*E).var_F = ((0xa8b30di32) as u16);
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
}
pub unsafe fn NorfairLavaMan_Func_4(game: &mut Game) {
    let E = Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as i32, game);
    let v1: i32 = 8i32 * *(&raw mut (*E).parameter_2 as *mut u8).offset(1isize) as i32 >> 1i32;
    (*E).var_08 = crate::rtl::rom_table_word!(game, 0xa28187, v1 as isize);
    (*E).var_07 = crate::rtl::rom_table_word!(game, 0xa28187, (v1 + 1i32) as isize);
    (*E).var_0A = crate::rtl::rom_table_word!(game, 0xa28187, (v1 + 2i32) as isize);
    (*E).var_09 = crate::rtl::rom_table_word!(game, 0xa28187, (v1 + 3i32) as isize);
}
pub unsafe fn NorfairLavaMan_Func_5(k: u16, game: &mut Game) {
    (*(ram_ptr!(game, LongPtr; 0x178c))).addr = ((0xa8b0b2i32) as u16);
    *(&raw mut (*(ram_ptr!(game, LongPtr; 0x178c))).bank as *mut u16) = 168u16;
    set_ram_word!(
        game.variables_for_enemy_graphics_drawn_hook[0] =
            (((((*Get_NorfairLavaMan(k as i32, game)).base.palette_index) & (0xe00u16)) as i32
                >> 4i32)
                + 256i32) as u16
    );
    set_ram_word!(game.variables_for_enemy_graphics_drawn_hook[2] = 8u16);
    set_ram_word!(game.variables_for_enemy_graphics_drawn_hook[1] = 0u16);
}
pub unsafe fn NorfairLavaMan_Func_6(game: &mut Game) {
    if ram_word!(game, door_transition_flag_enemies) == 0 && {
        update_ram_word!(
            game.variables_for_enemy_graphics_drawn_hook[2],
            wrapping_sub(1),
            reread
        ) == 0
    } {
        set_ram_word!(game.variables_for_enemy_graphics_drawn_hook[2] = 8u16);
        update_ram_word!(
            game.variables_for_enemy_graphics_drawn_hook[1],
            wrapping_add(1)
        );
        let v0: i32 = (32i32
            * (ram_word!(game, variables_for_enemy_graphics_drawn_hook[1]) as i32 & 3i32))
            as u16 as i32
            >> 1i32;
        let r18: u16 = crate::rtl::rom_table_word!(game, 0xa8ac1c, (v0 + 9i32) as isize);
        let r20: u16 = crate::rtl::rom_table_word!(game, 0xa8ac1c, (v0 + 10i32) as isize);
        let r22: u16 = crate::rtl::rom_table_word!(game, 0xa8ac1c, (v0 + 11i32) as isize);
        let r24: u16 = crate::rtl::rom_table_word!(game, 0xa8ac1c, (v0 + 12i32) as isize);
        let v1: i32 = ram_word!(game, variables_for_enemy_graphics_drawn_hook[0]) as i32 >> 1i32;
        set_ram_word!(game.palette_buffer[(v1 + 9i32)] = r18);
        set_ram_word!(game.palette_buffer[(v1 + 10i32)] = r20);
        set_ram_word!(game.palette_buffer[(v1 + 11i32)] = r22);
        set_ram_word!(game.palette_buffer[(v1 + 12i32)] = r24);
    }
}
pub unsafe fn NorfairLavaMan_Main(game: &mut Game) {
    let E = Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).var_04 = (*E).var_04.wrapping_sub(1);
    CallEnemyPreInstr((((*E).var_F) as u32) | (0xa80000u32), game);
}
pub unsafe fn NorfairLavaMan_Func_7(k: u16, game: &mut Game) {
    let E = Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).var_00 = 0u16;
    (*E).var_D = addr_kNorfairLavaMan_Ilist_AC9C as u16;
    if (GetSamusEnemyDelta_X(ram_word!(game, cur_enemy_index), game) & 0x8000) == 0 {
        (*E).var_D = addr_kNorfairLavaMan_Ilist_AD3C as u16;
        (*E).var_00 = 1u16;
    }
    NorfairLavaMan_Func_20(game);
    let E2 = Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as i32 + 128i32, game);
    if ((*E2).var_04 & 0x8000) != 0 {
        (*E2).var_04 = 0u16;
        if IsSamusWithinEnemy_X(
            ram_word!(game, cur_enemy_index),
            *(&raw mut (*E).parameter_2 as *mut u8).offset(0) as u16,
            game,
        ) != 0
        {
            (*E).var_D = addr_kNorfairLavaMan_Ilist_ACDE as u16;
            if (*E).var_00 != 0 {
                (*E).var_D = addr_kNorfairLavaMan_Ilist_AD7E as u16;
            }
            NorfairLavaMan_Func_20(game);
            (*E).var_F = ((0xa8b175i32) as u16);
        }
    }
}
pub unsafe fn NorfairLavaMan_Func_8(k: u16, game: &mut Game) {
    let E = Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).var_01 == 0 {
        (*E).var_02 = 1u16;
        (*Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as i32 + 64i32, game)).var_0D = 0u16;
        (*E).var_F = ((0xa8b193i32) as u16);
    }
}
pub unsafe fn sub_A8B193(k: u16, game: &mut Game) {
    let E = Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).var_02 == 0 {
        (*E).var_D = addr_kNorfairLavaMan_Ilist_AD0C as u16;
        if (GetSamusEnemyDelta_X(ram_word!(game, cur_enemy_index), game) & 0x8000) == 0 {
            (*E).var_D = addr_kNorfairLavaMan_Ilist_ADAC as u16;
        }
        NorfairLavaMan_Func_20(game);
        (*E).var_F = ((0xa8b1b8i32) as u16);
    }
}
pub unsafe fn NorfairLavaMan_Func_9(k: u16, game: &mut Game) {
    let E = Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).var_01 == 0 {
        (*E).var_D = addr_kNorfairLavaMan_Ilist_AC9C as u16;
        if (GetSamusEnemyDelta_X(ram_word!(game, cur_enemy_index), game) & 0x8000) == 0 {
            (*E).var_D = addr_kNorfairLavaMan_Ilist_AD3C as u16;
        }
        NorfairLavaMan_Func_20(game);
        (*E).var_F = ((0xa8b11ai32) as u16);
    }
}
pub unsafe fn NorfairLavaMan_Func_10(k: u16, game: &mut Game) {
    let E = Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).var_0D == 0 {
        (*E).var_F = ((0xa8b204i32) as u16);
        set_fields! {[(*E)], 0u16; var_E, var_0D}
        (*E).var_B = 2u16;
        (*E).base.y_pos = (*E).var_03;
    }
}
pub unsafe fn NorfairLavaMan_Func_11(k: u16, game: &mut Game) {
    let E = Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as i32, game);
    let v3 = gEnemySpawnData(ram_word!(game, cur_enemy_index) as i32, game);
    let v: u32 = (((*v3.offset(31)).ypos2) as u32).wrapping_add(((*E).base.y_subpos) as u32);
    (*E).base.y_subpos = v as u16;
    (*E).base.y_pos = ((*E).base.y_pos as u32)
        .wrapping_add(((*v3.offset(31)).field_14 as u32).wrapping_add(v >> 16i32))
        as u16;
    (*E).var_E = ((*E).var_E as u32)
        .wrapping_add(((*v3.offset(31)).field_14 as u32).wrapping_add(v >> 16i32))
        as u16;
    NorfairLavaMan_Func_12(ram_word!(game, cur_enemy_index), game);
}
pub unsafe fn NorfairLavaMan_Func_12(k: u16, game: &mut Game) {
    let mut v3: i32 = 0;
    let E = Get_NorfairLavaMan(k as i32, game);
    if -((*E).var_E as i32) - 108i32 & 0x8000i32 == 0 {
        printf(
            b"Y undefined %d!\n\0" as *const u8 as *const c_char,
            -((*E).var_E as i16 as i32),
        );
    }
    let nlmn_var_B: u16 = (*E).var_B;
    let advance_lava_phase: bool = if -((*E).var_E as i32) - 108i32 & 0x8000i32 == 0 || {
        v3 = nlmn_var_B as i32 >> 1i32;
        (((*E).base.y_pos as i32
            - crate::rtl::rom_table_word!(game, 0xa8af79, v3 as isize) as i32
            - ram_word!(game, samus_y_pos) as i32) as i16)
            < 0i16
    } {
        (*E).var_F = ((0xa8b291i32) as u16);
        (*E).var_0C = 1u16;
        if ((-((*E).var_E as i32)
            - crate::rtl::rom_table_word!(game, 0xa8af55, (nlmn_var_B >> 1) as isize) as i32)
            as i16)
            < 0i16
        {
            return;
        }
        true
    } else {
        (((-((*E).var_E as i32) - crate::rtl::rom_table_word!(game, 0xa8af55, v3 as isize) as i32)
            as i16)
            >= 0i16)
    };
    if advance_lava_phase {
        (*E).var_B = (*E).var_B.wrapping_add(1);
        (*E).var_B = (*E).var_B.wrapping_add(1);
        (*E).base.y_pos = ((*E).base.y_pos).wrapping_add(8u16);
        (*E).var_D = crate::rtl::rom_table_word!(game, 0xa8af67, ((*E).var_B >> 1) as isize);
        NorfairLavaMan_Func_20(game);
    };
}
pub fn sub_A8B291(k: u16) {}
pub unsafe fn NorfairLavaMan_Func_13(game: &mut Game) {
    let E = Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as i32, game);
    let mut v2 = gEnemySpawnData(ram_word!(game, cur_enemy_index) as i32, game);
    let v4: bool = u16::overflowing_add((*v2.offset(31)).field_E, (*E).base.y_subpos).1;
    (*E).base.y_subpos = ((*v2.offset(31)).field_E).wrapping_add((*E).base.y_subpos);
    (*E).base.y_pos = ((*E).base.y_pos as i32 + ((*v2.offset(31)).xpos2 as i32 + v4 as i32)) as u16;
    (*E).var_E = ((*E).var_E as i32 + ((*v2.offset(31)).xpos2 as i32 + v4 as i32)) as u16;
    NorfairLavaMan_Func_14(ram_word!(game, cur_enemy_index), game);
}
pub unsafe fn NorfairLavaMan_Func_14(k: u16, game: &mut Game) {
    let E = Get_NorfairLavaMan(k as i32, game);
    let mut nlmn_var_E: i16 = (*E).var_E as i16;
    if nlmn_var_E >= 0i16 {
        (*E).var_0D = 1u16;
        (*gEnemySpawnData(k as i32, game).offset(31)).field_4 = 0u16;
        (*E).var_F = ((0xa8b1ddi32) as u16);
    } else if ((-(nlmn_var_E as i32)
        - crate::rtl::rom_table_word!(
            game,
            0xa8af55,
            ((((*E).var_B).wrapping_sub(2u16)) >> 1) as isize
        ) as i32) as i16)
        < 0i16
    {
        (*E).var_B = ((*E).var_B).wrapping_sub(2u16);
        (*E).base.y_pos = ((*E).base.y_pos).wrapping_sub(8u16);
        (*E).var_D = crate::rtl::rom_table_word!(game, 0xa8af67, ((*E).var_B >> 1) as isize);
        NorfairLavaMan_Func_20(game);
    }
}
pub unsafe fn NorfairLavaMan_Func_15(k: u16, game: &mut Game) {
    if *(ram_ptr!(game, u16; 0xe84))
        .offset(((ram_word!(game, cur_enemy_index) as i32 >> 1i32) + 87i32) as isize) as i32
        == 0xa8b175i32 & 0xffffi32
    {
        (*Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as i32, game)).var_F =
            ((0xa8b356i32) as u16);
    }
}
pub unsafe fn NorfairLavaMan_Func_16(k: u16, game: &mut Game) {
    let E = Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).var_01 == 0 {
        (*E).var_D = addr_kNorfairLavaMan_Ilist_AE0C as u16;
        NorfairLavaMan_Func_20(game);
        *(ram_ptr!(game, u16; 0xf68))
            .offset(((ram_word!(game, cur_enemy_index) as i32 >> 1i32) + 5i32) as isize) =
            ((0xa8b295i32) as u16);
        (*E).var_F = ((0xa8b3a7i32) as u16);
        (*gEnemySpawnData(ram_word!(game, cur_enemy_index) as i32, game).offset(31)).field_18 =
            0u16;
        (*E).base.x_pos = (*E).var_12;
        (*E).base.y_pos = (*E).var_13;
    }
    NorfairLavaMan_Func_19(ram_word!(game, cur_enemy_index), game);
}
pub unsafe fn NorfairLavaMan_Func_17(k: u16, game: &mut Game) {
    if (*gEnemySpawnData(ram_word!(game, cur_enemy_index) as i32, game).offset(31)).field_18 != 0 {
        let E = Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as i32, game);
        (*E).var_D = addr_kNorfairLavaMan_Ilist_ACB0 as u16;
        (*E).var_B = 0u16;
        if (GetSamusEnemyDelta_X(ram_word!(game, cur_enemy_index), game) & 0x8000) == 0 {
            (*E).var_D = addr_kNorfairLavaMan_Ilist_AD50 as u16;
            (*E).var_B = 1u16;
        }
        NorfairLavaMan_Func_20(game);
        (*E).var_F = ((0xa8b31fi32) as u16);
        (*E).var_12 = (*E).base.x_pos;
        (*E).var_13 = (*E).base.y_pos;
    } else {
        (*Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as i32, game))
            .base
            .y_pos = (*(ram_ptr!(game, u16; 0xe84))
            .offset(((ram_word!(game, cur_enemy_index) as i32 >> 1i32) + 93i32) as isize))
        .wrapping_sub(crate::rtl::rom_table_word!(
            game,
            0xa8af79,
            (*(ram_ptr!(game, u16; 0xf68))
                .offset(((ram_word!(game, cur_enemy_index) as i32 >> 1i32) + 1i32) as isize)
                >> 1) as isize,
        ));
    }
    NorfairLavaMan_Func_19(ram_word!(game, cur_enemy_index), game);
}
pub unsafe fn NorfairLavaMan_Func_18(k: u16, game: &mut Game) {
    let v1: i32 = ram_word!(game, cur_enemy_index) as i32 >> 1i32;
    if ram_word!(game, enemy_drawing_queue[(v1 + 87i32)]) as i32 == 0xa8b11ai32 & 0xffffi32 {
        (*Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as i32, game)).var_F =
            ((0xa8b30di32) as u16);
    } else {
        (*Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as i32, game))
            .base
            .y_pos = (ram_word!(game, enemy_drawing_queue[(v1 + 93i32)])).wrapping_sub(
            crate::rtl::rom_table_word!(
                game,
                0xa8af79,
                (ram_word!(game, enemy_drawing_queue_sizes[(v1 + 1i32)]) >> 1) as isize,
            ),
        );
    }
    NorfairLavaMan_Func_19(ram_word!(game, cur_enemy_index), game);
}
pub unsafe fn NorfairLavaMan_Func_19(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    let v2: u16 = (ram_word!(game, enemy_drawing_queue[(v1 + 61i32)]) as i32
        - (*Get_NorfairLavaMan(k as i32, game)).base.y_pos as i32
        + 2i32) as u16;
    set_ram_word!(game.enemy_drawing_queue[(v1 + 64i32)] = v2);
    if v2 as i32 - 8i32 & 0x8000i32 != 0 {
        set_ram_word!(game.enemy_drawing_queue[(v1 + 64i32)] = 8u16);
    }
}
pub unsafe fn NorfairLavaMan_Func_20(game: &mut Game) {
    let E = Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as i32, game);
    let nlmn_var_D: u16 = (*E).var_D;
    if nlmn_var_D != (*E).var_C {
        (*E).base.current_instruction = nlmn_var_D;
        (*E).var_C = nlmn_var_D;
        (*E).base.instruction_timer = 1u16;
        (*E).base.timer = 0u16;
    }
}
pub unsafe fn NorfairLavaMan_Powerbomb(game: &mut Game) {
    Enemy_NormalPowerBombAI_A8(game);
    NorfairLavaMan_Common(game);
}
pub unsafe fn NorfairLavaMan_Touch(game: &mut Game) {
    Enemy_NormalTouchAI_A8(game);
    NorfairLavaMan_Common(game);
}
pub unsafe fn NorfairLavaMan_Shot(game: &mut Game) {
    NormalEnemyShotAi(game);
    NorfairLavaMan_Common(game);
}
pub unsafe fn NorfairLavaMan_Common(game: &mut Game) {
    let E = Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as i32, game);
    let E1 = Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as i32 + 64i32, game);
    let E2 = Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as i32 + 128i32, game);
    if (*E).base.health == 0 {
        (*E1).base.properties = ((*E1).base.properties) | (0x200u16);
        (*E2).base.properties = ((*E2).base.properties) | (0x200u16);
    }
    let frozen_timer: u16 = (*E).base.frozen_timer;
    if frozen_timer != 0 {
        (*E1).base.frozen_timer = frozen_timer;
        (*E1).base.ai_handler_bits = ((*E1).base.ai_handler_bits) | (4u16);
        (*E2).base.frozen_timer = frozen_timer;
        (*E2).base.ai_handler_bits = ((*E2).base.ai_handler_bits) | (4u16);
    }
}
pub fn Beetom_Instr_1(k: u16, jp: *const u16) -> *const u16 {
    jp
}
pub unsafe fn Beetom_Func_1(game: &mut Game) {
    let E = Get_Beetom(ram_word!(game, cur_enemy_index) as i32, game);
    enemy_instruction! {[(*E).base]; instruction = (*E).var_00, timer = 1u16}
    (*E).base.timer = 0u16;
}
pub unsafe fn Beetom_Init(game: &mut Game) {
    let E = Get_Beetom(ram_word!(game, cur_enemy_index) as i32, game);
    set_fields! {[(*E)], 0u16; var_D, var_C, var_05, var_08, var_A}
    (*E).var_E = 64u16;
    (*E).var_F = ram_word!(game, joypad1_lastkeys);
    set_ram_word!(game.random_number = 23u16);
    (*E).var_02 = Beetom_Func_2(12288u16, 4u16, game);
    (*E).var_03 = Beetom_Func_2(0x4000u16, 5u16, game);
    (*E).var_04 = Beetom_Func_2(12288u16, 3u16, game);
    (*E).var_00 = addr_kBeetom_Ilist_B6F2 as u16;
    if (GetSamusEnemyDelta_X(ram_word!(game, cur_enemy_index), game) & 0x8000) != 0 {
        (*E).var_00 = addr_kBeetom_Ilist_B696 as u16;
    }
    Beetom_Func_1(game);
    (*E).var_C = ((0xa8b814i32) as u16);
}
pub unsafe fn Beetom_Func_2(r22: u16, r24: u16, game: &mut Game) -> u16 {
    let mut r18: u16 = 0u16;
    let mut r20: u16 = 0u16;
    loop {
        r18 = (r18).wrapping_add(r24);
        r20 = (r20).wrapping_add(
            *((game.g_rom.offset(crate::rtl::lorom_offset(0xa2838f)) as *mut u16 as *mut u8)
                .offset((8i32 * r18 as i32) as isize)
                .offset(1) as *mut u16),
        );
        if (r20 as i32 - r22 as i32 & 0x8000i32 == 0) {
            break;
        }
    }
    r18
}
pub unsafe fn Beetom_Func_3(game: &mut Game) {
    if IsSamusWithinEnemy_X(ram_word!(game, cur_enemy_index), 0x60u16, game) != 0 {
        (*Get_Beetom(ram_word!(game, cur_enemy_index) as i32, game)).var_C = ((0xa8b90fi32) as u16);
    } else {
        (*Get_Beetom(ram_word!(game, cur_enemy_index) as i32, game)).var_C = ((0xa8b82fi32) as u16);
    };
}
pub unsafe fn Beetom_Func_4(game: &mut Game) {
    NextRandom(game);
    let E = Get_Beetom(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).var_C = crate::rtl::rom_table_word!(
        game,
        0xa8b74e,
        (ram_word!(game, random_number) as i32 & 7i32) as isize
    );
    (*E).var_09 = (ram_word!(game, random_number)) & (1u16);
}
pub unsafe fn Beetom_Func_5(game: &mut Game) {
    let E = Get_Beetom(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).var_D = 32u16;
    (*E).var_C = ((0xa8b9b2i32) as u16);
}
pub unsafe fn Beetom_Func_6(game: &mut Game) {
    let E = Get_Beetom(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).var_C = ((0xa8b9c1i32) as u16);
    (*E).var_00 = addr_kBeetom_Ilist_B696 as u16;
    Beetom_Func_1(game);
}
pub unsafe fn Beetom_Func_7(game: &mut Game) {
    let E = Get_Beetom(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).var_C = ((0xa8ba24i32) as u16);
    (*E).var_00 = addr_kBeetom_Ilist_B6F2 as u16;
    Beetom_Func_1(game);
}
pub unsafe fn Beetom_Func_8(game: &mut Game) {
    let E = Get_Beetom(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).var_B = (*E).var_02;
    (*E).var_C = ((0xa8ba84i32) as u16);
    (*E).var_05 = 0u16;
    (*E).var_00 = addr_kBeetom_Ilist_B6AC as u16;
    Beetom_Func_1(game);
}
pub unsafe fn Beetom_Func_9(game: &mut Game) {
    let E = Get_Beetom(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).var_B = (*E).var_02;
    (*E).var_C = ((0xa8bab7i32) as u16);
    (*E).var_05 = 0u16;
    (*E).var_00 = addr_kBeetom_Ilist_B708 as u16;
    Beetom_Func_1(game);
}
pub unsafe fn Beetom_Func_10(game: &mut Game) {
    let E = Get_Beetom(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).var_B = (*E).var_03;
    (*E).var_C = ((0xa8bb55i32) as u16);
    (*E).var_05 = 0u16;
    (*E).var_00 = addr_kBeetom_Ilist_B6AC as u16;
    Beetom_Func_1(game);
}
pub unsafe fn Beetom_Func_11(game: &mut Game) {
    let E = Get_Beetom(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).var_B = (*E).var_03;
    (*E).var_C = ((0xa8bb88i32) as u16);
    (*E).var_05 = 0u16;
    (*E).var_00 = addr_kBeetom_Ilist_B708 as u16;
    Beetom_Func_1(game);
}
pub unsafe fn Beetom_Func_12(game: &mut Game) {
    let E = Get_Beetom(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).var_B = (*E).var_04;
    (*E).var_00 = addr_kBeetom_Ilist_B708 as u16;
    (*E).var_C = ((0xa8bc5ai32) as u16);
    (*E).var_09 = 1u16;
    if (GetSamusEnemyDelta_X(ram_word!(game, cur_enemy_index), game) & 0x8000) != 0 {
        (*E).var_00 = addr_kBeetom_Ilist_B6AC as u16;
        (*E).var_C = ((0xa8bc26i32) as u16);
        (*E).var_09 = 0u16;
    }
    Beetom_Func_1(game);
    (*E).var_05 = 0u16;
}
pub unsafe fn Beetom_Func_13(game: &mut Game) {
    let E = Get_Beetom(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).var_00 = addr_kBeetom_Ilist_B6CC as u16;
    Beetom_Func_1(game);
    (*E).var_C = ((0xa8bcf8i32) as u16);
}
pub unsafe fn Beetom_Func_14(game: &mut Game) {
    let E = Get_Beetom(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).var_00 = addr_kBeetom_Ilist_B728 as u16;
    Beetom_Func_1(game);
    (*E).var_C = ((0xa8bd42i32) as u16);
}
pub unsafe fn Beetom_Func_15(game: &mut Game) {
    let E = Get_Beetom(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).var_00 = addr_kBeetom_Ilist_B696 as u16;
    if (*E).var_09 != 0 {
        (*E).var_00 = addr_kBeetom_Ilist_B6F2 as u16;
    }
    Beetom_Func_1(game);
    (*E).var_05 = 0u16;
    (*E).var_C = ((0xa8bd9di32) as u16);
}
pub unsafe fn Beetom_Func_16(game: &mut Game) {
    let E = Get_Beetom(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).var_B = 0u16;
    (*E).var_C = ((0xa8bdc5i32) as u16);
}
pub unsafe fn Beetom_Func_17(game: &mut Game) {
    let E = Get_Beetom(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).var_D = (*E).var_D.wrapping_sub(1);
    if ((*E).var_D & 0x8000) != 0 {
        (*E).var_C = ((0xa8b814i32) as u16);
    }
}
pub unsafe fn Beetom_Func_18(game: &mut Game) {
    let E = Get_Beetom(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).var_D = (*E).var_D.wrapping_sub(1);
    if ((*E).var_D & 0x8000) != 0 {
        (*E).var_D = 64u16;
        (*E).var_C = ((0xa8b814i32) as u16);
    } else {
        (*E).base.x_pos = ((*E).base.x_pos).wrapping_sub(8u16);
        if Enemy_MoveDown(
            ram_word!(game, cur_enemy_index),
            (1i16 as i32) << 16i32,
            game,
        ) != 0
        {
            (*E).base.x_pos = ((*E).base.x_pos).wrapping_add(8u16);
            if Enemy_MoveRight_IgnoreSlopes(ram_word!(game, cur_enemy_index), -16384i32, game) != 0
            {
                (*E).var_C = ((0xa8b873i32) as u16);
            }
        } else {
            (*E).var_C = ((0xa8b873i32) as u16);
            (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
            (*E).base.x_pos = ((*E).base.x_pos).wrapping_add(8u16);
        }
    };
}
pub unsafe fn Beetom_Func_19(game: &mut Game) {
    let E = Get_Beetom(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).var_D = (*E).var_D.wrapping_sub(1);
    if ((*E).var_D & 0x8000) != 0 {
        (*E).var_D = 64u16;
        (*E).var_C = ((0xa8b814i32) as u16);
    } else {
        (*E).base.x_pos = ((*E).base.x_pos).wrapping_add(8u16);
        if Enemy_MoveDown(
            ram_word!(game, cur_enemy_index),
            (1i16 as i32) << 16i32,
            game,
        ) != 0
        {
            (*E).base.x_pos = ((*E).base.x_pos).wrapping_sub(8u16);
            if Enemy_MoveRight_IgnoreSlopes(ram_word!(game, cur_enemy_index), 0x4000i32, game) != 0
            {
                (*E).var_C = ((0xa8b85fi32) as u16);
            }
        } else {
            (*E).var_C = ((0xa8b85fi32) as u16);
            (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
            (*E).base.x_pos = ((*E).base.x_pos).wrapping_sub(8u16);
        }
    };
}
pub unsafe fn Beetom_Func_22(k: u16, game: &mut Game) {
    let E = Get_Beetom(k as i32, game);
    let v2: i32 = 8i32 * (*E).var_B as i32 >> 1i32;
    if Enemy_MoveDown(
        k,
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa0cbc7)) as *mut u32)
            .offset((v2 + 2i32 >> 1i32) as isize) as i32,
        game,
    ) != 0
    {
        (*E).var_C = ((0xa8b97ai32) as u16);
    } else {
        (*E).var_B = ((*E).var_B).wrapping_sub(4u16);
        if (*E).var_B & 0x8000 != 0 {
            (*E).var_B = 0u16;
            (*E).var_05 = 1u16;
        }
    };
}
pub unsafe fn Beetom_Func_23(k: u16, game: &mut Game) {
    let E = Get_Beetom(k as i32, game);
    let v2: i32 = 8i32 * (*E).var_B as i32 >> 1i32;
    if Enemy_MoveDown(
        k,
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa0cbc7)) as *mut u32)
            .offset((v2 >> 1i32) as isize) as i32,
        game,
    ) != 0
    {
        (*E).var_C = ((0xa8b814i32) as u16);
    } else {
        (*E).var_B = ((*E).var_B).wrapping_add(4u16);
        if (*E).var_B as i32 - 64i32 & 0x8000i32 == 0 {
            (*E).var_B = 64u16;
        }
    };
}
pub unsafe fn Beetom_Func_20(game: &mut Game) {
    let E = Get_Beetom(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).var_05 != 0 {
        Beetom_Func_23(ram_word!(game, cur_enemy_index), game);
    } else {
        Beetom_Func_22(ram_word!(game, cur_enemy_index), game);
    }
    if Enemy_MoveRight_IgnoreSlopes(ram_word!(game, cur_enemy_index), -16384i32, game) != 0 {
        (*E).var_09 = ((*E).var_09) ^ (1u16);
        (*E).var_C = ((0xa8b97ai32) as u16);
    }
}
pub unsafe fn Beetom_Func_21(game: &mut Game) {
    let E = Get_Beetom(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).var_05 != 0 {
        Beetom_Func_23(ram_word!(game, cur_enemy_index), game);
    } else {
        Beetom_Func_22(ram_word!(game, cur_enemy_index), game);
    }
    if Enemy_MoveRight_IgnoreSlopes(ram_word!(game, cur_enemy_index), 0x4000i32, game) != 0 {
        (*E).var_09 = ((*E).var_09) ^ (1u16);
        (*E).var_C = ((0xa8b97ai32) as u16);
    }
}
pub unsafe fn Beetom_Func_26(k: u16, game: &mut Game) {
    let E = Get_Beetom(k as i32, game);
    let v2: i32 = 8i32 * (*E).var_B as i32 >> 1i32;
    if Enemy_MoveDown(
        k,
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa0cbc7)) as *mut u32)
            .offset((v2 + 2i32 >> 1i32) as isize) as i32,
        game,
    ) != 0
    {
        (*E).var_C = ((0xa8b97ai32) as u16);
    } else {
        (*E).var_B = ((*E).var_B).wrapping_sub(5u16);
        if (*E).var_B & 0x8000 != 0 {
            (*E).var_B = 0u16;
            (*E).var_05 = 1u16;
        }
    };
}
pub unsafe fn Beetom_Func_27(k: u16, game: &mut Game) {
    let E = Get_Beetom(k as i32, game);
    let v2: i32 = 8i32 * (*E).var_B as i32 >> 1i32;
    if Enemy_MoveDown(
        k,
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa0cbc7)) as *mut u32)
            .offset((v2 >> 1i32) as isize) as i32,
        game,
    ) != 0
    {
        (*E).var_C = ((0xa8b814i32) as u16);
    } else {
        (*E).var_B = ((*E).var_B).wrapping_add(5u16);
        if (*E).var_B as i32 - 64i32 & 0x8000i32 == 0 {
            (*E).var_B = 64u16;
        }
    };
}
pub unsafe fn Beetom_Func_24(game: &mut Game) {
    let E = Get_Beetom(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).var_05 != 0 {
        Beetom_Func_27(ram_word!(game, cur_enemy_index), game);
    } else {
        Beetom_Func_26(ram_word!(game, cur_enemy_index), game);
    }
    if Enemy_MoveRight_IgnoreSlopes(ram_word!(game, cur_enemy_index), -16384i32, game) != 0 {
        (*E).var_09 = ((*E).var_09) ^ (1u16);
        (*E).var_C = ((0xa8b97ai32) as u16);
    }
}
pub unsafe fn Beetom_Func_25(game: &mut Game) {
    let E = Get_Beetom(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).var_05 != 0 {
        Beetom_Func_27(ram_word!(game, cur_enemy_index), game);
    } else {
        Beetom_Func_26(ram_word!(game, cur_enemy_index), game);
    }
    if Enemy_MoveRight_IgnoreSlopes(ram_word!(game, cur_enemy_index), 0x4000i32, game) != 0 {
        (*E).var_09 = ((*E).var_09) ^ (1u16);
        (*E).var_C = ((0xa8b97ai32) as u16);
    }
}
pub unsafe fn Beetom_Func_30(k: u16, game: &mut Game) {
    let E = Get_Beetom(k as i32, game);
    let v2: i32 = 8i32 * (*E).var_B as i32 >> 1i32;
    if Enemy_MoveDown(
        k,
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa0cbc7)) as *mut u32)
            .offset((v2 + 2i32 >> 1i32) as isize) as i32,
        game,
    ) != 0
    {
        (*E).var_C = ((0xa8b97ai32) as u16);
    } else {
        let v3: i16 = (((*E).var_B) as i16).wrapping_sub(5i16);
        (*E).var_B = v3 as u16;
        if v3 < 0i16 {
            (*E).var_B = 0u16;
            (*E).var_05 = 1u16;
        }
    };
}
pub unsafe fn Beetom_Func_31(k: u16, game: &mut Game) {
    let E = Get_Beetom(k as i32, game);
    let v2: i32 = 8i32 * (*E).var_B as i32 >> 1i32;
    if Enemy_MoveDown(
        k,
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa0cbc7)) as *mut u32)
            .offset((v2 >> 1i32) as isize) as i32,
        game,
    ) != 0
    {
        (*E).var_C = ((0xa8b814i32) as u16);
    } else {
        let v3: u16 = ((*E).var_B).wrapping_add(3u16);
        (*E).var_B = v3;
        if v3 as i32 - 64i32 & 0x8000i32 == 0 {
            (*E).var_B = 64u16;
        }
    };
}
pub unsafe fn Beetom_Func_28(game: &mut Game) {
    let E = Get_Beetom(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).var_05 != 0 {
        Beetom_Func_31(ram_word!(game, cur_enemy_index), game);
    } else {
        Beetom_Func_30(ram_word!(game, cur_enemy_index), game);
    }
    if Enemy_MoveRight_IgnoreSlopes(
        ram_word!(game, cur_enemy_index),
        ((-3i32 as u32) << 16i32 | 0u16 as u32) as i32,
        game,
    ) != 0
    {
        (*E).var_09 = ((*E).var_09) ^ (1u16);
        (*E).var_C = ((0xa8b97ai32) as u16);
    }
}
pub unsafe fn Beetom_Func_29(game: &mut Game) {
    let E = Get_Beetom(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).var_05 != 0 {
        Beetom_Func_31(ram_word!(game, cur_enemy_index), game);
    } else {
        Beetom_Func_30(ram_word!(game, cur_enemy_index), game);
    }
    if Enemy_MoveRight_IgnoreSlopes(
        ram_word!(game, cur_enemy_index),
        ((3u32) << 16i32 | 0u16 as u32) as i32,
        game,
    ) != 0
    {
        (*E).var_09 = ((*E).var_09) ^ (1u16);
        (*E).var_C = ((0xa8b97ai32) as u16);
    }
}
pub unsafe fn Beetom_Func_34(game: &mut Game) {
    let E = Get_Beetom(ram_word!(game, cur_enemy_index) as i32, game);
    if ram_word!(game, joypad1_lastkeys) != (*E).var_F {
        (*E).var_F = ram_word!(game, joypad1_lastkeys);
        (*E).var_E = (*E).var_E.wrapping_sub(1);
    }
}
pub unsafe fn Beetom_Func_32(game: &mut Game) {
    let E = Get_Beetom(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).var_E != 0 {
        (*E).base.x_pos = ram_word!(game, samus_x_pos);
        (*E).base.y_pos = (ram_word!(game, samus_y_pos)).wrapping_sub(4u16);
        Beetom_Func_34(game);
    } else {
        if Enemy_MoveRight_IgnoreSlopes(
            ram_word!(game, cur_enemy_index),
            ((16u32) << 16i32 | 0u16 as u32) as i32,
            game,
        ) != 0
        {
            (*E).var_09 = 1u16;
            Enemy_MoveRight_IgnoreSlopes(
                ram_word!(game, cur_enemy_index),
                ((-32i32 as u32) << 16i32 | 0u16 as u32) as i32,
                game,
            );
        }
        (*E).var_08 = 0u16;
        (*E).var_C = ((0xa8b9a2i32) as u16);
    };
}
pub unsafe fn Beetom_Func_33(game: &mut Game) {
    let E = Get_Beetom(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).var_E != 0 {
        (*E).base.x_pos = ram_word!(game, samus_x_pos);
        (*E).base.y_pos = (ram_word!(game, samus_y_pos)).wrapping_sub(4u16);
        Beetom_Func_34(game);
    } else {
        if Enemy_MoveRight_IgnoreSlopes(
            ram_word!(game, cur_enemy_index),
            ((-16i32 as u32) << 16i32 | 0u16 as u32) as i32,
            game,
        ) != 0
        {
            (*E).var_09 = 0u16;
            Enemy_MoveRight_IgnoreSlopes(
                ram_word!(game, cur_enemy_index),
                ((32u32) << 16i32 | 0u16 as u32) as i32,
                game,
            );
        }
        (*E).var_08 = 0u16;
        (*E).var_C = ((0xa8b9a2i32) as u16);
    };
}
pub unsafe fn Beetom_Func_35(game: &mut Game) {
    if Enemy_MoveDown(
        ram_word!(game, cur_enemy_index),
        ((3u32) << 16i32 | 0u16 as u32) as i32,
        game,
    ) != 0
    {
        let E = Get_Beetom(ram_word!(game, cur_enemy_index) as i32, game);
        if (*E).var_09 != 0 {
            (*E).var_C = ((0xa8b873i32) as u16);
        } else {
            (*E).var_C = ((0xa8b85fi32) as u16);
        }
    }
}
pub unsafe fn Beetom_Func_37(k: u16, game: &mut Game) {
    let E = Get_Beetom(k as i32, game);
    let v2: i32 = 8i32 * (*E).var_B as i32 >> 1i32;
    if Enemy_MoveDown(
        k,
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa0cbc7)) as *mut u32)
            .offset((v2 >> 1i32) as isize) as i32,
        game,
    ) != 0
    {
        (*E).var_C = ((0xa8b814i32) as u16);
    } else {
        let v3: u16 = ((*E).var_B).wrapping_add(1u16);
        (*E).var_B = v3;
        if v3 as i32 - 64i32 & 0x8000i32 == 0 {
            (*E).var_B = 64u16;
        }
    }
    let v4: u16 = (if (*E).var_09 != 0u16 { -2i32 } else { 2i32 }) as u16;
    if Enemy_MoveRight_IgnoreSlopes(k, ((v4 as u32) << 16i32 | 0u16 as u32) as i32, game) != 0 {
        (*E).var_09 = ((*E).var_09) ^ (1u16);
        (*E).var_C = ((0xa8b97ai32) as u16);
    }
}
pub unsafe fn Beetom_Func_36(game: &mut Game) {
    Beetom_Func_37(ram_word!(game, cur_enemy_index), game);
}
crate::game::address_dispatch! {
    pub unsafe fn CallBeetomFunc(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        11057172 => Beetom_Func_3; 11057199 => Beetom_Func_4; 11057231 => Beetom_Func_5;
        11057247 => Beetom_Func_6; 11057267 => Beetom_Func_7; 11057287 => Beetom_Func_8;
        11057321 => Beetom_Func_9; 11057355 => Beetom_Func_10; 11057389 => Beetom_Func_11;
        11057423 => Beetom_Func_12; 11057490 => Beetom_Func_13; 11057510 => Beetom_Func_14;
        11057530 => Beetom_Func_15; 11057570 => Beetom_Func_16; 11057586 => Beetom_Func_17;
        11057601 => Beetom_Func_18; 11057700 => Beetom_Func_19; 11057796 => Beetom_Func_20;
        11057847 => Beetom_Func_21; 11058005 => Beetom_Func_24; 11058056 => Beetom_Func_25;
        11058214 => Beetom_Func_28; 11058266 => Beetom_Func_29; 11058424 => Beetom_Func_32;
        11058498 => Beetom_Func_33; 11058589 => Beetom_Func_35; 11058629 => Beetom_Func_36;
    }
}
pub unsafe fn Beetom_Main(game: &mut Game) {
    let E = Get_Beetom(ram_word!(game, cur_enemy_index) as i32, game);
    CallBeetomFunc((((*E).var_C) as u32) | (0xa80000u32), game);
}
pub unsafe fn Beetom_Touch(game: &mut Game) {
    let E = Get_Beetom(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).var_08 == 0 {
        if (*E).var_09 != 0 {
            (*E).var_C = ((0xa8b966i32) as u16);
        } else {
            (*E).var_C = ((0xa8b952i32) as u16);
        }
        (*E).var_08 = 1u16;
        (*E).var_E = 64u16;
        (*E).base.layer = 2u16;
        (*E).var_06 = (ram_word!(game, samus_x_pos)).wrapping_sub((*E).base.x_pos);
        (*E).var_07 = (ram_word!(game, samus_y_pos)).wrapping_sub((*E).base.y_pos);
    }
    let apply_touch_damage: bool = (ram_word!(game, samus_contact_damage_index) != 0)
        || ({
            if (ram_word!(game, random_enemy_counter) & 7) == 7
                && ram_word!(game, samus_health) as i32 - 30i32 & 0x8000i32 == 0
            {
                QueueSfx3_Max6(0x2du16, game);
            }
            (((*E).base.frame_counter & 0x3f) == 63)
        });
    if apply_touch_damage {
        Enemy_NormalTouchAI_A8(game);
        set_ram_words! { game, 0u16; samus_invincibility_timer, samus_knockback_timer }
    };
}
pub unsafe fn Beetom_Shot(game: &mut Game) {
    Enemy_NormalShotAI_A8(game);
    let E = Get_Beetom(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).base.frozen_timer != 0u16
        && ((*E).var_C as i32 == 0xa8bd42i32 & 0xffffi32
            || (*E).var_C as i32 == 0xa8bcf8i32 & 0xffffi32)
    {
        (*E).var_C = ((0xa8b97ai32) as u16);
    }
    (*E).var_08 = 0u16;
}
pub unsafe fn MaridiaFloater_Init(game: &mut Game) {
    let E = Get_MaridiaFloater(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_DisableSamusColl) as u16);
    (*E).base.spritemap_pointer = addr_kSpritemap_Nothing_A8 as u16;
    (*E).base.instruction_timer = 1u16;
    (*E).base.timer = 0u16;
    if (*E).parameter_1 != 0 {
        (*E).var_A = (*E).base.x_pos;
        (*E).var_B = (*E).base.y_pos;
        (*E).var_F = ((0xa8c568i32) as u16);
        (*E).base.current_instruction = addr_stru_A8C199 as u16;
        (*E).var_D = (*E).parameter_2;
    } else {
        (*E).var_E = 60u16;
        (*E).var_F = ((0xa8c283i32) as u16);
        (*E).base.current_instruction = addr_kMaridiaFloater_Ilist_C173 as u16;
    };
}
pub unsafe fn MaridiaFloater_Main(game: &mut Game) {
    let E = Get_MaridiaFloater(ram_word!(game, cur_enemy_index) as i32, game);
    CallEnemyPreInstr((((*E).var_F) as u32) | (0xa80000u32), game);
}
pub unsafe fn MaridiaFloater_Func_1(k: u16, game: &mut Game) {
    let mut i: i32 = 7i32;
    while i >= 0i32 {
        SpawnEprojWithGfx(i as u16, k, addr_kEproj_MaridiaFloatersSpikes as u16, game);
        i -= 1;
    }
}
pub unsafe fn MaridiaFloater_Func_2(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    let v2: u16 = ram_word!(game, enemy_drawing_queue[(v1 + 103i32)]);
    if v2 as i32 + 0x3e6fi32 & 0x8000i32 != 0 {
        let mut v3: u16 = ((v2 as i32 - 4i32 + 15997i32) as u16 >> 1);
        if v3 as i32 - 6i32 & 0x8000i32 == 0 {
            v3 = 0u16;
        }
        set_ram_word!(
            game.enemy_drawing_queue[(v1 + 93i32)] = ((g_word_A8C277[(v3 >> 1) as usize]) as u16)
                .wrapping_add((*Get_MaridiaFloater(k as i32, game)).base.y_pos,)
        );
    } else {
        let mut v4: u16 = ((v2 as i32 - 4i32 + 15983i32) as u16 >> 1);
        if v4 as i32 - 6i32 & 0x8000i32 == 0 {
            v4 = 0u16;
        }
        set_ram_word!(
            game.enemy_drawing_queue[(v1 + 93i32)] = ((g_word_A8C27D[(v4 >> 1) as usize]) as u16)
                .wrapping_add((*Get_MaridiaFloater(k as i32, game)).base.y_pos,)
        );
    };
}
pub unsafe fn MaridiaFloater_Func_3(k: u16, game: &mut Game) {
    let E = Get_MaridiaFloater(k as i32, game);
    let v2: bool = (*E).var_E == 1u16;
    (*E).var_E = (*E).var_E.wrapping_sub(1);
    let v3: bool = ((*E).var_E & 0x8000) != 0;
    if v2 || v3 {
        let v4: i32 = k as i32 >> 1i32;
        set_ram_word!(game.enemy_drawing_queue[(v4 + 104i32)] = 1u16);
        set_ram_word!(
            game.enemy_drawing_queue[(v4 + 103i32)] = addr_kMaridiaFloater_Ilist_C183 as u16
        );
        (*E).var_F = ((0xa8c2a6i32) as u16);
        (*E).var_E = 10u16;
    }
    MaridiaFloater_Func_2(k, game);
}
pub unsafe fn MaridiaFloater_Func_4(k: u16, game: &mut Game) {
    let E = Get_MaridiaFloater(k as i32, game);
    let v2: bool = (*E).var_E == 1u16;
    (*E).var_E = (*E).var_E.wrapping_sub(1);
    let v3: bool = ((*E).var_E & 0x8000) != 0;
    if v2 || v3 {
        (*E).var_F = ((0xa8c2cfi32) as u16);
        (*E).var_A = g_word_A8C1C5;
        (*E).var_B = g_word_A8C1C7;
        enemy_instruction! {[(*E).base]; timer = 1u16, instruction = addr_kMaridiaFloater_Ilist_C163 as u16}
    }
    MaridiaFloater_Func_2(k, game);
}
pub unsafe fn MaridiaFloater_Func_5(k: u16, game: &mut Game) {
    let E = Get_MaridiaFloater(k as i32, game);
    if ((*E).base.ai_handler_bits & 1) != 0 {
        (*E).var_F = ((0xa8c3e1i32) as u16);
        MaridiaFloater_Func_7(k, game);
    } else {
        let v2: bool = (*E).var_D == 1u16;
        (*E).var_D = (*E).var_D.wrapping_sub(1);
        let v3: bool = ((*E).var_D & 0x8000) != 0;
        if v2 || v3 {
            (*E).var_D = 5u16;
            let v4: i32 = k as i32 >> 1i32;
            let v5: u16 = (g_word_A8C1A1[(*E).var_C as usize])
                .wrapping_add(ram_word!(game, enemy_drawing_queue_sizes[v4]));
            set_ram_word!(game.enemy_drawing_queue[(v4 + 91i32)] = v5);
            (*E).base.x_pos = v5;
            let mut v6: u16 = ((*E).var_C).wrapping_add(1u16);
            if (*E).var_C as i32 - 11i32 & 0x8000i32 == 0 {
                v6 = 0u16;
            }
            (*E).var_C = v6;
        }
        AddToHiLo(
            &raw mut (*E).var_A,
            &raw mut (*E).var_B,
            (g_word_A8C1B9 as u32) << 16i32 | g_word_A8C1BB as u32,
        );
        if Enemy_MoveDown(
            k,
            (((*E).var_A as u32) << 16i32 | (*E).var_B as u32) as i32,
            game,
        ) != 0u8
            || ((*(ram_ptr!(game, u16; 0xf68)).offset(((k as i32 >> 1i32) + 1i32) as isize) as i32
                - g_word_A8C19F as i32
                - (*E).base.y_pos as i32) as i16)
                >= 0i16
        {
            let mfr_var_C: u16 = (*E).var_C;
            if mfr_var_C == 0 || mfr_var_C == 6u16 {
                (*E).var_F = ((0xa8c500i32) as u16);
                (*E).var_E = 10u16;
                let v10: i32 = k as i32 >> 1i32;
                set_ram_word!(game.enemy_drawing_queue[(v10 + 104i32)] = 1u16);
                set_ram_word!(
                    game.enemy_drawing_queue[(v10 + 103i32)] =
                        addr_kMaridiaFloater_Ilist_C191 as u16
                );
            } else {
                (*E).var_F = ((0xa8c36bi32) as u16);
            }
        }
        MaridiaFloater_Func_2(k, game);
    };
}
pub unsafe fn MaridiaFloater_Func_6(k: u16, game: &mut Game) {
    let E = Get_MaridiaFloater(k as i32, game);
    let v2: bool = (*E).var_D == 1u16;
    (*E).var_D = (*E).var_D.wrapping_sub(1);
    let v3: bool = ((*E).var_D & 0x8000) != 0;
    let move_down: bool = (!v2 && !v3)
        || ({
            (*E).var_D = 5u16;
            let mut v4: i32 = 0;
            v4 = k as i32 >> 1i32;
            let v5: u16 = (g_word_A8C1A1[(*E).var_C as usize])
                .wrapping_add(ram_word!(game, enemy_drawing_queue_sizes[v4]));
            set_ram_word!(game.enemy_drawing_queue[(v4 + 91i32)] = v5);
            (*E).base.x_pos = v5;
            if (*E).var_C != 0u16 && (*E).var_C != 6u16 {
                let mut v7: u16 = ((*E).var_C).wrapping_add(1u16);
                if v7 as i32 - 12i32 & 0x8000i32 == 0 {
                    v7 = 0u16;
                }
                (*E).var_C = v7;
                true
            } else {
                (*E).var_F = ((0xa8c500i32) as u16);
                (*E).var_E = 10u16;
                set_ram_word!(game.enemy_drawing_queue[(v4 + 104i32)] = 1u16);
                set_ram_word!(
                    game.enemy_drawing_queue[(v4 + 103i32)] =
                        addr_kMaridiaFloater_Ilist_C191 as u16
                );
                false
            }
        });
    if move_down {
        AddToHiLo(
            &raw mut (*E).var_A,
            &raw mut (*E).var_B,
            (g_word_A8C1B9 as u32) << 16i32 | g_word_A8C1BB as u32,
        );
        Enemy_MoveDown(
            k,
            (((*E).var_A as u32) << 16i32 | (*E).var_B as u32) as i32,
            game,
        );
    }
    MaridiaFloater_Func_2(k, game);
}
pub unsafe fn MaridiaFloater_Func_7(k: u16, game: &mut Game) {
    let E = Get_MaridiaFloater(k as i32, game);
    if ((*E).base.ai_handler_bits & 1) != 0 {
        let v2: bool = (*E).var_D == 1u16;
        (*E).var_D = (*E).var_D.wrapping_sub(1);
        let v3: bool = ((*E).var_D & 0x8000) != 0;
        if v2 || v3 {
            (*E).var_D = 5u16;
            let v4: i32 = k as i32 >> 1i32;
            let v5: u16 = (g_word_A8C1A1[(*E).var_C as usize])
                .wrapping_add(ram_word!(game, enemy_drawing_queue_sizes[v4]));
            set_ram_word!(game.enemy_drawing_queue[(v4 + 91i32)] = v5);
            (*E).base.x_pos = v5;
            let mut v6: u16 = ((*E).var_C).wrapping_add(1u16);
            if (*E).var_C as i32 - 11i32 & 0x8000i32 == 0 {
                v6 = 0u16;
            }
            (*E).var_C = v6;
        }
        AddToHiLo(
            &raw mut (*E).var_A,
            &raw mut (*E).var_B,
            (g_word_A8C1B9 as u32) << 16i32 | g_word_A8C1BB as u32,
        );
        if Enemy_MoveDown(
            k,
            (((*E).var_A as u32) << 16i32 | (*E).var_B as u32) as i32,
            game,
        ) != 0u8
            || ((*(ram_ptr!(game, u16; 0xf68)).offset(((k as i32 >> 1i32) + 1i32) as isize) as i32
                - *(ram_ptr!(game, u16; 0xf68)).offset(((k as i32 >> 1i32) + 3i32) as isize) as i32
                - (*E).base.y_pos as i32) as i16)
                >= 0i16
        {
            if (*E).var_C == 0 || (*E).var_C == 6u16 {
                (*E).var_F = ((0xa8c4dci32) as u16);
            } else {
                (*E).var_F = ((0xa8c469i32) as u16);
            }
        }
        MaridiaFloater_Func_2(k, game);
    } else {
        (*E).var_F = ((0xa8c36bi32) as u16);
    };
}
pub unsafe fn MaridiaFloater_Func_8(k: u16, game: &mut Game) {
    let E = Get_MaridiaFloater(k as i32, game);
    if ((*E).base.ai_handler_bits & 1) == 0 {
        (*E).var_F = ((0xa8c36bi32) as u16);
        return;
    }
    let v2: bool = (*E).var_D == 1u16;
    (*E).var_D = (*E).var_D.wrapping_sub(1);
    let v3: bool = ((*E).var_D & 0x8000) != 0;
    let move_down: bool = (!v2 && !v3)
        || ({
            (*E).var_D = 5u16;
            let mut v4: i32 = 0;
            v4 = k as i32 >> 1i32;
            let mut v5: u16 = 0;
            v5 = (g_word_A8C1A1[(*E).var_C as usize])
                .wrapping_add(ram_word!(game, enemy_drawing_queue_sizes[v4]));
            set_ram_word!(game.enemy_drawing_queue[(v4 + 91i32)] = v5);
            (*E).base.x_pos = v5;
            if (*E).var_C != 0u16 && (*E).var_C != 6u16 {
                let mut v7: u16 = ((*E).var_C).wrapping_add(1u16);
                if v7 as i32 - 12i32 & 0x8000i32 == 0 {
                    v7 = 0u16;
                }
                (*E).var_C = v7;
                true
            } else {
                (*E).var_F = ((0xa8c4dci32) as u16);
                false
            }
        });
    if move_down {
        AddToHiLo(
            &raw mut (*E).var_A,
            &raw mut (*E).var_B,
            (g_word_A8C1B9 as u32) << 16i32 | g_word_A8C1BB as u32,
        );
        Enemy_MoveDown(
            k,
            (((*E).var_A as u32) << 16i32 | (*E).var_B as u32) as i32,
            game,
        );
    }
    MaridiaFloater_Func_2(k, game);
}
pub unsafe fn MaridiaFloater_Func_9(k: u16, game: &mut Game) {
    let E = Get_MaridiaFloater(k as i32, game);
    if ((*E).base.ai_handler_bits & 1) == 0 {
        (*E).var_F = ((0xa8c500i32) as u16);
        (*E).var_E = 10u16;
        let v2: i32 = k as i32 >> 1i32;
        set_ram_word!(game.enemy_drawing_queue[(v2 + 104i32)] = 1u16);
        set_ram_word!(
            game.enemy_drawing_queue[(v2 + 103i32)] = addr_kMaridiaFloater_Ilist_C191 as u16
        );
    }
    MaridiaFloater_Func_2(k, game);
}
pub unsafe fn MaridiaFloater_Func_10(k: u16, game: &mut Game) {
    let E = Get_MaridiaFloater(k as i32, game);
    let v2: bool = (*E).var_E == 1u16;
    (*E).var_E = (*E).var_E.wrapping_sub(1);
    let v3: bool = ((*E).var_E & 0x8000) != 0;
    if v2 || v3 {
        (*E).var_F = ((0xa8c51di32) as u16);
        (*E).var_A = g_word_A8C1C1;
        (*E).var_B = g_word_A8C1C3;
    }
    MaridiaFloater_Func_2(k, game);
}
pub unsafe fn MaridiaFloater_Func_11(k: u16, game: &mut Game) {
    let E = Get_MaridiaFloater(k as i32, game);
    AddToHiLo(
        &raw mut (*E).var_A,
        &raw mut (*E).var_B,
        (g_word_A8C1BD as u32) << 16i32 | g_word_A8C1BF as u32,
    );
    Enemy_MoveDown(
        k,
        (((*E).var_A as u32) << 16i32 | (*E).var_B as u32) as i32,
        game,
    );
    let v4: i32 = k as i32 >> 1i32;
    if (((*E).base.y_pos) as i16)
        .wrapping_sub((ram_word!(game, enemy_drawing_queue_sizes[(v4 + 1i32)])) as i16)
        >= 0i16
    {
        (*E).base.y_pos = ram_word!(game, enemy_drawing_queue_sizes[(v4 + 1i32)]);
        (*E).var_F = ((0xa8c283i32) as u16);
        (*E).var_E = 60u16;
        enemy_instruction! {[(*E).base]; timer = 1u16, instruction = addr_kMaridiaFloater_Ilist_C173 as u16}
    }
    MaridiaFloater_Func_2(k, game);
}
pub unsafe fn MaridiaFloater_Func_12(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    if ram_word!(game, enemy_drawing_queue[(v1 + 103i32)]) as i32 + 0x3e6fi32 & 0x8000i32 == 0 {
        let v2: u16 = ((ram_word!(game, enemy_drawing_queue[(v1 + 103i32)]) as i32 - 4i32
            + 15983i32) as u16
            >> 1);
        if v2 != 0 {
            set_ram_word!(
                game.enemy_drawing_queue[(v1 + 103i32)] =
                    crate::rtl::rom_table_word!(game, 0xa8c599, (v2 >> 1) as isize)
            );
            set_ram_word!(game.enemy_drawing_queue[(v1 + 104i32)] = 1u16);
        }
    }
    let E = Get_MaridiaFloater(k as i32, game);
    (*E).var_F = ((0xa8c59fi32) as u16);
    (*E).var_E = 32u16;
    MaridiaFloater_Func_2(k, game);
}
pub unsafe fn MaridiaFloater_Func_13(k: u16, game: &mut Game) {
    let E = Get_MaridiaFloater(k as i32, game);
    let v2: bool = (*E).var_E == 1u16;
    (*E).var_E = (*E).var_E.wrapping_sub(1);
    let v3: bool = ((*E).var_E & 0x8000) != 0;
    if v2 || v3 {
        let v4: i32 = k as i32 >> 1i32;
        set_ram_word!(game.enemy_drawing_queue[(v4 + 100i32)] = 0u16);
        update_ram_word!(game.enemy_drawing_queue[(v4 + 97i32)], |fresh2| (fresh2)
            | (0x200u16));
        MaridiaFloater_Func_1(k, game);
        let mut v5: u16 = 0u16;
        EnemyDeathAnimation(k, v5, game);
    } else {
        MaridiaFloater_Func_2(k, game);
    };
}
pub unsafe fn MaridiaFloater_Touch(game: &mut Game) {
    let E = Get_MaridiaFloater(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).parameter_2 == 0 {
        let palette_index: u16 = (*E).base.palette_index;
        NormalEnemyTouchAi(game);
        if (*E).base.health == 0 {
            update_ram_word!(
                game.enemy_drawing_queue
                    [((ram_word!(game, cur_enemy_index) as i32 >> 1i32) + 97i32)],
                |fresh6| (fresh6) | (0x200u16)
            );
            (*E).base.palette_index = palette_index;
            MaridiaFloater_Func_1(ram_word!(game, cur_enemy_index), game);
            (*E).base.palette_index = 2560u16;
        }
    }
}
pub unsafe fn MaridiaFloater_Shot(game: &mut Game) {
    let E = Get_MaridiaFloater(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).parameter_2 == 0 {
        NormalEnemyShotAiSkipDeathAnim_CurEnemy(game);
        if ((*E).base.ai_handler_bits & 4) != 0 {
            let v2: i32 = ram_word!(game, cur_enemy_index) as i32 >> 1i32;
            set_ram_word!(game.enemy_drawing_queue[(v2 + 109i32)] = (*E).base.frozen_timer);
            update_ram_word!(game.enemy_drawing_queue[(v2 + 99i32)], |fresh4| (fresh4)
                | (4u16));
        }
        if ((*E).base.ai_handler_bits & 2) != 0 {
            let v3: i32 = ram_word!(game, cur_enemy_index) as i32 >> 1i32;
            set_ram_word!(game.enemy_drawing_queue[(v3 + 108i32)] = (*E).base.flash_timer);
            update_ram_word!(game.enemy_drawing_queue[(v3 + 99i32)], |fresh5| (fresh5)
                | (2u16));
        }
        if (*E).base.health == 0 {
            (*E).var_F = ((0xa8c569i32) as u16);
            (*E).parameter_2 = 1u16;
        }
    }
}
pub unsafe fn MaridiaFloater_Powerbomb(game: &mut Game) {
    Enemy_NormalPowerBombAI_A8(game);
    let E = Get_MaridiaFloater(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).base.health != 0 {
        let v1: i32 = ram_word!(game, cur_enemy_index) as i32 >> 1i32;
        set_ram_word!(game.enemy_drawing_queue[(v1 + 111i32)] = (*E).base.shake_timer);
        set_ram_word!(game.enemy_drawing_queue[(v1 + 110i32)] = (*E).base.invincibility_timer);
        set_ram_word!(game.enemy_drawing_queue[(v1 + 108i32)] = (*E).base.flash_timer);
        set_ram_word!(game.enemy_drawing_queue[(v1 + 109i32)] = (*E).base.frozen_timer);
        set_ram_word!(game.enemy_drawing_queue[(v1 + 99i32)] = (*E).base.ai_handler_bits);
    } else {
        update_ram_word!(
            game.enemy_drawing_queue[((ram_word!(game, cur_enemy_index) as i32 >> 1i32) + 97i32)],
            |fresh3| (fresh3) | (0x200u16)
        );
    };
}
pub unsafe fn WreckedShipRobot_Init(game: &mut Game) {
    if (*(ram_ptr!(game, u16; 0xd828, ram_word!(game, area_index) as isize)) & 1) != 0 {
        (*(ram_ptr!(game, LongPtr; 0x178c))).bank = -88i32 as u8;
        (*(ram_ptr!(game, LongPtr; 0x178c))).addr = ((0xa8cc67i32) as u16);
        let E = Get_WreckedShipRobot(ram_word!(game, cur_enemy_index) as i32, game);
        (*E).base.properties = ((*E).base.properties) | (0xa000u16);
        (*E).base.instruction_timer = 4u16;
        (*E).base.timer = 0u16;
        (*E).base.current_instruction = addr_kWreckedShipRobot_Ilist_C6E5 as u16;
        (*E).var_A = -512i32 as u16;
        set_ram_word!(game.wrecked_ship_robot_palanim_timer = 1u16);
        *(ram_ptr!(game, u16; 0xd656)) = 0u16;
        (*E).var_B = 0u16;
        set_ram_word!(game.wrecked_ship_robot_palanim_palindex = (*E).base.palette_index);
    } else {
        WreckedShipRobotDeactivated_Init(game);
    };
}
pub unsafe fn WreckedShipRobotDeactivated_Init(game: &mut Game) {
    let v0 = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    let mut parameter_1: u16 = (*v0).parameter_1;
    if parameter_1 & 0x8000 != 0 || parameter_1 as i32 - 4i32 & 0x8000i32 == 0 {
        parameter_1 = 0u16;
    }
    (*v0).parameter_1 = parameter_1;
    (*v0).current_instruction = crate::rtl::rom_table_word!(game, 0xa8cc30, parameter_1 as isize);
    (*v0).properties = ((*v0).properties) | (0x8000u16);
    (*v0).instruction_timer = 1u16;
    (*v0).timer = 0u16;
    set_ram_word!(game.wrecked_ship_robot_palanim_timer = 0u16);
    (*v0).ai_var_E = 0u16;
    (*v0).ai_preinstr = 1u16;
    let v2: i32 = 16i32
        * ((ram_word!(game, wrecked_ship_robot_palanim_palindex) as i32 & 0xff00i32) >> 8i32)
        >> 1i32;
    set_ram_words! { game, 10u16;
        palette_buffer[(v2 + 137i32)], palette_buffer[(v2 + 138i32)], palette_buffer[(v2 + 139i32)],
        palette_buffer[(v2 + 140i32)],
    }
}
pub unsafe fn WreckedShipRobot_Main(game: &mut Game) {
    let E = Get_WreckedShipRobot(ram_word!(game, cur_enemy_index) as i32, game);
    if Enemy_MoveDown(
        ram_word!(game, cur_enemy_index),
        (((*E).var_F as u32) << 16i32 | (*E).var_E as u32) as i32,
        game,
    ) == 0
    {
        (*E).base.instruction_timer = (*E).base.instruction_timer.wrapping_add(1);
        AddToHiLo(&raw mut (*E).var_F, &raw mut (*E).var_E, 0x8000u32);
    }
}
pub unsafe fn WreckedShipRobot_Func_1(game: &mut Game) {
    let mut v2: i16 = 0;
    if ram_word!(game, door_transition_flag_enemies) == 0
        && ram_word!(game, wrecked_ship_robot_palanim_timer) != 0
    {
        if update_ram_word!(
            game.wrecked_ship_robot_palanim_timer,
            wrapping_sub(1),
            reread
        ) == 0
        {
            let mut v1: i32 = 0;
            let mut i: u16 = *(ram_ptr!(game, u16; 0xd656));
            loop {
                v1 = i as i32 >> 1i32;
                v2 = g_word_A8CCC1[v1 as usize];
                if v2 >= 0i16 {
                    break;
                }
                i = 0u16;
            }
            let v3: i32 = 16i32
                * ((ram_word!(game, wrecked_ship_robot_palanim_palindex) as i32 & 0xff00i32)
                    >> 8i32)
                >> 1i32;
            set_ram_word!(game.palette_buffer[(v3 + 137i32)] = v2 as u16);
            set_ram_word!(
                game.palette_buffer[(v3 + 138i32)] = g_word_A8CCC1[(v1 + 1i32) as usize] as u16
            );
            set_ram_word!(
                game.palette_buffer[(v3 + 139i32)] = g_word_A8CCC1[(v1 + 2i32) as usize] as u16
            );
            set_ram_word!(
                game.palette_buffer[(v3 + 140i32)] = g_word_A8CCC1[(v1 + 3i32) as usize] as u16
            );
            set_ram_word!(
                game.wrecked_ship_robot_palanim_timer = g_word_A8CCC1[(v1 + 4i32) as usize] as u16
            );
            *(ram_ptr!(game, u16; 0xd656)) = (i).wrapping_add(10u16);
        }
    }
}
pub unsafe fn WreckedShipRobot_Func_2(k: u16, game: &mut Game) {
    let E = Get_WreckedShipRobot(k as i32, game);
    let wsrt_var_B: u16 = (*E).var_B;
    if wsrt_var_B != 0 {
        (*E).var_B = (wsrt_var_B).wrapping_sub(1u16);
    }
}
// Movement callbacks can change the live enemy index. Keep the captured
// robot for its state/coordinate stores and reread the index before each call.
unsafe fn move_wrecked_ship_robot(
    jp: *const u16,
    motion: (u16, i16),
    instructions: (u16, Option<u16>),
    restore_y_first: bool,
    game: &mut Game,
) -> *const u16 {
    let (facing, step) = motion;
    let (wall_instruction, edge_instruction) = instructions;
    WreckedShipRobot_Func_2(ram_word!(game, cur_enemy_index), game);
    let robot = Get_WreckedShipRobot(ram_word!(game, cur_enemy_index) as i32, game);
    (*robot).var_A = facing;
    if Enemy_MoveRight_IgnoreSlopes(ram_word!(game, cur_enemy_index), (step as i32) << 16, game)
        != 0
    {
        (*robot).var_B = (*robot).var_B.wrapping_add(8);
        return wall_instruction as usize as *const u16;
    }
    if CheckIfEnemyTouchesSamus(ram_word!(game, cur_enemy_index), game) != 0 {
        set_ram_word!(game.extra_samus_x_subdisplacement = 0);
        set_ram_word!(game.extra_samus_x_displacement = step as u16);
    }
    let Some(edge_instruction) = edge_instruction else {
        return jp;
    };
    (*robot).var_D = (*robot).base.y_pos;
    let x = (*robot).base.x_pos;
    (*robot).var_C = x;
    (*robot).base.x_pos = if step < 0 {
        (x as i32 - (*robot).base.x_width as i32 - (*robot).base.x_width as i32) as u16
    } else {
        ((*robot).base.x_width as i32 + (*robot).base.x_width as i32 + x as i32) as u16
    };
    let ground = Enemy_MoveDown(ram_word!(game, cur_enemy_index), 1 << 16, game) != 0;
    if !ground {
        (*robot).var_B = (*robot).var_B.wrapping_add(8);
    }
    if restore_y_first {
        (*robot).base.y_pos = (*robot).var_D;
        (*robot).base.x_pos = (*robot).var_C;
    } else {
        (*robot).base.x_pos = (*robot).var_C;
        (*robot).base.y_pos = (*robot).var_D;
    }
    if !ground {
        (*robot).var_A = if step < 0 { 512 } else { 0xfe00 };
        return edge_instruction as usize as *const u16;
    }
    jp
}

crate::forward_calls! { pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16;
    WreckedShipRobot_Instr_4 => move_wrecked_ship_robot(
        jp,
        (0xfe00, -4),
        (
            addr_kWreckedShipRobot_Ilist_C73F as u16,
            Some(addr_kWreckedShipRobot_Ilist_CB65 as u16),
        ),
        false,
        game,
    );
    WreckedShipRobot_Instr_9 => move_wrecked_ship_robot(
        jp,
        (0xfe00, -4),
        (addr_kWreckedShipRobot_Ilist_C73F as u16, None),
        false,
        game,
    );
    WreckedShipRobot_Instr_6 => move_wrecked_ship_robot(
        jp,
        (0xfe00, 4),
        (
            addr_stru_A8C6E9 as u16,
            Some(addr_kWreckedShipRobot_Ilist_C91B as u16),
        ),
        false,
        game,
    );
    WreckedShipRobot_Instr_8 => move_wrecked_ship_robot(
        jp,
        (0xfe00, 4),
        (addr_stru_A8C6E9 as u16, None),
        false,
        game,
    );
}
pub unsafe fn WreckedShipRobot_Instr_7(k: u16, jp: *const u16) -> *const u16 {
    addr_kWreckedShipRobot_Ilist_C92D as i32 as usize as *const u16
}
crate::forward_calls! { pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16;
    WreckedShipRobot_Instr_15 => move_wrecked_ship_robot(
        jp,
        (512, 4),
        (
            addr_kWreckedShipRobot_Ilist_C985 as u16,
            Some(addr_kWreckedShipRobot_Ilist_C91B as u16),
        ),
        false,
        game,
    );
    WreckedShipRobot_Instr_18 => move_wrecked_ship_robot(
        jp,
        (512, 4),
        (addr_kWreckedShipRobot_Ilist_C985 as u16, None),
        false,
        game,
    );
    WreckedShipRobot_Instr_16 => move_wrecked_ship_robot(
        jp,
        (512, -4),
        (
            addr_stru_A8C6E9 as u16,
            Some(addr_kWreckedShipRobot_Ilist_CB65 as u16),
        ),
        true,
        game,
    );
    WreckedShipRobot_Instr_17 =>
        move_wrecked_ship_robot(jp, (512, -4), (addr_stru_A8C6E9 as u16, None), false, game);
}
pub unsafe fn WreckedShipRobot_Instr_3(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E = Get_WreckedShipRobot(ram_word!(game, cur_enemy_index) as i32, game);
    if ((ram_word!(game, layer1_x_pos)) as i16).wrapping_sub(((*E).base.x_pos) as i16) < 0i16
        && ((ram_word!(game, layer1_x_pos) as i32 + 256i32 - (*E).base.x_pos as i32) as i16) >= 0i16
        && ((ram_word!(game, layer1_y_pos)) as i16).wrapping_sub(((*E).base.y_pos) as i16) < 0i16
        && ((ram_word!(game, layer1_y_pos) as i32 + 224i32 - (*E).base.y_pos as i32) as i16) >= 0i16
    {
        QueueSfx2_Max6(0x68u16, game);
    }
    jp
}
pub unsafe fn WreckedShipRobot_Instr_10(k: u16, jp: *const u16) -> *const u16 {
    addr_stru_A8C6E9 as i32 as usize as *const u16
}
crate::forward_calls! { pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16;
    WreckedShipRobot_Instr_14 => WreckedShipRobot_CommonInstr(
        k,
        jp,
        addr_kWreckedShipRobot_Ilist_CB1D as u16,
        addr_kEproj_WreckedShipRobotLaserUpRight as u16,
        game,
    );
    WreckedShipRobot_Instr_2 => WreckedShipRobot_CommonInstr(
        k,
        jp,
        addr_kWreckedShipRobot_Ilist_C8D1 as u16,
        addr_kEproj_WreckedShipRobotLaserUpLeft as u16,
        game,
    );
}
pub unsafe fn WreckedShipRobot_CommonInstr(
    k: u16,
    jp: *const u16,
    r50: u16,
    r48: u16,
    game: &mut Game,
) -> *const u16 {
    let E = Get_WreckedShipRobot(k as i32, game);
    if (*E).var_B != 0 {
        WreckedShipRobot_Func_2(k, game);
    } else {
        let v3: u16 = ((ram_word!(game, random_number) as i32 & 0x1fi32) + 16i32) as u16;
        (*E).var_B = v3;
        SpawnEprojWithGfx(v3, ram_word!(game, cur_enemy_index), r48, game);
        return r50 as usize as *const u16;
    }
    jp
}
crate::forward_calls! { pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16;
    WreckedShipRobot_Instr_13 =>
        WreckedShipRobot_D10C(k, jp, addr_kWreckedShipRobot_Ilist_CB09 as u16, game);
    WreckedShipRobot_Instr_1 =>
        WreckedShipRobot_D10C(k, jp, addr_kWreckedShipRobot_Ilist_C8BD as u16, game);
}
pub unsafe fn WreckedShipRobot_D10C(
    k: u16,
    jp: *const u16,
    r50: u16,
    game: &mut Game,
) -> *const u16 {
    let E = Get_WreckedShipRobot(k as i32, game);
    if (*E).var_B != 0 {
        WreckedShipRobot_Func_2(k, game);
    } else {
        let v3: u16 = ((ram_word!(game, random_number) as i32 & 0x1fi32) + 16i32) as u16;
        (*E).var_B = v3;
        SpawnEprojWithGfx(
            v3,
            ram_word!(game, cur_enemy_index),
            addr_kEproj_WreckedShipRobotLaserHorizontal as u16,
            game,
        );
        return r50 as usize as *const u16;
    }
    jp
}
crate::forward_calls! { pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16;
    WreckedShipRobot_Instr_12 => WreckedShipRobot_D147(
        k,
        jp,
        addr_kEproj_WreckedShipRobotLaserDownRight as u16,
        addr_kWreckedShipRobot_Ilist_CAFD as u16,
        game,
    );
    WreckedShipRobot_Instr_5 => WreckedShipRobot_D147(
        k,
        jp,
        addr_kEproj_WreckedShipRobotLaserDownLeft as u16,
        addr_kWreckedShipRobot_Ilist_C8B1 as u16,
        game,
    );
}
pub unsafe fn WreckedShipRobot_D147(
    k: u16,
    jp: *const u16,
    r48: u16,
    r50: u16,
    game: &mut Game,
) -> *const u16 {
    let E = Get_WreckedShipRobot(k as i32, game);
    if (*E).var_B != 0 {
        WreckedShipRobot_Func_2(k, game);
    } else {
        let v3: u16 = ((ram_word!(game, random_number) as i32 & 0x1fi32) + 16i32) as u16;
        (*E).var_B = v3;
        SpawnEprojWithGfx(v3, ram_word!(game, cur_enemy_index), r48, game);
        return r50 as usize as *const u16;
    }
    jp
}
pub unsafe fn WreckedShipRobot_Instr_11(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    WreckedShipRobot_Func_2(ram_word!(game, cur_enemy_index), game);
    jp
}
pub unsafe fn WreckedShipRobotDeactivated_Touch(game: &mut Game) {
    if (((*gEnemyData(ram_word!(game, cur_enemy_index) as i32, game)).x_pos) as i16)
        .wrapping_sub((ram_word!(game, samus_x_pos)) as i16)
        < 0i16
    {
        set_ram_word!(game.extra_samus_x_displacement = 4u16);
    } else {
        set_ram_word!(game.extra_samus_x_displacement = -4i32 as u16);
    };
}
pub unsafe fn WreckedShipRobotDeactivated_Shot(game: &mut Game) {
    NormalEnemyShotAi(game);
}
pub unsafe fn WreckedShipRobot_Shot(game: &mut Game) {
    if (*(ram_ptr!(game, u16; 0xd828, ram_word!(game, area_index) as isize)) & 1) != 0 {
        NormalEnemyShotAi(game);
        let E = Get_WreckedShipRobot(ram_word!(game, cur_enemy_index) as i32, game);
        if (*E).base.health != 0 {
            let mut v1: u16 = 0;
            if ((*E).var_A & 0x8000) != 0 {
                if ((ram_word!(game, samus_x_pos)) as i16).wrapping_sub(((*E).base.x_pos) as i16)
                    >= 0i16
                {
                    v1 = addr_kWreckedShipRobot_Ilist_C833 as u16;
                } else {
                    v1 = addr_kWreckedShipRobot_Ilist_C7BB as u16;
                }
            } else if ((ram_word!(game, samus_x_pos)) as i16).wrapping_sub(((*E).base.x_pos) as i16)
                < 0i16
            {
                v1 = addr_kWreckedShipRobot_Ilist_CA7D as u16;
            } else {
                v1 = addr_kWreckedShipRobot_Ilist_CA01 as u16;
            }
            enemy_instruction! {[(*E).base]; instruction = v1, timer = 1u16}
            (*E).var_B = ((*E).var_B).wrapping_add(64u16);
        }
    }
}
pub unsafe fn MaridiaPuffer_Init(game: &mut Game) {
    let E = Get_MaridiaPuffer(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.instruction_timer = 1u16;
    (*E).base.timer = 0u16;
    (*E).base.current_instruction = addr_kMaridiaPuffer_Ilist_D841 as u16;
    let v1: i32 = ((4u16).wrapping_mul((*E).parameter_1)) as i32 >> 1i32;
    let v2: u16 = g_word_A8D895[v1 as usize] as u16;
    (*E).var_05 = v2;
    (*E).var_F = v2;
    (*E).var_06 = g_word_A8D895[(v1 + 1i32) as usize] as u16;
    (*E).var_E = 16u16;
    (*E).var_A = ((0xa8d92bi32) as u16);
    (*E).var_00 = g_word_A8D885[(*E).parameter_2 as usize] as u16;
}
crate::game::address_dispatch! {
    pub unsafe fn CallMaridiaPuffer(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        11065643 => MaridiaPuffer_Func_1; 11065664 => MaridiaPuffer_Func_2;
        11065699 => MaridiaPuffer_Func_3; 11065724 => MaridiaPuffer_Func_4;
    }
}
pub unsafe fn MaridiaPuffer_Main(game: &mut Game) {
    let E = Get_MaridiaPuffer(ram_word!(game, cur_enemy_index) as i32, game);
    let v1: u16 = ((*E).var_07).wrapping_sub(1u16);
    (*E).var_07 = v1;
    if v1 == 0 {
        (*E).var_07 = 1u16;
        (*E).var_03 = 0u16;
    }
    CallMaridiaPuffer((((*E).var_A) as u32) | (0xa80000u32), game);
}
pub unsafe fn MaridiaPuffer_Func_1(game: &mut Game) {
    let E = Get_MaridiaPuffer(ram_word!(game, cur_enemy_index) as i32, game);
    crate::update_place! { previous fresh7 = (*E).var_E => (*E).var_E.wrapping_sub(1); }
    if fresh7 == 1u16 {
        (*E).var_E = 16u16;
        (*E).var_A = ((0xa8d940i32) as u16);
    }
}
pub unsafe fn MaridiaPuffer_Func_2(game: &mut Game) {
    let v1: u8 = ((CalculateAngleOfSamusFromEnemy(ram_word!(game, cur_enemy_index), game)) as u8)
        .wrapping_sub(64u8);
    let E = Get_MaridiaPuffer(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).var_01 = v1 as u16;
    (*E).var_02 = v1 as u16;
    (*E).var_A = ((0xa8d963i32) as u16);
    (*E).var_C = 24u16;
}
pub unsafe fn MaridiaPuffer_Func_3(game: &mut Game) {
    let E = Get_MaridiaPuffer(ram_word!(game, cur_enemy_index) as i32, game);
    if (((*E).var_D) as i16).wrapping_sub(((*E).var_00) as i16) < 0i16 {
        MaridiaPuffer_Func_8(ram_word!(game, cur_enemy_index), game);
    }
    MaridiaPuffer_Func_6(game);
    MaridiaPuffer_Func_7(game);
    MaridiaPuffer_Func_5(game);
}
pub unsafe fn MaridiaPuffer_Func_4(game: &mut Game) {
    let E = Get_MaridiaPuffer(ram_word!(game, cur_enemy_index) as i32, game);
    if ((*E).var_D as i16) > 0i16 && ((*E).var_B as i16) > 0i16 {
        MaridiaPuffer_Func_6(game);
        MaridiaPuffer_Func_7(game);
        MaridiaPuffer_Func_9(ram_word!(game, cur_enemy_index), game);
    } else {
        (*E).var_A = ((0xa8d92bi32) as u16);
        set_fields! {[(*E)], 0u16; var_B, var_C, var_D}
    };
}
pub unsafe fn MaridiaPuffer_Func_5(game: &mut Game) {
    let v1: u8 = ((CalculateAngleOfSamusFromEnemy(ram_word!(game, cur_enemy_index), game)) as u8)
        .wrapping_sub(64u8);
    let E = Get_MaridiaPuffer(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).var_01 = v1 as u16;
    let v3: u16 = SignExtend8(((v1) as u16).wrapping_sub((*E).var_02));
    let v4: u16 = Abs16(v3);
    if v4 as i32 - 48i32 & 0x8000i32 == 0 {
        (*E).var_A = ((0xa8d97ci32) as u16);
        (*E).var_C = 24u16;
    }
}
pub unsafe fn MaridiaPuffer_Func_6(game: &mut Game) {
    let E = Get_MaridiaPuffer(ram_word!(game, cur_enemy_index) as i32, game);
    let mut R26: u16 = 0u16;
    let r18: u16 = crate::rtl::rom_table_word!(
        game,
        0xa0b1c3,
        ((((*E).var_02) as u8).wrapping_add(64u8)) as isize
    );
    if (r18 & 0x8000) != 0 {
        R26 = R26.wrapping_add(1);
    }
    let mut t: u32 = (((Abs16(r18) as i32 & 0xff00i32) >> 8i32) * (*E).var_D as i32) as u32;
    if R26 != 0 {
        t = -(t as i32) as u32;
    }
    AddToHiLo(&raw mut (*E).base.x_pos, &raw mut (*E).base.x_subpos, t);
}
pub unsafe fn MaridiaPuffer_Func_7(game: &mut Game) {
    let E = Get_MaridiaPuffer(ram_word!(game, cur_enemy_index) as i32, game);
    let mut R26: u16 = 0u16;
    let r18: u16 = crate::rtl::rom_table_word!(game, 0xa0b1c3, (*E).var_02 as u8 as isize);
    if (r18 & 0x8000) != 0 {
        R26 = R26.wrapping_add(1);
    }
    let mut t: u32 = (((Abs16(r18) as i32 & 0xff00i32) >> 8i32) * (*E).var_D as i32) as u32;
    if R26 != 0 {
        t = -(t as i32) as u32;
    }
    AddToHiLo(&raw mut (*E).base.y_pos, &raw mut (*E).base.y_subpos, t);
}
pub unsafe fn MaridiaPuffer_Func_8(k: u16, game: &mut Game) {
    let E = Get_MaridiaPuffer(k as i32, game);
    crate::update_place! { previous fresh8 = (*E).var_F => (*E).var_F.wrapping_sub(1); }
    if fresh8 == 1u16 {
        (*E).var_F = (*E).var_05;
        (*E).var_B = ((*E).var_B).wrapping_add((*E).var_C);
        (*E).var_D = ((*E).var_D).wrapping_add((*E).var_B);
    }
}
pub unsafe fn MaridiaPuffer_Func_9(k: u16, game: &mut Game) {
    let E = Get_MaridiaPuffer(k as i32, game);
    crate::update_place! { previous fresh9 = (*E).var_F => (*E).var_F.wrapping_sub(1); }
    if fresh9 == 1u16 {
        (*E).var_F = (*E).var_06;
        (*E).var_B = ((*E).var_B).wrapping_sub((*E).var_C);
        (*E).var_D = ((*E).var_D).wrapping_sub((*E).var_B);
    }
}
pub unsafe fn MaridiaPuffer_Shot(game: &mut Game) {
    let E = Get_MaridiaPuffer(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).var_40 = (*E).base.health;
    Enemy_NormalShotAI_A8(game);
    if (*E).base.health == (*E).var_40 && (*E).var_03 == 0 {
        (*E).base.instruction_timer = 1u16;
        (*E).base.timer = 0u16;
        (*E).base.current_instruction = addr_kMaridiaPuffer_Ilist_D855 as u16;
        (*E).var_02 = g_word_A8D871[(ram_word!(
            game,
            projectile_dir[ram_word!(game, collision_detection_index)]
        ) as i32
            & 0xfi32) as usize] as u16;
        (*E).var_B = 256u16;
        (*E).var_D = 1536u16;
        (*E).var_A = ((0xa8d97ci32) as u16);
        (*E).var_07 = 48u16;
        (*E).var_03 = 1u16;
    }
}
pub unsafe fn WalkingLavaSeahorse_Init(game: &mut Game) {
    let E = Get_WalkingLavaSeahorse(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).var_04 = 0u16;
    (*E).var_F = (*E).base.y_pos;
    (*E).var_02 = (*E).base.x_pos;
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_DisableSamusColl) as u16);
    (*E).base.current_instruction = addr_kWalkingLavaSeahorse_Ilist_DBE7 as u16;
    (*E).var_A = ((0xa8dd71i32) as u16);
    WalkingLavaSeahorse_Func_1(ram_word!(game, cur_enemy_index), game);
    loop {
        AddToHiLo(
            &raw mut (*E).base.y_pos,
            &raw mut (*E).base.y_subpos,
            ((*E).var_B as u32) << 16i32 | (*E).var_C as u32,
        );
        if ((WalkingLavaSeahorse_Func_2(ram_word!(game, cur_enemy_index), game) & 0x8000) == 0) {
            break;
        }
    }
    while (Enemy_MoveDown(
        ram_word!(game, cur_enemy_index),
        (((*E).var_B as u32) << 16i32 | (*E).var_C as u32) as i32,
        game,
    ) == 0)
    {
        WalkingLavaSeahorse_Func_2(ram_word!(game, cur_enemy_index), game);
    }
    (*E).var_03 = (*E).base.y_pos;
    (*E).base.y_pos = (*E).var_F;
}
pub unsafe fn WalkingLavaSeahorse_Func_1(k: u16, game: &mut Game) {
    let E = Get_WalkingLavaSeahorse(k as i32, game);
    (*E).var_B = -12i32 as u16;
    set_fields! {[(*E)], 0u16; var_C, var_D, var_E, var_00}
    (*E).var_01 = 0x8000u16;
}
pub unsafe fn WalkingLavaSeahorse_Func_2(k: u16, game: &mut Game) -> u16 {
    let E = Get_WalkingLavaSeahorse(k as i32, game);
    AddToHiLo(
        &raw mut (*E).var_B,
        &raw mut (*E).var_C,
        ((*E).var_00 as u32) << 16i32 | (*E).var_01 as u32,
    );
    (*E).var_B
}
crate::game::address_dispatch! {
    pub unsafe fn CallWalkingLavaSeahorseFunc(ea: u32, k: u16, game: &mut Game) => ea;
        call (k, game); default { Unreachable(game); };
    {
        11067084 => [()]; 11066737 => WalkingLavaSeahorse_Func_3;
        11066822 => WalkingLavaSeahorse_Func_4; 11066885 => WalkingLavaSeahorse_Func_6;
        11066955 => WalkingLavaSeahorse_Func_7; 11067085 => WalkingLavaSeahorse_Func_8;
        11067116 => WalkingLavaSeahorse_Func_9;
    }
}
pub unsafe fn WalkingLavaSeahorse_Main(game: &mut Game) {
    let E = Get_WalkingLavaSeahorse(ram_word!(game, cur_enemy_index) as i32, game);
    CallWalkingLavaSeahorseFunc(
        (((*E).var_A) as u32) | (0xa80000u32),
        ram_word!(game, cur_enemy_index),
        game,
    );
}
pub unsafe fn WalkingLavaSeahorse_Func_3(k: u16, game: &mut Game) {
    let E = Get_WalkingLavaSeahorse(k as i32, game);
    let mut xxx: u16 = 0;
    let v3: u16 = abs16(((*E).var_03).wrapping_sub(ram_word!(game, samus_y_pos)));
    if v3 as i32 - 32i32 & 0x8000i32 != 0 && {
        xxx = (ram_word!(game, samus_x_pos)).wrapping_sub((*E).base.x_pos);
        ((abs16(xxx)) as i16).wrapping_sub((g_word_A8DCC7) as i16) < 0i16
    } {
        WalkingLavaSeahorse_Func_1(k, game);
        let mut v4: u16 = addr_stru_A8DC4B as u16;
        let mut v5: i16 = -2i16;
        if xxx & 0x8000 == 0 {
            v4 = addr_stru_A8DCBB as u16;
            v5 = 2i16;
        }
        (*E).var_D = v5 as u16;
        enemy_instruction! {[(*E).base]; instruction = v4, timer = 1u16}
        (*E).var_A = ((0xa8ddc6i32) as u16);
        QueueSfx2_Max6(0x5eu16, game);
    }
}
pub unsafe fn WalkingLavaSeahorse_Func_4(k: u16, game: &mut Game) {
    let E = Get_WalkingLavaSeahorse(k as i32, game);
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        ((*E).var_B as u32) << 16i32 | (*E).var_C as u32,
    );
    if (WalkingLavaSeahorse_Func_2(k, game) & 0x8000) == 0 {
        sub_A8DDDE(k, game);
    }
}
pub unsafe fn sub_A8DDDE(k: u16, game: &mut Game) {
    let v1 = gEnemyData(k as i32, game);
    (*v1).ai_var_A = ((0xa8de05i32) as u16);
    let mut v2: u16 = addr_kWalkingLavaSeahorse_Ilist_DC51 as u16;
    if ((*v1).ai_var_D & 0x8000) == 0 {
        v2 = addr_kWalkingLavaSeahorse_Ilist_DCC1 as u16;
    }
    enemy_instruction! {[(*v1)]; instruction = v2, timer = 1u16}
}
pub unsafe fn WalkingLavaSeahorse_Func_6(k: u16, game: &mut Game) {
    let E = Get_WalkingLavaSeahorse(k as i32, game);
    if Enemy_MoveDown(
        k,
        (((*E).var_B as u32) << 16i32 | (*E).var_C as u32) as i32,
        game,
    ) != 0
    {
        let t: i32 = ram_word!(game, samus_x_pos) as i32 - (*E).base.x_pos as i32;
        let mut v2: u16 = addr_kWalkingLavaSeahorse_Ilist_DBE7 as u16;
        let mut v3: i16 = -2i16;
        if t & 0x8000i32 == 0 {
            v3 = 2i16;
            v2 = addr_kWalkingLavaSeahorse_Ilist_DC57 as u16;
        }
        (*E).var_D = v3 as u16;
        enemy_instruction! {[(*E).base]; instruction = v2, timer = 1u16}
        (*E).var_A = ((0xa8de4bi32) as u16);
        (*E).var_04 = 1u16;
    } else {
        WalkingLavaSeahorse_Func_2(k, game);
    };
}
pub unsafe fn WalkingLavaSeahorse_Func_7(k: u16, game: &mut Game) {
    Enemy_MoveDown(k, (2i16 as i32) << 16i32, game);
    let E = Get_WalkingLavaSeahorse(k as i32, game);
    if ((abs16(((*E).var_02).wrapping_sub((*E).base.x_pos))) as i16)
        .wrapping_sub((g_word_A8DCCB) as i16)
        >= 0i16
    {
        (*E).var_A = ((0xa8decdi32) as u16);
        (*E).var_B = -4i32 as u16;
        (*E).var_C = 0u16;
        let mut v4: u16 = addr_kWalkingLavaSeahorse_Ilist_DC51 as u16;
        if ((*E).var_D & 0x8000) == 0 {
            v4 = addr_kWalkingLavaSeahorse_Ilist_DCC1 as u16;
        }
        enemy_instruction! {[(*E).base]; instruction = v4, timer = 1u16}
    } else {
        if (*E).var_04 != 0 {
            return;
        }
        let t: i32 = ram_word!(game, samus_x_pos) as i32 - (*E).base.x_pos as i32;
        let mut v3: u16 = 0;
        if t & 0x8000i32 == 0 {
            v3 = addr_kWalkingLavaSeahorse_Ilist_DC73 as u16;
            if ((*E).var_D & 0x8000) != 0 {
                return;
            }
        } else {
            v3 = addr_kWalkingLavaSeahorse_Ilist_DC03 as u16;
            if ((*E).var_D & 0x8000) == 0 {
                return;
            }
        }
        enemy_instruction! {[(*E).base]; instruction = v3, timer = 1u16}
        (*E).var_A = ((0xa8decci32) as u16);
    };
}
pub unsafe fn WalkingLavaSeahorse_Func_8(k: u16, game: &mut Game) {
    let E = Get_WalkingLavaSeahorse(k as i32, game);
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        ((*E).var_B as u32) << 16i32 | (*E).var_C as u32,
    );
    if (WalkingLavaSeahorse_Func_2(k, game) & 0x8000) == 0 {
        (*E).var_A = ((0xa8deeci32) as u16);
    }
}
pub unsafe fn WalkingLavaSeahorse_Func_9(k: u16, game: &mut Game) {
    let E = Get_WalkingLavaSeahorse(k as i32, game);
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        ((*E).var_B as u32) << 16i32 | (*E).var_C as u32,
    );
    if (((*E).base.y_pos) as i16).wrapping_sub(((*E).var_F) as i16) >= 0i16 {
        (*E).base.y_pos = (*E).var_F;
        (*E).base.x_pos = (*E).var_02;
        (*E).var_A = ((0xa8dd71i32) as u16);
    } else {
        WalkingLavaSeahorse_Func_2(k, game);
    };
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16;
    WalkingLavaSeahorse_Instr_4 [jp] { WalkingLavaSeahorse_DF20(0u16, game); };
}
pub unsafe fn WalkingLavaSeahorse_DF20(a: u16, game: &mut Game) {
    SpawnEprojWithGfx(
        a,
        ram_word!(game, cur_enemy_index),
        addr_loc_A89E90 as u16,
        game,
    );
    QueueSfx2_Max6(0x3fu16, game);
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16;
    WalkingLavaSeahorse_Instr_3 [jp] { WalkingLavaSeahorse_DF20(2u16, game); };
    WalkingLavaSeahorse_Instr_5 [jp] { WalkingLavaSeahorse_DF20(4u16, game); };
    WalkingLavaSeahorse_Instr_6 [addr_kWalkingLavaSeahorse_Ilist_DBE7 as i32 as usize as *const u16] {
        let E = Get_WalkingLavaSeahorse(ram_word!(game, cur_enemy_index) as i32, game);
        (*E).var_A = ((0xa8de4bi32) as u16);
        let mut v3: i16 = ((ram_word!(game, random_number)) as i16) & (3i16);
        if (ram_word!(game, random_number) & 3) == 0 {
            v3 = 2i16;
        }
        (*E).var_04 = v3 as u16;
        if ((*E).var_D & 0x8000) == 0 {
            return addr_kWalkingLavaSeahorse_Ilist_DC57 as i32 as usize as *const u16;
        }
    };
    WalkingLavaSeahorse_Instr_2 [WalkingLavaSeahorse_Instr_1(ram_word!(game, cur_enemy_index), jp, game)] {
        let E = Get_WalkingLavaSeahorse(ram_word!(game, cur_enemy_index) as i32, game);
        let wlse_var_04: u16 = (*E).var_04;
        if wlse_var_04 != 0 {
            (*E).var_04 = (wlse_var_04).wrapping_sub(1u16);
        }
    };
}
pub unsafe fn WalkingLavaSeahorse_Instr_1(
    k: u16,
    mut jp: *const u16,
    game: &mut Game,
) -> *const u16 {
    let E = Get_WalkingLavaSeahorse(ram_word!(game, cur_enemy_index) as i32, game);
    if Enemy_MoveRight_IgnoreSlopes(
        ram_word!(game, cur_enemy_index),
        ((*E).var_D as i16 as i32) << 16i32,
        game,
    ) != 0
    {
        jp = addr_stru_A8DBE9 as i32 as usize as *const u16;
        let mut wlse_var_D: i16 = (*E).var_D as i16;
        if wlse_var_D < 0i16 {
            jp = addr_stru_A8DC59 as i32 as usize as *const u16;
        }
        (*E).var_D = -(wlse_var_D as i32) as u16;
    } else {
        EnemyFunc_C8AD(ram_word!(game, cur_enemy_index), game);
    }
    jp
}
pub unsafe fn WreckedShipOrbs_Init(game: &mut Game) {
    let E = Get_WreckedShipOrbs(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.instruction_timer = 1u16;
    (*E).base.timer = 0u16;
    (*E).base.current_instruction =
        crate::rtl::rom_table_word!(game, 0xa8e380, (*E).parameter_1 as isize);
    let v1: i32 = 8i32 * (*E).parameter_2 as i32 >> 1i32;
    (*E).var_01 = crate::rtl::rom_table_word!(game, 0xa28187, v1 as isize);
    (*E).var_00 = crate::rtl::rom_table_word!(game, 0xa28187, (v1 + 1i32) as isize);
    (*E).var_03 = crate::rtl::rom_table_word!(game, 0xa28187, (v1 + 2i32) as isize);
    (*E).var_02 = crate::rtl::rom_table_word!(game, 0xa28187, (v1 + 3i32) as isize);
}
crate::game::address_dispatch! {
    pub unsafe fn CallWreckedShipOrbsA(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        11068421 => WreckedShipOrbs_Func_3; 11068452 => WreckedShipOrbs_Func_4;
    }
    pub unsafe fn CallWreckedShipOrbsB(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        11068483 => WreckedShipOrbs_Func_5; 11068514 => WreckedShipOrbs_Func_6;
    }
}
pub unsafe fn WreckedShipOrbs_Main(game: &mut Game) {
    WreckedShipOrbs_Func_1(game);
    WreckedShipOrbs_Func_2(game);
    let E = Get_WreckedShipOrbs(ram_word!(game, cur_enemy_index) as i32, game);
    CallWreckedShipOrbsA((((*E).var_A) as u32) | (0xa80000u32), game);
    CallWreckedShipOrbsB((((*E).var_B) as u32) | (0xa80000u32), game);
}
pub unsafe fn WreckedShipOrbs_Func_1(game: &mut Game) {
    let E = Get_WreckedShipOrbs(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).var_A = ((0xa8e405i32) as u16);
    if (GetSamusEnemyDelta_Y(ram_word!(game, cur_enemy_index), game) & 0x8000) == 0 {
        (*E).var_A = ((0xa8e424i32) as u16);
    }
}
pub unsafe fn WreckedShipOrbs_Func_2(game: &mut Game) {
    let E = Get_WreckedShipOrbs(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).var_B = ((0xa8e443i32) as u16);
    if (GetSamusEnemyDelta_X(ram_word!(game, cur_enemy_index), game) & 0x8000) == 0 {
        (*E).var_B = ((0xa8e462i32) as u16);
    }
}
pub unsafe fn WreckedShipOrbs_Func_3(game: &mut Game) {
    let E = Get_WreckedShipOrbs(ram_word!(game, cur_enemy_index) as i32, game);
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        ((*E).var_03 as u32) << 16i32 | (*E).var_02 as u32,
    );
}
pub unsafe fn WreckedShipOrbs_Func_4(game: &mut Game) {
    let E = Get_WreckedShipOrbs(ram_word!(game, cur_enemy_index) as i32, game);
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        ((*E).var_01 as u32) << 16i32 | (*E).var_00 as u32,
    );
}
pub unsafe fn WreckedShipOrbs_Func_5(game: &mut Game) {
    let E = Get_WreckedShipOrbs(ram_word!(game, cur_enemy_index) as i32, game);
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        ((*E).var_03 as u32) << 16i32 | (*E).var_02 as u32,
    );
}
pub unsafe fn WreckedShipOrbs_Func_6(game: &mut Game) {
    let E = Get_WreckedShipOrbs(ram_word!(game, cur_enemy_index) as i32, game);
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        ((*E).var_01 as u32) << 16i32 | (*E).var_00 as u32,
    );
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16;
    WreckedShipSpark_Instr_2 [jp] {
        let E = Get_WreckedShipSpark(ram_word!(game, cur_enemy_index) as i32, game);
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
    };
    WreckedShipSpark_Instr_1 [jp] {
        let E = Get_WreckedShipSpark(ram_word!(game, cur_enemy_index) as i32, game);
        (*E).base.properties = ((*E).base.properties as i32 & !(kEnemyProps_Tangible as i32)) as u16;
    };
}
unsafe fn WreckedShipSpark_Func_4(k: u16, r18: u16, game: &mut Game) {
    let E = Get_WreckedShipSpark(k as i32, game);
    let mut wssk_var_E: i16 = (*E).var_E as i16;
    if wssk_var_E < 0i16 {
        wssk_var_E = ((NextRandom(game) as i32 & 0x3fi32) + 4i32) as i16;
    }
    (*E).var_F = (r18).wrapping_add((wssk_var_E) as u16);
}
pub unsafe fn WreckedShipSpark_Init(game: &mut Game) {
    let E = Get_WreckedShipSpark(ram_word!(game, cur_enemy_index) as i32, game);
    let v2: i32 = (*E).parameter_1 as i32 & 3i32;
    (*E).var_B = crate::rtl::rom_table_word!(game, 0xa8e688, v2 as isize);
    (*E).var_E = (*E).parameter_2;
    WreckedShipSpark_Func_4(ram_word!(game, cur_enemy_index), 0u16, game);
    enemy_instruction! {[(*E).base]; timer = 1u16, instruction = crate::rtl::rom_table_word!(game, 0xa8e682, v2 as isize)}
    (*E).base.timer = 0u16;
    (*E).base.instruction_timer = 1u16;
    if (*ram_ptr!(game; 0xd828, ram_word!(game, area_index) as isize) & 1) == 0 {
        (*E).base.properties = ((*E).base.properties)
            | (*((&raw mut (*(gVramWriteEntry as unsafe fn(i32, &mut Game) -> *mut VramWriteEntry)(
                0i32, game,
            )
            .offset(6))
            .vram_dst as *mut u8)
                .offset(1) as *mut u16));
    }
}
crate::game::address_dispatch! {
    pub unsafe fn CallWreckedShipSpark(ea: u32, k: u16, game: &mut Game) => ea;
        call (k, game); default { Unreachable(game); };
    {
        11069077 => WreckedShipSpark_Func_1; 11069111 => WreckedShipSpark_Func_2;
        11069148 => WreckedShipSpark_Func_3; 11069076 => [()];
    }
}
pub unsafe fn WreckedShipSpark_Main(game: &mut Game) {
    let E = Get_WreckedShipSpark(ram_word!(game, cur_enemy_index) as i32, game);
    CallWreckedShipSpark(
        (((*E).var_B) as u32) | (0xa80000u32),
        ram_word!(game, cur_enemy_index),
        game,
    );
}
pub unsafe fn WreckedShipSpark_Func_1(k: u16, game: &mut Game) {
    let E = Get_WreckedShipSpark(k as i32, game);
    if (*E).var_F == 1u16 {
        (*E).var_B = ((0xa8e6b7i32) as u16);
        enemy_instruction! {[(*E).base]; instruction = addr_kWreckedShipSpark_Ilist_E5A7 as u16, timer = 1u16}
        WreckedShipSpark_Func_4(k, 0u16, game);
    } else {
        (*E).var_F = (*E).var_F.wrapping_sub(1);
    };
}
pub unsafe fn WreckedShipSpark_Func_2(k: u16, game: &mut Game) {
    let E = Get_WreckedShipSpark(k as i32, game);
    if (*E).var_F == 1u16 {
        (*E).var_B = ((0xa8e695i32) as u16);
        enemy_instruction! {[(*E).base]; instruction = addr_kWreckedShipSpark_Ilist_E5E5 as u16, timer = 1u16}
        WreckedShipSpark_Func_4(k, 8u16, game);
    } else {
        (*E).var_F = (*E).var_F.wrapping_sub(1);
    };
}
pub unsafe fn WreckedShipSpark_Func_3(k: u16, game: &mut Game) {
    let E = Get_WreckedShipSpark(k as i32, game);
    if (*E).var_F == 1u16 {
        let v2: u16 = ram_word!(game, cur_enemy_index);
        SpawnEprojWithGfx(
            0u16,
            ram_word!(game, cur_enemy_index),
            addr_kEproj_Sparks as u16,
            game,
        );
        WreckedShipSpark_Func_4(v2, 0u16, game);
    } else {
        (*E).var_F = (*E).var_F.wrapping_sub(1);
    };
}
pub unsafe fn WreckedShipSpark_Shot(game: &mut Game) {
    update_ram_word!(
        game.projectile_dir[ram_word!(game, collision_detection_index)],
        |fresh14| (fresh14 & !(0x10))
    );
}
pub unsafe fn BlueBrinstarFaceBlock_Init(game: &mut Game) {
    let v0 = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    (*v0).current_instruction = addr_kBlueBrinstarFaceBlock_Ilist_E828 as u16;
    let mut v1: u16 = ((0xa8e86ei32) as u16);
    if (ram_word!(game, collected_items) & 4) == 0 {
        v1 = ((0xa8804ci32) as u16);
    }
    (*(ram_ptr!(game, LongPtr; 0x178c))).addr = v1;
    *(&raw mut (*(ram_ptr!(game, LongPtr; 0x178c))).bank as *mut u16) = 168u16;
    set_ram_word!(
        game.variables_for_enemy_graphics_drawn_hook[0] =
            ((16i32 * (*v0).palette_index as i32 & 0xff00i32) >> 8i32) as u16
    );
    set_ram_word!(game.variables_for_enemy_graphics_drawn_hook[2] = 16u16);
    (*v0).parameter_2 = (((*v0).parameter_2 as i32 & 1i32) >> 2i32
        | ((*v0).parameter_2 as i32 & 1i32) << 15i32) as u16;
}
pub unsafe fn BlueBrinstarFaceBlock_Func_1(game: &mut Game) {
    if ram_word!(game, door_transition_flag_enemies) == 0 && {
        update_ram_word!(
            game.variables_for_enemy_graphics_drawn_hook[2],
            wrapping_sub(1),
            reread
        ) == 0
    } {
        set_ram_word!(game.variables_for_enemy_graphics_drawn_hook[2] = 16u16);
        let mut v0: u16 = ram_word!(game, variables_for_enemy_graphics_drawn_hook[0]);
        let mut v1: u16 =
            (8u16).wrapping_mul(ram_word!(game, variables_for_enemy_graphics_drawn_hook[1]));
        for _ in 0i32..4i32 {
            *(ram_ptr!(game, u16; 0xc000)).offset(((v0 as i32 >> 1i32) + 137i32) as isize) =
                crate::rtl::rom_table_word!(game, 0xa8e7cc, (v1 >> 1) as isize);
            v1 = (v1).wrapping_add(2u16);
            v0 = (v0).wrapping_add(2u16);
        }
        set_ram_word!(
            game.variables_for_enemy_graphics_drawn_hook[1] =
                (*((ram_ptr!(game, u16; 0x1794)).offset(1) as *mut u8).offset(0) as i32 + 1i32
                    & 7i32) as u16
        );
    }
}
pub unsafe fn BlueBrinstarFaceBlock_Main(game: &mut Game) {
    if (ram_word!(game, collected_items) & 4) != 0 {
        (*(ram_ptr!(game, LongPtr; 0x178c))).addr = ((0xa8e86ei32) as u16);
        let v1 = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
        set_ram_word!(
            game.variables_for_enemy_graphics_drawn_hook[0] =
                ((16i32 * (*v1).palette_index as i32 & 0xff00i32) >> 8i32) as u16
        );
        if (*v1).ai_var_A == 0 {
            let SamusEnemyDelta_Y: u16 =
                GetSamusEnemyDelta_Y(ram_word!(game, cur_enemy_index), game);
            if ((Abs16(SamusEnemyDelta_Y)) as i16).wrapping_sub(((*v1).parameter_1) as i16) < 0i16 {
                let SamusEnemyDelta_X: u16 =
                    GetSamusEnemyDelta_X(ram_word!(game, cur_enemy_index), game);
                (*v1).ai_var_B = SamusEnemyDelta_X;
                if ((Abs16(SamusEnemyDelta_X)) as i16).wrapping_sub(((*v1).parameter_1) as i16)
                    < 0i16
                    && (*v1).ai_var_B & 0x8000 != (*v1).parameter_2
                {
                    let mut v4: u16 = addr_kBlueBrinstarFaceBlock_Ilist_E80C as u16;
                    if ((*v1).ai_var_B & 0x8000) == 0 {
                        v4 = addr_kBlueBrinstarFaceBlock_Ilist_E81A as u16;
                    }
                    enemy_instruction! {[(*v1)]; instruction = v4, timer = 1u16}
                    (*v1).ai_var_A = 1u16;
                    set_ram_word!(game.variables_for_enemy_graphics_drawn_hook[2] = 16u16);
                }
            }
        }
    }
}
pub unsafe fn BlueBrinstarFaceBlock_Shot(game: &mut Game) {
    update_ram_word!(
        game.projectile_dir[ram_word!(game, collision_detection_index)],
        |fresh1| (fresh1 & !(0x10))
    );
}
pub unsafe fn KiHunter_Init(game: &mut Game) {
    let E = Get_KiHunter(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_DisableSamusColl) as u16);
    (*E).var_14 = 0u16;
    (*E).base.instruction_timer = 1u16;
    (*E).base.timer = 0u16;
    (*E).base.current_instruction = addr_kKiHunter_Ilist_E9FA as u16;
    (*E).var_F = 0u16;
    (*E).var_A = ((0xa8f268i32) as u16);
    (*E).var_08 = 0u16;
    (*E).var_09 = 1u16;
    (*E).var_06 = 0u16;
    (*E).var_07 = -1i32 as u16;
    let v1: u16 = ((*E).base.y_pos).wrapping_sub(16u16);
    (*E).var_0A = v1;
    (*E).var_0B = (v1).wrapping_add(32u16);
    (*E).var_0C = (*E).base.x_pos;
    (*E).var_0D = (*E).base.y_pos;
    if ((*E).parameter_1 & 0x8000) != 0 {
        (*E).var_14 = 1u16;
        (*E).var_A = ((0xa8f55ai32) as u16);
        (*E).var_08 = 0u16;
        (*E).var_09 = 1u16;
    }
}
pub unsafe fn KiHunterWings_Init(game: &mut Game) {
    let E = Get_KiHunter(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_DisableSamusColl) as u16);
    (*E).base.instruction_timer = 1u16;
    (*E).base.timer = 0u16;
    (*E).base.current_instruction = addr_kKiHunter_Ilist_EA4E as u16;
    let v2: i32 = ram_word!(game, cur_enemy_index) as i32 >> 1i32;
    (*E).base.y_pos = ram_word!(game, enemy_drawing_queue[(v2 + 93i32)]);
    (*E).base.x_pos = ram_word!(game, enemy_drawing_queue[(v2 + 91i32)]);
    (*E).var_A = ((0xa8f6f3i32) as u16);
    (*E).base.palette_index = ram_word!(game, enemy_drawing_queue[(v2 + 105i32)]);
    (*E).base.vram_tiles_index = ram_word!(game, enemy_drawing_queue[(v2 + 106i32)]);
    if (ram_word!(game, enemy_drawing_queue_sizes[(v2 + 6i32)]) & 0x8000) != 0 {
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
    }
}
crate::game::address_dispatch! {
    pub unsafe fn CallKiHunterFunc(ea: u32, k: u16, game: &mut Game) => ea;
        call (k, game); default { Unreachable(game); };
    {
        11072104 => KiHunter_Func_1; 11072440 => KiHunter_Func_2; 11072749 => KiHunter_Func_3;
        11072858 => KiHunter_Func_4; 11072907 => KiHunter_Func_5; 11073008 => KiHunter_Func_6;
        11073163 => KiHunter_Func_7; 11073203 => KiHunter_Func_8; 11073267 => KiHunter_Func_9;
        11073487 => KiHunter_Func_10; 11072995 => [()];
    }
}
pub unsafe fn KiHunter_Main(game: &mut Game) {
    let E = Get_KiHunter(ram_word!(game, cur_enemy_index) as i32, game);
    CallKiHunterFunc(
        (((*E).var_A) as u32) | (0xa80000u32),
        ram_word!(game, cur_enemy_index),
        game,
    );
}
pub unsafe fn KiHunterWings_Main(game: &mut Game) {
    let E = Get_KiHunter(ram_word!(game, cur_enemy_index) as i32, game);
    CallKiHunterFunc(
        (((*E).var_A) as u32) | (0xa80000u32),
        ram_word!(game, cur_enemy_index),
        game,
    );
}
pub unsafe fn KiHunter_Func_1(k: u16, game: &mut Game) {
    let mut v4: i16 = 0;
    let E = Get_KiHunter(k as i32, game);
    let update_velocity: bool = if Enemy_MoveDown(
        k,
        (((*E).var_09 as u32) << 16i32 | (*E).var_08 as u32) as i32,
        game,
    ) != 0
    {
        v4 = -((*E).var_09 as i32) as i16;
        true
    } else {
        let y_pos: u16 = (*E).base.y_pos;
        if ((y_pos) as i16).wrapping_sub(((*E).var_0A) as i16) < 0i16 {
            v4 = 1i16;
            true
        } else {
            !(((y_pos) as i16).wrapping_sub(((*E).var_0B) as i16) < 0i16)
                && ({
                    v4 = -1i16;
                    true
                })
        }
    };
    if update_velocity {
        (*E).var_09 = v4 as u16;
    }
    if Enemy_MoveRight_IgnoreSlopes(
        k,
        (((*E).var_07 as u32) << 16i32 | (*E).var_06 as u32) as i32,
        game,
    ) != 0
    {
        let mut v5: u16 = 0u16;
        let v6: bool = (-((*E).var_07 as i32) & 0x8000i32) != 0i32;
        (*E).var_07 = -((*E).var_07 as i32) as u16;
        if !v6 {
            v5 = 4u16;
        }
        let v7: i32 = v5 as i32 >> 1i32;
        (*E).base.current_instruction = crate::rtl::rom_table_word!(game, 0xa8f3b0, v7 as isize);
        let E1 = Get_KiHunter(k as i32 + 64i32, game);
        (*E1).base.current_instruction =
            crate::rtl::rom_table_word!(game, 0xa8f3b0, (v7 + 1i32) as isize);
        (*E).base.instruction_timer = 1u16;
        (*E1).base.instruction_timer = 1u16;
    }
    EnemyFunc_C8AD(k, game);
    let r18: u16 = (ram_word!(game, samus_x_pos)).wrapping_sub((*E).base.x_pos);
    let r20: u16 = abs16(r18);
    if ((r20) as i16).wrapping_sub((g_word_A8F180) as i16) < 0i16
        && ram_word!(game, samus_y_pos) as i32 - (*E).base.y_pos as i32 - 32i32 & 0x8000i32 == 0
    {
        let r24: u16 = (ram_word!(game, samus_y_pos)).wrapping_sub((*E).base.y_pos);
        let mut v9: u16 = 0;
        if (r18 & 0x8000) != 0 {
            (*E).var_00 = -2i32 as u16;
            set_fields! {[(*E)], 0u16; var_02, var_03}
            (*E).var_04 = -1i32 as u16;
            (*E).var_07 = -1i32 as u16;
            (*E).var_05 = -8192i32 as u16;
            (*E).var_F = 255u16;
            (*E).var_0E = 240u16;
            v9 = 0u16;
        } else {
            (*E).var_00 = 2u16;
            set_fields! {[(*E)], 0u16; var_02, var_03, var_04}
            (*E).var_05 = 0x2000u16;
            (*E).var_F = 128u16;
            (*E).var_07 = 1u16;
            (*E).var_0E = 144u16;
            v9 = 4u16;
        }
        let v10: i32 = v9 as i32 >> 1i32;
        (*E).base.current_instruction = crate::rtl::rom_table_word!(game, 0xa8f3b0, v10 as isize);
        let E1_0 = Get_KiHunter(k as i32 + 64i32, game);
        (*E1_0).base.current_instruction =
            crate::rtl::rom_table_word!(game, 0xa8f3b0, (v10 + 1i32) as isize);
        (*E).base.instruction_timer = 1u16;
        (*E1_0).base.instruction_timer = 1u16;
        (*E).var_B = (r18).wrapping_add((*E).base.x_pos);
        (*E).var_C = (*E).base.y_pos;
        (*E).var_A = ((0xa8f3b8i32) as u16);
        (*E).var_12 = r24;
        (*E).var_11 = r20;
        (*E).var_10 = 0u16;
    }
}
pub unsafe fn KiHunter_Func_2(k: u16, game: &mut Game) {
    let mut current_block: u64;
    let E = Get_KiHunter(k as i32, game);
    if ((*E).var_04 & 0x8000) == 0 {
        if (((*E).var_F) as i16).wrapping_sub(((*E).var_0E) as i16) < 0i16 {
            current_block = 17315748505249742599;
        } else {
            current_block = 17179679302217393232;
        }
    } else if (((*E).var_F) as i16).wrapping_sub(((*E).var_0E) as i16) >= 0i16 {
        current_block = 17315748505249742599;
    } else {
        current_block = 17179679302217393232;
    }
    if current_block == 17179679302217393232 && (*E).var_10 == 0 {
        (*E).var_10 = 1u16;
        let mut v2: u16 = addr_kKiHunter_Ilist_EA32 as u16;
        if ((*E).var_04 & 0x8000) != 0 {
            v2 = addr_kKiHunter_Ilist_EA08 as u16;
        }
        enemy_instruction! {[(*E).base]; instruction = v2, timer = 1u16}
    }
    if ((*E).var_04 & 0x8000) == 0 {
        AddToHiLo(
            &raw mut (*E).var_02,
            &raw mut (*E).var_03,
            ((*E).var_04 as u32) << 16i32 | (*E).var_05 as u32,
        );
        if (((*E).var_02) as i16).wrapping_sub(((*E).var_00) as i16) >= 0i16 {
            (*E).var_02 = (*E).var_00;
        }
        (*E).var_F = ((*E).var_F).wrapping_add((*E).var_02);
        if (*E).var_F as i32 - 256i32 & 0x8000i32 == 0 {
            (*E).var_A = ((0xa8f268i32) as u16);
            return;
        }
    } else {
        AddToHiLo(
            &raw mut (*E).var_02,
            &raw mut (*E).var_03,
            ((*E).var_04 as u32) << 16i32 | (*E).var_05 as u32,
        );
        if (((*E).var_02) as i16).wrapping_sub(((*E).var_00) as i16) < 0i16 {
            (*E).var_02 = (*E).var_00;
        }
        (*E).var_F = ((*E).var_F).wrapping_add((*E).var_02);
        if (*E).var_F as i32 - 128i32 & 0x8000i32 != 0 {
            (*E).var_A = ((0xa8f268i32) as u16);
            return;
        }
    }
    let mut r20: u16 = ((*E).var_B as i32 + CosineMult8bit((*E).var_F, (*E).var_11, game) as i32
        - (*E).base.x_pos as i32) as u16;
    if Enemy_MoveRight_IgnoreSlopes(k, (r20 as i16 as i32) << 16i32, game) != 0 {
        if ((*E).var_04 & 0x8000) == 0 {
            (*E).var_06 = 0u16;
            (*E).var_07 = -1i32 as u16;
        } else {
            (*E).var_06 = 0u16;
            (*E).var_07 = 1u16;
        }
        current_block = 14751541640656221110;
    } else {
        EnemyFunc_C8AD(k, game);
        r20 = ((*E).var_C as i32 + SineMult8bit((*E).var_F, (*E).var_12, game) as i32
            - (*E).base.y_pos as i32) as u16;
        if Enemy_MoveDown(k, (r20 as i16 as i32) << 16i32, game) != 0 {
            current_block = 14751541640656221110;
        } else {
            current_block = 7828949454673616476;
        }
    }
    if current_block == 14751541640656221110 {
        (*E).var_A = ((0xa8f4edi32) as u16);
        (*E).var_08 = 0u16;
        (*E).var_09 = -1i32 as u16;
    };
}
pub unsafe fn KiHunter_Func_3(k: u16, game: &mut Game) {
    let E = Get_KiHunter(k as i32, game);
    if Enemy_MoveRight_IgnoreSlopes(
        k,
        (((*E).var_07 as u32) << 16i32 | (*E).var_06 as u32) as i32,
        game,
    ) != 0u8
        || {
            EnemyFunc_C8AD(k, game);
            Enemy_MoveDown(
                k,
                (((*E).var_09 as u32) << 16i32 | (*E).var_08 as u32) as i32,
                game,
            ) != 0u8
        }
        || (((*E).base.y_pos) as i16).wrapping_sub(((*E).var_0D) as i16) < 0i16
    {
        (*E).var_A = ((0xa8f268i32) as u16);
    }
}
pub unsafe fn KiHunter_Instr_1(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let mut result: u16 = addr_kKiHunter_Ilist_E9FA as u16;
    let mut r18: u16 = addr_kKiHunter_Ilist_EA4E as u16;
    let E = Get_KiHunter(k as i32, game);
    if ((*E).var_07 & 0x8000) == 0 {
        result = addr_kKiHunter_Ilist_EA24 as u16;
        r18 = addr_kKiHunter_Ilist_EA5E as u16;
    }
    enemy_instruction! {[(*E).base]; instruction = r18, timer = 1u16}
    let E1 = Get_KiHunter(k as i32 + 64i32, game);
    if (*E1).var_A as i32 == 0xa8f6f3i32 & 0xffffi32 {
        enemy_instruction! {[(*E1).base]; instruction = r18, timer = 1u16}
    }
    result as usize as *const u16
}
pub unsafe fn KiHunter_Func_4(k: u16, game: &mut Game) {
    let E = Get_KiHunter(k as i32, game);
    if Enemy_MoveDown(
        k,
        (((*E).var_09 as u32) << 16i32 | (*E).var_08 as u32) as i32,
        game,
    ) != 0
    {
        (*E).var_A = ((0xa8f58bi32) as u16);
    } else {
        AddToHiLo(
            &raw mut (*E).var_09,
            &raw mut (*E).var_08,
            (g_word_A8F184 as u32) << 16i32 | g_word_A8F182 as u32,
        );
    };
}
pub unsafe fn KiHunter_Func_5(k: u16, game: &mut Game) {
    let E = Get_KiHunter(k as i32, game);
    (*E).var_A = addr_locret_A8F5E3 as u16;
    (*E).var_08 = 0u16;
    (*E).var_09 = ((ram_word!(game, random_number) as i32 & 1i32) - 8i32) as u16;
    if (((*E).base.x_pos) as i16).wrapping_sub((ram_word!(game, samus_x_pos)) as i16) >= 0i16 {
        (*E).var_06 = 0u16;
        (*E).var_07 = -2i32 as u16;
        (*E).base.current_instruction = addr_kKiHunter_Ilist_EA8A as u16;
    } else {
        (*E).var_06 = 0u16;
        (*E).var_07 = 2u16;
        (*E).base.current_instruction = addr_kKiHunter_Ilist_EAA6 as u16;
    }
    (*E).base.instruction_timer = 1u16;
}
pub unsafe fn KiHunter_Instr_2(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_KiHunter(ram_word!(game, cur_enemy_index) as i32, game)).var_A = ((0xa8f5f0i32) as u16);
    jp
}
pub unsafe fn KiHunter_Func_6(k: u16, game: &mut Game) {
    let E = Get_KiHunter(k as i32, game);
    if Enemy_MoveDown(
        k,
        (((*E).var_09 as u32) << 16i32 | (*E).var_08 as u32) as i32,
        game,
    ) != 0
    {
        if ((*E).var_09 & 0x8000) != 0 {
            (*E).var_09 = 1u16;
        } else {
            (*E).var_08 = 0u16;
            (*E).var_09 = -4i32 as u16;
            (*E).var_A = addr_locret_A8F5E3 as u16;
            (*E).var_0F = 12u16;
            let mut v4: u16 = addr_kKiHunter_Ilist_EAC2 as u16;
            if (*E).base.current_instruction as i32 + 0x155ai32 & 0x8000i32 == 0 {
                v4 = addr_kKiHunter_Ilist_EADA as u16;
            }
            enemy_instruction! {[(*E).base]; instruction = v4, timer = 1u16}
        }
    } else if Enemy_MoveRight_IgnoreSlopes(
        k,
        (((*E).var_07 as u32) << 16i32 | (*E).var_06 as u32) as i32,
        game,
    ) != 0
    {
        (*E).var_07 = -((*E).var_07 as i32) as u16;
    } else {
        EnemyFunc_C8AD(k, game);
        AddToHiLo(
            &raw mut (*E).var_09,
            &raw mut (*E).var_08,
            (g_word_A8F184 as u32) << 16i32 | g_word_A8F182 as u32,
        );
    };
}
pub unsafe fn KiHunter_Instr_3(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_KiHunter(ram_word!(game, cur_enemy_index) as i32, game)).var_A = ((0xa8f68bi32) as u16);
    jp
}
pub unsafe fn KiHunter_Func_7(k: u16, game: &mut Game) {
    let E = Get_KiHunter(k as i32, game);
    let v2: u16 = ((*E).var_0F).wrapping_sub(1u16);
    (*E).var_0F = v2;
    if v2 == 0 {
        let mut v3: u16 = ((0xa8f58bi32) as u16);
        let v4: u16 = abs16(((*E).base.x_pos).wrapping_sub(ram_word!(game, samus_x_pos)));
        if v4 as i32 - 96i32 & 0x8000i32 != 0 {
            v3 = ((0xa8f6b3i32) as u16);
        }
        (*E).var_A = v3;
    }
}
pub unsafe fn KiHunter_Func_8(k: u16, game: &mut Game) {
    let mut v1: u16 = addr_kKiHunter_Ilist_EAF2 as u16;
    let E = Get_KiHunter(k as i32, game);
    if (((*E).base.x_pos) as i16).wrapping_sub((ram_word!(game, samus_x_pos)) as i16) < 0i16 {
        v1 = addr_kKiHunter_Ilist_EB10 as u16;
    }
    enemy_instruction! {[(*E).base]; instruction = v1, timer = 1u16}
    (*E).var_A = addr_locret_A8F5E3 as u16;
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16;
    KiHunter_Instr_4 [jp] { sub_A8F6DC(k, addr_loc_A8CF18 as u16, game); };
    KiHunter_Instr_5 [jp] { sub_A8F6DC(k, addr_loc_A8CF26 as u16, game); };
}
pub unsafe fn sub_A8F6DC(k: u16, j: u16, game: &mut Game) {
    QueueSfx2_Max6(0x4cu16, game);
    SpawnEprojWithGfx(0u16, ram_word!(game, cur_enemy_index), j, game);
    (*Get_KiHunter(ram_word!(game, cur_enemy_index) as i32, game)).var_0F = 24u16;
}
pub unsafe fn KiHunter_Func_9(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1i32;
    let E = Get_KiHunter(k as i32, game);
    (*E).base.x_pos = ram_word!(game, enemy_drawing_queue[(v1 + 91i32)]);
    (*E).base.y_pos = ram_word!(game, enemy_drawing_queue[(v1 + 93i32)]);
}
pub unsafe fn KiHunter_Shot(game: &mut Game) {
    NormalEnemyShotAi(game);
    let old_cur_enemy_index: u16 = ram_word!(game, cur_enemy_index);
    let EK = Get_KiHunter(ram_word!(game, cur_enemy_index) as i32, game);
    let E1 = Get_KiHunter(ram_word!(game, cur_enemy_index) as i32 + 64i32, game);
    let health: u16 = (*EK).base.health;
    if health != 0 {
        if health == (*E1).parameter_1
            || ((health) as i16).wrapping_sub(((*E1).parameter_1) as i16) < 0i16
        {
            if (*EK).var_14 == 0 {
                (*EK).var_14 = 1u16;
                (*EK).var_A = ((0xa8f55ai32) as u16);
                (*EK).var_08 = 0u16;
                (*EK).var_09 = 1u16;
                set_ram_word!(game.cur_enemy_index = (old_cur_enemy_index).wrapping_add(64u16));
                if (*E1).var_A as i32 != 0xa8f7cfi32 & 0xffffi32 {
                    (*E1).var_07 = (*E1).base.y_pos;
                    (*E1).var_08 = (*E1).base.x_pos;
                    KiHunter_Func_17(game);
                    KiHunter_Func_12(game);
                    KiHunter_Func_13(game);
                    (*E1).var_F = -8192i32 as u16;
                    (*E1).var_A = ((0xa8f7cfi32) as u16);
                    (*E1).var_00 = ((0xa8f7dbi32) as u16);
                    (*E1).var_06 = ((*E1).var_07).wrapping_sub((*E1).var_0B);
                    (*E1).var_05 = (*E1).base.x_pos;
                    (*E1).var_B = (*E1).var_0A;
                    (*E1).base.current_instruction = addr_kKiHunter_Ilist_EA7E as u16;
                    (*E1).base.spritemap_pointer = addr_kSpritemap_Nothing_A8 as u16;
                    (*E1).base.instruction_timer = 1u16;
                    (*E1).base.timer = 0u16;
                    (*E1).base.properties =
                        ((*E1).base.properties) | ((kEnemyProps_ProcessedOffscreen) as u16);
                }
                set_ram_word!(game.cur_enemy_index = old_cur_enemy_index);
            }
        } else {
            (*E1).base.ai_handler_bits = (*EK).base.ai_handler_bits;
            (*E1).base.frozen_timer = (*EK).base.frozen_timer;
            (*E1).base.invincibility_timer = (*EK).base.invincibility_timer;
            (*E1).base.flash_timer = (*EK).base.flash_timer;
        }
    } else {
        (*E1).base.properties = 512u16;
    };
}
crate::game::address_dispatch! {
    pub unsafe fn CallKiHunterBFunc(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        11073499 => KiHunter_Func_11; 11073709 => KiHunter_Func_14;
    }
}
pub unsafe fn KiHunter_Func_10(k: u16, game: &mut Game) {
    let r18: u16 = (*Get_KiHunter(ram_word!(game, cur_enemy_index) as i32, game)).var_00;
    CallKiHunterBFunc(((r18) as u32) | (0xa80000u32), game);
}
pub unsafe fn KiHunter_Func_11(game: &mut Game) {
    let E = Get_KiHunter(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).var_F = ((*E).var_F).wrapping_add(
        *(((game.g_rom.offset(crate::rtl::lorom_offset(0xa2838f)) as *mut u16).offset(2) as *mut u8)
            .offset((8i32 * *(&raw mut (*E).var_B as *mut u8).offset(1isize) as i32) as isize)
            .offset(1) as *mut u16),
    );
    (*E).base.y_pos = ((*E).var_06 as i32
        + SineMult8bit(
            *(&raw mut (*E).var_F as *mut u8).offset(1isize) as u16,
            g_byte_A8F186 as u16,
            game,
        ) as i32
        - (*E).var_04 as i32) as u16;
    (*E).base.x_pos = ((*E).var_05 as i32
        + CosineMult8bit(
            *(&raw mut (*E).var_F as *mut u8).offset(1isize) as u16,
            g_byte_A8F186 as u16,
            game,
        ) as i32
        - (*E).var_03 as i32) as u16;
    if (*E).var_F as i32 + 0x4000i32 & 0x8000i32 != 0 {
        KiHunter_Func_16(ram_word!(game, cur_enemy_index), game);
    } else {
        let mut v2: i16 = (((*E).var_B) as i16).wrapping_sub(384i16);
        if v2 < 0i16 {
            v2 = 256i16;
        }
        (*E).var_B = v2 as u16;
    };
}
pub unsafe fn KiHunter_Func_12(game: &mut Game) {
    let v1: u16 = CosineMult8bit(0xe0u16, g_byte_A8F186 as u16, game);
    let E = Get_KiHunter(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).var_03 = v1;
    (*E).var_04 = SineMult8bit(0xe0u16, g_byte_A8F186 as u16, game);
}
pub unsafe fn KiHunter_Func_13(game: &mut Game) {
    let v1: u16 = CosineMult8bit(0xa0u16, g_byte_A8F186 as u16, game);
    let E = Get_KiHunter(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).var_01 = v1;
    (*E).var_02 = SineMult8bit(0xa0u16, g_byte_A8F186 as u16, game);
}
pub unsafe fn KiHunter_Func_14(game: &mut Game) {
    let mut khr_var_07: i16 = 0;
    let E = Get_KiHunter(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).var_F = ((*E).var_F).wrapping_add(
        *((game.g_rom.offset(crate::rtl::lorom_offset(0xa2838f)) as *mut u16 as *mut u8)
            .offset((8i32 * *(&raw mut (*E).var_B as *mut u8).offset(1isize) as i32) as isize)
            .offset(1) as *mut u16),
    );
    let r20: u16 = ((*E).var_06 as i32
        + SineMult8bit(
            *(&raw mut (*E).var_F as *mut u8).offset(1isize) as u16,
            g_byte_A8F186 as u16,
            game,
        ) as i32
        - (*E).var_02 as i32
        - (*E).base.y_pos as i32) as u16;
    if Enemy_MoveDown(
        ram_word!(game, cur_enemy_index),
        (r20 as i16 as i32) << 16i32,
        game,
    ) != 0
    {
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
        (*E).base.x_pos = (*E).var_08;
        khr_var_07 = (*E).var_07 as i16;
        (*E).base.y_pos = khr_var_07 as u16;
    } else {
        (*E).base.x_pos = ((*E).var_05 as i32
            + CosineMult8bit(
                *(&raw mut (*E).var_F as *mut u8).offset(1isize) as u16,
                g_byte_A8F186 as u16,
                game,
            ) as i32
            - (*E).var_01 as i32) as u16;
        if (*E).var_F as i32 + 0x4000i32 & 0x8000i32 == 0 {
            KiHunter_Func_15(ram_word!(game, cur_enemy_index), game);
            return;
        }
        khr_var_07 = (((*E).var_B) as i16).wrapping_sub(384i16);
        if khr_var_07 < 0i16 {
            khr_var_07 = 256i16;
        }
    }
    (*Get_KiHunter(ram_word!(game, cur_enemy_index) as i32, game)).var_B = khr_var_07 as u16;
}
pub unsafe fn KiHunter_Func_15(k: u16, game: &mut Game) {
    let E = Get_KiHunter(k as i32, game);
    (*E).var_00 = ((0xa8f7dbi32) as u16);
    (*E).var_B = (*E).var_0A;
    (*E).var_F = -8192i32 as u16;
    (*E).var_05 = (*E).base.x_pos;
    (*E).var_06 = (*E).base.y_pos;
}
pub unsafe fn KiHunter_Func_16(k: u16, game: &mut Game) {
    let E = Get_KiHunter(k as i32, game);
    (*E).var_00 = ((0xa8f8adi32) as u16);
    (*E).var_B = (*E).var_0A;
    (*E).var_F = -24576i32 as u16;
    (*E).var_05 = (*E).base.x_pos;
    (*E).var_06 = (*E).base.y_pos;
}
pub unsafe fn KiHunter_Func_17(game: &mut Game) {
    let E = Get_KiHunter(ram_word!(game, cur_enemy_index) as i32, game);
    set_fields! {[(*E)], 0u16; var_0A, var_B}
    loop {
        let mut v3: u16 = ((*E).var_0A).wrapping_add(384u16);
        (*E).var_0A = v3;
        v3 = (v3 >> 8);
        let mut v5: u16 = (*((game.g_rom.offset(crate::rtl::lorom_offset(0xa2838f)) as *mut u16
            as *mut u8)
            .offset((8i32 * v3 as i32) as isize)
            .offset(1) as *mut u16))
            .wrapping_add((*E).var_D);
        (*E).var_D = v5;
        if (v5 as i32 - 0x2000i32 & 0x8000i32 == 0) {
            break;
        }
    }
}
