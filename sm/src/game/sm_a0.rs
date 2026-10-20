use crate::c_library::{__assert_fail, memcpy, memset};
pub use crate::game::constants::bank_60::*;
pub use crate::game::constants::bank_86::*;
pub use crate::game::constants::bank_a0::*;
pub use crate::game::constants::bank_a4::*;
pub use crate::game::constants::bank_b4::*;
pub use crate::game::constants::enemy_properties::*;
use crate::game::moving_platforms::{self as mechanisms, Event, Kind};
use crate::game::sm_80::{
    NextRandom, QueueSfx1_Max3, QueueSfx1_Max6, QueueSfx2_Max1, QueueSfx2_Max3, QueueSfx3_Max3,
};
use crate::game::sm_81::{
    DrawSpritemapWithBaseTile, DrawSpritemapWithBaseTile2, DrawSpritemapWithBaseTileOffscreen,
};
use crate::game::sm_84::SpawnPLM;
use crate::game::sm_86::{DrawEprojsWithPriority, EprojPriority, SpawnEprojWithGfx};
use crate::game::sm_90::{CallSomeSamusCode, DrawSamusAndProjectiles};
use crate::game::sm_91::Samus_DealDamage;
use crate::game::sm_93::DrawBombAndProjectileExplosions;
use crate::game::sm_94::square_slope_occupied;
use crate::game::sm_b4::{
    ClearSpriteObjects, CreateSpriteAtPos, DrawSpriteObjects, HandleSpriteObjects,
};
use crate::game::{sm_a2, sm_a3, sm_a4, sm_a5, sm_a6, sm_a7, sm_a8, sm_a9, sm_aa, sm_b2, sm_b3};
use crate::game_state::Game;
use crate::ram_map::ram_ptr;
use crate::ram_map::word;
use crate::rtl::abs16;
use crate::rtl::rom_ptr;
use crate::rtl::words;
use crate::rtl::{Load24, Mult8x8, RomPtr, SnesDivide, Unreachable};
use crate::set_fields;
pub use crate::types::{size_t, EnemyData, LongPtr, PairU16, Point16U, VoidP, VramWriteEntry};
pub use crate::types::{EnemyDef, EnemySpawnData, Vulnerability};
pub use crate::types::{EnemyProps, EprojDef, ExtendedSpriteMap, Hitbox};
use ::core::ffi::{c_char, c_void};
use ::core::ptr::{null, null_mut};
crate::typed_struct! {
    #[derive(Copy, Clone)]
    #[repr(C, packed)]
    pub struct EnemyPopulation {
        pub enemy_ptr: VoidP;
        pub x_pos, y_pos, init_param, properties, extra_properties, parameter1, parameter2: u16;
    }
}
pub use crate::types::{EnemyAiBits, EnemyTileset};
crate::game::constants::typed_constants! { EnemyAiBits;
    kEnemyAiBits_Frozen = 4, kEnemyAiBits_Hurt = 2,
    kEnemyAiBits_Grapple = 1,
}
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct EnemyTileLoadData {
    pub tile_data_size: u16,
    pub tile_data_ptr: LongPtr,
    pub offset_into_ram: u16,
}
pub use crate::types::Consts_60;
pub use crate::types::Consts_86;
pub use crate::types::Consts_A0;
pub use crate::types::Consts_A4;
pub use crate::types::Consts_B4;
pub use crate::types::{
    CalculateAngleFromXYFunc, CheckEnemyColl_Result, CheckSolidEnemyColl_Result,
};
crate::typed_struct! {
    #[derive(Copy, Clone)]
    #[repr(C)]
    pub struct PositionAndWidth {
        pub x_pos, x_subpos, y_pos, y_subpos, width, height: u16;
    }
}
crate::typed_struct! {
    #[derive(Copy, Clone)]
    #[repr(C)]
    pub struct EnemyBlockCollInfo {
        pub ebci_r18_r20: i32;
        pub ebci_r24, ebci_r26, ebci_r28, ebci_r30, ebci_r32: u16;
    }
}
pub type Func_EnemyBlockCollInfo_U8 = unsafe fn(&mut EnemyBlockCollInfo, &mut Game) -> u8;
pub use crate::types::Point32;
pub const NULL: *mut c_void = null_mut::<c_void>();
#[inline]
unsafe fn RomPtr_RAM(addr: u16, game: &mut Game) -> *mut u8 {
    crate::native_assertion! {
        '_c2rust_label: addr < 0x2000;
        b"addr < 0x2000\0", b"src/sm_rtl.h\0", 51, b"uint8 *RomPtr_RAM(uint16_t)\0"
    };
    ram_ptr!(game; addr as i32 as isize)
}
const RomPtr_86: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x86>;
const RomPtr_A1: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0xa1>;
const RomPtr_B4: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0xb4>;
use crate::rtl::rom_ptr_with_bank as RomPtrWithBank;
#[inline]
unsafe fn RomBankBase(bank: u8, game: &mut Game) -> *const u8 {
    RomPtr((((bank as i32) << 16i32) + 0x8000i32) as u32, game).offset(-(0x8000i32 as isize))
}
#[inline]
unsafe fn get_EnemyDef_A2(a: u16, game: &mut Game) -> *mut EnemyDef {
    RomPtr((0xa00000u32) | ((a) as u32), game) as *mut EnemyDef
}
#[inline]
unsafe fn get_EnemyTileset(a: u16, game: &mut Game) -> *mut EnemyTileset {
    RomPtr((0xb40000u32) | ((a) as u32), game) as *mut EnemyTileset
}
#[inline]
unsafe fn get_EprojDef(a: u16, game: &mut Game) -> *mut EprojDef {
    RomPtr((0x860000u32) | ((a) as u32), game) as *mut EprojDef
}
#[inline]
unsafe fn get_Vulnerability(a: u16, game: &mut Game) -> *mut Vulnerability {
    RomPtr((0xb40000u32) | ((a) as u32), game) as *mut Vulnerability
}
#[inline]
unsafe fn get_ExtendedSpriteMap(db: u8, a: u16, game: &mut Game) -> *mut ExtendedSpriteMap {
    RomPtr((words!(i32, (db, a))) as u32, game) as *mut ExtendedSpriteMap
}
#[inline]
unsafe fn get_EnemyPopulation(db: u8, a: u16, game: &mut Game) -> *mut EnemyPopulation {
    RomPtr((words!(i32, (db, a))) as u32, game) as *mut EnemyPopulation
}
crate::game::constants::typed_constants! { i32;
    fnEprojPreInstr_nullsub_83 = 0x8684fb, fnGrappleBeam_Func2 = 0x9bc8c5,
}
crate::game::constants::typed_constants! { u32;
    fnMiniCrocomire_PreInstr5 = 10652227, fnMiniCrocomire_PreInstr6 = 10652252,
    fnnullsub_175 = 10652277,
    fnSpikeShootingPlant_2 = 10657722, fnSpikeShootingPlant_3 = 10657772,
    fnnullsub_182 = 10657819,
    fnnullsub_187 = 10659799,
    fnGunshipTop_3 = 10659852, fnGunshipTop_4 = 10660048,
    fnGunshipTop_5 = 10660162, fnGunshipTop_6 = 10660176,
    fnGunshipTop_7 = 10660231, fnGunshipTop_8 = 10660285,
    fnGunshipTop_9 = 10660431, fnGunshipTop_10 = 10660445,
    fnGunshipTop_11 = 10660500, fnGunshipTop_12 = 10660514,
    fnGunshipTop_13 = 10660639, fnGunshipTop_14 = 10660704,
    fnGunshipTop_15 = 10660718, fnGunshipTop_16 = 10660773,
    fnGunshipTop_17 = 10660807, fnGunshipTop_18 = 10660891,
    fnGunshipTop_19 = 10661079, fnGunshipTop_20 = 10661134,
    fnGunshipTop_21 = 10661165,
    fnFlies_4 = 10662222,
    fnFlies_5 = 10662268, fnFlies_6 = 10662314,
    fnFlies_7 = 10662354,
    fnNorfairLavajumpingEnemy_Func_1 = 10665692, fnNorfairLavajumpingEnemy_Func_2 = 10665754,
    fnNorfairLavajumpingEnemy_Func_3 = 10665790, fnNorfairLavajumpingEnemy_Func_4 = 10665852,
    fnNorfairLavajumpingEnemy_Func_5 = 10665916,
    fnNorfairRio_Func_1 = 10666625, fnNorfairRio_Func_2 = 10666727,
    fnNorfairRio_Func_3 = 10666815, fnNorfairRio_Func_4 = 10666849,
    fnNorfairRio_Func_5 = 10666929, fnNorfairRio_Func_6 = 10667014,
    fnLowerNorfairRio_Func_1 = 10667822,
    fnLowerNorfairRio_Func_2 = 10667889, fnLowerNorfairRio_Func_3 = 10667963,
    fnLowerNorfairRio_Func_4 = 10667990, fnLowerNorfairRio_Func_5 = 10668077,
    fnLowerNorfairRio_Func_6 = 10668168,
    fnMaridiaLargeSnail_Func_7 = 10669926,
    fnMaridiaLargeSnail_Func_8 = 10669993,
    fnLavaSeahorse_Func_1 = 10675796, fnLavaSeahorse_Func_2 = 10675885,
    fnLavaSeahorse_Func_3 = 10675953, fnLavaSeahorse_Func_4 = 10676020,
    fnLavaSeahorse_Func_5 = 10676041, fnnullsub_196 = 10676097,
    fnBang_Func_6 = 10730661,
    fnBang_Func_7 = 10730689, fnBang_Func_8 = 10730693,
    fnBang_Func_10 = 10730780, fnBang_Func_11 = 10730796,
    fnnullsub_215 = 10735455,
    fnMaridiaSnail_Func_15 = 10736051,
    fnReflec_Func_1 = 10738444,
    fnnullsub_304 = 10741441,
    fnFireZoomer_Func_1 = 10741448, fnFireZoomer_Func_2 = 10741637,
    fnDraygon_Func_36 = 10851138,
    fnnullsub_170_A7 = 10977356,
    fnnullsub_237 = 10998952, fnPhantoon_Spawn8FireballsInCircleAtStart = 10998953,
    fnPhantoon_WaitBetweenSpawningAndSpinningFireballs = 10999022, fnPhantoon_SpawnFireballsBeforeFight = 10999048,
    fnPhantoon_WavyFadeIn = 10999114, fnPhantoon_PickPatternForRound1 = 10999190,
    fnPhantoon_MovePhantoonInFigure8ThenOpenEye = 10999271, fnPhantoon_EyeFollowsSamusUntilTimerRunsOut = 10999309,
    fnPhantoon_BecomesSolidAndBodyVuln = 10999388, fnPhantoon_IsSwooping = 10999416,
    fnPhantoon_FadeoutWithSwoop = 10999481, fnPhantoon_WaitAfterFadeOut = 10999508,
    fnPhantoon_MoveLeftOrRightAndPickEyeOpenPatt = 10999522, fnPhantoon_FadeInBeforeFigure8 = 10999597,
    fnPhantoon_BecomeSolidAfterRainingFireballs = 10999615, fnPhantoon_FadeInDuringFireballRain = 10999655,
    fnPhantoon_FollowSamusWithEyeDuringFireballRain = 10999688, fnPhantoon_FadeOutDuringFireballRain = 10999765,
    fnPhantoon_SpawnRainingFireballs = 10999799, fnPhantoon_FadeOutBeforeFirstFireballRain = 10999850,
    fnPhantoon_FadeOutBeforeEnrage = 10999900, fnPhantoon_MoveEnragedPhantoonToTopCenter = 10999924,
    fnPhantoon_FadeInEnragedPhantoon = 10999953, fnPhantoon_Enraged = 10999980,
    fnPhantoon_FadeoutAfterEnrage = 11000086, fnPhantoon_CompleteSwoopAfterFatalShot = 11000110,
    fnPhantoon_DyingPhantoonFadeInOut = 11000136, fnPhantoon_DyingPhantoonExplosions = 11000203,
    fnPhantoon_WavyDyingPhantoonAndCry = 11000401, fnPhantoon_DyingFadeOut = 11000454,
    fnPhantoon_AlmostDead = 11000535, fnPhantoon_Dead = 11000637,
    fnEtecoon_Func_4 = 11004335,
    fnEtecoon_Func_5 = 11004416, fnEtecoon_Func_6 = 11004471,
    fnEtecoon_Func_7 = 11004597, fnEtecoon_Func_8 = 11004674,
    fnEtecoon_Func_9 = 11004716, fnEtecoon_Func_10 = 11004752,
    fnEtecoon_Func_11 = 11004877, fnEtecoon_Func_12 = 11004955,
    fnEtecoon_Func_16 = 11005079, fnEtecoon_Func_17 = 11005115,
    fnEtecoon_Func_18 = 11005151, fnEtecoon_Func_19 = 11005193,
    fnEtecoon_Func_20 = 11005226, fnEtecoon_Func_21 = 11005268,
    fnEtecoon_Func_22 = 11005301, fnEtecoon_Func_23 = 11005383,
    fnEtecoon_Func_24 = 11005502, fnEtecoon_Func_25 = 11005594,
    fnEtecoon_Func_26 = 11005624,
    fnDachora_Func_2 = 11007344,
    fnDachora_Func_3 = 11007420, fnDachora_Func_4 = 11007469,
    fnDachora_Func_5 = 11007582, fnDachora_Func_7 = 11007887,
    fnDachora_Func_8 = 11008006, fnDachora_Func_11 = 11008309,
    fnDachora_Func_12 = 11008396,
    fnnullsub_170_A8 = 11042892,
    fnMorphBallEye_Func_1 = 11047153, fnMorphBallEye_Func_2 = 11047214,
    fnMorphBallEye_Func_3 = 11047264, fnMorphBallEye_Func_4 = 11047374,
    fnnullsub_244 = 11047388,
    fnNorfairLavaMan_Func_6 = 11055282,
    fnNorfairLavaMan_Func_7 = 11055386, fnNorfairLavaMan_Func_8 = 11055477,
    fnsub_A8B193 = 11055507, fnNorfairLavaMan_Func_9 = 11055544,
    fnNorfairLavaMan_Func_10 = 11055581, fnNorfairLavaMan_Func_11 = 11055620,
    fnsub_A8B291 = 11055761, fnNorfairLavaMan_Func_13 = 11055765,
    fnNorfairLavaMan_Func_15 = 11055885, fnNorfairLavaMan_Func_16 = 11055903,
    fnNorfairLavaMan_Func_17 = 11055958, fnNorfairLavaMan_Func_18 = 11056039,
    fnMaridiaFloater_Func_3 = 11059843, fnMaridiaFloater_Func_4 = 11059878,
    fnMaridiaFloater_Func_5 = 11059919, fnMaridiaFloater_Func_6 = 11060075,
    fnMaridiaFloater_Func_7 = 11060193, fnMaridiaFloater_Func_8 = 11060329,
    fnMaridiaFloater_Func_9 = 11060444, fnMaridiaFloater_Func_10 = 11060480,
    fnMaridiaFloater_Func_11 = 11060509, fnnullsub_256 = 11060584,
    fnMaridiaFloater_Func_12 = 11060585, fnMaridiaFloater_Func_13 = 11060639,
    fnWreckedShipRobot_Func_1 = 11062375,
    fnBlueBrinstarFaceBlock_Func_1 = 11069550,
    fnnullsub_171_AA = 11174011,
    fnnullsub_274 = 11197610,
    fnnullsub_276 = 11200422,
    fnnullsub_171_B3 = 11763835,
    fnBrinstarPipeBug_PreInstr_1 = 11765888, fnBrinstarPipeBug_PreInstr_2 = 11765904,
    fnBrinstarPipeBug_PreInstr_3 = 11765987, fnBrinstarPipeBug_PreInstr_4 = 11766044,
    fnBrinstarPipeBug_PreInstr_5 = 11766142,
    fnNorfairPipeBug_Func_5 = 11767039,
    fnBotwoon_Func_26 = 11771328, fnBotwoon_Func_27 = 11771517,
    fnBotwoon_Func_28 = 11771616, fnBotwoon_Func_29 = 11771700,
    fnBotwoon_Func_30 = 11771770,
}
crate::ram_map::ram_pointer_functions! {
    gEnemyData(o: i32) -> EnemyData = [0xf78, o as isize];
    gEnemySpawnData(o: i32) -> EnemySpawnData = [0x7000, o as isize];
}
pub unsafe fn EnemyInstr_Goto(_k: u16, jp: *const u16) -> *const u16 {
    *jp as usize as *const u16
}
static kRoomShakes: [u16; 144] = [
    1, 0, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0, 2, 0, 0, 0, 0, 2, 0, 0, 2, 2, 0, 0, 3, 0, 0, 0, 0, 3, 0, 0,
    3, 3, 0, 0, 1, 0, 1, 0, 0, 1, 0, 1, 1, 1, 1, 1, 2, 0, 2, 0, 0, 2, 0, 2, 2, 2, 2, 2, 3, 0, 3, 0,
    0, 3, 0, 3, 3, 3, 3, 3, 1, 0, 1, 0, 0, 1, 0, 1, 1, 1, 1, 1, 2, 0, 2, 0, 0, 2, 0, 2, 2, 2, 2, 2,
    3, 0, 3, 0, 0, 3, 0, 3, 3, 3, 3, 3, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0, 2, 0, 0, 0, 0, 2,
    0, 0, 2, 2, 0, 0, 3, 0, 0, 0, 0, 3, 0, 0, 3, 3,
];

macro_rules! adjust_room_shake_scroll {
    ($game:ident, $adjust:ident, $shake_index:ident) => {
        word!(
            $game.reg_BG1HOFS,
            $adjust(kRoomShakes[$shake_index as usize])
        );
        word!(
            $game.reg_BG1VOFS,
            $adjust(kRoomShakes[($shake_index + 1) as usize])
        );
        word!(
            $game.reg_BG2HOFS,
            $adjust(kRoomShakes[($shake_index + 2) as usize])
        );
        word!(
            $game.reg_BG2VOFS,
            $adjust(kRoomShakes[($shake_index + 3) as usize])
        );
    };
}

