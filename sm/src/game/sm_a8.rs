pub use crate::game::prelude::*;

pub use crate::game::constants::bank_86::*;

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

pub use crate::types::Consts_A8;

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
crate::type_aliases! {
    pub Enemy_Fune, Enemy_KiHunter, Enemy_MiniDraygon, Enemy_NorfairLavaMan,
        Enemy_WalkingLavaSeahorse, Enemy_WreckedShipGhost, Enemy_Beetom, Enemy_Kago,
        Enemy_WreckedShipOrbs = crate::game::enemy_layouts::EnemyLayout40;
}

crate::type_aliases! {
    pub Enemy_WreckedShipRobot, Enemy_WreckedShipSpark, Enemy_MaridiaFloater, Enemy_MorphBallEye =
        crate::game::enemy_layouts::EnemyLayout8;
}

pub type Enemy_YappingMaw = crate::game::enemy_layouts::EnemyLayout104;

pub type Enemy_MaridiaPuffer = crate::game::enemy_layouts::EnemyLayout72Late;

const RomPtr_A8: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0xa8>;
crate::ram_map::ram_pointer_functions! {
    gVramWriteEntry(a: i32) -> VramWriteEntry = [(0xd0 + a) as isize];
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
static g_word_A89050: u16 = 0x80;
static g_word_A89052: u16 = 0xb0;
static g_word_A89054: u16 = 0x80;
static g_word_A89056: u16 = 0x80;
static g_word_A89A9C: u16 = 0x10;
static g_word_A89A9E: u16 = 0x40;
static g_word_A89AA0: u16 = 0x1800;
static g_word_A89AA2: u16 = 1;
static g_word_A89AA4: u16 = 0x78;
static g_word_A89AA6: u16 = 0x78;
static g_word_A89AA8: [u16; 18] = [
    0xffc0, 0xffc0, 0, 0xffc0, 0x40, 0, 0xffc0, 0, 0, 0, 0x40, 0, 0xffc0, 0x40, 0, 0x40, 0x40, 0x40,
];
static g_word_A89ACC: [u16; 17] = [1, 8, 1, 8, 1, 7, 1, 7, 2, 6, 2, 6, 3, 5, 3, 5, 0xffff];
static kWreckedShipGhost_Palette: [u16; 16] = [
    0x3800, 0x57ff, 0x42f7, 0x929, 0xa5, 0x4f5a, 0x36b5, 0x2610, 0x1dce, 0x1df, 0x1f, 0x18, 0xa,
    0x6b9, 0xea, 0x45,
];
static g_word_A89D32: u16 = 1;
static g_word_A89D34: u16 = 1;
static g_word_A8C277: [i16; 3] = [-12, -16, -20];
static g_word_A8C27D: [i16; 3] = [-20, -16, -12];
static g_word_A8C19F: u16 = 0x40;
static g_word_A8C1A1: [u16; 12] = [0, 1, 2, 3, 2, 1, 0, 0xffff, 0xfffe, 0xfffd, 0xfffe, 0xffff];
static g_word_A8C1B9: u16 = 0;
static g_word_A8C1BB: u16 = 0;
static g_word_A8C1BD: u16 = 0;
static g_word_A8C1BF: u16 = 0;
static g_word_A8C1C1: u16 = 1;
static g_word_A8C1C3: u16 = 0;
static g_word_A8C1C5: u16 = 0xffff;
static g_word_A8C1C7: u16 = 0x8000;
static g_word_A8CCC1: [i16; 31] = [
    0x1f, 0x18, 0xf, 8, 0x40, 0x18, 0xf, 8, 0x1f, 0x10, 0xf, 8, 0x1f, 0x18, 0x10, 8, 0x1f, 0x18,
    0xf, 0x40, 0xf, 8, 0x1f, 0x18, 0x10, 0x18, 0xf, 8, 0x1f, 0x10, -1,
];
static g_word_A8D871: [i16; 10] = [0xc0, 0xe0, 0, 0x20, 0x40, 0x40, 0x60, 0x80, 0xa0, 0xc0];
static g_word_A8D885: [i16; 8] = [0x3ff, 0x4ff, 0x5ff, 0x6ff, 0x7ff, 0x8ff, 0x9ff, 0xaff];
static g_word_A8D895: [i16; 26] = [
    3, 1, 4, 1, 5, 2, 6, 2, 7, 2, 8, 3, 9, 3, 0xa, 4, 0xb, 4, 0xc, 5, 0xd, 5, 0xe, 6, 0xf, 6,
];
static g_word_A8DCC7: u16 = 0x50;
static g_word_A8DCCB: u16 = 0x70;
static g_word_A8F180: u16 = 0x60;
static g_word_A8F182: u16 = 0xe000;
static g_word_A8F184: u16 = 0;
static g_byte_A8F186: u8 = 0x30;
pub unsafe fn MiniDraygon_Func_1(game: &mut Game) {
    (*Get_MiniDraygon(word!(game.cur_enemy_index) as i32, game)).var_B =
        ((GetSamusEnemyDelta_X(word!(game.cur_enemy_index), game) & 0x8000) == 0) as u16;
    let E = Get_MiniDraygon(word!(game.cur_enemy_index) as i32, game);
    (*E).var_02 = addr_kMiniDraygon_Ilist_86A7 as u16;
    if (*E).var_B != 0 {
        (*E).var_02 = addr_kMiniDraygon_Ilist_870B as u16;
    }
    MiniDraygon_Func_12(game);
}
pub unsafe fn MiniDraygon_Func_2(game: &mut Game) {
    let v0: i32 = word!(game.cur_enemy_index) as i32 >> 1;
    let v1: u16 = word!(game.enemy_drawing_queue_sizes[(v0 + 1)]);
    let E = Get_MiniDraygon(word!(game.cur_enemy_index) as i32, game);
    (*E).var_B = v1;
    if v1 != 0 {
        (*E).base.x_pos = (word!(game.enemy_drawing_queue[(v0 + 91)])).wrapping_add(4);
        (*E).base.y_pos = (word!(game.enemy_drawing_queue[(v0 + 93)])).wrapping_add(10);
        (*E).var_02 = addr_kMiniDraygon_Ilist_8727 as u16;
    } else {
        (*E).base.x_pos = (word!(game.enemy_drawing_queue[(v0 + 91)])).wrapping_sub(4);
        (*E).base.y_pos = (word!(game.enemy_drawing_queue[(v0 + 93)])).wrapping_add(10);
        (*E).var_02 = addr_kMiniDraygon_Ilist_86C3 as u16;
    }
    MiniDraygon_Func_12(game);
}
pub unsafe fn MiniDraygon_Func_3(game: &mut Game) {
    let v0: i32 = word!(game.cur_enemy_index) as i32 >> 1;
    let v1: u16 = word!(game.enemy_drawing_queue[(v0 + 83)]);
    let E = Get_MiniDraygon(word!(game.cur_enemy_index) as i32, game);
    (*E).var_B = v1;
    if v1 != 0 {
        (*E).base.x_pos = (word!(game.enemy_drawing_queue[(v0 + 59)])).wrapping_add(4);
    } else {
        (*E).base.x_pos = (word!(game.enemy_drawing_queue[(v0 + 59)])).wrapping_sub(4);
    }
    (*E).base.y_pos = (word!(game.enemy_drawing_queue[(v0 + 61)])).wrapping_add(18);
}
crate::game::callback_dispatch! {
    pub unsafe fn CallMiniDraygonFunc(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [11045154] MiniDraygon_Func_4 [] {
        if (*Get_MiniDraygon(word!(game.cur_enemy_index) as i32, game)).parameter_1 != 0 {
            MiniDraygon_Func_6(game);
        } else {
            MiniDraygon_Func_5(game);
        };
    };
    [11045428] MiniDraygon_Func_8 [] {
        MiniDraygon_Func_3(game);
    };
    [11045435] MiniDraygon_Func_9 [] {
        MiniDraygon_Func_11(game);
        let E = Get_MiniDraygon(word!(game.cur_enemy_index) as i32, game);
        words!((*E).base, x_pos, x_subpos += words!((*E), var_07, var_08));
        words!((*E).base, y_pos, y_subpos += words!((*E), var_09, var_0A));
    };
    [11045496] MiniDraygon_Func_10 [] {
        if *(ram_ptr!(game, u16; 0xe84))
            .offset(((word!(game.cur_enemy_index) as i32 >> 1) + 77) as isize)
            == 0
        {
            let E = Get_MiniDraygon(word!(game.cur_enemy_index) as i32, game);
            if (*E).var_0C != 0 {
                MiniDraygon_Func_3(game);
                (*E).base.x_pos = ((*E).base.x_pos).wrapping_add((*E).var_F);
            } else {
                (*E).var_02 = addr_kMiniDraygon_Ilist_876F as u16;
                MiniDraygon_Func_12(game);
                set_fields! {[(*E)], 0; var_0C, var_0B}
                (*E).var_C = ((0xa88a34i32) as u16);
            }
        }
    };
}
pub unsafe fn MiniDraygon_Func_5(game: &mut Game) {
    if (*Get_MiniDraygon(word!(game.cur_enemy_index) as i32 + 128i32, game)).var_0C == 0 {
        MiniDraygon_Func_1(game);
    }
    let E = Get_MiniDraygon(word!(game.cur_enemy_index) as i32, game);
    if (*E).var_00 != 0 {
        words!((*E).base, y_pos, y_subpos += words!((*E), var_04, var_03));
    } else {
        words!((*E).base, y_pos, y_subpos += words!((*E), var_06, var_05));
    }
    (*E).var_E = (*E).var_E.wrapping_sub(1);
    if ((*E).var_E & 0x8000) != 0 {
        (*E).var_E = byte!((*E).parameter_2; 1) as u16;
        (*E).var_00 = ((*E).var_00) ^ (1u16);
    }
}
pub unsafe fn MiniDraygon_Func_6(game: &mut Game) {
    MiniDraygon_Func_2(game);
}
pub unsafe fn MiniDraygon_Func_7(game: &mut Game) {
    if IsSamusWithinEnemy_X((word!(game.cur_enemy_index)).wrapping_sub(128), 0x80, game) != 0 {
        let r22: u16 = -((((CalculateAngleOfSamusFromEnemy(word!(game.cur_enemy_index), game))
            as u8)
            .wrapping_sub(64u8)) as i32) as u16;
        let E = Get_MiniDraygon(word!(game.cur_enemy_index) as i32, game);
        words!((*E), var_07, var_08 = CosineMult8bitFull(r22, 4, game));
        words!((*E), var_09, var_0A = SineMult8bitFull(r22, 4, game));
        (*E).var_02 = addr_kMiniDraygon_Ilist_876F as u16;
        MiniDraygon_Func_12(game);
        (*E).var_0B = 1;
        (*E).var_C = ((0xa88a3bi32) as u16);
    }
}
pub unsafe fn MiniDraygon_Func_11(game: &mut Game) {
    if EnemyFunc_ADA3(0x100, game) != 0
        && *(ram_ptr!(game, u16; 0xe84))
            .offset(((word!(game.cur_enemy_index) as i32 >> 1) + 77) as isize)
            == 0
    {
        let E = Get_MiniDraygon(word!(game.cur_enemy_index) as i32, game);
        set_fields! {[(*E)]; var_0B = 0, var_0C = 1, var_C = ((0xa88a78i32) as u16), var_0C = 1,
        var_02 = addr_kMiniDraygon_Ilist_8775 as u16, };
        MiniDraygon_Func_12(game);
    }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallFuneFunc(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    @prefix { let E = Get_Fune(word!(game.cur_enemy_index) as i32, game); };
    Fune_Init [] {
        (*E).var_A = addr_off_A896D7 as u16;
        (*E).var_B = ((0xa89737i32) as u16);
        let v1: u16 = ((*E).parameter_1) & (0xfu16);
        (*E).var_D = v1;
        if v1 != 0 {
            (*E).var_A = addr_off_A896DF as u16;
            (*E).var_B = ((0xa8975ci32) as u16);
        }
        if ((*E).parameter_1 & 0xf0) != 0 {
            (*E).var_A = ((*E).var_A).wrapping_add(2);
        }
        Fune_Func_4(game);
        (*E).var_C = byte!((*E).parameter_2; 1) as u16;
        (*E).var_F = byte!((*E).parameter_1; 1) as u16;
        (*E).var_E = 0;
    };
    Fune_Main [] {
        CallFuneFunc((((*E).var_B) as u32) | (0xa80000u32), game);
    };
    [11048759] Fune_Func_1 [] {
        (*E).var_E = (*E).var_E.wrapping_add(1);
        if (((*E).var_E) as i16).wrapping_sub(((*E).var_F) as i16) >= 0 {
            (*E).var_A = ((*E).var_A).wrapping_sub(4);
            Fune_Func_4(game);
            (*E).var_B = ((0xa8978ei32) as u16);
            (*E).var_E = 0;
        }
    };
    [11048796] Fune_Func_2 [] {
        (*E).var_00 = (*E).var_A;
        if Fune_Func_3(game) != 0 {
            let mut fune_var_00: u16 = (*E).var_00;
            (*E).var_A = fune_var_00;
            fune_var_00 = (fune_var_00).wrapping_sub(4);
            (*E).var_00 = fune_var_00;
            (*E).var_A = fune_var_00;
            Fune_Func_4(game);
            (*E).var_B = ((0xa8978fi32) as u16);
        }
        (*E).var_A = (*E).var_00;
    };
    Fune_Func_4 [] {
        set_fields! {[(*E).base]; instruction_timer = 1, timer = 0,
            current_instruction = *(RomPtr_A8((*E).var_A, game) as *mut u16), };
    };
    @external {
        11048846 => [()]; 11048847 => [()];
    }
}
pub unsafe fn Fune_Func_3(game: &mut Game) -> u16 {
    let E = Get_Fune(word!(game.cur_enemy_index) as i32, game);
    IsSamusWithinEnemy_Y(word!(game.cur_enemy_index), (*E).var_C, game)
}
crate::game::callback_dispatch! {
    pub unsafe fn CallWreckedShipGhost(ea: u32, k: u16, game: &mut Game) => ea;
    call (k, game); default { Unreachable(game); };
    @functions [pub unsafe fn (k: u16, game: &mut Game)];
    [11049794] WreckedShipGhost_Func_1 [] {
        WreckedShipGhost_Func_3(k, game);
        let E = Get_WreckedShipGhost(word!(game.cur_enemy_index) as i32, game);
        let v3: u16 = u16::swap_bytes((*E).base.palette_index);
        let mut v4: u16 = (16 * v3 as i32 + 256) as u16;
        let mut v1: i32 = 16;
        for _ in 0..16 {
            let v5: i32 = v4 as i32 >> 1;
            if (word!(game.palette_buffer[v5]) as i32 & 0x1f) - 31i32 & 0x8000 != 0 {
                word!(game.palette_buffer[v5] += 1057);
                v1 -= 1;
            }
            v4 = (v4).wrapping_add(2);
        }
        if ((v1 - 16i32) as i16) >= 0 {
            (*E).var_A = ((0xa89badi32) as u16);
            let v7: u16 = u16::swap_bytes((*E).base.palette_index);
            let mut v8: u16 = (16 * v7 as i32 + 256) as u16;
            for v9 in (0u16..32).step_by(2) {
                word!(
                    game.target_palettes[(v8 >> 1)] = kWreckedShipGhost_Palette[(v9 >> 1) as usize]
                );
                v8 = (v8).wrapping_add(2);
            }
        }
    };
    [11049901] WreckedShipGhost_Func_2 [] {
        let v4: u16 = WreckedShipGhost_Func_8(game);
        WreckedShipGhost_Func_3(word!(game.cur_enemy_index), game);
        let E = Get_WreckedShipGhost(word!(game.cur_enemy_index) as i32, game);
        if (*E).var_B | v4 == 0 {
            (*E).base.properties = ((*E).base.properties) & (0xfaffu16);
            set_fields! {[(*E)]; var_A = ((0xa89c8ai32) as u16), var_00 = (*E).base.y_pos,
                var_B = 1, var_C = 2, var_01 = 0, var_02 = g_word_A89AA2, var_B = g_word_A89AA6,
                var_05 = 4, };
            let v2: u16 = word!(game.samus_x_pos);
            (*E).var_06 = word!(game.samus_x_pos);
            (*E).var_07 = v2;
            (*E).var_08 = v2;
            (*E).var_09 = 12;
            let v3: u16 = *((ram_ptr!(game, u16; 0xafa) as *mut u8)
                .offset(word!(game.cur_enemy_index) as i32 as isize)
                as *mut u16);
            set_fields! {[(*E)]; var_0A = v3, var_0B = v3, var_0C = v3, var_0D = g_word_A89A9E,
                var_0E = g_word_A89A9C, };
        }
    };
    [11050089] WreckedShipGhost_Func_4 [] {
        if WreckedShipGhost_Func_8(game) == 0 {
            let E = Get_WreckedShipGhost(word!(game.cur_enemy_index) as i32, game);
            (*E).var_A = ((0xa89d13i32) as u16);
            (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
            (*E).var_B = g_word_A89AA4;
        }
    };
    [11050122] WreckedShipGhost_Func_5 [] {
        let E = Get_WreckedShipGhost(k as i32, game);
        words!((*E).base, y_pos, y_subpos += words!((*E), var_02, var_01));
        words!(
            (*E),
            var_02,
            var_01 += (if (((*E).base.y_pos) as i16).wrapping_sub(((*E).var_00) as i16) < 0 {
                g_word_A89AA0 as i32
            } else {
                -(g_word_A89AA0 as i32)
            }) as u32
        );
        (*E).var_B = (*E).var_B.wrapping_sub(1);
        if (*E).var_B == 0 {
            (*E).var_A = ((0xa89c69i32) as u16);
            (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
            let t: i32 = u16::swap_bytes((*E).base.palette_index) as i32;
            let mut v9: u16 = (16 * t + 256) as u16;
            for _ in 0..=16 {
                word!(game.target_palettes[(v9 >> 1)] = 0x7fff);
                v9 = (v9).wrapping_add(2);
            }
        }
    };
    [11050259] WreckedShipGhost_Func_6 [] {
        let E = Get_WreckedShipGhost(k as i32, game);
        let wsgt_var_B: u16 = (*E).var_B;
        if wsgt_var_B == 0 || {
            let mut v3: u16 = (wsgt_var_B).wrapping_sub(1);
            (*E).var_B = v3;
            (*E).var_B == 0
        } {
            (*E).var_B = 1;
            (*E).var_C = 2;
            (*E).var_A = ((0xa89d36i32) as u16);
        }
    };
    [11050294] WreckedShipGhost_Func_7 [] {
        let mut v2: i16 = 0;
        let E = Get_WreckedShipGhost(k as i32, game);

        let skip_reposition: bool =
            if ((word!(game.samus_x_pos)) as i16).wrapping_sub(((*E).var_07) as i16) < 0
                || ((word!(game.samus_x_pos)) as i16).wrapping_sub(((*E).var_08) as i16) >= 0
                || {
                    v2 = *((ram_ptr!(game, u16; 0xafa) as *mut u8).offset(k as i32 as isize)
                        as *mut u16) as i16;
                    (v2).wrapping_sub(((*E).var_0B) as i16) < 0
                }
                || (v2).wrapping_sub(((*E).var_0C) as i16) >= 0
            {
                (*E).var_0D = g_word_A89A9E;
                let mut v4: i16 = 0;
                if ((word!(game.samus_x_pos)) as i16).wrapping_sub(((*E).var_06) as i16) >= 0 {
                    v4 = (if word!(game.samus_x_pos) == (*E).var_06 {
                        4
                    } else {
                        8
                    }) as i16;
                }
                if v4 as i32 == (*E).var_05 as i32 {
                    let mut v7: i16 = 0;
                    let mut v9: i16 =
                        ((word!(game.samus_y_pos)) as i16).wrapping_sub(((*E).var_0A) as i16);
                    if v9 >= 0 {
                        v7 = (if v9 != 0 { 24 } else { 12 }) as i16;
                    }
                    if v7 as i32 == (*E).var_09 as i32 {
                        let v10: u16 = ((*E).var_0E).wrapping_sub(1);
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
                let mut v3: u16 = ((*E).var_0D).wrapping_sub(1);
                (*E).var_0D = v3;
                !(v3 == 0)
                    || ({
                        (*E).var_05 = 4;
                        (*E).var_09 = 12;
                        false
                    })
            };
        if skip_reposition {
        } else {
            (*E).var_A = ((0xa89b42i32) as u16);
            (*E).var_0D = g_word_A89A9E;
            (*E).var_0E = g_word_A89A9C;
            let v6: i32 = (((*E).var_09).wrapping_add((*E).var_05)) as i32 >> 1;
            (*E).base.x_pos = (g_word_A89AA8[v6 as usize]).wrapping_add(
                *((ram_ptr!(game, u16; 0xaf6) as *mut u8).offset(k as i32 as isize) as *mut u16),
            );
            (*E).base.y_pos = (g_word_A89AA8[(v6 + 1i32) as usize]).wrapping_add(
                *((ram_ptr!(game, u16; 0xafa) as *mut u8).offset(k as i32 as isize) as *mut u16),
            );
            return;
        }
        let v11: u16 = word!(game.samus_x_pos);
        (*E).var_06 = word!(game.samus_x_pos);
        (*E).var_07 = (v11).wrapping_sub(g_word_A89D32);
        (*E).var_08 = (g_word_A89D32).wrapping_add(word!(game.samus_x_pos));
        let v13: u16 = word!(game.samus_y_pos);
        (*E).var_0A = word!(game.samus_y_pos);
        (*E).var_0B = (v13).wrapping_sub(g_word_A89D34);
        (*E).var_0C = (g_word_A89D34).wrapping_add(word!(game.samus_y_pos));
    };
}
pub unsafe fn WreckedShipGhost_Func_3(k: u16, game: &mut Game) {
    let E = Get_WreckedShipGhost(k as i32, game);
    let wsgt_var_B: u16 = (*E).var_B;
    let make_visible: bool = (wsgt_var_B == 0)
        || ({
            let mut v3: u16 = (wsgt_var_B).wrapping_sub(1);
            (*E).var_B = v3;
            (v3 == 0)
                && ({
                    let mut wsgt_var_C: u16 = (*E).var_C;
                    let mut v5: i16 = g_word_A89ACC[(wsgt_var_C >> 1) as usize] as i16;
                    if v5 < 0 {
                        set_fields! {[(*E)], 0; var_B, var_C}
                        return;
                    }
                    (*E).var_B = v5 as u16;
                    (*E).var_C = (wsgt_var_C).wrapping_add(2);
                    ((wsgt_var_C & 2) == 0)
                })
        });
    if make_visible {
        (*E).base.properties =
            ((*E).base.properties as i32 & !(kEnemyProps_Invisible as i32)) as u16;
    };
}
pub unsafe fn WreckedShipGhost_Func_8(game: &mut Game) -> u16 {
    let mut v0: u16 = 0;
    if word!(game.door_transition_flag_enemies) == 0 {
        let E = Get_WreckedShipGhost(word!(game.cur_enemy_index) as i32, game);
        let v2: u16 = u16::swap_bytes((*E).base.palette_index);
        let mut v3: u16 = (16 * v2 as i32 + 256) as u16;
        let r20: u16 = (16 * v2 as i32 + 288) as u16;
        do_until! { (((v3) as i16).wrapping_sub((r20) as i16) >= 0);
            let v4: i32 = v3 as i32 >> 1;
            if word!(game.target_palettes[v4]) != word!(game.palette_buffer[v4]) {
                let mut r18: u16 = (word!(game.target_palettes[v4])) & (0x1fu16);
                let v5: u16 = (word!(game.palette_buffer[v4])) & (0x1fu16);
                if v5 != r18 {
                    let mut v6: u16 = if ((v5) as i16).wrapping_sub((r18) as i16) >= 0 {
                        (v5).wrapping_sub(1)
                    } else {
                        (v5).wrapping_add(1)
                    };
                    r18 = v6;
                    word!(game.palette_buffer[v4] = (v6 | word!(game.palette_buffer[v4]) & 0xffe0));
                    v0 = v0.wrapping_add(1);
                }
                r18 = (word!(game.target_palettes[v4])) & (0x3e0u16);
                let v7: u16 = (word!(game.palette_buffer[v4])) & (0x3e0u16);
                if v7 != r18 {
                    let mut v8: u16 = if ((v7) as i16).wrapping_sub((r18) as i16) >= 0 {
                        (v7).wrapping_sub(32)
                    } else {
                        (v7).wrapping_add(32)
                    };
                    word!(game.palette_buffer[v4] = (v8 | word!(game.palette_buffer[v4]) & 0xfc1f));
                    v0 = v0.wrapping_add(1);
                }
                r18 = (word!(game.target_palettes[v4])) & (0x7c00u16);
                let v9: u16 = (word!(game.palette_buffer[v4])) & (0x7c00u16);
                if v9 != r18 {
                    let mut v10: u16 = if ((v9) as i16).wrapping_sub((r18) as i16) >= 0 {
                        (v9).wrapping_sub(1024)
                    } else {
                        (v9).wrapping_add(1024)
                    };
                    r18 = v10;
                    word!(
                        game.palette_buffer[v4] = (v10 | word!(game.palette_buffer[v4]) & 0x83ff)
                    );
                    v0 = v0.wrapping_add(1);
                }
            }
            v3 = (v3).wrapping_add(2);
        }
    }
    v0
}
crate::game::callback_dispatch! {
    pub unsafe fn CallYappingMawFunc(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    @prefix { let E = Get_YappingMaw(word!(game.cur_enemy_index) as i32, game); };
    YappingMaw_Init [] {
        (*E).var_0C = (*E).base.x_pos;
        (*E).var_0D = (*E).base.y_pos;
        set_fields! {[(*E)], 0; var_00, var_01, var_02, var_03, var_04, var_05, var_06, var_07, var_30}
        (*E).var_F = (*E).parameter_1;
        (*E).var_E = 64;
        (*E).base.current_instruction = addr_kYappingMaw_Ilist_9F6F as u16;
        let mut R36: u16 = 57;
        let mut R34: u16 = 8;
        if (*E).parameter_2 == 0 {
            (*E).base.current_instruction = addr_kYappingMaw_Ilist_9FC7 as u16;
            R36 = 56;
            R34 = -8i32 as u16;
        }
        (*E).base.instruction_timer = 1;
        (*E).base.timer = 0;
        (*E).var_A = ((0xa8a235i32) as u16);
        (*E).var_45 = ((*E).base.palette_index) & (0xe00u16);
        let mut v1: i16 = 3;
        (*E).var_44 = 3;
        do_until! { (v1 < 0);
    SpawnEprojWithGfx(
        v1 as u16,
        word!(game.cur_enemy_index),
        addr_kEproj_YappingMawsBody as u16,
        game,
    );
    v1 = (((*E).var_44) as i16).wrapping_sub(1);
    (*E).var_44 = v1 as u16;
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
    };
    YappingMaw_Main [] {
        (*E).var_35 = (*E).var_35.wrapping_sub(1);
        (*E).var_36 = CheckIfEnemyIsOnScreen(game);
        CallYappingMawFunc((((*E).var_A) as u32) | (0xa80000u32), game);
        YappingMaw_Func_15(game);
        YappingMaw_Func_14(game);
        YappingMaw_Func_13(game);
        YappingMaw_Func_12(game);
    };
    [11051573] YappingMaw_Func_1 [] {
        let pair: PairU16 = EnemyFunc_ACA8(
            Point16U {
                x: (*E).base.x_pos,
                y: (*E).base.y_pos,
            },
            Point16U {
                x: word!(game.samus_x_pos),
                y: word!(game.samus_y_pos),
            },
            game,
        );
        let v2: u16 = Abs16(pair.k);
        (*E).var_08 = v2;
        if v2 as i32 - 32i32 & 0x8000 != 0 {
            (*E).var_35 = 48;
        } else if ((v2) as i16).wrapping_sub(((*E).var_F) as i16) < 0 {
            if (*E).var_08 as i32 - 64i32 & 0x8000 == 0 {
                (*E).var_08 = 64;
            }
            (*E).var_0A = pair.j;
            (*E).var_A = ((0xa8a28ci32) as u16);
        }
    };
    [11051660] YappingMaw_Func_2 [] {
        set_fields! {[(*E)], 0; var_B, var_C, var_D}
        (*E).var_09 = ((*E).var_08 >> 1);
        let v1: u16 = ((64u8).wrapping_sub(((*E).var_0A) as u8)) as u16;
        (*E).var_0B = v1;
        if v1 as i32 - 128i32 & 0x8000 != 0 {
            (*E).var_2F = 0;
        } else {
            (*E).var_2F = 1;
        }
        YappingMaw_Func_3(game);
        let v2: u16 = (2 * (((((*E).var_0A) as u8).wrapping_add(16)) as i32 >> 5)) as u16;
        (*E).var_34 = v2;
        enemy_instruction! {[(*E).base]; instruction = crate::rtl::rom_table_word!(game, 0xa8a097, (v2 >> 1) as isize), timer = 1}
        (*E).base.timer = 0;
        (*E).var_32 = *((rom_ptr!(game, u16; 0xa8a0a7) as *mut u8)
            .offset(((2u16).wrapping_mul(v2)) as i32 as isize) as *mut u16);
        (*E).var_33 = *((rom_ptr!(game, u16; 0xa8a0a9) as *mut u8)
            .offset(((2u16).wrapping_mul(v2)) as i32 as isize) as *mut u16);
        (*E).var_A = ((0xa8a445i32) as u16);
    };
    YappingMaw_Func_3 [] {
        (*E).var_0E = YappingMaw_Func_16(0x80, (*E).var_09, game);
        (*E).var_0F = YappingMaw_Func_17(0x80, ((*E).var_09 >> 1), game);
    };
    YappingMaw_Func_4 [] {
        let varE32: u16 = (*E).var_00;
        (*E).var_10 = YappingMaw_Func_16(0, varE32, game);
        (*E).var_11 = YappingMaw_Func_17(0, varE32, game);
        (*E).var_27 = (YappingMaw_Func_16((*E).var_0B, varE32, game)).wrapping_sub((*E).var_10);
        (*E).var_28 = (YappingMaw_Func_17((*E).var_0B, varE32, game)).wrapping_sub((*E).var_11);
    };
    YappingMaw_Func_5 [] {
        let varE32: u16 = (*E).var_02;
        (*E).var_10 = YappingMaw_Func_16(0, varE32, game);
        (*E).var_11 = YappingMaw_Func_17(0, varE32, game);
        (*E).var_29 = (YappingMaw_Func_16((*E).var_0B, varE32, game)).wrapping_sub((*E).var_10);
        (*E).var_2A = (YappingMaw_Func_17((*E).var_0B, varE32, game)).wrapping_sub((*E).var_11);
    };
    YappingMaw_Func_6 [] {
        let varE32: u16 = (*E).var_04;
        (*E).var_10 = YappingMaw_Func_16(0, varE32, game);
        (*E).var_11 = YappingMaw_Func_17(0, varE32, game);
        (*E).var_2B = (YappingMaw_Func_16((*E).var_0B, varE32, game)).wrapping_sub((*E).var_10);
        (*E).var_2C = (YappingMaw_Func_17((*E).var_0B, varE32, game)).wrapping_sub((*E).var_11);
    };
    YappingMaw_Func_7 [] {
        let varE32: u16 = (*E).var_06;
        (*E).var_10 = YappingMaw_Func_16(0, varE32, game);
        (*E).var_11 = YappingMaw_Func_17(0, varE32, game);
        (*E).var_2D = (YappingMaw_Func_16((*E).var_0B, varE32, game)).wrapping_sub((*E).var_10);
        (*E).var_2E = (YappingMaw_Func_17((*E).var_0B, varE32, game)).wrapping_sub((*E).var_11);
    };
    [11052101] YappingMaw_Func_8 [] {
        let r20: u16 = ((*E).var_B >> 2);
        if (*E).var_2F != 0 {
            (*E).var_21 = (128u16).wrapping_add(r20);
            (*E).var_22 = (128 + r20 as i32 * 2) as u16;
            (*E).var_23 = (128 + r20 as i32 * 3) as u16;
            (*E).var_24 = (128 + r20 as i32 * 4) as u16;
        } else {
            (*E).var_21 = (128u16).wrapping_sub(r20);
            (*E).var_22 = (128 - r20 as i32 * 2) as u16;
            (*E).var_23 = (128 - r20 as i32 * 3) as u16;
            (*E).var_24 = (128 - r20 as i32 * 4) as u16;
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
        YappingMaw_Func_9(word!(game.cur_enemy_index), game);
        if (*E).var_B & 0x8000 == 0 {
            if (*E).var_B as i32 - 128i32 & 0x8000 == 0 {
                (*E).var_B = 128;
                (*E).var_C = 0;
                (*E).var_D = ((*E).var_D).wrapping_add(4);
            }
        } else {
            (*E).var_A = ((0xa8a68ai32) as u16);
            (*E).var_35 = 48;
            if (*E).var_34 == 4 {
                (*E).base.current_instruction = addr_kYappingMaw_Ilist_A01F as u16;
            } else if (*E).var_34 == 12 {
                (*E).base.current_instruction = addr_kYappingMaw_Ilist_A03D as u16;
            } else {
                (*E).base.current_instruction = addr_off_A8A025 as u16;
            }
            if (*E).parameter_2 == 0 {
                if (*E).var_34 == 4 {
                    (*E).base.current_instruction = addr_kYappingMaw_Ilist_A05B as u16;
                } else {
                    if (*E).var_34 != 12 {
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
    };
    [11052682] YappingMaw_Func_11 [] {
        if (*E).var_30 != 0 {
            YappingMaw_Func_10(game);
        }
        (*E).var_E = (*E).var_E.wrapping_sub(1);
        if ((*E).var_E & 0x8000) != 0 && word!(game.samus_input_handler) as i32 != 0x90e91d & 0xffff {
            word!(game.samus_input_handler = ((0x90e913i32) as u16));
            (*E).var_30 = 0;
            (*E).var_35 = 48;
            (*E).var_E = 64;
            (*E).var_A = ((0xa8a235i32) as u16);
        }
    };
    YappingMaw_Func_12 [] {
        let v1: i32 = (*E).var_40 as i32 >> 1;
        word! {game; eproj_x_pos[v1] = (*E).var_0C; eproj_y_pos[v1] = (*E).var_0D; }
    };
    YappingMaw_Func_13 [] {
        let v1: i32 = (*E).var_41 as i32 >> 1;
        word!(game.eproj_x_pos[v1] = ((*E).var_00).wrapping_add((*E).var_0C));
        word!(game.eproj_y_pos[v1] = ((*E).var_01).wrapping_add((*E).var_0D));
    };
    YappingMaw_Func_14 [] {
        let v1: i32 = (*E).var_42 as i32 >> 1;
        word!(game.eproj_x_pos[v1] = ((*E).var_02).wrapping_add((*E).var_0C));
        word!(game.eproj_y_pos[v1] = ((*E).var_03).wrapping_add((*E).var_0D));
    };
    YappingMaw_Func_15 [] {
        let v1: i32 = (*E).var_43 as i32 >> 1;
        word!(game.eproj_x_pos[v1] = ((*E).var_04).wrapping_add((*E).var_0C));
        word!(game.eproj_y_pos[v1] = ((*E).var_05).wrapping_add((*E).var_0D));
    };
    YappingMaw_Touch [] {
        if ((*E).var_35 & 0x8000) != 0 && (*E).var_30 == 0 {
            (*E).var_35 = 0;
            (*E).var_30 = 1;
            word!(game.samus_input_handler = ((0x90e90ei32) as u16));
        }
    };
    YappingMaw_Func_19 [] {
        let r18: u16 = (*E).var_45;
        let r20: u16 = (*E).var_46;
        word!(
            game.sprite_palettes[(r20 >> 1)] = (r18 | word!(game.sprite_palettes[(r20 >> 1)]) & 0xf1ff)
        );
        if (*E).base.frozen_timer != 0 {
            word!(
                game.sprite_palettes[(r20 >> 1)] =
                    (word!(game.sprite_palettes[(r20 >> 1)]) & 0xf1ff | 0xc00)
            );
            let frozen_timer: u16 = (*E).base.frozen_timer;
            if frozen_timer as i32 - 90i32 & 0x8000 != 0 && (frozen_timer & 2) == 0 {
                word!(
                    game.sprite_palettes[(r20 >> 1)] =
                        (r18 | word!(game.sprite_palettes[(r20 >> 1)]) & 0xf1ff)
                );
            }
        }
    };
}
pub unsafe fn YappingMaw_Func_9(k: u16, game: &mut Game) {
    let E = Get_YappingMaw(k as i32, game);
    let v2: i32 = (*E).var_D as i32 >> 1;
    words!(
        (*E),
        var_B,
        var_C += words!((
            crate::rtl::rom_table_word!(game, 0xa2838f, (v2 + 1) as isize),
            crate::rtl::rom_table_word!(game, 0xa2838f, v2 as isize)
        ))
    );
    (*E).var_D = ((*E).var_D).wrapping_add(8);
}
pub unsafe fn YappingMaw_Func_10(game: &mut Game) {
    CallSomeSamusCode(3, game);
    let E = Get_YappingMaw(word!(game.cur_enemy_index) as i32, game);
    word!(game.samus_x_pos = ((*E).var_32).wrapping_add((*E).base.x_pos));
    word!(game.samus_y_pos = ((*E).var_33).wrapping_add((*E).base.y_pos));
    EnemyFunc_B7A1(game);
}
pub unsafe fn YappingMaw_Func_16(a: u16, varE32: u16, game: &mut Game) -> u16 {
    YappingMaw_Func_17((a).wrapping_sub(64), varE32, game)
}
pub unsafe fn YappingMaw_Func_17(a: u16, varE32: u16, game: &mut Game) -> u16 {
    let mut r20: u16 = 0;
    let mut r22: u16 = 0;
    let mut v1: i16 =
        crate::rtl::rom_table_word!(game, 0xa0b1c3, -(a as i32) as u8 as isize) as i16;
    if v1 < 0 {
        v1 = -(v1 as i32) as i16;
        r20 = r20.wrapping_add(1);
    }
    WriteReg(WRMPYA as u16, byte!(v1; 1), game);
    if varE32 as u8 != 0 {
        let prod: u16 = Mult8x8(byte!(v1; 1), varE32 as u8);
        r22 = (2 * ((prod as i32 & 0xff00) >> 8)) as u16;
        if r20 != 0 {
            r22 = (2 * (-(prod as i32) as u16 as i32 >> 8) | 0xff00) as u16;
        }
    }
    r22
}
pub unsafe fn YappingMaw_Func_18(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1;
    let E = Get_YappingMaw(word!(game.cur_enemy_index) as i32, game);
    word!(game.eproj_gfx_idx[v1] = ((*E).var_45 | word!(game.eproj_gfx_idx[v1]) & 0xf1ff));
    if (*E).base.frozen_timer != 0 {
        word!(game.eproj_gfx_idx[v1] = (word!(game.eproj_gfx_idx[v1]) & 0xf1ff | 0xc00));
        let frozen_timer: u16 = (*E).base.frozen_timer;
        if frozen_timer as i32 - 90i32 & 0x8000 != 0 && (frozen_timer & 2) == 0 {
            word!(game.eproj_gfx_idx[v1] = ((*E).var_45 | word!(game.eproj_gfx_idx[v1]) & 0xf1ff));
        }
    }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallKagoFunc(ea: u32, k: u16, game: &mut Game) => ea;
    call (k, game); default { Unreachable(game); };
    @functions [pub unsafe fn (k: u16, game: &mut Game)];
    [11053947] Kago_Func_1 [] {
        (*Get_Kago(k as i32, game)).var_A = ((0xa8ab81i32) as u16);
    };
    @external {
        11053953 => [()];
    }
}
static g_off_A8AF4F: [Func_V; 3] = callback_table! { nonnull Func_V;
    NorfairLavaMan_Func_1, NorfairLavaMan_Func_2, NorfairLavaMan_Func_3,
};
pub unsafe fn NorfairLavaMan_Func_5(k: u16, game: &mut Game) {
    (*(ram_ptr!(game, LongPtr; 0x178c))).addr = ((0xa8b0b2i32) as u16);
    *(&raw mut (*(ram_ptr!(game, LongPtr; 0x178c))).bank as *mut u16) = 168;
    word!(
        game.variables_for_enemy_graphics_drawn_hook[0] =
            (((((*Get_NorfairLavaMan(k as i32, game)).base.palette_index) & (0xe00u16)) as i32
                >> 4)
                + 256) as u16
    );
    word!(game.variables_for_enemy_graphics_drawn_hook[2] = 8);
    word!(game.variables_for_enemy_graphics_drawn_hook[1] = 0);
}
pub unsafe fn NorfairLavaMan_Func_6(game: &mut Game) {
    if word!(game.door_transition_flag_enemies) == 0 && {
        word!(game.variables_for_enemy_graphics_drawn_hook[2] -= 1, reread) == 0
    } {
        word!(game.variables_for_enemy_graphics_drawn_hook[2] = 8);
        word!(game.variables_for_enemy_graphics_drawn_hook[1] += 1);
        let v0: i32 = (32 * (word!(game.variables_for_enemy_graphics_drawn_hook[1]) as i32 & 3))
            as u16 as i32
            >> 1;
        let r18: u16 = crate::rtl::rom_table_word!(game, 0xa8ac1c, (v0 + 9) as isize);
        let r20: u16 = crate::rtl::rom_table_word!(game, 0xa8ac1c, (v0 + 10) as isize);
        let r22: u16 = crate::rtl::rom_table_word!(game, 0xa8ac1c, (v0 + 11) as isize);
        let r24: u16 = crate::rtl::rom_table_word!(game, 0xa8ac1c, (v0 + 12) as isize);
        let v1: i32 = word!(game.variables_for_enemy_graphics_drawn_hook[0]) as i32 >> 1;
        word! {game; palette_buffer[(v1 + 9)] = r18; palette_buffer[(v1 + 10)] = r20;
        palette_buffer[(v1 + 11)] = r22; palette_buffer[(v1 + 12)] = r24; }
    }
}
pub unsafe fn NorfairLavaMan_Func_12(k: u16, game: &mut Game) {
    let mut v3: i32 = 0;
    let E = Get_NorfairLavaMan(k as i32, game);
    if -((*E).var_E as i32) - 108i32 & 0x8000 == 0 {
        printf(c"Y undefined %d!\n".as_ptr(), -((*E).var_E as i16 as i32));
    }
    let nlmn_var_B: u16 = (*E).var_B;
    let advance_lava_phase: bool = if -((*E).var_E as i32) - 108i32 & 0x8000 == 0 || {
        v3 = nlmn_var_B as i32 >> 1;
        (((*E).base.y_pos as i32
            - crate::rtl::rom_table_word!(game, 0xa8af79, v3 as isize) as i32
            - word!(game.samus_y_pos) as i32) as i16)
            < 0
    } {
        (*E).var_F = ((0xa8b291i32) as u16);
        (*E).var_0C = 1;
        if ((-((*E).var_E as i32)
            - crate::rtl::rom_table_word!(game, 0xa8af55, (nlmn_var_B >> 1) as isize) as i32)
            as i16)
            < 0
        {
            return;
        }
        true
    } else {
        (((-((*E).var_E as i32) - crate::rtl::rom_table_word!(game, 0xa8af55, v3 as isize) as i32)
            as i16)
            >= 0)
    };
    if advance_lava_phase {
        (*E).var_B = (*E).var_B.wrapping_add(1);
        (*E).var_B = (*E).var_B.wrapping_add(1);
        (*E).base.y_pos = ((*E).base.y_pos).wrapping_add(8);
        (*E).var_D = crate::rtl::rom_table_word!(game, 0xa8af67, ((*E).var_B >> 1) as isize);
        NorfairLavaMan_Func_20(game);
    };
}
pub fn sub_A8B291(k: u16) {}
pub unsafe fn NorfairLavaMan_Func_14(k: u16, game: &mut Game) {
    let E = Get_NorfairLavaMan(k as i32, game);
    let mut nlmn_var_E: i16 = (*E).var_E as i16;
    if nlmn_var_E >= 0 {
        (*E).var_0D = 1;
        (*gEnemySpawnData(k as i32, game).offset(31)).field_4 = 0;
        (*E).var_F = ((0xa8b1ddi32) as u16);
    } else if ((-(nlmn_var_E as i32)
        - crate::rtl::rom_table_word!(
            game,
            0xa8af55,
            ((((*E).var_B).wrapping_sub(2)) >> 1) as isize
        ) as i32) as i16)
        < 0
    {
        (*E).var_B = ((*E).var_B).wrapping_sub(2);
        (*E).base.y_pos = ((*E).base.y_pos).wrapping_sub(8);
        (*E).var_D = crate::rtl::rom_table_word!(game, 0xa8af67, ((*E).var_B >> 1) as isize);
        NorfairLavaMan_Func_20(game);
    }
}
pub unsafe fn NorfairLavaMan_Func_19(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    let v2: u16 = (word!(game.enemy_drawing_queue[(v1 + 61)]) as i32
        - (*Get_NorfairLavaMan(k as i32, game)).base.y_pos as i32
        + 2) as u16;
    word!(game.enemy_drawing_queue[(v1 + 64)] = v2);
    if v2 as i32 - 8i32 & 0x8000 != 0 {
        word!(game.enemy_drawing_queue[(v1 + 64)] = 8);
    }
}
pub fn Beetom_Instr_1(k: u16, jp: *const u16) -> *const u16 {
    jp
}
pub use crate::game::sm_a3::Sidehopper_Func_2 as Beetom_Func_2;

unsafe fn begin_beetom_hop(high: bool, state: u16, animation: u16, game: &mut Game) {
    let E = Get_Beetom(word!(game.cur_enemy_index) as i32, game);
    (*E).var_B = if high { (*E).var_03 } else { (*E).var_02 };
    (*E).var_C = state;
    (*E).var_05 = 0;
    (*E).var_00 = animation;
    Beetom_Func_1(game);
}
unsafe fn walk_beetom<const LEFT: bool>(game: &mut Game) {
    let E = Get_Beetom(word!(game.cur_enemy_index) as i32, game);
    (*E).var_D = (*E).var_D.wrapping_sub(1);
    if ((*E).var_D & 0x8000) != 0 {
        (*E).var_D = 64;
        (*E).var_C = ((0xa8b814i32) as u16);
    } else {
        (*E).base.x_pos = (*E).base.x_pos.wrapping_add(if LEFT { 0xfff8 } else { 8 });
        if Enemy_MoveDown(word!(game.cur_enemy_index), (1i16 as i32) << 16, game) != 0 {
            (*E).base.x_pos = (*E).base.x_pos.wrapping_add(if LEFT { 8 } else { 0xfff8 });
            if Enemy_MoveRight_IgnoreSlopes(
                word!(game.cur_enemy_index),
                if LEFT { -16384 } else { 16384 },
                game,
            ) != 0
            {
                (*E).var_C = if LEFT { 0xb873 } else { 0xb85f };
            }
        } else {
            (*E).var_C = if LEFT { 0xb873 } else { 0xb85f };
            (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
            (*E).base.x_pos = (*E).base.x_pos.wrapping_add(if LEFT { 8 } else { 0xfff8 });
        }
    }
}
// Preserve the captured-value versus post-store-read variants of phase tests.
macro_rules! advance_beetom_jump_descent {
    ($enemy_index:ident, $game:ident, $frame_step:literal) => {
        let E = Get_Beetom($enemy_index as i32, $game);
        let v2: i32 = 8 * (*E).var_B as i32 >> 1;
        if Enemy_MoveDown(
            $enemy_index,
            *($crate::rtl::rom_ptr!($game, u32; 0xa0cbc7))
                .offset((v2 >> 1) as isize) as i32,
            $game,
        ) != 0
        {
            (*E).var_C = ((0xa8b814i32) as u16);
        } else {
            (*E).var_B = ((*E).var_B).wrapping_add($frame_step);
            if (*E).var_B as i32 - 64i32 & 0x8000 == 0 {
                (*E).var_B = 64;
            }
        }
    };
}

pub unsafe fn Beetom_Func_23(k: u16, game: &mut Game) {
    advance_beetom_jump_descent!(k, game, 4);
}
pub unsafe fn Beetom_Func_27(k: u16, game: &mut Game) {
    advance_beetom_jump_descent!(k, game, 5);
}

unsafe fn advance_beetom_jump<const FALLING: bool, const STEP: u16, const CAPTURED: bool>(
    k: u16,
    game: &mut Game,
) -> *mut Enemy_Beetom {
    let E = Get_Beetom(k as i32, game);
    let index: i32 = 8 * (*E).var_B as i32 >> 1;
    let index = (index + if FALLING { 0 } else { 2 }) >> 1;
    if Enemy_MoveDown(
        k,
        *(rom_ptr!(game, u32; 0xa0cbc7)).offset(index as isize) as i32,
        game,
    ) != 0
    {
        (*E).var_C = if FALLING { 0xb814 } else { 0xb97a };
    } else {
        let next = if FALLING {
            (*E).var_B.wrapping_add(STEP)
        } else {
            (*E).var_B.wrapping_sub(STEP)
        };
        (*E).var_B = next;
        let tested = if CAPTURED { next } else { (*E).var_B };
        if FALLING {
            if tested as i32 - 64i32 & 0x8000 == 0 {
                (*E).var_B = 64;
            }
        } else if tested & 0x8000 != 0 {
            (*E).var_B = 0;
            (*E).var_05 = 1;
        }
    }
    E
}
crate::forward_calls! { pub unsafe fn (k: u16, game: &mut Game);
    Beetom_Func_22 => { crate::verify_probe::hit("hop.Beetom_Func_22"); advance_beetom_jump::<false, 4, false>(k, game) };
    Beetom_Func_26 => { crate::verify_probe::hit("hop.Beetom_Func_26"); advance_beetom_jump::<false, 5, false>(k, game) };
    Beetom_Func_30 => { crate::verify_probe::hit("hop.Beetom_Func_30"); advance_beetom_jump::<false, 5, true>(k, game) };
    Beetom_Func_31 => { crate::verify_probe::hit("hop.Beetom_Func_31"); advance_beetom_jump::<true, 3, true>(k, game) };
}

unsafe fn move_beetom_hop(
    rise: unsafe fn(u16, &mut Game),
    fall: unsafe fn(u16, &mut Game),
    horizontal: i32,
    game: &mut Game,
) {
    let E = Get_Beetom(word!(game.cur_enemy_index) as i32, game);
    if (*E).var_05 != 0 {
        fall(word!(game.cur_enemy_index), game);
    } else {
        rise(word!(game.cur_enemy_index), game);
    }
    if Enemy_MoveRight_IgnoreSlopes(word!(game.cur_enemy_index), horizontal, game) != 0 {
        (*E).var_09 = ((*E).var_09) ^ (1u16);
        (*E).var_C = ((0xa8b97ai32) as u16);
    }
}
pub unsafe fn Beetom_Func_37(k: u16, game: &mut Game) {
    crate::verify_probe::hit("hop.Beetom_Func_37");
    let E = advance_beetom_jump::<true, 1, true>(k, game);
    let v4: u16 = (if (*E).var_09 != 0 { -2i32 } else { 2 }) as u16;
    if Enemy_MoveRight_IgnoreSlopes(k, (words!((v4, 0u16))) as i32, game) != 0 {
        (*E).var_09 = ((*E).var_09) ^ (1u16);
        (*E).var_C = ((0xa8b97ai32) as u16);
    }
}

crate::game::callback_dispatch! {
    pub unsafe fn CallBeetomFunc(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [11057172] Beetom_Func_3 [] {
        if IsSamusWithinEnemy_X(word!(game.cur_enemy_index), 0x60, game) != 0 {
            (*Get_Beetom(word!(game.cur_enemy_index) as i32, game)).var_C = ((0xa8b90fi32) as u16);
        } else {
            (*Get_Beetom(word!(game.cur_enemy_index) as i32, game)).var_C = ((0xa8b82fi32) as u16);
        };
    };
    [11057199] Beetom_Func_4 [] {
        NextRandom(game);
        let E = Get_Beetom(word!(game.cur_enemy_index) as i32, game);
        (*E).var_C = crate::rtl::rom_table_word!(
            game,
            0xa8b74e,
            (word!(game.random_number) as i32 & 7) as isize
        );
        (*E).var_09 = (word!(game.random_number)) & (1u16);
    };
    [11058589] Beetom_Func_35 [] {
        if Enemy_MoveDown(
            word!(game.cur_enemy_index),
            ((3u32) << 16 | 0u16 as u32) as i32,
            game,
        ) != 0
        {
            let E = Get_Beetom(word!(game.cur_enemy_index) as i32, game);
            if (*E).var_09 != 0 {
                (*E).var_C = ((0xa8b873i32) as u16);
            } else {
                (*E).var_C = ((0xa8b85fi32) as u16);
            }
        }
    };
    [11058629] Beetom_Func_36 [] {
        Beetom_Func_37(word!(game.cur_enemy_index), game);
    };
    @functions [pub unsafe fn (game: &mut Game)];
    @prefix { let E = Get_Beetom(word!(game.cur_enemy_index) as i32, game); };
    Beetom_Func_1 [] {
        enemy_instruction! {[(*E).base]; instruction = (*E).var_00, timer = 1}
        (*E).base.timer = 0;
    };
    Beetom_Init [] {
        set_fields! {[(*E)], 0; var_D, var_C, var_05, var_08, var_A}
        (*E).var_E = 64;
        (*E).var_F = word!(game.joypad1_lastkeys);
        word!(game.random_number = 23);
        (*E).var_02 = Beetom_Func_2(12288, 4, game);
        (*E).var_03 = Beetom_Func_2(0x4000, 5, game);
        (*E).var_04 = Beetom_Func_2(12288, 3, game);
        (*E).var_00 = addr_kBeetom_Ilist_B6F2 as u16;
        if (GetSamusEnemyDelta_X(word!(game.cur_enemy_index), game) & 0x8000) != 0 {
            (*E).var_00 = addr_kBeetom_Ilist_B696 as u16;
        }
        Beetom_Func_1(game);
        (*E).var_C = ((0xa8b814i32) as u16);
    };
    [11057231] Beetom_Func_5 [] {
        (*E).var_D = 32;
        (*E).var_C = ((0xa8b9b2i32) as u16);
    };
    [11057247] Beetom_Func_6 [] {
        (*E).var_C = ((0xa8b9c1i32) as u16);
        (*E).var_00 = addr_kBeetom_Ilist_B696 as u16;
        Beetom_Func_1(game);
    };
    [11057267] Beetom_Func_7 [] {
        (*E).var_C = ((0xa8ba24i32) as u16);
        (*E).var_00 = addr_kBeetom_Ilist_B6F2 as u16;
        Beetom_Func_1(game);
    };
    [11057423] Beetom_Func_12 [] {
        (*E).var_B = (*E).var_04;
        (*E).var_00 = addr_kBeetom_Ilist_B708 as u16;
        (*E).var_C = ((0xa8bc5ai32) as u16);
        (*E).var_09 = 1;
        if (GetSamusEnemyDelta_X(word!(game.cur_enemy_index), game) & 0x8000) != 0 {
            (*E).var_00 = addr_kBeetom_Ilist_B6AC as u16;
            (*E).var_C = ((0xa8bc26i32) as u16);
            (*E).var_09 = 0;
        }
        Beetom_Func_1(game);
        (*E).var_05 = 0;
    };
    [11057490] Beetom_Func_13 [] {
        (*E).var_00 = addr_kBeetom_Ilist_B6CC as u16;
        Beetom_Func_1(game);
        (*E).var_C = ((0xa8bcf8i32) as u16);
    };
    [11057510] Beetom_Func_14 [] {
        (*E).var_00 = addr_kBeetom_Ilist_B728 as u16;
        Beetom_Func_1(game);
        (*E).var_C = ((0xa8bd42i32) as u16);
    };
    [11057530] Beetom_Func_15 [] {
        (*E).var_00 = addr_kBeetom_Ilist_B696 as u16;
        if (*E).var_09 != 0 {
            (*E).var_00 = addr_kBeetom_Ilist_B6F2 as u16;
        }
        Beetom_Func_1(game);
        (*E).var_05 = 0;
        (*E).var_C = ((0xa8bd9di32) as u16);
    };
    [11057570] Beetom_Func_16 [] {
        (*E).var_B = 0;
        (*E).var_C = ((0xa8bdc5i32) as u16);
    };
    [11057586] Beetom_Func_17 [] {
        (*E).var_D = (*E).var_D.wrapping_sub(1);
        if ((*E).var_D & 0x8000) != 0 {
            (*E).var_C = ((0xa8b814i32) as u16);
        }
    };
    Beetom_Func_34 [] {
        if word!(game.joypad1_lastkeys) != (*E).var_F {
            (*E).var_F = word!(game.joypad1_lastkeys);
            (*E).var_E = (*E).var_E.wrapping_sub(1);
        }
    };
    [11058424] Beetom_Func_32 [] {
        if (*E).var_E != 0 {
            (*E).base.x_pos = word!(game.samus_x_pos);
            (*E).base.y_pos = (word!(game.samus_y_pos)).wrapping_sub(4);
            Beetom_Func_34(game);
        } else {
            if Enemy_MoveRight_IgnoreSlopes(
                word!(game.cur_enemy_index),
                ((16u32) << 16 | 0u16 as u32) as i32,
                game,
            ) != 0
            {
                (*E).var_09 = 1;
                Enemy_MoveRight_IgnoreSlopes(
                    word!(game.cur_enemy_index),
                    (words!((-32i32, 0u16))) as i32,
                    game,
                );
            }
            (*E).var_08 = 0;
            (*E).var_C = ((0xa8b9a2i32) as u16);
        };
    };
    [11058498] Beetom_Func_33 [] {
        if (*E).var_E != 0 {
            (*E).base.x_pos = word!(game.samus_x_pos);
            (*E).base.y_pos = (word!(game.samus_y_pos)).wrapping_sub(4);
            Beetom_Func_34(game);
        } else {
            if Enemy_MoveRight_IgnoreSlopes(
                word!(game.cur_enemy_index),
                (words!((-16i32, 0u16))) as i32,
                game,
            ) != 0
            {
                (*E).var_09 = 0;
                Enemy_MoveRight_IgnoreSlopes(
                    word!(game.cur_enemy_index),
                    ((32u32) << 16 | 0u16 as u32) as i32,
                    game,
                );
            }
            (*E).var_08 = 0;
            (*E).var_C = ((0xa8b9a2i32) as u16);
        };
    };
    Beetom_Main [] {
        CallBeetomFunc((((*E).var_C) as u32) | (0xa80000u32), game);
    };
    Beetom_Touch [] {
        if (*E).var_08 == 0 {
            if (*E).var_09 != 0 {
                (*E).var_C = ((0xa8b966i32) as u16);
            } else {
                (*E).var_C = ((0xa8b952i32) as u16);
            }
            (*E).var_08 = 1;
            (*E).var_E = 64;
            (*E).base.layer = 2;
            (*E).var_06 = (word!(game.samus_x_pos)).wrapping_sub((*E).base.x_pos);
            (*E).var_07 = (word!(game.samus_y_pos)).wrapping_sub((*E).base.y_pos);
        }
        let apply_touch_damage: bool = (word!(game.samus_contact_damage_index) != 0)
            || ({
                if (word!(game.random_enemy_counter) & 7) == 7
                    && word!(game.samus_health) as i32 - 30i32 & 0x8000 == 0
                {
                    QueueSfx3_Max6(0x2d, game);
                }
                (((*E).base.frame_counter & 0x3f) == 63)
            });
        if apply_touch_damage {
            Enemy_NormalTouchAI_A8(game);
            word! { game, 0; samus_invincibility_timer, samus_knockback_timer }
        };
    };
    @forward [pub unsafe fn (game: &mut Game)];
    [11057287]
    Beetom_Func_8 => { crate::verify_probe::hit("hop.Beetom_Func_8"); begin_beetom_hop(false, 0xba84, addr_kBeetom_Ilist_B6AC as u16, game) };
    [11057321]
    Beetom_Func_9 => { crate::verify_probe::hit("hop.Beetom_Func_9"); begin_beetom_hop(false, 0xbab7, addr_kBeetom_Ilist_B708 as u16, game) };
    [11057355]
    Beetom_Func_10 => { crate::verify_probe::hit("hop.Beetom_Func_10"); begin_beetom_hop(true, 0xbb55, addr_kBeetom_Ilist_B6AC as u16, game) };
    [11057389]
    Beetom_Func_11 => { crate::verify_probe::hit("hop.Beetom_Func_11"); begin_beetom_hop(true, 0xbb88, addr_kBeetom_Ilist_B708 as u16, game) };
    @forward [pub unsafe fn (game: &mut Game)];
    [11057601]
    Beetom_Func_18 => { crate::verify_probe::hit("hop.Beetom_Func_18"); walk_beetom::<true>(game) };
    [11057700]
    Beetom_Func_19 => { crate::verify_probe::hit("hop.Beetom_Func_19"); walk_beetom::<false>(game) };
    @forward [pub unsafe fn (game: &mut Game)];
    [11057796]
    Beetom_Func_20 => { crate::verify_probe::hit("hop.Beetom_Func_20"); move_beetom_hop(Beetom_Func_22, Beetom_Func_23, -16384, game) };
    [11057847]
    Beetom_Func_21 => { crate::verify_probe::hit("hop.Beetom_Func_21"); move_beetom_hop(Beetom_Func_22, Beetom_Func_23, 16384, game) };
    [11058005]
    Beetom_Func_24 => { crate::verify_probe::hit("hop.Beetom_Func_24"); move_beetom_hop(Beetom_Func_26, Beetom_Func_27, -16384, game) };
    [11058056]
    Beetom_Func_25 => { crate::verify_probe::hit("hop.Beetom_Func_25"); move_beetom_hop(Beetom_Func_26, Beetom_Func_27, 16384, game) };
    [11058214]
    Beetom_Func_28 => { crate::verify_probe::hit("hop.Beetom_Func_28"); move_beetom_hop(Beetom_Func_30, Beetom_Func_31, -196608, game) };
    [11058266]
    Beetom_Func_29 => { crate::verify_probe::hit("hop.Beetom_Func_29"); move_beetom_hop(Beetom_Func_30, Beetom_Func_31, 196608, game) };
}
pub unsafe fn MaridiaFloater_Func_1(k: u16, game: &mut Game) {
    for i in (0i32..=7).rev() {
        SpawnEprojWithGfx(i as u16, k, addr_kEproj_MaridiaFloatersSpikes as u16, game);
    }
}
pub unsafe fn MaridiaFloater_Func_2(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    let v2: u16 = word!(game.enemy_drawing_queue[(v1 + 103)]);
    if v2 as i32 + 0x3e6f & 0x8000 != 0 {
        let mut v3: u16 = ((v2 as i32 - 4i32 + 15997) as u16 >> 1);
        if v3 as i32 - 6i32 & 0x8000 == 0 {
            v3 = 0;
        }
        word!(
            game.enemy_drawing_queue[(v1 + 93)] = ((g_word_A8C277[(v3 >> 1) as usize]) as u16)
                .wrapping_add((*Get_MaridiaFloater(k as i32, game)).base.y_pos,)
        );
    } else {
        let mut v4: u16 = ((v2 as i32 - 4i32 + 15983) as u16 >> 1);
        if v4 as i32 - 6i32 & 0x8000 == 0 {
            v4 = 0;
        }
        word!(
            game.enemy_drawing_queue[(v1 + 93)] = ((g_word_A8C27D[(v4 >> 1) as usize]) as u16)
                .wrapping_add((*Get_MaridiaFloater(k as i32, game)).base.y_pos,)
        );
    };
}
macro_rules! advance_maridia_floater_bob {
    ($enemy:ident, $enemy_index:ident, $game:ident) => {
        let v2: bool = (*$enemy).var_D == 1;
        (*$enemy).var_D = (*$enemy).var_D.wrapping_sub(1);
        let v3: bool = ((*$enemy).var_D & 0x8000) != 0;
        if v2 || v3 {
            (*$enemy).var_D = 5;
            let v4: i32 = $enemy_index as i32 >> 1;
            let v5: u16 = (g_word_A8C1A1[(*$enemy).var_C as usize])
                .wrapping_add(word!($game.enemy_drawing_queue_sizes[v4]));
            word!($game.enemy_drawing_queue[(v4 + 91)] = v5);
            (*$enemy).base.x_pos = v5;
            let mut v6: u16 = ((*$enemy).var_C).wrapping_add(1);
            if (*$enemy).var_C as i32 - 11i32 & 0x8000 == 0 {
                v6 = 0;
            }
            (*$enemy).var_C = v6;
        }
    };
}

pub unsafe fn WreckedShipRobot_Func_1(game: &mut Game) {
    let mut v2: i16 = 0;
    if word!(game.door_transition_flag_enemies) == 0
        && word!(game.wrecked_ship_robot_palanim_timer) != 0
    {
        if word!(game.wrecked_ship_robot_palanim_timer -= 1, reread) == 0 {
            let mut v1: i32 = 0;
            let mut i: u16 = *(ram_ptr!(game, u16; 0xd656));
            loop {
                v1 = i as i32 >> 1;
                v2 = g_word_A8CCC1[v1 as usize];
                if v2 >= 0 {
                    break;
                }
                i = 0;
            }
            let v3: i32 =
                16 * ((word!(game.wrecked_ship_robot_palanim_palindex) as i32 & 0xff00) >> 8) >> 1;
            word!(game.palette_buffer[(v3 + 137)] = v2 as u16);
            word!(game.palette_buffer[(v3 + 138)] = g_word_A8CCC1[(v1 + 1) as usize] as u16);
            word!(game.palette_buffer[(v3 + 139)] = g_word_A8CCC1[(v1 + 2) as usize] as u16);
            word!(game.palette_buffer[(v3 + 140)] = g_word_A8CCC1[(v1 + 3) as usize] as u16);
            word!(game.wrecked_ship_robot_palanim_timer = g_word_A8CCC1[(v1 + 4) as usize] as u16);
            *(ram_ptr!(game, u16; 0xd656)) = (i).wrapping_add(10);
        }
    }
}
pub unsafe fn WreckedShipRobot_Func_2(k: u16, game: &mut Game) {
    let E = Get_WreckedShipRobot(k as i32, game);
    let wsrt_var_B: u16 = (*E).var_B;
    if wsrt_var_B != 0 {
        (*E).var_B = (wsrt_var_B).wrapping_sub(1);
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
    WreckedShipRobot_Func_2(word!(game.cur_enemy_index), game);
    let robot = Get_WreckedShipRobot(word!(game.cur_enemy_index) as i32, game);
    (*robot).var_A = facing;
    if Enemy_MoveRight_IgnoreSlopes(word!(game.cur_enemy_index), (step as i32) << 16, game) != 0 {
        (*robot).var_B = (*robot).var_B.wrapping_add(8);
        return wall_instruction as usize as *const u16;
    }
    if CheckIfEnemyTouchesSamus(word!(game.cur_enemy_index), game) != 0 {
        word! {game; extra_samus_x_subdisplacement = 0; extra_samus_x_displacement = step as u16; }
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
    let ground = Enemy_MoveDown(word!(game.cur_enemy_index), 1 << 16, game) != 0;
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

pub unsafe fn WreckedShipRobot_Instr_7(k: u16, jp: *const u16) -> *const u16 {
    addr_kWreckedShipRobot_Ilist_C92D as i32 as usize as *const u16
}
pub unsafe fn WreckedShipRobot_Instr_10(k: u16, jp: *const u16) -> *const u16 {
    addr_stru_A8C6E9 as i32 as usize as *const u16
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
        let v3: u16 = ((word!(game.random_number) as i32 & 0x1f) + 16) as u16;
        (*E).var_B = v3;
        SpawnEprojWithGfx(v3, word!(game.cur_enemy_index), r48, game);
        return r50 as usize as *const u16;
    }
    jp
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
        let v3: u16 = ((word!(game.random_number) as i32 & 0x1f) + 16) as u16;
        (*E).var_B = v3;
        SpawnEprojWithGfx(
            v3,
            word!(game.cur_enemy_index),
            addr_kEproj_WreckedShipRobotLaserHorizontal as u16,
            game,
        );
        return r50 as usize as *const u16;
    }
    jp
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
        let v3: u16 = ((word!(game.random_number) as i32 & 0x1f) + 16) as u16;
        (*E).var_B = v3;
        SpawnEprojWithGfx(v3, word!(game.cur_enemy_index), r48, game);
        return r50 as usize as *const u16;
    }
    jp
}
crate::game::callback_dispatch! {
    pub unsafe fn CallMaridiaPuffer(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [11065664] MaridiaPuffer_Func_2 [] {
        let v1: u8 = ((CalculateAngleOfSamusFromEnemy(word!(game.cur_enemy_index), game)) as u8)
            .wrapping_sub(64);
        let E = Get_MaridiaPuffer(word!(game.cur_enemy_index) as i32, game);
        (*E).var_01 = v1 as u16;
        (*E).var_02 = v1 as u16;
        (*E).var_A = ((0xa8d963i32) as u16);
        (*E).var_C = 24;
    };
    @functions [pub unsafe fn (game: &mut Game)];
    @prefix { let E = Get_MaridiaPuffer(word!(game.cur_enemy_index) as i32, game); };
    MaridiaPuffer_Init [] {
        set_fields! {[(*E).base]; instruction_timer = 1, timer = 0,
            current_instruction = addr_kMaridiaPuffer_Ilist_D841 as u16, };
        let v1: i32 = ((4u16).wrapping_mul((*E).parameter_1)) as i32 >> 1;
        let v2: u16 = g_word_A8D895[v1 as usize] as u16;
        set_fields! {[(*E)]; var_05 = v2, var_F = v2,
            var_06 = g_word_A8D895[(v1 + 1) as usize] as u16, var_E = 16,
            var_A = ((0xa8d92bi32) as u16),
            var_00 = g_word_A8D885[(*E).parameter_2 as usize] as u16, };
    };
    MaridiaPuffer_Main [] {
        let v1: u16 = ((*E).var_07).wrapping_sub(1);
        (*E).var_07 = v1;
        if v1 == 0 {
            (*E).var_07 = 1;
            (*E).var_03 = 0;
        }
        CallMaridiaPuffer((((*E).var_A) as u32) | (0xa80000u32), game);
    };
    [11065643] MaridiaPuffer_Func_1 [] {
        crate::update_place! { previous fresh7 = (*E).var_E => (*E).var_E.wrapping_sub(1); }
        if fresh7 == 1 {
            (*E).var_E = 16;
            (*E).var_A = ((0xa8d940i32) as u16);
        }
    };
    [11065699] MaridiaPuffer_Func_3 [] {
        if (((*E).var_D) as i16).wrapping_sub(((*E).var_00) as i16) < 0 {
            MaridiaPuffer_Func_8(word!(game.cur_enemy_index), game);
        }
        MaridiaPuffer_Func_6(game);
        MaridiaPuffer_Func_7(game);
        MaridiaPuffer_Func_5(game);
    };
    [11065724] MaridiaPuffer_Func_4 [] {
        if ((*E).var_D as i16) > 0 && ((*E).var_B as i16) > 0 {
            MaridiaPuffer_Func_6(game);
            MaridiaPuffer_Func_7(game);
            MaridiaPuffer_Func_9(word!(game.cur_enemy_index), game);
        } else {
            (*E).var_A = ((0xa8d92bi32) as u16);
            set_fields! {[(*E)], 0; var_B, var_C, var_D}
        };
    };
    MaridiaPuffer_Func_6 [] {
        let mut R26: u16 = 0;
        let r18: u16 = crate::rtl::rom_table_word!(
            game,
            0xa0b1c3,
            ((((*E).var_02) as u8).wrapping_add(64)) as isize
        );
        if (r18 & 0x8000) != 0 {
            R26 = R26.wrapping_add(1);
        }
        let mut t: u32 = (((Abs16(r18) as i32 & 0xff00) >> 8) * (*E).var_D as i32) as u32;
        if R26 != 0 {
            t = -(t as i32) as u32;
        }
        words!((*E).base, x_pos, x_subpos += t);
    };
    MaridiaPuffer_Func_7 [] {
        let mut R26: u16 = 0;
        let r18: u16 = crate::rtl::rom_table_word!(game, 0xa0b1c3, (*E).var_02 as u8 as isize);
        if (r18 & 0x8000) != 0 {
            R26 = R26.wrapping_add(1);
        }
        let mut t: u32 = (((Abs16(r18) as i32 & 0xff00) >> 8) * (*E).var_D as i32) as u32;
        if R26 != 0 {
            t = -(t as i32) as u32;
        }
        words!((*E).base, y_pos, y_subpos += t);
    };
    MaridiaPuffer_Shot [] {
        (*E).var_40 = (*E).base.health;
        Enemy_NormalShotAI_A8(game);
        if (*E).base.health == (*E).var_40 && (*E).var_03 == 0 {
            set_fields! {[(*E).base]; instruction_timer = 1, timer = 0,
                current_instruction = addr_kMaridiaPuffer_Ilist_D855 as u16, };
            (*E).var_02 =
                g_word_A8D871[(word!(game.projectile_dir[word!(game.collision_detection_index)]) as i32
                    & 0xf) as usize] as u16;
            (*E).var_B = 256;
            (*E).var_D = 1536;
            (*E).var_A = ((0xa8d97ci32) as u16);
            (*E).var_07 = 48;
            (*E).var_03 = 1;
        }
    };
}
pub unsafe fn MaridiaPuffer_Func_5(game: &mut Game) {
    let v1: u8 = ((CalculateAngleOfSamusFromEnemy(word!(game.cur_enemy_index), game)) as u8)
        .wrapping_sub(64);
    let E = Get_MaridiaPuffer(word!(game.cur_enemy_index) as i32, game);
    (*E).var_01 = v1 as u16;
    let v3: u16 = SignExtend8(((v1) as u16).wrapping_sub((*E).var_02));
    let v4: u16 = Abs16(v3);
    if v4 as i32 - 48i32 & 0x8000 == 0 {
        (*E).var_A = ((0xa8d97ci32) as u16);
        (*E).var_C = 24;
    }
}
crate::game::countdown_callbacks! {
    pub unsafe fn (k: u16, game: &mut Game);
    before {
        let E = Get_MaridiaPuffer(k as i32, game);
        crate::update_place! { previous previous_timer = (*E).var_F => (*E).var_F.wrapping_sub(1); }
    } when previous_timer == 1;
    MaridiaPuffer_Func_8 {
        (*E).var_F = (*E).var_05;
        (*E).var_B = ((*E).var_B).wrapping_add((*E).var_C);
        (*E).var_D = ((*E).var_D).wrapping_add((*E).var_B);
    };
    MaridiaPuffer_Func_9 {
        (*E).var_F = (*E).var_06;
        (*E).var_B = ((*E).var_B).wrapping_sub((*E).var_C);
        (*E).var_D = ((*E).var_D).wrapping_sub((*E).var_B);
    };
}
pub unsafe fn WalkingLavaSeahorse_Func_2(k: u16, game: &mut Game) -> u16 {
    let E = Get_WalkingLavaSeahorse(k as i32, game);
    words!((*E), var_B, var_C += words!((*E), var_00, var_01));
    (*E).var_B
}
crate::game::callback_dispatch! {
    pub unsafe fn CallWalkingLavaSeahorseFunc(ea: u32, k: u16, game: &mut Game) => ea;
    call (k, game); default { Unreachable(game); };
    @functions [pub unsafe fn (k: u16, game: &mut Game)];
    [11066955] WalkingLavaSeahorse_Func_7 [] {
        Enemy_MoveDown(k, (2i16 as i32) << 16, game);
        let E = Get_WalkingLavaSeahorse(k as i32, game);
        if ((abs16(((*E).var_02).wrapping_sub((*E).base.x_pos))) as i16)
            .wrapping_sub((g_word_A8DCCB) as i16)
            >= 0
        {
            (*E).var_A = ((0xa8decdi32) as u16);
            (*E).var_B = -4i32 as u16;
            (*E).var_C = 0;
            let mut v4: u16 = addr_kWalkingLavaSeahorse_Ilist_DC51 as u16;
            if ((*E).var_D & 0x8000) == 0 {
                v4 = addr_kWalkingLavaSeahorse_Ilist_DCC1 as u16;
            }
            enemy_instruction! {[(*E).base]; instruction = v4, timer = 1}
        } else {
            if (*E).var_04 != 0 {
                return;
            }
            let t: i32 = word!(game.samus_x_pos) as i32 - (*E).base.x_pos as i32;
            let mut v3: u16 = 0;
            if t & 0x8000 == 0 {
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
            enemy_instruction! {[(*E).base]; instruction = v3, timer = 1}
            (*E).var_A = ((0xa8decci32) as u16);
        };
    };
    @functions [pub unsafe fn (k: u16, game: &mut Game)];
    @prefix { let E = Get_WalkingLavaSeahorse(k as i32, game); };
    WalkingLavaSeahorse_Func_1 [] {
        (*E).var_B = -12i32 as u16;
        set_fields! {[(*E)], 0; var_C, var_D, var_E, var_00}
        (*E).var_01 = 0x8000;
    };
    [11066737] WalkingLavaSeahorse_Func_3 [] {
        let mut xxx: u16 = 0;
        let v3: u16 = abs16(((*E).var_03).wrapping_sub(word!(game.samus_y_pos)));
        if v3 as i32 - 32i32 & 0x8000 != 0 && {
            xxx = (word!(game.samus_x_pos)).wrapping_sub((*E).base.x_pos);
            ((abs16(xxx)) as i16).wrapping_sub((g_word_A8DCC7) as i16) < 0
        } {
            WalkingLavaSeahorse_Func_1(k, game);
            let mut v4: u16 = addr_stru_A8DC4B as u16;
            let mut v5: i16 = -2i16;
            if xxx & 0x8000 == 0 {
                v4 = addr_stru_A8DCBB as u16;
                v5 = 2;
            }
            (*E).var_D = v5 as u16;
            enemy_instruction! {[(*E).base]; instruction = v4, timer = 1}
            (*E).var_A = ((0xa8ddc6i32) as u16);
            QueueSfx2_Max6(0x5e, game);
        }
    };
    [11066822] WalkingLavaSeahorse_Func_4 [] {
        words!((*E).base, y_pos, y_subpos += words!((*E), var_B, var_C));
        if (WalkingLavaSeahorse_Func_2(k, game) & 0x8000) == 0 {
            sub_A8DDDE(k, game);
        }
    };
    [11066885] WalkingLavaSeahorse_Func_6 [] {
        if Enemy_MoveDown(k, (words!((*E), var_B, var_C)) as i32, game) != 0 {
            let t: i32 = word!(game.samus_x_pos) as i32 - (*E).base.x_pos as i32;
            let mut v2: u16 = addr_kWalkingLavaSeahorse_Ilist_DBE7 as u16;
            let mut v3: i16 = -2i16;
            if t & 0x8000 == 0 {
                v3 = 2;
                v2 = addr_kWalkingLavaSeahorse_Ilist_DC57 as u16;
            }
            (*E).var_D = v3 as u16;
            enemy_instruction! {[(*E).base]; instruction = v2, timer = 1}
            (*E).var_A = ((0xa8de4bi32) as u16);
            (*E).var_04 = 1;
        } else {
            WalkingLavaSeahorse_Func_2(k, game);
        };
    };
    [11067085] WalkingLavaSeahorse_Func_8 [] {
        words!((*E).base, y_pos, y_subpos += words!((*E), var_B, var_C));
        if (WalkingLavaSeahorse_Func_2(k, game) & 0x8000) == 0 {
            (*E).var_A = ((0xa8deeci32) as u16);
        }
    };
    [11067116] WalkingLavaSeahorse_Func_9 [] {
        words!((*E).base, y_pos, y_subpos += words!((*E), var_B, var_C));
        if (((*E).base.y_pos) as i16).wrapping_sub(((*E).var_F) as i16) >= 0 {
            (*E).base.y_pos = (*E).var_F;
            (*E).base.x_pos = (*E).var_02;
            (*E).var_A = ((0xa8dd71i32) as u16);
        } else {
            WalkingLavaSeahorse_Func_2(k, game);
        };
    };
    @external {
        11067084 => [()];
    }
}
pub unsafe fn sub_A8DDDE(k: u16, game: &mut Game) {
    let v1 = gEnemyData(k as i32, game);
    (*v1).ai_var_A = ((0xa8de05i32) as u16);
    let mut v2: u16 = addr_kWalkingLavaSeahorse_Ilist_DC51 as u16;
    if ((*v1).ai_var_D & 0x8000) == 0 {
        v2 = addr_kWalkingLavaSeahorse_Ilist_DCC1 as u16;
    }
    enemy_instruction! {[(*v1)]; instruction = v2, timer = 1}
}
pub unsafe fn WalkingLavaSeahorse_DF20(a: u16, game: &mut Game) {
    SpawnEprojWithGfx(a, word!(game.cur_enemy_index), addr_loc_A89E90 as u16, game);
    QueueSfx2_Max6(0x3f, game);
}
crate::game::callback_dispatch! {
    pub unsafe fn CallWreckedShipOrbsA(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    @prefix { let E = Get_WreckedShipOrbs(word!(game.cur_enemy_index) as i32, game); };
    WreckedShipOrbs_Init [] {
        (*E).base.instruction_timer = 1;
        (*E).base.timer = 0;
        (*E).base.current_instruction =
            crate::rtl::rom_table_word!(game, 0xa8e380, (*E).parameter_1 as isize);
        let v1: i32 = 8 * (*E).parameter_2 as i32 >> 1;
        (*E).var_01 = crate::rtl::rom_table_word!(game, 0xa28187, v1 as isize);
        (*E).var_00 = crate::rtl::rom_table_word!(game, 0xa28187, (v1 + 1) as isize);
        (*E).var_03 = crate::rtl::rom_table_word!(game, 0xa28187, (v1 + 2) as isize);
        (*E).var_02 = crate::rtl::rom_table_word!(game, 0xa28187, (v1 + 3) as isize);
    };
    WreckedShipOrbs_Func_1 [] {
        (*E).var_A = ((0xa8e405i32) as u16);
        if (GetSamusEnemyDelta_Y(word!(game.cur_enemy_index), game) & 0x8000) == 0 {
            (*E).var_A = ((0xa8e424i32) as u16);
        }
    };
    WreckedShipOrbs_Func_2 [] {
        (*E).var_B = ((0xa8e443i32) as u16);
        if (GetSamusEnemyDelta_X(word!(game.cur_enemy_index), game) & 0x8000) == 0 {
            (*E).var_B = ((0xa8e462i32) as u16);
        }
    };
    [11068421] WreckedShipOrbs_Func_3 [] {
        words!((*E).base, y_pos, y_subpos += words!((*E), var_03, var_02));
    };
    [11068452] WreckedShipOrbs_Func_4 [] {
        words!((*E).base, y_pos, y_subpos += words!((*E), var_01, var_00));
    };
    WreckedShipOrbs_Func_5 [] {
        words!((*E).base, x_pos, x_subpos += words!((*E), var_03, var_02));
    };
    WreckedShipOrbs_Func_6 [] {
        words!((*E).base, x_pos, x_subpos += words!((*E), var_01, var_00));
    };
}
crate::game::address_dispatch! {
    pub unsafe fn CallWreckedShipOrbsB(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        11068483 => WreckedShipOrbs_Func_5; 11068514 => WreckedShipOrbs_Func_6;
    }
}
unsafe fn WreckedShipSpark_Func_4(k: u16, r18: u16, game: &mut Game) {
    let E = Get_WreckedShipSpark(k as i32, game);
    let mut wssk_var_E: i16 = (*E).var_E as i16;
    if wssk_var_E < 0 {
        wssk_var_E = ((NextRandom(game) as i32 & 0x3f) + 4) as i16;
    }
    (*E).var_F = (r18).wrapping_add((wssk_var_E) as u16);
}
crate::game::callback_dispatch! {
    pub unsafe fn CallWreckedShipSpark(ea: u32, k: u16, game: &mut Game) => ea;
    call (k, game); default { Unreachable(game); };
    @countdown [pub unsafe fn (k: u16, game: &mut Game)];
    before {
        let E = Get_WreckedShipSpark(k as i32, game);
    }
    when (*E).var_F == 1;
    [11069077] WreckedShipSpark_Func_1 {
        (*E).var_B = ((0xa8e6b7i32) as u16);
        enemy_instruction! {[(*E).base]; instruction = addr_kWreckedShipSpark_Ilist_E5A7 as u16, timer = 1}
        WreckedShipSpark_Func_4(k, 0, game);
    } else {
        (*E).var_F = (*E).var_F.wrapping_sub(1);
    } after { ; };
    [11069111] WreckedShipSpark_Func_2 {
        (*E).var_B = ((0xa8e695i32) as u16);
        enemy_instruction! {[(*E).base]; instruction = addr_kWreckedShipSpark_Ilist_E5E5 as u16, timer = 1}
        WreckedShipSpark_Func_4(k, 8, game);
    } else {
        (*E).var_F = (*E).var_F.wrapping_sub(1);
    } after { ; };
    [11069148] WreckedShipSpark_Func_3 {
        let v2: u16 = word!(game.cur_enemy_index);
        SpawnEprojWithGfx(
            0,
            word!(game.cur_enemy_index),
            addr_kEproj_Sparks as u16,
            game,
        );
        WreckedShipSpark_Func_4(v2, 0, game);
    } else {
        (*E).var_F = (*E).var_F.wrapping_sub(1);
    } after { ; };
    @external {
        11069076 => [()];
    }
}
pub unsafe fn BlueBrinstarFaceBlock_Func_1(game: &mut Game) {
    if word!(game.door_transition_flag_enemies) == 0 && {
        word!(game.variables_for_enemy_graphics_drawn_hook[2] -= 1, reread) == 0
    } {
        word!(game.variables_for_enemy_graphics_drawn_hook[2] = 16);
        let mut v0: u16 = word!(game.variables_for_enemy_graphics_drawn_hook[0]);
        let mut v1: u16 =
            (8u16).wrapping_mul(word!(game.variables_for_enemy_graphics_drawn_hook[1]));
        for _ in 0..4 {
            *(ram_ptr!(game, u16; 0xc000)).offset(((v0 as i32 >> 1) + 137) as isize) =
                crate::rtl::rom_table_word!(game, 0xa8e7cc, (v1 >> 1) as isize);
            v1 = (v1).wrapping_add(2);
            v0 = (v0).wrapping_add(2);
        }
        word!(
            game.variables_for_enemy_graphics_drawn_hook[1] =
                (*((ram_ptr!(game, u16; 0x1794)).offset(1) as *mut u8) as i32 + 1 & 7) as u16
        );
    }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallKiHunterFunc(ea: u32, k: u16, game: &mut Game) => ea;
    call (k, game); default { Unreachable(game); };
    @functions [pub unsafe fn (k: u16, game: &mut Game)];
    [11072104] KiHunter_Func_1 [] {
        let mut v4: i16 = 0;
        let E = Get_KiHunter(k as i32, game);
        let update_velocity: bool =
            if Enemy_MoveDown(k, (words!((*E), var_09, var_08)) as i32, game) != 0 {
                v4 = -((*E).var_09 as i32) as i16;
                true
            } else {
                let y_pos: u16 = (*E).base.y_pos;
                if ((y_pos) as i16).wrapping_sub(((*E).var_0A) as i16) < 0 {
                    v4 = 1;
                    true
                } else {
                    !(((y_pos) as i16).wrapping_sub(((*E).var_0B) as i16) < 0)
                        && ({
                            v4 = -1i16;
                            true
                        })
                }
            };
        if update_velocity {
            (*E).var_09 = v4 as u16;
        }
        if Enemy_MoveRight_IgnoreSlopes(k, (words!((*E), var_07, var_06)) as i32, game) != 0 {
            let mut v5: u16 = 0;
            let v6: bool = (-((*E).var_07 as i32) & 0x8000) != 0;
            (*E).var_07 = -((*E).var_07 as i32) as u16;
            if !v6 {
                v5 = 4;
            }
            let v7: i32 = v5 as i32 >> 1;
            (*E).base.current_instruction =
                crate::rtl::rom_table_word!(game, 0xa8f3b0, v7 as isize);
            let E1 = Get_KiHunter(k as i32 + 64, game);
            (*E1).base.current_instruction =
                crate::rtl::rom_table_word!(game, 0xa8f3b0, (v7 + 1) as isize);
            (*E).base.instruction_timer = 1;
            (*E1).base.instruction_timer = 1;
        }
        EnemyFunc_C8AD(k, game);
        let r18: u16 = (word!(game.samus_x_pos)).wrapping_sub((*E).base.x_pos);
        let r20: u16 = abs16(r18);
        if ((r20) as i16).wrapping_sub((g_word_A8F180) as i16) < 0
            && word!(game.samus_y_pos) as i32 - (*E).base.y_pos as i32 - 32i32 & 0x8000 == 0
        {
            let r24: u16 = (word!(game.samus_y_pos)).wrapping_sub((*E).base.y_pos);
            let mut v9: u16 = if (r18 & 0x8000) != 0 {
                (*E).var_00 = -2i32 as u16;
                set_fields! {[(*E)], 0; var_02, var_03}
                (*E).var_04 = -1i32 as u16;
                (*E).var_07 = -1i32 as u16;
                (*E).var_05 = -8192i32 as u16;
                (*E).var_F = 255;
                (*E).var_0E = 240;
                0
            } else {
                (*E).var_00 = 2;
                set_fields! {[(*E)], 0; var_02, var_03, var_04}
                (*E).var_05 = 0x2000;
                (*E).var_F = 128;
                (*E).var_07 = 1;
                (*E).var_0E = 144;
                4
            };
            let v10: i32 = v9 as i32 >> 1;
            (*E).base.current_instruction =
                crate::rtl::rom_table_word!(game, 0xa8f3b0, v10 as isize);
            let E1_0 = Get_KiHunter(k as i32 + 64, game);
            (*E1_0).base.current_instruction =
                crate::rtl::rom_table_word!(game, 0xa8f3b0, (v10 + 1) as isize);
            (*E).base.instruction_timer = 1;
            (*E1_0).base.instruction_timer = 1;
            set_fields! {[(*E)]; var_B = (r18).wrapping_add((*E).base.x_pos),
                var_C = (*E).base.y_pos, var_A = ((0xa8f3b8i32) as u16), var_12 = r24,
                var_11 = r20, var_10 = 0, };
        }
    };
    [11073203] KiHunter_Func_8 [] {
        let mut v1: u16 = addr_kKiHunter_Ilist_EAF2 as u16;
        let E = Get_KiHunter(k as i32, game);
        if (((*E).base.x_pos) as i16).wrapping_sub((word!(game.samus_x_pos)) as i16) < 0 {
            v1 = addr_kKiHunter_Ilist_EB10 as u16;
        }
        enemy_instruction! {[(*E).base]; instruction = v1, timer = 1}
        (*E).var_A = addr_locret_A8F5E3 as u16;
    };
    [11073267] KiHunter_Func_9 [] {
        let v1: i32 = k as i32 >> 1;
        let E = Get_KiHunter(k as i32, game);
        (*E).base.x_pos = word!(game.enemy_drawing_queue[(v1 + 91)]);
        (*E).base.y_pos = word!(game.enemy_drawing_queue[(v1 + 93)]);
    };
    [11073487] KiHunter_Func_10 [] {
        let r18: u16 = (*Get_KiHunter(word!(game.cur_enemy_index) as i32, game)).var_00;
        CallKiHunterBFunc(((r18) as u32) | (0xa80000u32), game);
    };
    @functions [pub unsafe fn (k: u16, game: &mut Game)];
    @prefix { let E = Get_KiHunter(k as i32, game); };
    [11072440] KiHunter_Func_2 [] {
        let moving_forward = ((*E).var_04 & 0x8000) == 0;
        let past_turn = (((*E).var_F) as i16).wrapping_sub(((*E).var_0E) as i16) >= 0;
        if moving_forward == past_turn && (*E).var_10 == 0 {
            (*E).var_10 = 1;
            let mut v2: u16 = addr_kKiHunter_Ilist_EA32 as u16;
            if ((*E).var_04 & 0x8000) != 0 {
                v2 = addr_kKiHunter_Ilist_EA08 as u16;
            }
            enemy_instruction! {[(*E).base]; instruction = v2, timer = 1}
        }
        let moving_forward = ((*E).var_04 & 0x8000) == 0;
        words!((*E), var_02, var_03 += words!((*E), var_04, var_05));
        let reached_speed_limit = if moving_forward {
            (((*E).var_02) as i16).wrapping_sub(((*E).var_00) as i16) >= 0
        } else {
            (((*E).var_02) as i16).wrapping_sub(((*E).var_00) as i16) < 0
        };
        if reached_speed_limit {
            (*E).var_02 = (*E).var_00;
        }
        (*E).var_F = ((*E).var_F).wrapping_add((*E).var_02);
        let finished_arc = if moving_forward {
            (*E).var_F as i32 - 256i32 & 0x8000 == 0
        } else {
            (*E).var_F as i32 - 128i32 & 0x8000 != 0
        };
        if finished_arc {
            (*E).var_A = ((0xa8f268i32) as u16);
            return;
        }
        let mut r20: u16 = ((*E).var_B as i32 + CosineMult8bit((*E).var_F, (*E).var_11, game) as i32
            - (*E).base.x_pos as i32) as u16;
        let collided = if Enemy_MoveRight_IgnoreSlopes(k, (r20 as i16 as i32) << 16, game) != 0 {
            if ((*E).var_04 & 0x8000) == 0 {
                (*E).var_06 = 0;
                (*E).var_07 = -1i32 as u16;
            } else {
                (*E).var_06 = 0;
                (*E).var_07 = 1;
            }
            true
        } else {
            EnemyFunc_C8AD(k, game);
            r20 = ((*E).var_C as i32 + SineMult8bit((*E).var_F, (*E).var_12, game) as i32
                - (*E).base.y_pos as i32) as u16;
            Enemy_MoveDown(k, (r20 as i16 as i32) << 16, game) != 0
        };
        if collided {
            (*E).var_A = ((0xa8f4edi32) as u16);
            (*E).var_08 = 0;
            (*E).var_09 = -1i32 as u16;
        };
    };
    [11072749] KiHunter_Func_3 [] {
        if Enemy_MoveRight_IgnoreSlopes(k, (words!((*E), var_07, var_06)) as i32, game) != 0
            || {
                EnemyFunc_C8AD(k, game);
                Enemy_MoveDown(k, (words!((*E), var_09, var_08)) as i32, game) != 0
            }
            || (((*E).base.y_pos) as i16).wrapping_sub(((*E).var_0D) as i16) < 0
        {
            (*E).var_A = ((0xa8f268i32) as u16);
        }
    };
    [11072858] KiHunter_Func_4 [] {
        if Enemy_MoveDown(k, (words!((*E), var_09, var_08)) as i32, game) != 0 {
            (*E).var_A = ((0xa8f58bi32) as u16);
        } else {
            words!(
                (*E),
                var_09,
                var_08 += words!((g_word_A8F184, g_word_A8F182))
            );
        };
    };
    [11072907] KiHunter_Func_5 [] {
        (*E).var_A = addr_locret_A8F5E3 as u16;
        (*E).var_08 = 0;
        (*E).var_09 = ((word!(game.random_number) as i32 & 1) - 8i32) as u16;
        if (((*E).base.x_pos) as i16).wrapping_sub((word!(game.samus_x_pos)) as i16) >= 0 {
            (*E).var_06 = 0;
            (*E).var_07 = -2i32 as u16;
            (*E).base.current_instruction = addr_kKiHunter_Ilist_EA8A as u16;
        } else {
            (*E).var_06 = 0;
            (*E).var_07 = 2;
            (*E).base.current_instruction = addr_kKiHunter_Ilist_EAA6 as u16;
        }
        (*E).base.instruction_timer = 1;
    };
    [11073008] KiHunter_Func_6 [] {
        if Enemy_MoveDown(k, (words!((*E), var_09, var_08)) as i32, game) != 0 {
            if ((*E).var_09 & 0x8000) != 0 {
                (*E).var_09 = 1;
            } else {
                (*E).var_08 = 0;
                (*E).var_09 = -4i32 as u16;
                (*E).var_A = addr_locret_A8F5E3 as u16;
                (*E).var_0F = 12;
                let mut v4: u16 = addr_kKiHunter_Ilist_EAC2 as u16;
                if (*E).base.current_instruction as i32 + 0x155a & 0x8000 == 0 {
                    v4 = addr_kKiHunter_Ilist_EADA as u16;
                }
                enemy_instruction! {[(*E).base]; instruction = v4, timer = 1}
            }
        } else if Enemy_MoveRight_IgnoreSlopes(k, (words!((*E), var_07, var_06)) as i32, game) != 0 {
            (*E).var_07 = -((*E).var_07 as i32) as u16;
        } else {
            EnemyFunc_C8AD(k, game);
            words!(
                (*E),
                var_09,
                var_08 += words!((g_word_A8F184, g_word_A8F182))
            );
        };
    };
    [11073163] KiHunter_Func_7 [] {
        let v2: u16 = ((*E).var_0F).wrapping_sub(1);
        (*E).var_0F = v2;
        if v2 == 0 {
            let mut v3: u16 = ((0xa8f58bi32) as u16);
            let v4: u16 = abs16(((*E).base.x_pos).wrapping_sub(word!(game.samus_x_pos)));
            if v4 as i32 - 96i32 & 0x8000 != 0 {
                v3 = ((0xa8f6b3i32) as u16);
            }
            (*E).var_A = v3;
        }
    };
    KiHunter_Func_15 [] {
        set_fields! {[(*E)]; var_00 = ((0xa8f7dbi32) as u16), var_B = (*E).var_0A,
            var_F = -8192i32 as u16, var_05 = (*E).base.x_pos, var_06 = (*E).base.y_pos, };
    };
    KiHunter_Func_16 [] {
        set_fields! {[(*E)]; var_00 = ((0xa8f8adi32) as u16), var_B = (*E).var_0A,
            var_F = -24576i32 as u16, var_05 = (*E).base.x_pos, var_06 = (*E).base.y_pos, };
    };
    @external {
        11072995 => [()];
    }
}
pub use self::KiHunter_Main as KiHunterWings_Main;
pub unsafe fn sub_A8F6DC(k: u16, j: u16, game: &mut Game) {
    QueueSfx2_Max6(0x4c, game);
    SpawnEprojWithGfx(0, word!(game.cur_enemy_index), j, game);
    (*Get_KiHunter(word!(game.cur_enemy_index) as i32, game)).var_0F = 24;
}
crate::game::callback_dispatch! {
    pub unsafe fn CallKiHunterBFunc(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [11073709] KiHunter_Func_14 [] {
        let mut khr_var_07: i16 = 0;
        let E = Get_KiHunter(word!(game.cur_enemy_index) as i32, game);
        (*E).var_F = ((*E).var_F).wrapping_add(
            *((rom_ptr!(game, u16; 0xa2838f) as *mut u8)
                .offset((8i32 * byte!((*E).var_B; 1isize) as i32) as isize)
                .offset(1) as *mut u16),
        );
        let r20: u16 = ((*E).var_06 as i32
            + SineMult8bit(byte!((*E).var_F; 1) as u16, g_byte_A8F186 as u16, game) as i32
            - (*E).var_02 as i32
            - (*E).base.y_pos as i32) as u16;
        if Enemy_MoveDown(word!(game.cur_enemy_index), (r20 as i16 as i32) << 16, game) != 0 {
            (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
            (*E).base.x_pos = (*E).var_08;
            khr_var_07 = (*E).var_07 as i16;
            (*E).base.y_pos = khr_var_07 as u16;
        } else {
            (*E).base.x_pos = ((*E).var_05 as i32
                + CosineMult8bit(byte!((*E).var_F; 1) as u16, g_byte_A8F186 as u16, game) as i32
                - (*E).var_01 as i32) as u16;
            if (*E).var_F as i32 + 0x4000 & 0x8000 == 0 {
                KiHunter_Func_15(word!(game.cur_enemy_index), game);
                return;
            }
            khr_var_07 = (((*E).var_B) as i16).wrapping_sub(384);
            if khr_var_07 < 0 {
                khr_var_07 = 256;
            }
        }
        (*Get_KiHunter(word!(game.cur_enemy_index) as i32, game)).var_B = khr_var_07 as u16;
    };
    @functions [pub unsafe fn (game: &mut Game)];
    @prefix { let E = Get_KiHunter(word!(game.cur_enemy_index) as i32, game); };
    KiHunter_Init [] {
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_DisableSamusColl) as u16);
        (*E).var_14 = 0;
        set_fields! {[(*E).base]; instruction_timer = 1, timer = 0,
            current_instruction = addr_kKiHunter_Ilist_E9FA as u16, };
        set_fields! {[(*E)]; var_F = 0, var_A = ((0xa8f268i32) as u16), var_08 = 0, var_09 = 1,
            var_06 = 0, var_07 = -1i32 as u16, };
        let v1: u16 = ((*E).base.y_pos).wrapping_sub(16);
        (*E).var_0A = v1;
        (*E).var_0B = (v1).wrapping_add(32);
        (*E).var_0C = (*E).base.x_pos;
        (*E).var_0D = (*E).base.y_pos;
        if ((*E).parameter_1 & 0x8000) != 0 {
            (*E).var_14 = 1;
            (*E).var_A = ((0xa8f55ai32) as u16);
            (*E).var_08 = 0;
            (*E).var_09 = 1;
        }
    };
    KiHunterWings_Init [] {
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_DisableSamusColl) as u16);
        (*E).base.instruction_timer = 1;
        (*E).base.timer = 0;
        (*E).base.current_instruction = addr_kKiHunter_Ilist_EA4E as u16;
        let v2: i32 = word!(game.cur_enemy_index) as i32 >> 1;
        (*E).base.y_pos = word!(game.enemy_drawing_queue[(v2 + 93)]);
        (*E).base.x_pos = word!(game.enemy_drawing_queue[(v2 + 91)]);
        (*E).var_A = ((0xa8f6f3i32) as u16);
        (*E).base.palette_index = word!(game.enemy_drawing_queue[(v2 + 105)]);
        (*E).base.vram_tiles_index = word!(game.enemy_drawing_queue[(v2 + 106)]);
        if (word!(game.enemy_drawing_queue_sizes[(v2 + 6)]) & 0x8000) != 0 {
            (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
        }
    };
    KiHunter_Main [] {
        CallKiHunterFunc(
            (((*E).var_A) as u32) | (0xa80000u32),
            word!(game.cur_enemy_index),
            game,
        );
    };
    [11073499] KiHunter_Func_11 [] {
        (*E).var_F = ((*E).var_F).wrapping_add(
            *(((rom_ptr!(game, u16; 0xa2838f)).offset(2) as *mut u8)
                .offset((8i32 * byte!((*E).var_B; 1isize) as i32) as isize)
                .offset(1) as *mut u16),
        );
        (*E).base.y_pos = ((*E).var_06 as i32
            + SineMult8bit(byte!((*E).var_F; 1) as u16, g_byte_A8F186 as u16, game) as i32
            - (*E).var_04 as i32) as u16;
        (*E).base.x_pos = ((*E).var_05 as i32
            + CosineMult8bit(byte!((*E).var_F; 1) as u16, g_byte_A8F186 as u16, game) as i32
            - (*E).var_03 as i32) as u16;
        if (*E).var_F as i32 + 0x4000 & 0x8000 != 0 {
            KiHunter_Func_16(word!(game.cur_enemy_index), game);
        } else {
            let mut v2: i16 = (((*E).var_B) as i16).wrapping_sub(384);
            if v2 < 0 {
                v2 = 256;
            }
            (*E).var_B = v2 as u16;
        };
    };
    KiHunter_Func_17 [] {
        set_fields! {[(*E)], 0; var_0A, var_B}
        do_until! { (v5 as i32 - 0x2000i32 & 0x8000 == 0);
    let mut v3: u16 = ((*E).var_0A).wrapping_add(384);
    (*E).var_0A = v3;
    v3 = (v3 >> 8);
    let mut v5: u16 = (*((rom_ptr!(game, u16; 0xa2838f) as *mut u8)
        .offset((8i32 * v3 as i32) as isize)
        .offset(1) as *mut u16))
        .wrapping_add((*E).var_D);
    (*E).var_D = v5;
}
    };
}
pub unsafe fn KiHunter_Func_12(game: &mut Game) {
    let v1: u16 = CosineMult8bit(0xe0, g_byte_A8F186 as u16, game);
    let E = Get_KiHunter(word!(game.cur_enemy_index) as i32, game);
    (*E).var_03 = v1;
    (*E).var_04 = SineMult8bit(0xe0, g_byte_A8F186 as u16, game);
}
pub unsafe fn KiHunter_Func_13(game: &mut Game) {
    let v1: u16 = CosineMult8bit(0xa0, g_byte_A8F186 as u16, game);
    let E = Get_KiHunter(word!(game.cur_enemy_index) as i32, game);
    (*E).var_01 = v1;
    (*E).var_02 = SineMult8bit(0xa0, g_byte_A8F186 as u16, game);
}

crate::game::callback_dispatch! {
    pub(crate) unsafe fn DispatchEnemyAi(ea: u32, game: &mut Game) => ea;
    call (game); default { crate::rtl::Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    @prefix { let E = Get_MiniDraygon(word!(game.cur_enemy_index) as i32, game); };
    [11044832] MiniDraygon_Init [] {
        if (*E).parameter_1 != 0 {
            MiniDraygon_Func_2(game);
            (*E).base.layer = 4;
        } else {
            MiniDraygon_Func_1(game);
            let v2: u16 = (8u16).wrapping_mul((byte!((*E).parameter_2; 0)) as u16);
            let v3: i32 = v2 as i32 >> 1;
            (*E).var_04 = crate::rtl::rom_table_word!(game, 0xa28187, v3 as isize);
            (*E).var_03 = crate::rtl::rom_table_word!(game, 0xa28187, (v3 + 1) as isize);
            (*E).var_06 = crate::rtl::rom_table_word!(game, 0xa28187, (v3 + 2) as isize);
            (*E).var_05 = crate::rtl::rom_table_word!(game, 0xa28187, (v3 + 3) as isize);
            (*E).var_E = (byte!((*(Get_MiniDraygon
                as unsafe fn(i32, &mut Game) -> *mut Enemy_MiniDraygon)(
                v2 as i32, game
            ))
            .parameter_2; 1)
                >> 1) as u16;
        }
        set_fields! {[(*E)], 0; var_00, var_01}
        (*E).var_C = ((0xa88922i32) as u16);
    };
    [11045040] EvirProjectile_Init [] {
        (*E).var_02 = addr_kMiniDraygon_Ilist_876F as u16;
        MiniDraygon_Func_12(game);
        let v2: i32 = word!(game.cur_enemy_index) as i32 >> 1;
        (*E).base.palette_index = word!(game.enemy_drawing_queue[(v2 + 73)]);
        (*E).base.vram_tiles_index = word!(game.enemy_drawing_queue[(v2 + 74)]);
        MiniDraygon_Func_3(game);
        set_fields! {[(*E)], 0; var_01, var_0C, var_0B, var_F}
        (*E).var_C = ((0xa88a34i32) as u16);
    };
    [11045147] MiniDraygon_Main [] {
        CallMiniDraygonFunc((((*E).var_C) as u32) | (0xa80000u32), game);
    };
    [11045278] EvirProjectile_Main [] {
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
    };
    MiniDraygon_Func_12 [] {
        let mdn_var_02: u16 = (*E).var_02;
        if mdn_var_02 != (*E).var_01 {
            (*E).base.current_instruction = mdn_var_02;
            (*E).var_01 = mdn_var_02;
            (*E).base.instruction_timer = 1;
            (*E).base.timer = 0;
        }
    };
    MiniDraygon_Func_13 [] {
        let E1 = Get_MiniDraygon(word!(game.cur_enemy_index) as i32 + 64, game);
        let E2 = Get_MiniDraygon(word!(game.cur_enemy_index) as i32 + 128, game);
        if (*E).base.health == 0 {
            (*E1).base.properties = ((*E1).base.properties) | (0x200u16);
            (*E2).base.properties = ((*E2).base.properties) | (0x200u16);
        }
        let frozen_timer: u16 = (*E).base.frozen_timer;
        if frozen_timer != 0 {
            (*E1).base.frozen_timer = frozen_timer;
            (*E1).base.ai_handler_bits = ((*E1).base.ai_handler_bits) | (4u16);
            if (*E2).var_C as i32 != 0xa88a3b & 0xffff {
                (*E2).base.ai_handler_bits = ((*E2).base.ai_handler_bits) | (4u16);
                (*E2).base.frozen_timer = (*E).base.frozen_timer;
            }
        }
    };
    @functions [pub unsafe fn (game: &mut Game)];
    [11045638] MiniDraygon_Touch [] {
        Enemy_NormalTouchAI_A8(game);
        MiniDraygon_Func_13(game);
    };
    [11045644] MiniDraygon_Powerbomb [] {
        Enemy_NormalPowerBombAI_A8(game);
        MiniDraygon_Func_13(game);
    };
    [11045650] MiniDraygon_Shot [] {
        NormalEnemyShotAi(game);
        MiniDraygon_Func_13(game);
    };
    [11047000] MorphBallEye_Init [] {
        let E = Get_MorphBallEye(word!(game.cur_enemy_index) as i32, game);
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_DisableSamusColl) as u16);
        (*E).base.spritemap_pointer = addr_kSpritemap_Nothing_A8 as u16;
        (*E).base.instruction_timer = 1;
        (*E).base.timer = 0;
        if ((*E).parameter_2 & 0x8000) != 0 {
            let v1: i32 = (*E).parameter_2 as i32 & 0xf;
            (*E).base.x_pos = ((*E).base.x_pos).wrapping_add((g_word_A890CA[v1 as usize]) as u16);
            (*E).base.y_pos = ((*E).base.y_pos).wrapping_add((g_word_A890D2[v1 as usize]) as u16);
            (*E).var_F = ((0xa891dci32) as u16);
            (*E).base.current_instruction =
                crate::rtl::rom_table_word!(game, 0xa890da, v1 as isize);
            word_transfers!(game, descending, 256; native(0x92fe) <- value(255));
        } else {
            (*E).var_F = ((0xa890f1i32) as u16);
            if ((*E).parameter_1 & 1) != 0 {
                (*E).base.current_instruction = addr_stru_A8900E as u16;
            } else {
                (*E).base.current_instruction = addr_stru_A88FFC as u16;
            }
        };
    };
    [11047138] MorphBallEye_Main [] {
        if (word!(game.collected_items) & 4) != 0 {
            let E = Get_MorphBallEye(word!(game.cur_enemy_index) as i32, game);
            CallEnemyPreInstr((((*E).var_F) as u32) | (0xa80000u32), game);
        }
    };
    [11049710] WreckedShipGhost_Init [] {
        let E = Get_WreckedShipGhost(word!(game.cur_enemy_index) as i32, game);
        (*E).base.properties = ((*E).base.properties as i32
            | (kEnemyProps_DisableSamusColl as i32
                | kEnemyProps_Tangible as i32
                | kEnemyProps_Invisible as i32)) as u16;
        (*E).base.instruction_timer = 1;
        (*E).base.timer = 0;
        (*E).base.current_instruction = addr_kWreckedShipGhost_Ilist_9A8C as u16;
        (*E).var_A = ((0xa89d13i32) as u16);
        (*E).var_B = (g_word_A89AA4).wrapping_add(160);
        let v2: u16 = u16::swap_bytes((*E).base.palette_index);
        let mut v3: u16 = (16 * v2 as i32 + 256) as u16;
        for _ in 0..=16 {
            word!(game.target_palettes[(v3 >> 1)] = 0);
            v3 = (v3).wrapping_add(2);
        }
    };
    [11049788] WreckedShipGhost_Main [] {
        let E = Get_WreckedShipGhost(word!(game.cur_enemy_index) as i32, game);
        CallWreckedShipGhost(
            (((*E).var_A) as u32) | (0xa80000u32),
            word!(game.cur_enemy_index),
            game,
        );
    };
    [11052989] YappingMaw_Shot [] {
        NormalEnemyShotAi(game);
        let E = Get_YappingMaw(word!(game.cur_enemy_index) as i32, game);
        if (*E).base.health != 0 {
            if (*E).base.frozen_timer != 0 {
                if word!(game.samus_input_handler) as i32 != 0x90e91d & 0xffff {
                    word!(game.samus_input_handler = ((0x90e913i32) as u16));
                }
                (*E).var_30 = 0;
            }
        } else {
            *((ram_ptr!(game, u16; 0x1997) as *mut u8).offset((*E).var_40 as i32 as isize)
                as *mut u16) = 0;
            *((ram_ptr!(game, u16; 0x1997) as *mut u8).offset((*E).var_41 as i32 as isize)
                as *mut u16) = 0;
            *((ram_ptr!(game, u16; 0x1997) as *mut u8).offset((*E).var_42 as i32 as isize)
                as *mut u16) = 0;
            *((ram_ptr!(game, u16; 0x1997) as *mut u8).offset((*E).var_43 as i32 as isize)
                as *mut u16) = 0;
            word!(game.sprite_instr_list_ptrs[((*E).var_46 >> 1)] = 0);
            if word!(game.samus_input_handler) as i32 != 0x90e91d & 0xffff {
                word!(game.samus_input_handler = ((0x90e913i32) as u16));
            }
            (*Get_YappingMaw(word!(game.cur_enemy_index) as i32, game)).var_30 = 0;
        };
    };
    [11053109] YappingMaw_Frozen [] {
        Enemy_NormalFrozenAI_A8(game);
        let E = Get_YappingMaw(word!(game.cur_enemy_index) as i32, game);
        YappingMaw_Func_18((*E).var_40, game);
        YappingMaw_Func_18((*E).var_41, game);
        YappingMaw_Func_18((*E).var_42, game);
        YappingMaw_Func_18((*E).var_43, game);
        YappingMaw_Func_19(game);
    };
    [11053894] Kago_Init [] {
        let E = Get_Kago(word!(game.cur_enemy_index) as i32, game);
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_DisableSamusColl) as u16);
        (*E).base.instruction_timer = 1;
        (*E).base.timer = 0;
        (*E).var_E = 0;
        (*E).base.current_instruction = addr_kKago_Ilist_AB1E as u16;
        (*E).var_A = ((0xa8ab7bi32) as u16);
        (*E).var_F = 0;
        (*E).var_04 = (*E).parameter_1;
    };
    [11053941] Kago_Main [] {
        let E = Get_Kago(word!(game.cur_enemy_index) as i32, game);
        CallKagoFunc(
            (((*E).var_A) as u32) | (0xa80000u32),
            word!(game.cur_enemy_index),
            game,
        );
    };
    [11053955] Kago_Shot [] {
        NormalEnemyShotAi(game);
        word! {game; earthquake_type = 2; earthquake_timer = 16; }
        let E = Get_Kago(word!(game.cur_enemy_index) as i32, game);
        if (*E).var_B == 0 {
            (*E).var_B = 1;
            enemy_instruction! {[(*E).base]; instruction = addr_kKago_Ilist_AB32 as u16, timer = 1}
        }
        let mut v2: i16 = (((*E).var_04) as i16).wrapping_sub(1);
        (*E).var_04 = v2 as u16;
        if v2 < 0 {
            EnemyDeathAnimation(word!(game.cur_enemy_index), 4, game);
            (*E).var_F = 1;
        }
        SpawnEprojWithGfx(
            (*E).base.y_pos,
            word!(game.cur_enemy_index),
            addr_loc_A8D02E as u16,
            game,
        );
    };
    [11054987] NorfairLavaMan_Init [] {
        let v0: u16 = (2u16).wrapping_mul(
            (*Get_NorfairLavaMan(word!(game.cur_enemy_index) as i32, game)).parameter_1,
        );
        g_off_A8AF4F[(v0 >> 1) as usize](game);
        NorfairLavaMan_Func_4(game);
        NorfairLavaMan_Func_5(word!(game.cur_enemy_index), game);
    };
    [11056128] NorfairLavaMan_Powerbomb [] {
        Enemy_NormalPowerBombAI_A8(game);
        NorfairLavaMan_Common(game);
    };
    [11056134] NorfairLavaMan_Touch [] {
        Enemy_NormalTouchAI_A8(game);
        NorfairLavaMan_Common(game);
    };
    [11056140] NorfairLavaMan_Shot [] {
        NormalEnemyShotAi(game);
        NorfairLavaMan_Common(game);
    };
    [11058860] Beetom_Shot [] {
        Enemy_NormalShotAI_A8(game);
        let E = Get_Beetom(word!(game.cur_enemy_index) as i32, game);
        if (*E).base.frozen_timer != 0
            && ((*E).var_C as i32 == 0xa8bd42 & 0xffff || (*E).var_C as i32 == 0xa8bcf8 & 0xffff)
        {
            (*E).var_C = ((0xa8b97ai32) as u16);
        }
        (*E).var_08 = 0;
    };
    [11059657] MaridiaFloater_Init [] {
        let E = Get_MaridiaFloater(word!(game.cur_enemy_index) as i32, game);
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_DisableSamusColl) as u16);
        (*E).base.spritemap_pointer = addr_kSpritemap_Nothing_A8 as u16;
        (*E).base.instruction_timer = 1;
        (*E).base.timer = 0;
        if (*E).parameter_1 != 0 {
            (*E).var_A = (*E).base.x_pos;
            (*E).var_B = (*E).base.y_pos;
            (*E).var_F = ((0xa8c568i32) as u16);
            (*E).base.current_instruction = addr_stru_A8C199 as u16;
            (*E).var_D = (*E).parameter_2;
        } else {
            (*E).var_E = 60;
            (*E).var_F = ((0xa8c283i32) as u16);
            (*E).base.current_instruction = addr_kMaridiaFloater_Ilist_C173 as u16;
        };
    };
    [11059740] MaridiaFloater_Main [] {
        let E = Get_MaridiaFloater(word!(game.cur_enemy_index) as i32, game);
        CallEnemyPreInstr((((*E).var_F) as u32) | (0xa80000u32), game);
    };
    [11060670] MaridiaFloater_Touch [] {
        let E = Get_MaridiaFloater(word!(game.cur_enemy_index) as i32, game);
        if (*E).parameter_2 == 0 {
            let palette_index: u16 = (*E).base.palette_index;
            NormalEnemyTouchAi(game);
            if (*E).base.health == 0 {
                word!(game.enemy_drawing_queue[((word!(game.cur_enemy_index) as i32 >> 1) + 97)] |= (0x200u16));
                (*E).base.palette_index = palette_index;
                MaridiaFloater_Func_1(word!(game.cur_enemy_index), game);
                (*E).base.palette_index = 2560;
            }
        }
    };
    [11060719] MaridiaFloater_Shot [] {
        let E = Get_MaridiaFloater(word!(game.cur_enemy_index) as i32, game);
        if (*E).parameter_2 == 0 {
            NormalEnemyShotAiSkipDeathAnim_CurEnemy(game);
            if ((*E).base.ai_handler_bits & 4) != 0 {
                let v2: i32 = word!(game.cur_enemy_index) as i32 >> 1;
                word!(game.enemy_drawing_queue[(v2 + 109)] = (*E).base.frozen_timer);
                word!(game.enemy_drawing_queue[(v2 + 99)] |= (4u16));
            }
            if ((*E).base.ai_handler_bits & 2) != 0 {
                let v3: i32 = word!(game.cur_enemy_index) as i32 >> 1;
                word!(game.enemy_drawing_queue[(v3 + 108)] = (*E).base.flash_timer);
                word!(game.enemy_drawing_queue[(v3 + 99)] |= (2u16));
            }
            if (*E).base.health == 0 {
                (*E).var_F = ((0xa8c569i32) as u16);
                (*E).parameter_2 = 1;
            }
        }
    };
    [11060799] MaridiaFloater_Powerbomb [] {
        Enemy_NormalPowerBombAI_A8(game);
        let E = Get_MaridiaFloater(word!(game.cur_enemy_index) as i32, game);
        if (*E).base.health != 0 {
            let v1: i32 = word!(game.cur_enemy_index) as i32 >> 1;
            word!(game.enemy_drawing_queue[(v1 + 111)] = (*E).base.shake_timer);
            word!(game.enemy_drawing_queue[(v1 + 110)] = (*E).base.invincibility_timer);
            word!(game.enemy_drawing_queue[(v1 + 108)] = (*E).base.flash_timer);
            word!(game.enemy_drawing_queue[(v1 + 109)] = (*E).base.frozen_timer);
            word!(game.enemy_drawing_queue[(v1 + 99)] = (*E).base.ai_handler_bits);
        } else {
            word!(game.enemy_drawing_queue[((word!(game.cur_enemy_index) as i32 >> 1) + 97)] |= (0x200u16));
        };
    };
    [11062135] WreckedShipRobot_Init [] {
        if (*(ram_ptr!(game, u16; 0xd828, word!(game.area_index) as isize)) & 1) != 0 {
            (*(ram_ptr!(game, LongPtr; 0x178c))).bank = -88i32 as u8;
            (*(ram_ptr!(game, LongPtr; 0x178c))).addr = ((0xa8cc67i32) as u16);
            let E = Get_WreckedShipRobot(word!(game.cur_enemy_index) as i32, game);
            set_fields! {[(*E).base]; properties = ((*E).base.properties) | (0xa000u16),
            instruction_timer = 4, timer = 0,
            current_instruction = addr_kWreckedShipRobot_Ilist_C6E5 as u16, };
            (*E).var_A = -512i32 as u16;
            word!(game.wrecked_ship_robot_palanim_timer = 1);
            *(ram_ptr!(game, u16; 0xd656)) = 0;
            (*E).var_B = 0;
            word!(game.wrecked_ship_robot_palanim_palindex = (*E).base.palette_index);
        } else {
            WreckedShipRobotDeactivated_Init(game);
        };
    };
    [11062220] WreckedShipRobotDeactivated_Init [] {
        let v0 = gEnemyData(word!(game.cur_enemy_index) as i32, game);
        let mut parameter_1: u16 = (*v0).parameter_1;
        if parameter_1 & 0x8000 != 0 || parameter_1 as i32 - 4i32 & 0x8000 == 0 {
            parameter_1 = 0;
        }
        set_fields! {[(*v0)]; parameter_1 = parameter_1,
        current_instruction = crate::rtl::rom_table_word!(game, 0xa8cc30, parameter_1 as isize),
        properties = ((*v0).properties) | (0x8000u16), instruction_timer = 1, timer = 0, };
        word!(game.wrecked_ship_robot_palanim_timer = 0);
        (*v0).ai_var_E = 0;
        (*v0).ai_preinstr = 1;
        let v2: i32 =
            16 * ((word!(game.wrecked_ship_robot_palanim_palindex) as i32 & 0xff00) >> 8) >> 1;
        word! { game, 10;
            palette_buffer[(v2 + 137)], palette_buffer[(v2 + 138)], palette_buffer[(v2 + 139)],
            palette_buffer[(v2 + 140)],
        }
    };
    [11062326] WreckedShipRobot_Main [] {
        let E = Get_WreckedShipRobot(word!(game.cur_enemy_index) as i32, game);
        if Enemy_MoveDown(
            word!(game.cur_enemy_index),
            (words!((*E), var_F, var_E)) as i32,
            game,
        ) == 0
        {
            (*E).base.instruction_timer = (*E).base.instruction_timer.wrapping_add(1);
            words!((*E), var_F, var_E += 0x8000);
        }
    };
    [11063668] WreckedShipRobotDeactivated_Touch [] {
        if (((*gEnemyData(word!(game.cur_enemy_index) as i32, game)).x_pos) as i16)
            .wrapping_sub((word!(game.samus_x_pos)) as i16)
            < 0
        {
            word!(game.extra_samus_x_displacement = 4);
        } else {
            word!(game.extra_samus_x_displacement = -4i32 as u16);
        };
    };
    [11063693] WreckedShipRobotDeactivated_Shot [] {
        NormalEnemyShotAi(game);
    };
    [11063698] WreckedShipRobot_Shot [] {
        if (*(ram_ptr!(game, u16; 0xd828, word!(game.area_index) as isize)) & 1) != 0 {
            NormalEnemyShotAi(game);
            let E = Get_WreckedShipRobot(word!(game.cur_enemy_index) as i32, game);
            if (*E).base.health != 0 {
                let mut v1: u16 = if ((*E).var_A & 0x8000) != 0 {
                    if ((word!(game.samus_x_pos)) as i16).wrapping_sub(((*E).base.x_pos) as i16)
                        >= 0
                    {
                        addr_kWreckedShipRobot_Ilist_C833 as u16
                    } else {
                        addr_kWreckedShipRobot_Ilist_C7BB as u16
                    }
                } else if ((word!(game.samus_x_pos)) as i16).wrapping_sub(((*E).base.x_pos) as i16)
                    < 0
                {
                    addr_kWreckedShipRobot_Ilist_CA7D as u16
                } else {
                    addr_kWreckedShipRobot_Ilist_CA01 as u16
                };
                enemy_instruction! {[(*E).base]; instruction = v1, timer = 1}
                (*E).var_B = ((*E).var_B).wrapping_add(64);
            }
        }
    };
    [11066573] WalkingLavaSeahorse_Init [] {
        let E = Get_WalkingLavaSeahorse(word!(game.cur_enemy_index) as i32, game);
        (*E).var_04 = 0;
        (*E).var_F = (*E).base.y_pos;
        (*E).var_02 = (*E).base.x_pos;
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_DisableSamusColl) as u16);
        (*E).base.current_instruction = addr_kWalkingLavaSeahorse_Ilist_DBE7 as u16;
        (*E).var_A = ((0xa8dd71i32) as u16);
        WalkingLavaSeahorse_Func_1(word!(game.cur_enemy_index), game);
        do_until! { ((WalkingLavaSeahorse_Func_2(word!(game.cur_enemy_index), game) & 0x8000) == 0);
            words!((*E).base, y_pos, y_subpos += words!((*E), var_B, var_C));
        }
        while (Enemy_MoveDown(
            word!(game.cur_enemy_index),
            (words!((*E), var_B, var_C)) as i32,
            game,
        ) == 0)
        {
            WalkingLavaSeahorse_Func_2(word!(game.cur_enemy_index), game);
        }
        (*E).var_03 = (*E).base.y_pos;
        (*E).base.y_pos = (*E).var_F;
    };
    [11066731] WalkingLavaSeahorse_Main [] {
        let E = Get_WalkingLavaSeahorse(word!(game.cur_enemy_index) as i32, game);
        CallWalkingLavaSeahorseFunc(
            (((*E).var_A) as u32) | (0xa80000u32),
            word!(game.cur_enemy_index),
            game,
        );
    };
    [11068355] WreckedShipOrbs_Main [] {
        WreckedShipOrbs_Func_1(game);
        WreckedShipOrbs_Func_2(game);
        let E = Get_WreckedShipOrbs(word!(game.cur_enemy_index) as i32, game);
        CallWreckedShipOrbsA((((*E).var_A) as u32) | (0xa80000u32), game);
        CallWreckedShipOrbsB((((*E).var_B) as u32) | (0xa80000u32), game);
    };
    [11068983] WreckedShipSpark_Init [] {
        let E = Get_WreckedShipSpark(word!(game.cur_enemy_index) as i32, game);
        let v2: i32 = (*E).parameter_1 as i32 & 3;
        (*E).var_B = crate::rtl::rom_table_word!(game, 0xa8e688, v2 as isize);
        (*E).var_E = (*E).parameter_2;
        WreckedShipSpark_Func_4(word!(game.cur_enemy_index), 0, game);
        enemy_instruction! {[(*E).base]; timer = 1, instruction = crate::rtl::rom_table_word!(game, 0xa8e682, v2 as isize)}
        (*E).base.timer = 0;
        (*E).base.instruction_timer = 1;
        if (*ram_ptr!(game; 0xd828, word!(game.area_index) as isize) & 1) == 0 {
            (*E).base.properties = ((*E).base.properties)
                | (*((&raw mut (*(gVramWriteEntry
                    as unsafe fn(i32, &mut Game) -> *mut VramWriteEntry)(
                    0i32, game
                )
                .offset(6))
                .vram_dst as *mut u8)
                    .offset(1) as *mut u16));
        }
    };
    [11069070] WreckedShipSpark_Main [] {
        let E = Get_WreckedShipSpark(word!(game.cur_enemy_index) as i32, game);
        CallWreckedShipSpark(
            (((*E).var_B) as u32) | (0xa80000u32),
            word!(game.cur_enemy_index),
            game,
        );
    };
    [11069198] WreckedShipSpark_Shot [] {
        word!(game.projectile_dir[word!(game.collision_detection_index)] &= !(0x10));
    };
    [11069486] BlueBrinstarFaceBlock_Init [] {
        let v0 = gEnemyData(word!(game.cur_enemy_index) as i32, game);
        (*v0).current_instruction = addr_kBlueBrinstarFaceBlock_Ilist_E828 as u16;
        let mut v1: u16 = ((0xa8e86ei32) as u16);
        if (word!(game.collected_items) & 4) == 0 {
            v1 = ((0xa8804ci32) as u16);
        }
        (*(ram_ptr!(game, LongPtr; 0x178c))).addr = v1;
        *(&raw mut (*(ram_ptr!(game, LongPtr; 0x178c))).bank as *mut u16) = 168;
        word!(
            game.variables_for_enemy_graphics_drawn_hook[0] =
                ((16 * (*v0).palette_index as i32 & 0xff00) >> 8) as u16
        );
        word!(game.variables_for_enemy_graphics_drawn_hook[2] = 16);
        (*v0).parameter_2 =
            (((*v0).parameter_2 as i32 & 1) >> 2 | ((*v0).parameter_2 as i32 & 1) << 15) as u16;
    };
    [11069614] BlueBrinstarFaceBlock_Main [] {
        if (word!(game.collected_items) & 4) != 0 {
            (*(ram_ptr!(game, LongPtr; 0x178c))).addr = ((0xa8e86ei32) as u16);
            let v1 = gEnemyData(word!(game.cur_enemy_index) as i32, game);
            word!(
                game.variables_for_enemy_graphics_drawn_hook[0] =
                    ((16 * (*v1).palette_index as i32 & 0xff00) >> 8) as u16
            );
            if (*v1).ai_var_A == 0 {
                let SamusEnemyDelta_Y: u16 =
                    GetSamusEnemyDelta_Y(word!(game.cur_enemy_index), game);
                if ((Abs16(SamusEnemyDelta_Y)) as i16).wrapping_sub(((*v1).parameter_1) as i16) < 0
                {
                    let SamusEnemyDelta_X: u16 =
                        GetSamusEnemyDelta_X(word!(game.cur_enemy_index), game);
                    (*v1).ai_var_B = SamusEnemyDelta_X;
                    if ((Abs16(SamusEnemyDelta_X)) as i16).wrapping_sub(((*v1).parameter_1) as i16)
                        < 0
                        && (*v1).ai_var_B & 0x8000 != (*v1).parameter_2
                    {
                        let mut v4: u16 = addr_kBlueBrinstarFaceBlock_Ilist_E80C as u16;
                        if ((*v1).ai_var_B & 0x8000) == 0 {
                            v4 = addr_kBlueBrinstarFaceBlock_Ilist_E81A as u16;
                        }
                        enemy_instruction! {[(*v1)]; instruction = v4, timer = 1}
                        (*v1).ai_var_A = 1;
                        word!(game.variables_for_enemy_graphics_drawn_hook[2] = 16);
                    }
                }
            }
        }
    };
    [11069725] BlueBrinstarFaceBlock_Shot [] {
        word!(game.projectile_dir[word!(game.collision_detection_index)] &= !(0x10));
    };
    [11073281] KiHunter_Shot [] {
        NormalEnemyShotAi(game);
        let old_cur_enemy_index: u16 = word!(game.cur_enemy_index);
        let EK = Get_KiHunter(word!(game.cur_enemy_index) as i32, game);
        let E1 = Get_KiHunter(word!(game.cur_enemy_index) as i32 + 64, game);
        let health: u16 = (*EK).base.health;
        if health != 0 {
            if health == (*E1).parameter_1
                || ((health) as i16).wrapping_sub(((*E1).parameter_1) as i16) < 0
            {
                if (*EK).var_14 == 0 {
                    set_fields! {[(*EK)]; var_14 = 1, var_A = ((0xa8f55ai32) as u16), var_08 = 0,
                    var_09 = 1, };
                    word!(game.cur_enemy_index = (old_cur_enemy_index).wrapping_add(64));
                    if (*E1).var_A as i32 != 0xa8f7cf & 0xffff {
                        (*E1).var_07 = (*E1).base.y_pos;
                        (*E1).var_08 = (*E1).base.x_pos;
                        KiHunter_Func_17(game);
                        KiHunter_Func_12(game);
                        KiHunter_Func_13(game);
                        set_fields! {[(*E1)]; var_F = -8192i32 as u16,
                        var_A = ((0xa8f7cfi32) as u16), var_00 = ((0xa8f7dbi32) as u16),
                        var_06 = ((*E1).var_07).wrapping_sub((*E1).var_0B),
                        var_05 = (*E1).base.x_pos, var_B = (*E1).var_0A, };
                        (*E1).base.current_instruction = addr_kKiHunter_Ilist_EA7E as u16;
                        (*E1).base.spritemap_pointer = addr_kSpritemap_Nothing_A8 as u16;
                        (*E1).base.instruction_timer = 1;
                        (*E1).base.timer = 0;
                        (*E1).base.properties =
                            ((*E1).base.properties) | ((kEnemyProps_ProcessedOffscreen) as u16);
                    }
                    word!(game.cur_enemy_index = old_cur_enemy_index);
                }
            } else {
                (*E1).base.ai_handler_bits = (*EK).base.ai_handler_bits;
                (*E1).base.frozen_timer = (*EK).base.frozen_timer;
                (*E1).base.invincibility_timer = (*EK).base.invincibility_timer;
                (*E1).base.flash_timer = (*EK).base.flash_timer;
            }
        } else {
            (*E1).base.properties = 512;
        };
    };
    @functions [pub unsafe fn (game: &mut Game)];
    @prefix { let E = Get_NorfairLavaMan(word!(game.cur_enemy_index) as i32, game); };
    NorfairLavaMan_Func_1 [] {
        set_fields! {[(*E)], 0; var_C, var_00, var_02}
        (*E).var_03 = (*E).base.y_pos;
        if (GetSamusEnemyDelta_X(word!(game.cur_enemy_index), game) & 0x8000) != 0 {
            (*E).var_00 = 1;
        }
        (*E).var_D = addr_kNorfairLavaMan_Ilist_AC9C as u16;
        if (*E).var_00 == 0 {
            (*E).var_D = addr_kNorfairLavaMan_Ilist_AD3C as u16;
        }
        NorfairLavaMan_Func_20(game);
        (*E).var_F = ((0xa8b11ai32) as u16);
    };
    NorfairLavaMan_Func_2 [] {
        (*E).var_03 = (*E).base.y_pos;
        set_fields! {[(*E)], 0; var_C, var_0C}
        (*E).var_0D = 1;
        (*E).var_D = addr_kNorfairLavaMan_Ilist_ADDC as u16;
        NorfairLavaMan_Func_20(game);
        (*E).base.y_pos = ((*E).base.y_pos).wrapping_add(32);
        (*E).var_F = ((0xa8b1ddi32) as u16);
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
    };
    NorfairLavaMan_Func_3 [] {
        (*E).var_03 = (*E).base.y_pos;
        (*E).var_11 = (*E).base.x_pos;
        set_fields! {[(*E)], 0; var_C, var_04}
        (*E).var_D = addr_kNorfairLavaMan_Ilist_AE0C as u16;
        NorfairLavaMan_Func_20(game);
        (*E).base.y_pos = ((*E).base.y_pos).wrapping_add(32);
        (*E).var_F = ((0xa8b30di32) as u16);
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
    };
    NorfairLavaMan_Func_4 [] {
        let v1: i32 = 8 * byte!((*E).parameter_2; 1) as i32 >> 1;
        (*E).var_08 = crate::rtl::rom_table_word!(game, 0xa28187, v1 as isize);
        (*E).var_07 = crate::rtl::rom_table_word!(game, 0xa28187, (v1 + 1) as isize);
        (*E).var_0A = crate::rtl::rom_table_word!(game, 0xa28187, (v1 + 2) as isize);
        (*E).var_09 = crate::rtl::rom_table_word!(game, 0xa28187, (v1 + 3) as isize);
    };
    [11055370] NorfairLavaMan_Main [] {
        (*E).var_04 = (*E).var_04.wrapping_sub(1);
        CallEnemyPreInstr((((*E).var_F) as u32) | (0xa80000u32), game);
    };
    NorfairLavaMan_Func_13 [] {
        let mut v2 = gEnemySpawnData(word!(game.cur_enemy_index) as i32, game);
        let v4: bool = u16::overflowing_add((*v2.offset(31)).field_E, (*E).base.y_subpos).1;
        (*E).base.y_subpos = ((*v2.offset(31)).field_E).wrapping_add((*E).base.y_subpos);
        (*E).base.y_pos = ((*E).base.y_pos as i32 + ((*v2.offset(31)).xpos2 as i32 + v4 as i32)) as u16;
        (*E).var_E = ((*E).var_E as i32 + ((*v2.offset(31)).xpos2 as i32 + v4 as i32)) as u16;
        NorfairLavaMan_Func_14(word!(game.cur_enemy_index), game);
    };
    NorfairLavaMan_Func_20 [] {
        let nlmn_var_D: u16 = (*E).var_D;
        if nlmn_var_D != (*E).var_C {
            (*E).base.current_instruction = nlmn_var_D;
            (*E).var_C = nlmn_var_D;
            (*E).base.instruction_timer = 1;
            (*E).base.timer = 0;
        }
    };
    NorfairLavaMan_Common [] {
        let E1 = Get_NorfairLavaMan(word!(game.cur_enemy_index) as i32 + 64, game);
        let E2 = Get_NorfairLavaMan(word!(game.cur_enemy_index) as i32 + 128, game);
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
    };
    @forward [pub unsafe fn (game: &mut Game)];
    [11042816] Enemy_GrappleReact_NoInteract_A8 => SwitchEnemyAiToMainAi(game);
    [11042826] Enemy_GrappleReact_KillEnemy_A8 => EnemyGrappleDeath(game);
    [11042831] Enemy_GrappleReact_CancelBeam_A8 => Enemy_SwitchToFrozenAi(game);
    [11042836]
    Enemy_GrappleReact_SamusLatchesNoInvinc_A8 => SamusLatchesOnWithGrappleNoInvinc(game);
    [11042846] Enemy_GrappleReact_HurtSamus_A8 => SamusHurtFromGrapple(game);
    [11042851] Enemy_NormalTouchAI_A8 => NormalEnemyTouchAi(game);
    [11042861] Enemy_NormalShotAI_A8 => NormalEnemyShotAi(game);
    Enemy_NormalPowerBombAI_A8 => NormalEnemyPowerBombAi(game);
    [11042881] Enemy_NormalFrozenAI_A8 => NormalEnemyFrozenAI(game);
}

crate::game::callback_dispatch! {
    pub(crate) unsafe fn DispatchEnemyInstr(ea: u32, k: u16, j: *const u16, game: &mut Game) -> *const u16 => ea;
    call (k, j, game); default { crate::rtl::Unreachable(game); core::ptr::null::<u16>() };
    @functions [pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16];
    [11044751] MiniDraygon_Instr_2 [jp] { QueueSfx2_Max6(0x5e, game); };
    [11044763] MiniDraygon_Instr_1 [jp] {
        if (*gEnemySpawnData(word!(game.cur_enemy_index) as i32, game).offset(30)).cause_of_death
            as i32
            == addr_kMiniDraygon_Ilist_870B as i32
        {
            (*Get_MiniDraygon(word!(game.cur_enemy_index) as i32, game)).var_F = -8i32 as u16;
        } else {
            (*Get_MiniDraygon(word!(game.cur_enemy_index) as i32, game)).var_F = 8;
        }
    };
    [11044790] MiniDraygon_Instr_3 [jp] {
        let E = Get_MiniDraygon(word!(game.cur_enemy_index) as i32, game);
        if (*gEnemySpawnData(word!(game.cur_enemy_index) as i32, game).offset(30)).cause_of_death
            as i32
            == addr_kMiniDraygon_Ilist_870B as i32
        {
            (*E).var_F = (*E).var_F.wrapping_add(1);
        } else {
            (*E).var_F = (*E).var_F.wrapping_sub(1);
        }
    };
    [11044811] MiniDraygon_Instr_4 [jp] {
        let E = Get_MiniDraygon(word!(game.cur_enemy_index) as i32, game);
        set_fields! {[(*E)], 0; var_0C, var_0B}
        (*E).var_C = ((0xa88a34i32) as u16);
    };
    @functions [pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16];
    [11048485] Fune_Instr_2 [jp] { QueueSfx2_Max9(0x1f, game); };
    @functions [pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16];
    @prefix {
        word!(
            game.cinematic_var9 = *(&raw mut (*(Get_Fune
                as unsafe fn(i32, &mut Game) -> *mut Enemy_Fune)(
                word!(game.cur_enemy_index) as i32,
                game,
            ))
            .parameter_2 as *mut u8)
                 as u16
        );
    };
    [11048497] Fune_Instr_6 [jp] {

        SpawnEprojWithGfx(
            0,
            word!(game.cur_enemy_index),
            addr_stru_86DFBC as u16,
            game,
        );
    };
    [11048522] Fune_Instr_7 [jp] {

        SpawnEprojWithGfx(
            1,
            word!(game.cur_enemy_index),
            addr_stru_86DFBC as u16,
            game,
        );
    };
    [11048547] Fune_Instr_1 [jp] {

        SpawnEprojWithGfx(
            0,
            word!(game.cur_enemy_index),
            addr_stru_86DFCA as u16,
            game,
        );
    };
    [11048572] Fune_Instr_4 [jp] {

        SpawnEprojWithGfx(
            1,
            word!(game.cur_enemy_index),
            addr_stru_86DFCA as u16,
            game,
        );
    };
    @functions [pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16];
    [11048597] Fune_Instr_3 [jp] {
        let E = Get_Fune(word!(game.cur_enemy_index) as i32, game);
        (*E).var_A = ((*E).var_A).wrapping_add(4);
        (*E).var_B = ((0xa89737i32) as u16);
        if (*E).var_D != 0 {
            (*E).var_B = ((0xa8975ci32) as u16);
        }
    };
    [11048628] Fune_Instr_5 [jp] {
        let E = Get_Fune(word!(game.cur_enemy_index) as i32, game);
        (*E).var_A = ((*E).var_A).wrapping_add(4);
        (*E).var_B = ((0xa89737i32) as u16);
        if (*E).var_D != 0 {
            (*E).var_B = ((0xa8975ci32) as u16);
        }
    };
    @functions [pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16];
    [11051207] YappingMaw_Instr_2 [jp] {
        let E = Get_YappingMaw(word!(game.cur_enemy_index) as i32, game);
        (*E).var_32 = crate::rtl::rom_table_word!(game, 0xa8a0ab, 0);
        (*E).var_33 = crate::rtl::rom_table_word!(game, 0xa8a0ad, 0);
    };
    [11051225] YappingMaw_Instr_4 [jp] {
        let E = Get_YappingMaw(word!(game.cur_enemy_index) as i32, game);
        (*E).var_32 = crate::rtl::rom_table_word!(game, 0xa8a0c3, 0);
        (*E).var_33 = crate::rtl::rom_table_word!(game, 0xa8a0c5, 0);
    };
    [11051243] YappingMaw_Instr_5 [jp] {
        let E = Get_YappingMaw(word!(game.cur_enemy_index) as i32, game);
        (*E).var_32 = crate::rtl::rom_table_word!(game, 0xa8a0b3, 0);
        (*E).var_33 = crate::rtl::rom_table_word!(game, 0xa8a0b5, 0);
    };
    [11051261] YappingMaw_Instr_7 [jp] {
        let E = Get_YappingMaw(word!(game.cur_enemy_index) as i32, game);
        (*E).var_32 = crate::rtl::rom_table_word!(game, 0xa8a0bb, 0);
        (*E).var_33 = crate::rtl::rom_table_word!(game, 0xa8a0bd, 0);
    };
    [11051279] YappingMaw_Instr_3 [jp] {
        let E = Get_YappingMaw(word!(game.cur_enemy_index) as i32, game);
        (*E).var_32 = crate::rtl::rom_table_word!(game, 0xa8a0a7, 0);
        (*E).var_33 = crate::rtl::rom_table_word!(game, 0xa8a0a9, 0);
    };
    [11051297] YappingMaw_Instr_6 [jp] {
        let E = Get_YappingMaw(word!(game.cur_enemy_index) as i32, game);
        (*E).var_32 = crate::rtl::rom_table_word!(game, 0xa8a0b7, 0);
        (*E).var_33 = crate::rtl::rom_table_word!(game, 0xa8a0b9, 0);
    };
    [11051315] YappingMaw_Instr_1 [jp] {
        if (*Get_YappingMaw(word!(game.cur_enemy_index) as i32, game)).var_36 == 0 {
            QueueSfx2_Max6(0x2f, game);
        }
    };
    @functions [pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16];
    [11054610] NorfairLavaMan_Instr_1 [jp.offset(1)] {
        if CheckIfEnemyIsOnScreen(game) == 0 {
            QueueSfx2_Max6(*jp, game);
        }
    };
    [11054630] NorfairLavaMan_Instr_8 [jp] {
        let E = Get_NorfairLavaMan(word!(game.cur_enemy_index) as i32, game);
        (*E).base.y_pos = (*E).base.y_pos.wrapping_add(1);
        (*E).base.y_pos = (*E).base.y_pos.wrapping_add(1);
    };
    [11054640] NorfairLavaMan_Instr_14 [jp] {
        let E = Get_NorfairLavaMan(word!(game.cur_enemy_index) as i32, game);
        (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
        (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
    };
    [11054650] NorfairLavaMan_Instr_2 [jp] {
        (*Get_NorfairLavaMan(word!(game.cur_enemy_index) as i32, game)).var_01 = 1;
    };
    [11054661] NorfairLavaMan_Instr_7 [jp] {
        (*Get_NorfairLavaMan(word!(game.cur_enemy_index) as i32, game)).var_01 = 0;
    };
    [11054672] NorfairLavaMan_Instr_10 [jp] {
        let E = Get_NorfairLavaMan(word!(game.cur_enemy_index) as i32, game);
        (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
        let E1 = Get_NorfairLavaMan(word!(game.cur_enemy_index) as i32 + 64, game);
        (*E1).base.y_pos = (*E1).base.y_pos.wrapping_sub(1);
    };
    [11054682] NorfairLavaMan_Instr_12 [jp] {
        let E = Get_NorfairLavaMan(word!(game.cur_enemy_index) as i32, game);
        (*E).base.y_pos = (*E).base.y_pos.wrapping_add(1);
        let E1 = Get_NorfairLavaMan(word!(game.cur_enemy_index) as i32 + 64, game);
        (*E1).base.y_pos = (*E1).base.y_pos.wrapping_add(1);
    };
    [11054692] NorfairLavaMan_Instr_9 [jp] {
        let E = Get_NorfairLavaMan(word!(game.cur_enemy_index) as i32, game);
        let v4: u16 = ((*E).var_03).wrapping_add(24);
        (*E).base.y_pos = v4;
        let E1 = Get_NorfairLavaMan(word!(game.cur_enemy_index) as i32 + 64, game);
        (*E1).base.y_pos = v4;
        (*E1).base.properties = ((*E1).base.properties & !(0x100));
        let E2 = Get_NorfairLavaMan(word!(game.cur_enemy_index) as i32 + 128, game);
        (*E2).base.properties = ((*E2).base.properties & !(0x100));
    };
    [11054728] NorfairLavaMan_Instr_11 [jp] {
        let E = Get_NorfairLavaMan(word!(game.cur_enemy_index) as i32, game);
        let nlmn_var_03: u16 = (*E).var_03;
        (*E).base.y_pos = nlmn_var_03;
        (*Get_NorfairLavaMan(word!(game.cur_enemy_index) as i32 + 64, game))
            .base
            .y_pos = nlmn_var_03;
    };
    [11054742] NorfairLavaMan_Instr_13 [jp] {
        let E = Get_NorfairLavaMan(word!(game.cur_enemy_index) as i32, game);
        let v4: u16 = ((*E).var_03).wrapping_add(4);
        (*E).base.y_pos = v4;
        let E1 = Get_NorfairLavaMan(word!(game.cur_enemy_index) as i32 + 64, game);
        (*E1).base.y_pos = v4;
        (*E1).base.properties = ((*E1).base.properties) | (0x100u16);
        let E2 = Get_NorfairLavaMan(word!(game.cur_enemy_index) as i32 + 128, game);
        (*E2).base.properties = ((*E2).base.properties) | (0x100u16);
    };
    [11054778] NorfairLavaMan_Instr_5 [jp] {
        let E = Get_NorfairLavaMan(word!(game.cur_enemy_index) as i32, game);
        SpawnEprojWithGfx(
            (*E).var_B,
            word!(game.cur_enemy_index),
            addr_kEproj_LavaThrownByLavaman as u16,
            game,
        );
    };
    [11054794] NorfairLavaMan_Instr_15 [jp] {
        let E = Get_NorfairLavaMan(word!(game.cur_enemy_index) as i32, game);
        (*E).base.x_pos = ((*E).var_12).wrapping_add(8);
        (*E).base.y_pos = ((*E).var_13).wrapping_sub(4);
    };
    [11054820] NorfairLavaMan_Instr_4 [jp] {
        let E = Get_NorfairLavaMan(word!(game.cur_enemy_index) as i32, game);
        (*E).base.x_pos = ((*E).var_12).wrapping_sub(8);
        (*E).base.y_pos = ((*E).var_13).wrapping_sub(4);
    };
    [11054846] NorfairLavaMan_Instr_16 [jp] {
        let E = Get_NorfairLavaMan(word!(game.cur_enemy_index) as i32, game);
        (*E).base.x_pos = ((*E).var_12).wrapping_add(8);
        (*E).base.y_pos = ((*E).var_13).wrapping_sub(8);
    };
    [11054872] NorfairLavaMan_Instr_6 [jp] {
        let E = Get_NorfairLavaMan(word!(game.cur_enemy_index) as i32, game);
        (*E).base.x_pos = ((*E).var_12).wrapping_sub(8);
        (*E).base.y_pos = ((*E).var_13).wrapping_sub(4);
    };
    [11054916] NorfairLavaMan_Instr_3 [jp] {
        (*Get_NorfairLavaMan(word!(game.cur_enemy_index) as i32, game)).var_04 = 256;
    };
    @functions [pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16];
    [11063441] WreckedShipRobot_Instr_3 [] {
        let E = Get_WreckedShipRobot(word!(game.cur_enemy_index) as i32, game);
        if ((word!(game.layer1_x_pos)) as i16).wrapping_sub(((*E).base.x_pos) as i16) < 0
            && ((word!(game.layer1_x_pos) as i32 + 256 - (*E).base.x_pos as i32) as i16) >= 0
            && ((word!(game.layer1_y_pos)) as i16).wrapping_sub(((*E).base.y_pos) as i16) < 0
            && ((word!(game.layer1_y_pos) as i32 + 224 - (*E).base.y_pos as i32) as i16) >= 0
        {
            QueueSfx2_Max6(0x68, game);
        }
        jp
    };
    [11063659] WreckedShipRobot_Instr_11 [] {
        WreckedShipRobot_Func_2(word!(game.cur_enemy_index), game);
        jp
    };
    [11072806] KiHunter_Instr_1 [] {
        let mut result: u16 = addr_kKiHunter_Ilist_E9FA as u16;
        let mut r18: u16 = addr_kKiHunter_Ilist_EA4E as u16;
        let E = Get_KiHunter(k as i32, game);
        if ((*E).var_07 & 0x8000) == 0 {
            result = addr_kKiHunter_Ilist_EA24 as u16;
            r18 = addr_kKiHunter_Ilist_EA5E as u16;
        }
        enemy_instruction! {[(*E).base]; instruction = r18, timer = 1}
        let E1 = Get_KiHunter(k as i32 + 64, game);
        if (*E1).var_A as i32 == 0xa8f6f3 & 0xffff {
            enemy_instruction! {[(*E1).base]; instruction = r18, timer = 1}
        }
        result as usize as *const u16
    };
    [11072996] KiHunter_Instr_2 [] {
        (*Get_KiHunter(word!(game.cur_enemy_index) as i32, game)).var_A = ((0xa8f5f0i32) as u16);
        jp
    };
    [11073151] KiHunter_Instr_3 [] {
        (*Get_KiHunter(word!(game.cur_enemy_index) as i32, game)).var_A = ((0xa8f68bi32) as u16);
        jp
    };
    @functions [pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16];
    [11067164] WalkingLavaSeahorse_Instr_4 [jp] { WalkingLavaSeahorse_DF20(0, game); };
    @functions [pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16];
    [11067187] WalkingLavaSeahorse_Instr_3 [jp] { WalkingLavaSeahorse_DF20(2, game); };
    [11067193] WalkingLavaSeahorse_Instr_5 [jp] { WalkingLavaSeahorse_DF20(4, game); };
    [11067199]
    WalkingLavaSeahorse_Instr_6 [addr_kWalkingLavaSeahorse_Ilist_DBE7 as i32 as usize as *const u16] {
        let E = Get_WalkingLavaSeahorse(word!(game.cur_enemy_index) as i32, game);
        (*E).var_A = ((0xa8de4bi32) as u16);
        let mut v3: i16 = ((word!(game.random_number)) as i16) & (3i16);
        if (word!(game.random_number) & 3) == 0 {
            v3 = 2;
        }
        (*E).var_04 = v3 as u16;
        if ((*E).var_D & 0x8000) == 0 {
            return addr_kWalkingLavaSeahorse_Ilist_DC57 as i32 as usize as *const u16;
        }
    };
    [11067235]
    WalkingLavaSeahorse_Instr_2 [WalkingLavaSeahorse_Instr_1(word!(game.cur_enemy_index), jp, game)] {
        let E = Get_WalkingLavaSeahorse(word!(game.cur_enemy_index) as i32, game);
        let wlse_var_04: u16 = (*E).var_04;
        if wlse_var_04 != 0 {
            (*E).var_04 = (wlse_var_04).wrapping_sub(1);
        }
    };
    @functions [pub unsafe fn (
    k: u16,
    mut jp: *const u16,
    game: &mut Game,
) -> *const u16];
    [11067249] WalkingLavaSeahorse_Instr_1 [] {
        let E = Get_WalkingLavaSeahorse(word!(game.cur_enemy_index) as i32, game);
        if Enemy_MoveRight_IgnoreSlopes(
            word!(game.cur_enemy_index),
            ((*E).var_D as i16 as i32) << 16,
            game,
        ) != 0
        {
            jp = addr_stru_A8DBE9 as i32 as usize as *const u16;
            let mut wlse_var_D: i16 = (*E).var_D as i16;
            if wlse_var_D < 0 {
                jp = addr_stru_A8DC59 as i32 as usize as *const u16;
            }
            (*E).var_D = -(wlse_var_D as i32) as u16;
        } else {
            EnemyFunc_C8AD(word!(game.cur_enemy_index), game);
        }
        jp
    };
    @functions [pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16];
    [11068957] WreckedShipSpark_Instr_2 [jp] {
        let E = Get_WreckedShipSpark(word!(game.cur_enemy_index) as i32, game);
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
    };
    [11068970] WreckedShipSpark_Instr_1 [jp] {
        let E = Get_WreckedShipSpark(word!(game.cur_enemy_index) as i32, game);
        (*E).base.properties = ((*E).base.properties as i32 & !(kEnemyProps_Tangible as i32)) as u16;
    };
    @functions [pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16];
    [11073234] KiHunter_Instr_4 [jp] { sub_A8F6DC(k, addr_loc_A8CF18 as u16, game); };
    [11073240] KiHunter_Instr_5 [jp] { sub_A8F6DC(k, addr_loc_A8CF26 as u16, game); };
    @forward [pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16];
    [11062537] WreckedShipRobot_Instr_4 => move_wrecked_ship_robot(
        jp,
        (0xfe00, -4),
        (
            addr_kWreckedShipRobot_Ilist_C73F as u16,
            Some(addr_kWreckedShipRobot_Ilist_CB65 as u16),
        ),
        false,
        game,
    );
    [11062692] WreckedShipRobot_Instr_9 => move_wrecked_ship_robot(
        jp,
        (0xfe00, -4),
        (addr_kWreckedShipRobot_Ilist_C73F as u16, None),
        false,
        game,
    );
    [11062762] WreckedShipRobot_Instr_6 => move_wrecked_ship_robot(
        jp,
        (0xfe00, 4),
        (
            addr_stru_A8C6E9 as u16,
            Some(addr_kWreckedShipRobot_Ilist_C91B as u16),
        ),
        false,
        game,
    );
    [11062917] WreckedShipRobot_Instr_8 => move_wrecked_ship_robot(
        jp,
        (0xfe00, 4),
        (addr_stru_A8C6E9 as u16, None),
        false,
        game,
    );
    @forward [pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16];
    [11062991] WreckedShipRobot_Instr_15 => move_wrecked_ship_robot(
        jp,
        (512, 4),
        (
            addr_kWreckedShipRobot_Ilist_C985 as u16,
            Some(addr_kWreckedShipRobot_Ilist_C91B as u16),
        ),
        false,
        game,
    );
    [11063146] WreckedShipRobot_Instr_18 => move_wrecked_ship_robot(
        jp,
        (512, 4),
        (addr_kWreckedShipRobot_Ilist_C985 as u16, None),
        false,
        game,
    );
    [11063216] WreckedShipRobot_Instr_16 => move_wrecked_ship_robot(
        jp,
        (512, -4),
        (
            addr_stru_A8C6E9 as u16,
            Some(addr_kWreckedShipRobot_Ilist_CB65 as u16),
        ),
        true,
        game,
    );
    [11063371]
    WreckedShipRobot_Instr_17 => move_wrecked_ship_robot(jp, (512, -4), (addr_stru_A8C6E9 as u16, None), false, game);
    @forward [pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16];
    [11063494] WreckedShipRobot_Instr_14 => WreckedShipRobot_CommonInstr(
        k,
        jp,
        addr_kWreckedShipRobot_Ilist_CB1D as u16,
        addr_kEproj_WreckedShipRobotLaserUpRight as u16,
        game,
    );
    [11063506] WreckedShipRobot_Instr_2 => WreckedShipRobot_CommonInstr(
        k,
        jp,
        addr_kWreckedShipRobot_Ilist_C8D1 as u16,
        addr_kEproj_WreckedShipRobotLaserUpLeft as u16,
        game,
    );
    @forward [pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16];
    [11063552]
    WreckedShipRobot_Instr_13 => WreckedShipRobot_D10C(k, jp, addr_kWreckedShipRobot_Ilist_CB09 as u16, game);
    [11063559]
    WreckedShipRobot_Instr_1 => WreckedShipRobot_D10C(k, jp, addr_kWreckedShipRobot_Ilist_C8BD as u16, game);
    @forward [pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16];
    [11063601] WreckedShipRobot_Instr_12 => WreckedShipRobot_D147(
        k,
        jp,
        addr_kEproj_WreckedShipRobotLaserDownRight as u16,
        addr_kWreckedShipRobot_Ilist_CAFD as u16,
        game,
    );
    [11063613] WreckedShipRobot_Instr_5 => WreckedShipRobot_D147(
        k,
        jp,
        addr_kEproj_WreckedShipRobotLaserDownLeft as u16,
        addr_kWreckedShipRobot_Ilist_C8B1 as u16,
        game,
    );
}

crate::game::callback_dispatch! {
    pub(crate) unsafe fn DispatchEnemyPreInstr(ea: u32, k: u16, game: &mut Game) => ea;
    call (k, game); default { crate::rtl::Unreachable(game); };
    @functions [pub unsafe fn (k: u16, game: &mut Game)];
    [11047153] MorphBallEye_Func_1 [] {
        if IsSamusWithinEnemy_Y(k, g_word_A89054, game) != 0
            && IsSamusWithinEnemy_X(k, g_word_A89050, game) != 0
        {
            let E = Get_MorphBallEye(k as i32, game);
            (*E).var_E = 32;
            (*E).base.instruction_timer = 1;
            if ((*E).parameter_1 & 1) != 0 {
                (*E).base.current_instruction = addr_kMorphBallEye_Ilist_9026 as u16;
            } else {
                (*E).base.current_instruction = addr_kMorphBallEye_Ilist_9014 as u16;
            }
            (*E).var_F = ((0xa8912ei32) as u16);
        }
    };
    [11047264] MorphBallEye_Func_3 [] {
        let E = Get_MorphBallEye(k as i32, game);
        if IsSamusWithinEnemy_Y(k, g_word_A89056, game) != 0
            && IsSamusWithinEnemy_X(k, g_word_A89052, game) != 0
        {
            let v3: u16 = CalculateAngleFromXY(
                (word!(game.samus_x_pos)).wrapping_sub((*E).base.x_pos),
                (word!(game.samus_y_pos)).wrapping_sub((*E).base.y_pos),
                game,
            );
            (*E).var_D = v3;
            (*E).base.current_instruction = (((v3 as i32 & 0xf0) >> 2) - 28756i32) as u16;
        } else {
            QueueSfx2_Max6(0x71, game);
            (*E).var_C = 0;
            (*E).var_E = 32;
            if ((*E).parameter_1 & 1) != 0 {
                (*E).base.current_instruction = addr_kMorphBallEye_Ilist_9002 as u16;
            } else {
                (*E).base.current_instruction = addr_kMorphBallEye_Ilist_8FF0 as u16;
            }
            (*E).var_F = ((0xa891cei32) as u16);
        }
        (*E).base.instruction_timer = 1;
    };
    [11055885] NorfairLavaMan_Func_15 [] {
        if *(ram_ptr!(game, u16; 0xe84))
            .offset(((word!(game.cur_enemy_index) as i32 >> 1) + 87) as isize) as i32
            == 0xa8b175 & 0xffff
        {
            (*Get_NorfairLavaMan(word!(game.cur_enemy_index) as i32, game)).var_F =
                ((0xa8b356i32) as u16);
        }
    };
    [11055958] NorfairLavaMan_Func_17 [] {
        if (*gEnemySpawnData(word!(game.cur_enemy_index) as i32, game).offset(31)).field_18 != 0 {
            let E = Get_NorfairLavaMan(word!(game.cur_enemy_index) as i32, game);
            (*E).var_D = addr_kNorfairLavaMan_Ilist_ACB0 as u16;
            (*E).var_B = 0;
            if (GetSamusEnemyDelta_X(word!(game.cur_enemy_index), game) & 0x8000) == 0 {
                (*E).var_D = addr_kNorfairLavaMan_Ilist_AD50 as u16;
                (*E).var_B = 1;
            }
            NorfairLavaMan_Func_20(game);
            (*E).var_F = ((0xa8b31fi32) as u16);
            (*E).var_12 = (*E).base.x_pos;
            (*E).var_13 = (*E).base.y_pos;
        } else {
            (*Get_NorfairLavaMan(word!(game.cur_enemy_index) as i32, game))
                .base
                .y_pos = (*(ram_ptr!(game, u16; 0xe84))
                .offset(((word!(game.cur_enemy_index) as i32 >> 1i32) + 93i32) as isize))
            .wrapping_sub(crate::rtl::rom_table_word!(
                game,
                0xa8af79,
                (*(ram_ptr!(game, u16; 0xf68))
                    .offset(((word!(game.cur_enemy_index) as i32 >> 1) + 1) as isize)
                    >> 1) as isize,
            ));
        }
        NorfairLavaMan_Func_19(word!(game.cur_enemy_index), game);
    };
    [11056039] NorfairLavaMan_Func_18 [] {
        let v1: i32 = word!(game.cur_enemy_index) as i32 >> 1;
        if word!(game.enemy_drawing_queue[(v1 + 87)]) as i32 == 0xa8b11a & 0xffff {
            (*Get_NorfairLavaMan(word!(game.cur_enemy_index) as i32, game)).var_F =
                ((0xa8b30di32) as u16);
        } else {
            (*Get_NorfairLavaMan(word!(game.cur_enemy_index) as i32, game))
                .base
                .y_pos = (word!(game.enemy_drawing_queue[(v1 + 93)])).wrapping_sub(
                crate::rtl::rom_table_word!(
                    game,
                    0xa8af79,
                    (word!(game.enemy_drawing_queue_sizes[(v1 + 1)]) >> 1) as isize,
                ),
            );
        }
        NorfairLavaMan_Func_19(word!(game.cur_enemy_index), game);
    };
    [11060585] MaridiaFloater_Func_12 [] {
        let v1: i32 = k as i32 >> 1;
        if word!(game.enemy_drawing_queue[(v1 + 103)]) as i32 + 0x3e6f & 0x8000 == 0 {
            let v2: u16 =
                ((word!(game.enemy_drawing_queue[(v1 + 103)]) as i32 - 4i32 + 15983) as u16 >> 1);
            if v2 != 0 {
                word!(
                    game.enemy_drawing_queue[(v1 + 103)] =
                        crate::rtl::rom_table_word!(game, 0xa8c599, (v2 >> 1) as isize)
                );
                word!(game.enemy_drawing_queue[(v1 + 104)] = 1);
            }
        }
        let E = Get_MaridiaFloater(k as i32, game);
        (*E).var_F = ((0xa8c59fi32) as u16);
        (*E).var_E = 32;
        MaridiaFloater_Func_2(k, game);
    };
    @functions [pub unsafe fn (k: u16, game: &mut Game)];
    @prefix { let E = Get_NorfairLavaMan(word!(game.cur_enemy_index) as i32, game); };
    [11055386] NorfairLavaMan_Func_7 [] {
        (*E).var_00 = 0;
        (*E).var_D = addr_kNorfairLavaMan_Ilist_AC9C as u16;
        if (GetSamusEnemyDelta_X(word!(game.cur_enemy_index), game) & 0x8000) == 0 {
            (*E).var_D = addr_kNorfairLavaMan_Ilist_AD3C as u16;
            (*E).var_00 = 1;
        }
        NorfairLavaMan_Func_20(game);
        let E2 = Get_NorfairLavaMan(word!(game.cur_enemy_index) as i32 + 128, game);
        if ((*E2).var_04 & 0x8000) != 0 {
            (*E2).var_04 = 0;
            if IsSamusWithinEnemy_X(
                word!(game.cur_enemy_index),
                byte!((*E).parameter_2; 0) as u16,
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
    };
    [11055477] NorfairLavaMan_Func_8 [] {
        if (*E).var_01 == 0 {
            (*E).var_02 = 1;
            (*Get_NorfairLavaMan(word!(game.cur_enemy_index) as i32 + 64i32, game)).var_0D = 0;
            (*E).var_F = ((0xa8b193i32) as u16);
        }
    };
    [11055507] sub_A8B193 [] {
        if (*E).var_02 == 0 {
            (*E).var_D = addr_kNorfairLavaMan_Ilist_AD0C as u16;
            if (GetSamusEnemyDelta_X(word!(game.cur_enemy_index), game) & 0x8000) == 0 {
                (*E).var_D = addr_kNorfairLavaMan_Ilist_ADAC as u16;
            }
            NorfairLavaMan_Func_20(game);
            (*E).var_F = ((0xa8b1b8i32) as u16);
        }
    };
    [11055544] NorfairLavaMan_Func_9 [] {
        if (*E).var_01 == 0 {
            (*E).var_D = addr_kNorfairLavaMan_Ilist_AC9C as u16;
            if (GetSamusEnemyDelta_X(word!(game.cur_enemy_index), game) & 0x8000) == 0 {
                (*E).var_D = addr_kNorfairLavaMan_Ilist_AD3C as u16;
            }
            NorfairLavaMan_Func_20(game);
            (*E).var_F = ((0xa8b11ai32) as u16);
        }
    };
    [11055581] NorfairLavaMan_Func_10 [] {
        if (*E).var_0D == 0 {
            (*E).var_F = ((0xa8b204i32) as u16);
            set_fields! {[(*E)], 0; var_E, var_0D}
            (*E).var_B = 2;
            (*E).base.y_pos = (*E).var_03;
        }
    };
    [11055620] NorfairLavaMan_Func_11 [] {
        let v3 = gEnemySpawnData(word!(game.cur_enemy_index) as i32, game);
        let v: u32 = (((*v3.offset(31)).ypos2) as u32).wrapping_add(((*E).base.y_subpos) as u32);
        (*E).base.y_subpos = v as u16;
        (*E).base.y_pos = ((*E).base.y_pos as u32)
            .wrapping_add(((*v3.offset(31)).field_14 as u32).wrapping_add(v >> 16))
            as u16;
        (*E).var_E = ((*E).var_E as u32)
            .wrapping_add(((*v3.offset(31)).field_14 as u32).wrapping_add(v >> 16))
            as u16;
        NorfairLavaMan_Func_12(word!(game.cur_enemy_index), game);
    };
    [11055903] NorfairLavaMan_Func_16 [] {
        if (*E).var_01 == 0 {
            (*E).var_D = addr_kNorfairLavaMan_Ilist_AE0C as u16;
            NorfairLavaMan_Func_20(game);
            *(ram_ptr!(game, u16; 0xf68))
                .offset(((word!(game.cur_enemy_index) as i32 >> 1) + 5) as isize) =
                ((0xa8b295i32) as u16);
            (*E).var_F = ((0xa8b3a7i32) as u16);
            (*gEnemySpawnData(word!(game.cur_enemy_index) as i32, game).offset(31)).field_18 = 0;
            (*E).base.x_pos = (*E).var_12;
            (*E).base.y_pos = (*E).var_13;
        }
        NorfairLavaMan_Func_19(word!(game.cur_enemy_index), game);
    };
    @functions [pub unsafe fn (k: u16, game: &mut Game)];
    @prefix { let E = Get_MaridiaFloater(k as i32, game); };
    [11059919] MaridiaFloater_Func_5 [] {
        if ((*E).base.ai_handler_bits & 1) != 0 {
            (*E).var_F = ((0xa8c3e1i32) as u16);
            MaridiaFloater_Func_7(k, game);
        } else {
            advance_maridia_floater_bob!(E, k, game);
            words!((*E), var_A, var_B += words!((g_word_A8C1B9, g_word_A8C1BB)));
            if Enemy_MoveDown(k, (words!((*E), var_A, var_B)) as i32, game) != 0
                || ((*(ram_ptr!(game, u16; 0xf68)).offset(((k as i32 >> 1) + 1) as isize) as i32
                    - g_word_A8C19F as i32
                    - (*E).base.y_pos as i32) as i16)
                    >= 0
            {
                let mfr_var_C: u16 = (*E).var_C;
                if mfr_var_C == 0 || mfr_var_C == 6 {
                    (*E).var_F = ((0xa8c500i32) as u16);
                    (*E).var_E = 10;
                    let v10: i32 = k as i32 >> 1;
                    word! {game; enemy_drawing_queue[(v10 + 104)] = 1;
                        enemy_drawing_queue[(v10 + 103)] = addr_kMaridiaFloater_Ilist_C191 as u16; }
                } else {
                    (*E).var_F = ((0xa8c36bi32) as u16);
                }
            }
            MaridiaFloater_Func_2(k, game);
        };
    };
    [11060075] MaridiaFloater_Func_6 [] {
        let v2: bool = (*E).var_D == 1;
        (*E).var_D = (*E).var_D.wrapping_sub(1);
        let v3: bool = ((*E).var_D & 0x8000) != 0;
        let move_down: bool = (!v2 && !v3)
            || ({
                (*E).var_D = 5;
                let mut v4: i32 = k as i32 >> 1;
                let v5: u16 = (g_word_A8C1A1[(*E).var_C as usize])
                    .wrapping_add(word!(game.enemy_drawing_queue_sizes[v4]));
                word!(game.enemy_drawing_queue[(v4 + 91)] = v5);
                (*E).base.x_pos = v5;
                if (*E).var_C != 0 && (*E).var_C != 6 {
                    let mut v7: u16 = ((*E).var_C).wrapping_add(1);
                    if v7 as i32 - 12i32 & 0x8000 == 0 {
                        v7 = 0;
                    }
                    (*E).var_C = v7;
                    true
                } else {
                    (*E).var_F = ((0xa8c500i32) as u16);
                    (*E).var_E = 10;
                    word! {game; enemy_drawing_queue[(v4 + 104)] = 1;
                        enemy_drawing_queue[(v4 + 103)] = addr_kMaridiaFloater_Ilist_C191 as u16; }
                    false
                }
            });
        if move_down {
            words!((*E), var_A, var_B += words!((g_word_A8C1B9, g_word_A8C1BB)));
            Enemy_MoveDown(k, (words!((*E), var_A, var_B)) as i32, game);
        }
        MaridiaFloater_Func_2(k, game);
    };
    [11060193] MaridiaFloater_Func_7 [] {
        if ((*E).base.ai_handler_bits & 1) != 0 {
            advance_maridia_floater_bob!(E, k, game);
            words!((*E), var_A, var_B += words!((g_word_A8C1B9, g_word_A8C1BB)));
            if Enemy_MoveDown(k, (words!((*E), var_A, var_B)) as i32, game) != 0
                || ((*(ram_ptr!(game, u16; 0xf68)).offset(((k as i32 >> 1) + 1) as isize) as i32
                    - *(ram_ptr!(game, u16; 0xf68)).offset(((k as i32 >> 1) + 3) as isize) as i32
                    - (*E).base.y_pos as i32) as i16)
                    >= 0
            {
                if (*E).var_C == 0 || (*E).var_C == 6 {
                    (*E).var_F = ((0xa8c4dci32) as u16);
                } else {
                    (*E).var_F = ((0xa8c469i32) as u16);
                }
            }
            MaridiaFloater_Func_2(k, game);
        } else {
            (*E).var_F = ((0xa8c36bi32) as u16);
        };
    };
    [11060329] MaridiaFloater_Func_8 [] {
        if ((*E).base.ai_handler_bits & 1) == 0 {
            (*E).var_F = ((0xa8c36bi32) as u16);
            return;
        }
        let v2: bool = (*E).var_D == 1;
        (*E).var_D = (*E).var_D.wrapping_sub(1);
        let v3: bool = ((*E).var_D & 0x8000) != 0;
        let move_down: bool = (!v2 && !v3)
            || ({
                (*E).var_D = 5;
                let mut v4: i32 = k as i32 >> 1;
                let mut v5: u16 = (g_word_A8C1A1[(*E).var_C as usize])
                    .wrapping_add(word!(game.enemy_drawing_queue_sizes[v4]));
                word!(game.enemy_drawing_queue[(v4 + 91)] = v5);
                (*E).base.x_pos = v5;
                if (*E).var_C != 0 && (*E).var_C != 6 {
                    let mut v7: u16 = ((*E).var_C).wrapping_add(1);
                    if v7 as i32 - 12i32 & 0x8000 == 0 {
                        v7 = 0;
                    }
                    (*E).var_C = v7;
                    true
                } else {
                    (*E).var_F = ((0xa8c4dci32) as u16);
                    false
                }
            });
        if move_down {
            words!((*E), var_A, var_B += words!((g_word_A8C1B9, g_word_A8C1BB)));
            Enemy_MoveDown(k, (words!((*E), var_A, var_B)) as i32, game);
        }
        MaridiaFloater_Func_2(k, game);
    };
    [11060444] MaridiaFloater_Func_9 [] {
        if ((*E).base.ai_handler_bits & 1) == 0 {
            (*E).var_F = ((0xa8c500i32) as u16);
            (*E).var_E = 10;
            let v2: i32 = k as i32 >> 1;
            word!(game.enemy_drawing_queue[(v2 + 104)] = 1);
            word!(game.enemy_drawing_queue[(v2 + 103)] = addr_kMaridiaFloater_Ilist_C191 as u16);
        }
        MaridiaFloater_Func_2(k, game);
    };
    [11060509] MaridiaFloater_Func_11 [] {
        words!((*E), var_A, var_B += words!((g_word_A8C1BD, g_word_A8C1BF)));
        Enemy_MoveDown(k, (words!((*E), var_A, var_B)) as i32, game);
        let v4: i32 = k as i32 >> 1;
        if (((*E).base.y_pos) as i16)
            .wrapping_sub((word!(game.enemy_drawing_queue_sizes[(v4 + 1)])) as i16)
            >= 0
        {
            (*E).base.y_pos = word!(game.enemy_drawing_queue_sizes[(v4 + 1)]);
            (*E).var_F = ((0xa8c283i32) as u16);
            (*E).var_E = 60;
            enemy_instruction! {[(*E).base]; timer = 1, instruction = addr_kMaridiaFloater_Ilist_C173 as u16}
        }
        MaridiaFloater_Func_2(k, game);
    };
    @countdown [pub unsafe fn (k: u16, game: &mut Game)];
    before {
        let E = Get_MorphBallEye(k as i32, game);
        let v2: bool = (*E).var_E == 1;
        (*E).var_E = (*E).var_E.wrapping_sub(1);
        let v3: bool = ((*E).var_E & 0x8000) != 0;
    }
    when v2 || v3;
    [11047214] MorphBallEye_Func_2 {
        QueueSfx2_Max6(0x17, game);
        SpawnMorphBallEyeBeamHdma(game);
        (*E).var_F = ((0xa89160i32) as u16);
        (*E).var_D = CalculateAngleFromXY(
            (word!(game.samus_x_pos)).wrapping_sub((*E).base.x_pos),
            (word!(game.samus_y_pos)).wrapping_sub((*E).base.y_pos),
            game,
        );
    };
    [11047374] MorphBallEye_Func_4 {
        (*E).var_F = ((0xa890f1i32) as u16);
    };
    @countdown [pub unsafe fn (k: u16, game: &mut Game)];
    before {
        let E = Get_MaridiaFloater(k as i32, game);
        let v2: bool = (*E).var_E == 1;
        (*E).var_E = (*E).var_E.wrapping_sub(1);
        let v3: bool = ((*E).var_E & 0x8000) != 0;
    }
    when v2 || v3;
    [11059843] MaridiaFloater_Func_3 {
        let v4: i32 = k as i32 >> 1;
        word!(game.enemy_drawing_queue[(v4 + 104)] = 1);
        word!(game.enemy_drawing_queue[(v4 + 103)] = addr_kMaridiaFloater_Ilist_C183 as u16);
        (*E).var_F = ((0xa8c2a6i32) as u16);
        (*E).var_E = 10;
    } after {
        MaridiaFloater_Func_2(k, game);
    };
    [11059878] MaridiaFloater_Func_4 {
        (*E).var_F = ((0xa8c2cfi32) as u16);
        (*E).var_A = g_word_A8C1C5;
        (*E).var_B = g_word_A8C1C7;
        enemy_instruction! {[(*E).base]; timer = 1, instruction = addr_kMaridiaFloater_Ilist_C163 as u16}
    } after {
        MaridiaFloater_Func_2(k, game);
    };
    [11060480] MaridiaFloater_Func_10 {
        (*E).var_F = ((0xa8c51di32) as u16);
        (*E).var_A = g_word_A8C1C1;
        (*E).var_B = g_word_A8C1C3;
    } after {
        MaridiaFloater_Func_2(k, game);
    };
    [11060639] MaridiaFloater_Func_13 {
        let v4: i32 = k as i32 >> 1;
        word!(game.enemy_drawing_queue[(v4 + 100)] = 0);
        word!(game.enemy_drawing_queue[(v4 + 97)] |= (0x200u16));
        MaridiaFloater_Func_1(k, game);
        let mut v5: u16 = 0;
        EnemyDeathAnimation(k, v5, game);
    } else {
        MaridiaFloater_Func_2(k, game);
    } after { ; };
}
