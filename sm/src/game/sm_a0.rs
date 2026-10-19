pub use crate::game::constants::bank_60::*;
pub use crate::game::constants::bank_86::*;
pub use crate::game::constants::bank_a0::*;
pub use crate::game::constants::bank_a4::*;
pub use crate::game::constants::bank_b4::*;
pub use crate::game::constants::enemy_properties::*;
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
use crate::game::sm_a2::{
    BouncingGoofball_Init, BouncingGoofball_Instr_88C5, BouncingGoofball_Instr_88C6,
    BouncingGoofball_Main, EnemyInstr_Rio_Instr_1, Enemy_GrappleReact_CancelBeam_A2,
    Enemy_GrappleReact_KillEnemy_A2, Enemy_GrappleReact_SamusLatchesOn_A2, Enemy_NormalFrozenAI_A2,
    Enemy_NormalShotAI_A2, Enemy_NormalTouchAI_A2, Flies_4, Flies_5, Flies_6, Flies_7, Flies_Init,
    Flies_Main, Gripper_Init, Gripper_Main, GunshipBottom_Init, GunshipTop_10, GunshipTop_11,
    GunshipTop_12, GunshipTop_13, GunshipTop_14, GunshipTop_15, GunshipTop_16, GunshipTop_17,
    GunshipTop_18, GunshipTop_19, GunshipTop_20, GunshipTop_21, GunshipTop_3, GunshipTop_4,
    GunshipTop_5, GunshipTop_6, GunshipTop_7, GunshipTop_8, GunshipTop_9, GunshipTop_Init,
    GunshipTop_Main, HirisingSlowfalling_Init, HirisingSlowfalling_Main,
    HorizontalShootableShutter_Init, HorizontalShootableShutter_Main,
    HorizontalShootableShutter_Powerbomb, HorizontalShootableShutter_Shot,
    HorizontalShootableShutter_Touch, JetPowerRipper_Init, JetPowerRipper_Main,
    JetPowerRipper_Shot, LavaSeahorse_Func_1, LavaSeahorse_Func_2, LavaSeahorse_Func_3,
    LavaSeahorse_Func_4, LavaSeahorse_Func_5, LavaSeahorse_Init, LavaSeahorse_Instr_E5FB,
    LavaSeahorse_Main, LavaSeahorse_Powerbomb, LavaSeahorse_Shot, LavaSeahorse_Touch,
    LavaquakeRocks_Init, LavaquakeRocks_Main, LowerNorfairRio_Func_1, LowerNorfairRio_Func_2,
    LowerNorfairRio_Func_3, LowerNorfairRio_Func_4, LowerNorfairRio_Func_5, LowerNorfairRio_Func_6,
    LowerNorfairRio_Init, LowerNorfairRio_Instr_C6D2, LowerNorfairRio_Instr_C6DD,
    LowerNorfairRio_Instr_C6E8, LowerNorfairRio_Main, MaridiaBeybladeTurtle_Init,
    MaridiaBeybladeTurtle_Instr_9381, MaridiaBeybladeTurtle_Instr_9412,
    MaridiaBeybladeTurtle_Instr_9447, MaridiaBeybladeTurtle_Instr_9451,
    MaridiaBeybladeTurtle_Instr_946B, MaridiaBeybladeTurtle_Instr_9485,
    MaridiaBeybladeTurtle_Instr_94A1, MaridiaBeybladeTurtle_Instr_94C7,
    MaridiaBeybladeTurtle_Instr_94D1, MaridiaBeybladeTurtle_Main, MaridiaBeybladeTurtle_Touch,
    MaridiaLargeSnail_Func_12, MaridiaLargeSnail_Func_7, MaridiaLargeSnail_Func_8,
    MaridiaLargeSnail_Init, MaridiaLargeSnail_Instr_CB6B, MaridiaLargeSnail_Instr_CCB3,
    MaridiaLargeSnail_Instr_CCBE, MaridiaLargeSnail_Instr_CCC9, MaridiaLargeSnail_Main,
    MaridiaLargeSnail_Shot, MaridiaLargeSnail_Touch, MaridiaSpikeyShell_Init,
    MaridiaSpikeyShell_Instr_A56D, MaridiaSpikeyShell_Instr_A571, MaridiaSpikeyShell_Main,
    MaridiaSpikeyShell_Shot, MiniCrocomire_Init, MiniCrocomire_Instr_897E,
    MiniCrocomire_Instr_8990, MiniCrocomire_Instr_899D, MiniCrocomire_Main,
    MiniCrocomire_PreInstr5, MiniCrocomire_PreInstr6, MiniMaridiaBeybladeTurtle_Init,
    MiniMaridiaBeybladeTurtle_Main, MiniMaridiaBeybladeTurtle_Shot,
    MiniMaridiaBeybladeTurtle_Touch, NorfairErraticFireball_Init, NorfairErraticFireball_Main,
    NorfairLavajumpingEnemy_Func_1, NorfairLavajumpingEnemy_Func_2, NorfairLavajumpingEnemy_Func_3,
    NorfairLavajumpingEnemy_Func_4, NorfairLavajumpingEnemy_Func_5, NorfairLavajumpingEnemy_Init,
    NorfairLavajumpingEnemy_Instr_BE8E, NorfairLavajumpingEnemy_Main, NorfairRio_Func_1,
    NorfairRio_Func_2, NorfairRio_Func_3, NorfairRio_Func_4, NorfairRio_Func_5, NorfairRio_Func_6,
    NorfairRio_Init, NorfairRio_Instr_C1C9, NorfairRio_Instr_C1D4, NorfairRio_Instr_C1DF,
    NorfairRio_Instr_C1EA, NorfairRio_Instr_C1F5, NorfairRio_Instr_C200, NorfairRio_Instr_C20B,
    NorfairRio_Instr_C216, NorfairRio_Instr_C221, NorfairRio_Instr_C22C, NorfairRio_Instr_C237,
    NorfairRio_Main, Rinka_Frozen, Rinka_Init, Rinka_Instr_B9B3, Rinka_Instr_B9BD,
    Rinka_Instr_B9C7, Rinka_Main, Rinka_Powerbomb, Rinka_Shot, Rinka_Touch, Rio_Init, Rio_Main,
    Ripper_Init, Ripper_Main, RisingFallingPlatform_Init, RisingFallingPlatform_Main,
    RisingFallingPlatform_Powerbomb, RisingFallingPlatform_Shot, RisingFallingPlatform_Touch,
    ShootableShutter_Init, ShootableShutter_Shot, SpikeShootingPlant_2, SpikeShootingPlant_3,
    SpikeShootingPlant_Init, SpikeShootingPlant_Instr_9F2A, SpikeShootingPlant_Instr_A095,
    SpikeShootingPlant_Instr_A0A7, SpikeShootingPlant_Main, ThinHoppingBlobs_Init,
    ThinHoppingBlobs_Main, TimedShutter_Init, TimedShutter_Main,
};
use crate::game::sm_a3::{
    sub_A3E168, Bang_Func_10, Bang_Func_11, Bang_Func_6, Bang_Func_7, Bang_Func_8, Bang_Init,
    Bang_Instr_1, Bang_Instr_2, Bang_Main, Bang_Shot, BigEyeBugs_Init, Crab_Init, Elevator_Frozen,
    Elevator_Init, Enemy_GrappleReact_CancelBeam_A3, Enemy_GrappleReact_KillEnemy_A3,
    Enemy_GrappleReact_NoInteract_A3, Enemy_NormalFrozenAI_A3, Enemy_NormalShotAI_A3,
    Enemy_NormalTouchAI_A3, FastMovingSlowSinkingPlatform_Init, FastMovingSlowSinkingPlatform_Shot,
    FireZoomer_Func_1, FireZoomer_Func_2, FireZoomer_Func_3, FireZoomer_Init, Fireflea_Init,
    Fireflea_Main, Fireflea_Powerbomb, Fireflea_Shot, Fireflea_Touch, MaridiaFish_Init,
    MaridiaFish_Instr_1, MaridiaFish_Instr_2, MaridiaFish_Instr_3, MaridiaFish_Main,
    MaridiaRefillCandy_Init, MaridiaRefillCandy_Instr_1, MaridiaRefillCandy_Instr_2,
    MaridiaRefillCandy_Instr_3, MaridiaRefillCandy_Main, MaridiaSnail_CFB7, MaridiaSnail_CFBD,
    MaridiaSnail_CFCE, MaridiaSnail_CFD4, MaridiaSnail_CFE5, MaridiaSnail_CFEB, MaridiaSnail_CFFC,
    MaridiaSnail_Func_15, MaridiaSnail_Func_7, MaridiaSnail_Func_9, MaridiaSnail_Init,
    MaridiaSnail_Instr_1, MaridiaSnail_Instr_2, MaridiaSnail_Instr_3, MaridiaSnail_Instr_4,
    MaridiaSnail_Instr_5, MaridiaSnail_Main, MaridiaSnail_Shot, MaridiaSnail_Touch, Metalee_Init,
    Metalee_Instr_1, Metalee_Main, Metalee_Shot, Metroid_Frozen, Metroid_Hurt, Metroid_Init,
    Metroid_Instr_1, Metroid_Instr_2, Metroid_Main, Metroid_Powerbomb, Metroid_Shot, Metroid_Touch,
    Mochtroid_Init, Mochtroid_Main, Mochtroid_Shot, Mochtroid_Touch, NorfairSlowFireball_Init,
    PlatformThatFallsWithSamus_Init, PlatformThatFallsWithSamus_Instr_1,
    PlatformThatFallsWithSamus_Instr_2, PlatformThatFallsWithSamus_Instr_3,
    PlatformThatFallsWithSamus_Instr_4, PlatformThatFallsWithSamus_Main, Reflec_Func_1,
    Reflec_Init, Reflec_Instr_1, Reflec_Shot, Roach_Init, Roach_Main, Sidehopper_Func_1,
    Sidehopper_Init, Sidehopper_Instr_1, Sidehopper_Main, Skree_Init, Skree_Instr_1, Skree_Main,
    Skree_Shot, Slug_Init, StoneZoomer_Init, StoneZoomer_Main, Waver_Init, Waver_Instr_1,
    Waver_Main, WreckedShipOrangeZoomer_Func_1, WreckedShipOrangeZoomer_Func_2,
    WreckedShipOrangeZoomer_Init, WreckedShipOrangeZoomer_Main, Zoomer_Instr_SetPreinstr,
};
use crate::game::sm_a4::{
    CrocomireTongue_Init, CrocomireTongue_Main, Crocomire_Func_1, Crocomire_Func_92,
    Crocomire_Func_93, Crocomire_Func_94, Crocomire_Func_95, Crocomire_Hurt, Crocomire_Init,
    Crocomire_Instr_1, Crocomire_Instr_10, Crocomire_Instr_11, Crocomire_Instr_12,
    Crocomire_Instr_13, Crocomire_Instr_14, Crocomire_Instr_15, Crocomire_Instr_16,
    Crocomire_Instr_17, Crocomire_Instr_18, Crocomire_Instr_19, Crocomire_Instr_2,
    Crocomire_Instr_20, Crocomire_Instr_21, Crocomire_Instr_22, Crocomire_Instr_23,
    Crocomire_Instr_24, Crocomire_Instr_25, Crocomire_Instr_26, Crocomire_Instr_27,
    Crocomire_Instr_3, Crocomire_Instr_4, Crocomire_Instr_5, Crocomire_Instr_6, Crocomire_Instr_7,
    Crocomire_Instr_8, Crocomire_Instr_9, Crocomire_Main, Crocomire_Powerbomb,
    Enemy_GrappleReact_SamusLatchesOn_A4, Enemy_NormalFrozenAI_A4, Enemy_NormalShotAI_A4,
    Enemy_NormalTouchAI_A4,
};
use crate::game::sm_a5::{
    Draygon_Func_36, Draygon_Hurt, Draygon_Init, Draygon_Instr_1, Draygon_Instr_10,
    Draygon_Instr_11, Draygon_Instr_12, Draygon_Instr_13, Draygon_Instr_14, Draygon_Instr_15,
    Draygon_Instr_16, Draygon_Instr_17, Draygon_Instr_18, Draygon_Instr_19, Draygon_Instr_2,
    Draygon_Instr_20, Draygon_Instr_21, Draygon_Instr_22, Draygon_Instr_23, Draygon_Instr_24,
    Draygon_Instr_25, Draygon_Instr_26, Draygon_Instr_27, Draygon_Instr_28, Draygon_Instr_29,
    Draygon_Instr_3, Draygon_Instr_30, Draygon_Instr_4, Draygon_Instr_5, Draygon_Instr_6,
    Draygon_Instr_7, Draygon_Instr_8, Draygon_Instr_9, Draygon_Main, Draygon_Powerbomb,
    Draygon_Shot, Draygon_Touch, DraygonsArms_Init, DraygonsEye_Init, DraygonsEye_Main,
    DraygonsTail_Init, Enemy_GrappleReact_CancelBeam_A5, Enemy_GrappleReact_NoInteract_A5,
    Enemy_NormalFrozenAI_A5, Enemy_NormalShotAI_A5, Enemy_NormalTouchAI_A5, SporeSpawn_Init,
    SporeSpawn_Main, SporeSpawn_Shot, SporeSpawn_Touch,
};
use crate::game::sm_a6::{
    sub_A6DF59, Boulder_Init, Boulder_Main, CeresDoor_Init, CeresDoor_Instr_1, CeresDoor_Instr_2,
    CeresDoor_Instr_3, CeresDoor_Instr_4, CeresDoor_Instr_5, CeresDoor_Instr_6, CeresDoor_Instr_7,
    CeresDoor_Instr_8, CeresDoor_Main, CeresRidley_Hurt, CeresRidley_Init, CeresRidley_Main,
    CeresSteam_Init, CeresSteam_Instr_1, CeresSteam_Instr_2, CeresSteam_Instr_3,
    CeresSteam_Instr_4, CeresSteam_Instr_5, CeresSteam_Main, CeresSteam_Touch,
    Enemy_GrappleReact_CancelBeam_A6, Enemy_GrappleReact_NoInteract_A6, Enemy_NormalFrozenAI_A6,
    Enemy_NormalShotAI_A6, Enemy_NormalTouchAI_A6, FakeKraid_Init, FakeKraid_Instr_1,
    FakeKraid_Instr_2, FakeKraid_Instr_3, FakeKraid_Instr_4, FakeKraid_Instr_5, FakeKraid_Main,
    FakeKraid_Shot, FakeKraid_Touch, FireGeyser_Init, FireGeyser_Instr_1, FireGeyser_Instr_10,
    FireGeyser_Instr_11, FireGeyser_Instr_12, FireGeyser_Instr_13, FireGeyser_Instr_14,
    FireGeyser_Instr_15, FireGeyser_Instr_16, FireGeyser_Instr_17, FireGeyser_Instr_18,
    FireGeyser_Instr_19, FireGeyser_Instr_2, FireGeyser_Instr_20, FireGeyser_Instr_21,
    FireGeyser_Instr_22, FireGeyser_Instr_23, FireGeyser_Instr_24, FireGeyser_Instr_3,
    FireGeyser_Instr_4, FireGeyser_Instr_5, FireGeyser_Instr_6, FireGeyser_Instr_7,
    FireGeyser_Instr_8, FireGeyser_Instr_9, FireGeyser_Main, NuclearWaffle_Init,
    NuclearWaffle_Main, Ridley_A2F2, Ridley_Func_2, Ridley_Hurt, Ridley_Instr_1, Ridley_Instr_10,
    Ridley_Instr_11, Ridley_Instr_12, Ridley_Instr_13, Ridley_Instr_14, Ridley_Instr_15,
    Ridley_Instr_16, Ridley_Instr_2, Ridley_Instr_3, Ridley_Instr_4, Ridley_Instr_5,
    Ridley_Instr_6, Ridley_Instr_7, Ridley_Instr_8, Ridley_Instr_9, Ridley_Main, Ridley_Powerbomb,
    Ridley_Shot, RidleysExplosion_Init, RidleysExplosion_Main, SpikeyPlatform2ndEnemy_Init,
    SpikeyPlatform2ndEnemy_Main, SpikeyPlatform_Init, SpikeyPlatform_Main, Zebetites_Init,
    Zebetites_Main, Zebetites_Shot, Zebetites_Touch,
};
use crate::game::sm_a7::{
    Dachora_Func_11, Dachora_Func_12, Dachora_Func_2, Dachora_Func_3, Dachora_Func_4,
    Dachora_Func_5, Dachora_Func_7, Dachora_Func_8, Dachora_Init, Dachora_Main, EnemyInstr_Call_A7,
    Enemy_GrappleReact_CancelBeam_A7, Enemy_GrappleReact_NoInteract_A7, Enemy_NormalFrozenAI_A7,
    Enemy_NormalPowerBombAI_SkipDeathAnim_A7, Enemy_NormalShotAI_A7, Etecoon_Func_10,
    Etecoon_Func_11, Etecoon_Func_12, Etecoon_Func_16, Etecoon_Func_17, Etecoon_Func_18,
    Etecoon_Func_19, Etecoon_Func_20, Etecoon_Func_21, Etecoon_Func_22, Etecoon_Func_23,
    Etecoon_Func_24, Etecoon_Func_25, Etecoon_Func_26, Etecoon_Func_4, Etecoon_Func_5,
    Etecoon_Func_6, Etecoon_Func_7, Etecoon_Func_8, Etecoon_Func_9, Etecoon_Init, Etecoon_Main,
    Kraid_Arm_Shot, Kraid_Init, Kraid_Instr_1, Kraid_Instr_9, Kraid_Instr_DecYpos,
    Kraid_Instr_IncrYpos_Shake, Kraid_Instr_MoveHimRight, Kraid_Instr_PlaySound_0x76,
    Kraid_Instr_XposMinus3, Kraid_Instr_XposMinus3b, Kraid_Instr_XposPlus3, Kraid_Main,
    Kraid_Touch, Kraid_Touch_ArmFoot, KraidsArm_Init, KraidsArm_Main, KraidsArm_Touch,
    KraidsBadFingernail_Init, KraidsBadFingernail_Main, KraidsBadFingernail_Touch,
    KraidsBottomLint_Init, KraidsBottomLint_Main, KraidsFoot_Init, KraidsFoot_Main,
    KraidsGoodFingernail_Init, KraidsGoodFingernail_Main, KraidsGoodFingernail_Touch,
    KraidsMiddleLint_Init, KraidsMiddleLint_Main, KraidsTopLint_Init, KraidsTopLint_Main,
    Phantoon2_Init, Phantoon_AlmostDead, Phantoon_BecomeSolidAfterRainingFireballs,
    Phantoon_BecomesSolidAndBodyVuln, Phantoon_CompleteSwoopAfterFatalShot, Phantoon_Dead,
    Phantoon_DyingFadeOut, Phantoon_DyingPhantoonExplosions, Phantoon_DyingPhantoonFadeInOut,
    Phantoon_Enraged, Phantoon_EyeFollowsSamusUntilTimerRunsOut, Phantoon_FadeInBeforeFigure8,
    Phantoon_FadeInDuringFireballRain, Phantoon_FadeInEnragedPhantoon,
    Phantoon_FadeOutBeforeEnrage, Phantoon_FadeOutBeforeFirstFireballRain,
    Phantoon_FadeOutDuringFireballRain, Phantoon_FadeoutAfterEnrage, Phantoon_FadeoutWithSwoop,
    Phantoon_FollowSamusWithEyeDuringFireballRain, Phantoon_Hurt, Phantoon_Init,
    Phantoon_IsSwooping, Phantoon_Main, Phantoon_MoveEnragedPhantoonToTopCenter,
    Phantoon_MoveLeftOrRightAndPickEyeOpenPatt, Phantoon_MovePhantoonInFigure8ThenOpenEye,
    Phantoon_PickPatternForRound1, Phantoon_Shot, Phantoon_Spawn8FireballsInCircleAtStart,
    Phantoon_SpawnFireballsBeforeFight, Phantoon_SpawnRainingFireballs, Phantoon_Touch,
    Phantoon_WaitAfterFadeOut, Phantoon_WaitBetweenSpawningAndSpinningFireballs,
    Phantoon_WavyDyingPhantoonAndCry, Phantoon_WavyFadeIn,
};
use crate::game::sm_a8::{
    sub_A8B193, sub_A8B291, Beetom_Init, Beetom_Instr_1, Beetom_Main, Beetom_Shot, Beetom_Touch,
    BlueBrinstarFaceBlock_Func_1, BlueBrinstarFaceBlock_Init, BlueBrinstarFaceBlock_Main,
    BlueBrinstarFaceBlock_Shot, Enemy_GrappleReact_CancelBeam_A8, Enemy_GrappleReact_HurtSamus_A8,
    Enemy_GrappleReact_KillEnemy_A8, Enemy_GrappleReact_NoInteract_A8,
    Enemy_GrappleReact_SamusLatchesNoInvinc_A8, Enemy_NormalFrozenAI_A8, Enemy_NormalShotAI_A8,
    Enemy_NormalTouchAI_A8, EvirProjectile_Init, EvirProjectile_Main, Fune_Init, Fune_Instr_1,
    Fune_Instr_2, Fune_Instr_3, Fune_Instr_4, Fune_Instr_5, Fune_Instr_6, Fune_Instr_7, Fune_Main,
    Kago_Init, Kago_Main, Kago_Shot, KiHunterWings_Init, KiHunterWings_Main, KiHunter_Init,
    KiHunter_Instr_1, KiHunter_Instr_2, KiHunter_Instr_3, KiHunter_Instr_4, KiHunter_Instr_5,
    KiHunter_Main, KiHunter_Shot, MaridiaFloater_Func_10, MaridiaFloater_Func_11,
    MaridiaFloater_Func_12, MaridiaFloater_Func_13, MaridiaFloater_Func_3, MaridiaFloater_Func_4,
    MaridiaFloater_Func_5, MaridiaFloater_Func_6, MaridiaFloater_Func_7, MaridiaFloater_Func_8,
    MaridiaFloater_Func_9, MaridiaFloater_Init, MaridiaFloater_Main, MaridiaFloater_Powerbomb,
    MaridiaFloater_Shot, MaridiaFloater_Touch, MaridiaPuffer_Init, MaridiaPuffer_Main,
    MaridiaPuffer_Shot, MiniDraygon_Init, MiniDraygon_Instr_1, MiniDraygon_Instr_2,
    MiniDraygon_Instr_3, MiniDraygon_Instr_4, MiniDraygon_Main, MiniDraygon_Powerbomb,
    MiniDraygon_Shot, MiniDraygon_Touch, MorphBallEye_Func_1, MorphBallEye_Func_2,
    MorphBallEye_Func_3, MorphBallEye_Func_4, MorphBallEye_Init, MorphBallEye_Main,
    NorfairLavaMan_Func_10, NorfairLavaMan_Func_11, NorfairLavaMan_Func_13, NorfairLavaMan_Func_15,
    NorfairLavaMan_Func_16, NorfairLavaMan_Func_17, NorfairLavaMan_Func_18, NorfairLavaMan_Func_6,
    NorfairLavaMan_Func_7, NorfairLavaMan_Func_8, NorfairLavaMan_Func_9, NorfairLavaMan_Init,
    NorfairLavaMan_Instr_1, NorfairLavaMan_Instr_10, NorfairLavaMan_Instr_11,
    NorfairLavaMan_Instr_12, NorfairLavaMan_Instr_13, NorfairLavaMan_Instr_14,
    NorfairLavaMan_Instr_15, NorfairLavaMan_Instr_16, NorfairLavaMan_Instr_2,
    NorfairLavaMan_Instr_3, NorfairLavaMan_Instr_4, NorfairLavaMan_Instr_5, NorfairLavaMan_Instr_6,
    NorfairLavaMan_Instr_7, NorfairLavaMan_Instr_8, NorfairLavaMan_Instr_9, NorfairLavaMan_Main,
    NorfairLavaMan_Powerbomb, NorfairLavaMan_Shot, NorfairLavaMan_Touch, WalkingLavaSeahorse_Init,
    WalkingLavaSeahorse_Instr_1, WalkingLavaSeahorse_Instr_2, WalkingLavaSeahorse_Instr_3,
    WalkingLavaSeahorse_Instr_4, WalkingLavaSeahorse_Instr_5, WalkingLavaSeahorse_Instr_6,
    WalkingLavaSeahorse_Main, WreckedShipGhost_Init, WreckedShipGhost_Main, WreckedShipOrbs_Init,
    WreckedShipOrbs_Main, WreckedShipRobotDeactivated_Init, WreckedShipRobotDeactivated_Shot,
    WreckedShipRobotDeactivated_Touch, WreckedShipRobot_Func_1, WreckedShipRobot_Init,
    WreckedShipRobot_Instr_1, WreckedShipRobot_Instr_10, WreckedShipRobot_Instr_11,
    WreckedShipRobot_Instr_12, WreckedShipRobot_Instr_13, WreckedShipRobot_Instr_14,
    WreckedShipRobot_Instr_15, WreckedShipRobot_Instr_16, WreckedShipRobot_Instr_17,
    WreckedShipRobot_Instr_18, WreckedShipRobot_Instr_2, WreckedShipRobot_Instr_3,
    WreckedShipRobot_Instr_4, WreckedShipRobot_Instr_5, WreckedShipRobot_Instr_6,
    WreckedShipRobot_Instr_7, WreckedShipRobot_Instr_8, WreckedShipRobot_Instr_9,
    WreckedShipRobot_Main, WreckedShipRobot_Shot, WreckedShipSpark_Init, WreckedShipSpark_Instr_1,
    WreckedShipSpark_Instr_2, WreckedShipSpark_Main, WreckedShipSpark_Shot, YappingMaw_Frozen,
    YappingMaw_Init, YappingMaw_Instr_1, YappingMaw_Instr_2, YappingMaw_Instr_3,
    YappingMaw_Instr_4, YappingMaw_Instr_5, YappingMaw_Instr_6, YappingMaw_Instr_7,
    YappingMaw_Main, YappingMaw_Shot, YappingMaw_Touch,
};
use crate::game::sm_a9::{
    sub_A9ECD0, DeadRipper_Init, DeadRipper_Powerbomb, DeadRipper_Shot, DeadSidehopper_Init,
    DeadSidehopper_Main, DeadSidehopper_Powerbomb, DeadSidehopper_Shot, DeadSidehopper_Touch,
    DeadSkree_Init, DeadSkree_Powerbomb, DeadSkree_Shot, DeadTorizo_Init, DeadTorizo_Main,
    DeadTorizo_MainGfxHook, DeadTorizo_Powerbomb, DeadTorizo_Shot, DeadZoomer_Init,
    DeadZoomer_Powerbomb, DeadZoomer_Shot, Enemy_GrappleReact_CancelBeam_A9,
    Enemy_NormalFrozenAI_A9, MotherBrain_DrawBrainNeck_EnemyGfxDrawHook,
    MotherBrain_Instr_Down2Right15, MotherBrain_Instr_Down4Right6,
    MotherBrain_Instr_IncrBeamAttackPhase, MotherBrain_Instr_MoveBodyUp10Left4,
    MotherBrain_Instr_MoveBodyUp12Right2, MotherBrain_Instr_MoveBodyUp16Left4,
    MotherBrain_Instr_MoveDown1, MotherBrain_Instr_MoveDown10Right2,
    MotherBrain_Instr_MoveDown12Left4, MotherBrain_Instr_MoveDown16Right2,
    MotherBrain_Instr_MoveDown1Left3, MotherBrain_Instr_MoveDown2Right1,
    MotherBrain_Instr_MoveDown4Right2, MotherBrain_Instr_MoveLeft2, MotherBrain_Instr_MoveRight2,
    MotherBrain_Instr_MoveUp1, MotherBrain_Instr_MoveUp1Right3_Sfx,
    MotherBrain_Instr_MoveUp2Left15_Sfx, MotherBrain_Instr_MoveUp2Right1,
    MotherBrain_Instr_MoveUp4Left6, MotherBrain_Instr_SetPose_Crouched,
    MotherBrain_Instr_SetPose_CrouchedTrans, MotherBrain_Instr_SetPose_DeathBeamMode,
    MotherBrain_Instr_SetPose_LeaningDown, MotherBrain_Instr_SetPose_Standing,
    MotherBrain_Instr_SetPose_Walking, MotherBrain_Instr_SpawnDeathBeamEproj,
    MotherBrain_Instr_SpawnEprojToOffset, MotherBrain_Instr_Up2Left1_Sfx,
    MotherBrain_Instr_Up2Left1_Sfx2, MotherBrain_Instr_Up4Left2, MotherBrainsBody_Hurt,
    MotherBrainsBody_Init, MotherBrainsBody_Powerbomb, MotherBrainsBody_Shot,
    MotherBrainsBrain_GfxDrawHook, MotherBrainsBrain_Hurt, MotherBrainsBrain_Init,
    MotherBrainsBrain_Shot, MotherBrainsBrain_Touch, MotherBrainsTubesFalling_Init,
    MotherBrainsTubesFalling_Main, ShitroidInCutscene_Init, ShitroidInCutscene_Main,
    ShitroidInCutscene_Touch, Shitroid_Init, Shitroid_Instr_1, Shitroid_Instr_2, Shitroid_Instr_3,
    Shitroid_Instr_4, Shitroid_Instr_5, Shitroid_Instr_6, Shitroid_Main, Shitroid_Powerbomb,
    Shitroid_Shot, Shitroid_Touch,
};
use crate::game::sm_aa::{
    sub_AAE445, ChozoStatue_Init, ChozoStatue_Main, Enemy_ClearAiPreInstr_AA,
    Enemy_GrappleReact_CancelBeam_AA, Enemy_NormalFrozenAI_AA, Enemy_SetAiPreInstr_AA,
    GoldTorizo_Hurt, GoldTorizo_Main, GoldTorizo_Shot, GoldTorizo_Touch, N00bTubeCracks_Init,
    Shaktool_DCAC, Shaktool_DCD7, Shaktool_DD25, Shaktool_Hurt, Shaktool_Init, Shaktool_Instr_1,
    Shaktool_Instr_10, Shaktool_Instr_11, Shaktool_Instr_12, Shaktool_Instr_13, Shaktool_Instr_14,
    Shaktool_Instr_2, Shaktool_Instr_3, Shaktool_Instr_4, Shaktool_Instr_5, Shaktool_Instr_6,
    Shaktool_Instr_7, Shaktool_Instr_8, Shaktool_Instr_9, Shaktool_PreInstr_0, Shaktool_Shot,
    Shaktool_Touch, Torizo_D5ED, Torizo_D5F1, Torizo_Func_5, Torizo_Func_6, Torizo_Func_7,
    Torizo_Func_8, Torizo_Hurt, Torizo_Init, Torizo_Instr_1, Torizo_Instr_10, Torizo_Instr_11,
    Torizo_Instr_12, Torizo_Instr_13, Torizo_Instr_14, Torizo_Instr_15, Torizo_Instr_16,
    Torizo_Instr_17, Torizo_Instr_18, Torizo_Instr_19, Torizo_Instr_2, Torizo_Instr_20,
    Torizo_Instr_21, Torizo_Instr_22, Torizo_Instr_23, Torizo_Instr_24, Torizo_Instr_25,
    Torizo_Instr_26, Torizo_Instr_27, Torizo_Instr_28, Torizo_Instr_29, Torizo_Instr_3,
    Torizo_Instr_30, Torizo_Instr_31, Torizo_Instr_32, Torizo_Instr_33, Torizo_Instr_34,
    Torizo_Instr_35, Torizo_Instr_36, Torizo_Instr_37, Torizo_Instr_38, Torizo_Instr_39,
    Torizo_Instr_4, Torizo_Instr_40, Torizo_Instr_41, Torizo_Instr_42, Torizo_Instr_43,
    Torizo_Instr_44, Torizo_Instr_45, Torizo_Instr_46, Torizo_Instr_47, Torizo_Instr_48,
    Torizo_Instr_49, Torizo_Instr_5, Torizo_Instr_50, Torizo_Instr_51, Torizo_Instr_52,
    Torizo_Instr_53, Torizo_Instr_54, Torizo_Instr_55, Torizo_Instr_56, Torizo_Instr_57,
    Torizo_Instr_58, Torizo_Instr_59, Torizo_Instr_6, Torizo_Instr_60, Torizo_Instr_61,
    Torizo_Instr_62, Torizo_Instr_63, Torizo_Instr_7, Torizo_Instr_8, Torizo_Instr_9, Torizo_Main,
    Torizo_Shot, TourianEntranceStatue_Init,
};
use crate::game::sm_b2::{
    Enemy_GrappleReact_CancelBeam_B2, Enemy_NormalFrozenAI_B2, NinjaSpacePirates_Init,
    NinjaSpacePirates_Main, SpacePirates_Instr_11, SpacePirates_Instr_12, SpacePirates_Instr_13,
    SpacePirates_Instr_14, SpacePirates_Instr_15, SpacePirates_Instr_16, SpacePirates_Instr_17,
    SpacePirates_Instr_18, SpacePirates_Instr_19, SpacePirates_Instr_20, SpacePirates_Instr_21,
    SpacePirates_Instr_FireLaserL, SpacePirates_Instr_FireLaserR,
    SpacePirates_Instr_MovePixelsDownAndChangeDirFaceLeft,
    SpacePirates_Instr_MovePixelsDownAndChangeDirFaceRight, SpacePirates_Instr_PlaySfx,
    SpacePirates_Instr_PrepareWallJumpL, SpacePirates_Instr_PrepareWallJumpR,
    SpacePirates_Instr_RandomNewDirFaceL, SpacePirates_Instr_RandomNewDirFaceR,
    SpacePirates_Instr_SetEnemyFunc, WalkingSpacePirates_87C8, WalkingSpacePirates_883E,
    WalkingSpacePirates_Init, WalkingSpacePirates_Main, WalkingSpacePirates_Powerbomb,
    WalkingSpacePirates_Shot, WalkingSpacePirates_Touch, WallSpacePirates_Init,
    WallSpacePirates_Main,
};
use crate::game::sm_b3::{
    Botwoon_Func_26, Botwoon_Func_27, Botwoon_Func_28, Botwoon_Func_29, Botwoon_Func_30,
    Botwoon_Init, Botwoon_Instr_1, Botwoon_Instr_10, Botwoon_Instr_2, Botwoon_Instr_3,
    Botwoon_Instr_4, Botwoon_Instr_5, Botwoon_Instr_6, Botwoon_Instr_7, Botwoon_Instr_8,
    Botwoon_Instr_9, Botwoon_Instr_QueueSpitSfx, Botwoon_Instr_SetSpitting, Botwoon_Main,
    Botwoon_Powerbomb, Botwoon_Shot, Botwoon_Touch, BrinstarPipeBug_Init, BrinstarPipeBug_Main,
    BrinstarPipeBug_PreInstr_1, BrinstarPipeBug_PreInstr_2, BrinstarPipeBug_PreInstr_3,
    BrinstarPipeBug_PreInstr_4, BrinstarPipeBug_PreInstr_5, BrinstarYellowPipeBug_Init,
    BrinstarYellowPipeBug_Main, Enemy_ClearAiPreInstr_B3, Enemy_GrappleReact_CancelBeam_B3,
    Enemy_GrappleReact_KillEnemy_B3, Enemy_GrappleReact_NoInteract_B3, Enemy_NormalFrozenAI_B3,
    Enemy_NormalShotAI_B3, Enemy_NormalTouchAI_B3, Enemy_SetAiPreInstr_B3, EscapeDachora_Init,
    EscapeDachora_Instr_1, EscapeDachora_Instr_2, EscapeDachora_Instr_3, EscapeDachora_Instr_4,
    EscapeEtecoon_E65C, EscapeEtecoon_E670, EscapeEtecoon_E680, EscapeEtecoon_Init,
    EscapeEtecoon_Instr_1, EscapeEtecoon_Instr_2, EscapeEtecoon_Main, NorfairPipeBug_Func_5,
    NorfairPipeBug_Init, NorfairPipeBug_Main, UnusedSpinningTurtleEye_Init,
    UnusedSpinningTurtleEye_Main,
};
use crate::game::sm_b4::{
    ClearSpriteObjects, CreateSpriteAtPos, DrawSpriteObjects, HandleSpriteObjects,
};
use crate::game_state::Game;
use crate::ram_map::{ram_word, set_ram_word, update_ram_word};
use crate::rtl::abs16;
use crate::rtl::{Load24, Mult8x8, RomPtr, SnesDivide, Unreachable};
use ::core::ffi::{c_char, c_void};
use ::core::ptr::{null, null_mut};
extern "C" {
    fn __assert_fail(
        __assertion: *const c_char,
        __file: *const c_char,
        __line: u32,
        __function: *const c_char,
    ) -> !;
    fn memcpy(__dest: *mut c_void, __src: *const c_void, __n: size_t) -> *mut c_void;
    fn memset(__s: *mut c_void, __c: i32, __n: size_t) -> *mut c_void;

}
pub use crate::types::{size_t, EnemyData, LongPtr, PairU16, Point16U, VoidP, VramWriteEntry};
pub use crate::types::{EnemyDef, EnemySpawnData, Vulnerability};
pub use crate::types::{EnemyProps, EprojDef, ExtendedSpriteMap, Hitbox};
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct EnemyPopulation {
    pub enemy_ptr: VoidP,
    pub x_pos: u16,
    pub y_pos: u16,
    pub init_param: u16,
    pub properties: u16,
    pub extra_properties: u16,
    pub parameter1: u16,
    pub parameter2: u16,
}
pub use crate::types::{EnemyAiBits, EnemyTileset};
pub const kEnemyAiBits_Frozen: EnemyAiBits = 4;
pub const kEnemyAiBits_Hurt: EnemyAiBits = 2;
pub const kEnemyAiBits_Grapple: EnemyAiBits = 1;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct PositionAndWidth {
    pub x_pos: u16,
    pub x_subpos: u16,
    pub y_pos: u16,
    pub y_subpos: u16,
    pub width: u16,
    pub height: u16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct EnemyBlockCollInfo {
    pub ebci_r18_r20: i32,
    pub ebci_r24: u16,
    pub ebci_r26: u16,
    pub ebci_r28: u16,
    pub ebci_r30: u16,
    pub ebci_r32: u16,
}
pub type Func_EnemyBlockCollInfo_U8 = unsafe fn(&mut EnemyBlockCollInfo, &mut Game) -> u8;
pub use crate::types::Point32;
pub const NULL: *mut c_void = null_mut::<c_void>();
#[inline]
unsafe fn RomPtr_RAM(addr: u16, game: &mut Game) -> *mut u8 {
    '_c2rust_label: {
        if addr < 0x2000u16 {
        } else {
            __assert_fail(
                b"addr < 0x2000\0" as *const u8 as *const c_char,
                b"src/sm_rtl.h\0" as *const u8 as *const c_char,
                51u32,
                b"uint8 *RomPtr_RAM(uint16_t)\0" as *const u8 as *const c_char,
            );
        }
    };
    (&raw mut game.g_ram as *mut u8).offset(addr as i32 as isize)
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
    RomPtr(((db as i32) << 16i32 | a as i32) as u32, game) as *mut ExtendedSpriteMap
}
#[inline]
unsafe fn get_EnemyPopulation(db: u8, a: u16, game: &mut Game) -> *mut EnemyPopulation {
    RomPtr(((db as i32) << 16i32 | a as i32) as u32, game) as *mut EnemyPopulation
}
pub const fnEprojPreInstr_nullsub_83: i32 = 0x8684fb;
pub const fnGrappleBeam_Func2: i32 = 0x9bc8c5;
pub const fnEnemy_GrappleReact_NoInteract_A0: i32 = 0xa08000;
pub const fnEnemy_GrappleReact_SamusLatchesOn_A0: i32 = 0xa08005;
pub const fnEnemy_GrappleReact_KillEnemy_A0: i32 = 0xa0800a;
pub const fnEnemy_GrappleReact_CancelBeam_A0: i32 = 0xa0800f;
pub const fnEnemy_GrappleReact_SamusLatchesNoInvinc_A0: i32 = 0xa08014;
pub const fnEnemy_GrappleReact_SamusLatchesParalyze_A0: i32 = 0xa08019;
pub const fnEnemy_GrappleReact_HurtSamus_A0: i32 = 0xa0801e;
pub const fnEnemy_NormalTouchAI_A0: u32 = 10518563;
pub const fnEnemy_NormalShotAI_A0: u32 = 10518573;
pub const fnEnemy_NormalPowerBombAI_A0: i32 = 0xa08037;
pub const fnnullsub_169: i32 = 0xa0804b;
pub const fnnullsub_170: i32 = 0xa0804c;
pub const fnnullsub_171: i32 = 0xa0807b;
pub const fnEnemy_GrappleReact_SamusLatchesOn_A2: u32 = 10649605;
pub const fnEnemy_GrappleReact_KillEnemy_A2: u32 = 10649610;
pub const fnEnemy_GrappleReact_CancelBeam_A2: u32 = 10649615;
pub const fnEnemy_NormalTouchAI_A2: u32 = 10649635;
pub const fnEnemy_NormalShotAI_A2: u32 = 10649645;
pub const fnEnemy_NormalPowerBombAI_A2: u32 = 10649655;
pub const fnEnemy_NormalFrozenAI_A2: u32 = 10649665;
pub const fnnullsub_170_A2: u32 = 10649676;
pub const fnnullsub_171_A2: u32 = 10649723;
pub const fnEnemyInstr_Goto_A2: u32 = 10649837;
pub const fnEnemyInstr_DecTimerAndGoto2_A2: u32 = 10649872;
pub const fnEnemyInstr_SetTimer_A2: u32 = 10649891;
pub const fnEnemyInstr_Sleep_A2: u32 = 10649903;
pub const fnEnemyInstr_EnableOffScreenProcessing_A2: u32 = 10649971;
pub const fnEnemyInstr_DisableOffScreenProcessing_A2: u32 = 10649981;
pub const fnBouncingGoofball_Init: u32 = 10651420;
pub const fnBouncingGoofball_Main: u32 = 10651548;
pub const fnBouncingGoofball_Instr_88C5: u32 = 10651845;
pub const fnBouncingGoofball_Instr_88C6: u32 = 10651846;
pub const fnMiniCrocomire_Instr_897E: u32 = 10652030;
pub const fnMiniCrocomire_Instr_8990: u32 = 10652048;
pub const fnMiniCrocomire_Instr_899D: u32 = 10652061;
pub const fnMiniCrocomire_Init: u32 = 10652077;
pub const fnMiniCrocomire_Main: u32 = 10652144;
pub const fnMiniCrocomire_PreInstr5: u32 = 10652227;
pub const fnMiniCrocomire_PreInstr6: u32 = 10652252;
pub const fnnullsub_175: u32 = 10652277;
pub const fnMaridiaBeybladeTurtle_Init: u32 = 10653036;
pub const fnMiniMaridiaBeybladeTurtle_Init: u32 = 10653085;
pub const fnMaridiaBeybladeTurtle_Main: u32 = 10653138;
pub const fnMiniMaridiaBeybladeTurtle_Main: u32 = 10653998;
pub const fnMaridiaBeybladeTurtle_Touch: u32 = 10654337;
pub const fnMiniMaridiaBeybladeTurtle_Touch: u32 = 10654367;
pub const fnMiniMaridiaBeybladeTurtle_Shot: u32 = 10654479;
pub const fnMaridiaBeybladeTurtle_Instr_9381: u32 = 10654593;
pub const fnMaridiaBeybladeTurtle_Instr_9412: u32 = 10654738;
pub const fnMaridiaBeybladeTurtle_Instr_9447: u32 = 10654791;
pub const fnMaridiaBeybladeTurtle_Instr_9451: u32 = 10654801;
pub const fnMaridiaBeybladeTurtle_Instr_946B: u32 = 10654827;
pub const fnMaridiaBeybladeTurtle_Instr_9485: u32 = 10654853;
pub const fnMaridiaBeybladeTurtle_Instr_94A1: u32 = 10654881;
pub const fnMaridiaBeybladeTurtle_Instr_94C7: u32 = 10654919;
pub const fnMaridiaBeybladeTurtle_Instr_94D1: u32 = 10654929;
pub const fnThinHoppingBlobs_Init: u32 = 10656319;
pub const fnThinHoppingBlobs_Main: u32 = 10656381;
pub const fnSpikeShootingPlant_Instr_9F2A: u32 = 10657578;
pub const fnSpikeShootingPlant_Init: u32 = 10657608;
pub const fnSpikeShootingPlant_Main: u32 = 10657715;
pub const fnSpikeShootingPlant_2: u32 = 10657722;
pub const fnSpikeShootingPlant_3: u32 = 10657772;
pub const fnnullsub_182: u32 = 10657819;
pub const fnSpikeShootingPlant_Instr_A095: u32 = 10657941;
pub const fnSpikeShootingPlant_Instr_A0A7: u32 = 10657959;
pub const fnMaridiaSpikeyShell_Init: u32 = 10658809;
pub const fnMaridiaSpikeyShell_Main: u32 = 10658942;
pub const fnMaridiaSpikeyShell_Instr_A56D: u32 = 10659181;
pub const fnMaridiaSpikeyShell_Instr_A571: u32 = 10659185;
pub const fnMaridiaSpikeyShell_Shot: u32 = 10659193;
pub const fnGunshipTop_Init: u32 = 10659396;
pub const fnGunshipBottom_Init: u32 = 10659538;
pub const fnGunshipTop_Main: u32 = 10659673;
pub const fnnullsub_187: u32 = 10659799;
pub const fnGunshipTop_3: u32 = 10659852;
pub const fnGunshipTop_4: u32 = 10660048;
pub const fnGunshipTop_5: u32 = 10660162;
pub const fnGunshipTop_6: u32 = 10660176;
pub const fnGunshipTop_7: u32 = 10660231;
pub const fnGunshipTop_8: u32 = 10660285;
pub const fnGunshipTop_9: u32 = 10660431;
pub const fnGunshipTop_10: u32 = 10660445;
pub const fnGunshipTop_11: u32 = 10660500;
pub const fnGunshipTop_12: u32 = 10660514;
pub const fnGunshipTop_13: u32 = 10660639;
pub const fnGunshipTop_14: u32 = 10660704;
pub const fnGunshipTop_15: u32 = 10660718;
pub const fnGunshipTop_16: u32 = 10660773;
pub const fnGunshipTop_17: u32 = 10660807;
pub const fnGunshipTop_18: u32 = 10660891;
pub const fnGunshipTop_19: u32 = 10661079;
pub const fnGunshipTop_20: u32 = 10661134;
pub const fnGunshipTop_21: u32 = 10661165;
pub const fnFlies_Init: u32 = 10661995;
pub const fnFlies_Main: u32 = 10662175;
pub const fnFlies_4: u32 = 10662222;
pub const fnFlies_5: u32 = 10662268;
pub const fnFlies_6: u32 = 10662314;
pub const fnFlies_7: u32 = 10662354;
pub const fnNorfairErraticFireball_Init: u32 = 10662880;
pub const fnNorfairErraticFireball_Main: u32 = 10662927;
pub const fnLavaquakeRocks_Init: u32 = 10663280;
pub const fnLavaquakeRocks_Main: u32 = 10663311;
pub const fnRinka_Init: u32 = 10663426;
pub const fnRinka_Main: u32 = 10663876;
pub const fnRinka_Frozen: u32 = 10664233;
pub const fnRinka_Touch: u32 = 10664263;
pub const fnRinka_Shot: u32 = 10664269;
pub const fnRinka_Powerbomb: u32 = 10664275;
pub const fnRinka_Instr_B9B3: u32 = 10664371;
pub const fnRinka_Instr_B9BD: u32 = 10664381;
pub const fnRinka_Instr_B9C7: u32 = 10664391;
pub const fnEnemyInstr_Rio_Instr_1: u32 = 10664899;
pub const fnRio_Init: u32 = 10664909;
pub const fnRio_Main: u32 = 10664931;
pub const fnNorfairLavajumpingEnemy_Instr_BE8E: u32 = 10665614;
pub const fnNorfairLavajumpingEnemy_Init: u32 = 10665625;
pub const fnNorfairLavajumpingEnemy_Main: u32 = 10665682;
pub const fnNorfairLavajumpingEnemy_Func_1: u32 = 10665692;
pub const fnNorfairLavajumpingEnemy_Func_2: u32 = 10665754;
pub const fnNorfairLavajumpingEnemy_Func_3: u32 = 10665790;
pub const fnNorfairLavajumpingEnemy_Func_4: u32 = 10665852;
pub const fnNorfairLavajumpingEnemy_Func_5: u32 = 10665916;
pub const fnNorfairRio_Instr_C1C9: u32 = 10666441;
pub const fnNorfairRio_Instr_C1D4: u32 = 10666452;
pub const fnNorfairRio_Instr_C1DF: u32 = 10666463;
pub const fnNorfairRio_Instr_C1EA: u32 = 10666474;
pub const fnNorfairRio_Instr_C1F5: u32 = 10666485;
pub const fnNorfairRio_Instr_C200: u32 = 10666496;
pub const fnNorfairRio_Instr_C20B: u32 = 10666507;
pub const fnNorfairRio_Instr_C216: u32 = 10666518;
pub const fnNorfairRio_Instr_C221: u32 = 10666529;
pub const fnNorfairRio_Instr_C22C: u32 = 10666540;
pub const fnNorfairRio_Instr_C237: u32 = 10666551;
pub const fnNorfairRio_Init: u32 = 10666562;
pub const fnNorfairRio_Main: u32 = 10666615;
pub const fnNorfairRio_Func_1: u32 = 10666625;
pub const fnNorfairRio_Func_2: u32 = 10666727;
pub const fnNorfairRio_Func_3: u32 = 10666815;
pub const fnNorfairRio_Func_4: u32 = 10666849;
pub const fnNorfairRio_Func_5: u32 = 10666929;
pub const fnNorfairRio_Func_6: u32 = 10667014;
pub const fnLowerNorfairRio_Instr_C6D2: u32 = 10667730;
pub const fnLowerNorfairRio_Instr_C6DD: u32 = 10667741;
pub const fnLowerNorfairRio_Instr_C6E8: u32 = 10667752;
pub const fnLowerNorfairRio_Init: u32 = 10667763;
pub const fnLowerNorfairRio_Main: u32 = 10667812;
pub const fnLowerNorfairRio_Func_1: u32 = 10667822;
pub const fnLowerNorfairRio_Func_2: u32 = 10667889;
pub const fnLowerNorfairRio_Func_3: u32 = 10667963;
pub const fnLowerNorfairRio_Func_4: u32 = 10667990;
pub const fnLowerNorfairRio_Func_5: u32 = 10668077;
pub const fnLowerNorfairRio_Func_6: u32 = 10668168;
pub const fnMaridiaLargeSnail_Instr_CB6B: u32 = 10668907;
pub const fnMaridiaLargeSnail_Instr_CCB3: u32 = 10669235;
pub const fnMaridiaLargeSnail_Instr_CCBE: u32 = 10669246;
pub const fnMaridiaLargeSnail_Instr_CCC9: u32 = 10669257;
pub const fnMaridiaLargeSnail_Init: u32 = 10669268;
pub const fnMaridiaLargeSnail_Main: u32 = 10669331;
pub const fnMaridiaLargeSnail_Func_7: u32 = 10669926;
pub const fnMaridiaLargeSnail_Func_8: u32 = 10669993;
pub const fnMaridiaLargeSnail_Func_12: u32 = 10670984;
pub const fnMaridiaLargeSnail_Touch: u32 = 10670988;
pub const fnMaridiaLargeSnail_Shot: u32 = 10671028;
pub const fnHirisingSlowfalling_Init: u32 = 10674038;
pub const fnHirisingSlowfalling_Main: u32 = 10674222;
pub const fnGripper_Init: u32 = 10674643;
pub const fnGripper_Main: u32 = 10674721;
pub const fnJetPowerRipper_Init: u32 = 10674968;
pub const fnJetPowerRipper_Main: u32 = 10675027;
pub const fnJetPowerRipper_Shot: u32 = 10675113;
pub const fnRipper_Init: u32 = 10675359;
pub const fnRipper_Main: u32 = 10675418;
pub const fnLavaSeahorse_Instr_E5FB: u32 = 10675707;
pub const fnLavaSeahorse_Init: u32 = 10675718;
pub const fnLavaSeahorse_Main: u32 = 10675790;
pub const fnLavaSeahorse_Func_1: u32 = 10675796;
pub const fnLavaSeahorse_Func_2: u32 = 10675885;
pub const fnLavaSeahorse_Func_3: u32 = 10675953;
pub const fnLavaSeahorse_Func_4: u32 = 10676020;
pub const fnLavaSeahorse_Func_5: u32 = 10676041;
pub const fnnullsub_196: u32 = 10676097;
pub const fnLavaSeahorse_Touch: u32 = 10676168;
pub const fnLavaSeahorse_Shot: u32 = 10676174;
pub const fnLavaSeahorse_Powerbomb: u32 = 10676180;
pub const fnTimedShutter_Init: u32 = 10676698;
pub const fnTimedShutter_Main: u32 = 10676918;
pub const fnRisingFallingPlatform_Init: u32 = 10677765;
pub const fnShootableShutter_Init: u32 = 10677778;
pub const fnRisingFallingPlatform_Main: u32 = 10677969;
pub const fnRisingFallingPlatform_Touch: u32 = 10678429;
pub const fnRisingFallingPlatform_Shot: u32 = 10678434;
pub const fnShootableShutter_Shot: u32 = 10678442;
pub const fnRisingFallingPlatform_Powerbomb: u32 = 10678454;
pub const fnHorizontalShootableShutter_Init: u32 = 10678545;
pub const fnHorizontalShootableShutter_Main: u32 = 10678750;
pub const fnHorizontalShootableShutter_Touch: u32 = 10679256;
pub const fnHorizontalShootableShutter_Shot: u32 = 10679310;
pub const fnHorizontalShootableShutter_Powerbomb: u32 = 10679322;
pub const fnEnemy_GrappleReact_NoInteract_A3: u32 = 10715136;
pub const fnEnemy_GrappleReact_KillEnemy_A3: u32 = 10715146;
pub const fnEnemy_GrappleReact_CancelBeam_A3: u32 = 10715151;
pub const fnEnemy_NormalTouchAI_A3: u32 = 10715171;
pub const fnEnemy_NormalShotAI_A3: u32 = 10715181;
pub const fnEnemy_NormalPowerBombAI_A3: u32 = 10715191;
pub const fnEnemy_NormalFrozenAI_A3: u32 = 10715201;
pub const fnnullsub_170_A3: u32 = 10715212;
pub const fnEnemyInstr_Goto_A3: u32 = 10715373;
pub const fnEnemyInstr_Sleep_A3: u32 = 10715439;
pub const fnEnemyInstr_EnableOffScreenProcessing_A3: u32 = 10715507;
pub const fnEnemyInstr_DisableOffScreenProcessing_A3: u32 = 10715517;
pub const fnWaver_Instr_1: u32 = 10716899;
pub const fnWaver_Init: u32 = 10716909;
pub const fnWaver_Main: u32 = 10717004;
pub const fnMetalee_Instr_1: u32 = 10717526;
pub const fnMetalee_Init: u32 = 10717536;
pub const fnMetalee_Main: u32 = 10717561;
pub const fnMetalee_Shot: u32 = 10717967;
pub const fnFireflea_Init: u32 = 10718509;
pub const fnFireflea_Main: u32 = 10718702;
pub const fnFireflea_Touch: u32 = 10718827;
pub const fnFireflea_Powerbomb: u32 = 10718851;
pub const fnFireflea_Shot: u32 = 10718857;
pub const fnMaridiaFish_Instr_3: u32 = 10719382;
pub const fnMaridiaFish_Instr_1: u32 = 10719392;
pub const fnMaridiaFish_Instr_2: u32 = 10719402;
pub const fnMaridiaFish_Init: u32 = 10719413;
pub const fnMaridiaFish_Main: u32 = 10719531;
pub const fnElevator_Init: u32 = 10720486;
pub const fnElevator_Frozen: u32 = 10720554;
pub const fnCrab_Init: u32 = 10720995;
pub const fnSlug_Init: u32 = 10721595;
pub const fnPlatformThatFallsWithSamus_Instr_3: u32 = 10722411;
pub const fnPlatformThatFallsWithSamus_Instr_4: u32 = 10722422;
pub const fnPlatformThatFallsWithSamus_Instr_1: u32 = 10722433;
pub const fnPlatformThatFallsWithSamus_Instr_2: u32 = 10722444;
pub const fnPlatformThatFallsWithSamus_Init: u32 = 10722463;
pub const fnFastMovingSlowSinkingPlatform_Init: u32 = 10722490;
pub const fnPlatformThatFallsWithSamus_Main: u32 = 10722582;
pub const fnnullsub_32: u32 = 10723079;
pub const fnFastMovingSlowSinkingPlatform_Shot: u32 = 10723080;
pub const fnRoach_Init: u32 = 10723661;
pub const fnRoach_Main: u32 = 10724048;
pub const fnMochtroid_Init: u32 = 10725245;
pub const fnMochtroid_Main: u32 = 10725264;
pub const fnMochtroid_Touch: u32 = 10725715;
pub const fnMochtroid_Shot: u32 = 10725800;
pub const fnSidehopper_Func_1: u32 = 10725992;
pub const fnSidehopper_Instr_1: u32 = 10726142;
pub const fnSidehopper_Init: u32 = 10726153;
pub const fnSidehopper_Main: u32 = 10726351;
pub const fnMaridiaRefillCandy_Instr_1: u32 = 10728489;
pub const fnMaridiaRefillCandy_Instr_2: u32 = 10728500;
pub const fnMaridiaRefillCandy_Instr_3: u32 = 10728511;
pub const fnMaridiaRefillCandy_Init: u32 = 10728522;
pub const fnMaridiaRefillCandy_Main: u32 = 10728572;
pub const fnNorfairSlowFireball_Init: u32 = 10729071;
pub const fnBang_Instr_1: u32 = 10730104;
pub const fnBang_Instr_2: u32 = 10730152;
pub const fnBang_Init: u32 = 10730163;
pub const fnBang_Main: u32 = 10730277;
pub const fnBang_Func_6: u32 = 10730661;
pub const fnBang_Func_7: u32 = 10730689;
pub const fnBang_Func_8: u32 = 10730693;
pub const fnBang_Func_10: u32 = 10730780;
pub const fnBang_Func_11: u32 = 10730796;
pub const fnBang_Shot: u32 = 10731261;
pub const fnSkree_Instr_1: u32 = 10733220;
pub const fnSkree_Init: u32 = 10733230;
pub const fnSkree_Main: u32 = 10733255;
pub const fnSkree_Shot: u32 = 10733557;
pub const fnMaridiaSnail_Instr_1: u32 = 10734646;
pub const fnMaridiaSnail_Instr_2: u32 = 10734655;
pub const fnMaridiaSnail_Instr_4: u32 = 10734664;
pub const fnMaridiaSnail_Instr_3: u32 = 10734687;
pub const fnMaridiaSnail_Instr_5: u32 = 10734712;
pub const fnMaridiaSnail_Init: u32 = 10735074;
pub const fnMaridiaSnail_Main: u32 = 10735204;
pub const fnnullsub_215: u32 = 10735455;
pub const fnMaridiaSnail_Func_7: u32 = 10735456;
pub const fnMaridiaSnail_Func_9: u32 = 10735526;
pub const fnMaridiaSnail_CFB7: u32 = 10735543;
pub const fnMaridiaSnail_CFBD: u32 = 10735549;
pub const fnMaridiaSnail_CFCE: u32 = 10735566;
pub const fnMaridiaSnail_CFD4: u32 = 10735572;
pub const fnMaridiaSnail_CFE5: u32 = 10735589;
pub const fnMaridiaSnail_CFEB: u32 = 10735595;
pub const fnMaridiaSnail_CFFC: u32 = 10735612;
pub const fnMaridiaSnail_Func_15: u32 = 10736051;
pub const fnMaridiaSnail_Touch: u32 = 10736560;
pub const fnMaridiaSnail_Shot: u32 = 10736745;
pub const fnReflec_Func_1: u32 = 10738444;
pub const fnReflec_Instr_1: u32 = 10738632;
pub const fnReflec_Init: u32 = 10738643;
pub const fnnullsub_33: u32 = 10738715;
pub const fnReflec_Shot: u32 = 10738716;
pub const fnWreckedShipOrangeZoomer_Func_1: u32 = 10739650;
pub const fnWreckedShipOrangeZoomer_Init: u32 = 10739779;
pub const fnnullsub_343: u32 = 10739850;
pub const fnWreckedShipOrangeZoomer_Func_2: u32 = 10739857;
pub const fnWreckedShipOrangeZoomer_Main: u32 = 10739851;
pub const fnsub_A3E168: u32 = 10740072;
pub const fnBigEyeBugs_Init: u32 = 10740436;
pub const fnFireZoomer_Init: u32 = 10741148;
pub const fnZoomer_Instr_SetPreinstr: u32 = 10741344;
pub const fnStoneZoomer_Init: u32 = 10741353;
pub const fnnullsub_304: u32 = 10741441;
pub const fnStoneZoomer_Main: u32 = 10741442;
pub const fnFireZoomer_Func_1: u32 = 10741448;
pub const fnFireZoomer_Func_2: u32 = 10741637;
pub const fnFireZoomer_Func_3: u32 = 10741746;
pub const fnMetroid_Init: u32 = 10742351;
pub const fnMetroid_Instr_2: u32 = 10742437;
pub const fnMetroid_Instr_1: u32 = 10742449;
pub const fnMetroid_Frozen: u32 = 10742502;
pub const fnMetroid_Hurt: u32 = 10742579;
pub const fnMetroid_Main: u32 = 10742680;
pub const fnMetroid_Touch: u32 = 10743275;
pub const fnMetroid_Shot: u32 = 10743559;
pub const fnMetroid_Powerbomb: u32 = 10743874;
pub const fnEnemy_GrappleReact_SamusLatchesOn_A4: u32 = 10780677;
pub const fnEnemy_NormalTouchAI_A4: u32 = 10780707;
pub const fnEnemy_NormalShotAI_A4: u32 = 10780717;
pub const fnEnemy_NormalPowerBombAI_A4: u32 = 10780727;
pub const fnEnemy_NormalFrozenAI_A4: u32 = 10780737;
pub const fnnullsub_170_A4: u32 = 10780748;
pub const fnEnemyInstr_Goto_A4: u32 = 10780909;
pub const fnEnemyInstr_Sleep_A4: u32 = 10780975;
pub const fnCrocomire_Hurt: u32 = 10782343;
pub const fnCrocomire_Instr_1: u32 = 10782374;
pub const fnCrocomire_Instr_14: u32 = 10782546;
pub const fnCrocomire_Init: u32 = 10783322;
pub const fnCrocomire_Main: u32 = 10783748;
pub const fnCrocomire_Instr_11: u32 = 10783995;
pub const fnCrocomire_Instr_7: u32 = 10784007;
pub const fnCrocomire_Instr_19: u32 = 10784019;
pub const fnCrocomire_Instr_2: u32 = 10784711;
pub const fnCrocomire_Instr_4: u32 = 10784735;
pub const fnCrocomire_Instr_3: u32 = 10784762;
pub const fnCrocomire_Instr_15: u32 = 10784767;
pub const fnCrocomire_Instr_16: u32 = 10784797;
pub const fnCrocomire_Instr_13: u32 = 10784859;
pub const fnCrocomire_Instr_18: u32 = 10784895;
pub const fnCrocomire_Instr_12: u32 = 10784911;
pub const fnCrocomire_Instr_17: u32 = 10784916;
pub const fnCrocomire_Instr_8: u32 = 10787483;
pub const fnCrocomire_Instr_6: u32 = 10787488;
pub const fnCrocomire_Instr_9: u32 = 10787493;
pub const fnCrocomire_Instr_5: u32 = 10787498;
pub const fnCrocomire_Instr_20: u32 = 10787503;
pub const fnCrocomire_Instr_21: u32 = 10787508;
pub const fnCrocomire_Instr_22: u32 = 10787513;
pub const fnCrocomire_Instr_23: u32 = 10787518;
pub const fnCrocomire_Instr_24: u32 = 10787523;
pub const fnCrocomire_Instr_10: u32 = 10787528;
pub const fnCrocomire_Instr_25: u32 = 10787533;
pub const fnCrocomire_Instr_26: u32 = 10787538;
pub const fnCrocomire_Instr_27: u32 = 10787543;
pub const fnCrocomire_Func_92: u32 = 10795325;
pub const fnnullsub_34: u32 = 10795344;
pub const fnCrocomire_Func_93: u32 = 10795345;
pub const fnCrocomire_Func_94: u32 = 10795368;
pub const fnCrocomire_Powerbomb: u32 = 10795410;
pub const fnCrocomire_Func_95: u32 = 10795525;
pub const fnCrocomire_Func_1: u32 = 10795700;
pub const fnCrocomireTongue_Init: u32 = 10811002;
pub const fnCrocomireTongue_Main: u32 = 10811067;
pub const fnEnemy_GrappleReact_NoInteract_A5: u32 = 10846208;
pub const fnEnemy_GrappleReact_CancelBeam_A5: u32 = 10846223;
pub const fnEnemy_NormalTouchAI_A5: u32 = 10846243;
pub const fnEnemy_NormalShotAI_A5: u32 = 10846253;
pub const fnEnemy_NormalPowerBombAI_A5: u32 = 10846263;
pub const fnEnemy_NormalFrozenAI_A5: u32 = 10846273;
pub const fnCreateADudShot_A5: u32 = 10846278;
pub const fnnullsub_170_A5: u32 = 10846284;
pub const fnEnemyInstr_StopScript_A5: u32 = 10846332;
pub const fnEnemyInstr_Goto_A5: u32 = 10846445;
pub const fnEnemyInstr_DecTimerAndGoto2_A5: u32 = 10846480;
pub const fnEnemyInstr_SetTimer_A5: u32 = 10846499;
pub const fnEnemyInstr_Sleep_A5: u32 = 10846511;
pub const fnEnemyInstr_WaitNframes_A5: u32 = 10846522;
pub const fnDraygon_Init: u32 = 10847879;
pub const fnDraygon_Main: u32 = 10847996;
pub const fnDraygon_Func_36: u32 = 10851138;
pub const fnDraygon_Instr_1: u32 = 10851549;
pub const fnDraygon_Hurt: u32 = 10851661;
pub const fnDraygon_Touch: u32 = 10851818;
pub const fnDraygon_Shot: u32 = 10851824;
pub const fnDraygon_Powerbomb: u32 = 10851847;
pub const fnDraygon_Instr_13: u32 = 10852150;
pub const fnDraygon_Instr_8: u32 = 10852159;
pub const fnDraygon_Instr_7: u32 = 10852178;
pub const fnDraygon_Instr_6: u32 = 10852197;
pub const fnDraygon_Instr_9: u32 = 10852216;
pub const fnDraygon_Instr_2: u32 = 10852501;
pub const fnDraygon_Instr_11: u32 = 10852563;
pub const fnDraygon_Instr_5: u32 = 10852591;
pub const fnDraygon_Instr_15: u32 = 10853274;
pub const fnDraygon_Instr_17: u32 = 10853514;
pub const fnDraygon_Instr_14: u32 = 10853898;
pub const fnDraygon_Instr_16: u32 = 10854231;
pub const fnDraygon_Instr_10: u32 = 10854240;
pub const fnDraygon_Instr_4: u32 = 10854254;
pub const fnDraygon_Instr_12: u32 = 10854268;
pub const fnDraygon_Instr_18: u32 = 10854318;
pub const fnDraygonsEye_Init: u32 = 10863723;
pub const fnDraygon_Instr_3: u32 = 10863739;
pub const fnDraygonsEye_Main: u32 = 10863750;
pub const fnDraygonsTail_Init: u32 = 10864025;
pub const fnnullsub_37: u32 = 10864042;
pub const fnDraygonsArms_Init: u32 = 10864045;
pub const fnnullsub_38: u32 = 10864068;
pub const fnDraygon_Instr_25: u32 = 10872671;
pub const fnDraygon_Instr_24: u32 = 10872689;
pub const fnDraygon_Instr_21: u32 = 10872877;
pub const fnDraygon_Instr_22: u32 = 10872946;
pub const fnDraygon_Instr_27: u32 = 10872956;
pub const fnDraygon_Instr_23: u32 = 10872981;
pub const fnDraygon_Instr_30: u32 = 10873009;
pub const fnDraygon_Instr_20: u32 = 10873018;
pub const fnDraygon_Instr_29: u32 = 10873034;
pub const fnDraygon_Instr_19: u32 = 10873116;
pub const fnDraygon_Instr_28: u32 = 10873198;
pub const fnDraygon_Instr_26: u32 = 10873265;
pub const fnSporeSpawn_Init: u32 = 10873386;
pub const fnSporeSpawn_Main: u32 = 10873619;
pub const fnSporeSpawn_Shot: u32 = 10874202;
pub const fnSporeSpawn_Touch: u32 = 10874348;
pub const fnnullsub_39: u32 = 10874354;
pub const fnEnemy_GrappleReact_NoInteract_A6: u32 = 10911744;
pub const fnEnemy_GrappleReact_CancelBeam_A6: u32 = 10911759;
pub const fnEnemy_NormalTouchAI_A6: u32 = 10911779;
pub const fnEnemy_NormalShotAI_A6: u32 = 10911789;
pub const fnEnemy_NormalPowerBombAI_A6: u32 = 10911799;
pub const fnEnemy_NormalFrozenAI_A6: u32 = 10911809;
pub const fnnullsub_170_A6: u32 = 10911820;
pub const fnEnemyInstr_Goto_A6: u32 = 10911981;
pub const fnEnemyInstr_Sleep_A6: u32 = 10912047;
pub const fnBoulder_Init: u32 = 10913525;
pub const fnBoulder_Main: u32 = 10913683;
pub const fnSpikeyPlatform_Init: u32 = 10914607;
pub const fnSpikeyPlatform2ndEnemy_Init: u32 = 10914693;
pub const fnSpikeyPlatform2ndEnemy_Main: u32 = 10914713;
pub const fnSpikeyPlatform_Main: u32 = 10914733;
pub const fnFireGeyser_Instr_1: u32 = 10915247;
pub const fnFireGeyser_Instr_2: u32 = 10915347;
pub const fnFireGeyser_Instr_3: u32 = 10915373;
pub const fnFireGeyser_Instr_4: u32 = 10915393;
pub const fnFireGeyser_Instr_5: u32 = 10915413;
pub const fnFireGeyser_Instr_6: u32 = 10915433;
pub const fnFireGeyser_Instr_7: u32 = 10915453;
pub const fnFireGeyser_Instr_8: u32 = 10915473;
pub const fnFireGeyser_Instr_9: u32 = 10915493;
pub const fnFireGeyser_Instr_10: u32 = 10915513;
pub const fnFireGeyser_Instr_11: u32 = 10915533;
pub const fnFireGeyser_Instr_12: u32 = 10915553;
pub const fnFireGeyser_Instr_13: u32 = 10915573;
pub const fnFireGeyser_Instr_14: u32 = 10915593;
pub const fnFireGeyser_Instr_15: u32 = 10915613;
pub const fnFireGeyser_Instr_16: u32 = 10915633;
pub const fnFireGeyser_Instr_17: u32 = 10915653;
pub const fnFireGeyser_Instr_18: u32 = 10915673;
pub const fnFireGeyser_Instr_19: u32 = 10915693;
pub const fnFireGeyser_Instr_20: u32 = 10915713;
pub const fnFireGeyser_Instr_21: u32 = 10915733;
pub const fnFireGeyser_Instr_22: u32 = 10915753;
pub const fnFireGeyser_Instr_23: u32 = 10915773;
pub const fnFireGeyser_Instr_24: u32 = 10915793;
pub const fnFireGeyser_Init: u32 = 10915836;
pub const fnFireGeyser_Main: u32 = 10915875;
pub const fnNuclearWaffle_Init: u32 = 10917060;
pub const fnNuclearWaffle_Main: u32 = 10917390;
pub const fnFakeKraid_Init: u32 = 10918488;
pub const fnFakeKraid_Main: u32 = 10918594;
pub const fnFakeKraid_Instr_2: u32 = 10918694;
pub const fnFakeKraid_Instr_1: u32 = 10918772;
pub const fnFakeKraid_Instr_3: u32 = 10918834;
pub const fnFakeKraid_Instr_4: u32 = 10918852;
pub const fnFakeKraid_Instr_5: u32 = 10918914;
pub const fnFakeKraid_Touch: u32 = 10918946;
pub const fnFakeKraid_Shot: u32 = 10918969;
pub const fnCeresRidley_Init: u32 = 10920181;
pub const fnCeresRidley_Main: u32 = 10920584;
pub const fnCeresRidley_Hurt: u32 = 10920659;
pub const fnRidley_A2F2: u32 = 10920690;
pub const fnRidley_Main: u32 = 10924583;
pub const fnRidley_Func_2: u32 = 10924682;
pub const fnRidley_Hurt: u32 = 10924695;
pub const fnRidleysExplosion_Init: u32 = 10929814;
pub const fnRidleysExplosion_Main: u32 = 10930388;
pub const fnsub_A6DF59: u32 = 10936153;
pub const fnRidley_Shot: u32 = 10936202;
pub const fnRidley_Powerbomb: u32 = 10936242;
pub const fnRidley_Instr_5: u32 = 10937534;
pub const fnRidley_Instr_6: u32 = 10937546;
pub const fnRidley_Instr_10: u32 = 10937554;
pub const fnRidley_Instr_4: u32 = 10937582;
pub const fnRidley_Instr_3: u32 = 10937592;
pub const fnRidley_Instr_2: u32 = 10937601;
pub const fnRidley_Instr_1: u32 = 10937623;
pub const fnRidley_Instr_14: u32 = 10937631;
pub const fnRidley_Instr_9: u32 = 10938140;
pub const fnRidley_Instr_7: u32 = 10938151;
pub const fnRidley_Instr_8: u32 = 10938159;
pub const fnRidley_Instr_11: u32 = 10938445;
pub const fnRidley_Instr_12: u32 = 10938628;
pub const fnRidley_Instr_13: u32 = 10938633;
pub const fnRidley_Instr_15: u32 = 10938729;
pub const fnRidley_Instr_16: u32 = 10938742;
pub const fnCeresSteam_Init: u32 = 10940337;
pub const fnCeresSteam_Main: u32 = 10940429;
pub const fnCeresSteam_Touch: u32 = 10940479;
pub const fnCeresSteam_Instr_1: u32 = 10940701;
pub const fnCeresSteam_Instr_2: u32 = 10940711;
pub const fnCeresSteam_Instr_3: u32 = 10940725;
pub const fnCeresDoor_Instr_6: u32 = 10942014;
pub const fnCeresDoor_Instr_4: u32 = 10942058;
pub const fnCeresDoor_Instr_8: u32 = 10942072;
pub const fnCeresSteam_Instr_4: u32 = 10942091;
pub const fnCeresDoor_Instr_1: u32 = 10942101;
pub const fnCeresDoor_Instr_3: u32 = 10942111;
pub const fnCeresSteam_Instr_5: u32 = 10942118;
pub const fnCeresDoor_Instr_5: u32 = 10942128;
pub const fnCeresDoor_Instr_2: u32 = 10942131;
pub const fnCeresDoor_Instr_7: u32 = 10942141;
pub const fnCeresDoor_Init: u32 = 10942149;
pub const fnCeresDoor_Main: u32 = 10942309;
pub const fnnullsub_41: u32 = 10942752;
pub const fnZebetites_Init: u32 = 10943346;
pub const fnZebetites_Main: u32 = 10943539;
pub const fnZebetites_Touch: u32 = 10943911;
pub const fnZebetites_Shot: u32 = 10943916;
pub const fnEnemy_GrappleReact_NoInteract_A7: u32 = 10977280;
pub const fnEnemy_GrappleReact_CancelBeam_A7: u32 = 10977295;
pub const fnEnemy_NormalShotAI_A7: u32 = 10977325;
pub const fnEnemy_NormalPowerBombAI_A7: u32 = 10977335;
pub const fnEnemy_NormalPowerBombAI_SkipDeathAnim_A7: u32 = 10977340;
pub const fnEnemy_NormalFrozenAI_A7: u32 = 10977345;
pub const fnnullsub_170_A7: u32 = 10977356;
pub const fnEnemyInstr_Call_A7: u32 = 10977418;
pub const fnEnemyInstr_Goto_A7: u32 = 10977517;
pub const fnEnemyInstr_DecTimerAndGoto2_A7: u32 = 10977552;
pub const fnEnemyInstr_SetTimer_A7: u32 = 10977571;
pub const fnEnemyInstr_Sleep_A7: u32 = 10977583;
pub const fnKraid_Instr_9: u32 = 10979983;
pub const fnKraid_Touch_ArmFoot: u32 = 10982539;
pub const fnnullsub_44: u32 = 10982543;
pub const fnKraidsArm_Touch: u32 = 10982544;
pub const fnKraid_Touch: u32 = 10982559;
pub const fnnullsub_43: u32 = 10982581;
pub const fnKraid_Arm_Shot: u32 = 10982582;
pub const fnKraid_Init: u32 = 10987865;
pub const fnKraidsArm_Init: u32 = 10988355;
pub const fnKraidsTopLint_Init: u32 = 10988392;
pub const fnKraidsMiddleLint_Init: u32 = 10988444;
pub const fnKraidsBottomLint_Init: u32 = 10988490;
pub const fnKraidsFoot_Init: u32 = 10988536;
pub const fnKraid_Main: u32 = 10988577;
pub const fnKraid_Instr_1: u32 = 10991155;
pub const fnKraid_Instr_DecYpos: u32 = 10991158;
pub const fnKraid_Instr_IncrYpos_Shake: u32 = 10991164;
pub const fnKraid_Instr_PlaySound_0x76: u32 = 10991182;
pub const fnKraid_Instr_XposMinus3: u32 = 10991194;
pub const fnKraid_Instr_XposMinus3b: u32 = 10991207;
pub const fnKraid_Instr_XposPlus3: u32 = 10991220;
pub const fnKraid_Instr_MoveHimRight: u32 = 10991235;
pub const fnKraidsArm_Main: u32 = 10991549;
pub const fnKraidsTopLint_Main: u32 = 10991617;
pub const fnKraidsMiddleLint_Main: u32 = 10991629;
pub const fnKraidsBottomLint_Main: u32 = 10991641;
pub const fnKraidsFoot_Main: u32 = 10992118;
pub const fnKraidsGoodFingernail_Touch: u32 = 10992847;
pub const fnKraidsBadFingernail_Touch: u32 = 10992862;
pub const fnKraidsGoodFingernail_Init: u32 = 10992879;
pub const fnKraidsBadFingernail_Init: u32 = 10992941;
pub const fnKraidsGoodFingernail_Main: u32 = 10992946;
pub const fnKraidsBadFingernail_Main: u32 = 10992969;
pub const fnPhantoon_Init: u32 = 10997235;
pub const fnPhantoon2_Init: u32 = 10997333;
pub const fnPhantoon_Main: u32 = 10997414;
pub const fnnullsub_237: u32 = 10998952;
pub const fnPhantoon_Spawn8FireballsInCircleAtStart: u32 = 10998953;
pub const fnPhantoon_WaitBetweenSpawningAndSpinningFireballs: u32 = 10999022;
pub const fnPhantoon_SpawnFireballsBeforeFight: u32 = 10999048;
pub const fnPhantoon_WavyFadeIn: u32 = 10999114;
pub const fnPhantoon_PickPatternForRound1: u32 = 10999190;
pub const fnPhantoon_MovePhantoonInFigure8ThenOpenEye: u32 = 10999271;
pub const fnPhantoon_EyeFollowsSamusUntilTimerRunsOut: u32 = 10999309;
pub const fnPhantoon_BecomesSolidAndBodyVuln: u32 = 10999388;
pub const fnPhantoon_IsSwooping: u32 = 10999416;
pub const fnPhantoon_FadeoutWithSwoop: u32 = 10999481;
pub const fnPhantoon_WaitAfterFadeOut: u32 = 10999508;
pub const fnPhantoon_MoveLeftOrRightAndPickEyeOpenPatt: u32 = 10999522;
pub const fnPhantoon_FadeInBeforeFigure8: u32 = 10999597;
pub const fnPhantoon_BecomeSolidAfterRainingFireballs: u32 = 10999615;
pub const fnPhantoon_FadeInDuringFireballRain: u32 = 10999655;
pub const fnPhantoon_FollowSamusWithEyeDuringFireballRain: u32 = 10999688;
pub const fnPhantoon_FadeOutDuringFireballRain: u32 = 10999765;
pub const fnPhantoon_SpawnRainingFireballs: u32 = 10999799;
pub const fnPhantoon_FadeOutBeforeFirstFireballRain: u32 = 10999850;
pub const fnPhantoon_FadeOutBeforeEnrage: u32 = 10999900;
pub const fnPhantoon_MoveEnragedPhantoonToTopCenter: u32 = 10999924;
pub const fnPhantoon_FadeInEnragedPhantoon: u32 = 10999953;
pub const fnPhantoon_Enraged: u32 = 10999980;
pub const fnPhantoon_FadeoutAfterEnrage: u32 = 11000086;
pub const fnPhantoon_CompleteSwoopAfterFatalShot: u32 = 11000110;
pub const fnPhantoon_DyingPhantoonFadeInOut: u32 = 11000136;
pub const fnPhantoon_DyingPhantoonExplosions: u32 = 11000203;
pub const fnPhantoon_WavyDyingPhantoonAndCry: u32 = 11000401;
pub const fnPhantoon_DyingFadeOut: u32 = 11000454;
pub const fnPhantoon_AlmostDead: u32 = 11000535;
pub const fnPhantoon_Dead: u32 = 11000637;
pub const fnPhantoon_Hurt: u32 = 11001151;
pub const fnPhantoon_Touch: u32 = 11001237;
pub const fnnullsub_358: u32 = 11001242;
pub const fnPhantoon_Shot: u32 = 11001243;
pub const fnnullsub_45: u32 = 11001873;
pub const fnEtecoon_Init: u32 = 11004178;
pub const fnEtecoon_Main: u32 = 11004224;
pub const fnEtecoon_Func_4: u32 = 11004335;
pub const fnEtecoon_Func_5: u32 = 11004416;
pub const fnEtecoon_Func_6: u32 = 11004471;
pub const fnEtecoon_Func_7: u32 = 11004597;
pub const fnEtecoon_Func_8: u32 = 11004674;
pub const fnEtecoon_Func_9: u32 = 11004716;
pub const fnEtecoon_Func_10: u32 = 11004752;
pub const fnEtecoon_Func_11: u32 = 11004877;
pub const fnEtecoon_Func_12: u32 = 11004955;
pub const fnEtecoon_Func_16: u32 = 11005079;
pub const fnEtecoon_Func_17: u32 = 11005115;
pub const fnEtecoon_Func_18: u32 = 11005151;
pub const fnEtecoon_Func_19: u32 = 11005193;
pub const fnEtecoon_Func_20: u32 = 11005226;
pub const fnEtecoon_Func_21: u32 = 11005268;
pub const fnEtecoon_Func_22: u32 = 11005301;
pub const fnEtecoon_Func_23: u32 = 11005383;
pub const fnEtecoon_Func_24: u32 = 11005502;
pub const fnEtecoon_Func_25: u32 = 11005594;
pub const fnEtecoon_Func_26: u32 = 11005624;
pub const fnDachora_Init: u32 = 11007197;
pub const fnDachora_Main: u32 = 11007278;
pub const fnDachora_Func_2: u32 = 11007344;
pub const fnDachora_Func_3: u32 = 11007420;
pub const fnDachora_Func_4: u32 = 11007469;
pub const fnDachora_Func_5: u32 = 11007582;
pub const fnDachora_Func_7: u32 = 11007887;
pub const fnDachora_Func_8: u32 = 11008006;
pub const fnDachora_Func_11: u32 = 11008309;
pub const fnDachora_Func_12: u32 = 11008396;
pub const fnEnemy_GrappleReact_NoInteract_A8: u32 = 11042816;
pub const fnEnemy_GrappleReact_KillEnemy_A8: u32 = 11042826;
pub const fnEnemy_GrappleReact_CancelBeam_A8: u32 = 11042831;
pub const fnEnemy_GrappleReact_SamusLatchesNoInvinc_A8: u32 = 11042836;
pub const fnEnemy_GrappleReact_HurtSamus_A8: u32 = 11042846;
pub const fnEnemy_NormalTouchAI_A8: u32 = 11042851;
pub const fnEnemy_NormalShotAI_A8: u32 = 11042861;
pub const fnEnemy_NormalPowerBombAI_A8: u32 = 11042871;
pub const fnEnemy_NormalFrozenAI_A8: u32 = 11042881;
pub const fnnullsub_170_A8: u32 = 11042892;
pub const fnEnemyInstr_Goto_A8: u32 = 11043053;
pub const fnEnemyInstr_DecTimerAndGoto2_A8: u32 = 11043088;
pub const fnEnemyInstr_SetTimer_A8: u32 = 11043107;
pub const fnEnemyInstr_Sleep_A8: u32 = 11043119;
pub const fnEnemyInstr_EnableOffScreenProcessing_A8: u32 = 11043187;
pub const fnEnemyInstr_DisableOffScreenProcessing_A8: u32 = 11043197;
pub const fnMiniDraygon_Instr_2: u32 = 11044751;
pub const fnMiniDraygon_Instr_1: u32 = 11044763;
pub const fnMiniDraygon_Instr_3: u32 = 11044790;
pub const fnMiniDraygon_Instr_4: u32 = 11044811;
pub const fnMiniDraygon_Init: u32 = 11044832;
pub const fnEvirProjectile_Init: u32 = 11045040;
pub const fnMiniDraygon_Main: u32 = 11045147;
pub const fnEvirProjectile_Main: u32 = 11045278;
pub const fnMiniDraygon_Touch: u32 = 11045638;
pub const fnMiniDraygon_Powerbomb: u32 = 11045644;
pub const fnMiniDraygon_Shot: u32 = 11045650;
pub const fnMorphBallEye_Init: u32 = 11047000;
pub const fnMorphBallEye_Main: u32 = 11047138;
pub const fnMorphBallEye_Func_1: u32 = 11047153;
pub const fnMorphBallEye_Func_2: u32 = 11047214;
pub const fnMorphBallEye_Func_3: u32 = 11047264;
pub const fnMorphBallEye_Func_4: u32 = 11047374;
pub const fnnullsub_244: u32 = 11047388;
pub const fnFune_Instr_2: u32 = 11048485;
pub const fnFune_Instr_6: u32 = 11048497;
pub const fnFune_Instr_7: u32 = 11048522;
pub const fnFune_Instr_1: u32 = 11048547;
pub const fnFune_Instr_4: u32 = 11048572;
pub const fnFune_Instr_3: u32 = 11048597;
pub const fnFune_Instr_5: u32 = 11048628;
pub const fnFune_Init: u32 = 11048675;
pub const fnFune_Main: u32 = 11048752;
pub const fnWreckedShipGhost_Init: u32 = 11049710;
pub const fnWreckedShipGhost_Main: u32 = 11049788;
pub const fnYappingMaw_Instr_2: u32 = 11051207;
pub const fnYappingMaw_Instr_4: u32 = 11051225;
pub const fnYappingMaw_Instr_5: u32 = 11051243;
pub const fnYappingMaw_Instr_7: u32 = 11051261;
pub const fnYappingMaw_Instr_3: u32 = 11051279;
pub const fnYappingMaw_Instr_6: u32 = 11051297;
pub const fnYappingMaw_Instr_1: u32 = 11051315;
pub const fnYappingMaw_Init: u32 = 11051336;
pub const fnYappingMaw_Main: u32 = 11051537;
pub const fnYappingMaw_Touch: u32 = 11052953;
pub const fnYappingMaw_Shot: u32 = 11052989;
pub const fnYappingMaw_Frozen: u32 = 11053109;
pub const fnKago_Init: u32 = 11053894;
pub const fnKago_Main: u32 = 11053941;
pub const fnKago_Shot: u32 = 11053955;
pub const fnNorfairLavaMan_Instr_1: u32 = 11054610;
pub const fnNorfairLavaMan_Instr_8: u32 = 11054630;
pub const fnNorfairLavaMan_Instr_14: u32 = 11054640;
pub const fnNorfairLavaMan_Instr_2: u32 = 11054650;
pub const fnNorfairLavaMan_Instr_7: u32 = 11054661;
pub const fnNorfairLavaMan_Instr_10: u32 = 11054672;
pub const fnNorfairLavaMan_Instr_12: u32 = 11054682;
pub const fnNorfairLavaMan_Instr_9: u32 = 11054692;
pub const fnNorfairLavaMan_Instr_11: u32 = 11054728;
pub const fnNorfairLavaMan_Instr_13: u32 = 11054742;
pub const fnNorfairLavaMan_Instr_5: u32 = 11054778;
pub const fnNorfairLavaMan_Instr_15: u32 = 11054794;
pub const fnNorfairLavaMan_Instr_4: u32 = 11054820;
pub const fnNorfairLavaMan_Instr_16: u32 = 11054846;
pub const fnNorfairLavaMan_Instr_6: u32 = 11054872;
pub const fnNorfairLavaMan_Instr_3: u32 = 11054916;
pub const fnNorfairLavaMan_Init: u32 = 11054987;
pub const fnNorfairLavaMan_Func_6: u32 = 11055282;
pub const fnNorfairLavaMan_Main: u32 = 11055370;
pub const fnNorfairLavaMan_Func_7: u32 = 11055386;
pub const fnNorfairLavaMan_Func_8: u32 = 11055477;
pub const fnsub_A8B193: u32 = 11055507;
pub const fnNorfairLavaMan_Func_9: u32 = 11055544;
pub const fnNorfairLavaMan_Func_10: u32 = 11055581;
pub const fnNorfairLavaMan_Func_11: u32 = 11055620;
pub const fnsub_A8B291: u32 = 11055761;
pub const fnNorfairLavaMan_Func_13: u32 = 11055765;
pub const fnNorfairLavaMan_Func_15: u32 = 11055885;
pub const fnNorfairLavaMan_Func_16: u32 = 11055903;
pub const fnNorfairLavaMan_Func_17: u32 = 11055958;
pub const fnNorfairLavaMan_Func_18: u32 = 11056039;
pub const fnNorfairLavaMan_Powerbomb: u32 = 11056128;
pub const fnNorfairLavaMan_Touch: u32 = 11056134;
pub const fnNorfairLavaMan_Shot: u32 = 11056140;
pub const fnBeetom_Instr_1: u32 = 11056990;
pub const fnBeetom_Init: u32 = 11057014;
pub const fnBeetom_Main: u32 = 11057165;
pub const fnBeetom_Touch: u32 = 11058734;
pub const fnBeetom_Shot: u32 = 11058860;
pub const fnMaridiaFloater_Init: u32 = 11059657;
pub const fnMaridiaFloater_Main: u32 = 11059740;
pub const fnMaridiaFloater_Func_3: u32 = 11059843;
pub const fnMaridiaFloater_Func_4: u32 = 11059878;
pub const fnMaridiaFloater_Func_5: u32 = 11059919;
pub const fnMaridiaFloater_Func_6: u32 = 11060075;
pub const fnMaridiaFloater_Func_7: u32 = 11060193;
pub const fnMaridiaFloater_Func_8: u32 = 11060329;
pub const fnMaridiaFloater_Func_9: u32 = 11060444;
pub const fnMaridiaFloater_Func_10: u32 = 11060480;
pub const fnMaridiaFloater_Func_11: u32 = 11060509;
pub const fnnullsub_256: u32 = 11060584;
pub const fnMaridiaFloater_Func_12: u32 = 11060585;
pub const fnMaridiaFloater_Func_13: u32 = 11060639;
pub const fnMaridiaFloater_Touch: u32 = 11060670;
pub const fnMaridiaFloater_Shot: u32 = 11060719;
pub const fnMaridiaFloater_Powerbomb: u32 = 11060799;
pub const fnWreckedShipRobot_Init: u32 = 11062135;
pub const fnWreckedShipRobotDeactivated_Init: u32 = 11062220;
pub const fnWreckedShipRobot_Main: u32 = 11062326;
pub const fnnullsub_342: u32 = 11062374;
pub const fnWreckedShipRobot_Func_1: u32 = 11062375;
pub const fnWreckedShipRobot_Instr_4: u32 = 11062537;
pub const fnWreckedShipRobot_Instr_9: u32 = 11062692;
pub const fnWreckedShipRobot_Instr_6: u32 = 11062762;
pub const fnWreckedShipRobot_Instr_8: u32 = 11062917;
pub const fnWreckedShipRobot_Instr_7: u32 = 11062987;
pub const fnWreckedShipRobot_Instr_15: u32 = 11062991;
pub const fnWreckedShipRobot_Instr_18: u32 = 11063146;
pub const fnWreckedShipRobot_Instr_16: u32 = 11063216;
pub const fnWreckedShipRobot_Instr_17: u32 = 11063371;
pub const fnWreckedShipRobot_Instr_3: u32 = 11063441;
pub const fnWreckedShipRobot_Instr_10: u32 = 11063490;
pub const fnWreckedShipRobot_Instr_14: u32 = 11063494;
pub const fnWreckedShipRobot_Instr_2: u32 = 11063506;
pub const fnWreckedShipRobot_Instr_13: u32 = 11063552;
pub const fnWreckedShipRobot_Instr_1: u32 = 11063559;
pub const fnWreckedShipRobot_Instr_12: u32 = 11063601;
pub const fnWreckedShipRobot_Instr_5: u32 = 11063613;
pub const fnWreckedShipRobot_Instr_11: u32 = 11063659;
pub const fnWreckedShipRobotDeactivated_Touch: u32 = 11063668;
pub const fnWreckedShipRobotDeactivated_Shot: u32 = 11063693;
pub const fnWreckedShipRobot_Shot: u32 = 11063698;
pub const fnMaridiaPuffer_Init: u32 = 11065545;
pub const fnMaridiaPuffer_Main: u32 = 11065611;
pub const fnMaridiaPuffer_Shot: u32 = 11066132;
pub const fnWalkingLavaSeahorse_Init: u32 = 11066573;
pub const fnWalkingLavaSeahorse_Main: u32 = 11066731;
pub const fnWalkingLavaSeahorse_Instr_4: u32 = 11067164;
pub const fnWalkingLavaSeahorse_Instr_3: u32 = 11067187;
pub const fnWalkingLavaSeahorse_Instr_5: u32 = 11067193;
pub const fnWalkingLavaSeahorse_Instr_6: u32 = 11067199;
pub const fnWalkingLavaSeahorse_Instr_2: u32 = 11067235;
pub const fnWalkingLavaSeahorse_Instr_1: u32 = 11067249;
pub const fnWreckedShipOrbs_Init: u32 = 11068296;
pub const fnWreckedShipOrbs_Main: u32 = 11068355;
pub const fnWreckedShipSpark_Instr_2: u32 = 11068957;
pub const fnWreckedShipSpark_Instr_1: u32 = 11068970;
pub const fnWreckedShipSpark_Init: u32 = 11068983;
pub const fnWreckedShipSpark_Main: u32 = 11069070;
pub const fnWreckedShipSpark_Shot: u32 = 11069198;
pub const fnBlueBrinstarFaceBlock_Init: u32 = 11069486;
pub const fnBlueBrinstarFaceBlock_Func_1: u32 = 11069550;
pub const fnBlueBrinstarFaceBlock_Main: u32 = 11069614;
pub const fnBlueBrinstarFaceBlock_Shot: u32 = 11069725;
pub const fnKiHunter_Init: u32 = 11071880;
pub const fnKiHunterWings_Init: u32 = 11072020;
pub const fnKiHunter_Main: u32 = 11072092;
pub const fnKiHunterWings_Main: u32 = 11072098;
pub const fnKiHunter_Instr_1: u32 = 11072806;
pub const fnKiHunter_Instr_2: u32 = 11072996;
pub const fnKiHunter_Instr_3: u32 = 11073151;
pub const fnKiHunter_Instr_4: u32 = 11073234;
pub const fnKiHunter_Instr_5: u32 = 11073240;
pub const fnKiHunter_Shot: u32 = 11073281;
pub const fnEnemy_GrappleReact_CancelBeam_A9: u32 = 11108367;
pub const fnEnemy_NormalPowerBombAI_A9: u32 = 11108407;
pub const fnEnemy_NormalFrozenAI_A9: u32 = 11108417;
pub const fnnullsub_170_A9: u32 = 11108428;
pub const fnEnemyInstr_Sleep_A9: u32 = 11108655;
pub const fnMotherBrainsBody_Init: u32 = 11110023;
pub const fnMotherBrainsBrain_Init: u32 = 11110149;
pub const fnMotherBrainsBody_Hurt: u32 = 11110206;
pub const fnnullsub_264: u32 = 11110278;
pub const fnMotherBrainsBody_Powerbomb: u32 = 11110279;
pub const fnMotherBrainsBrain_Hurt: u32 = 11110283;
pub const fnMotherBrain_DrawBrainNeck_EnemyGfxDrawHook: u32 = 11110345;
pub const fnMotherBrainsBrain_GfxDrawHook: u32 = 11110365;
pub const fnMotherBrainsTubesFalling_Init: u32 = 11111221;
pub const fnMotherBrainsTubesFalling_Main: u32 = 11111301;
pub const fnMotherBrain_Instr_MoveBodyUp10Left4: u32 = 11113910;
pub const fnMotherBrain_Instr_MoveBodyUp16Left4: u32 = 11113920;
pub const fnMotherBrain_Instr_MoveBodyUp12Right2: u32 = 11113930;
pub const fnMotherBrain_Instr_MoveDown12Left4: u32 = 11113950;
pub const fnMotherBrain_Instr_MoveDown16Right2: u32 = 11113960;
pub const fnMotherBrain_Instr_MoveDown10Right2: u32 = 11113970;
pub const fnMotherBrain_Instr_MoveUp2Right1: u32 = 11113980;
pub const fnMotherBrain_Instr_MoveRight2: u32 = 11113996;
pub const fnMotherBrain_Instr_MoveUp1: u32 = 11114012;
pub const fnMotherBrain_Instr_MoveUp1Right3_Sfx: u32 = 11114018;
pub const fnMotherBrain_Instr_Down2Right15: u32 = 11114040;
pub const fnMotherBrain_Instr_Down4Right6: u32 = 11114056;
pub const fnMotherBrain_Instr_Up4Left2: u32 = 11114072;
pub const fnMotherBrain_Instr_Up2Left1_Sfx: u32 = 11114088;
pub const fnMotherBrain_Instr_Up2Left1_Sfx2: u32 = 11114110;
pub const fnMotherBrain_Instr_MoveLeft2: u32 = 11114132;
pub const fnMotherBrain_Instr_MoveDown1: u32 = 11114148;
pub const fnMotherBrain_Instr_MoveDown1Left3: u32 = 11114154;
pub const fnMotherBrain_Instr_MoveUp2Left15_Sfx: u32 = 11114170;
pub const fnMotherBrain_Instr_MoveUp4Left6: u32 = 11114192;
pub const fnMotherBrain_Instr_MoveDown4Right2: u32 = 11114208;
pub const fnMotherBrain_Instr_MoveDown2Right1: u32 = 11114224;
pub const fnMotherBrain_Instr_SetPose_Standing: u32 = 11114240;
pub const fnMotherBrain_Instr_SetPose_Walking: u32 = 11114248;
pub const fnMotherBrain_Instr_SetPose_Crouched: u32 = 11114256;
pub const fnMotherBrain_Instr_SetPose_CrouchedTrans: u32 = 11114264;
pub const fnMotherBrain_Instr_SetPose_DeathBeamMode: u32 = 11114272;
pub const fnMotherBrain_Instr_SetPose_LeaningDown: u32 = 11114280;
pub const fnMotherBrain_Instr_SpawnEprojToOffset: u32 = 11115208;
pub const fnMotherBrain_Instr_SpawnDeathBeamEproj: u32 = 11115247;
pub const fnMotherBrain_Instr_IncrBeamAttackPhase: u32 = 11115269;
pub const fnMotherBrainsBody_Shot: u32 = 11121923;
pub const fnMotherBrainsBrain_Shot: u32 = 11121927;
pub const fnnullsub_47: u32 = 11122117;
pub const fnMotherBrainsBrain_Touch: u32 = 11122118;
pub const fnShitroidInCutscene_Init: u32 = 11126544;
pub const fnShitroidInCutscene_Main: u32 = 11126649;
pub const fnShitroidInCutscene_Touch: u32 = 11128579;
pub const fnShitroid_Instr_1: u32 = 11128756;
pub const fnShitroid_Instr_2: u32 = 11128778;
pub const fnDeadTorizo_Init: u32 = 11129608;
pub const fnDeadTorizo_Main: u32 = 11129704;
pub const fnDeadTorizo_MainGfxHook: u32 = 11129754;
pub const fnDeadTorizo_Powerbomb: u32 = 11129898;
pub const fnDeadTorizo_Shot: u32 = 11129907;
pub const fnDeadSidehopper_Init: u32 = 11130806;
pub const fnDeadZoomer_Init: u32 = 11130953;
pub const fnDeadRipper_Init: u32 = 11130998;
pub const fnDeadSkree_Init: u32 = 11131039;
pub const fnDeadSidehopper_Powerbomb: u32 = 11131084;
pub const fnDeadSidehopper_Main: u32 = 11131099;
pub const fnDeadZoomer_Powerbomb: u32 = 11132141;
pub const fnDeadZoomer_Shot: u32 = 11132152;
pub const fnDeadRipper_Powerbomb: u32 = 11132157;
pub const fnDeadRipper_Shot: u32 = 11132168;
pub const fnDeadSkree_Powerbomb: u32 = 11132173;
pub const fnDeadSkree_Shot: u32 = 11132184;
pub const fnDeadSidehopper_Shot: u32 = 11132189;
pub const fnDeadSidehopper_Touch: u32 = 11132228;
pub const fnsub_A9ECD0: u32 = 11136208;
pub const fnShitroid_Init: u32 = 11136823;
pub const fnShitroid_Powerbomb: u32 = 11136954;
pub const fnShitroid_Main: u32 = 11136965;
pub const fnShitroid_Touch: u32 = 11138953;
pub const fnShitroid_Shot: u32 = 11139138;
pub const fnShitroid_Instr_3: u32 = 11139360;
pub const fnShitroid_Instr_4: u32 = 11139382;
pub const fnShitroid_Instr_6: u32 = 11139472;
pub const fnShitroid_Instr_5: u32 = 11139476;
pub const fnEnemy_GrappleReact_CancelBeam_AA: u32 = 11173903;
pub const fnEnemy_NormalPowerBombAI_AA: u32 = 11173943;
pub const fnEnemy_NormalFrozenAI_AA: u32 = 11173953;
pub const fnnullsub_170_AA: u32 = 11173964;
pub const fnEnemy_SetAiPreInstr_AA: u32 = 11173995;
pub const fnEnemy_ClearAiPreInstr_AA: u32 = 11174004;
pub const fnnullsub_171_AA: u32 = 11174011;
pub const fnEnemyInstr_StopScript_AA: u32 = 11174012;
pub const fnEnemyInstr_Goto_AA: u32 = 11174125;
pub const fnEnemyInstr_DecTimerAndGoto2_AA: u32 = 11174160;
pub const fnEnemyInstr_SetTimer_AA: u32 = 11174179;
pub const fnEnemyInstr_Sleep_AA: u32 = 11174191;
pub const fnEnemyInstr_WaitNframes_AA: u32 = 11174202;
pub const fnEnemyInstr_CopyToVram_AA: u32 = 11174219;
pub const fnTorizo_Instr_3: u32 = 11186332;
pub const fnTorizo_Instr_31: u32 = 11186461;
pub const fnTorizo_Instr_33: u32 = 11186622;
pub const fnTorizo_Instr_36: u32 = 11186724;
pub const fnTorizo_Instr_37: u32 = 11186734;
pub const fnTorizo_Instr_35: u32 = 11186744;
pub const fnTorizo_Instr_38: u32 = 11186765;
pub const fnTorizo_Instr_6: u32 = 11186801;
pub const fnTorizo_Instr_5: u32 = 11188557;
pub const fnTorizo_Instr_9: u32 = 11188561;
pub const fnTorizo_Instr_7: u32 = 11190984;
pub const fnTorizo_Instr_2: u32 = 11190985;
pub const fnTorizo_Instr_8: u32 = 11190993;
pub const fnTorizo_Instr_25: u32 = 11191001;
pub const fnTorizo_Instr_22: u32 = 11191021;
pub const fnTorizo_Instr_19: u32 = 11191031;
pub const fnTorizo_Instr_32: u32 = 11191037;
pub const fnTorizo_Instr_30: u32 = 11191043;
pub const fnTorizo_Instr_34: u32 = 11191087;
pub const fnTorizo_Instr_24: u32 = 11191114;
pub const fnTorizo_Instr_12: u32 = 11191131;
pub const fnTorizo_Instr_10: u32 = 11191149;
pub const fnTorizo_Instr_11: u32 = 11191159;
pub const fnTorizo_Instr_29: u32 = 11191178;
pub const fnTorizo_Instr_1: u32 = 11191200;
pub const fnTorizo_Instr_28: u32 = 11191222;
pub const fnTorizo_Instr_4: u32 = 11191244;
pub const fnTorizo_Instr_40: u32 = 11191326;
pub const fnTorizo_Instr_16: u32 = 11191408;
pub const fnTorizo_Instr_27: u32 = 11191525;
pub const fnTorizo_Instr_23: u32 = 11191642;
pub const fnTorizo_Instr_14: u32 = 11191655;
pub const fnTorizo_Instr_15: u32 = 11191691;
pub const fnTorizo_Instr_26: u32 = 11191716;
pub const fnTorizo_Instr_18: u32 = 11191755;
pub const fnTorizo_Instr_20: u32 = 11191779;
pub const fnTorizo_Instr_44: u32 = 11191794;
pub const fnTorizo_Instr_21: u32 = 11191809;
pub const fnTorizo_Instr_17: u32 = 11191824;
pub const fnTorizo_Instr_13: u32 = 11191832;
pub const fnTorizo_Hurt: u32 = 11191934;
pub const fnTorizo_Main: u32 = 11191972;
pub const fnTorizo_Func_5: u32 = 11192146;
pub const fnTorizo_Func_6: u32 = 11192360;
pub const fnTorizo_Func_7: u32 = 11192364;
pub const fnTorizo_Init: u32 = 11192447;
pub const fnnullsub_344: u32 = 11192670;
pub const fnGoldTorizo_Touch: u32 = 11192695;
pub const fnTorizo_Shot: u32 = 11192700;
pub const fnnullsub_271: u32 = 11192769;
pub const fnTorizo_Func_8: u32 = 11192770;
pub const fnTorizo_Instr_39: u32 = 11193038;
pub const fnTorizo_Instr_41: u32 = 11193054;
pub const fnTorizo_Instr_42: u32 = 11193058;
pub const fnTorizo_Instr_48: u32 = 11193815;
pub const fnTorizo_Instr_57: u32 = 11194601;
pub const fnTorizo_Instr_58: u32 = 11194611;
pub const fnTorizo_Instr_59: u32 = 11194747;
pub const fnTorizo_Instr_62: u32 = 11194759;
pub const fnTorizo_Instr_63: u32 = 11194855;
pub const fnGoldTorizo_Main: u32 = 11195241;
pub const fnTorizo_Instr_56: u32 = 11195279;
pub const fnTorizo_Instr_60: u32 = 11195287;
pub const fnTorizo_Instr_46: u32 = 11195295;
pub const fnGoldTorizo_Hurt: u32 = 11195322;
pub const fnTorizo_Instr_47: u32 = 11195360;
pub const fnTorizo_Instr_49: u32 = 11195370;
pub const fnTorizo_Instr_61: u32 = 11195446;
pub const fnTorizo_Instr_53: u32 = 11195461;
pub const fnTorizo_Instr_55: u32 = 11195508;
pub const fnTorizo_Instr_52: u32 = 11195547;
pub const fnTorizo_Instr_50: u32 = 11195578;
pub const fnTorizo_Instr_43: u32 = 11195635;
pub const fnTorizo_Instr_51: u32 = 11195645;
pub const fnTorizo_Instr_45: u32 = 11195686;
pub const fnTorizo_Instr_54: u32 = 11195725;
pub const fnTorizo_D5ED: u32 = 11195885;
pub const fnTorizo_D5F1: u32 = 11195889;
pub const fnGoldTorizo_Shot: u32 = 11196007;
pub const fnnullsub_49: u32 = 11196359;
pub const fnTourianEntranceStatue_Init: u32 = 11196360;
pub const fnShaktool_Instr_2: u32 = 11196721;
pub const fnShaktool_Instr_3: u32 = 11196735;
pub const fnShaktool_Instr_4: u32 = 11196746;
pub const fnShaktool_Instr_5: u32 = 11196755;
pub const fnShaktool_Instr_6: u32 = 11196831;
pub const fnShaktool_Instr_1: u32 = 11196858;
pub const fnShaktool_Hurt: u32 = 11197603;
pub const fnnullsub_274: u32 = 11197610;
pub const fnnullsub_275: u32 = 11197611;
pub const fnShaktool_DCAC: u32 = 11197612;
pub const fnShaktool_DCD7: u32 = 11197655;
pub const fnShaktool_DD25: u32 = 11197733;
pub const fnShaktool_Init: u32 = 11198019;
pub const fnShaktool_Touch: u32 = 11198255;
pub const fnShaktool_Shot: u32 = 11198260;
pub const fnShaktool_Instr_9: u32 = 11199529;
pub const fnShaktool_Instr_11: u32 = 11199542;
pub const fnShaktool_Instr_10: u32 = 11199549;
pub const fnsub_AAE445: u32 = 11199557;
pub const fnShaktool_Instr_8: u32 = 11199871;
pub const fnShaktool_Instr_13: u32 = 11199879;
pub const fnShaktool_Instr_12: u32 = 11199887;
pub const fnShaktool_Instr_7: u32 = 11199960;
pub const fnShaktool_Instr_14: u32 = 11200240;
pub const fnN00bTubeCracks_Init: u32 = 11200278;
pub const fnChozoStatue_Init: u32 = 11200293;
pub const fnnullsub_276: u32 = 11200422;
pub const fnChozoStatue_Main: u32 = 11200423;
pub const fnShaktool_PreInstr_0: u32 = 11200430;
pub const fnnullsub_277: u32 = 11200474;
pub const fnnullsub_51: u32 = 11200475;
pub const fnnullsub_52: u32 = 11200476;
pub const fnEnemy_GrappleReact_CancelBeam_B2: u32 = 11698191;
pub const fnEnemy_NormalPowerBombAI_B2: u32 = 11698231;
pub const fnEnemy_NormalFrozenAI_B2: u32 = 11698241;
pub const fnnullsub_170_B2: u32 = 11698252;
pub const fnEnemyInstr_Goto_B2: u32 = 11698413;
pub const fnEnemyInstr_DecTimerAndGoto2_B2: u32 = 11698448;
pub const fnEnemyInstr_SetTimer_B2: u32 = 11698467;
pub const fnEnemyInstr_Sleep_B2: u32 = 11698479;
pub const fnEnemyInstr_WaitNframes_B2: u32 = 11698490;
pub const fnWalkingSpacePirates_Powerbomb: u32 = 11700071;
pub const fnWalkingSpacePirates_Touch: u32 = 11700076;
pub const fnWalkingSpacePirates_Shot: u32 = 11700089;
pub const fnWalkingSpacePirates_87C8: u32 = 11700168;
pub const fnWalkingSpacePirates_883E: u32 = 11700286;
pub const fnSpacePirates_Instr_MovePixelsDownAndChangeDirFaceRight: u32 = 11726400;
pub const fnSpacePirates_Instr_MovePixelsDownAndChangeDirFaceLeft: u32 = 11726450;
pub const fnSpacePirates_Instr_RandomNewDirFaceR: u32 = 11726500;
pub const fnSpacePirates_Instr_RandomNewDirFaceL: u32 = 11726524;
pub const fnSpacePirates_Instr_PrepareWallJumpR: u32 = 11726548;
pub const fnSpacePirates_Instr_PrepareWallJumpL: u32 = 11726589;
pub const fnSpacePirates_Instr_FireLaserL: u32 = 11726634;
pub const fnSpacePirates_Instr_FireLaserR: u32 = 11726685;
pub const fnSpacePirates_Instr_SetEnemyFunc: u32 = 11726723;
pub const fnSpacePirates_Instr_PlaySfx: u32 = 11726739;
pub const fnWallSpacePirates_Init: u32 = 11726751;
pub const fnWallSpacePirates_Main: u32 = 11726893;
pub const fnSpacePirates_Instr_20: u32 = 11728182;
pub const fnSpacePirates_Instr_16: u32 = 11728198;
pub const fnSpacePirates_Instr_15: u32 = 11728228;
pub const fnSpacePirates_Instr_18: u32 = 11728272;
pub const fnSpacePirates_Instr_17: u32 = 11728342;
pub const fnNinjaSpacePirates_Init: u32 = 11728350;
pub const fnNinjaSpacePirates_Main: u32 = 11728546;
pub const fnSpacePirates_Instr_19: u32 = 11729257;
pub const fnSpacePirates_Instr_21: u32 = 11729469;
pub const fnSpacePirates_Instr_12: u32 = 11730024;
pub const fnSpacePirates_Instr_14: u32 = 11730064;
pub const fnSpacePirates_Instr_11: u32 = 11730104;
pub const fnSpacePirates_Instr_13: u32 = 11730120;
pub const fnWalkingSpacePirates_Init: u32 = 11730178;
pub const fnWalkingSpacePirates_Main: u32 = 11730226;
pub const fnEnemy_GrappleReact_NoInteract_B3: u32 = 11763712;
pub const fnEnemy_GrappleReact_KillEnemy_B3: u32 = 11763722;
pub const fnEnemy_GrappleReact_CancelBeam_B3: u32 = 11763727;
pub const fnEnemy_NormalTouchAI_B3: u32 = 11763747;
pub const fnEnemy_NormalShotAI_B3: u32 = 11763757;
pub const fnEnemy_NormalPowerBombAI_B3: u32 = 11763767;
pub const fnEnemy_NormalFrozenAI_B3: u32 = 11763777;
pub const fnnullsub_170_B3: u32 = 11763788;
pub const fnEnemy_SetAiPreInstr_B3: u32 = 11763819;
pub const fnEnemy_ClearAiPreInstr_B3: u32 = 11763828;
pub const fnnullsub_171_B3: u32 = 11763835;
pub const fnEnemyInstr_Goto_B3: u32 = 11763949;
pub const fnEnemyInstr_DecTimerAndGoto2_B3: u32 = 11763984;
pub const fnEnemyInstr_SetTimer_B3: u32 = 11764003;
pub const fnEnemyInstr_Sleep_B3: u32 = 11764015;
pub const fnUnusedSpinningTurtleEye_Init: u32 = 11765499;
pub const fnUnusedSpinningTurtleEye_Main: u32 = 11765518;
pub const fnBrinstarPipeBug_Init: u32 = 11765819;
pub const fnBrinstarPipeBug_Main: u32 = 11765882;
pub const fnBrinstarPipeBug_PreInstr_1: u32 = 11765888;
pub const fnBrinstarPipeBug_PreInstr_2: u32 = 11765904;
pub const fnBrinstarPipeBug_PreInstr_3: u32 = 11765987;
pub const fnBrinstarPipeBug_PreInstr_4: u32 = 11766044;
pub const fnBrinstarPipeBug_PreInstr_5: u32 = 11766142;
pub const fnNorfairPipeBug_Init: u32 = 11766625;
pub const fnNorfairPipeBug_Main: u32 = 11766686;
pub const fnNorfairPipeBug_Func_5: u32 = 11767039;
pub const fnBrinstarYellowPipeBug_Init: u32 = 11767628;
pub const fnBrinstarYellowPipeBug_Main: u32 = 11767726;
pub const fnBotwoon_Instr_1: u32 = 11769031;
pub const fnBotwoon_Instr_2: u32 = 11769047;
pub const fnBotwoon_Instr_3: u32 = 11769063;
pub const fnBotwoon_Instr_4: u32 = 11769079;
pub const fnBotwoon_Instr_5: u32 = 11769095;
pub const fnBotwoon_Instr_6: u32 = 11769111;
pub const fnBotwoon_Instr_7: u32 = 11769127;
pub const fnBotwoon_Instr_8: u32 = 11769143;
pub const fnBotwoon_Instr_9: u32 = 11769159;
pub const fnBotwoon_Instr_10: u32 = 11769175;
pub const fnBotwoon_Instr_SetSpitting: u32 = 11769191;
pub const fnBotwoon_Instr_QueueSpitSfx: u32 = 11769202;
pub const fnBotwoon_Init: u32 = 11769219;
pub const fnBotwoon_Main: u32 = 11769448;
pub const fnBotwoon_Func_26: u32 = 11771328;
pub const fnBotwoon_Func_27: u32 = 11771517;
pub const fnBotwoon_Func_28: u32 = 11771616;
pub const fnBotwoon_Func_29: u32 = 11771700;
pub const fnBotwoon_Func_30: u32 = 11771770;
pub const fnBotwoon_Touch: u32 = 11771903;
pub const fnBotwoon_Shot: u32 = 11771926;
pub const fnBotwoon_Powerbomb: u32 = 11771969;
pub const fnEscapeEtecoon_Instr_1: u32 = 11789637;
pub const fnEscapeEtecoon_Instr_2: u32 = 11789840;
pub const fnEscapeEtecoon_Main: u32 = 11789909;
pub const fnEscapeEtecoon_E65C: u32 = 11789916;
pub const fnEscapeEtecoon_E670: u32 = 11789936;
pub const fnEscapeEtecoon_E680: u32 = 11789952;
pub const fnEscapeEtecoon_Init: u32 = 11790027;
pub const fnEscapeDachora_Instr_2: u32 = 11791016;
pub const fnEscapeDachora_Instr_3: u32 = 11791032;
pub const fnEscapeDachora_Instr_1: u32 = 11791049;
pub const fnEscapeDachora_Instr_4: u32 = 11791063;
pub const fnEscapeDachora_Init: u32 = 11791077;
pub const fnnullsub_54: u32 = 11791130;
crate::ram_map::ram_pointer_functions! {
    gEnemyData(o: i32) -> EnemyData = [0xf78, o as isize];
    gEnemySpawnData(o: i32) -> EnemySpawnData = [0x7000, o as isize];
}
pub unsafe fn Enemy_GrappleReact_NoInteract_A0(game: &mut Game) {
    SwitchEnemyAiToMainAi(game);
}
pub unsafe fn Enemy_GrappleReact_SamusLatchesOn_A0(game: &mut Game) {
    SamusLatchesOnWithGrapple(game);
}
pub unsafe fn Enemy_GrappleReact_KillEnemy_A0(game: &mut Game) {
    EnemyGrappleDeath(game);
}
pub unsafe fn Enemy_GrappleReact_CancelBeam_A0(game: &mut Game) {
    Enemy_SwitchToFrozenAi(game);
}
pub unsafe fn Enemy_GrappleReact_SamusLatchesNoInvinc_A0(game: &mut Game) {
    SamusLatchesOnWithGrappleNoInvinc(game);
}
pub unsafe fn Enemy_GrappleReact_SamusLatchesParalyze_A0(game: &mut Game) {
    SamusLatchesOnWithGrappleParalyze(game);
}
pub unsafe fn Enemy_GrappleReact_HurtSamus_A0(game: &mut Game) {
    SamusHurtFromGrapple(game);
}
pub unsafe fn Enemy_NormalTouchAI_A0(game: &mut Game) {
    NormalEnemyTouchAi(game);
}
pub unsafe fn Enemy_NormalTouchAI_SkipDeathAnim_A0(game: &mut Game) {
    NormalEnemyTouchAiSkipDeathAnim_CurEnemy(game);
}
pub unsafe fn Enemy_NormalShotAI_A0(game: &mut Game) {
    NormalEnemyShotAi(game);
}
pub unsafe fn Enemy_NormalShotAI_SkipSomeParts_A0(game: &mut Game) {
    NormalEnemyShotAiSkipDeathAnim_CurEnemy(game);
}
pub unsafe fn Enemy_NormalPowerBombAI_A0(game: &mut Game) {
    NormalEnemyPowerBombAi(game);
}
pub unsafe fn Enemy_NormalPowerBombAI_SkipDeathAnim_A0(game: &mut Game) {
    NormalEnemyPowerBombAiSkipDeathAnim_CurEnemy(game);
}
pub unsafe fn Enemy_NormalFrozenAI(game: &mut Game) {
    NormalEnemyFrozenAI(game);
}
pub unsafe fn CreateADudShot(game: &mut Game) {
    CreateDudShot(game);
}
pub fn func_nullsub_169() {}
pub fn func_nullsub_170() {}
pub unsafe fn EnemyInstr_SetAiPreInstr(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    game.enemy_instructions(k).set_ai_preinstr(*jp);
    jp.add(1)
}
pub fn EnemyInstr_ClearAiPreInstr(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    game.enemy_instructions(k).set_ai_preinstr(0x807b);
    jp
}
pub fn func_nullsub_171() {}
pub fn EnemyInstr_StopScript(k: u16, _jp: *const u16, game: &mut Game) -> *const u16 {
    let properties = game.enemy_instructions(k).properties() | kEnemyProps_Deleted as u16;
    game.enemy_instructions(k).set_properties(properties);
    core::ptr::null()
}
pub unsafe fn EnemyInstr_Goto(_k: u16, jp: *const u16) -> *const u16 {
    *jp as usize as *const u16
}
pub unsafe fn EnemyInstr_GotoRel(_k: u16, jp: *const u16) -> *const u16 {
    (jp as *const u8).offset(isize::from(*(jp as *const i8))) as *const u16
}
pub unsafe fn EnemyInstr_DecTimerAndGoto(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let timer = game.enemy_instructions(k).timer();
    let decremented = game.enemy_instructions(k).timer().wrapping_sub(1);
    game.enemy_instructions(k).set_timer(decremented);
    if timer == 1 {
        jp.add(1)
    } else {
        EnemyInstr_Goto(k, jp)
    }
}
pub unsafe fn EnemyInstr_DecTimerAndGotoRel(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let timer = game.enemy_instructions(k).timer_low();
    let decremented = game.enemy_instructions(k).timer_low().wrapping_sub(1);
    game.enemy_instructions(k).set_timer_low(decremented);
    if timer == 1 {
        (jp as *const u8).add(1) as *const u16
    } else {
        EnemyInstr_GotoRel(k, jp)
    }
}
pub unsafe fn EnemyInstr_SetTimer(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    game.enemy_instructions(k).set_timer(*jp);
    jp.add(1)
}
pub unsafe fn EnemyInstr_Skip2bytes(_k: u16, jp: *const u16) -> *const u16 {
    jp.add(1)
}
pub unsafe fn EnemyInstr_Sleep(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let base_ptr = RomBankBase(game.enemy_instructions(k).bank(), game);
    let instruction = (jp as *const u8).sub(2).offset_from(base_ptr) as u16;
    game.enemy_instructions(k)
        .set_current_instruction(instruction);
    core::ptr::null()
}
pub unsafe fn EnemyInstr_WaitNframes(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let base_ptr = RomBankBase(game.enemy_instructions(k).bank(), game);
    game.enemy_instructions(k).set_instruction_timer(*jp);
    let instruction = (jp as *const u8).add(2).offset_from(base_ptr) as u16;
    game.enemy_instructions(k)
        .set_current_instruction(instruction);
    core::ptr::null()
}
pub unsafe fn EnemyInstr_CopyToVram(_k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
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
}
pub fn EnemyInstr_EnableOffScreenProcessing(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let properties = game.enemy_instructions(k).properties() | 0x800;
    game.enemy_instructions(k).set_properties(properties);
    jp
}
pub fn EnemyInstr_DisableOffScreenProcessing(
    k: u16,
    jp: *const u16,
    game: &mut Game,
) -> *const u16 {
    let properties = game.enemy_instructions(k).properties() & !0x800;
    game.enemy_instructions(k).set_properties(properties);
    jp
}
static kRoomShakes: [u16; 144] = [
    1, 0, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0, 2, 0, 0, 0, 0, 2, 0, 0, 2, 2, 0, 0, 3, 0, 0, 0, 0, 3, 0, 0,
    3, 3, 0, 0, 1, 0, 1, 0, 0, 1, 0, 1, 1, 1, 1, 1, 2, 0, 2, 0, 0, 2, 0, 2, 2, 2, 2, 2, 3, 0, 3, 0,
    0, 3, 0, 3, 3, 3, 3, 3, 1, 0, 1, 0, 0, 1, 0, 1, 1, 1, 1, 1, 2, 0, 2, 0, 0, 2, 0, 2, 2, 2, 2, 2,
    3, 0, 3, 0, 0, 3, 0, 3, 3, 3, 3, 3, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0, 2, 0, 0, 0, 0, 2,
    0, 0, 2, 2, 0, 0, 3, 0, 0, 0, 0, 3, 0, 0, 3, 3,
];
pub unsafe fn HandleRoomShaking(game: &mut Game) {
    if ram_word!(game, earthquake_timer) != 0u16
        && ram_word!(game, time_is_frozen_flag) == 0
        && ram_word!(game, earthquake_type) as i32 - 36i32 & 0x8000i32 != 0
    {
        let v0: i32 = 8i32 * ram_word!(game, earthquake_type) as i32 >> 1i32;
        if (ram_word!(game, earthquake_timer) as i32 & 2i32) != 0i32 {
            update_ram_word!(game.reg_BG1HOFS, wrapping_sub(kRoomShakes[v0 as usize]));
            update_ram_word!(
                game.reg_BG1VOFS,
                wrapping_sub(kRoomShakes[(v0 + 1i32) as usize])
            );
            update_ram_word!(
                game.reg_BG2HOFS,
                wrapping_sub(kRoomShakes[(v0 + 2i32) as usize])
            );
            update_ram_word!(
                game.reg_BG2VOFS,
                wrapping_sub(kRoomShakes[(v0 + 3i32) as usize])
            );
        } else {
            update_ram_word!(game.reg_BG1HOFS, wrapping_add(kRoomShakes[v0 as usize]));
            update_ram_word!(
                game.reg_BG1VOFS,
                wrapping_add(kRoomShakes[(v0 + 1i32) as usize])
            );
            update_ram_word!(
                game.reg_BG2HOFS,
                wrapping_add(kRoomShakes[(v0 + 2i32) as usize])
            );
            update_ram_word!(
                game.reg_BG2VOFS,
                wrapping_add(kRoomShakes[(v0 + 3i32) as usize])
            );
        }
        update_ram_word!(game.earthquake_timer, wrapping_sub(1));
        if ram_word!(game, earthquake_type) as i32 - 18i32 & 0x8000i32 == 0 {
            SetAllEnemiesToShakeFor2Frames(game);
        }
    }
    update_ram_word!(game.frame_counter_every_frame, wrapping_add(1));
}
pub unsafe fn SetAllEnemiesToShakeFor2Frames(game: &mut Game) {
    let mut i: i32 = 0i32;
    loop {
        let v1: i32 = i >> 1i32;
        if ram_word!(game, active_enemy_indexes[v1]) == 0xffffu16 {
            break;
        }
        (*gEnemyData(ram_word!(game, active_enemy_indexes[v1]) as i32, game)).shake_timer = 2u16;
        i += 2i32;
    }
}
pub unsafe fn CallEnemyGfxDrawHook(ea: u32, game: &mut Game) {
    match ea {
        10518604 => (),
        10738444 => Reflec_Func_1(game),
        10851138 => Draygon_Func_36(game),
        10920690 => Ridley_A2F2(game),
        11042892 => (),
        11055282 => NorfairLavaMan_Func_6(game),
        11062375 => WreckedShipRobot_Func_1(game),
        11069550 => BlueBrinstarFaceBlock_Func_1(game),
        11110278 => (),
        11110345 => MotherBrain_DrawBrainNeck_EnemyGfxDrawHook(game),
        11110365 => MotherBrainsBrain_GfxDrawHook(game),
        11129754 => DeadTorizo_MainGfxHook(game),
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn DrawSamusEnemiesAndProjectiles(game: &mut Game) {
    DrawSpriteObjects(game);
    DrawBombAndProjectileExplosions(game);
    DrawEprojsWithPriority(EprojPriority::Low, game);
    let mut phase_varE32: u16 = 0u16;
    while phase_varE32 != 8u16 {
        if phase_varE32 == 3u16 {
            DrawSamusAndProjectiles(game);
        } else if phase_varE32 == 6u16 {
            DrawEprojsWithPriority(EprojPriority::High, game);
        }
        if ram_word!(game, enemy_drawing_queue_sizes[phase_varE32]) != 0 {
            let varE36: u16 = ram_word!(game, enemy_drawing_queue_sizes[phase_varE32]);
            let varE3A: u16 = crate::rtl::rom_table_word!(game, 0xa0b133, phase_varE32 as isize);
            let mut v1: u16 = 0u16;
            let mut varE38: u16 = 0;
            set_ram_word!(game.enemy_drawing_queue_sizes[phase_varE32] = 0u16);
            loop {
                varE38 = v1;
                let v2: *mut u8 = RomPtr_RAM((varE3A).wrapping_add(v1), game);
                let v3: u16 = *(v2 as *mut u16);
                *(v2 as *mut u16) = 0u16;
                set_ram_word!(game.cur_enemy_index = v3);
                WriteEnemyOams(game);
                v1 = (varE38).wrapping_add(2u16);
                if (varE38 as i32 + 2i32 == varE36 as i32) {
                    break;
                }
            }
        }
        phase_varE32 = phase_varE32.wrapping_add(1);
    }
    CallEnemyGfxDrawHook(
        Load24((&raw mut game.g_ram as *mut u8).offset(0x178c) as *mut LongPtr),
        game,
    );
}
pub unsafe fn RecordEnemySpawnData(j: u16, game: &mut Game) {
    let mut v2: *mut EnemySpawnData = null_mut::<EnemySpawnData>();
    let mut v5: *mut EnemySpawnData = null_mut::<EnemySpawnData>();
    let v1: *mut EnemyData = gEnemyData(j as i32, game);
    v2 = gEnemySpawnData(j as i32, game);
    (*v2).id = (*v1).enemy_ptr;
    (*v2).x_pos = (*v1).x_pos;
    (*v2).y_pos = (*v1).y_pos;
    (*v2).init_param = (*v1).current_instruction;
    (*v2).properties = (*v1).properties;
    (*v2).extra_properties = (*v1).extra_properties;
    (*v2).param_1 = (*v1).parameter_1;
    (*v2).param_2 = (*v1).parameter_2;
    let mut r18: u16 = 0u16;
    let mut r20: u16 = 0u16;
    let mut r22: u16 = 0u16;
    let mut r24: u16 = 0u16;
    let mut R26: u16 = 0u16;
    let mut R28: u16 = 0u16;
    let name_ptr: u16 = (*get_EnemyDef_A2((*v1).enemy_ptr, game)).name_ptr;
    if name_ptr != 0 {
        let v4: *const u16 = RomPtr_B4(name_ptr, game) as *const u16;
        r18 = *v4.offset(0);
        r20 = *v4.offset(1);
        r22 = *v4.offset(2);
        r24 = *v4.offset(3);
        R26 = *v4.offset(4);
        R28 = *v4.offset(6);
    }
    v5 = gEnemySpawnData(j as i32, game);
    *(&raw mut (*v5).name as *mut u8 as *mut u16) = r18;
    *((&raw mut (*v5).name as *mut u8).offset(2) as *mut u8 as *mut u16) = r20;
    *((&raw mut (*v5).name as *mut u8).offset(4) as *mut u8 as *mut u16) = r22;
    *((&raw mut (*v5).name as *mut u8).offset(6) as *mut u8 as *mut u16) = r24;
    *((&raw mut (*v5).name as *mut u8).offset(8) as *mut u8 as *mut u16) = R26;
    *((&raw mut (*v5).name as *mut u8).offset(10) as *mut u8 as *mut u16) = R28;
}
pub unsafe fn LoadEnemies(game: &mut Game) {
    set_ram_word!(game.debug_time_frozen_for_enemies = 0u16);
    (*((&raw mut game.g_ram as *mut u8).offset(0x178c) as *mut LongPtr)).bank = 160u8;
    (*((&raw mut game.g_ram as *mut u8).offset(0x178c) as *mut LongPtr)).addr =
        ((0xa0804ci32) as u16);
    set_ram_word!(game.enemy_bg2_tilemap_size = 2048u16);
    set_ram_word!(game.UNUSED_word_7E179E = 0u16);
    set_ram_word!(game.UNUSED_word_7E17A0 = 0u16);
    set_ram_word!(game.boss_id = 0u16);
    ClearEnemyDataAndProcessEnemySet(game);
    LoadEnemyTileData(game);
    set_ram_word!(game.enemy_tile_vram_src = 0u16);
    set_ram_word!(game.flag_disable_projectile_interaction = 0u16);
    ClearSpriteObjects(game);
}
pub unsafe fn ClearEnemyDataAndProcessEnemySet(game: &mut Game) {
    memset(
        (&raw mut game.g_ram as *mut u8).offset(0xf78) as *mut c_void,
        0i32,
        2048 as size_t,
    );
    if *(RomPtr_A1(ram_word!(game, room_enemy_population_ptr), game) as *mut u16) != 0xffffu16 {
        ProcessEnemyTilesets(game);
    }
}
pub unsafe fn InitializeEnemies(game: &mut Game) {
    memset(
        (&raw mut game.g_ram as *mut u8).offset(0x7000) as *mut u8 as *mut c_void,
        0i32,
        0x2800 as size_t,
    );
    set_ram_word!(game.num_enemies_in_room = 0u16);
    set_ram_word!(game.num_enemies_killed_in_room = 0u16);
    set_ram_word!(game.flag_process_all_enemies = 0u16);
    let mut i: i32 = 286i32;
    while i >= 0i32 {
        set_ram_word!(game.eproj_flags[(i >> 1i32)] = 0u16);
        i -= 2i32;
    }
    let mut j: i32 = 34i32;
    while j >= 0i32 {
        set_ram_word!(game.eproj_killed_enemy_index[(j >> 1i32)] = -1i32 as u16);
        j -= 2i32;
    }
    let mut v4: u16 = ram_word!(game, room_enemy_population_ptr);
    if (*get_EnemyPopulation(0xa1u8, ram_word!(game, room_enemy_population_ptr), game)).enemy_ptr
        == 0xffffu16
    {
        return;
    }
    set_ram_word!(game.UNUSED_word_7E0E48 = 0u16);
    let mut v5: u16 = 0u16;
    loop {
        LoadEnemyGfxIndexes(v4, v5, game);
        let EP: *mut EnemyPopulation = get_EnemyPopulation(0xa1u8, v4, game);
        let ED: *mut EnemyDef = get_EnemyDef_A2((*EP).enemy_ptr, game);
        let E: *mut EnemyData = gEnemyData(v5 as i32, game);
        (*E).x_width = (*ED).x_radius;
        (*E).y_height = (*ED).y_radius;
        (*E).health = (*ED).health;
        (*E).layer = (*ED).layer as u16;
        *(&raw mut (*E).bank as *mut u16) = *(&raw mut (*ED).bank as *mut u16);
        if (*ED).boss_fight_value != 0 {
            set_ram_word!(game.boss_id = (*ED).boss_fight_value);
        }
        (*E).enemy_ptr = (*EP).enemy_ptr;
        (*E).x_pos = (*EP).x_pos;
        (*E).y_pos = (*EP).y_pos;
        (*E).current_instruction = (*EP).init_param;
        (*E).properties = (*EP).properties;
        (*E).extra_properties = (*EP).extra_properties;
        (*E).parameter_1 = (*EP).parameter1;
        (*E).parameter_2 = (*EP).parameter2;
        (*E).frame_counter = 0u16;
        (*E).timer = 0u16;
        (*E).instruction_timer = 1u16;
        (*E).frame_counter = 0u16;
        RecordEnemySpawnData(v5, game);
        set_ram_word!(game.cur_enemy_index = v5);
        CallEnemyAi(
            (((*ED).bank as i32) << 16i32 | (*ED).ai_init as i32) as u32,
            game,
        );
        (*E).spritemap_pointer = 0u16;
        if ((*E).properties as i32 & 0x2000i32) != 0i32 {
            let mut v12: u16 = addr_kSpritemap_Nothing_A4 as u16;
            if ((*E).extra_properties as i32 & 4i32) != 0i32 {
                v12 = addr_kExtendedSpritemap_Nothing_A4 as u16;
            }
            (*E).spritemap_pointer = v12;
        }
        v5 = (v5).wrapping_add(64u16);
        v4 = (v4).wrapping_add(16u16);
        if ((*get_EnemyPopulation(0xa1u8, v4, game)).enemy_ptr == 0xffffu16) {
            break;
        }
    }
    set_ram_word!(game.first_free_enemy_index = v5);
    set_ram_word!(game.num_enemies_in_room = (v5 as i32 >> 6i32) as u16);
    set_ram_word!(game.num_enemy_deaths_left_to_clear = *RomPtr_A1(v4, game).offset(2) as u16);
}
pub unsafe fn LoadEnemyGfxIndexes(k: u16, j: u16, game: &mut Game) {
    let E: *mut EnemyData = gEnemyData(j as i32, game);
    let ES: *mut EnemySpawnData = gEnemySpawnData(j as i32, game);
    let mut e: u16 = ram_word!(game, room_enemy_tilesets_ptr);
    let mut f: u16 = 0u16;
    loop {
        let ET: *mut EnemyTileset = get_EnemyTileset(e, game);
        if (*get_EnemyPopulation(0xa1u8, k, game)).enemy_ptr == (*ET).enemy_def {
            (*ES).palette_index = (((*ET).vram_dst as i32 & 0xfi32) << 9i32) as u16;
            (*E).palette_index = (*ES).palette_index;
            (*ES).vram_tiles_index = f;
            (*E).vram_tiles_index = (*ES).vram_tiles_index;
            return;
        }
        if (*ET).enemy_def == 0xffffu16 {
            (*ES).vram_tiles_index = 0u16;
            (*E).vram_tiles_index = (*ES).vram_tiles_index;
            (*ES).palette_index = 2560u16;
            (*E).palette_index = (*ES).palette_index;
            return;
        }
        f = (f as i32 + ((*get_EnemyDef_A2((*ET).enemy_def, game)).tile_data_size as i32 >> 5i32))
            as u16;
        e = (e).wrapping_add(4u16);
    }
}
pub unsafe fn LoadEnemyTileData(game: &mut Game) {
    let mut i: i32 = 510i32;
    while i >= 0i32 {
        (*gEnemySpawnData(i, game)).some_flag =
            crate::rtl::rom_table_word!(game, 0x9ad200, ((i >> 1i32) + 3072i32) as isize);
        i -= 2i32;
    }
    if ram_word!(game, enemy_tile_load_data_write_pos) != 0 {
        let mut v1: u16 = 0u16;
        let mut load_data: *mut EnemyTileLoadData =
            (&raw mut game.g_ram as *mut u8).offset(0xe5a) as *mut EnemyTileLoadData;
        loop {
            let mut v2: u16 = (*load_data).tile_data_ptr.addr;
            let r18: u16 = ((*load_data).tile_data_size).wrapping_add(v2);
            let mut v3: u16 = (*load_data).offset_into_ram;
            let db: u8 = (*load_data).tile_data_ptr.bank;
            loop {
                memcpy(
                    (&raw mut game.g_ram as *mut u8).offset((0x7000i32 + v3 as i32) as isize)
                        as *mut u8 as *mut c_void,
                    RomPtrWithBank(db, v2, game) as *const c_void,
                    8 as size_t,
                );
                v3 = (v3).wrapping_add(8u16);
                v2 = (v2).wrapping_add(8u16);
                if (v2 == r18) {
                    break;
                }
            }
            load_data = load_data.offset(1);
            v1 = (v1).wrapping_add(7u16);
            if (v1 == ram_word!(game, enemy_tile_load_data_write_pos)) {
                break;
            }
        }
        set_ram_word!(game.enemy_tile_load_data_write_pos = 0u16);
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
    set_ram_word!(game.enemy_tile_load_data_write_pos = 0u16);
    let mut r30: u16 = 2048u16;
    set_ram_word!(game.enemy_def_ptr[0] = 0u16);
    set_ram_word!(game.enemy_def_ptr[1] = 0u16);
    set_ram_word!(game.enemy_def_ptr[2] = 0u16);
    set_ram_word!(game.enemy_def_ptr[3] = 0u16);
    set_ram_word!(game.enemy_gfxdata_tiles_index[0] = 0u16);
    set_ram_word!(game.enemy_gfxdata_tiles_index[1] = 0u16);
    set_ram_word!(game.enemy_gfxdata_tiles_index[2] = 0u16);
    set_ram_word!(game.enemy_gfxdata_tiles_index[3] = 0u16);
    set_ram_word!(game.enemy_gfxdata_vram_ptr[0] = 0u16);
    set_ram_word!(game.enemy_gfxdata_vram_ptr[1] = 0u16);
    set_ram_word!(game.enemy_gfxdata_vram_ptr[2] = 0u16);
    set_ram_word!(game.enemy_gfxdata_vram_ptr[3] = 0u16);
    let mut enemy_gfx_data_write_ptr: u16 = 0u16;
    let mut next_enemy_tiles_index: u16 = 0u16;
    let mut ET: *mut EnemyTileset =
        get_EnemyTileset(ram_word!(game, room_enemy_tilesets_ptr), game);
    let mut LD: *mut EnemyTileLoadData =
        (&raw mut game.g_ram as *mut u8).offset(0xe5a) as *mut EnemyTileLoadData;
    while (*ET).enemy_def != 0xffffu16 {
        let ED: *mut EnemyDef = get_EnemyDef_A2((*ET).enemy_def, game);
        memcpy(
            ((&raw mut game.g_ram as *mut u8).offset(0xc200) as *mut u16).offset(
                ((*(&raw mut (*ET).vram_dst as *mut u8).offset(0) as i32 + 8i32) * 16i32) as isize,
            ) as *mut u16 as *mut c_void,
            RomPtrWithBank((*ED).bank, (*ED).palette_ptr, game) as *const c_void,
            32 as size_t,
        );
        (*LD).tile_data_size = ((*ED).tile_data_size) & (0x7fffu16);
        (*LD).tile_data_ptr = (*ED).tile_data;
        let mut v10: u16 = r30;
        if ((*ED).tile_data_size as i32 & 0x8000i32) != 0i32 {
            v10 = ((((*ET).vram_dst) & (0x3000u16)) as i32 >> 3i32) as u16;
        }
        (*LD).offset_into_ram = v10;
        update_ram_word!(game.enemy_tile_load_data_write_pos, wrapping_add(7u16));
        LD = LD.offset(1);
        let v11: u16 = enemy_gfx_data_write_ptr;
        *((&raw mut game.g_ram as *mut u8).offset(0xef64) as *mut u16)
            .offset((enemy_gfx_data_write_ptr as i32 >> 1i32) as isize) = next_enemy_tiles_index;
        set_ram_word!(game.enemy_def_ptr[(v11 as i32 >> 1i32)] = (*ET).enemy_def);
        set_ram_word!(game.enemy_gfxdata_vram_ptr[(v11 as i32 >> 1i32)] = (*ET).vram_dst);
        enemy_gfx_data_write_ptr = (enemy_gfx_data_write_ptr).wrapping_add(2u16);
        next_enemy_tiles_index =
            (next_enemy_tiles_index as i32 + ((*ED).tile_data_size as i32 >> 5i32)) as u16;
        r30 = (r30).wrapping_add((*ED).tile_data_size);
        ET = ET.offset(1);
    }
}
pub unsafe fn DetermineWhichEnemiesToProcess(game: &mut Game) {
    update_ram_word!(game.UNUSED_word_7E0E46, wrapping_add(1));
    set_ram_word!(game.cur_enemy_index = 0u16);
    set_ram_word!(game.active_enemy_indexes_write_ptr = 0u16);
    set_ram_word!(game.interactive_enemy_indexes_write_ptr = 0u16);
    if ram_word!(game, flag_process_all_enemies) != 0 {
        loop {
            let v5: u16 = ram_word!(game, cur_enemy_index);
            let v6: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
            if (*v6).enemy_ptr != 0u16 && (*v6).enemy_ptr as i32 != addr_kEnemyDef_DAFF as i32 {
                if ((*v6).properties as i32 & 0x200i32) != 0i32 {
                    (*v6).enemy_ptr = 0u16;
                } else {
                    let v7: u16 = ram_word!(game, active_enemy_indexes_write_ptr);
                    let v8: i32 = ram_word!(game, active_enemy_indexes_write_ptr) as i32 >> 1i32;
                    set_ram_word!(game.active_enemy_indexes[v8] = ram_word!(game, cur_enemy_index));
                    set_ram_word!(game.interactive_enemy_indexes[v8] = v5);
                    set_ram_word!(game.active_enemy_indexes_write_ptr = (v7).wrapping_add(2u16));
                    if ((*v6).properties as i32 & 0x400i32) == 0i32 {
                        let v9: u16 = ram_word!(game, interactive_enemy_indexes_write_ptr);
                        *((&raw mut game.g_ram as *mut u8).offset(0x17ec) as *mut u16).offset(
                            (ram_word!(game, interactive_enemy_indexes_write_ptr) as i32 >> 1i32)
                                as isize,
                        ) = v5;
                        set_ram_word!(
                            game.interactive_enemy_indexes_write_ptr = (v9).wrapping_add(2u16)
                        );
                    }
                }
            }
            update_ram_word!(game.cur_enemy_index, wrapping_add(64u16));
            if (ram_word!(game, cur_enemy_index) as i32 - 2048i32 & 0x8000i32 == 0) {
                break;
            }
        }
        *((&raw mut game.g_ram as *mut u8).offset(0x17ac) as *mut u16)
            .offset((ram_word!(game, active_enemy_indexes_write_ptr) as i32 >> 1i32) as isize) =
            -1i32 as u16;
        *((&raw mut game.g_ram as *mut u8).offset(0x17ec) as *mut u16).offset(
            (ram_word!(game, interactive_enemy_indexes_write_ptr) as i32 >> 1i32) as isize,
        ) = -1i32 as u16;
    } else {
        loop {
            let v0: u16 = ram_word!(game, cur_enemy_index);
            let v1: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
            if (*v1).enemy_ptr != 0u16 && (*v1).enemy_ptr as i32 != addr_kEnemyDef_DAFF as i32 {
                let properties: u16 = (*v1).properties;
                if (properties as i32 & 0x200i32) != 0i32 {
                    (*v1).enemy_ptr = 0u16;
                } else if (properties as i32 & 0x800i32) != 0i32
                    || ((*v1).ai_handler_bits as i32 & 4i32) != 0i32
                    || (((*v1).x_width as i32 + (*v1).x_pos as i32
                        - ram_word!(game, layer1_x_pos) as i32) as i16)
                        >= 0i16
                        && (((*v1).x_width as i32 + ram_word!(game, layer1_x_pos) as i32 + 256i32
                            - (*v1).x_pos as i32) as i16)
                            >= 0i16
                        && (((*v1).y_pos as i32 + 8i32 - ram_word!(game, layer1_y_pos) as i32)
                            as i16)
                            >= 0i16
                        && ((ram_word!(game, layer1_y_pos) as i32 + 248i32 - (*v1).y_pos as i32)
                            as i16)
                            >= 0i16
                {
                    let v3: u16 = ram_word!(game, active_enemy_indexes_write_ptr);
                    *((&raw mut game.g_ram as *mut u8).offset(0x17ac) as *mut u16).offset(
                        (ram_word!(game, active_enemy_indexes_write_ptr) as i32 >> 1i32) as isize,
                    ) = ram_word!(game, cur_enemy_index);
                    set_ram_word!(game.active_enemy_indexes_write_ptr = (v3).wrapping_add(2u16));
                    if ((*v1).properties as i32 & 0x400i32) == 0i32 {
                        let v4: u16 = ram_word!(game, interactive_enemy_indexes_write_ptr);
                        *((&raw mut game.g_ram as *mut u8).offset(0x17ec) as *mut u16).offset(
                            (ram_word!(game, interactive_enemy_indexes_write_ptr) as i32 >> 1i32)
                                as isize,
                        ) = v0;
                        set_ram_word!(
                            game.interactive_enemy_indexes_write_ptr = (v4).wrapping_add(2u16)
                        );
                    }
                }
            }
            update_ram_word!(game.cur_enemy_index, wrapping_add(64u16));
            if (ram_word!(game, cur_enemy_index) as i32 - 2048i32 & 0x8000i32 == 0) {
                break;
            }
        }
        *((&raw mut game.g_ram as *mut u8).offset(0x17ac) as *mut u16)
            .offset((ram_word!(game, active_enemy_indexes_write_ptr) as i32 >> 1i32) as isize) =
            -1i32 as u16;
        *((&raw mut game.g_ram as *mut u8).offset(0x17ec) as *mut u16).offset(
            (ram_word!(game, interactive_enemy_indexes_write_ptr) as i32 >> 1i32) as isize,
        ) = -1i32 as u16;
    };
}
pub unsafe fn CallEnemyAi(ea: u32, game: &mut Game) {
    match ea {
        10649605 => Enemy_GrappleReact_SamusLatchesOn_A2(game),
        10649610 => Enemy_GrappleReact_KillEnemy_A2(game),
        10649615 => Enemy_GrappleReact_CancelBeam_A2(game),
        10649635 => Enemy_NormalTouchAI_A2(game),
        10649645 => Enemy_NormalShotAI_A2(game),
        10649665 => Enemy_NormalFrozenAI_A2(game),
        10649676 => (),
        10651420 => BouncingGoofball_Init(game),
        10651548 => BouncingGoofball_Main(game),
        10652077 => MiniCrocomire_Init(game),
        10652144 => MiniCrocomire_Main(game),
        10653036 => MaridiaBeybladeTurtle_Init(game),
        10653085 => MiniMaridiaBeybladeTurtle_Init(game),
        10653138 => MaridiaBeybladeTurtle_Main(game),
        10653998 => MiniMaridiaBeybladeTurtle_Main(game),
        10654337 => MaridiaBeybladeTurtle_Touch(game),
        10654367 => MiniMaridiaBeybladeTurtle_Touch(game),
        10654479 => MiniMaridiaBeybladeTurtle_Shot(game),
        10656319 => ThinHoppingBlobs_Init(game),
        10656381 => ThinHoppingBlobs_Main(game),
        10657608 => SpikeShootingPlant_Init(game),
        10657715 => SpikeShootingPlant_Main(game),
        10658809 => MaridiaSpikeyShell_Init(game),
        10658942 => MaridiaSpikeyShell_Main(game),
        10659193 => MaridiaSpikeyShell_Shot(game),
        10659396 => GunshipTop_Init(game),
        10659538 => GunshipBottom_Init(game),
        10659673 => GunshipTop_Main(game),
        10661995 => Flies_Init(game),
        10662175 => Flies_Main(game),
        10662880 => NorfairErraticFireball_Init(game),
        10662927 => NorfairErraticFireball_Main(game),
        10663280 => LavaquakeRocks_Init(game),
        10663311 => LavaquakeRocks_Main(game),
        10663426 => Rinka_Init(game),
        10663876 => Rinka_Main(game),
        10664233 => {
            Rinka_Frozen(ram_word!(game, cur_enemy_index), game);
        }
        10664263 => Rinka_Touch(game),
        10664269 => Rinka_Shot(game),
        10664275 => {
            Rinka_Powerbomb(ram_word!(game, cur_enemy_index), game);
        }
        10664909 => Rio_Init(game),
        10664931 => Rio_Main(game),
        10665625 => NorfairLavajumpingEnemy_Init(game),
        10665682 => NorfairLavajumpingEnemy_Main(game),
        10666562 => NorfairRio_Init(game),
        10666615 => NorfairRio_Main(game),
        10667763 => LowerNorfairRio_Init(game),
        10667812 => LowerNorfairRio_Main(game),
        10669268 => MaridiaLargeSnail_Init(game),
        10669331 => MaridiaLargeSnail_Main(game),
        10670988 => MaridiaLargeSnail_Touch(game),
        10671028 => MaridiaLargeSnail_Shot(game),
        10674038 => HirisingSlowfalling_Init(game),
        10674222 => HirisingSlowfalling_Main(game),
        10674643 => Gripper_Init(game),
        10674721 => Gripper_Main(game),
        10674968 => JetPowerRipper_Init(game),
        10675027 => JetPowerRipper_Main(game),
        10675113 => JetPowerRipper_Shot(game),
        10675359 => Ripper_Init(game),
        10675418 => Ripper_Main(game),
        10675718 => LavaSeahorse_Init(game),
        10675790 => LavaSeahorse_Main(game),
        10676168 => LavaSeahorse_Touch(game),
        10676174 => LavaSeahorse_Shot(game),
        10676180 => LavaSeahorse_Powerbomb(game),
        10676698 => TimedShutter_Init(game),
        10676918 => TimedShutter_Main(game),
        10677765 => RisingFallingPlatform_Init(game),
        10677778 => ShootableShutter_Init(game),
        10677969 => RisingFallingPlatform_Main(game),
        10678429 => RisingFallingPlatform_Touch(game),
        10678434 => RisingFallingPlatform_Shot(game),
        10678442 => ShootableShutter_Shot(game),
        10678454 => RisingFallingPlatform_Powerbomb(game),
        10678545 => HorizontalShootableShutter_Init(game),
        10678750 => HorizontalShootableShutter_Main(game),
        10679256 => HorizontalShootableShutter_Touch(game),
        10679310 => HorizontalShootableShutter_Shot(game),
        10679322 => HorizontalShootableShutter_Powerbomb(game),
        10715136 => Enemy_GrappleReact_NoInteract_A3(game),
        10715146 => Enemy_GrappleReact_KillEnemy_A3(game),
        10715151 => Enemy_GrappleReact_CancelBeam_A3(game),
        10715171 => Enemy_NormalTouchAI_A3(game),
        10715181 => Enemy_NormalShotAI_A3(game),
        10715201 => Enemy_NormalFrozenAI_A3(game),
        10715212 => (),
        10716909 => Waver_Init(game),
        10717004 => Waver_Main(game),
        10717536 => Metalee_Init(game),
        10717561 => Metalee_Main(game),
        10717967 => Metalee_Shot(game),
        10718509 => Fireflea_Init(game),
        10718702 => Fireflea_Main(game),
        10718827 => {
            Fireflea_Touch(ram_word!(game, cur_enemy_index), game);
        }
        10718851 => Fireflea_Powerbomb(game),
        10718857 => Fireflea_Shot(game),
        10719413 => MaridiaFish_Init(game),
        10719531 => MaridiaFish_Main(game),
        10720486 => Elevator_Init(game),
        10720554 => Elevator_Frozen(game),
        10720995 => Crab_Init(game),
        10721595 => Slug_Init(game),
        10722463 => PlatformThatFallsWithSamus_Init(game),
        10722490 => FastMovingSlowSinkingPlatform_Init(game),
        10722582 => PlatformThatFallsWithSamus_Main(game),
        10723079 => (),
        10723080 => FastMovingSlowSinkingPlatform_Shot(game),
        10723661 => Roach_Init(game),
        10724048 => Roach_Main(game),
        10725245 => Mochtroid_Init(game),
        10725264 => Mochtroid_Main(game),
        10725715 => Mochtroid_Touch(game),
        10725800 => Mochtroid_Shot(game),
        10726153 => Sidehopper_Init(game),
        10726351 => Sidehopper_Main(game),
        10728522 => MaridiaRefillCandy_Init(game),
        10728572 => MaridiaRefillCandy_Main(game),
        10729071 => NorfairSlowFireball_Init(game),
        10730163 => Bang_Init(game),
        10730277 => Bang_Main(game),
        10731261 => Bang_Shot(game),
        10733230 => Skree_Init(game),
        10733255 => Skree_Main(game),
        10733557 => Skree_Shot(game),
        10735074 => MaridiaSnail_Init(game),
        10735204 => MaridiaSnail_Main(game),
        10736560 => MaridiaSnail_Touch(game),
        10736745 => MaridiaSnail_Shot(game),
        10738643 => Reflec_Init(game),
        10738715 => (),
        10738716 => Reflec_Shot(game),
        10739779 => WreckedShipOrangeZoomer_Init(game),
        10739851 => WreckedShipOrangeZoomer_Main(game),
        10740436 => BigEyeBugs_Init(game),
        10741148 => FireZoomer_Init(game),
        10741353 => StoneZoomer_Init(game),
        10741442 => StoneZoomer_Main(game),
        10742351 => Metroid_Init(game),
        10742502 => Metroid_Frozen(game),
        10742579 => Metroid_Hurt(game),
        10742680 => Metroid_Main(game),
        10743275 => Metroid_Touch(game),
        10743559 => Metroid_Shot(game),
        10743874 => {
            Metroid_Powerbomb(ram_word!(game, cur_enemy_index), game);
        }
        10780677 => Enemy_GrappleReact_SamusLatchesOn_A4(game),
        10780707 => Enemy_NormalTouchAI_A4(game),
        10780717 => Enemy_NormalShotAI_A4(game),
        10780737 => Enemy_NormalFrozenAI_A4(game),
        10780748 => (),
        10782343 => Crocomire_Hurt(game),
        10783322 => Crocomire_Init(game),
        10783748 => Crocomire_Main(game),
        10795344 => (),
        10795410 => Crocomire_Powerbomb(game),
        10811002 => CrocomireTongue_Init(game),
        10811067 => CrocomireTongue_Main(),
        10846208 => Enemy_GrappleReact_NoInteract_A5(game),
        10846223 => Enemy_GrappleReact_CancelBeam_A5(game),
        10846243 => Enemy_NormalTouchAI_A5(game),
        10846253 => Enemy_NormalShotAI_A5(game),
        10846273 => Enemy_NormalFrozenAI_A5(game),
        10846284 => (),
        10847879 => Draygon_Init(game),
        10847996 => Draygon_Main(game),
        10851661 => Draygon_Hurt(game),
        10851818 => Draygon_Touch(game),
        10851824 => Draygon_Shot(game),
        10851847 => Draygon_Powerbomb(game),
        10863723 => DraygonsEye_Init(game),
        10863750 => DraygonsEye_Main(game),
        10864025 => DraygonsTail_Init(game),
        10864042 => (),
        10864045 => DraygonsArms_Init(game),
        10864068 => (),
        10873386 => SporeSpawn_Init(game),
        10873619 => SporeSpawn_Main(game),
        10874202 => SporeSpawn_Shot(game),
        10874348 => SporeSpawn_Touch(game),
        10874354 => (),
        10911744 => Enemy_GrappleReact_NoInteract_A6(game),
        10911759 => Enemy_GrappleReact_CancelBeam_A6(game),
        10911779 => Enemy_NormalTouchAI_A6(game),
        10911789 => Enemy_NormalShotAI_A6(game),
        10911809 => Enemy_NormalFrozenAI_A6(game),
        10911820 => (),
        10913525 => Boulder_Init(game),
        10913683 => Boulder_Main(game),
        10914607 => SpikeyPlatform_Init(game),
        10914693 => SpikeyPlatform2ndEnemy_Init(game),
        10914713 => SpikeyPlatform2ndEnemy_Main(game),
        10914733 => SpikeyPlatform_Main(game),
        10915836 => FireGeyser_Init(game),
        10915875 => FireGeyser_Main(game),
        10917060 => NuclearWaffle_Init(game),
        10917390 => NuclearWaffle_Main(game),
        10918488 => FakeKraid_Init(game),
        10918594 => FakeKraid_Main(game),
        10918946 => FakeKraid_Touch(game),
        10918969 => FakeKraid_Shot(game),
        10920181 => CeresRidley_Init(game),
        10920584 => CeresRidley_Main(game),
        10920659 => CeresRidley_Hurt(game),
        10924583 => Ridley_Main(game),
        10924682 => Ridley_Func_2(game),
        10924695 => Ridley_Hurt(game),
        10929814 => RidleysExplosion_Init(game),
        10930388 => RidleysExplosion_Main(game),
        10936202 => Ridley_Shot(game),
        10936242 => Ridley_Powerbomb(game),
        10940337 => CeresSteam_Init(game),
        10940429 => CeresSteam_Main(game),
        10940479 => CeresSteam_Touch(game),
        10942149 => CeresDoor_Init(game),
        10942309 => CeresDoor_Main(game),
        10942752 => (),
        10943346 => Zebetites_Init(game),
        10943539 => Zebetites_Main(game),
        10943911 => Zebetites_Touch(game),
        10943916 => Zebetites_Shot(game),
        10977280 => Enemy_GrappleReact_NoInteract_A7(game),
        10977295 => Enemy_GrappleReact_CancelBeam_A7(game),
        10977325 => Enemy_NormalShotAI_A7(game),
        10977340 => Enemy_NormalPowerBombAI_SkipDeathAnim_A7(game),
        10977345 => Enemy_NormalFrozenAI_A7(game),
        10977356 => (),
        10982543 => (),
        10982544 => KraidsArm_Touch(game),
        10982559 => Kraid_Touch(game),
        10982581 => (),
        10987865 => Kraid_Init(game),
        10988355 => KraidsArm_Init(game),
        10988392 => KraidsTopLint_Init(game),
        10988444 => KraidsMiddleLint_Init(game),
        10988490 => KraidsBottomLint_Init(game),
        10988536 => KraidsFoot_Init(game),
        10988577 => Kraid_Main(game),
        10991549 => KraidsArm_Main(game),
        10991617 => KraidsTopLint_Main(game),
        10991629 => KraidsMiddleLint_Main(game),
        10991641 => KraidsBottomLint_Main(game),
        10992118 => KraidsFoot_Main(game),
        10992847 => KraidsGoodFingernail_Touch(game),
        10992862 => KraidsBadFingernail_Touch(game),
        10992879 => KraidsGoodFingernail_Init(game),
        10992941 => KraidsBadFingernail_Init(game),
        10992946 => KraidsGoodFingernail_Main(game),
        10992969 => KraidsBadFingernail_Main(game),
        10997235 => Phantoon_Init(game),
        10997333 => Phantoon2_Init(game),
        10997414 => Phantoon_Main(game),
        11001151 => Phantoon_Hurt(game),
        11001237 => Phantoon_Touch(game),
        11001242 => (),
        11001243 => Phantoon_Shot(game),
        11001873 => (),
        11004178 => Etecoon_Init(game),
        11004224 => Etecoon_Main(game),
        11007197 => Dachora_Init(game),
        11007278 => Dachora_Main(game),
        11042816 => Enemy_GrappleReact_NoInteract_A8(game),
        11042826 => Enemy_GrappleReact_KillEnemy_A8(game),
        11042831 => Enemy_GrappleReact_CancelBeam_A8(game),
        11042836 => Enemy_GrappleReact_SamusLatchesNoInvinc_A8(game),
        11042846 => Enemy_GrappleReact_HurtSamus_A8(game),
        11042851 => Enemy_NormalTouchAI_A8(game),
        11042861 => Enemy_NormalShotAI_A8(game),
        11042881 => Enemy_NormalFrozenAI_A8(game),
        11042892 => (),
        11044832 => MiniDraygon_Init(game),
        11045040 => EvirProjectile_Init(game),
        11045147 => MiniDraygon_Main(game),
        11045278 => EvirProjectile_Main(game),
        11045638 => MiniDraygon_Touch(game),
        11045644 => MiniDraygon_Powerbomb(game),
        11045650 => MiniDraygon_Shot(game),
        11047000 => MorphBallEye_Init(game),
        11047138 => MorphBallEye_Main(game),
        11048675 => Fune_Init(game),
        11048752 => Fune_Main(game),
        11049710 => WreckedShipGhost_Init(game),
        11049788 => WreckedShipGhost_Main(game),
        11051336 => YappingMaw_Init(game),
        11051537 => YappingMaw_Main(game),
        11052953 => YappingMaw_Touch(game),
        11052989 => YappingMaw_Shot(game),
        11053109 => YappingMaw_Frozen(game),
        11053894 => Kago_Init(game),
        11053941 => Kago_Main(game),
        11053955 => Kago_Shot(game),
        11054987 => NorfairLavaMan_Init(game),
        11055370 => NorfairLavaMan_Main(game),
        11056128 => NorfairLavaMan_Powerbomb(game),
        11056134 => NorfairLavaMan_Touch(game),
        11056140 => NorfairLavaMan_Shot(game),
        11057014 => Beetom_Init(game),
        11057165 => Beetom_Main(game),
        11058734 => Beetom_Touch(game),
        11058860 => Beetom_Shot(game),
        11059657 => MaridiaFloater_Init(game),
        11059740 => MaridiaFloater_Main(game),
        11060670 => MaridiaFloater_Touch(game),
        11060719 => MaridiaFloater_Shot(game),
        11060799 => MaridiaFloater_Powerbomb(game),
        11062135 => WreckedShipRobot_Init(game),
        11062220 => WreckedShipRobotDeactivated_Init(game),
        11062326 => WreckedShipRobot_Main(game),
        11062374 => (),
        11063668 => WreckedShipRobotDeactivated_Touch(game),
        11063693 => WreckedShipRobotDeactivated_Shot(game),
        11063698 => WreckedShipRobot_Shot(game),
        11065545 => MaridiaPuffer_Init(game),
        11065611 => MaridiaPuffer_Main(game),
        11066132 => MaridiaPuffer_Shot(game),
        11066573 => WalkingLavaSeahorse_Init(game),
        11066731 => WalkingLavaSeahorse_Main(game),
        11068296 => WreckedShipOrbs_Init(game),
        11068355 => WreckedShipOrbs_Main(game),
        11068983 => WreckedShipSpark_Init(game),
        11069070 => WreckedShipSpark_Main(game),
        11069198 => WreckedShipSpark_Shot(game),
        11069486 => BlueBrinstarFaceBlock_Init(game),
        11069614 => BlueBrinstarFaceBlock_Main(game),
        11069725 => BlueBrinstarFaceBlock_Shot(game),
        11071880 => KiHunter_Init(game),
        11072020 => KiHunterWings_Init(game),
        11072092 => KiHunter_Main(game),
        11072098 => KiHunterWings_Main(game),
        11073281 => KiHunter_Shot(game),
        11108367 => Enemy_GrappleReact_CancelBeam_A9(game),
        11108417 => Enemy_NormalFrozenAI_A9(game),
        11108428 => (),
        11110023 => MotherBrainsBody_Init(game),
        11110149 => MotherBrainsBrain_Init(game),
        11110206 => MotherBrainsBody_Hurt(game),
        11110279 => MotherBrainsBody_Powerbomb(game),
        11110283 => MotherBrainsBrain_Hurt(game),
        11111221 => MotherBrainsTubesFalling_Init(game),
        11111301 => {
            MotherBrainsTubesFalling_Main(ram_word!(game, cur_enemy_index), game);
        }
        11121923 => MotherBrainsBody_Shot(game),
        11121927 => MotherBrainsBrain_Shot(game),
        11122117 => (),
        11122118 => MotherBrainsBrain_Touch(game),
        11126544 => ShitroidInCutscene_Init(game),
        11126649 => ShitroidInCutscene_Main(game),
        11128579 => ShitroidInCutscene_Touch(game),
        11129608 => DeadTorizo_Init(game),
        11129704 => DeadTorizo_Main(game),
        11129898 => DeadTorizo_Powerbomb(game),
        11129907 => DeadTorizo_Shot(game),
        11130806 => DeadSidehopper_Init(game),
        11130953 => DeadZoomer_Init(game),
        11130998 => DeadRipper_Init(game),
        11131039 => DeadSkree_Init(game),
        11131084 => DeadSidehopper_Powerbomb(game),
        11131099 => DeadSidehopper_Main(game),
        11132141 => DeadZoomer_Powerbomb(game),
        11132152 => DeadZoomer_Shot(game),
        11132157 => DeadRipper_Powerbomb(game),
        11132168 => DeadRipper_Shot(game),
        11132173 => DeadSkree_Powerbomb(game),
        11132184 => DeadSkree_Shot(game),
        11132189 => DeadSidehopper_Shot(game),
        11132228 => DeadSidehopper_Touch(game),
        11136823 => Shitroid_Init(game),
        11136954 => Shitroid_Powerbomb(game),
        11136965 => Shitroid_Main(game),
        11138953 => Shitroid_Touch(game),
        11139138 => Shitroid_Shot(game),
        11173903 => Enemy_GrappleReact_CancelBeam_AA(game),
        11173953 => Enemy_NormalFrozenAI_AA(game),
        11173964 => (),
        11191934 => Torizo_Hurt(game),
        11191972 => Torizo_Main(game),
        11192447 => Torizo_Init(game),
        11192695 => GoldTorizo_Touch(game),
        11192700 => Torizo_Shot(game),
        11195241 => GoldTorizo_Main(game),
        11195322 => GoldTorizo_Hurt(game),
        11196007 => GoldTorizo_Shot(game),
        11196359 => (),
        11196360 => TourianEntranceStatue_Init(game),
        11197603 => Shaktool_Hurt(game),
        11198019 => Shaktool_Init(game),
        11198255 => Shaktool_Touch(game),
        11198260 => Shaktool_Shot(game),
        11200278 => N00bTubeCracks_Init(game),
        11200293 => ChozoStatue_Init(game),
        11200423 => ChozoStatue_Main(game),
        11200475 => (),
        11200476 => (),
        11698191 => Enemy_GrappleReact_CancelBeam_B2(game),
        11698241 => Enemy_NormalFrozenAI_B2(game),
        11698252 => (),
        11700071 => WalkingSpacePirates_Powerbomb(game),
        11700076 => WalkingSpacePirates_Touch(game),
        11700089 => WalkingSpacePirates_Shot(game),
        11726751 => WallSpacePirates_Init(game),
        11726893 => WallSpacePirates_Main(game),
        11728350 => NinjaSpacePirates_Init(game),
        11728546 => NinjaSpacePirates_Main(game),
        11730178 => WalkingSpacePirates_Init(game),
        11730226 => WalkingSpacePirates_Main(game),
        11763712 => Enemy_GrappleReact_NoInteract_B3(game),
        11763722 => Enemy_GrappleReact_KillEnemy_B3(game),
        11763727 => Enemy_GrappleReact_CancelBeam_B3(game),
        11763747 => Enemy_NormalTouchAI_B3(game),
        11763757 => Enemy_NormalShotAI_B3(game),
        11763777 => Enemy_NormalFrozenAI_B3(game),
        11763788 => (),
        11765499 => UnusedSpinningTurtleEye_Init(game),
        11765518 => UnusedSpinningTurtleEye_Main(),
        11765819 => BrinstarPipeBug_Init(game),
        11765882 => BrinstarPipeBug_Main(game),
        11766625 => NorfairPipeBug_Init(game),
        11766686 => NorfairPipeBug_Main(game),
        11767628 => BrinstarYellowPipeBug_Init(game),
        11767726 => BrinstarYellowPipeBug_Main(game),
        11769219 => Botwoon_Init(game),
        11769448 => Botwoon_Main(game),
        11771903 => Botwoon_Touch(game),
        11771926 => Botwoon_Shot(game),
        11771969 => Botwoon_Powerbomb(game),
        11789909 => EscapeEtecoon_Main(game),
        11790027 => EscapeEtecoon_Init(game),
        11791077 => EscapeDachora_Init(game),
        11791130 => (),
        10518583 | 10649655 | 10715191 | 10780727 | 10846263 | 10911799 | 10977335 | 11042871
        | 11108407 | 11173943 | 11698231 | 11763767 => NormalEnemyPowerBombAi(game),
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn CallEnemyPreInstr(ea: u32, game: &mut Game) {
    let k: u16 = ram_word!(game, cur_enemy_index);
    match ea {
        10518651 => (),
        10649723 => (),
        10652227 => MiniCrocomire_PreInstr5(k, game),
        10652252 => MiniCrocomire_PreInstr6(k, game),
        10652277 => (),
        10657722 => SpikeShootingPlant_2(k, game),
        10657772 => SpikeShootingPlant_3(k, game),
        10657819 => (),
        10659799 => (),
        10659852 => GunshipTop_3(k, game),
        10660048 => GunshipTop_4(k, game),
        10660162 => GunshipTop_5(k, game),
        10660176 => GunshipTop_6(k, game),
        10660231 => GunshipTop_7(k, game),
        10660285 => GunshipTop_8(k, game),
        10660431 => GunshipTop_9(k, game),
        10660445 => GunshipTop_10(k, game),
        10660500 => GunshipTop_11(k, game),
        10660514 => GunshipTop_12(k, game),
        10660639 => GunshipTop_13(k, game),
        10660704 => GunshipTop_14(k, game),
        10660718 => GunshipTop_15(k, game),
        10660773 => GunshipTop_16(k, game),
        10660807 => GunshipTop_17(k, game),
        10660891 => GunshipTop_18(k, game),
        10661079 => GunshipTop_19(k, game),
        10661134 => GunshipTop_20(k, game),
        10661165 => GunshipTop_21(k, game),
        10662222 => Flies_4(k, game),
        10662268 => Flies_5(k, game),
        10662314 => Flies_6(k, game),
        10662354 => Flies_7(k, game),
        10665692 => NorfairLavajumpingEnemy_Func_1(k, game),
        10665754 => NorfairLavajumpingEnemy_Func_2(k, game),
        10665790 => NorfairLavajumpingEnemy_Func_3(k, game),
        10665852 => NorfairLavajumpingEnemy_Func_4(k, game),
        10665916 => NorfairLavajumpingEnemy_Func_5(k, game),
        10666625 => NorfairRio_Func_1(k, game),
        10666727 => NorfairRio_Func_2(k, game),
        10666815 => NorfairRio_Func_3(k, game),
        10666849 => NorfairRio_Func_4(k, game),
        10666929 => NorfairRio_Func_5(k, game),
        10667014 => NorfairRio_Func_6(k, game),
        10667822 => LowerNorfairRio_Func_1(k, game),
        10667889 => LowerNorfairRio_Func_2(k, game),
        10667963 => LowerNorfairRio_Func_3(k, game),
        10667990 => LowerNorfairRio_Func_4(k, game),
        10668077 => LowerNorfairRio_Func_5(k, game),
        10668168 => LowerNorfairRio_Func_6(k, game),
        10669926 => MaridiaLargeSnail_Func_7(k, game),
        10669993 => MaridiaLargeSnail_Func_8(k, game),
        10675796 => LavaSeahorse_Func_1(k, game),
        10675885 => LavaSeahorse_Func_2(k, game),
        10675953 => LavaSeahorse_Func_3(k, game),
        10676020 => LavaSeahorse_Func_4(k, game),
        10676041 => LavaSeahorse_Func_5(k, game),
        10676097 => (),
        10730661 => Bang_Func_6(k, game),
        10730689 => Bang_Func_7(k),
        10730693 => Bang_Func_8(k, game),
        10730780 => Bang_Func_10(k, game),
        10730796 => Bang_Func_11(k, game),
        10735455 => (),
        10736051 => MaridiaSnail_Func_15(k, game),
        10739850 => (),
        10739857 => WreckedShipOrangeZoomer_Func_2(k, game),
        10741441 => (),
        10741448 => FireZoomer_Func_1(k, game),
        10741637 => FireZoomer_Func_2(k, game),
        10741746 => FireZoomer_Func_3(k, game),
        10998952 => (),
        10998953 => Phantoon_Spawn8FireballsInCircleAtStart(k, game),
        10999022 => Phantoon_WaitBetweenSpawningAndSpinningFireballs(k, game),
        10999048 => Phantoon_SpawnFireballsBeforeFight(k, game),
        10999114 => Phantoon_WavyFadeIn(k, game),
        10999190 => Phantoon_PickPatternForRound1(k, game),
        10999271 => Phantoon_MovePhantoonInFigure8ThenOpenEye(k, game),
        10999309 => Phantoon_EyeFollowsSamusUntilTimerRunsOut(k, game),
        10999388 => Phantoon_BecomesSolidAndBodyVuln(k, game),
        10999416 => Phantoon_IsSwooping(k, game),
        10999481 => Phantoon_FadeoutWithSwoop(k, game),
        10999508 => Phantoon_WaitAfterFadeOut(k, game),
        10999522 => Phantoon_MoveLeftOrRightAndPickEyeOpenPatt(k, game),
        10999597 => Phantoon_FadeInBeforeFigure8(k, game),
        10999615 => Phantoon_BecomeSolidAfterRainingFireballs(k, game),
        10999655 => Phantoon_FadeInDuringFireballRain(k, game),
        10999688 => Phantoon_FollowSamusWithEyeDuringFireballRain(k, game),
        10999765 => Phantoon_FadeOutDuringFireballRain(k, game),
        10999799 => Phantoon_SpawnRainingFireballs(k, game),
        10999850 => Phantoon_FadeOutBeforeFirstFireballRain(k, game),
        10999900 => Phantoon_FadeOutBeforeEnrage(k, game),
        10999924 => Phantoon_MoveEnragedPhantoonToTopCenter(k, game),
        10999953 => Phantoon_FadeInEnragedPhantoon(k, game),
        10999980 => Phantoon_Enraged(k, game),
        11000086 => Phantoon_FadeoutAfterEnrage(k, game),
        11000110 => Phantoon_CompleteSwoopAfterFatalShot(k, game),
        11000136 => Phantoon_DyingPhantoonFadeInOut(k, game),
        11000203 => Phantoon_DyingPhantoonExplosions(k, game),
        11000401 => Phantoon_WavyDyingPhantoonAndCry(k, game),
        11000454 => Phantoon_DyingFadeOut(k, game),
        11000535 => Phantoon_AlmostDead(k, game),
        11000637 => Phantoon_Dead(k, game),
        11004335 => Etecoon_Func_4(k, game),
        11004416 => Etecoon_Func_5(k, game),
        11004471 => Etecoon_Func_6(k, game),
        11004597 => Etecoon_Func_7(k, game),
        11004674 => Etecoon_Func_8(k, game),
        11004716 => Etecoon_Func_9(k, game),
        11004752 => Etecoon_Func_10(k, game),
        11004877 => Etecoon_Func_11(k, game),
        11004955 => Etecoon_Func_12(k, game),
        11005079 => Etecoon_Func_16(k, game),
        11005115 => Etecoon_Func_17(k, game),
        11005151 => Etecoon_Func_18(k, game),
        11005193 => Etecoon_Func_19(k, game),
        11005226 => Etecoon_Func_20(k, game),
        11005268 => Etecoon_Func_21(k, game),
        11005301 => Etecoon_Func_22(k, game),
        11005383 => Etecoon_Func_23(k, game),
        11005502 => Etecoon_Func_24(k, game),
        11005594 => Etecoon_Func_25(k, game),
        11005624 => Etecoon_Func_26(k, game),
        11007344 => Dachora_Func_2(k, game),
        11007420 => Dachora_Func_3(k, game),
        11007469 => Dachora_Func_4(k, game),
        11007582 => Dachora_Func_5(k, game),
        11007887 => Dachora_Func_7(k, game),
        11008006 => Dachora_Func_8(k, game),
        11008309 => Dachora_Func_11(k, game),
        11008396 => Dachora_Func_12(k, game),
        11047153 => MorphBallEye_Func_1(k, game),
        11047214 => MorphBallEye_Func_2(k, game),
        11047264 => MorphBallEye_Func_3(k, game),
        11047374 => MorphBallEye_Func_4(k, game),
        11047388 => (),
        11055386 => NorfairLavaMan_Func_7(k, game),
        11055477 => NorfairLavaMan_Func_8(k, game),
        11055507 => sub_A8B193(k, game),
        11055544 => NorfairLavaMan_Func_9(k, game),
        11055581 => NorfairLavaMan_Func_10(k, game),
        11055620 => NorfairLavaMan_Func_11(k, game),
        11055761 => sub_A8B291(k),
        11055765 => NorfairLavaMan_Func_13(game),
        11055885 => NorfairLavaMan_Func_15(k, game),
        11055903 => NorfairLavaMan_Func_16(k, game),
        11055958 => NorfairLavaMan_Func_17(k, game),
        11056039 => NorfairLavaMan_Func_18(k, game),
        11059843 => MaridiaFloater_Func_3(k, game),
        11059878 => MaridiaFloater_Func_4(k, game),
        11059919 => MaridiaFloater_Func_5(k, game),
        11060075 => MaridiaFloater_Func_6(k, game),
        11060193 => MaridiaFloater_Func_7(k, game),
        11060329 => MaridiaFloater_Func_8(k, game),
        11060444 => MaridiaFloater_Func_9(k, game),
        11060480 => MaridiaFloater_Func_10(k, game),
        11060509 => MaridiaFloater_Func_11(k, game),
        11060584 => (),
        11060585 => MaridiaFloater_Func_12(k, game),
        11060639 => MaridiaFloater_Func_13(k, game),
        11192670 => (),
        11197610 => (),
        11200422 => (),
        11174011 => (),
        11763835 => (),
        11765888 => BrinstarPipeBug_PreInstr_1(k, game),
        11765904 => BrinstarPipeBug_PreInstr_2(k, game),
        11765987 => BrinstarPipeBug_PreInstr_3(k, game),
        11766044 => BrinstarPipeBug_PreInstr_4(k, game),
        11766142 => BrinstarPipeBug_PreInstr_5(k, game),
        11767039 => NorfairPipeBug_Func_5(k, game),
        11771328 => Botwoon_Func_26(k, game),
        11771517 => Botwoon_Func_27(k, game),
        11771616 => Botwoon_Func_28(k, game),
        11771700 => Botwoon_Func_29(k, game),
        11771770 => Botwoon_Func_30(k, game),
        11192146 => Torizo_Func_5(k, game),
        11192360 => Torizo_Func_6(k, game),
        11192364 => Torizo_Func_7(k, game),
        11195885 => Torizo_D5ED(k, game),
        11195889 => Torizo_D5F1(k, game),
        11199557 => sub_AAE445(k, game),
        11200430 => Shaktool_PreInstr_0(k, game),
        11200474 => (),
        11789916 => EscapeEtecoon_E65C(k, game),
        11789936 => EscapeEtecoon_E670(k, game),
        11789952 => EscapeEtecoon_E680(k, game),
        10740072 => sub_A3E168(k, game),
        10735456 => MaridiaSnail_Func_7(k, game),
        10735526 => MaridiaSnail_Func_9(k, game),
        10735543 => MaridiaSnail_CFB7(k, game),
        10735549 => MaridiaSnail_CFBD(k, game),
        10735566 => MaridiaSnail_CFCE(k, game),
        10735572 => MaridiaSnail_CFD4(k, game),
        10735589 => MaridiaSnail_CFE5(k, game),
        10735595 => MaridiaSnail_CFEB(k, game),
        10735612 => MaridiaSnail_CFFC(k, game),
        11197611 => (),
        11197612 => Shaktool_DCAC(k, game),
        11197655 => Shaktool_DCD7(k, game),
        11197733 => Shaktool_DD25(k, game),
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn CallEnemyInstr(ea: u32, k: u16, j: *const u16, game: &mut Game) -> *const u16 {
    match ea {
        10649837 => EnemyInstr_Goto(k, j),
        10649872 => EnemyInstr_DecTimerAndGoto(k, j, game),
        10649891 => EnemyInstr_SetTimer(k, j, game),
        10649903 => EnemyInstr_Sleep(k, j, game),
        10649971 => EnemyInstr_EnableOffScreenProcessing(k, j, game),
        10649981 => EnemyInstr_DisableOffScreenProcessing(k, j, game),
        10651845 => BouncingGoofball_Instr_88C5(k, j),
        10651846 => BouncingGoofball_Instr_88C6(k, j, game),
        10652030 => MiniCrocomire_Instr_897E(k, j, game),
        10652048 => MiniCrocomire_Instr_8990(k, j, game),
        10652061 => MiniCrocomire_Instr_899D(k, j, game),
        10654593 => MaridiaBeybladeTurtle_Instr_9381(k, j, game),
        10654738 => MaridiaBeybladeTurtle_Instr_9412(k, j, game),
        10654791 => MaridiaBeybladeTurtle_Instr_9447(k, j, game),
        10654801 => MaridiaBeybladeTurtle_Instr_9451(k, j, game),
        10654827 => MaridiaBeybladeTurtle_Instr_946B(k, j, game),
        10654853 => MaridiaBeybladeTurtle_Instr_9485(k, j, game),
        10654881 => MaridiaBeybladeTurtle_Instr_94A1(k, j, game),
        10654919 => MaridiaBeybladeTurtle_Instr_94C7(k, j, game),
        10654929 => MaridiaBeybladeTurtle_Instr_94D1(k, j, game),
        10657578 => SpikeShootingPlant_Instr_9F2A(k, j, game),
        10657941 => SpikeShootingPlant_Instr_A095(k, j, game),
        10657959 => SpikeShootingPlant_Instr_A0A7(k, j, game),
        10659181 => MaridiaSpikeyShell_Instr_A56D(k, j, game),
        10659185 => MaridiaSpikeyShell_Instr_A571(k, j, game),
        10664371 => Rinka_Instr_B9B3(k, j, game),
        10664381 => Rinka_Instr_B9BD(k, j, game),
        10664391 => Rinka_Instr_B9C7(k, j, game),
        10664899 => EnemyInstr_Rio_Instr_1(k, j, game),
        10665614 => NorfairLavajumpingEnemy_Instr_BE8E(k, j, game),
        10666441 => NorfairRio_Instr_C1C9(k, j, game),
        10666452 => NorfairRio_Instr_C1D4(k, j, game),
        10666463 => NorfairRio_Instr_C1DF(k, j, game),
        10666474 => NorfairRio_Instr_C1EA(k, j, game),
        10666485 => NorfairRio_Instr_C1F5(k, j, game),
        10666496 => NorfairRio_Instr_C200(k, j, game),
        10666507 => NorfairRio_Instr_C20B(k, j, game),
        10666518 => NorfairRio_Instr_C216(k, j, game),
        10666529 => NorfairRio_Instr_C221(k, j, game),
        10666540 => NorfairRio_Instr_C22C(k, j, game),
        10666551 => NorfairRio_Instr_C237(k, j, game),
        10667730 => LowerNorfairRio_Instr_C6D2(k, j, game),
        10667741 => LowerNorfairRio_Instr_C6DD(k, j, game),
        10667752 => LowerNorfairRio_Instr_C6E8(k, j, game),
        10668907 => MaridiaLargeSnail_Instr_CB6B(k, j, game),
        10669235 => MaridiaLargeSnail_Instr_CCB3(k, j, game),
        10669246 => MaridiaLargeSnail_Instr_CCBE(k, j, game),
        10669257 => MaridiaLargeSnail_Instr_CCC9(k, j, game),
        10675707 => LavaSeahorse_Instr_E5FB(k, j, game),
        10715373 => EnemyInstr_Goto(k, j),
        10715439 => EnemyInstr_Sleep(k, j, game),
        10715507 => EnemyInstr_EnableOffScreenProcessing(k, j, game),
        10715517 => EnemyInstr_DisableOffScreenProcessing(k, j, game),
        10716899 => Waver_Instr_1(k, j, game),
        10717526 => Metalee_Instr_1(k, j, game),
        10719382 => MaridiaFish_Instr_3(k, j, game),
        10719392 => MaridiaFish_Instr_1(k, j, game),
        10719402 => MaridiaFish_Instr_2(k, j, game),
        10722411 => PlatformThatFallsWithSamus_Instr_3(k, j, game),
        10722422 => PlatformThatFallsWithSamus_Instr_4(k, j, game),
        10722433 => PlatformThatFallsWithSamus_Instr_1(k, j, game),
        10722444 => PlatformThatFallsWithSamus_Instr_2(k, j, game),
        10725992 => Sidehopper_Func_1(k, j, game),
        10726142 => Sidehopper_Instr_1(k, j, game),
        10728489 => MaridiaRefillCandy_Instr_1(k, j, game),
        10728500 => MaridiaRefillCandy_Instr_2(k, j, game),
        10728511 => MaridiaRefillCandy_Instr_3(k, j, game),
        10730104 => Bang_Instr_1(k, j, game),
        10730152 => Bang_Instr_2(k, j, game),
        10733220 => Skree_Instr_1(k, j, game),
        10734646 => MaridiaSnail_Instr_1(k, j, game),
        10734655 => MaridiaSnail_Instr_2(k, j, game),
        10734664 => MaridiaSnail_Instr_4(k, j, game),
        10734687 => MaridiaSnail_Instr_3(k, j, game),
        10734712 => MaridiaSnail_Instr_5(k, j, game),
        10738632 => Reflec_Instr_1(k, j, game),
        10739650 => WreckedShipOrangeZoomer_Func_1(k, j, game),
        10741344 => Zoomer_Instr_SetPreinstr(k, j, game),
        10742437 => Metroid_Instr_2(k, j, game),
        10742449 => Metroid_Instr_1(k, j, game),
        10780909 => EnemyInstr_Goto(k, j),
        10780975 => EnemyInstr_Sleep(k, j, game),
        10782374 => Crocomire_Instr_1(k, j, game),
        10782546 => Crocomire_Instr_14(k, j, game),
        10783995 => Crocomire_Instr_11(k, j, game),
        10784007 => Crocomire_Instr_7(k, j, game),
        10784019 => Crocomire_Instr_19(k, j, game),
        10784711 => Crocomire_Instr_2(k, j, game),
        10784735 => Crocomire_Instr_4(k, j, game),
        10784762 => Crocomire_Instr_3(k, j, game),
        10784767 => Crocomire_Instr_15(k, j, game),
        10784797 => Crocomire_Instr_16(k, j, game),
        10784859 => Crocomire_Instr_13(k, j, game),
        10784895 => Crocomire_Instr_18(k, j, game),
        10784911 => Crocomire_Instr_12(k, j, game),
        10784916 => Crocomire_Instr_17(k, j, game),
        10787483 => Crocomire_Instr_8(k, j, game),
        10787488 => Crocomire_Instr_6(k, j, game),
        10787493 => Crocomire_Instr_9(k, j, game),
        10787498 => Crocomire_Instr_5(k, j, game),
        10787503 => Crocomire_Instr_20(k, j, game),
        10787508 => Crocomire_Instr_21(k, j, game),
        10787513 => Crocomire_Instr_22(k, j, game),
        10787518 => Crocomire_Instr_23(k, j, game),
        10787523 => Crocomire_Instr_24(k, j, game),
        10787528 => Crocomire_Instr_10(k, j, game),
        10787533 => Crocomire_Instr_25(k, j, game),
        10787538 => Crocomire_Instr_26(k, j, game),
        10787543 => Crocomire_Instr_27(k, j, game),
        10846332 => EnemyInstr_StopScript(k, j, game),
        10846445 => EnemyInstr_Goto(k, j),
        10846480 => EnemyInstr_DecTimerAndGoto(k, j, game),
        10846499 => EnemyInstr_SetTimer(k, j, game),
        10846511 => EnemyInstr_Sleep(k, j, game),
        10846522 => EnemyInstr_WaitNframes(k, j, game),
        10851549 => Draygon_Instr_1(k, j, game),
        10852150 => Draygon_Instr_13(k, j, game),
        10852159 => Draygon_Instr_8(k, j, game),
        10852178 => Draygon_Instr_7(k, j, game),
        10852197 => Draygon_Instr_6(k, j, game),
        10852216 => Draygon_Instr_9(k, j, game),
        10852501 => Draygon_Instr_2(k, j, game),
        10852563 => Draygon_Instr_11(k, j, game),
        10852591 => Draygon_Instr_5(k, j, game),
        10853274 => Draygon_Instr_15(k, j, game),
        10853514 => Draygon_Instr_17(k, j, game),
        10853898 => Draygon_Instr_14(k, j, game),
        10854231 => Draygon_Instr_16(k, j, game),
        10854240 => Draygon_Instr_10(k, j, game),
        10854254 => Draygon_Instr_4(k, j, game),
        10854268 => Draygon_Instr_12(k, j, game),
        10854318 => Draygon_Instr_18(k, j, game),
        10863739 => Draygon_Instr_3(k, j, game),
        10872671 => Draygon_Instr_25(k, j, game),
        10872689 => Draygon_Instr_24(k, j, game),
        10872877 => Draygon_Instr_21(k, j, game),
        10872946 => Draygon_Instr_22(k, j, game),
        10872956 => Draygon_Instr_27(k, j, game),
        10872981 => Draygon_Instr_23(k, j, game),
        10873009 => Draygon_Instr_30(k, j, game),
        10873018 => Draygon_Instr_20(k, j, game),
        10873034 => Draygon_Instr_29(k, j, game),
        10873116 => Draygon_Instr_19(k, j, game),
        10873198 => Draygon_Instr_28(k, j, game),
        10873265 => Draygon_Instr_26(k, j, game),
        10911981 => EnemyInstr_Goto(k, j),
        10912047 => EnemyInstr_Sleep(k, j, game),
        10915247 => FireGeyser_Instr_1(k, j, game),
        10915347 => FireGeyser_Instr_2(k, j, game),
        10915373 => FireGeyser_Instr_3(k, j, game),
        10915393 => FireGeyser_Instr_4(k, j, game),
        10915413 => FireGeyser_Instr_5(k, j, game),
        10915433 => FireGeyser_Instr_6(k, j, game),
        10915453 => FireGeyser_Instr_7(k, j, game),
        10915473 => FireGeyser_Instr_8(k, j, game),
        10915493 => FireGeyser_Instr_9(k, j, game),
        10915513 => FireGeyser_Instr_10(k, j, game),
        10915533 => FireGeyser_Instr_11(k, j, game),
        10915553 => FireGeyser_Instr_12(k, j, game),
        10915573 => FireGeyser_Instr_13(k, j, game),
        10915593 => FireGeyser_Instr_14(k, j, game),
        10915613 => FireGeyser_Instr_15(k, j, game),
        10915633 => FireGeyser_Instr_16(k, j, game),
        10915653 => FireGeyser_Instr_17(k, j, game),
        10915673 => FireGeyser_Instr_18(k, j, game),
        10915693 => FireGeyser_Instr_19(k, j, game),
        10915713 => FireGeyser_Instr_20(k, j, game),
        10915733 => FireGeyser_Instr_21(k, j, game),
        10915753 => FireGeyser_Instr_22(k, j, game),
        10915773 => FireGeyser_Instr_23(k, j, game),
        10915793 => FireGeyser_Instr_24(k, j, game),
        10918694 => FakeKraid_Instr_2(k, j, game),
        10918772 => FakeKraid_Instr_1(k, j, game),
        10918834 => FakeKraid_Instr_3(k, j, game),
        10918852 => FakeKraid_Instr_4(k, j, game),
        10918914 => FakeKraid_Instr_5(k, j, game),
        10937534 => Ridley_Instr_5(k, j, game),
        10937546 => Ridley_Instr_6(k, j, game),
        10937554 => Ridley_Instr_10(k, j, game),
        10937582 => Ridley_Instr_4(k, j, game),
        10937592 => Ridley_Instr_3(k, j, game),
        10937601 => Ridley_Instr_2(k, j, game),
        10937623 => Ridley_Instr_1(k, j, game),
        10937631 => Ridley_Instr_14(k, j, game),
        10938140 => Ridley_Instr_9(k, j, game),
        10938151 => Ridley_Instr_7(k, j, game),
        10938159 => Ridley_Instr_8(k, j, game),
        10938445 => Ridley_Instr_11(k, j, game),
        10938628 => Ridley_Instr_12(k, j, game),
        10938633 => Ridley_Instr_13(k, j, game),
        10938729 => Ridley_Instr_15(k, j, game),
        10938742 => Ridley_Instr_16(k, j, game),
        10940701 => CeresSteam_Instr_1(k, j, game),
        10940711 => CeresSteam_Instr_2(k, j, game),
        10940725 => CeresSteam_Instr_3(k, j, game),
        10942014 => CeresDoor_Instr_6(k, j, game),
        10942058 => CeresDoor_Instr_4(k, j, game),
        10942072 => CeresDoor_Instr_8(k, j, game),
        10942091 => CeresSteam_Instr_4(k, j, game),
        10942101 => CeresDoor_Instr_1(k, j, game),
        10942111 => CeresDoor_Instr_3(k, j, game),
        10942118 => CeresSteam_Instr_5(k, j, game),
        10942128 => CeresDoor_Instr_5(k, j, game),
        10942131 => CeresDoor_Instr_2(k, j, game),
        10942141 => CeresDoor_Instr_7(k, j, game),
        10977418 => EnemyInstr_Call_A7(k, j, game),
        10977517 => EnemyInstr_Goto(k, j),
        10977552 => EnemyInstr_DecTimerAndGoto(k, j, game),
        10977571 => EnemyInstr_SetTimer(k, j, game),
        10977583 => EnemyInstr_Sleep(k, j, game),
        10979983 => Kraid_Instr_9(k, j, game),
        10991155 => Kraid_Instr_1(k, j),
        10991158 => Kraid_Instr_DecYpos(k, j, game),
        10991164 => Kraid_Instr_IncrYpos_Shake(k, j, game),
        10991182 => Kraid_Instr_PlaySound_0x76(k, j, game),
        10991194 => Kraid_Instr_XposMinus3(k, j, game),
        10991207 => Kraid_Instr_XposMinus3b(k, j, game),
        10991220 => Kraid_Instr_XposPlus3(k, j, game),
        10991235 => Kraid_Instr_MoveHimRight(k, j, game),
        11043053 => EnemyInstr_Goto(k, j),
        11043088 => EnemyInstr_DecTimerAndGoto(k, j, game),
        11043107 => EnemyInstr_SetTimer(k, j, game),
        11043119 => EnemyInstr_Sleep(k, j, game),
        11043187 => EnemyInstr_EnableOffScreenProcessing(k, j, game),
        11043197 => EnemyInstr_DisableOffScreenProcessing(k, j, game),
        11044751 => MiniDraygon_Instr_2(k, j, game),
        11044763 => MiniDraygon_Instr_1(k, j, game),
        11044790 => MiniDraygon_Instr_3(k, j, game),
        11044811 => MiniDraygon_Instr_4(k, j, game),
        11048485 => Fune_Instr_2(k, j, game),
        11048497 => Fune_Instr_6(k, j, game),
        11048522 => Fune_Instr_7(k, j, game),
        11048547 => Fune_Instr_1(k, j, game),
        11048572 => Fune_Instr_4(k, j, game),
        11048597 => Fune_Instr_3(k, j, game),
        11048628 => Fune_Instr_5(k, j, game),
        11051207 => YappingMaw_Instr_2(k, j, game),
        11051225 => YappingMaw_Instr_4(k, j, game),
        11051243 => YappingMaw_Instr_5(k, j, game),
        11051261 => YappingMaw_Instr_7(k, j, game),
        11051279 => YappingMaw_Instr_3(k, j, game),
        11051297 => YappingMaw_Instr_6(k, j, game),
        11051315 => YappingMaw_Instr_1(k, j, game),
        11054610 => NorfairLavaMan_Instr_1(k, j, game),
        11054630 => NorfairLavaMan_Instr_8(k, j, game),
        11054640 => NorfairLavaMan_Instr_14(k, j, game),
        11054650 => NorfairLavaMan_Instr_2(k, j, game),
        11054661 => NorfairLavaMan_Instr_7(k, j, game),
        11054672 => NorfairLavaMan_Instr_10(k, j, game),
        11054682 => NorfairLavaMan_Instr_12(k, j, game),
        11054692 => NorfairLavaMan_Instr_9(k, j, game),
        11054728 => NorfairLavaMan_Instr_11(k, j, game),
        11054742 => NorfairLavaMan_Instr_13(k, j, game),
        11054778 => NorfairLavaMan_Instr_5(k, j, game),
        11054794 => NorfairLavaMan_Instr_15(k, j, game),
        11054820 => NorfairLavaMan_Instr_4(k, j, game),
        11054846 => NorfairLavaMan_Instr_16(k, j, game),
        11054872 => NorfairLavaMan_Instr_6(k, j, game),
        11054916 => NorfairLavaMan_Instr_3(k, j, game),
        11056990 => Beetom_Instr_1(k, j),
        11062537 => WreckedShipRobot_Instr_4(k, j, game),
        11062692 => WreckedShipRobot_Instr_9(k, j, game),
        11062762 => WreckedShipRobot_Instr_6(k, j, game),
        11062917 => WreckedShipRobot_Instr_8(k, j, game),
        11062987 => WreckedShipRobot_Instr_7(k, j),
        11062991 => WreckedShipRobot_Instr_15(k, j, game),
        11063146 => WreckedShipRobot_Instr_18(k, j, game),
        11063216 => WreckedShipRobot_Instr_16(k, j, game),
        11063371 => WreckedShipRobot_Instr_17(k, j, game),
        11063441 => WreckedShipRobot_Instr_3(k, j, game),
        11063490 => WreckedShipRobot_Instr_10(k, j),
        11063494 => WreckedShipRobot_Instr_14(k, j, game),
        11063506 => WreckedShipRobot_Instr_2(k, j, game),
        11063552 => WreckedShipRobot_Instr_13(k, j, game),
        11063559 => WreckedShipRobot_Instr_1(k, j, game),
        11063601 => WreckedShipRobot_Instr_12(k, j, game),
        11063613 => WreckedShipRobot_Instr_5(k, j, game),
        11063659 => WreckedShipRobot_Instr_11(k, j, game),
        11067164 => WalkingLavaSeahorse_Instr_4(k, j, game),
        11067187 => WalkingLavaSeahorse_Instr_3(k, j, game),
        11067193 => WalkingLavaSeahorse_Instr_5(k, j, game),
        11067199 => WalkingLavaSeahorse_Instr_6(k, j, game),
        11067235 => WalkingLavaSeahorse_Instr_2(k, j, game),
        11067249 => WalkingLavaSeahorse_Instr_1(k, j, game),
        11068957 => WreckedShipSpark_Instr_2(k, j, game),
        11068970 => WreckedShipSpark_Instr_1(k, j, game),
        11072806 => KiHunter_Instr_1(k, j, game),
        11072996 => KiHunter_Instr_2(k, j, game),
        11073151 => KiHunter_Instr_3(k, j, game),
        11073234 => KiHunter_Instr_4(k, j, game),
        11073240 => KiHunter_Instr_5(k, j, game),
        11108655 => EnemyInstr_Sleep(k, j, game),
        11128756 => Shitroid_Instr_1(k, j),
        11128778 => Shitroid_Instr_2(k, j),
        11136208 => sub_A9ECD0(k, j, game),
        11139360 => Shitroid_Instr_3(k, j),
        11139382 => Shitroid_Instr_4(k, j),
        11139472 => Shitroid_Instr_6(k, j),
        11139476 => Shitroid_Instr_5(k, j, game),
        11173995 => Enemy_SetAiPreInstr_AA(k, j, game),
        11174004 => Enemy_ClearAiPreInstr_AA(k, j, game),
        11174012 => EnemyInstr_StopScript(k, j, game),
        11174125 => EnemyInstr_Goto(k, j),
        11174160 => EnemyInstr_DecTimerAndGoto(k, j, game),
        11174179 => EnemyInstr_SetTimer(k, j, game),
        11174191 => EnemyInstr_Sleep(k, j, game),
        11174202 => EnemyInstr_WaitNframes(k, j, game),
        11174219 => EnemyInstr_CopyToVram(k, j, game),
        11186332 => Torizo_Instr_3(k, j, game),
        11186461 => Torizo_Instr_31(k, j, game),
        11186622 => Torizo_Instr_33(k, j, game),
        11186724 => Torizo_Instr_36(k, j, game),
        11186734 => Torizo_Instr_37(k, j, game),
        11186744 => Torizo_Instr_35(k, j, game),
        11186765 => Torizo_Instr_38(k, j, game),
        11186801 => Torizo_Instr_6(k, j, game),
        11188557 => Torizo_Instr_5(k, j, game),
        11188561 => Torizo_Instr_9(k, j, game),
        11190984 => Torizo_Instr_7(k, j),
        11190985 => Torizo_Instr_2(k, j, game),
        11190993 => Torizo_Instr_8(k, j, game),
        11191001 => Torizo_Instr_25(k, j, game),
        11191021 => Torizo_Instr_22(k, j, game),
        11191031 => Torizo_Instr_19(k, j, game),
        11191037 => Torizo_Instr_32(k, j, game),
        11191043 => Torizo_Instr_30(k, j, game),
        11191087 => Torizo_Instr_34(k, j, game),
        11191114 => Torizo_Instr_24(k, j, game),
        11191131 => Torizo_Instr_12(k, j, game),
        11191149 => Torizo_Instr_10(k, j, game),
        11191159 => Torizo_Instr_11(k, j, game),
        11191178 => Torizo_Instr_29(k, j, game),
        11191200 => Torizo_Instr_1(k, j, game),
        11191222 => Torizo_Instr_28(k, j, game),
        11191244 => Torizo_Instr_4(k, j, game),
        11191326 => Torizo_Instr_40(k, j, game),
        11191408 => Torizo_Instr_16(k, j, game),
        11191525 => Torizo_Instr_27(k, j, game),
        11191642 => Torizo_Instr_23(k, j, game),
        11191655 => Torizo_Instr_14(k, j, game),
        11191691 => Torizo_Instr_15(k, j, game),
        11191716 => Torizo_Instr_26(k, j, game),
        11191755 => Torizo_Instr_18(k, j, game),
        11191779 => Torizo_Instr_20(k, j, game),
        11191794 => Torizo_Instr_44(k, j, game),
        11191809 => Torizo_Instr_21(k, j, game),
        11191824 => Torizo_Instr_17(k, j, game),
        11191832 => Torizo_Instr_13(k, j, game),
        11193038 => Torizo_Instr_39(k, j, game),
        11193054 => Torizo_Instr_41(k, j, game),
        11193058 => Torizo_Instr_42(k, j, game),
        11193815 => Torizo_Instr_48(k, j, game),
        11194601 => Torizo_Instr_57(k, j, game),
        11194611 => Torizo_Instr_58(k, j, game),
        11194747 => Torizo_Instr_59(k, j, game),
        11194759 => Torizo_Instr_62(k, j, game),
        11194855 => Torizo_Instr_63(k, j, game),
        11195279 => Torizo_Instr_56(k, j, game),
        11195287 => Torizo_Instr_60(k, j, game),
        11195295 => Torizo_Instr_46(k, j, game),
        11195360 => Torizo_Instr_47(k, j, game),
        11195370 => Torizo_Instr_49(k, j, game),
        11195446 => Torizo_Instr_61(k, j, game),
        11195461 => Torizo_Instr_53(k, j, game),
        11195508 => Torizo_Instr_55(k, j, game),
        11195547 => Torizo_Instr_52(k, j, game),
        11195578 => Torizo_Instr_50(k, j, game),
        11195635 => Torizo_Instr_43(k, j, game),
        11195645 => Torizo_Instr_51(k, j, game),
        11195686 => Torizo_Instr_45(k, j, game),
        11195725 => Torizo_Instr_54(k, j, game),
        11196721 => Shaktool_Instr_2(k, j, game),
        11196735 => Shaktool_Instr_3(k, j, game),
        11196746 => Shaktool_Instr_4(k, j, game),
        11196755 => Shaktool_Instr_5(k, j, game),
        11196831 => Shaktool_Instr_6(k, j),
        11196858 => Shaktool_Instr_1(k, j, game),
        11199529 => Shaktool_Instr_9(k, j, game),
        11199542 => Shaktool_Instr_11(k, j, game),
        11199549 => Shaktool_Instr_10(k, j, game),
        11199871 => Shaktool_Instr_8(k, j, game),
        11199879 => Shaktool_Instr_13(k, j, game),
        11199887 => Shaktool_Instr_12(k, j, game),
        11199960 => Shaktool_Instr_7(k, j, game),
        11200240 => Shaktool_Instr_14(k, j, game),
        11698413 => EnemyInstr_Goto(k, j),
        11698448 => EnemyInstr_DecTimerAndGoto(k, j, game),
        11698467 => EnemyInstr_SetTimer(k, j, game),
        11698479 => EnemyInstr_Sleep(k, j, game),
        11698490 => EnemyInstr_WaitNframes(k, j, game),
        11726400 => SpacePirates_Instr_MovePixelsDownAndChangeDirFaceRight(k, j, game),
        11726450 => SpacePirates_Instr_MovePixelsDownAndChangeDirFaceLeft(k, j, game),
        11726500 => SpacePirates_Instr_RandomNewDirFaceR(k, j, game),
        11726524 => SpacePirates_Instr_RandomNewDirFaceL(k, j, game),
        11726548 => SpacePirates_Instr_PrepareWallJumpR(k, j, game),
        11726589 => SpacePirates_Instr_PrepareWallJumpL(k, j, game),
        11726634 => SpacePirates_Instr_FireLaserL(k, j, game),
        11726685 => SpacePirates_Instr_FireLaserR(k, j, game),
        11726723 => SpacePirates_Instr_SetEnemyFunc(k, j, game),
        11726739 => SpacePirates_Instr_PlaySfx(k, j, game),
        11728182 => SpacePirates_Instr_20(k, j, game),
        11728198 => SpacePirates_Instr_16(k, j, game),
        11728228 => SpacePirates_Instr_15(k, j, game),
        11728272 => SpacePirates_Instr_18(k, j, game),
        11728342 => SpacePirates_Instr_17(k, j, game),
        11729257 => SpacePirates_Instr_19(k, j, game),
        11729469 => SpacePirates_Instr_21(k, j, game),
        11730024 => SpacePirates_Instr_12(k, j, game),
        11730064 => SpacePirates_Instr_14(k, j, game),
        11730104 => SpacePirates_Instr_11(k, j, game),
        11730120 => SpacePirates_Instr_13(k, j, game),
        11763819 => Enemy_SetAiPreInstr_B3(k, j, game),
        11763828 => Enemy_ClearAiPreInstr_B3(k, j, game),
        11763949 => EnemyInstr_Goto(k, j),
        11763984 => EnemyInstr_DecTimerAndGoto(k, j, game),
        11764003 => EnemyInstr_SetTimer(k, j, game),
        11764015 => EnemyInstr_Sleep(k, j, game),
        11769031 => Botwoon_Instr_1(k, j, game),
        11769047 => Botwoon_Instr_2(k, j, game),
        11769063 => Botwoon_Instr_3(k, j, game),
        11769079 => Botwoon_Instr_4(k, j, game),
        11769095 => Botwoon_Instr_5(k, j, game),
        11769111 => Botwoon_Instr_6(k, j, game),
        11769127 => Botwoon_Instr_7(k, j, game),
        11769143 => Botwoon_Instr_8(k, j, game),
        11769159 => Botwoon_Instr_9(k, j, game),
        11769175 => Botwoon_Instr_10(k, j, game),
        11769191 => Botwoon_Instr_SetSpitting(k, j, game),
        11769202 => Botwoon_Instr_QueueSpitSfx(k, j, game),
        11789637 => EscapeEtecoon_Instr_1(k, j, game),
        11789840 => EscapeEtecoon_Instr_2(k, j, game),
        11791016 => EscapeDachora_Instr_2(k, j, game),
        11791032 => EscapeDachora_Instr_3(k, j, game),
        11791049 => EscapeDachora_Instr_1(k, j, game),
        11791063 => EscapeDachora_Instr_4(k, j, game),
        11113910 => MotherBrain_Instr_MoveBodyUp10Left4(k, j, game),
        11113920 => MotherBrain_Instr_MoveBodyUp16Left4(k, j, game),
        11113930 => MotherBrain_Instr_MoveBodyUp12Right2(k, j, game),
        11113950 => MotherBrain_Instr_MoveDown12Left4(k, j, game),
        11113960 => MotherBrain_Instr_MoveDown16Right2(k, j, game),
        11113970 => MotherBrain_Instr_MoveDown10Right2(k, j, game),
        11113980 => MotherBrain_Instr_MoveUp2Right1(k, j, game),
        11113996 => MotherBrain_Instr_MoveRight2(k, j, game),
        11114012 => MotherBrain_Instr_MoveUp1(k, j, game),
        11114018 => MotherBrain_Instr_MoveUp1Right3_Sfx(k, j, game),
        11114040 => MotherBrain_Instr_Down2Right15(k, j, game),
        11114056 => MotherBrain_Instr_Down4Right6(k, j, game),
        11114072 => MotherBrain_Instr_Up4Left2(k, j, game),
        11114088 => MotherBrain_Instr_Up2Left1_Sfx(k, j, game),
        11114110 => MotherBrain_Instr_Up2Left1_Sfx2(k, j, game),
        11114132 => MotherBrain_Instr_MoveLeft2(k, j, game),
        11114148 => MotherBrain_Instr_MoveDown1(k, j, game),
        11114154 => MotherBrain_Instr_MoveDown1Left3(k, j, game),
        11114170 => MotherBrain_Instr_MoveUp2Left15_Sfx(k, j, game),
        11114192 => MotherBrain_Instr_MoveUp4Left6(k, j, game),
        11114208 => MotherBrain_Instr_MoveDown4Right2(k, j, game),
        11114224 => MotherBrain_Instr_MoveDown2Right1(k, j, game),
        11114240 => MotherBrain_Instr_SetPose_Standing(k, j, game),
        11114248 => MotherBrain_Instr_SetPose_Walking(k, j, game),
        11114256 => MotherBrain_Instr_SetPose_Crouched(k, j, game),
        11114264 => MotherBrain_Instr_SetPose_CrouchedTrans(k, j, game),
        11114272 => MotherBrain_Instr_SetPose_DeathBeamMode(k, j, game),
        11114280 => MotherBrain_Instr_SetPose_LeaningDown(k, j, game),
        11115208 => MotherBrain_Instr_SpawnEprojToOffset(k, j, game),
        11115247 => MotherBrain_Instr_SpawnDeathBeamEproj(k, j, game),
        11115269 => MotherBrain_Instr_IncrBeamAttackPhase(k, j, game),
        _ => {
            Unreachable(game);
            null::<u16>()
        }
    }
}
pub unsafe fn EnemyMain(game: &mut Game) {
    let mut ED: *mut EnemyDef = null_mut::<EnemyDef>();
    let mut v6: i16 = 0;
    let mut v8: i8 = 0;
    if ram_word!(game, first_free_enemy_index) != 0 {
        if ram_word!(game, enemy_index_to_shake) != 0xffffu16 {
            (*gEnemyData(ram_word!(game, enemy_index_to_shake) as i32, game)).shake_timer = 64u16;
            set_ram_word!(game.enemy_index_to_shake = -1i32 as u16);
        }
        let mut current_block_39: u64;
        let mut active_enemy_indexes_index: i32 = 0i32;
        loop {
            let v1: u16 = ram_word!(
                game,
                active_enemy_indexes[(active_enemy_indexes_index >> 1i32)]
            );
            if v1 == 0xffffu16 {
                break;
            }
            set_ram_word!(game.cur_enemy_index = v1);
            let E: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
            if ((*E).properties as i32 & 0x400i32) == 0i32 {
                if (*E).invincibility_timer != 0 {
                    (*E).invincibility_timer = (*E).invincibility_timer.wrapping_sub(1);
                    current_block_39 = 8457315219000651999;
                } else if ram_word!(game, debug_disable_sprite_interact) == 0 {
                    if ram_word!(game, debug_time_frozen_for_enemies) as i32
                        | ram_word!(game, time_is_frozen_flag) as i32
                        == 0
                    {
                        EnemyCollisionHandler(game);
                        if (*E).enemy_ptr == 0 {
                            current_block_39 = 10758786907990354186;
                        } else {
                            current_block_39 = 8236137900636309791;
                        }
                    } else {
                        current_block_39 = 8236137900636309791;
                    }
                    match current_block_39 {
                        10758786907990354186 => {}
                        _ => {
                            if ((*E).extra_properties as i32 & 1i32) != 0i32 {
                                current_block_39 = 1836292691772056875;
                            } else {
                                current_block_39 = 8457315219000651999;
                            }
                        }
                    }
                } else {
                    current_block_39 = 8457315219000651999;
                }
            } else {
                current_block_39 = 8457315219000651999;
            }
            if current_block_39 == 8457315219000651999 {
                set_ram_word!(game.UNUSED_word_7E17A2 = 0u16);
                if ram_word!(game, debug_time_frozen_for_enemies) as i32
                    | ram_word!(game, time_is_frozen_flag) as i32
                    == 0
                {
                    v6 = 0i16;
                    let mut ai_handler_bits: u16 = (*E).ai_handler_bits;
                    if ai_handler_bits != 0 {
                        loop {
                            v6 += 1;
                            v8 = ((ai_handler_bits) as i8) & (1i8);
                            ai_handler_bits = (ai_handler_bits as i32 >> 1i32) as u16;
                            if (v8 != 0) {
                                break;
                            }
                        }
                    }
                    CallEnemyAi(
                        (((*E).bank as i32) << 16i32
                            | (*get_EnemyDef_A2(
                                ((*E).enemy_ptr as i32 + 2i32 * v6 as i32) as u16,
                                game,
                            ))
                            .main_ai as i32) as u32,
                        game,
                    );
                    current_block_39 = 5447921980315815038;
                } else {
                    ED = get_EnemyDef_A2((*E).enemy_ptr, game);
                    if (*ED).time_is_frozen_ai != 0 {
                        CallEnemyAi(
                            (((*E).bank as i32) << 16i32 | (*ED).time_is_frozen_ai as i32) as u32,
                            game,
                        );
                        current_block_39 = 5447921980315815038;
                    } else {
                        current_block_39 = 1836292691772056875;
                    }
                }
                if current_block_39 == 1836292691772056875 {
                } else {
                    if ram_word!(game, debug_time_frozen_for_enemies) as i32
                        | ram_word!(game, time_is_frozen_flag) as i32
                        == 0
                    {
                        (*E).frame_counter = (*E).frame_counter.wrapping_add(1);
                        if ((*E).properties as i32 & 0x2000i32) != 0i32 {
                            set_ram_word!(game.enemy_processing_stage = 2u16);
                            ProcessEnemyInstructions(game);
                        }
                    }
                    current_block_39 = 1836292691772056875;
                }
            }
            if current_block_39 == 1836292691772056875 {
                if ((*E).extra_properties as i32 & 1i32) != 0i32
                    && ((*E).flash_timer == 1u16 || (*E).frozen_timer == 1u16)
                {
                    (*gEnemySpawnData(ram_word!(game, cur_enemy_index) as i32, game))
                        .cause_of_death = 0u16;
                    EnemyDeathAnimation(ram_word!(game, cur_enemy_index), 0u16, game);
                }
                if (((*E).extra_properties as i32 & 4i32) != 0i32
                    || EnemyWithNormalSpritesIsOffScreen(game) == 0)
                    && ((*E).properties as i32 & 0x300i32) == 0i32
                    && (ram_word!(game, UNUSED_word_7E17A2) as i32 & 1i32) == 0i32
                {
                    DrawOneEnemy(game);
                }
            }
            if (*E).flash_timer != 0u16
                && ram_word!(game, debug_time_frozen_for_enemies) as i32
                    | ram_word!(game, time_is_frozen_flag) as i32
                    == 0
            {
                (*E).flash_timer = (*E).flash_timer.wrapping_sub(1);
                if (*E).flash_timer as i32 - 8i32 & 0x8000i32 != 0 {
                    (*E).ai_handler_bits = ((*E).ai_handler_bits as i32 & !(2i32)) as u16;
                }
            }
            active_enemy_indexes_index += 2i32;
        }
    }
    HandleSpriteObjects(game);
    update_ram_word!(game.random_enemy_counter, wrapping_add(1));
    set_ram_word!(game.enemy_index_colliding_dirs[0] = -1i32 as u16);
    set_ram_word!(game.enemy_index_colliding_dirs[1] = -1i32 as u16);
    set_ram_word!(game.enemy_index_colliding_dirs[2] = -1i32 as u16);
    set_ram_word!(game.enemy_index_colliding_dirs[3] = -1i32 as u16);
}
pub unsafe fn DecrementSamusTimers(game: &mut Game) {
    if ram_word!(game, samus_invincibility_timer) != 0 {
        update_ram_word!(game.samus_invincibility_timer, wrapping_sub(1));
    }
    if ram_word!(game, samus_knockback_timer) != 0 {
        update_ram_word!(game.samus_knockback_timer, wrapping_sub(1));
    }
    if ram_word!(game, projectile_invincibility_timer) != 0 {
        update_ram_word!(game.projectile_invincibility_timer, wrapping_sub(1));
    }
    set_ram_word!(game.interactive_enemy_indexes[0] = -1i32 as u16);
    set_ram_word!(game.active_enemy_indexes[0] = -1i32 as u16);
}
pub unsafe fn SpawnEnemyDrops(a: u16, k: u16, varE20: u16, game: &mut Game) {
    game.eproj_spawn_varE24 = a;
    SpawnEprojWithGfx(varE20, k, addr_kEproj_Pickup as u16, game);
}
pub unsafe fn DeleteEnemyAndConnectedEnemies(game: &mut Game) {
    let v0: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    let mut num_parts: u16 = (*get_EnemyDef_A2((*v0).enemy_ptr, game)).num_parts;
    if num_parts == 0 {
        num_parts = 1u16;
    }
    let mut v2: u16 = num_parts;
    loop {
        (*gEnemyData(ram_word!(game, cur_enemy_index) as i32, game)).enemy_ptr = 0u16;
        update_ram_word!(game.cur_enemy_index, wrapping_add(64u16));
        v2 = v2.wrapping_sub(1);
        if (v2 == 0) {
            break;
        }
    }
}
pub unsafe fn SpawnEnemy(db: u8, k: u16, game: &mut Game) -> u16 {
    let current_block: u64;
    let mut varE20: u16 = k;
    let cur_enemy_index_backup: u16 = ram_word!(game, cur_enemy_index);
    let enemy_ptr: u16 = (*get_EnemyPopulation(db, k, game)).enemy_ptr;
    let mut v3: i16 = (((*get_EnemyDef_A2(enemy_ptr, game)).num_parts) as i16).wrapping_sub(1i16);
    if v3 < 0i16 {
        v3 = 0i16;
    }
    let mut varE22: u16 = v3 as u16;
    let mut varE26: u16 = v3 as u16;
    let mut new_enemy_index: u16 = 0u16;
    's_26: loop {
        let mut v4: u16 = new_enemy_index;
        while (*gEnemyData(v4 as i32, game)).enemy_ptr == 0 {
            if varE22 == 0 {
                current_block = 12137821391237576008;
                break 's_26;
            }
            varE22 = varE22.wrapping_sub(1);
            v4 = (v4).wrapping_add(64u16);
            if v4 >= 2048u16 {
                return 0xffffu16;
            }
        }
        new_enemy_index = (new_enemy_index).wrapping_add(64u16);
        if (new_enemy_index >= 2048u16) {
            current_block = 7651349459974463963;
            break;
        }
    }
    match current_block {
        12137821391237576008 => loop {
            let E: *mut EnemyData = gEnemyData(new_enemy_index as i32, game);
            let EP: *mut EnemyPopulation = get_EnemyPopulation(db, varE20, game);
            let mut v10: i32 = 0i32;
            if (*EP).enemy_ptr == ram_word!(game, enemy_def_ptr[0])
                || {
                    v10 = 1i32;
                    (*EP).enemy_ptr == ram_word!(game, enemy_def_ptr[1])
                }
                || {
                    v10 = 2i32;
                    (*EP).enemy_ptr == ram_word!(game, enemy_def_ptr[2])
                }
                || {
                    v10 = 3i32;
                    (*EP).enemy_ptr == ram_word!(game, enemy_def_ptr[3])
                }
            {
                (*E).vram_tiles_index = ram_word!(game, enemy_gfxdata_tiles_index[v10]);
                (*E).palette_index = (2u16).wrapping_mul(u16::swap_bytes(ram_word!(
                    game,
                    enemy_gfxdata_vram_ptr[v10]
                )));
            } else {
                (*E).vram_tiles_index = 0u16;
                (*E).palette_index = 0u16;
            }
            let ED: *mut EnemyDef = get_EnemyDef_A2((*EP).enemy_ptr, game);
            (*E).x_width = (*ED).x_radius;
            (*E).y_height = (*ED).y_radius;
            (*E).health = (*ED).health;
            (*E).layer = (*ED).layer as u16;
            *(&raw mut (*E).bank as *mut u16) = *(&raw mut (*ED).bank as *mut u16);
            (*E).enemy_ptr = (*EP).enemy_ptr;
            (*E).x_pos = (*EP).x_pos;
            (*E).y_pos = (*EP).y_pos;
            (*E).current_instruction = (*EP).init_param;
            (*E).properties = (*EP).properties;
            (*E).extra_properties = (*EP).extra_properties;
            (*E).parameter_1 = (*EP).parameter1;
            (*E).parameter_2 = (*EP).parameter2;
            (*E).frame_counter = 0u16;
            (*E).timer = 0u16;
            (*E).ai_var_A = 0u16;
            (*E).ai_var_B = 0u16;
            (*E).ai_var_C = 0u16;
            (*E).ai_var_D = 0u16;
            (*E).ai_var_E = 0u16;
            (*E).ai_preinstr = 0u16;
            (*E).instruction_timer = 1u16;
            (*E).frame_counter = 0u16;
            RecordEnemySpawnData(new_enemy_index, game);
            set_ram_word!(game.cur_enemy_index = new_enemy_index);
            if (*ED).ai_init as i32 & 0x8000i32 != 0 {
                CallEnemyAi(
                    (((*ED).bank as i32) << 16i32 | (*ED).ai_init as i32) as u32,
                    game,
                );
            }
            if ((*E).properties as i32 & 0x2000i32) != 0i32 {
                (*E).spritemap_pointer = addr_kSpritemap_Nothing_A0 as u16;
            }
            if varE26 == 0 || {
                varE26 = varE26.wrapping_sub(1);
                varE26 == 0
            } {
                set_ram_word!(game.cur_enemy_index = cur_enemy_index_backup);
                return new_enemy_index;
            }
            new_enemy_index = (new_enemy_index).wrapping_add(64u16);
            varE20 = (varE20).wrapping_add(16u16);
        },
        _ => 0xffffu16,
    }
}
pub unsafe fn DrawOneEnemy(game: &mut Game) {
    let varE34: u16 =
        (2u16).wrapping_mul((*gEnemyData(ram_word!(game, cur_enemy_index) as i32, game)).layer);
    *(RomPtr_RAM(
        (ram_word!(game, enemy_drawing_queue_sizes[(varE34 as i32 >> 1i32)])).wrapping_add(
            crate::rtl::rom_table_word!(game, 0xa0b133, (varE34 as i32 >> 1i32) as isize),
        ),
        game,
    ) as *mut u16) = ram_word!(game, cur_enemy_index);
    update_ram_word!(
        game.enemy_drawing_queue_sizes[(varE34 as i32 >> 1i32)],
        wrapping_add(2u16)
    );
}
pub unsafe fn WriteEnemyOams(game: &mut Game) {
    let mut palette_index: VoidP = 0;
    let E: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    let ES: *mut EnemySpawnData = gEnemySpawnData(ram_word!(game, cur_enemy_index) as i32, game);
    let mut x2: u16 =
        ((*ES).xpos2 as i32 + (*E).x_pos as i32 - ram_word!(game, layer1_x_pos) as i32) as u16;
    let y2: u16 =
        ((*ES).ypos2 as i32 + (*E).y_pos as i32 - ram_word!(game, layer1_y_pos) as i32) as u16;
    if (*E).shake_timer != 0 {
        x2 = (x2 as i32
            + if ((*E).frame_counter as i32 & 2i32) == 0i32 {
                1i32
            } else {
                -1i32
            }) as u16;
        (*E).shake_timer = (*E).shake_timer.wrapping_sub(1);
    }
    let mut x: u16 = x2;
    let mut y: u16 = y2;
    if (*E).flash_timer != 0u16 && (ram_word!(game, random_enemy_counter) as i32 & 2i32) != 0i32 {
        palette_index = 0 as VoidP;
    } else if (*E).frozen_timer != 0u16
        && ((*E).frozen_timer >= 0x5au16 || ((*E).frozen_timer as i32 & 2i32) != 0i32)
    {
        palette_index = 3072 as VoidP;
    } else {
        palette_index = (*E).palette_index;
    }
    let r3: u16 = palette_index;
    let r0: u16 = (*E).vram_tiles_index;
    if ((*E).extra_properties as i32 & 4i32) != 0i32 {
        if (((*E).spritemap_pointer) as i16).wrapping_add((0x8000i32) as i16) < 0i16 {
            Unreachable(game);
        }
        let mut n: i32 = *RomPtrWithBank((*E).bank, (*E).spritemap_pointer, game) as i32;
        let mut v5: u16 = ((*E).spritemap_pointer).wrapping_add(2u16);
        loop {
            let ext: *mut ExtendedSpriteMap = get_ExtendedSpriteMap((*E).bank, v5, game);
            if *(RomPtrWithBank((*E).bank, (*ext).spritemap, game) as *mut u16) == 0xfffeu16 {
                x = (x2).wrapping_add((*ext).xpos);
                y = (y2).wrapping_add((*ext).ypos);
                if ((*E).extra_properties as i32 & 0x8000i32) != 0i32 {
                    ProcessExtendedTilemap((*E).bank, (*ext).spritemap, game);
                }
            } else {
                x = (x2).wrapping_add((*ext).xpos);
                y = (y2).wrapping_add((*ext).ypos);
                if (x as i32 + 128i32 & 0xfe00i32) == 0i32
                    && (y as i32 + 128i32 & 0xfe00i32) == 0i32
                {
                    if *(&raw mut y as *mut u8).offset(1isize) != 0 {
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
            v5 = (v5).wrapping_add(8u16);
            n -= 1;
            if (n == 0) {
                break;
            }
        }
    } else {
        set_ram_word!(game.enemy_processing_stage = 1u16);
        DrawSpritemapWithBaseTile((*E).bank, (*E).spritemap_pointer, x, y, r3, r0, game);
    };
}
pub unsafe fn NormalEnemyFrozenAI(game: &mut Game) {
    let v0: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    (*v0).flash_timer = 0u16;
    if (*v0).frozen_timer == 0 || {
        (*v0).frozen_timer = (*v0).frozen_timer.wrapping_sub(1);
        (ram_word!(game, equipped_beams) as i32 & 2i32) == 0i32
    } {
        let v1: u16 = ((*v0).ai_handler_bits) & (0xfffbu16);
        (*v0).ai_handler_bits = v1;
        (*v0).frozen_timer = v1;
    }
}
pub unsafe fn ProcessExtendedTilemap(db: u8, r22: u16, game: &mut Game) {
    let mut p: *const u8 = RomPtrWithBank(db, (r22).wrapping_add(2u16), game);
    loop {
        let v2: u16 = *(p as *mut u16);
        if v2 == 0xffffu16 {
            break;
        }
        let n: i32 = *(p as *mut u16).offset(1) as i32;
        p = p.offset(4);
        memcpy(
            (&raw mut game.g_ram as *mut u8).offset(v2 as i32 as isize) as *mut c_void,
            p as *const c_void,
            (n * 2i32) as size_t,
        );
        p = p.offset((n * 2i32) as isize);
    }
    update_ram_word!(game.nmi_flag_bg2_enemy_vram_transfer, wrapping_add(1));
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
    if ((*gEnemyData(ram_word!(game, cur_enemy_index) as i32, game)).extra_properties as i32 & 4i32)
        != 0i32
    {
        EprojCollHandler_Multibox(game);
        EnemyBombCollHandler_Multibox(game);
        EnemySamusCollHandler_Multibox(game);
    } else {
        EprojCollHandler(game);
        EnemyBombCollHandler(game);
        EnemySamusCollHandler(game);
    };
}
pub fn func_nullsub_4() {}
pub unsafe fn SamusProjectileInteractionHandler(game: &mut Game) {
    set_ram_word!(game.enemy_processing_stage = 10u16);
    if ram_word!(game, flag_disable_projectile_interaction) != 0 {
        return;
    }
    let mut num_colls_to_check: i32 = 5i32;
    if ram_word!(game, bomb_counter) != 0 {
        num_colls_to_check = 10i32;
    } else if ram_word!(game, projectile_counter) == 0 {
        return;
    }
    if ram_word!(game, projectile_invincibility_timer) != 0u16
        || ram_word!(game, samus_contact_damage_index) != 0u16
    {
        return;
    }
    let mut pidx: i32 = 0i32;
    while pidx != num_colls_to_check {
        set_ram_word!(game.collision_detection_index = pidx as u16);
        if !(ram_word!(game, projectile_damage[pidx]) == 0
            || ram_word!(game, projectile_type[pidx]) as i32 & 0x8000i32 != 0
            || (ram_word!(game, projectile_type[pidx]) as i32 & 0xf00i32) - 1792i32 & 0x8000i32
                == 0)
            && ((ram_word!(game, projectile_dir[pidx]) as i32 & 0x10i32) == 0i32)
            && (abs16(
                (ram_word!(game, projectile_x_pos[pidx]))
                    .wrapping_sub(ram_word!(game, samus_x_pos)),
            ) as i32
                - ram_word!(game, projectile_x_radius[pidx]) as i32)
                < ram_word!(game, samus_x_radius) as i32
            && (abs16(
                (ram_word!(game, projectile_y_pos[pidx]))
                    .wrapping_sub(ram_word!(game, samus_y_pos)),
            ) as i32
                - ram_word!(game, projectile_y_radius[pidx]) as i32)
                < ram_word!(game, samus_y_radius) as i32
        {
            if (ram_word!(game, projectile_type[pidx]) as i32 & 0xff00i32) != 768i32
                && (ram_word!(game, projectile_type[pidx]) as i32 & 0xff00i32) != 1280i32
            {
                update_ram_word!(game.projectile_dir[pidx], |fresh43| (fresh43) | (0x10u16));
                Samus_DealDamage(
                    SuitDamageDivision(ram_word!(game, projectile_damage[pidx]), game),
                    game,
                );
                set_ram_word!(game.samus_invincibility_timer = 96u16);
                set_ram_word!(game.samus_knockback_timer = 5u16);
                '_c2rust_label: {
                    __assert_fail(
                        b"0\0" as *const u8 as *const c_char,
                        b"src/sm_a0.c\0" as *const u8 as *const c_char,
                        2048u32,
                        b"void SamusProjectileInteractionHandler(void)\0" as *const u8
                            as *const c_char,
                    );
                };
                let mut v0: u16 = 0u16;
                set_ram_word!(
                    game.knockback_x_dir = (((ram_word!(game, samus_x_pos)) as i16).wrapping_sub(
                        (*((&raw mut game.g_ram as *mut u8).offset(0x1a4b) as *mut u16)
                            .offset((v0 as i32 >> 1i32) as isize,)) as i16,
                    ) >= 0i16) as u16
                );
                return;
            }
            if ram_word!(game, projectile_variables[pidx]) == 8u16 {
                set_ram_word!(
                    game.bomb_jump_dir = (if ram_word!(game, samus_x_pos)
                        == ram_word!(game, projectile_x_pos[pidx])
                    {
                        2i32
                    } else if ((ram_word!(game, samus_x_pos)) as i16)
                        .wrapping_sub((ram_word!(game, projectile_x_pos[pidx])) as i16,)
                        < 0i16
                    {
                        1i32
                    } else {
                        3i32
                    }) as u16
                );
            }
        }
        pidx += 1;
    }
}
pub unsafe fn EprojSamusCollDetect(game: &mut Game) {
    set_ram_word!(game.enemy_processing_stage = 11u16);
    if ram_word!(game, samus_invincibility_timer) != 0u16
        || ram_word!(game, samus_contact_damage_index) != 0u16
    {
        return;
    }
    let mut i: i32 = 17i32;
    while i >= 0i32 {
        if ram_word!(game, eproj_id[i]) != 0u16
            && (ram_word!(game, eproj_properties[i]) as i32 & 0x2000i32) == 0i32
            && ram_word!(game, eproj_radius[i]) != 0u16
        {
            let varE20: u16 = *(((&raw mut game.g_ram as *mut u8).offset(0x1bb3) as *mut u16)
                .offset(i as isize) as *mut u16 as *mut u8)
                .offset(0) as u16;
            let varE22: u16 = *(((&raw mut game.g_ram as *mut u8).offset(0x1bb3) as *mut u16)
                .offset(i as isize) as *mut u16 as *mut u8)
                .offset(1isize) as u16;
            if (abs16((ram_word!(game, samus_x_pos)).wrapping_sub(ram_word!(game, eproj_x_pos[i])))
                as i32
                - ram_word!(game, samus_x_radius) as i32)
                < varE20 as i32
                && (abs16(
                    (ram_word!(game, samus_y_pos)).wrapping_sub(ram_word!(game, eproj_y_pos[i])),
                ) as i32
                    - ram_word!(game, samus_y_radius) as i32)
                    < varE22 as i32
            {
                set_ram_word!(game.collision_detection_index = (i * 2i32) as u16);
                HandleEprojCollWithSamus((i * 2i32) as u16, game);
            }
        }
        i -= 1;
    }
}
pub unsafe fn HandleEprojCollWithSamus(k: u16, game: &mut Game) {
    set_ram_word!(game.samus_invincibility_timer = 96u16);
    set_ram_word!(game.samus_knockback_timer = 5u16);
    let v1: u16 = *(RomPtr_86(
        *(((&raw mut game.g_ram as *mut u8).offset(0x1997) as *mut u16 as *mut u8)
            .offset(k as i32 as isize) as *mut u16),
        game,
    ) as *mut u16)
        .offset(5);
    if v1 != 0 {
        let v2: i32 = k as i32 >> 1i32;
        set_ram_word!(game.eproj_instr_list_ptr[v2] = v1);
        set_ram_word!(game.eproj_instr_timers[v2] = 1u16);
    }
    let v3: i32 = k as i32 >> 1i32;
    if (ram_word!(game, eproj_properties[v3]) as i32 & 0x4000i32) == 0i32 {
        *(((&raw mut game.g_ram as *mut u8).offset(0x1997) as *mut u16 as *mut u8)
            .offset(k as i32 as isize) as *mut u16) = 0u16;
    }
    Samus_DealDamage(
        SuitDamageDivision((ram_word!(game, eproj_properties[v3])) & (0xfffu16), game),
        game,
    );
    set_ram_word!(
        game.knockback_x_dir = (((ram_word!(game, samus_x_pos)) as i16)
            .wrapping_sub((ram_word!(game, eproj_x_pos[v3])) as i16,)
            >= 0i16) as u16
    );
}
pub unsafe fn EprojProjCollDet(game: &mut Game) {
    set_ram_word!(game.enemy_processing_stage = 12u16);
    if ram_word!(game, projectile_counter) == 0 {
        return;
    }
    let mut i: i32 = 17i32;
    while i >= 0i32 {
        if !(ram_word!(game, eproj_id[i]) == 0
            || (ram_word!(game, eproj_properties[i]) as i32 & 0x8000i32) == 0i32)
        {
            let mut j: i32 = 0i32;
            while j < 5i32 {
                if ram_word!(game, eproj_flags[i]) == 2u16 {
                    break;
                }
                let v4: u16 = ram_word!(game, projectile_type[j]);
                if v4 != 0u16
                    && (v4 as i32 & 0xf00i32) != 768i32
                    && (v4 as i32 & 0xf00i32) != 1280i32
                    && (v4 as i32 & 0xf00i32) - 1792i32 & 0x8000i32 != 0
                    && ram_word!(game, eproj_x_pos[i]) as i32 & 0xffe0i32
                        == ram_word!(game, projectile_x_pos[j]) as i32 & 0xffe0i32
                    && ram_word!(game, eproj_y_pos[i]) as i32 & 0xffe0i32
                        == ram_word!(game, projectile_y_pos[j]) as i32 & 0xffe0i32
                {
                    HandleEprojCollWithProj((i * 2i32) as u16, (j * 2i32) as u16, game);
                }
                j += 1;
            }
        }
        i -= 1;
    }
}
pub unsafe fn HandleEprojCollWithProj(k: u16, j: u16, game: &mut Game) {
    let i: i32 = j as i32 >> 1i32;
    if (ram_word!(game, projectile_type[i]) as i32 & 8i32) == 0i32 {
        update_ram_word!(game.projectile_dir[i], |fresh30| (fresh30) | (0x10u16));
    }
    if ram_word!(game, eproj_flags[(k as i32 >> 1i32)]) == 1u16 {
        let v4: i32 = k as i32 >> 1i32;
        CreateSpriteAtPos(
            ram_word!(game, projectile_x_pos[v4]),
            ram_word!(game, projectile_y_pos[v4]),
            6u16,
            0u16,
            game,
        );
        QueueSfx1_Max6(0x3du16, game);
    } else {
        let j_0: i32 = k as i32 >> 1i32;
        set_ram_word!(game.eproj_G[j_0] = ram_word!(game, projectile_type[i]));
        set_ram_word!(
            game.eproj_instr_list_ptr[j_0] =
                (*get_EprojDef(ram_word!(game, eproj_id[j_0]), game,)).shot_instruction_list
        );
        set_ram_word!(game.eproj_instr_timers[j_0] = 1u16);
        set_ram_word!(game.eproj_pre_instr[j_0] = ((0x8684fbi32) as u16));
        update_ram_word!(game.eproj_properties[j_0], |fresh31| (fresh31) & (0xfffu16));
    };
}
pub unsafe fn CallHitboxTouch(ea: u32, game: &mut Game) {
    match ea {
        10518563 => Enemy_NormalTouchAI_A0(game),
        10670984 => MaridiaLargeSnail_Func_12(game),
        10670988 => MaridiaLargeSnail_Touch(game),
        10780707 => Enemy_NormalTouchAI_A4(game),
        10795325 => Crocomire_Func_92(game),
        10795344 => (),
        10846284 => (),
        10851818 => Draygon_Touch(game),
        10874348 => SporeSpawn_Touch(game),
        10936153 => sub_A6DF59(game),
        10940479 => CeresSteam_Touch(game),
        10977356 => (),
        10982539 => Kraid_Touch_ArmFoot(game),
        10982544 => KraidsArm_Touch(game),
        11001237 => Phantoon_Touch(game),
        11122117 => (),
        11122118 => MotherBrainsBrain_Touch(game),
        11192695 => GoldTorizo_Touch(game),
        11700076 => WalkingSpacePirates_Touch(game),
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn CallHitboxShot(ea: u32, j: u16, game: &mut Game) {
    match ea {
        10518573 => Enemy_NormalShotAI_A0(game),
        10649676 => (),
        10671028 => MaridiaLargeSnail_Shot(game),
        10780717 => Enemy_NormalShotAI_A4(game),
        10795345 => Crocomire_Func_93(game),
        10795368 => Crocomire_Func_94(game),
        10795525 => Crocomire_Func_95(game),
        10795700 => Crocomire_Func_1(game),
        10846253 => Enemy_NormalShotAI_A5(game),
        10846278 => CreateADudShot(game),
        10846284 => (),
        10851824 => Draygon_Shot(game),
        10874202 => SporeSpawn_Shot(game),
        10911820 => (),
        10936202 => Ridley_Shot(game),
        10977356 => (),
        10982581 => (),
        10982582 => Kraid_Arm_Shot(j, game),
        11001243 => Phantoon_Shot(game),
        11121923 => MotherBrainsBody_Shot(game),
        11121927 => MotherBrainsBrain_Shot(game),
        11192700 => Torizo_Shot(game),
        11192769 => (),
        11192770 => Torizo_Func_8(game),
        11700089 => WalkingSpacePirates_Shot(game),
        11700168 => WalkingSpacePirates_87C8(game),
        11700286 => WalkingSpacePirates_883E(game),
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn EnemySamusCollHandler_Multibox(game: &mut Game) {
    let E: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    set_ram_word!(game.enemy_processing_stage = 6u16);
    if (*E).spritemap_pointer == 0 {
        return;
    }
    let touch_ai: u16 = (*get_EnemyDef_A2((*E).enemy_ptr, game)).touch_ai;
    if touch_ai as i32 == 0xa0804ci32 & 0xffffi32 || touch_ai as i32 == 0xa0804bi32 & 0xffffi32 {
        return;
    }
    if ram_word!(game, samus_contact_damage_index) != 0 {
        set_ram_word!(game.samus_invincibility_timer = 0u16);
    } else if ram_word!(game, samus_invincibility_timer) != 0 {
        return;
    }
    if (*E).spritemap_pointer as i32 & 0x8000i32 == 0 {
        return;
    }
    let samus_right_border_coll: u16 =
        (ram_word!(game, samus_x_radius)).wrapping_add(ram_word!(game, samus_x_pos));
    let samus_left_border_coll: u16 =
        (ram_word!(game, samus_x_pos)).wrapping_sub(ram_word!(game, samus_x_radius));
    let samus_bottom_border_coll: u16 =
        (ram_word!(game, samus_y_radius)).wrapping_add(ram_word!(game, samus_y_pos));
    let samus_top_border_coll: u16 =
        (ram_word!(game, samus_y_pos)).wrapping_sub(ram_word!(game, samus_y_radius));
    let mut n: i32 = *RomPtrWithBank((*E).bank, (*E).spritemap_pointer, game) as i32;
    let mut enemy_spritemap_entry_pointer: u16 = ((*E).spritemap_pointer).wrapping_add(2u16);
    loop {
        let ES: *mut ExtendedSpriteMap =
            get_ExtendedSpriteMap((*E).bank, enemy_spritemap_entry_pointer, game);
        let coll_x_pos: u16 = ((*ES).xpos).wrapping_add((*E).x_pos);
        let coll_y_pos: u16 = ((*ES).ypos).wrapping_add((*E).y_pos);
        let p: *const u8 = RomPtrWithBank((*E).bank, (*ES).hitbox_ptr_, game);
        let mut m: i32 = *(p as *mut u16) as i32;
        let mut hb: *mut Hitbox = p.offset(2) as *mut Hitbox;
        while m != 0 {
            if (((*hb).left as i32 + coll_x_pos as i32 - samus_right_border_coll as i32) as i16)
                < 0i16
                && (((*hb).right as i32 + coll_x_pos as i32 - samus_left_border_coll as i32) as i16)
                    >= 0i16
                && (((*hb).top as i32 + coll_y_pos as i32 - samus_bottom_border_coll as i32) as i16)
                    < 0i16
                && (((*hb).bottom as i32 + coll_y_pos as i32 - samus_top_border_coll as i32) as i16)
                    >= 0i16
            {
                CallHitboxTouch(
                    (((*E).bank as i32) << 16i32 | (*hb).func_ptr as i32) as u32,
                    game,
                );
                return;
            }
            m -= 1;
            hb = hb.offset(1);
        }
        enemy_spritemap_entry_pointer = (enemy_spritemap_entry_pointer).wrapping_add(8u16);
        n -= 1;
        if (n == 0) {
            break;
        }
    }
}
pub unsafe fn EprojCollHandler_Multibox(game: &mut Game) {
    let E: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    set_ram_word!(game.enemy_processing_stage = 3u16);
    if ram_word!(game, projectile_counter) == 0
        || (*E).spritemap_pointer == 0
        || (*E).spritemap_pointer as i32 == addr_kExtendedSpritemap_Nothing_A0 as i32
    {
        return;
    }
    let shot_ai: u16 = (*get_EnemyDef_A2((*E).enemy_ptr, game)).shot_ai;
    if shot_ai as i32 == 0xa0804ci32 & 0xffffi32 || shot_ai as i32 == 0xa0804bi32 & 0xffffi32 {
        return;
    }
    if ((*E).properties as i32 & 0x400i32) != 0i32
        || (*E).invincibility_timer != 0u16
        || (*E).enemy_ptr as i32 == addr_kEnemyDef_DAFF as i32
    {
        return;
    }
    let mut pidx: i32 = 0i32;
    while pidx < 5i32 {
        let v4: u16 = ram_word!(game, projectile_type[pidx]);
        if v4 != 0u16
            && (v4 as i32 & 0xf00i32) != 768i32
            && (v4 as i32 & 0xf00i32) != 1280i32
            && (v4 as i32 & 0xf00i32) - 1792i32 & 0x8000i32 != 0
        {
            if (*E).spritemap_pointer as i32 & 0x8000i32 == 0 {
                Unreachable(game);
            }
            let esep: *const u8 = RomPtrWithBank((*E).bank, (*E).spritemap_pointer, game);
            let mut n: i32 = *esep.offset(0) as i32;
            let mut ES: *mut ExtendedSpriteMap = esep.offset(2) as *mut ExtendedSpriteMap;
            while n != 0 {
                let coll_x_pos: u16 = ((*ES).xpos).wrapping_add((*E).x_pos);
                let coll_y_pos: u16 = ((*ES).ypos).wrapping_add((*E).y_pos);
                let p: *const u8 = RomPtrWithBank((*E).bank, (*ES).hitbox_ptr_, game);
                let mut m: i32 = *(p as *mut u16) as i32;
                let mut hb: *mut Hitbox = p.offset(2) as *mut Hitbox;
                while m != 0 {
                    if ((ram_word!(game, projectile_x_radius[pidx]) as i32
                        + ram_word!(game, projectile_x_pos[pidx]) as i32
                        - (coll_x_pos as i32 + (*hb).left as i32)) as i16)
                        >= 0i16
                        && ((ram_word!(game, projectile_x_pos[pidx]) as i32
                            - ram_word!(game, projectile_x_radius[pidx]) as i32
                            - (coll_x_pos as i32 + (*hb).right as i32))
                            as i16)
                            < 0i16
                        && ((ram_word!(game, projectile_y_radius[pidx]) as i32
                            + ram_word!(game, projectile_y_pos[pidx]) as i32
                            - (coll_y_pos as i32 + (*hb).top as i32))
                            as i16)
                            >= 0i16
                        && ((ram_word!(game, projectile_y_pos[pidx]) as i32
                            - ram_word!(game, projectile_y_radius[pidx]) as i32
                            - (coll_y_pos as i32 + (*hb).bottom as i32))
                            as i16)
                            < 0i16
                    {
                        if (ram_word!(game, projectile_type[pidx]) as i32 & 0xf00i32) == 512i32 {
                            set_ram_word!(game.earthquake_timer = 30u16);
                            set_ram_word!(game.earthquake_type = 18u16);
                        }
                        if ((*E).properties as i32 & 0x1000i32) != 0i32
                            || (ram_word!(game, projectile_type[pidx]) as i32 & 8i32) == 0i32
                        {
                            update_ram_word!(game.projectile_dir[pidx], |fresh27| (fresh27)
                                | (0x10u16));
                        }
                        set_ram_word!(game.collision_detection_index = pidx as u16);
                        CallHitboxShot(
                            (((*E).bank as i32) << 16i32 | (*hb).func_ptrA as i32) as u32,
                            (pidx * 2i32) as u16,
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
        pidx += 1;
    }
}
pub unsafe fn EnemyBombCollHandler_Multibox(game: &mut Game) {
    let E: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    set_ram_word!(game.enemy_processing_stage = 4u16);
    if (*E).spritemap_pointer == 0
        || ((*E).properties as i32 & 0x400i32) != 0i32
        || (*E).invincibility_timer != 0u16
    {
        return;
    }
    let shot_ai: u16 = (*get_EnemyDef_A2((*E).enemy_ptr, game)).shot_ai;
    if shot_ai as i32 == 0xa0804ci32 & 0xffffi32
        || shot_ai as i32 == 0xa0804bi32 & 0xffffi32
        || ram_word!(game, bomb_counter) == 0
    {
        return;
    }
    let mut pidx: i32 = 5i32;
    while pidx != 10i32 {
        if (ram_word!(game, projectile_x_pos[pidx]) != 0) {
            let v4: u16 = ram_word!(game, projectile_type[pidx]);
            if v4 != 0u16
                && (v4 as i32 & 0xf00i32) == 1280i32
                && ram_word!(game, projectile_variables[pidx]) == 0
            {
                if (*E).spritemap_pointer as i32 & 0x8000i32 == 0 {
                    Unreachable(game);
                }
                let esep: *const u8 = RomPtrWithBank((*E).bank, (*E).spritemap_pointer, game);
                let mut n: i32 = *(esep as *mut u16) as i32;
                let mut ES: *mut ExtendedSpriteMap = esep.offset(2) as *mut ExtendedSpriteMap;
                while n != 0 {
                    let coll_x_pos: u16 = ((*ES).xpos).wrapping_add((*E).x_pos);
                    let coll_y_pos: u16 = ((*ES).ypos).wrapping_add((*E).y_pos);
                    let p: *const u8 = RomPtrWithBank((*E).bank, (*ES).hitbox_ptr_, game);
                    let mut m: i32 = *(p as *mut u16) as i32;
                    let mut hb: *mut Hitbox = p.offset(2) as *mut Hitbox;
                    while m != 0 {
                        if ((ram_word!(game, projectile_x_radius[pidx]) as i32
                            + ram_word!(game, projectile_x_pos[pidx]) as i32
                            - (coll_x_pos as i32 + (*hb).left as i32))
                            as i16)
                            >= 0i16
                            && ((ram_word!(game, projectile_x_pos[pidx]) as i32
                                - ram_word!(game, projectile_x_radius[pidx]) as i32
                                - (coll_x_pos as i32 + (*hb).right as i32))
                                as i16)
                                < 0i16
                            && ((ram_word!(game, projectile_y_radius[pidx]) as i32
                                + ram_word!(game, projectile_y_pos[pidx]) as i32
                                - (coll_y_pos as i32 + (*hb).top as i32))
                                as i16)
                                >= 0i16
                            && ((ram_word!(game, projectile_y_pos[pidx]) as i32
                                - ram_word!(game, projectile_y_radius[pidx]) as i32
                                - (coll_y_pos as i32 + (*hb).bottom as i32))
                                as i16)
                                < 0i16
                        {
                            update_ram_word!(game.projectile_dir[pidx], |fresh24| (fresh24)
                                | (0x10u16));
                            set_ram_word!(game.collision_detection_index = pidx as u16);
                            CallHitboxShot(
                                (((*E).bank as i32) << 16i32 | (*hb).func_ptrA as i32) as u32,
                                (pidx * 2i32) as u16,
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
        pidx += 1;
    }
}
pub unsafe fn GrappleBeam_CollDetect_Enemy(game: &mut Game) -> PairU16 {
    let mut grapple_ai: VoidP = 0;
    let mut E: *mut EnemyData = null_mut::<EnemyData>();
    CallSomeSamusCode(0xdu16, game);
    set_ram_word!(game.collision_detection_index = 0u16);
    let mut i: i32 = 0i32;
    loop {
        set_ram_word!(game.cur_enemy_index = ram_word!(game, interactive_enemy_indexes[i]));
        if ram_word!(game, cur_enemy_index) == 0xffffu16 {
            return PairU16 { k: 0u16, j: 0u16 };
        }
        E = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
        if (*E).invincibility_timer == 0 {
            let v5: u16 = abs16(((*E).x_pos).wrapping_sub(ram_word!(game, grapple_beam_end_x_pos)));
            let mut v6: bool = v5 < (*E).x_width;
            let v7: u16 = (v5).wrapping_sub((*E).x_width);
            if v6 || v7 < 8u16 {
                let v8: u16 =
                    abs16(((*E).y_pos).wrapping_sub(ram_word!(game, grapple_beam_end_y_pos)));
                v6 = v8 < (*E).y_height;
                let v9: u16 = (v8).wrapping_sub((*E).y_height);
                if v6 || v9 < 8u16 {
                    break;
                }
            }
        }
        i += 1;
    }
    (*E).ai_handler_bits = 1u16;
    let mut v0: u16 = 0u16;
    let enemy_ptr: u16 = (*E).enemy_ptr;
    grapple_ai = (*get_EnemyDef_A2((*E).enemy_ptr, game)).grapple_ai;
    if grapple_ai as i32 != 0xa08000i32 & 0xffffi32 {
        v0 = 1u16;
        if grapple_ai as i32 != 0xa08005i32 & 0xffffi32 {
            v0 = 2u16;
            if grapple_ai as i32 != 0xa0800ai32 & 0xffffi32 {
                v0 = 3u16;
                if grapple_ai as i32 != 0xa0800fi32 & 0xffffi32 {
                    v0 = 4u16;
                    if grapple_ai as i32 != 0xa08014i32 & 0xffffi32 {
                        v0 = 5u16;
                        if grapple_ai as i32 != 0xa08019i32 & 0xffffi32 {
                            v0 = 6u16;
                            if grapple_ai as i32 != 0xa0801ei32 & 0xffffi32 {
                                v0 = 0u16;
                            }
                        }
                    }
                }
            }
        }
    }
    if v0 == 1u16 || v0 == 4u16 || v0 == 5u16 {
        let v11: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
        set_ram_word!(game.grapple_beam_end_x_pos = (*v11).x_pos);
        set_ram_word!(game.grapple_beam_end_y_pos = (*v11).y_pos);
    }
    PairU16 {
        k: v0,
        j: enemy_ptr,
    }
}
pub unsafe fn SwitchEnemyAiToMainAi(game: &mut Game) {
    let E: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).ai_handler_bits = 0u16;
    (*E).invincibility_timer = 0u16;
    (*E).frozen_timer = 0u16;
    (*E).shake_timer = 0u16;
}
pub unsafe fn SamusLatchesOnWithGrapple(game: &mut Game) {
    let E: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    set_ram_word!(game.grapple_beam_end_x_pos = (*E).x_pos);
    set_ram_word!(game.grapple_beam_end_y_pos = (*E).y_pos);
    if (*E).frozen_timer != 0 {
        (*E).ai_handler_bits = 4u16;
    } else {
        let ED: *mut EnemyDef = get_EnemyDef_A2((*E).enemy_ptr, game);
        (*E).flash_timer = (if (*ED).hurt_ai_time != 0u8 {
            (*ED).hurt_ai_time as i32
        } else {
            4i32
        }) as u16;
        (*E).ai_handler_bits = 0u16;
    };
}
pub unsafe fn EnemyGrappleDeath(game: &mut Game) {
    (*gEnemySpawnData(ram_word!(game, cur_enemy_index) as i32, game)).cause_of_death = 4u16;
    EnemyDeathAnimation(ram_word!(game, cur_enemy_index), 0u16, game);
    (*gEnemyData(ram_word!(game, cur_enemy_index) as i32, game)).ai_handler_bits = 0u16;
}
pub unsafe fn Enemy_SwitchToFrozenAi(game: &mut Game) {
    (*gEnemyData(ram_word!(game, cur_enemy_index) as i32, game)).ai_handler_bits = 4u16;
}
pub unsafe fn SamusLatchesOnWithGrappleNoInvinc(game: &mut Game) {
    let E: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).frozen_timer != 0 {
        (*E).ai_handler_bits = 4u16;
    } else {
        CallEnemyAi(
            (((*E).bank as i32) << 16i32 | (*get_EnemyDef_A2((*E).enemy_ptr, game)).main_ai as i32)
                as u32,
            game,
        );
        (*E).ai_handler_bits = 0u16;
    }
    set_ram_word!(game.grapple_beam_end_x_pos = (*E).x_pos);
    set_ram_word!(game.grapple_beam_end_y_pos = (*E).y_pos);
}
pub unsafe fn SamusLatchesOnWithGrappleParalyze(game: &mut Game) {
    let E: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    let ED: *mut EnemyDef = get_EnemyDef_A2((*E).enemy_ptr, game);
    (*E).flash_timer = (if (*ED).hurt_ai_time != 0u8 {
        (*ED).hurt_ai_time as i32
    } else {
        4i32
    }) as u16;
    (*E).ai_handler_bits = 0u16;
    (*E).extra_properties = ((*E).extra_properties) | (1u16);
}
pub unsafe fn SamusHurtFromGrapple(game: &mut Game) {
    (*gEnemyData(ram_word!(game, cur_enemy_index) as i32, game)).ai_handler_bits = 4u16;
}
pub unsafe fn EnemySamusCollHandler(game: &mut Game) {
    let E: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    set_ram_word!(game.enemy_processing_stage = 9u16);
    if (*E).spritemap_pointer == 0 {
        return;
    }
    if ram_word!(game, samus_contact_damage_index) != 0 {
        set_ram_word!(game.samus_invincibility_timer = 0u16);
    } else if ram_word!(game, samus_invincibility_timer) != 0 {
        if (*E).enemy_ptr as i32 != addr_kEnemyDef_DAFF as i32 {
            return;
        }
        let some_flag: u16 =
            (*gEnemySpawnData(ram_word!(game, cur_enemy_index) as i32, game)).some_flag;
        if some_flag == 0u16 || some_flag == 8u16 {
            return;
        }
    }
    let ED: *mut EnemyDef = get_EnemyDef_A2((*E).enemy_ptr, game);
    if (*ED).touch_ai as i32 == 0xa0804ci32 & 0xffffi32
        || (*ED).touch_ai as i32 == 0xa0804bi32 & 0xffffi32
    {
        return;
    }
    if (abs16((ram_word!(game, samus_x_pos)).wrapping_sub((*E).x_pos)) as i32
        - ram_word!(game, samus_x_radius) as i32)
        < (*E).x_width as i32
        && (abs16((ram_word!(game, samus_y_pos)).wrapping_sub((*E).y_pos)) as i32
            - ram_word!(game, samus_y_radius) as i32)
            < (*E).y_height as i32
        && ((*E).enemy_ptr as i32 == addr_kEnemyDef_DAFF as i32 || (*E).frozen_timer == 0)
    {
        CallEnemyAi(
            (((*E).bank as i32) << 16i32 | (*ED).touch_ai as i32) as u32,
            game,
        );
    }
}
pub unsafe fn EprojCollHandler(game: &mut Game) {
    let E: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    set_ram_word!(game.enemy_processing_stage = 7u16);
    if ram_word!(game, projectile_counter) == 0 {
        return;
    }
    if (*E).spritemap_pointer == 0
        || (*E).spritemap_pointer as i32 == addr_kSpritemap_Nothing_A0 as i32
        || ((*E).properties as i32 & 0x400i32) != 0i32
        || (*E).enemy_ptr as i32 == addr_kEnemyDef_DAFF as i32
        || (*E).invincibility_timer != 0u16
    {
        return;
    }
    let mut pidx: i32 = 0i32;
    while pidx < 5i32 {
        let j: u16 = ram_word!(game, projectile_type[pidx]);
        if j != 0u16
            && (j as i32 & 0xf00i32) != 768i32
            && (j as i32 & 0xf00i32) != 1280i32
            && (j as i32 & 0xf00i32) - 1792i32 & 0x8000i32 != 0
        {
            let x: u16 = abs16((ram_word!(game, projectile_x_pos[pidx])).wrapping_sub((*E).x_pos));
            let y: u16 = abs16((ram_word!(game, projectile_y_pos[pidx])).wrapping_sub((*E).y_pos));
            if (x as i32 - ram_word!(game, projectile_x_radius[pidx]) as i32) < (*E).x_width as i32
                && (y as i32 - ram_word!(game, projectile_y_radius[pidx]) as i32)
                    < (*E).y_height as i32
            {
                if (ram_word!(game, projectile_type[pidx]) as i32 & 0xf00i32) == 512i32 {
                    set_ram_word!(game.earthquake_timer = 30u16);
                    set_ram_word!(game.earthquake_type = 18u16);
                }
                if ((*E).properties as i32 & 0x1000i32) != 0i32
                    || (ram_word!(game, projectile_type[pidx]) as i32 & 8i32) == 0i32
                {
                    update_ram_word!(game.projectile_dir[pidx], |fresh26| (fresh26) | (0x10u16));
                }
                set_ram_word!(game.collision_detection_index = pidx as u16);
                CallEnemyAi(
                    (((*E).bank as i32) << 16i32
                        | (*get_EnemyDef_A2((*E).enemy_ptr, game)).shot_ai as i32)
                        as u32,
                    game,
                );
                return;
            }
        }
        pidx += 1;
    }
}
pub unsafe fn EnemyBombCollHandler(game: &mut Game) {
    let E: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    set_ram_word!(game.enemy_processing_stage = 8u16);
    if ram_word!(game, bomb_counter) == 0
        || (*E).spritemap_pointer == 0
        || (*E).invincibility_timer != 0u16
        || (*E).enemy_ptr as i32 == addr_kEnemyDef_DAFF as i32
    {
        return;
    }
    let mut pidx: i32 = 5i32;
    while pidx < 10i32 {
        if !(ram_word!(game, projectile_type[pidx]) == 0
            || ram_word!(game, projectile_variables[pidx]) != 0u16
            || (ram_word!(game, projectile_type[pidx]) as i32 & 0xf00i32) != 1280i32
                && (ram_word!(game, projectile_type[pidx]) as i32 & 0x8000i32) == 0i32)
            && (abs16((ram_word!(game, projectile_x_pos[pidx])).wrapping_sub((*E).x_pos)) as i32
                - ram_word!(game, projectile_x_radius[pidx]) as i32)
                < (*E).x_width as i32
            && (abs16((ram_word!(game, projectile_y_pos[pidx])).wrapping_sub((*E).y_pos)) as i32
                - ram_word!(game, projectile_y_radius[pidx]) as i32)
                < (*E).y_height as i32
        {
            set_ram_word!(game.collision_detection_index = pidx as u16);
            update_ram_word!(game.projectile_dir[pidx], |fresh23| (fresh23) | (0x10u16));
            CallEnemyAi(
                (((*E).bank as i32) << 16i32
                    | (*get_EnemyDef_A2((*E).enemy_ptr, game)).shot_ai as i32)
                    as u32,
                game,
            );
            return;
        }
        pidx += 1;
    }
}
pub unsafe fn ProcessEnemyPowerBombInteraction(game: &mut Game) {
    set_ram_word!(game.enemy_processing_stage = 5u16);
    let rx: u16 = *((&raw mut game.g_ram as *mut u8).offset(0xcea) as *mut u16 as *mut u8)
        .offset(1isize) as u16;
    let ry: u16 = (rx as i32 + (rx as i32 >> 1i32) >> 1i32) as u16;
    let mut i: i32 = 0x7c0i32;
    while rx != 0u16 && i >= 0i32 {
        let E: *mut EnemyData = gEnemyData(i, game);
        if !((*E).invincibility_timer != 0u16
            || (*E).enemy_ptr == 0
            || (*E).enemy_ptr as i32 == addr_kEnemyDef_DAFF as i32)
        {
            let ED: *mut EnemyDef = get_EnemyDef_A2((*E).enemy_ptr, game);
            if (((*get_Vulnerability(
                (if (*ED).vulnerability_ptr != 0u16 {
                    (*ED).vulnerability_ptr as i32
                } else {
                    addr_stru_B4EC1C as i32
                }) as u16,
                game,
            ))
            .power_bomb as i32
                & 0x7fi32)
                != 0i32)
                && abs16((ram_word!(game, power_bomb_explosion_x_pos)).wrapping_sub((*E).x_pos))
                    < rx
                && abs16((ram_word!(game, power_bomb_explosion_y_pos)).wrapping_sub((*E).y_pos))
                    < ry
            {
                set_ram_word!(game.cur_enemy_index = i as u16);
                let func: u16 = (if (*ED).powerbomb_reaction != 0u16 {
                    (*ED).powerbomb_reaction as i32
                } else {
                    0xa08037i32 & 0xffffi32
                }) as u16;
                CallEnemyAi((((*E).bank as i32) << 16i32 | func as i32) as u32, game);
                (*E).properties = ((*E).properties) | (0x800u16);
            }
        }
        i -= 0x40i32;
    }
}
pub unsafe fn EnemyDeathAnimation(k: u16, mut a: u16, game: &mut Game) {
    let E: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).ai_handler_bits == 1u16 {
        set_ram_word!(game.grapple_beam_function = ((0x9bc8c5i32) as u16));
    }
    if a as i32 - 5i32 & 0x8000i32 == 0 {
        a = 0u16;
    }
    SpawnEprojWithGfx(
        a,
        ram_word!(game, cur_enemy_index),
        addr_kEproj_EnemyDeathExplosion as u16,
        game,
    );
    let r18: u16 = ((*E).properties) & (0x4000u16);
    memset(E as *mut c_void, 0i32, 64 as size_t);
    if r18 != 0 {
        (*E).enemy_ptr = addr_kEnemyDef_DAFF as u16;
        (*E).bank = 0xa3u8;
    }
    update_ram_word!(game.num_enemies_killed_in_room, wrapping_add(1));
}
pub unsafe fn RinkasDeathAnimation(mut a: u16, game: &mut Game) {
    let E: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    if a as i32 - 3i32 & 0x8000i32 == 0 {
        a = 0u16;
    }
    SpawnEprojWithGfx(
        a,
        ram_word!(game, cur_enemy_index),
        addr_kEproj_EnemyDeathExplosion as u16,
        game,
    );
    let r18: u16 = ((*E).properties) & (0x4000u16);
    memset(E as *mut c_void, 0i32, 64 as size_t);
    if r18 != 0 {
        (*E).enemy_ptr = addr_kEnemyDef_DAFF as u16;
        (*E).bank = 0xa3u8;
    }
}
pub unsafe fn SuitDamageDivision(a: u16, game: &mut Game) -> u16 {
    if (ram_word!(game, equipped_items) as i32 & 0x20i32) != 0i32 {
        return (a as i32 >> 2i32) as u16;
    }
    if ram_word!(game, equipped_items) as i32 & 1i32 != 0 {
        return (a as i32 >> 1i32) as u16;
    }
    a
}
pub unsafe fn NormalEnemyTouchAi(game: &mut Game) {
    NormalEnemyTouchAiSkipDeathAnim(game);
    if (*gEnemyData(ram_word!(game, cur_enemy_index) as i32, game)).health == 0 {
        (*gEnemySpawnData(ram_word!(game, cur_enemy_index) as i32, game)).cause_of_death = 6u16;
        EnemyDeathAnimation(ram_word!(game, cur_enemy_index), 1u16, game);
    }
}
pub unsafe fn NormalEnemyTouchAiSkipDeathAnim_CurEnemy(game: &mut Game) {
    NormalEnemyTouchAiSkipDeathAnim(game);
}
pub unsafe fn NormalEnemyTouchAiSkipDeathAnim(game: &mut Game) {
    let E: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    let ED: *mut EnemyDef = get_EnemyDef_A2((*E).enemy_ptr, game);
    if ram_word!(game, samus_contact_damage_index) == 0u16 {
        Samus_DealDamage(SuitDamageDivision((*ED).damage, game), game);
        set_ram_word!(game.samus_invincibility_timer = 96u16);
        set_ram_word!(game.samus_knockback_timer = 5u16);
        set_ram_word!(
            game.knockback_x_dir = (((ram_word!(game, samus_x_pos)) as i16)
                .wrapping_sub(((*E).x_pos) as i16)
                >= 0i16) as u16
        );
        return;
    }
    let mut r20: u16 = (ram_word!(game, samus_contact_damage_index)).wrapping_add(15u16);
    let mut r22: u16 = 0;
    if ram_word!(game, samus_contact_damage_index) == 1u16 {
        r22 = 500u16;
    } else if ram_word!(game, samus_contact_damage_index) == 2u16 {
        r22 = 300u16;
    } else if ram_word!(game, samus_contact_damage_index) == 3u16 {
        r22 = 2000u16;
    } else {
        r20 = r20.wrapping_add(1);
        if ram_word!(game, samus_contact_damage_index) == 4u16 {
            CallSomeSamusCode(4u16, game);
        }
        r22 = 200u16;
    }
    let vp: u16 = (if (*ED).vulnerability_ptr != 0u16 {
        (*ED).vulnerability_ptr as i32
    } else {
        addr_stru_B4EC1C as i32
    }) as u16;
    set_ram_word!(
        game.last_enemy_power = *(&raw mut (*(get_Vulnerability
            as unsafe fn(u16, &mut Game) -> *mut Vulnerability)(
            (r20).wrapping_add(vp), game,
        ))
        .power as *mut u16)
    );
    let varE32: u16 = (ram_word!(game, last_enemy_power)) & (0x7fu16);
    if (ram_word!(game, last_enemy_power) as i32 & 0x7fi32) != 0i32 {
        let dmg: u16 = ((r22 as i32 >> 1i32) * varE32 as i32) as u16;
        if dmg != 0 {
            (*E).flash_timer = (if (*ED).hurt_ai_time != 0u8 {
                (*ED).hurt_ai_time as i32
            } else {
                4i32
            }) as u16;
            (*E).ai_handler_bits = ((*E).ai_handler_bits) | ((kEnemyAiBits_Hurt) as u16);
            set_ram_word!(game.samus_invincibility_timer = 0u16);
            set_ram_word!(game.samus_knockback_timer = 0u16);
            (*E).health = (if (((*E).health) as i16).wrapping_sub((dmg) as i16) < 0i16 {
                0i32
            } else {
                ((((*E).health) as i16).wrapping_sub((dmg) as i16)) as i32
            }) as u16;
            QueueSfx2_Max1(0xbu16, game);
        }
    }
}
pub unsafe fn NormalEnemyPowerBombAi(game: &mut Game) {
    NormalEnemyPowerBombAiSkipDeathAnim(game);
    if (*gEnemyData(ram_word!(game, cur_enemy_index) as i32, game)).health == 0 {
        (*gEnemySpawnData(ram_word!(game, cur_enemy_index) as i32, game)).cause_of_death = 3u16;
        EnemyDeathAnimation(ram_word!(game, cur_enemy_index), 0u16, game);
    }
}
pub unsafe fn NormalEnemyPowerBombAiSkipDeathAnim_CurEnemy(game: &mut Game) {
    NormalEnemyPowerBombAiSkipDeathAnim(game);
}
pub unsafe fn NormalEnemyPowerBombAiSkipDeathAnim(game: &mut Game) {
    let mut EnemyDef_A2: *mut EnemyDef = null_mut::<EnemyDef>();
    let mut hurt_ai_time: i16 = 0;
    let v0: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    let mut vulnerability_ptr: u16 = (*get_EnemyDef_A2((*v0).enemy_ptr, game)).vulnerability_ptr;
    if vulnerability_ptr == 0 {
        vulnerability_ptr = -5092i32 as u16;
    }
    let power_bomb: u8 = (*get_Vulnerability(vulnerability_ptr, game)).power_bomb;
    if power_bomb != 255u8 {
        let varE32: u16 = ((power_bomb) as u16) & (0x7fu16);
        if (power_bomb as i32 & 0x7fi32) != 0i32 {
            let ttt: u32 = (100u32).wrapping_mul((varE32) as u32);
            if ttt != 0 {
                (*gEnemyData(ram_word!(game, cur_enemy_index) as i32, game)).invincibility_timer =
                    48u16;
                let j: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
                EnemyDef_A2 = get_EnemyDef_A2((*j).enemy_ptr, game);
                hurt_ai_time = (*EnemyDef_A2).hurt_ai_time as i16;
                if (*EnemyDef_A2).hurt_ai_time == 0 {
                    hurt_ai_time = 4i16;
                }
                let v6: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
                (*v6).flash_timer = ((hurt_ai_time) as u16).wrapping_add(8u16);
                (*v6).ai_handler_bits = ((*v6).ai_handler_bits) | (2u16);
                let health: u16 = (*v6).health;
                let mut v9: bool = (health as u32) < ttt;
                let mut v8: u16 = (health as u32).wrapping_sub(ttt) as u16;
                v9 = !v9;
                if v8 == 0 || !v9 {
                    v8 = 0u16;
                }
                (*v6).health = v8;
            }
        }
    }
}
pub unsafe fn NormalEnemyShotAi(game: &mut Game) {
    let varE2E: u16 = NormalEnemyShotAiSkipDeathAnim(game);
    let E: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    if varE2E != 0 {
        CreateSpriteAtPos((*E).x_pos, (*E).y_pos, 55u16, 0u16, game);
    }
    if (*E).health == 0 {
        let j: u16 = ((*(((&raw mut game.g_ram as *mut u8).offset(0xc18) as *mut u16)
            .offset(ram_word!(game, collision_detection_index) as isize)
            as *mut u16 as *mut u8)
            .offset(1isize)) as u16)
            & (0xfu16);
        (*gEnemySpawnData(ram_word!(game, cur_enemy_index) as i32, game)).cause_of_death = j;
        let mut death_anim: u16 = 2u16;
        if j == 2u16 {
            let ED: *mut EnemyDef = get_EnemyDef_A2((*E).enemy_ptr, game);
            if (*ED).death_anim as i32 - 3i32 & 0x8000i32 == 0 {
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
        let ED: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
        CreateSpriteAtPos((*ED).x_pos, (*ED).y_pos, 55u16, 0u16, game);
    }
}
pub unsafe fn NormalEnemyShotAiSkipDeathAnim(game: &mut Game) -> u16 {
    let mut ttt: u32 = 0;
    let mut r42: u16 = 0;
    let current_block: u64;
    let mut varE2E: u16 = 0u16;
    let mut v5: i16 = 0;
    let mut v6: i16 = 0;
    let mut v9: i16 = 0;
    let mut EnemyDef_A2: *mut EnemyDef = null_mut::<EnemyDef>();
    let mut hurt_ai_time: i16 = 0;
    let mut v20: i16 = 0;
    let mut varE32: u16 = 0;
    let v0: i32 = ram_word!(game, collision_detection_index) as i32;
    let mut pd: u16 = ram_word!(game, projectile_damage[v0]);
    let r18: u16 = ram_word!(game, projectile_type[v0]);
    let j: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    let mut vulnerability_ptr: u16 = (*get_EnemyDef_A2((*j).enemy_ptr, game)).vulnerability_ptr;
    if vulnerability_ptr == 0 {
        vulnerability_ptr = addr_stru_B4EC1C as u16;
    }
    let r20: u16 = vulnerability_ptr;
    if (r18 as i32 & 0xf00i32) != 0i32 {
        v5 = ((r18) as i16) & (0xf00i16);
        if (r18 as i32 & 0xf00i32) == 256i32 || v5 == 512i16 {
            v6 = ((r18 as i32 & 0xf00i32) >> 8i32) as i16;
            varE32 = (((*get_Vulnerability((r20).wrapping_add((v6) as u16), game)).plasma_ice_wave)
                as u16)
                & (0x7fu16);
            current_block = 5143058163439228106;
        } else if v5 == 1280i16 {
            varE32 = (((*get_Vulnerability(r20, game)).bomb) as u16) & (0x7fu16);
            current_block = 5143058163439228106;
        } else if v5 != 768i16 {
            current_block = 3938820862080741272;
        } else {
            varE32 = (((*get_Vulnerability(r20, game)).power_bomb) as u16) & (0x7fu16);
            current_block = 5143058163439228106;
        }
    } else {
        set_ram_word!(
            game.last_enemy_power =
                (*get_Vulnerability((r20 as i32 + (r18 as i32 & 0xfi32)) as u16, game,)).power
                    as u16
        );
        varE32 = (ram_word!(game, last_enemy_power)) & (0x7fu16);
        if ram_word!(game, last_enemy_power) != 255u16 {
            if (r18 as i32 & 0x10i32) != 0i32 {
                let charged_beam: u8 = (*get_Vulnerability(r20, game)).charged_beam;
                if charged_beam == 255u8 {
                    current_block = 3938820862080741272;
                } else {
                    let v4: u16 = ((charged_beam) as u16) & (0xfu16);
                    if v4 == 0 {
                        current_block = 3938820862080741272;
                    } else {
                        varE32 = v4;
                        current_block = 5143058163439228106;
                    }
                }
            } else {
                current_block = 5143058163439228106;
            }
        } else {
            let v8: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
            if (*v8).frozen_timer == 0 {
                QueueSfx3_Max3(0xau16, game);
            }
            v9 = 400i16;
            if ram_word!(game, area_index) == 2u16 {
                v9 = 300i16;
            }
            (*v8).frozen_timer = v9 as u16;
            (*v8).ai_handler_bits = ((*v8).ai_handler_bits) | (4u16);
            (*v8).invincibility_timer = 10u16;
            return varE2E;
        }
    }
    if current_block == 5143058163439228106 {
        ttt = ((pd as i32 >> 1i32) as u32).wrapping_mul(varE32 as u32);
        r42 = ttt as u16;
        if r42 != 0 {
            pd = r42;
            let v10: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
            EnemyDef_A2 = get_EnemyDef_A2((*v10).enemy_ptr, game);
            hurt_ai_time = (*EnemyDef_A2).hurt_ai_time as i16;
            if (*EnemyDef_A2).hurt_ai_time == 0 {
                hurt_ai_time = 4i16;
            }
            let v13: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
            (*v13).flash_timer = ((hurt_ai_time) as u16).wrapping_add(8u16);
            (*v13).ai_handler_bits = ((*v13).ai_handler_bits) | (2u16);
            if (*v13).frozen_timer == 0 {
                let hurt_sfx: u16 = (*get_EnemyDef_A2((*v13).enemy_ptr, game)).hurt_sfx;
                if hurt_sfx != 0 {
                    QueueSfx2_Max3(hurt_sfx, game);
                }
                varE2E = varE2E.wrapping_add(1);
            }
            let v15: u16 = ram_word!(game, cur_enemy_index);
            if (ram_word!(
                game,
                projectile_type[ram_word!(game, collision_detection_index)]
            ) as i32
                & 8i32)
                != 0i32
            {
                (*gEnemyData(ram_word!(game, cur_enemy_index) as i32, game)).invincibility_timer =
                    16u16;
            }
            let v16: *mut EnemyData = gEnemyData(v15 as i32, game);
            let health: u16 = (*v16).health;
            let mut v19: bool = health < pd;
            let mut v18: u16 = (health).wrapping_sub(pd);
            v19 = !v19;
            if v18 == 0 || !v19 {
                if (ram_word!(
                    game,
                    projectile_type[ram_word!(game, collision_detection_index)]
                ) as i32
                    & 2i32)
                    != 0i32
                    && (ram_word!(game, last_enemy_power) as i32 & 0xf0i32) != 128i32
                    && (*v16).frozen_timer == 0
                {
                    v20 = 400i16;
                    if ram_word!(game, area_index) == 2u16 {
                        v20 = 300i16;
                    }
                    (*v16).frozen_timer = v20 as u16;
                    (*v16).ai_handler_bits = ((*v16).ai_handler_bits) | (4u16);
                    (*v16).invincibility_timer = 10u16;
                    QueueSfx3_Max3(0xau16, game);
                    return varE2E;
                }
                v18 = 0u16;
            }
            (*v16).health = v18;
            return varE2E;
        }
    }
    let v7: i32 = ram_word!(game, collision_detection_index) as i32;
    update_ram_word!(game.projectile_dir[v7], |fresh25| (fresh25) | (0x10u16));
    CreateSpriteAtPos(
        ram_word!(game, projectile_x_pos[v7]),
        ram_word!(game, projectile_y_pos[v7]),
        6u16,
        0u16,
        game,
    );
    QueueSfx1_Max3(0x3du16, game);
    varE2E
}
pub unsafe fn CreateDudShot(game: &mut Game) {
    let v0: i32 = ram_word!(game, collision_detection_index) as i32;
    CreateSpriteAtPos(
        ram_word!(game, projectile_x_pos[v0]),
        ram_word!(game, projectile_y_pos[v0]),
        6u16,
        0u16,
        game,
    );
    QueueSfx1_Max3(0x3du16, game);
    update_ram_word!(
        game.projectile_dir[ram_word!(game, collision_detection_index)],
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
pub fn Mult32(a: u16) -> u16 {
    a.wrapping_mul(32)
}
pub fn Abs16(a: u16) -> u16 {
    (a as i16).wrapping_abs() as u16
}
pub fn SubtractThenAbs16(k: u16, j: u16) -> u16 {
    Abs16(j.wrapping_sub(k))
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

pub fn CosineMult8bitFull(phase: u16, amplitude: u16, game: &mut Game) -> u32 {
    SineMult8bitInner(amplitude, phase.wrapping_add(64) as u8 as u16, game)
}

pub fn SineMult8bitFull(phase: u16, amplitude: u16, game: &mut Game) -> u32 {
    SineMult8bitInner(amplitude, phase.wrapping_add(0x80) as u8 as u16, game)
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
        0u32
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
pub unsafe fn Enemy_ItemDrop_LowerNorfairSpacePirate(k: u16, game: &mut Game) {
    spawn_nearby_enemy_item_drops(addr_kEnemyDef_F593 as u16, k, 5, 0, game);
}
pub unsafe fn Enemy_ItemDrop_Metroid(k: u16, game: &mut Game) {
    spawn_nearby_enemy_item_drops(addr_kEnemyDef_DD7F as u16, k, 5, 0, game);
}
pub unsafe fn Enemy_ItemDrop_Ridley(k: u16, game: &mut Game) {
    spawn_area_enemy_item_drops(addr_kEnemyDef_E17F as u16, k, 64, 320, 127, game);
}
pub unsafe fn Enemy_ItemDrop_Crocomire(k: u16, game: &mut Game) {
    spawn_area_enemy_item_drops(addr_kEnemyDef_DDBF as u16, k, 576, 96, 127, game);
}
pub unsafe fn Enemy_ItemDrop_Phantoon(k: u16, game: &mut Game) {
    spawn_area_enemy_item_drops(addr_kEnemyDef_E4BF as u16, k, 64, 96, 127, game);
}
pub unsafe fn Enemy_ItemDrop_Botwoon(k: u16, game: &mut Game) {
    spawn_area_enemy_item_drops(addr_kEnemyDef_F293 as u16, k, 64, 128, 127, game);
}
pub unsafe fn Enemy_ItemDrop_Kraid(k: u16, game: &mut Game) {
    spawn_area_enemy_item_drops(addr_kEnemyDef_E2BF as u16, k, 128, 352, 255, game);
}
pub unsafe fn Enemy_ItemDrop_BombTorizo(k: u16, game: &mut Game) {
    spawn_area_enemy_item_drops(addr_kEnemyDef_EEFF as u16, k, 64, 96, 127, game);
}
pub unsafe fn Enemy_ItemDrop_GoldenTorizo(k: u16, game: &mut Game) {
    spawn_area_enemy_item_drops(addr_kEnemyDef_EEFF as u16, k, 128, 288, 255, game);
}
pub unsafe fn Enemy_ItemDrop_SporeSpawn(k: u16, game: &mut Game) {
    spawn_area_enemy_item_drops(addr_kEnemyDef_DF3F as u16, k, 64, 528, 127, game);
}
pub unsafe fn Enemy_ItemDrop_Draygon(k: u16, game: &mut Game) {
    spawn_area_enemy_item_drops(addr_kEnemyDef_DE3F as u16, k, 128, 352, 255, game);
}
pub fn CompareDistToSamus_X(k: u16, a: u16, game: &mut Game) -> u8 {
    let delta = GetSamusEnemyDelta_X(k, game);
    u8::from(Abs16(delta) >= a)
}
pub fn CompareDistToSamus_Y(k: u16, a: u16, game: &mut Game) -> u8 {
    let delta = GetSamusEnemyDelta_Y(k, game);
    u8::from(Abs16(delta) >= a)
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
    let position = (amt as u32).wrapping_add(
        (u32::from(game.enemy_coordinates(k).x_pos()) << 16)
            | u32::from(game.enemy_coordinates(k).x_subpos()),
    );
    let x = (position >> 16) as u16;
    let edge = if amt < 0 {
        x.wrapping_sub(game.enemy_coordinates(k).x_width())
    } else {
        game.enemy_coordinates(k)
            .x_width()
            .wrapping_add(x)
            .wrapping_sub(1)
    };
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
    let position = (amt as u32).wrapping_add(
        (u32::from(game.enemy_coordinates(k).y_pos()) << 16)
            | u32::from(game.enemy_coordinates(k).y_subpos()),
    );
    let y = (position >> 16) as u16;
    let edge = if amt < 0 {
        y.wrapping_sub(game.enemy_coordinates(k).y_height())
    } else {
        game.enemy_coordinates(k)
            .y_height()
            .wrapping_add(y)
            .wrapping_sub(1)
    };
    let row_offset = u16::from((edge >> 4) as u8) * u16::from(game.room_width_in_blocks() as u8);
    // This cursor is not truncated to a word as the horizontal scan's is.
    let mut block_offset = 2 * (usize::from(row_offset) + usize::from(left >> 4));
    loop {
        let offset = 0x10002 + block_offset;
        let block = u16::from_le_bytes([game.g_ram[offset], game.g_ram[offset + 1]]);
        if block & 0x8000 != 0 {
            return 1;
        }
        columns = columns.wrapping_sub(1);
        if columns & 0x8000 != 0 {
            return 0;
        }
        block_offset += 2;
    }
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
    let row_offset = u16::from((edge >> 4) as u8) * u16::from(game.room_width_in_blocks() as u8);
    let mut block_offset = 2 * (usize::from(row_offset) + usize::from(left >> 4));
    loop {
        let offset = 0x10002 + block_offset;
        let block = u16::from_le_bytes([game.g_ram[offset], game.g_ram[offset + 1]]);
        if block & 0x8000 != 0 {
            return 1;
        }
        columns = columns.wrapping_sub(1);
        if columns & 0x8000 != 0 {
            return 0;
        }
        block_offset += 2;
    }
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
        __assert_fail(
            b"E->current_instruction & 0x8000\0" as *const u8 as *const c_char,
            b"src/sm_a0.c\0" as *const u8 as *const c_char,
            3399u32,
            b"void ProcessEnemyInstructions(void)\0" as *const u8 as *const c_char,
        );
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
static byte_A0C435: [u8; 20] = [
    0, 1, 0x82, 0x83, 0, 0x81, 2, 0x83, 0, 1, 2, 0x83, 0, 0x81, 0x82, 0x83, 0x80, 0x81, 0x82, 0x83,
];
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
    let near_is_solid = byte_A0C435[index] & 0x80 != 0;
    if collision.ebci_r28 == 0 {
        // The original bottom-edge test reads only the low bytes of both fields.
        let far_edge = (position as u8).wrapping_add(size as u8).wrapping_sub(1);
        if far_edge & 8 == 0 {
            return u8::from(near_is_solid);
        }
    } else if collision.ebci_r28 == collision.ebci_r30 && position.wrapping_sub(size) & 8 != 0 {
        return u8::from(byte_A0C435[index ^ 2] & 0x80 != 0);
    }
    if near_is_solid {
        return 1;
    }
    u8::from(byte_A0C435[index ^ 2] & 0x80 != 0)
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
    let near_is_solid = byte_A0C435[index] & 0x80 != 0;
    if collision.ebci_r28 == 0 {
        // The original bottom-edge test reads only the low bytes of both fields.
        let far_edge = (position as u8).wrapping_add(size as u8).wrapping_sub(1);
        if far_edge & 8 == 0 {
            return u8::from(near_is_solid);
        }
    } else if collision.ebci_r28 == collision.ebci_r30 && position.wrapping_sub(size) & 8 != 0 {
        return u8::from(byte_A0C435[index ^ 1] & 0x80 != 0);
    }
    if near_is_solid {
        return 1;
    }
    u8::from(byte_A0C435[index ^ 1] & 0x80 != 0)
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
pub unsafe fn Enemy_MoveRight_SlopesAsWalls(k: u16, amt: i32, game: &mut Game) -> u8 {
    Enemy_MoveRight_IgnoreSlopes_Inner(k, amt, 0x4000u16, game)
}
pub unsafe fn Enemy_MoveRight_ProcessSlopes(k: u16, amt: i32, game: &mut Game) -> u8 {
    Enemy_MoveRight_IgnoreSlopes_Inner(k, amt, 0x8000u16, game)
}
pub unsafe fn Enemy_MoveRight_IgnoreSlopes(k: u16, amt: i32, game: &mut Game) -> u8 {
    Enemy_MoveRight_IgnoreSlopes_Inner(k, amt, 0u16, game)
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
    let prod: u16 = Mult8x8(
        (ypos as i32 >> 4i32) as u8,
        ram_word!(game, room_width_in_blocks) as u8,
    );
    set_ram_word!(game.cur_block_index = (prod as i32 + (xpos as i32 >> 4i32)) as u16);
}
pub unsafe fn EnemyFunc_C8AD(k: u16, game: &mut Game) -> u8 {
    let mut result: u8 = 0u8;
    let E: *mut EnemyData = gEnemyData(k as i32, game);
    CalculateBlockContainingPixelPos(
        (*E).x_pos,
        ((*E).y_pos as i32 + (*E).y_height as i32 - 1i32) as u16,
        game,
    );
    if (ram_word!(game, level_data[ram_word!(game, cur_block_index)]) as i32 & 0xf000i32) == 4096i32
        && (*(&raw mut game.g_ram as *mut u8)
            .offset(0x16402)
            .offset(ram_word!(game, cur_block_index) as isize) as i32
            & 0x1fi32)
            >= 5i32
    {
        result = 1u8;
        let temp_collision_DD4: u16 =
            ((*E).y_height as i32 + (*E).y_pos as i32 - 1i32 & 0xfi32) as u16;
        let temp_collision_DD6: u16 = (16i32
            * (*(&raw mut game.g_ram as *mut u8)
                .offset(0x16402)
                .offset(ram_word!(game, cur_block_index) as isize) as i32
                & 0x1fi32)) as u16;
        if (*(&raw mut game.g_ram as *mut u8)
            .offset(0x16402)
            .offset(ram_word!(game, cur_block_index) as isize) as i32
            & 0x80i32)
            == 0i32
        {
            let j: u16 = (if (*(&raw mut game.g_ram as *mut u8)
                .offset(0x16402)
                .offset(ram_word!(game, cur_block_index) as isize)
                as i32
                & 0x40i32)
                != 0i32
            {
                (*E).x_pos as i32 ^ 0xfi32
            } else {
                (*E).x_pos as i32
            }) as u16;
            let v4: i16 = ((*(game.g_rom.offset(crate::rtl::lorom_offset(0x948b2b)) as *const u8
                as *mut u8)
                .offset((temp_collision_DD6 as i32 + (j as i32 & 0xfi32)) as isize)
                as i32
                & 0x1fi32)
                - temp_collision_DD4 as i32
                - 1i32) as i16;
            if v4 < 0i16 {
                (*E).y_pos = ((*E).y_pos).wrapping_add((v4) as u16);
            }
        }
    }
    CalculateBlockContainingPixelPos((*E).x_pos, ((*E).y_pos).wrapping_sub((*E).y_height), game);
    if (ram_word!(game, level_data[ram_word!(game, cur_block_index)]) as i32 & 0xf000i32) == 4096i32
        && (*(&raw mut game.g_ram as *mut u8)
            .offset(0x16402)
            .offset(ram_word!(game, cur_block_index) as isize) as i32
            & 0x1fi32)
            >= 5i32
    {
        result = 1u8;
        let temp_collision_DD4_0: u16 =
            ((*E).y_pos as i32 - (*E).y_height as i32 & 0xfi32 ^ 0xfi32) as u16;
        let temp_collision_DD6_0: u16 = (16i32
            * (*(&raw mut game.g_ram as *mut u8)
                .offset(0x16402)
                .offset(ram_word!(game, cur_block_index) as isize) as i32
                & 0x1fi32)) as u16;
        if *(&raw mut game.g_ram as *mut u8)
            .offset(0x16402)
            .offset(ram_word!(game, cur_block_index) as isize) as i32
            & 0x80i32
            != 0
        {
            let j_0: u16 = (if (*(&raw mut game.g_ram as *mut u8)
                .offset(0x16402)
                .offset(ram_word!(game, cur_block_index) as isize)
                as i32
                & 0x40i32)
                != 0i32
            {
                (*E).x_pos as i32 ^ 0xfi32
            } else {
                (*E).x_pos as i32
            }) as u16;
            let v7: i16 = ((*(game.g_rom.offset(crate::rtl::lorom_offset(0x948b2b)) as *const u8
                as *mut u8)
                .offset((temp_collision_DD6_0 as i32 + (j_0 as i32 & 0xfi32)) as isize)
                as i32
                & 0x1fi32)
                - temp_collision_DD4_0 as i32
                - 1i32) as i16;
            if v7 < 0i16 {
                (*E).y_pos = ((*E).y_pos).wrapping_sub((v7) as u16);
            }
        }
    }
    result
}