pub unsafe fn HandleRoomShaking(game: &mut Game) {
    if word!(game.earthquake_timer) != 0
        && word!(game.time_is_frozen_flag) == 0
        && word!(game.earthquake_type) as i32 - 36i32 & 0x8000 != 0
    {
        let v0: i32 = 8 * word!(game.earthquake_type) as i32 >> 1;
        if (word!(game.earthquake_timer) & 2) != 0 {
            adjust_room_shake_scroll!(game, wrapping_sub, v0);
        } else {
            adjust_room_shake_scroll!(game, wrapping_add, v0);
        }
        word!(game.earthquake_timer, wrapping_sub(1));
        if word!(game.earthquake_type) as i32 - 18i32 & 0x8000 == 0 {
            SetAllEnemiesToShakeFor2Frames(game);
        }
    }
    word!(game.frame_counter_every_frame, wrapping_add(1));
}
pub unsafe fn SetAllEnemiesToShakeFor2Frames(game: &mut Game) {
    let mut i: i32 = 0;
    loop {
        let v1: i32 = i >> 1;
        if word!(game.active_enemy_indexes[v1]) == 0xffff {
            break;
        }
        (*gEnemyData(word!(game.active_enemy_indexes[v1]) as i32, game)).shake_timer = 2;
        i += 2;
    }
}
crate::game::address_dispatch! {
    pub unsafe fn CallEnemyGfxDrawHook(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        10518604 => [()]; 10738444 => sm_a3::Reflec_Func_1; 10851138 => sm_a5::Draygon_Func_36;
        10920690 => sm_a6::Ridley_A2F2; 11042892 => [()]; 11055282 => sm_a8::NorfairLavaMan_Func_6;
        11062375 => sm_a8::WreckedShipRobot_Func_1; 11069550 => sm_a8::BlueBrinstarFaceBlock_Func_1;
        11110278 => [()]; 11110345 => sm_a9::MotherBrain_DrawBrainNeck_EnemyGfxDrawHook;
        11110365 => sm_a9::MotherBrainsBrain_GfxDrawHook; 11129754 => sm_a9::DeadTorizo_MainGfxHook;
    }
}
pub unsafe fn DrawSamusEnemiesAndProjectiles(game: &mut Game) {
    DrawSpriteObjects(game);
    DrawBombAndProjectileExplosions(game);
    DrawEprojsWithPriority(EprojPriority::Low, game);
    for phase_varE32 in 0u16..8 {
        if phase_varE32 == 3 {
            DrawSamusAndProjectiles(game);
        } else if phase_varE32 == 6 {
            DrawEprojsWithPriority(EprojPriority::High, game);
        }
        if word!(game.enemy_drawing_queue_sizes[phase_varE32]) != 0 {
            let varE36: u16 = word!(game.enemy_drawing_queue_sizes[phase_varE32]);
            let varE3A: u16 = crate::rtl::rom_table_word!(game, 0xa0b133, phase_varE32 as isize);
            let mut v1: u16 = 0;
            word!(game.enemy_drawing_queue_sizes[phase_varE32] = 0);
            do_until! { (varE38 as i32 + 2 == varE36 as i32);
                let mut varE38: u16 = v1;
                let v2 = RomPtr_RAM((varE3A).wrapping_add(v1), game);
                let v3: u16 = *(v2 as *mut u16);
                *(v2 as *mut u16) = 0;
                word!(game.cur_enemy_index = v3);
                WriteEnemyOams(game);
                v1 = (varE38).wrapping_add(2);
            }
        }
    }
    CallEnemyGfxDrawHook(Load24(ram_ptr!(game, LongPtr; 0x178c)), game);
}
pub unsafe fn RecordEnemySpawnData(j: u16, game: &mut Game) {
    let v1 = gEnemyData(j as i32, game);
    let mut v2 = gEnemySpawnData(j as i32, game);
    set_fields! {[(*v2)]; id = (*v1).enemy_ptr, x_pos = (*v1).x_pos, y_pos = (*v1).y_pos,
    init_param = (*v1).current_instruction, properties = (*v1).properties,
    extra_properties = (*v1).extra_properties, param_1 = (*v1).parameter_1,
    param_2 = (*v1).parameter_2, };
    let mut r18: u16 = 0;
    let mut r20: u16 = 0;
    let mut r22: u16 = 0;
    let mut r24: u16 = 0;
    let mut R26: u16 = 0;
    let mut R28: u16 = 0;
    let name_ptr: u16 = (*get_EnemyDef_A2((*v1).enemy_ptr, game)).name_ptr;
    if name_ptr != 0 {
        let v4 = RomPtr_B4(name_ptr, game) as *const u16;
        r18 = *v4;
        r20 = *v4.offset(1);
        r22 = *v4.offset(2);
        r24 = *v4.offset(3);
        R26 = *v4.offset(4);
        R28 = *v4.offset(6);
    }
    let mut v5 = gEnemySpawnData(j as i32, game);
    *(&raw mut (*v5).name as *mut u8 as *mut u16) = r18;
    *((&raw mut (*v5).name as *mut u8).offset(2) as *mut u16) = r20;
    *((&raw mut (*v5).name as *mut u8).offset(4) as *mut u16) = r22;
    *((&raw mut (*v5).name as *mut u8).offset(6) as *mut u16) = r24;
    *((&raw mut (*v5).name as *mut u8).offset(8) as *mut u16) = R26;
    *((&raw mut (*v5).name as *mut u8).offset(10) as *mut u16) = R28;
}
pub unsafe fn LoadEnemies(game: &mut Game) {
    word!(game.debug_time_frozen_for_enemies = 0);
    (*(ram_ptr!(game, LongPtr; 0x178c))).bank = 160;
    (*(ram_ptr!(game, LongPtr; 0x178c))).addr = ((0xa0804ci32) as u16);
    word!(game.enemy_bg2_tilemap_size = 2048);
    word! { game, 0; UNUSED_word_7E179E, UNUSED_word_7E17A0, boss_id }
    ClearEnemyDataAndProcessEnemySet(game);
    LoadEnemyTileData(game);
    word! { game, 0; enemy_tile_vram_src, flag_disable_projectile_interaction }
    ClearSpriteObjects(game);
}
pub unsafe fn ClearEnemyDataAndProcessEnemySet(game: &mut Game) {
    memset(ram_ptr!(game, c_void; 0xf78), 0, 2048 as size_t);
    if *(RomPtr_A1(word!(game.room_enemy_population_ptr), game) as *mut u16) != 0xffff {
        ProcessEnemyTilesets(game);
    }
}
pub unsafe fn InitializeEnemies(game: &mut Game) {
    memset(ram_ptr!(game, c_void; 0x7000), 0, 0x2800 as size_t);
    word! { game, 0;
        num_enemies_in_room, num_enemies_killed_in_room, flag_process_all_enemies,
    }
    word_transfers!(game, descending, 144; le(eproj_flags, 143) <- value(0));
    word_transfers!(game, descending, 18; le(eproj_killed_enemy_index, 17) <- value(-1i32 as u16));
    let mut v4: u16 = word!(game.room_enemy_population_ptr);
    if (*get_EnemyPopulation(0xa1u8, word!(game.room_enemy_population_ptr), game)).enemy_ptr
        == 0xffff
    {
        return;
    }
    word!(game.UNUSED_word_7E0E48 = 0);
    let mut v5: u16 = 0;
    do_until! { ((*get_EnemyPopulation(0xa1u8, v4, game)).enemy_ptr == 0xffff);
        LoadEnemyGfxIndexes(v4, v5, game);
        let EP = get_EnemyPopulation(0xa1, v4, game);
        let ED = get_EnemyDef_A2((*EP).enemy_ptr, game);
        let E = gEnemyData(v5 as i32, game);
        (*E).x_width = (*ED).x_radius;
        (*E).y_height = (*ED).y_radius;
        (*E).health = (*ED).health;
        (*E).layer = (*ED).layer as u16;
        *(&raw mut (*E).bank as *mut u16) = *(&raw mut (*ED).bank as *mut u16);
        if (*ED).boss_fight_value != 0 {
            word!(game.boss_id = (*ED).boss_fight_value);
        }
        set_fields! {[(*E)]; enemy_ptr = (*EP).enemy_ptr, x_pos = (*EP).x_pos,
            y_pos = (*EP).y_pos, current_instruction = (*EP).init_param,
            properties = (*EP).properties, extra_properties = (*EP).extra_properties,
            parameter_1 = (*EP).parameter1, parameter_2 = (*EP).parameter2, };
        set_fields! {[(*E)], 0; frame_counter, timer}
        (*E).instruction_timer = 1;
        (*E).frame_counter = 0;
        RecordEnemySpawnData(v5, game);
        word!(game.cur_enemy_index = v5);
        CallEnemyAi((words!(i32, (*ED), bank, ai_init)) as u32, game);
        (*E).spritemap_pointer = 0;
        if ((*E).properties & 0x2000) != 0 {
            let mut v12: u16 = addr_kSpritemap_Nothing_A4 as u16;
            if ((*E).extra_properties & 4) != 0 {
                v12 = addr_kExtendedSpritemap_Nothing_A4 as u16;
            }
            (*E).spritemap_pointer = v12;
        }
        v5 = (v5).wrapping_add(64);
        v4 = (v4).wrapping_add(16);
    }
    word!(game.first_free_enemy_index = v5);
    word!(game.num_enemies_in_room = (v5 >> 6));
    word!(game.num_enemy_deaths_left_to_clear = *RomPtr_A1(v4, game).offset(2) as u16);
}
pub unsafe fn LoadEnemyGfxIndexes(k: u16, j: u16, game: &mut Game) {
    let E = gEnemyData(j as i32, game);
    let ES = gEnemySpawnData(j as i32, game);
    let mut e: u16 = word!(game.room_enemy_tilesets_ptr);
    let mut f: u16 = 0;
    loop {
        let ET = get_EnemyTileset(e, game);
        if (*get_EnemyPopulation(0xa1u8, k, game)).enemy_ptr == (*ET).enemy_def {
            (*ES).palette_index = (((*ET).vram_dst as i32 & 0xf) << 9) as u16;
            (*E).palette_index = (*ES).palette_index;
            (*ES).vram_tiles_index = f;
            (*E).vram_tiles_index = (*ES).vram_tiles_index;
            return;
        }
        if (*ET).enemy_def == 0xffff {
            (*ES).vram_tiles_index = 0;
            (*E).vram_tiles_index = (*ES).vram_tiles_index;
            (*ES).palette_index = 2560;
            (*E).palette_index = (*ES).palette_index;
            return;
        }
        f = (f as i32 + ((*get_EnemyDef_A2((*ET).enemy_def, game)).tile_data_size as i32 >> 5))
            as u16;
        e = (e).wrapping_add(4);
    }
}
pub unsafe fn LoadEnemyTileData(game: &mut Game) {
    for i in (0i32..=510).rev().step_by(2) {
        (*gEnemySpawnData(i, game)).some_flag =
            crate::rtl::rom_table_word!(game, 0x9ad200, ((i >> 1) + 3072) as isize);
    }
    if word!(game.enemy_tile_load_data_write_pos) != 0 {
        let mut v1: u16 = 0;
        let mut load_data: *mut EnemyTileLoadData = ram_ptr!(game, EnemyTileLoadData; 0xe5a);
        do_until! { (v1 == word!(game.enemy_tile_load_data_write_pos));
            let mut v2: u16 = (*load_data).tile_data_ptr.addr;
            let r18: u16 = ((*load_data).tile_data_size).wrapping_add(v2);
            let mut v3: u16 = (*load_data).offset_into_ram;
            let db: u8 = (*load_data).tile_data_ptr.bank;
            do_until! { (v2 == r18);
            memcpy(
                ram_ptr!(game, c_void; (0x7000 + v3 as i32) as isize),
                RomPtrWithBank(db, v2, game) as *const c_void,
                8 as size_t,
            );
            v3 = (v3).wrapping_add(8);
            v2 = (v2).wrapping_add(8);
        }
            load_data = load_data.offset(1);
            v1 = (v1).wrapping_add(7);
        }
        word!(game.enemy_tile_load_data_write_pos = 0);
    }
}
pub unsafe fn TransferEnemyTilesToVramAndInit(game: &mut Game) {
    let mut source = game.enemy_tile_vram_src();
    if game.enemy_tile_vram_src() == 0 {
        source = 0x7000;
        game.set_enemy_tile_vram_src(0x7000);
        game.set_enemy_tile_vram_dst(0x6c00);
    }
    match source {
        0xffff => {}
        0xfffe => {
            InitializeEnemies(game);
            game.set_enemy_tile_vram_src(0xffff);
        }
        0x9800 => game.set_enemy_tile_vram_src(0xfffe),
        _ => {
            let tail = game.vram_write_queue_tail();
            let cursor = game.vram_write_queue_tail();
            game.write_vram_queue_word(cursor, 0, 2048);
            let source = game.enemy_tile_vram_src();
            let address = game.enemy_tile_vram_src();
            game.write_vram_queue_word(cursor, 2, address);
            game.set_enemy_tile_vram_src(source.wrapping_add(2048));
            // Original word store clears the first destination byte too.
            game.write_vram_queue_word(cursor, 4, 126);
            let destination = game.enemy_tile_vram_dst();
            let address = game.enemy_tile_vram_dst();
            game.write_vram_queue_word(cursor, 5, address);
            game.set_enemy_tile_vram_dst(destination.wrapping_add(1024));
            game.set_vram_write_queue_tail(tail.wrapping_add(7));
        }
    }
}
pub unsafe fn ProcessEnemyTilesets(game: &mut Game) {
    word!(game.enemy_tile_load_data_write_pos = 0);
    let mut r30: u16 = 2048;
    word! { game, 0;
        enemy_def_ptr[0], enemy_def_ptr[1], enemy_def_ptr[2], enemy_def_ptr[3],
        enemy_gfxdata_tiles_index[0], enemy_gfxdata_tiles_index[1], enemy_gfxdata_tiles_index[2],
        enemy_gfxdata_tiles_index[3], enemy_gfxdata_vram_ptr[0], enemy_gfxdata_vram_ptr[1],
        enemy_gfxdata_vram_ptr[2], enemy_gfxdata_vram_ptr[3],
    }
    let mut enemy_gfx_data_write_ptr: u16 = 0;
    let mut next_enemy_tiles_index: u16 = 0;
    let mut ET = get_EnemyTileset(word!(game.room_enemy_tilesets_ptr), game);
    let mut LD: *mut EnemyTileLoadData = ram_ptr!(game, EnemyTileLoadData; 0xe5a);
    while (*ET).enemy_def != 0xffff {
        let ED = get_EnemyDef_A2((*ET).enemy_def, game);
        memcpy(
            (ram_ptr!(game, u16; 0xc200))
                .offset(((byte!((*ET).vram_dst; 0) as i32 + 8) * 16) as isize)
                as *mut c_void,
            RomPtrWithBank((*ED).bank, (*ED).palette_ptr, game) as *const c_void,
            32 as size_t,
        );
        (*LD).tile_data_size = ((*ED).tile_data_size) & (0x7fffu16);
        (*LD).tile_data_ptr = (*ED).tile_data;
        let mut v10: u16 = r30;
        if ((*ED).tile_data_size & 0x8000) != 0 {
            v10 = ((((*ET).vram_dst) & (0x3000u16)) >> 3);
        }
        (*LD).offset_into_ram = v10;
        word!(game.enemy_tile_load_data_write_pos, wrapping_add(7));
        LD = LD.offset(1);
        let v11: u16 = enemy_gfx_data_write_ptr;
        *(ram_ptr!(game, u16; 0xef64)).offset((enemy_gfx_data_write_ptr >> 1) as isize) =
            next_enemy_tiles_index;
        word!(game.enemy_def_ptr[(v11 >> 1)] = (*ET).enemy_def);
        word!(game.enemy_gfxdata_vram_ptr[(v11 >> 1)] = (*ET).vram_dst);
        enemy_gfx_data_write_ptr = (enemy_gfx_data_write_ptr).wrapping_add(2);
        next_enemy_tiles_index =
            (next_enemy_tiles_index as i32 + ((*ED).tile_data_size as i32 >> 5)) as u16;
        r30 = (r30).wrapping_add((*ED).tile_data_size);
        ET = ET.offset(1);
    }
}
pub unsafe fn DetermineWhichEnemiesToProcess(game: &mut Game) {
    word!(game.UNUSED_word_7E0E46, wrapping_add(1));
    word! { game, 0;
        cur_enemy_index, active_enemy_indexes_write_ptr, interactive_enemy_indexes_write_ptr,
    }
    if word!(game.flag_process_all_enemies) != 0 {
        do_until! { (word!(game.cur_enemy_index) as i32 - 2048i32 & 0x8000 == 0);
            let v5: u16 = word!(game.cur_enemy_index);
            let v6 = gEnemyData(word!(game.cur_enemy_index) as i32, game);
            if (*v6).enemy_ptr != 0 && (*v6).enemy_ptr as i32 != addr_kEnemyDef_DAFF as i32 {
                if ((*v6).properties & 0x200) != 0 {
                    (*v6).enemy_ptr = 0;
                } else {
                    let v7: u16 = word!(game.active_enemy_indexes_write_ptr);
                    let v8: i32 = word!(game.active_enemy_indexes_write_ptr) as i32 >> 1;
                    word!(game.active_enemy_indexes[v8] = word!(game.cur_enemy_index));
                    word!(game.interactive_enemy_indexes[v8] = v5);
                    word!(game.active_enemy_indexes_write_ptr = (v7).wrapping_add(2));
                    if ((*v6).properties & 0x400) == 0 {
                        let v9: u16 = word!(game.interactive_enemy_indexes_write_ptr);
                        *(ram_ptr!(game, u16; 0x17ec)).offset(
                            (word!(game.interactive_enemy_indexes_write_ptr) >> 1) as isize,
                        ) = v5;
                        word!(game.interactive_enemy_indexes_write_ptr = (v9).wrapping_add(2));
                    }
                }
            }
            word!(game.cur_enemy_index, wrapping_add(64));
        }
        *(ram_ptr!(game, u16; 0x17ac))
            .offset((word!(game.active_enemy_indexes_write_ptr) >> 1) as isize) = -1i32 as u16;
        *(ram_ptr!(game, u16; 0x17ec))
            .offset((word!(game.interactive_enemy_indexes_write_ptr) >> 1) as isize) = -1i32 as u16;
    } else {
        do_until! { (word!(game.cur_enemy_index) as i32 - 2048i32 & 0x8000 == 0);
            let v0: u16 = word!(game.cur_enemy_index);
            let v1 = gEnemyData(word!(game.cur_enemy_index) as i32, game);
            if (*v1).enemy_ptr != 0 && (*v1).enemy_ptr as i32 != addr_kEnemyDef_DAFF as i32 {
                let properties: u16 = (*v1).properties;
                if (properties & 0x200) != 0 {
                    (*v1).enemy_ptr = 0;
                } else if (properties & 0x800) != 0
                    || ((*v1).ai_handler_bits & 4) != 0
                    || (((*v1).x_width as i32 + (*v1).x_pos as i32
                        - word!(game.layer1_x_pos) as i32) as i16)
                        >= 0
                        && (((*v1).x_width as i32 + word!(game.layer1_x_pos) as i32 + 256
                            - (*v1).x_pos as i32) as i16)
                            >= 0
                        && (((*v1).y_pos as i32 + 8 - word!(game.layer1_y_pos) as i32) as i16) >= 0
                        && ((word!(game.layer1_y_pos) as i32 + 248 - (*v1).y_pos as i32) as i16)
                            >= 0
                {
                    let v3: u16 = word!(game.active_enemy_indexes_write_ptr);
                    *(ram_ptr!(game, u16; 0x17ac))
                        .offset((word!(game.active_enemy_indexes_write_ptr) >> 1) as isize) =
                        word!(game.cur_enemy_index);
                    word!(game.active_enemy_indexes_write_ptr = (v3).wrapping_add(2));
                    if ((*v1).properties & 0x400) == 0 {
                        let v4: u16 = word!(game.interactive_enemy_indexes_write_ptr);
                        *(ram_ptr!(game, u16; 0x17ec)).offset(
                            (word!(game.interactive_enemy_indexes_write_ptr) >> 1) as isize,
                        ) = v0;
                        word!(game.interactive_enemy_indexes_write_ptr = (v4).wrapping_add(2));
                    }
                }
            }
            word!(game.cur_enemy_index, wrapping_add(64));
        }
        *(ram_ptr!(game, u16; 0x17ac))
            .offset((word!(game.active_enemy_indexes_write_ptr) >> 1) as isize) = -1i32 as u16;
        *(ram_ptr!(game, u16; 0x17ec))
            .offset((word!(game.interactive_enemy_indexes_write_ptr) >> 1) as isize) = -1i32 as u16;
    };
}
crate::game::callback_dispatch! {
    pub unsafe fn CallEnemyAi(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [10518583 | 10649655 | 10715191 | 10780727 | 10846263 | 10911799 | 10977335 | 11042871
                | 11108407 | 11173943 | 11698231 | 11763767] NormalEnemyPowerBombAi [] {
        NormalEnemyPowerBombAiSkipDeathAnim(game);
        if (*gEnemyData(word!(game.cur_enemy_index) as i32, game)).health == 0 {
            (*gEnemySpawnData(word!(game.cur_enemy_index) as i32, game)).cause_of_death = 3;
            EnemyDeathAnimation(word!(game.cur_enemy_index), 0, game);
        }
    };
    @external {
        10649605 => sm_a2::Enemy_GrappleReact_SamusLatchesOn_A2;
        10649610 => sm_a2::Enemy_GrappleReact_KillEnemy_A2;
        10649615 => sm_a2::Enemy_GrappleReact_CancelBeam_A2;
        10649635 => sm_a2::Enemy_NormalTouchAI_A2; 10649645 => sm_a2::Enemy_NormalShotAI_A2;
        10649665 => sm_a2::Enemy_NormalFrozenAI_A2; 10649676 => [()];
        10651420 => sm_a2::BouncingGoofball_Init; 10651548 => sm_a2::BouncingGoofball_Main;
        10652077 => sm_a2::MiniCrocomire_Init; 10652144 => sm_a2::MiniCrocomire_Main;
        10653036 => sm_a2::MaridiaBeybladeTurtle_Init;
        10653085 => sm_a2::MiniMaridiaBeybladeTurtle_Init;
        10653138 => sm_a2::MaridiaBeybladeTurtle_Main;
        10653998 => sm_a2::MiniMaridiaBeybladeTurtle_Main;
        10654337 => sm_a2::MaridiaBeybladeTurtle_Touch;
        10654367 => sm_a2::MiniMaridiaBeybladeTurtle_Touch;
        10654479 => sm_a2::MiniMaridiaBeybladeTurtle_Shot; 10656319 => sm_a2::ThinHoppingBlobs_Init;
        10656381 => sm_a2::ThinHoppingBlobs_Main; 10657608 => sm_a2::SpikeShootingPlant_Init;
        10657715 => sm_a2::SpikeShootingPlant_Main; 10658809 => sm_a2::MaridiaSpikeyShell_Init;
        10658942 => sm_a2::MaridiaSpikeyShell_Main; 10659193 => sm_a2::MaridiaSpikeyShell_Shot;
        10659396 => sm_a2::GunshipTop_Init; 10659538 => sm_a2::GunshipBottom_Init;
        10659673 => sm_a2::GunshipTop_Main; 10661995 => sm_a2::Flies_Init;
        10662175 => sm_a2::Flies_Main; 10662880 => sm_a2::NorfairErraticFireball_Init;
        10662927 => sm_a2::NorfairErraticFireball_Main; 10663280 => sm_a2::LavaquakeRocks_Init;
        10663311 => sm_a2::LavaquakeRocks_Main; 10663426 => sm_a2::Rinka_Init;
        10663876 => sm_a2::Rinka_Main;
        10664233 => [{
                    sm_a2::Rinka_Frozen(word!(game.cur_enemy_index), game);
                }];
        10664263 => sm_a2::Rinka_Touch; 10664269 => sm_a2::Rinka_Shot;
        10664275 => [{
                    sm_a2::Rinka_Powerbomb(word!(game.cur_enemy_index), game);
                }];
        10664909 => sm_a2::Rio_Init; 10664931 => sm_a2::Rio_Main;
        10665625 => sm_a2::NorfairLavajumpingEnemy_Init;
        10665682 => sm_a2::NorfairLavajumpingEnemy_Main; 10666562 => sm_a2::NorfairRio_Init;
        10666615 => sm_a2::NorfairRio_Main; 10667763 => sm_a2::LowerNorfairRio_Init;
        10667812 => sm_a2::LowerNorfairRio_Main; 10669268 => sm_a2::MaridiaLargeSnail_Init;
        10669331 => sm_a2::MaridiaLargeSnail_Main; 10670988 => sm_a2::MaridiaLargeSnail_Touch;
        10671028 => sm_a2::MaridiaLargeSnail_Shot; 10674038 => sm_a2::HirisingSlowfalling_Init;
        10674222 => sm_a2::HirisingSlowfalling_Main; 10674643 => sm_a2::Gripper_Init;
        10674721 => sm_a2::Gripper_Main; 10674968 => sm_a2::JetPowerRipper_Init;
        10675027 => sm_a2::JetPowerRipper_Main; 10675113 => sm_a2::JetPowerRipper_Shot;
        10675359 => sm_a2::Ripper_Init; 10675418 => sm_a2::Ripper_Main;
        10675718 => sm_a2::LavaSeahorse_Init; 10675790 => sm_a2::LavaSeahorse_Main;
        10676168 => sm_a2::LavaSeahorse_Touch; 10676174 => sm_a2::LavaSeahorse_Shot;
        10676180 => sm_a2::LavaSeahorse_Powerbomb;
        10676698 => [mechanisms::initialize(Kind::TimedShutter, game)];
        10676918 => [mechanisms::update(Kind::TimedShutter, game)];
        10677765 => [mechanisms::initialize(Kind::Platform, game)];
        10677778 => [mechanisms::initialize(Kind::VerticalShutter, game)];
        10677969 => [mechanisms::update(Kind::Platform, game)];
        10678429 => [mechanisms::react(Kind::Platform, Event::Touch, game)];
        10678434 => [mechanisms::react(Kind::Platform, Event::Shot, game)];
        10678442 => [mechanisms::react(Kind::VerticalShutter, Event::Shot, game)];
        10678454 => [mechanisms::react(Kind::Platform, Event::PowerBomb, game)];
        10678545 => [mechanisms::initialize(Kind::HorizontalShutter, game)];
        10678750 => [mechanisms::update(Kind::HorizontalShutter, game)];
        10679256 => [mechanisms::react(Kind::HorizontalShutter, Event::Touch, game)];
        10679310 => [mechanisms::react(Kind::HorizontalShutter, Event::Shot, game)];
        10679322 => [mechanisms::react(Kind::HorizontalShutter, Event::PowerBomb, game)];
        10715136 => sm_a3::Enemy_GrappleReact_NoInteract_A3;
        10715146 => sm_a3::Enemy_GrappleReact_KillEnemy_A3;
        10715151 => sm_a3::Enemy_GrappleReact_CancelBeam_A3;
        10715171 => sm_a3::Enemy_NormalTouchAI_A3; 10715181 => sm_a3::Enemy_NormalShotAI_A3;
        10715201 => sm_a3::Enemy_NormalFrozenAI_A3; 10715212 => [()]; 10716909 => sm_a3::Waver_Init;
        10717004 => sm_a3::Waver_Main; 10717536 => sm_a3::Metalee_Init;
        10717561 => sm_a3::Metalee_Main; 10717967 => sm_a3::Metalee_Shot;
        10718509 => sm_a3::Fireflea_Init; 10718702 => sm_a3::Fireflea_Main;
        10718827 => [{
                    sm_a3::Fireflea_Touch(word!(game.cur_enemy_index), game);
                }];
        10718851 => sm_a3::Fireflea_Powerbomb; 10718857 => sm_a3::Fireflea_Shot;
        10719413 => sm_a3::MaridiaFish_Init; 10719531 => sm_a3::MaridiaFish_Main;
        10720486 => sm_a3::Elevator_Init; 10720554 => sm_a3::Elevator_Frozen;
        10720995 => sm_a3::Crab_Init; 10721595 => sm_a3::Slug_Init;
        10722463 => sm_a3::PlatformThatFallsWithSamus_Init;
        10722490 => sm_a3::FastMovingSlowSinkingPlatform_Init;
        10722582 => sm_a3::PlatformThatFallsWithSamus_Main; 10723079 => [()];
        10723080 => sm_a3::FastMovingSlowSinkingPlatform_Shot; 10723661 => sm_a3::Roach_Init;
        10724048 => sm_a3::Roach_Main; 10725245 => sm_a3::Mochtroid_Init;
        10725264 => sm_a3::Mochtroid_Main; 10725715 => sm_a3::Mochtroid_Touch;
        10725800 => sm_a3::Mochtroid_Shot; 10726153 => sm_a3::Sidehopper_Init;
        10726351 => sm_a3::Sidehopper_Main; 10728522 => sm_a3::MaridiaRefillCandy_Init;
        10728572 => sm_a3::MaridiaRefillCandy_Main; 10729071 => sm_a3::NorfairSlowFireball_Init;
        10730163 => sm_a3::Bang_Init; 10730277 => sm_a3::Bang_Main; 10731261 => sm_a3::Bang_Shot;
        10733230 => sm_a3::Skree_Init; 10733255 => sm_a3::Skree_Main; 10733557 => sm_a3::Skree_Shot;
        10735074 => sm_a3::MaridiaSnail_Init; 10735204 => sm_a3::MaridiaSnail_Main;
        10736560 => sm_a3::MaridiaSnail_Touch; 10736745 => sm_a3::MaridiaSnail_Shot;
        10738643 => sm_a3::Reflec_Init; 10738715 => [()]; 10738716 => sm_a3::Reflec_Shot;
        10739779 => sm_a3::WreckedShipOrangeZoomer_Init;
        10739851 => sm_a3::WreckedShipOrangeZoomer_Main; 10740436 => sm_a3::BigEyeBugs_Init;
        10741148 => sm_a3::FireZoomer_Init; 10741353 => sm_a3::StoneZoomer_Init;
        10741442 => sm_a3::StoneZoomer_Main; 10742351 => sm_a3::Metroid_Init;
        10742502 => sm_a3::Metroid_Frozen; 10742579 => sm_a3::Metroid_Hurt;
        10742680 => sm_a3::Metroid_Main; 10743275 => sm_a3::Metroid_Touch;
        10743559 => sm_a3::Metroid_Shot;
        10743874 => [{
                    sm_a3::Metroid_Powerbomb(word!(game.cur_enemy_index), game);
                }];
        10780677 => sm_a4::Enemy_GrappleReact_SamusLatchesOn_A4;
        10780707 => sm_a4::Enemy_NormalTouchAI_A4; 10780717 => sm_a4::Enemy_NormalShotAI_A4;
        10780737 => sm_a4::Enemy_NormalFrozenAI_A4; 10780748 => [()];
        10782343 => sm_a4::Crocomire_Hurt; 10783322 => sm_a4::Crocomire_Init;
        10783748 => sm_a4::Crocomire_Main; 10795344 => [()]; 10795410 => sm_a4::Crocomire_Powerbomb;
        10811002 => sm_a4::CrocomireTongue_Init; 10811067 => [sm_a4::CrocomireTongue_Main()];
        10846208 => sm_a5::Enemy_GrappleReact_NoInteract_A5;
        10846223 => sm_a5::Enemy_GrappleReact_CancelBeam_A5;
        10846243 => sm_a5::Enemy_NormalTouchAI_A5; 10846253 => sm_a5::Enemy_NormalShotAI_A5;
        10846273 => sm_a5::Enemy_NormalFrozenAI_A5; 10846284 => [()];
        10847879 => sm_a5::Draygon_Init; 10847996 => sm_a5::Draygon_Main;
        10851661 => sm_a5::Draygon_Hurt; 10851818 => sm_a5::Draygon_Touch;
        10851824 => sm_a5::Draygon_Shot; 10851847 => sm_a5::Draygon_Powerbomb;
        10863723 => sm_a5::DraygonsEye_Init; 10863750 => sm_a5::DraygonsEye_Main;
        10864025 => sm_a5::DraygonsTail_Init; 10864042 => [()];
        10864045 => sm_a5::DraygonsArms_Init; 10864068 => [()]; 10873386 => sm_a5::SporeSpawn_Init;
        10873619 => sm_a5::SporeSpawn_Main; 10874202 => sm_a5::SporeSpawn_Shot;
        10874348 => sm_a5::SporeSpawn_Touch; 10874354 => [()];
        10911744 => sm_a6::Enemy_GrappleReact_NoInteract_A6;
        10911759 => sm_a6::Enemy_GrappleReact_CancelBeam_A6;
        10911779 => sm_a6::Enemy_NormalTouchAI_A6; 10911789 => sm_a6::Enemy_NormalShotAI_A6;
        10911809 => sm_a6::Enemy_NormalFrozenAI_A6; 10911820 => [()];
        10913525 => sm_a6::Boulder_Init; 10913683 => sm_a6::Boulder_Main;
        10914607 => sm_a6::SpikeyPlatform_Init; 10914693 => sm_a6::SpikeyPlatform2ndEnemy_Init;
        10914713 => sm_a6::SpikeyPlatform2ndEnemy_Main; 10914733 => sm_a6::SpikeyPlatform_Main;
        10915836 => sm_a6::FireGeyser_Init; 10915875 => sm_a6::FireGeyser_Main;
        10917060 => sm_a6::NuclearWaffle_Init; 10917390 => sm_a6::NuclearWaffle_Main;
        10918488 => sm_a6::FakeKraid_Init; 10918594 => sm_a6::FakeKraid_Main;
        10918946 => sm_a6::FakeKraid_Touch; 10918969 => sm_a6::FakeKraid_Shot;
        10920181 => sm_a6::CeresRidley_Init; 10920584 => sm_a6::CeresRidley_Main;
        10920659 => sm_a6::CeresRidley_Hurt; 10924583 => sm_a6::Ridley_Main;
        10924682 => sm_a6::Ridley_Func_2; 10924695 => sm_a6::Ridley_Hurt;
        10929814 => sm_a6::RidleysExplosion_Init; 10930388 => sm_a6::RidleysExplosion_Main;
        10936202 => sm_a6::Ridley_Shot; 10936242 => sm_a6::Ridley_Powerbomb;
        10940337 => sm_a6::CeresSteam_Init; 10940429 => sm_a6::CeresSteam_Main;
        10940479 => sm_a6::CeresSteam_Touch; 10942149 => sm_a6::CeresDoor_Init;
        10942309 => sm_a6::CeresDoor_Main; 10942752 => [()]; 10943346 => sm_a6::Zebetites_Init;
        10943539 => sm_a6::Zebetites_Main; 10943911 => sm_a6::Zebetites_Touch;
        10943916 => sm_a6::Zebetites_Shot; 10977280 => sm_a7::Enemy_GrappleReact_NoInteract_A7;
        10977295 => sm_a7::Enemy_GrappleReact_CancelBeam_A7;
        10977325 => sm_a7::Enemy_NormalShotAI_A7;
        10977340 => sm_a7::Enemy_NormalPowerBombAI_SkipDeathAnim_A7;
        10977345 => sm_a7::Enemy_NormalFrozenAI_A7; 10977356 => [()]; 10982543 => [()];
        10982544 => sm_a7::KraidsArm_Touch; 10982559 => sm_a7::Kraid_Touch; 10982581 => [()];
        10987865 => sm_a7::Kraid_Init; 10988355 => sm_a7::KraidsArm_Init;
        10988392 => sm_a7::KraidsTopLint_Init; 10988444 => sm_a7::KraidsMiddleLint_Init;
        10988490 => sm_a7::KraidsBottomLint_Init; 10988536 => sm_a7::KraidsFoot_Init;
        10988577 => sm_a7::Kraid_Main; 10991549 => sm_a7::KraidsArm_Main;
        10991617 => sm_a7::KraidsTopLint_Main; 10991629 => sm_a7::KraidsMiddleLint_Main;
        10991641 => sm_a7::KraidsBottomLint_Main; 10992118 => sm_a7::KraidsFoot_Main;
        10992847 => sm_a7::KraidsGoodFingernail_Touch; 10992862 => sm_a7::KraidsBadFingernail_Touch;
        10992879 => sm_a7::KraidsGoodFingernail_Init; 10992941 => sm_a7::KraidsBadFingernail_Init;
        10992946 => sm_a7::KraidsGoodFingernail_Main; 10992969 => sm_a7::KraidsBadFingernail_Main;
        10997235 => sm_a7::Phantoon_Init; 10997333 => sm_a7::Phantoon2_Init;
        10997414 => sm_a7::Phantoon_Main; 11001151 => sm_a7::Phantoon_Hurt;
        11001237 => sm_a7::Phantoon_Touch; 11001242 => [()]; 11001243 => sm_a7::Phantoon_Shot;
        11001873 => [()]; 11004178 => sm_a7::Etecoon_Init; 11004224 => sm_a7::Etecoon_Main;
        11007197 => sm_a7::Dachora_Init; 11007278 => sm_a7::Dachora_Main;
        11042816 => sm_a8::Enemy_GrappleReact_NoInteract_A8;
        11042826 => sm_a8::Enemy_GrappleReact_KillEnemy_A8;
        11042831 => sm_a8::Enemy_GrappleReact_CancelBeam_A8;
        11042836 => sm_a8::Enemy_GrappleReact_SamusLatchesNoInvinc_A8;
        11042846 => sm_a8::Enemy_GrappleReact_HurtSamus_A8;
        11042851 => sm_a8::Enemy_NormalTouchAI_A8; 11042861 => sm_a8::Enemy_NormalShotAI_A8;
        11042881 => sm_a8::Enemy_NormalFrozenAI_A8; 11042892 => [()];
        11044832 => sm_a8::MiniDraygon_Init; 11045040 => sm_a8::EvirProjectile_Init;
        11045147 => sm_a8::MiniDraygon_Main; 11045278 => sm_a8::EvirProjectile_Main;
        11045638 => sm_a8::MiniDraygon_Touch; 11045644 => sm_a8::MiniDraygon_Powerbomb;
        11045650 => sm_a8::MiniDraygon_Shot; 11047000 => sm_a8::MorphBallEye_Init;
        11047138 => sm_a8::MorphBallEye_Main; 11048675 => sm_a8::Fune_Init;
        11048752 => sm_a8::Fune_Main; 11049710 => sm_a8::WreckedShipGhost_Init;
        11049788 => sm_a8::WreckedShipGhost_Main; 11051336 => sm_a8::YappingMaw_Init;
        11051537 => sm_a8::YappingMaw_Main; 11052953 => sm_a8::YappingMaw_Touch;
        11052989 => sm_a8::YappingMaw_Shot; 11053109 => sm_a8::YappingMaw_Frozen;
        11053894 => sm_a8::Kago_Init; 11053941 => sm_a8::Kago_Main; 11053955 => sm_a8::Kago_Shot;
        11054987 => sm_a8::NorfairLavaMan_Init; 11055370 => sm_a8::NorfairLavaMan_Main;
        11056128 => sm_a8::NorfairLavaMan_Powerbomb; 11056134 => sm_a8::NorfairLavaMan_Touch;
        11056140 => sm_a8::NorfairLavaMan_Shot; 11057014 => sm_a8::Beetom_Init;
        11057165 => sm_a8::Beetom_Main; 11058734 => sm_a8::Beetom_Touch;
        11058860 => sm_a8::Beetom_Shot; 11059657 => sm_a8::MaridiaFloater_Init;
        11059740 => sm_a8::MaridiaFloater_Main; 11060670 => sm_a8::MaridiaFloater_Touch;
        11060719 => sm_a8::MaridiaFloater_Shot; 11060799 => sm_a8::MaridiaFloater_Powerbomb;
        11062135 => sm_a8::WreckedShipRobot_Init;
        11062220 => sm_a8::WreckedShipRobotDeactivated_Init;
        11062326 => sm_a8::WreckedShipRobot_Main; 11062374 => [()];
        11063668 => sm_a8::WreckedShipRobotDeactivated_Touch;
        11063693 => sm_a8::WreckedShipRobotDeactivated_Shot;
        11063698 => sm_a8::WreckedShipRobot_Shot; 11065545 => sm_a8::MaridiaPuffer_Init;
        11065611 => sm_a8::MaridiaPuffer_Main; 11066132 => sm_a8::MaridiaPuffer_Shot;
        11066573 => sm_a8::WalkingLavaSeahorse_Init; 11066731 => sm_a8::WalkingLavaSeahorse_Main;
        11068296 => sm_a8::WreckedShipOrbs_Init; 11068355 => sm_a8::WreckedShipOrbs_Main;
        11068983 => sm_a8::WreckedShipSpark_Init; 11069070 => sm_a8::WreckedShipSpark_Main;
        11069198 => sm_a8::WreckedShipSpark_Shot; 11069486 => sm_a8::BlueBrinstarFaceBlock_Init;
        11069614 => sm_a8::BlueBrinstarFaceBlock_Main;
        11069725 => sm_a8::BlueBrinstarFaceBlock_Shot; 11071880 => sm_a8::KiHunter_Init;
        11072020 => sm_a8::KiHunterWings_Init; 11072092 => sm_a8::KiHunter_Main;
        11072098 => sm_a8::KiHunterWings_Main; 11073281 => sm_a8::KiHunter_Shot;
        11108367 => sm_a9::Enemy_GrappleReact_CancelBeam_A9;
        11108417 => sm_a9::Enemy_NormalFrozenAI_A9; 11108428 => [()];
        11110023 => sm_a9::MotherBrainsBody_Init; 11110149 => sm_a9::MotherBrainsBrain_Init;
        11110206 => sm_a9::MotherBrainsBody_Hurt; 11110279 => sm_a9::MotherBrainsBody_Powerbomb;
        11110283 => sm_a9::MotherBrainsBrain_Hurt; 11111221 => sm_a9::MotherBrainsTubesFalling_Init;
        11111301 => [{
                    sm_a9::MotherBrainsTubesFalling_Main(word!(game.cur_enemy_index), game);
                }];
        11121923 => sm_a9::MotherBrainsBody_Shot; 11121927 => sm_a9::MotherBrainsBrain_Shot;
        11122117 => [()]; 11122118 => sm_a9::MotherBrainsBrain_Touch;
        11126544 => sm_a9::ShitroidInCutscene_Init; 11126649 => sm_a9::ShitroidInCutscene_Main;
        11128579 => sm_a9::ShitroidInCutscene_Touch; 11129608 => sm_a9::DeadTorizo_Init;
        11129704 => sm_a9::DeadTorizo_Main; 11129898 => sm_a9::DeadTorizo_Powerbomb;
        11129907 => sm_a9::DeadTorizo_Shot; 11130806 => sm_a9::DeadSidehopper_Init;
        11130953 => sm_a9::DeadZoomer_Init; 11130998 => sm_a9::DeadRipper_Init;
        11131039 => sm_a9::DeadSkree_Init; 11131084 => sm_a9::DeadSidehopper_Powerbomb;
        11131099 => sm_a9::DeadSidehopper_Main; 11132141 => sm_a9::DeadZoomer_Powerbomb;
        11132152 => sm_a9::DeadZoomer_Shot; 11132157 => sm_a9::DeadRipper_Powerbomb;
        11132168 => sm_a9::DeadRipper_Shot; 11132173 => sm_a9::DeadSkree_Powerbomb;
        11132184 => sm_a9::DeadSkree_Shot; 11132189 => sm_a9::DeadSidehopper_Shot;
        11132228 => sm_a9::DeadSidehopper_Touch; 11136823 => sm_a9::Shitroid_Init;
        11136954 => sm_a9::Shitroid_Powerbomb; 11136965 => sm_a9::Shitroid_Main;
        11138953 => sm_a9::Shitroid_Touch; 11139138 => sm_a9::Shitroid_Shot;
        11173903 => sm_aa::Enemy_GrappleReact_CancelBeam_AA;
        11173953 => sm_aa::Enemy_NormalFrozenAI_AA; 11173964 => [()];
        11191934 => sm_aa::Torizo_Hurt; 11191972 => sm_aa::Torizo_Main;
        11192447 => sm_aa::Torizo_Init; 11192695 => sm_aa::GoldTorizo_Touch;
        11192700 => sm_aa::Torizo_Shot; 11195241 => sm_aa::GoldTorizo_Main;
        11195322 => sm_aa::GoldTorizo_Hurt; 11196007 => sm_aa::GoldTorizo_Shot; 11196359 => [()];
        11196360 => sm_aa::TourianEntranceStatue_Init; 11197603 => sm_aa::Shaktool_Hurt;
        11198019 => sm_aa::Shaktool_Init; 11198255 => sm_aa::Shaktool_Touch;
        11198260 => sm_aa::Shaktool_Shot; 11200278 => sm_aa::N00bTubeCracks_Init;
        11200293 => sm_aa::ChozoStatue_Init; 11200423 => sm_aa::ChozoStatue_Main; 11200475 => [()];
        11200476 => [()]; 11698191 => sm_b2::Enemy_GrappleReact_CancelBeam_B2;
        11698241 => sm_b2::Enemy_NormalFrozenAI_B2; 11698252 => [()];
        11700071 => sm_b2::WalkingSpacePirates_Powerbomb;
        11700076 => sm_b2::WalkingSpacePirates_Touch; 11700089 => sm_b2::WalkingSpacePirates_Shot;
        11726751 => sm_b2::WallSpacePirates_Init; 11726893 => sm_b2::WallSpacePirates_Main;
        11728350 => sm_b2::NinjaSpacePirates_Init; 11728546 => sm_b2::NinjaSpacePirates_Main;
        11730178 => sm_b2::WalkingSpacePirates_Init; 11730226 => sm_b2::WalkingSpacePirates_Main;
        11763712 => sm_b3::Enemy_GrappleReact_NoInteract_B3;
        11763722 => sm_b3::Enemy_GrappleReact_KillEnemy_B3;
        11763727 => sm_b3::Enemy_GrappleReact_CancelBeam_B3;
        11763747 => sm_b3::Enemy_NormalTouchAI_B3; 11763757 => sm_b3::Enemy_NormalShotAI_B3;
        11763777 => sm_b3::Enemy_NormalFrozenAI_B3; 11763788 => [()];
        11765499 => sm_b3::UnusedSpinningTurtleEye_Init;
        11765518 => [sm_b3::UnusedSpinningTurtleEye_Main()];
        11765819 => sm_b3::BrinstarPipeBug_Init; 11765882 => sm_b3::BrinstarPipeBug_Main;
        11766625 => sm_b3::NorfairPipeBug_Init; 11766686 => sm_b3::NorfairPipeBug_Main;
        11767628 => sm_b3::BrinstarYellowPipeBug_Init;
        11767726 => sm_b3::BrinstarYellowPipeBug_Main; 11769219 => sm_b3::Botwoon_Init;
        11769448 => sm_b3::Botwoon_Main; 11771903 => sm_b3::Botwoon_Touch;
        11771926 => sm_b3::Botwoon_Shot; 11771969 => sm_b3::Botwoon_Powerbomb;
        11789909 => sm_b3::EscapeEtecoon_Main; 11790027 => sm_b3::EscapeEtecoon_Init;
        11791077 => sm_b3::EscapeDachora_Init; 11791130 => [()];
    }
}
crate::game::address_dispatch! {
    pub unsafe fn CallEnemyPreInstr(ea: u32, game: &mut Game) => ea;
        setup { let k: u16 = word!(game.cur_enemy_index); }
        call (k, game); default { Unreachable(game); };
    {
        10518651 => [()]; 10649723 => [()]; 10652227 => sm_a2::MiniCrocomire_PreInstr5;
        10652252 => sm_a2::MiniCrocomire_PreInstr6; 10652277 => [()];
        10657722 => sm_a2::SpikeShootingPlant_2; 10657772 => sm_a2::SpikeShootingPlant_3;
        10657819 => [()]; 10659799 => [()]; 10659852 => sm_a2::GunshipTop_3;
        10660048 => sm_a2::GunshipTop_4; 10660162 => sm_a2::GunshipTop_5;
        10660176 => sm_a2::GunshipTop_6; 10660231 => sm_a2::GunshipTop_7;
        10660285 => sm_a2::GunshipTop_8; 10660431 => sm_a2::GunshipTop_9;
        10660445 => sm_a2::GunshipTop_10; 10660500 => sm_a2::GunshipTop_11;
        10660514 => sm_a2::GunshipTop_12; 10660639 => sm_a2::GunshipTop_13;
        10660704 => sm_a2::GunshipTop_14; 10660718 => sm_a2::GunshipTop_15;
        10660773 => sm_a2::GunshipTop_16; 10660807 => sm_a2::GunshipTop_17;
        10660891 => sm_a2::GunshipTop_18; 10661079 => sm_a2::GunshipTop_19;
        10661134 => sm_a2::GunshipTop_20; 10661165 => sm_a2::GunshipTop_21;
        10662222 => sm_a2::Flies_4; 10662268 => sm_a2::Flies_5; 10662314 => sm_a2::Flies_6;
        10662354 => sm_a2::Flies_7; 10665692 => sm_a2::NorfairLavajumpingEnemy_Func_1;
        10665754 => sm_a2::NorfairLavajumpingEnemy_Func_2;
        10665790 => sm_a2::NorfairLavajumpingEnemy_Func_3;
        10665852 => sm_a2::NorfairLavajumpingEnemy_Func_4;
        10665916 => sm_a2::NorfairLavajumpingEnemy_Func_5; 10666625 => sm_a2::NorfairRio_Func_1;
        10666727 => sm_a2::NorfairRio_Func_2; 10666815 => sm_a2::NorfairRio_Func_3;
        10666849 => sm_a2::NorfairRio_Func_4; 10666929 => sm_a2::NorfairRio_Func_5;
        10667014 => sm_a2::NorfairRio_Func_6; 10667822 => sm_a2::LowerNorfairRio_Func_1;
        10667889 => sm_a2::LowerNorfairRio_Func_2; 10667963 => sm_a2::LowerNorfairRio_Func_3;
        10667990 => sm_a2::LowerNorfairRio_Func_4; 10668077 => sm_a2::LowerNorfairRio_Func_5;
        10668168 => sm_a2::LowerNorfairRio_Func_6; 10669926 => sm_a2::MaridiaLargeSnail_Func_7;
        10669993 => sm_a2::MaridiaLargeSnail_Func_8; 10675796 => sm_a2::LavaSeahorse_Func_1;
        10675885 => sm_a2::LavaSeahorse_Func_2; 10675953 => sm_a2::LavaSeahorse_Func_3;
        10676020 => sm_a2::LavaSeahorse_Func_4; 10676041 => sm_a2::LavaSeahorse_Func_5;
        10676097 => [()]; 10730661 => sm_a3::Bang_Func_6; 10730689 => [sm_a3::Bang_Func_7(k)];
        10730693 => sm_a3::Bang_Func_8; 10730780 => sm_a3::Bang_Func_10;
        10730796 => sm_a3::Bang_Func_11; 10735455 => [()]; 10736051 => sm_a3::MaridiaSnail_Func_15;
        10739850 => [()]; 10739857 => sm_a3::WreckedShipOrangeZoomer_Func_2; 10741441 => [()];
        10741448 => sm_a3::FireZoomer_Func_1; 10741637 => sm_a3::FireZoomer_Func_2;
        10741746 => sm_a3::FireZoomer_Func_3; 10998952 => [()];
        10998953 => sm_a7::Phantoon_Spawn8FireballsInCircleAtStart;
        10999022 => sm_a7::Phantoon_WaitBetweenSpawningAndSpinningFireballs;
        10999048 => sm_a7::Phantoon_SpawnFireballsBeforeFight;
        10999114 => sm_a7::Phantoon_WavyFadeIn; 10999190 => sm_a7::Phantoon_PickPatternForRound1;
        10999271 => sm_a7::Phantoon_MovePhantoonInFigure8ThenOpenEye;
        10999309 => sm_a7::Phantoon_EyeFollowsSamusUntilTimerRunsOut;
        10999388 => sm_a7::Phantoon_BecomesSolidAndBodyVuln; 10999416 => sm_a7::Phantoon_IsSwooping;
        10999481 => sm_a7::Phantoon_FadeoutWithSwoop; 10999508 => sm_a7::Phantoon_WaitAfterFadeOut;
        10999522 => sm_a7::Phantoon_MoveLeftOrRightAndPickEyeOpenPatt;
        10999597 => sm_a7::Phantoon_FadeInBeforeFigure8;
        10999615 => sm_a7::Phantoon_BecomeSolidAfterRainingFireballs;
        10999655 => sm_a7::Phantoon_FadeInDuringFireballRain;
        10999688 => sm_a7::Phantoon_FollowSamusWithEyeDuringFireballRain;
        10999765 => sm_a7::Phantoon_FadeOutDuringFireballRain;
        10999799 => sm_a7::Phantoon_SpawnRainingFireballs;
        10999850 => sm_a7::Phantoon_FadeOutBeforeFirstFireballRain;
        10999900 => sm_a7::Phantoon_FadeOutBeforeEnrage;
        10999924 => sm_a7::Phantoon_MoveEnragedPhantoonToTopCenter;
        10999953 => sm_a7::Phantoon_FadeInEnragedPhantoon; 10999980 => sm_a7::Phantoon_Enraged;
        11000086 => sm_a7::Phantoon_FadeoutAfterEnrage;
        11000110 => sm_a7::Phantoon_CompleteSwoopAfterFatalShot;
        11000136 => sm_a7::Phantoon_DyingPhantoonFadeInOut;
        11000203 => sm_a7::Phantoon_DyingPhantoonExplosions;
        11000401 => sm_a7::Phantoon_WavyDyingPhantoonAndCry;
        11000454 => sm_a7::Phantoon_DyingFadeOut; 11000535 => sm_a7::Phantoon_AlmostDead;
        11000637 => sm_a7::Phantoon_Dead; 11004335 => sm_a7::Etecoon_Func_4;
        11004416 => sm_a7::Etecoon_Func_5; 11004471 => sm_a7::Etecoon_Func_6;
        11004597 => sm_a7::Etecoon_Func_7; 11004674 => sm_a7::Etecoon_Func_8;
        11004716 => sm_a7::Etecoon_Func_9; 11004752 => sm_a7::Etecoon_Func_10;
        11004877 => sm_a7::Etecoon_Func_11; 11004955 => sm_a7::Etecoon_Func_12;
        11005079 => sm_a7::Etecoon_Func_16; 11005115 => sm_a7::Etecoon_Func_17;
        11005151 => sm_a7::Etecoon_Func_18; 11005193 => sm_a7::Etecoon_Func_19;
        11005226 => sm_a7::Etecoon_Func_20; 11005268 => sm_a7::Etecoon_Func_21;
        11005301 => sm_a7::Etecoon_Func_22; 11005383 => sm_a7::Etecoon_Func_23;
        11005502 => sm_a7::Etecoon_Func_24; 11005594 => sm_a7::Etecoon_Func_25;
        11005624 => sm_a7::Etecoon_Func_26; 11007344 => sm_a7::Dachora_Func_2;
        11007420 => sm_a7::Dachora_Func_3; 11007469 => sm_a7::Dachora_Func_4;
        11007582 => sm_a7::Dachora_Func_5; 11007887 => sm_a7::Dachora_Func_7;
        11008006 => sm_a7::Dachora_Func_8; 11008309 => sm_a7::Dachora_Func_11;
        11008396 => sm_a7::Dachora_Func_12; 11047153 => sm_a8::MorphBallEye_Func_1;
        11047214 => sm_a8::MorphBallEye_Func_2; 11047264 => sm_a8::MorphBallEye_Func_3;
        11047374 => sm_a8::MorphBallEye_Func_4; 11047388 => [()];
        11055386 => sm_a8::NorfairLavaMan_Func_7; 11055477 => sm_a8::NorfairLavaMan_Func_8;
        11055507 => sm_a8::sub_A8B193; 11055544 => sm_a8::NorfairLavaMan_Func_9;
        11055581 => sm_a8::NorfairLavaMan_Func_10; 11055620 => sm_a8::NorfairLavaMan_Func_11;
        11055761 => [sm_a8::sub_A8B291(k)]; 11055765 => [sm_a8::NorfairLavaMan_Func_13(game)];
        11055885 => sm_a8::NorfairLavaMan_Func_15; 11055903 => sm_a8::NorfairLavaMan_Func_16;
        11055958 => sm_a8::NorfairLavaMan_Func_17; 11056039 => sm_a8::NorfairLavaMan_Func_18;
        11059843 => sm_a8::MaridiaFloater_Func_3; 11059878 => sm_a8::MaridiaFloater_Func_4;
        11059919 => sm_a8::MaridiaFloater_Func_5; 11060075 => sm_a8::MaridiaFloater_Func_6;
        11060193 => sm_a8::MaridiaFloater_Func_7; 11060329 => sm_a8::MaridiaFloater_Func_8;
        11060444 => sm_a8::MaridiaFloater_Func_9; 11060480 => sm_a8::MaridiaFloater_Func_10;
        11060509 => sm_a8::MaridiaFloater_Func_11; 11060584 => [()];
        11060585 => sm_a8::MaridiaFloater_Func_12; 11060639 => sm_a8::MaridiaFloater_Func_13;
        11192670 => [()]; 11197610 => [()]; 11200422 => [()]; 11174011 => [()]; 11763835 => [()];
        11765888 => sm_b3::BrinstarPipeBug_PreInstr_1;
        11765904 => sm_b3::BrinstarPipeBug_PreInstr_2;
        11765987 => sm_b3::BrinstarPipeBug_PreInstr_3;
        11766044 => sm_b3::BrinstarPipeBug_PreInstr_4;
        11766142 => sm_b3::BrinstarPipeBug_PreInstr_5; 11767039 => sm_b3::NorfairPipeBug_Func_5;
        11771328 => sm_b3::Botwoon_Func_26; 11771517 => sm_b3::Botwoon_Func_27;
        11771616 => sm_b3::Botwoon_Func_28; 11771700 => sm_b3::Botwoon_Func_29;
        11771770 => sm_b3::Botwoon_Func_30; 11192146 => sm_aa::Torizo_Func_5;
        11192360 => sm_aa::Torizo_Func_6; 11192364 => sm_aa::Torizo_Func_7;
        11195885 => sm_aa::Torizo_D5ED; 11195889 => sm_aa::Torizo_D5F1;
        11199557 => sm_aa::sub_AAE445; 11200430 => sm_aa::Shaktool_PreInstr_0; 11200474 => [()];
        11789916 => sm_b3::EscapeEtecoon_E65C; 11789936 => sm_b3::EscapeEtecoon_E670;
        11789952 => sm_b3::EscapeEtecoon_E680; 10740072 => sm_a3::sub_A3E168;
        10735456 => sm_a3::MaridiaSnail_Func_7; 10735526 => sm_a3::MaridiaSnail_Func_9;
        10735543 => sm_a3::MaridiaSnail_CFB7; 10735549 => sm_a3::MaridiaSnail_CFBD;
        10735566 => sm_a3::MaridiaSnail_CFCE; 10735572 => sm_a3::MaridiaSnail_CFD4;
        10735589 => sm_a3::MaridiaSnail_CFE5; 10735595 => sm_a3::MaridiaSnail_CFEB;
        10735612 => sm_a3::MaridiaSnail_CFFC; 11197611 => [()]; 11197612 => sm_aa::Shaktool_DCAC;
        11197655 => sm_aa::Shaktool_DCD7; 11197733 => sm_aa::Shaktool_DD25;
    }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallEnemyInstr(ea: u32, k: u16, j: *const u16, game: &mut Game) -> *const u16 => ea;
    call (k, j, game); default {
            Unreachable(game);
            null::<u16>()
        };
    @functions [pub fn (k: u16, _jp: *const u16, game: &mut Game) -> *const u16];
    [10846332 | 11174012] EnemyInstr_StopScript [] {
        let properties = game.enemy_instructions(k).properties() | kEnemyProps_Deleted as u16;
        game.enemy_instructions(k).set_properties(properties);
        core::ptr::null()
    };
    @functions [pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16];
    [10649872 | 10846480 | 10977552 | 11043088 | 11174160 | 11698448 | 11763984]
    EnemyInstr_DecTimerAndGoto [] {
        let timer = game.enemy_instructions(k).timer();
        let decremented = game.enemy_instructions(k).timer().wrapping_sub(1);
        game.enemy_instructions(k).set_timer(decremented);
        if timer == 1 {
            jp.add(1)
        } else {
            EnemyInstr_Goto(k, jp)
        }
    };
    [10649891 | 10846499 | 10977571 | 11043107 | 11174179 | 11698467 | 11764003]
    EnemyInstr_SetTimer [jp.add(1)] { game.enemy_instructions(k).set_timer(*jp); };
    [10649903 | 10715439 | 10780975 | 10846511 | 10912047 | 10977583 | 11043119 | 11108655 | 11174191 | 11698479 | 11764015]
    EnemyInstr_Sleep [core::ptr::null()] {
        let base_ptr = RomBankBase(game.enemy_instructions(k).bank(), game);
        let instruction = (jp as *const u8).sub(2).offset_from(base_ptr) as u16;
        game.enemy_instructions(k)
            .set_current_instruction(instruction);
    };
    [10846522 | 11174202 | 11698490] EnemyInstr_WaitNframes [core::ptr::null()] {
        let base_ptr = RomBankBase(game.enemy_instructions(k).bank(), game);
        game.enemy_instructions(k).set_instruction_timer(*jp);
        let instruction = (jp as *const u8).add(2).offset_from(base_ptr) as u16;
        game.enemy_instructions(k)
            .set_current_instruction(instruction);
    };
    @functions [pub unsafe fn (_k: u16, jp: *const u16, game: &mut Game) -> *const u16];
    [11174219] EnemyInstr_CopyToVram [] {
        let tail = game.vram_write_queue_tail();
        let cursor = game.vram_write_queue_tail();
        let source = jp as *const u8;
        let word = (source as *const u16).read_unaligned();
        game.write_vram_queue_word(cursor, 0, word);
        let word = (source.add(2) as *const u16).read_unaligned();
        game.write_vram_queue_word(cursor, 2, word);
        let word = (source.add(3) as *const u16).read_unaligned();
        game.write_vram_queue_word(cursor, 3, word);
        let word = (source.add(5) as *const u16).read_unaligned();
        game.write_vram_queue_word(cursor, 5, word);
        game.set_vram_write_queue_tail(tail.wrapping_add(7));
        (jp as usize).wrapping_add(7) as *const u16
    };
    @functions [pub fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16];
    [10649971 | 10715507 | 11043187] EnemyInstr_EnableOffScreenProcessing [] {
        let properties = game.enemy_instructions(k).properties() | 0x800;
        game.enemy_instructions(k).set_properties(properties);
        jp
    };
    @functions [pub fn (
    k: u16,
    jp: *const u16,
    game: &mut Game,
) -> *const u16];
    [10649981 | 10715517 | 11043197] EnemyInstr_DisableOffScreenProcessing [] {
        let properties = game.enemy_instructions(k).properties() & !0x800;
        game.enemy_instructions(k).set_properties(properties);
        jp
    };
    @external {
        10649837 => [EnemyInstr_Goto(k, j)];
        10651845 => [sm_a2::BouncingGoofball_Instr_88C5(k, j)];
        10651846 => sm_a2::BouncingGoofball_Instr_88C6; 10652030 => sm_a2::MiniCrocomire_Instr_897E;
        10652048 => sm_a2::MiniCrocomire_Instr_8990; 10652061 => sm_a2::MiniCrocomire_Instr_899D;
        10654593 => sm_a2::MaridiaBeybladeTurtle_Instr_9381;
        10654738 => sm_a2::MaridiaBeybladeTurtle_Instr_9412;
        10654791 => sm_a2::MaridiaBeybladeTurtle_Instr_9447;
        10654801 => sm_a2::MaridiaBeybladeTurtle_Instr_9451;
        10654827 => sm_a2::MaridiaBeybladeTurtle_Instr_946B;
        10654853 => sm_a2::MaridiaBeybladeTurtle_Instr_9485;
        10654881 => sm_a2::MaridiaBeybladeTurtle_Instr_94A1;
        10654919 => sm_a2::MaridiaBeybladeTurtle_Instr_94C7;
        10654929 => sm_a2::MaridiaBeybladeTurtle_Instr_94D1;
        10657578 => sm_a2::SpikeShootingPlant_Instr_9F2A;
        10657941 => sm_a2::SpikeShootingPlant_Instr_A095;
        10657959 => sm_a2::SpikeShootingPlant_Instr_A0A7;
        10659181 => sm_a2::MaridiaSpikeyShell_Instr_A56D;
        10659185 => sm_a2::MaridiaSpikeyShell_Instr_A571; 10664371 => sm_a2::Rinka_Instr_B9B3;
        10664381 => sm_a2::Rinka_Instr_B9BD; 10664391 => sm_a2::Rinka_Instr_B9C7;
        10664899 => sm_a2::EnemyInstr_Rio_Instr_1;
        10665614 => sm_a2::NorfairLavajumpingEnemy_Instr_BE8E;
        10666441 => sm_a2::NorfairRio_Instr_C1C9; 10666452 => sm_a2::NorfairRio_Instr_C1D4;
        10666463 => sm_a2::NorfairRio_Instr_C1DF; 10666474 => sm_a2::NorfairRio_Instr_C1EA;
        10666485 => sm_a2::NorfairRio_Instr_C1F5; 10666496 => sm_a2::NorfairRio_Instr_C200;
        10666507 => sm_a2::NorfairRio_Instr_C20B; 10666518 => sm_a2::NorfairRio_Instr_C216;
        10666529 => sm_a2::NorfairRio_Instr_C221; 10666540 => sm_a2::NorfairRio_Instr_C22C;
        10666551 => sm_a2::NorfairRio_Instr_C237; 10667730 => sm_a2::LowerNorfairRio_Instr_C6D2;
        10667741 => sm_a2::LowerNorfairRio_Instr_C6DD;
        10667752 => sm_a2::LowerNorfairRio_Instr_C6E8;
        10668907 => sm_a2::MaridiaLargeSnail_Instr_CB6B;
        10669235 => sm_a2::MaridiaLargeSnail_Instr_CCB3;
        10669246 => sm_a2::MaridiaLargeSnail_Instr_CCBE;
        10669257 => sm_a2::MaridiaLargeSnail_Instr_CCC9; 10675707 => sm_a2::LavaSeahorse_Instr_E5FB;
        10715373 => [EnemyInstr_Goto(k, j)];
        10716899 => sm_a3::Waver_Instr_1;
        10717526 => sm_a3::Metalee_Instr_1; 10719382 => sm_a3::MaridiaFish_Instr_3;
        10719392 => sm_a3::MaridiaFish_Instr_1; 10719402 => sm_a3::MaridiaFish_Instr_2;
        10722411 => sm_a3::PlatformThatFallsWithSamus_Instr_3;
        10722422 => sm_a3::PlatformThatFallsWithSamus_Instr_4;
        10722433 => sm_a3::PlatformThatFallsWithSamus_Instr_1;
        10722444 => sm_a3::PlatformThatFallsWithSamus_Instr_2; 10725992 => sm_a3::Sidehopper_Func_1;
        10726142 => sm_a3::Sidehopper_Instr_1; 10728489 => sm_a3::MaridiaRefillCandy_Instr_1;
        10728500 => sm_a3::MaridiaRefillCandy_Instr_2;
        10728511 => sm_a3::MaridiaRefillCandy_Instr_3; 10730104 => sm_a3::Bang_Instr_1;
        10730152 => sm_a3::Bang_Instr_2; 10733220 => sm_a3::Skree_Instr_1;
        10734646 => sm_a3::MaridiaSnail_Instr_1; 10734655 => sm_a3::MaridiaSnail_Instr_2;
        10734664 => sm_a3::MaridiaSnail_Instr_4; 10734687 => sm_a3::MaridiaSnail_Instr_3;
        10734712 => sm_a3::MaridiaSnail_Instr_5; 10738632 => sm_a3::Reflec_Instr_1;
        10739650 => sm_a3::WreckedShipOrangeZoomer_Func_1;
        10741344 => sm_a3::Zoomer_Instr_SetPreinstr; 10742437 => sm_a3::Metroid_Instr_2;
        10742449 => sm_a3::Metroid_Instr_1; 10780909 => [EnemyInstr_Goto(k, j)];
        10782374 => sm_a4::Crocomire_Instr_1;
        10782546 => sm_a4::Crocomire_Instr_14; 10783995 => sm_a4::Crocomire_Instr_11;
        10784007 => sm_a4::Crocomire_Instr_7; 10784019 => sm_a4::Crocomire_Instr_19;
        10784711 => sm_a4::Crocomire_Instr_2; 10784735 => sm_a4::Crocomire_Instr_4;
        10784762 => sm_a4::Crocomire_Instr_3; 10784767 => sm_a4::Crocomire_Instr_15;
        10784797 => sm_a4::Crocomire_Instr_16; 10784859 => sm_a4::Crocomire_Instr_13;
        10784895 => sm_a4::Crocomire_Instr_18; 10784911 => sm_a4::Crocomire_Instr_12;
        10784916 => sm_a4::Crocomire_Instr_17; 10787483 => sm_a4::Crocomire_Instr_8;
        10787488 => sm_a4::Crocomire_Instr_6; 10787493 => sm_a4::Crocomire_Instr_9;
        10787498 => sm_a4::Crocomire_Instr_5; 10787503 => sm_a4::Crocomire_Instr_20;
        10787508 => sm_a4::Crocomire_Instr_21; 10787513 => sm_a4::Crocomire_Instr_22;
        10787518 => sm_a4::Crocomire_Instr_23; 10787523 => sm_a4::Crocomire_Instr_24;
        10787528 => sm_a4::Crocomire_Instr_10; 10787533 => sm_a4::Crocomire_Instr_25;
        10787538 => sm_a4::Crocomire_Instr_26; 10787543 => sm_a4::Crocomire_Instr_27;
        10846445 => [EnemyInstr_Goto(k, j)];
        10851549 => sm_a5::Draygon_Instr_1; 10852150 => sm_a5::Draygon_Instr_13;
        10852159 => sm_a5::Draygon_Instr_8; 10852178 => sm_a5::Draygon_Instr_7;
        10852197 => sm_a5::Draygon_Instr_6; 10852216 => sm_a5::Draygon_Instr_9;
        10852501 => sm_a5::Draygon_Instr_2; 10852563 => sm_a5::Draygon_Instr_11;
        10852591 => sm_a5::Draygon_Instr_5; 10853274 => sm_a5::Draygon_Instr_15;
        10853514 => sm_a5::Draygon_Instr_17; 10853898 => sm_a5::Draygon_Instr_14;
        10854231 => sm_a5::Draygon_Instr_16; 10854240 => sm_a5::Draygon_Instr_10;
        10854254 => sm_a5::Draygon_Instr_4; 10854268 => sm_a5::Draygon_Instr_12;
        10854318 => sm_a5::Draygon_Instr_18; 10863739 => sm_a5::Draygon_Instr_3;
        10872671 => sm_a5::Draygon_Instr_25; 10872689 => sm_a5::Draygon_Instr_24;
        10872877 => sm_a5::Draygon_Instr_21; 10872946 => sm_a5::Draygon_Instr_22;
        10872956 => sm_a5::Draygon_Instr_27; 10872981 => sm_a5::Draygon_Instr_23;
        10873009 => sm_a5::Draygon_Instr_30; 10873018 => sm_a5::Draygon_Instr_20;
        10873034 => sm_a5::Draygon_Instr_29; 10873116 => sm_a5::Draygon_Instr_19;
        10873198 => sm_a5::Draygon_Instr_28; 10873265 => sm_a5::Draygon_Instr_26;
        10911981 => [EnemyInstr_Goto(k, j)];
        10915247 => sm_a6::FireGeyser_Instr_1; 10915347 => sm_a6::FireGeyser_Instr_2;
        10915373 => sm_a6::FireGeyser_Instr_3; 10915393 => sm_a6::FireGeyser_Instr_4;
        10915413 => sm_a6::FireGeyser_Instr_5; 10915433 => sm_a6::FireGeyser_Instr_6;
        10915453 => sm_a6::FireGeyser_Instr_7; 10915473 => sm_a6::FireGeyser_Instr_8;
        10915493 => sm_a6::FireGeyser_Instr_9; 10915513 => sm_a6::FireGeyser_Instr_10;
        10915533 => sm_a6::FireGeyser_Instr_11; 10915553 => sm_a6::FireGeyser_Instr_12;
        10915573 => sm_a6::FireGeyser_Instr_13; 10915593 => sm_a6::FireGeyser_Instr_14;
        10915613 => sm_a6::FireGeyser_Instr_15; 10915633 => sm_a6::FireGeyser_Instr_16;
        10915653 => sm_a6::FireGeyser_Instr_17; 10915673 => sm_a6::FireGeyser_Instr_18;
        10915693 => sm_a6::FireGeyser_Instr_19; 10915713 => sm_a6::FireGeyser_Instr_20;
        10915733 => sm_a6::FireGeyser_Instr_21; 10915753 => sm_a6::FireGeyser_Instr_22;
        10915773 => sm_a6::FireGeyser_Instr_23; 10915793 => sm_a6::FireGeyser_Instr_24;
        10918694 => sm_a6::FakeKraid_Instr_2; 10918772 => sm_a6::FakeKraid_Instr_1;
        10918834 => sm_a6::FakeKraid_Instr_3; 10918852 => sm_a6::FakeKraid_Instr_4;
        10918914 => sm_a6::FakeKraid_Instr_5; 10937534 => sm_a6::Ridley_Instr_5;
        10937546 => sm_a6::Ridley_Instr_6; 10937554 => sm_a6::Ridley_Instr_10;
        10937582 => sm_a6::Ridley_Instr_4; 10937592 => sm_a6::Ridley_Instr_3;
        10937601 => sm_a6::Ridley_Instr_2; 10937623 => sm_a6::Ridley_Instr_1;
        10937631 => sm_a6::Ridley_Instr_14; 10938140 => sm_a6::Ridley_Instr_9;
        10938151 => sm_a6::Ridley_Instr_7; 10938159 => sm_a6::Ridley_Instr_8;
        10938445 => sm_a6::Ridley_Instr_11; 10938628 => sm_a6::Ridley_Instr_12;
        10938633 => sm_a6::Ridley_Instr_13; 10938729 => sm_a6::Ridley_Instr_15;
        10938742 => sm_a6::Ridley_Instr_16; 10940701 => sm_a6::CeresSteam_Instr_1;
        10940711 => sm_a6::CeresSteam_Instr_2; 10940725 => sm_a6::CeresSteam_Instr_3;
        10942014 => sm_a6::CeresDoor_Instr_6; 10942058 => sm_a6::CeresDoor_Instr_4;
        10942072 => sm_a6::CeresDoor_Instr_8; 10942091 => sm_a6::CeresSteam_Instr_4;
        10942101 => sm_a6::CeresDoor_Instr_1; 10942111 => sm_a6::CeresDoor_Instr_3;
        10942118 => sm_a6::CeresSteam_Instr_5; 10942128 => sm_a6::CeresDoor_Instr_5;
        10942131 => sm_a6::CeresDoor_Instr_2; 10942141 => sm_a6::CeresDoor_Instr_7;
        10977418 => sm_a7::EnemyInstr_Call_A7; 10977517 => [EnemyInstr_Goto(k, j)];
        10979983 => sm_a7::Kraid_Instr_9;
        10991155 => [sm_a7::Kraid_Instr_1(k, j)]; 10991158 => sm_a7::Kraid_Instr_DecYpos;
        10991164 => sm_a7::Kraid_Instr_IncrYpos_Shake;
        10991182 => sm_a7::Kraid_Instr_PlaySound_0x76; 10991194 => sm_a7::Kraid_Instr_XposMinus3;
        10991207 => sm_a7::Kraid_Instr_XposMinus3b; 10991220 => sm_a7::Kraid_Instr_XposPlus3;
        10991235 => sm_a7::Kraid_Instr_MoveHimRight; 11043053 => [EnemyInstr_Goto(k, j)];
        11044751 => sm_a8::MiniDraygon_Instr_2;
        11044763 => sm_a8::MiniDraygon_Instr_1; 11044790 => sm_a8::MiniDraygon_Instr_3;
        11044811 => sm_a8::MiniDraygon_Instr_4; 11048485 => sm_a8::Fune_Instr_2;
        11048497 => sm_a8::Fune_Instr_6; 11048522 => sm_a8::Fune_Instr_7;
        11048547 => sm_a8::Fune_Instr_1; 11048572 => sm_a8::Fune_Instr_4;
        11048597 => sm_a8::Fune_Instr_3; 11048628 => sm_a8::Fune_Instr_5;
        11051207 => sm_a8::YappingMaw_Instr_2; 11051225 => sm_a8::YappingMaw_Instr_4;
        11051243 => sm_a8::YappingMaw_Instr_5; 11051261 => sm_a8::YappingMaw_Instr_7;
        11051279 => sm_a8::YappingMaw_Instr_3; 11051297 => sm_a8::YappingMaw_Instr_6;
        11051315 => sm_a8::YappingMaw_Instr_1; 11054610 => sm_a8::NorfairLavaMan_Instr_1;
        11054630 => sm_a8::NorfairLavaMan_Instr_8; 11054640 => sm_a8::NorfairLavaMan_Instr_14;
        11054650 => sm_a8::NorfairLavaMan_Instr_2; 11054661 => sm_a8::NorfairLavaMan_Instr_7;
        11054672 => sm_a8::NorfairLavaMan_Instr_10; 11054682 => sm_a8::NorfairLavaMan_Instr_12;
        11054692 => sm_a8::NorfairLavaMan_Instr_9; 11054728 => sm_a8::NorfairLavaMan_Instr_11;
        11054742 => sm_a8::NorfairLavaMan_Instr_13; 11054778 => sm_a8::NorfairLavaMan_Instr_5;
        11054794 => sm_a8::NorfairLavaMan_Instr_15; 11054820 => sm_a8::NorfairLavaMan_Instr_4;
        11054846 => sm_a8::NorfairLavaMan_Instr_16; 11054872 => sm_a8::NorfairLavaMan_Instr_6;
        11054916 => sm_a8::NorfairLavaMan_Instr_3; 11056990 => [sm_a8::Beetom_Instr_1(k, j)];
        11062537 => sm_a8::WreckedShipRobot_Instr_4; 11062692 => sm_a8::WreckedShipRobot_Instr_9;
        11062762 => sm_a8::WreckedShipRobot_Instr_6; 11062917 => sm_a8::WreckedShipRobot_Instr_8;
        11062987 => [sm_a8::WreckedShipRobot_Instr_7(k, j)];
        11062991 => sm_a8::WreckedShipRobot_Instr_15; 11063146 => sm_a8::WreckedShipRobot_Instr_18;
        11063216 => sm_a8::WreckedShipRobot_Instr_16; 11063371 => sm_a8::WreckedShipRobot_Instr_17;
        11063441 => sm_a8::WreckedShipRobot_Instr_3;
        11063490 => [sm_a8::WreckedShipRobot_Instr_10(k, j)];
        11063494 => sm_a8::WreckedShipRobot_Instr_14; 11063506 => sm_a8::WreckedShipRobot_Instr_2;
        11063552 => sm_a8::WreckedShipRobot_Instr_13; 11063559 => sm_a8::WreckedShipRobot_Instr_1;
        11063601 => sm_a8::WreckedShipRobot_Instr_12; 11063613 => sm_a8::WreckedShipRobot_Instr_5;
        11063659 => sm_a8::WreckedShipRobot_Instr_11;
        11067164 => sm_a8::WalkingLavaSeahorse_Instr_4;
        11067187 => sm_a8::WalkingLavaSeahorse_Instr_3;
        11067193 => sm_a8::WalkingLavaSeahorse_Instr_5;
        11067199 => sm_a8::WalkingLavaSeahorse_Instr_6;
        11067235 => sm_a8::WalkingLavaSeahorse_Instr_2;
        11067249 => sm_a8::WalkingLavaSeahorse_Instr_1; 11068957 => sm_a8::WreckedShipSpark_Instr_2;
        11068970 => sm_a8::WreckedShipSpark_Instr_1; 11072806 => sm_a8::KiHunter_Instr_1;
        11072996 => sm_a8::KiHunter_Instr_2; 11073151 => sm_a8::KiHunter_Instr_3;
        11073234 => sm_a8::KiHunter_Instr_4; 11073240 => sm_a8::KiHunter_Instr_5;
        11128756 => [sm_a9::Shitroid_Instr_1(k, j)];
        11128778 => [sm_a9::Shitroid_Instr_2(k, j)]; 11136208 => sm_a9::sub_A9ECD0;
        11139360 => [sm_a9::Shitroid_Instr_3(k, j)]; 11139382 => [sm_a9::Shitroid_Instr_4(k, j)];
        11139472 => [sm_a9::Shitroid_Instr_6(k, j)]; 11139476 => sm_a9::Shitroid_Instr_5;
        11173995 => sm_aa::Enemy_SetAiPreInstr_AA; 11174004 => sm_aa::Enemy_ClearAiPreInstr_AA;
        11174125 => [EnemyInstr_Goto(k, j)];
        11186332 => sm_aa::Torizo_Instr_3;
        11186461 => sm_aa::Torizo_Instr_31; 11186622 => sm_aa::Torizo_Instr_33;
        11186724 => sm_aa::Torizo_Instr_36; 11186734 => sm_aa::Torizo_Instr_37;
        11186744 => sm_aa::Torizo_Instr_35; 11186765 => sm_aa::Torizo_Instr_38;
        11186801 => sm_aa::Torizo_Instr_6; 11188557 => sm_aa::Torizo_Instr_5;
        11188561 => sm_aa::Torizo_Instr_9; 11190984 => [sm_aa::Torizo_Instr_7(k, j)];
        11190985 => sm_aa::Torizo_Instr_2; 11190993 => sm_aa::Torizo_Instr_8;
        11191001 => sm_aa::Torizo_Instr_25; 11191021 => sm_aa::Torizo_Instr_22;
        11191031 => sm_aa::Torizo_Instr_19; 11191037 => sm_aa::Torizo_Instr_32;
        11191043 => sm_aa::Torizo_Instr_30; 11191087 => sm_aa::Torizo_Instr_34;
        11191114 => sm_aa::Torizo_Instr_24; 11191131 => sm_aa::Torizo_Instr_12;
        11191149 => sm_aa::Torizo_Instr_10; 11191159 => sm_aa::Torizo_Instr_11;
        11191178 => sm_aa::Torizo_Instr_29; 11191200 => sm_aa::Torizo_Instr_1;
        11191222 => sm_aa::Torizo_Instr_28; 11191244 => sm_aa::Torizo_Instr_4;
        11191326 => sm_aa::Torizo_Instr_40; 11191408 => sm_aa::Torizo_Instr_16;
        11191525 => sm_aa::Torizo_Instr_27; 11191642 => sm_aa::Torizo_Instr_23;
        11191655 => sm_aa::Torizo_Instr_14; 11191691 => sm_aa::Torizo_Instr_15;
        11191716 => sm_aa::Torizo_Instr_26; 11191755 => sm_aa::Torizo_Instr_18;
        11191779 => sm_aa::Torizo_Instr_20; 11191794 => sm_aa::Torizo_Instr_44;
        11191809 => sm_aa::Torizo_Instr_21; 11191824 => sm_aa::Torizo_Instr_17;
        11191832 => sm_aa::Torizo_Instr_13; 11193038 => sm_aa::Torizo_Instr_39;
        11193054 => sm_aa::Torizo_Instr_41; 11193058 => sm_aa::Torizo_Instr_42;
        11193815 => sm_aa::Torizo_Instr_48; 11194601 => sm_aa::Torizo_Instr_57;
        11194611 => sm_aa::Torizo_Instr_58; 11194747 => sm_aa::Torizo_Instr_59;
        11194759 => sm_aa::Torizo_Instr_62; 11194855 => sm_aa::Torizo_Instr_63;
        11195279 => sm_aa::Torizo_Instr_56; 11195287 => sm_aa::Torizo_Instr_60;
        11195295 => sm_aa::Torizo_Instr_46; 11195360 => sm_aa::Torizo_Instr_47;
        11195370 => sm_aa::Torizo_Instr_49; 11195446 => sm_aa::Torizo_Instr_61;
        11195461 => sm_aa::Torizo_Instr_53; 11195508 => sm_aa::Torizo_Instr_55;
        11195547 => sm_aa::Torizo_Instr_52; 11195578 => sm_aa::Torizo_Instr_50;
        11195635 => sm_aa::Torizo_Instr_43; 11195645 => sm_aa::Torizo_Instr_51;
        11195686 => sm_aa::Torizo_Instr_45; 11195725 => sm_aa::Torizo_Instr_54;
        11196721 => sm_aa::Shaktool_Instr_2; 11196735 => sm_aa::Shaktool_Instr_3;
        11196746 => sm_aa::Shaktool_Instr_4; 11196755 => sm_aa::Shaktool_Instr_5;
        11196831 => [sm_aa::Shaktool_Instr_6(k, j)]; 11196858 => sm_aa::Shaktool_Instr_1;
        11199529 => sm_aa::Shaktool_Instr_9; 11199542 => sm_aa::Shaktool_Instr_11;
        11199549 => sm_aa::Shaktool_Instr_10; 11199871 => sm_aa::Shaktool_Instr_8;
        11199879 => sm_aa::Shaktool_Instr_13; 11199887 => sm_aa::Shaktool_Instr_12;
        11199960 => sm_aa::Shaktool_Instr_7; 11200240 => sm_aa::Shaktool_Instr_14;
        11698413 => [EnemyInstr_Goto(k, j)];
        11726400 => sm_b2::SpacePirates_Instr_MovePixelsDownAndChangeDirFaceRight;
        11726450 => sm_b2::SpacePirates_Instr_MovePixelsDownAndChangeDirFaceLeft;
        11726500 => sm_b2::SpacePirates_Instr_RandomNewDirFaceR;
        11726524 => sm_b2::SpacePirates_Instr_RandomNewDirFaceL;
        11726548 => sm_b2::SpacePirates_Instr_PrepareWallJumpR;
        11726589 => sm_b2::SpacePirates_Instr_PrepareWallJumpL;
        11726634 => sm_b2::SpacePirates_Instr_FireLaserL;
        11726685 => sm_b2::SpacePirates_Instr_FireLaserR;
        11726723 => sm_b2::SpacePirates_Instr_SetEnemyFunc;
        11726739 => sm_b2::SpacePirates_Instr_PlaySfx; 11728182 => sm_b2::SpacePirates_Instr_20;
        11728198 => sm_b2::SpacePirates_Instr_16; 11728228 => sm_b2::SpacePirates_Instr_15;
        11728272 => sm_b2::SpacePirates_Instr_18; 11728342 => sm_b2::SpacePirates_Instr_17;
        11729257 => sm_b2::SpacePirates_Instr_19; 11729469 => sm_b2::SpacePirates_Instr_21;
        11730024 => sm_b2::SpacePirates_Instr_12; 11730064 => sm_b2::SpacePirates_Instr_14;
        11730104 => sm_b2::SpacePirates_Instr_11; 11730120 => sm_b2::SpacePirates_Instr_13;
        11763819 => sm_b3::Enemy_SetAiPreInstr_B3; 11763828 => sm_b3::Enemy_ClearAiPreInstr_B3;
        11763949 => [EnemyInstr_Goto(k, j)];
        11769031 => sm_b3::Botwoon_Instr_1; 11769047 => sm_b3::Botwoon_Instr_2;
        11769063 => sm_b3::Botwoon_Instr_3; 11769079 => sm_b3::Botwoon_Instr_4;
        11769095 => sm_b3::Botwoon_Instr_5; 11769111 => sm_b3::Botwoon_Instr_6;
        11769127 => sm_b3::Botwoon_Instr_7; 11769143 => sm_b3::Botwoon_Instr_8;
        11769159 => sm_b3::Botwoon_Instr_9; 11769175 => sm_b3::Botwoon_Instr_10;
        11769191 => sm_b3::Botwoon_Instr_SetSpitting; 11769202 => sm_b3::Botwoon_Instr_QueueSpitSfx;
        11789637 => sm_b3::EscapeEtecoon_Instr_1; 11789840 => sm_b3::EscapeEtecoon_Instr_2;
        11791016 => sm_b3::EscapeDachora_Instr_2; 11791032 => sm_b3::EscapeDachora_Instr_3;
        11791049 => sm_b3::EscapeDachora_Instr_1; 11791063 => sm_b3::EscapeDachora_Instr_4;
        11113910 => sm_a9::MotherBrain_Instr_MoveBodyUp10Left4;
        11113920 => sm_a9::MotherBrain_Instr_MoveBodyUp16Left4;
        11113930 => sm_a9::MotherBrain_Instr_MoveBodyUp12Right2;
        11113950 => sm_a9::MotherBrain_Instr_MoveDown12Left4;
        11113960 => sm_a9::MotherBrain_Instr_MoveDown16Right2;
        11113970 => sm_a9::MotherBrain_Instr_MoveDown10Right2;
        11113980 => sm_a9::MotherBrain_Instr_MoveUp2Right1;
        11113996 => sm_a9::MotherBrain_Instr_MoveRight2;
        11114012 => sm_a9::MotherBrain_Instr_MoveUp1;
        11114018 => sm_a9::MotherBrain_Instr_MoveUp1Right3_Sfx;
        11114040 => sm_a9::MotherBrain_Instr_Down2Right15;
        11114056 => sm_a9::MotherBrain_Instr_Down4Right6;
        11114072 => sm_a9::MotherBrain_Instr_Up4Left2;
        11114088 => sm_a9::MotherBrain_Instr_Up2Left1_Sfx;
        11114110 => sm_a9::MotherBrain_Instr_Up2Left1_Sfx2;
        11114132 => sm_a9::MotherBrain_Instr_MoveLeft2;
        11114148 => sm_a9::MotherBrain_Instr_MoveDown1;
        11114154 => sm_a9::MotherBrain_Instr_MoveDown1Left3;
        11114170 => sm_a9::MotherBrain_Instr_MoveUp2Left15_Sfx;
        11114192 => sm_a9::MotherBrain_Instr_MoveUp4Left6;
        11114208 => sm_a9::MotherBrain_Instr_MoveDown4Right2;
        11114224 => sm_a9::MotherBrain_Instr_MoveDown2Right1;
        11114240 => sm_a9::MotherBrain_Instr_SetPose_Standing;
        11114248 => sm_a9::MotherBrain_Instr_SetPose_Walking;
        11114256 => sm_a9::MotherBrain_Instr_SetPose_Crouched;
        11114264 => sm_a9::MotherBrain_Instr_SetPose_CrouchedTrans;
        11114272 => sm_a9::MotherBrain_Instr_SetPose_DeathBeamMode;
        11114280 => sm_a9::MotherBrain_Instr_SetPose_LeaningDown;
        11115208 => sm_a9::MotherBrain_Instr_SpawnEprojToOffset;
        11115247 => sm_a9::MotherBrain_Instr_SpawnDeathBeamEproj;
        11115269 => sm_a9::MotherBrain_Instr_IncrBeamAttackPhase;
    }
}
pub unsafe fn EnemyMain(game: &mut Game) {
    if word!(game.first_free_enemy_index) != 0 {
        if word!(game.enemy_index_to_shake) != 0xffff {
            (*gEnemyData(word!(game.enemy_index_to_shake) as i32, game)).shake_timer = 64;
            word!(game.enemy_index_to_shake = -1i32 as u16);
        }
        let mut active_enemy_indexes_index: i32 = 0;
        loop {
            let v1: u16 = word!(game.active_enemy_indexes[(active_enemy_indexes_index >> 1)]);
            if v1 == 0xffff {
                break;
            }
            word!(game.cur_enemy_index = v1);
            let E = gEnemyData(word!(game.cur_enemy_index) as i32, game);
            'process_enemy: {
                let run_ai = if ((*E).properties & 0x400) == 0 {
                    if (*E).invincibility_timer != 0 {
                        (*E).invincibility_timer = (*E).invincibility_timer.wrapping_sub(1);
                        true
                    } else if word!(game.debug_disable_sprite_interact) == 0 {
                        if word!(game.debug_time_frozen_for_enemies)
                            | word!(game.time_is_frozen_flag)
                            == 0
                        {
                            EnemyCollisionHandler(game);
                            if (*E).enemy_ptr == 0 {
                                break 'process_enemy;
                            }
                        }
                        ((*E).extra_properties & 1) == 0
                    } else {
                        true
                    }
                } else {
                    true
                };
                if run_ai {
                    word!(game.UNUSED_word_7E17A2 = 0);
                    let ran_ai = if word!(game.debug_time_frozen_for_enemies)
                        | word!(game.time_is_frozen_flag)
                        == 0
                    {
                        let mut v6: i16 = 0;
                        let mut ai_handler_bits: u16 = (*E).ai_handler_bits;
                        if ai_handler_bits != 0 {
                            do_until! { (v8 != 0);
                                v6 += 1;
                                let mut v8: i8 = ((ai_handler_bits) as i8) & (1i8);
                                ai_handler_bits = (ai_handler_bits >> 1);
                            }
                        }
                        CallEnemyAi(
                            (words!(
                                i32,
                                (
                                    (*E).bank,
                                    (*get_EnemyDef_A2(
                                        ((*E).enemy_ptr as i32 + 2i32 * v6 as i32) as u16,
                                        game,
                                    ))
                                    .main_ai
                                )
                            )) as u32,
                            game,
                        );
                        true
                    } else {
                        let ED = get_EnemyDef_A2((*E).enemy_ptr, game);
                        if (*ED).time_is_frozen_ai != 0 {
                            CallEnemyAi(
                                (words!(i32, ((*E).bank, (*ED).time_is_frozen_ai))) as u32,
                                game,
                            );
                            true
                        } else {
                            false
                        }
                    };
                    if ran_ai
                        && word!(game.debug_time_frozen_for_enemies)
                            | word!(game.time_is_frozen_flag)
                            == 0
                    {
                        (*E).frame_counter = (*E).frame_counter.wrapping_add(1);
                        if ((*E).properties & 0x2000) != 0 {
                            word!(game.enemy_processing_stage = 2);
                            ProcessEnemyInstructions(game);
                        }
                    }
                }
                if ((*E).extra_properties & 1) != 0
                    && ((*E).flash_timer == 1 || (*E).frozen_timer == 1)
                {
                    (*gEnemySpawnData(word!(game.cur_enemy_index) as i32, game)).cause_of_death = 0;
                    EnemyDeathAnimation(word!(game.cur_enemy_index), 0, game);
                }
                if (((*E).extra_properties & 4) != 0
                    || EnemyWithNormalSpritesIsOffScreen(game) == 0)
                    && ((*E).properties & 0x300) == 0
                    && (word!(game.UNUSED_word_7E17A2) & 1) == 0
                {
                    DrawOneEnemy(game);
                }
            }
            if (*E).flash_timer != 0
                && word!(game.debug_time_frozen_for_enemies) | word!(game.time_is_frozen_flag) == 0
            {
                (*E).flash_timer = (*E).flash_timer.wrapping_sub(1);
                if (*E).flash_timer as i32 - 8i32 & 0x8000 != 0 {
                    (*E).ai_handler_bits = ((*E).ai_handler_bits & !(2));
                }
            }
            active_enemy_indexes_index += 2;
        }
    }
    HandleSpriteObjects(game);
    word!(game.random_enemy_counter, wrapping_add(1));
    word!(game.enemy_index_colliding_dirs[0] = -1i32 as u16);
    word!(game.enemy_index_colliding_dirs[1] = -1i32 as u16);
    word!(game.enemy_index_colliding_dirs[2] = -1i32 as u16);
    word!(game.enemy_index_colliding_dirs[3] = -1i32 as u16);
}
pub unsafe fn DecrementSamusTimers(game: &mut Game) {
    if word!(game.samus_invincibility_timer) != 0 {
        word!(game.samus_invincibility_timer, wrapping_sub(1));
    }
    if word!(game.samus_knockback_timer) != 0 {
        word!(game.samus_knockback_timer, wrapping_sub(1));
    }
    if word!(game.projectile_invincibility_timer) != 0 {
        word!(game.projectile_invincibility_timer, wrapping_sub(1));
    }
    word!(game.interactive_enemy_indexes[0] = -1i32 as u16);
    word!(game.active_enemy_indexes[0] = -1i32 as u16);
}
pub unsafe fn SpawnEnemyDrops(a: u16, k: u16, varE20: u16, game: &mut Game) {
    game.eproj_spawn_varE24 = a;
    SpawnEprojWithGfx(varE20, k, addr_kEproj_Pickup as u16, game);
}
pub unsafe fn DeleteEnemyAndConnectedEnemies(game: &mut Game) {
    let v0 = gEnemyData(word!(game.cur_enemy_index) as i32, game);
    let mut num_parts: u16 = (*get_EnemyDef_A2((*v0).enemy_ptr, game)).num_parts;
    if num_parts == 0 {
        num_parts = 1;
    }
    let mut v2: u16 = num_parts;
    do_until! { (v2 == 0);
        (*gEnemyData(word!(game.cur_enemy_index) as i32, game)).enemy_ptr = 0;
        word!(game.cur_enemy_index, wrapping_add(64));
        v2 = v2.wrapping_sub(1);
    }
}
pub unsafe fn SpawnEnemy(db: u8, k: u16, game: &mut Game) -> u16 {
    let initialize_enemy_slots: bool;
    let mut varE20: u16 = k;
    let cur_enemy_index_backup: u16 = word!(game.cur_enemy_index);
    let enemy_ptr: u16 = (*get_EnemyPopulation(db, k, game)).enemy_ptr;
    let mut v3: i16 = (((*get_EnemyDef_A2(enemy_ptr, game)).num_parts) as i16).wrapping_sub(1);
    if v3 < 0 {
        v3 = 0;
    }
    let mut varE22: u16 = v3 as u16;
    let mut varE26: u16 = v3 as u16;
    let mut new_enemy_index: u16 = 0;
    's_26: loop {
        let mut v4: u16 = new_enemy_index;
        while (*gEnemyData(v4 as i32, game)).enemy_ptr == 0 {
            if varE22 == 0 {
                initialize_enemy_slots = true;
                break 's_26;
            }
            varE22 = varE22.wrapping_sub(1);
            v4 = (v4).wrapping_add(64);
            if v4 >= 2048 {
                return 0xffff;
            }
        }
        new_enemy_index = (new_enemy_index).wrapping_add(64);
        if (new_enemy_index >= 2048) {
            initialize_enemy_slots = false;
            break;
        }
    }
    if initialize_enemy_slots {
        loop {
            let E = gEnemyData(new_enemy_index as i32, game);
            let EP = get_EnemyPopulation(db, varE20, game);
            let mut v10: i32 = 0;
            if (*EP).enemy_ptr == word!(game.enemy_def_ptr[0])
                || {
                    v10 = 1;
                    (*EP).enemy_ptr == word!(game.enemy_def_ptr[1])
                }
                || {
                    v10 = 2;
                    (*EP).enemy_ptr == word!(game.enemy_def_ptr[2])
                }
                || {
                    v10 = 3;
                    (*EP).enemy_ptr == word!(game.enemy_def_ptr[3])
                }
            {
                (*E).vram_tiles_index = word!(game.enemy_gfxdata_tiles_index[v10]);
                (*E).palette_index =
                    (2u16).wrapping_mul(u16::swap_bytes(word!(game.enemy_gfxdata_vram_ptr[v10])));
            } else {
                set_fields! {[(*E)], 0; vram_tiles_index, palette_index}
            }
            let ED = get_EnemyDef_A2((*EP).enemy_ptr, game);
            (*E).x_width = (*ED).x_radius;
            (*E).y_height = (*ED).y_radius;
            (*E).health = (*ED).health;
            (*E).layer = (*ED).layer as u16;
            *(&raw mut (*E).bank as *mut u16) = *(&raw mut (*ED).bank as *mut u16);
            set_fields! {[(*E)]; enemy_ptr = (*EP).enemy_ptr, x_pos = (*EP).x_pos,
            y_pos = (*EP).y_pos, current_instruction = (*EP).init_param,
            properties = (*EP).properties, extra_properties = (*EP).extra_properties,
            parameter_1 = (*EP).parameter1, parameter_2 = (*EP).parameter2, };
            set_fields! {[(*E)], 0; frame_counter, timer, ai_var_A, ai_var_B, ai_var_C, ai_var_D, ai_var_E, ai_preinstr}
            (*E).instruction_timer = 1;
            (*E).frame_counter = 0;
            RecordEnemySpawnData(new_enemy_index, game);
            word!(game.cur_enemy_index = new_enemy_index);
            if (*ED).ai_init & 0x8000 != 0 {
                CallEnemyAi((words!(i32, (*ED), bank, ai_init)) as u32, game);
            }
            if ((*E).properties & 0x2000) != 0 {
                (*E).spritemap_pointer = addr_kSpritemap_Nothing_A0 as u16;
            }
            if varE26 == 0 || {
                varE26 = varE26.wrapping_sub(1);
                varE26 == 0
            } {
                word!(game.cur_enemy_index = cur_enemy_index_backup);
                return new_enemy_index;
            }
            new_enemy_index = (new_enemy_index).wrapping_add(64);
            varE20 = (varE20).wrapping_add(16);
        }
    } else {
        0xffff
    }
}
pub unsafe fn DrawOneEnemy(game: &mut Game) {
    let varE34: u16 =
        (2u16).wrapping_mul((*gEnemyData(word!(game.cur_enemy_index) as i32, game)).layer);
    *(RomPtr_RAM(
        (word!(game.enemy_drawing_queue_sizes[(varE34 >> 1)])).wrapping_add(
            crate::rtl::rom_table_word!(game, 0xa0b133, (varE34 >> 1) as isize),
        ),
        game,
    ) as *mut u16) = word!(game.cur_enemy_index);
    word!(
        game.enemy_drawing_queue_sizes[(varE34 >> 1)],
        wrapping_add(2)
    );
}
pub unsafe fn WriteEnemyOams(game: &mut Game) {
    let mut palette_index: VoidP = 0;
    let E = gEnemyData(word!(game.cur_enemy_index) as i32, game);
    let ES = gEnemySpawnData(word!(game.cur_enemy_index) as i32, game);
    let mut x2: u16 =
        ((*ES).xpos2 as i32 + (*E).x_pos as i32 - word!(game.layer1_x_pos) as i32) as u16;
    let y2: u16 = ((*ES).ypos2 as i32 + (*E).y_pos as i32 - word!(game.layer1_y_pos) as i32) as u16;
    if (*E).shake_timer != 0 {
        x2 = (x2 as i32
            + if ((*E).frame_counter & 2) == 0 {
                1
            } else {
                -1i32
            }) as u16;
        (*E).shake_timer = (*E).shake_timer.wrapping_sub(1);
    }
    let mut x: u16 = x2;
    let mut y: u16 = y2;
    if (*E).flash_timer != 0 && (word!(game.random_enemy_counter) & 2) != 0 {
        palette_index = 0 as VoidP;
    } else if (*E).frozen_timer != 0 && ((*E).frozen_timer >= 0x5a || ((*E).frozen_timer & 2) != 0)
    {
        palette_index = 3072 as VoidP;
    } else {
        palette_index = (*E).palette_index;
    }
    let r3: u16 = palette_index;
    let r0: u16 = (*E).vram_tiles_index;
    if ((*E).extra_properties & 4) != 0 {
        if (((*E).spritemap_pointer) as i16).wrapping_add((0x8000i32) as i16) < 0 {
            Unreachable(game);
        }
        let mut n: i32 = *RomPtrWithBank((*E).bank, (*E).spritemap_pointer, game) as i32;
        let mut v5: u16 = ((*E).spritemap_pointer).wrapping_add(2);
        do_until! { (n == 0);
            let ext = get_ExtendedSpriteMap((*E).bank, v5, game);
            if *(RomPtrWithBank((*E).bank, (*ext).spritemap, game) as *mut u16) == 0xfffe {
                x = (x2).wrapping_add((*ext).xpos);
                y = (y2).wrapping_add((*ext).ypos);
                if ((*E).extra_properties & 0x8000) != 0 {
                    ProcessExtendedTilemap((*E).bank, (*ext).spritemap, game);
                }
            } else {
                x = (x2).wrapping_add((*ext).xpos);
                y = (y2).wrapping_add((*ext).ypos);
                if (x as i32 + 128 & 0xfe00) == 0 && (y as i32 + 128 & 0xfe00) == 0 {
                    if byte!(y; 1) != 0 {
                        DrawSpritemapWithBaseTileOffscreen(
                            (*E).bank,
                            (*ext).spritemap,
                            x,
                            y,
                            r3,
                            r0,
                            game,
                        );
                    } else {
                        DrawSpritemapWithBaseTile2((*E).bank, (*ext).spritemap, x, y, r3, r0, game);
                    }
                }
            }
            v5 = (v5).wrapping_add(8);
            n -= 1;
        }
    } else {
        word!(game.enemy_processing_stage = 1);
        DrawSpritemapWithBaseTile((*E).bank, (*E).spritemap_pointer, x, y, r3, r0, game);
    };
}
pub unsafe fn NormalEnemyFrozenAI(game: &mut Game) {
    let v0 = gEnemyData(word!(game.cur_enemy_index) as i32, game);
    (*v0).flash_timer = 0;
    if (*v0).frozen_timer == 0 || {
        (*v0).frozen_timer = (*v0).frozen_timer.wrapping_sub(1);
        (word!(game.equipped_beams) & 2) == 0
    } {
        let v1: u16 = ((*v0).ai_handler_bits) & (0xfffbu16);
        (*v0).ai_handler_bits = v1;
        (*v0).frozen_timer = v1;
    }
}
pub unsafe fn ProcessExtendedTilemap(db: u8, r22: u16, game: &mut Game) {
    let mut p = RomPtrWithBank(db, (r22).wrapping_add(2), game);
    loop {
        let v2: u16 = *(p as *mut u16);
        if v2 == 0xffff {
            break;
        }
        let n: i32 = *(p as *mut u16).offset(1) as i32;
        p = p.offset(4);
        memcpy(
            ram_ptr!(game, c_void; v2 as i32 as isize),
            p as *const c_void,
            (n * 2) as size_t,
        );
        p = p.offset((n * 2) as isize);
    }
    word!(game.nmi_flag_bg2_enemy_vram_transfer, wrapping_add(1));
}
pub fn QueueEnemyBG2TilemapTransfers(game: &mut Game) {
    if game.nmi_flag_bg2_enemy_vram_transfer() != 0
        && game.debug_time_frozen_for_enemies() | game.time_is_frozen_flag() == 0
    {
        let cursor = game.vram_write_queue_tail();
        let size = game.enemy_bg2_tilemap_size();
        game.write_vram_queue_word(cursor, 0, size);
        game.write_vram_queue_word(cursor, 2, 0x2000);
        game.write_vram_queue_byte(cursor, 4, 126);
        game.write_vram_queue_word(cursor, 5, addr_unk_604800 as u16);
        // Entry writes can overlap the tail; advance its live value.
        let tail = game.vram_write_queue_tail().wrapping_add(7);
        game.set_vram_write_queue_tail(tail);
    }
    game.set_nmi_flag_bg2_enemy_vram_transfer(0);
}
pub unsafe fn EnemyCollisionHandler(game: &mut Game) {
    if ((*gEnemyData(word!(game.cur_enemy_index) as i32, game)).extra_properties & 4) != 0 {
        EprojCollHandler_Multibox(game);
        EnemyBombCollHandler_Multibox(game);
        EnemySamusCollHandler_Multibox(game);
    } else {
        EprojCollHandler(game);
        EnemyBombCollHandler(game);
        EnemySamusCollHandler(game);
    };
}
pub unsafe fn SamusProjectileInteractionHandler(game: &mut Game) {
    word!(game.enemy_processing_stage = 10);
    if word!(game.flag_disable_projectile_interaction) != 0 {
        return;
    }
    let mut num_colls_to_check: i32 = 5;
    if word!(game.bomb_counter) != 0 {
        num_colls_to_check = 10;
    } else if word!(game.projectile_counter) == 0 {
        return;
    }
    if word!(game.projectile_invincibility_timer) != 0
        || word!(game.samus_contact_damage_index) != 0
    {
        return;
    }
    let mut pidx: i32 = 0;
    while pidx != num_colls_to_check {
        word!(game.collision_detection_index = pidx as u16);
        if !(word!(game.projectile_damage[pidx]) == 0
            || word!(game.projectile_type[pidx]) & 0x8000 != 0
            || (word!(game.projectile_type[pidx]) as i32 & 0xf00) - 1792i32 & 0x8000 == 0)
            && ((word!(game.projectile_dir[pidx]) & 0x10) == 0)
            && (abs16((word!(game.projectile_x_pos[pidx])).wrapping_sub(word!(game.samus_x_pos)))
                as i32
                - word!(game.projectile_x_radius[pidx]) as i32)
                < word!(game.samus_x_radius) as i32
            && (abs16((word!(game.projectile_y_pos[pidx])).wrapping_sub(word!(game.samus_y_pos)))
                as i32
                - word!(game.projectile_y_radius[pidx]) as i32)
                < word!(game.samus_y_radius) as i32
        {
            if (word!(game.projectile_type[pidx]) & 0xff00) != 768
                && (word!(game.projectile_type[pidx]) & 0xff00) != 1280
            {
                word!(game.projectile_dir[pidx], |fresh43| (fresh43) | (0x10u16));
                Samus_DealDamage(
                    SuitDamageDivision(word!(game.projectile_damage[pidx]), game),
                    game,
                );
                word!(game.samus_invincibility_timer = 96);
                word!(game.samus_knockback_timer = 5);
                crate::native_assertion! {
                    '_c2rust_label;
                    b"0\0", b"src/sm_a0.c\0", 2048,
                    b"void SamusProjectileInteractionHandler(void)\0"
                };
                let mut v0: u16 = 0;
                word!(
                    game.knockback_x_dir = (((word!(game.samus_x_pos)) as i16).wrapping_sub(
                        (*(ram_ptr!(game, u16; 0x1a4b)).offset((v0 >> 1) as isize,)) as i16,
                    ) >= 0) as u16
                );
                return;
            }
            if word!(game.projectile_variables[pidx]) == 8 {
                word!(
                    game.bomb_jump_dir =
                        (if word!(game.samus_x_pos) == word!(game.projectile_x_pos[pidx]) {
                            2
                        } else if ((word!(game.samus_x_pos)) as i16)
                            .wrapping_sub((word!(game.projectile_x_pos[pidx])) as i16,)
                            < 0
                        {
                            1
                        } else {
                            3
                        }) as u16
                );
            }
        }
        pidx += 1;
    }
}
pub unsafe fn EprojSamusCollDetect(game: &mut Game) {
    word!(game.enemy_processing_stage = 11);
    if word!(game.samus_invincibility_timer) != 0 || word!(game.samus_contact_damage_index) != 0 {
        return;
    }
    for i in (0i32..=17).rev() {
        if word!(game.eproj_id[i]) != 0
            && (word!(game.eproj_properties[i]) & 0x2000) == 0
            && word!(game.eproj_radius[i]) != 0
        {
            let varE20: u16 =
                byte!(@pointer (ram_ptr!(game, u16; 0x1bb3)).offset(i as isize); 0) as u16;
            let varE22: u16 =
                byte!(@pointer (ram_ptr!(game, u16; 0x1bb3)).offset(i as isize); 1) as u16;
            if (abs16((word!(game.samus_x_pos)).wrapping_sub(word!(game.eproj_x_pos[i]))) as i32
                - word!(game.samus_x_radius) as i32)
                < varE20 as i32
                && (abs16((word!(game.samus_y_pos)).wrapping_sub(word!(game.eproj_y_pos[i])))
                    as i32
                    - word!(game.samus_y_radius) as i32)
                    < varE22 as i32
            {
                word!(game.collision_detection_index = (i * 2) as u16);
                HandleEprojCollWithSamus((i * 2) as u16, game);
            }
        }
    }
}
pub unsafe fn HandleEprojCollWithSamus(k: u16, game: &mut Game) {
    word!(game.samus_invincibility_timer = 96);
    word!(game.samus_knockback_timer = 5);
    let v1: u16 = *(RomPtr_86(
        *((ram_ptr!(game, u16; 0x1997) as *mut u8).offset(k as i32 as isize) as *mut u16),
        game,
    ) as *mut u16)
        .offset(5);
    if v1 != 0 {
        let v2: i32 = k as i32 >> 1;
        word!(game.eproj_instr_list_ptr[v2] = v1);
        word!(game.eproj_instr_timers[v2] = 1);
    }
    let v3: i32 = k as i32 >> 1;
    if (word!(game.eproj_properties[v3]) & 0x4000) == 0 {
        *((ram_ptr!(game, u16; 0x1997) as *mut u8).offset(k as i32 as isize) as *mut u16) = 0;
    }
    Samus_DealDamage(
        SuitDamageDivision((word!(game.eproj_properties[v3])) & (0xfffu16), game),
        game,
    );
    word!(
        game.knockback_x_dir = (((word!(game.samus_x_pos)) as i16)
            .wrapping_sub((word!(game.eproj_x_pos[v3])) as i16,)
            >= 0) as u16
    );
}
pub unsafe fn EprojProjCollDet(game: &mut Game) {
    word!(game.enemy_processing_stage = 12);
    if word!(game.projectile_counter) == 0 {
        return;
    }
    for i in (0i32..=17).rev() {
        if !(word!(game.eproj_id[i]) == 0 || (word!(game.eproj_properties[i]) & 0x8000) == 0) {
            for j in 0i32..5 {
                if word!(game.eproj_flags[i]) == 2 {
                    break;
                }
                let v4: u16 = word!(game.projectile_type[j]);
                if v4 != 0
                    && (v4 & 0xf00) != 768
                    && (v4 & 0xf00) != 1280
                    && (v4 as i32 & 0xf00) - 1792i32 & 0x8000 != 0
                    && word!(game.eproj_x_pos[i]) & 0xffe0
                        == word!(game.projectile_x_pos[j]) & 0xffe0
                    && word!(game.eproj_y_pos[i]) & 0xffe0
                        == word!(game.projectile_y_pos[j]) & 0xffe0
                {
                    HandleEprojCollWithProj((i * 2) as u16, (j * 2) as u16, game);
                }
            }
        }
    }
}
pub unsafe fn HandleEprojCollWithProj(k: u16, j: u16, game: &mut Game) {
    let i: i32 = j as i32 >> 1;
    if (word!(game.projectile_type[i]) & 8) == 0 {
        word!(game.projectile_dir[i], |fresh30| (fresh30) | (0x10u16));
    }
    if word!(game.eproj_flags[(k >> 1)]) == 1 {
        let v4: i32 = k as i32 >> 1;
        CreateSpriteAtPos(
            word!(game.projectile_x_pos[v4]),
            word!(game.projectile_y_pos[v4]),
            6,
            0,
            game,
        );
        QueueSfx1_Max6(0x3d, game);
    } else {
        let j_0: i32 = k as i32 >> 1;
        word!(game.eproj_G[j_0] = word!(game.projectile_type[i]));
        word!(
            game.eproj_instr_list_ptr[j_0] =
                (*get_EprojDef(word!(game.eproj_id[j_0]), game,)).shot_instruction_list
        );
        word!(game.eproj_instr_timers[j_0] = 1);
        word!(game.eproj_pre_instr[j_0] = ((0x8684fbi32) as u16));
        word!(game.eproj_properties[j_0], |fresh31| (fresh31) & (0xfffu16));
    };
}
crate::game::address_dispatch! {
    pub unsafe fn CallHitboxTouch(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        10518563 => Enemy_NormalTouchAI_A0; 10670984 => sm_a2::MaridiaLargeSnail_Func_12;
        10670988 => sm_a2::MaridiaLargeSnail_Touch; 10780707 => sm_a4::Enemy_NormalTouchAI_A4;
        10795325 => sm_a4::Crocomire_Func_92; 10795344 => [()]; 10846284 => [()];
        10851818 => sm_a5::Draygon_Touch; 10874348 => sm_a5::SporeSpawn_Touch;
        10936153 => sm_a6::sub_A6DF59; 10940479 => sm_a6::CeresSteam_Touch; 10977356 => [()];
        10982539 => sm_a7::Kraid_Touch_ArmFoot; 10982544 => sm_a7::KraidsArm_Touch;
        11001237 => sm_a7::Phantoon_Touch; 11122117 => [()];
        11122118 => sm_a9::MotherBrainsBrain_Touch; 11192695 => sm_aa::GoldTorizo_Touch;
        11700076 => sm_b2::WalkingSpacePirates_Touch;
    }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallHitboxShot(ea: u32, j: u16, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @forward [pub unsafe fn (game: &mut Game)];
    Enemy_GrappleReact_NoInteract_A0 => SwitchEnemyAiToMainAi(game);
    Enemy_GrappleReact_SamusLatchesOn_A0 => SamusLatchesOnWithGrapple(game);
    Enemy_GrappleReact_KillEnemy_A0 => EnemyGrappleDeath(game);
    Enemy_GrappleReact_CancelBeam_A0 => Enemy_SwitchToFrozenAi(game);
    Enemy_GrappleReact_SamusLatchesNoInvinc_A0 => SamusLatchesOnWithGrappleNoInvinc(game);
    Enemy_GrappleReact_SamusLatchesParalyze_A0 => SamusLatchesOnWithGrappleParalyze(game);
    Enemy_GrappleReact_HurtSamus_A0 => SamusHurtFromGrapple(game);
    Enemy_NormalTouchAI_A0 => NormalEnemyTouchAi(game);
    Enemy_NormalTouchAI_SkipDeathAnim_A0 => NormalEnemyTouchAiSkipDeathAnim_CurEnemy(game);
    [10518573] Enemy_NormalShotAI_A0 => NormalEnemyShotAi(game);
    Enemy_NormalShotAI_SkipSomeParts_A0 => NormalEnemyShotAiSkipDeathAnim_CurEnemy(game);
    Enemy_NormalPowerBombAI_A0 => NormalEnemyPowerBombAi(game);
    Enemy_NormalPowerBombAI_SkipDeathAnim_A0 => NormalEnemyPowerBombAiSkipDeathAnim_CurEnemy(game);
    Enemy_NormalFrozenAI => NormalEnemyFrozenAI(game);
    [10846278] CreateADudShot => CreateDudShot(game);
    @external {
        10649676 => [()];
        10671028 => sm_a2::MaridiaLargeSnail_Shot; 10780717 => sm_a4::Enemy_NormalShotAI_A4;
        10795345 => sm_a4::Crocomire_Func_93; 10795368 => sm_a4::Crocomire_Func_94;
        10795525 => sm_a4::Crocomire_Func_95; 10795700 => sm_a4::Crocomire_Func_1;
        10846253 => sm_a5::Enemy_NormalShotAI_A5; 10846284 => [()];
        10851824 => sm_a5::Draygon_Shot; 10874202 => sm_a5::SporeSpawn_Shot; 10911820 => [()];
        10936202 => sm_a6::Ridley_Shot; 10977356 => [()]; 10982581 => [()];
        10982582 => [sm_a7::Kraid_Arm_Shot(j, game)]; 11001243 => sm_a7::Phantoon_Shot;
        11121923 => sm_a9::MotherBrainsBody_Shot; 11121927 => sm_a9::MotherBrainsBrain_Shot;
        11192700 => sm_aa::Torizo_Shot; 11192769 => [()]; 11192770 => sm_aa::Torizo_Func_8;
        11700089 => sm_b2::WalkingSpacePirates_Shot; 11700168 => sm_b2::WalkingSpacePirates_87C8;
        11700286 => sm_b2::WalkingSpacePirates_883E;
    }
}
pub unsafe fn GrappleBeam_CollDetect_Enemy(game: &mut Game) -> PairU16 {
    let mut E = null_mut::<EnemyData>();
    CallSomeSamusCode(0xd, game);
    word!(game.collision_detection_index = 0);
    let mut i: i32 = 0;
    loop {
        word!(game.cur_enemy_index = word!(game.interactive_enemy_indexes[i]));
        if word!(game.cur_enemy_index) == 0xffff {
            return PairU16 { k: 0, j: 0 };
        }
        E = gEnemyData(word!(game.cur_enemy_index) as i32, game);
        if (*E).invincibility_timer == 0 {
            let v5: u16 = abs16(((*E).x_pos).wrapping_sub(word!(game.grapple_beam_end_x_pos)));
            let mut v6: bool = v5 < (*E).x_width;
            let v7: u16 = (v5).wrapping_sub((*E).x_width);
            if v6 || v7 < 8 {
                let v8: u16 = abs16(((*E).y_pos).wrapping_sub(word!(game.grapple_beam_end_y_pos)));
                v6 = v8 < (*E).y_height;
                let v9: u16 = (v8).wrapping_sub((*E).y_height);
                if v6 || v9 < 8 {
                    break;
                }
            }
        }
        i += 1;
    }
    (*E).ai_handler_bits = 1;
    let mut v0: u16 = 0;
    let enemy_ptr: u16 = (*E).enemy_ptr;
    let mut grapple_ai: VoidP = (*get_EnemyDef_A2((*E).enemy_ptr, game)).grapple_ai;
    if grapple_ai as i32 != 0xa08000 & 0xffff {
        v0 = 1;
        if grapple_ai as i32 != 0xa08005 & 0xffff {
            v0 = 2;
            if grapple_ai as i32 != 0xa0800a & 0xffff {
                v0 = 3;
                if grapple_ai as i32 != 0xa0800f & 0xffff {
                    v0 = 4;
                    if grapple_ai as i32 != 0xa08014 & 0xffff {
                        v0 = 5;
                        if grapple_ai as i32 != 0xa08019 & 0xffff {
                            v0 = 6;
                            if grapple_ai as i32 != 0xa0801e & 0xffff {
                                v0 = 0;
                            }
                        }
                    }
                }
            }
        }
    }
    if v0 == 1 || v0 == 4 || v0 == 5 {
        let v11 = gEnemyData(word!(game.cur_enemy_index) as i32, game);
        word!(game.grapple_beam_end_x_pos = (*v11).x_pos);
        word!(game.grapple_beam_end_y_pos = (*v11).y_pos);
    }
    PairU16 {
        k: v0,
        j: enemy_ptr,
    }
}
pub unsafe fn EnemyGrappleDeath(game: &mut Game) {
    (*gEnemySpawnData(word!(game.cur_enemy_index) as i32, game)).cause_of_death = 4;
    EnemyDeathAnimation(word!(game.cur_enemy_index), 0, game);
    (*gEnemyData(word!(game.cur_enemy_index) as i32, game)).ai_handler_bits = 0;
}
pub unsafe fn Enemy_SwitchToFrozenAi(game: &mut Game) {
    (*gEnemyData(word!(game.cur_enemy_index) as i32, game)).ai_handler_bits = 4;
}
pub use self::Enemy_SwitchToFrozenAi as SamusHurtFromGrapple;
pub unsafe fn ProcessEnemyPowerBombInteraction(game: &mut Game) {
    word!(game.enemy_processing_stage = 5);
    let rx: u16 = byte!(@pointer ram_ptr!(game, u16; 0xcea); 1) as u16;
    let ry: u16 = (rx as i32 + (rx as i32 >> 1) >> 1) as u16;
    let mut i: i32 = 0x7c0;
    while rx != 0 && i >= 0 {
        let E = gEnemyData(i, game);
        if !((*E).invincibility_timer != 0
            || (*E).enemy_ptr == 0
            || (*E).enemy_ptr as i32 == addr_kEnemyDef_DAFF as i32)
        {
            let ED = get_EnemyDef_A2((*E).enemy_ptr, game);
            if (((*get_Vulnerability(
                (if (*ED).vulnerability_ptr != 0u16 {
                    (*ED).vulnerability_ptr as i32
                } else {
                    addr_stru_B4EC1C as i32
                }) as u16,
                game,
            ))
            .power_bomb
                & 0x7f)
                != 0)
                && abs16((word!(game.power_bomb_explosion_x_pos)).wrapping_sub((*E).x_pos)) < rx
                && abs16((word!(game.power_bomb_explosion_y_pos)).wrapping_sub((*E).y_pos)) < ry
            {
                word!(game.cur_enemy_index = i as u16);
                let func: u16 = (if (*ED).powerbomb_reaction != 0 {
                    (*ED).powerbomb_reaction as i32
                } else {
                    0xa08037 & 0xffff
                }) as u16;
                CallEnemyAi((words!(i32, ((*E).bank, func))) as u32, game);
                (*E).properties = ((*E).properties) | (0x800u16);
            }
        }
        i -= 0x40;
    }
}

macro_rules! spawn_enemy_death_and_clear_slot {
    ($explosion_kind:ident, $kind_limit:literal, $game:ident, $enemy:ident) => {
        if $explosion_kind as i32 - $kind_limit & 0x8000 == 0 {
            $explosion_kind = 0;
        }
        SpawnEprojWithGfx(
            $explosion_kind,
            word!($game.cur_enemy_index),
            addr_kEproj_EnemyDeathExplosion as u16,
            $game,
        );
        let r18: u16 = ((*$enemy).properties) & (0x4000u16);
        memset($enemy as *mut c_void, 0, 64 as size_t);
        if r18 != 0 {
            (*$enemy).enemy_ptr = addr_kEnemyDef_DAFF as u16;
            (*$enemy).bank = 0xa3;
        }
    };
}

pub unsafe fn EnemyDeathAnimation(k: u16, mut a: u16, game: &mut Game) {
    let E = gEnemyData(word!(game.cur_enemy_index) as i32, game);
    if (*E).ai_handler_bits == 1 {
        word!(game.grapple_beam_function = ((0x9bc8c5i32) as u16));
    }
    spawn_enemy_death_and_clear_slot!(a, 5i32, game, E);
    word!(game.num_enemies_killed_in_room, wrapping_add(1));
}
pub unsafe fn RinkasDeathAnimation(mut a: u16, game: &mut Game) {
    let E = gEnemyData(word!(game.cur_enemy_index) as i32, game);
    spawn_enemy_death_and_clear_slot!(a, 3i32, game, E);
}
pub unsafe fn SuitDamageDivision(a: u16, game: &mut Game) -> u16 {
    if (word!(game.equipped_items) & 0x20) != 0 {
        return (a >> 2);
    }
    if word!(game.equipped_items) & 1 != 0 {
        return (a >> 1);
    }
    a
}
pub unsafe fn NormalEnemyTouchAi(game: &mut Game) {
    NormalEnemyTouchAiSkipDeathAnim(game);
    if (*gEnemyData(word!(game.cur_enemy_index) as i32, game)).health == 0 {
        (*gEnemySpawnData(word!(game.cur_enemy_index) as i32, game)).cause_of_death = 6;
        EnemyDeathAnimation(word!(game.cur_enemy_index), 1, game);
    }
}
pub unsafe fn NormalEnemyTouchAiSkipDeathAnim_CurEnemy(game: &mut Game) {
    NormalEnemyTouchAiSkipDeathAnim(game);
}
crate::prefixed_callbacks! {
    pub unsafe fn (game: &mut Game);
    @prefix { let E = gEnemyData(word!(game.cur_enemy_index) as i32, game); };
    EnemySamusCollHandler_Multibox [] {
        word!(game.enemy_processing_stage = 6);
        if (*E).spritemap_pointer == 0 {
            return;
        }
        let touch_ai: u16 = (*get_EnemyDef_A2((*E).enemy_ptr, game)).touch_ai;
        if touch_ai as i32 == 0xa0804c & 0xffff || touch_ai as i32 == 0xa0804b & 0xffff {
            return;
        }
        if word!(game.samus_contact_damage_index) != 0 {
            word!(game.samus_invincibility_timer = 0);
        } else if word!(game.samus_invincibility_timer) != 0 {
            return;
        }
        if (*E).spritemap_pointer & 0x8000 == 0 {
            return;
        }
        let samus_right_border_coll: u16 =
            (word!(game.samus_x_radius)).wrapping_add(word!(game.samus_x_pos));
        let samus_left_border_coll: u16 =
            (word!(game.samus_x_pos)).wrapping_sub(word!(game.samus_x_radius));
        let samus_bottom_border_coll: u16 =
            (word!(game.samus_y_radius)).wrapping_add(word!(game.samus_y_pos));
        let samus_top_border_coll: u16 =
            (word!(game.samus_y_pos)).wrapping_sub(word!(game.samus_y_radius));
        let mut n: i32 = *RomPtrWithBank((*E).bank, (*E).spritemap_pointer, game) as i32;
        let mut enemy_spritemap_entry_pointer: u16 = ((*E).spritemap_pointer).wrapping_add(2);
        do_until! { (n == 0);
    let ES = get_ExtendedSpriteMap((*E).bank, enemy_spritemap_entry_pointer, game);
    let coll_x_pos: u16 = ((*ES).xpos).wrapping_add((*E).x_pos);
    let coll_y_pos: u16 = ((*ES).ypos).wrapping_add((*E).y_pos);
    let p = RomPtrWithBank((*E).bank, (*ES).hitbox_ptr_, game);
    let mut m: i32 = *(p as *mut u16) as i32;
    let mut hb = p.offset(2) as *mut Hitbox;
    while m != 0 {
        if (((*hb).left as i32 + coll_x_pos as i32 - samus_right_border_coll as i32) as i16) < 0
            && (((*hb).right as i32 + coll_x_pos as i32 - samus_left_border_coll as i32) as i16)
                >= 0
            && (((*hb).top as i32 + coll_y_pos as i32 - samus_bottom_border_coll as i32) as i16)
                < 0
            && (((*hb).bottom as i32 + coll_y_pos as i32 - samus_top_border_coll as i32) as i16)
                >= 0
        {
            CallHitboxTouch((words!(i32, ((*E).bank, (*hb).func_ptr))) as u32, game);
            return;
        }
        m -= 1;
        hb = hb.offset(1);
    }
    enemy_spritemap_entry_pointer = (enemy_spritemap_entry_pointer).wrapping_add(8);
    n -= 1;
}
    };
    EprojCollHandler_Multibox [] {
        word!(game.enemy_processing_stage = 3);
        if word!(game.projectile_counter) == 0
            || (*E).spritemap_pointer == 0
            || (*E).spritemap_pointer as i32 == addr_kExtendedSpritemap_Nothing_A0 as i32
        {
            return;
        }
        let shot_ai: u16 = (*get_EnemyDef_A2((*E).enemy_ptr, game)).shot_ai;
        if shot_ai as i32 == 0xa0804c & 0xffff || shot_ai as i32 == 0xa0804b & 0xffff {
            return;
        }
        if ((*E).properties & 0x400) != 0
            || (*E).invincibility_timer != 0
            || (*E).enemy_ptr as i32 == addr_kEnemyDef_DAFF as i32
        {
            return;
        }
        for pidx in 0i32..5 {
            let v4: u16 = word!(game.projectile_type[pidx]);
            if v4 != 0
                && (v4 & 0xf00) != 768
                && (v4 & 0xf00) != 1280
                && (v4 as i32 & 0xf00) - 1792i32 & 0x8000 != 0
            {
                if (*E).spritemap_pointer & 0x8000 == 0 {
                    Unreachable(game);
                }
                let esep = RomPtrWithBank((*E).bank, (*E).spritemap_pointer, game);
                let mut n: i32 = *esep as i32;
                let mut ES = esep.offset(2) as *mut ExtendedSpriteMap;
                while n != 0 {
                    let coll_x_pos: u16 = ((*ES).xpos).wrapping_add((*E).x_pos);
                    let coll_y_pos: u16 = ((*ES).ypos).wrapping_add((*E).y_pos);
                    let p = RomPtrWithBank((*E).bank, (*ES).hitbox_ptr_, game);
                    let mut m: i32 = *(p as *mut u16) as i32;
                    let mut hb = p.offset(2) as *mut Hitbox;
                    while m != 0 {
                        if ((word!(game.projectile_x_radius[pidx]) as i32
                            + word!(game.projectile_x_pos[pidx]) as i32
                            - (coll_x_pos as i32 + (*hb).left as i32)) as i16)
                            >= 0
                            && ((word!(game.projectile_x_pos[pidx]) as i32
                                - word!(game.projectile_x_radius[pidx]) as i32
                                - (coll_x_pos as i32 + (*hb).right as i32))
                                as i16)
                                < 0
                            && ((word!(game.projectile_y_radius[pidx]) as i32
                                + word!(game.projectile_y_pos[pidx]) as i32
                                - (coll_y_pos as i32 + (*hb).top as i32))
                                as i16)
                                >= 0
                            && ((word!(game.projectile_y_pos[pidx]) as i32
                                - word!(game.projectile_y_radius[pidx]) as i32
                                - (coll_y_pos as i32 + (*hb).bottom as i32))
                                as i16)
                                < 0
                        {
                            if (word!(game.projectile_type[pidx]) & 0xf00) == 512 {
                                word!(game.earthquake_timer = 30);
                                word!(game.earthquake_type = 18);
                            }
                            if ((*E).properties & 0x1000) != 0
                                || (word!(game.projectile_type[pidx]) & 8) == 0
                            {
                                word!(game.projectile_dir[pidx], |fresh27| (fresh27) | (0x10u16));
                            }
                            word!(game.collision_detection_index = pidx as u16);
                            CallHitboxShot(
                                (words!(i32, ((*E).bank, (*hb).func_ptrA))) as u32,
                                (pidx * 2) as u16,
                                game,
                            );
                            return;
                        }
                        m -= 1;
                        hb = hb.offset(1);
                    }
                    n -= 1;
                    ES = ES.offset(1);
                }
            }
        }
    };
    EnemyBombCollHandler_Multibox [] {
        word!(game.enemy_processing_stage = 4);
        if (*E).spritemap_pointer == 0
            || ((*E).properties & 0x400) != 0
            || (*E).invincibility_timer != 0
        {
            return;
        }
        let shot_ai: u16 = (*get_EnemyDef_A2((*E).enemy_ptr, game)).shot_ai;
        if shot_ai as i32 == 0xa0804c & 0xffff
            || shot_ai as i32 == 0xa0804b & 0xffff
            || word!(game.bomb_counter) == 0
        {
            return;
        }
        for pidx in 5i32..10 {
            if (word!(game.projectile_x_pos[pidx]) != 0) {
                let v4: u16 = word!(game.projectile_type[pidx]);
                if v4 != 0 && (v4 & 0xf00) == 1280 && word!(game.projectile_variables[pidx]) == 0 {
                    if (*E).spritemap_pointer & 0x8000 == 0 {
                        Unreachable(game);
                    }
                    let esep = RomPtrWithBank((*E).bank, (*E).spritemap_pointer, game);
                    let mut n: i32 = *(esep as *mut u16) as i32;
                    let mut ES = esep.offset(2) as *mut ExtendedSpriteMap;
                    while n != 0 {
                        let coll_x_pos: u16 = ((*ES).xpos).wrapping_add((*E).x_pos);
                        let coll_y_pos: u16 = ((*ES).ypos).wrapping_add((*E).y_pos);
                        let p = RomPtrWithBank((*E).bank, (*ES).hitbox_ptr_, game);
                        let mut m: i32 = *(p as *mut u16) as i32;
                        let mut hb = p.offset(2) as *mut Hitbox;
                        while m != 0 {
                            if ((word!(game.projectile_x_radius[pidx]) as i32
                                + word!(game.projectile_x_pos[pidx]) as i32
                                - (coll_x_pos as i32 + (*hb).left as i32))
                                as i16)
                                >= 0
                                && ((word!(game.projectile_x_pos[pidx]) as i32
                                    - word!(game.projectile_x_radius[pidx]) as i32
                                    - (coll_x_pos as i32 + (*hb).right as i32))
                                    as i16)
                                    < 0
                                && ((word!(game.projectile_y_radius[pidx]) as i32
                                    + word!(game.projectile_y_pos[pidx]) as i32
                                    - (coll_y_pos as i32 + (*hb).top as i32))
                                    as i16)
                                    >= 0
                                && ((word!(game.projectile_y_pos[pidx]) as i32
                                    - word!(game.projectile_y_radius[pidx]) as i32
                                    - (coll_y_pos as i32 + (*hb).bottom as i32))
                                    as i16)
                                    < 0
                            {
                                word!(game.projectile_dir[pidx], |fresh24| (fresh24) | (0x10u16));
                                word!(game.collision_detection_index = pidx as u16);
                                CallHitboxShot(
                                    (words!(i32, ((*E).bank, (*hb).func_ptrA))) as u32,
                                    (pidx * 2) as u16,
                                    game,
                                );
                                return;
                            }
                            m -= 1;
                            hb = hb.offset(1);
                        }
                        n -= 1;
                        ES = ES.offset(1);
                    }
                }
            }
        }
    };
    SwitchEnemyAiToMainAi [] {
        set_fields! {[(*E)], 0; ai_handler_bits, invincibility_timer, frozen_timer, shake_timer}
    };
    SamusLatchesOnWithGrapple [] {
        word!(game.grapple_beam_end_x_pos = (*E).x_pos);
        word!(game.grapple_beam_end_y_pos = (*E).y_pos);
        if (*E).frozen_timer != 0 {
            (*E).ai_handler_bits = 4;
        } else {
            let ED = get_EnemyDef_A2((*E).enemy_ptr, game);
            (*E).flash_timer = (if (*ED).hurt_ai_time != 0 {
                (*ED).hurt_ai_time as i32
            } else {
                4
            }) as u16;
            (*E).ai_handler_bits = 0;
        };
    };
    SamusLatchesOnWithGrappleNoInvinc [] {
        if (*E).frozen_timer != 0 {
            (*E).ai_handler_bits = 4;
        } else {
            CallEnemyAi(
                (words!(
                    i32,
                    ((*E).bank, (*get_EnemyDef_A2((*E).enemy_ptr, game)).main_ai)
                )) as u32,
                game,
            );
            (*E).ai_handler_bits = 0;
        }
        word!(game.grapple_beam_end_x_pos = (*E).x_pos);
        word!(game.grapple_beam_end_y_pos = (*E).y_pos);
    };
    SamusLatchesOnWithGrappleParalyze [] {
        let ED = get_EnemyDef_A2((*E).enemy_ptr, game);
        (*E).flash_timer = (if (*ED).hurt_ai_time != 0 {
            (*ED).hurt_ai_time as i32
        } else {
            4
        }) as u16;
        (*E).ai_handler_bits = 0;
        (*E).extra_properties = ((*E).extra_properties) | (1u16);
    };
    EnemySamusCollHandler [] {
        word!(game.enemy_processing_stage = 9);
        if (*E).spritemap_pointer == 0 {
            return;
        }
        if word!(game.samus_contact_damage_index) != 0 {
            word!(game.samus_invincibility_timer = 0);
        } else if word!(game.samus_invincibility_timer) != 0 {
            if (*E).enemy_ptr as i32 != addr_kEnemyDef_DAFF as i32 {
                return;
            }
            let some_flag: u16 = (*gEnemySpawnData(word!(game.cur_enemy_index) as i32, game)).some_flag;
            if some_flag == 0 || some_flag == 8 {
                return;
            }
        }
        let ED = get_EnemyDef_A2((*E).enemy_ptr, game);
        if (*ED).touch_ai as i32 == 0xa0804c & 0xffff || (*ED).touch_ai as i32 == 0xa0804b & 0xffff {
            return;
        }
        if (abs16((word!(game.samus_x_pos)).wrapping_sub((*E).x_pos)) as i32
            - word!(game.samus_x_radius) as i32)
            < (*E).x_width as i32
            && (abs16((word!(game.samus_y_pos)).wrapping_sub((*E).y_pos)) as i32
                - word!(game.samus_y_radius) as i32)
                < (*E).y_height as i32
            && ((*E).enemy_ptr as i32 == addr_kEnemyDef_DAFF as i32 || (*E).frozen_timer == 0)
        {
            CallEnemyAi((words!(i32, ((*E).bank, (*ED).touch_ai))) as u32, game);
        }
    };
    EprojCollHandler [] {
        word!(game.enemy_processing_stage = 7);
        if word!(game.projectile_counter) == 0 {
            return;
        }
        if (*E).spritemap_pointer == 0
            || (*E).spritemap_pointer as i32 == addr_kSpritemap_Nothing_A0 as i32
            || ((*E).properties & 0x400) != 0
            || (*E).enemy_ptr as i32 == addr_kEnemyDef_DAFF as i32
            || (*E).invincibility_timer != 0
        {
            return;
        }
        for pidx in 0i32..5 {
            let j: u16 = word!(game.projectile_type[pidx]);
            if j != 0
                && (j & 0xf00) != 768
                && (j & 0xf00) != 1280
                && (j as i32 & 0xf00) - 1792i32 & 0x8000 != 0
            {
                let x: u16 = abs16((word!(game.projectile_x_pos[pidx])).wrapping_sub((*E).x_pos));
                let y: u16 = abs16((word!(game.projectile_y_pos[pidx])).wrapping_sub((*E).y_pos));
                if (x as i32 - word!(game.projectile_x_radius[pidx]) as i32) < (*E).x_width as i32
                    && (y as i32 - word!(game.projectile_y_radius[pidx]) as i32) < (*E).y_height as i32
                {
                    if (word!(game.projectile_type[pidx]) & 0xf00) == 512 {
                        word!(game.earthquake_timer = 30);
                        word!(game.earthquake_type = 18);
                    }
                    if ((*E).properties & 0x1000) != 0 || (word!(game.projectile_type[pidx]) & 8) == 0 {
                        word!(game.projectile_dir[pidx], |fresh26| (fresh26) | (0x10u16));
                    }
                    word!(game.collision_detection_index = pidx as u16);
                    CallEnemyAi(
                        (words!(
                            i32,
                            ((*E).bank, (*get_EnemyDef_A2((*E).enemy_ptr, game)).shot_ai)
                        )) as u32,
                        game,
                    );
                    return;
                }
            }
        }
    };
    EnemyBombCollHandler [] {
        word!(game.enemy_processing_stage = 8);
        if word!(game.bomb_counter) == 0
            || (*E).spritemap_pointer == 0
            || (*E).invincibility_timer != 0
            || (*E).enemy_ptr as i32 == addr_kEnemyDef_DAFF as i32
        {
            return;
        }
        for pidx in 5i32..10 {
            if !(word!(game.projectile_type[pidx]) == 0
                || word!(game.projectile_variables[pidx]) != 0
                || (word!(game.projectile_type[pidx]) & 0xf00) != 1280
                    && (word!(game.projectile_type[pidx]) & 0x8000) == 0)
                && (abs16((word!(game.projectile_x_pos[pidx])).wrapping_sub((*E).x_pos)) as i32
                    - word!(game.projectile_x_radius[pidx]) as i32)
                    < (*E).x_width as i32
                && (abs16((word!(game.projectile_y_pos[pidx])).wrapping_sub((*E).y_pos)) as i32
                    - word!(game.projectile_y_radius[pidx]) as i32)
                    < (*E).y_height as i32
            {
                word!(game.collision_detection_index = pidx as u16);
                word!(game.projectile_dir[pidx], |fresh23| (fresh23) | (0x10u16));
                CallEnemyAi(
                    (words!(
                        i32,
                        ((*E).bank, (*get_EnemyDef_A2((*E).enemy_ptr, game)).shot_ai)
                    )) as u32,
                    game,
                );
                return;
            }
        }
    };
    NormalEnemyTouchAiSkipDeathAnim [] {
        let ED = get_EnemyDef_A2((*E).enemy_ptr, game);
        if word!(game.samus_contact_damage_index) == 0 {
            Samus_DealDamage(SuitDamageDivision((*ED).damage, game), game);
            word!(game.samus_invincibility_timer = 96);
            word!(game.samus_knockback_timer = 5);
            word!(
                game.knockback_x_dir =
                    (((word!(game.samus_x_pos)) as i16).wrapping_sub(((*E).x_pos) as i16) >= 0) as u16
            );
            return;
        }
        let mut r20: u16 = (word!(game.samus_contact_damage_index)).wrapping_add(15);
        let mut r22: u16 = if word!(game.samus_contact_damage_index) == 1 {
            500
        } else if word!(game.samus_contact_damage_index) == 2 {
            300
        } else if word!(game.samus_contact_damage_index) == 3 {
            2000
        } else {
            r20 = r20.wrapping_add(1);
            if word!(game.samus_contact_damage_index) == 4 {
                CallSomeSamusCode(4, game);
            }
            200
        };
        let vp: u16 = (if (*ED).vulnerability_ptr != 0 {
            (*ED).vulnerability_ptr as i32
        } else {
            addr_stru_B4EC1C as i32
        }) as u16;
        word!(
            game.last_enemy_power = *(&raw mut (*(get_Vulnerability
                as unsafe fn(u16, &mut Game) -> *mut Vulnerability)(
                (r20).wrapping_add(vp), game,
            ))
            .power as *mut u16)
        );
        let varE32: u16 = (word!(game.last_enemy_power)) & (0x7fu16);
        if (word!(game.last_enemy_power) & 0x7f) != 0 {
            let dmg: u16 = ((r22 as i32 >> 1) * varE32 as i32) as u16;
            if dmg != 0 {
                (*E).flash_timer = (if (*ED).hurt_ai_time != 0 {
                    (*ED).hurt_ai_time as i32
                } else {
                    4
                }) as u16;
                (*E).ai_handler_bits = ((*E).ai_handler_bits) | ((kEnemyAiBits_Hurt) as u16);
                word! { game, 0; samus_invincibility_timer, samus_knockback_timer }
                (*E).health = (if (((*E).health) as i16).wrapping_sub((dmg) as i16) < 0 {
                    0
                } else {
                    ((((*E).health) as i16).wrapping_sub((dmg) as i16)) as i32
                }) as u16;
                QueueSfx2_Max1(0xb, game);
            }
        }
    };
}
pub unsafe fn NormalEnemyPowerBombAiSkipDeathAnim_CurEnemy(game: &mut Game) {
    NormalEnemyPowerBombAiSkipDeathAnim(game);
}
pub unsafe fn NormalEnemyPowerBombAiSkipDeathAnim(game: &mut Game) {
    let v0 = gEnemyData(word!(game.cur_enemy_index) as i32, game);
    let mut vulnerability_ptr: u16 = (*get_EnemyDef_A2((*v0).enemy_ptr, game)).vulnerability_ptr;
    if vulnerability_ptr == 0 {
        vulnerability_ptr = -5092i32 as u16;
    }
    let power_bomb: u8 = (*get_Vulnerability(vulnerability_ptr, game)).power_bomb;
    if power_bomb != 255 {
        let varE32: u16 = ((power_bomb) as u16) & (0x7fu16);
        if (power_bomb & 0x7f) != 0 {
            let ttt: u32 = (100u32).wrapping_mul((varE32) as u32);
            if ttt != 0 {
                (*gEnemyData(word!(game.cur_enemy_index) as i32, game)).invincibility_timer = 48;
                let j = gEnemyData(word!(game.cur_enemy_index) as i32, game);
                let mut EnemyDef_A2 = get_EnemyDef_A2((*j).enemy_ptr, game);
                let mut hurt_ai_time: i16 = (*EnemyDef_A2).hurt_ai_time as i16;
                if (*EnemyDef_A2).hurt_ai_time == 0 {
                    hurt_ai_time = 4;
                }
                let v6 = gEnemyData(word!(game.cur_enemy_index) as i32, game);
                (*v6).flash_timer = ((hurt_ai_time) as u16).wrapping_add(8);
                (*v6).ai_handler_bits = ((*v6).ai_handler_bits) | (2u16);
                let health: u16 = (*v6).health;
                let mut v9: bool = (health as u32) < ttt;
                let mut v8: u16 = (health as u32).wrapping_sub(ttt) as u16;
                v9 = !v9;
                if v8 == 0 || !v9 {
                    v8 = 0;
                }
                (*v6).health = v8;
            }
        }
    }
}
pub unsafe fn NormalEnemyShotAi(game: &mut Game) {
    let varE2E: u16 = NormalEnemyShotAiSkipDeathAnim(game);
    let E = gEnemyData(word!(game.cur_enemy_index) as i32, game);
    if varE2E != 0 {
        CreateSpriteAtPos((*E).x_pos, (*E).y_pos, 55, 0, game);
    }
    if (*E).health == 0 {
        let j: u16 = ((byte!(@pointer (ram_ptr!(game, u16; 0xc18))
            .offset(word!(game.collision_detection_index) as isize); 1))
            as u16)
            & (0xfu16);
        (*gEnemySpawnData(word!(game.cur_enemy_index) as i32, game)).cause_of_death = j;
        let mut death_anim: u16 = 2;
        if j == 2 {
            let ED = get_EnemyDef_A2((*E).enemy_ptr, game);
            if (*ED).death_anim as i32 - 3i32 & 0x8000 == 0 {
                death_anim = (*ED).death_anim;
            }
        } else {
            death_anim = (*get_EnemyDef_A2((*E).enemy_ptr, game)).death_anim;
        }
        EnemyDeathAnimation((*E).enemy_ptr, death_anim, game);
    }
}
pub unsafe fn NormalEnemyShotAiSkipDeathAnim_CurEnemy(game: &mut Game) {
    NormalEnemyShotAiSkipDeathAnim(game);
}
pub unsafe fn EnemyFunc_A6B4_UsedBySporeSpawn(game: &mut Game) {
    let varE2E: u16 = NormalEnemyShotAiSkipDeathAnim(game);
    if varE2E != 0 {
        let ED = gEnemyData(word!(game.cur_enemy_index) as i32, game);
        CreateSpriteAtPos((*ED).x_pos, (*ED).y_pos, 55, 0, game);
    }
}
pub unsafe fn NormalEnemyShotAiSkipDeathAnim(game: &mut Game) -> u16 {
    let mut varE2E: u16 = 0;
    let mut varE32: u16 = 0;
    let v0: i32 = word!(game.collision_detection_index) as i32;
    let mut pd: u16 = word!(game.projectile_damage[v0]);
    let r18: u16 = word!(game.projectile_type[v0]);
    let j = gEnemyData(word!(game.cur_enemy_index) as i32, game);
    let mut vulnerability_ptr: u16 = (*get_EnemyDef_A2((*j).enemy_ptr, game)).vulnerability_ptr;
    if vulnerability_ptr == 0 {
        vulnerability_ptr = addr_stru_B4EC1C as u16;
    }
    let r20: u16 = vulnerability_ptr;
    let apply_damage: bool = if (r18 & 0xf00) != 0 {
        let mut v5: i16 = ((r18) as i16) & (0xf00i16);
        if (r18 & 0xf00) == 256 || v5 == 512 {
            let mut v6: i16 = ((r18 as i32 & 0xf00) >> 8) as i16;
            varE32 = (((*get_Vulnerability((r20).wrapping_add((v6) as u16), game)).plasma_ice_wave)
                as u16)
                & (0x7fu16);
            true
        } else if v5 == 1280 {
            varE32 = (((*get_Vulnerability(r20, game)).bomb) as u16) & (0x7fu16);
            true
        } else {
            !(v5 != 768)
                && ({
                    varE32 = (((*get_Vulnerability(r20, game)).power_bomb) as u16) & (0x7fu16);
                    true
                })
        }
    } else {
        word!(
            game.last_enemy_power =
                (*get_Vulnerability((r20 as i32 + (r18 as i32 & 0xf)) as u16, game,)).power as u16
        );
        varE32 = (word!(game.last_enemy_power)) & (0x7fu16);
        if word!(game.last_enemy_power) != 255 {
            !((r18 & 0x10) != 0)
                || ({
                    let charged_beam: u8 = (*get_Vulnerability(r20, game)).charged_beam;
                    !(charged_beam == 255)
                        && ({
                            let v4: u16 = ((charged_beam) as u16) & (0xfu16);
                            !(v4 == 0)
                                && ({
                                    varE32 = v4;
                                    true
                                })
                        })
                })
        } else {
            let v8 = gEnemyData(word!(game.cur_enemy_index) as i32, game);
            if (*v8).frozen_timer == 0 {
                QueueSfx3_Max3(0xa, game);
            }
            let mut v9: i16 = 400;
            if word!(game.area_index) == 2 {
                v9 = 300;
            }
            (*v8).frozen_timer = v9 as u16;
            (*v8).ai_handler_bits = ((*v8).ai_handler_bits) | (4u16);
            (*v8).invincibility_timer = 10;
            return varE2E;
        }
    };
    if apply_damage {
        let mut ttt: u32 = ((pd >> 1) as u32).wrapping_mul(varE32 as u32);
        let mut r42: u16 = ttt as u16;
        if r42 != 0 {
            pd = r42;
            let v10 = gEnemyData(word!(game.cur_enemy_index) as i32, game);
            let mut EnemyDef_A2 = get_EnemyDef_A2((*v10).enemy_ptr, game);
            let mut hurt_ai_time: i16 = (*EnemyDef_A2).hurt_ai_time as i16;
            if (*EnemyDef_A2).hurt_ai_time == 0 {
                hurt_ai_time = 4;
            }
            let v13 = gEnemyData(word!(game.cur_enemy_index) as i32, game);
            (*v13).flash_timer = ((hurt_ai_time) as u16).wrapping_add(8);
            (*v13).ai_handler_bits = ((*v13).ai_handler_bits) | (2u16);
            if (*v13).frozen_timer == 0 {
                let hurt_sfx: u16 = (*get_EnemyDef_A2((*v13).enemy_ptr, game)).hurt_sfx;
                if hurt_sfx != 0 {
                    QueueSfx2_Max3(hurt_sfx, game);
                }
                varE2E = varE2E.wrapping_add(1);
            }
            let v15: u16 = word!(game.cur_enemy_index);
            if (word!(game.projectile_type[word!(game.collision_detection_index)]) & 8) != 0 {
                (*gEnemyData(word!(game.cur_enemy_index) as i32, game)).invincibility_timer = 16;
            }
            let v16 = gEnemyData(v15 as i32, game);
            let health: u16 = (*v16).health;
            let mut v19: bool = health < pd;
            let mut v18: u16 = (health).wrapping_sub(pd);
            v19 = !v19;
            if v18 == 0 || !v19 {
                if (word!(game.projectile_type[word!(game.collision_detection_index)]) & 2) != 0
                    && (word!(game.last_enemy_power) & 0xf0) != 128
                    && (*v16).frozen_timer == 0
                {
                    let mut v20: i16 = 400;
                    if word!(game.area_index) == 2 {
                        v20 = 300;
                    }
                    (*v16).frozen_timer = v20 as u16;
                    (*v16).ai_handler_bits = ((*v16).ai_handler_bits) | (4u16);
                    (*v16).invincibility_timer = 10;
                    QueueSfx3_Max3(0xa, game);
                    return varE2E;
                }
                v18 = 0;
            }
            (*v16).health = v18;
            return varE2E;
        }
    }
    let v7: i32 = word!(game.collision_detection_index) as i32;
    word!(game.projectile_dir[v7], |fresh25| (fresh25) | (0x10u16));
    CreateSpriteAtPos(
        word!(game.projectile_x_pos[v7]),
        word!(game.projectile_y_pos[v7]),
        6,
        0,
        game,
    );
    QueueSfx1_Max3(0x3d, game);
    varE2E
}
pub unsafe fn CreateDudShot(game: &mut Game) {
    let v0: i32 = word!(game.collision_detection_index) as i32;
    CreateSpriteAtPos(
        word!(game.projectile_x_pos[v0]),
        word!(game.projectile_y_pos[v0]),
        6,
        0,
        game,
    );
    QueueSfx1_Max3(0x3d, game);
    word!(
        game.projectile_dir[word!(game.collision_detection_index)],
        |fresh13| (fresh13) | (0x10u16)
    );
}
fn solid_enemy_probe_coordinate(position: u16, fraction: u16, amount: i32, negative: bool) -> u16 {
    let integer_amount = (amount >> 16) as u16;
    let fractional_amount = amount as u16;
    if negative {
        let mut coordinate = position.wrapping_sub(integer_amount);
        let mut exact = fraction == fractional_amount;
        if fraction < fractional_amount {
            // Borrow changes the rounding test to the integer before decrement.
            exact = coordinate == 1;
            coordinate = coordinate.wrapping_sub(1);
        }
        if !exact {
            coordinate = coordinate.wrapping_sub(1);
        }
        coordinate
    } else {
        let mut coordinate = position.wrapping_add(integer_amount);
        let sum = u32::from(fraction) + u32::from(fractional_amount);
        let mut exact = sum == 0;
        if sum > 0xffff {
            // Carry changes the rounding test to the integer before increment.
            exact = coordinate == 0xffff;
            coordinate = coordinate.wrapping_add(1);
        }
        if !exact {
            coordinate = coordinate.wrapping_add(1);
        }
        coordinate
    }
}

fn solid_enemy_probe_position(amount: i32, game: &Game) -> PositionAndWidth {
    let direction = game.samus_collision_direction() & 3;
    let mut pos = PositionAndWidth {
        x_pos: 0,
        x_subpos: 0,
        y_pos: 0,
        y_subpos: 0,
        width: 0,
        height: 0,
    };
    if direction < 2 {
        pos.x_pos = solid_enemy_probe_coordinate(
            game.samus_x_pos(),
            game.samus_x_subpos(),
            amount,
            direction == 0,
        );
        pos.y_pos = game.samus_y_pos();
        pos.y_subpos = game.samus_y_subpos();
    } else {
        pos.y_pos = solid_enemy_probe_coordinate(
            game.samus_y_pos(),
            game.samus_y_subpos(),
            amount,
            direction == 2,
        );
        pos.x_pos = game.samus_x_pos();
        pos.x_subpos = game.samus_x_subpos();
    }
    pos.width = game.samus_x_radius();
    pos.height = game.samus_y_radius();
    pos
}

struct SolidEnemyCollisionBody<'a> {
    bytes: &'a [u8],
}
impl<'a> SolidEnemyCollisionBody<'a> {
    fn new(game: &'a Game, index: u16) -> Self {
        Self {
            bytes: &game.g_ram[0xf78 + usize::from(index)..],
        }
    }
    fn word(&self, offset: usize) -> u16 {
        u16::from_le_bytes([self.bytes[offset], self.bytes[offset + 1]])
    }
    fn x_pos(&self) -> u16 {
        self.word(core::mem::offset_of!(EnemyData, x_pos))
    }
    fn y_pos(&self) -> u16 {
        self.word(core::mem::offset_of!(EnemyData, y_pos))
    }
    fn x_width(&self) -> u16 {
        self.word(core::mem::offset_of!(EnemyData, x_width))
    }
    fn y_height(&self) -> u16 {
        self.word(core::mem::offset_of!(EnemyData, y_height))
    }
    fn frozen_timer(&self) -> u16 {
        self.word(core::mem::offset_of!(EnemyData, frozen_timer))
    }
    fn properties(&self) -> u16 {
        self.word(core::mem::offset_of!(EnemyData, properties))
    }
}

