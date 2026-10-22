use crate::game::enemy_scheduling::{shake_active_enemies, update_frozen_enemy};
use crate::game::enemy_population_ram::initialize_room;
pub use crate::game::prelude::*;

pub use crate::game::constants::bank_60::*;
pub use crate::game::constants::bank_86::*;
pub use crate::game::constants::bank_a0::*;
pub use crate::game::constants::bank_a4::*;
pub use crate::game::constants::bank_b4::*;

use crate::game::moving_platforms::{self as mechanisms, Event, Kind};
use crate::game::sm_80::{
    NextRandom, QueueSfx1_Max3, QueueSfx1_Max6, QueueSfx2_Max1, QueueSfx2_Max3, QueueSfx3_Max3,
};
use crate::game::plm_spawning::SpawnPLM;
use crate::game::sm_86::{SpawnEprojWithGfx};
use crate::game::sm_90::CallSomeSamusCode;
use crate::game::sm_91::Samus_DealDamage;
use crate::game::sm_94::square_slope_occupied;
use crate::game::sprite_objects::{
    create_sprite,
};
use crate::game::{sm_a2, sm_a3, sm_a4, sm_a5, sm_a6, sm_a7, sm_a8, sm_a9, sm_aa, sm_b2, sm_b3};

pub use crate::types::EnemyAiBits;
crate::game::constants::typed_constants! { EnemyAiBits;
    kEnemyAiBits_Frozen = 4, kEnemyAiBits_Hurt = 2,
    kEnemyAiBits_Grapple = 1,
}

crate::typed_struct! {
    [
        #[derive(Copy, Clone)]
        #[repr(C)]
    ];
    pub struct PositionAndWidth {
        pub x_pos, x_subpos, y_pos, y_subpos, width, height: u16;
    }

    pub struct EnemyBlockCollInfo {
        pub ebci_r18_r20: i32;
        pub ebci_r24, ebci_r26, ebci_r28, ebci_r30, ebci_r32: u16;
    }
}

pub type Func_EnemyBlockCollInfo_U8 = unsafe fn(&mut EnemyBlockCollInfo, &mut Game) -> u8;

