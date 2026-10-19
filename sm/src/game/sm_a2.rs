use crate::game::sm_80::{CheckEventHappened, NextRandom, QueueSfx2_Max6, QueueSfx3_Max6};
use crate::game::sm_81::SaveToSram;
use crate::game::sm_85::DisplayMessageBox_Poll;
use crate::game::sm_86::{
    Math_MultByCos, Math_MultBySin, SpawnEprojWithGfx, SpawnEprojWithRoomGfx,
};
use crate::game::sm_8d::SpawnPalfxObject;
use crate::game::sm_90::CallSomeSamusCode;
use crate::game::sm_91::{
    MakeSamusFaceForward, Samus_RestoreHealth, Samus_RestoreMissiles, Samus_RestorePowerBombs,
    Samus_RestoreSuperMissiles,
};
use crate::game::sm_94::kSinCosTable8bit_Sext;
use crate::game::sm_a0::{
    CalculateAngleFromXY, CalculateAngleOfSamusFromEnemy, CallEnemyPreInstr,
    CheckIfEnemyIsOnScreen, CheckIfEnemyTouchesSamus, CompareDistToSamus_X, ConvertAngleToXy,
    DeleteEnemyAndConnectedEnemies, EnemyFunc_AC67, EnemyFunc_BC76, EnemyGrappleDeath,
    Enemy_MoveDown, Enemy_MoveRight_IgnoreSlopes, Enemy_MoveRight_SlopesAsWalls,
    Enemy_SwitchToFrozenAi, GetSamusEnemyDelta_X, IsSamusWithinEnemy_X, IsSamusWithinEnemy_Y,
    NormalEnemyFrozenAI, NormalEnemyPowerBombAi, NormalEnemyPowerBombAiSkipDeathAnim_CurEnemy,
    NormalEnemyShotAi, NormalEnemyShotAiSkipDeathAnim_CurEnemy, NormalEnemyTouchAi,
    NormalEnemyTouchAiSkipDeathAnim_CurEnemy, RinkasDeathAnimation, SamusLatchesOnWithGrapple,
};
use crate::game::sm_a9::MoveEnemyWithVelocity;
use crate::game_state::Game;
use crate::rtl::{Mult8x8, RomPtr, Unreachable};

pub use crate::types::{
    __int16_t, __int32_t, __int8_t, __uint16_t, __uint32_t, __uint8_t, int16, int16_t, int32,
    int32_t, int8, int8_t, uint16, uint16_t, uint32, uint32_t, uint8, uint8_t, uintptr_t,
    EnemyBase, EnemyData, Func_V, LongPtr, Point16U, SamusPose, VoidP, VramWriteEntry,
};
pub const kPose_FC_FaceL_StandTrans_AimDL: SamusPose = 252;
pub const kPose_FB_FaceR_StandTrans_AimDR: SamusPose = 251;
pub const kPose_FA_FaceL_StandTrans_AimUL: SamusPose = 250;
pub const kPose_F9_FaceR_StandTrans_AimUR: SamusPose = 249;
pub const kPose_F8_FaceL_StandTrans_AimU: SamusPose = 248;
pub const kPose_F7_FaceR_StandTrans_AimU: SamusPose = 247;
pub const kPose_F6_FaceL_CrouchTrans_AimDL: SamusPose = 246;
pub const kPose_F5_FaceR_CrouchTrans_AimDR: SamusPose = 245;
pub const kPose_F4_FaceL_CrouchTrans_AimUL: SamusPose = 244;
pub const kPose_F3_FaceR_CrouchTrans_AimUR: SamusPose = 243;
pub const kPose_F2_FaceL_CrouchTrans_AimU: SamusPose = 242;
pub const kPose_F1_FaceR_CrouchTrans_AimU: SamusPose = 241;
pub const kPose_F0_FaceR_Draygon_Move: SamusPose = 240;
pub const kPose_EF_FaceR_Draygon_NoMove_AimDR: SamusPose = 239;
pub const kPose_EE_FaceR_Draygon_Fire: SamusPose = 238;
pub const kPose_ED_FaceR_Draygon_NoMove_AimUR: SamusPose = 237;
pub const kPose_EC_FaceR_Draygon_NoMove_NoAim: SamusPose = 236;
pub const kPose_EB_FaceL_Drained_Stand: SamusPose = 235;
pub const kPose_EA_FaceR_Drained_Stand: SamusPose = 234;
pub const kPose_E9_FaceL_Drained_CrouchFalling: SamusPose = 233;
pub const kPose_E8_FaceR_Drained_CrouchFalling: SamusPose = 232;
pub const kPose_E7_FaceL_LandJump_Fire: SamusPose = 231;
pub const kPose_E6_FaceR_LandJump_Fire: SamusPose = 230;
pub const kPose_E5_FaceL_LandJump_AimDL: SamusPose = 229;
pub const kPose_E4_FaceR_LandJump_AimDR: SamusPose = 228;
pub const kPose_E3_FaceL_LandJump_AimUL: SamusPose = 227;
pub const kPose_E2_FaceR_LandJump_AimUR: SamusPose = 226;
pub const kPose_E1_FaceL_LandJump_AimU: SamusPose = 225;
pub const kPose_E0_FaceR_LandJump_AimU: SamusPose = 224;
pub const kPose_DB: SamusPose = 219;
pub const kPose_DA_FaceL_Xray_Crouch: SamusPose = 218;
pub const kPose_D9_FaceR_Xray_Crouch: SamusPose = 217;
pub const kPose_D8_FaceL_CrystalFlashEnd: SamusPose = 216;
pub const kPose_D7_FaceR_CrystalFlashEnd: SamusPose = 215;
pub const kPose_D6_FaceL_Xray_Stand: SamusPose = 214;
pub const kPose_D5_FaceR_Xray_Stand: SamusPose = 213;
pub const kPose_D4_FaceL_CrystalFlash: SamusPose = 212;
pub const kPose_D3_FaceR_CrystalFlash: SamusPose = 211;
pub const kPose_D2_FaceL_Ranintowall_AimDL: SamusPose = 210;
pub const kPose_D1_FaceR_Ranintowall_AimDR: SamusPose = 209;
pub const kPose_D0_FaceL_Ranintowall_AimUL: SamusPose = 208;
pub const kPose_CF_FaceR_Ranintowall_AimUR: SamusPose = 207;
pub const kPose_CE_FaceL_Shinespark_Diag: SamusPose = 206;
pub const kPose_CD_FaceR_Shinespark_Diag: SamusPose = 205;
pub const kPose_CC_FaceL_Shinespark_Vert: SamusPose = 204;
pub const kPose_CB_FaceR_Shinespark_Vert: SamusPose = 203;
pub const kPose_CA_FaceL_Shinespark_Horiz: SamusPose = 202;
pub const kPose_C9_FaceR_Shinespark_Horiz: SamusPose = 201;
pub const kPose_C8_FaceL_ShinesparkWindup_Vert: SamusPose = 200;
pub const kPose_C7_FaceR_ShinesparkWindup_Vert: SamusPose = 199;
pub const kPose_C4_FaceL_Moonwalk_TurnjumpR_AimDL: SamusPose = 196;
pub const kPose_C3_FaceR_Moonwalk_TurnjumpL_AimDR: SamusPose = 195;
pub const kPose_C2_FaceL_Moonwalk_TurnjumpR_AimUL: SamusPose = 194;
pub const kPose_C1_FaceR_Moonwalk_TurnjumpL_AimUR: SamusPose = 193;
pub const kPose_C0_FaceL_Moonwalk_TurnjumpR: SamusPose = 192;
pub const kPose_BF_FaceR_Moonwalk_TurnjumpL: SamusPose = 191;
pub const kPose_BE_FaceL_Draygon_Move: SamusPose = 190;
pub const kPose_BD_FaceL_Draygon_NoMove_AimDL: SamusPose = 189;
pub const kPose_BC_FaceL_Draygon_Fire: SamusPose = 188;
pub const kPose_BB_FaceL_Draygon_NoMove_AimUL: SamusPose = 187;
pub const kPose_BA_FaceL_Draygon_NoMove_NoAim: SamusPose = 186;
pub const kPose_B9_FaceR_GrappleWalljumpPose: SamusPose = 185;
pub const kPose_B8_FaceL_GrappleWalljumpPose: SamusPose = 184;
pub const kPose_B7_FaceL_Grappling_Crouch_AimDL: SamusPose = 183;
pub const kPose_B6_FaceR_Grappling_Crouch_AimDR: SamusPose = 182;
pub const kPose_B5_FaceL_Grappling_Crouch: SamusPose = 181;
pub const kPose_B4_FaceR_Grappling_Crouch: SamusPose = 180;
pub const kPose_B3_FaceL_Grapple_Air: SamusPose = 179;
pub const kPose_B2_FaceR_Grapple_Air: SamusPose = 178;
pub const kPose_AB_FaceL_Grappling_AimDL: SamusPose = 171;
pub const kPose_AA_FaceR_Grappling_AimDR: SamusPose = 170;
pub const kPose_A9_FaceL_Grappling: SamusPose = 169;
pub const kPose_A8_FaceR_Grappling: SamusPose = 168;
pub const kPose_A7_FaceL_LandSpinJump: SamusPose = 167;
pub const kPose_A6_FaceR_LandSpinJump: SamusPose = 166;
pub const kPose_A5_FaceL_LandJump: SamusPose = 165;
pub const kPose_A4_FaceR_LandJump: SamusPose = 164;
pub const kPose_A3_FaceL_Turn_Crouch_AimUL: SamusPose = 163;
pub const kPose_A2_FaceR_Turn_Crouch_AimUR: SamusPose = 162;
pub const kPose_A1_FaceL_Turn_Fall_AimUL: SamusPose = 161;
pub const kPose_A0_FaceR_Turn_Fall_AimUR: SamusPose = 160;
pub const kPose_9F_FaceL_Turn_Air_AimUL: SamusPose = 159;
pub const kPose_9E_FaceR_Turn_Air_AimUR: SamusPose = 158;
pub const kPose_9D_FaceL_Turn_Stand_AimUL: SamusPose = 157;
pub const kPose_9C_FaceR_Turn_Stand_AimUR: SamusPose = 156;
pub const kPose_9B_FaceF_VariaGravitySuit: SamusPose = 155;
pub const kPose_9A_FaceL_Turn_Crouch_AimDDL: SamusPose = 154;
pub const kPose_99_FaceR_Turn_Crouch_AimDDR: SamusPose = 153;
pub const kPose_98_FaceL_Turn_Crouch_AimU: SamusPose = 152;
pub const kPose_97_FaceR_Turn_Crouch_AimU: SamusPose = 151;
pub const kPose_96_FaceL_Turn_Fall_AimDDL: SamusPose = 150;
pub const kPose_95_FaceR_Turn_Fall_AimDDR: SamusPose = 149;
pub const kPose_94_FaceL_Turn_Fall_AimU: SamusPose = 148;
pub const kPose_93_FaceR_Turn_Fall_AimU: SamusPose = 147;
pub const kPose_92_FaceL_Turn_Air_AimDDL: SamusPose = 146;
pub const kPose_91_FaceR_Turn_Air_AimDDR: SamusPose = 145;
pub const kPose_90_FaceL_Turn_Air_AimU: SamusPose = 144;
pub const kPose_8F_FaceR_Turn_Air_AimU: SamusPose = 143;
pub const kPose_8E_FaceL_Turn_Stand_AimDL: SamusPose = 142;
pub const kPose_8D_FaceR_Turn_Stand_AimDR: SamusPose = 141;
pub const kPose_8C_FaceL_Turn_Stand_AimU: SamusPose = 140;
pub const kPose_8B_FaceR_Turn_Stand_AimU: SamusPose = 139;
pub const kPose_8A_FaceL_Ranintowall: SamusPose = 138;
pub const kPose_89_FaceR_Ranintowall: SamusPose = 137;
pub const kPose_88_FaceL_Turn_Fall: SamusPose = 136;
pub const kPose_87_FaceR_Turn_Fall: SamusPose = 135;
pub const kPose_86_FaceL_Crouch_AimU: SamusPose = 134;
pub const kPose_85_FaceR_Crouch_AimU: SamusPose = 133;
pub const kPose_84_FaceL_Walljump: SamusPose = 132;
pub const kPose_83_FaceR_Walljump: SamusPose = 131;
pub const kPose_82_FaceL_Screwattack: SamusPose = 130;
pub const kPose_81_FaceR_Screwattack: SamusPose = 129;
pub const kPose_80_FaceL_Springball_Air: SamusPose = 128;
pub const kPose_7F_FaceR_Springball_Air: SamusPose = 127;
pub const kPose_7E_FaceL_Springball_Fall: SamusPose = 126;
pub const kPose_7D_FaceR_Springball_Fall: SamusPose = 125;
pub const kPose_7C_MoveL_Springball_Ground: SamusPose = 124;
pub const kPose_7B_MoveR_Springball_Ground: SamusPose = 123;
pub const kPose_7A_FaceL_Springball_Ground: SamusPose = 122;
pub const kPose_79_FaceR_Springball_Ground: SamusPose = 121;
pub const kPose_78_FaceR_Moonwalk_AimDR: SamusPose = 120;
pub const kPose_77_FaceL_Moonwalk_AimDL: SamusPose = 119;
pub const kPose_76_FaceR_Moonwalk_AimUR: SamusPose = 118;
pub const kPose_75_FaceL_Moonwalk_AimUL: SamusPose = 117;
pub const kPose_74_FaceL_Crouch_AimDL: SamusPose = 116;
pub const kPose_73_FaceR_Crouch_AimDR: SamusPose = 115;
pub const kPose_72_FaceL_Crouch_AimUL: SamusPose = 114;
pub const kPose_71_FaceR_Crouch_AimUR: SamusPose = 113;
pub const kPose_70_FaceL_Fall_AimDL: SamusPose = 112;
pub const kPose_6F_FaceR_Fall_AimDR: SamusPose = 111;
pub const kPose_6E_FaceL_Fall_AimUL: SamusPose = 110;
pub const kPose_6D_FaceR_Fall_AimUR: SamusPose = 109;
pub const kPose_6C_FaceL_Jump_AimDL: SamusPose = 108;
pub const kPose_6B_FaceR_Jump_AimDR: SamusPose = 107;
pub const kPose_6A_FaceL_Jump_AimUL: SamusPose = 106;
pub const kPose_69_FaceR_Jump_AimUR: SamusPose = 105;
pub const kPose_68_FaceL_Fall_Gun: SamusPose = 104;
pub const kPose_67_FaceR_Fall_Gun: SamusPose = 103;
pub const kPose_5B: SamusPose = 91;
pub const kPose_5A_FaceL_Jumptrans_AimDL: SamusPose = 90;
pub const kPose_59_FaceR_Jumptrans_AimDR: SamusPose = 89;
pub const kPose_58_FaceL_Jumptrans_AimUL: SamusPose = 88;
pub const kPose_57_FaceR_Jumptrans_AimUR: SamusPose = 87;
pub const kPose_56_FaceL_Jumptrans_AimU: SamusPose = 86;
pub const kPose_55_FaceR_Jumptrans_AimU: SamusPose = 85;
pub const kPose_54_FaceL_Knockback: SamusPose = 84;
pub const kPose_53_FaceR_Knockback: SamusPose = 83;
pub const kPose_52_FaceL_Jump_NoAim_MoveF: SamusPose = 82;
pub const kPose_51_FaceR_Jump_NoAim_MoveF: SamusPose = 81;
pub const kPose_50_FaceR_Dmgboost: SamusPose = 80;
pub const kPose_4F_FaceL_Dmgboost: SamusPose = 79;
pub const kPose_4E_FaceL_Jump_NoAim_NoMove_NoGun: SamusPose = 78;
pub const kPose_4D_FaceR_Jump_NoAim_NoMove_NoGun: SamusPose = 77;
pub const kPose_4C_FaceL_Jumptrans: SamusPose = 76;
pub const kPose_4B_FaceR_Jumptrans: SamusPose = 75;
pub const kPose_4A_FaceR_Moonwalk: SamusPose = 74;
pub const kPose_49_FaceL_Moonwalk: SamusPose = 73;
pub const kPose_44_FaceL_Turn_Crouch: SamusPose = 68;
pub const kPose_43_FaceR_Turn_Crouch: SamusPose = 67;
pub const kPose_41_FaceL_Morphball_Ground: SamusPose = 65;
pub const kPose_3E_FaceL_UnmorphTrans: SamusPose = 62;
pub const kPose_3D_FaceR_UnmorphTrans: SamusPose = 61;
pub const kPose_3C_FaceL_StandTrans: SamusPose = 60;
pub const kPose_3B_FaceR_StandTrans: SamusPose = 59;
pub const kPose_38_FaceL_MorphTrans: SamusPose = 56;
pub const kPose_37_FaceR_MorphTrans: SamusPose = 55;
pub const kPose_36_FaceL_CrouchTrans: SamusPose = 54;
pub const kPose_35_FaceR_CrouchTrans: SamusPose = 53;
pub const kPose_32_FaceL_Morphball_Air: SamusPose = 50;
pub const kPose_31_FaceR_Morphball_Air: SamusPose = 49;
pub const kPose_30_FaceL_Turn_Jump: SamusPose = 48;
pub const kPose_2F_FaceR_Turn_Jump: SamusPose = 47;
pub const kPose_2E_FaceL_Fall_AimD: SamusPose = 46;
pub const kPose_2D_FaceR_Fall_AimD: SamusPose = 45;
pub const kPose_2C_FaceL_Fall_AimU: SamusPose = 44;
pub const kPose_2B_FaceR_Fall_AimU: SamusPose = 43;
pub const kPose_2A_FaceL_Fall: SamusPose = 42;
pub const kPose_29_FaceR_Fall: SamusPose = 41;
pub const kPose_28_FaceL_Crouch: SamusPose = 40;
pub const kPose_27_FaceR_Crouch: SamusPose = 39;
pub const kPose_26_FaceL_Turn_Stand: SamusPose = 38;
pub const kPose_25_FaceR_Turn_Stand: SamusPose = 37;
pub const kPose_1F_MoveL_Morphball_Ground: SamusPose = 31;
pub const kPose_1E_MoveR_Morphball_Ground: SamusPose = 30;
pub const kPose_1D_FaceR_Morphball_Ground: SamusPose = 29;
pub const kPose_1C_FaceL_SpaceJump: SamusPose = 28;
pub const kPose_1B_FaceR_SpaceJump: SamusPose = 27;
pub const kPose_1A_FaceL_SpinJump: SamusPose = 26;
pub const kPose_19_FaceR_SpinJump: SamusPose = 25;
pub const kPose_18_FaceL_Jump_AimD: SamusPose = 24;
pub const kPose_17_FaceR_Jump_AimD: SamusPose = 23;
pub const kPose_16_FaceL_Jump_AimU: SamusPose = 22;
pub const kPose_15_FaceR_Jump_AimU: SamusPose = 21;
pub const kPose_14_FaceL_Jump_NoAim_NoMove_Gun: SamusPose = 20;
pub const kPose_13_FaceR_Jump_NoAim_NoMove_Gun: SamusPose = 19;
pub const kPose_12_MoveL_AimDL: SamusPose = 18;
pub const kPose_11_MoveR_AimDR: SamusPose = 17;
pub const kPose_10_MoveL_AimUL: SamusPose = 16;
pub const kPose_0F_MoveR_AimUR: SamusPose = 15;
pub const kPose_0C_MoveL_Gun: SamusPose = 12;
pub const kPose_0B_MoveR_Gun: SamusPose = 11;
pub const kPose_0A_MoveL_NoAim: SamusPose = 10;
pub const kPose_09_MoveR_NoAim: SamusPose = 9;
pub const kPose_08_FaceL_AimDL: SamusPose = 8;
pub const kPose_07_FaceR_AimDR: SamusPose = 7;
pub const kPose_06_FaceL_AimUL: SamusPose = 6;
pub const kPose_05_FaceR_AimUR: SamusPose = 5;
pub const kPose_04_FaceL_AimU: SamusPose = 4;
pub const kPose_03_FaceR_AimU: SamusPose = 3;
pub const kPose_02_FaceL_Normal: SamusPose = 2;
pub const kPose_01_FaceR_Normal: SamusPose = 1;
pub const kPose_00_FaceF_Powersuit: SamusPose = 0;
pub use crate::types::{EnemyProps, EnemySpawnData};
pub const kEnemyProps_RespawnIfKilled: EnemyProps = 16384;
pub const kEnemyProps_DisableSamusColl: EnemyProps = 8192;
pub const kEnemyProps_BlockPlasmaBeam: EnemyProps = 4096;
pub const kEnemyProps_ProcessedOffscreen: EnemyProps = 2048;
pub const kEnemyProps_Tangible: EnemyProps = 1024;
pub const kEnemyProps_Deleted: EnemyProps = 512;
pub const kEnemyProps_Invisible: EnemyProps = 256;
pub use crate::types::GameState;
pub const kGameState_44_TransitionFromDemo: GameState = 44;
pub const kGameState_43_TransitionFromDemo: GameState = 43;
pub const kGameState_42_PlayingDemo: GameState = 42;
pub const kGameState_41_TransitionToDemo: GameState = 41;
pub const kGameState_40_TransitionToDemo: GameState = 40;
pub const kGameState_39_EndingAndCredits: GameState = 39;
pub const kGameState_38_SamusEscapesFromZebes: GameState = 38;
pub const kGameState_37_CeresGoesBoomWithSamus: GameState = 37;
pub const kGameState_36_WhitingOutFromTimeUp: GameState = 36;
pub const kGameState_35_TimeUp: GameState = 35;
pub const kGameState_34_CeresGoesBoom: GameState = 34;
pub const kGameState_33_BlackoutFromCeres: GameState = 33;
pub const kGameState_32_MadeItToCeresElevator: GameState = 32;
pub const kGameState_31_SetUpNewGame: GameState = 31;
pub const kGameState_30_IntroCinematic: GameState = 30;
pub const kGameState_29_DebugGameOverMenu: GameState = 29;
pub const kGameState_28_Unused: GameState = 28;
pub const kGameState_27_ReserveTanksAuto: GameState = 27;
pub const kGameState_26_GameOverMenu: GameState = 26;
pub const kGameState_25_SamusNoHealth: GameState = 25;
pub const kGameState_24_SamusNoHealth: GameState = 24;
pub const kGameState_23_SamusNoHealth: GameState = 23;
pub const kGameState_22_SamusNoHealth: GameState = 22;
pub const kGameState_21_SamusNoHealth: GameState = 21;
pub const kGameState_20_SamusNoHealth: GameState = 20;
pub const kGameState_19_SamusNoHealth: GameState = 19;
pub const kGameState_18_Unpausing: GameState = 18;
pub const kGameState_17_Unpausing: GameState = 17;
pub const kGameState_16_Unpausing: GameState = 16;
pub const kGameState_15_Paused: GameState = 15;
pub const kGameState_14_Paused: GameState = 14;
pub const kGameState_13_Pausing: GameState = 13;
pub const kGameState_12_Pausing: GameState = 12;
pub const kGameState_11_LoadingNextRoom: GameState = 11;
pub const kGameState_10_LoadingNextRoom: GameState = 10;
pub const kGameState_9_HitDoorBlock: GameState = 9;
pub const kGameState_8_MainGameplay: GameState = 8;
pub const kGameState_7_MainGameplayFadeIn: GameState = 7;
pub const kGameState_6_LoadingGameData: GameState = 6;
pub const kGameState_5_FileSelectMap: GameState = 5;
pub const kGameState_4_FileSelectMenus: GameState = 4;
pub const kGameState_3_Unused: GameState = 3;
pub const kGameState_2_GameOptionsMenu: GameState = 2;
pub const kGameState_1_OpeningCinematic: GameState = 1;
pub const kGameState_0_Reset: GameState = 0;
pub use crate::types::LoadingGameState;
pub const kLoadingGameState_22_EscapingCeres: LoadingGameState = 34;
pub const kLoadingGameState_1F_StartingAtCeres: LoadingGameState = 31;
pub const kLoadingGameState_5_Main: LoadingGameState = 5;
pub const kLoadingGameState_0_Intro: LoadingGameState = 0;
pub use crate::types::Buttons;
pub const kButton_B: Buttons = 32768;
pub const kButton_Y: Buttons = 16384;
pub const kButton_Select: Buttons = 8192;
pub const kButton_Start: Buttons = 4096;
pub const kButton_Up: Buttons = 2048;
pub const kButton_Down: Buttons = 1024;
pub const kButton_Left: Buttons = 512;
pub const kButton_Right: Buttons = 256;
pub const kButton_A: Buttons = 128;
pub const kButton_X: Buttons = 64;
pub const kButton_L: Buttons = 32;
pub const kButton_R: Buttons = 16;
pub use crate::types::Consts_86;
pub const addr_kEproj_Sparks: Consts_86 = 62616;
pub const addr_kEproj_EnemyDeathExplosion: Consts_86 = 62277;
pub const addr_kEproj_Pickup: Consts_86 = 62263;
pub const addr_word_86ECA3: Consts_86 = 60579;
pub const addr_kEproj_YappingMawsBody: Consts_86 = 60565;
pub const addr_word_86EC5C: Consts_86 = 60508;
pub const addr_word_86EC56: Consts_86 = 60502;
pub const addr_kEproj_BotwoonsSpit: Consts_86 = 60488;
pub const addr_word_86EBAE: Consts_86 = 60334;
pub const addr_kEproj_BotwoonsBody: Consts_86 = 60320;
pub const addr_kEproj_SaveStationElectricity: Consts_86 = 59090;
pub const addr_kEproj_EyeDoorSmoke: Consts_86 = 58647;
pub const addr_kEproj_DustCloudExplosion: Consts_86 = 58633;
pub const addr_word_86E208: Consts_86 = 57864;
pub const addr_kEproj_LavaThrownByLavaman: Consts_86 = 57568;
pub const addr_word_86DFDE: Consts_86 = 57310;
pub const addr_word_86DFD8: Consts_86 = 57304;
pub const addr_stru_86DFCA: Consts_86 = 57290;
pub const addr_stru_86DFBC: Consts_86 = 57276;
pub const addr_word_86DEA6: Consts_86 = 56998;
pub const addr_word_86DE96: Consts_86 = 56982;
pub const addr_kEproj_SporeSpawners: Consts_86 = 56968;
pub const addr_kEproj_Spores: Consts_86 = 56954;
pub const addr_kEproj_SporeSpawnsStalk: Consts_86 = 56940;
pub const addr_word_86DC06: Consts_86 = 56326;
pub const addr_Eproj_DBF2: Consts_86 = 56306;
pub const addr_word_86DB0C: Consts_86 = 56076;
pub const addr_kEproj_SpikeShootingPlantSpikes: Consts_86 = 56062;
pub const addr_kEproj_N00bTubeReleasedAirBubbles: Consts_86 = 55584;
pub const addr_kEproj_N00bTubeShards: Consts_86 = 55570;
pub const addr_kEproj_WreckedShipRobotLaserDownRight: Consts_86 = 53982;
pub const addr_kEproj_WreckedShipRobotLaserUpRight: Consts_86 = 53968;
pub const addr_kEproj_WreckedShipRobotLaserDownLeft: Consts_86 = 53954;
pub const addr_kEproj_WreckedShipRobotLaserHorizontal: Consts_86 = 53940;
pub const addr_kEproj_WreckedShipRobotLaserUpLeft: Consts_86 = 53926;
pub const addr_kEproj_MaridiaFloatersSpikes: Consts_86 = 53912;
pub const addr_off_86D218: Consts_86 = 53784;
pub const addr_word_86D052: Consts_86 = 53330;
pub const addr_word_86D04A: Consts_86 = 53322;
pub const addr_word_86D03C: Consts_86 = 53308;
pub const addr_off_86CF56: Consts_86 = 53078;
pub const addr_kEproj_MotherBrainGlassShatteringSparkle: Consts_86 = 53002;
pub const addr_kEproj_MotherBrainTubeFalling_TopMiddleRight: Consts_86 = 52357;
pub const addr_kEproj_MotherBrainTubeFalling_TopMiddleLeft: Consts_86 = 52343;
pub const addr_kEproj_MotherBrainTubeFalling_TopLeft: Consts_86 = 52329;
pub const addr_kEproj_TimeBombSetJapaneseText: Consts_86 = 52155;
pub const addr_kEproj_MotherBrainRainbowBeamExplosion: Consts_86 = 52141;
pub const addr_kEproj_MotherBrainDyingDrool: Consts_86 = 52127;
pub const addr_kEproj_MotherBrainDrool: Consts_86 = 52113;
pub const addr_kEproj_MotherBrainRainbowBeamCharging: Consts_86 = 52099;
pub const addr_kEproj_MotherBrainDeathBeamFired: Consts_86 = 52085;
pub const addr_kEproj_MotherBrainDeathBeamCharging: Consts_86 = 52071;
pub const addr_kEproj_MotherBrainBomb: Consts_86 = 52057;
pub const addr_kEproj_MotherBrainBlueRingLasers: Consts_86 = 52043;
pub const addr_kEproj_MotherBrainPurpleBreathSmall: Consts_86 = 52029;
pub const addr_kEproj_MotherBrainPurpleBreathBig: Consts_86 = 52015;
pub const addr_kEproj_MotherBrainExplodedEscapeDoor: Consts_86 = 52001;
pub const addr_kEproj_MotherBrainDeathExplosion: Consts_86 = 51987;
pub const addr_off_86C8E1: Consts_86 = 51425;
pub const addr_kEproj_MotherBrainRoomTurretBullets: Consts_86 = 49548;
pub const addr_kEproj_MotherBrainRoomTurrets: Consts_86 = 49534;
pub const addr_kEproj_ShaktoolAttackBackCircle: Consts_86 = 48705;
pub const addr_kEproj_ShaktoolAttackMiddleCircle: Consts_86 = 48691;
pub const addr_kEproj_ShaktoolAttackFrontCircle: Consts_86 = 48677;
pub const addr_stru_86BD5A: Consts_86 = 48474;
pub const addr_word_86BBD5: Consts_86 = 48085;
pub const addr_kEproj_NuclearWaffleBody: Consts_86 = 48071;
pub const addr_kEproj_TourianStatueBaseDecoration: Consts_86 = 47806;
pub const addr_kEproj_TourianStatuePhantoon: Consts_86 = 47792;
pub const addr_kEproj_TourianStatueRidley: Consts_86 = 47778;
pub const addr_kEproj_TourianStatueSoul: Consts_86 = 47764;
pub const addr_kEproj_TourianStatueUnlockingParticleTail: Consts_86 = 47750;
pub const addr_kEproj_TourianStatueUnlockingParticle: Consts_86 = 47736;
pub const addr_kEproj_TourianStatueEyeGlow: Consts_86 = 47722;
pub const addr_stru_86BA5C: Consts_86 = 47708;
pub const addr_off_86B79F: Consts_86 = 47007;
pub const addr_kEproj_EyeDoorSweat: Consts_86 = 46929;
pub const addr_kEproj_EyeDoorProjectile: Consts_86 = 46915;
pub const addr_off_86B61D: Consts_86 = 46621;
pub const addr_off_86B5F3: Consts_86 = 46579;
pub const addr_kEproj_LavaSeahorseFireball: Consts_86 = 46539;
pub const addr_word_86B4E3: Consts_86 = 46307;
pub const addr_word_86B4D7: Consts_86 = 46295;
pub const addr_word_86B4CB: Consts_86 = 46283;
pub const addr_word_86B4BF: Consts_86 = 46271;
pub const addr_kEproj_GoldenTorizoEyeBeam: Consts_86 = 46120;
pub const addr_off_86B3E5: Consts_86 = 46053;
pub const addr_off_86B3CD: Consts_86 = 46029;
pub const addr_stru_86B380: Consts_86 = 45952;
pub const addr_stru_86B376: Consts_86 = 45942;
pub const addr_kEproj_GoldenTorizoSuperMissile: Consts_86 = 45850;
pub const addr_off_86B2EF: Consts_86 = 45807;
pub const addr_stru_86B1C0: Consts_86 = 45504;
pub const addr_off_86B1A8: Consts_86 = 45480;
pub const addr_off_86B190: Consts_86 = 45456;
pub const addr_off_86B166: Consts_86 = 45414;
pub const addr_off_86B14B: Consts_86 = 45387;
pub const addr_kEprojInit_GoldenTorizoEgg1: Consts_86 = 45113;
pub const addr_kEprojInit_GoldenTorizoEgg0: Consts_86 = 45103;
pub const addr_stru_86AFF3: Consts_86 = 45043;
pub const addr_kEproj_TourianLandingDustCloudsRightFoot: Consts_86 = 45029;
pub const addr_kEproj_TourianStatueDustClouds: Consts_86 = 44932;
pub const addr_kEproj_WreckedShipChozoSpikeFootsteps_1: Consts_86 = 44904;
pub const addr_kEproj_GoldenTorizoSonicBoom: Consts_86 = 44726;
pub const addr_kEproj_BombTorizoSonicBoom: Consts_86 = 44712;
pub const addr_off_86ADE5: Consts_86 = 44517;
pub const addr_off_86ADD2: Consts_86 = 44498;
pub const addr_off_86ADBF: Consts_86 = 44479;
pub const addr_kEproj_GoldenTorizosChozoOrbs: Consts_86 = 44410;
pub const addr_kEproj_BombTorizosChozoOrbs: Consts_86 = 44382;
pub const addr_kEprojInit_GoldenTorizosChozoOrbs_init1: Consts_86 = 44195;
pub const addr_kEprojInit_GoldenTorizosChozoOrbs_init0: Consts_86 = 44185;
pub const addr_kEprojInit_BombTorizosChozoOrbs_init1: Consts_86 = 44050;
pub const addr_kEprojInit_BombTorizosChozoOrbs_init0: Consts_86 = 44040;
pub const addr_off_86AB41: Consts_86 = 43841;
pub const addr_off_86AB25: Consts_86 = 43813;
pub const addr_kEprojInit_AB07_Tile1: Consts_86 = 43517;
pub const addr_kEprojInit_AB07_Tile0: Consts_86 = 43453;
pub const addr_kEproj_BombTorizoDeathExplosion: Consts_86 = 43439;
pub const addr_kEproj_BombTorizoLowHealthExplode: Consts_86 = 43425;
pub const addr_kEproj_BombTorizoExplosiveSwipe: Consts_86 = 43397;
pub const addr_kEproj_BombTorizoLowHealthInitialDrool: Consts_86 = 43369;
pub const addr_stru_86A95B: Consts_86 = 43355;
pub const addr_kEproj_PrePhantomRoom: Consts_86 = 41904;
pub const addr_kEproj_CeresElevatorPlatform: Consts_86 = 41877;
pub const addr_kEproj_CeresElevatorPad: Consts_86 = 41863;
pub const addr_stru_86A379: Consts_86 = 41849;
pub const addr_off_86A28B: Consts_86 = 41611;
pub const addr_kEproj_PirateClaw: Consts_86 = 41353;
pub const addr_stru_86A17B: Consts_86 = 41339;
pub const addr_off_869FE1: Consts_86 = 40929;
pub const addr_off_869FB9: Consts_86 = 40889;
pub const addr_word_869F7D: Consts_86 = 40829;
pub const addr_word_869F41: Consts_86 = 40769;
pub const addr_kEproj_MiniKraidSpikesRight: Consts_86 = 40396;
pub const addr_kEproj_MiniKraidSpikesLeft: Consts_86 = 40382;
pub const addr_kEproj_MiniKraidSpit: Consts_86 = 40368;
pub const addr_kEproj_RocksWhenKraidRisesRight: Consts_86 = 40047;
pub const addr_kEproj_RocksWhenKraidRisesLeft: Consts_86 = 40033;
pub const addr_kEproj_RocksFallingKraidCeiling: Consts_86 = 40019;
pub const addr_kEproj_RocksKraidSpits: Consts_86 = 40005;
pub const addr_kEproj_StartingFireballs: Consts_86 = 39991;
pub const addr_kEproj_DestroyableFireballs: Consts_86 = 39977;
pub const addr_off_8697F8: Consts_86 = 38904;
pub const addr_off_8697B4: Consts_86 = 38836;
pub const addr_word_8697AC: Consts_86 = 38828;
pub const addr_word_869782: Consts_86 = 38786;
pub const addr_word_869772: Consts_86 = 38770;
pub const addr_word_86976C: Consts_86 = 38764;
pub const addr_stru_869742: Consts_86 = 38722;
pub const addr_stru_869734: Consts_86 = 38708;
pub const addr_stru_869696: Consts_86 = 38550;
pub const addr_stru_869688: Consts_86 = 38536;
pub const addr_stru_86967A: Consts_86 = 38522;
pub const addr_stru_86966C: Consts_86 = 38508;
pub const addr_stru_869650: Consts_86 = 38480;
pub const addr_stru_869634: Consts_86 = 38452;
pub const addr_off_869574: Consts_86 = 38260;
pub const addr_kRidleysFireball_Tab0: Consts_86 = 37896;
pub const addr_kEproj_CrocomireSpikeWallPieces: Consts_86 = 37057;
pub const addr_stru_868F9D: Consts_86 = 36765;
pub const addr_stru_868F8F: Consts_86 = 36751;
pub const addr_word_868EDF: Consts_86 = 36575;
pub const addr_stru_868E5E: Consts_86 = 36446;
pub const addr_stru_868E50: Consts_86 = 36432;
pub const addr_stru_868C24: Consts_86 = 35876;
pub const addr_stru_868C16: Consts_86 = 35862;
pub const addr_stru_868C08: Consts_86 = 35848;
pub const addr_stru_868BFA: Consts_86 = 35834;
pub const addr_stru_868BEC: Consts_86 = 35820;
pub const addr_stru_868BDE: Consts_86 = 35806;
pub const addr_stru_868BD0: Consts_86 = 35792;
pub const addr_stru_868BC2: Consts_86 = 35778;
pub use crate::types::Consts_8D;
pub const addr_kPalfx_FFD5: Consts_8D = 65493;
pub const addr_kPalfx_FFD1: Consts_8D = 65489;
pub const addr_kPalfx_FFCD: Consts_8D = 65485;
pub const addr_kPalfx_FFC9: Consts_8D = 65481;
pub const addr_stru_8DF75D: Consts_8D = 63325;
pub const addr_stru_8DF759: Consts_8D = 63321;
pub const addr_word_8DEC76: Consts_8D = 60534;
pub const addr_word_8DEB43: Consts_8D = 60227;
pub const addr_off_8DE8B6: Consts_8D = 59574;
pub const addr_off_8DE68A: Consts_8D = 59018;
pub const addr_kPalfxInstrList_E45E: Consts_8D = 58462;
pub const addr_off_8DE420: Consts_8D = 58400;
pub const addr_off_8DE400: Consts_8D = 58368;
pub const addr_off_8DE3E0: Consts_8D = 58336;
pub const addr_kPalfx_E200: Consts_8D = 57856;
pub const addr_stru_8DE1FC: Consts_8D = 57852;
pub const addr_stru_8DE1F8: Consts_8D = 57848;
pub const addr_stru_8DE1F4: Consts_8D = 57844;
pub const addr_stru_8DE1F0: Consts_8D = 57840;
pub const addr_kPalfx_E1E8: Consts_8D = 57832;
pub const addr_kPalfx_E1E4: Consts_8D = 57828;
pub const addr_kPalfx_E1E0: Consts_8D = 57824;
pub const addr_kPalfx_E1DC: Consts_8D = 57820;
pub const addr_kPalfx_E1D8: Consts_8D = 57816;
pub const addr_kPalfx_E1D4: Consts_8D = 57812;
pub const addr_kPalfx_E1D0: Consts_8D = 57808;
pub const addr_kPalfx_E1CC: Consts_8D = 57804;
pub const addr_kPalfx_E1C8: Consts_8D = 57800;
pub const addr_kPalfx_E1C4: Consts_8D = 57796;
pub const addr_stru_8DE1C0: Consts_8D = 57792;
pub const addr_kPalfx_E1BC: Consts_8D = 57788;
pub const addr_kPalfx_E1B8: Consts_8D = 57784;
pub const addr_kPalfx_E1B0: Consts_8D = 57776;
pub const addr_kPalfx_E1AC: Consts_8D = 57772;
pub const addr_kPalfx_E1A8: Consts_8D = 57768;
pub const addr_kPalfx_E1A4: Consts_8D = 57764;
pub const addr_kPalfx_E1A0: Consts_8D = 57760;
pub const addr_kPalfx_E19C: Consts_8D = 57756;
pub const addr_kPalfx_E198: Consts_8D = 57752;
pub const addr_kPalfx_E194: Consts_8D = 57748;
pub const addr_off_8DE192: Consts_8D = 57746;
pub const addr_locret_8DC5E3: Consts_8D = 50659;
pub use crate::types::Consts_A2;
pub const addr_kRisingFallingPlatform_Ilist_EDE7: Consts_A2 = 60903;
pub const addr_kTimedShutter_Ilist_E9D4: Consts_A2 = 59860;
pub const addr_kTimedShutter_Ilist_E9AA: Consts_A2 = 59818;
pub const addr_kTimedShutter_Ilist_E9A4: Consts_A2 = 59812;
pub const addr_kTimedShutter_Ilist_E99E: Consts_A2 = 59806;
pub const addr_kTimedShutter_Ilist_E998: Consts_A2 = 59800;
pub const addr_kLavaSeahorse_Ilist_E5A1: Consts_A2 = 58785;
pub const addr_kLavaSeahorse_Ilist_E59B: Consts_A2 = 58779;
pub const addr_kRipper_Ilist_E48B: Consts_A2 = 58507;
pub const addr_kRipper_Ilist_E477: Consts_A2 = 58487;
pub const addr_kGripper_Sprmap_E44B: Consts_A2 = 58443;
pub const addr_kGripper_Sprmap_E43F: Consts_A2 = 58431;
pub const addr_kRipper_Ilist_E2F4: Consts_A2 = 58100;
pub const addr_kRipper_Ilist_E2E0: Consts_A2 = 58080;
pub const addr_kGripper_Ilist_E1AF: Consts_A2 = 57775;
pub const addr_kGripper_Ilist_E19B: Consts_A2 = 57755;
pub const addr_kHirisingSlowfalling_Ilist_D840: Consts_A2 = 55360;
pub const addr_kHirisingSlowfalling_Ilist_D834: Consts_A2 = 55348;
pub const addr_kHirisingSlowfalling_Ilist_D82C: Consts_A2 = 55340;
pub const addr_kMaridiaLargeSnail_Ilist_CA4B: Consts_A2 = 51787;
pub const addr_kLowerNorfairRio_Ilist_C6B0: Consts_A2 = 50864;
pub const addr_kLowerNorfairRio_Ilist_C686: Consts_A2 = 50822;
pub const addr_kLowerNorfairRio_Ilist_C674: Consts_A2 = 50804;
pub const addr_kLowerNorfairRio_Ilist_C662: Consts_A2 = 50786;
pub const addr_kLowerNorfairRio_Ilist_C65A: Consts_A2 = 50778;
pub const addr_kLowerNorfairRio_Ilist_C630: Consts_A2 = 50736;
pub const addr_kLowerNorfairRio_Ilist_C61A: Consts_A2 = 50714;
pub const addr_kNorfairRio_Ilist_C1A3: Consts_A2 = 49571;
pub const addr_kNorfairRio_Ilist_C18F: Consts_A2 = 49551;
pub const addr_kNorfairRio_Ilist_C179: Consts_A2 = 49529;
pub const addr_kNorfairRio_Ilist_C145: Consts_A2 = 49477;
pub const addr_kNorfairRio_Ilist_C12F: Consts_A2 = 49455;
pub const addr_kNorfairRio_Ilist_C107: Consts_A2 = 49415;
pub const addr_kNorfairRio_Ilist_C0F1: Consts_A2 = 49393;
pub const addr_kNorfairLavajumpingEnemy_Ilist_BE62: Consts_A2 = 48738;
pub const addr_kNorfairLavajumpingEnemy_Ilist_BE42: Consts_A2 = 48706;
pub const addr_kNorfairLavajumpingEnemy_Ilist_BE3C: Consts_A2 = 48700;
pub const addr_kRio_Ilist_BBA3: Consts_A2 = 48035;
pub const addr_kRio_Ilist_BB97: Consts_A2 = 48023;
pub const addr_kRio_Ilist_BB7F: Consts_A2 = 47999;
pub const addr_stru_A2BB53: Consts_A2 = 47955;
pub const addr_kRio_Ilist_BB4B: Consts_A2 = 47947;
pub const addr_kRinka_Ilist_BA0C: Consts_A2 = 47628;
pub const addr_kRinka_Ilist_B9E0: Consts_A2 = 47584;
pub const addr_kNorfairErraticFireball_Ilist_B51A: Consts_A2 = 46362;
pub const addr_kNorfairErraticFireball_Ilist_B2DC: Consts_A2 = 45788;
pub const addr_kFlies_Ilist_B013: Consts_A2 = 45075;
pub const addr_kGunshipTop_Ilist_A61C: Consts_A2 = 42524;
pub const addr_kGunshipTop_Ilist_A616: Consts_A2 = 42518;
pub const addr_kGunshipTop_Ilist_A60E: Consts_A2 = 42510;
pub const addr_kGunshipTop_Ilist_A5EE: Consts_A2 = 42478;
pub const addr_kGunshipTop_Ilist_A5BE: Consts_A2 = 42430;
pub const addr_kMaridiaSpikeyShell_Ilist_A3BD: Consts_A2 = 41917;
pub const addr_kMaridiaSpikeyShell_Ilist_A3AB: Consts_A2 = 41899;
pub const addr_kSpikeShootingPlant_Ilist_9F00: Consts_A2 = 40704;
pub const addr_kSpikeShootingPlant_Ilist_9EDA: Consts_A2 = 40666;
pub const addr_kSpikeShootingPlant_Ilist_9EB0: Consts_A2 = 40624;
pub const addr_kSpikeShootingPlant_Ilist_9E8A: Consts_A2 = 40586;
pub const addr_kThinHoppingBlobs_Ilist_9A01: Consts_A2 = 39425;
pub const addr_kThinHoppingBlobs_Ilist_99FB: Consts_A2 = 39419;
pub const addr_kThinHoppingBlobs_Ilist_99F5: Consts_A2 = 39413;
pub const addr_kThinHoppingBlobs_Ilist_99EF: Consts_A2 = 39407;
pub const addr_kThinHoppingBlobs_Ilist_99E9: Consts_A2 = 39401;
pub const addr_kThinHoppingBlobs_Ilist_99D5: Consts_A2 = 39381;
pub const addr_kThinHoppingBlobs_Ilist_99C1: Consts_A2 = 39361;
pub const addr_kThinHoppingBlobs_Ilist_99AD: Consts_A2 = 39341;
pub const addr_locret_A28E09: Consts_A2 = 36361;
pub const addr_kMaridiaBeybladeTurtle_Ilist_8D40: Consts_A2 = 36160;
pub const addr_kMaridiaBeybladeTurtle_Ilist_8D28: Consts_A2 = 36136;
pub const addr_kMaridiaBeybladeTurtle_Ilist_8D14: Consts_A2 = 36116;
pub const addr_kMaridiaBeybladeTurtle_Ilist_8D00: Consts_A2 = 36096;
pub const addr_kMaridiaBeybladeTurtle_Ilist_8C72: Consts_A2 = 35954;
pub const addr_kMaridiaBeybladeTurtle_Ilist_8C62: Consts_A2 = 35938;
pub const addr_kMaridiaBeybladeTurtle_Ilist_8C4A: Consts_A2 = 35914;
pub const addr_kMaridiaBeybladeTurtle_Ilist_8C44: Consts_A2 = 35908;
pub const addr_kMaridiaBeybladeTurtle_Ilist_8C30: Consts_A2 = 35888;
pub const addr_kMaridiaBeybladeTurtle_Ilist_8C1C: Consts_A2 = 35868;
pub const addr_kMaridiaBeybladeTurtle_Ilist_8C02: Consts_A2 = 35842;
pub const addr_kMaridiaBeybladeTurtle_Ilist_8BD2: Consts_A2 = 35794;
pub const addr_kMaridiaBeybladeTurtle_Ilist_8B80: Consts_A2 = 35712;
pub const addr_kMiniCrocomire_Ilist_896E: Consts_A2 = 35182;
pub const addr_kMiniCrocomire_Ilist_8958: Consts_A2 = 35160;
pub const addr_kMiniCrocomire_Ilist_8948: Consts_A2 = 35144;
pub const addr_kMiniCrocomire_Ilist_8932: Consts_A2 = 35122;
pub const addr_kBouncingGoofball_Ilist_86BF: Consts_A2 = 34495;
pub const addr_kBouncingGoofball_Ilist_86A7: Consts_A2 = 34471;
pub const addr_kSpritemap_Nothing_A2: Consts_A2 = 32845;
pub use crate::types::Point32;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_BouncingGoofball {
    pub base: EnemyBase,
    pub bgl_var_A: uint16,
    pub bgl_var_B: uint16,
    pub bgl_var_C: uint16,
    pub bgl_var_D: uint16,
    pub bgl_var_E: uint16,
    pub bgl_var_F: uint16,
    pub bgl_parameter_1: uint16,
    pub bgl_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub bgl_var_00: uint16,
    pub bgl_var_01: uint16,
    pub bgl_var_02: uint16,
    pub bgl_var_03: uint16,
    pub bgl_var_04: uint16,
    pub bgl_var_05: uint16,
    pub bgl_var_06: uint16,
    pub bgl_var_07: uint16,
    pub bgl_var_08: uint16,
    pub bgl_var_09: uint16,
    pub bgl_var_0A: uint16,
    pub bgl_var_0B: uint16,
    pub bgl_var_0C: uint16,
    pub bgl_var_0D: uint16,
    pub bgl_var_0E: uint16,
    pub bgl_var_0F: uint16,
    pub bgl_var_10: uint16,
    pub bgl_var_11: uint16,
    pub bgl_var_12: uint16,
    pub bgl_var_13: uint16,
    pub bgl_var_14: uint16,
    pub bgl_var_15: uint16,
    pub bgl_var_16: uint16,
    pub bgl_var_17: uint16,
    pub bgl_var_18: uint16,
    pub bgl_var_19: uint16,
    pub bgl_var_1A: uint16,
    pub bgl_var_1B: uint16,
    pub bgl_var_1C: uint16,
    pub bgl_var_1D: uint16,
    pub bgl_var_1E: uint16,
    pub bgl_var_1F: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_Rio {
    pub base: EnemyBase,
    pub rio_var_A: uint16,
    pub rio_var_B: uint16,
    pub rio_var_C: uint16,
    pub rio_var_D: uint16,
    pub rio_var_E: uint16,
    pub rio_var_F: uint16,
    pub rio_parameter_1: uint16,
    pub rio_parameter_2: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_Flies {
    pub base: EnemyBase,
    pub flies_var_A: uint16,
    pub flies_var_B: uint16,
    pub flies_var_C: uint16,
    pub flies_var_D: uint16,
    pub flies_var_E: uint16,
    pub flies_var_F: uint16,
    pub flies_parameter_1: uint16,
    pub flies_parameter_2: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_LavaSeahorse {
    pub base: EnemyBase,
    pub lse_var_A: uint16,
    pub lse_var_B: uint16,
    pub lse_var_C: uint16,
    pub lse_var_D: uint16,
    pub lse_var_E: uint16,
    pub lse_var_F: uint16,
    pub lse_parameter_1: uint16,
    pub lse_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub lse_var_00: uint16,
    pub lse_var_01: uint16,
    pub lse_var_02: uint16,
    pub lse_var_03: uint16,
    pub lse_var_04: uint16,
    pub lse_var_05: uint16,
    pub lse_var_06: uint16,
    pub lse_var_07: uint16,
    pub lse_var_08: uint16,
    pub lse_var_09: uint16,
    pub lse_var_0A: uint16,
    pub lse_var_0B: uint16,
    pub lse_var_0C: uint16,
    pub lse_var_0D: uint16,
    pub lse_var_0E: uint16,
    pub lse_var_0F: uint16,
    pub lse_var_10: uint16,
    pub lse_var_11: uint16,
    pub lse_var_12: uint16,
    pub lse_var_13: uint16,
    pub lse_var_14: uint16,
    pub lse_var_15: uint16,
    pub lse_var_16: uint16,
    pub lse_var_17: uint16,
    pub lse_var_18: uint16,
    pub lse_var_19: uint16,
    pub lse_var_1A: uint16,
    pub lse_var_1B: uint16,
    pub lse_var_1C: uint16,
    pub lse_var_1D: uint16,
    pub lse_var_1E: uint16,
    pub lse_var_1F: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_NorfairRio {
    pub base: EnemyBase,
    pub nro_var_A: uint16,
    pub nro_var_B: uint16,
    pub nro_var_C: uint16,
    pub nro_var_D: uint16,
    pub nro_var_E: uint16,
    pub nro_var_F: uint16,
    pub nro_parameter_1: uint16,
    pub nro_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub nro_var_00: uint16,
    pub nro_var_01: uint16,
    pub nro_var_02: uint16,
    pub nro_var_03: uint16,
    pub nro_var_04: uint16,
    pub nro_var_05: uint16,
    pub nro_var_06: uint16,
    pub nro_var_07: uint16,
    pub nro_var_08: uint16,
    pub nro_var_09: uint16,
    pub nro_var_0A: uint16,
    pub nro_var_0B: uint16,
    pub nro_var_0C: uint16,
    pub nro_var_0D: uint16,
    pub nro_var_0E: uint16,
    pub nro_var_0F: uint16,
    pub nro_var_10: uint16,
    pub nro_var_11: uint16,
    pub nro_var_12: uint16,
    pub nro_var_13: uint16,
    pub nro_var_14: uint16,
    pub nro_var_15: uint16,
    pub nro_var_16: uint16,
    pub nro_var_17: uint16,
    pub nro_var_18: uint16,
    pub nro_var_19: uint16,
    pub nro_var_1A: uint16,
    pub nro_var_1B: uint16,
    pub nro_var_1C: uint16,
    pub nro_var_1D: uint16,
    pub nro_var_1E: uint16,
    pub nro_var_1F: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_MaridiaBeybladeTurtle {
    pub base: EnemyBase,
    pub mbte_var_A: uint16,
    pub mbte_var_B: uint16,
    pub mbte_var_C: uint16,
    pub mbte_var_D: uint16,
    pub mbte_var_E: uint16,
    pub mbte_var_F: uint16,
    pub mbte_parameter_1: uint16,
    pub mbte_parameter_2: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_MaridiaTurtle {
    pub base: EnemyBase,
    pub mte_var_A: uint16,
    pub mte_var_B: uint16,
    pub mte_var_C: uint16,
    pub mte_var_D: uint16,
    pub mte_var_E: uint16,
    pub mte_var_F: uint16,
    pub mte_parameter_1: uint16,
    pub mte_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub mte_var_00: uint16,
    pub mte_var_01: uint16,
    pub mte_var_02: uint16,
    pub mte_var_03: uint16,
    pub mte_var_04: uint16,
    pub mte_var_05: uint16,
    pub mte_var_06: uint16,
    pub mte_var_07: uint16,
    pub mte_var_08: uint16,
    pub mte_var_09: uint16,
    pub mte_var_0A: uint16,
    pub mte_var_0B: uint16,
    pub mte_var_0C: uint16,
    pub mte_var_0D: uint16,
    pub mte_var_0E: uint16,
    pub mte_var_0F: uint16,
    pub mte_var_10: uint16,
    pub mte_var_11: uint16,
    pub mte_var_12: uint16,
    pub mte_var_13: uint16,
    pub mte_var_14: uint16,
    pub mte_var_15: uint16,
    pub mte_var_16: uint16,
    pub mte_var_17: uint16,
    pub mte_var_18: uint16,
    pub mte_var_19: uint16,
    pub mte_var_1A: uint16,
    pub mte_var_1B: uint16,
    pub mte_var_1C: uint16,
    pub mte_var_1D: uint16,
    pub mte_var_1E: uint16,
    pub mte_var_1F: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_MaridiaLargeSnail {
    pub base: EnemyBase,
    pub mlsl_var_A: uint16,
    pub mlsl_var_B: uint16,
    pub mlsl_var_C: uint16,
    pub mlsl_var_D: uint16,
    pub mlsl_var_E: uint16,
    pub mlsl_var_F: uint16,
    pub mlsl_parameter_1: uint16,
    pub mlsl_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub mlsl_var_00: uint16,
    pub mlsl_var_01: uint16,
    pub mlsl_var_02: uint16,
    pub mlsl_var_03: uint16,
    pub mlsl_var_04: uint16,
    pub mlsl_var_05: uint16,
    pub mlsl_var_06: uint16,
    pub mlsl_var_07: uint16,
    pub mlsl_var_08: uint16,
    pub mlsl_var_09: uint16,
    pub mlsl_var_0A: uint16,
    pub mlsl_var_0B: uint16,
    pub mlsl_var_0C: uint16,
    pub mlsl_var_0D: uint16,
    pub mlsl_var_0E: uint16,
    pub mlsl_var_0F: uint16,
    pub mlsl_var_10: uint16,
    pub mlsl_var_11: uint16,
    pub mlsl_var_12: uint16,
    pub mlsl_var_13: uint16,
    pub mlsl_var_14: uint16,
    pub mlsl_var_15: uint16,
    pub mlsl_var_16: uint16,
    pub mlsl_var_17: uint16,
    pub mlsl_var_18: uint16,
    pub mlsl_var_19: uint16,
    pub mlsl_var_1A: uint16,
    pub mlsl_var_1B: uint16,
    pub mlsl_var_1C: uint16,
    pub mlsl_var_1D: uint16,
    pub mlsl_var_1E: uint16,
    pub mlsl_var_1F: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_MaridiaSpikeyShell {
    pub base: EnemyBase,
    pub mssl_var_A: uint16,
    pub mssl_var_B: uint16,
    pub mssl_var_C: uint16,
    pub mssl_var_D: uint16,
    pub mssl_var_E: uint16,
    pub mssl_var_F: uint16,
    pub mssl_parameter_1: uint16,
    pub mssl_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub mssl_var_00: uint16,
    pub mssl_var_01: uint16,
    pub mssl_var_02: uint16,
    pub mssl_var_03: uint16,
    pub mssl_var_04: uint16,
    pub mssl_var_05: uint16,
    pub mssl_var_06: uint16,
    pub mssl_var_07: uint16,
    pub mssl_var_08: uint16,
    pub mssl_var_09: uint16,
    pub mssl_var_0A: uint16,
    pub mssl_var_0B: uint16,
    pub mssl_var_0C: uint16,
    pub mssl_var_0D: uint16,
    pub mssl_var_0E: uint16,
    pub mssl_var_0F: uint16,
    pub mssl_var_10: uint16,
    pub mssl_var_11: uint16,
    pub mssl_var_12: uint16,
    pub mssl_var_13: uint16,
    pub mssl_var_14: uint16,
    pub mssl_var_15: uint16,
    pub mssl_var_16: uint16,
    pub mssl_var_17: uint16,
    pub mssl_var_18: uint16,
    pub mssl_var_19: uint16,
    pub mssl_var_1A: uint16,
    pub mssl_var_1B: uint16,
    pub mssl_var_1C: uint16,
    pub mssl_var_1D: uint16,
    pub mssl_var_1E: uint16,
    pub mssl_var_1F: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_MiniCrocomire {
    pub base: EnemyBase,
    pub mce_var_A: uint16,
    pub mce_var_B: uint16,
    pub mce_var_C: uint16,
    pub mce_var_D: uint16,
    pub mce_var_E: uint16,
    pub mce_var_F: uint16,
    pub mce_parameter_1: uint16,
    pub mce_parameter_2: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_NorfairLavajumpingEnemy {
    pub base: EnemyBase,
    pub nley_var_A: uint16,
    pub nley_var_B: uint16,
    pub nley_var_C: uint16,
    pub nley_var_D: uint16,
    pub nley_var_E: uint16,
    pub nley_var_F: uint16,
    pub nley_parameter_1: uint16,
    pub nley_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub nley_var_00: uint16,
    pub nley_var_01: uint16,
    pub nley_var_02: uint16,
    pub nley_var_03: uint16,
    pub nley_var_04: uint16,
    pub nley_var_05: uint16,
    pub nley_var_06: uint16,
    pub nley_var_07: uint16,
    pub nley_var_08: uint16,
    pub nley_var_09: uint16,
    pub nley_var_0A: uint16,
    pub nley_var_0B: uint16,
    pub nley_var_0C: uint16,
    pub nley_var_0D: uint16,
    pub nley_var_0E: uint16,
    pub nley_var_0F: uint16,
    pub nley_var_10: uint16,
    pub nley_var_11: uint16,
    pub nley_var_12: uint16,
    pub nley_var_13: uint16,
    pub nley_var_14: uint16,
    pub nley_var_15: uint16,
    pub nley_var_16: uint16,
    pub nley_var_17: uint16,
    pub nley_var_18: uint16,
    pub nley_var_19: uint16,
    pub nley_var_1A: uint16,
    pub nley_var_1B: uint16,
    pub nley_var_1C: uint16,
    pub nley_var_1D: uint16,
    pub nley_var_1E: uint16,
    pub nley_var_1F: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_Rinka {
    pub base: EnemyBase,
    pub rinka_var_A: uint16,
    pub rinka_var_B: uint16,
    pub rinka_var_C: uint16,
    pub rinka_var_D: uint16,
    pub rinka_var_E: uint16,
    pub rinka_var_F: uint16,
    pub rinka_parameter_1: uint16,
    pub rinka_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub rinka_var_00: uint16,
    pub rinka_var_01: uint16,
    pub rinka_var_02: uint16,
    pub rinka_var_03: uint16,
    pub rinka_var_04: uint16,
    pub rinka_var_05: uint16,
    pub rinka_var_06: uint16,
    pub rinka_var_07: uint16,
    pub rinka_var_08: uint16,
    pub rinka_var_09: uint16,
    pub rinka_var_0A: uint16,
    pub rinka_var_0B: uint16,
    pub rinka_var_0C: uint16,
    pub rinka_var_0D: uint16,
    pub rinka_var_0E: uint16,
    pub rinka_var_0F: uint16,
    pub rinka_var_10: uint16,
    pub rinka_var_11: uint16,
    pub rinka_var_12: uint16,
    pub rinka_var_13: uint16,
    pub rinka_var_14: uint16,
    pub rinka_var_15: uint16,
    pub rinka_var_16: uint16,
    pub rinka_var_17: uint16,
    pub rinka_var_18: uint16,
    pub rinka_var_19: uint16,
    pub rinka_var_1A: uint16,
    pub rinka_var_1B: uint16,
    pub rinka_var_1C: uint16,
    pub rinka_var_1D: uint16,
    pub rinka_var_1E: uint16,
    pub rinka_var_1F: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_SpikeShootingPlant {
    pub base: EnemyBase,
    pub sspt_var_A: uint16,
    pub sspt_var_B: uint16,
    pub sspt_var_C: uint16,
    pub sspt_var_D: uint16,
    pub sspt_var_E: uint16,
    pub sspt_var_F: uint16,
    pub sspt_parameter_1: uint16,
    pub sspt_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub sspt_var_00: uint16,
    pub sspt_var_01: uint16,
    pub sspt_var_02: uint16,
    pub sspt_var_03: uint16,
    pub sspt_var_04: uint16,
    pub sspt_var_05: uint16,
    pub sspt_var_06: uint16,
    pub sspt_var_07: uint16,
    pub sspt_var_08: uint16,
    pub sspt_var_09: uint16,
    pub sspt_var_0A: uint16,
    pub sspt_var_0B: uint16,
    pub sspt_var_0C: uint16,
    pub sspt_var_0D: uint16,
    pub sspt_var_0E: uint16,
    pub sspt_var_0F: uint16,
    pub sspt_var_10: uint16,
    pub sspt_var_11: uint16,
    pub sspt_var_12: uint16,
    pub sspt_var_13: uint16,
    pub sspt_var_14: uint16,
    pub sspt_var_15: uint16,
    pub sspt_var_16: uint16,
    pub sspt_var_17: uint16,
    pub sspt_var_18: uint16,
    pub sspt_var_19: uint16,
    pub sspt_var_1A: uint16,
    pub sspt_var_1B: uint16,
    pub sspt_var_1C: uint16,
    pub sspt_var_1D: uint16,
    pub sspt_var_1E: uint16,
    pub sspt_var_1F: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_Gripper {
    pub base: EnemyBase,
    pub gripper_var_A: uint16,
    pub gripper_var_B: uint16,
    pub gripper_var_C: uint16,
    pub gripper_var_D: uint16,
    pub gripper_var_E: uint16,
    pub gripper_var_F: uint16,
    pub gripper_parameter_1: uint16,
    pub gripper_parameter_2: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_GunshipBottom {
    pub base: EnemyBase,
    pub gbm_var_A: uint16,
    pub gbm_var_B: uint16,
    pub gbm_var_C: uint16,
    pub gbm_var_D: uint16,
    pub gbm_var_E: uint16,
    pub gbm_var_F: uint16,
    pub gbm_parameter_1: uint16,
    pub gbm_parameter_2: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_GunshipTop {
    pub base: EnemyBase,
    pub gtp_var_A: uint16,
    pub gtp_var_B: uint16,
    pub gtp_var_C: uint16,
    pub gtp_var_D: uint16,
    pub gtp_var_E: uint16,
    pub gtp_var_F: uint16,
    pub gtp_parameter_1: uint16,
    pub gtp_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub gtp_var_00: uint16,
    pub gtp_var_01: uint16,
    pub gtp_var_02: uint16,
    pub gtp_var_03: uint16,
    pub gtp_var_04: uint16,
    pub gtp_var_05: uint16,
    pub gtp_var_06: uint16,
    pub gtp_var_07: uint16,
    pub gtp_var_08: uint16,
    pub gtp_var_09: uint16,
    pub gtp_var_0A: uint16,
    pub gtp_var_0B: uint16,
    pub gtp_var_0C: uint16,
    pub gtp_var_0D: uint16,
    pub gtp_var_0E: uint16,
    pub gtp_var_0F: uint16,
    pub gtp_var_10: uint16,
    pub gtp_var_11: uint16,
    pub gtp_var_12: uint16,
    pub gtp_var_13: uint16,
    pub gtp_var_14: uint16,
    pub gtp_var_15: uint16,
    pub gtp_var_16: uint16,
    pub gtp_var_17: uint16,
    pub gtp_var_18: uint16,
    pub gtp_var_19: uint16,
    pub gtp_var_1A: uint16,
    pub gtp_var_1B: uint16,
    pub gtp_var_1C: uint16,
    pub gtp_var_1D: uint16,
    pub gtp_var_1E: uint16,
    pub gtp_var_1F: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_HirisingSlowfalling {
    pub base: EnemyBase,
    pub hsg_var_A: uint16,
    pub hsg_var_B: uint16,
    pub hsg_var_C: uint16,
    pub hsg_var_D: uint16,
    pub hsg_var_E: uint16,
    pub hsg_var_F: uint16,
    pub hsg_parameter_1: uint16,
    pub hsg_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub hsg_var_00: uint16,
    pub hsg_var_01: uint16,
    pub hsg_var_02: uint16,
    pub hsg_var_03: uint16,
    pub hsg_var_04: uint16,
    pub hsg_var_05: uint16,
    pub hsg_var_06: uint16,
    pub hsg_var_07: uint16,
    pub hsg_var_08: uint16,
    pub hsg_var_09: uint16,
    pub hsg_var_0A: uint16,
    pub hsg_var_0B: uint16,
    pub hsg_var_0C: uint16,
    pub hsg_var_0D: uint16,
    pub hsg_var_0E: uint16,
    pub hsg_var_0F: uint16,
    pub hsg_var_10: uint16,
    pub hsg_var_11: uint16,
    pub hsg_var_12: uint16,
    pub hsg_var_13: uint16,
    pub hsg_var_14: uint16,
    pub hsg_var_15: uint16,
    pub hsg_var_16: uint16,
    pub hsg_var_17: uint16,
    pub hsg_var_18: uint16,
    pub hsg_var_19: uint16,
    pub hsg_var_1A: uint16,
    pub hsg_var_1B: uint16,
    pub hsg_var_1C: uint16,
    pub hsg_var_1D: uint16,
    pub hsg_var_1E: uint16,
    pub hsg_var_1F: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_RisingFallingPlatform {
    pub base: EnemyBase,
    pub rfpm_var_A: uint16,
    pub rfpm_var_B: uint16,
    pub rfpm_var_C: uint16,
    pub rfpm_var_D: uint16,
    pub rfpm_var_E: uint16,
    pub rfpm_var_F: uint16,
    pub rfpm_parameter_1: uint16,
    pub rfpm_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub rfpm_var_00: uint16,
    pub rfpm_var_01: uint16,
    pub rfpm_var_02: uint16,
    pub rfpm_var_03: uint16,
    pub rfpm_var_04: uint16,
    pub rfpm_var_05: uint16,
    pub rfpm_var_06: uint16,
    pub rfpm_var_07: uint16,
    pub rfpm_var_08: uint16,
    pub rfpm_var_09: uint16,
    pub rfpm_var_0A: uint16,
    pub rfpm_var_0B: uint16,
    pub rfpm_var_0C: uint16,
    pub rfpm_var_0D: uint16,
    pub rfpm_var_0E: uint16,
    pub rfpm_var_0F: uint16,
    pub rfpm_var_10: uint16,
    pub rfpm_var_11: uint16,
    pub rfpm_var_12: uint16,
    pub rfpm_var_13: uint16,
    pub rfpm_var_14: uint16,
    pub rfpm_var_15: uint16,
    pub rfpm_var_16: uint16,
    pub rfpm_var_17: uint16,
    pub rfpm_var_18: uint16,
    pub rfpm_var_19: uint16,
    pub rfpm_var_1A: uint16,
    pub rfpm_var_1B: uint16,
    pub rfpm_var_1C: uint16,
    pub rfpm_var_1D: uint16,
    pub rfpm_var_1E: uint16,
    pub rfpm_var_1F: uint16,
    pub pad2: [uint8; 1984],
    pub rfpm_var_20: uint16,
    pub rfpm_var_21: uint16,
    pub rfpm_var_22: uint16,
    pub rfpm_var_23: uint16,
    pub rfpm_var_24: uint16,
    pub rfpm_var_25: uint16,
    pub rfpm_var_26: uint16,
    pub rfpm_var_27: uint16,
    pub rfpm_var_28: uint16,
    pub rfpm_var_29: uint16,
    pub rfpm_var_2A: uint16,
    pub rfpm_var_2B: uint16,
    pub rfpm_var_2C: uint16,
    pub rfpm_var_2D: uint16,
    pub rfpm_var_2E: uint16,
    pub rfpm_var_2F: uint16,
    pub rfpm_var_30: uint16,
    pub rfpm_var_31: uint16,
    pub rfpm_var_32: uint16,
    pub rfpm_var_33: uint16,
    pub rfpm_var_34: uint16,
    pub rfpm_var_35: uint16,
    pub rfpm_var_36: uint16,
    pub rfpm_var_37: uint16,
    pub rfpm_var_38: uint16,
    pub rfpm_var_39: uint16,
    pub rfpm_var_3A: uint16,
    pub rfpm_var_3B: uint16,
    pub rfpm_var_3C: uint16,
    pub rfpm_var_3D: uint16,
    pub rfpm_var_3E: uint16,
    pub rfpm_var_3F: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_HorizontalShootableShutter {
    pub base: EnemyBase,
    pub hssr_var_A: uint16,
    pub hssr_var_B: uint16,
    pub hssr_var_C: uint16,
    pub hssr_var_D: uint16,
    pub hssr_var_E: uint16,
    pub hssr_var_F: uint16,
    pub hssr_parameter_1: uint16,
    pub hssr_parameter_2: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_JetPowerRipper {
    pub base: EnemyBase,
    pub jprr_var_A: uint16,
    pub jprr_var_B: uint16,
    pub jprr_var_C: uint16,
    pub jprr_var_D: uint16,
    pub jprr_var_E: uint16,
    pub jprr_var_F: uint16,
    pub jprr_parameter_1: uint16,
    pub jprr_parameter_2: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_LavaquakeRocks {
    pub base: EnemyBase,
    pub lrs_var_A: uint16,
    pub lrs_var_B: uint16,
    pub lrs_var_C: uint16,
    pub lrs_var_D: uint16,
    pub lrs_var_E: uint16,
    pub lrs_var_F: uint16,
    pub lrs_parameter_1: uint16,
    pub lrs_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub lrs_var_00: uint16,
    pub lrs_var_01: uint16,
    pub lrs_var_02: uint16,
    pub lrs_var_03: uint16,
    pub lrs_var_04: uint16,
    pub lrs_var_05: uint16,
    pub lrs_var_06: uint16,
    pub lrs_var_07: uint16,
    pub lrs_var_08: uint16,
    pub lrs_var_09: uint16,
    pub lrs_var_0A: uint16,
    pub lrs_var_0B: uint16,
    pub lrs_var_0C: uint16,
    pub lrs_var_0D: uint16,
    pub lrs_var_0E: uint16,
    pub lrs_var_0F: uint16,
    pub lrs_var_10: uint16,
    pub lrs_var_11: uint16,
    pub lrs_var_12: uint16,
    pub lrs_var_13: uint16,
    pub lrs_var_14: uint16,
    pub lrs_var_15: uint16,
    pub lrs_var_16: uint16,
    pub lrs_var_17: uint16,
    pub lrs_var_18: uint16,
    pub lrs_var_19: uint16,
    pub lrs_var_1A: uint16,
    pub lrs_var_1B: uint16,
    pub lrs_var_1C: uint16,
    pub lrs_var_1D: uint16,
    pub lrs_var_1E: uint16,
    pub lrs_var_1F: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_LowerNorfairRio {
    pub base: EnemyBase,
    pub lnro_var_A: uint16,
    pub lnro_var_B: uint16,
    pub lnro_var_C: uint16,
    pub lnro_var_D: uint16,
    pub lnro_var_E: uint16,
    pub lnro_var_F: uint16,
    pub lnro_parameter_1: uint16,
    pub lnro_parameter_2: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_MiniMaridiaTurtle {
    pub base: EnemyBase,
    pub mmte_var_A: uint16,
    pub mmte_var_B: uint16,
    pub mmte_var_C: uint16,
    pub mmte_var_D: uint16,
    pub mmte_var_E: uint16,
    pub mmte_var_F: uint16,
    pub mmte_parameter_1: uint16,
    pub mmte_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub mmte_var_00: uint16,
    pub mmte_var_01: uint16,
    pub mmte_var_02: uint16,
    pub mmte_var_03: uint16,
    pub mmte_var_04: uint16,
    pub mmte_var_05: uint16,
    pub mmte_var_06: uint16,
    pub mmte_var_07: uint16,
    pub mmte_var_08: uint16,
    pub mmte_var_09: uint16,
    pub mmte_var_0A: uint16,
    pub mmte_var_0B: uint16,
    pub mmte_var_0C: uint16,
    pub mmte_var_0D: uint16,
    pub mmte_var_0E: uint16,
    pub mmte_var_0F: uint16,
    pub mmte_var_10: uint16,
    pub mmte_var_11: uint16,
    pub mmte_var_12: uint16,
    pub mmte_var_13: uint16,
    pub mmte_var_14: uint16,
    pub mmte_var_15: uint16,
    pub mmte_var_16: uint16,
    pub mmte_var_17: uint16,
    pub mmte_var_18: uint16,
    pub mmte_var_19: uint16,
    pub mmte_var_1A: uint16,
    pub mmte_var_1B: uint16,
    pub mmte_var_1C: uint16,
    pub mmte_var_1D: uint16,
    pub mmte_var_1E: uint16,
    pub mmte_var_1F: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_MiniMaridiaBeybladeTurtle {
    pub base: EnemyBase,
    pub mmbte_var_A: uint16,
    pub mmbte_var_B: uint16,
    pub mmbte_var_C: uint16,
    pub mmbte_var_D: uint16,
    pub mmbte_var_E: uint16,
    pub mmbte_var_F: uint16,
    pub mmbte_parameter_1: uint16,
    pub mmbte_parameter_2: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_NorfairErraticFireball {
    pub base: EnemyBase,
    pub nefl_var_A: uint16,
    pub nefl_var_B: uint16,
    pub nefl_var_C: uint16,
    pub nefl_var_D: uint16,
    pub nefl_var_E: uint16,
    pub nefl_var_F: uint16,
    pub nefl_parameter_1: uint16,
    pub nefl_parameter_2: uint16,
}
pub use crate::types::Ram8000_Default;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_Ripper {
    pub base: EnemyBase,
    pub ripper_var_A: uint16,
    pub ripper_var_B: uint16,
    pub ripper_var_C: uint16,
    pub ripper_var_D: uint16,
    pub ripper_var_E: uint16,
    pub ripper_var_F: uint16,
    pub ripper_parameter_1: uint16,
    pub ripper_parameter_2: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_ThinHoppingBlobs {
    pub base: EnemyBase,
    pub thbs_var_A: uint16,
    pub thbs_var_B: uint16,
    pub thbs_var_C: uint16,
    pub thbs_var_D: uint16,
    pub thbs_var_E: uint16,
    pub thbs_var_F: uint16,
    pub thbs_parameter_1: uint16,
    pub thbs_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub thbs_var_00: uint16,
    pub thbs_var_01: uint16,
    pub thbs_var_02: uint16,
    pub thbs_var_03: uint16,
    pub thbs_var_04: uint16,
    pub thbs_var_05: uint16,
    pub thbs_var_06: uint16,
    pub thbs_var_07: uint16,
    pub thbs_var_08: uint16,
    pub thbs_var_09: uint16,
    pub thbs_var_0A: uint16,
    pub thbs_var_0B: uint16,
    pub thbs_var_0C: uint16,
    pub thbs_var_0D: uint16,
    pub thbs_var_0E: uint16,
    pub thbs_var_0F: uint16,
    pub thbs_var_10: uint16,
    pub thbs_var_11: uint16,
    pub thbs_var_12: uint16,
    pub thbs_var_13: uint16,
    pub thbs_var_14: uint16,
    pub thbs_var_15: uint16,
    pub thbs_var_16: uint16,
    pub thbs_var_17: uint16,
    pub thbs_var_18: uint16,
    pub thbs_var_19: uint16,
    pub thbs_var_1A: uint16,
    pub thbs_var_1B: uint16,
    pub thbs_var_1C: uint16,
    pub thbs_var_1D: uint16,
    pub thbs_var_1E: uint16,
    pub thbs_var_1F: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Enemy_TimedShutter {
    pub base: EnemyBase,
    pub tsr_var_A: uint16,
    pub tsr_var_B: uint16,
    pub tsr_var_C: uint16,
    pub tsr_var_D: uint16,
    pub tsr_var_E: uint16,
    pub tsr_var_F: uint16,
    pub tsr_parameter_1: uint16,
    pub tsr_parameter_2: uint16,
    pub pad1: [uint8; 26696],
    pub tsr_var_00: uint16,
    pub tsr_var_01: uint16,
    pub tsr_var_02: uint16,
    pub tsr_var_03: uint16,
    pub tsr_var_04: uint16,
    pub tsr_var_05: uint16,
    pub tsr_var_06: uint16,
    pub tsr_var_07: uint16,
    pub tsr_var_08: uint16,
    pub tsr_var_09: uint16,
    pub tsr_var_0A: uint16,
    pub tsr_var_0B: uint16,
    pub tsr_var_0C: uint16,
    pub tsr_var_0D: uint16,
    pub tsr_var_0E: uint16,
    pub tsr_var_0F: uint16,
    pub tsr_var_10: uint16,
    pub tsr_var_11: uint16,
    pub tsr_var_12: uint16,
    pub tsr_var_13: uint16,
    pub tsr_var_14: uint16,
    pub tsr_var_15: uint16,
    pub tsr_var_16: uint16,
    pub tsr_var_17: uint16,
    pub tsr_var_18: uint16,
    pub tsr_var_19: uint16,
    pub tsr_var_1A: uint16,
    pub tsr_var_1B: uint16,
    pub tsr_var_1C: uint16,
    pub tsr_var_1D: uint16,
    pub tsr_var_1E: uint16,
    pub tsr_var_1F: uint16,
    pub pad3: [uint8; 4032],
    pub tsr_var_40: uint16,
    pub tsr_var_41: uint16,
    pub tsr_var_42: uint16,
    pub tsr_var_43: uint16,
    pub tsr_var_44: uint16,
    pub tsr_var_45: uint16,
    pub tsr_var_46: uint16,
    pub tsr_var_47: uint16,
    pub tsr_var_48: uint16,
    pub tsr_var_49: uint16,
    pub tsr_var_4A: uint16,
    pub tsr_var_4B: uint16,
    pub tsr_var_4C: uint16,
    pub tsr_var_4D: uint16,
    pub tsr_var_4E: uint16,
    pub tsr_var_4F: uint16,
    pub tsr_var_50: uint16,
    pub tsr_var_51: uint16,
    pub tsr_var_52: uint16,
    pub tsr_var_53: uint16,
    pub tsr_var_54: uint16,
    pub tsr_var_55: uint16,
    pub tsr_var_56: uint16,
    pub tsr_var_57: uint16,
    pub tsr_var_58: uint16,
    pub tsr_var_59: uint16,
    pub tsr_var_5A: uint16,
    pub tsr_var_5B: uint16,
    pub tsr_var_5C: uint16,
    pub tsr_var_5D: uint16,
    pub tsr_var_5E: uint16,
    pub tsr_var_5F: uint16,
}
#[inline]
unsafe fn abs16(mut t: uint16) -> uint16 {
    return (if t as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        -(t as ::core::ffi::c_int)
    } else {
        t as ::core::ffi::c_int
    }) as uint16;
}
#[inline]
unsafe fn swap16(mut v: uint16) -> uint16 {
    return ((v as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
        | v as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as uint16;
}
#[inline]
unsafe fn RomPtr_A2(mut addr: uint16_t, game: &mut Game) -> *const uint8 {
    return RomPtr(
        (0xa20000 as ::core::ffi::c_int | addr as ::core::ffi::c_int) as uint32_t,
        game,
    );
}
#[inline]
unsafe fn gVramWriteEntry(mut a: ::core::ffi::c_int, game: &mut Game) -> *mut VramWriteEntry {
    return (&raw mut game.g_ram as *mut uint8).offset((0xd0 as ::core::ffi::c_int + a) as isize)
        as *mut uint8 as *mut VramWriteEntry;
}
pub const fnSamus_FrameHandlerAlfa_Func11: ::core::ffi::c_int = 0x90e695;
pub const fnSamus_FrameHandlerBeta_Func17: ::core::ffi::c_int = 0x90e725;
pub const fnMiniCrocomire_PreInstr5: ::core::ffi::c_int = 0xa28a43;
pub const fnMiniCrocomire_PreInstr6: ::core::ffi::c_int = 0xa28a5c;
pub const fnnullsub_175: ::core::ffi::c_int = 0xa28a75;
pub const fnMaridiaBeybladeTurtle_Func1: ::core::ffi::c_int = 0xa28dd8;
pub const fnnullsub_360: uint32 = 10653193;
pub const fnMaridiaBeybladeTurtle_Func2: ::core::ffi::c_int = 0xa28e0a;
pub const fnMaridiaBeybladeTurtle_Func3: ::core::ffi::c_int = 0xa28ee0;
pub const fnMaridiaBeybladeTurtle_Func5: ::core::ffi::c_int = 0xa28f3f;
pub const fnMaridiaBeybladeTurtle_Func7: ::core::ffi::c_int = 0xa28f8d;
pub const fnMaridiaBeybladeTurtle_Func8: ::core::ffi::c_int = 0xa28feb;
pub const fnMaridiaBeybladeTurtle_Func9: ::core::ffi::c_int = 0xa29083;
pub const fnMaridiaBeybladeTurtle_Func10: ::core::ffi::c_int = 0xa290cc;
pub const fnMaridiaBeybladeTurtle_Func11: ::core::ffi::c_int = 0xa290e1;
pub const fnMiniMaridiaBeybladeTurtle_Func1: ::core::ffi::c_int = 0xa29142;
pub const fnMiniMaridiaBeybladeTurtle_Func2: ::core::ffi::c_int = 0xa2916e;
pub const fnMiniMaridiaBeybladeTurtle_Func3: ::core::ffi::c_int = 0xa29198;
pub const fnMiniMaridiaBeybladeTurtle_Func4: ::core::ffi::c_int = 0xa291f8;
pub const fnMiniMaridiaBeybladeTurtle_Func6: ::core::ffi::c_int = 0xa29239;
pub const fnMiniMaridiaBeybladeTurtle_Func7: ::core::ffi::c_int = 0xa2925e;
pub const fnThinHoppingBlobs_Func6: ::core::ffi::c_int = 0xa29b65;
pub const fnThinHoppingBlobs_Func7: ::core::ffi::c_int = 0xa29b81;
pub const fnThinHoppingBlobs_Func11: uint32 = 10657035;
pub const fnThinHoppingBlobs_Func12: uint32 = 10657067;
pub const fnThinHoppingBlobs_Func13: uint32 = 10657099;
pub const fnThinHoppingBlobs_Func14: uint32 = 10657131;
pub const fnThinHoppingBlobs_Func15: ::core::ffi::c_int = 0xa29d98;
pub const fnThinHoppingBlobs_Func16: uint32 = 10657229;
pub const fnSpikeShootingPlant_2: ::core::ffi::c_int = 0xa29fba;
pub const fnSpikeShootingPlant_3: ::core::ffi::c_int = 0xa29fec;
pub const fnnullsub_182: ::core::ffi::c_int = 0xa2a01b;
pub const fnnullsub_187: ::core::ffi::c_int = 0xa2a7d7;
pub const fnGunshipTop_3: ::core::ffi::c_int = 0xa2a80c;
pub const fnGunshipTop_4: ::core::ffi::c_int = 0xa2a8d0;
pub const fnGunshipTop_5: ::core::ffi::c_int = 0xa2a942;
pub const fnGunshipTop_6: ::core::ffi::c_int = 0xa2a950;
pub const fnGunshipTop_7: ::core::ffi::c_int = 0xa2a987;
pub const fnGunshipTop_8: ::core::ffi::c_int = 0xa2a9bd;
pub const fnGunshipTop_9: ::core::ffi::c_int = 0xa2aa4f;
pub const fnGunshipTop_10: ::core::ffi::c_int = 0xa2aa5d;
pub const fnGunshipTop_11: ::core::ffi::c_int = 0xa2aa94;
pub const fnGunshipTop_12: ::core::ffi::c_int = 0xa2aaa2;
pub const fnGunshipTop_13: ::core::ffi::c_int = 0xa2ab1f;
pub const fnGunshipTop_14: ::core::ffi::c_int = 0xa2ab60;
pub const fnGunshipTop_15: ::core::ffi::c_int = 0xa2ab6e;
pub const fnGunshipTop_16: ::core::ffi::c_int = 0xa2aba5;
pub const fnGunshipTop_17: ::core::ffi::c_int = 0xa2abc7;
pub const fnGunshipTop_18: ::core::ffi::c_int = 0xa2ac1b;
pub const fnGunshipTop_19: ::core::ffi::c_int = 0xa2acd7;
pub const fnGunshipTop_20: ::core::ffi::c_int = 0xa2ad0e;
pub const fnGunshipTop_21: ::core::ffi::c_int = 0xa2ad2d;
pub const fnFlies_4: ::core::ffi::c_int = 0xa2b14e;
pub const fnFlies_5: ::core::ffi::c_int = 0xa2b17c;
pub const fnFlies_6: ::core::ffi::c_int = 0xa2b1aa;
pub const fnFlies_7: ::core::ffi::c_int = 0xa2b1d2;
pub const fnLavaquakeRocks_1: ::core::ffi::c_int = 0xa2b596;
pub const fnLavaquakeRocks_2: ::core::ffi::c_int = 0xa2b5b2;
pub const fnLavaquakeRocks_3: ::core::ffi::c_int = 0xa2b5ea;
pub const fnRinka_3: ::core::ffi::c_int = 0xa2b7df;
pub const fnRinka_4: ::core::ffi::c_int = 0xa2b844;
pub const fnRinka_5: ::core::ffi::c_int = 0xa2b852;
pub const fnRinka_B85B: ::core::ffi::c_int = 0xa2b85b;
pub const fnRio_1: ::core::ffi::c_int = 0xa2bbed;
pub const fnRio_2: ::core::ffi::c_int = 0xa2bc32;
pub const fnRio_3: ::core::ffi::c_int = 0xa2bc48;
pub const fnRio_4: ::core::ffi::c_int = 0xa2bcb7;
pub const fnRio_5: ::core::ffi::c_int = 0xa2bcff;
pub const fnNorfairLavajumpingEnemy_Func_1: ::core::ffi::c_int = 0xa2bedc;
pub const fnNorfairLavajumpingEnemy_Func_2: ::core::ffi::c_int = 0xa2bf1a;
pub const fnNorfairLavajumpingEnemy_Func_3: ::core::ffi::c_int = 0xa2bf3e;
pub const fnNorfairLavajumpingEnemy_Func_4: ::core::ffi::c_int = 0xa2bf7c;
pub const fnNorfairLavajumpingEnemy_Func_5: ::core::ffi::c_int = 0xa2bfbc;
pub const fnNorfairRio_Func_1: ::core::ffi::c_int = 0xa2c281;
pub const fnNorfairRio_Func_2: ::core::ffi::c_int = 0xa2c2e7;
pub const fnNorfairRio_Func_3: ::core::ffi::c_int = 0xa2c33f;
pub const fnNorfairRio_Func_4: ::core::ffi::c_int = 0xa2c361;
pub const fnNorfairRio_Func_5: ::core::ffi::c_int = 0xa2c3b1;
pub const fnNorfairRio_Func_6: ::core::ffi::c_int = 0xa2c406;
pub const fnLowerNorfairRio_Func_1: ::core::ffi::c_int = 0xa2c72e;
pub const fnLowerNorfairRio_Func_2: ::core::ffi::c_int = 0xa2c771;
pub const fnLowerNorfairRio_Func_3: ::core::ffi::c_int = 0xa2c7bb;
pub const fnLowerNorfairRio_Func_4: ::core::ffi::c_int = 0xa2c7d6;
pub const fnLowerNorfairRio_Func_5: ::core::ffi::c_int = 0xa2c82d;
pub const fnLowerNorfairRio_Func_6: ::core::ffi::c_int = 0xa2c888;
pub const fnMaridiaLargeSnail_Func_4: ::core::ffi::c_int = 0xa2cde6;
pub const fnMaridiaLargeSnail_Func_5: ::core::ffi::c_int = 0xa2ce2b;
pub const fnMaridiaLargeSnail_Func_6: ::core::ffi::c_int = 0xa2cf40;
pub const fnMaridiaLargeSnail_Func_7: ::core::ffi::c_int = 0xa2cf66;
pub const fnMaridiaLargeSnail_Func_8: ::core::ffi::c_int = 0xa2cfa9;
pub const fnHirisingSlowfalling_Func_4: ::core::ffi::c_int = 0xa2e035;
pub const fnHirisingSlowfalling_Func_5: ::core::ffi::c_int = 0xa2e04f;
pub const fnHirisingSlowfalling_Func_6: ::core::ffi::c_int = 0xa2e06a;
pub const fnHirisingSlowfalling_Func_7: ::core::ffi::c_int = 0xa2e0cd;
pub const fnLavaSeahorse_Func_1: ::core::ffi::c_int = 0xa2e654;
pub const fnLavaSeahorse_Func_2: ::core::ffi::c_int = 0xa2e6ad;
pub const fnLavaSeahorse_Func_3: ::core::ffi::c_int = 0xa2e6f1;
pub const fnLavaSeahorse_Func_4: ::core::ffi::c_int = 0xa2e734;
pub const fnLavaSeahorse_Func_5: ::core::ffi::c_int = 0xa2e749;
pub const fnnullsub_196: ::core::ffi::c_int = 0xa2e781;
pub const fnTimedShutter_Func_1: uint32 = 10676925;
pub const fnTimedShutter_Func_2: uint32 = 10676945;
pub const fnTimedShutter_Func_3: uint32 = 10676967;
pub const fnTimedShutter_Func_4: uint32 = 10676989;
pub const fnTimedShutter_Func_5: ::core::ffi::c_int = 0xa2eb11;
pub const fnTimedShutter_Func_10: ::core::ffi::c_int = 0xa2ec13;
pub const fnRisingFallingPlatform_Func_2: ::core::ffi::c_int = 0xa2ef09;
pub const fnRisingFallingPlatform_Func_4: ::core::ffi::c_int = 0xa2ef28;
pub const fnRisingFallingPlatform_Func_9: ::core::ffi::c_int = 0xa2ef68;
pub const fnRisingFallingPlatform_Func_10: ::core::ffi::c_int = 0xa2efd4;
pub const fnRisingFallingPlatform_Func_11: ::core::ffi::c_int = 0xa2f040;
pub const fnRisingFallingPlatform_Func_12: ::core::ffi::c_int = 0xa2f072;
pub const fnRisingFallingPlatform_Func_13: ::core::ffi::c_int = 0xa2f099;
pub const fnHorizontalShootableShutter_Func_2: ::core::ffi::c_int = 0xa2f224;
pub const fnHorizontalShootableShutter_Func_4: ::core::ffi::c_int = 0xa2f243;
pub const fnHorizontalShootableShutter_Func_8: ::core::ffi::c_int = 0xa2f272;
pub const fnHorizontalShootableShutter_Func_9: ::core::ffi::c_int = 0xa2f2e4;
pub const fnHorizontalShootableShutter_Func_12: ::core::ffi::c_int = 0xa2f38c;
pub const fnHorizontalShootableShutter_Func_13: ::core::ffi::c_int = 0xa2f3b0;
pub const fnHorizontalShootableShutter_Func_14: ::core::ffi::c_int = 0xa2f3d4;
#[inline]
unsafe fn gEnemyData(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut EnemyData {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut EnemyData;
}
#[inline]
unsafe fn gEnemySpawnData(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut EnemySpawnData {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0x7000)
        .offset(o as isize) as *mut EnemySpawnData;
}
#[inline]
unsafe fn gRam8000_Default(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Ram8000_Default {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0x8000)
        .offset(o as isize) as *mut Ram8000_Default;
}
#[inline]
unsafe fn Get_Flies(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_Flies {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_Flies;
}
#[inline]
unsafe fn Get_Gripper(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_Gripper {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_Gripper;
}
#[inline]
unsafe fn Get_GunshipBottom(
    mut o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_GunshipBottom {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_GunshipBottom;
}
#[inline]
unsafe fn Get_GunshipTop(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_GunshipTop {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_GunshipTop;
}
#[inline]
unsafe fn Get_HirisingSlowfalling(
    mut o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_HirisingSlowfalling {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_HirisingSlowfalling;
}
#[inline]
unsafe fn Get_HorizontalShootableShutter(
    mut o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_HorizontalShootableShutter {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_HorizontalShootableShutter;
}
#[inline]
unsafe fn Get_JetPowerRipper(
    mut o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_JetPowerRipper {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_JetPowerRipper;
}
#[inline]
unsafe fn Get_LavaSeahorse(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_LavaSeahorse {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_LavaSeahorse;
}
#[inline]
unsafe fn Get_LavaquakeRocks(
    mut o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_LavaquakeRocks {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_LavaquakeRocks;
}
#[inline]
unsafe fn Get_LowerNorfairRio(
    mut o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_LowerNorfairRio {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_LowerNorfairRio;
}
#[inline]
unsafe fn Get_MaridiaBeybladeTurtle(
    mut o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_MaridiaBeybladeTurtle {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_MaridiaBeybladeTurtle;
}
#[inline]
unsafe fn Get_MaridiaLargeSnail(
    mut o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_MaridiaLargeSnail {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_MaridiaLargeSnail;
}
#[inline]
unsafe fn Get_MaridiaSpikeyShell(
    mut o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_MaridiaSpikeyShell {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_MaridiaSpikeyShell;
}
#[inline]
unsafe fn Get_MaridiaTurtle(
    mut o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_MaridiaTurtle {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_MaridiaTurtle;
}
#[inline]
unsafe fn Get_MiniMaridiaBeybladeTurtle(
    mut o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_MiniMaridiaBeybladeTurtle {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_MiniMaridiaBeybladeTurtle;
}
#[inline]
unsafe fn Get_MiniMaridiaTurtle(
    mut o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_MiniMaridiaTurtle {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_MiniMaridiaTurtle;
}
#[inline]
unsafe fn Get_NorfairErraticFireball(
    mut o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_NorfairErraticFireball {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_NorfairErraticFireball;
}
#[inline]
unsafe fn Get_NorfairLavajumpingEnemy(
    mut o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_NorfairLavajumpingEnemy {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_NorfairLavajumpingEnemy;
}
#[inline]
unsafe fn Get_NorfairRio(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_NorfairRio {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_NorfairRio;
}
#[inline]
unsafe fn Get_Rinka(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_Rinka {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_Rinka;
}
#[inline]
unsafe fn Get_Rio(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_Rio {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_Rio;
}
#[inline]
unsafe fn Get_Ripper(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_Ripper {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_Ripper;
}
#[inline]
unsafe fn Get_RisingFallingPlatform(
    mut o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_RisingFallingPlatform {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_RisingFallingPlatform;
}
#[inline]
unsafe fn Get_SpikeShootingPlant(
    mut o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_SpikeShootingPlant {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_SpikeShootingPlant;
}
#[inline]
unsafe fn Get_ThinHoppingBlobs(
    mut o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_ThinHoppingBlobs {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_ThinHoppingBlobs;
}
#[inline]
unsafe fn Get_TimedShutter(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_TimedShutter {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_TimedShutter;
}
#[inline]
unsafe fn AddToHiLo(mut hi: *mut uint16, mut lo: *mut uint16, mut v: uint32) {
    let mut t: uint32 = ((*lo as ::core::ffi::c_int
        + ((*hi as ::core::ffi::c_int) << 16 as ::core::ffi::c_int))
        as uint32)
        .wrapping_add(v);
    *lo = t as uint16;
    *hi = (t >> 16 as ::core::ffi::c_int) as uint16;
}
#[inline]
unsafe fn SetHiLo(mut hi: *mut uint16, mut lo: *mut uint16, mut t: uint32) {
    *lo = t as uint16;
    *hi = (t >> 16 as ::core::ffi::c_int) as uint16;
}
static g_word_A28D50: uint16 = 0x30 as uint16;
static g_word_A28D52: uint16 = 1 as uint16;
static g_word_A28D5E: uint16 = 3 as uint16;
static g_word_A28D60: uint16 = 0x1e8 as uint16;
static g_word_A28D62: uint16 = 7 as uint16;
static g_word_A28D64: uint16 = 0x1e as uint16;
static g_word_A28D66: uint16 = 4 as uint16;
static g_word_A28D68: uint16 = 0xfffd as uint16;
static g_word_A28D6A: uint16 = 3 as uint16;
static g_word_A28E80: [int16; 48] = [
    -(16 as ::core::ffi::c_int) as int16,
    -(16 as ::core::ffi::c_int) as int16,
    -(16 as ::core::ffi::c_int) as int16,
    -(16 as ::core::ffi::c_int) as int16,
    -(15 as ::core::ffi::c_int) as int16,
    -(15 as ::core::ffi::c_int) as int16,
    -(15 as ::core::ffi::c_int) as int16,
    -(15 as ::core::ffi::c_int) as int16,
    -(15 as ::core::ffi::c_int) as int16,
    -(14 as ::core::ffi::c_int) as int16,
    -(13 as ::core::ffi::c_int) as int16,
    -(13 as ::core::ffi::c_int) as int16,
    -(12 as ::core::ffi::c_int) as int16,
    -(11 as ::core::ffi::c_int) as int16,
    -(10 as ::core::ffi::c_int) as int16,
    -(9 as ::core::ffi::c_int) as int16,
    -(8 as ::core::ffi::c_int) as int16,
    -(7 as ::core::ffi::c_int) as int16,
    -(6 as ::core::ffi::c_int) as int16,
    -(5 as ::core::ffi::c_int) as int16,
    -(4 as ::core::ffi::c_int) as int16,
    -(4 as ::core::ffi::c_int) as int16,
    0 as ::core::ffi::c_int as int16,
    0 as ::core::ffi::c_int as int16,
    -(16 as ::core::ffi::c_int) as int16,
    -(16 as ::core::ffi::c_int) as int16,
    -(16 as ::core::ffi::c_int) as int16,
    -(15 as ::core::ffi::c_int) as int16,
    -(15 as ::core::ffi::c_int) as int16,
    -(15 as ::core::ffi::c_int) as int16,
    -(14 as ::core::ffi::c_int) as int16,
    -(13 as ::core::ffi::c_int) as int16,
    -(12 as ::core::ffi::c_int) as int16,
    -(11 as ::core::ffi::c_int) as int16,
    -(10 as ::core::ffi::c_int) as int16,
    -(9 as ::core::ffi::c_int) as int16,
    -(8 as ::core::ffi::c_int) as int16,
    -(7 as ::core::ffi::c_int) as int16,
    -(6 as ::core::ffi::c_int) as int16,
    -(5 as ::core::ffi::c_int) as int16,
    -(4 as ::core::ffi::c_int) as int16,
    -(3 as ::core::ffi::c_int) as int16,
    -(3 as ::core::ffi::c_int) as int16,
    -(2 as ::core::ffi::c_int) as int16,
    0 as ::core::ffi::c_int as int16,
    0 as ::core::ffi::c_int as int16,
    0 as ::core::ffi::c_int as int16,
    0 as ::core::ffi::c_int as int16,
];
pub unsafe fn Enemy_GrappleReact_CancelBeam_A2(game: &mut Game) {
    Enemy_SwitchToFrozenAi(game);
}
pub unsafe fn Enemy_GrappleReact_KillEnemy_A2(game: &mut Game) {
    EnemyGrappleDeath(game);
}
pub unsafe fn Enemy_GrappleReact_SamusLatchesOn_A2(game: &mut Game) {
    SamusLatchesOnWithGrapple(game);
}
pub unsafe fn Enemy_NormalFrozenAI_A2(game: &mut Game) {
    NormalEnemyFrozenAI(game);
}
pub unsafe fn Enemy_NormalPowerBombAI_A2(game: &mut Game) {
    NormalEnemyPowerBombAi(game);
}
pub unsafe fn Enemy_NormalTouchAI_A2(game: &mut Game) {
    NormalEnemyTouchAi(game);
}
pub unsafe fn Enemy_NormalShotAI_A2(game: &mut Game) {
    NormalEnemyShotAi(game);
}
pub fn BouncingGoofball_Init(game: &mut Game) {
    let index = game.cur_enemy_index();
    game.enemy_instructions(index)
        .set_spritemap_pointer(addr_kSpritemap_Nothing_A2 as u16);
    BouncingGoofball_Func2(game);
    GoofballState::new(game, index).set_state(1);
    let parameter = GoofballState::new(game, index).parameter_low();
    let scale = game.read_rom_word(0xa286df, usize::from(parameter) * 2);
    GoofballState::new(game, index).set_scale(scale);
    let parameter = GoofballState::new(game, index).parameter_high();
    let distance = game.read_rom_word(0xa286ef, usize::from(parameter) * 2);
    GoofballState::new(game, index).set_distance(distance);
    GoofballState::new(game, index).set_accumulated(0);
    GoofballState::new(game, index).set_phase(0);
    GoofballState::new(game, index).set_animation_trigger(0);
}
pub fn BouncingGoofball_Func1(k: uint16, game: &mut Game) {
    loop {
        let phase = GoofballState::new(game, k).phase();
        let product = goofball_movement_product(k, phase, game);
        GoofballState::new(game, k).set_product(product);
        let accumulated = GoofballState::new(game, k)
            .accumulated()
            .wrapping_add(product);
        GoofballState::new(game, k).set_accumulated(accumulated);
        let phase = GoofballState::new(game, k).phase().wrapping_add(1);
        GoofballState::new(game, k).set_phase(phase);
        if (accumulated.wrapping_sub(GoofballState::new(game, k).distance()) as i16) >= 0 {
            break;
        }
    }
    let phase = GoofballState::new(game, k).phase();
    GoofballState::new(game, k).set_limit(phase);
    GoofballState::new(game, k).set_active(1);
    GoofballState::new(game, k).set_animation_ready(1);
}
pub fn BouncingGoofball_Main(game: &mut Game) {
    let index = game.cur_enemy_index();
    if GoofballState::new(game, index).initialized() == 0 {
        BouncingGoofball_Func1(game.cur_enemy_index(), game);
        GoofballState::new(game, index).set_initialized(1);
        return;
    }
    let scale = GoofballState::new(game, index).scale();
    GoofballState::new(game, index).set_saved_scale(scale);
    if BouncingGoofball_SamusCloseX(game) != 0 {
        GoofballState::new(game, index).set_active(0);
        GoofballState::new(game, index).set_animation_ready(0);
        if GoofballState::new(game, index).animation_trigger() == 0 {
            GoofballState::new(game, index).set_animation_trigger(1);
            BouncingGoofball_Func3(game);
        }
    } else if GoofballState::new(game, index).active() != 0 {
        if GoofballState::new(game, index).animation_ready() == 0 {
            GoofballState::new(game, index).set_animation_ready(1);
            BouncingGoofball_Func2(game);
        }
        return;
    }
    let scale = GoofballState::new(game, index).saved_scale();
    GoofballState::new(game, index).set_scale(scale);
    match GoofballState::new(game, index).state() {
        0 => BouncingGoofball_State0(game),
        1 => BouncingGoofball_State1(game),
        _ => panic!("invalid Goofball state"),
    }
}
struct GoofballState<'a> {
    bytes: &'a mut [u8],
}
impl<'a> GoofballState<'a> {
    fn accumulated(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_BouncingGoofball, bgl_var_D))
    }
    fn distance(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_BouncingGoofball, bgl_var_C))
    }
    fn set_limit(&mut self, value: u16) {
        self.write(
            core::mem::offset_of!(Enemy_BouncingGoofball, bgl_var_00),
            value,
        );
    }
    fn initialized(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_BouncingGoofball, bgl_var_05))
    }
    fn set_initialized(&mut self, value: u16) {
        self.write(
            core::mem::offset_of!(Enemy_BouncingGoofball, bgl_var_05),
            value,
        );
    }
    fn saved_scale(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_BouncingGoofball, bgl_var_03))
    }
    fn set_saved_scale(&mut self, value: u16) {
        self.write(
            core::mem::offset_of!(Enemy_BouncingGoofball, bgl_var_03),
            value,
        );
    }
    fn animation_ready(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_BouncingGoofball, bgl_var_04))
    }
    fn set_animation_ready(&mut self, value: u16) {
        self.write(
            core::mem::offset_of!(Enemy_BouncingGoofball, bgl_var_04),
            value,
        );
    }
    fn active(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_BouncingGoofball, bgl_var_02))
    }
    fn animation_trigger(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_BouncingGoofball, bgl_var_F))
    }
    fn state(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_BouncingGoofball, bgl_var_01))
    }
    fn parameter_low(&self) -> u8 {
        self.bytes[core::mem::offset_of!(Enemy_BouncingGoofball, bgl_parameter_1)]
    }
    fn parameter_high(&self) -> u8 {
        self.bytes[core::mem::offset_of!(Enemy_BouncingGoofball, bgl_parameter_1) + 1]
    }
    fn set_scale(&mut self, value: u16) {
        self.write(
            core::mem::offset_of!(Enemy_BouncingGoofball, bgl_var_A),
            value,
        );
    }
    fn set_distance(&mut self, value: u16) {
        self.write(
            core::mem::offset_of!(Enemy_BouncingGoofball, bgl_var_C),
            value,
        );
    }
    fn set_accumulated(&mut self, value: u16) {
        self.write(
            core::mem::offset_of!(Enemy_BouncingGoofball, bgl_var_D),
            value,
        );
    }
    fn new(game: &'a mut Game, index: u16) -> Self {
        Self {
            bytes: &mut game.g_ram[0xf78 + usize::from(index)..],
        }
    }
    fn read(&self, offset: usize) -> u16 {
        u16::from_le_bytes([self.bytes[offset], self.bytes[offset + 1]])
    }
    fn write(&mut self, offset: usize, value: u16) {
        self.bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }
    fn phase(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_BouncingGoofball, bgl_var_E))
    }
    fn set_phase(&mut self, value: u16) {
        self.write(
            core::mem::offset_of!(Enemy_BouncingGoofball, bgl_var_E),
            value,
        );
    }
    fn scale(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_BouncingGoofball, bgl_var_A))
    }
    fn set_product(&mut self, value: u16) {
        self.write(
            core::mem::offset_of!(Enemy_BouncingGoofball, bgl_var_B),
            value,
        );
    }
    fn limit(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_BouncingGoofball, bgl_var_00))
    }
    fn set_state(&mut self, value: u16) {
        self.write(
            core::mem::offset_of!(Enemy_BouncingGoofball, bgl_var_01),
            value,
        );
    }
    fn set_active(&mut self, value: u16) {
        self.write(
            core::mem::offset_of!(Enemy_BouncingGoofball, bgl_var_02),
            value,
        );
    }
    fn set_animation_trigger(&mut self, value: u16) {
        self.write(
            core::mem::offset_of!(Enemy_BouncingGoofball, bgl_var_F),
            value,
        );
    }
    fn y_pos(&self) -> u16 {
        self.read(
            core::mem::offset_of!(Enemy_BouncingGoofball, base)
                + core::mem::offset_of!(EnemyBase, y_pos),
        )
    }
    fn set_y_pos(&mut self, value: u16) {
        self.write(
            core::mem::offset_of!(Enemy_BouncingGoofball, base)
                + core::mem::offset_of!(EnemyBase, y_pos),
            value,
        );
    }
}
fn goofball_movement_product(index: u16, phase: u16, game: &mut Game) -> u16 {
    // The ROM byte is read even when the signed gate replaces it with FF.
    let sample = game.read_rom_byte(0xa28701, usize::from(phase));
    let sample = if (phase.wrapping_sub(23) as i16) >= 0 {
        255
    } else {
        sample
    };
    u16::from(sample) * u16::from(GoofballState::new(game, index).scale() as u8)
}
pub fn BouncingGoofball_State0(game: &mut Game) {
    let index = game.cur_enemy_index();
    let phase = GoofballState::new(game, index).phase().wrapping_add(1);
    GoofballState::new(game, index).set_phase(phase);
    let phase = GoofballState::new(game, index).phase();
    let product = goofball_movement_product(index, phase, game);
    GoofballState::new(game, index).set_product(product);
    let y = GoofballState::new(game, index)
        .y_pos()
        .wrapping_add(product >> 8);
    GoofballState::new(game, index).set_y_pos(y);
    if (GoofballState::new(game, index)
        .phase()
        .wrapping_sub(GoofballState::new(game, index).limit()) as i16)
        >= 0
    {
        GoofballState::new(game, index).set_state(1);
        GoofballState::new(game, index).set_active(1);
        GoofballState::new(game, index).set_animation_trigger(0);
    }
}
pub fn BouncingGoofball_State1(game: &mut Game) {
    let index = game.cur_enemy_index();
    let phase = GoofballState::new(game, index).phase();
    let product = goofball_movement_product(index, phase, game);
    GoofballState::new(game, index).set_product(product);
    let y = GoofballState::new(game, index)
        .y_pos()
        .wrapping_sub(product >> 8);
    GoofballState::new(game, index).set_y_pos(y);
    let phase = GoofballState::new(game, index).phase().wrapping_sub(1);
    GoofballState::new(game, index).set_phase(phase);
    if GoofballState::new(game, index).phase() & 0x8000 != 0 {
        GoofballState::new(game, index).set_state(0);
    }
}
pub fn BouncingGoofball_SamusCloseX(game: &mut Game) -> uint16 {
    let slot = game.cur_enemy_index();
    let index = game.cur_enemy_index();
    let offset =
        0xf78 + usize::from(slot) + core::mem::offset_of!(Enemy_BouncingGoofball, bgl_parameter_2);
    let radius = u16::from_le_bytes([game.g_ram[offset], game.g_ram[offset + 1]]);
    IsSamusWithinEnemy_X(index, radius, game)
}
pub fn BouncingGoofball_Func2(game: &mut Game) {
    let index = game.cur_enemy_index();
    set_goofball_animation(index, addr_kBouncingGoofball_Ilist_86A7 as u16, game);
}
pub fn BouncingGoofball_Func3(game: &mut Game) {
    let index = game.cur_enemy_index();
    set_goofball_animation(index, addr_kBouncingGoofball_Ilist_86BF as u16, game);
}
fn set_goofball_animation(index: u16, instruction: u16, game: &mut Game) {
    game.enemy_instructions(index)
        .set_current_instruction(instruction);
    game.enemy_instructions(index).set_instruction_timer(1);
    game.enemy_instructions(index).set_timer(0);
}
pub fn BouncingGoofball_Instr_88C5(_k: uint16, jp: *const uint16) -> *const uint16 {
    jp
}
pub unsafe fn BouncingGoofball_Instr_88C6(
    _k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let index = game.cur_enemy_index();
    let offset =
        0xf78 + usize::from(index) + core::mem::offset_of!(Enemy_BouncingGoofball, bgl_var_02);
    game.g_ram[offset..offset + 2].copy_from_slice(&0u16.to_le_bytes());
    QueueSfx2_Max6(0xe, game);
    jp
}
pub unsafe fn MiniCrocomire_Instr_897E(
    _k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let parameter = *jp;
    let index = game.cur_enemy_index();
    SpawnEprojWithGfx(parameter, index, addr_Eproj_DBF2 as u16, game);
    jp.add(1)
}
pub fn MiniCrocomire_Instr_8990(_k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let index = game.cur_enemy_index();
    set_mini_crocomire_preinstruction(index, 0x8a43, 0, game);
    jp
}
pub fn MiniCrocomire_Instr_899D(_k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let index = game.cur_enemy_index();
    set_mini_crocomire_preinstruction(index, 0x8a5c, 1, game);
    jp
}
fn set_mini_crocomire_preinstruction(index: u16, preinstruction: u16, state: u16, game: &mut Game) {
    let base = 0xf78 + usize::from(index);
    let field = base + core::mem::offset_of!(Enemy_MiniCrocomire, mce_var_F);
    game.g_ram[field..field + 2].copy_from_slice(&preinstruction.to_le_bytes());
    let field = base + core::mem::offset_of!(Enemy_MiniCrocomire, mce_var_E);
    game.g_ram[field..field + 2].copy_from_slice(&state.to_le_bytes());
}

struct MiniCrocomireState<'a> {
    bytes: &'a mut [u8],
}
impl<'a> MiniCrocomireState<'a> {
    fn new(game: &'a mut Game, index: u16) -> Self {
        Self {
            bytes: &mut game.g_ram[0xf78 + usize::from(index)..],
        }
    }
    fn read(&self, offset: usize) -> u16 {
        u16::from_le_bytes([self.bytes[offset], self.bytes[offset + 1]])
    }
    fn write(&mut self, offset: usize, value: u16) {
        self.bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }
    fn parameter_1(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_MiniCrocomire, mce_parameter_1))
    }
    fn parameter_2(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_MiniCrocomire, mce_parameter_2))
    }
    fn var_a(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_MiniCrocomire, mce_var_A))
    }
    fn var_b(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_MiniCrocomire, mce_var_B))
    }
    fn var_c(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_MiniCrocomire, mce_var_C))
    }
    fn var_d(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_MiniCrocomire, mce_var_D))
    }
    fn state(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_MiniCrocomire, mce_var_E))
    }
    fn frame_counter_low(&self) -> u8 {
        self.bytes[core::mem::offset_of!(Enemy_MiniCrocomire, base)
            + core::mem::offset_of!(EnemyBase, frame_counter)]
    }
    fn set_var_b(&mut self, value: u16) {
        self.write(core::mem::offset_of!(Enemy_MiniCrocomire, mce_var_B), value);
    }
    fn set_var_a(&mut self, value: u16) {
        self.write(core::mem::offset_of!(Enemy_MiniCrocomire, mce_var_A), value);
    }
    fn set_var_d(&mut self, value: u16) {
        self.write(core::mem::offset_of!(Enemy_MiniCrocomire, mce_var_D), value);
    }
    fn set_var_c(&mut self, value: u16) {
        self.write(core::mem::offset_of!(Enemy_MiniCrocomire, mce_var_C), value);
    }
    fn preinstruction(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_MiniCrocomire, mce_var_F))
    }
    fn set_preinstruction(&mut self, value: u16) {
        self.write(core::mem::offset_of!(Enemy_MiniCrocomire, mce_var_F), value);
    }
    fn set_state(&mut self, value: u16) {
        self.write(core::mem::offset_of!(Enemy_MiniCrocomire, mce_var_E), value);
    }
}
pub fn MiniCrocomire_Init(game: &mut Game) {
    let index = game.cur_enemy_index();
    game.enemy_instructions(index)
        .set_spritemap_pointer(addr_kSpritemap_Nothing_A2 as u16);
    // Parameter indexing is wider than a SNES word; do not wrap this offset.
    let table_offset = usize::from(MiniCrocomireState::new(game, index).parameter_2()) * 8;
    let value = game.read_rom_word(0xa28187, table_offset);
    MiniCrocomireState::new(game, index).set_var_b(value);
    let value = game.read_rom_word(0xa28187, table_offset + 2);
    MiniCrocomireState::new(game, index).set_var_a(value);
    let value = game.read_rom_word(0xa28187, table_offset + 4);
    MiniCrocomireState::new(game, index).set_var_d(value);
    let value = game.read_rom_word(0xa28187, table_offset + 6);
    MiniCrocomireState::new(game, index).set_var_c(value);
    MiniCrocomire_Func1(game);
    MiniCrocomireState::new(game, index).set_preinstruction(0x8a43);
    let parameter = MiniCrocomireState::new(game, index).parameter_1();
    MiniCrocomireState::new(game, index).set_state(parameter);
    if parameter != 0 {
        MiniCrocomire_Func3(game);
        MiniCrocomireState::new(game, index).set_preinstruction(0x8a5c);
    }
}
pub unsafe fn MiniCrocomire_Main(game: &mut Game) {
    let index = game.cur_enemy_index();
    let preinstruction = MiniCrocomireState::new(game, index).preinstruction();
    CallEnemyPreInstr(u32::from(preinstruction) | 0xa20000, game);
}
pub fn MiniCrocomire_Func1(game: &mut Game) {
    let index = game.cur_enemy_index();
    set_mini_crocomire_animation(index, addr_kMiniCrocomire_Ilist_8932 as u16, game);
}
pub fn MiniCrocomire_Func2(game: &mut Game) {
    let index = game.cur_enemy_index();
    set_mini_crocomire_animation(index, addr_kMiniCrocomire_Ilist_8948 as u16, game);
}
pub fn MiniCrocomire_Func3(game: &mut Game) {
    let index = game.cur_enemy_index();
    set_mini_crocomire_animation(index, addr_kMiniCrocomire_Ilist_8958 as u16, game);
}
pub fn MiniCrocomire_Func4(game: &mut Game) {
    let index = game.cur_enemy_index();
    set_mini_crocomire_animation(index, addr_kMiniCrocomire_Ilist_896E as u16, game);
}
fn set_mini_crocomire_animation(index: u16, instruction: u16, game: &mut Game) {
    game.enemy_instructions(index).set_instruction_timer(1);
    game.enemy_instructions(index).set_timer(0);
    game.enemy_instructions(index)
        .set_current_instruction(instruction);
}
pub unsafe fn MiniCrocomire_PreInstr5(_k: uint16, game: &mut Game) {
    let index = game.cur_enemy_index();
    let high = MiniCrocomireState::new(game, index).var_d();
    let low = MiniCrocomireState::new(game, index).var_c();
    MiniCrocomire_Func7((u32::from(high) << 16 | u32::from(low)) as i32, game);
    if MiniCrocomire_Func9(game) != 0 {
        MiniCrocomire_Func2(game);
    }
}
pub unsafe fn MiniCrocomire_PreInstr6(_k: uint16, game: &mut Game) {
    let index = game.cur_enemy_index();
    let high = MiniCrocomireState::new(game, index).var_b();
    let low = MiniCrocomireState::new(game, index).var_a();
    MiniCrocomire_Func7((u32::from(high) << 16 | u32::from(low)) as i32, game);
    if MiniCrocomire_Func9(game) != 0 {
        MiniCrocomire_Func4(game);
    }
}
pub unsafe fn MiniCrocomire_Func7(amt: int32, game: &mut Game) {
    let index = game.cur_enemy_index();
    if Enemy_MoveRight_IgnoreSlopes(index, amt, game) == 0 {
        let index = game.cur_enemy_index();
        if EnemyFunc_BC76(index, 2 << 16, game) != 0 {
            return;
        }
    }
    MiniCrocomire_Func8(game);
}
pub fn MiniCrocomire_Func8(game: &mut Game) {
    MiniCrocomire_Func1(game);
    let index = game.cur_enemy_index();
    if MiniCrocomireState::new(game, index).state() != 1 {
        MiniCrocomire_Func3(game);
    }
}
pub unsafe fn MiniCrocomire_Func9(game: &mut Game) -> uint8 {
    NextRandom(game);
    let index = game.cur_enemy_index();
    let frame = MiniCrocomireState::new(game, index).frame_counter_low();
    if frame.wrapping_add(game.random_number() as u8) < 2 {
        MiniCrocomireState::new(game, index).set_preinstruction(0x8a75);
        return 1;
    }
    0
}
pub fn MaridiaBeybladeTurtle_Init(game: &mut Game) {
    let index = game.cur_enemy_index();
    let properties = game.enemy_instructions(index).properties();
    game.enemy_instructions(index)
        .set_properties(properties | kEnemyProps_DisableSamusColl as u16);
    game.enemy_instructions(index)
        .set_spritemap_pointer(addr_kSpritemap_Nothing_A2 as u16);
    game.enemy_instructions(index).set_instruction_timer(1);
    game.enemy_instructions(index).set_timer(0);
    game.enemy_coordinates(index).set_y_height(0);
    game.enemy_instructions(index)
        .set_current_instruction(addr_kMaridiaBeybladeTurtle_Ilist_8C44 as u16);
    let base = 0xf78 + usize::from(index);
    let field = base + core::mem::offset_of!(Enemy_MaridiaBeybladeTurtle, mbte_var_A);
    game.g_ram[field..field + 2].copy_from_slice(&0x8dd8u16.to_le_bytes());
    let field = base + core::mem::offset_of!(Enemy_MaridiaBeybladeTurtle, mbte_var_F);
    game.g_ram[field..field + 2].copy_from_slice(&g_word_A28D52.to_le_bytes());
}
pub fn MiniMaridiaBeybladeTurtle_Init(game: &mut Game) {
    let index = game.cur_enemy_index();
    let base = 0xf78 + usize::from(index);
    let x = game.enemy_coordinates(index).x_pos();
    let field = base + core::mem::offset_of!(Enemy_MiniMaridiaBeybladeTurtle, mmbte_var_C);
    game.g_ram[field..field + 2].copy_from_slice(&x.to_le_bytes());
    let y = game.enemy_coordinates(index).y_pos();
    let radius = game.enemy_coordinates(index).y_height();
    let field = base + core::mem::offset_of!(Enemy_MiniMaridiaBeybladeTurtle, mmbte_var_D);
    game.g_ram[field..field + 2].copy_from_slice(&y.wrapping_sub(radius).to_le_bytes());
    let field = base + core::mem::offset_of!(Enemy_MiniMaridiaBeybladeTurtle, mmbte_var_A);
    game.g_ram[field..field + 2].copy_from_slice(&0x9142u16.to_le_bytes());
    game.enemy_instructions(index).set_instruction_timer(1);
    game.enemy_instructions(index).set_timer(0);
    let field = base + core::mem::offset_of!(Enemy_MiniMaridiaBeybladeTurtle, mmbte_parameter_1);
    let parameter = u16::from_le_bytes([game.g_ram[field], game.g_ram[field + 1]]);
    let field = base + core::mem::offset_of!(Enemy_MiniMaridiaBeybladeTurtle, mmbte_var_E);
    game.g_ram[field..field + 2].copy_from_slice(&parameter.to_le_bytes());
    let instruction = if (parameter as i16) >= 0 {
        addr_kMaridiaBeybladeTurtle_Ilist_8C72 as u16
    } else {
        addr_kMaridiaBeybladeTurtle_Ilist_8B80 as u16
    };
    game.enemy_instructions(index)
        .set_current_instruction(instruction);
}
pub unsafe fn CallMaridiaBeybladeTurtleFunc(ea: uint32, k: uint16, game: &mut Game) {
    match ea {
        0xa28dd8 => MaridiaBeybladeTurtle_Func1(k, game),
        0xa28e0a => MaridiaBeybladeTurtle_Func2(k, game),
        0xa28ee0 => MaridiaBeybladeTurtle_Func3(k, game),
        0xa28f3f => MaridiaBeybladeTurtle_Func5(k, game),
        0xa28f8d => MaridiaBeybladeTurtle_Func7(k, game),
        0xa28feb => MaridiaBeybladeTurtle_Func8(k, game),
        0xa29083 => MaridiaBeybladeTurtle_Func9(k, game),
        0xa290cc => MaridiaBeybladeTurtle_Func10(k, game),
        0xa290e1 => MaridiaBeybladeTurtle_Func11(k, game),
        0xa28e09 => {}
        0xa29142 => MiniMaridiaBeybladeTurtle_Func1(k, game),
        0xa2916e => MiniMaridiaBeybladeTurtle_Func2(k, game),
        0xa29198 => MiniMaridiaBeybladeTurtle_Func3(k, game),
        0xa291f8 => MiniMaridiaBeybladeTurtle_Func4(k, game),
        0xa29239 => MiniMaridiaBeybladeTurtle_Func6(k, game),
        0xa2925e => MiniMaridiaBeybladeTurtle_Func7(game),
        _ => {
            Unreachable(game);
        }
    }
}

pub unsafe fn MaridiaBeybladeTurtle_Main(game: &mut Game) {
    let index = game.cur_enemy_index();
    let preinstruction_offset =
        0xf78 + usize::from(index) + core::mem::offset_of!(Enemy_MaridiaBeybladeTurtle, mbte_var_A);
    let preinstruction = u16::from_le_bytes([
        game.g_ram[preinstruction_offset],
        game.g_ram[preinstruction_offset + 1],
    ]);
    let live_index = game.cur_enemy_index();
    CallMaridiaBeybladeTurtleFunc(0xa20000 | u32::from(preinstruction), live_index, game);
}
pub fn MaridiaBeybladeTurtle_Func1(k: uint16, game: &mut Game) {
    let base = 0xf78 + usize::from(k);
    let palette_field = core::mem::offset_of!(Enemy_MaridiaBeybladeTurtle, base)
        + core::mem::offset_of!(EnemyBase, palette_index);
    let field = base + palette_field;
    let palette = [game.g_ram[field], game.g_ram[field + 1]];
    // Child offsets remain wide even when k is near the end of a word.
    for child in [64usize, 128, 192, 256] {
        let field = base + child + palette_field;
        game.g_ram[field..field + 2].copy_from_slice(&palette);
    }
    let tiles_field = core::mem::offset_of!(Enemy_MaridiaBeybladeTurtle, base)
        + core::mem::offset_of!(EnemyBase, vram_tiles_index);
    let field = base + tiles_field;
    let tiles = [game.g_ram[field], game.g_ram[field + 1]];
    for child in [64usize, 128, 192, 256] {
        let field = base + child + tiles_field;
        game.g_ram[field..field + 2].copy_from_slice(&tiles);
    }
    for child in [64usize, 128, 192, 256] {
        let field = base + child + core::mem::offset_of!(Enemy_MaridiaBeybladeTurtle, mbte_var_B);
        game.g_ram[field..field + 2].copy_from_slice(&k.to_le_bytes());
    }
    let field = base + core::mem::offset_of!(Enemy_MaridiaBeybladeTurtle, mbte_var_A);
    game.g_ram[field..field + 2].copy_from_slice(&0x8e0au16.to_le_bytes());
}
pub fn MaridiaBeybladeTurtle_Func2(k: uint16, game: &mut Game) {
    let base = 0xf78 + usize::from(k);
    let field = base + core::mem::offset_of!(Enemy_MaridiaBeybladeTurtle, mbte_var_F);
    let active = u16::from_le_bytes([game.g_ram[field], game.g_ram[field + 1]]);
    if active != 0 {
        game.enemy_coordinates(k).set_y_height(0);
        let delta = game
            .enemy_coordinates(k)
            .x_pos()
            .wrapping_sub(game.samus_x_pos());
        let magnitude = if delta & 0x8000 != 0 {
            delta.wrapping_neg()
        } else {
            delta
        };
        if magnitude < 24 {
            let table_index = if delta & 0x8000 != 0 {
                magnitude + 24
            } else {
                magnitude
            };
            let index = game.cur_enemy_index();
            let radius = g_word_A28E80[usize::from(table_index)].wrapping_neg() as u16;
            game.enemy_coordinates(k).set_y_height(radius);
            let properties = game.enemy_instructions(k).properties();
            game.enemy_instructions(k)
                .set_properties(properties | 0x8000);
            if CheckIfEnemyTouchesSamus(index, game) != 0 {
                let top = game
                    .enemy_coordinates(k)
                    .y_pos()
                    .wrapping_sub(game.enemy_coordinates(k).y_height());
                let samus_radius = game.samus_y_radius();
                let samus_y = game.samus_y_pos();
                if (samus_radius.wrapping_add(samus_y).wrapping_sub(top) as i16) >= 0 {
                    let displacement = game.extra_samus_y_displacement();
                    let samus_radius = game.samus_y_radius();
                    let samus_y = game.samus_y_pos();
                    game.set_extra_samus_y_displacement(
                        displacement
                            .wrapping_add(top.wrapping_sub(samus_radius.wrapping_add(samus_y))),
                    );
                }
            }
        }
    } else {
        let field = base + core::mem::offset_of!(Enemy_MaridiaBeybladeTurtle, mbte_var_A);
        game.g_ram[field..field + 2].copy_from_slice(&0x8ee0u16.to_le_bytes());
        let properties = game.enemy_instructions(k).properties();
        game.enemy_instructions(k)
            .set_properties(properties & !(kEnemyProps_Tangible as u16));
    }
}
pub unsafe fn MaridiaBeybladeTurtle_Func3(mut k: uint16, game: &mut Game) {
    MaridiaBeybladeTurtle_Func4(game);
    if *(&raw mut game.g_ram as *mut uint8).offset(0x5b5) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        if CheckIfEnemyTouchesSamus(k, game) != 0 {
            let ref mut fresh23 =
                *((&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16);
            *fresh23 = (*fresh23).wrapping_sub(1);
        }
        let mut E: *mut Enemy_MaridiaBeybladeTurtle =
            Get_MaridiaBeybladeTurtle(k as ::core::ffi::c_int, game);
        (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
        (*E).base.y_height = 16 as uint16;
        if (*E).base.y_pos as ::core::ffi::c_int & 1 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            (*E).base.x_pos = (*E).base.x_pos.wrapping_sub(1);
        } else {
            (*E).base.x_pos = (*E).base.x_pos.wrapping_add(1);
        }
        if Enemy_MoveRight_IgnoreSlopes(
            k,
            (1 as ::core::ffi::c_int as int16 as int32) << 16 as ::core::ffi::c_int,
            game,
        ) == 0
        {
            (*E).base.current_instruction =
                addr_kMaridiaBeybladeTurtle_Ilist_8C4A as ::core::ffi::c_int as uint16;
            (*E).base.instruction_timer = 1 as uint16;
            (*E).mbte_var_A = addr_locret_A28E09 as ::core::ffi::c_int as uint16;
        }
    }
}
pub unsafe fn MaridiaBeybladeTurtle_Func5(mut k: uint16, game: &mut Game) {
    let mut v1: uint16 = addr_kMaridiaBeybladeTurtle_Ilist_8C1C as ::core::ffi::c_int as uint16;
    let mut E: *mut Enemy_MaridiaBeybladeTurtle =
        Get_MaridiaBeybladeTurtle(k as ::core::ffi::c_int, game);
    if (((*E).base.x_pos as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) as ::core::ffi::c_int)
        as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        v1 = addr_kMaridiaBeybladeTurtle_Ilist_8D00 as ::core::ffi::c_int as uint16;
    }
    (*E).base.current_instruction = v1;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).mbte_var_A = addr_locret_A28E09 as ::core::ffi::c_int as uint16;
}
pub unsafe fn MaridiaBeybladeTurtle_Func6(mut k: uint16, game: &mut Game) {
    if CheckIfEnemyTouchesSamus(k, game) != 0 {
        let mut E: *mut Enemy_MaridiaTurtle = Get_MaridiaTurtle(k as ::core::ffi::c_int, game);
        (*E).mte_var_A = (0xa29083 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        AddToHiLo(
            (&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16,
            (&raw mut game.g_ram as *mut uint8).offset(0xb56) as *mut uint16,
            -((((*E).mte_var_E as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
                | (*E).mte_var_03 as ::core::ffi::c_int) as int32) as uint32,
        );
        if *((&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16) as ::core::ffi::c_int
            + 16 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16) =
                -(16 as ::core::ffi::c_int) as uint16;
        }
    }
}
pub unsafe fn MaridiaBeybladeTurtle_Func7(mut k: uint16, game: &mut Game) {
    MaridiaBeybladeTurtle_Func4(game);
    if Enemy_MoveDown(
        k,
        (-(1 as ::core::ffi::c_int) as int16 as int32) << 16 as ::core::ffi::c_int,
        game,
    ) == 0
    {
        if CheckIfEnemyTouchesSamus(k, game) != 0 {
            let ref mut fresh24 =
                *((&raw mut game.g_ram as *mut uint8).offset(0xb5c) as *mut uint16);
            *fresh24 = (*fresh24).wrapping_sub(1);
        }
        let mut E: *mut Enemy_MaridiaTurtle = Get_MaridiaTurtle(k as ::core::ffi::c_int, game);
        let mut v2: uint16 =
            ((*E).mte_var_00 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
        (*E).mte_var_00 = v2;
        if v2 == 0 {
            let mut v3: uint16 = 0 as uint16;
            if (((*E).base.x_pos as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                    as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int
            {
                v3 = 4 as uint16;
            }
            (*E).mte_var_01 = *((game.g_rom.offset(
                (((0xa28d56 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa28d56 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16 as *mut uint8)
                .offset(v3 as ::core::ffi::c_int as isize)
                as *mut uint16);
            (*E).mte_var_02 = *((game.g_rom.offset(
                (((0xa28d58 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa28d58 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16 as *mut uint8)
                .offset(v3 as ::core::ffi::c_int as isize)
                as *mut uint16);
            (*E).mte_var_E = 0 as uint16;
            (*E).mte_var_03 = 0 as uint16;
            (*E).mte_var_A =
                (0xa28feb as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
    }
}
pub unsafe fn MaridiaBeybladeTurtle_Func8(mut k: uint16, game: &mut Game) {
    MaridiaBeybladeTurtle_Func4(game);
    let mut E: *mut Enemy_MaridiaTurtle = Get_MaridiaTurtle(k as ::core::ffi::c_int, game);
    if Enemy_MoveRight_IgnoreSlopes(
        k,
        (((*E).mte_var_E as uint32) << 16 as ::core::ffi::c_int | (*E).mte_var_03 as uint32)
            as int32,
        game,
    ) != 0
    {
        SetHiLo(
            &raw mut (*E).mte_var_E,
            &raw mut (*E).mte_var_03,
            -((((*E).mte_var_E as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
                | (*E).mte_var_03 as ::core::ffi::c_int) as int32) as uint32,
        );
        SetHiLo(
            &raw mut (*E).mte_var_02,
            &raw mut (*E).mte_var_01,
            -((((*E).mte_var_02 as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
                | (*E).mte_var_01 as ::core::ffi::c_int) as int32) as uint32,
        );
        *((&raw mut game.g_ram as *mut uint8).offset(0x183e) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1840) as *mut uint16) = 16 as uint16;
        QueueSfx2_Max6(0x1b as uint16, game);
    } else {
        MaridiaBeybladeTurtle_Func6(k, game);
        AddToHiLo(
            &raw mut (*E).mte_var_E,
            &raw mut (*E).mte_var_03,
            ((*E).mte_var_02 as uint32) << 16 as ::core::ffi::c_int | (*E).mte_var_01 as uint32,
        );
        if (abs16((*E).mte_var_E) as ::core::ffi::c_int - g_word_A28D5E as ::core::ffi::c_int)
            as int16 as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        {
            (*E).mte_var_E = (if (*E).mte_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                -(g_word_A28D5E as ::core::ffi::c_int)
            } else {
                g_word_A28D5E as ::core::ffi::c_int
            }) as uint16;
        }
    };
}
pub unsafe fn MaridiaBeybladeTurtle_Func9(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_MaridiaTurtle = Get_MaridiaTurtle(k as ::core::ffi::c_int, game);
    let mut v2: uint16 = 0;
    MaridiaBeybladeTurtle_Func4(game);
    if (((*E).base.y_pos as ::core::ffi::c_int - g_word_A28D60 as ::core::ffi::c_int) as int16
        as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        (*E).mte_var_00 = g_word_A28D64;
        v2 = (0xa290cc as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    } else {
        if CheckIfEnemyTouchesSamus(k, game) != 0 {
            (*E).base.y_pos = ((*E).base.y_pos as ::core::ffi::c_int
                - g_word_A28D62 as ::core::ffi::c_int) as uint16;
            let ref mut fresh25 =
                *((&raw mut game.g_ram as *mut uint8).offset(0xb5c) as *mut uint16);
            *fresh25 =
                (*fresh25 as ::core::ffi::c_int - g_word_A28D62 as ::core::ffi::c_int) as uint16;
            return;
        }
        v2 = (0xa290e1 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    (*E).mte_var_A = v2;
    (*E).mte_var_07 = 0 as uint16;
    (*E).mte_var_04 = 0 as uint16;
}
pub unsafe fn MaridiaBeybladeTurtle_Func10(mut k: uint16, game: &mut Game) {
    MaridiaBeybladeTurtle_Func4(game);
    let mut E: *mut Enemy_MaridiaTurtle = Get_MaridiaTurtle(k as ::core::ffi::c_int, game);
    let mut v2: uint16 =
        ((*E).mte_var_00 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
    (*E).mte_var_00 = v2;
    if v2 == 0 {
        (*E).mte_var_A = (0xa290e1 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn MaridiaBeybladeTurtle_Func11(mut k: uint16, game: &mut Game) {
    MaridiaBeybladeTurtle_Func4(game);
    let mut E: *mut Enemy_MaridiaTurtle = Get_MaridiaTurtle(k as ::core::ffi::c_int, game);
    if (((*Get_MaridiaTurtle(0 as ::core::ffi::c_int, game)).mte_var_04 as ::core::ffi::c_int
        - g_word_A28D66 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        AddToHiLo(
            &raw mut (*E).mte_var_04,
            &raw mut (*E).mte_var_07,
            0x2000 as uint32,
        );
    }
    if Enemy_MoveDown(
        k,
        ((*E).mte_var_04 as int16 as int32) << 16 as ::core::ffi::c_int,
        game,
    ) != 0
    {
        (*E).base.current_instruction = (if (*E).mte_var_E as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            addr_kMaridiaBeybladeTurtle_Ilist_8D28 as ::core::ffi::c_int
        } else {
            addr_kMaridiaBeybladeTurtle_Ilist_8C4A as ::core::ffi::c_int
        }) as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).mte_var_A = addr_locret_A28E09 as ::core::ffi::c_int as uint16;
    }
}
pub unsafe fn MiniMaridiaBeybladeTurtle_Main(game: &mut Game) {
    let index = game.cur_enemy_index();
    let base = 0xf78 + usize::from(index);
    let parent_offset = base + core::mem::offset_of!(Enemy_MiniMaridiaTurtle, mmte_var_B);
    let parent = u16::from_le_bytes([game.g_ram[parent_offset], game.g_ram[parent_offset + 1]]);
    let parent_trigger_offset =
        0xf78 + usize::from(parent) + core::mem::offset_of!(Enemy_MiniMaridiaTurtle, mmte_var_06);
    game.g_ram[parent_trigger_offset..parent_trigger_offset + 2]
        .copy_from_slice(&0u16.to_le_bytes());
    let preinstruction_offset = base + core::mem::offset_of!(Enemy_MiniMaridiaTurtle, mmte_var_A);
    let preinstruction = u16::from_le_bytes([
        game.g_ram[preinstruction_offset],
        game.g_ram[preinstruction_offset + 1],
    ]);
    let live_index = game.cur_enemy_index();
    CallMaridiaBeybladeTurtleFunc(0xa20000 | u32::from(preinstruction), live_index, game);
}
pub unsafe fn MiniMaridiaBeybladeTurtle_Func1(mut k: uint16, game: &mut Game) {
    if CheckIfEnemyTouchesSamus(k, game) != 0 {
        let mut E: *mut Enemy_MiniMaridiaTurtle =
            Get_MiniMaridiaTurtle(k as ::core::ffi::c_int, game);
        (*E).mmte_var_A = (0xa2916e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).mmte_var_05 = 4 as uint16;
        let mut v2: uint16 = addr_kMaridiaBeybladeTurtle_Ilist_8C30 as ::core::ffi::c_int as uint16;
        if (*E).mmte_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            v2 = addr_kMaridiaBeybladeTurtle_Ilist_8D14 as ::core::ffi::c_int as uint16;
        }
        (*E).base.current_instruction = v2;
        (*E).base.instruction_timer = 1 as uint16;
    }
}
pub unsafe fn MiniMaridiaBeybladeTurtle_Func2(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_MiniMaridiaTurtle = Get_MiniMaridiaTurtle(k as ::core::ffi::c_int, game);
    if CheckIfEnemyTouchesSamus(k, game) != 0 {
        (*E).mmte_var_05 = 4 as uint16;
    } else {
        let mut v2: uint16 =
            ((*E).mmte_var_05 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
        (*E).mmte_var_05 = v2;
        if v2 == 0 {
            (*E).mmte_var_A =
                (0xa29198 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            (*E).mmte_var_00 = 60 as uint16;
        }
    };
}
pub unsafe fn MiniMaridiaBeybladeTurtle_Func3(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_MiniMaridiaTurtle = Get_MiniMaridiaTurtle(k as ::core::ffi::c_int, game);
    if CheckIfEnemyTouchesSamus(k, game) != 0 {
        (*E).mmte_var_A = (0xa291f8 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).base.current_instruction =
            addr_kMaridiaBeybladeTurtle_Ilist_8BD2 as ::core::ffi::c_int as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).mmte_var_04 = 1 as uint16;
        let mut v2: uint16 = g_word_A28D68;
        if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int
            == 8 as ::core::ffi::c_int
        {
            v2 = g_word_A28D6A;
        }
        (*E).mmte_var_E = v2;
    } else {
        let mut v4: uint16 =
            ((*E).mmte_var_00 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
        (*E).mmte_var_00 = v4;
        if v4 == 0 {
            let mut v5: uint16 =
                addr_kMaridiaBeybladeTurtle_Ilist_8B80 as ::core::ffi::c_int as uint16;
            if (*E).mmte_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            {
                v5 = addr_kMaridiaBeybladeTurtle_Ilist_8C72 as ::core::ffi::c_int as uint16;
            }
            (*E).base.current_instruction = v5;
            (*E).base.instruction_timer = 1 as uint16;
            (*E).mmte_var_A =
                (0xa29142 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
    };
}
pub unsafe fn MiniMaridiaBeybladeTurtle_Func4(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_MiniMaridiaTurtle = Get_MiniMaridiaTurtle(k as ::core::ffi::c_int, game);
    if Enemy_MoveRight_IgnoreSlopes(
        k,
        (((*E).mmte_var_E as uint32) << 16 as ::core::ffi::c_int
            | 0 as ::core::ffi::c_int as uint16 as uint32) as int32,
        game,
    ) != 0
    {
        (*E).mmte_var_E = -((*E).mmte_var_E as ::core::ffi::c_int) as uint16;
    } else {
        Enemy_MoveDown(
            k,
            ((*E).mmte_var_04 as int16 as int32) << 16 as ::core::ffi::c_int,
            game,
        );
    };
}
pub unsafe fn MiniMaridiaBeybladeTurtle_Func5(mut k: uint16, game: &mut Game) {
    let mut v1: uint16 = addr_kMaridiaBeybladeTurtle_Ilist_8B80 as ::core::ffi::c_int as uint16;
    let mut E: *mut Enemy_MiniMaridiaBeybladeTurtle =
        Get_MiniMaridiaBeybladeTurtle(k as ::core::ffi::c_int, game);
    if (*E).mmbte_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        v1 = addr_kMaridiaBeybladeTurtle_Ilist_8C72 as ::core::ffi::c_int as uint16;
    }
    (*E).base.current_instruction = v1;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).mmbte_var_A = (0xa29142 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn MiniMaridiaBeybladeTurtle_Func6(mut k: uint16, game: &mut Game) {
    if CheckIfEnemyTouchesSamus(k, game) != 0 {
        let mut v1: uint16 = addr_kMaridiaBeybladeTurtle_Ilist_8B80 as ::core::ffi::c_int as uint16;
        let mut E: *mut Enemy_MiniMaridiaBeybladeTurtle =
            Get_MiniMaridiaBeybladeTurtle(k as ::core::ffi::c_int, game);
        if (*E).mmbte_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            v1 = addr_kMaridiaBeybladeTurtle_Ilist_8C72 as ::core::ffi::c_int as uint16;
        }
        (*E).base.current_instruction = v1;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).mmbte_var_A =
            (0xa29142 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    } else {
        MiniMaridiaBeybladeTurtle_Func4(k, game);
    };
}
pub unsafe fn MiniMaridiaBeybladeTurtle_Func7(game: &mut Game) {
    let mut E: *mut Enemy_MiniMaridiaTurtle = Get_MiniMaridiaTurtle(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*Get_MiniMaridiaTurtle((*E).mmte_var_B as ::core::ffi::c_int, game)).mmte_var_06 =
        (*E).base.y_height;
    if CheckIfEnemyTouchesSamus(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    ) == 0
    {
        (*Get_MiniMaridiaTurtle(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        ))
        .mmte_var_A = (0xa29142 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn MaridiaBeybladeTurtle_Touch(game: &mut Game) {
    let mut E: *mut Enemy_MiniMaridiaTurtle = Get_MiniMaridiaTurtle(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).base.properties as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        Enemy_NormalTouchAI_A2(game);
        (*E).mmte_var_A = (0xa290e1 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).mmte_var_04 = 2 as uint16;
    }
}
pub unsafe fn MiniMaridiaBeybladeTurtle_Touch(game: &mut Game) {
    let mut v2: int16 = 0;
    let mut E: *mut Enemy_MiniMaridiaBeybladeTurtle = Get_MiniMaridiaBeybladeTurtle(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).mmbte_var_A as ::core::ffi::c_int != 0x925e as ::core::ffi::c_int {
        if (*E).mmbte_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            (*E).base.current_instruction =
                addr_kMaridiaBeybladeTurtle_Ilist_8C72 as ::core::ffi::c_int as uint16;
            v2 = 1 as int16;
        } else {
            (*E).base.current_instruction =
                addr_kMaridiaBeybladeTurtle_Ilist_8B80 as ::core::ffi::c_int as uint16;
            v2 = -(1 as ::core::ffi::c_int) as int16;
        }
        (*E).mmbte_var_E = v2 as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        let mut v3: uint16 = 0;
        if ((*E).base.x_pos as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        {
            v3 = ((*E).base.x_width as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0xafe) as *mut uint16)
                    as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                    as ::core::ffi::c_int) as uint16;
        } else {
            v3 = ((((*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0xafe) as *mut uint16)
                    as ::core::ffi::c_int) as uint32)
                << 16 as ::core::ffi::c_int
                | *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) as uint32)
                .wrapping_sub(
                    ((*E).base.x_width as uint32) << 16 as ::core::ffi::c_int
                        | *((&raw mut game.g_ram as *mut uint8).offset(0xafe) as *mut uint16)
                            as uint32,
                )
                >> 16 as ::core::ffi::c_int) as uint16;
        }
        (*E).base.x_pos = v3;
        (*E).mmbte_var_A =
            (0xa29142 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        Enemy_MoveRight_IgnoreSlopes(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            (((*E).mmbte_var_E as uint32) << 16 as ::core::ffi::c_int
                | 0 as ::core::ffi::c_int as uint16 as uint32) as int32,
            game,
        );
        MaridiaBeybladeTurtle_Func21(game);
    }
}
pub unsafe fn MaridiaBeybladeTurtle_Func21(game: &mut Game) {
    let mut mbte_var_B: uint16 = (*Get_MaridiaBeybladeTurtle(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .mbte_var_B;
    let mut E: *mut Enemy_MaridiaBeybladeTurtle =
        Get_MaridiaBeybladeTurtle(mbte_var_B as ::core::ffi::c_int, game);
    if (*E).mbte_var_F != 0 {
        (*E).mbte_var_F = (*E).mbte_var_F.wrapping_sub(1);
    }
}
pub unsafe fn MiniMaridiaBeybladeTurtle_Shot(game: &mut Game) {
    NormalEnemyShotAi(game);
    MaridiaBeybladeTurtle_Func21(game);
}
pub unsafe fn MaridiaBeybladeTurtle_Func4(game: &mut Game) {
    let mut E: *mut Enemy_MaridiaBeybladeTurtle =
        Get_MaridiaBeybladeTurtle(0 as ::core::ffi::c_int, game);
    let mut r18: uint16 = ((*E).base.x_pos as ::core::ffi::c_int
        - (*E).base.x_width as ::core::ffi::c_int
        - 8 as ::core::ffi::c_int) as uint16;
    let mut r20: uint16 = ((*E).base.x_width as ::core::ffi::c_int
        + (*E).base.x_pos as ::core::ffi::c_int
        + 8 as ::core::ffi::c_int) as uint16;
    let mut r22: uint16 = ((*E).base.y_pos as ::core::ffi::c_int
        - (*E).base.y_height as ::core::ffi::c_int
        + 4 as ::core::ffi::c_int) as uint16;
    let mut r24: uint16 = ((*E).base.y_height as ::core::ffi::c_int
        + (*E).base.y_pos as ::core::ffi::c_int
        - 4 as ::core::ffi::c_int) as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xafe) as *mut uint16) as ::core::ffi::c_int
        - 1 as ::core::ffi::c_int
        - r20 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
        && *((&raw mut game.g_ram as *mut uint8).offset(0xafe) as *mut uint16) as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                as ::core::ffi::c_int
            - r18 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        && *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0xb00) as *mut uint16)
                as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            - r24 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        && *((&raw mut game.g_ram as *mut uint8).offset(0xb00) as *mut uint16) as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
                as ::core::ffi::c_int
            - r22 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        && *((&raw mut game.g_ram as *mut uint8).offset(0x18a8) as *mut uint16) == 0
    {
        NormalEnemyTouchAi(game);
    }
}
pub unsafe fn MaridiaBeybladeTurtle_Instr_9381(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut R48: uint16 = 0 as uint16;
    if CheckIfEnemyTouchesSamus(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    ) != 0
    {
        let ref mut fresh0 = *((&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16);
        *fresh0 = (*fresh0 as ::core::ffi::c_int
            + (*Get_MaridiaBeybladeTurtle(
                *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                    as ::core::ffi::c_int,
                game,
            ))
            .mbte_var_E as ::core::ffi::c_int) as uint16;
        R48 = 1 as uint16;
    }
    let mut E: *mut Enemy_MaridiaBeybladeTurtle = Get_MaridiaBeybladeTurtle(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut R50: uint16 = (*E).base.y_pos;
    (*E).base.y_pos = (*E).mbte_var_D;
    Enemy_MoveRight_IgnoreSlopes(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        ((*E).mbte_var_E as int16 as int32) << 16 as ::core::ffi::c_int,
        game,
    );
    if (*Get_MaridiaBeybladeTurtle((*E).mbte_var_B as ::core::ffi::c_int, game)).mbte_var_A
        as ::core::ffi::c_int
        == 0x8e0a as ::core::ffi::c_int
    {
        let mut x_pos: uint16 =
            (*Get_MaridiaBeybladeTurtle((*E).mbte_var_B as ::core::ffi::c_int, game))
                .base
                .x_pos;
        let mut xd: uint16 =
            (x_pos as ::core::ffi::c_int - (*E).base.x_pos as ::core::ffi::c_int) as uint16;
        let mut v7: uint16 = abs16(xd);
        let mut v8: uint16 = 0;
        if v7 as ::core::ffi::c_int - 24 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            if xd as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
                v7 = (v7 as ::core::ffi::c_int + 24 as ::core::ffi::c_int) as uint16;
            }
            v8 = g_word_A28E80[v7 as usize] as uint16;
        } else {
            v8 = 1 as uint16;
        }
        Enemy_MoveDown(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            (v8 as int16 as int32) << 16 as ::core::ffi::c_int,
            game,
        );
        if R48 != 0 {
            let ref mut fresh1 =
                *((&raw mut game.g_ram as *mut uint8).offset(0xb5c) as *mut uint16);
            *fresh1 = (*fresh1 as ::core::ffi::c_int
                + ((*Get_MaridiaBeybladeTurtle(
                    *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                        as ::core::ffi::c_int,
                    game,
                ))
                .base
                .y_pos as ::core::ffi::c_int
                    - R50 as ::core::ffi::c_int)) as uint16;
        }
    }
    return jp;
}
pub unsafe fn MaridiaBeybladeTurtle_Instr_9412(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut v4: int16 = 0;
    let mut E: *mut Enemy_MaridiaBeybladeTurtle = Get_MaridiaBeybladeTurtle(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut xd: uint16 =
        ((*E).mbte_var_C as ::core::ffi::c_int - (*E).base.x_pos as ::core::ffi::c_int) as uint16;
    if (abs16(xd) as ::core::ffi::c_int - g_word_A28D50 as ::core::ffi::c_int) as int16
        as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        if xd as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
            v4 = 1 as int16;
        } else {
            v4 = -(1 as ::core::ffi::c_int) as int16;
        }
        (*E).mbte_var_E = v4 as uint16;
    }
    if (*Get_MaridiaBeybladeTurtle(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .mbte_var_E as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        return addr_kMaridiaBeybladeTurtle_Ilist_8B80 as ::core::ffi::c_int as uintptr_t
            as *const uint16;
    }
    return addr_kMaridiaBeybladeTurtle_Ilist_8C72 as ::core::ffi::c_int as uintptr_t
        as *const uint16;
}
pub unsafe fn MaridiaBeybladeTurtle_Instr_9447(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_MaridiaBeybladeTurtle(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .mbte_var_A = (0xa28f3f as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    return jp;
}
pub unsafe fn MaridiaBeybladeTurtle_Instr_9451(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_MaridiaTurtle = Get_MaridiaTurtle(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).mte_var_A = (0xa28f8d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).mte_var_E = -(1 as ::core::ffi::c_int) as uint16;
    (*E).mte_var_00 = 16 as uint16;
    return addr_kMaridiaBeybladeTurtle_Ilist_8C02 as ::core::ffi::c_int as uintptr_t
        as *const uint16;
}
pub unsafe fn MaridiaBeybladeTurtle_Instr_946B(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_MaridiaTurtle = Get_MaridiaTurtle(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).mte_var_A = (0xa28f8d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).mte_var_E = 1 as uint16;
    (*E).mte_var_00 = 16 as uint16;
    return addr_kMaridiaBeybladeTurtle_Ilist_8C02 as ::core::ffi::c_int as uintptr_t
        as *const uint16;
}
pub unsafe fn MaridiaBeybladeTurtle_Instr_9485(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    if CheckIfEnemyTouchesSamus(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    ) != 0
    {
        if (*Get_MaridiaBeybladeTurtle(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        ))
        .mbte_var_E as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            return addr_kMaridiaBeybladeTurtle_Ilist_8D40 as ::core::ffi::c_int as uintptr_t
                as *const uint16;
        } else {
            return addr_kMaridiaBeybladeTurtle_Ilist_8C62 as ::core::ffi::c_int as uintptr_t
                as *const uint16;
        }
    }
    return jp;
}
pub unsafe fn MaridiaBeybladeTurtle_Instr_94A1(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_MaridiaBeybladeTurtle = Get_MaridiaBeybladeTurtle(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if CheckIfEnemyTouchesSamus(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    ) != 0
    {
        (*E).mbte_var_A = (0xa2925e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    } else {
        (*E).mbte_var_A = (0xa29142 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    if (*E).mbte_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        return addr_kMaridiaBeybladeTurtle_Ilist_8C72 as ::core::ffi::c_int as uintptr_t
            as *const uint16;
    }
    return addr_kMaridiaBeybladeTurtle_Ilist_8B80 as ::core::ffi::c_int as uintptr_t
        as *const uint16;
}
pub unsafe fn MaridiaBeybladeTurtle_Instr_94C7(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_MaridiaBeybladeTurtle = Get_MaridiaBeybladeTurtle(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).mbte_var_A = (0xa29239 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    return jp;
}
pub unsafe fn MaridiaBeybladeTurtle_Instr_94D1(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    QueueSfx2_Max6(0x3a as uint16, game);
    return jp;
}
pub unsafe fn ThinHoppingBlobs_Init(game: &mut Game) {
    let mut E: *mut Enemy_ThinHoppingBlobs = Get_ThinHoppingBlobs(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.spritemap_pointer = addr_kSpritemap_Nothing_A2 as ::core::ffi::c_int as uint16;
    (*E).thbs_var_A = 0 as uint16;
    sub_A29A6C(
        addr_kThinHoppingBlobs_Ilist_99AD as ::core::ffi::c_int as uint16,
        game,
    );
    (*E).thbs_var_00 = 0 as uint16;
    (*E).thbs_var_D = (0xa29b65 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).thbs_var_C = (*E).thbs_parameter_1;
    (*E).thbs_var_04 = 0 as uint16;
}
pub unsafe fn sub_A29A6C(mut a: uint16, game: &mut Game) {
    let mut v1: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*v1).current_instruction = a;
    (*v1).instruction_timer = 1 as uint16;
    (*v1).timer = 0 as uint16;
}
pub unsafe fn CallThinHoppingBlobs(mut ea: uint32, game: &mut Game) {
    match ea {
        10656613 => {
            ThinHoppingBlobs_Func6(
                *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
                game,
            );
            return;
        }
        10656641 => {
            ThinHoppingBlobs_Func7(game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn ThinHoppingBlobs_Main(game: &mut Game) {
    let mut E: *mut Enemy_ThinHoppingBlobs = Get_ThinHoppingBlobs(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallThinHoppingBlobs(
        ((*E).thbs_var_D as ::core::ffi::c_int | 0xa20000 as ::core::ffi::c_int) as uint32,
        game,
    );
}
pub unsafe fn ThinHoppingBlobs_Func1(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_ThinHoppingBlobs = Get_ThinHoppingBlobs(k as ::core::ffi::c_int, game);
    if (*E).thbs_var_00 as ::core::ffi::c_int - 3 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        TwinHoppingBlobs_Func2(game);
    }
    ThinHoppingBlobs_Func3(game);
    ThinHoppingBlobs_Func5(game);
}
pub unsafe fn TwinHoppingBlobs_Func2(game: &mut Game) {
    let mut E: *mut Enemy_ThinHoppingBlobs = Get_ThinHoppingBlobs(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).thbs_var_00 = IsSamusWithinEnemy_X(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        (*E).thbs_parameter_2,
        game,
    );
}
pub unsafe fn ThinHoppingBlobs_Func3(game: &mut Game) {
    let mut E: *mut Enemy_ThinHoppingBlobs = Get_ThinHoppingBlobs(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).thbs_var_02 = 1 as uint16;
    if GetSamusEnemyDelta_X(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    ) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        (*E).thbs_var_02 = 0 as uint16;
    }
    if (*E).thbs_var_04 != 0 {
        (*E).thbs_var_02 = (*E).thbs_var_05;
    }
    (*E).thbs_var_04 = 0 as uint16;
    let mut R28: uint16 = ThinHoppingBlobs_Func4(game);
    let mut thbs_var_00: uint16 = (*E).thbs_var_00;
    if thbs_var_00 as ::core::ffi::c_int - 3 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0
    {
        if thbs_var_00 == 0 {
            R28 = (R28 as ::core::ffi::c_int & 1 as ::core::ffi::c_int) as uint16;
        }
        thbs_var_00 = R28;
        if R28 as ::core::ffi::c_int - 2 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
            thbs_var_00 = 2 as uint16;
        }
    }
    let mut v3: uint16 = (8 as ::core::ffi::c_int * thbs_var_00 as ::core::ffi::c_int) as uint16;
    (*E).thbs_var_F = v3;
    (*E).thbs_var_E = *((game.g_rom.offset(
        (((0xa29a0d as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa29a0d as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16 as *mut uint8)
        .offset(v3 as ::core::ffi::c_int as isize) as *mut uint16);
}
pub unsafe fn ThinHoppingBlobs_Func4(game: &mut Game) -> uint16 {
    NextRandom(game);
    return ((*Get_ThinHoppingBlobs(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .base
    .frame_counter as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) as ::core::ffi::c_int
        & 7 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn ThinHoppingBlobs_Func5(game: &mut Game) {
    let mut v2: uint16 = 0;
    let mut E: *mut Enemy_ThinHoppingBlobs = Get_ThinHoppingBlobs(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut thbs_var_F: uint16 = (*E).thbs_var_F;
    let mut r22: uint16 = 0 as uint16;
    let mut r24: uint16 = 0 as uint16;
    loop {
        r22 = (r22 as ::core::ffi::c_int
            + *((game.g_rom.offset(
                (((0xa29a09 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa29a09 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16 as *mut uint8)
                .offset(thbs_var_F as ::core::ffi::c_int as isize) as *mut uint16)
                as ::core::ffi::c_int) as uint16;
        r24 = (r24 as ::core::ffi::c_int
            + *((game.g_rom.offset(
                (((0xa2838f as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa2838f as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16 as *mut uint8)
                .offset(
                    (8 as ::core::ffi::c_int
                        * ((r22 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int)
                            >> 8 as ::core::ffi::c_int)) as isize,
                )
                .offset(1) as *mut uint16) as ::core::ffi::c_int) as uint16;
        thbs_var_F = (*E).thbs_var_F;
        v2 = swap16(
            *((game.g_rom.offset(
                (((0xa29a07 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa29a07 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16 as *mut uint8)
                .offset(thbs_var_F as ::core::ffi::c_int as isize) as *mut uint16),
        );
        if !(v2 as ::core::ffi::c_int - r24 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            == 0)
        {
            break;
        }
    }
    (*E).thbs_var_B = r22;
    (*E).thbs_var_03 = 0 as uint16;
    let mut v3: uint16 = (r22 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
    (*E).thbs_var_07 = (r22 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
    (*E).thbs_var_06 = ((*E).thbs_var_07 as ::core::ffi::c_int
        + (v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)) as uint16;
}
pub unsafe fn ThinHoppingBlobs_Func6(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_ThinHoppingBlobs = Get_ThinHoppingBlobs(k as ::core::ffi::c_int, game);
    (*E).thbs_var_C = (*E).thbs_var_C.wrapping_sub(1);
    if (*E).thbs_var_C as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).thbs_var_D = (0xa29b81 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).thbs_var_C = (*E).thbs_parameter_1;
        (*E).thbs_var_01 = 1 as uint16;
        ThinHoppingBlobs_Func1(k, game);
    }
}
pub unsafe fn CallThinHoppingBlobsVarE(mut ea: uint32, game: &mut Game) {
    match ea {
        10657035 => {
            ThinHoppingBlobs_Func11(game);
            return;
        }
        10657067 => {
            ThinHoppingBlobs_Func12(game);
            return;
        }
        10657099 => {
            ThinHoppingBlobs_Func13(game);
            return;
        }
        10657131 => {
            ThinHoppingBlobs_Func14(game);
            return;
        }
        10657176 => {
            ThinHoppingBlobs_Func15(game);
            return;
        }
        10657229 => {
            ThinHoppingBlobs_Func16(game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn ThinHoppingBlobs_Func7(game: &mut Game) {
    let mut E: *mut Enemy_ThinHoppingBlobs = Get_ThinHoppingBlobs(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallThinHoppingBlobsVarE(
        ((*E).thbs_var_E as ::core::ffi::c_int | 0xa20000 as ::core::ffi::c_int) as uint32,
        game,
    );
}
pub unsafe fn ThinHoppingBlobs_Func8(game: &mut Game) {
    let mut E: *mut Enemy_ThinHoppingBlobs = Get_ThinHoppingBlobs(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut thbs_var_B: uint16 = (*E).thbs_var_B;
    if thbs_var_B as ::core::ffi::c_int - 0x4000 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        thbs_var_B = 0x4000 as uint16;
    }
    let mut v6: uint16 = (8 as ::core::ffi::c_int
        * (thbs_var_B as ::core::ffi::c_int >> 8 as ::core::ffi::c_int))
        as uint16;
    if (*E).thbs_var_03 == 0 {
        v6 = (v6 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
    }
    if Enemy_MoveDown(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        *(game.g_rom.offset(
            (((0xa0cbc7 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa0cbc7 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint32)
            .offset((v6 as ::core::ffi::c_int >> 2 as ::core::ffi::c_int) as isize)
            as int32,
        game,
    ) != 0
    {
        if (*E).thbs_var_03 != 0 {
            (*E).thbs_var_D =
                (0xa29b65 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        } else {
            (*E).thbs_var_04 = 1 as uint16;
            (*E).thbs_var_05 =
                ((*E).thbs_var_02 as ::core::ffi::c_int ^ 1 as ::core::ffi::c_int) as uint16;
            (*E).thbs_var_00 = 4 as uint16;
            (*E).thbs_var_E =
                (0xa29d98 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
        (*E).thbs_var_01 = 0 as uint16;
    } else {
        let mut thbs_var_F: uint16 = (*E).thbs_var_F;
        if (*E).thbs_var_03 != 0 {
            ThinHoppingBlobs_Func10(game);
            (*E).thbs_var_B = ((*E).thbs_var_B as ::core::ffi::c_int
                + *((game.g_rom.offset(
                    (((0xa29a0b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xa29a0b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16 as *mut uint8)
                    .offset(thbs_var_F as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint16;
        } else {
            ThinHoppingBlobs_Func9(game);
            (*E).thbs_var_B = ((*E).thbs_var_B as ::core::ffi::c_int
                - *((game.g_rom.offset(
                    (((0xa29a0b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xa29a0b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16 as *mut uint8)
                    .offset(thbs_var_F as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint16;
        }
        if (*E).thbs_var_B as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            (*E).thbs_var_03 = 1 as uint16;
            (*E).thbs_var_B = 0 as uint16;
        }
        let mut R20: uint16 = ((*((game.g_rom.offset(
            (((0xa29a09 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa29a09 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16 as *mut uint8)
            .offset((*E).thbs_var_F as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0xff00 as ::core::ffi::c_int)
            >> 8 as ::core::ffi::c_int) as uint16;
        if (*E).thbs_var_02 != 0 {
            R20 = -(R20 as ::core::ffi::c_int) as uint16;
        }
        if Enemy_MoveRight_IgnoreSlopes(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            (R20 as int16 as int32) << 16 as ::core::ffi::c_int,
            game,
        ) != 0
        {
            (*E).thbs_var_04 = 1 as uint16;
            (*E).thbs_var_05 =
                ((*E).thbs_var_02 as ::core::ffi::c_int ^ 1 as ::core::ffi::c_int) as uint16;
            (*E).thbs_var_01 = 0 as uint16;
            (*E).thbs_var_00 = 4 as uint16;
            (*E).thbs_var_E =
                (0xa29d98 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
    };
}
pub unsafe fn ThinHoppingBlobs_Func9(game: &mut Game) {
    let mut E: *mut Enemy_ThinHoppingBlobs = Get_ThinHoppingBlobs(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v2: uint16 = 0;
    if (*E).thbs_var_02 != 0 {
        let mut thbs_var_B: uint16 = (*E).thbs_var_B;
        if (thbs_var_B as ::core::ffi::c_int - (*E).thbs_var_06 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        {
            v2 = addr_kThinHoppingBlobs_Ilist_9A01 as ::core::ffi::c_int as uint16;
        } else if (thbs_var_B as ::core::ffi::c_int - (*E).thbs_var_07 as ::core::ffi::c_int)
            as int16 as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        {
            v2 = addr_kThinHoppingBlobs_Ilist_99FB as ::core::ffi::c_int as uint16;
        } else {
            v2 = addr_kThinHoppingBlobs_Ilist_99F5 as ::core::ffi::c_int as uint16;
        }
    } else {
        let mut v1: uint16 = (*E).thbs_var_B;
        if (v1 as ::core::ffi::c_int - (*E).thbs_var_06 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        {
            v2 = addr_kThinHoppingBlobs_Ilist_99E9 as ::core::ffi::c_int as uint16;
        } else if (v1 as ::core::ffi::c_int - (*E).thbs_var_07 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        {
            v2 = addr_kThinHoppingBlobs_Ilist_99EF as ::core::ffi::c_int as uint16;
        } else {
            v2 = addr_kThinHoppingBlobs_Ilist_99F5 as ::core::ffi::c_int as uint16;
        }
    }
    sub_A29A6C(v2, game);
}
pub unsafe fn ThinHoppingBlobs_Func10(game: &mut Game) {
    let mut E: *mut Enemy_ThinHoppingBlobs = Get_ThinHoppingBlobs(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v2: uint16 = 0;
    if (*E).thbs_var_02 != 0 {
        let mut thbs_var_B: uint16 = (*E).thbs_var_B;
        if ((thbs_var_B as ::core::ffi::c_int - (*E).thbs_var_07 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        {
            v2 = addr_kThinHoppingBlobs_Ilist_99F5 as ::core::ffi::c_int as uint16;
        } else if ((thbs_var_B as ::core::ffi::c_int - (*E).thbs_var_06 as ::core::ffi::c_int)
            as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        {
            v2 = addr_kThinHoppingBlobs_Ilist_99EF as ::core::ffi::c_int as uint16;
        } else {
            v2 = addr_kThinHoppingBlobs_Ilist_99E9 as ::core::ffi::c_int as uint16;
        }
    } else {
        let mut v1: uint16 = (*E).thbs_var_B;
        if ((v1 as ::core::ffi::c_int - (*E).thbs_var_07 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        {
            v2 = addr_kThinHoppingBlobs_Ilist_99F5 as ::core::ffi::c_int as uint16;
        } else if ((v1 as ::core::ffi::c_int - (*E).thbs_var_06 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        {
            v2 = addr_kThinHoppingBlobs_Ilist_99FB as ::core::ffi::c_int as uint16;
        } else {
            v2 = addr_kThinHoppingBlobs_Ilist_9A01 as ::core::ffi::c_int as uint16;
        }
    }
    sub_A29A6C(v2, game);
}
pub unsafe fn ThinHoppingBlobs_Func11(game: &mut Game) {
    ThinHoppingBlobs_Func8(game);
    let mut E: *mut Enemy_ThinHoppingBlobs = Get_ThinHoppingBlobs(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).thbs_var_04 as ::core::ffi::c_int != 0 || (*E).thbs_var_01 == 0 {
        (*E).thbs_var_01 = 0 as uint16;
        sub_A29A6C(
            addr_kThinHoppingBlobs_Ilist_99D5 as ::core::ffi::c_int as uint16,
            game,
        );
    }
}
pub unsafe fn ThinHoppingBlobs_Func12(game: &mut Game) {
    ThinHoppingBlobs_Func8(game);
    let mut E: *mut Enemy_ThinHoppingBlobs = Get_ThinHoppingBlobs(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).thbs_var_04 as ::core::ffi::c_int != 0 || (*E).thbs_var_01 == 0 {
        (*E).thbs_var_01 = 0 as uint16;
        sub_A29A6C(
            addr_kThinHoppingBlobs_Ilist_99C1 as ::core::ffi::c_int as uint16,
            game,
        );
    }
}
pub unsafe fn ThinHoppingBlobs_Func13(game: &mut Game) {
    ThinHoppingBlobs_Func8(game);
    let mut E: *mut Enemy_ThinHoppingBlobs = Get_ThinHoppingBlobs(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).thbs_var_04 as ::core::ffi::c_int != 0 || (*E).thbs_var_01 == 0 {
        (*E).thbs_var_01 = 0 as uint16;
        sub_A29A6C(
            addr_kThinHoppingBlobs_Ilist_99AD as ::core::ffi::c_int as uint16,
            game,
        );
    }
}
pub unsafe fn ThinHoppingBlobs_Func14(game: &mut Game) {
    ThinHoppingBlobs_Func8(game);
    let mut E: *mut Enemy_ThinHoppingBlobs = Get_ThinHoppingBlobs(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).thbs_var_04 == 0 {
        if (*E).thbs_var_01 != 0 {
            return;
        }
        (*E).thbs_var_00 = 0 as uint16;
        (*E).thbs_var_D = (0xa29b65 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    (*E).thbs_var_01 = 0 as uint16;
    sub_A29A6C(
        addr_kThinHoppingBlobs_Ilist_99D5 as ::core::ffi::c_int as uint16,
        game,
    );
}
pub unsafe fn ThinHoppingBlobs_Func15(game: &mut Game) {
    let mut E: *mut Enemy_ThinHoppingBlobs = Get_ThinHoppingBlobs(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut thbs_var_F: uint16 = (*E).thbs_var_F;
    let mut R20: uint16 = ((*((game.g_rom.offset(
        (((0xa29a0b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa29a0b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16 as *mut uint8)
        .offset(thbs_var_F as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0xff00 as ::core::ffi::c_int)
        >> 8 as ::core::ffi::c_int) as uint16;
    let mut R18: uint16 = ((*(game.g_rom.offset(
        (((0xa29a0b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa29a0b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16 as *mut uint8)
        .offset(thbs_var_F as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int) as uint16;
    if Enemy_MoveDown(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        ((R20 as uint32) << 16 as ::core::ffi::c_int | R18 as uint32) as int32,
        game,
    ) != 0
    {
        let mut R28: uint16 = ThinHoppingBlobs_Func4(game);
        (*E).thbs_var_00 = ((R28 as ::core::ffi::c_int & 1 as ::core::ffi::c_int)
            + 5 as ::core::ffi::c_int) as uint16;
        (*E).thbs_var_D = (0xa29b65 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn ThinHoppingBlobs_Func16(game: &mut Game) {
    ThinHoppingBlobs_Func8(game);
    let mut E: *mut Enemy_ThinHoppingBlobs = Get_ThinHoppingBlobs(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).thbs_var_01 == 0 {
        (*E).thbs_var_01 = 0 as uint16;
        (*E).thbs_var_00 = 3 as uint16;
        (*E).thbs_var_D = (0xa29b65 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        sub_A29A6C(
            addr_kThinHoppingBlobs_Ilist_99D5 as ::core::ffi::c_int as uint16,
            game,
        );
    }
}
pub unsafe fn SpikeShootingPlant_Instr_9F2A(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    QueueSfx2_Max6(0x34 as uint16, game);
    return jp;
}
pub unsafe fn SpikeShootingPlant_Init(game: &mut Game) {
    let mut E: *mut Enemy_SpikeShootingPlant = Get_SpikeShootingPlant(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.spritemap_pointer = addr_kSpritemap_Nothing_A2 as ::core::ffi::c_int as uint16;
    if *(&raw mut (*E).sspt_parameter_1 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>() as usize)
            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) != 0
    {
        SpikeShootingPlant_5(game);
    } else {
        SpikeShootingPlant_7(game);
    }
    let mut sspt_parameter_1_low: uint16 =
        *(&raw mut (*E).sspt_parameter_1 as *mut uint8).offset(0) as uint16;
    (*E).sspt_var_E = sspt_parameter_1_low;
    (*E).sspt_var_F = *(game.g_rom.offset(
        (((0xa29f42 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa29f42 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(sspt_parameter_1_low as isize);
    let mut v2: ::core::ffi::c_int =
        *(&raw mut (*E).sspt_parameter_2 as *mut uint8).offset(0) as ::core::ffi::c_int;
    (*E).sspt_var_01 = (*(game.g_rom.offset(
        (((0xa29f36 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa29f36 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v2 as isize) as ::core::ffi::c_int
        + (*E).base.x_pos as ::core::ffi::c_int) as uint16;
    (*E).sspt_var_00 = ((*E).base.x_pos as ::core::ffi::c_int
        - *(game.g_rom.offset(
            (((0xa29f36 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa29f36 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v2 as isize) as ::core::ffi::c_int) as uint16;
    let mut v3: ::core::ffi::c_int = 8 as ::core::ffi::c_int
        * *(&raw mut (*E).sspt_parameter_2 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    (*E).sspt_var_B = *(game.g_rom.offset(
        (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v3 as isize);
    (*E).sspt_var_A = *(game.g_rom.offset(
        (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((v3 + 1 as ::core::ffi::c_int) as isize);
    (*E).sspt_var_D = *(game.g_rom.offset(
        (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((v3 + 2 as ::core::ffi::c_int) as isize);
    (*E).sspt_var_C = *(game.g_rom.offset(
        (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((v3 + 3 as ::core::ffi::c_int) as isize);
}
pub unsafe fn SpikeShootingPlant_Main(game: &mut Game) {
    let mut E: *mut Enemy_SpikeShootingPlant = Get_SpikeShootingPlant(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallEnemyPreInstr(
        ((*E).sspt_var_F as ::core::ffi::c_int | 0xa20000 as ::core::ffi::c_int) as uint32,
        game,
    );
}
pub unsafe fn SpikeShootingPlant_2(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_SpikeShootingPlant = Get_SpikeShootingPlant(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        ((*E).sspt_var_D as uint32) << 16 as ::core::ffi::c_int | (*E).sspt_var_C as uint32,
    );
    if (((*E).base.x_pos as ::core::ffi::c_int - (*E).sspt_var_00 as ::core::ffi::c_int) as int16
        as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        (*E).sspt_var_F = (0xa29fec as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).sspt_var_E = 1 as uint16;
    }
    SpikeShootingPlant_4(game);
}
pub unsafe fn SpikeShootingPlant_3(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_SpikeShootingPlant = Get_SpikeShootingPlant(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        ((*E).sspt_var_B as uint32) << 16 as ::core::ffi::c_int | (*E).sspt_var_A as uint32,
    );
    if ((*E).base.x_pos as ::core::ffi::c_int - (*E).sspt_var_01 as ::core::ffi::c_int) as int16
        as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        (*E).sspt_var_F = (0xa29fba as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).sspt_var_E = 0 as uint16;
    }
    SpikeShootingPlant_4(game);
}
pub unsafe fn SpikeShootingPlant_4(game: &mut Game) {
    NextRandom(game);
    let mut E: *mut Enemy_SpikeShootingPlant = Get_SpikeShootingPlant(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*(&raw mut (*E).base.frame_counter as *mut uint8).offset(0) as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) as ::core::ffi::c_int)
        as uint8 as ::core::ffi::c_int
        - 3 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        (*E).sspt_var_F = (0xa2a01b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        if *(&raw mut (*E).sspt_parameter_1 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) != 0
        {
            SpikeShootingPlant_6(game);
        } else {
            SpikeShootingPlant_8(game);
        }
    }
}
pub unsafe fn SpikeShootingPlant_5(game: &mut Game) {
    let mut E: *mut Enemy_SpikeShootingPlant = Get_SpikeShootingPlant(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.current_instruction =
        addr_kSpikeShootingPlant_Ilist_9E8A as ::core::ffi::c_int as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
}
pub unsafe fn SpikeShootingPlant_6(game: &mut Game) {
    let mut E: *mut Enemy_SpikeShootingPlant = Get_SpikeShootingPlant(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.current_instruction =
        addr_kSpikeShootingPlant_Ilist_9EB0 as ::core::ffi::c_int as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
}
pub unsafe fn SpikeShootingPlant_7(game: &mut Game) {
    let mut E: *mut Enemy_SpikeShootingPlant = Get_SpikeShootingPlant(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.current_instruction =
        addr_kSpikeShootingPlant_Ilist_9EDA as ::core::ffi::c_int as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
}
pub unsafe fn SpikeShootingPlant_8(game: &mut Game) {
    let mut E: *mut Enemy_SpikeShootingPlant = Get_SpikeShootingPlant(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.current_instruction =
        addr_kSpikeShootingPlant_Ilist_9F00 as ::core::ffi::c_int as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
}
pub unsafe fn SpikeShootingPlant_Instr_A095(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_SpikeShootingPlant =
        Get_SpikeShootingPlant(k as ::core::ffi::c_int, game);
    (*E).sspt_var_F = (0xa29fba as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    if (*E).sspt_var_E != 0 {
        (*E).sspt_var_F = (0xa29fec as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    return jp;
}
pub unsafe fn SpikeShootingPlant_Instr_A0A7(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    SpawnEprojWithGfx(
        *jp,
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        addr_kEproj_SpikeShootingPlantSpikes as ::core::ffi::c_int as uint16,
        game,
    );
    return jp.offset(1);
}
static off_A2A3D3: [Option<Func_V>; 7] = unsafe {
    [
        Some(MaridiaSpikeyShell_1 as unsafe fn(&mut Game) -> ()),
        Some(MaridiaSpikeyShell_2 as unsafe fn(&mut Game) -> ()),
        Some(MaridiaSpikeyShell_3 as unsafe fn(&mut Game) -> ()),
        Some(MaridiaSpikeyShell_4 as unsafe fn(&mut Game) -> ()),
        Some(MaridiaSpikeyShell_5 as unsafe fn(&mut Game) -> ()),
        Some(MaridiaSpikeyShell_6 as unsafe fn(&mut Game) -> ()),
        Some(MaridiaSpikeyShell_7 as unsafe fn(&mut Game) -> ()),
    ]
};
pub unsafe fn MaridiaSpikeyShell_Init(game: &mut Game) {
    let mut E: *mut Enemy_MaridiaSpikeyShell = Get_MaridiaSpikeyShell(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.spritemap_pointer = addr_kSpritemap_Nothing_A2 as ::core::ffi::c_int as uint16;
    let mut mssl_parameter_1_low: uint16 =
        *(&raw mut (*E).mssl_parameter_1 as *mut uint8).offset(0) as uint16;
    (*E).mssl_var_E = mssl_parameter_1_low;
    off_A2A3D3[(mssl_parameter_1_low as ::core::ffi::c_int & 1 as ::core::ffi::c_int) as usize]
        .expect("non-null function pointer")(game);
    let mut v3: ::core::ffi::c_int = 4 as ::core::ffi::c_int
        * *(&raw mut (*E).mssl_parameter_2 as *mut uint8).offset(0) as ::core::ffi::c_int;
    (*E).mssl_var_B = *(game.g_rom.offset(
        (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v3 as isize);
    (*E).mssl_var_A = *(game.g_rom.offset(
        (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((v3 + 1 as ::core::ffi::c_int) as isize);
    (*E).mssl_var_D = *(game.g_rom.offset(
        (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((v3 + 2 as ::core::ffi::c_int) as isize);
    (*E).mssl_var_C = *(game.g_rom.offset(
        (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((v3 + 3 as ::core::ffi::c_int) as isize);
    let mut v4: ::core::ffi::c_int = *(&raw mut (*E).mssl_parameter_2 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>() as usize)
            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) as ::core::ffi::c_int;
    (*E).mssl_var_02 = (*(game.g_rom.offset(
        (((0xa2a3dd as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa2a3dd as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v4 as isize) as ::core::ffi::c_int
        + (*E).base.x_pos as ::core::ffi::c_int) as uint16;
    (*E).mssl_var_01 = ((*E).base.x_pos as ::core::ffi::c_int
        - *(game.g_rom.offset(
            (((0xa2a3dd as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa2a3dd as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v4 as isize) as ::core::ffi::c_int) as uint16;
    (*E).mssl_var_00 = *(game.g_rom.offset(
        (((0xa2a3ed as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa2a3ed as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            *(&raw mut (*E).mssl_parameter_1 as *mut uint8).offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as isize,
        );
    (*E).mssl_var_F = 0 as uint16;
    if (*E).mssl_var_E as ::core::ffi::c_int == 2 as ::core::ffi::c_int {
        (*E).mssl_var_F = 16 as uint16;
        (*E).base.y_pos =
            ((*E).base.y_pos as ::core::ffi::c_int + 16 as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn MaridiaSpikeyShell_Main(game: &mut Game) {
    let mut e: *mut Enemy_MaridiaSpikeyShell = Get_MaridiaSpikeyShell(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    off_A2A3D3[((*e).mssl_var_E as int16 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as usize]
        .expect("non-null function pointer")(game);
}
pub unsafe fn MaridiaSpikeyShell_1(game: &mut Game) {
    let mut E: *mut Enemy_MaridiaSpikeyShell = Get_MaridiaSpikeyShell(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.current_instruction =
        addr_kMaridiaSpikeyShell_Ilist_A3AB as ::core::ffi::c_int as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
}
pub unsafe fn MaridiaSpikeyShell_2(game: &mut Game) {
    let mut E: *mut Enemy_MaridiaSpikeyShell = Get_MaridiaSpikeyShell(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.current_instruction =
        addr_kMaridiaSpikeyShell_Ilist_A3BD as ::core::ffi::c_int as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
}
pub unsafe fn MaridiaSpikeyShell_3(game: &mut Game) {
    let mut E: *mut Enemy_MaridiaSpikeyShell = Get_MaridiaSpikeyShell(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        ((*E).mssl_var_D as uint32) << 16 as ::core::ffi::c_int | (*E).mssl_var_C as uint32,
    );
    if (((*E).base.x_pos as ::core::ffi::c_int - (*E).mssl_var_01 as ::core::ffi::c_int) as int16
        as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        (*E).mssl_var_E = (*E).mssl_var_E.wrapping_sub(1);
    }
    MaridiaSpikeyShell_8(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
}
pub unsafe fn MaridiaSpikeyShell_4(game: &mut Game) {
    let mut E: *mut Enemy_MaridiaSpikeyShell = Get_MaridiaSpikeyShell(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        ((*E).mssl_var_B as uint32) << 16 as ::core::ffi::c_int | (*E).mssl_var_A as uint32,
    );
    if ((*E).base.x_pos as ::core::ffi::c_int - (*E).mssl_var_02 as ::core::ffi::c_int) as int16
        as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        (*E).mssl_var_E = 0 as uint16;
    }
    MaridiaSpikeyShell_8(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
}
pub unsafe fn MaridiaSpikeyShell_5(game: &mut Game) {
    let mut E: *mut Enemy_MaridiaSpikeyShell = Get_MaridiaSpikeyShell(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v1: uint16 =
        ((*E).mssl_var_00 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
    (*E).mssl_var_00 = v1;
    if v1 == 0 {
        (*E).mssl_var_E = 4 as uint16;
    }
}
pub unsafe fn MaridiaSpikeyShell_6(game: &mut Game) {
    let mut E: *mut Enemy_MaridiaSpikeyShell = Get_MaridiaSpikeyShell(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.y_pos = (*E).base.y_pos.wrapping_add(1);
    (*E).mssl_var_F = (*E).mssl_var_F.wrapping_add(1);
    if (*E).mssl_var_F as ::core::ffi::c_int - 16 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        (*E).mssl_var_E = 2 as uint16;
        (*E).mssl_var_00 = *(game.g_rom.offset(
            (((0xa2a3ed as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa2a3ed as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(
                *(&raw mut (*E).mssl_parameter_1 as *mut uint8).offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ) as isize,
            );
    }
}
pub unsafe fn MaridiaSpikeyShell_7(game: &mut Game) {
    let mut E: *mut Enemy_MaridiaSpikeyShell = Get_MaridiaSpikeyShell(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
    let fresh29 = (*E).mssl_var_F;
    (*E).mssl_var_F = (*E).mssl_var_F.wrapping_sub(1);
    if fresh29 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        (*E).mssl_var_E = (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16
            as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn MaridiaSpikeyShell_8(mut k: uint16, game: &mut Game) {
    NextRandom(game);
    let mut E: *mut Enemy_MaridiaSpikeyShell =
        Get_MaridiaSpikeyShell(k as ::core::ffi::c_int, game);
    if (*(&raw mut (*E).base.frame_counter as *mut uint8).offset(0) as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) as ::core::ffi::c_int)
        as uint8 as ::core::ffi::c_int
        - 6 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        (*E).mssl_var_E = 3 as uint16;
    }
}
pub unsafe fn MaridiaSpikeyShell_Instr_A56D(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_MaridiaSpikeyShell(k as ::core::ffi::c_int, game)).mssl_var_E = 0 as uint16;
    return jp;
}
pub unsafe fn MaridiaSpikeyShell_Instr_A571(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_MaridiaSpikeyShell(k as ::core::ffi::c_int, game)).mssl_var_E = 1 as uint16;
    return jp;
}
pub unsafe fn MaridiaSpikeyShell_Shot(game: &mut Game) {
    let mut E: *mut Enemy_MaridiaSpikeyShell = Get_MaridiaSpikeyShell(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).mssl_var_E as ::core::ffi::c_int - 1 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        Enemy_NormalShotAI_A2(game);
    }
}
pub unsafe fn GunshipTop_Init(game: &mut Game) {
    let mut E: *mut Enemy_GunshipTop = Get_GunshipTop(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        | (kEnemyProps_DisableSamusColl as ::core::ffi::c_int
            | kEnemyProps_Tangible as ::core::ffi::c_int)) as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    (*E).base.current_instruction = addr_kGunshipTop_Ilist_A616 as ::core::ffi::c_int as uint16;
    (*E).base.palette_index = 3584 as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16) as ::core::ffi::c_int
        != 40 as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0x1f55) as *mut uint16)
            as ::core::ffi::c_int
            != 0
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0xd914) as *mut uint16)
            as ::core::ffi::c_int
            == kLoadingGameState_22_EscapingCeres as ::core::ffi::c_int
        {
            (*E).base.y_pos = (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
                as ::core::ffi::c_int
                - 17 as ::core::ffi::c_int) as uint16;
            (*E).gtp_var_F =
                (0xa2a80c as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        } else {
            let mut v3: uint16 =
                ((*E).base.y_pos as ::core::ffi::c_int - 25 as ::core::ffi::c_int) as uint16;
            (*E).base.y_pos = v3;
            (*E).gtp_var_E = v3;
            (*E).gtp_var_F =
                (0xa2a9bd as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) = 1138 as uint16;
        let mut v5: uint16 =
            ((*E).base.y_pos as ::core::ffi::c_int - 25 as ::core::ffi::c_int) as uint16;
        (*E).base.y_pos = v5;
        (*E).gtp_var_E = v5;
        (*E).gtp_var_F = (0xa2ab60 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*Get_GunshipTop(0 as ::core::ffi::c_int, game)).gtp_var_A = 144 as uint16;
    }
    SpawnPalfxObject(addr_stru_8DE1C0 as ::core::ffi::c_int as uint16, game);
    (*Get_GunshipTop(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .gtp_var_D = 1 as uint16;
    (*Get_GunshipTop(0 as ::core::ffi::c_int, game)).gtp_var_C = 0 as uint16;
}
pub unsafe fn GunshipBottom_Init(game: &mut Game) {
    let mut E: *mut Enemy_GunshipBottom = Get_GunshipBottom(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        | (kEnemyProps_DisableSamusColl as ::core::ffi::c_int
            | kEnemyProps_Tangible as ::core::ffi::c_int)) as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    if (*E).gbm_parameter_2 != 0 {
        (*E).base.current_instruction = addr_kGunshipTop_Ilist_A60E as ::core::ffi::c_int as uint16;
    } else {
        (*E).base.current_instruction = addr_kGunshipTop_Ilist_A61C as ::core::ffi::c_int as uint16;
    }
    let mut v2: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0xe54)
        as *mut uint16) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    (*E).base.vram_tiles_index = *((&raw mut game.g_ram as *mut uint8).offset(0xe84)
        as *mut uint16)
        .offset((v2 + 106 as ::core::ffi::c_int) as isize);
    (*E).base.palette_index = 3584 as uint16;
    if (*E).gbm_parameter_2 != 0 {
        (*E).base.y_pos = (*((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v2 + 61 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            - 1 as ::core::ffi::c_int) as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16) as ::core::ffi::c_int
            == kPose_28_FaceL_Crouch as ::core::ffi::c_int
            && *((&raw mut game.g_ram as *mut uint8).offset(0x1f55) as *mut uint16) == 0
        {
            (*E).base.instruction_timer = 1 as uint16;
            (*E).base.current_instruction =
                addr_kGunshipTop_Ilist_A5BE as ::core::ffi::c_int as uint16;
        }
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0xd914) as *mut uint16)
        as ::core::ffi::c_int
        == kGameState_34_CeresGoesBoom as ::core::ffi::c_int
    {
        (*E).base.y_pos = (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
            as ::core::ffi::c_int
            + 23 as ::core::ffi::c_int) as uint16;
    } else {
        (*E).base.y_pos =
            ((*E).base.y_pos as ::core::ffi::c_int + 15 as ::core::ffi::c_int) as uint16;
        (*E).gbm_var_D = 71 as uint16;
    }
    (*E).gbm_var_F = (0xa2a7d7 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn GunshipTop_Main(game: &mut Game) {
    let mut E1: *mut Enemy_GunshipTop = Get_GunshipTop(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    let mut v2: bool = (*E1).gtp_var_D as ::core::ffi::c_int == 1 as ::core::ffi::c_int;
    (*E1).gtp_var_D = (*E1).gtp_var_D.wrapping_sub(1);
    let mut v3: bool = (*E1).gtp_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int;
    if v2 as ::core::ffi::c_int != 0 || v3 as ::core::ffi::c_int != 0 {
        QueueSfx2_Max6(0x4d as uint16, game);
        (*E1).gtp_var_D = 70 as uint16;
    }
    let mut E0: *mut Enemy_GunshipTop = Get_GunshipTop(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E0).gtp_var_F as ::core::ffi::c_int + 0x56be as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
        && (*E0).gtp_var_F as ::core::ffi::c_int + 0x53e5 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
    {
        GunshipTop_1(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        );
    }
    CallEnemyPreInstr(
        ((*E0).gtp_var_F as ::core::ffi::c_int | 0xa20000 as ::core::ffi::c_int) as uint32,
        game,
    );
}
pub unsafe fn GunshipTop_1(mut k: uint16, game: &mut Game) {
    let mut E0: *mut Enemy_GunshipTop = Get_GunshipTop(k as ::core::ffi::c_int, game);
    let mut v3: bool = (*E0).gtp_var_D as ::core::ffi::c_int == 1 as ::core::ffi::c_int;
    (*E0).gtp_var_D = (*E0).gtp_var_D.wrapping_sub(1);
    let mut v4: bool = (*E0).gtp_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int;
    if v3 as ::core::ffi::c_int != 0 || v4 as ::core::ffi::c_int != 0 {
        let mut v5: uint16 =
            (2 as ::core::ffi::c_int * (*E0).gtp_var_C as ::core::ffi::c_int) as uint16;
        (*E0).gtp_var_D = *(game.g_rom.offset(
            (((0xa2a7cf as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa2a7cf as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint8)
            .offset(v5 as isize) as uint16;
        let mut v6: uint16 = *(game.g_rom.offset(
            (((0xa2a7cf as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa2a7cf as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint8)
            .offset((v5 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize)
            as int8 as uint16;
        (*E0).base.y_pos =
            ((*E0).base.y_pos as ::core::ffi::c_int + v6 as ::core::ffi::c_int) as uint16;
        let mut E1: *mut Enemy_GunshipTop =
            Get_GunshipTop(k as ::core::ffi::c_int + 64 as ::core::ffi::c_int, game);
        (*E1).base.y_pos =
            ((*E1).base.y_pos as ::core::ffi::c_int + v6 as ::core::ffi::c_int) as uint16;
        let mut E2: *mut Enemy_GunshipTop =
            Get_GunshipTop(k as ::core::ffi::c_int + 128 as ::core::ffi::c_int, game);
        (*E2).base.y_pos =
            ((*E2).base.y_pos as ::core::ffi::c_int + v6 as ::core::ffi::c_int) as uint16;
        (*E0).gtp_var_C = ((*E0).gtp_var_C as ::core::ffi::c_int + 1 as ::core::ffi::c_int
            & 3 as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn GunshipTop_2(mut k: uint16, game: &mut Game) {
    let ref mut fresh8 = *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16);
    *fresh8 = (*fresh8 as ::core::ffi::c_int - 8 as ::core::ffi::c_int) as uint16;
    let mut E2: *mut Enemy_GunshipTop =
        Get_GunshipTop(k as ::core::ffi::c_int + 128 as ::core::ffi::c_int, game);
    (*E2).base.y_pos = ((*E2).base.y_pos as ::core::ffi::c_int - 8 as ::core::ffi::c_int) as uint16;
    let mut E1: *mut Enemy_GunshipTop =
        Get_GunshipTop(k as ::core::ffi::c_int + 64 as ::core::ffi::c_int, game);
    (*E1).base.y_pos = ((*E1).base.y_pos as ::core::ffi::c_int - 8 as ::core::ffi::c_int) as uint16;
    let mut E0: *mut Enemy_GunshipTop = Get_GunshipTop(k as ::core::ffi::c_int, game);
    let mut v4: uint16 =
        ((*E0).base.y_pos as ::core::ffi::c_int - 8 as ::core::ffi::c_int) as uint16;
    (*E0).base.y_pos = v4;
    if v4 as ::core::ffi::c_int - 128 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        (*E0).gtp_var_F = (0xa2a80c as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn GunshipTop_3(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_GunshipTop = Get_GunshipTop(k as ::core::ffi::c_int, game);
    if (*E).base.y_pos as ::core::ffi::c_int - 768 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        AddToHiLo(
            (&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16,
            (&raw mut game.g_ram as *mut uint8).offset(0xafc) as *mut uint16,
            0x48000 as uint32,
        );
        let mut v3: *mut Enemy_GunshipTop =
            Get_GunshipTop(k as ::core::ffi::c_int + 128 as ::core::ffi::c_int, game);
        AddToHiLo(
            &raw mut (*v3).base.y_pos,
            &raw mut (*v3).base.y_subpos,
            0x48000 as uint32,
        );
        let mut v5: *mut Enemy_GunshipTop =
            Get_GunshipTop(k as ::core::ffi::c_int + 64 as ::core::ffi::c_int, game);
        AddToHiLo(
            &raw mut (*v5).base.y_pos,
            &raw mut (*v5).base.y_subpos,
            0x48000 as uint32,
        );
        AddToHiLo(
            &raw mut (*E).base.y_pos,
            &raw mut (*E).base.y_subpos,
            0x48000 as uint32,
        );
    } else {
        AddToHiLo(
            (&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16,
            (&raw mut game.g_ram as *mut uint8).offset(0xafc) as *mut uint16,
            0x28000 as uint32,
        );
        let mut v8: *mut Enemy_GunshipTop =
            Get_GunshipTop(k as ::core::ffi::c_int + 128 as ::core::ffi::c_int, game);
        AddToHiLo(
            &raw mut (*v8).base.y_pos,
            &raw mut (*v8).base.y_subpos,
            0x28000 as uint32,
        );
        let mut v10: *mut Enemy_GunshipTop =
            Get_GunshipTop(k as ::core::ffi::c_int + 64 as ::core::ffi::c_int, game);
        AddToHiLo(
            &raw mut (*v10).base.y_pos,
            &raw mut (*v10).base.y_subpos,
            0x28000 as uint32,
        );
        AddToHiLo(
            &raw mut (*E).base.y_pos,
            &raw mut (*E).base.y_subpos,
            0x28000 as uint32,
        );
        if (*E).base.y_pos as ::core::ffi::c_int - 1119 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            (*E).base.y_pos = 1119 as uint16;
            (*v10).base.y_pos = 1159 as uint16;
            (*v8).base.y_pos =
                ((*E).base.y_pos as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
            (*E).gtp_var_F =
                (0xa2a8d0 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            (*E).gtp_var_E = 0 as uint16;
        }
    };
}
pub unsafe fn GunshipTop_4(mut k: uint16, game: &mut Game) {
    let mut v2: ::core::ffi::c_int = 0;
    let mut v5: uint16 = 0;
    let mut GunshipTop: *mut Enemy_GunshipTop = Get_GunshipTop(k as ::core::ffi::c_int, game);
    v2 = (*GunshipTop).gtp_var_E as ::core::ffi::c_int;
    let ref mut fresh9 = *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16);
    *fresh9 = (*fresh9 as ::core::ffi::c_int
        + *(game.g_rom.offset(
            (((0xa2a622 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa2a622 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v2 as isize) as ::core::ffi::c_int) as uint16;
    let mut v3: *mut Enemy_GunshipTop =
        Get_GunshipTop(k as ::core::ffi::c_int + 128 as ::core::ffi::c_int, game);
    (*v3).base.y_pos = ((*v3).base.y_pos as ::core::ffi::c_int
        + *(game.g_rom.offset(
            (((0xa2a622 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa2a622 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v2 as isize) as ::core::ffi::c_int) as uint16;
    let mut v4: *mut Enemy_GunshipTop =
        Get_GunshipTop(k as ::core::ffi::c_int + 64 as ::core::ffi::c_int, game);
    (*v4).base.y_pos = ((*v4).base.y_pos as ::core::ffi::c_int
        + *(game.g_rom.offset(
            (((0xa2a622 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa2a622 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v2 as isize) as ::core::ffi::c_int) as uint16;
    (*GunshipTop).base.y_pos = ((*GunshipTop).base.y_pos as ::core::ffi::c_int
        + *(game.g_rom.offset(
            (((0xa2a622 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa2a622 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v2 as isize) as ::core::ffi::c_int) as uint16;
    v5 = ((*GunshipTop).gtp_var_E as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
    (*GunshipTop).gtp_var_E = v5;
    if v5 as ::core::ffi::c_int - 17 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
        (*GunshipTop).gtp_var_F =
            (0xa2a942 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*GunshipTop).gtp_var_E = (*GunshipTop).base.y_pos;
        (*GunshipTop).gtp_var_D = 1 as uint16;
        let mut v6: *mut Enemy_GunshipTop = Get_GunshipTop(0 as ::core::ffi::c_int, game);
        (*v6).gtp_var_C = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) =
            ((*GunshipTop).base.x_pos as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb10) as *mut uint16) =
            *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16);
        let mut v7: *mut Enemy_GunshipTop =
            Get_GunshipTop(k as ::core::ffi::c_int + 128 as ::core::ffi::c_int, game);
        (*v7).base.instruction_timer = 1 as uint16;
        (*v7).base.current_instruction =
            addr_kGunshipTop_Ilist_A5BE as ::core::ffi::c_int as uint16;
        (*v6).gtp_var_A = 144 as uint16;
        QueueSfx3_Max6(0x14 as uint16, game);
    }
}
pub unsafe fn GunshipTop_5(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_GunshipTop = Get_GunshipTop(0 as ::core::ffi::c_int, game);
    let mut v2: bool = (*E).gtp_var_A as ::core::ffi::c_int == 1 as ::core::ffi::c_int;
    (*E).gtp_var_A = (*E).gtp_var_A.wrapping_sub(1);
    let mut v3: bool = (*E).gtp_var_A as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int;
    if v2 as ::core::ffi::c_int != 0 || v3 as ::core::ffi::c_int != 0 {
        (*Get_GunshipTop(k as ::core::ffi::c_int, game)).gtp_var_F =
            (0xa2a950 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn GunshipTop_6(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_GunshipTop = Get_GunshipTop(k as ::core::ffi::c_int, game);
    let mut r18: uint16 =
        ((*E).gtp_var_E as ::core::ffi::c_int - 30 as ::core::ffi::c_int) as uint16;
    let ref mut fresh10 = *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16);
    *fresh10 = (*fresh10).wrapping_sub(1);
    if *fresh10 as ::core::ffi::c_int - r18 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0
    {
        (*E).gtp_var_F = (0xa2a987 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        let mut E2: *mut Enemy_GunshipTop =
            Get_GunshipTop(k as ::core::ffi::c_int + 128 as ::core::ffi::c_int, game);
        (*E2).base.instruction_timer = 1 as uint16;
        (*E2).base.current_instruction =
            addr_kGunshipTop_Ilist_A5EE as ::core::ffi::c_int as uint16;
        (*Get_GunshipTop(0 as ::core::ffi::c_int, game)).gtp_var_A = 144 as uint16;
        QueueSfx3_Max6(0x15 as uint16, game);
    }
}
pub unsafe fn GunshipTop_7(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_GunshipTop = Get_GunshipTop(0 as ::core::ffi::c_int, game);
    let mut v2: bool = (*E).gtp_var_A as ::core::ffi::c_int == 1 as ::core::ffi::c_int;
    (*E).gtp_var_A = (*E).gtp_var_A.wrapping_sub(1);
    let mut v3: bool = (*E).gtp_var_A as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int;
    if v2 as ::core::ffi::c_int != 0 || v3 as ::core::ffi::c_int != 0 {
        (*Get_GunshipTop(k as ::core::ffi::c_int, game)).gtp_var_F =
            (0xa2a9bd as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xa42) as *mut uint16) =
            (0x90e695 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xa44) as *mut uint16) =
            (0x90e725 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xd914) as *mut uint16) =
            kLoadingGameState_5_Main as ::core::ffi::c_int as uint16;
        let ref mut fresh11 = *((&raw mut game.g_ram as *mut uint8).offset(0xd8f8) as *mut uint16);
        *fresh11 = (*fresh11 as ::core::ffi::c_int | 1 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x78b) as *mut uint16) = 0 as uint16;
        SaveToSram(
            *((&raw mut game.g_ram as *mut uint8).offset(0x952) as *mut uint16),
            game,
        );
    }
}
pub unsafe fn GunshipTop_8(mut k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16) as ::core::ffi::c_int
        == 8 as ::core::ffi::c_int
        && *((&raw mut game.g_ram as *mut uint8).offset(0xa42) as *mut uint16) as ::core::ffi::c_int
            == 0x90e695 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        let mut E: *mut Enemy_GunshipTop = Get_GunshipTop(k as ::core::ffi::c_int, game);
        if (((*E).base.x_pos as ::core::ffi::c_int
            - 8 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
            && ((*E).base.x_pos as ::core::ffi::c_int + 8 as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                    as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int
            && (((*E).base.y_pos as ::core::ffi::c_int
                - 64 as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
                    as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int
            && ((*E).base.y_pos as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
                    as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int
            && *(&raw mut game.g_ram as *mut uint8).offset(0xa1f) == 0
            && *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16)
                as ::core::ffi::c_int
                & kButton_Down as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
        {
            (*E).gtp_var_F =
                (0xa2aa4f as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            if *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                as ::core::ffi::c_int
                != 1152 as ::core::ffi::c_int
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) =
                    (*E).base.x_pos;
                *((&raw mut game.g_ram as *mut uint8).offset(0xb10) as *mut uint16) =
                    *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16);
            }
            MakeSamusFaceForward(game);
            CallSomeSamusCode(0x1a as uint16, game);
            *((&raw mut game.g_ram as *mut uint8).offset(0xe18) as *mut uint16) = 0 as uint16;
            let mut v3: uint16 =
                ((*E).base.y_pos as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
            let mut E2: *mut Enemy_GunshipTop =
                Get_GunshipTop(k as ::core::ffi::c_int + 128 as ::core::ffi::c_int, game);
            (*E2).base.y_pos = v3;
            (*E2).base.instruction_timer = 1 as uint16;
            (*E2).base.current_instruction =
                addr_kGunshipTop_Ilist_A5BE as ::core::ffi::c_int as uint16;
            (*Get_GunshipTop(0 as ::core::ffi::c_int, game)).gtp_var_A = 144 as uint16;
            QueueSfx3_Max6(0x14 as uint16, game);
        }
    }
}
pub unsafe fn GunshipTop_9(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_GunshipTop = Get_GunshipTop(0 as ::core::ffi::c_int, game);
    let mut v2: bool = (*E).gtp_var_A as ::core::ffi::c_int == 1 as ::core::ffi::c_int;
    (*E).gtp_var_A = (*E).gtp_var_A.wrapping_sub(1);
    let mut v3: bool = (*E).gtp_var_A as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int;
    if v2 as ::core::ffi::c_int != 0 || v3 as ::core::ffi::c_int != 0 {
        (*Get_GunshipTop(k as ::core::ffi::c_int, game)).gtp_var_F =
            (0xa2aa5d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn GunshipTop_10(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_GunshipTop = Get_GunshipTop(k as ::core::ffi::c_int, game);
    let mut r18: uint16 =
        ((*E).gtp_var_E as ::core::ffi::c_int + 18 as ::core::ffi::c_int) as uint16;
    let ref mut fresh2 = *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16);
    *fresh2 = (*fresh2 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) as ::core::ffi::c_int
        - r18 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        (*E).gtp_var_F = (0xa2aa94 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        let mut E2: *mut Enemy_GunshipTop =
            Get_GunshipTop(k as ::core::ffi::c_int + 128 as ::core::ffi::c_int, game);
        (*E2).base.instruction_timer = 1 as uint16;
        (*E2).base.current_instruction =
            addr_kGunshipTop_Ilist_A5EE as ::core::ffi::c_int as uint16;
        (*Get_GunshipTop(0 as ::core::ffi::c_int, game)).gtp_var_A = 144 as uint16;
        QueueSfx3_Max6(0x15 as uint16, game);
    }
}
pub unsafe fn GunshipTop_11(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_GunshipTop = Get_GunshipTop(0 as ::core::ffi::c_int, game);
    let mut v2: bool = (*E).gtp_var_A as ::core::ffi::c_int == 1 as ::core::ffi::c_int;
    (*E).gtp_var_A = (*E).gtp_var_A.wrapping_sub(1);
    let mut v3: bool = (*E).gtp_var_A as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int;
    if v2 as ::core::ffi::c_int != 0 || v3 as ::core::ffi::c_int != 0 {
        (*Get_GunshipTop(k as ::core::ffi::c_int, game)).gtp_var_F =
            (0xa2aaa2 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn GunshipTop_12(mut k: uint16, game: &mut Game) {
    if CheckEventHappened(0xe as uint16, game) != 0 {
        (*Get_GunshipTop(k as ::core::ffi::c_int, game)).gtp_var_F =
            (0xa2abc7 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        let mut E: *mut Enemy_GunshipTop =
            Get_GunshipTop(k as ::core::ffi::c_int + 64 as ::core::ffi::c_int, game);
        (*E).gtp_var_F = 0 as uint16;
        (*E).gtp_var_E = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xdec) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xdee) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xdf0) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xdf2) as *mut uint16) = 0 as uint16;
        CallSomeSamusCode(0xa as uint16, game);
    } else {
        Samus_RestoreHealth(2 as uint16, game);
        Samus_RestoreMissiles(2 as uint16, game);
        Samus_RestoreSuperMissiles(2 as uint16, game);
        Samus_RestorePowerBombs(2 as uint16, game);
        if (*((&raw mut game.g_ram as *mut uint8).offset(0x9d6) as *mut uint16)
            as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x9d4) as *mut uint16)
                as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
            && (*((&raw mut game.g_ram as *mut uint8).offset(0x9c2) as *mut uint16)
                as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0x9c4) as *mut uint16)
                    as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int
            && (*((&raw mut game.g_ram as *mut uint8).offset(0x9c6) as *mut uint16)
                as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0x9c8) as *mut uint16)
                    as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int
            && (*((&raw mut game.g_ram as *mut uint8).offset(0x9ca) as *mut uint16)
                as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0x9cc) as *mut uint16)
                    as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int
            && (*((&raw mut game.g_ram as *mut uint8).offset(0x9ce) as *mut uint16)
                as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0x9d0) as *mut uint16)
                    as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int
        {
            (*Get_GunshipTop(k as ::core::ffi::c_int, game)).gtp_var_F =
                (0xa2ab1f as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
    };
}
pub unsafe fn GunshipTop_13(mut k: uint16, game: &mut Game) {
    let mut rv: ::core::ffi::c_int = DisplayMessageBox_Poll(0x1c as uint16, game);
    if rv < 0 as ::core::ffi::c_int {
        return;
    }
    if rv != 2 as ::core::ffi::c_int {
        let ref mut fresh3 = *((&raw mut game.g_ram as *mut uint8).offset(0xd8f8) as *mut uint16);
        *fresh3 = (*fresh3 as ::core::ffi::c_int | 1 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x78b) as *mut uint16) = 0 as uint16;
        SaveToSram(
            *((&raw mut game.g_ram as *mut uint8).offset(0x952) as *mut uint16),
            game,
        );
    }
    (*Get_GunshipTop(k as ::core::ffi::c_int, game)).gtp_var_F =
        (0xa2ab60 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    let mut E: *mut Enemy_GunshipTop =
        Get_GunshipTop(k as ::core::ffi::c_int + 128 as ::core::ffi::c_int, game);
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.current_instruction = addr_kGunshipTop_Ilist_A5BE as ::core::ffi::c_int as uint16;
    (*Get_GunshipTop(0 as ::core::ffi::c_int, game)).gtp_var_A = 144 as uint16;
    QueueSfx3_Max6(0x14 as uint16, game);
}
pub unsafe fn GunshipTop_14(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_GunshipTop = Get_GunshipTop(0 as ::core::ffi::c_int, game);
    let mut v2: bool = (*E).gtp_var_A as ::core::ffi::c_int == 1 as ::core::ffi::c_int;
    (*E).gtp_var_A = (*E).gtp_var_A.wrapping_sub(1);
    let mut v3: bool = (*E).gtp_var_A as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int;
    if v2 as ::core::ffi::c_int != 0 || v3 as ::core::ffi::c_int != 0 {
        (*Get_GunshipTop(k as ::core::ffi::c_int, game)).gtp_var_F =
            (0xa2ab6e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn GunshipTop_15(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_GunshipTop = Get_GunshipTop(k as ::core::ffi::c_int, game);
    let mut r18: uint16 =
        ((*E).gtp_var_E as ::core::ffi::c_int - 30 as ::core::ffi::c_int) as uint16;
    let ref mut fresh4 = *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16);
    *fresh4 = (*fresh4 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) as ::core::ffi::c_int
        - r18 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        (*E).gtp_var_F = (0xa2aba5 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        let mut E2: *mut Enemy_GunshipTop =
            Get_GunshipTop(k as ::core::ffi::c_int + 128 as ::core::ffi::c_int, game);
        (*E2).base.instruction_timer = 1 as uint16;
        (*E2).base.current_instruction =
            addr_kGunshipTop_Ilist_A5EE as ::core::ffi::c_int as uint16;
        (*Get_GunshipTop(0 as ::core::ffi::c_int, game)).gtp_var_A = 144 as uint16;
        QueueSfx3_Max6(0x15 as uint16, game);
    }
}
pub unsafe fn GunshipTop_16(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_GunshipTop = Get_GunshipTop(0 as ::core::ffi::c_int, game);
    let mut v2: bool = (*E).gtp_var_A as ::core::ffi::c_int == 1 as ::core::ffi::c_int;
    (*E).gtp_var_A = (*E).gtp_var_A.wrapping_sub(1);
    let mut v3: bool = (*E).gtp_var_A as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int;
    if v2 as ::core::ffi::c_int != 0 || v3 as ::core::ffi::c_int != 0 {
        (*Get_GunshipTop(k as ::core::ffi::c_int, game)).gtp_var_F =
            (0xa2a9bd as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16) as ::core::ffi::c_int
            - kGameState_40_TransitionToDemo as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa42) as *mut uint16) =
                (0x90e695 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xa44) as *mut uint16) =
                (0x90e725 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
    }
}
pub unsafe fn GunshipTop_17(mut k: uint16, game: &mut Game) {
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xdec) as *mut uint16);
    let mut v2: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
    (*gVramWriteEntry(
        *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .size = 1024 as uint16;
    v2 = (v2 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    let mut v3: ::core::ffi::c_int = v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    (*gVramWriteEntry(v2 as ::core::ffi::c_int, game)).size = *(game.g_rom.offset(
        (((0xa2ac07 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa2ac07 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8
        as *mut uint16)
        .offset(v3 as isize);
    v2 = (v2 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    let fresh5 = v2;
    v2 = v2.wrapping_add(1);
    *(&raw mut (*(gVramWriteEntry
        as unsafe fn(::core::ffi::c_int, &mut Game) -> *mut VramWriteEntry)(
        fresh5 as ::core::ffi::c_int,
        game,
    ))
    .size as *mut uint8)
        .offset(0) = -(108 as ::core::ffi::c_int) as uint8;
    (*gVramWriteEntry(v2 as ::core::ffi::c_int, game)).size = *(game.g_rom.offset(
        (((0xa2ac11 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa2ac11 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8
        as *mut uint16)
        .offset(v3 as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
        (v2 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    let ref mut fresh6 = *((&raw mut game.g_ram as *mut uint8).offset(0xdec) as *mut uint16);
    *fresh6 = (*fresh6 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xdec) as *mut uint16) as ::core::ffi::c_int
        - 10 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        (*Get_GunshipTop(k as ::core::ffi::c_int, game)).gtp_var_F =
            (0xa2ac1b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xdec) as *mut uint16) = 0 as uint16;
    }
}
pub unsafe fn GunshipTop_18(mut k: uint16, game: &mut Game) {
    let mut v5: int16 = 0;
    let mut E1: *mut Enemy_GunshipTop =
        Get_GunshipTop(k as ::core::ffi::c_int + 64 as ::core::ffi::c_int, game);
    let mut gtp_var_E: uint16 = (*E1).gtp_var_E;
    if gtp_var_E as ::core::ffi::c_int - 64 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v3: uint16 = 0;
        if gtp_var_E as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            v3 = (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
                as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as uint16;
        } else {
            v3 = (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
                as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int) as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) = v3;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb14) as *mut uint16) = v3;
    } else {
        let mut v4: uint16 = 0;
        if gtp_var_E as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            v4 = (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
                as ::core::ffi::c_int
                + 2 as ::core::ffi::c_int) as uint16;
        } else {
            v4 = (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
                as ::core::ffi::c_int
                - 2 as ::core::ffi::c_int) as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) = v4;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb14) as *mut uint16) = v4;
    }
    v5 = (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) as ::core::ffi::c_int
        - 17 as ::core::ffi::c_int) as int16;
    let mut E0: *mut Enemy_GunshipTop = Get_GunshipTop(k as ::core::ffi::c_int, game);
    (*E0).base.y_pos = (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
        as ::core::ffi::c_int
        - 17 as ::core::ffi::c_int) as uint16;
    (*Get_GunshipTop(k as ::core::ffi::c_int + 128 as ::core::ffi::c_int, game))
        .base
        .y_pos = (v5 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
    (*E1).base.y_pos = (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
        as ::core::ffi::c_int
        + 23 as ::core::ffi::c_int) as uint16;
    let mut v7: uint16 =
        ((*E1).gtp_var_E as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
    (*E1).gtp_var_E = v7;
    if v7 as ::core::ffi::c_int - 128 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        if v7 as ::core::ffi::c_int == 64 as ::core::ffi::c_int {
            SpawnEprojWithRoomGfx(
                addr_stru_86A379 as ::core::ffi::c_int as uint16,
                0 as uint16,
                game,
            );
            SpawnEprojWithRoomGfx(
                addr_stru_86A379 as ::core::ffi::c_int as uint16,
                2 as uint16,
                game,
            );
            SpawnEprojWithRoomGfx(
                addr_stru_86A379 as ::core::ffi::c_int as uint16,
                4 as uint16,
                game,
            );
            SpawnEprojWithRoomGfx(
                addr_stru_86A379 as ::core::ffi::c_int as uint16,
                6 as uint16,
                game,
            );
            SpawnEprojWithRoomGfx(
                addr_stru_86A379 as ::core::ffi::c_int as uint16,
                8 as uint16,
                game,
            );
            SpawnEprojWithRoomGfx(
                addr_stru_86A379 as ::core::ffi::c_int as uint16,
                0xa as uint16,
                game,
            );
        }
    } else {
        (*E0).gtp_var_F = (0xa2acd7 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*Get_GunshipTop(0 as ::core::ffi::c_int, game)).gtp_var_A = 0 as uint16;
    };
}
pub unsafe fn GunshipTop_19(mut k: uint16, game: &mut Game) {
    let mut v1: int16 = 0;
    let ref mut fresh7 = *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16);
    *fresh7 = (*fresh7 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
    v1 = (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) as ::core::ffi::c_int
        - 17 as ::core::ffi::c_int) as int16;
    let mut E0: *mut Enemy_GunshipTop = Get_GunshipTop(k as ::core::ffi::c_int, game);
    (*E0).base.y_pos = (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
        as ::core::ffi::c_int
        - 17 as ::core::ffi::c_int) as uint16;
    (*Get_GunshipTop(k as ::core::ffi::c_int + 128 as ::core::ffi::c_int, game))
        .base
        .y_pos = (v1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
    let mut E1: *mut Enemy_GunshipTop =
        Get_GunshipTop(k as ::core::ffi::c_int + 64 as ::core::ffi::c_int, game);
    (*E1).base.y_pos = (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
        as ::core::ffi::c_int
        + 23 as ::core::ffi::c_int) as uint16;
    if (*E0).base.y_pos as ::core::ffi::c_int - 896 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        (*E0).gtp_var_F = (0xa2ad0e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E1).gtp_var_F = 512 as uint16;
    }
}
pub unsafe fn GunshipTop_20(mut k: uint16, game: &mut Game) {
    GunshipTop_21(k, game);
    let mut E: *mut Enemy_GunshipTop = Get_GunshipTop(k as ::core::ffi::c_int, game);
    if (*E).base.y_pos as ::core::ffi::c_int - 256 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        (*E).gtp_var_F = (0xa2ad2d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16) = 38 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 0 as uint16;
    }
}
pub unsafe fn GunshipTop_21(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_GunshipTop =
        Get_GunshipTop(k as ::core::ffi::c_int + 64 as ::core::ffi::c_int, game);
    let mut v3: uint16 =
        ((*E).gtp_var_F as ::core::ffi::c_int + 64 as ::core::ffi::c_int) as uint16;
    (*E).gtp_var_F = v3;
    if (v3 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int) - 2560 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        (*E).gtp_var_F = 2304 as uint16;
    }
    let mut v6: uint32 = ((*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
        as uint32)
        << 16 as ::core::ffi::c_int
        | *((&raw mut game.g_ram as *mut uint8).offset(0xafc) as *mut uint16) as uint32)
        .wrapping_sub(
            (((*E).gtp_var_F as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as uint32)
                << 16 as ::core::ffi::c_int
                | (((*E).gtp_var_F as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint16
                    as uint32,
        );
    *((&raw mut game.g_ram as *mut uint8).offset(0xafc) as *mut uint16) = v6 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) =
        (v6 >> 16 as ::core::ffi::c_int) as uint16;
    let mut v5: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
        as ::core::ffi::c_int
        - 17 as ::core::ffi::c_int) as uint16;
    (*Get_GunshipTop(k as ::core::ffi::c_int, game)).base.y_pos = v5;
    (*Get_GunshipTop(k as ::core::ffi::c_int + 128 as ::core::ffi::c_int, game))
        .base
        .y_pos = (v5 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
    (*E).base.y_pos = (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
        as ::core::ffi::c_int
        + 23 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Flies_Init(game: &mut Game) {
    let mut E: *mut Enemy_Flies = Get_Flies(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).flies_var_E = 0 as uint16;
    (*E).flies_var_F = (0xa2b14e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).base.current_instruction = addr_kFlies_Ilist_B013 as ::core::ffi::c_int as uint16;
    (*E).base.spritemap_pointer = addr_kSpritemap_Nothing_A2 as ::core::ffi::c_int as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.spritemap_pointer = addr_kSpritemap_Nothing_A2 as ::core::ffi::c_int as uint16;
}
pub unsafe fn CarryAdd16(mut rv: *mut uint16, mut v: uint16) -> bool {
    let mut i: ::core::ffi::c_int = *rv as ::core::ffi::c_int + v as ::core::ffi::c_int;
    *rv = i as uint16;
    return i >> 16 as ::core::ffi::c_int != 0;
}
pub unsafe fn Flies_1(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Flies = Get_Flies(k as ::core::ffi::c_int, game);
    let mut v2: ::core::ffi::c_int =
        (*E).flies_var_E as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut carry: bool = CarryAdd16(
        &raw mut (*E).base.x_subpos,
        ((*((&raw const kSinCosTable8bit_Sext as *const int16)
            .offset((v2 + 64 as ::core::ffi::c_int) as isize) as *const int16
            as *mut uint8)
            .offset(0) as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint16,
    );
    (*E).base.x_pos = ((*E).base.x_pos as ::core::ffi::c_int
        + (*((&raw const kSinCosTable8bit_Sext as *const int16)
            .offset((v2 + 64 as ::core::ffi::c_int) as isize) as *const int16
            as *mut uint8)
            .offset(
                (::core::mem::size_of::<int16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as int8 as ::core::ffi::c_int
            + carry as ::core::ffi::c_int)) as uint16;
    carry = CarryAdd16(
        &raw mut (*E).base.y_subpos,
        ((*((&raw const kSinCosTable8bit_Sext as *const int16).offset(v2 as isize) as *const int16
            as *mut uint8)
            .offset(0) as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint16,
    );
    (*E).base.y_pos = ((*E).base.y_pos as ::core::ffi::c_int
        + (*((&raw const kSinCosTable8bit_Sext as *const int16).offset(v2 as isize) as *const int16
            as *mut uint8)
            .offset(
                (::core::mem::size_of::<int16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as int8 as ::core::ffi::c_int
            + carry as ::core::ffi::c_int)) as uint16;
}
pub unsafe fn Flies_2(mut k: uint16, game: &mut Game) -> uint16 {
    let mut E: *mut Enemy_Flies = Get_Flies(k as ::core::ffi::c_int, game);
    let mut carry: bool = CarryAdd16(
        &raw mut (*E).base.x_subpos,
        ((*(&raw mut (*E).flies_var_B as *mut uint8).offset(0) as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint16,
    );
    (*E).base.x_pos = ((*E).base.x_pos as ::core::ffi::c_int
        + (*(&raw mut (*E).flies_var_B as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as int8 as ::core::ffi::c_int
            + carry as ::core::ffi::c_int)) as uint16;
    carry = CarryAdd16(
        &raw mut (*E).base.y_subpos,
        ((*(&raw mut (*E).flies_var_C as *mut uint8).offset(0) as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint16,
    );
    let mut result: uint16 = ((*E).base.y_pos as ::core::ffi::c_int
        + *(&raw mut (*E).flies_var_C as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as int8 as ::core::ffi::c_int
        + carry as ::core::ffi::c_int) as uint16;
    (*E).base.y_pos = result;
    return result;
}
pub unsafe fn Flies_Main(game: &mut Game) {
    NextRandom(game);
    let mut E: *mut Enemy_Flies = Get_Flies(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallEnemyPreInstr(
        ((*E).flies_var_F as ::core::ffi::c_int | 0xa20000 as ::core::ffi::c_int) as uint32,
        game,
    );
}
pub unsafe fn Flies_3(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = CalculateAngleOfSamusFromEnemy(k, game) as ::core::ffi::c_int;
    let mut E: *mut Enemy_Flies = Get_Flies(k as ::core::ffi::c_int, game);
    (*E).flies_var_B = (2 as ::core::ffi::c_int
        * kSinCosTable8bit_Sext[(v1 + 64 as ::core::ffi::c_int) as usize] as ::core::ffi::c_int)
        as uint16;
    (*E).flies_var_C = (4 as ::core::ffi::c_int
        * kSinCosTable8bit_Sext[v1 as usize] as ::core::ffi::c_int)
        as uint16;
    (*E).flies_var_D = *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16);
    (*E).flies_var_F = (0xa2b1aa as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Flies_4(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Flies = Get_Flies(k as ::core::ffi::c_int, game);
    let mut flies_var_A: uint16 = (*E).flies_var_A;
    if flies_var_A != 0 {
        (*E).flies_var_A = (flies_var_A as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
    } else if CompareDistToSamus_X(k, 0x70 as uint16, game) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        == 0
    {
        Flies_3(k, game);
        return;
    }
    Flies_1(k, game);
    let mut v3: bool = (*E).flies_var_E as ::core::ffi::c_int + 32 as ::core::ffi::c_int
        & 0x1ff as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int;
    (*E).flies_var_E = ((*E).flies_var_E as ::core::ffi::c_int + 32 as ::core::ffi::c_int
        & 0x1ff as ::core::ffi::c_int) as uint16;
    if v3 {
        (*E).flies_var_F =
            (0xa2b17c as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Flies_5(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Flies = Get_Flies(k as ::core::ffi::c_int, game);
    let mut flies_var_A: uint16 = (*E).flies_var_A;
    if flies_var_A != 0 {
        (*E).flies_var_A = (flies_var_A as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
    } else if CompareDistToSamus_X(k, 0x70 as uint16, game) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        == 0
    {
        Flies_3(k, game);
        return;
    }
    Flies_1(k, game);
    let mut v3: bool = (*E).flies_var_E as ::core::ffi::c_int - 32 as ::core::ffi::c_int
        & 0x1ff as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int;
    (*E).flies_var_E = ((*E).flies_var_E as ::core::ffi::c_int - 32 as ::core::ffi::c_int
        & 0x1ff as ::core::ffi::c_int) as uint16;
    if v3 {
        (*E).flies_var_F =
            (0xa2b14e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Flies_6(mut k: uint16, game: &mut Game) {
    let mut current_block: u64;
    let mut v1: uint16 = Flies_2(k, game);
    let mut E: *mut Enemy_Flies = Get_Flies(k as ::core::ffi::c_int, game);
    (*E).flies_var_A = (*E).flies_var_A.wrapping_add(1);
    if (*E).flies_var_C as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        if (v1 as ::core::ffi::c_int) < (*E).flies_var_D as ::core::ffi::c_int {
            return;
        }
        current_block = 6833738745606017744;
    } else if (v1 as ::core::ffi::c_int) < (*E).flies_var_D as ::core::ffi::c_int {
        current_block = 6833738745606017744;
    } else {
        current_block = 10879442775620481940;
    }
    match current_block {
        6833738745606017744 => {
            (*E).flies_var_C = -((*E).flies_var_C as ::core::ffi::c_int) as uint16;
            (*E).flies_var_F =
                (0xa2b1d2 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
        _ => {}
    };
}
pub unsafe fn Flies_7(mut k: uint16, game: &mut Game) {
    Flies_2(k, game);
    let mut E: *mut Enemy_Flies = Get_Flies(k as ::core::ffi::c_int, game);
    (*E).flies_var_A = (*E).flies_var_A.wrapping_sub(1);
    if (*E).flies_var_A as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).flies_var_A = 24 as uint16;
        (*E).flies_var_F =
            (0xa2b14e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn NorfairErraticFireball_Init(game: &mut Game) {
    let mut E: *mut Enemy_NorfairErraticFireball = Get_NorfairErraticFireball(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut pt: Point32 = ConvertAngleToXy(
        (*E).nefl_parameter_1,
        *(&raw mut (*E).nefl_parameter_2 as *mut uint8).offset(0) as uint16,
        game,
    );
    (*E).nefl_var_C = (pt.x >> 16 as ::core::ffi::c_int) as uint16;
    (*E).nefl_var_D = pt.x as uint16;
    (*E).nefl_var_E = (pt.y >> 16 as ::core::ffi::c_int) as uint16;
    (*E).nefl_var_F = pt.y as uint16;
    (*E).base.current_instruction =
        addr_kNorfairErraticFireball_Ilist_B2DC as ::core::ffi::c_int as uint16;
}
pub unsafe fn NorfairErraticFireball_Main(game: &mut Game) {
    let mut E: *mut Enemy_NorfairErraticFireball = Get_NorfairErraticFireball(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut pt: Point32 = ConvertAngleToXy(
        (*E).nefl_parameter_1,
        *(&raw mut (*E).nefl_parameter_2 as *mut uint8).offset(0) as uint16,
        game,
    );
    (*E).nefl_var_C = (pt.x >> 16 as ::core::ffi::c_int) as uint16;
    (*E).nefl_var_D = pt.x as uint16;
    (*E).nefl_var_E = (pt.y >> 16 as ::core::ffi::c_int) as uint16;
    (*E).nefl_var_F = pt.y as uint16;
    if (*E).nefl_parameter_1 as ::core::ffi::c_int + 64 as ::core::ffi::c_int
        & 0x80 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        (*E).nefl_var_C = !((*E).nefl_var_C as ::core::ffi::c_int) as uint16;
        (*E).nefl_var_D = -((*E).nefl_var_D as ::core::ffi::c_int) as uint16;
    }
    if Enemy_MoveRight_IgnoreSlopes(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        (((*E).nefl_var_C as uint32) << 16 as ::core::ffi::c_int | (*E).nefl_var_D as uint32)
            as int32,
        game,
    ) != 0
    {
        (*E).nefl_parameter_1 =
            ((*E).nefl_parameter_1 as ::core::ffi::c_int ^ 0x40 as ::core::ffi::c_int) as uint16;
    } else {
        if (*E).nefl_parameter_1 as ::core::ffi::c_int + 128 as ::core::ffi::c_int
            & 0x80 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            (*E).nefl_var_E = !((*E).nefl_var_E as ::core::ffi::c_int) as uint16;
            (*E).nefl_var_F = -((*E).nefl_var_F as ::core::ffi::c_int) as uint16;
        }
        if Enemy_MoveDown(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            (((*E).nefl_var_E as uint32) << 16 as ::core::ffi::c_int | (*E).nefl_var_F as uint32)
                as int32,
            game,
        ) != 0
        {
            (*E).nefl_parameter_1 = ((*E).nefl_parameter_1 as ::core::ffi::c_int
                ^ 0xc0 as ::core::ffi::c_int) as uint16;
        }
    };
}
pub unsafe fn LavaquakeRocks_Init(game: &mut Game) {
    let mut E: *mut Enemy_LavaquakeRocks = Get_LavaquakeRocks(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.current_instruction =
        addr_kNorfairErraticFireball_Ilist_B51A as ::core::ffi::c_int as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    (*E).lrs_var_A = (0xa2b596 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) = 17 as uint16;
}
pub unsafe fn CallLavaQuakeRocksFunc(mut ea: uint32, game: &mut Game) {
    match ea {
        10663318 => {
            LavaquakeRocks_1(game);
            return;
        }
        10663346 => {
            LavaquakeRocks_2(game);
            return;
        }
        10663402 => {
            LavaquakeRocks_3(game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn LavaquakeRocks_Main(game: &mut Game) {
    let mut E: *mut Enemy_LavaquakeRocks = Get_LavaquakeRocks(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallLavaQuakeRocksFunc(
        ((*E).lrs_var_A as ::core::ffi::c_int | 0xa20000 as ::core::ffi::c_int) as uint32,
        game,
    );
}
pub unsafe fn LavaquakeRocks_1(game: &mut Game) {
    if IsSamusWithinEnemy_X(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        0x40 as uint16,
        game,
    ) != 0
    {
        if IsSamusWithinEnemy_Y(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            0x40 as uint16,
            game,
        ) != 0
        {
            (*Get_LavaquakeRocks(
                *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                    as ::core::ffi::c_int,
                game,
            ))
            .lrs_var_A = (0xa2b5b2 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
    }
}
pub unsafe fn LavaquakeRocks_2(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1995) as *mut uint16) = *(game.g_rom.offset(
        (((0xa2b550 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa2b550 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8
        as *mut uint16)
        .offset(
            ((NextRandom(game) as ::core::ffi::c_int & 0x1e as ::core::ffi::c_int)
                >> 1 as ::core::ffi::c_int) as isize,
        );
    let mut v1: ::core::ffi::c_int = (NextRandom(game) as ::core::ffi::c_int
        & 0x1e as ::core::ffi::c_int)
        >> 1 as ::core::ffi::c_int;
    SpawnEprojWithGfx(
        *(game.g_rom.offset(
            (((0xa2b530 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa2b530 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v1 as isize),
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        addr_stru_86BD5A as ::core::ffi::c_int as uint16,
        game,
    );
    let mut E: *mut Enemy_LavaquakeRocks = Get_LavaquakeRocks(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).lrs_var_A = (0xa2b5ea as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).lrs_var_B = *(game.g_rom.offset(
        (((0xa2b520 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa2b520 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            ((NextRandom(game) as ::core::ffi::c_int & 0xe as ::core::ffi::c_int) as uint16
                as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as isize,
        );
}
pub unsafe fn LavaquakeRocks_3(game: &mut Game) {
    let mut E: *mut Enemy_LavaquakeRocks = Get_LavaquakeRocks(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).lrs_var_B = (*E).lrs_var_B.wrapping_sub(1);
    if (*E).lrs_var_B as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).lrs_var_A = (0xa2b596 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Rinka_Init(game: &mut Game) {
    let mut E: *mut Enemy_Rinka = Get_Rinka(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).rinka_parameter_1 != 0 {
        Rinka_1(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        );
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            & !(kEnemyProps_RespawnIfKilled as ::core::ffi::c_int
                | kEnemyProps_DisableSamusColl as ::core::ffi::c_int
                | kEnemyProps_ProcessedOffscreen as ::core::ffi::c_int
                | kEnemyProps_Tangible as ::core::ffi::c_int)
            | 0x2c00 as ::core::ffi::c_int) as uint16;
    } else {
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            & !(kEnemyProps_RespawnIfKilled as ::core::ffi::c_int
                | kEnemyProps_DisableSamusColl as ::core::ffi::c_int
                | kEnemyProps_ProcessedOffscreen as ::core::ffi::c_int
                | kEnemyProps_Tangible as ::core::ffi::c_int)
            | 0x6400 as ::core::ffi::c_int) as uint16;
    }
    (*E).base.palette_index = 1024 as uint16;
    Rinka_Init3(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
}
pub unsafe fn Rinka_Init2(mut k: uint16, game: &mut Game) {
    let mut v2: *mut EnemySpawnData = ::core::ptr::null_mut::<EnemySpawnData>();
    let mut E: *mut Enemy_Rinka = Get_Rinka(k as ::core::ffi::c_int, game);
    if (*E).rinka_parameter_1 != 0 {
        Rinka_1(k, game);
    }
    v2 = gEnemySpawnData(k as ::core::ffi::c_int, game);
    (*E).base.x_pos = (*v2).x_pos;
    (*E).base.y_pos = (*v2).y_pos;
    Rinka_Init3(k, game);
}
pub unsafe fn Rinka_Init3(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Rinka = Get_Rinka(k as ::core::ffi::c_int, game);
    (*E).rinka_var_A = (0xa2b852 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).rinka_var_F = 26 as uint16;
    (*E).rinka_var_B = 0 as uint16;
    (*E).rinka_var_C = 0 as uint16;
    if (*E).rinka_parameter_1 != 0 {
        if (*Get_Rinka(0 as ::core::ffi::c_int, game)).rinka_var_1D != 0 {
            (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
                | kEnemyProps_Deleted as ::core::ffi::c_int)
                as uint16;
        } else {
            (*E).base.current_instruction = addr_kRinka_Ilist_BA0C as ::core::ffi::c_int as uint16;
            (*E).base.instruction_timer = 1 as uint16;
            (*E).base.timer = 0 as uint16;
        }
    } else {
        (*E).base.current_instruction = addr_kRinka_Ilist_B9E0 as ::core::ffi::c_int as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
    };
}
pub unsafe fn Rinka_1(mut k: uint16, game: &mut Game) {
    let mut v2: uint16 = 0;
    let mut ES: *mut EnemySpawnData = gEnemySpawnData(k as ::core::ffi::c_int, game);
    if Rinka_10((*ES).x_pos, (*ES).y_pos, game) as ::core::ffi::c_int != 0 || {
        v2 = Rinka_2(k, game);
        (*gRam8000_Default(v2 as ::core::ffi::c_int, game).offset(31)).var_3F as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            != 0
    } {
        let mut v3: uint16 = 0 as uint16;
        loop {
            let mut v4: ::core::ffi::c_int = v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            let mut x: uint16 = *(game.g_rom.offset(
                (((0xa2b75b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa2b75b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v4 as isize);
            let mut y: uint16 = *(game.g_rom.offset(
                (((0xa2b75b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa2b75b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v4 + 1 as ::core::ffi::c_int) as isize);
            if !Rinka_10(x, y, game)
                && (*gRam8000_Default(
                    *(game.g_rom.offset(
                        (((0xa2b75b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                            << 15 as ::core::ffi::c_int
                            | 0xa2b75b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                            & 0x3fffff as ::core::ffi::c_int) as isize,
                    ) as *const uint8 as *mut uint16)
                        .offset((v4 + 2 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_int,
                    game,
                )
                .offset(31))
                .var_3F as ::core::ffi::c_int
                    & 1 as ::core::ffi::c_int
                    == 0
            {
                let mut E: *mut Enemy_Rinka = Get_Rinka(k as ::core::ffi::c_int, game);
                (*ES).x_pos = x;
                (*E).base.x_pos = x;
                (*ES).y_pos = y;
                (*E).base.y_pos = y;
                let mut v9: uint16 = *(game.g_rom.offset(
                    (((0xa2b75b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xa2b75b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset(
                        ((v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
                            + 2 as ::core::ffi::c_int) as isize,
                    );
                (*gRam8000_Default(v9 as ::core::ffi::c_int, game).offset(31)).var_3F =
                    -(1 as ::core::ffi::c_int) as uint16;
                (*E).rinka_var_D = v9;
                return;
            }
            v3 = (v3 as ::core::ffi::c_int + 6 as ::core::ffi::c_int) as uint16;
            if !(((v3 as ::core::ffi::c_int - 66 as ::core::ffi::c_int) as int16
                as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int)
            {
                break;
            }
        }
        let mut v10: uint16 = 0 as uint16;
        while (*gRam8000_Default(
            *(game.g_rom.offset(
                (((0xa2b75b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa2b75b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(
                    ((v10 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
                        + 2 as ::core::ffi::c_int) as isize,
                ) as ::core::ffi::c_int,
            game,
        )
        .offset(31))
        .var_3F as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            != 0
        {
            v10 = (v10 as ::core::ffi::c_int + 6 as ::core::ffi::c_int) as uint16;
            if (v10 as ::core::ffi::c_int - 66 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int
            {
                return;
            }
        }
        let mut v11: ::core::ffi::c_int = v10 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v12: uint16 = *(game.g_rom.offset(
            (((0xa2b75b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa2b75b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v11 as isize);
        (*ES).x_pos = v12;
        let mut E_0: *mut Enemy_Rinka = Get_Rinka(k as ::core::ffi::c_int, game);
        (*E_0).base.x_pos = v12;
        let mut v15: uint16 = *(game.g_rom.offset(
            (((0xa2b75b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa2b75b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((v11 + 1 as ::core::ffi::c_int) as isize);
        (*ES).y_pos = v15;
        (*E_0).base.y_pos = v15;
        let mut v16: uint16 = *(game.g_rom.offset(
            (((0xa2b75b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa2b75b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((v11 + 2 as ::core::ffi::c_int) as isize);
        (*E_0).rinka_var_D = v16;
        (*gRam8000_Default(v16 as ::core::ffi::c_int, game).offset(31)).var_3F =
            -(1 as ::core::ffi::c_int) as uint16;
    } else {
        (*Get_Rinka(k as ::core::ffi::c_int, game)).rinka_var_D = v2;
        (*gRam8000_Default(v2 as ::core::ffi::c_int, game).offset(31)).var_3F =
            -(1 as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn Rinka_2(mut k: uint16, game: &mut Game) -> uint16 {
    let mut v3: *mut EnemySpawnData = ::core::ptr::null_mut::<EnemySpawnData>();
    let mut v1: uint16 = 0 as uint16;
    loop {
        let mut v2: ::core::ffi::c_int = v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        v3 = gEnemySpawnData(k as ::core::ffi::c_int, game);
        if *(game.g_rom.offset(
            (((0xa2b75b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa2b75b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v2 as isize) as ::core::ffi::c_int
            == (*v3).x_pos as ::core::ffi::c_int
            && *(game.g_rom.offset(
                (((0xa2b75b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa2b75b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v2 + 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int
                == (*v3).y_pos as ::core::ffi::c_int
        {
            break;
        }
        v1 = (v1 as ::core::ffi::c_int + 6 as ::core::ffi::c_int) as uint16;
        if v1 as ::core::ffi::c_int - 66 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
            v1 = 0 as uint16;
            return *(game.g_rom.offset(
                (((0xa2b75b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa2b75b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(
                    ((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
                        + 2 as ::core::ffi::c_int) as isize,
                );
        }
    }
    return *(game.g_rom.offset(
        (((0xa2b75b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa2b75b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            ((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 2 as ::core::ffi::c_int)
                as isize,
        );
}
pub unsafe fn CallRinkaFunc(mut ea: uint32, mut k: uint16, game: &mut Game) {
    match ea {
        10663903 => {
            Rinka_3(k, game);
            return;
        }
        10664004 => {
            Rinka_4(k, game);
            return;
        }
        10664018 => {
            Rinka_5(k, game);
            return;
        }
        10664027 => {
            Rinka_B85B(k, game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn Rinka_Main(game: &mut Game) {
    let mut v2: uint16 = 0;
    let mut E: *mut Enemy_Rinka = Get_Rinka(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).rinka_parameter_1 as ::core::ffi::c_int != 0
        && (*Get_Rinka(0 as ::core::ffi::c_int, game)).rinka_var_1D as ::core::ffi::c_int != 0
    {
        Rinka_6(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        );
        Rinka_8(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        );
        v2 = 0 as uint16;
        RinkasDeathAnimation(v2, game);
    } else {
        CallRinkaFunc(
            ((*E).rinka_var_A as ::core::ffi::c_int | 0xa20000 as ::core::ffi::c_int) as uint32,
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        );
    };
}
pub unsafe fn Rinka_3(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Rinka = Get_Rinka(k as ::core::ffi::c_int, game);
    (*E).rinka_var_F = (*E).rinka_var_F.wrapping_sub(1);
    if (*E).rinka_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).rinka_var_A =
            (0xa2b85b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        let mut v3: uint16 = 0;
        if (*E).rinka_parameter_1 != 0 {
            v3 = ((*E).base.properties as ::core::ffi::c_int
                & !(kEnemyProps_Tangible as ::core::ffi::c_int)) as uint16;
        } else {
            v3 = ((*E).base.properties as ::core::ffi::c_int & 0xf3ff as ::core::ffi::c_int
                | 0x800 as ::core::ffi::c_int) as uint16;
        }
        (*E).base.properties = v3;
        let mut r18: uint16 = -(CalculateAngleFromXY(
            (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                as ::core::ffi::c_int
                - (*E).base.x_pos as ::core::ffi::c_int) as uint16,
            (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
                as ::core::ffi::c_int
                - (*E).base.y_pos as ::core::ffi::c_int) as uint16,
            game,
        ) as ::core::ffi::c_int
            + 0x80 as ::core::ffi::c_int) as uint8 as uint16;
        (*E).rinka_var_B = Math_MultBySin(0x120 as uint16, r18);
        (*E).rinka_var_C = Math_MultByCos(0x120 as uint16, r18);
    }
}
pub unsafe fn Rinka_4(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Rinka = Get_Rinka(k as ::core::ffi::c_int, game);
    (*E).rinka_var_F = (*E).rinka_var_F.wrapping_sub(1);
    if (*E).rinka_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).base.health = 10 as uint16;
        Rinka_Init2(k, game);
    }
}
pub unsafe fn Rinka_5(mut k: uint16, game: &mut Game) {
    if Rinka_9(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        Rinka_B865(k, game);
    }
}
pub unsafe fn Rinka_B85B(mut k: uint16, game: &mut Game) {
    MoveEnemyWithVelocity(game);
    if Rinka_9(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        Rinka_B865(k, game);
    }
}
pub unsafe fn Rinka_B865(mut k: uint16, game: &mut Game) {
    if (*Get_Rinka(k as ::core::ffi::c_int, game)).rinka_parameter_1 as ::core::ffi::c_int != 0 && {
        Rinka_8(k, game);
        (*Get_Rinka(0 as ::core::ffi::c_int, game)).rinka_var_1D as ::core::ffi::c_int != 0
    } {
        Rinka_6(k, game);
        DeleteEnemyAndConnectedEnemies(game);
    } else {
        Rinka_6(k, game);
        Rinka_Init2(k, game);
    };
}
pub unsafe fn Rinka_6(mut k: uint16, game: &mut Game) {
    let mut v3: int16 = 0;
    let mut E: *mut Enemy_Rinka = Get_Rinka(k as ::core::ffi::c_int, game);
    if (*E).rinka_parameter_1 as ::core::ffi::c_int != 0
        && (*E).base.properties as ::core::ffi::c_int & kEnemyProps_Invisible as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
    {
        let mut E0: *mut Enemy_Rinka = Get_Rinka(0 as ::core::ffi::c_int, game);
        v3 = ((*E0).rinka_var_1E as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as int16;
        if (v3 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
            v3 = 0 as int16;
        }
        (*E0).rinka_var_1E = v3 as uint16;
    }
}
pub unsafe fn Rinka_7(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Rinka = Get_Rinka(k as ::core::ffi::c_int, game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0xe44) as *mut uint16) as ::core::ffi::c_int
        & 3 as ::core::ffi::c_int
        == (*E).rinka_parameter_1 as ::core::ffi::c_int
    {
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            & !(kEnemyProps_Tangible as ::core::ffi::c_int))
            as uint16;
    } else {
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            | kEnemyProps_Tangible as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn Rinka_8(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Rinka = Get_Rinka(k as ::core::ffi::c_int, game);
    if (*E).rinka_parameter_1 != 0 {
        let mut rinka_var_D: uint16 = (*E).rinka_var_D;
        if rinka_var_D != 0 {
            (*gRam8000_Default(rinka_var_D as ::core::ffi::c_int, game).offset(31)).var_3F =
                0 as uint16;
            (*E).rinka_var_D = 0 as uint16;
        }
    }
}
pub unsafe fn Rinka_9(mut k: uint16, game: &mut Game) -> uint8 {
    let mut y_pos: int16 = 0;
    let mut v3: int16 = 0;
    let mut x_pos: int16 = 0;
    let mut v5: int16 = 0;
    let mut E: *mut Enemy_Rinka = Get_Rinka(k as ::core::ffi::c_int, game);
    y_pos = (*E).base.y_pos as int16;
    let mut result: uint8 = 1 as uint8;
    if y_pos as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
        v3 = (y_pos as ::core::ffi::c_int + 16 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16)
                as ::core::ffi::c_int) as int16;
        if v3 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
            if v3 as ::core::ffi::c_int - 256 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0
            {
                x_pos = (*E).base.x_pos as int16;
                if x_pos as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
                    v5 = (x_pos as ::core::ffi::c_int + 16 as ::core::ffi::c_int
                        - *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16)
                            as ::core::ffi::c_int) as int16;
                    if v5 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
                        if v5 as ::core::ffi::c_int - 288 as ::core::ffi::c_int
                            & 0x8000 as ::core::ffi::c_int
                            != 0
                        {
                            return 0 as uint8;
                        }
                    }
                }
            }
        }
    }
    return result;
}
pub unsafe fn Rinka_10(mut r18: uint16, mut r20: uint16, game: &mut Game) -> bool {
    return r20 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int
        || ((r20 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16)
                as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        || r20 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16)
                as ::core::ffi::c_int
            - 224 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        || r20 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int
        || ((r18 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16)
                as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        || r18 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16)
                as ::core::ffi::c_int
            - 256 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0;
}
pub unsafe fn Rinka_Frozen(mut k: uint16, game: &mut Game) {
    if Rinka_9(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        (*Get_Rinka(k as ::core::ffi::c_int, game))
            .base
            .frozen_timer = 0 as uint16;
    }
    NormalEnemyFrozenAI(game);
    if (*Get_Rinka(0 as ::core::ffi::c_int, game)).rinka_var_1D != 0 {
        Rinka_6(k, game);
        Rinka_8(k, game);
        let mut v1: uint16 = 0 as uint16;
        RinkasDeathAnimation(v1, game);
    }
}
pub unsafe fn Rinka_Touch(game: &mut Game) {
    NormalEnemyTouchAiSkipDeathAnim_CurEnemy(game);
    Rinka_B960(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
}
pub unsafe fn Rinka_Shot(game: &mut Game) {
    NormalEnemyShotAiSkipDeathAnim_CurEnemy(game);
    Rinka_B960(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
}
pub unsafe fn Rinka_Powerbomb(mut k: uint16, game: &mut Game) {
    if (*Get_Rinka(k as ::core::ffi::c_int, game)).base.properties as ::core::ffi::c_int
        & kEnemyProps_Invisible as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        NormalEnemyPowerBombAiSkipDeathAnim_CurEnemy(game);
        Rinka_B960(k, game);
    }
}
pub unsafe fn Rinka_B960(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Rinka = Get_Rinka(k as ::core::ffi::c_int, game);
    if (*E).base.health == 0 {
        Rinka_6(k, game);
        Rinka_8(k, game);
        if (*E).rinka_parameter_1 != 0 {
            (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
                | (kEnemyProps_Tangible as ::core::ffi::c_int
                    | kEnemyProps_Invisible as ::core::ffi::c_int))
                as uint16;
            game.eproj_spawn_pt = Point16U {
                x: (*E).base.x_pos,
                y: (*E).base.y_pos,
            };
            SpawnEprojWithRoomGfx(0xe509 as uint16, 3 as uint16, game);
            (*E).rinka_var_A =
                (0xa2b844 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            (*E).rinka_var_F = 1 as uint16;
        } else {
            RinkasDeathAnimation(0 as uint16, game);
        }
    }
}
pub unsafe fn Rinka_Instr_B9A2(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    let mut E: *mut Enemy_Rinka = Get_Rinka(0 as ::core::ffi::c_int, game);
    if (*E).rinka_var_1E as ::core::ffi::c_int - 3 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        return (j as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    } else {
        return *(RomPtr_A2(j as uint16_t, game) as *mut uint16);
    };
}
pub unsafe fn Rinka_Instr_B9B3(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Rinka = Get_Rinka(k as ::core::ffi::c_int, game);
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        | (kEnemyProps_Tangible as ::core::ffi::c_int
            | kEnemyProps_Invisible as ::core::ffi::c_int)) as uint16;
    return jp;
}
pub unsafe fn Rinka_Instr_B9BD(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Rinka = Get_Rinka(k as ::core::ffi::c_int, game);
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        | (kEnemyProps_ProcessedOffscreen as ::core::ffi::c_int
            | kEnemyProps_Tangible as ::core::ffi::c_int
            | kEnemyProps_Invisible as ::core::ffi::c_int)) as uint16;
    return jp;
}
pub unsafe fn Rinka_Instr_B9C7(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Rinka = Get_Rinka(k as ::core::ffi::c_int, game);
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        & !(kEnemyProps_Tangible as ::core::ffi::c_int
            | kEnemyProps_Invisible as ::core::ffi::c_int)) as uint16;
    (*E).rinka_var_A = (0xa2b7df as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    let mut E0: *mut Enemy_Rinka = Get_Rinka(0 as ::core::ffi::c_int, game);
    (*E0).rinka_var_1E = (*E0).rinka_var_1E.wrapping_add(1);
    return jp;
}
pub unsafe fn EnemyInstr_Rio_Instr_1(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_Rio(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .rio_var_E = 1 as uint16;
    return jp;
}
pub unsafe fn Rio_Init(game: &mut Game) {
    let mut E: *mut Enemy_Rio = Get_Rio(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).rio_var_E = 0 as uint16;
    (*E).rio_var_F = 0 as uint16;
    (*E).base.current_instruction = addr_kRio_Ilist_BB4B as ::core::ffi::c_int as uint16;
    (*E).rio_var_B = (0xa2bbed as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn CallRioFunc(mut ea: uint32, game: &mut Game) {
    let mut k: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16);
    match ea {
        10664941 => {
            Rio_1(k, game);
            return;
        }
        10665010 => {
            Rio_2(k, game);
            return;
        }
        10665032 => {
            Rio_3(k, game);
            return;
        }
        10665143 => {
            Rio_4(k, game);
            return;
        }
        10665215 => {
            Rio_5(k, game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn Rio_Main(game: &mut Game) {
    NextRandom(game);
    let mut E: *mut Enemy_Rio = Get_Rio(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallRioFunc(
        ((*E).rio_var_B as ::core::ffi::c_int | 0xa20000 as ::core::ffi::c_int) as uint32,
        game,
    );
}
pub unsafe fn Rio_1(mut k: uint16, game: &mut Game) {
    if CompareDistToSamus_X(k, 0xa0 as uint16, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
        == 0
    {
        let mut E: *mut Enemy_Rio = Get_Rio(k as ::core::ffi::c_int, game);
        (*E).rio_var_C = *(game.g_rom.offset(
            (((0xa2bbbb as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa2bbbb as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16);
        (*E).rio_var_D = *(game.g_rom.offset(
            (((0xa2bbbf as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa2bbbf as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16);
        if ((*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
            as ::core::ffi::c_int
            - (*E).base.x_pos as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        {
            (*E).rio_var_D = -((*E).rio_var_D as ::core::ffi::c_int) as uint16;
        }
        Rio_6(addr_kRio_Ilist_BB7F as ::core::ffi::c_int as uint16, game);
        (*E).rio_var_B = (0xa2bc48 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        if CheckIfEnemyIsOnScreen(game) == 0 {
            QueueSfx2_Max6(0x65 as uint16, game);
        }
    }
}
pub unsafe fn Rio_2(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Rio = Get_Rio(k as ::core::ffi::c_int, game);
    if (*E).rio_var_E != 0 {
        (*E).rio_var_E = 0 as uint16;
        Rio_6(addr_stru_A2BB53 as ::core::ffi::c_int as uint16, game);
        (*E).rio_var_B = (0xa2bbed as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Rio_3(mut k: uint16, game: &mut Game) {
    let mut v4: int16 = 0;
    let mut E: *mut Enemy_Rio = Get_Rio(k as ::core::ffi::c_int, game);
    if Enemy_MoveRight_IgnoreSlopes(
        k,
        ((*E).rio_var_D as int16 as int32) << 8 as ::core::ffi::c_int,
        game,
    ) != 0
    {
        (*E).rio_var_D = -((*E).rio_var_D as ::core::ffi::c_int) as uint16;
    } else if !(Enemy_MoveDown(
        k,
        ((*E).rio_var_C as int16 as int32) << 8 as ::core::ffi::c_int,
        game,
    ) != 0)
    {
        v4 = ((*E).rio_var_C as ::core::ffi::c_int - 24 as ::core::ffi::c_int) as int16;
        (*E).rio_var_C = v4 as uint16;
        if (v4 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
            (*E).rio_var_A = (*E).rio_var_D;
            (*E).rio_var_D = 0 as uint16;
            (*E).rio_var_C = 0 as uint16;
            (*E).rio_var_B =
                (0xa2bcff as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        } else if (*E).rio_var_E != 0 {
            (*E).rio_var_E = 0 as uint16;
            Rio_6(addr_kRio_Ilist_BB97 as ::core::ffi::c_int as uint16, game);
        }
        return;
    }
    (*E).rio_var_C = -((*E).rio_var_C as ::core::ffi::c_int) as uint16;
    (*E).rio_var_B = (0xa2bcb7 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Rio_4(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Rio = Get_Rio(k as ::core::ffi::c_int, game);
    if Enemy_MoveRight_IgnoreSlopes(
        k,
        ((*E).rio_var_D as int16 as int32) << 8 as ::core::ffi::c_int,
        game,
    ) != 0
    {
        (*E).rio_var_D = -((*E).rio_var_D as ::core::ffi::c_int) as uint16;
    }
    if Enemy_MoveDown(
        k,
        ((*E).rio_var_C as int16 as int32) << 8 as ::core::ffi::c_int,
        game,
    ) != 0
    {
        Rio_6(addr_kRio_Ilist_BBA3 as ::core::ffi::c_int as uint16, game);
        (*E).rio_var_B = (0xa2bc32 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    } else {
        (*E).rio_var_C =
            ((*E).rio_var_C as ::core::ffi::c_int - 24 as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn Rio_5(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Rio = Get_Rio(k as ::core::ffi::c_int, game);
    if ((*Get_Rio(k as ::core::ffi::c_int, game)).base.y_pos as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) as ::core::ffi::c_int)
        as int16 as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        (*E).rio_var_D = (*E).rio_var_A;
        (*E).rio_var_C = -(1 as ::core::ffi::c_int) as uint16;
        (*E).rio_var_B = (0xa2bcb7 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    } else {
        let mut v1: ::core::ffi::c_int =
            CalculateAngleOfSamusFromEnemy(k, game) as ::core::ffi::c_int;
        (*E).rio_var_D = kSinCosTable8bit_Sext[(v1 + 64 as ::core::ffi::c_int) as usize] as uint16;
        (*E).rio_var_C = kSinCosTable8bit_Sext[v1 as usize] as uint16;
        Enemy_MoveRight_IgnoreSlopes(
            k,
            ((*E).rio_var_D as int16 as int32) << 8 as ::core::ffi::c_int,
            game,
        );
        Enemy_MoveDown(
            k,
            ((*E).rio_var_C as int16 as int32) << 8 as ::core::ffi::c_int,
            game,
        );
    };
}
pub unsafe fn Rio_6(mut a: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Rio = Get_Rio(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if a as ::core::ffi::c_int != (*E).rio_var_F as ::core::ffi::c_int {
        (*E).rio_var_F = a;
        (*E).base.current_instruction = a;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
    }
}
pub unsafe fn NorfairLavajumpingEnemy_Instr_BE8E(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_NorfairLavajumpingEnemy(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .nley_var_00 = 1 as uint16;
    return jp;
}
pub unsafe fn NorfairLavajumpingEnemy_Init(game: &mut Game) {
    let mut E: *mut Enemy_NorfairLavajumpingEnemy = Get_NorfairLavajumpingEnemy(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).nley_var_00 = 0 as uint16;
    (*E).nley_var_01 = 0 as uint16;
    if (*E).nley_parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).base.current_instruction =
            addr_kNorfairLavajumpingEnemy_Ilist_BE62 as ::core::ffi::c_int as uint16;
        (*E).nley_var_F = (0xa2bedc as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    } else {
        (*E).nley_var_D = (*E).base.x_pos;
        (*E).nley_var_E = (*E).base.y_pos;
        (*E).base.current_instruction =
            addr_kNorfairLavajumpingEnemy_Ilist_BE3C as ::core::ffi::c_int as uint16;
        (*E).nley_var_F = (0xa2bf1a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn NorfairLavajumpingEnemy_Main(game: &mut Game) {
    NextRandom(game);
    let mut E: *mut Enemy_NorfairLavajumpingEnemy = Get_NorfairLavajumpingEnemy(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallEnemyPreInstr(
        ((*E).nley_var_F as ::core::ffi::c_int | 0xa20000 as ::core::ffi::c_int) as uint32,
        game,
    );
}
pub unsafe fn NorfairLavajumpingEnemy_Func_1(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_NorfairLavajumpingEnemy =
        Get_NorfairLavajumpingEnemy(k as ::core::ffi::c_int, game);
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
        .offset((v1 + 100 as ::core::ffi::c_int) as isize)
        != 0
    {
        let mut v3: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v1 + 109 as ::core::ffi::c_int) as isize);
        (*E).base.frozen_timer = v3;
        if v3 as ::core::ffi::c_int != 0
            || *((&raw mut game.g_ram as *mut uint8).offset(0xf68) as *mut uint16)
                .offset((v1 + 2 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
        {
            (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
                | kEnemyProps_Invisible as ::core::ffi::c_int)
                as uint16;
        } else {
            (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
                & !(kEnemyProps_Invisible as ::core::ffi::c_int))
                as uint16;
            (*E).base.y_pos = *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
                .offset((v1 + 93 as ::core::ffi::c_int) as isize);
        }
    } else {
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            | kEnemyProps_Deleted as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn NorfairLavajumpingEnemy_Func_2(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_NorfairLavajumpingEnemy =
        Get_NorfairLavajumpingEnemy(k as ::core::ffi::c_int, game);
    (*E).nley_var_C = *(game.g_rom.offset(
        (((0xa2be86 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa2be86 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            ((*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ) as ::core::ffi::c_int
                & 6 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as isize,
        );
    (*E).nley_var_F = (0xa2bf3e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        | kEnemyProps_ProcessedOffscreen as ::core::ffi::c_int)
        as uint16;
    QueueSfx2_Max6(0xd as uint16, game);
}
pub unsafe fn NorfairLavajumpingEnemy_Func_3(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_NorfairLavajumpingEnemy =
        Get_NorfairLavajumpingEnemy(k as ::core::ffi::c_int, game);
    let mut t: ::core::ffi::c_int = (*E).base.y_subpos as ::core::ffi::c_int
        + ((*(&raw mut (*E).nley_var_C as *mut uint8).offset(0) as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int);
    (*E).base.y_subpos = t as uint16;
    (*E).base.y_pos = ((*E).base.y_pos as ::core::ffi::c_int
        + (*(&raw mut (*E).nley_var_C as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as int8 as ::core::ffi::c_int
            + (t >> 16 as ::core::ffi::c_int))) as uint16;
    let mut v3: uint16 =
        ((*E).nley_var_C as ::core::ffi::c_int + 56 as ::core::ffi::c_int) as uint16;
    (*E).nley_var_C = v3;
    if v3 as ::core::ffi::c_int >= 0xfc00 as ::core::ffi::c_int {
        NorfairLavajumpingEnemy_Func_6(
            addr_kNorfairLavajumpingEnemy_Ilist_BE42 as ::core::ffi::c_int as uint16,
            game,
        );
        (*E).nley_var_F = (0xa2bf7c as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn NorfairLavajumpingEnemy_Func_4(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_NorfairLavajumpingEnemy =
        Get_NorfairLavajumpingEnemy(k as ::core::ffi::c_int, game);
    let mut t: ::core::ffi::c_int = (*E).base.y_subpos as ::core::ffi::c_int
        + ((*(&raw mut (*E).nley_var_C as *mut uint8).offset(0) as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int);
    (*E).base.y_subpos = t as uint16;
    (*E).base.y_pos = ((*E).base.y_pos as ::core::ffi::c_int
        + (*(&raw mut (*E).nley_var_C as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as int8 as ::core::ffi::c_int
            + (t >> 16 as ::core::ffi::c_int))) as uint16;
    (*E).nley_var_C = ((*E).nley_var_C as ::core::ffi::c_int + 56 as ::core::ffi::c_int) as uint16;
    if (*E).nley_var_00 != 0 {
        (*E).nley_var_00 = 0 as uint16;
        (*E).nley_var_F = (0xa2bfbc as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn NorfairLavajumpingEnemy_Func_5(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_NorfairLavajumpingEnemy =
        Get_NorfairLavajumpingEnemy(k as ::core::ffi::c_int, game);
    let mut t: ::core::ffi::c_int = (*E).base.y_subpos as ::core::ffi::c_int
        + ((*(&raw mut (*E).nley_var_C as *mut uint8).offset(0) as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int);
    (*E).base.y_subpos = t as uint16;
    (*E).base.y_pos = ((*E).base.y_pos as ::core::ffi::c_int
        + *(&raw mut (*E).nley_var_C as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as int8 as ::core::ffi::c_int
        + (t >> 16 as ::core::ffi::c_int)) as uint16;
    if (*E).base.y_pos as ::core::ffi::c_int & 0xf0 as ::core::ffi::c_int
        == 240 as ::core::ffi::c_int
    {
        (*E).base.x_pos = (*E).nley_var_D;
        (*E).base.y_pos = (*E).nley_var_E;
        NorfairLavajumpingEnemy_Func_6(
            addr_kNorfairLavajumpingEnemy_Ilist_BE3C as ::core::ffi::c_int as uint16,
            game,
        );
        (*E).nley_var_F = (0xa2bf1a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            & !(kEnemyProps_ProcessedOffscreen as ::core::ffi::c_int))
            as uint16;
    } else {
        (*E).nley_var_C =
            ((*E).nley_var_C as ::core::ffi::c_int + 56 as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn NorfairLavajumpingEnemy_Func_6(mut a: uint16, game: &mut Game) {
    let mut E: *mut Enemy_NorfairLavajumpingEnemy = Get_NorfairLavajumpingEnemy(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if a as ::core::ffi::c_int != (*E).nley_var_01 as ::core::ffi::c_int {
        (*E).nley_var_01 = a;
        (*E).base.current_instruction = a;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
    }
}
pub unsafe fn NorfairRio_Instr_C1C9(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_NorfairRio(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .nro_var_01 = 1 as uint16;
    return jp;
}
pub unsafe fn NorfairRio_Instr_C1D4(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_NorfairRio(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .nro_var_02 = 8 as uint16;
    return jp;
}
pub unsafe fn NorfairRio_Instr_C1DF(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_NorfairRio(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .nro_var_02 = 8 as uint16;
    return jp;
}
pub unsafe fn NorfairRio_Instr_C1EA(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_NorfairRio(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .nro_var_02 = 12 as uint16;
    return jp;
}
pub unsafe fn NorfairRio_Instr_C1F5(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_NorfairRio(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .nro_var_02 = -(12 as ::core::ffi::c_int) as uint16;
    return jp;
}
pub unsafe fn NorfairRio_Instr_C200(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_NorfairRio(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .nro_var_02 = 4 as uint16;
    return jp;
}
pub unsafe fn NorfairRio_Instr_C20B(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_NorfairRio(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .nro_var_02 = 0 as uint16;
    return jp;
}
pub unsafe fn NorfairRio_Instr_C216(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_NorfairRio(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .nro_var_02 = -(4 as ::core::ffi::c_int) as uint16;
    return jp;
}
pub unsafe fn NorfairRio_Instr_C221(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_NorfairRio(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .nro_var_02 = -(12 as ::core::ffi::c_int) as uint16;
    return jp;
}
pub unsafe fn NorfairRio_Instr_C22C(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_NorfairRio(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .nro_var_02 = -(16 as ::core::ffi::c_int) as uint16;
    return jp;
}
pub unsafe fn NorfairRio_Instr_C237(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_NorfairRio(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .nro_var_02 = 12 as uint16;
    return jp;
}
pub unsafe fn NorfairRio_Init(game: &mut Game) {
    let mut E: *mut Enemy_NorfairRio = Get_NorfairRio(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).nro_var_01 = 0 as uint16;
    (*E).nro_var_02 = 0 as uint16;
    if (*E).nro_parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).nro_var_00 = addr_kNorfairRio_Ilist_C18F as ::core::ffi::c_int as uint16;
        (*E).base.current_instruction = addr_kNorfairRio_Ilist_C18F as ::core::ffi::c_int as uint16;
        (*E).nro_var_F = (0xa2c281 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    } else {
        (*E).nro_var_00 = addr_kNorfairRio_Ilist_C0F1 as ::core::ffi::c_int as uint16;
        (*E).base.current_instruction = addr_kNorfairRio_Ilist_C0F1 as ::core::ffi::c_int as uint16;
        (*E).nro_var_F = (0xa2c2e7 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn NorfairRio_Main(game: &mut Game) {
    NextRandom(game);
    let mut E: *mut Enemy_NorfairRio = Get_NorfairRio(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallEnemyPreInstr(
        ((*E).nro_var_F as ::core::ffi::c_int | 0xa20000 as ::core::ffi::c_int) as uint32,
        game,
    );
}
pub unsafe fn NorfairRio_Func_1(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_NorfairRio = Get_NorfairRio(k as ::core::ffi::c_int, game);
    let mut v6: *mut EnemySpawnData = ::core::ptr::null_mut::<EnemySpawnData>();
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
        .offset((v1 + 100 as ::core::ffi::c_int) as isize)
        != 0
    {
        let mut v3: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v1 + 109 as ::core::ffi::c_int) as isize);
        (*E).base.frozen_timer = v3;
        if v3 as ::core::ffi::c_int != 0 || {
            (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
                & !(kEnemyProps_Invisible as ::core::ffi::c_int))
                as uint16;
            (*gEnemySpawnData(k as ::core::ffi::c_int, game).offset(31)).some_flag
                as ::core::ffi::c_int
                == addr_kNorfairRio_Ilist_C0F1 as ::core::ffi::c_int
        } {
            (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
                | kEnemyProps_Invisible as ::core::ffi::c_int)
                as uint16;
        } else {
            let mut r18: uint16 = addr_kNorfairRio_Ilist_C18F as ::core::ffi::c_int as uint16;
            v6 = gEnemySpawnData(k as ::core::ffi::c_int, game);
            if (*v6.offset(31)).field_4 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                r18 = addr_kNorfairRio_Ilist_C1A3 as ::core::ffi::c_int as uint16;
            }
            NorfairRio_Func_7(r18, game);
            (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
                & !(kEnemyProps_Invisible as ::core::ffi::c_int))
                as uint16;
            (*E).base.x_pos = *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
                .offset((v1 + 91 as ::core::ffi::c_int) as isize);
            (*E).base.y_pos = ((*v6.offset(31)).field_4 as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
                    .offset((v1 + 93 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int) as uint16;
        }
    } else {
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            | kEnemyProps_Deleted as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn NorfairRio_Func_2(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_NorfairRio = Get_NorfairRio(k as ::core::ffi::c_int, game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) as ::core::ffi::c_int
        & 0x101 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
        && CompareDistToSamus_X(k, 0xc0 as uint16, game) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            == 0
    {
        (*E).nro_var_A = *(game.g_rom.offset(
            (((0xa2c1c1 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa2c1c1 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(
                ((*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
                    as ::core::ffi::c_int
                    >> 1 as ::core::ffi::c_int
                    & 2 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
                    >> 1 as ::core::ffi::c_int) as isize,
            );
        (*E).nro_var_B = *(game.g_rom.offset(
            (((0xa2c1c5 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa2c1c5 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16);
        if ((*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
            as ::core::ffi::c_int
            - (*E).base.x_pos as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        {
            (*E).nro_var_B = -((*E).nro_var_B as ::core::ffi::c_int) as uint16;
        }
        NorfairRio_Func_7(
            addr_kNorfairRio_Ilist_C107 as ::core::ffi::c_int as uint16,
            game,
        );
        (*E).nro_var_F = (0xa2c33f as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    } else if (*E).nro_var_01 != 0 {
        (*E).nro_var_01 = 0 as uint16;
        NorfairRio_Func_7(
            addr_kNorfairRio_Ilist_C0F1 as ::core::ffi::c_int as uint16,
            game,
        );
    }
}
pub unsafe fn NorfairRio_Func_3(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_NorfairRio = Get_NorfairRio(k as ::core::ffi::c_int, game);
    if (*E).nro_var_01 != 0 {
        (*E).nro_var_01 = 0 as uint16;
        NorfairRio_Func_7(
            addr_kNorfairRio_Ilist_C12F as ::core::ffi::c_int as uint16,
            game,
        );
        (*E).nro_var_F = (0xa2c361 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        QueueSfx2_Max6(0x65 as uint16, game);
    }
}
pub unsafe fn NorfairRio_Func_4(mut k: uint16, game: &mut Game) {
    let mut v4: int16 = 0;
    let mut E: *mut Enemy_NorfairRio = Get_NorfairRio(k as ::core::ffi::c_int, game);
    if Enemy_MoveRight_IgnoreSlopes(
        k,
        ((*E).nro_var_B as int16 as int32) << 8 as ::core::ffi::c_int,
        game,
    ) != 0
    {
        (*E).nro_var_B = -((*E).nro_var_B as ::core::ffi::c_int) as uint16;
    }
    if Enemy_MoveDown(
        k,
        ((*E).nro_var_A as int16 as int32) << 8 as ::core::ffi::c_int,
        game,
    ) as ::core::ffi::c_int
        != 0
        || {
            v4 = ((*E).nro_var_A as ::core::ffi::c_int - 32 as ::core::ffi::c_int) as int16;
            (*E).nro_var_A = v4 as uint16;
            (v4 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int
        }
    {
        (*E).nro_var_A = -(1 as ::core::ffi::c_int) as uint16;
        NorfairRio_Func_7(
            addr_kNorfairRio_Ilist_C145 as ::core::ffi::c_int as uint16,
            game,
        );
        (*E).nro_var_F = (0xa2c3b1 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn NorfairRio_Func_5(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_NorfairRio = Get_NorfairRio(k as ::core::ffi::c_int, game);
    if Enemy_MoveRight_IgnoreSlopes(
        k,
        ((*E).nro_var_B as int16 as int32) << 8 as ::core::ffi::c_int,
        game,
    ) != 0
    {
        (*E).nro_var_B = -((*E).nro_var_B as ::core::ffi::c_int) as uint16;
    }
    if Enemy_MoveDown(
        k,
        ((*E).nro_var_A as int16 as int32) << 8 as ::core::ffi::c_int,
        game,
    ) != 0
    {
        (*E).nro_var_F = (0xa2c406 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    } else {
        (*E).nro_var_A =
            ((*E).nro_var_A as ::core::ffi::c_int - 32 as ::core::ffi::c_int) as uint16;
        if (*E).nro_var_01 != 0 {
            (*E).nro_var_01 = 0 as uint16;
            NorfairRio_Func_7(
                addr_kNorfairRio_Ilist_C179 as ::core::ffi::c_int as uint16,
                game,
            );
        }
    };
}
pub unsafe fn NorfairRio_Func_6(mut k: uint16, game: &mut Game) {
    (*Get_NorfairRio(k as ::core::ffi::c_int, game)).nro_var_F =
        (0xa2c2e7 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn NorfairRio_Func_7(mut a: uint16, game: &mut Game) {
    let mut E: *mut Enemy_NorfairRio = Get_NorfairRio(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if a as ::core::ffi::c_int != (*E).nro_var_00 as ::core::ffi::c_int {
        (*E).nro_var_00 = a;
        (*E).base.current_instruction = a;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
    }
}
pub unsafe fn LowerNorfairRio_Instr_C6D2(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_NorfairRio(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .nro_var_01 = 1 as uint16;
    return jp;
}
pub unsafe fn LowerNorfairRio_Instr_C6DD(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_NorfairRio(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .nro_var_02 = 0 as uint16;
    return jp;
}
pub unsafe fn LowerNorfairRio_Instr_C6E8(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_NorfairRio(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .nro_var_02 = 1 as uint16;
    return jp;
}
pub unsafe fn LowerNorfairRio_Init(game: &mut Game) {
    let mut E: *mut Enemy_NorfairRio = Get_NorfairRio(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).nro_var_01 = 0 as uint16;
    if (*E).nro_parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        (*E).nro_var_F = (0xa2c771 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).nro_var_00 = addr_kLowerNorfairRio_Ilist_C61A as ::core::ffi::c_int as uint16;
        (*E).base.current_instruction =
            addr_kLowerNorfairRio_Ilist_C61A as ::core::ffi::c_int as uint16;
    } else {
        (*E).nro_var_F = (0xa2c72e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).nro_var_00 = addr_kLowerNorfairRio_Ilist_C6B0 as ::core::ffi::c_int as uint16;
        (*E).base.current_instruction =
            addr_kLowerNorfairRio_Ilist_C6B0 as ::core::ffi::c_int as uint16;
    };
}
pub unsafe fn LowerNorfairRio_Main(game: &mut Game) {
    NextRandom(game);
    let mut E: *mut Enemy_LowerNorfairRio = Get_LowerNorfairRio(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallEnemyPreInstr(
        ((*E).lnro_var_F as ::core::ffi::c_int | 0xa20000 as ::core::ffi::c_int) as uint32,
        game,
    );
}
pub unsafe fn LowerNorfairRio_Func_1(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_LowerNorfairRio = Get_LowerNorfairRio(k as ::core::ffi::c_int, game);
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
        .offset((v1 + 100 as ::core::ffi::c_int) as isize)
        != 0
    {
        let mut v3: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v1 + 109 as ::core::ffi::c_int) as isize);
        (*E).base.frozen_timer = v3;
        if v3 == 0
            && (*gEnemySpawnData(k as ::core::ffi::c_int, game).offset(31)).field_4
                as ::core::ffi::c_int
                != 0
        {
            (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
                & !(kEnemyProps_Invisible as ::core::ffi::c_int))
                as uint16;
            (*E).base.x_pos = *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
                .offset((v1 + 91 as ::core::ffi::c_int) as isize);
            (*E).base.y_pos = (*((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
                .offset((v1 + 93 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int
                + 12 as ::core::ffi::c_int) as uint16;
        } else {
            (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
                | kEnemyProps_Invisible as ::core::ffi::c_int)
                as uint16;
        }
    } else {
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            | kEnemyProps_Deleted as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn LowerNorfairRio_Func_2(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_NorfairRio = Get_NorfairRio(k as ::core::ffi::c_int, game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) as ::core::ffi::c_int
        & 0x101 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
        && CompareDistToSamus_X(k, 0x70 as uint16, game) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            == 0
    {
        (*E).nro_var_C = *(game.g_rom.offset(
            (((0xa2c6ca as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa2c6ca as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16);
        (*E).nro_var_D = *(game.g_rom.offset(
            (((0xa2c6ce as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa2c6ce as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16);
        if ((*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
            as ::core::ffi::c_int
            - (*E).base.x_pos as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        {
            (*E).nro_var_D = -((*E).nro_var_D as ::core::ffi::c_int) as uint16;
        }
        LowerNorfairRio_Func_7(
            addr_kLowerNorfairRio_Ilist_C630 as ::core::ffi::c_int as uint16,
            game,
        );
        (*E).nro_var_F = (0xa2c7bb as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    } else {
        (*E).nro_var_01 = 0 as uint16;
        LowerNorfairRio_Func_7(
            addr_kLowerNorfairRio_Ilist_C61A as ::core::ffi::c_int as uint16,
            game,
        );
    };
}
pub unsafe fn LowerNorfairRio_Func_3(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_NorfairRio = Get_NorfairRio(k as ::core::ffi::c_int, game);
    if (*E).nro_var_01 != 0 {
        (*E).nro_var_01 = 0 as uint16;
        LowerNorfairRio_Func_7(
            addr_kLowerNorfairRio_Ilist_C65A as ::core::ffi::c_int as uint16,
            game,
        );
        (*E).nro_var_F = (0xa2c7d6 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn LowerNorfairRio_Func_4(mut k: uint16, game: &mut Game) {
    let mut v4: int16 = 0;
    let mut E: *mut Enemy_LowerNorfairRio = Get_LowerNorfairRio(k as ::core::ffi::c_int, game);
    if Enemy_MoveRight_IgnoreSlopes(
        k,
        ((*E).lnro_var_D as int16 as int32) << 8 as ::core::ffi::c_int,
        game,
    ) != 0
    {
        (*E).lnro_var_D = -((*E).lnro_var_D as ::core::ffi::c_int) as uint16;
    }
    if Enemy_MoveDown(
        k,
        ((*E).lnro_var_C as int16 as int32) << 8 as ::core::ffi::c_int,
        game,
    ) as ::core::ffi::c_int
        != 0
        || {
            v4 = ((*E).lnro_var_C as ::core::ffi::c_int - 32 as ::core::ffi::c_int) as int16;
            (*E).lnro_var_C = v4 as uint16;
            (v4 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int
        }
    {
        (*E).lnro_var_C = -(1 as ::core::ffi::c_int) as uint16;
        LowerNorfairRio_Func_7(
            addr_kLowerNorfairRio_Ilist_C662 as ::core::ffi::c_int as uint16,
            game,
        );
        (*E).lnro_var_F = (0xa2c82d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        QueueSfx2_Max6(0x64 as uint16, game);
    }
}
pub unsafe fn LowerNorfairRio_Func_5(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_NorfairRio = Get_NorfairRio(k as ::core::ffi::c_int, game);
    if Enemy_MoveRight_IgnoreSlopes(
        k,
        ((*E).nro_var_D as int16 as int32) << 8 as ::core::ffi::c_int,
        game,
    ) != 0
    {
        (*E).nro_var_D = -((*E).nro_var_D as ::core::ffi::c_int) as uint16;
    }
    if Enemy_MoveDown(
        k,
        ((*E).nro_var_C as int16 as int32) << 8 as ::core::ffi::c_int,
        game,
    ) != 0
    {
        LowerNorfairRio_Func_7(
            addr_kLowerNorfairRio_Ilist_C686 as ::core::ffi::c_int as uint16,
            game,
        );
        (*E).nro_var_F = (0xa2c888 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    } else {
        (*E).nro_var_C =
            ((*E).nro_var_C as ::core::ffi::c_int - 32 as ::core::ffi::c_int) as uint16;
        if (*E).nro_var_01 != 0 {
            (*E).nro_var_01 = 0 as uint16;
            LowerNorfairRio_Func_7(
                addr_kLowerNorfairRio_Ilist_C674 as ::core::ffi::c_int as uint16,
                game,
            );
        }
    };
}
pub unsafe fn LowerNorfairRio_Func_6(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_NorfairRio = Get_NorfairRio(k as ::core::ffi::c_int, game);
    if (*E).nro_var_01 != 0 {
        (*E).nro_var_01 = 0 as uint16;
        LowerNorfairRio_Func_7(
            addr_kLowerNorfairRio_Ilist_C686 as ::core::ffi::c_int as uint16,
            game,
        );
        (*E).nro_var_F = (0xa2c771 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn LowerNorfairRio_Func_7(mut a: uint16, game: &mut Game) {
    let mut E: *mut Enemy_NorfairRio = Get_NorfairRio(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if a as ::core::ffi::c_int != (*E).nro_var_00 as ::core::ffi::c_int {
        (*E).nro_var_00 = a;
        (*E).base.current_instruction = a;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
    }
}
pub unsafe fn MaridiaLargeSnail_Instr_CB6B(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    QueueSfx2_Max6(0xe as uint16, game);
    return jp;
}
pub unsafe fn MaridiaLargeSnail_Instr_CCB3(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_MaridiaLargeSnail(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .mlsl_var_02 = 1 as uint16;
    return jp;
}
pub unsafe fn MaridiaLargeSnail_Instr_CCBE(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_MaridiaLargeSnail(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .mlsl_var_03 = 1 as uint16;
    return jp;
}
pub unsafe fn MaridiaLargeSnail_Instr_CCC9(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_MaridiaLargeSnail(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .mlsl_var_03 = 0 as uint16;
    return jp;
}
pub unsafe fn MaridiaLargeSnail_Init(game: &mut Game) {
    let mut E: *mut Enemy_MaridiaLargeSnail = Get_MaridiaLargeSnail(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).mlsl_var_B = 0 as uint16;
    (*E).mlsl_var_D = 0 as uint16;
    (*E).mlsl_var_00 = 0 as uint16;
    (*E).mlsl_var_01 = 0 as uint16;
    (*E).mlsl_var_02 = 0 as uint16;
    (*E).mlsl_var_03 = 0 as uint16;
    (*E).mlsl_var_0A = 0 as uint16;
    (*E).mlsl_var_E = 3 as uint16;
    (*E).mlsl_var_C = 128 as uint16;
    (*E).base.current_instruction =
        addr_kMaridiaLargeSnail_Ilist_CA4B as ::core::ffi::c_int as uint16;
    (*E).mlsl_var_A = (0xa2cde6 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).mlsl_var_F = (0xa2cf66 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn CallMaridiaLargeSnailFunc(mut ea: uint32, game: &mut Game) {
    match ea {
        10669542 => {
            MaridiaLargeSnail_Func_4(game);
            return;
        }
        10669611 => {
            MaridiaLargeSnail_Func_5(game);
            return;
        }
        10669888 => {
            MaridiaLargeSnail_Func_6(game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn MaridiaLargeSnail_Main(game: &mut Game) {
    MaridiaLargeSnail_Func_1(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
    let mut E: *mut Enemy_MaridiaLargeSnail = Get_MaridiaLargeSnail(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallMaridiaLargeSnailFunc(
        ((*E).mlsl_var_A as ::core::ffi::c_int | 0xa20000 as ::core::ffi::c_int) as uint32,
        game,
    );
    MaridiaLargeSnail_Func_2(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
    MaridiaLargeSnail_Func_3(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
}
pub unsafe fn MaridiaLargeSnail_Func_1(mut k: uint16, game: &mut Game) {
    MaridiaLargeSnail_Func_11(game);
    let mut E: *mut Enemy_MaridiaLargeSnail = Get_MaridiaLargeSnail(k as ::core::ffi::c_int, game);
    (*E).mlsl_var_06 = (*E).base.x_pos;
    (*E).mlsl_var_08 = (*E).base.y_pos;
}
pub unsafe fn MaridiaLargeSnail_Func_2(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_MaridiaLargeSnail = Get_MaridiaLargeSnail(k as ::core::ffi::c_int, game);
    (*E).mlsl_var_0B = 0 as uint16;
    if CheckIfEnemyTouchesSamus(k, game) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0xb5c) as *mut uint16) =
            ((*E).base.y_pos as ::core::ffi::c_int - (*E).mlsl_var_08 as ::core::ffi::c_int)
                as uint16;
    }
    if (*E).mlsl_var_04 != 0 {
        let mut r18: uint16 = ((*E).base.x_pos as ::core::ffi::c_int
            - (*E).mlsl_var_06 as ::core::ffi::c_int) as uint16;
        if r18 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            if (*E).mlsl_var_05 == 0 {
                return;
            }
        } else if (*E).mlsl_var_05 != 0 {
            return;
        }
        let ref mut fresh28 = *((&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16);
        *fresh28 = (*fresh28 as ::core::ffi::c_int + r18 as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn MaridiaLargeSnail_Func_3(mut k: uint16, game: &mut Game) {
    let mut v2: int16 = 0;
    let mut E: *mut Enemy_MaridiaLargeSnail = Get_MaridiaLargeSnail(k as ::core::ffi::c_int, game);
    if (*E).mlsl_var_04 != 0 {
        v2 = ((*E).base.x_pos as ::core::ffi::c_int - (*E).mlsl_var_06 as ::core::ffi::c_int)
            as int16;
        if v2 == 0 {
            if (*E).mlsl_var_05 != 0 {
                if (*E).mlsl_var_D == 0
                    || *((&raw mut game.g_ram as *mut uint8).offset(0x8b) as *mut uint16)
                        as ::core::ffi::c_int
                        & 0x100 as ::core::ffi::c_int
                        == 0 as ::core::ffi::c_int
                {
                    return;
                }
            } else if (*E).mlsl_var_D as ::core::ffi::c_int != 0
                || *((&raw mut game.g_ram as *mut uint8).offset(0x8b) as *mut uint16)
                    as ::core::ffi::c_int
                    & 0x200 as ::core::ffi::c_int
                    == 0 as ::core::ffi::c_int
            {
                return;
            }
            (*E).mlsl_var_0B = 1 as uint16;
            (*E).base.x_pos = (*E).mlsl_var_06;
            return;
        }
        if (v2 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
            if *((&raw mut game.g_ram as *mut uint8).offset(0x8b) as *mut uint16)
                as ::core::ffi::c_int
                & 0x100 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                (*E).mlsl_var_0B = 1 as uint16;
                (*E).base.x_pos = (*E).mlsl_var_06;
            }
        } else if *((&raw mut game.g_ram as *mut uint8).offset(0x8b) as *mut uint16)
            as ::core::ffi::c_int
            & 0x200 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            (*E).mlsl_var_0B = 1 as uint16;
            (*E).base.x_pos = (*E).mlsl_var_06;
        }
    }
}
pub unsafe fn MaridiaLargeSnail_Func_4(game: &mut Game) {
    let mut E: *mut Enemy_MaridiaLargeSnail = Get_MaridiaLargeSnail(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).mlsl_var_E != 0 {
        CallEnemyPreInstr(
            ((*E).mlsl_var_F as ::core::ffi::c_int | 0xa20000 as ::core::ffi::c_int) as uint32,
            game,
        );
    } else {
        (*E).mlsl_var_00 = 0 as uint16;
        if GetSamusEnemyDelta_X(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        ) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            (*E).mlsl_var_D = 1 as uint16;
            (*E).mlsl_var_00 = 1 as uint16;
        }
        MaridiaLargeSnail_Func_10(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        );
        if IsSamusWithinEnemy_X(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            0x18 as uint16,
            game,
        ) != 0
        {
            (*E).mlsl_var_00 =
                ((*E).mlsl_var_00 as ::core::ffi::c_int | 2 as ::core::ffi::c_int) as uint16;
            MaridiaLargeSnail_Func_10(
                *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
                game,
            );
            (*E).mlsl_var_A =
                (0xa2ce2b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
    };
}
pub unsafe fn MaridiaLargeSnail_Func_5(game: &mut Game) {
    let mut E: *mut Enemy_MaridiaLargeSnail = Get_MaridiaLargeSnail(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).mlsl_var_E != 0 {
        CallEnemyPreInstr(
            ((*E).mlsl_var_F as ::core::ffi::c_int | 0xa20000 as ::core::ffi::c_int) as uint32,
            game,
        );
    } else if Enemy_MoveDown(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        (1 as ::core::ffi::c_int as int16 as int32) << 16 as ::core::ffi::c_int,
        game,
    ) == 0
    {
        (*E).mlsl_var_B = 0 as uint16;
        (*E).mlsl_var_F = (0xa2cf66 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        (*E).mlsl_var_E = 3 as uint16;
    }
    if (*E).mlsl_var_0B == 0 {
        let mut R36: uint16 = 0 as uint16;
        (*E).mlsl_var_C = (*E).mlsl_var_C.wrapping_sub(1);
        let mut v4: bool = (*E).mlsl_var_C as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int;
        if v4 as ::core::ffi::c_int != 0
            && {
                (*E).mlsl_var_C = 0 as uint16;
                IsSamusWithinEnemy_X(
                    *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
                    0x20 as uint16,
                    game,
                ) as ::core::ffi::c_int
                    != 0
            }
            && (*E).mlsl_var_03 as ::core::ffi::c_int != 0
            && (*E).mlsl_var_E == 0
        {
            (*E).mlsl_var_00 = 0 as uint16;
            (*E).mlsl_var_D = 0 as uint16;
            if GetSamusEnemyDelta_X(
                *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
                game,
            ) as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                (*E).mlsl_var_00 = 1 as uint16;
                (*E).mlsl_var_D = 1 as uint16;
            }
            (*E).mlsl_var_C = 128 as uint16;
            (*E).mlsl_var_00 = ((*E).mlsl_var_00 as ::core::ffi::c_int & 1 as ::core::ffi::c_int
                | 4 as ::core::ffi::c_int) as uint16;
            MaridiaLargeSnail_Func_10(
                *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
                game,
            );
            (*E).mlsl_var_A =
                (0xa2cf40 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        } else {
            let mut v5: uint16 = 128 as uint16;
            if (*E).mlsl_var_D != 0 {
                v5 = 132 as uint16;
            }
            let mut v6: ::core::ffi::c_int = v5 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            let mut r18: uint16 = *(game.g_rom.offset(
                (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v6 + 1 as ::core::ffi::c_int) as isize);
            let mut r20: uint16 = *(game.g_rom.offset(
                (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v6 as isize);
            if (*E).mlsl_var_04 as ::core::ffi::c_int != 0
                && (*E).mlsl_var_D as ::core::ffi::c_int == (*E).mlsl_var_05 as ::core::ffi::c_int
            {
                if (*E).mlsl_var_D != 0 {
                    r20 = (r20 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as uint16;
                } else {
                    r20 = (r20 as ::core::ffi::c_int + 16 as ::core::ffi::c_int) as uint16;
                }
            }
            if Enemy_MoveRight_SlopesAsWalls(
                *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
                ((r20 as uint32) << 16 as ::core::ffi::c_int | r18 as uint32) as int32,
                game,
            ) != 0
            {
                R36 = R36.wrapping_add(1);
            }
            if (*E).mlsl_var_04 as ::core::ffi::c_int != 0
                && (*E).mlsl_var_D as ::core::ffi::c_int == (*E).mlsl_var_05 as ::core::ffi::c_int
            {
                let mut v7: uint16 = 0;
                if (*E).mlsl_var_D != 0 {
                    v7 = ((*E).base.x_pos as ::core::ffi::c_int + 16 as ::core::ffi::c_int)
                        as uint16;
                } else {
                    v7 = ((*E).base.x_pos as ::core::ffi::c_int - 16 as ::core::ffi::c_int)
                        as uint16;
                }
                (*E).base.x_pos = v7;
            }
            if R36 != 0 {
                (*E).mlsl_var_D =
                    ((*E).mlsl_var_D as ::core::ffi::c_int ^ 1 as ::core::ffi::c_int) as uint16;
                (*E).mlsl_var_00 =
                    ((*E).mlsl_var_00 as ::core::ffi::c_int ^ 4 as ::core::ffi::c_int) as uint16;
                MaridiaLargeSnail_Func_10(
                    *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
                    game,
                );
            }
        }
    }
}
pub unsafe fn MaridiaLargeSnail_Func_6(game: &mut Game) {
    let mut E: *mut Enemy_MaridiaLargeSnail = Get_MaridiaLargeSnail(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).mlsl_var_02 != 0 {
        (*E).mlsl_var_02 = 0 as uint16;
        (*E).mlsl_var_00 =
            ((*E).mlsl_var_00 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
        MaridiaLargeSnail_Func_10(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        );
        (*E).mlsl_var_A = (0xa2ce2b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn MaridiaLargeSnail_Func_7(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_MaridiaLargeSnail = Get_MaridiaLargeSnail(k as ::core::ffi::c_int, game);
    let mut v2: uint16 =
        ((*E).mlsl_var_B as ::core::ffi::c_int + 384 as ::core::ffi::c_int) as uint16;
    if v2 as ::core::ffi::c_int - 0x4000 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
        v2 = 0x4000 as uint16;
    }
    (*E).mlsl_var_B = v2;
    let mut v5: ::core::ffi::c_int = 8 as ::core::ffi::c_int
        * (v2 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int)
        >> 1 as ::core::ffi::c_int;
    if Enemy_MoveDown(
        k,
        *(game.g_rom.offset(
            (((0xa0cbc7 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa0cbc7 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint32)
            .offset((v5 >> 1 as ::core::ffi::c_int) as isize) as int32,
        game,
    ) != 0
    {
        (*E).mlsl_var_E = (*E).mlsl_var_E.wrapping_sub(1);
        let mut v6: int16 =
            ((*E).mlsl_var_B as ::core::ffi::c_int - 4096 as ::core::ffi::c_int) as int16;
        if (v6 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
            (*E).mlsl_var_E = 0 as uint16;
        }
        (*E).mlsl_var_B = v6 as uint16;
        (*E).mlsl_var_F = (0xa2cfa9 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn MaridiaLargeSnail_Func_8(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_MaridiaLargeSnail = Get_MaridiaLargeSnail(k as ::core::ffi::c_int, game);
    let mut v2: int16 =
        ((*E).mlsl_var_B as ::core::ffi::c_int - 384 as ::core::ffi::c_int) as int16;
    (*E).mlsl_var_B = v2 as uint16;
    if v2 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
        let mut v3: uint16 = ((v2 as ::core::ffi::c_int & 0x7f00 as ::core::ffi::c_int)
            >> 8 as ::core::ffi::c_int) as uint16;
        let mut v5: ::core::ffi::c_int =
            8 as ::core::ffi::c_int * v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        Enemy_MoveDown(
            k,
            *(game.g_rom.offset(
                (((0xa0cbc7 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa0cbc7 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint32)
                .offset((v5 + 2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                as int32,
            game,
        );
    } else {
        (*E).mlsl_var_B = 0 as uint16;
        (*E).mlsl_var_F = (0xa2cf66 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn MaridiaLargeSnail_Func_9(game: &mut Game) {
    QueueSfx2_Max6(0x63 as uint16, game);
}
pub unsafe fn MaridiaLargeSnail_Func_10(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_MaridiaLargeSnail = Get_MaridiaLargeSnail(k as ::core::ffi::c_int, game);
    let mut mlsl_var_00: uint16 = (*E).mlsl_var_00;
    if mlsl_var_00 as ::core::ffi::c_int != (*E).mlsl_var_01 as ::core::ffi::c_int {
        (*E).mlsl_var_01 = mlsl_var_00;
        (*E).base.current_instruction = *(game.g_rom.offset(
            (((0xa2cb77 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa2cb77 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(mlsl_var_00 as isize);
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
    }
}
pub unsafe fn MaridiaLargeSnail_Func_11(game: &mut Game) {
    let mut E: *mut Enemy_MaridiaLargeSnail = Get_MaridiaLargeSnail(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).mlsl_var_04 = 0 as uint16;
    (*E).mlsl_var_05 = 0 as uint16;
    if IsSamusWithinEnemy_Y(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        0x20 as uint16,
        game,
    ) as ::core::ffi::c_int
        != 0
        && IsSamusWithinEnemy_X(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            0x18 as uint16,
            game,
        ) as ::core::ffi::c_int
            != 0
    {
        (*E).mlsl_var_04 = 1 as uint16;
        if GetSamusEnemyDelta_X(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        ) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            (*E).mlsl_var_05 = 1 as uint16;
        }
    }
}
pub unsafe fn MaridiaLargeSnail_Func_12(game: &mut Game) {
    Enemy_NormalTouchAI_A2(game);
    MaridiaLargeSnail_Touch(game);
}
pub unsafe fn MaridiaLargeSnail_Touch(game: &mut Game) {
    let mut k: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16);
    if CheckIfEnemyTouchesSamus(k, game) == 0 {
        if ((*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
            as ::core::ffi::c_int
            - (*Get_MaridiaLargeSnail(
                *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                    as ::core::ffi::c_int,
                game,
            ))
            .base
            .x_pos as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        {
            let ref mut fresh26 =
                *((&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16);
            *fresh26 = (*fresh26 as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as uint16;
        } else {
            let ref mut fresh27 =
                *((&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16);
            *fresh27 = (*fresh27 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
        }
    }
}
pub unsafe fn MaridiaLargeSnail_Shot(game: &mut Game) {
    NormalEnemyShotAi(game);
    QueueSfx2_Max6(0x57 as uint16, game);
}
unsafe fn HirisingSlowfalling_Func_1(mut k: uint16, game: &mut Game) -> uint16 {
    let mut E: *mut Enemy_HirisingSlowfalling =
        Get_HirisingSlowfalling(k as ::core::ffi::c_int, game);
    return Mult8x8((*E).hsg_parameter_1 as uint8, (*E).hsg_var_F as uint8);
}
pub unsafe fn HirisingSlowfalling_Init(game: &mut Game) {
    HirisingSlowfalling_Func_3(
        addr_kHirisingSlowfalling_Ilist_D82C as ::core::ffi::c_int as uint16,
        game,
    );
    let mut E: *mut Enemy_HirisingSlowfalling = Get_HirisingSlowfalling(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).hsg_var_A = (0xa2e035 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).hsg_var_00 = (*E).base.x_pos;
    (*E).hsg_var_01 = (*E).base.y_pos;
    let mut v2: ::core::ffi::c_int = *(&raw mut (*E).hsg_parameter_1 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>() as usize)
            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) as ::core::ffi::c_int;
    (*E).hsg_var_E = *(game.g_rom.offset(
        (((0xa2df5e as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa2df5e as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v2 as isize);
    (*E).hsg_var_F = *(RomPtr_A2(
        *(game.g_rom.offset(
            (((0xa2df6a as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa2df6a as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v2 as isize) as uint16_t,
        game,
    ) as *mut uint16);
    let mut r18: uint16 = HirisingSlowfalling_Func_1(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
    HirisingSlowfalling_Func_2(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        r18,
        game,
    );
    (*E).hsg_var_03 = ((*E).hsg_var_01 as ::core::ffi::c_int - r18 as ::core::ffi::c_int) as uint16;
    (*E).hsg_var_02 = (*E).hsg_var_00;
    (*E).hsg_var_B = (*E).hsg_var_06;
}
pub unsafe fn HirisingSlowfalling_Func_2(mut k: uint16, mut r18: uint16, game: &mut Game) {
    let mut v: uint32 = 0 as uint32;
    let mut r24: uint16 = 0 as uint16;
    loop {
        r24 = (r24 as ::core::ffi::c_int + 512 as ::core::ffi::c_int) as uint16;
        let mut v1: ::core::ffi::c_int =
            4 as ::core::ffi::c_int * (r24 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int);
        v = v.wrapping_add(
            (*(game.g_rom.offset(
                (((0xa2838f as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa2838f as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v1 + 1 as ::core::ffi::c_int) as isize) as uint32)
                << 16 as ::core::ffi::c_int
                | *(game.g_rom.offset(
                    (((0xa2838f as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0xa2838f as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset(v1 as isize) as uint32,
        );
        if !((v >> 16 as ::core::ffi::c_int).wrapping_sub(r18 as uint32) & 0x8000 as uint32 != 0) {
            break;
        }
    }
    (*Get_HirisingSlowfalling(k as ::core::ffi::c_int, game)).hsg_var_06 = r24;
}
pub unsafe fn HirisingSlowfalling_Func_3(mut a: uint16, game: &mut Game) {
    let mut E: *mut Enemy_HirisingSlowfalling = Get_HirisingSlowfalling(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.current_instruction = a;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
}
pub unsafe fn CallHirisingSlowfalling(mut ea: uint32, game: &mut Game) {
    match ea {
        10674229 => {
            HirisingSlowfalling_Func_4(game);
            return;
        }
        10674255 => {
            HirisingSlowfalling_Func_5(game);
            return;
        }
        10674282 => {
            HirisingSlowfalling_Func_6(game);
            return;
        }
        10674381 => {
            HirisingSlowfalling_Func_7(game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn HirisingSlowfalling_Main(game: &mut Game) {
    let mut E: *mut Enemy_HirisingSlowfalling = Get_HirisingSlowfalling(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallHirisingSlowfalling(
        ((*E).hsg_var_A as ::core::ffi::c_int | 0xa20000 as ::core::ffi::c_int) as uint32,
        game,
    );
}
pub unsafe fn HirisingSlowfalling_Func_4(game: &mut Game) {
    if IsSamusWithinEnemy_X(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        0x50 as uint16,
        game,
    ) != 0
    {
        let mut E: *mut Enemy_HirisingSlowfalling = Get_HirisingSlowfalling(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        );
        (*E).hsg_var_07 = (*E).hsg_parameter_2;
        (*E).hsg_var_A = (0xa2e04f as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn HirisingSlowfalling_Func_5(game: &mut Game) {
    let mut E: *mut Enemy_HirisingSlowfalling = Get_HirisingSlowfalling(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v1: int16 = ((*E).hsg_var_07 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as int16;
    (*E).hsg_var_07 = v1 as uint16;
    if (v1 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
        HirisingSlowfalling_Func_3(
            addr_kHirisingSlowfalling_Ilist_D834 as ::core::ffi::c_int as uint16,
            game,
        );
        (*E).hsg_var_A = (0xa2e06a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn HirisingSlowfalling_Func_6(game: &mut Game) {
    let mut E: *mut Enemy_HirisingSlowfalling = Get_HirisingSlowfalling(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v2: ::core::ffi::c_int = 4 as ::core::ffi::c_int
        * *(&raw mut (*E).hsg_var_B as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as ::core::ffi::c_int;
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        (*(game.g_rom.offset(
            (((0xa2838f as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa2838f as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((v2 + 3 as ::core::ffi::c_int) as isize) as uint32)
            << 16 as ::core::ffi::c_int
            | *(game.g_rom.offset(
                (((0xa2838f as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa2838f as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v2 + 2 as ::core::ffi::c_int) as isize) as uint32,
    );
    let mut v5: int16 = ((*E).hsg_var_B as ::core::ffi::c_int - 512 as ::core::ffi::c_int) as int16;
    (*E).hsg_var_B = v5 as uint16;
    if (v5 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
        (*E).hsg_var_04 = (*E).hsg_var_02;
        (*E).base.x_pos = (*E).hsg_var_04;
        (*E).hsg_var_05 = (*E).hsg_var_03;
        (*E).base.y_pos = (*E).hsg_var_05;
        (*E).hsg_var_C = 0 as uint16;
        (*E).hsg_var_D = (*(&raw mut (*E).hsg_parameter_1 as *mut uint8).offset(0)
            as ::core::ffi::c_int
            - 1 as ::core::ffi::c_int) as uint16;
        HirisingSlowfalling_Func_3(
            addr_kHirisingSlowfalling_Ilist_D840 as ::core::ffi::c_int as uint16,
            game,
        );
        (*E).hsg_var_A = (0xa2e0cd as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn HirisingSlowfalling_Func_7(game: &mut Game) {
    let mut E: *mut Enemy_HirisingSlowfalling = Get_HirisingSlowfalling(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v1: *const uint8 = RomPtr_A2(
        ((*E).hsg_var_E as ::core::ffi::c_int
            + 4 as ::core::ffi::c_int
                * *(&raw mut (*E).hsg_var_C as *mut uint8).offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ) as ::core::ffi::c_int) as uint16_t,
        game,
    );
    if *(v1 as *mut uint16) as ::core::ffi::c_int == 0x8000 as ::core::ffi::c_int {
        (*E).hsg_var_05 = ((*E).hsg_var_05 as ::core::ffi::c_int
            + (*E).hsg_var_F as ::core::ffi::c_int) as uint16;
        (*E).hsg_var_C = 0 as uint16;
        (*E).hsg_var_D = (*E).hsg_var_D.wrapping_sub(1);
        if (*E).hsg_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            (*E).hsg_var_B = (*E).hsg_var_06;
            (*E).base.x_pos = (*E).hsg_var_00;
            (*E).base.x_subpos = 0 as uint16;
            (*E).base.y_pos = (*E).hsg_var_01;
            (*E).base.y_subpos = 0 as uint16;
            HirisingSlowfalling_Func_3(
                addr_kHirisingSlowfalling_Ilist_D82C as ::core::ffi::c_int as uint16,
                game,
            );
            (*E).hsg_var_A =
                (0xa2e035 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
    } else {
        (*E).base.x_pos = (*(v1 as *mut uint16) as ::core::ffi::c_int
            + (*E).hsg_var_04 as ::core::ffi::c_int) as uint16;
        (*E).base.y_pos = (*(v1.offset(2) as *mut uint16) as ::core::ffi::c_int
            + (*E).hsg_var_05 as ::core::ffi::c_int) as uint16;
        (*E).hsg_var_C =
            ((*E).hsg_var_C as ::core::ffi::c_int + 256 as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn Gripper_Init(game: &mut Game) {
    let mut E: *mut Enemy_Gripper = Get_Gripper(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v1: uint16 = (8 as ::core::ffi::c_int
        * *(&raw mut (*E).base.current_instruction as *mut uint8).offset(0) as ::core::ffi::c_int)
        as uint16;
    (*E).gripper_var_E = v1;
    let mut v2: ::core::ffi::c_int = v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v3: uint16 = 0;
    if (*E).base.current_instruction as ::core::ffi::c_int & 0xfeff as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).gripper_var_D = *(game.g_rom.offset(
            (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v2 as isize);
        v3 = *(game.g_rom.offset(
            (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((v2 + 1 as ::core::ffi::c_int) as isize);
    } else {
        (*E).gripper_var_D = *(game.g_rom.offset(
            (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((v2 + 2 as ::core::ffi::c_int) as isize);
        v3 = *(game.g_rom.offset(
            (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((v2 + 3 as ::core::ffi::c_int) as isize);
    }
    (*E).gripper_var_C = v3;
    let mut v4: uint16 = addr_kGripper_Ilist_E19B as ::core::ffi::c_int as uint16;
    if (*E).gripper_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        v4 = addr_kGripper_Ilist_E1AF as ::core::ffi::c_int as uint16;
    }
    (*E).base.current_instruction = v4;
    (*E).gripper_var_A = (*E).gripper_parameter_1;
    (*E).gripper_var_B = (*E).gripper_parameter_2;
}
pub unsafe fn Gripper_Main(game: &mut Game) {
    let mut E: *mut Enemy_Gripper = Get_Gripper(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if Enemy_MoveRight_IgnoreSlopes(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        (((*E).gripper_var_D as uint32) << 16 as ::core::ffi::c_int | (*E).gripper_var_C as uint32)
            as int32,
        game,
    ) as ::core::ffi::c_int
        != 0
        || Gripper_Func_1(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        ) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            != 0
        || Gripper_Func_2(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        ) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            != 0
    {
        let mut v3: ::core::ffi::c_int =
            (*E).gripper_var_E as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v4: uint16 = 0;
        if (*E).gripper_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            (*E).gripper_var_D = *(game.g_rom.offset(
                (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v3 + 2 as ::core::ffi::c_int) as isize);
            (*E).gripper_var_C = *(game.g_rom.offset(
                (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v3 + 3 as ::core::ffi::c_int) as isize);
            v4 = addr_kGripper_Ilist_E19B as ::core::ffi::c_int as uint16;
        } else {
            (*E).gripper_var_D = *(game.g_rom.offset(
                (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v3 as isize);
            (*E).gripper_var_C = *(game.g_rom.offset(
                (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v3 + 1 as ::core::ffi::c_int) as isize);
            v4 = addr_kGripper_Ilist_E1AF as ::core::ffi::c_int as uint16;
        }
        (*E).base.current_instruction = v4;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
    }
}
pub unsafe fn Gripper_Func_1(mut k: uint16, game: &mut Game) -> uint8 {
    let mut E: *mut Enemy_Gripper = Get_Gripper(k as ::core::ffi::c_int, game);
    return ((*E).gripper_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
        && (((*E).base.x_pos as ::core::ffi::c_int - (*E).gripper_var_A as ::core::ffi::c_int)
            as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int) as ::core::ffi::c_int as uint8;
}
pub unsafe fn Gripper_Func_2(mut k: uint16, game: &mut Game) -> uint8 {
    let mut E: *mut Enemy_Gripper = Get_Gripper(k as ::core::ffi::c_int, game);
    return ((*E).gripper_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
        && ((*E).base.x_pos as ::core::ffi::c_int - (*E).gripper_var_B as ::core::ffi::c_int)
            as int16 as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int) as ::core::ffi::c_int as uint8;
}
pub unsafe fn Gripper_Func_3(game: &mut Game) {
    Enemy_NormalFrozenAI_A2(game);
}
pub unsafe fn Gripper_Func_4(game: &mut Game) {
    NormalEnemyShotAi(game);
    let mut E: *mut Enemy_Gripper = Get_Gripper(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).base.frozen_timer != 0 {
        let mut v1: uint16 = addr_kGripper_Sprmap_E43F as ::core::ffi::c_int as uint16;
        if (*E).gripper_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            v1 = addr_kGripper_Sprmap_E44B as ::core::ffi::c_int as uint16;
        }
        (*E).base.spritemap_pointer = v1;
    }
}
pub unsafe fn JetPowerRipper_Init(game: &mut Game) {
    let mut v0: uint16 = addr_kRipper_Ilist_E2E0 as ::core::ffi::c_int as uint16;
    let mut E: *mut Enemy_JetPowerRipper = Get_JetPowerRipper(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).jprr_parameter_2 != 0 {
        v0 = addr_kRipper_Ilist_E2F4 as ::core::ffi::c_int as uint16;
    }
    (*E).base.current_instruction = v0;
    let mut v2: uint16 =
        (8 as ::core::ffi::c_int * (*E).jprr_parameter_1 as ::core::ffi::c_int) as uint16;
    (*E).jprr_var_E = v2;
    let mut v3: ::core::ffi::c_int = v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v4: uint16 = 0;
    if (*E).jprr_parameter_2 != 0 {
        (*E).jprr_var_D = *(game.g_rom.offset(
            (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v3 as isize);
        v4 = *(game.g_rom.offset(
            (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((v3 + 1 as ::core::ffi::c_int) as isize);
    } else {
        (*E).jprr_var_D = *(game.g_rom.offset(
            (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((v3 + 2 as ::core::ffi::c_int) as isize);
        v4 = *(game.g_rom.offset(
            (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((v3 + 3 as ::core::ffi::c_int) as isize);
    }
    (*E).jprr_var_C = v4;
}
pub unsafe fn JetPowerRipper_Main(game: &mut Game) {
    let mut v3: int16 = 0;
    let mut E: *mut Enemy_JetPowerRipper = Get_JetPowerRipper(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if Enemy_MoveRight_IgnoreSlopes(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        (((*E).jprr_var_D as uint32) << 16 as ::core::ffi::c_int | (*E).jprr_var_C as uint32)
            as int32,
        game,
    ) != 0
    {
        let mut v2: ::core::ffi::c_int =
            (*E).jprr_var_E as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        if (*E).jprr_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            (*E).jprr_var_D = *(game.g_rom.offset(
                (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v2 + 2 as ::core::ffi::c_int) as isize);
            (*E).jprr_var_C = *(game.g_rom.offset(
                (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v2 + 3 as ::core::ffi::c_int) as isize);
            v3 = -(7456 as ::core::ffi::c_int) as int16;
        } else {
            (*E).jprr_var_D = *(game.g_rom.offset(
                (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v2 as isize);
            (*E).jprr_var_C = *(game.g_rom.offset(
                (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v2 + 1 as ::core::ffi::c_int) as isize);
            v3 = -(7436 as ::core::ffi::c_int) as int16;
        }
        (*E).base.current_instruction = v3 as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
    }
}
pub unsafe fn Ripper_Func_1(game: &mut Game) {
    Enemy_NormalFrozenAI_A2(game);
}
pub unsafe fn JetPowerRipper_Shot(game: &mut Game) {
    let mut v1: int16 = 0;
    NormalEnemyShotAi(game);
    let mut E: *mut Enemy_JetPowerRipper = Get_JetPowerRipper(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).base.frozen_timer != 0 {
        v1 = -(7105 as ::core::ffi::c_int) as int16;
        if (*E).jprr_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            v1 = -(7093 as ::core::ffi::c_int) as int16;
        }
        (*E).base.spritemap_pointer = v1 as uint16;
    }
}
pub unsafe fn Ripper_Init(game: &mut Game) {
    let mut v0: uint16 = addr_kRipper_Ilist_E477 as ::core::ffi::c_int as uint16;
    let mut E: *mut Enemy_Ripper = Get_Ripper(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).ripper_parameter_2 == 0 {
        v0 = addr_kRipper_Ilist_E48B as ::core::ffi::c_int as uint16;
    }
    (*E).base.current_instruction = v0;
    let mut v2: uint16 =
        (8 as ::core::ffi::c_int * (*E).ripper_parameter_1 as ::core::ffi::c_int) as uint16;
    (*E).ripper_var_E = v2;
    let mut v3: ::core::ffi::c_int = v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v4: uint16 = 0;
    if (*E).ripper_parameter_2 != 0 {
        (*E).ripper_var_D = *(game.g_rom.offset(
            (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v3 as isize);
        v4 = *(game.g_rom.offset(
            (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((v3 + 1 as ::core::ffi::c_int) as isize);
    } else {
        (*E).ripper_var_D = *(game.g_rom.offset(
            (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((v3 + 2 as ::core::ffi::c_int) as isize);
        v4 = *(game.g_rom.offset(
            (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((v3 + 3 as ::core::ffi::c_int) as isize);
    }
    (*E).ripper_var_C = v4;
}
pub unsafe fn Ripper_Main(game: &mut Game) {
    let mut v3: int16 = 0;
    let mut E: *mut Enemy_Ripper = Get_Ripper(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if Enemy_MoveRight_IgnoreSlopes(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        (((*E).ripper_var_D as uint32) << 16 as ::core::ffi::c_int | (*E).ripper_var_C as uint32)
            as int32,
        game,
    ) != 0
    {
        let mut v2: ::core::ffi::c_int =
            (*E).ripper_var_E as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        if (*E).ripper_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            (*E).ripper_var_D = *(game.g_rom.offset(
                (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v2 + 2 as ::core::ffi::c_int) as isize);
            (*E).ripper_var_C = *(game.g_rom.offset(
                (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v2 + 3 as ::core::ffi::c_int) as isize);
            v3 = -(7029 as ::core::ffi::c_int) as int16;
        } else {
            (*E).ripper_var_D = *(game.g_rom.offset(
                (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v2 as isize);
            (*E).ripper_var_C = *(game.g_rom.offset(
                (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v2 + 1 as ::core::ffi::c_int) as isize);
            v3 = -(7049 as ::core::ffi::c_int) as int16;
        }
        (*E).base.current_instruction = v3 as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
    }
}
pub unsafe fn LavaSeahorse_Instr_E5FB(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_LavaSeahorse(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .lse_var_02 = 1 as uint16;
    return jp;
}
pub unsafe fn LavaSeahorse_Init(game: &mut Game) {
    let mut E: *mut Enemy_LavaSeahorse = Get_LavaSeahorse(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).lse_var_02 = 0 as uint16;
    if (*E).lse_parameter_1 != 0 {
        (*E).lse_var_00 = 2 as uint16;
        (*E).lse_var_01 = 2 as uint16;
        (*E).base.current_instruction =
            addr_kLavaSeahorse_Ilist_E5A1 as ::core::ffi::c_int as uint16;
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            | kEnemyProps_Tangible as ::core::ffi::c_int) as uint16;
        (*E).lse_var_F = (0xa2e781 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    } else {
        (*E).lse_var_00 = 0 as uint16;
        (*E).lse_var_01 = 0 as uint16;
        (*E).base.current_instruction =
            addr_kLavaSeahorse_Ilist_E59B as ::core::ffi::c_int as uint16;
        (*E).lse_var_F = (0xa2e654 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn LavaSeahorse_Main(game: &mut Game) {
    let mut E: *mut Enemy_LavaSeahorse = Get_LavaSeahorse(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallEnemyPreInstr(
        ((*E).lse_var_F as ::core::ffi::c_int | 0xa20000 as ::core::ffi::c_int) as uint32,
        game,
    );
}
pub unsafe fn LavaSeahorse_Func_1(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_LavaSeahorse = Get_LavaSeahorse(k as ::core::ffi::c_int, game);
    (*E).lse_var_D = (*E).lse_var_D.wrapping_sub(1);
    if (*E).lse_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).lse_var_D = 48 as uint16;
        (*E).lse_var_F = (0xa2e6ad as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        let mut t: uint16 = GetSamusEnemyDelta_X(k, game);
        (*E).lse_var_A = ((*E).lse_var_A as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int
            | t as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int)
            as uint16;
        if (*E).lse_var_A as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            let ref mut fresh16 = *(&raw mut (*E).lse_var_00 as *mut uint8).offset(0);
            *fresh16 = (*fresh16 as ::core::ffi::c_int & !(1 as ::core::ffi::c_int)) as uint8;
            let mut E1: *mut Enemy_LavaSeahorse =
                Get_LavaSeahorse(k as ::core::ffi::c_int + 64 as ::core::ffi::c_int, game);
            let ref mut fresh17 = *(&raw mut (*E1).lse_var_00 as *mut uint8).offset(0);
            *fresh17 = (*fresh17 as ::core::ffi::c_int & !(1 as ::core::ffi::c_int)) as uint8;
        } else {
            let ref mut fresh18 = *(&raw mut (*E).lse_var_00 as *mut uint8).offset(0);
            *fresh18 = (*fresh18 as ::core::ffi::c_int | 1 as ::core::ffi::c_int) as uint8;
            let mut E1_0: *mut Enemy_LavaSeahorse =
                Get_LavaSeahorse(k as ::core::ffi::c_int + 64 as ::core::ffi::c_int, game);
            let ref mut fresh19 = *(&raw mut (*E1_0).lse_var_00 as *mut uint8).offset(0);
            *fresh19 = (*fresh19 as ::core::ffi::c_int | 1 as ::core::ffi::c_int) as uint8;
        }
        LavaSeahorse_Func_6(game);
        LavaSeahorse_Func_7(game);
    }
}
pub unsafe fn LavaSeahorse_Func_2(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_LavaSeahorse = Get_LavaSeahorse(k as ::core::ffi::c_int, game);
    (*E).lse_var_D = (*E).lse_var_D.wrapping_sub(1);
    if (*E).lse_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).lse_var_00 =
            ((*E).lse_var_00 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
        (*E).lse_var_D = 3 as uint16;
        (*E).lse_var_F = (0xa2e6f1 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
    let mut E1: *mut Enemy_LavaSeahorse =
        Get_LavaSeahorse(k as ::core::ffi::c_int + 64 as ::core::ffi::c_int, game);
    (*E1).base.y_pos = (*E1).base.y_pos.wrapping_sub(1);
}
pub unsafe fn LavaSeahorse_Func_3(mut k: uint16, game: &mut Game) {
    LavaSeahorse_Func_6(game);
    let mut E: *mut Enemy_LavaSeahorse = Get_LavaSeahorse(k as ::core::ffi::c_int, game);
    if (*E).lse_var_02 != 0 {
        (*E).lse_var_02 = 0 as uint16;
        (*E).lse_var_01 = -(1 as ::core::ffi::c_int) as uint16;
        SpawnEprojWithGfx(
            0xffff as uint16,
            k,
            addr_kEproj_LavaSeahorseFireball as ::core::ffi::c_int as uint16,
            game,
        );
        QueueSfx2_Max6(0x61 as uint16, game);
        let fresh20 = (*E).lse_var_D;
        (*E).lse_var_D = (*E).lse_var_D.wrapping_sub(1);
        if fresh20 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
            (*E).lse_var_00 =
                ((*E).lse_var_00 as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as uint16;
            (*E).lse_var_D = 96 as uint16;
            (*E).lse_var_F =
                (0xa2e734 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
    }
}
pub unsafe fn LavaSeahorse_Func_4(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_LavaSeahorse = Get_LavaSeahorse(k as ::core::ffi::c_int, game);
    let fresh21 = (*E).lse_var_D;
    (*E).lse_var_D = (*E).lse_var_D.wrapping_sub(1);
    if fresh21 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        (*E).lse_var_D = 48 as uint16;
        (*E).lse_var_F = (0xa2e749 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        LavaSeahorse_Func_6(game);
    }
}
pub unsafe fn LavaSeahorse_Func_5(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_LavaSeahorse = Get_LavaSeahorse(k as ::core::ffi::c_int, game);
    (*E).lse_var_D = (*E).lse_var_D.wrapping_sub(1);
    if (*E).lse_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).lse_var_D = 128 as uint16;
        (*E).lse_var_F = (0xa2e654 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    (*E).base.y_pos = (*E).base.y_pos.wrapping_add(1);
    let mut E1: *mut Enemy_LavaSeahorse =
        Get_LavaSeahorse(k as ::core::ffi::c_int + 64 as ::core::ffi::c_int, game);
    (*E1).base.y_pos = (*E1).base.y_pos.wrapping_add(1);
}
pub unsafe fn LavaSeahorse_Func_6(game: &mut Game) {
    let mut E: *mut Enemy_LavaSeahorse = Get_LavaSeahorse(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut lse_var_00: uint16 = (*E).lse_var_00;
    if lse_var_00 as ::core::ffi::c_int != (*E).lse_var_01 as ::core::ffi::c_int {
        (*E).lse_var_01 = lse_var_00;
        (*E).base.current_instruction = *(game.g_rom.offset(
            (((0xa2e5ef as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa2e5ef as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(lse_var_00 as isize);
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
    }
}
pub unsafe fn LavaSeahorse_Func_7(game: &mut Game) {
    let mut E: *mut Enemy_LavaSeahorse = Get_LavaSeahorse(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    let mut lse_var_00: uint16 = (*E).lse_var_00;
    if lse_var_00 as ::core::ffi::c_int != (*E).lse_var_01 as ::core::ffi::c_int {
        (*E).lse_var_01 = lse_var_00;
        (*E).base.current_instruction = *(game.g_rom.offset(
            (((0xa2e5ef as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0xa2e5ef as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(lse_var_00 as isize);
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
    }
}
pub unsafe fn LavaSeahorse_Touch(game: &mut Game) {
    Enemy_NormalTouchAI_A2(game);
    LavaSeahorse_E7DA(game);
}
pub unsafe fn LavaSeahorse_Shot(game: &mut Game) {
    Enemy_NormalShotAI_A2(game);
    LavaSeahorse_E7DA(game);
}
pub unsafe fn LavaSeahorse_Powerbomb(game: &mut Game) {
    Enemy_NormalPowerBombAI_A2(game);
    LavaSeahorse_E7DA(game);
}
pub unsafe fn LavaSeahorse_E7DA(game: &mut Game) {
    let mut E: *mut Enemy_LavaSeahorse = Get_LavaSeahorse(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).base.health != 0 {
        let mut shake_timer: uint16 = (*E).base.shake_timer;
        let mut E1: *mut Enemy_LavaSeahorse = Get_LavaSeahorse(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int
                + 64 as ::core::ffi::c_int,
            game,
        );
        (*E1).base.shake_timer = shake_timer;
        (*E1).base.invincibility_timer = (*E).base.invincibility_timer;
        (*E1).base.flash_timer = (*E).base.flash_timer;
        (*E1).base.frozen_timer = (*E).base.frozen_timer;
        (*E1).base.ai_handler_bits = (*E).base.ai_handler_bits;
    } else {
        let mut E1_0: *mut Enemy_LavaSeahorse = Get_LavaSeahorse(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int
                + 64 as ::core::ffi::c_int,
            game,
        );
        (*E1_0).base.properties =
            ((*E1_0).base.properties as ::core::ffi::c_int | 0x200 as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn TimedShutter_Init(game: &mut Game) {
    let mut E: *mut Enemy_TimedShutter = Get_TimedShutter(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).tsr_var_A = *(game.g_rom.offset(
        (((0xa2ea4e as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa2ea4e as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            ((*E).base.current_instruction as ::core::ffi::c_int
                + 2 as ::core::ffi::c_int * (*E).base.extra_properties as ::core::ffi::c_int)
                as isize,
        );
    let mut y_pos: uint16 = (*E).base.y_pos;
    let mut v3: uint16 = 0;
    if (*E).base.extra_properties != 0 {
        (*E).tsr_var_B = y_pos;
        let mut v2: uint16 = (y_pos as ::core::ffi::c_int - 8 as ::core::ffi::c_int) as uint16;
        (*E).tsr_var_C = v2;
        v2 = (v2 as ::core::ffi::c_int - 8 as ::core::ffi::c_int) as uint16;
        (*E).tsr_var_D = v2;
        v3 = (v2 as ::core::ffi::c_int - 8 as ::core::ffi::c_int) as uint16;
    } else {
        (*E).tsr_var_B = y_pos;
        let mut v4: uint16 = (y_pos as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint16;
        (*E).tsr_var_C = v4;
        v4 = (v4 as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint16;
        (*E).tsr_var_D = v4;
        v3 = (v4 as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint16;
    }
    (*E).tsr_var_E = v3;
    (*E).base.extra_properties = 0 as uint16;
    (*E).tsr_var_F = 0 as uint16;
    (*E).base.current_instruction = addr_kTimedShutter_Ilist_E998 as ::core::ffi::c_int as uint16;
    let mut v5: ::core::ffi::c_int = (4 as ::core::ffi::c_int
        * *(&raw mut (*E).tsr_parameter_2 as *mut uint8).offset(0) as ::core::ffi::c_int)
        as uint16 as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    (*E).tsr_var_00 = *(game.g_rom.offset(
        (((0xa2ea56 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa2ea56 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v5 as isize);
    (*E).tsr_var_01 = *(game.g_rom.offset(
        (((0xa2ea56 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa2ea56 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((v5 + 1 as ::core::ffi::c_int) as isize);
}
pub unsafe fn CallTimedShutterFunc(mut ea: uint32, mut k: uint16, game: &mut Game) {
    match ea {
        10676925 => {
            TimedShutter_Func_1(k, game);
            return;
        }
        10676945 => {
            TimedShutter_Func_2(k, game);
            return;
        }
        10676967 => {
            TimedShutter_Func_3(k, game);
            return;
        }
        10676989 => {
            TimedShutter_Func_4(k, game);
            return;
        }
        10677009 => {
            TimedShutter_Func_5(k, game);
            return;
        }
        10677267 => {
            TimedShutter_Func_10(k, game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn TimedShutter_Main(game: &mut Game) {
    let mut E: *mut Enemy_TimedShutter = Get_TimedShutter(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallTimedShutterFunc(
        ((*E).tsr_var_A as ::core::ffi::c_int | 0xa20000 as ::core::ffi::c_int) as uint32,
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
}
pub unsafe fn TimedShutter_Func_1(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_TimedShutter = Get_TimedShutter(k as ::core::ffi::c_int, game);
    let mut tsr_parameter_1: uint16 = (*E).tsr_parameter_1;
    if tsr_parameter_1 != 0 {
        (*E).tsr_parameter_1 =
            (tsr_parameter_1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
    } else {
        RisingFallingPlatform_Func_8(game);
        (*E).tsr_var_A = (0xa2ec13 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn TimedShutter_Func_2(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_TimedShutter = Get_TimedShutter(k as ::core::ffi::c_int, game);
    if IsSamusWithinEnemy_X(k, (*E).tsr_parameter_1, game) != 0 {
        RisingFallingPlatform_Func_8(game);
        (*E).tsr_var_A = (0xa2ec13 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn TimedShutter_Func_3(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_TimedShutter = Get_TimedShutter(k as ::core::ffi::c_int, game);
    if IsSamusWithinEnemy_X(k, (*E).tsr_parameter_1, game) != 0 {
        RisingFallingPlatform_Func_8(game);
        (*E).tsr_var_A = (0xa2eb11 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn TimedShutter_Func_4(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_TimedShutter = Get_TimedShutter(k as ::core::ffi::c_int, game);
    let mut tsr_parameter_1: uint16 = (*E).tsr_parameter_1;
    if tsr_parameter_1 != 0 {
        (*E).tsr_parameter_1 =
            (tsr_parameter_1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
    } else {
        RisingFallingPlatform_Func_8(game);
        (*E).tsr_var_A = (0xa2eb11 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn TimedShutter_Func_Null(game: &mut Game) {}
static off_A2EB1A: [Option<Func_V>; 5] = unsafe {
    [
        Some(TimedShutter_Func_6 as unsafe fn(&mut Game) -> ()),
        Some(TimedShutter_Func_7 as unsafe fn(&mut Game) -> ()),
        Some(TimedShutter_Func_8 as unsafe fn(&mut Game) -> ()),
        Some(TimedShutter_Func_9 as unsafe fn(&mut Game) -> ()),
        Some(TimedShutter_Func_Null as unsafe fn(&mut Game) -> ()),
    ]
};
pub unsafe fn TimedShutter_Func_5(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int =
        (*Get_TimedShutter(k as ::core::ffi::c_int, game)).tsr_var_F as ::core::ffi::c_int;
    off_A2EB1A[v1 as usize].expect("non-null function pointer")(game);
}
pub unsafe fn TimedShutter_Func_6(game: &mut Game) {
    let mut E: *mut Enemy_TimedShutter = Get_TimedShutter(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        ((*E).tsr_var_00 as uint32) << 16 as ::core::ffi::c_int | (*E).tsr_var_01 as uint32,
    );
    if (((*E).tsr_var_B as ::core::ffi::c_int + 16 as ::core::ffi::c_int
        - (*E).base.y_pos as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        (*E).base.y_pos =
            ((*E).tsr_var_B as ::core::ffi::c_int + 9 as ::core::ffi::c_int) as uint16;
        (*E).tsr_var_F = (*E).tsr_var_F.wrapping_add(1);
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.current_instruction =
            addr_kTimedShutter_Ilist_E99E as ::core::ffi::c_int as uint16;
        (*E).base.y_height = 16 as uint16;
    }
}
pub unsafe fn TimedShutter_Func_7(game: &mut Game) {
    let mut E: *mut Enemy_TimedShutter = Get_TimedShutter(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        ((*E).tsr_var_00 as uint32) << 16 as ::core::ffi::c_int | (*E).tsr_var_01 as uint32,
    );
    if (((*E).tsr_var_C as ::core::ffi::c_int + 16 as ::core::ffi::c_int
        - (*E).base.y_pos as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        (*E).base.y_pos =
            ((*E).tsr_var_C as ::core::ffi::c_int + 9 as ::core::ffi::c_int) as uint16;
        (*E).tsr_var_F = (*E).tsr_var_F.wrapping_add(1);
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.current_instruction =
            addr_kTimedShutter_Ilist_E9A4 as ::core::ffi::c_int as uint16;
        (*E).base.y_height = 24 as uint16;
    }
}
pub unsafe fn TimedShutter_Func_8(game: &mut Game) {
    let mut E: *mut Enemy_TimedShutter = Get_TimedShutter(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        ((*E).tsr_var_00 as uint32) << 16 as ::core::ffi::c_int | (*E).tsr_var_01 as uint32,
    );
    if (((*E).tsr_var_D as ::core::ffi::c_int + 16 as ::core::ffi::c_int
        - (*E).base.y_pos as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        (*E).base.y_pos =
            ((*E).tsr_var_D as ::core::ffi::c_int + 9 as ::core::ffi::c_int) as uint16;
        (*E).tsr_var_F = (*E).tsr_var_F.wrapping_add(1);
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.current_instruction =
            addr_kTimedShutter_Ilist_E9AA as ::core::ffi::c_int as uint16;
        (*E).base.y_height = 32 as uint16;
    }
}
pub unsafe fn TimedShutter_Func_9(game: &mut Game) {
    let mut E: *mut Enemy_TimedShutter = Get_TimedShutter(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        ((*E).tsr_var_00 as uint32) << 16 as ::core::ffi::c_int | (*E).tsr_var_01 as uint32,
    );
    let mut v3: uint16 =
        ((*E).tsr_var_E as ::core::ffi::c_int + 16 as ::core::ffi::c_int) as uint16;
    if ((v3 as ::core::ffi::c_int - (*E).base.y_pos as ::core::ffi::c_int) as int16
        as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        (*E).base.y_pos = v3;
        (*E).tsr_var_F = (*E).tsr_var_F.wrapping_add(1);
    }
}
pub unsafe fn TimedShutter_Func_Null2(game: &mut Game) {}
static off_A2EC3A: [Option<Func_V>; 5] = unsafe {
    [
        Some(TimedShutter_Func_11 as unsafe fn(&mut Game) -> ()),
        Some(TimedShutter_Func_12 as unsafe fn(&mut Game) -> ()),
        Some(TimedShutter_Func_13 as unsafe fn(&mut Game) -> ()),
        Some(TimedShutter_Func_14 as unsafe fn(&mut Game) -> ()),
        Some(TimedShutter_Func_Null2 as unsafe fn(&mut Game) -> ()),
    ]
};
pub unsafe fn TimedShutter_Func_10(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_TimedShutter = Get_TimedShutter(k as ::core::ffi::c_int, game);
    (*E).tsr_var_40 = (*E).base.y_pos;
    let mut v2: uint16 = (2 as ::core::ffi::c_int * (*E).tsr_var_F as ::core::ffi::c_int) as uint16;
    off_A2EC3A[(v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize]
        .expect("non-null function pointer")(game);
    if CheckIfEnemyTouchesSamus(k, game) != 0 {
        let mut v4: int16 = ((*E).base.y_pos as ::core::ffi::c_int
            - (*E).tsr_var_40 as ::core::ffi::c_int) as int16;
        if (v4 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
            let ref mut fresh30 =
                *((&raw mut game.g_ram as *mut uint8).offset(0xb5c) as *mut uint16);
            *fresh30 = (*fresh30 as ::core::ffi::c_int + v4 as ::core::ffi::c_int) as uint16;
        }
    }
}
pub unsafe fn TimedShutter_Func_11(game: &mut Game) {
    let mut E: *mut Enemy_TimedShutter = Get_TimedShutter(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        -((((*E).tsr_var_00 as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
            | (*E).tsr_var_01 as ::core::ffi::c_int) as int32) as uint32,
    );
    let mut v3: int16 = ((*E).tsr_var_B as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as int16;
    if (v3 as ::core::ffi::c_int - (*E).base.y_pos as ::core::ffi::c_int) as int16
        as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        (*E).base.y_pos = (v3 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
        (*E).tsr_var_F = (*E).tsr_var_F.wrapping_add(1);
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.current_instruction =
            addr_kTimedShutter_Ilist_E99E as ::core::ffi::c_int as uint16;
        (*E).base.y_height = 16 as uint16;
    }
}
pub unsafe fn TimedShutter_Func_12(game: &mut Game) {
    let mut E: *mut Enemy_TimedShutter = Get_TimedShutter(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        -((((*E).tsr_var_00 as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
            | (*E).tsr_var_01 as ::core::ffi::c_int) as int32) as uint32,
    );
    let mut v3: int16 = ((*E).tsr_var_C as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as int16;
    if (v3 as ::core::ffi::c_int - (*E).base.y_pos as ::core::ffi::c_int) as int16
        as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        (*E).base.y_pos = (v3 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
        (*E).tsr_var_F = (*E).tsr_var_F.wrapping_add(1);
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.current_instruction =
            addr_kTimedShutter_Ilist_E9A4 as ::core::ffi::c_int as uint16;
        (*E).base.y_height = 24 as uint16;
    }
}
pub unsafe fn TimedShutter_Func_13(game: &mut Game) {
    let mut E: *mut Enemy_TimedShutter = Get_TimedShutter(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        -((((*E).tsr_var_00 as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
            | (*E).tsr_var_01 as ::core::ffi::c_int) as int32) as uint32,
    );
    let mut v3: int16 = ((*E).tsr_var_D as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as int16;
    if (v3 as ::core::ffi::c_int - (*E).base.y_pos as ::core::ffi::c_int) as int16
        as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        (*E).base.y_pos = (v3 as ::core::ffi::c_int + 7 as ::core::ffi::c_int) as uint16;
        (*E).tsr_var_F = (*E).tsr_var_F.wrapping_add(1);
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.current_instruction =
            addr_kTimedShutter_Ilist_E9AA as ::core::ffi::c_int as uint16;
        (*E).base.y_height = 32 as uint16;
    }
}
pub unsafe fn TimedShutter_Func_14(game: &mut Game) {
    let mut E: *mut Enemy_TimedShutter = Get_TimedShutter(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        -((((*E).tsr_var_00 as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
            | (*E).tsr_var_01 as ::core::ffi::c_int) as int32) as uint32,
    );
    let mut v3: uint16 =
        ((*E).tsr_var_E as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as uint16;
    if (v3 as ::core::ffi::c_int - (*E).base.y_pos as ::core::ffi::c_int) as int16
        as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        (*E).base.y_pos = v3;
        (*E).tsr_var_F = (*E).tsr_var_F.wrapping_add(1);
    }
}
pub unsafe fn TimedShutter_Func_15() {}
pub unsafe fn RisingFallingPlatform_Init(game: &mut Game) {
    RisingFallingPlatform_Func_1(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
    (*Get_RisingFallingPlatform(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .base
    .current_instruction = addr_kRisingFallingPlatform_Ilist_EDE7 as ::core::ffi::c_int as uint16;
}
pub unsafe fn ShootableShutter_Init(game: &mut Game) {
    RisingFallingPlatform_Func_1(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
    (*gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .current_instruction = addr_kTimedShutter_Ilist_E9AA as ::core::ffi::c_int as uint16;
}
pub unsafe fn RisingFallingPlatform_Func_1(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_RisingFallingPlatform =
        Get_RisingFallingPlatform(k as ::core::ffi::c_int, game);
    let mut current_instruction_low: uint16 =
        *(&raw mut (*E).base.current_instruction as *mut uint8).offset(0) as uint16;
    (*E).rfpm_var_00 = current_instruction_low;
    let mut v3: ::core::ffi::c_int = 8 as ::core::ffi::c_int
        * current_instruction_low as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    (*E).rfpm_var_D = *(game.g_rom.offset(
        (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v3 as isize);
    (*E).rfpm_var_C = *(game.g_rom.offset(
        (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((v3 + 1 as ::core::ffi::c_int) as isize);
    (*E).rfpm_var_F = *(game.g_rom.offset(
        (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((v3 + 2 as ::core::ffi::c_int) as isize);
    (*E).rfpm_var_E = *(game.g_rom.offset(
        (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((v3 + 3 as ::core::ffi::c_int) as isize);
    let mut current_instruction_high: uint16 =
        *(&raw mut (*E).base.current_instruction as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as uint16;
    (*E).rfpm_var_01 = current_instruction_high;
    (*E).rfpm_var_20 = current_instruction_high;
    let mut extra_properties_low: uint16 =
        *(&raw mut (*E).base.extra_properties as *mut uint8).offset(0) as uint16;
    (*E).rfpm_var_02 = extra_properties_low;
    (*E).rfpm_var_08 =
        (16 as ::core::ffi::c_int * extra_properties_low as ::core::ffi::c_int) as uint16;
    let mut extra_properties_high: uint16 = *(&raw mut (*E).base.extra_properties as *mut uint8)
        .offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as uint16;
    (*E).rfpm_var_03 = extra_properties_high;
    (*E).rfpm_var_09 =
        (16 as ::core::ffi::c_int * extra_properties_high as ::core::ffi::c_int) as uint16;
    let mut rfpm_parameter_1_low: uint16 =
        *(&raw mut (*E).rfpm_parameter_1 as *mut uint8).offset(0) as uint16;
    (*E).rfpm_var_04 = rfpm_parameter_1_low;
    (*E).rfpm_var_07 =
        (2 as ::core::ffi::c_int * rfpm_parameter_1_low as ::core::ffi::c_int) as uint16;
    (*E).rfpm_var_05 = *(&raw mut (*E).rfpm_parameter_1 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>() as usize)
            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) as uint16;
    let mut rfpm_parameter_2: uint16 = (*E).rfpm_parameter_2;
    (*E).rfpm_var_06 = rfpm_parameter_2;
    (*E).rfpm_var_B = rfpm_parameter_2;
    let mut y_pos: uint16 = (*E).base.y_pos;
    (*E).rfpm_var_0F = y_pos;
    (*E).rfpm_var_10 =
        ((*E).rfpm_var_05 as ::core::ffi::c_int + y_pos as ::core::ffi::c_int) as uint16;
    if (*E).rfpm_var_01 == 0 {
        let mut v10: uint16 = (*E).base.y_pos;
        (*E).rfpm_var_10 = v10;
        (*E).rfpm_var_0F =
            (v10 as ::core::ffi::c_int - (*E).rfpm_var_05 as ::core::ffi::c_int) as uint16;
    }
    (*E).rfpm_var_A = (0xa2ef09 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).base.extra_properties = 0 as uint16;
    (*E).rfpm_var_0A = 0 as uint16;
}
pub unsafe fn CallRisingFallingPlatformFunc(mut ea: uint32, game: &mut Game) {
    match ea {
        10678025 => {
            RisingFallingPlatform_Func_2(game);
            return;
        }
        10678056 => {
            RisingFallingPlatform_Func_4(game);
            return;
        }
        10678120 => {
            RisingFallingPlatform_Func_9(game);
            return;
        }
        10678228 => {
            RisingFallingPlatform_Func_10(game);
            return;
        }
        10678336 => {
            RisingFallingPlatform_Func_11(game);
            return;
        }
        10678386 => {
            RisingFallingPlatform_Func_12(game);
            return;
        }
        10678425 => {
            RisingFallingPlatform_Func_13();
            return;
        }
        10678820 => {
            HorizontalShootableShutter_Func_2(game);
            return;
        }
        10678851 => {
            HorizontalShootableShutter_Func_4(game);
            return;
        }
        10678898 => {
            HorizontalShootableShutter_Func_8(game);
            return;
        }
        10679012 => {
            HorizontalShootableShutter_Func_9(game);
            return;
        }
        10679180 => {
            HorizontalShootableShutter_Func_12(game);
            return;
        }
        10679216 => {
            HorizontalShootableShutter_Func_13(game);
            return;
        }
        10679252 => {
            HorizontalShootableShutter_Func_14();
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn RisingFallingPlatform_Main(game: &mut Game) {
    let mut E: *mut Enemy_RisingFallingPlatform = Get_RisingFallingPlatform(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallRisingFallingPlatformFunc(
        ((*E).rfpm_var_A as ::core::ffi::c_int | 0xa20000 as ::core::ffi::c_int) as uint32,
        game,
    );
    if (*E).rfpm_var_A as ::core::ffi::c_int
        != 0xa2ef68 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
        && (*E).rfpm_var_A as ::core::ffi::c_int
            != 0xa2efd4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
        && *((&raw mut game.g_ram as *mut uint8).offset(0x182c) as *mut uint16).offset(3)
            as ::core::ffi::c_int
            & (*((&raw mut game.g_ram as *mut uint8).offset(0x182c) as *mut uint16).offset(2)
                as ::core::ffi::c_int
                & *((&raw mut game.g_ram as *mut uint8).offset(0x182c) as *mut uint16).offset(1)
                    as ::core::ffi::c_int
                & *((&raw mut game.g_ram as *mut uint8).offset(0x182c) as *mut uint16).offset(0)
                    as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
            != 0xffff as ::core::ffi::c_int
        && *((&raw mut game.g_ram as *mut uint8).offset(0x182c) as *mut uint16).offset(3)
            as ::core::ffi::c_int
            & (*((&raw mut game.g_ram as *mut uint8).offset(0x182c) as *mut uint16).offset(2)
                as ::core::ffi::c_int
                & *((&raw mut game.g_ram as *mut uint8).offset(0x182c) as *mut uint16).offset(1)
                    as ::core::ffi::c_int
                & *((&raw mut game.g_ram as *mut uint8).offset(0x182c) as *mut uint16).offset(0)
                    as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
            == *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0xa6e) as *mut uint16) != 0 {
            RisingFallingPlatform_Powerbomb(game);
        }
    }
}
static off_A2EDFB: [Option<Func_V>; 5] = unsafe {
    [
        Some(RisingFallingPlatform_Func_3 as unsafe fn(&mut Game) -> ()),
        Some(RisingFallingPlatform_Func_4 as unsafe fn(&mut Game) -> ()),
        Some(RisingFallingPlatform_Func_5 as unsafe fn(&mut Game) -> ()),
        Some(RisingFallingPlatform_Func_6 as unsafe fn(&mut Game) -> ()),
        Some(RisingFallingPlatform_Func_6 as unsafe fn(&mut Game) -> ()),
    ]
};
pub unsafe fn RisingFallingPlatform_Func_2(game: &mut Game) {
    let mut v0: ::core::ffi::c_int = (*Get_RisingFallingPlatform(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .rfpm_var_07 as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    off_A2EDFB[v0 as usize].expect("non-null function pointer")(game);
}
pub unsafe fn RisingFallingPlatform_Func_3(game: &mut Game) {
    let mut E: *mut Enemy_RisingFallingPlatform = Get_RisingFallingPlatform(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let fresh15 = (*E).rfpm_var_B;
    (*E).rfpm_var_B = (*E).rfpm_var_B.wrapping_sub(1);
    if fresh15 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        (*E).rfpm_var_B = (*E).rfpm_var_06;
        RisingFallingPlatform_Func_7(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        );
    }
}
pub unsafe fn RisingFallingPlatform_Func_4(game: &mut Game) {
    let mut E: *mut Enemy_RisingFallingPlatform = Get_RisingFallingPlatform(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if IsSamusWithinEnemy_X(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        (*E).rfpm_var_06,
        game,
    ) != 0
    {
        RisingFallingPlatform_Func_7(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        );
    }
}
pub unsafe fn RisingFallingPlatform_Func_5(game: &mut Game) {
    RisingFallingPlatform_Func_7(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
}
pub unsafe fn RisingFallingPlatform_Func_6(game: &mut Game) {}
pub unsafe fn RisingFallingPlatform_Func_7(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_RisingFallingPlatform =
        Get_RisingFallingPlatform(k as ::core::ffi::c_int, game);
    (*E).rfpm_var_A = (0xa2ef68 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    if (*E).rfpm_var_01 != 0 {
        (*E).rfpm_var_A = (0xa2efd4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    RisingFallingPlatform_Func_8(game);
}
pub unsafe fn RisingFallingPlatform_Func_8(game: &mut Game) {
    if CheckIfEnemyIsOnScreen(game) == 0 {
        QueueSfx3_Max6(0xe as uint16, game);
    }
}
pub unsafe fn RisingFallingPlatform_Func_9(game: &mut Game) {
    let mut E: *mut Enemy_RisingFallingPlatform = Get_RisingFallingPlatform(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).rfpm_var_0E = (*E).base.y_pos;
    (*E).rfpm_var_0A = 0 as uint16;
    if CheckIfEnemyTouchesSamus(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    ) != 0
    {
        (*E).rfpm_var_0A = 1 as uint16;
    }
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        ((*E).rfpm_var_F as uint32) << 16 as ::core::ffi::c_int | (*E).rfpm_var_E as uint32,
    );
    if (*E).rfpm_var_0A != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0xb5c) as *mut uint16) =
            ((*E).base.y_pos as ::core::ffi::c_int - (*E).rfpm_var_0E as ::core::ffi::c_int)
                as uint16;
    }
    if ((*E).rfpm_var_0F as ::core::ffi::c_int - (*E).base.y_pos as ::core::ffi::c_int) as int16
        as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        if (*E).rfpm_var_08 as ::core::ffi::c_int == 4080 as ::core::ffi::c_int {
            (*E).rfpm_var_A =
                (0xa2f099 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        } else {
            (*E).rfpm_var_B = (*E).rfpm_var_08;
            (*E).rfpm_var_A =
                (0xa2f040 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
    }
}
pub unsafe fn RisingFallingPlatform_Func_10(game: &mut Game) {
    let mut E: *mut Enemy_RisingFallingPlatform = Get_RisingFallingPlatform(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).rfpm_var_0E = (*E).base.y_pos;
    (*E).rfpm_var_0A = 0 as uint16;
    if CheckIfEnemyTouchesSamus(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    ) != 0
    {
        (*E).rfpm_var_0A = 1 as uint16;
    }
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        ((*E).rfpm_var_D as uint32) << 16 as ::core::ffi::c_int | (*E).rfpm_var_C as uint32,
    );
    if (*E).rfpm_var_0A != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0xb5c) as *mut uint16) =
            ((*E).base.y_pos as ::core::ffi::c_int - (*E).rfpm_var_0E as ::core::ffi::c_int)
                as uint16;
    }
    if ((*E).base.y_pos as ::core::ffi::c_int - (*E).rfpm_var_10 as ::core::ffi::c_int) as int16
        as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        if (*E).rfpm_var_09 as ::core::ffi::c_int == 4080 as ::core::ffi::c_int {
            (*E).rfpm_var_A =
                (0xa2f099 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        } else {
            (*E).rfpm_var_B = (*E).rfpm_var_09;
            (*E).rfpm_var_A =
                (0xa2f072 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
    }
}
pub unsafe fn RisingFallingPlatform_Func_11(game: &mut Game) {
    let mut E: *mut Enemy_RisingFallingPlatform = Get_RisingFallingPlatform(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).rfpm_var_B = (*E).rfpm_var_B.wrapping_sub(1);
    if (*E).rfpm_var_B as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).rfpm_var_A = (0xa2efd4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        if (*E).rfpm_var_04 as ::core::ffi::c_int == 1 as ::core::ffi::c_int
            && (*E).rfpm_var_01 as ::core::ffi::c_int != 0
        {
            (*E).rfpm_var_A =
                (0xa2ef28 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
        if (*Get_RisingFallingPlatform(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        ))
        .base
        .enemy_ptr as ::core::ffi::c_int
            != 0xd83f as ::core::ffi::c_int
        {
            RisingFallingPlatform_Func_8(game);
        }
    }
}
pub unsafe fn RisingFallingPlatform_Func_12(game: &mut Game) {
    let mut E: *mut Enemy_RisingFallingPlatform = Get_RisingFallingPlatform(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).rfpm_var_B = (*E).rfpm_var_B.wrapping_sub(1);
    if (*E).rfpm_var_B as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        RisingFallingPlatform_Func_8(game);
        (*E).rfpm_var_A = (0xa2ef68 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        if (*E).rfpm_var_04 as ::core::ffi::c_int == 1 as ::core::ffi::c_int
            && (*E).rfpm_var_01 == 0
        {
            (*E).rfpm_var_A =
                (0xa2ef28 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
    }
}
pub unsafe fn RisingFallingPlatform_Func_13() {}
pub unsafe fn RisingFallingPlatform_Touch(game: &mut Game) {
    RisingFallingPlatform_Powerbomb(game);
}
pub unsafe fn RisingFallingPlatform_Shot(game: &mut Game) {
    RisingFallingPlatform_Powerbomb(game);
}
pub unsafe fn ShootableShutter_Shot(game: &mut Game) {
    NormalEnemyShotAi(game);
    RisingFallingPlatform_Powerbomb(game);
}
pub unsafe fn RisingFallingPlatform_Powerbomb(game: &mut Game) {
    let mut E: *mut Enemy_RisingFallingPlatform = Get_RisingFallingPlatform(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if !((*E).rfpm_var_07 as ::core::ffi::c_int - 6 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0)
    {
        if (*E).rfpm_var_07 as ::core::ffi::c_int != 8 as ::core::ffi::c_int {
            if (*E).rfpm_var_0C != 0 {
                return;
            }
            (*E).rfpm_var_0C = 1 as uint16;
        }
        if (*E).rfpm_var_A as ::core::ffi::c_int
            != 0xa2ef68 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
            && (*E).rfpm_var_A as ::core::ffi::c_int
                != 0xa2efd4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
        {
            (*E).rfpm_var_A =
                (0xa2ef68 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            if (*E).rfpm_var_20 != 0 {
                (*E).rfpm_var_A =
                    (0xa2efd4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            }
            (*E).rfpm_var_20 =
                ((*E).rfpm_var_20 as ::core::ffi::c_int ^ 1 as ::core::ffi::c_int) as uint16;
        } else {
            return;
        }
    }
    RisingFallingPlatform_Func_8(game);
}
pub unsafe fn HorizontalShootableShutter_Init(game: &mut Game) {
    HorizontalShootableShutter_Func_1(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
    (*Get_HorizontalShootableShutter(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .base
    .current_instruction = addr_kTimedShutter_Ilist_E9D4 as ::core::ffi::c_int as uint16;
}
pub unsafe fn HorizontalShootableShutter_Func_1(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_RisingFallingPlatform =
        Get_RisingFallingPlatform(k as ::core::ffi::c_int, game);
    let mut current_instruction_low: uint16 =
        *(&raw mut (*E).base.current_instruction as *mut uint8).offset(0) as uint16;
    (*E).rfpm_var_00 = current_instruction_low;
    let mut v3: ::core::ffi::c_int = 8 as ::core::ffi::c_int
        * current_instruction_low as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    (*E).rfpm_var_D = *(game.g_rom.offset(
        (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v3 as isize);
    (*E).rfpm_var_C = *(game.g_rom.offset(
        (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((v3 + 1 as ::core::ffi::c_int) as isize);
    (*E).rfpm_var_F = *(game.g_rom.offset(
        (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((v3 + 2 as ::core::ffi::c_int) as isize);
    (*E).rfpm_var_E = *(game.g_rom.offset(
        (((0xa28187 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xa28187 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((v3 + 3 as ::core::ffi::c_int) as isize);
    let mut current_instruction_high: uint16 =
        *(&raw mut (*E).base.current_instruction as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as uint16;
    (*E).rfpm_var_01 = current_instruction_high;
    (*E).rfpm_var_20 =
        (current_instruction_high as ::core::ffi::c_int ^ 1 as ::core::ffi::c_int) as uint16;
    let mut extra_properties_low: uint16 =
        *(&raw mut (*E).base.extra_properties as *mut uint8).offset(0) as uint16;
    (*E).rfpm_var_02 = extra_properties_low;
    (*E).rfpm_var_08 =
        (16 as ::core::ffi::c_int * extra_properties_low as ::core::ffi::c_int) as uint16;
    let mut extra_properties_high: uint16 = *(&raw mut (*E).base.extra_properties as *mut uint8)
        .offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as uint16;
    (*E).rfpm_var_03 = extra_properties_high;
    (*E).rfpm_var_09 =
        (16 as ::core::ffi::c_int * extra_properties_high as ::core::ffi::c_int) as uint16;
    let mut rfpm_parameter_1_low: uint16 =
        *(&raw mut (*E).rfpm_parameter_1 as *mut uint8).offset(0) as uint16;
    (*E).rfpm_var_04 = rfpm_parameter_1_low;
    (*E).rfpm_var_07 =
        (2 as ::core::ffi::c_int * rfpm_parameter_1_low as ::core::ffi::c_int) as uint16;
    (*E).rfpm_var_05 = *(&raw mut (*E).rfpm_parameter_1 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>() as usize)
            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) as uint16;
    let mut rfpm_parameter_2: uint16 = (*E).rfpm_parameter_2;
    (*E).rfpm_var_06 = rfpm_parameter_2;
    (*E).rfpm_var_B = rfpm_parameter_2;
    let mut x_pos: uint16 = (*E).base.x_pos;
    (*E).rfpm_var_11 = x_pos;
    (*E).rfpm_var_12 =
        ((*E).rfpm_var_05 as ::core::ffi::c_int + x_pos as ::core::ffi::c_int) as uint16;
    if (*E).rfpm_var_01 == 0 {
        let mut v10: uint16 = (*E).base.x_pos;
        (*E).rfpm_var_12 = v10;
        (*E).rfpm_var_11 =
            (v10 as ::core::ffi::c_int - (*E).rfpm_var_05 as ::core::ffi::c_int) as uint16;
    }
    (*E).rfpm_var_A = (0xa2f224 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    (*E).rfpm_var_15 = *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16);
    (*E).base.extra_properties = 0 as uint16;
    (*E).rfpm_var_0A = 0 as uint16;
    (*E).rfpm_var_0B = 0 as uint16;
}
pub unsafe fn HorizontalShootableShutter_Main(game: &mut Game) {
    let mut E: *mut Enemy_RisingFallingPlatform = Get_RisingFallingPlatform(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallRisingFallingPlatformFunc(
        ((*E).rfpm_var_A as ::core::ffi::c_int | 0xa20000 as ::core::ffi::c_int) as uint32,
        game,
    );
    if (*E).rfpm_var_A as ::core::ffi::c_int
        != 0xa2f272 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
        && (*E).rfpm_var_A as ::core::ffi::c_int
            != 0xa2f2e4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
        && *((&raw mut game.g_ram as *mut uint8).offset(0x182c) as *mut uint16).offset(3)
            as ::core::ffi::c_int
            & (*((&raw mut game.g_ram as *mut uint8).offset(0x182c) as *mut uint16).offset(2)
                as ::core::ffi::c_int
                & *((&raw mut game.g_ram as *mut uint8).offset(0x182c) as *mut uint16).offset(1)
                    as ::core::ffi::c_int
                & *((&raw mut game.g_ram as *mut uint8).offset(0x182c) as *mut uint16).offset(0)
                    as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
            != 0xffff as ::core::ffi::c_int
        && *((&raw mut game.g_ram as *mut uint8).offset(0x182c) as *mut uint16).offset(3)
            as ::core::ffi::c_int
            & (*((&raw mut game.g_ram as *mut uint8).offset(0x182c) as *mut uint16).offset(2)
                as ::core::ffi::c_int
                & *((&raw mut game.g_ram as *mut uint8).offset(0x182c) as *mut uint16).offset(1)
                    as ::core::ffi::c_int
                & *((&raw mut game.g_ram as *mut uint8).offset(0x182c) as *mut uint16).offset(0)
                    as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
            == *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int
        && *((&raw mut game.g_ram as *mut uint8).offset(0xa6e) as *mut uint16) as ::core::ffi::c_int
            != 0
    {
        HorizontalShootableShutter_Powerbomb(game);
    }
    (*E).rfpm_var_15 = *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16);
    (*E).rfpm_var_16 = *((&raw mut game.g_ram as *mut uint8).offset(0xaf8) as *mut uint16);
}
static off_A2F107: [Option<Func_V>; 5] = unsafe {
    [
        Some(HorizontalShootableShutter_Func_3 as unsafe fn(&mut Game) -> ()),
        Some(HorizontalShootableShutter_Func_4 as unsafe fn(&mut Game) -> ()),
        Some(HorizontalShootableShutter_Func_5 as unsafe fn(&mut Game) -> ()),
        Some(HorizontalShootableShutter_Func_6 as unsafe fn(&mut Game) -> ()),
        Some(HorizontalShootableShutter_Func_6 as unsafe fn(&mut Game) -> ()),
    ]
};
pub unsafe fn HorizontalShootableShutter_Func_2(game: &mut Game) {
    let mut v0: ::core::ffi::c_int = (*Get_RisingFallingPlatform(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .rfpm_var_07 as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    off_A2F107[v0 as usize].expect("non-null function pointer")(game);
}
pub unsafe fn HorizontalShootableShutter_Func_3(game: &mut Game) {
    let mut E: *mut Enemy_RisingFallingPlatform = Get_RisingFallingPlatform(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let fresh14 = (*E).rfpm_var_B;
    (*E).rfpm_var_B = (*E).rfpm_var_B.wrapping_sub(1);
    if fresh14 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        (*E).rfpm_var_B = (*E).rfpm_var_06;
        HorizontalShootableShutter_Func_7(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        );
    }
}
pub unsafe fn HorizontalShootableShutter_Func_4(game: &mut Game) {
    let mut E: *mut Enemy_RisingFallingPlatform = Get_RisingFallingPlatform(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if IsSamusWithinEnemy_X(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        (*E).rfpm_var_06,
        game,
    ) != 0
    {
        HorizontalShootableShutter_Func_7(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        );
    }
}
pub unsafe fn HorizontalShootableShutter_Func_5(game: &mut Game) {
    HorizontalShootableShutter_Func_7(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
}
pub unsafe fn HorizontalShootableShutter_Func_6(game: &mut Game) {}
pub unsafe fn HorizontalShootableShutter_Func_7(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_RisingFallingPlatform =
        Get_RisingFallingPlatform(k as ::core::ffi::c_int, game);
    (*E).rfpm_var_A = (0xa2f272 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    if (*E).rfpm_var_01 != 0 {
        (*E).rfpm_var_A = (0xa2f2e4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn HorizontalShootableShutter_Func_8(game: &mut Game) {
    let mut E: *mut Enemy_RisingFallingPlatform = Get_RisingFallingPlatform(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).rfpm_var_0D = (*E).base.x_pos;
    (*E).rfpm_var_13 = 0 as uint16;
    if EnemyFunc_AC67(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    ) as ::core::ffi::c_int
        != 0
        && ((*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
            as ::core::ffi::c_int
            - (*E).base.x_pos as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
    {
        (*E).rfpm_var_13 = 1 as uint16;
    }
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        ((*E).rfpm_var_F as uint32) << 16 as ::core::ffi::c_int | (*E).rfpm_var_E as uint32,
    );
    if (*E).rfpm_var_13 != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0xb56) as *mut uint16) = (*E).rfpm_var_E;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16) = (*E).rfpm_var_F;
        HorizontalShootableShutter_Func_10(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        );
    }
    if ((*E).rfpm_var_11 as ::core::ffi::c_int - (*E).base.x_pos as ::core::ffi::c_int) as int16
        as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        if (*E).rfpm_var_08 as ::core::ffi::c_int == 4080 as ::core::ffi::c_int {
            (*E).rfpm_var_A =
                (0xa2f3d4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        } else {
            (*E).rfpm_var_B = (*E).rfpm_var_08;
            (*E).rfpm_var_A =
                (0xa2f38c as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
    }
}
pub unsafe fn HorizontalShootableShutter_Func_9(game: &mut Game) {
    let mut E: *mut Enemy_RisingFallingPlatform = Get_RisingFallingPlatform(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).rfpm_var_0D = (*E).base.x_pos;
    (*E).rfpm_var_13 = 0 as uint16;
    if EnemyFunc_AC67(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    ) as ::core::ffi::c_int
        != 0
        && (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
            as ::core::ffi::c_int
            - (*E).base.x_pos as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
    {
        (*E).rfpm_var_13 = 1 as uint16;
    }
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        ((*E).rfpm_var_D as uint32) << 16 as ::core::ffi::c_int | (*E).rfpm_var_C as uint32,
    );
    if (*E).rfpm_var_13 != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0xb56) as *mut uint16) = (*E).rfpm_var_C;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16) = (*E).rfpm_var_D;
        HorizontalShootableShutter_Func_11(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        );
    }
    if ((*E).base.x_pos as ::core::ffi::c_int - (*E).rfpm_var_12 as ::core::ffi::c_int) as int16
        as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        if (*E).rfpm_var_09 as ::core::ffi::c_int == 4080 as ::core::ffi::c_int {
            (*E).rfpm_var_A =
                (0xa2f3d4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        } else {
            (*E).rfpm_var_B = (*E).rfpm_var_09;
            (*E).rfpm_var_A =
                (0xa2f3b0 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
    }
}
pub unsafe fn HorizontalShootableShutter_Func_10(mut k: uint16, game: &mut Game) {
    if (*Get_RisingFallingPlatform(k as ::core::ffi::c_int, game)).rfpm_var_13 != 0 {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x8b) as *mut uint16) as ::core::ffi::c_int
            & 0x100 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            let ref mut fresh12 =
                *((&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16);
            *fresh12 = (*fresh12 as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xb5a) as *mut uint16) = 0 as uint16;
        }
    }
}
pub unsafe fn HorizontalShootableShutter_Func_11(mut k: uint16, game: &mut Game) {
    if (*Get_RisingFallingPlatform(k as ::core::ffi::c_int, game)).rfpm_var_13 != 0 {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x8b) as *mut uint16) as ::core::ffi::c_int
            & 0x200 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            let ref mut fresh13 =
                *((&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16);
            *fresh13 = (*fresh13 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xb5a) as *mut uint16) = 0 as uint16;
        }
    }
}
pub unsafe fn HorizontalShootableShutter_Func_12(game: &mut Game) {
    let mut E: *mut Enemy_RisingFallingPlatform = Get_RisingFallingPlatform(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).rfpm_var_B = (*E).rfpm_var_B.wrapping_sub(1);
    if (*E).rfpm_var_B as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).rfpm_var_A = (0xa2f2e4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        if (*E).rfpm_var_04 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
            if (*E).rfpm_var_01 != 0 {
                (*E).rfpm_var_A =
                    (0xa2f243 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            }
        }
    }
}
pub unsafe fn HorizontalShootableShutter_Func_13(game: &mut Game) {
    let mut E: *mut Enemy_RisingFallingPlatform = Get_RisingFallingPlatform(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).rfpm_var_B = (*E).rfpm_var_B.wrapping_sub(1);
    if (*E).rfpm_var_B as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        (*E).rfpm_var_A = (0xa2f272 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        if (*E).rfpm_var_04 as ::core::ffi::c_int == 1 as ::core::ffi::c_int
            && (*E).rfpm_var_01 == 0
        {
            (*E).rfpm_var_A =
                (0xa2f243 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
    }
}
pub unsafe fn HorizontalShootableShutter_Func_14() {}
pub unsafe fn HorizontalShootableShutter_Touch(game: &mut Game) {
    let mut E: *mut Enemy_HorizontalShootableShutter = Get_HorizontalShootableShutter(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).hssr_var_A as ::core::ffi::c_int
        == 0xa2f3d4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        if (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
            as ::core::ffi::c_int
            - (*E).base.x_pos as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        {
            if *((&raw mut game.g_ram as *mut uint8).offset(0x8b) as *mut uint16)
                as ::core::ffi::c_int
                & 0x200 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16) = 4 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0xb5a) as *mut uint16) = 0 as uint16;
            }
        } else if *((&raw mut game.g_ram as *mut uint8).offset(0x8b) as *mut uint16)
            as ::core::ffi::c_int
            & 0x100 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16) =
                -(4 as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xb5a) as *mut uint16) = 0 as uint16;
        }
    }
}
pub unsafe fn HorizontalShootableShutter_Shot(game: &mut Game) {
    NormalEnemyShotAiSkipDeathAnim_CurEnemy(game);
    HorizontalShootableShutter_Powerbomb(game);
}
pub unsafe fn HorizontalShootableShutter_Powerbomb(game: &mut Game) {
    let mut E: *mut Enemy_RisingFallingPlatform = Get_RisingFallingPlatform(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).rfpm_var_07 as ::core::ffi::c_int - 6 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        if (*E).rfpm_var_07 as ::core::ffi::c_int != 8 as ::core::ffi::c_int {
            if (*E).rfpm_var_0C != 0 {
                return;
            }
            (*E).rfpm_var_0C = 1 as uint16;
        }
        if (*E).rfpm_var_A as ::core::ffi::c_int
            != 0xa2f272 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
            && (*E).rfpm_var_A as ::core::ffi::c_int
                != 0xa2f2e4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
        {
            (*E).rfpm_var_20 =
                ((*E).rfpm_var_20 as ::core::ffi::c_int ^ 1 as ::core::ffi::c_int) as uint16;
            (*E).rfpm_var_A =
                (0xa2f272 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            if (*E).rfpm_var_20 != 0 {
                (*E).rfpm_var_A =
                    (0xa2f2e4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            }
        }
    }
}