fn solid_enemy_axis_overlaps(
    enemy_position: u16,
    samus_position: u16,
    enemy_radius: u16,
    samus_radius: u16,
) -> bool {
    let delta = enemy_position.wrapping_sub(samus_position);
    let distance = (delta as i16).wrapping_abs() as u16;
    distance < enemy_radius || distance.wrapping_sub(enemy_radius) < samus_radius
}

pub fn Samus_CheckSolidEnemyColl(amt: i32, game: &mut Game) -> CheckEnemyColl_Result {
    if game.interactive_enemy_indexes_write_ptr() == 0 {
        return CheckSolidEnemyColl_Result { collision: 0, amt };
    }
    let pos = solid_enemy_probe_position(amt, game);
    game.set_collision_detection_index(0);
    let mut list_index = 0;
    loop {
        let index = game.interactive_enemy_index(list_index);
        if index == 0xffff {
            break;
        }
        game.set_collision_detection_index(index);
        let enemy = SolidEnemyCollisionBody::new(game, index);
        if (enemy.frozen_timer() != 0 || enemy.properties() & 0x8000 != 0)
            && solid_enemy_axis_overlaps(enemy.x_pos(), pos.x_pos, enemy.x_width(), pos.width)
            && solid_enemy_axis_overlaps(enemy.y_pos(), pos.y_pos, enemy.y_height(), pos.height)
        {
            let direction = game.samus_collision_direction() & 3;
            let (distance, exact) = match direction {
                0 | 2 => {
                    let (position, radius, enemy_position, enemy_radius) = if direction == 0 {
                        (
                            game.samus_x_pos(),
                            game.samus_x_radius(),
                            enemy.x_pos(),
                            enemy.x_width(),
                        )
                    } else {
                        (
                            game.samus_y_pos(),
                            game.samus_y_radius(),
                            enemy.y_pos(),
                            enemy.y_height(),
                        )
                    };
                    let enemy_edge = enemy_position.wrapping_add(enemy_radius);
                    // Negative-edge equality uses the unwrapped Samus subtraction.
                    let samus_edge = i32::from(position) - i32::from(radius);
                    (
                        (samus_edge - i32::from(enemy_edge)) as i16,
                        samus_edge == i32::from(enemy_edge),
                    )
                }
                1 | 3 => {
                    let (position, radius, enemy_position, enemy_radius) = if direction == 1 {
                        (
                            game.samus_x_pos(),
                            game.samus_x_radius(),
                            enemy.x_pos(),
                            enemy.x_width(),
                        )
                    } else {
                        (
                            game.samus_y_pos(),
                            game.samus_y_radius(),
                            enemy.y_pos(),
                            enemy.y_height(),
                        )
                    };
                    let distance = (i32::from(enemy_position)
                        - i32::from(enemy_radius)
                        - i32::from(radius)
                        - i32::from(position)) as i16;
                    (distance, distance == 0)
                }
                _ => unreachable!(),
            };
            if exact || distance >= 0 {
                if exact {
                    // Exact contact clears Y fraction even for horizontal probes.
                    game.set_samus_y_subpos(0);
                }
                let index = game.collision_detection_index();
                game.set_enemy_index_colliding_dir(game.samus_collision_direction(), index);
                return CheckSolidEnemyColl_Result {
                    collision: 0xffff,
                    amt: if exact { 0 } else { i32::from(distance) << 16 },
                };
            }
        }
        list_index += 1;
    }
    CheckSolidEnemyColl_Result { collision: 0, amt }
}
pub fn CheckIfEnemyTouchesSamus(k: u16, game: &mut Game) -> u16 {
    let enemy = SolidEnemyCollisionBody::new(game, k);
    if !solid_enemy_axis_overlaps(
        game.samus_x_pos(),
        enemy.x_pos(),
        game.samus_x_radius(),
        enemy.x_width(),
    ) {
        return 0;
    }
    let samus_y = game.samus_y_pos();
    let enemy_y = enemy.y_pos();
    if samus_y.wrapping_add(3).wrapping_sub(enemy_y) as i16 >= 0 {
        return 0;
    }
    let distance = enemy_y.wrapping_sub(samus_y.wrapping_add(3));
    let radius = game.samus_y_radius();
    if distance < radius || distance.wrapping_sub(radius) <= enemy.y_height() {
        0xffff
    } else {
        0
    }
}
pub fn EnemyFunc_AC67(k: u16, game: &mut Game) -> u16 {
    let enemy = SolidEnemyCollisionBody::new(game, k);
    let delta = game.samus_x_pos().wrapping_sub(enemy.x_pos());
    let distance = (delta as i16).wrapping_abs() as u16;
    let radius = game.samus_x_radius();
    let gap = distance.wrapping_sub(radius);
    if distance >= radius && gap >= enemy.x_width() && gap >= 8 {
        return 0;
    }
    if solid_enemy_axis_overlaps(
        game.samus_y_pos(),
        enemy.y_pos(),
        game.samus_y_radius(),
        enemy.y_height(),
    ) {
        0xffff
    } else {
        0
    }
}
pub unsafe fn EnemyFunc_ACA8(base_pt: Point16U, samus_pt: Point16U, game: &mut Game) -> PairU16 {
    let dx = samus_pt.x.wrapping_sub(base_pt.x);
    let x = (dx as i16).wrapping_abs() as u16;
    if x.wrapping_sub(255) & 0x8000 == 0 {
        return PairU16 { k: x, j: 0 };
    }
    let dy = samus_pt.y.wrapping_sub(base_pt.y);
    let y = (dy as i16).wrapping_abs() as u16;
    if y.wrapping_sub(255) & 0x8000 == 0 {
        return PairU16 { k: y, j: 0 };
    }
    let angle = CalculateAngleFromXY(x, y, game);
    let horizontal = (SineMult8bit(angle, x, game) as i16).wrapping_abs() as u16;
    let vertical = (CosineMult8bit(angle, y, game) as i16).wrapping_abs() as u16;
    let distance = horizontal.wrapping_add(vertical);
    let direction = CalculateAngleFromXY(dx, dy, game);
    PairU16 {
        k: distance,
        j: direction,
    }
}
fn enemy_axis_outside_screen(
    position: u16,
    camera: u16,
    lower_padding: u16,
    upper_edge: u16,
) -> bool {
    (position.wrapping_add(lower_padding).wrapping_sub(camera) as i16) < 0
        || (camera.wrapping_add(upper_edge).wrapping_sub(position) as i16) < 0
}