pub const NULL: *mut c_void = null_mut::<c_void>();
const RomPtr_86: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x86>;
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
        word!(game.earthquake_timer -= 1);
        if word!(game.earthquake_type) as i32 - 18i32 & 0x8000 == 0 {
            shake_active_enemies(game);
        }
    }
    word!(game.frame_counter_every_frame += 1);
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
            initialize_room(game);
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
        10649676 => [()];
        10656319 => sm_a2::ThinHoppingBlobs_Init;
        10656381 => sm_a2::ThinHoppingBlobs_Main;
        10664233 => [{
                    sm_a2::Rinka_Frozen(word!(game.cur_enemy_index), game);
                }];
        10664275 => [{
                    sm_a2::Rinka_Powerbomb(word!(game.cur_enemy_index), game);
                }];
        10669268 => sm_a2::MaridiaLargeSnail_Init;
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
        10715212 => [()]; 10716909 => sm_a3::Waver_Init;
        10717004 => sm_a3::Waver_Main; 10717536 => sm_a3::Metalee_Init;
        10717561 => sm_a3::Metalee_Main; 10717967 => sm_a3::Metalee_Shot;
        10718827 => [{
                    sm_a3::Fireflea_Touch(word!(game.cur_enemy_index), game);
                }];
        10719413 => sm_a3::MaridiaFish_Init; 10719531 => sm_a3::MaridiaFish_Main;
        10723079 => [()];
        10723661 => sm_a3::Roach_Init;
        10724048 => sm_a3::Roach_Main;
        10726351 => sm_a3::Sidehopper_Main;
        10733230 => sm_a3::Skree_Init; 10733255 => sm_a3::Skree_Main;
        10738715 => [()];
        10743874 => [{
                    sm_a3::Metroid_Powerbomb(word!(game.cur_enemy_index), game);
                }];
        10780748 => [()];
        10795344 => [()];
        10811067 => [sm_a4::CrocomireTongue_Main()];
        10846284 => [()];
        10851824 => sm_a5::Draygon_Shot;
        10864042 => [()];
        10864068 => [()];
        10874354 => [()];
        10911820 => [()];
        10913525 => sm_a6::Boulder_Init; 10913683 => sm_a6::Boulder_Main;
        10914607 => sm_a6::SpikeyPlatform_Init;
        10914733 => sm_a6::SpikeyPlatform_Main;
        10920181 => sm_a6::CeresRidley_Init; 10920584 => sm_a6::CeresRidley_Main;
        10924583 => sm_a6::Ridley_Main;
        10924682 => sm_a6::Ridley_Func_2; 10924695 => sm_a6::Ridley_Hurt;
        10942752 => [()];
        10977356 => [()]; 10982543 => [()];
        10982581 => [()];
        11001242 => [()];
        11001873 => [()];
        11042892 => [()];
        11048675 => sm_a8::Fune_Init;
        11048752 => sm_a8::Fune_Main;
        11051336 => sm_a8::YappingMaw_Init;
        11051537 => sm_a8::YappingMaw_Main; 11052953 => sm_a8::YappingMaw_Touch;
        11057014 => sm_a8::Beetom_Init;
        11057165 => sm_a8::Beetom_Main; 11058734 => sm_a8::Beetom_Touch;
        11062374 => [()];
        11065545 => sm_a8::MaridiaPuffer_Init;
        11065611 => sm_a8::MaridiaPuffer_Main; 11066132 => sm_a8::MaridiaPuffer_Shot;
        11068296 => sm_a8::WreckedShipOrbs_Init;
        11071880 => sm_a8::KiHunter_Init;
        11072020 => sm_a8::KiHunterWings_Init; 11072092 => sm_a8::KiHunter_Main;
        11072098 => sm_a8::KiHunterWings_Main;
        11108428 => [()];
        11111301 => [{
                    sm_a9::MotherBrainsTubesFalling_Main(word!(game.cur_enemy_index), game);
                }];
        11121927 => sm_a9::MotherBrainsBrain_Shot;
        11122117 => [()];
        11173964 => [()];
        11196359 => [()];
        11200475 => [()];
        11200476 => [()];
        11698252 => [()];
        11763788 => [()];
        11765518 => [sm_b3::UnusedSpinningTurtleEye_Main()];
        11767628 => sm_b3::BrinstarYellowPipeBug_Init;
        11767726 => sm_b3::BrinstarYellowPipeBug_Main; 11769219 => sm_b3::Botwoon_Init;
        11771926 => sm_b3::Botwoon_Shot;
        11791130 => [()];
        0xa20000..=0xa2ffff => [sm_a2::DispatchEnemyAi(ea, game)];
        0xa30000..=0xa3ffff => [sm_a3::DispatchEnemyAi(ea, game)];
        0xa40000..=0xa4ffff => [sm_a4::DispatchEnemyAi(ea, game)];
        0xa50000..=0xa5ffff => [sm_a5::DispatchEnemyAi(ea, game)];
        0xa60000..=0xa6ffff => [sm_a6::DispatchEnemyAi(ea, game)];
        0xa70000..=0xa7ffff => [sm_a7::DispatchEnemyAi(ea, game)];
        0xa80000..=0xa8ffff => [sm_a8::DispatchEnemyAi(ea, game)];
        0xa90000..=0xa9ffff => [sm_a9::DispatchEnemyAi(ea, game)];
        0xaa0000..=0xaaffff => [sm_aa::DispatchEnemyAi(ea, game)];
        0xb20000..=0xb2ffff => [sm_b2::DispatchEnemyAi(ea, game)];
        0xb30000..=0xb3ffff => [sm_b3::DispatchEnemyAi(ea, game)];
    }
}
crate::game::address_dispatch! {
    pub unsafe fn CallEnemyPreInstr(ea: u32, game: &mut Game) => ea;
        setup { let k: u16 = word!(game.cur_enemy_index); }
        call (k, game); default { Unreachable(game); };
    {
        10518651 => [()]; 10649723 => [()];
        10652277 => [()];
        10657819 => [()]; 10659799 => [()];
        10676097 => [()]; 10730689 => [sm_a3::Bang_Func_7(k)];
        10735455 => [()];
        10739850 => [()]; 10741441 => [()];
        10998952 => [()];
        11047388 => [()];
        11055761 => [sm_a8::sub_A8B291(k)]; 11055765 => [sm_a8::NorfairLavaMan_Func_13(game)];
        11060584 => [()];
        11192670 => [()]; 11197610 => [()]; 11200422 => [()]; 11174011 => [()]; 11763835 => [()];
        11192146 => sm_aa::Torizo_Func_5;
        11192364 => sm_aa::Torizo_Func_7;
        11195889 => sm_aa::Torizo_D5F1;
        11200474 => [()];
        11197611 => [()];
        0xa20000..=0xa2ffff => [sm_a2::DispatchEnemyPreInstr(ea, k, game)];
        0xa30000..=0xa3ffff => [sm_a3::DispatchEnemyPreInstr(ea, k, game)];
        0xa70000..=0xa7ffff => [sm_a7::DispatchEnemyPreInstr(ea, k, game)];
        0xa80000..=0xa8ffff => [sm_a8::DispatchEnemyPreInstr(ea, k, game)];
        0xaa0000..=0xaaffff => [sm_aa::DispatchEnemyPreInstr(ea, k, game)];
        0xb30000..=0xb3ffff => [sm_b3::DispatchEnemyPreInstr(ea, k, game)];
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
        10715373 => [EnemyInstr_Goto(k, j)];
        10716899 => sm_a3::Waver_Instr_1;
        10780909 => [EnemyInstr_Goto(k, j)];
        10846445 => [EnemyInstr_Goto(k, j)];
        10911981 => [EnemyInstr_Goto(k, j)];
        10977517 => [EnemyInstr_Goto(k, j)];
        10991155 => [sm_a7::Kraid_Instr_1(k, j)];
        11043053 => [EnemyInstr_Goto(k, j)];
        11056990 => [sm_a8::Beetom_Instr_1(k, j)];
        11062987 => [sm_a8::WreckedShipRobot_Instr_7(k, j)];
        11063490 => [sm_a8::WreckedShipRobot_Instr_10(k, j)];
        11128756 => [sm_a9::Shitroid_Instr_1(k, j)];
        11128778 => [sm_a9::Shitroid_Instr_2(k, j)];
        11139360 => [sm_a9::Shitroid_Instr_3(k, j)]; 11139382 => [sm_a9::Shitroid_Instr_4(k, j)];
        11139472 => [sm_a9::Shitroid_Instr_6(k, j)];
        11174125 => [EnemyInstr_Goto(k, j)];
        11190984 => [sm_aa::Torizo_Instr_7(k, j)];
        11196831 => [sm_aa::Shaktool_Instr_6(k, j)];
        11698413 => [EnemyInstr_Goto(k, j)];
        11763949 => [EnemyInstr_Goto(k, j)];
        0xa20000..=0xa2ffff => [sm_a2::DispatchEnemyInstr(ea, k, j, game)];
        0xa30000..=0xa3ffff => [sm_a3::DispatchEnemyInstr(ea, k, j, game)];
        0xa40000..=0xa4ffff => [sm_a4::DispatchEnemyInstr(ea, k, j, game)];
        0xa50000..=0xa5ffff => [sm_a5::DispatchEnemyInstr(ea, k, j, game)];
        0xa60000..=0xa6ffff => [sm_a6::DispatchEnemyInstr(ea, k, j, game)];
        0xa70000..=0xa7ffff => [sm_a7::DispatchEnemyInstr(ea, k, j, game)];
        0xa80000..=0xa8ffff => [sm_a8::DispatchEnemyInstr(ea, k, j, game)];
        0xa90000..=0xa9ffff => [sm_a9::DispatchEnemyInstr(ea, k, j, game)];
        0xaa0000..=0xaaffff => [sm_aa::DispatchEnemyInstr(ea, k, j, game)];
        0xb20000..=0xb2ffff => [sm_b2::DispatchEnemyInstr(ea, k, j, game)];
        0xb30000..=0xb3ffff => [sm_b3::DispatchEnemyInstr(ea, k, j, game)];
    }
}
pub unsafe fn DecrementSamusTimers(game: &mut Game) {
    if word!(game.samus_invincibility_timer) != 0 {
        word!(game.samus_invincibility_timer -= 1);
    }
    if word!(game.samus_knockback_timer) != 0 {
        word!(game.samus_knockback_timer -= 1);
    }
    if word!(game.projectile_invincibility_timer) != 0 {
        word!(game.projectile_invincibility_timer -= 1);
    }
    word!(game.interactive_enemy_indexes[0] = -1i32 as u16);
    word!(game.active_enemy_indexes[0] = -1i32 as u16);
}
pub unsafe fn SpawnEnemyDrops(a: u16, k: u16, varE20: u16, game: &mut Game) {
    game.eproj_spawn_varE24 = a;
    SpawnEprojWithGfx(varE20, k, addr_kEproj_Pickup as u16, game);
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
                word!(game.projectile_dir[pidx] |= (0x10u16));
                Samus_DealDamage(
                    SuitDamageDivision(word!(game.projectile_damage[pidx]), game),
                    game,
                );
                word! {game; samus_invincibility_timer = 96; samus_knockback_timer = 5; }
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
    word! {game; samus_invincibility_timer = 96; samus_knockback_timer = 5; }
    let v1: u16 =
        *(RomPtr_86(*(ram_ptr!(game, u16; 0x1997, k as i32 as isize)), game) as *mut u16).offset(5);
    if v1 != 0 {
        let v2: i32 = k as i32 >> 1;
        word! {game; eproj_instr_list_ptr[v2] = v1; eproj_instr_timers[v2] = 1; }
    }
    let v3: i32 = k as i32 >> 1;
    if (word!(game.eproj_properties[v3]) & 0x4000) == 0 {
        *(ram_ptr!(game, u16; 0x1997, k as i32 as isize)) = 0;
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
        word!(game.projectile_dir[i] |= (0x10u16));
    }
    if word!(game.eproj_flags[(k >> 1)]) == 1 {
        let v4: i32 = k as i32 >> 1;
        create_sprite(
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
        word!(game.eproj_properties[j_0] &= (0xfffu16));
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
    Enemy_NormalFrozenAI => update_frozen_enemy(game);
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
    word!(game.num_enemies_killed_in_room += 1);
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
                                word! {game; earthquake_timer = 30; earthquake_type = 18; }
                            }
                            if ((*E).properties & 0x1000) != 0
                                || (word!(game.projectile_type[pidx]) & 8) == 0
                            {
                                word!(game.projectile_dir[pidx] |= (0x10u16));
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
                                word!(game.projectile_dir[pidx] |= (0x10u16));
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
        word! {game; grapple_beam_end_x_pos = (*E).x_pos; grapple_beam_end_y_pos = (*E).y_pos; }
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
        word! {game; grapple_beam_end_x_pos = (*E).x_pos; grapple_beam_end_y_pos = (*E).y_pos; }
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
                        word! {game; earthquake_timer = 30; earthquake_type = 18; }
                    }
                    if ((*E).properties & 0x1000) != 0 || (word!(game.projectile_type[pidx]) & 8) == 0 {
                        word!(game.projectile_dir[pidx] |= (0x10u16));
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
                word!(game.projectile_dir[pidx] |= (0x10u16));
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
        create_sprite((*E).x_pos, (*E).y_pos, 55, 0, game);
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
        create_sprite((*ED).x_pos, (*ED).y_pos, 55, 0, game);
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
    word!(game.projectile_dir[v7] |= (0x10u16));
    create_sprite(
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
    create_sprite(
        word!(game.projectile_x_pos[v0]),
        word!(game.projectile_y_pos[v0]),
        6,
        0,
        game,
    );
    QueueSfx1_Max3(0x3d, game);
    word!(game.projectile_dir[word!(game.collision_detection_index)] |= (0x10u16));
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
