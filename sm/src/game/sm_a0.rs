use crate::game::combat_rules::LatchMode;
pub use crate::game::prelude::*;
use crate::game::{
    actors::{api as actors, scheduling as actor_scheduling},
    session_runtime::Runtime,
};

pub use crate::game::constants::bank_60::*;
pub use crate::game::constants::bank_86::*;
pub use crate::game::constants::bank_a0::*;
pub use crate::game::constants::bank_a4::*;
pub use crate::game::constants::bank_b4::*;

use crate::game::moving_platforms::{self as mechanisms, Event, Kind};
use crate::game::{sm_a2, sm_a3, sm_a4, sm_a5, sm_a6, sm_a7, sm_a8, sm_a9, sm_aa, sm_b2, sm_b3};

pub use crate::types::EnemyAiBits;
crate::game::constants::typed_constants! { EnemyAiBits;
    kEnemyAiBits_Frozen = 4, kEnemyAiBits_Hurt = 2,
    kEnemyAiBits_Grapple = 1,
}

pub const NULL: *mut c_void = null_mut::<c_void>();

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
    unsafe {
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
                actor_scheduling::shake_active_enemies(&mut Runtime::new(game));
            }
        }
        word!(game.frame_counter_every_frame += 1);
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

crate::game::callback_dispatch! {
    pub unsafe fn CallEnemyAi(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    @external {
        10715136 | 10846208 | 10911744 | 10977280 | 11042816 | 11763712 => [actors::resume_ai(&mut Runtime::new(game))];
        10649605 | 10780677 => [actors::latch(&mut Runtime::new(game), LatchMode::Passive)];
        11042836 => [actors::latch(&mut Runtime::new(game), LatchMode::Active)];
        10518583 | 10649655 | 10715191 | 10780727 | 10846263 | 10911799 | 10977335 | 11042871 | 11108407 | 11173943 | 11698231 | 11763767 => [actors::power_bomb_reaction(&mut Runtime::new(game), actors::DeathHandling::Finalize)];
        10649676 => [{
            #[cfg(sm_verify_dll)]
            crate::game::actors::reference_population_animation::dispatch_noop(game);
        }];
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
pub unsafe fn DecrementSamusTimers(game: &mut Game) {
    unsafe {
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
}

crate::game::address_dispatch! {
    pub unsafe fn CallHitboxTouch(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        10518563 => [actors::contact_reaction(&mut Runtime::new(game), actors::DeathHandling::Finalize)]; 10670984 => sm_a2::MaridiaLargeSnail_Func_12;
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
    Enemy_NormalFrozenAI => actor_scheduling::update_frozen_enemy(&mut Runtime::new(game));
    @external {
        10518573 => [actors::shot_reaction(&mut Runtime::new(game), actors::ShotHandling::Finalize)];
        10846278 => [actors::create_dud(&mut Runtime::new(game))];
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

pub fn SignExtend8(a: u16) -> u16 {
    // Preserve the input high byte when bit 7 is clear.
    if a & 0x80 != 0 { a | 0xff00 } else { a }
}
pub fn Abs16(a: u16) -> u16 {
    (a as i16).wrapping_abs() as u16
}
fn SineMult8bitInner(amplitude: u16, phase: u16, game: &mut Game) -> u32 {
    crate::game::actors::queries::sine_product(&mut Runtime::new(game), amplitude, phase)
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
pub unsafe fn EnemyFunc_Divide(a: u32, b: u32) -> u32 {
    if a != 0 { b.wrapping_div(a) } else { 0 }
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