pub fn CheckIfEnemyIsOnScreen(game: &mut Game) -> u16 {
    let enemy = SolidEnemyCollisionBody::new(game, game.cur_enemy_index());
    u16::from(
        enemy_axis_outside_screen(enemy.x_pos(), game.layer1_x_pos(), 0, 256)
            || enemy_axis_outside_screen(enemy.y_pos(), game.layer1_y_pos(), 0, 256),
    )
}
pub fn EnemyFunc_ADA3(a: u16, game: &mut Game) -> u16 {
    let enemy = SolidEnemyCollisionBody::new(game, game.cur_enemy_index());
    u16::from(
        enemy_axis_outside_screen(enemy.x_pos(), game.layer1_x_pos(), a, a.wrapping_add(256))
            || enemy_axis_outside_screen(
                enemy.y_pos(),
                game.layer1_y_pos(),
                a,
                a.wrapping_add(256),
            ),
    )
}
pub fn EnemyWithNormalSpritesIsOffScreen(game: &mut Game) -> u16 {
    let enemy = SolidEnemyCollisionBody::new(game, game.cur_enemy_index());
    u16::from(
        enemy_axis_outside_screen(
            enemy.x_pos(),
            game.layer1_x_pos(),
            enemy.x_width(),
            enemy.x_width().wrapping_add(256),
        ) || enemy_axis_outside_screen(enemy.y_pos(), game.layer1_y_pos(), 8, 248),
    )
}
pub fn DetermineDirectionOfSamusFromEnemy(game: &mut Game) -> u16 {
    let index = game.cur_enemy_index();
    if IsSamusWithinEnemy_Y(index, 32, game) != 0 {
        let index = game.cur_enemy_index();
        return if GetSamusEnemyDelta_X(index, game) & 0x8000 != 0 {
            7
        } else {
            2
        };
    }
    let index = game.cur_enemy_index();
    if IsSamusWithinEnemy_X(index, 32, game) != 0 {
        let index = game.cur_enemy_index();
        return if GetSamusEnemyDelta_Y(index, game) & 0x8000 != 0 {
            0
        } else {
            4
        };
    }
    let index = game.cur_enemy_index();
    let left = GetSamusEnemyDelta_X(index, game) & 0x8000 != 0;
    let index = game.cur_enemy_index();
    let above = GetSamusEnemyDelta_Y(index, game) & 0x8000 != 0;
    match (left, above) {
        (true, true) => 8,
        (true, false) => 6,
        (false, true) => 1,
        (false, false) => 3,
    }
}
pub fn GetSamusEnemyDelta_Y(k: u16, game: &mut Game) -> u16 {
    let enemy = SolidEnemyCollisionBody::new(game, k);
    game.samus_y_pos().wrapping_sub(enemy.y_pos())
}
pub fn GetSamusEnemyDelta_X(k: u16, game: &mut Game) -> u16 {
    let enemy = SolidEnemyCollisionBody::new(game, k);
    game.samus_x_pos().wrapping_sub(enemy.x_pos())
}
pub fn IsSamusWithinEnemy_Y(k: u16, a: u16, game: &mut Game) -> u16 {
    let delta = GetSamusEnemyDelta_Y(k, game);
    let distance = (delta as i16).wrapping_abs() as u16;
    u16::from((distance.wrapping_sub(a) as i16) < 0)
}
pub fn IsSamusWithinEnemy_X(k: u16, a: u16, game: &mut Game) -> u16 {
    let delta = GetSamusEnemyDelta_X(k, game);
    let distance = (delta as i16).wrapping_abs() as u16;
    u16::from((distance.wrapping_sub(a) as i16) < 0)
}
pub fn SignExtend8(a: u16) -> u16 {
    // Preserve the input high byte when bit 7 is clear.
    if a & 0x80 != 0 {
        a | 0xff00
    } else {
        a
    }
}
pub fn Abs16(a: u16) -> u16 {
    (a as i16).wrapping_abs() as u16
}
fn SineMult8bitInner(amplitude: u16, phase: u16, game: &Game) -> u32 {
    let sample = game.read_rom_byte(0xa0b143, usize::from(phase & 0x7f));
    let product = u16::from(sample) * u16::from(amplitude as u8);
    let mut integer = product >> 8;
    let mut fraction = product << 8;
    if phase & 0x80 != 0 {
        // The original negates each half independently, without a carry.
        integer = integer.wrapping_neg();
        fraction = fraction.wrapping_neg();
    }
    (u32::from(integer) << 16) | u32::from(fraction)
}

pub fn CosineMult8bit(phase: u16, amplitude: u16, game: &mut Game) -> u16 {
    (SineMult8bitInner(amplitude, phase.wrapping_add(64) as u8 as u16, game) >> 16) as u16
}

pub fn SineMult8bit(phase: u16, amplitude: u16, game: &mut Game) -> u16 {
    (SineMult8bitInner(amplitude, phase.wrapping_add(0x80) as u8 as u16, game) >> 16) as u16
}

crate::forward_calls! { pub fn (phase: u16, amplitude: u16, game: &mut Game) -> u32;
    CosineMult8bitFull => SineMult8bitInner(amplitude, phase.wrapping_add(64) as u8 as u16, game);
    SineMult8bitFull => SineMult8bitInner(amplitude, phase.wrapping_add(0x80) as u8 as u16, game);
}
pub fn ConvertAngleToXy(phase: u16, amplitude: u16, game: &mut Game) -> Point32 {
    let x_index = usize::from(phase.wrapping_add(0x40) & 0x7f);
    let x_sample = game.read_rom_word(0xa0b7ee, x_index * 2);
    let x = u32::from(x_sample) * u32::from(amplitude);
    let y_index = usize::from(phase.wrapping_add(0x80) & 0x7f);
    let y_sample = game.read_rom_word(0xa0b7ee, y_index * 2);
    let y = u32::from(y_sample) * u32::from(amplitude);
    Point32 { x, y }
}
fn add_enemy_vector_coordinate(bytes: &mut [u8], amount: u32) {
    let integer = u16::from_le_bytes([bytes[0], bytes[1]]);
    let fraction = u16::from_le_bytes([bytes[2], bytes[3]]);
    let position = (u32::from(integer) << 16) | u32::from(fraction);
    let adjusted = position.wrapping_add(amount);
    bytes[2..4].copy_from_slice(&(adjusted as u16).to_le_bytes());
    bytes[0..2].copy_from_slice(&((adjusted >> 16) as u16).to_le_bytes());
}

pub fn EnemyFunc_B691(phase: u16, pt: Point32, game: &mut Game) {
    let slot = 0xf78 + usize::from(game.cur_enemy_index());
    let x = if phase.wrapping_add(64) & 0x80 != 0 {
        pt.x.wrapping_neg()
    } else {
        pt.x
    };
    let offset = slot + core::mem::offset_of!(EnemyData, x_pos);
    add_enemy_vector_coordinate(&mut game.g_ram[offset..offset + 4], x);
    let y = if phase.wrapping_add(128) & 0x80 != 0 {
        pt.y.wrapping_neg()
    } else {
        pt.y
    };
    let offset = slot + core::mem::offset_of!(EnemyData, y_pos);
    add_enemy_vector_coordinate(&mut game.g_ram[offset..offset + 4], y);
}
pub unsafe fn EnemyFunc_Divide(a: u32, b: u32) -> u32 {
    if a != 0 {
        b.wrapping_div(a)
    } else {
        0
    }
}
fn corrected_samus_previous_position(position: u16, previous: u16) -> Option<u16> {
    let delta = position.wrapping_sub(previous);
    let distance = Abs16(delta);
    if distance.wrapping_sub(12) & 0x8000 != 0 {
        return None;
    }
    Some(if delta & 0x8000 == 0 {
        position.wrapping_add(12)
    } else {
        position.wrapping_sub(12)
    })
}

pub fn EnemyFunc_B7A1(game: &mut Game) {
    if let Some(previous) =
        corrected_samus_previous_position(game.samus_y_pos(), game.samus_prev_y_pos())
    {
        game.set_samus_prev_y_pos(previous);
    }
    if let Some(previous) =
        corrected_samus_previous_position(game.samus_x_pos(), game.samus_prev_x_pos())
    {
        game.set_samus_prev_x_pos(previous);
    }
}
unsafe fn spawn_nearby_enemy_item_drops(
    definition: u16,
    index: u16,
    count: u16,
    spawn_parameter: u16,
    game: &mut Game,
) {
    for _ in 0..count {
        // X origin is read before RNG; Y origin and random high byte are live afterward.
        let x = game.special_death_item_drop_x_origin_pos();
        let random = NextRandom(game);
        game.eproj_spawn_pt.x = x.wrapping_add(random & 0x1f).wrapping_sub(16);
        let y = game.special_death_item_drop_y_origin_pos();
        let random_y = (game.random_number() & 0x1f00) >> 8;
        game.eproj_spawn_pt.y = y.wrapping_add(random_y).wrapping_sub(16);
        SpawnEnemyDrops(definition, index, spawn_parameter, game);
    }
}

unsafe fn spawn_area_enemy_item_drops(
    definition: u16,
    index: u16,
    x: u16,
    y: u16,
    x_mask: u16,
    game: &mut Game,
) {
    for _ in 0..16 {
        game.eproj_spawn_pt.x = (NextRandom(game) & x_mask).wrapping_add(x);
        game.eproj_spawn_pt.y = ((game.random_number() & 0x3f00) >> 8).wrapping_add(y);
        SpawnEnemyDrops(definition, index, 0, game);
    }
}

pub unsafe fn Enemy_ItemDrop_MiniKraid(k: u16, death_animation: u16, game: &mut Game) {
    spawn_nearby_enemy_item_drops(addr_kEnemyDef_E0FF as u16, k, 4, death_animation, game);
}
crate::forward_calls! { pub unsafe fn (k: u16, game: &mut Game);
    Enemy_ItemDrop_LowerNorfairSpacePirate =>
        spawn_nearby_enemy_item_drops(addr_kEnemyDef_F593 as u16, k, 5, 0, game);
    Enemy_ItemDrop_Metroid =>
        spawn_nearby_enemy_item_drops(addr_kEnemyDef_DD7F as u16, k, 5, 0, game);
    Enemy_ItemDrop_Ridley =>
        spawn_area_enemy_item_drops(addr_kEnemyDef_E17F as u16, k, 64, 320, 127, game);
    Enemy_ItemDrop_Crocomire =>
        spawn_area_enemy_item_drops(addr_kEnemyDef_DDBF as u16, k, 576, 96, 127, game);
    Enemy_ItemDrop_Phantoon =>
        spawn_area_enemy_item_drops(addr_kEnemyDef_E4BF as u16, k, 64, 96, 127, game);
    Enemy_ItemDrop_Botwoon =>
        spawn_area_enemy_item_drops(addr_kEnemyDef_F293 as u16, k, 64, 128, 127, game);
    Enemy_ItemDrop_Kraid =>
        spawn_area_enemy_item_drops(addr_kEnemyDef_E2BF as u16, k, 128, 352, 255, game);
    Enemy_ItemDrop_BombTorizo =>
        spawn_area_enemy_item_drops(addr_kEnemyDef_EEFF as u16, k, 64, 96, 127, game);
    Enemy_ItemDrop_GoldenTorizo =>
        spawn_area_enemy_item_drops(addr_kEnemyDef_EEFF as u16, k, 128, 288, 255, game);
    Enemy_ItemDrop_SporeSpawn =>
        spawn_area_enemy_item_drops(addr_kEnemyDef_DF3F as u16, k, 64, 528, 127, game);
    Enemy_ItemDrop_Draygon =>
        spawn_area_enemy_item_drops(addr_kEnemyDef_DE3F as u16, k, 128, 352, 255, game);
}
pub fn CompareDistToSamus_X(k: u16, a: u16, game: &mut Game) -> u8 {
    let delta = GetSamusEnemyDelta_X(k, game);
    u8::from(Abs16(delta) >= a)
}

macro_rules! calculate_enemy_motion_leading_edge {
    (
        $amount:ident, $game:ident, $enemy_index:ident, $position:ident, $subposition:ident,
        $edge:ident, $radius:ident $(,)?
    ) => {
        let position = ($amount as u32).wrapping_add(
            (u32::from($game.enemy_coordinates($enemy_index).$position()) << 16)
                | u32::from($game.enemy_coordinates($enemy_index).$subposition()),
        );
        let x = (position >> 16) as u16;
        let $edge = if $amount < 0 {
            x.wrapping_sub($game.enemy_coordinates($enemy_index).$radius())
        } else {
            $game
                .enemy_coordinates($enemy_index)
                .$radius()
                .wrapping_add(x)
                .wrapping_sub(1)
        };
    };
}

pub fn EnemyFunc_BBBF(k: u16, amt: i32, game: &mut Game) -> u8 {
    let top = game
        .enemy_coordinates(k)
        .y_pos()
        .wrapping_sub(game.enemy_coordinates(k).y_height());
    let mut rows = game
        .enemy_coordinates(k)
        .y_height()
        .wrapping_add(game.enemy_coordinates(k).y_pos())
        .wrapping_sub(1)
        .wrapping_sub(top & 0xfff0)
        >> 4;
    let row_offset = u16::from((top >> 4) as u8) * u16::from(game.room_width_in_blocks() as u8);
    calculate_enemy_motion_leading_edge!(amt, game, k, x_pos, x_subpos, edge, x_width);
    let mut block_offset = row_offset.wrapping_add(edge >> 4).wrapping_mul(2);
    loop {
        let offset = 0x10002 + usize::from(block_offset);
        let block = u16::from_le_bytes([game.g_ram[offset], game.g_ram[offset + 1]]);
        if block & 0x8000 != 0 {
            return 1;
        }
        block_offset = block_offset.wrapping_add(game.room_width_in_blocks().wrapping_mul(2));
        rows = rows.wrapping_sub(1);
        if rows & 0x8000 != 0 {
            return 0;
        }
    }
}

macro_rules! scan_enemy_collision_block_row {
    ($edge:ident, $game:ident, $left:ident, $columns:ident) => {
        let row_offset =
            u16::from(($edge >> 4) as u8) * u16::from($game.room_width_in_blocks() as u8);
        // This cursor is not truncated to a word as the horizontal scan's is.
        let mut block_offset = 2 * (usize::from(row_offset) + usize::from($left >> 4));
        loop {
            let offset = 0x10002 + block_offset;
            let block = u16::from_le_bytes([$game.g_ram[offset], $game.g_ram[offset + 1]]);
            if block & 0x8000 != 0 {
                return 1;
            }
            $columns = $columns.wrapping_sub(1);
            if $columns & 0x8000 != 0 {
                return 0;
            }
            block_offset += 2;
        }
    };
}

pub fn EnemyFunc_BC76(k: u16, amt: i32, game: &mut Game) -> u8 {
    let left = game
        .enemy_coordinates(k)
        .x_pos()
        .wrapping_sub(game.enemy_coordinates(k).x_width());
    let mut columns = game
        .enemy_coordinates(k)
        .x_width()
        .wrapping_add(game.enemy_coordinates(k).x_pos())
        .wrapping_sub(1)
        .wrapping_sub(left & 0xfff0)
        >> 4;
    calculate_enemy_motion_leading_edge!(amt, game, k, y_pos, y_subpos, edge, y_height);
    scan_enemy_collision_block_row!(edge, game, left, columns);
}
pub fn EnemyFunc_BF8A(k: u16, a: u16, amt: i32, game: &mut Game) -> u8 {
    let left = game
        .enemy_coordinates(k)
        .x_pos()
        .wrapping_sub(game.enemy_coordinates(k).x_width());
    let mut columns = game
        .enemy_coordinates(k)
        .x_width()
        .wrapping_add(game.enemy_coordinates(k).x_pos())
        .wrapping_sub(1)
        .wrapping_sub(left & 0xfff0)
        >> 4;
    let edge = if a & 1 != 0 {
        let position = (amt as u32).wrapping_add(
            (u32::from(game.enemy_coordinates(k).y_pos()) << 16)
                | u32::from(game.enemy_coordinates(k).y_subpos()),
        );
        game.enemy_coordinates(k)
            .y_height()
            .wrapping_add((position >> 16) as u16)
            .wrapping_sub(1)
    } else {
        let position = ((u32::from(game.enemy_coordinates(k).y_pos()) << 16)
            | u32::from(game.enemy_coordinates(k).y_subpos()))
        .wrapping_sub(amt as u32);
        ((position >> 16) as u16).wrapping_sub(game.enemy_coordinates(k).y_height())
    };
    scan_enemy_collision_block_row!(edge, game, left, columns);
}
pub unsafe fn CalculateAngleOfSamusFromEproj(k: u16, game: &mut Game) -> u16 {
    let x = game
        .samus_x_pos()
        .wrapping_sub(game.enemy_projectile_x_pos(k));
    let y = game
        .samus_y_pos()
        .wrapping_sub(game.enemy_projectile_y_pos(k));
    CalculateAngleFromXY(x, y, game)
}
pub unsafe fn CalculateAngleOfSamusFromEnemy(k: u16, game: &mut Game) -> u16 {
    let x = GetSamusEnemyDelta_X(k, game);
    let y = GetSamusEnemyDelta_Y(k, game);
    CalculateAngleFromXY(x, y, game)
}
pub unsafe fn CalculateAngleOfEnemyXfromEnemyY(k: u16, j: u16, game: &mut Game) -> u16 {
    let x = game
        .enemy_coordinates(k)
        .x_pos()
        .wrapping_sub(game.enemy_coordinates(j).x_pos());
    let y = game
        .enemy_coordinates(k)
        .y_pos()
        .wrapping_sub(game.enemy_coordinates(j).y_pos());
    CalculateAngleFromXY(x, y, game)
}
pub unsafe fn CalculateAngleFromXY(x: u16, y: u16, game: &mut Game) -> u16 {
    let quadrant = (u8::from(x & 0x8000 != 0) << 1) | u8::from(y & 0x8000 != 0);
    let x = Abs16(x);
    let y = Abs16(y);
    let x_dominant = y < x;
    let (numerator, denominator) = if x_dominant { (y, x) } else { (x, y) };
    let div = SnesDivide(numerator.wrapping_shl(8), denominator as u8);
    let offset = (div >> 3) as u8;
    let angle = match (x_dominant, quadrant) {
        (true, 0) => offset.wrapping_add(64),
        (true, 1) => 64u8.wrapping_sub(offset),
        (true, 2) => 192u8.wrapping_sub(offset),
        (true, 3) => offset.wrapping_sub(64),
        (false, 0) => 128u8.wrapping_sub(offset),
        (false, 1) => offset,
        (false, 2) => offset.wrapping_add(128),
        (false, 3) => 0u8.wrapping_sub(offset),
        _ => unreachable!(),
    };
    u16::from(angle)
}
pub fn IsEnemyLeavingScreen(k: u16, game: &mut Game) -> u8 {
    let enemy = SolidEnemyCollisionBody::new(game, k);
    let x = enemy.x_pos();
    if (x as i16) < 0 {
        return 1;
    }
    let edge = enemy
        .x_width()
        .wrapping_add(x)
        .wrapping_sub(game.layer1_x_pos());
    let past_screen = edge.wrapping_sub(256).wrapping_sub(enemy.x_width());
    u8::from((edge as i16) < 0 || (past_screen as i16) >= 0)
}
pub unsafe fn ProcessEnemyInstructions(game: &mut Game) {
    // Callbacks may change the current index; animation writes retain this slot.
    let index = game.cur_enemy_index();
    if game.enemy_instructions(index).ai_handler_bits() & 4 != 0 {
        return;
    }
    let timer = game.enemy_instructions(index).instruction_timer();
    let decremented = game
        .enemy_instructions(index)
        .instruction_timer()
        .wrapping_sub(1);
    game.enemy_instructions(index)
        .set_instruction_timer(decremented);
    if timer != 1 {
        let properties = game.enemy_instructions(index).extra_properties() & !0x8000;
        game.enemy_instructions(index)
            .set_extra_properties(properties);
        return;
    }
    if game.enemy_instructions(index).current_instruction() & 0x8000 == 0 {
        crate::native_assertion! {
            @failure
            b"E->current_instruction & 0x8000\0", b"src/sm_a0.c\0", 3399,
            b"void ProcessEnemyInstructions(void)\0"
        };
    }
    let base_ptr = RomBankBase(game.enemy_instructions(index).bank(), game);
    let mut pc = base_ptr.add(usize::from(
        game.enemy_instructions(index).current_instruction(),
    )) as *const u16;
    while *pc & 0x8000 != 0 {
        let address = (u32::from(game.enemy_instructions(index).bank()) << 16) | u32::from(*pc);
        pc = CallEnemyInstr(address, game.cur_enemy_index(), pc.add(1), game);
        if pc.is_null() {
            return;
        }
        if (pc as usize) < 0x10000 {
            pc = base_ptr.add(pc as usize) as *const u16;
        }
    }
    game.enemy_instructions(index).set_instruction_timer(*pc);
    game.enemy_instructions(index)
        .set_spritemap_pointer(*pc.add(1));
    let instruction = (pc as *const u8).add(4).offset_from(base_ptr) as u16;
    game.enemy_instructions(index)
        .set_current_instruction(instruction);
    let properties = game.enemy_instructions(index).extra_properties() | 0x8000;
    game.enemy_instructions(index)
        .set_extra_properties(properties);
}
unsafe fn EnemyBlockCollReact_Spike(_collision: &mut EnemyBlockCollInfo, game: &mut Game) -> u8 {
    let behavior = game.current_block_bts() & 0x7f;
    let plm = game.read_rom_word(0xa0c2da, usize::from(behavior) * 2);
    if plm == 0 {
        return 1;
    }
    SpawnPLM(plm, game);
    0
}
fn EnemyBlockCollHorizReact_Slope(collision: &mut EnemyBlockCollInfo, game: &mut Game) -> u8 {
    let bts = game.current_block_bts();
    if bts & 0x1f >= 5 {
        game.set_current_slope_bts(u16::from(bts));
        EnemyBlockCollHorizReact_Slope_NonSquare(collision, game)
    } else {
        EnemyBlockCollHorizReact_Slope_Square(collision, bts, game)
    }
}
fn EnemyBlockCollVertReact_Slope(collision: &mut EnemyBlockCollInfo, game: &mut Game) -> u8 {
    let bts = game.current_block_bts();
    if bts & 0x1f >= 5 {
        EnemyBlockCollVertReact_Slope_NonSquare(collision, game)
    } else {
        EnemyBlockCollVertReact_Slope_Square(collision, bts, game)
    }
}
fn EnemyBlockCollHorizReact_Slope_Square(
    collision: &mut EnemyBlockCollInfo,
    bts: u8,
    game: &mut Game,
) -> u8 {
    let index = usize::from(bts & 0x1f) * 4
        + usize::from((bts >> 6) ^ ((collision.ebci_r26 & 8) >> 3) as u8);
    let enemy_index = game.cur_enemy_index();
    let enemy = game.enemy_coordinates(enemy_index);
    let position = enemy.y_pos();
    let size = enemy.y_height();
    crate::verify_probe::hit("square-slope.enemy.horiz");
    u8::from(square_slope_occupied(
        index as u16,
        collision.ebci_r28,
        collision.ebci_r30,
        position,
        size,
        2,
    ))
}
fn EnemyBlockCollVertReact_Slope_Square(
    collision: &mut EnemyBlockCollInfo,
    bts: u8,
    game: &mut Game,
) -> u8 {
    let index = usize::from(bts & 0x1f) * 4
        + usize::from((bts >> 6) ^ ((collision.ebci_r26 & 8) >> 2) as u8);
    let enemy_index = game.cur_enemy_index();
    let enemy = game.enemy_coordinates(enemy_index);
    let position = enemy.x_pos();
    let size = enemy.x_width();
    crate::verify_probe::hit("square-slope.enemy.vert");
    u8::from(square_slope_occupied(
        index as u16,
        collision.ebci_r28,
        collision.ebci_r30,
        position,
        size,
        1,
    ))
}
fn EnemyBlockCollHorizReact_Slope_NonSquare(
    collision: &mut EnemyBlockCollInfo,
    game: &mut Game,
) -> u8 {
    if collision.ebci_r32 & 0x8000 == 0 {
        return u8::from(collision.ebci_r32 & 0x4000 != 0);
    }
    let slope = usize::from(game.current_slope_bts() & 0x1f);
    let multiplier = i32::from(game.read_rom_word(0xa0c49f, (slope * 2 + 1) * 2));
    let displacement = collision.ebci_r18_r20 >> 8;
    // Negative movement truncates the shifted displacement to a signed word.
    // Keep this asymmetry: positive movement retains all 24 shifted bits.
    let displacement = if collision.ebci_r18_r20 < 0 {
        i32::from(displacement as i16)
    } else {
        displacement
    };
    collision.ebci_r18_r20 = displacement.wrapping_mul(multiplier);
    0
}
fn EnemyBlockCollVertReact_Slope_NonSquare(
    collision: &mut EnemyBlockCollInfo,
    game: &mut Game,
) -> u8 {
    let block = game.cur_block_index();
    // The SNES modulus uses only the low byte of room width and returns the
    // dividend unchanged when that byte is zero.
    let width = u16::from(game.room_width_in_blocks() as u8);
    let block_column = if width == 0 { block } else { block % width };
    let enemy_index = game.cur_enemy_index();
    let enemy = game.enemy_coordinates(enemy_index);
    let x_pos = enemy.x_pos();
    let height = enemy.y_height();
    if x_pos >> 4 != block_column {
        return 0;
    }
    let moving_up = collision.ebci_r18_r20 < 0;
    let bts = game.current_block_bts();
    if (bts & 0x80 != 0) != moving_up {
        return 0;
    }
    let edge = if moving_up {
        (collision.ebci_r24.wrapping_sub(height) & 0xf) ^ 0xf
    } else {
        height.wrapping_add(collision.ebci_r24).wrapping_sub(1) & 0xf
    };
    let x_in_block = (x_pos ^ if bts & 0x40 != 0 { 0xf } else { 0 }) & 0xf;
    let table_offset = usize::from(bts & 0x1f) * 16 + usize::from(x_in_block);
    let surface = game.read_rom_byte(0x948b2b, table_offset) & 0x1f;
    let correction = i16::from(surface) - edge as i16 - 1;
    // Downward movement accepts both the original equality case and negative
    // corrections; upward movement accepts nonpositive corrections directly.
    if correction > 0 {
        return 0;
    }
    let position = if moving_up {
        collision.ebci_r24.wrapping_sub(correction as u16)
    } else {
        collision.ebci_r24.wrapping_add(correction as u16)
    };
    let mut enemy = game.enemy_coordinates(enemy_index);
    enemy.set_y_pos(position);
    enemy.set_y_subpos(if moving_up { 0 } else { u16::MAX });
    1
}
fn EnemyBlockCollReact_HorizExt(_collision: &mut EnemyBlockCollInfo, game: &mut Game) -> u8 {
    let offset = game.current_block_bts() as i8 as i16;
    if offset == 0 {
        return 0;
    }
    game.set_cur_block_index(game.cur_block_index().wrapping_add(offset as u16));
    0xff
}

fn EnemyBlockCollReact_VertExt(_collision: &mut EnemyBlockCollInfo, game: &mut Game) -> u8 {
    let offset = game.current_block_bts() as i8 as i16;
    if offset == 0 {
        return 0;
    }
    let offset = (offset as u16).wrapping_mul(game.room_width_in_blocks());
    game.set_cur_block_index(game.cur_block_index().wrapping_add(offset));
    0xff
}
crate::forward_calls! { pub unsafe fn (k: u16, amt: i32, game: &mut Game) -> u8;
    Enemy_MoveRight_SlopesAsWalls => Enemy_MoveRight_IgnoreSlopes_Inner(k, amt, 0x4000, game);
    Enemy_MoveRight_ProcessSlopes => Enemy_MoveRight_IgnoreSlopes_Inner(k, amt, 0x8000, game);
    Enemy_MoveRight_IgnoreSlopes => Enemy_MoveRight_IgnoreSlopes_Inner(k, amt, 0, game);
}
unsafe fn Enemy_MoveRight_IgnoreSlopes_Inner(
    k: u16,
    amount32: i32,
    slope_mode: u16,
    game: &mut Game,
) -> u8 {
    if amount32 == 0 {
        return 0;
    }
    let top = game
        .enemy_coordinates(k)
        .y_pos()
        .wrapping_sub(game.enemy_coordinates(k).y_height());
    let rows = game
        .enemy_coordinates(k)
        .y_height()
        .wrapping_add(game.enemy_coordinates(k).y_pos())
        .wrapping_sub(1)
        .wrapping_sub(top & 0xfff0)
        >> 4;
    let row_offset = u16::from((top >> 4) as u8) * u16::from(game.room_width_in_blocks() as u8);
    let new_pos = (amount32 as u32).wrapping_add(
        (u32::from(game.enemy_coordinates(k).x_pos()) << 16)
            | u32::from(game.enemy_coordinates(k).x_subpos()),
    );
    let new_x = (new_pos >> 16) as u16;
    let moving_left = amount32 < 0;
    let edge = if moving_left {
        new_x.wrapping_sub(game.enemy_coordinates(k).x_width())
    } else {
        game.enemy_coordinates(k)
            .x_width()
            .wrapping_add(new_x)
            .wrapping_sub(1)
    };
    let mut collision = EnemyBlockCollInfo {
        ebci_r18_r20: amount32,
        ebci_r24: new_x,
        ebci_r26: edge,
        ebci_r28: rows,
        ebci_r30: rows,
        ebci_r32: slope_mode,
    };
    let mut block_offset = row_offset.wrapping_add(edge >> 4).wrapping_mul(2);
    while EnemyBlockCollReact_Horiz(&mut collision, block_offset, game) & 1 == 0 {
        // The callback can change room state; retain the original live read.
        block_offset = block_offset.wrapping_add(game.room_width_in_blocks().wrapping_mul(2));
        collision.ebci_r28 = collision.ebci_r28.wrapping_sub(1);
        if collision.ebci_r28 & 0x8000 != 0 {
            // Apply the callback-adjusted amount to the enemy's live coordinates.
            let position = (u32::from(game.enemy_coordinates(k).x_pos()) << 16)
                | u32::from(game.enemy_coordinates(k).x_subpos());
            let position = position.wrapping_add(collision.ebci_r18_r20 as u32);
            game.enemy_coordinates(k).set_x_subpos(position as u16);
            game.enemy_coordinates(k).set_x_pos((position >> 16) as u16);
            return 0;
        }
    }
    if moving_left {
        let boundary = game
            .enemy_coordinates(k)
            .x_width()
            .wrapping_add(1)
            .wrapping_add(collision.ebci_r26 | 0xf);
        if boundary <= game.enemy_coordinates(k).x_pos() {
            game.enemy_coordinates(k).set_x_pos(boundary);
        }
        game.enemy_coordinates(k).set_x_subpos(0);
    } else {
        let boundary =
            (collision.ebci_r26 & 0xfff0).wrapping_sub(game.enemy_coordinates(k).x_width());
        if boundary >= game.enemy_coordinates(k).x_pos() {
            game.enemy_coordinates(k).set_x_pos(boundary);
        }
        game.enemy_coordinates(k).set_x_subpos(u16::MAX);
    }
    1
}

pub unsafe fn Enemy_MoveDown(k: u16, amount32: i32, game: &mut Game) -> u8 {
    if amount32 == 0 {
        return 0;
    }
    let left = game
        .enemy_coordinates(k)
        .x_pos()
        .wrapping_sub(game.enemy_coordinates(k).x_width());
    let columns = game
        .enemy_coordinates(k)
        .x_width()
        .wrapping_add(game.enemy_coordinates(k).x_pos())
        .wrapping_sub(1)
        .wrapping_sub(left & 0xfff0)
        >> 4;
    let new_pos = (amount32 as u32).wrapping_add(
        (u32::from(game.enemy_coordinates(k).y_pos()) << 16)
            | u32::from(game.enemy_coordinates(k).y_subpos()),
    );
    let new_y = (new_pos >> 16) as u16;
    let moving_up = amount32 < 0;
    let edge = if moving_up {
        new_y.wrapping_sub(game.enemy_coordinates(k).y_height())
    } else {
        game.enemy_coordinates(k)
            .y_height()
            .wrapping_add(new_y)
            .wrapping_sub(1)
    };
    let row_offset = u16::from((edge >> 4) as u8) * u16::from(game.room_width_in_blocks() as u8);
    let mut collision = EnemyBlockCollInfo {
        ebci_r18_r20: amount32,
        ebci_r24: new_y,
        ebci_r26: edge,
        ebci_r28: columns,
        ebci_r30: columns,
        ebci_r32: 0,
    };
    let mut block_offset = row_offset.wrapping_add(left >> 4).wrapping_mul(2);
    while EnemyBlockCollReact_Vert(&mut collision, block_offset, game) & 1 == 0 {
        collision.ebci_r28 = collision.ebci_r28.wrapping_sub(1);
        if collision.ebci_r28 & 0x8000 != 0 {
            // Vertical movement stores the precomputed position, unlike horizontal.
            game.enemy_coordinates(k).set_y_subpos(new_pos as u16);
            game.enemy_coordinates(k).set_y_pos(new_y);
            return 0;
        }
        block_offset = block_offset.wrapping_add(2);
    }
    if moving_up {
        let boundary = game
            .enemy_coordinates(k)
            .y_height()
            .wrapping_add(1)
            .wrapping_add(collision.ebci_r26 | 0xf);
        if boundary <= game.enemy_coordinates(k).y_pos() {
            game.enemy_coordinates(k).set_y_pos(boundary);
        }
        game.enemy_coordinates(k).set_y_subpos(0);
    } else {
        let boundary =
            (collision.ebci_r26 & 0xfff0).wrapping_sub(game.enemy_coordinates(k).y_height());
        if boundary >= game.enemy_coordinates(k).y_pos() {
            game.enemy_coordinates(k).set_y_pos(boundary);
        }
        game.enemy_coordinates(k).set_y_subpos(u16::MAX);
    }
    1
}
unsafe fn EnemyBlockCollReact_Horiz(
    collision: &mut EnemyBlockCollInfo,
    block_offset: u16,
    game: &mut Game,
) -> u8 {
    game.set_cur_block_index(block_offset >> 1);
    game.react_to_block(|block_type, game| match block_type {
        0 | 2 | 3 | 4 | 6 | 7 => 0,
        1 => EnemyBlockCollHorizReact_Slope(collision, game),
        5 => EnemyBlockCollReact_HorizExt(collision, game),
        10 => EnemyBlockCollReact_Spike(collision, game),
        13 => EnemyBlockCollReact_VertExt(collision, game),
        _ => 1,
    })
}
unsafe fn EnemyBlockCollReact_Vert(
    collision: &mut EnemyBlockCollInfo,
    block_offset: u16,
    game: &mut Game,
) -> u8 {
    game.set_cur_block_index(block_offset >> 1);
    game.react_to_block(|block_type, game| match block_type {
        0 | 2 | 3 | 4 | 6 | 7 => 0,
        1 => EnemyBlockCollVertReact_Slope(collision, game),
        5 => EnemyBlockCollReact_HorizExt(collision, game),
        10 => EnemyBlockCollReact_Spike(collision, game),
        13 => EnemyBlockCollReact_VertExt(collision, game),
        _ => 1,
    })
}
pub unsafe fn CalculateBlockContainingPixelPos(xpos: u16, ypos: u16, game: &mut Game) {
    let prod: u16 = Mult8x8((ypos >> 4) as u8, word!(game.room_width_in_blocks) as u8);
    word!(game.cur_block_index = (prod as i32 + (xpos as i32 >> 4)) as u16);
}
pub unsafe fn EnemyFunc_C8AD(k: u16, game: &mut Game) -> u8 {
    let mut result: u8 = 0;
    let E = gEnemyData(k as i32, game);
    CalculateBlockContainingPixelPos(
        (*E).x_pos,
        ((*E).y_pos as i32 + (*E).y_height as i32 - 1i32) as u16,
        game,
    );
    if (word!(game.level_data[word!(game.cur_block_index)]) & 0xf000) == 4096
        && (*ram_ptr!(game; 0x16402, word!(game.cur_block_index) as isize) & 0x1f) >= 5
    {
        result = 1;
        let temp_collision_DD4: u16 =
            ((*E).y_height as i32 + (*E).y_pos as i32 - 1i32 & 0xf) as u16;
        let temp_collision_DD6: u16 = (16
            * (*ram_ptr!(game; 0x16402, word!(game.cur_block_index) as isize) as i32 & 0x1f))
            as u16;
        if (*ram_ptr!(game; 0x16402, word!(game.cur_block_index) as isize) & 0x80) == 0 {
            let j: u16 =
                (if (*ram_ptr!(game; 0x16402, word!(game.cur_block_index) as isize) & 0x40) != 0 {
                    (*E).x_pos as i32 ^ 0xf
                } else {
                    (*E).x_pos as i32
                }) as u16;
            let v4: i16 = ((byte!(@pointer rom_ptr!(game; 0x948b2b); (temp_collision_DD6 as i32 + (j as i32 & 0xf)) as isize)
                as i32
                & 0x1f)
                - temp_collision_DD4 as i32
                - 1i32) as i16;
            if v4 < 0 {
                (*E).y_pos = ((*E).y_pos).wrapping_add((v4) as u16);
            }
        }
    }
    CalculateBlockContainingPixelPos((*E).x_pos, ((*E).y_pos).wrapping_sub((*E).y_height), game);
    if (word!(game.level_data[word!(game.cur_block_index)]) & 0xf000) == 4096
        && (*ram_ptr!(game; 0x16402, word!(game.cur_block_index) as isize) & 0x1f) >= 5
    {
        result = 1;
        let temp_collision_DD4_0: u16 =
            ((*E).y_pos as i32 - (*E).y_height as i32 & 0xf ^ 0xf) as u16;
        let temp_collision_DD6_0: u16 = (16
            * (*ram_ptr!(game; 0x16402, word!(game.cur_block_index) as isize) as i32 & 0x1f))
            as u16;
        if *ram_ptr!(game; 0x16402, word!(game.cur_block_index) as isize) & 0x80 != 0 {
            let j_0: u16 =
                (if (*ram_ptr!(game; 0x16402, word!(game.cur_block_index) as isize) & 0x40) != 0 {
                    (*E).x_pos as i32 ^ 0xf
                } else {
                    (*E).x_pos as i32
                }) as u16;
            let v7: i16 = ((byte!(@pointer rom_ptr!(game; 0x948b2b); (temp_collision_DD6_0 as i32 + (j_0 as i32 & 0xf)) as isize)
                as i32
                & 0x1f)
                - temp_collision_DD4_0 as i32
                - 1i32) as i16;
            if v7 < 0 {
                (*E).y_pos = ((*E).y_pos).wrapping_sub((v7) as u16);
            }
        }
    }
    result
}
