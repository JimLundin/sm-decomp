use crate::game::sm_80::{
    CheckEventHappened, DrawTimer, HandleAutoscrolling_X, HandleAutoscrolling_Y,
    HandleScrollingWhenTriggeringScrollDown, HandleScrollingWhenTriggeringScrollLeft,
    HandleScrollingWhenTriggeringScrollRight, HandleScrollingWhenTriggeringScrollUp,
    InvalidInterrupt_Crash, NextRandom, ProcessTimer, QueueSfx1_Max15, QueueSfx1_Max6,
    QueueSfx1_Max9, QueueSfx2_Max6, QueueSfx3_Max15, QueueSfx3_Max3, QueueSfx3_Max6,
    QueueSfx3_Max9,
};
use crate::game::sm_81::{DrawBeamGrappleSpritemap, DrawSamusSpritemap};
use crate::game::sm_82::PlayRoomMusicTrackAfterAFrames;
use crate::game::sm_86::{DisableEprojs, EnableEprojs, SpawnEprojWithGfx};
use crate::game::sm_88::{EnableHdmaObjects, SpawnCrystalFlashHdmaObjs, SpawnPowerBombExplosion};
use crate::game::sm_8b::Samus_CalcPos_Mode7;
use crate::game::sm_8d::{DisablePaletteFx, SpawnPalfxObject};
use crate::game::sm_91::{
    DemoObjectInputHandler, MakeSamusFaceForward, SamusFunc_E633, SamusFunc_F433,
    Samus_CancelSpeedBoost, Samus_Func20, Samus_HandlePalette, Samus_HandleTransFromBlockColl,
    Samus_HandleTransitions, Samus_InputHandler, Samus_LoadSuitPalette,
    Samus_LoadSuitTargetPalette, Samus_RestoreHealth, Samus_SetAnimationFrameIfPoseChanged,
    XrayRunHandler,
};
use crate::game::sm_92::{PlaySamusFanfare, SetSamusTilesDefsForCurAnim};
use crate::game::sm_93::{
    DrawPlayerExplosions2, InitializeBombExplosion, InitializeInstrForMissile,
    InitializeInstrForSuperMissile, InitializeProjectile, InitializeSbaProjectile,
    InitializeShinesparkEchoOrSpazerSba, KillProjectileInner, RunProjectileInstructions,
};
use crate::game::sm_94::{
    kSinCosTable8bit_Sext, BlockCollMissileHoriz, BlockCollMissileVert, BlockCollNoWaveBeamHoriz,
    BlockCollNoWaveBeamVert, BlockCollSpreadBomb, BlockCollWaveBeamHoriz, BlockCollWaveBeamVert,
    BlockInsideDetection, BombOrPowerBomb_Func1, HandleGrappleBeamGfx,
    PostGrappleCollisionDetect_X, PostGrappleCollisionDetect_Y, Samus_AlignYPosSlope,
    Samus_MoveDown_NoSolidColl, Samus_MoveRight_NoSolidColl, WallJumpBlockCollDetect,
};
use crate::game::sm_9b::{
    GrappleBeamFunc_BF1B, GrappleBeamHandler, HandleGrappleBeamFlare, ProjectileTrail_Func5,
    UpdateGrappleBeamTilesAndIncrFlameCtr,
};
use crate::game::sm_a0::Samus_CheckSolidEnemyColl;
use crate::rtl::{g_ram, g_rom, Load24, MakePairU16, Mult8x8, RomPtr, Unreachable};
extern "C" {
    fn printf(__format: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;

}
pub use crate::types::{
    __int16_t, __int32_t, __int8_t, __uint16_t, __uint32_t, __uint8_t, int16, int16_t, int32,
    int32_t, int8, int8_t, uint16, uint16_t, uint32, uint32_t, uint8, uint8_t, FuncXY_V, Func_U8,
    Func_V, Func_V_A, Func_Y_V, HandlerFunc, LongPtr, PairU16, Point16U, VoidP,
};
pub type Func_Y_To_PairU16 = unsafe fn(uint16) -> PairU16;
pub use crate::types::{OamEnt, SamusPose, VramWriteEntry};
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
pub use crate::types::LiquidPhysicsType;
pub const kLiquidPhysicsType_LavaAcid: LiquidPhysicsType = 2;
pub const kLiquidPhysicsType_Water: LiquidPhysicsType = 1;
pub const kLiquidPhysicsType_Air: LiquidPhysicsType = 0;
pub use crate::types::SamusPoseParams;
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct SamusSpeedTableEntry {
    pub accel: uint16,
    pub accel_sub: uint16,
    pub max_speed: uint16,
    pub max_speed_sub: uint16,
    pub decel: uint16,
    pub decel_sub: uint16,
}
pub use crate::types::SamusMovementType;
pub const kMovementType_1B_ShinesparkEtc: SamusMovementType = 27;
pub const kMovementType_1A_GrabbedByDraygon: SamusMovementType = 26;
pub const kMovementType_19_DamageBoost: SamusMovementType = 25;
pub const kMovementType_18_TurningAroundFalling: SamusMovementType = 24;
pub const kMovementType_17_TurningAroundJumping: SamusMovementType = 23;
pub const kMovementType_16_Grappling: SamusMovementType = 22;
pub const kMovementType_15_RanIntoWall: SamusMovementType = 21;
pub const kMovementType_14_WallJumping: SamusMovementType = 20;
pub const kMovementType_13_SpringBallFalling: SamusMovementType = 19;
pub const kMovementType_12_SpringBallInAir: SamusMovementType = 18;
pub const kMovementType_11_SpringBallOnGround: SamusMovementType = 17;
pub const kMovementType_10_Moonwalking: SamusMovementType = 16;
pub const kMovementType_0F_CrouchingEtcTransition: SamusMovementType = 15;
pub const kMovementType_0E_TurningAroundOnGround: SamusMovementType = 14;
pub const kMovementType_0D_Unused: SamusMovementType = 13;
pub const kMovementType_0C_Unused: SamusMovementType = 12;
pub const kMovementType_0B_Unused: SamusMovementType = 11;
pub const kMovementType_0A_KnockbackOrCrystalFlashEnding: SamusMovementType = 10;
pub const kMovementType_09_Unused: SamusMovementType = 9;
pub const kMovementType_08_MorphBallFalling: SamusMovementType = 8;
pub const kMovementType_07_Unused: SamusMovementType = 7;
pub const kMovementType_06_Falling: SamusMovementType = 6;
pub const kMovementType_05_Crouching: SamusMovementType = 5;
pub const kMovementType_04_MorphBallOnGround: SamusMovementType = 4;
pub const kMovementType_03_SpinJumping: SamusMovementType = 3;
pub const kMovementType_02_NormalJumping: SamusMovementType = 2;
pub const kMovementType_01_Running: SamusMovementType = 1;
pub const kMovementType_00_Standing: SamusMovementType = 0;
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct DisableMinimapAndMarkBossRoomAsExploredEnt {
    pub boss_id_: uint16,
    pub ptrs: VoidP,
}
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
pub use crate::types::Consts_60;
pub const addr_unk_606300: Consts_60 = 25344;
pub const addr_unk_6061F0: Consts_60 = 25072;
pub const addr_unk_605BE0: Consts_60 = 23520;
pub const addr_unk_6059A0: Consts_60 = 22944;
pub const addr_unk_605880: Consts_60 = 22656;
pub const addr_unk_605820: Consts_60 = 22560;
pub const addr_unk_605800: Consts_60 = 22528;
pub const addr_unk_605400: Consts_60 = 21504;
pub const addr_unk_6053E0: Consts_60 = 21472;
pub const addr_unk_605000: Consts_60 = 20480;
pub const addr_unk_604C00: Consts_60 = 19456;
pub const addr_unk_604800: Consts_60 = 18432;
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
pub use crate::types::Consts_90;
pub const addr_loc_90C4F0: Consts_90 = 50416;
pub const addr_kSuperMissileAccelerations2: Consts_90 = 49963;
pub const addr_kSuperMissileAccelerations: Consts_90 = 49923;
pub const addr_kSamusSpeedTable_LavaAcid_X: Consts_90 = 41437;
pub const addr_kSamusSpeedTable_Water_X: Consts_90 = 41101;
pub const addr_kSamusSpeedTable_Normal_X: Consts_90 = 40777;
pub const addr_stru_909F3D: Consts_90 = 40765;
pub const addr_stru_909F31: Consts_90 = 40753;
pub const addr_stru_909F25: Consts_90 = 40741;
pub const addr_loc_908029: Consts_90 = 32809;
pub use crate::types::{
    CheckEnemyColl_Result, CheckSolidEnemyColl_Result, Func_AnimDelay, Func_SpazerSba_V,
    Pair_Bool_Amt,
};
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
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
unsafe fn RomPtr_82(mut addr: uint16_t) -> *const uint8 {
    return RomPtr((0x820000 as ::core::ffi::c_int | addr as ::core::ffi::c_int) as uint32_t);
}
#[inline]
unsafe fn RomPtr_90(mut addr: uint16_t) -> *const uint8 {
    return RomPtr((0x900000 as ::core::ffi::c_int | addr as ::core::ffi::c_int) as uint32_t);
}
#[inline]
unsafe fn RomPtr_91(mut addr: uint16_t) -> *const uint8 {
    return RomPtr((0x910000 as ::core::ffi::c_int | addr as ::core::ffi::c_int) as uint32_t);
}
#[inline]
unsafe fn gOamEnt(mut v: ::core::ffi::c_int) -> *mut OamEnt {
    return (&raw mut g_ram as *mut uint8).offset((0x370 as ::core::ffi::c_int + v) as isize)
        as *mut uint8 as *mut OamEnt;
}
#[inline]
unsafe fn gVramWriteEntry(mut a: ::core::ffi::c_int) -> *mut VramWriteEntry {
    return (&raw mut g_ram as *mut uint8).offset((0xd0 as ::core::ffi::c_int + a) as isize)
        as *mut uint8 as *mut VramWriteEntry;
}
#[inline]
unsafe fn get_SamusSpeedTableEntry(mut a: uint16) -> *mut SamusSpeedTableEntry {
    return RomPtr((0x900000 as ::core::ffi::c_int | a as ::core::ffi::c_int) as uint32_t)
        as *mut SamusSpeedTableEntry;
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
pub const fnSamus_MoveHandler_ReleaseFromGrapple: uint32 = 9475182 as uint32;
pub const fnSamus_HandleMovement_DrainedCrouching: ::core::ffi::c_int =
    0x9094cb as ::core::ffi::c_int;
pub const fnSamus_ScrollFinishedHook_SporeSpawnFight: uint32 = 9475465 as uint32;
pub const fnSamus_MovementHandler_Normal: ::core::ffi::c_int = 0x90a337 as ::core::ffi::c_int;
pub const fnProjPreInstr_Beam_NoWaveBeam: ::core::ffi::c_int = 0x90aef3 as ::core::ffi::c_int;
pub const fnProjPreInstr_Dir0459: uint32 = 9482058 as uint32;
pub const fnProjPreInstr_Dir1368: uint32 = 9482066 as uint32;
pub const fnProjPreInstr_Dir27: uint32 = 9482080 as uint32;
pub const fnProjPreInstr_Missile: ::core::ffi::c_int = 0x90af68 as ::core::ffi::c_int;
pub const fnProjPreInstr_Missile_Func0459: uint32 = 9482183 as uint32;
pub const fnProjPreInstr_Missile_Func1368: uint32 = 9482191 as uint32;
pub const fnProjPreInstr_Missile_Func27: uint32 = 9482205 as uint32;
pub const fnProjPreInstr_SuperMissile: ::core::ffi::c_int = 0x90afe5 as ::core::ffi::c_int;
pub const fnProjPreInstr_SuperMissile_Func0459: uint32 = 9482311 as uint32;
pub const fnProjPreInstr_SuperMissile_Func1368: uint32 = 9482322 as uint32;
pub const fnProjPreInstr_SuperMissile_Func27: uint32 = 9482346 as uint32;
pub const fnProjPreInstr_Func1: ::core::ffi::c_int = 0x90b075 as ::core::ffi::c_int;
pub const fnProjPreInstr_Bomb: ::core::ffi::c_int = 0x90b099 as ::core::ffi::c_int;
pub const fnProjPreInstr_PowerBomb: ::core::ffi::c_int = 0x90b0ae as ::core::ffi::c_int;
pub const fnProjPreInstr_WavePlasmaEtc: ::core::ffi::c_int = 0x90b0c3 as ::core::ffi::c_int;
pub const fnProjPreInstr_BeamOrIceWave: ::core::ffi::c_int = 0x90b0e4 as ::core::ffi::c_int;
pub const fnProjPreInstr_Wave_Shared: uint32 = 9482499 as uint32;
pub const fnProjPreInstr_WavePlasmaEtc_0459: uint32 = 9482555 as uint32;
pub const fnProjPreInstr_WavePlasmaEtc_1368: uint32 = 9482563 as uint32;
pub const fnProjPreInstr_WavePlasmaEtc_27: uint32 = 9482577 as uint32;
pub const fnProjPreInstr_HyperBeam: ::core::ffi::c_int = 0x90b159 as ::core::ffi::c_int;
pub const fnProjPreInstr_Empty: ::core::ffi::c_int = 0x90b169 as ::core::ffi::c_int;
pub const fnProjInstr_MoveLeftProjectileTrailDown: uint32 = 9483557 as uint32;
pub const fnProjInstr_MoveRightProjectileTrailDown: uint32 = 9483655 as uint32;
pub const fnProjInstr_MoveLeftProjectileTrailUp: uint32 = 9483699 as uint32;
pub const fnProjPreInstr_IceSba: ::core::ffi::c_int = 0x90cf09 as ::core::ffi::c_int;
pub const fnProjPreInstr_IceSba2: ::core::ffi::c_int = 0x90cf7a as ::core::ffi::c_int;
pub const fnSamus_MoveHandlerShinesparkWindup: ::core::ffi::c_int = 0x90d068 as ::core::ffi::c_int;
pub const fnSamus_MoveHandlerVerticalShinespark: ::core::ffi::c_int =
    0x90d0ab as ::core::ffi::c_int;
pub const fnSamus_MoveHandler_Shinespark_Diag: uint32 = 9490647 as uint32;
pub const fnSamus_MoveHandler_Shinespark_Horiz: uint32 = 9490694 as uint32;
pub const fnSamus_MoveHandler_ShinesparkCrash: ::core::ffi::c_int = 0x90d346 as ::core::ffi::c_int;
pub const fnSamus_MoveHandler_ShinesparkCrash_0: uint32 = 9491331 as uint32;
pub const fnSamus_MoveHandler_ShinesparkCrash_1: uint32 = 9491350 as uint32;
pub const fnSamus_MoveHandler_ShinesparkCrash_2: uint32 = 9491404 as uint32;
pub const fnSamus_MoveHandler_ShinesparkCrashEchoCircle: ::core::ffi::c_int =
    0x90d3f3 as ::core::ffi::c_int;
pub const fnSamus_MoveHandler_ShinesparkCrashFinish: ::core::ffi::c_int =
    0x90d40d as ::core::ffi::c_int;
pub const fnProjPreInstr_SpeedEcho: ::core::ffi::c_int = 0x90d4d2 as ::core::ffi::c_int;
pub const fnSamusMoveHandler_CrystalFlashStart: ::core::ffi::c_int = 0x90d678 as ::core::ffi::c_int;
pub const fnSamusMoveHandler_CrystalFlashMain: ::core::ffi::c_int = 0x90d6ce as ::core::ffi::c_int;
pub const fnkSamusMoveHandler_CrystalFlashFinish: ::core::ffi::c_int =
    0x90d75b as ::core::ffi::c_int;
pub const fnProjPreInstr_PlasmaSba: ::core::ffi::c_int = 0x90d793 as ::core::ffi::c_int;
pub const fnProjPreInstr_PlasmaSbaFunc_0: uint32 = 9492449 as uint32;
pub const fnProjPreInstr_PlasmaSbaFunc_1: uint32 = 9492474 as uint32;
pub const fnProjPreInstr_PlasmaSbaFunc_2: uint32 = 9492499 as uint32;
pub const fnProjPreInstr_SpreadBomb: ::core::ffi::c_int = 0x90d8f7 as ::core::ffi::c_int;
pub const fnProjPreInstr_WaveSba: ::core::ffi::c_int = 0x90da08 as ::core::ffi::c_int;
pub const fnProjPreInstr_SpazerSba: ::core::ffi::c_int = 0x90db06 as ::core::ffi::c_int;
pub const fnProjPreInstr_EndOfSpazerSba: ::core::ffi::c_int = 0x90dc9c as ::core::ffi::c_int;
pub const fnSamus_MoveHandler_Knockback: uint32 = 9494328 as uint32;
pub const fnSamus_MoveHandler_Knockback_0: uint32 = 9494352 as uint32;
pub const fnSamus_MoveHandler_Knockback_Up: uint32 = 9494355 as uint32;
pub const fnSamus_MoveHandler_Knockback_3: uint32 = 9494365 as uint32;
pub const fnSamus_MoveHandler_Knockback_Down: uint32 = 9494372 as uint32;
pub const fnSamus_MoveHandler_BombJumpStart: uint32 = 9494565 as uint32;
pub const fnSamus_MoveHandler_BombJumpMain: ::core::ffi::c_int = 0x90e032 as ::core::ffi::c_int;
pub const fnSamus_MoveHandler_BombJumpFunc1: uint32 = 9494653 as uint32;
pub const fnSamus_Func1: ::core::ffi::c_int = 0x90e09b as ::core::ffi::c_int;
pub const fnSamus_Func2: ::core::ffi::c_int = 0x90e0c5 as ::core::ffi::c_int;
pub const fnSamus_Func3: ::core::ffi::c_int = 0x90e0e6 as ::core::ffi::c_int;
pub const fnDrawTimer_: ::core::ffi::c_int = 0x90e114 as ::core::ffi::c_int;
pub const fnSamus_PushOutOfRidleysWay: ::core::ffi::c_int = 0x90e12e as ::core::ffi::c_int;
pub const fnSamus_Func4: ::core::ffi::c_int = 0x90e1c8 as ::core::ffi::c_int;
pub const fnSamus_GrabbedByDraygonFrameHandler: ::core::ffi::c_int = 0x90e2a1 as ::core::ffi::c_int;
pub const fnnullsub_151: uint32 = 9495422 as uint32;
pub const fnSamus_Func7: uint32 = 9495459 as uint32;
pub const fnSamus_Func9: ::core::ffi::c_int = 0x90e41b as ::core::ffi::c_int;
pub const fnSamus_FrameHandlerAlfa_Func11: ::core::ffi::c_int = 0x90e695 as ::core::ffi::c_int;
pub const fnSamus_FrameHandlerAlfa_Func12: uint32 = 9496265 as uint32;
pub const fnSamus_FrameHandlerAlfa_Func13: ::core::ffi::c_int = 0x90e713 as ::core::ffi::c_int;
pub const fnSamus_FrameHandlerBeta_Func17: ::core::ffi::c_int = 0x90e725 as ::core::ffi::c_int;
pub const fnHandleDemoRecorder_3: ::core::ffi::c_int = 0x90e7d2 as ::core::ffi::c_int;
pub const fnSamus_FrameHandlerBeta_Func14: uint32 = 9496565 as uint32;
pub const fnSamus_Func15: uint32 = 9496627 as uint32;
pub const fnSamus_Func16: uint32 = 9496682 as uint32;
pub const fnSamus_Func18: ::core::ffi::c_int = 0x90e8aa as ::core::ffi::c_int;
pub const fnEmptyFunction: ::core::ffi::c_int = 0x90e8cd as ::core::ffi::c_int;
pub const fnj_HandleDemoRecorder_2: ::core::ffi::c_int = 0x90e8d6 as ::core::ffi::c_int;
pub const fnj_HandleDemoRecorder_2_0: ::core::ffi::c_int = 0x90e8d9 as ::core::ffi::c_int;
pub const fnSetContactDamageIndexAndUpdateMinimap: ::core::ffi::c_int =
    0x90e8dc as ::core::ffi::c_int;
pub const fnSamus_Func19: ::core::ffi::c_int = 0x90e8ec as ::core::ffi::c_int;
pub const fnSamus_LowHealthCheck: ::core::ffi::c_int = 0x90e902 as ::core::ffi::c_int;
pub const fnnullsub_152: ::core::ffi::c_int = 0x90e90e as ::core::ffi::c_int;
pub const fnSamus_InputHandler_E913: ::core::ffi::c_int = 0x90e913 as ::core::ffi::c_int;
pub const fnSamus_Func20_: uint32 = 9496856 as uint32;
pub const fnSamus_InputHandler_E91D: ::core::ffi::c_int = 0x90e91d as ::core::ffi::c_int;
pub const fnHandleAutoJumpHack: ::core::ffi::c_int = 0x90e926 as ::core::ffi::c_int;
pub const fnSamusMovementType_Xray: uint32 = 9496911 as uint32;
pub const fnSamusDrawHandler_Default: ::core::ffi::c_int = 0x90eb52 as ::core::ffi::c_int;
pub const fnsub_90EB86: uint32 = 9497478 as uint32;
pub const fnnullsub_156: uint32 = 9497586 as uint32;
pub const fnSamus_DrawHandler_EndOfShinespark: ::core::ffi::c_int = 0x90ebf3 as ::core::ffi::c_int;
pub const fnSamusDisplayHandler_UsingElevator: ::core::ffi::c_int = 0x90ec14 as ::core::ffi::c_int;
pub const fnSamusDisplayHandler_SamusReceivedFatal: ::core::ffi::c_int =
    0x90ec1d as ::core::ffi::c_int;
pub const fnProjPreInstr_UnknownProj8027: uint32 = 9498579 as uint32;
pub const fnSamus_Func25_ShineSpark: ::core::ffi::c_int = 0x90f04b as ::core::ffi::c_int;
pub const fnSamus_MoveHandler_F072: ::core::ffi::c_int = 0x90f072 as ::core::ffi::c_int;
pub const fnGrappleBeamFunc_Inactive: ::core::ffi::c_int = 0x9bc4f0 as ::core::ffi::c_int;
pub const fnGrappleBeamFunc_Firing: ::core::ffi::c_int = 0x9bc703 as ::core::ffi::c_int;
pub const fnUNUSED_sub_9BC759: ::core::ffi::c_int = 0x9bc759 as ::core::ffi::c_int;
pub const fnGrappleBeamFunc_ConnectedLockedInPlace: ::core::ffi::c_int =
    0x9bc77e as ::core::ffi::c_int;
pub const fnGrappleBeamFunc_Cancel: ::core::ffi::c_int = 0x9bc856 as ::core::ffi::c_int;
pub const fnGrappleBeam_Func2: ::core::ffi::c_int = 0x9bc8c5 as ::core::ffi::c_int;
static mut kUnchargedProjectile_Sfx: [uint16; 12] = [
    0xb as ::core::ffi::c_int as uint16,
    0xd as ::core::ffi::c_int as uint16,
    0xc as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xf as ::core::ffi::c_int as uint16,
    0x12 as ::core::ffi::c_int as uint16,
    0x10 as ::core::ffi::c_int as uint16,
    0x11 as ::core::ffi::c_int as uint16,
    0x13 as ::core::ffi::c_int as uint16,
    0x16 as ::core::ffi::c_int as uint16,
    0x14 as ::core::ffi::c_int as uint16,
    0x15 as ::core::ffi::c_int as uint16,
];
static mut kChargedProjectile_Sfx: [uint16; 12] = [
    0x17 as ::core::ffi::c_int as uint16,
    0x19 as ::core::ffi::c_int as uint16,
    0x18 as ::core::ffi::c_int as uint16,
    0x1a as ::core::ffi::c_int as uint16,
    0x1b as ::core::ffi::c_int as uint16,
    0x1e as ::core::ffi::c_int as uint16,
    0x1c as ::core::ffi::c_int as uint16,
    0x1d as ::core::ffi::c_int as uint16,
    0x1f as ::core::ffi::c_int as uint16,
    0x22 as ::core::ffi::c_int as uint16,
    0x20 as ::core::ffi::c_int as uint16,
    0x21 as ::core::ffi::c_int as uint16,
];
static mut kNonBeamProjectile_Sfx: [uint16; 9] = [
    0 as ::core::ffi::c_int as uint16,
    3 as ::core::ffi::c_int as uint16,
    4 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
];
static mut kProjectileCooldown_Uncharged: [uint8; 38] = [
    15 as ::core::ffi::c_int as uint8,
    15 as ::core::ffi::c_int as uint8,
    15 as ::core::ffi::c_int as uint8,
    15 as ::core::ffi::c_int as uint8,
    15 as ::core::ffi::c_int as uint8,
    15 as ::core::ffi::c_int as uint8,
    15 as ::core::ffi::c_int as uint8,
    15 as ::core::ffi::c_int as uint8,
    15 as ::core::ffi::c_int as uint8,
    15 as ::core::ffi::c_int as uint8,
    12 as ::core::ffi::c_int as uint8,
    15 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    30 as ::core::ffi::c_int as uint8,
    30 as ::core::ffi::c_int as uint8,
    30 as ::core::ffi::c_int as uint8,
    30 as ::core::ffi::c_int as uint8,
    30 as ::core::ffi::c_int as uint8,
    30 as ::core::ffi::c_int as uint8,
    30 as ::core::ffi::c_int as uint8,
    30 as ::core::ffi::c_int as uint8,
    30 as ::core::ffi::c_int as uint8,
    30 as ::core::ffi::c_int as uint8,
    30 as ::core::ffi::c_int as uint8,
    30 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
];
static mut kNonBeamProjectileCooldowns: [uint8; 9] = [
    0 as ::core::ffi::c_int as uint8,
    0xa as ::core::ffi::c_int as uint8,
    0x14 as ::core::ffi::c_int as uint8,
    0x28 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0x10 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
];
static mut kBeamAutoFireCooldowns: [uint8; 12] = [
    0x19 as ::core::ffi::c_int as uint8,
    0x19 as ::core::ffi::c_int as uint8,
    0x19 as ::core::ffi::c_int as uint8,
    0x19 as ::core::ffi::c_int as uint8,
    0x19 as ::core::ffi::c_int as uint8,
    0x19 as ::core::ffi::c_int as uint8,
    0x19 as ::core::ffi::c_int as uint8,
    0x19 as ::core::ffi::c_int as uint8,
    0x19 as ::core::ffi::c_int as uint8,
    0x19 as ::core::ffi::c_int as uint8,
    0x19 as ::core::ffi::c_int as uint8,
    0x19 as ::core::ffi::c_int as uint8,
];
pub unsafe fn nullsub_12() {}
static mut kSamusFxHandlers: [Option<Func_V>; 8] = unsafe {
    [
        Some(Samus_Animate_NoFx as unsafe fn() -> ()),
        Some(Samus_Animate_LavaFx as unsafe fn() -> ()),
        Some(Samus_Animate_AcidFx as unsafe fn() -> ()),
        Some(Samus_Animate_WaterFx as unsafe fn() -> ()),
        Some(nullsub_12 as unsafe fn() -> ()),
        Some(nullsub_12 as unsafe fn() -> ()),
        Some(nullsub_12 as unsafe fn() -> ()),
        Some(nullsub_12 as unsafe fn() -> ()),
    ]
};
pub unsafe fn Samus_Animate() {
    kSamusFxHandlers[((*((&raw mut g_ram as *mut uint8)
        .offset(0x196e as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int)
        >> 1 as ::core::ffi::c_int) as usize]
        .expect("non-null function pointer")();
    if *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        == kPose_4D_FaceR_Jump_NoAim_NoMove_NoGun as ::core::ffi::c_int
        || *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == kPose_4E_FaceL_Jump_NoAim_NoMove_NoGun as ::core::ffi::c_int
    {
        if *((&raw mut g_ram as *mut uint8).offset(0xb36 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != 2 as ::core::ffi::c_int
            && *((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                == 1 as ::core::ffi::c_int
            && *((&raw mut g_ram as *mut uint8).offset(0xa94 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                == 1 as ::core::ffi::c_int
        {
            *((&raw mut g_ram as *mut uint8).offset(0xa94 as ::core::ffi::c_int as isize)
                as *mut uint16) = 4 as uint16;
        }
        let ref mut fresh87 = *((&raw mut g_ram as *mut uint8)
            .offset(0xa94 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh87 = (*fresh87).wrapping_sub(1);
        let mut v1: bool = *fresh87 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int;
        if *((&raw mut g_ram as *mut uint8).offset(0xa94 as ::core::ffi::c_int as isize)
            as *mut uint16)
            == 0
            || v1 as ::core::ffi::c_int != 0
        {
            let ref mut fresh88 = *((&raw mut g_ram as *mut uint8)
                .offset(0xa96 as ::core::ffi::c_int as isize)
                as *mut uint16);
            *fresh88 = (*fresh88).wrapping_add(1);
            Samus_HandleAnimDelay();
        }
    } else {
        let ref mut fresh90 = *((&raw mut g_ram as *mut uint8)
            .offset(0xa94 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh90 = (*fresh90).wrapping_sub(1);
        let mut v0: bool = *fresh90 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int;
        if *((&raw mut g_ram as *mut uint8).offset(0xa94 as ::core::ffi::c_int as isize)
            as *mut uint16)
            == 0
            || v0 as ::core::ffi::c_int != 0
        {
            let ref mut fresh91 = *((&raw mut g_ram as *mut uint8)
                .offset(0xa96 as ::core::ffi::c_int as isize)
                as *mut uint16);
            *fresh91 = (*fresh91).wrapping_add(1);
            Samus_HandleAnimDelay();
        }
    };
}
pub unsafe fn Samus_Animate_NoFx() {
    let mut r18: uint16 = Samus_GetBottom_R18();
    *((&raw mut g_ram as *mut uint8).offset(0xa9c as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0xa66 as ::core::ffi::c_int as isize)
            as *mut uint16);
    if *((&raw mut g_ram as *mut uint8).offset(0xad2 as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        if *((&raw mut g_ram as *mut uint8).offset(0xad2 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            *((&raw mut g_ram as *mut uint8).offset(0xad2 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            QueueSfx2_Max6(0xe as uint16);
            if *((&raw mut g_ram as *mut uint8).offset(0xa74 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 4 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
                && (*(&raw mut g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    == 3 as ::core::ffi::c_int
                    || *(&raw mut g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int
                        == 20 as ::core::ffi::c_int)
            {
                QueueSfx1_Max6(0x30 as uint16);
            }
            Samus_SpawnWaterSplash(r18);
        } else {
            *((&raw mut g_ram as *mut uint8).offset(0xad2 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
        }
    }
}
pub unsafe fn Samus_Animate_WaterFx() {
    static mut kSamusPhys_AnimDelayInWater: uint16 = 3 as uint16;
    let mut r18: uint16 = Samus_GetBottom_R18();
    if *((&raw mut g_ram as *mut uint8).offset(0x195e as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
        && *((&raw mut g_ram as *mut uint8).offset(0x195e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - r18 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        && *((&raw mut g_ram as *mut uint8).offset(0x197e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 4 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0xa9c as ::core::ffi::c_int as isize)
            as *mut uint16) = kSamusPhys_AnimDelayInWater;
        if *((&raw mut g_ram as *mut uint8).offset(0xad2 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == kLiquidPhysicsType_Water as ::core::ffi::c_int
        {
            Samus_SpawnAirBubbles();
        } else {
            *((&raw mut g_ram as *mut uint8).offset(0xad2 as ::core::ffi::c_int as isize)
                as *mut uint16) = kLiquidPhysicsType_Water as ::core::ffi::c_int as uint16;
            QueueSfx2_Max6(0xd as uint16);
            Samus_SpawnWaterSplash(r18);
        }
    } else {
        Samus_Animate_NoFx();
    };
}
pub unsafe fn Samus_SpawnWaterSplash(mut r18: uint16) {
    static mut kWaterSplashTypeTable: [uint8; 28] = [
        1 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
    ];
    if kWaterSplashTypeTable
        [*(&raw mut g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize) as usize]
        != 0
    {
        *((&raw mut g_ram as *mut uint8).offset(0xaec as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(0 as ::core::ffi::c_int as isize) = 256 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xaec as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(1 as ::core::ffi::c_int as isize) = 256 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xad4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(0 as ::core::ffi::c_int as isize) = 3 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xad4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(1 as ::core::ffi::c_int as isize) = 3 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xadc as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(0 as ::core::ffi::c_int as isize) =
            (*((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + 4 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xadc as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(1 as ::core::ffi::c_int as isize) =
            (*((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - 3 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xae4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(0 as ::core::ffi::c_int as isize) =
            (r18 as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xae4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(1 as ::core::ffi::c_int as isize) =
            (r18 as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as uint16;
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0xaec as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(0 as ::core::ffi::c_int as isize) = 768 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xad4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(0 as ::core::ffi::c_int as isize) = 2 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xadc as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(0 as ::core::ffi::c_int as isize) = *((&raw mut g_ram as *mut uint8)
            .offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *((&raw mut g_ram as *mut uint8).offset(0xae4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(0 as ::core::ffi::c_int as isize) = *((&raw mut g_ram as *mut uint8)
            .offset(0x195e as ::core::ffi::c_int as isize)
            as *mut uint16);
    }
    Samus_SpawnAirBubbles();
}
pub unsafe fn Samus_SpawnAirBubbles() {
    let mut r20: uint16 = Samus_GetTop_R20();
    if (r20 as ::core::ffi::c_int
        - 24 as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0x195e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
        && *((&raw mut g_ram as *mut uint8).offset(0x5b6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x7f as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        && *((&raw mut g_ram as *mut uint8).offset(0xaec as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(2 as ::core::ffi::c_int as isize)
            == 0
    {
        *((&raw mut g_ram as *mut uint8).offset(0xaec as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(2 as ::core::ffi::c_int as isize) = 1280 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xad4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(2 as ::core::ffi::c_int as isize) = 3 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xadc as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(2 as ::core::ffi::c_int as isize) = *((&raw mut g_ram as *mut uint8)
            .offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *((&raw mut g_ram as *mut uint8).offset(0xae4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(2 as ::core::ffi::c_int as isize) =
            (*((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - *((&raw mut g_ram as *mut uint8).offset(0xb00 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                + 6 as ::core::ffi::c_int) as uint16;
        QueueSfx2_Max6(
            (if NextRandom() as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
                15 as ::core::ffi::c_int
            } else {
                17 as ::core::ffi::c_int
            }) as uint16,
        );
    }
    if *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        == kPose_00_FaceF_Powersuit as ::core::ffi::c_int
        || *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == kPose_9B_FaceF_VariaGravitySuit as ::core::ffi::c_int
        || *((&raw mut g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x20 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0xa9c as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    }
}
pub unsafe fn Samus_Animate_LavaFx() {
    static mut kSamusPhys_LavaDamagePerFrame: uint16 = 0 as uint16;
    static mut kSamusPhys_LavaSubdamagePerFrame: uint16 = 0x8000 as uint16;
    let mut r18: uint16 = Samus_GetBottom_R18();
    if *((&raw mut g_ram as *mut uint8).offset(0x1962 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
        && *((&raw mut g_ram as *mut uint8).offset(0x1962 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - r18 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
    {
        if *((&raw mut g_ram as *mut uint8).offset(0xb3e as ::core::ffi::c_int as isize)
            as *mut uint16)
            != 0
        {
            Samus_CancelSpeedBoost();
            *((&raw mut g_ram as *mut uint8).offset(0xb42 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xb44 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
        }
        if *((&raw mut g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x20 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            *((&raw mut g_ram as *mut uint8).offset(0xa9c as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xad2 as ::core::ffi::c_int as isize)
                as *mut uint16) = 2 as uint16;
        } else {
            if *((&raw mut g_ram as *mut uint8).offset(0x9da as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 7 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
                && *((&raw mut g_ram as *mut uint8).offset(0x9c2 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    - 71 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    == 0
            {
                QueueSfx3_Max3(0x2d as uint16);
            }
            AddToHiLo(
                (&raw mut g_ram as *mut uint8).offset(0xa50 as ::core::ffi::c_int as isize)
                    as *mut uint16,
                (&raw mut g_ram as *mut uint8).offset(0xa4e as ::core::ffi::c_int as isize)
                    as *mut uint16,
                (kSamusPhys_LavaDamagePerFrame as uint32) << 16 as ::core::ffi::c_int
                    | kSamusPhys_LavaSubdamagePerFrame as uint32,
            );
            Samus_Animate_SubmergedLavaAcid();
        }
    } else {
        Samus_Animate_NoFx();
    };
}
pub unsafe fn Samus_Animate_AcidFx() {
    static mut kSamusPhys_AcidSubdamagePerFrame: uint16 = 0x8000 as uint16;
    static mut kSamusPhys_AcidDamagePerFrame: uint16 = 1 as uint16;
    let mut r18: uint16 = Samus_GetBottom_R18();
    if *((&raw mut g_ram as *mut uint8).offset(0x1962 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
        && *((&raw mut g_ram as *mut uint8).offset(0x1962 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - r18 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
    {
        if *((&raw mut g_ram as *mut uint8).offset(0x9da as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 7 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
            && *((&raw mut g_ram as *mut uint8).offset(0x9c2 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - 71 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
        {
            QueueSfx3_Max3(0x2d as uint16);
        }
        AddToHiLo(
            (&raw mut g_ram as *mut uint8).offset(0xa50 as ::core::ffi::c_int as isize)
                as *mut uint16,
            (&raw mut g_ram as *mut uint8).offset(0xa4e as ::core::ffi::c_int as isize)
                as *mut uint16,
            (kSamusPhys_AcidDamagePerFrame as uint32) << 16 as ::core::ffi::c_int
                | kSamusPhys_AcidSubdamagePerFrame as uint32,
        );
        Samus_Animate_SubmergedLavaAcid();
    } else {
        Samus_Animate_NoFx();
    };
}
pub unsafe fn Samus_Animate_SubmergedLavaAcid() {
    static mut kSamusPhys_AnimDelayInAcid: uint16 = 2 as uint16;
    let mut r20: uint16 = Samus_GetTop_R20();
    *((&raw mut g_ram as *mut uint8).offset(0xa9c as ::core::ffi::c_int as isize) as *mut uint16) =
        kSamusPhys_AnimDelayInAcid;
    *((&raw mut g_ram as *mut uint8).offset(0xad2 as ::core::ffi::c_int as isize) as *mut uint16) =
        2 as uint16;
    if ((r20 as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0x1962 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
        && *((&raw mut g_ram as *mut uint8).offset(0xaec as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            & 0x400 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0xaec as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(0 as ::core::ffi::c_int as isize) = 1024 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xaec as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(1 as ::core::ffi::c_int as isize) = 1024 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xaec as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(2 as ::core::ffi::c_int as isize) = 1024 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xaec as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(3 as ::core::ffi::c_int as isize) = 1024 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xad4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(0 as ::core::ffi::c_int as isize) = 3 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xad4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(3 as ::core::ffi::c_int as isize) = 3 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xad4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(1 as ::core::ffi::c_int as isize) = -(32766 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xad4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(2 as ::core::ffi::c_int as isize) = -(32766 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xae4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(0 as ::core::ffi::c_int as isize) = *((&raw mut g_ram as *mut uint8)
            .offset(0x1962 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *((&raw mut g_ram as *mut uint8).offset(0xae4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(1 as ::core::ffi::c_int as isize) = *((&raw mut g_ram as *mut uint8)
            .offset(0x1962 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *((&raw mut g_ram as *mut uint8).offset(0xae4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(2 as ::core::ffi::c_int as isize) = *((&raw mut g_ram as *mut uint8)
            .offset(0x1962 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *((&raw mut g_ram as *mut uint8).offset(0xae4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(3 as ::core::ffi::c_int as isize) = *((&raw mut g_ram as *mut uint8)
            .offset(0x1962 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *((&raw mut g_ram as *mut uint8).offset(0xadc as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(0 as ::core::ffi::c_int as isize) =
            (*((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + 6 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xadc as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(1 as ::core::ffi::c_int as isize) = *((&raw mut g_ram as *mut uint8)
            .offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *((&raw mut g_ram as *mut uint8).offset(0xadc as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(2 as ::core::ffi::c_int as isize) = *((&raw mut g_ram as *mut uint8)
            .offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *((&raw mut g_ram as *mut uint8).offset(0xadc as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(3 as ::core::ffi::c_int as isize) =
            (*((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - 6 as ::core::ffi::c_int) as uint16;
        if *((&raw mut g_ram as *mut uint8).offset(0x9da as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            QueueSfx2_Max6(0x10 as uint16);
        }
    }
    if *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        == kPose_00_FaceF_Powersuit as ::core::ffi::c_int
        || *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == kPose_9B_FaceF_VariaGravitySuit as ::core::ffi::c_int
        || *((&raw mut g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x20 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0xa9c as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    }
}
unsafe fn Samus_AnimDelayFunc_0to5(mut jp: *const uint8) -> bool {
    return false_0 != 0;
}
unsafe fn Samus_AnimDelayFunc_6_GotoStartIfLittleHealth(mut jp: *const uint8) -> bool {
    *((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize) as *mut uint16) =
        (if *((&raw mut g_ram as *mut uint8).offset(0x9c2 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 30 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            *((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) as uint16;
    return true_0 != 0;
}
unsafe fn Samus_AnimDelayFunc_7(mut jp: *const uint8) -> bool {
    *((&raw mut g_ram as *mut uint8).offset(0xa58 as ::core::ffi::c_int as isize) as *mut uint16) =
        (0x9094cb as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    let ref mut fresh97 = *((&raw mut g_ram as *mut uint8)
        .offset(0xa96 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh97 = (*fresh97).wrapping_add(1);
    return true_0 != 0;
}
unsafe fn Samus_AnimDelayFunc_13_TransToPose(mut jp: *const uint8) -> bool {
    *((&raw mut g_ram as *mut uint8).offset(0xa2c as ::core::ffi::c_int as isize) as *mut uint16) =
        *jp.offset(1 as ::core::ffi::c_int as isize) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa32 as ::core::ffi::c_int as isize) as *mut uint16) =
        3 as uint16;
    return false_0 != 0;
}
unsafe fn Samus_AnimDelayFunc_8_AutoJumpHack(mut jp: *const uint8) -> bool {
    if *((&raw mut g_ram as *mut uint8).offset(0xa60 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        == 0x90e91d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        return Samus_AnimDelayFunc_13_TransToPose(jp);
    }
    if *((&raw mut g_ram as *mut uint8).offset(0xa28 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        != kPose_4B_FaceR_Jumptrans as ::core::ffi::c_int
        && *((&raw mut g_ram as *mut uint8).offset(0xa28 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != kPose_4C_FaceL_Jumptrans as ::core::ffi::c_int
        && *((&raw mut g_ram as *mut uint8).offset(0xa28 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != kPose_19_FaceR_SpinJump as ::core::ffi::c_int
        && *((&raw mut g_ram as *mut uint8).offset(0xa28 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != kPose_1A_FaceL_SpinJump as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0xa60 as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x90e926 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        return Samus_AnimDelayFunc_13_TransToPose(jp);
    }
    return false_0 != 0;
}
unsafe fn Samus_AnimDelayFunc_9_TransToPose(mut jp: *const uint8) -> bool {
    if *(jp.offset(1 as ::core::ffi::c_int as isize) as *mut uint16) as ::core::ffi::c_int
        & *((&raw mut g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        if *((&raw mut g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != 0
            || *((&raw mut g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                != 0
        {
            *((&raw mut g_ram as *mut uint8).offset(0xa2c as ::core::ffi::c_int as isize)
                as *mut uint16) = *jp.offset(6 as ::core::ffi::c_int as isize) as uint16;
        } else {
            *((&raw mut g_ram as *mut uint8).offset(0xa2c as ::core::ffi::c_int as isize)
                as *mut uint16) = *jp.offset(5 as ::core::ffi::c_int as isize) as uint16;
        }
    } else if *((&raw mut g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        != 0
        || *((&raw mut g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != 0
    {
        *((&raw mut g_ram as *mut uint8).offset(0xa2c as ::core::ffi::c_int as isize)
            as *mut uint16) = *jp.offset(4 as ::core::ffi::c_int as isize) as uint16;
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0xa2c as ::core::ffi::c_int as isize)
            as *mut uint16) = *jp.offset(3 as ::core::ffi::c_int as isize) as uint16;
    }
    *((&raw mut g_ram as *mut uint8).offset(0xa32 as ::core::ffi::c_int as isize) as *mut uint16) =
        3 as uint16;
    return false_0 != 0;
}
unsafe fn UNUSED_Samus_AnimDelayFunc_10(mut jp: *const uint8) -> bool {
    if *((&raw mut g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        != 0
        || *((&raw mut g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != 0
    {
        *((&raw mut g_ram as *mut uint8).offset(0xa2c as ::core::ffi::c_int as isize)
            as *mut uint16) = *jp.offset(2 as ::core::ffi::c_int as isize) as uint16;
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0xa2c as ::core::ffi::c_int as isize)
            as *mut uint16) = *jp.offset(1 as ::core::ffi::c_int as isize) as uint16;
    }
    *((&raw mut g_ram as *mut uint8).offset(0xa32 as ::core::ffi::c_int as isize) as *mut uint16) =
        3 as uint16;
    return false_0 != 0;
}
unsafe fn Samus_AnimDelayFunc_11_SelectDelaySequenceWalljump(mut jp: *const uint8) -> bool {
    let mut current_block: u64;
    if *((&raw mut g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0x20 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        let mut r20: uint16 = Samus_GetTop_R20();
        if *((&raw mut g_ram as *mut uint8).offset(0x195e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            if *((&raw mut g_ram as *mut uint8).offset(0x1962 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
                && *((&raw mut g_ram as *mut uint8).offset(0x1962 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    - r20 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    != 0
            {
                current_block = 16256859414872971286;
            } else {
                current_block = 15427931788582360902;
            }
        } else if *((&raw mut g_ram as *mut uint8).offset(0x195e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - r20 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
            && *((&raw mut g_ram as *mut uint8).offset(0x197e as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 4 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
        {
            current_block = 16256859414872971286;
        } else {
            current_block = 15427931788582360902;
        }
    } else {
        current_block = 15427931788582360902;
    }
    match current_block {
        15427931788582360902 => {
            if *((&raw mut g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 8 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                QueueSfx1_Max6(0x33 as uint16);
                let ref mut fresh94 = *((&raw mut g_ram as *mut uint8)
                    .offset(0xa96 as ::core::ffi::c_int as isize)
                    as *mut uint16);
                *fresh94 = (*fresh94 as ::core::ffi::c_int + 21 as ::core::ffi::c_int) as uint16;
                current_block = 12349973810996921269;
            } else if *((&raw mut g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0x200 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            {
                current_block = 16256859414872971286;
            } else {
                QueueSfx1_Max6(0x3e as uint16);
                let ref mut fresh96 = *((&raw mut g_ram as *mut uint8)
                    .offset(0xa96 as ::core::ffi::c_int as isize)
                    as *mut uint16);
                *fresh96 = (*fresh96 as ::core::ffi::c_int + 11 as ::core::ffi::c_int) as uint16;
                current_block = 12349973810996921269;
            }
        }
        _ => {}
    }
    match current_block {
        16256859414872971286 => {
            QueueSfx1_Max6(0x31 as uint16);
            let ref mut fresh95 = *((&raw mut g_ram as *mut uint8)
                .offset(0xa96 as ::core::ffi::c_int as isize)
                as *mut uint16);
            *fresh95 = (*fresh95 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
        }
        _ => {}
    }
    return true_0 != 0;
}
unsafe fn Samus_AnimDelayFunc_12_TransToPose(mut jp: *const uint8) -> bool {
    if *(jp.offset(1 as ::core::ffi::c_int as isize) as *mut uint16) as ::core::ffi::c_int
        & *((&raw mut g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0xa2c as ::core::ffi::c_int as isize)
            as *mut uint16) = *jp.offset(4 as ::core::ffi::c_int as isize) as uint16;
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0xa2c as ::core::ffi::c_int as isize)
            as *mut uint16) = *jp.offset(3 as ::core::ffi::c_int as isize) as uint16;
    }
    *((&raw mut g_ram as *mut uint8).offset(0xa32 as ::core::ffi::c_int as isize) as *mut uint16) =
        3 as uint16;
    return false_0 != 0;
}
unsafe fn Samus_AnimDelayFunc_14_Goto(mut jp: *const uint8) -> bool {
    let ref mut fresh93 = *((&raw mut g_ram as *mut uint8)
        .offset(0xa96 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh93 = (*fresh93 as ::core::ffi::c_int
        - *jp.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
        as uint16;
    return true_0 != 0;
}
unsafe fn Samus_AnimDelayFunc_15_GotoStart(mut jp: *const uint8) -> bool {
    *((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    return true_0 != 0;
}
static mut kAnimDelayFuncs: [Option<Func_AnimDelay>; 16] = unsafe {
    [
        Some(Samus_AnimDelayFunc_0to5 as unsafe fn(*const uint8) -> bool),
        Some(Samus_AnimDelayFunc_0to5 as unsafe fn(*const uint8) -> bool),
        Some(Samus_AnimDelayFunc_0to5 as unsafe fn(*const uint8) -> bool),
        Some(Samus_AnimDelayFunc_0to5 as unsafe fn(*const uint8) -> bool),
        Some(Samus_AnimDelayFunc_0to5 as unsafe fn(*const uint8) -> bool),
        Some(Samus_AnimDelayFunc_0to5 as unsafe fn(*const uint8) -> bool),
        Some(Samus_AnimDelayFunc_6_GotoStartIfLittleHealth as unsafe fn(*const uint8) -> bool),
        Some(Samus_AnimDelayFunc_7 as unsafe fn(*const uint8) -> bool),
        Some(Samus_AnimDelayFunc_8_AutoJumpHack as unsafe fn(*const uint8) -> bool),
        Some(Samus_AnimDelayFunc_9_TransToPose as unsafe fn(*const uint8) -> bool),
        Some(UNUSED_Samus_AnimDelayFunc_10 as unsafe fn(*const uint8) -> bool),
        Some(Samus_AnimDelayFunc_11_SelectDelaySequenceWalljump as unsafe fn(*const uint8) -> bool),
        Some(Samus_AnimDelayFunc_12_TransToPose as unsafe fn(*const uint8) -> bool),
        Some(Samus_AnimDelayFunc_13_TransToPose as unsafe fn(*const uint8) -> bool),
        Some(Samus_AnimDelayFunc_14_Goto as unsafe fn(*const uint8) -> bool),
        Some(Samus_AnimDelayFunc_15_GotoStart as unsafe fn(*const uint8) -> bool),
    ]
};
unsafe fn Samus_HandleSpeedBoosterAnimDelay(mut jp: *const uint8) -> uint8 {
    if *((&raw mut g_ram as *mut uint8).offset(0xb3c as ::core::ffi::c_int as isize) as *mut uint16)
        == 0
        || *((&raw mut g_ram as *mut uint8).offset(0x9b6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & *((&raw mut g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        || *(&raw mut g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            != 1 as ::core::ffi::c_int
    {
        return *jp.offset(0 as ::core::ffi::c_int as isize);
    }
    if *((&raw mut g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0x2000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xa94 as ::core::ffi::c_int as isize)
            as *mut uint16) = (*((&raw mut g_ram as *mut uint8)
            .offset(0xa9c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + *RomPtr_91(
                *(g_rom.offset(
                    (((0x91b5d1 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x91b5d1 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16),
            )
            .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            as uint16;
        return 0 as uint8;
    }
    let ref mut fresh92 = *((&raw mut g_ram as *mut uint8)
        .offset(0xb3e as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh92 = (*fresh92).wrapping_sub(1);
    if *fresh92 as uint8 != 0 {
        return *jp.offset(0 as ::core::ffi::c_int as isize);
    }
    let mut v2: uint16 = *((&raw mut g_ram as *mut uint8)
        .offset(0xb3e as ::core::ffi::c_int as isize) as *mut uint16);
    if *((&raw mut g_ram as *mut uint8).offset(0xb3e as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0x400 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        v2 = (*((&raw mut g_ram as *mut uint8).offset(0xb3e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + 256 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb3e as ::core::ffi::c_int as isize)
            as *mut uint16) = v2;
        if v2 as ::core::ffi::c_int & 0x400 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            *((&raw mut g_ram as *mut uint8).offset(0xb40 as ::core::ffi::c_int as isize)
                as *mut uint16) = 1 as uint16;
            QueueSfx3_Max6(3 as uint16);
        }
    }
    let mut v3: ::core::ffi::c_int = *(&raw mut v2 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>() as usize)
            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) as ::core::ffi::c_int;
    *((&raw mut g_ram as *mut uint8).offset(0xb3e as ::core::ffi::c_int as isize) as *mut uint16) =
        (*(g_rom.offset(
            (((0x91b61f as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x91b61f as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(v3 as isize) as ::core::ffi::c_int
            | *((&raw mut g_ram as *mut uint8).offset(0xb3e as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0xff00 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa94 as ::core::ffi::c_int as isize) as *mut uint16) =
        (*((&raw mut g_ram as *mut uint8).offset(0xa9c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + *RomPtr_91(
                *(g_rom.offset(
                    (((0x91b5de as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x91b5de as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset(v3 as isize) as uint16_t,
            )
            .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int) as uint16;
    return 0 as uint8;
}
unsafe fn Samus_HandleAnimDelay() {
    let mut p: *const uint8 = RomPtr_91(
        *(g_rom.offset(
            (((0x91b010 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x91b010 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(
                *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                    as *mut uint16) as isize,
            ) as uint16_t,
    );
    if *p.offset(
        *((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
            as *mut uint16) as isize,
    ) as ::core::ffi::c_int
        & 0x80 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        let mut v1: uint8 = Samus_HandleSpeedBoosterAnimDelay(p.offset(
            *((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int as isize,
        ));
        printf(
            b"v1=%x\n\0" as *const u8 as *const ::core::ffi::c_char,
            v1 as ::core::ffi::c_int,
        );
        if kAnimDelayFuncs[(v1 as ::core::ffi::c_int & 0xf as ::core::ffi::c_int) as usize]
            .expect("non-null function pointer")(p.offset(
            *((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int as isize,
        )) {
            *((&raw mut g_ram as *mut uint8).offset(0xa94 as ::core::ffi::c_int as isize)
                as *mut uint16) = (*((&raw mut g_ram as *mut uint8)
                .offset(0xa9c as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + *p.offset(
                    *((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
                        as *mut uint16) as isize,
                ) as ::core::ffi::c_int) as uint16;
        }
    } else {
        if *((&raw mut g_ram as *mut uint8).offset(0xb3c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != 0
            && *(&raw mut g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
                == 1 as ::core::ffi::c_int
        {
            let mut addr: uint16 = (if *((&raw mut g_ram as *mut uint8)
                .offset(0x9a2 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0x2000 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                *(g_rom.offset(
                    (((0x91b5de as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x91b5de as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset(
                        *((&raw mut g_ram as *mut uint8)
                            .offset(0xb3e as ::core::ffi::c_int as isize)
                            as *mut uint16 as *mut uint8)
                            .offset(
                                (::core::mem::size_of::<uint16>() as usize)
                                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                                    .wrapping_sub(1 as usize)
                                    as isize,
                            ) as isize,
                    ) as ::core::ffi::c_int
            } else {
                *(g_rom.offset(
                    (((0x91b5d1 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x91b5d1 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16) as ::core::ffi::c_int
            }) as uint16;
            p = RomPtr_91(addr as uint16_t);
        }
        *((&raw mut g_ram as *mut uint8).offset(0xa94 as ::core::ffi::c_int as isize)
            as *mut uint16) = (*((&raw mut g_ram as *mut uint8)
            .offset(0xa9c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + *p.offset(
                *((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
                    as *mut uint16) as isize,
            ) as ::core::ffi::c_int) as uint16;
    };
}
static mut kSamusIsBottomDrawnFuncs: [Option<Func_U8>; 28] = unsafe {
    [
        Some(SamusBottomDrawn_0_Standing as unsafe fn() -> uint8),
        Some(SamusBottomDrawn_1 as unsafe fn() -> uint8),
        Some(SamusBottomDrawn_1 as unsafe fn() -> uint8),
        Some(SamusBottomDrawn_3_SpinJump as unsafe fn() -> uint8),
        Some(SamusBottomDrawn_4 as unsafe fn() -> uint8),
        Some(SamusBottomDrawn_1 as unsafe fn() -> uint8),
        Some(SamusBottomDrawn_1 as unsafe fn() -> uint8),
        Some(SamusBottomDrawn_4 as unsafe fn() -> uint8),
        Some(SamusBottomDrawn_4 as unsafe fn() -> uint8),
        Some(SamusBottomDrawn_4 as unsafe fn() -> uint8),
        Some(SamusBottomDrawn_A_Knockback as unsafe fn() -> uint8),
        Some(SamusBottomDrawn_1 as unsafe fn() -> uint8),
        Some(SamusBottomDrawn_1 as unsafe fn() -> uint8),
        Some(UNUSED_SamusBottomDrawn_D as unsafe fn() -> uint8),
        Some(SamusBottomDrawn_1 as unsafe fn() -> uint8),
        Some(SamusBottomDrawn_F_Transitions as unsafe fn() -> uint8),
        Some(SamusBottomDrawn_1 as unsafe fn() -> uint8),
        Some(SamusBottomDrawn_4 as unsafe fn() -> uint8),
        Some(SamusBottomDrawn_4 as unsafe fn() -> uint8),
        Some(SamusBottomDrawn_4 as unsafe fn() -> uint8),
        Some(SamusBottomDrawn_14_WallJump as unsafe fn() -> uint8),
        Some(SamusBottomDrawn_1 as unsafe fn() -> uint8),
        Some(SamusBottomDrawn_1 as unsafe fn() -> uint8),
        Some(SamusBottomDrawn_1 as unsafe fn() -> uint8),
        Some(SamusBottomDrawn_1 as unsafe fn() -> uint8),
        Some(SamusBottomDrawn_19_DamageBoost as unsafe fn() -> uint8),
        Some(SamusBottomDrawn_1 as unsafe fn() -> uint8),
        Some(SamusBottomDrawn_1B as unsafe fn() -> uint8),
    ]
};
pub unsafe fn Samus_Draw() {
    let mut v0: PairU16 = PairU16 { k: 0, j: 0 };
    if *((&raw mut g_ram as *mut uint8).offset(0x18aa as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        != 0
        || *((&raw mut g_ram as *mut uint8).offset(0x18a8 as ::core::ffi::c_int as isize)
            as *mut uint16)
            == 0
        || *((&raw mut g_ram as *mut uint8).offset(0xa68 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != 0
        || *((&raw mut g_ram as *mut uint8).offset(0x5b6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
    {
        let mut v2: uint16 = (2 as ::core::ffi::c_int
            * *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xac8 as ::core::ffi::c_int as isize)
            as *mut uint16) = (*((&raw mut g_ram as *mut uint8)
            .offset(0xa96 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + *(g_rom.offset(
                (((0x929263 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x929263 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(
                    *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                        as *mut uint16) as isize,
                ) as ::core::ffi::c_int) as uint16;
        let mut a: uint16 = *((&raw mut g_ram as *mut uint8)
            .offset(0xac8 as ::core::ffi::c_int as isize)
            as *mut uint16);
        v0 = Samus_CalcSpritemapPos(
            (2 as ::core::ffi::c_int
                * *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint16,
        );
        DrawSamusSpritemap(a, v0.k, v0.j);
        let mut R36: uint16 = v2;
        if kSamusIsBottomDrawnFuncs
            [*(&raw mut g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize) as usize]
            .expect("non-null function pointer")() as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            != 0
        {
            *((&raw mut g_ram as *mut uint8).offset(0xaca as ::core::ffi::c_int as isize)
                as *mut uint16) = (*((&raw mut g_ram as *mut uint8)
                .offset(0xa96 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + *(g_rom.offset(
                    (((0x92945d as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x92945d as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset((R36 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int) as uint16;
            DrawSamusSpritemap(
                *((&raw mut g_ram as *mut uint8).offset(0xaca as ::core::ffi::c_int as isize)
                    as *mut uint16),
                *((&raw mut g_ram as *mut uint8).offset(0xb04 as ::core::ffi::c_int as isize)
                    as *mut uint16),
                *((&raw mut g_ram as *mut uint8).offset(0xb06 as ::core::ffi::c_int as isize)
                    as *mut uint16),
            );
        }
    }
    SetSamusTilesDefsForCurAnim();
}
pub unsafe fn SamusBottomDrawn_1() -> uint8 {
    return 1 as uint8;
}
pub unsafe fn SamusBottomDrawn_4() -> uint8 {
    *((&raw mut g_ram as *mut uint8).offset(0xaca as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    return 0 as uint8;
}
pub unsafe fn SamusBottomDrawn_0_Standing() -> uint8 {
    let mut v1: *mut OamEnt = ::core::ptr::null_mut::<OamEnt>();
    if *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        == kPose_00_FaceF_Powersuit as ::core::ffi::c_int
    {
        let mut v0: uint16 = *((&raw mut g_ram as *mut uint8)
            .offset(0x590 as ::core::ffi::c_int as isize)
            as *mut uint16);
        v1 = gOamEnt(
            *((&raw mut g_ram as *mut uint8).offset(0x590 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int,
        );
        *(&raw mut (*v1).xcoord as *mut uint16) =
            (*((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - 7 as ::core::ffi::c_int
                - *((&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint16;
        *(&raw mut (*v1).ycoord as *mut uint16) =
            (*((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - 17 as ::core::ffi::c_int
                - *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint16;
        *(&raw mut (*v1).charnum as *mut uint16) = 14369 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x590 as ::core::ffi::c_int as isize)
            as *mut uint16) = (v0 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
    }
    return 1 as uint8;
}
pub unsafe fn SamusBottomDrawn_3_SpinJump() -> uint8 {
    if *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        == kPose_81_FaceR_Screwattack as ::core::ffi::c_int
        || *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == kPose_82_FaceL_Screwattack as ::core::ffi::c_int
        || *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == kPose_1B_FaceR_SpaceJump as ::core::ffi::c_int
        || *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == kPose_1C_FaceL_SpaceJump as ::core::ffi::c_int
        || *((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
            as *mut uint16)
            == 0
        || *((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - kPose_0B_MoveR_Gun as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
    {
        return 1 as uint8;
    }
    *((&raw mut g_ram as *mut uint8).offset(0xaca as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    return 0 as uint8;
}
pub unsafe fn SamusBottomDrawn_A_Knockback() -> uint8 {
    if *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        != kPose_D7_FaceR_CrystalFlashEnd as ::core::ffi::c_int
        && *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != kPose_D8_FaceL_CrystalFlashEnd as ::core::ffi::c_int
        || *((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 3 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
    {
        return 1 as uint8;
    }
    *((&raw mut g_ram as *mut uint8).offset(0xaca as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    return 0 as uint8;
}
pub unsafe fn SamusBottomDrawn_F_Transitions() -> uint8 {
    let mut current_block: u64;
    if *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - kPose_F1_FaceR_CrouchTrans_AimU as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        return 1 as uint8;
    }
    if *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - 219 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        if *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 221 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            if *((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
                as *mut uint16)
                != 0
            {
                current_block = 10229858379301036899;
            } else {
                current_block = 6873731126896040597;
            }
        } else if *((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != 2 as ::core::ffi::c_int
        {
            current_block = 10229858379301036899;
        } else {
            current_block = 6873731126896040597;
        }
        match current_block {
            10229858379301036899 => {}
            _ => return 1 as uint8,
        }
    } else if *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        == kPose_35_FaceR_CrouchTrans as ::core::ffi::c_int
        || *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == kPose_36_FaceL_CrouchTrans as ::core::ffi::c_int
        || *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == kPose_3B_FaceR_StandTrans as ::core::ffi::c_int
        || *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == kPose_3C_FaceL_StandTrans as ::core::ffi::c_int
    {
        return 1 as uint8;
    }
    *((&raw mut g_ram as *mut uint8).offset(0xaca as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    return 0 as uint8;
}
pub unsafe fn UNUSED_SamusBottomDrawn_D() -> uint8 {
    if *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        != kPose_44_FaceL_Turn_Crouch as ::core::ffi::c_int
            | kPose_01_FaceR_Normal as ::core::ffi::c_int
            | 0x20 as ::core::ffi::c_int
        && *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != kPose_44_FaceL_Turn_Crouch as ::core::ffi::c_int
                | kPose_02_FaceL_Normal as ::core::ffi::c_int
                | 0x20 as ::core::ffi::c_int
        || *((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 1 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
    {
        return 1 as uint8;
    }
    *((&raw mut g_ram as *mut uint8).offset(0xaca as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    return 0 as uint8;
}
pub unsafe fn SamusBottomDrawn_14_WallJump() -> uint8 {
    if *((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - 3 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
        || *((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 13 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
    {
        return 1 as uint8;
    }
    *((&raw mut g_ram as *mut uint8).offset(0xaca as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    return 0 as uint8;
}
pub unsafe fn SamusBottomDrawn_19_DamageBoost() -> uint8 {
    if *((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - 2 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
        || *((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 9 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
    {
        return 1 as uint8;
    }
    *((&raw mut g_ram as *mut uint8).offset(0xaca as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    return 0 as uint8;
}
pub unsafe fn SamusBottomDrawn_1B() -> uint8 {
    if *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - kPose_CF_FaceR_Ranintowall_AimUR as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        if *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != kPose_CB_FaceR_Shinespark_Vert as ::core::ffi::c_int
            && *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                != kPose_CC_FaceL_Shinespark_Vert as ::core::ffi::c_int
        {
            return 1 as uint8;
        }
    } else if *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        != kPose_E8_FaceR_Drained_CrouchFalling as ::core::ffi::c_int
        && *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != kPose_E9_FaceL_Drained_CrouchFalling as ::core::ffi::c_int
        || *((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 2 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
    {
        return 1 as uint8;
    }
    *((&raw mut g_ram as *mut uint8).offset(0xaca as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    return 0 as uint8;
}
pub unsafe fn Samus_DrawEchoes() {
    if *((&raw mut g_ram as *mut uint8).offset(0xaae as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        if *((&raw mut g_ram as *mut uint8).offset(0xb3e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0xff00 as ::core::ffi::c_int
            == 1024 as ::core::ffi::c_int
        {
            if *((&raw mut g_ram as *mut uint8).offset(0xab0 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(1 as ::core::ffi::c_int as isize)
                != 0
            {
                Samus_DrawEcho(2 as uint16);
            }
            if *((&raw mut g_ram as *mut uint8).offset(0xab0 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(0 as ::core::ffi::c_int as isize)
                != 0
            {
                Samus_DrawEcho(0 as uint16);
            }
        }
        return;
    }
    let mut current_block_19: u64;
    let mut i: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        let mut v1: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
        if !(*((&raw mut g_ram as *mut uint8).offset(0xab0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize)
            == 0)
        {
            let mut v2: uint16 = *((&raw mut g_ram as *mut uint8)
                .offset(0xab8 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize);
            let mut v3: uint16 = 0;
            if v2 as ::core::ffi::c_int
                != *((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
            {
                if ((v2 as ::core::ffi::c_int
                    - *((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int) as int16
                    as ::core::ffi::c_int)
                    < 0 as ::core::ffi::c_int
                {
                    v3 = (v2 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
                } else {
                    v3 = (v2 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
                }
                *((&raw mut g_ram as *mut uint8).offset(0xab8 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) = v3;
            }
            if *((&raw mut g_ram as *mut uint8).offset(0xac0 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                let mut v5: uint16 = (*((&raw mut g_ram as *mut uint8)
                    .offset(0xac0 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize)
                    as ::core::ffi::c_int
                    + *((&raw mut g_ram as *mut uint8).offset(0xab0 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset(v1 as isize) as ::core::ffi::c_int)
                    as uint16;
                *((&raw mut g_ram as *mut uint8).offset(0xab0 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) = v5;
                if ((v5 as ::core::ffi::c_int
                    - *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int) as int16
                    as ::core::ffi::c_int)
                    < 0 as ::core::ffi::c_int
                {
                    *((&raw mut g_ram as *mut uint8).offset(0xab0 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset(v1 as isize) = 0 as uint16;
                    current_block_19 = 7815301370352969686;
                } else {
                    current_block_19 = 5634871135123216486;
                }
            } else {
                let mut v4: uint16 = (*((&raw mut g_ram as *mut uint8)
                    .offset(0xac0 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize)
                    as ::core::ffi::c_int
                    + *((&raw mut g_ram as *mut uint8).offset(0xab0 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset(v1 as isize) as ::core::ffi::c_int)
                    as uint16;
                *((&raw mut g_ram as *mut uint8).offset(0xab0 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) = v4;
                if (v4 as ::core::ffi::c_int
                    - *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int) as int16
                    as ::core::ffi::c_int
                    >= 0 as ::core::ffi::c_int
                {
                    *((&raw mut g_ram as *mut uint8).offset(0xab0 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset(v1 as isize) = 0 as uint16;
                    current_block_19 = 7815301370352969686;
                } else {
                    current_block_19 = 5634871135123216486;
                }
            }
            match current_block_19 {
                7815301370352969686 => {}
                _ => {
                    Samus_DrawEcho(i as uint16);
                }
            }
        }
        i -= 2 as ::core::ffi::c_int;
    }
    if *((&raw mut g_ram as *mut uint8).offset(0xab0 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(1 as ::core::ffi::c_int as isize)
        == 0
        && *((&raw mut g_ram as *mut uint8).offset(0xab0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(0 as ::core::ffi::c_int as isize)
            == 0
    {
        *((&raw mut g_ram as *mut uint8).offset(0xaae as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    }
}
pub unsafe fn Samus_DrawEcho(mut j: uint16) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v2: int16 = (*((&raw mut g_ram as *mut uint8)
        .offset(0xab8 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        - (*(g_rom.offset(
            (((0x91b629 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x91b629 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut SamusPoseParams)
            .offset(
                *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                    as *mut uint16) as isize,
            ))
        .y_offset_to_gfx as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16;
    if v2 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int
        && v2 as ::core::ffi::c_int - 248 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0
    {
        DrawSamusSpritemap(
            *((&raw mut g_ram as *mut uint8).offset(0xac8 as ::core::ffi::c_int as isize)
                as *mut uint16),
            (*((&raw mut g_ram as *mut uint8).offset(0xab0 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                - *((&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint16,
            v2 as uint16,
        );
        if *((&raw mut g_ram as *mut uint8).offset(0xaca as ::core::ffi::c_int as isize)
            as *mut uint16)
            != 0
        {
            DrawSamusSpritemap(
                *((&raw mut g_ram as *mut uint8).offset(0xaca as ::core::ffi::c_int as isize)
                    as *mut uint16),
                (*((&raw mut g_ram as *mut uint8).offset(0xab0 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) as ::core::ffi::c_int
                    - *((&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int) as uint16,
                v2 as uint16,
            );
        }
    }
}
pub unsafe fn Samus_DrawShinesparkCrashEchoes(mut k: uint16) {
    if *((&raw mut g_ram as *mut uint8).offset(0x5b6 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        let mut a: uint16 = (*((&raw mut g_ram as *mut uint8)
            .offset(0xa96 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + *(g_rom.offset(
                (((0x929263 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x929263 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(
                    *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                        as *mut uint16) as isize,
                ) as ::core::ffi::c_int) as uint16;
        let mut v2: int16 = (*((&raw mut g_ram as *mut uint8)
            .offset(0xab8 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            - (*(g_rom.offset(
                (((0x91b629 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x91b629 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut SamusPoseParams)
                .offset(
                    *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                        as *mut uint16) as isize,
                ))
            .y_offset_to_gfx as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as int16;
        if v2 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int
            && v2 as ::core::ffi::c_int - 248 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0
        {
            DrawSamusSpritemap(
                a,
                (*((&raw mut g_ram as *mut uint8).offset(0xab0 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int
                    - *((&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int) as uint16,
                v2 as uint16,
            );
            if kSamusIsBottomDrawnFuncs[*(&raw mut g_ram as *mut uint8)
                .offset(0xa1f as ::core::ffi::c_int as isize)
                as usize]
                .expect("non-null function pointer")() as ::core::ffi::c_int
                & 1 as ::core::ffi::c_int
                != 0
            {
                let mut v5: uint16 = (*((&raw mut g_ram as *mut uint8)
                    .offset(0xa96 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    + *(g_rom.offset(
                        (((0x92945d as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                            << 15 as ::core::ffi::c_int
                            | 0x92945d as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                            & 0x3fffff as ::core::ffi::c_int) as isize,
                    ) as *const uint8 as *mut uint16)
                        .offset(
                            *((&raw mut g_ram as *mut uint8)
                                .offset(0xa1c as ::core::ffi::c_int as isize)
                                as *mut uint16) as isize,
                        ) as ::core::ffi::c_int) as uint16;
                DrawSamusSpritemap(
                    v5,
                    (*((&raw mut g_ram as *mut uint8).offset(0xab0 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_int
                        - *((&raw mut g_ram as *mut uint8)
                            .offset(0x911 as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int) as uint16,
                    v2 as uint16,
                );
            }
        }
    }
}
pub unsafe fn Samus_DrawShinesparkCrashEchoProjectiles() {
    if *((&raw mut g_ram as *mut uint8).offset(0x5b6 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        if *((&raw mut g_ram as *mut uint8).offset(0xac0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(3 as ::core::ffi::c_int as isize)
            != 0
        {
            Samus_DrawEcho(6 as uint16);
        }
        if *((&raw mut g_ram as *mut uint8).offset(0xac0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(2 as ::core::ffi::c_int as isize)
            != 0
        {
            Samus_DrawEcho(4 as uint16);
        }
    }
}
pub unsafe fn Samus_DrawStartingDeathAnim() {
    let ref mut fresh149 = *((&raw mut g_ram as *mut uint8)
        .offset(0xa94 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh149 = (*fresh149).wrapping_sub(1);
    let mut v0: bool =
        *fresh149 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int;
    if *((&raw mut g_ram as *mut uint8).offset(0xa94 as ::core::ffi::c_int as isize) as *mut uint16)
        == 0
        || v0 as ::core::ffi::c_int != 0
    {
        let ref mut fresh150 = *((&raw mut g_ram as *mut uint8)
            .offset(0xa96 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh150 = (*fresh150).wrapping_add(1);
        Samus_HandleAnimDelay();
    }
    Samus_DrawDuringDeathAnim();
}
pub unsafe fn Samus_DrawDuringDeathAnim() {
    let mut v1: uint16 = (2 as ::core::ffi::c_int
        * *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as uint16;
    let mut a: uint16 = (*((&raw mut g_ram as *mut uint8)
        .offset(0xa96 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        + *(g_rom.offset(
            (((0x929263 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x929263 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(
                *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                    as *mut uint16) as isize,
            ) as ::core::ffi::c_int) as uint16;
    Samus_CalcSpritemapPos(
        (2 as ::core::ffi::c_int
            * *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16,
    );
    DrawSamusSpritemap(
        a,
        (*((&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + *((&raw mut g_ram as *mut uint8).offset(0xb04 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16,
        (*((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + *((&raw mut g_ram as *mut uint8).offset(0xb06 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16,
    );
    let mut R36: uint16 = v1;
    if kSamusIsBottomDrawnFuncs
        [*(&raw mut g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize) as usize]
        .expect("non-null function pointer")() as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        DrawSamusSpritemap(
            (*((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + *(g_rom.offset(
                    (((0x92945d as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x92945d as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset((R36 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int) as uint16,
            (*((&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + *((&raw mut g_ram as *mut uint8).offset(0xb04 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint16,
            (*((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + *((&raw mut g_ram as *mut uint8).offset(0xb06 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint16,
        );
    }
    SetSamusTilesDefsForCurAnim();
}
pub unsafe fn Samus_DrawWhenNotAnimatingOrDying() {
    let mut v0: PairU16 = PairU16 { k: 0, j: 0 };
    let mut v2: uint16 = (2 as ::core::ffi::c_int
        * *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as uint16;
    let mut a: uint16 = (*((&raw mut g_ram as *mut uint8)
        .offset(0xa96 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        + *(g_rom.offset(
            (((0x929263 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x929263 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(
                *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                    as *mut uint16) as isize,
            ) as ::core::ffi::c_int) as uint16;
    v0 = Samus_CalcSpritemapPos(
        (2 as ::core::ffi::c_int
            * *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16,
    );
    DrawSamusSpritemap(a, v0.k, v0.j);
    let mut R36: uint16 = v2;
    if kSamusIsBottomDrawnFuncs
        [*(&raw mut g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize) as usize]
        .expect("non-null function pointer")() as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        DrawSamusSpritemap(
            (*((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + *(g_rom.offset(
                    (((0x92945d as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x92945d as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset((R36 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int) as uint16,
            *((&raw mut g_ram as *mut uint8).offset(0xb04 as ::core::ffi::c_int as isize)
                as *mut uint16),
            *((&raw mut g_ram as *mut uint8).offset(0xb06 as ::core::ffi::c_int as isize)
                as *mut uint16),
        );
    }
    SetSamusTilesDefsForCurAnim();
}
static mut kAtmosphericTypeFuncs: [Option<FuncXY_V>; 8] = unsafe {
    [
        None,
        Some(AtmosphericTypeFunc_1_FootstepSplash as unsafe fn(uint16, uint16) -> ()),
        Some(AtmosphericTypeFunc_1_FootstepSplash as unsafe fn(uint16, uint16) -> ()),
        Some(AtmosphericTypeFunc_3_DivingSplash as unsafe fn(uint16, uint16) -> ()),
        Some(AtmosphericTypeFunc_4_LavaSurfaceDmg as unsafe fn(uint16, uint16) -> ()),
        Some(AtmosphericTypeFunc_5_Bubbles as unsafe fn(uint16, uint16) -> ()),
        Some(AtmosphericTypeFunc_67_Dust as unsafe fn(uint16, uint16) -> ()),
        Some(AtmosphericTypeFunc_67_Dust as unsafe fn(uint16, uint16) -> ()),
    ]
};
pub unsafe fn HandleAtmosphericEffects() {
    let mut v4: int16 = 0;
    let mut current_block_6: u64;
    let mut i: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        let mut v1: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
        let mut v2: uint16 = *((&raw mut g_ram as *mut uint8)
            .offset(0xaec as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        if !(v2 == 0) {
            let mut r18: uint16 =
                (2 as ::core::ffi::c_int * v2 as uint8 as ::core::ffi::c_int) as uint16;
            let mut v3: uint16 = (2 as ::core::ffi::c_int
                * *(&raw mut v2 as *mut uint8).offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ) as ::core::ffi::c_int) as uint16;
            v4 = (*((&raw mut g_ram as *mut uint8).offset(0xad4 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int) as int16;
            *((&raw mut g_ram as *mut uint8).offset(0xad4 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = v4 as uint16;
            if v4 != 0 {
                if (v4 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
                    if v4 as ::core::ffi::c_int
                        != 0x8000 as ::core::ffi::c_int as int16 as ::core::ffi::c_int
                    {
                        current_block_6 = 11174649648027449784;
                    } else {
                        *((&raw mut g_ram as *mut uint8)
                            .offset(0xad4 as ::core::ffi::c_int as isize)
                            as *mut uint16)
                            .offset(v1 as isize) = *(RomPtr_90(
                            (r18 as ::core::ffi::c_int
                                + *(g_rom.offset(
                                    (((0x908b93 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                                        << 15 as ::core::ffi::c_int
                                        | 0x908b93 as ::core::ffi::c_int
                                            & 0x7fff as ::core::ffi::c_int)
                                        & 0x3fffff as ::core::ffi::c_int)
                                        as isize,
                                ) as *const uint8
                                    as *mut uint16)
                                    .offset(
                                        (v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
                                            as isize,
                                    ) as ::core::ffi::c_int)
                                as uint16_t,
                        ) as *mut uint16);
                        current_block_6 = 12349973810996921269;
                    }
                } else {
                    current_block_6 = 12349973810996921269;
                }
            } else {
                *((&raw mut g_ram as *mut uint8).offset(0xad4 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) = *(RomPtr_90(
                    (r18 as ::core::ffi::c_int
                        + *(g_rom.offset(
                            (((0x908b93 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                                << 15 as ::core::ffi::c_int
                                | 0x908b93 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                                & 0x3fffff as ::core::ffi::c_int)
                                as isize,
                        ) as *const uint8 as *mut uint16)
                            .offset((v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_int) as uint16_t,
                ) as *mut uint16);
                let mut v5: uint16 =
                    (*((&raw mut g_ram as *mut uint8).offset(0xaec as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset(v1 as isize) as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int) as uint16;
                *((&raw mut g_ram as *mut uint8).offset(0xaec as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) = v5;
                if (v5 as uint8 as ::core::ffi::c_int
                    - *(g_rom.offset(
                        (((0x908bef as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                            << 15 as ::core::ffi::c_int
                            | 0x908bef as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                            & 0x3fffff as ::core::ffi::c_int) as isize,
                    ) as *const uint8 as *mut uint16)
                        .offset((v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
                    >= 0 as ::core::ffi::c_int
                {
                    *((&raw mut g_ram as *mut uint8).offset(0xaec as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset(v1 as isize) = 0 as uint16;
                    current_block_6 = 11174649648027449784;
                } else {
                    current_block_6 = 12349973810996921269;
                }
            }
            match current_block_6 {
                11174649648027449784 => {}
                _ => {
                    kAtmosphericTypeFuncs
                        [(v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize]
                        .expect("non-null function pointer")(v3, i as uint16);
                }
            }
        }
        i -= 2 as ::core::ffi::c_int;
    }
}
pub unsafe fn AtmosphericTypeFunc_1_FootstepSplash(mut k: uint16, mut j: uint16) {
    let mut v4: int16 = 0;
    let mut v5: *mut OamEnt = ::core::ptr::null_mut::<OamEnt>();
    let mut v6: int16 = 0;
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut r18: uint16 = (2 as ::core::ffi::c_int
        * *(((&raw mut g_ram as *mut uint8).offset(0xaec as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) as *mut uint16 as *mut uint8)
            .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
        as uint16;
    let mut v3: uint16 = *((&raw mut g_ram as *mut uint8)
        .offset(0x590 as ::core::ffi::c_int as isize) as *mut uint16);
    v4 = (*((&raw mut g_ram as *mut uint8).offset(0xadc as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize) as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        - 4 as ::core::ffi::c_int) as int16;
    if v4 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
        if *((&raw mut g_ram as *mut uint8).offset(0xadc as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            - 260 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            v5 = gOamEnt(
                *((&raw mut g_ram as *mut uint8).offset(0x590 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int,
            );
            (*v5).xcoord = v4 as uint8;
            v6 = (*((&raw mut g_ram as *mut uint8).offset(0xae4 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v2 as isize) as ::core::ffi::c_int
                - *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                - 4 as ::core::ffi::c_int) as int16;
            if v6 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
                if *((&raw mut g_ram as *mut uint8).offset(0xae4 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v2 as isize) as ::core::ffi::c_int
                    - *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                    - 260 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    != 0
                {
                    (*v5).ycoord = v6 as uint8;
                    *(&raw mut (*v5).charnum as *mut uint16) = *(RomPtr_90(
                        (r18 as ::core::ffi::c_int
                            + *(g_rom.offset(
                                (((0x908bff as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                                    << 15 as ::core::ffi::c_int
                                    | 0x908bff as ::core::ffi::c_int
                                        & 0x7fff as ::core::ffi::c_int)
                                    & 0x3fffff as ::core::ffi::c_int)
                                    as isize,
                            ) as *const uint8 as *mut uint16)
                                .offset(
                                    (k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                                ) as ::core::ffi::c_int) as uint16_t,
                    )
                        as *mut uint16);
                    *((&raw mut g_ram as *mut uint8).offset(0x590 as ::core::ffi::c_int as isize)
                        as *mut uint16) =
                        (v3 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
                }
            }
        }
    }
}
pub unsafe fn AtmosphericTypeFunc_Common(mut j: uint16, mut a: uint16) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut g_ram as *mut uint8).offset(0xae4 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        & 0xff00 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        DrawSamusSpritemap(
            a,
            (*((&raw mut g_ram as *mut uint8).offset(0xadc as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                - *((&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint16,
            (*((&raw mut g_ram as *mut uint8).offset(0xae4 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                - *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint16,
        );
    }
}
pub unsafe fn AtmosphericTypeFunc_3_DivingSplash(mut k: uint16, mut j: uint16) {
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut r18: uint16 = *(((&raw mut g_ram as *mut uint8)
        .offset(0xaec as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v2 as isize) as *mut uint16 as *mut uint8)
        .offset(0 as ::core::ffi::c_int as isize) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xae4 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v2 as isize) = *((&raw mut g_ram as *mut uint8)
        .offset(0x195e as ::core::ffi::c_int as isize)
        as *mut uint16);
    AtmosphericTypeFunc_Common(
        j,
        (r18 as ::core::ffi::c_int + 399 as ::core::ffi::c_int) as uint16,
    );
}
pub unsafe fn AtmosphericTypeFunc_4_LavaSurfaceDmg(mut k: uint16, mut j: uint16) {
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v3: uint16 = 0;
    if j as ::core::ffi::c_int & 4 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        v3 = (*((&raw mut g_ram as *mut uint8).offset(0xadc as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) as ::core::ffi::c_int
            - 1 as ::core::ffi::c_int) as uint16;
    } else {
        v3 = (*((&raw mut g_ram as *mut uint8).offset(0xadc as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int) as uint16;
    }
    *((&raw mut g_ram as *mut uint8).offset(0xadc as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v2 as isize) = v3;
    let ref mut fresh10 = *((&raw mut g_ram as *mut uint8)
        .offset(0xae4 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    *fresh10 = (*fresh10).wrapping_sub(1);
    AtmosphericTypeFunc_1_FootstepSplash(k, j);
}
pub unsafe fn AtmosphericTypeFunc_67_Dust(mut k: uint16, mut j: uint16) {
    let ref mut fresh11 = *((&raw mut g_ram as *mut uint8)
        .offset(0xae4 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    *fresh11 = (*fresh11).wrapping_sub(1);
    AtmosphericTypeFunc_1_FootstepSplash(k, j);
}
pub unsafe fn AtmosphericTypeFunc_5_Bubbles(mut k: uint16, mut j: uint16) {
    let mut r18: uint16 = *(((&raw mut g_ram as *mut uint8)
        .offset(0xaec as ::core::ffi::c_int as isize) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        as *mut uint16 as *mut uint8)
        .offset(0 as ::core::ffi::c_int as isize) as uint16;
    AtmosphericTypeFunc_Common(
        j,
        (r18 as ::core::ffi::c_int + 390 as ::core::ffi::c_int) as uint16,
    );
}
static mut kSamus_CalcSpritemapPos: [Option<Func_Y_To_PairU16>; 28] = [
    Some(Samus_CalcSpritemapPos_Standing as unsafe fn(uint16) -> PairU16),
    Some(Samus_CalcSpritemapPos_Default as unsafe fn(uint16) -> PairU16),
    Some(Samus_CalcSpritemapPos_Default as unsafe fn(uint16) -> PairU16),
    Some(Samus_CalcSpritemapPos_Default as unsafe fn(uint16) -> PairU16),
    Some(Samus_CalcSpritemapPos_Default as unsafe fn(uint16) -> PairU16),
    Some(Samus_CalcSpritemapPos_Default as unsafe fn(uint16) -> PairU16),
    Some(Samus_CalcSpritemapPos_Default as unsafe fn(uint16) -> PairU16),
    Some(Samus_CalcSpritemapPos_Default as unsafe fn(uint16) -> PairU16),
    Some(Samus_CalcSpritemapPos_Default as unsafe fn(uint16) -> PairU16),
    Some(Samus_CalcSpritemapPos_Default as unsafe fn(uint16) -> PairU16),
    Some(Samus_CalcSpritemapPos_Default as unsafe fn(uint16) -> PairU16),
    Some(Samus_CalcSpritemapPos_Default as unsafe fn(uint16) -> PairU16),
    Some(Samus_CalcSpritemapPos_Default as unsafe fn(uint16) -> PairU16),
    Some(Samus_CalcSpritemapPos_Default as unsafe fn(uint16) -> PairU16),
    Some(Samus_CalcSpritemapPos_Default as unsafe fn(uint16) -> PairU16),
    Some(Samus_CalcSpritemapPos_Crouch as unsafe fn(uint16) -> PairU16),
    Some(Samus_CalcSpritemapPos_Default as unsafe fn(uint16) -> PairU16),
    Some(Samus_CalcSpritemapPos_Default as unsafe fn(uint16) -> PairU16),
    Some(Samus_CalcSpritemapPos_Default as unsafe fn(uint16) -> PairU16),
    Some(Samus_CalcSpritemapPos_Default as unsafe fn(uint16) -> PairU16),
    Some(Samus_CalcSpritemapPos_Default as unsafe fn(uint16) -> PairU16),
    Some(Samus_CalcSpritemapPos_Default as unsafe fn(uint16) -> PairU16),
    Some(Samus_CalcSpritemapPos_Default as unsafe fn(uint16) -> PairU16),
    Some(Samus_CalcSpritemapPos_Default as unsafe fn(uint16) -> PairU16),
    Some(Samus_CalcSpritemapPos_Default as unsafe fn(uint16) -> PairU16),
    Some(Samus_CalcSpritemapPos_Default as unsafe fn(uint16) -> PairU16),
    Some(Samus_CalcSpritemapPos_Default as unsafe fn(uint16) -> PairU16),
    Some(Samus_CalcSpritemapPos_Special as unsafe fn(uint16) -> PairU16),
];
pub unsafe fn Samus_CalcSpritemapPos(mut k: uint16) -> PairU16 {
    let mut v1: PairU16 = PairU16 { k: 0, j: 0 };
    if *((&raw mut g_ram as *mut uint8).offset(0x93f as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        v1 = kSamus_CalcSpritemapPos
            [*(&raw mut g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize) as usize]
            .expect("non-null function pointer")(k);
    } else {
        let mut old_x: uint16 = *((&raw mut g_ram as *mut uint8)
            .offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16);
        let mut old_y: uint16 = *((&raw mut g_ram as *mut uint8)
            .offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16);
        Samus_CalcPos_Mode7();
        v1 = kSamus_CalcSpritemapPos
            [*(&raw mut g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize) as usize]
            .expect("non-null function pointer")(k);
        *((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16) = old_y;
        *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) = old_x;
    }
    return MakePairU16(v1.k, v1.j);
}
pub unsafe fn Samus_CalcSpritemapPos_Default(mut j: uint16) -> PairU16 {
    let mut v1: ::core::ffi::c_int = *(&raw mut (*(g_rom.offset(
        (((0x91b629 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x91b629 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut SamusPoseParams)
        .offset(0 as ::core::ffi::c_int as isize))
    .y_offset_to_gfx)
        .offset((4 as ::core::ffi::c_int * j as ::core::ffi::c_int) as isize)
        as int8 as ::core::ffi::c_int;
    *((&raw mut g_ram as *mut uint8).offset(0xb06 as ::core::ffi::c_int as isize) as *mut uint16) =
        (*((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - v1
            - *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb04 as ::core::ffi::c_int as isize) as *mut uint16) =
        (*((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16;
    return MakePairU16(
        *((&raw mut g_ram as *mut uint8).offset(0xb04 as ::core::ffi::c_int as isize)
            as *mut uint16),
        *((&raw mut g_ram as *mut uint8).offset(0xb06 as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
}
pub unsafe fn Samus_CalcSpritemapPos_Standing(mut j: uint16) -> PairU16 {
    static mut g_byte_908D28: [uint8; 16] = [
        3 as ::core::ffi::c_int as uint8,
        6 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        3 as ::core::ffi::c_int as uint8,
        6 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        3 as ::core::ffi::c_int as uint8,
        3 as ::core::ffi::c_int as uint8,
        6 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        3 as ::core::ffi::c_int as uint8,
        3 as ::core::ffi::c_int as uint8,
        6 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
    ];
    let mut v1: int16 = 0;
    let mut v3: PairU16 = PairU16 { k: 0, j: 0 };
    v1 = (j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as int16;
    if j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int == 0
        || v1 as ::core::ffi::c_int == kPose_9B_FaceF_VariaGravitySuit as ::core::ffi::c_int
    {
        if *((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 2 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            *((&raw mut g_ram as *mut uint8).offset(0xb06 as ::core::ffi::c_int as isize)
                as *mut uint16) =
                (*((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    - 1 as ::core::ffi::c_int
                    - *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int) as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xb04 as ::core::ffi::c_int as isize)
                as *mut uint16) =
                (*((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    - *((&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int) as uint16;
            return MakePairU16(
                (*((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    - *((&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int) as uint16,
                (*((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    - 1 as ::core::ffi::c_int
                    - *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int) as uint16,
            );
        }
    } else if v1 as ::core::ffi::c_int - kPose_A4_FaceR_LandJump as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
        && v1 as ::core::ffi::c_int - kPose_A8_FaceR_Grappling as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
    {
        let mut r18: uint16 = *((&raw const g_byte_908D28 as *const uint8).offset(
            (*((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + 4 as ::core::ffi::c_int * (v1 as ::core::ffi::c_int - 164 as ::core::ffi::c_int))
                as uint16 as isize,
        ) as *const uint8 as *mut uint16);
        *((&raw mut g_ram as *mut uint8).offset(0xb06 as ::core::ffi::c_int as isize)
            as *mut uint16) = (*((&raw mut g_ram as *mut uint8)
            .offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - r18 as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb04 as ::core::ffi::c_int as isize)
            as *mut uint16) = (*((&raw mut g_ram as *mut uint8)
            .offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16;
        return MakePairU16(
            (*((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - *((&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint16,
            (*((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - r18 as ::core::ffi::c_int
                - *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint16,
        );
    }
    v3 = Samus_CalcSpritemapPos_Default(j);
    return MakePairU16(v3.k, v3.j);
}
pub unsafe fn Samus_CalcSpritemapPos_Crouch(mut j: uint16) -> PairU16 {
    static mut byte_908D80: [int8; 24] = [
        -(8 as ::core::ffi::c_int) as int8,
        0 as ::core::ffi::c_int as int8,
        -(8 as ::core::ffi::c_int) as int8,
        0 as ::core::ffi::c_int as int8,
        -(4 as ::core::ffi::c_int) as int8,
        -(2 as ::core::ffi::c_int) as int8,
        -(4 as ::core::ffi::c_int) as int8,
        -(2 as ::core::ffi::c_int) as int8,
        0 as ::core::ffi::c_int as int8,
        0 as ::core::ffi::c_int as int8,
        0 as ::core::ffi::c_int as int8,
        0 as ::core::ffi::c_int as int8,
        -(4 as ::core::ffi::c_int) as int8,
        0 as ::core::ffi::c_int as int8,
        -(4 as ::core::ffi::c_int) as int8,
        0 as ::core::ffi::c_int as int8,
        5 as ::core::ffi::c_int as int8,
        4 as ::core::ffi::c_int as int8,
        5 as ::core::ffi::c_int as int8,
        4 as ::core::ffi::c_int as int8,
        0 as ::core::ffi::c_int as int8,
        0 as ::core::ffi::c_int as int8,
        0 as ::core::ffi::c_int as int8,
        0 as ::core::ffi::c_int as int8,
    ];
    let mut v1: int16 = 0;
    v1 = (j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as int16;
    if (j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
        - kPose_35_FaceR_CrouchTrans as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
        || v1 as ::core::ffi::c_int - kPose_41_FaceL_Morphball_Ground as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
    {
        return Samus_CalcSpritemapPos_Default(j);
    } else {
        let mut v4: ::core::ffi::c_int = byte_908D80[(*((&raw mut g_ram as *mut uint8)
            .offset(0xa96 as ::core::ffi::c_int as isize)
            as *mut uint16)
            as ::core::ffi::c_int
            + 2 as ::core::ffi::c_int * (v1 as ::core::ffi::c_int - 53 as ::core::ffi::c_int))
            as uint16 as usize] as ::core::ffi::c_int;
        *((&raw mut g_ram as *mut uint8).offset(0xb06 as ::core::ffi::c_int as isize)
            as *mut uint16) = (v4
            + *((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb04 as ::core::ffi::c_int as isize)
            as *mut uint16) = (*((&raw mut g_ram as *mut uint8)
            .offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16;
        return MakePairU16(
            (*((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - *((&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint16,
            *((&raw mut g_ram as *mut uint8).offset(0xb06 as ::core::ffi::c_int as isize)
                as *mut uint16),
        );
    };
}
pub unsafe fn Samus_CalcSpritemapPos_Special(mut j: uint16) -> PairU16 {
    static mut byte_908DEF: [int8; 32] = [
        7 as ::core::ffi::c_int as int8,
        5 as ::core::ffi::c_int as int8,
        -(8 as ::core::ffi::c_int) as int8,
        -(8 as ::core::ffi::c_int) as int8,
        -(8 as ::core::ffi::c_int) as int8,
        -(8 as ::core::ffi::c_int) as int8,
        -(8 as ::core::ffi::c_int) as int8,
        -(5 as ::core::ffi::c_int) as int8,
        4 as ::core::ffi::c_int as int8,
        4 as ::core::ffi::c_int as int8,
        4 as ::core::ffi::c_int as int8,
        4 as ::core::ffi::c_int as int8,
        0 as ::core::ffi::c_int as int8,
        0 as ::core::ffi::c_int as int8,
        4 as ::core::ffi::c_int as int8,
        -(3 as ::core::ffi::c_int) as int8,
        -(5 as ::core::ffi::c_int) as int8,
        0 as ::core::ffi::c_int as int8,
        0 as ::core::ffi::c_int as int8,
        4 as ::core::ffi::c_int as int8,
        -(3 as ::core::ffi::c_int) as int8,
        -(5 as ::core::ffi::c_int) as int8,
        -(3 as ::core::ffi::c_int) as int8,
        4 as ::core::ffi::c_int as int8,
        0 as ::core::ffi::c_int as int8,
        0 as ::core::ffi::c_int as int8,
        4 as ::core::ffi::c_int as int8,
        0 as ::core::ffi::c_int as int8,
        0 as ::core::ffi::c_int as int8,
        4 as ::core::ffi::c_int as int8,
        0 as ::core::ffi::c_int as int8,
        0 as ::core::ffi::c_int as int8,
    ];
    let mut v1: ::core::ffi::c_int = 0;
    let mut v2: ::core::ffi::c_int = 0;
    v1 = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if v1 == kPose_E8_FaceR_Drained_CrouchFalling as ::core::ffi::c_int
        || v1 == kPose_E9_FaceL_Drained_CrouchFalling as ::core::ffi::c_int
    {
        v2 = byte_908DEF[*((&raw mut g_ram as *mut uint8)
            .offset(0xa96 as ::core::ffi::c_int as isize) as *mut uint16)
            as usize] as ::core::ffi::c_int;
    } else {
        if v1 != kPose_EA_FaceR_Drained_Stand as ::core::ffi::c_int
            && v1 != kPose_EB_FaceL_Drained_Stand as ::core::ffi::c_int
            || ((*((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - 5 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int
        {
            return Samus_CalcSpritemapPos_Default(j);
        }
        v2 = -(3 as ::core::ffi::c_int);
    }
    *((&raw mut g_ram as *mut uint8).offset(0xb06 as ::core::ffi::c_int as isize) as *mut uint16) =
        (v2 + *((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb04 as ::core::ffi::c_int as isize) as *mut uint16) =
        (*((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16;
    return MakePairU16(
        (*((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16,
        (v2 + *((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16,
    );
}
pub unsafe fn SetLiquidPhysicsType() {
    let mut r18: uint16 = Samus_GetBottom_R18();
    match (*((&raw mut g_ram as *mut uint8).offset(0x196e as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int)
        >> 1 as ::core::ffi::c_int
    {
        1 | 2 => {
            if *((&raw mut g_ram as *mut uint8).offset(0x1962 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
                && *((&raw mut g_ram as *mut uint8).offset(0x1962 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    - r18 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    != 0
            {
                *((&raw mut g_ram as *mut uint8).offset(0xad2 as ::core::ffi::c_int as isize)
                    as *mut uint16) = kLiquidPhysicsType_LavaAcid as ::core::ffi::c_int as uint16;
            } else {
                *((&raw mut g_ram as *mut uint8).offset(0xad2 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 0 as uint16;
            }
        }
        3 => {
            if *((&raw mut g_ram as *mut uint8).offset(0x195e as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
                && *((&raw mut g_ram as *mut uint8).offset(0x195e as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    - r18 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    != 0
                && *((&raw mut g_ram as *mut uint8).offset(0x197e as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    & 4 as ::core::ffi::c_int
                    == 0 as ::core::ffi::c_int
            {
                *((&raw mut g_ram as *mut uint8).offset(0xad2 as ::core::ffi::c_int as isize)
                    as *mut uint16) = kLiquidPhysicsType_Water as ::core::ffi::c_int as uint16;
            } else {
                *((&raw mut g_ram as *mut uint8).offset(0xad2 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 0 as uint16;
            }
        }
        _ => {
            *((&raw mut g_ram as *mut uint8).offset(0xad2 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
        }
    };
}
pub unsafe fn Samus_HandleMovement_X() {
    Samus_HandleExtraRunspeedX();
    Samus_MoveX(Samus_CalcBaseSpeed_X(Samus_DetermineSpeedTableEntryPtr_X()));
}
pub unsafe fn Samus_MoveX(mut amt: int32) {
    let mut current_block: u64;
    if *((&raw mut g_ram as *mut uint8).offset(0xb4a as ::core::ffi::c_int as isize) as *mut uint16)
        == 0
        || *((&raw mut g_ram as *mut uint8).offset(0xb4a as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == 2 as ::core::ffi::c_int
    {
        if *(&raw mut g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            != 4 as ::core::ffi::c_int
        {
            current_block = 14900655479109243551;
        } else {
            current_block = 7095457783677275021;
        }
    } else if *(&raw mut g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        != 8 as ::core::ffi::c_int
    {
        current_block = 14900655479109243551;
    } else {
        current_block = 7095457783677275021;
    }
    match current_block {
        7095457783677275021 => {
            amt = Samus_CalcDisplacementMoveLeft(amt);
        }
        _ => {
            amt = Samus_CalcDisplacementMoveRight(amt);
        }
    }
    if amt < 0 as int32 {
        Samus_MoveLeft(amt);
    } else {
        Samus_MoveRight(amt);
    };
}
pub unsafe fn Samus_BombJumpRisingXMovement_() {
    let mut current_block: u64;
    let mut amt: int32 = 0;
    if *((&raw mut g_ram as *mut uint8).offset(0xa52 as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        amt = Samus_CalcBaseSpeed_X(Samus_DetermineSpeedTableEntryPtr_X());
        if *((&raw mut g_ram as *mut uint8).offset(0xa54 as ::core::ffi::c_int as isize)
            as *mut uint16)
            == 0
        {
            current_block = 1279983688403190034;
        } else {
            current_block = 17762005381502015266;
        }
    } else {
        amt = Samus_CalcBaseSpeed_X(addr_stru_909F25 as ::core::ffi::c_int as uint16);
        if *((&raw mut g_ram as *mut uint8).offset(0xa56 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint8 as ::core::ffi::c_int
            != 1 as ::core::ffi::c_int
        {
            current_block = 17762005381502015266;
        } else {
            current_block = 1279983688403190034;
        }
    }
    match current_block {
        1279983688403190034 => {
            amt = Samus_CalcDisplacementMoveLeft(amt);
        }
        _ => {
            amt = Samus_CalcDisplacementMoveRight(amt);
        }
    }
    if amt < 0 as int32 {
        Samus_MoveLeft(amt);
    } else {
        Samus_MoveRight(amt);
    };
}
pub unsafe fn Samus_BombJumpRisingYMovement_() {
    if *((&raw mut g_ram as *mut uint8).offset(0xb36 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        == 1 as ::core::ffi::c_int
    {
        if *((&raw mut g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            *((&raw mut g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xb36 as ::core::ffi::c_int as isize)
                as *mut uint16) = 2 as uint16;
            if *((&raw mut g_ram as *mut uint8).offset(0xa56 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint8 as ::core::ffi::c_int
                != 2 as ::core::ffi::c_int
            {
                *((&raw mut g_ram as *mut uint8).offset(0xb4a as ::core::ffi::c_int as isize)
                    as *mut uint16) = 2 as uint16;
            }
        } else if *((&raw mut g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 1 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            if *((&raw mut g_ram as *mut uint8).offset(0xa60 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                != 0x90e91d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
            {
                *((&raw mut g_ram as *mut uint8).offset(0xa60 as ::core::ffi::c_int as isize)
                    as *mut uint16) =
                    (0x90e913 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            }
        }
    }
}
pub unsafe fn Samus_BombJumpFallingXMovement_() {
    let mut amt: int32 = Samus_CalcBaseSpeed_X(Samus_DetermineSpeedTableEntryPtr_X());
    if *((&raw mut g_ram as *mut uint8).offset(0xa62 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        == 1 as ::core::ffi::c_int
    {
        amt = Samus_CalcDisplacementMoveLeft(amt);
    } else {
        amt = Samus_CalcDisplacementMoveRight(amt);
    }
    if amt < 0 as int32 {
        Samus_MoveLeft(amt);
    } else {
        Samus_MoveRight(amt);
    };
}
pub unsafe fn Samus_BombJumpFallingYMovement_() {
    if *((&raw mut g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - 5 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        AddToHiLo(
            (&raw mut g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
                as *mut uint16,
            (&raw mut g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize)
                as *mut uint16,
            (*((&raw mut g_ram as *mut uint8).offset(0xb34 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint32)
                << 16 as ::core::ffi::c_int
                | *((&raw mut g_ram as *mut uint8).offset(0xb32 as ::core::ffi::c_int as isize)
                    as *mut uint16) as uint32,
        );
    }
    Samus_MoveDown(
        ((*((&raw mut g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
            as *mut uint16) as uint32)
            << 16 as ::core::ffi::c_int
            | *((&raw mut g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize)
                as *mut uint16) as uint32) as int32,
    );
}
pub unsafe fn Samus_JumpingMovement() {
    let mut current_block: u64;
    Samus_HandleExtraRunspeedX();
    if *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        == kPose_4B_FaceR_Jumptrans as ::core::ffi::c_int
        || *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == kPose_4C_FaceL_Jumptrans as ::core::ffi::c_int
        || *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - kPose_55_FaceR_Jumptrans_AimU as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
            && *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - kPose_5B as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
    {
        *((&raw mut g_ram as *mut uint8).offset(0xb4a as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        Samus_Move_NoBaseSpeed_X();
        Samus_MoveExtraY();
        return;
    }
    if *((&raw mut g_ram as *mut uint8).offset(0xb36 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        == 1 as ::core::ffi::c_int
        && (*((&raw mut g_ram as *mut uint8).offset(0x9b4 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & *((&raw mut g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
            || *((&raw mut g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int)
    {
        *((&raw mut g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb36 as ::core::ffi::c_int as isize)
            as *mut uint16) = 2 as uint16;
    }
    let mut pair: Pair_Bool_Amt =
        Samus_CalcBaseSpeed_NoDecel_X(Samus_DetermineSpeedTableEntryPtr_X());
    if *(&raw mut g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == 20 as ::core::ffi::c_int
    {
        if *((&raw mut g_ram as *mut uint8).offset(0xb4a as ::core::ffi::c_int as isize)
            as *mut uint16)
            == 0
        {
            *((&raw mut g_ram as *mut uint8).offset(0xb4a as ::core::ffi::c_int as isize)
                as *mut uint16) = 2 as uint16;
        }
        current_block = 13338778356158832468;
    } else if *((&raw mut g_ram as *mut uint8).offset(0xb4a as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        != 0
        || *((&raw mut g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x100 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        || *((&raw mut g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x200 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
    {
        current_block = 13338778356158832468;
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0xb46 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb48 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xdd0 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        current_block = 294518303723010965;
    }
    match current_block {
        13338778356158832468 => {
            Samus_MoveX(pair.amt);
        }
        _ => {}
    }
    Samus_MoveY_WithSpeedCalc();
}
pub unsafe fn Samus_SpinJumpMovement() {
    let mut current_block: u64;
    static mut g_word_909E9F: uint16 = 8 as uint16;
    Samus_HandleExtraRunspeedX();
    if *((&raw mut g_ram as *mut uint8).offset(0xb36 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        == 1 as ::core::ffi::c_int
        && (*((&raw mut g_ram as *mut uint8).offset(0x9b4 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & *((&raw mut g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
            || *((&raw mut g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int)
    {
        *((&raw mut g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb36 as ::core::ffi::c_int as isize)
            as *mut uint16) = 2 as uint16;
    }
    let mut pair: Pair_Bool_Amt =
        Samus_CalcBaseSpeed_NoDecel_X(Samus_DetermineSpeedTableEntryPtr_X());
    if pair.flag as ::core::ffi::c_int != 0
        || *((&raw mut g_ram as *mut uint8).offset(0xb4a as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == 1 as ::core::ffi::c_int
    {
        current_block = 15844657471994422988;
    } else {
        if *(&raw mut g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            == 4 as ::core::ffi::c_int
        {
            if *((&raw mut g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & kButton_Left as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                current_block = 15844657471994422988;
            } else {
                current_block = 11746438652493739369;
            }
        } else if *((&raw mut g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & kButton_Right as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            current_block = 11746438652493739369;
        } else {
            current_block = 15844657471994422988;
        }
        match current_block {
            15844657471994422988 => {}
            _ => {
                pair.amt = 0 as ::core::ffi::c_int as int32;
                *((&raw mut g_ram as *mut uint8).offset(0xb46 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 0 as uint16;
                *((&raw mut g_ram as *mut uint8).offset(0xb48 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 0 as uint16;
                *((&raw mut g_ram as *mut uint8).offset(0xdd0 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 0 as uint16;
                *((&raw mut g_ram as *mut uint8).offset(0xb4a as ::core::ffi::c_int as isize)
                    as *mut uint16) = 0 as uint16;
                current_block = 7037269245041226537;
            }
        }
    }
    match current_block {
        15844657471994422988 => {
            if *((&raw mut g_ram as *mut uint8).offset(0xb4a as ::core::ffi::c_int as isize)
                as *mut uint16)
                == 0
            {
                *((&raw mut g_ram as *mut uint8).offset(0xb4a as ::core::ffi::c_int as isize)
                    as *mut uint16) = 2 as uint16;
            }
        }
        _ => {}
    }
    Samus_MoveX(pair.amt);
    if Samus_WallJumpCheck((g_word_909E9F as int16 as int32) << 16 as ::core::ffi::c_int) == 0 {
        Samus_MoveY_WithSpeedCalc();
    }
}
pub unsafe fn Samus_CheckStartFalling() {
    if *((&raw mut g_ram as *mut uint8).offset(0xb36 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        == 1 as ::core::ffi::c_int
        && *((&raw mut g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb36 as ::core::ffi::c_int as isize)
            as *mut uint16) = 2 as uint16;
    }
}
pub unsafe fn Samus_MoveY_WithSpeedCalc() {
    let mut amt: int32 = ((*((&raw mut g_ram as *mut uint8)
        .offset(0xb2e as ::core::ffi::c_int as isize) as *mut uint16)
        as uint32)
        << 16 as ::core::ffi::c_int
        | *((&raw mut g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize)
            as *mut uint16) as uint32) as int32;
    if *((&raw mut g_ram as *mut uint8).offset(0xb36 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        == 2 as ::core::ffi::c_int
    {
        if *((&raw mut g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != 5 as ::core::ffi::c_int
        {
            AddToHiLo(
                (&raw mut g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
                    as *mut uint16,
                (&raw mut g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize)
                    as *mut uint16,
                (*((&raw mut g_ram as *mut uint8).offset(0xb34 as ::core::ffi::c_int as isize)
                    as *mut uint16) as uint32)
                    << 16 as ::core::ffi::c_int
                    | *((&raw mut g_ram as *mut uint8).offset(0xb32 as ::core::ffi::c_int as isize)
                        as *mut uint16) as uint32,
            );
        }
    } else {
        AddToHiLo(
            (&raw mut g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
                as *mut uint16,
            (&raw mut g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize)
                as *mut uint16,
            -(((*((&raw mut g_ram as *mut uint8).offset(0xb34 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int)
                << 16 as ::core::ffi::c_int
                | *((&raw mut g_ram as *mut uint8).offset(0xb32 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as int32) as uint32,
        );
    }
    if *((&raw mut g_ram as *mut uint8).offset(0xb36 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        != 2 as ::core::ffi::c_int
    {
        amt = -amt;
    }
    amt = (amt as uint32).wrapping_add(
        (*((&raw mut g_ram as *mut uint8).offset(0xb5c as ::core::ffi::c_int as isize)
            as *mut uint16) as uint32)
            << 16 as ::core::ffi::c_int
            | *((&raw mut g_ram as *mut uint8).offset(0xb5a as ::core::ffi::c_int as isize)
                as *mut uint16) as uint32,
    ) as int32 as int32;
    if amt < 0 as int32 {
        Samus_MoveUp(amt);
    } else {
        Samus_MoveDown(amt);
    };
}
pub unsafe fn Samus_FallingMovement() {
    Samus_HandleExtraRunspeedX();
    let mut pair: Pair_Bool_Amt =
        Samus_CalcBaseSpeed_NoDecel_X(Samus_DetermineSpeedTableEntryPtr_X());
    if *((&raw mut g_ram as *mut uint8).offset(0xb4a as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        != 0
        || *((&raw mut g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & kButton_Right as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        || *((&raw mut g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & kButton_Left as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
    {
        Samus_MoveX(pair.amt);
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0xb46 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb48 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xdd0 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    }
    Samus_CheckStartFalling();
    Samus_MoveY_WithSpeedCalc();
}
pub unsafe fn Samus_MorphedFallingMovement() {
    let mut pair: Pair_Bool_Amt =
        Samus_CalcBaseSpeed_NoDecel_X(Samus_DetermineSpeedTableEntryPtr_X());
    if *((&raw mut g_ram as *mut uint8).offset(0xb4a as ::core::ffi::c_int as isize) as *mut uint16)
        == 0
        && *((&raw mut g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & kButton_Right as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        && *((&raw mut g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & kButton_Left as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
    {
        pair.amt = 0 as ::core::ffi::c_int as int32;
        *((&raw mut g_ram as *mut uint8).offset(0xb46 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb48 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xdd0 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    }
    Samus_MoveX(pair.amt);
    Samus_CheckStartFalling();
    Samus_MoveY_WithSpeedCalc();
}
pub unsafe fn Samus_MorphedBouncingMovement() {
    let mut pair: Pair_Bool_Amt =
        Samus_CalcBaseSpeed_NoDecel_X(Samus_DetermineSpeedTableEntryPtr_X());
    if *((&raw mut g_ram as *mut uint8).offset(0xb4a as ::core::ffi::c_int as isize) as *mut uint16)
        == 0
        && *((&raw mut g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & kButton_Right as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        && *((&raw mut g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & kButton_Left as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
    {
        pair.amt = 0 as ::core::ffi::c_int as int32;
        *((&raw mut g_ram as *mut uint8).offset(0xb46 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb48 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xdd0 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    }
    Samus_MoveX(pair.amt);
    if *((&raw mut g_ram as *mut uint8).offset(0xa52 as ::core::ffi::c_int as isize) as *mut uint16)
        == 0
    {
        if *((&raw mut g_ram as *mut uint8).offset(0xb5c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != 0
            || *((&raw mut g_ram as *mut uint8).offset(0xb5a as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                != 0
        {
            *((&raw mut g_ram as *mut uint8).offset(0xb36 as ::core::ffi::c_int as isize)
                as *mut uint16) = 2 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            let mut amt: int32 = ((*((&raw mut g_ram as *mut uint8)
                .offset(0xb5c as ::core::ffi::c_int as isize)
                as *mut uint16) as uint32)
                << 16 as ::core::ffi::c_int
                | *((&raw mut g_ram as *mut uint8).offset(0xb5a as ::core::ffi::c_int as isize)
                    as *mut uint16) as uint32) as int32;
            if amt < 0 as int32 {
                Samus_MoveUp(amt);
            } else {
                Samus_MoveDown(
                    amt + ((1 as ::core::ffi::c_int as int16 as int32) << 16 as ::core::ffi::c_int),
                );
            }
        } else {
            Samus_CheckStartFalling();
            Samus_MoveY_WithSpeedCalc();
        }
    }
}
pub unsafe fn Samus_Move_NoSpeedCalc_Y() {
    let mut amt: int32 = ((*((&raw mut g_ram as *mut uint8)
        .offset(0xb5c as ::core::ffi::c_int as isize) as *mut uint16)
        as uint32)
        << 16 as ::core::ffi::c_int
        | *((&raw mut g_ram as *mut uint8).offset(0xb5a as ::core::ffi::c_int as isize)
            as *mut uint16) as uint32) as int32;
    if amt == 0 as int32 {
        if *((&raw mut g_ram as *mut uint8).offset(0xdba as ::core::ffi::c_int as isize)
            as *mut uint16)
            == 0
        {
            amt = ((*((&raw mut g_ram as *mut uint8).offset(0xdbc as ::core::ffi::c_int as isize)
                as *mut uint16) as uint32)
                << 16 as ::core::ffi::c_int
                | *((&raw mut g_ram as *mut uint8).offset(0xdbe as ::core::ffi::c_int as isize)
                    as *mut uint16) as uint32) as int32;
        }
        Samus_MoveDown(
            amt + ((1 as ::core::ffi::c_int as int16 as int32) << 16 as ::core::ffi::c_int),
        );
    } else if amt < 0 as int32 {
        Samus_MoveUp(amt);
    } else {
        Samus_MoveDown(
            amt + ((1 as ::core::ffi::c_int as int16 as int32) << 16 as ::core::ffi::c_int),
        );
    };
}
pub unsafe fn Samus_MoveExtraY() {
    let mut amt: int32 = ((*((&raw mut g_ram as *mut uint8)
        .offset(0xb5c as ::core::ffi::c_int as isize) as *mut uint16)
        as uint32)
        << 16 as ::core::ffi::c_int
        | *((&raw mut g_ram as *mut uint8).offset(0xb5a as ::core::ffi::c_int as isize)
            as *mut uint16) as uint32) as int32;
    if amt != 0 as int32 {
        if amt < 0 as int32 {
            Samus_MoveUp(amt);
        } else {
            Samus_MoveDown(
                amt + ((1 as ::core::ffi::c_int as int16 as int32) << 16 as ::core::ffi::c_int),
            );
        }
    }
}
pub unsafe fn Samus_CheckAndMoveY() -> uint8 {
    if *((&raw mut g_ram as *mut uint8).offset(0xb36 as ::core::ffi::c_int as isize) as *mut uint16)
        == 0
    {
        return 0 as uint8;
    }
    Samus_CheckStartFalling();
    Samus_MoveY_WithSpeedCalc();
    return 1 as uint8;
}
pub unsafe fn Samus_MoveY_Simple_() -> uint8 {
    if *((&raw mut g_ram as *mut uint8).offset(0xb36 as ::core::ffi::c_int as isize) as *mut uint16)
        == 0
    {
        return 0 as uint8;
    }
    Samus_CheckStartFalling();
    Samus_MoveY_WithSpeedCalc();
    return 1 as uint8;
}
pub unsafe fn Samus_Move_NoBaseSpeed_X() {
    Samus_MoveX(0 as int32);
}
pub unsafe fn Samus_MoveLeft(mut amt: int32) {
    amt = -amt;
    let mut cres: CheckEnemyColl_Result = Samus_CheckSolidEnemyColl(amt);
    *((&raw mut g_ram as *mut uint8).offset(0xdd0 as ::core::ffi::c_int as isize) as *mut uint16) =
        cres.collision;
    amt = cres.amt;
    if *((&raw mut g_ram as *mut uint8).offset(0xdd0 as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        Samus_ClearXSpeedIfColl();
        Samus_MoveLeft_NoColl(amt);
        Samus_AlignYPosSlope();
    } else {
        amt = Samus_MoveRight_NoSolidColl(-amt);
        SetHiLo(
            (&raw mut g_ram as *mut uint8).offset(0xdaa as ::core::ffi::c_int as isize)
                as *mut uint16,
            (&raw mut g_ram as *mut uint8).offset(0xdac as ::core::ffi::c_int as isize)
                as *mut uint16,
            amt as uint32,
        );
        if *((&raw mut g_ram as *mut uint8).offset(0xb02 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            *((&raw mut g_ram as *mut uint8).offset(0xdd0 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
        }
        Samus_ClearXSpeedIfColl();
        Samus_AlignYPosSlope();
    };
}
pub unsafe fn Samus_MoveRight(mut amt: int32) {
    let mut cres: CheckEnemyColl_Result = Samus_CheckSolidEnemyColl(amt);
    *((&raw mut g_ram as *mut uint8).offset(0xdd0 as ::core::ffi::c_int as isize) as *mut uint16) =
        cres.collision;
    amt = cres.amt;
    if *((&raw mut g_ram as *mut uint8).offset(0xdd0 as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        Samus_ClearXSpeedIfColl();
        Samus_MoveRight_NoColl(amt);
        Samus_AlignYPosSlope();
    } else {
        amt = Samus_MoveRight_NoSolidColl(amt);
        SetHiLo(
            (&raw mut g_ram as *mut uint8).offset(0xdae as ::core::ffi::c_int as isize)
                as *mut uint16,
            (&raw mut g_ram as *mut uint8).offset(0xdb0 as ::core::ffi::c_int as isize)
                as *mut uint16,
            amt as uint32,
        );
        if *((&raw mut g_ram as *mut uint8).offset(0xb02 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            *((&raw mut g_ram as *mut uint8).offset(0xdd0 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
        }
        Samus_ClearXSpeedIfColl();
        Samus_AlignYPosSlope();
    };
}
pub unsafe fn Samus_MoveUp(mut amt: int32) {
    *((&raw mut g_ram as *mut uint8).offset(0xb02 as ::core::ffi::c_int as isize) as *mut uint16) =
        2 as uint16;
    let mut cres: CheckEnemyColl_Result = Samus_CheckSolidEnemyColl(-amt);
    *((&raw mut g_ram as *mut uint8).offset(0xdd0 as ::core::ffi::c_int as isize) as *mut uint16) =
        cres.collision;
    amt = cres.amt;
    if *((&raw mut g_ram as *mut uint8).offset(0xdd0 as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        Samus_MoveUp_SetPoseCalcInput();
        Samus_MoveUp_NoColl(amt);
    } else {
        amt = Samus_MoveDown_NoSolidColl(-amt);
        SetHiLo(
            (&raw mut g_ram as *mut uint8).offset(0xdb2 as ::core::ffi::c_int as isize)
                as *mut uint16,
            (&raw mut g_ram as *mut uint8).offset(0xdb4 as ::core::ffi::c_int as isize)
                as *mut uint16,
            amt as uint32,
        );
        Samus_MoveUp_SetPoseCalcInput();
    };
}
pub unsafe fn Samus_MoveDown(mut amt: int32) {
    *((&raw mut g_ram as *mut uint8).offset(0xb02 as ::core::ffi::c_int as isize) as *mut uint16) =
        3 as uint16;
    let mut cres: CheckEnemyColl_Result = Samus_CheckSolidEnemyColl(amt);
    *((&raw mut g_ram as *mut uint8).offset(0xdd0 as ::core::ffi::c_int as isize) as *mut uint16) =
        cres.collision;
    amt = cres.amt;
    if *((&raw mut g_ram as *mut uint8).offset(0xdd0 as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        Samus_MoveDown_SetPoseCalcInput();
        Samus_MoveDown_NoColl(amt);
    } else {
        amt = Samus_MoveDown_NoSolidColl(amt);
        *((&raw mut g_ram as *mut uint8).offset(0xdb8 as ::core::ffi::c_int as isize)
            as *mut uint16) = amt as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xdb6 as ::core::ffi::c_int as isize)
            as *mut uint16) = (amt >> 16 as ::core::ffi::c_int) as uint16;
        Samus_MoveDown_SetPoseCalcInput();
    };
}
pub unsafe fn Samus_MoveHandler_ReleaseFromGrapple() {
    if *((&raw mut g_ram as *mut uint8).offset(0xb36 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        == 1 as ::core::ffi::c_int
        && *((&raw mut g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb36 as ::core::ffi::c_int as isize)
            as *mut uint16) = 2 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xa58 as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x90a337 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    *((&raw mut g_ram as *mut uint8).offset(0xb4a as ::core::ffi::c_int as isize) as *mut uint16) =
        2 as uint16;
    let mut amt: int32 = Samus_CalcBaseSpeed_X(Samus_DetermineGrappleSwingSpeed_X());
    if *((&raw mut g_ram as *mut uint8).offset(0xb4a as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        != 0
        || *((&raw mut g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & kButton_Right as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        || *((&raw mut g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & kButton_Left as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
    {
        Samus_MoveX(amt);
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0xb46 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb48 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xdd0 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    }
    Samus_MoveY_WithSpeedCalc();
    if *((&raw mut g_ram as *mut uint8).offset(0xdc6 as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        *((&raw mut g_ram as *mut uint8).offset(0xa58 as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x90a337 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Samus_HandleMovement_DrainedCrouching() {
    Samus_MoveY_WithSpeedCalc();
    if *((&raw mut g_ram as *mut uint8).offset(0xdd0 as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        *((&raw mut g_ram as *mut uint8).offset(0xa58 as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x90a337 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xa94 as ::core::ffi::c_int as isize)
            as *mut uint16) = 8 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
            as *mut uint16) = 7 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    }
}
pub unsafe fn CallScrollingFinishedHook(mut ea: uint32) {
    match ea {
        9475465 => {
            Samus_ScrollFinishedHook_SporeSpawnFight();
            return;
        }
        _ => {
            Unreachable();
        }
    };
}
pub unsafe fn MainScrollingRoutine() {
    let mut v0: uint16 = 0;
    let mut current_block: u64;
    if *((&raw mut g_ram as *mut uint8).offset(0xcf8 as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        if !(*((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int)
        {
            v0 = 0;
            v0 = (*((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - *((&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint16;
            if (*((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int)
                < *((&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
            {
                current_block = 8565863539623487812;
            } else if v0 as ::core::ffi::c_int >= 0xa0 as ::core::ffi::c_int {
                let ref mut fresh144 = *((&raw mut g_ram as *mut uint8)
                    .offset(0x911 as ::core::ffi::c_int as isize)
                    as *mut uint16);
                *fresh144 = (*fresh144 as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as uint16;
                current_block = 9616406889445156080;
            } else if (v0 as ::core::ffi::c_int) < 0x60 as ::core::ffi::c_int {
                current_block = 8565863539623487812;
            } else {
                current_block = 9616406889445156080;
            }
            match current_block {
                8565863539623487812 => {
                    let ref mut fresh145 = *((&raw mut g_ram as *mut uint8)
                        .offset(0x911 as ::core::ffi::c_int as isize)
                        as *mut uint16);
                    *fresh145 =
                        (*fresh145 as ::core::ffi::c_int - 3 as ::core::ffi::c_int) as uint16;
                }
                _ => {}
            }
            if *((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            {
                let mut v1: uint16 = (*((&raw mut g_ram as *mut uint8)
                    .offset(0xafa as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    - *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int)
                    as uint16;
                if *((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    >= *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                {
                    if v1 as ::core::ffi::c_int >= 0x90 as ::core::ffi::c_int {
                        let ref mut fresh146 = *((&raw mut g_ram as *mut uint8)
                            .offset(0x915 as ::core::ffi::c_int as isize)
                            as *mut uint16);
                        *fresh146 =
                            (*fresh146 as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as uint16;
                        current_block = 13986801909406869559;
                    } else if v1 as ::core::ffi::c_int >= 0x70 as ::core::ffi::c_int {
                        current_block = 13986801909406869559;
                    } else {
                        current_block = 1856101646708284338;
                    }
                } else {
                    current_block = 1856101646708284338;
                }
                match current_block {
                    13986801909406869559 => {}
                    _ => {
                        let ref mut fresh147 = *((&raw mut g_ram as *mut uint8)
                            .offset(0x915 as ::core::ffi::c_int as isize)
                            as *mut uint16);
                        *fresh147 =
                            (*fresh147 as ::core::ffi::c_int - 3 as ::core::ffi::c_int) as uint16;
                    }
                }
            }
        }
        HandleAutoscrolling_X();
        HandleAutoscrolling_Y();
    } else {
        Samus_CalcDistanceMoved_X();
        Samus_HandleScroll_X();
        Samus_CalcDistanceMoved_Y();
        Samus_HandleScroll_Y();
    }
    if *((&raw mut g_ram as *mut uint8).offset(0x7e9 as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        CallScrollingFinishedHook(
            (*((&raw mut g_ram as *mut uint8).offset(0x7e9 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                | 0x900000 as ::core::ffi::c_int) as uint32,
        );
    }
    *((&raw mut g_ram as *mut uint8).offset(0xb10 as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0xb12 as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0xaf8 as ::core::ffi::c_int as isize)
            as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0xb14 as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0xb16 as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0xafc as ::core::ffi::c_int as isize)
            as *mut uint16);
}
pub unsafe fn Samus_ScrollFinishedHook_SporeSpawnFight() {
    if *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        <= 0x1d0 as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16) = 464 as uint16;
    }
}
pub unsafe fn Samus_HandleScroll_X() {
    static mut kSamus_HandleScroll_X_FaceLeft: [uint16; 4] = [
        0xa0 as ::core::ffi::c_int as uint16,
        0x50 as ::core::ffi::c_int as uint16,
        0x20 as ::core::ffi::c_int as uint16,
        0xe0 as ::core::ffi::c_int as uint16,
    ];
    static mut kSamus_HandleScroll_X_FaceRight: [uint16; 4] = [
        0x60 as ::core::ffi::c_int as uint16,
        0x40 as ::core::ffi::c_int as uint16,
        0x20 as ::core::ffi::c_int as uint16,
        0xe0 as ::core::ffi::c_int as uint16,
    ];
    if *((&raw mut g_ram as *mut uint8).offset(0xb10 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        == *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
    {
        HandleAutoscrolling_X();
        return;
    }
    if (*((&raw mut g_ram as *mut uint8).offset(0xa52 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        != 0
        || *(&raw mut g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            == 16 as ::core::ffi::c_int
        || *((&raw mut g_ram as *mut uint8).offset(0xb4a as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == 1 as ::core::ffi::c_int) as ::core::ffi::c_int
        ^ (*(&raw mut g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            != 4 as ::core::ffi::c_int) as ::core::ffi::c_int
        != 0
    {
        *((&raw mut g_ram as *mut uint8).offset(0xb0a as ::core::ffi::c_int as isize)
            as *mut uint16) = (*((&raw mut g_ram as *mut uint8)
            .offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - kSamus_HandleScroll_X_FaceRight[(*((&raw mut g_ram as *mut uint8)
                .offset(0x941 as ::core::ffi::c_int as isize)
                as *mut uint16)
                as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as usize] as ::core::ffi::c_int)
            as uint16;
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0xb0a as ::core::ffi::c_int as isize)
            as *mut uint16) = (*((&raw mut g_ram as *mut uint8)
            .offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - kSamus_HandleScroll_X_FaceLeft[(*((&raw mut g_ram as *mut uint8)
                .offset(0x941 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as usize] as ::core::ffi::c_int)
            as uint16;
    }
    if *((&raw mut g_ram as *mut uint8).offset(0xb0a as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        != *((&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
    {
        if ((*((&raw mut g_ram as *mut uint8).offset(0xb0a as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        {
            AddToHiLo(
                (&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                    as *mut uint16,
                (&raw mut g_ram as *mut uint8).offset(0x90f as ::core::ffi::c_int as isize)
                    as *mut uint16,
                -(((*((&raw mut g_ram as *mut uint8).offset(0xda2 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int)
                    << 16 as ::core::ffi::c_int
                    | *((&raw mut g_ram as *mut uint8).offset(0xda4 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int) as int32) as uint32,
            );
            HandleScrollingWhenTriggeringScrollLeft();
        } else {
            AddToHiLo(
                (&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                    as *mut uint16,
                (&raw mut g_ram as *mut uint8).offset(0x90f as ::core::ffi::c_int as isize)
                    as *mut uint16,
                (*((&raw mut g_ram as *mut uint8).offset(0xda2 as ::core::ffi::c_int as isize)
                    as *mut uint16) as uint32)
                    << 16 as ::core::ffi::c_int
                    | *((&raw mut g_ram as *mut uint8).offset(0xda4 as ::core::ffi::c_int as isize)
                        as *mut uint16) as uint32,
            );
            HandleScrollingWhenTriggeringScrollRight();
        }
    }
}
pub unsafe fn Samus_HandleScroll_Y() {
    if *((&raw mut g_ram as *mut uint8).offset(0xb14 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        == *((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
    {
        HandleAutoscrolling_Y();
    } else {
        if *((&raw mut g_ram as *mut uint8).offset(0xb36 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == 1 as ::core::ffi::c_int
        {
            *((&raw mut g_ram as *mut uint8).offset(0xb0e as ::core::ffi::c_int as isize)
                as *mut uint16) =
                (*((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    - *((&raw mut g_ram as *mut uint8).offset(0x7af as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int) as uint16;
        } else {
            *((&raw mut g_ram as *mut uint8).offset(0xb0e as ::core::ffi::c_int as isize)
                as *mut uint16) =
                (*((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    - *((&raw mut g_ram as *mut uint8).offset(0x7ad as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int) as uint16;
        }
        if *((&raw mut g_ram as *mut uint8).offset(0xb0e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
        {
            if ((*((&raw mut g_ram as *mut uint8).offset(0xb0e as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as int16
                as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int
            {
                AddToHiLo(
                    (&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                        as *mut uint16,
                    (&raw mut g_ram as *mut uint8).offset(0x913 as ::core::ffi::c_int as isize)
                        as *mut uint16,
                    -(((*((&raw mut g_ram as *mut uint8)
                        .offset(0xda6 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int)
                        << 16 as ::core::ffi::c_int
                        | *((&raw mut g_ram as *mut uint8)
                            .offset(0xda8 as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int) as int32)
                        as uint32,
                );
                HandleScrollingWhenTriggeringScrollUp();
            } else {
                AddToHiLo(
                    (&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                        as *mut uint16,
                    (&raw mut g_ram as *mut uint8).offset(0x913 as ::core::ffi::c_int as isize)
                        as *mut uint16,
                    (*((&raw mut g_ram as *mut uint8).offset(0xda6 as ::core::ffi::c_int as isize)
                        as *mut uint16) as uint32)
                        << 16 as ::core::ffi::c_int
                        | *((&raw mut g_ram as *mut uint8)
                            .offset(0xda8 as ::core::ffi::c_int as isize)
                            as *mut uint16) as uint32,
                );
                HandleScrollingWhenTriggeringScrollDown();
            }
        }
    };
}
pub unsafe fn Samus_CalcDistanceMoved_X() {
    if (*((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0xb10 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        SetHiLo(
            (&raw mut g_ram as *mut uint8).offset(0xda2 as ::core::ffi::c_int as isize)
                as *mut uint16,
            (&raw mut g_ram as *mut uint8).offset(0xda4 as ::core::ffi::c_int as isize)
                as *mut uint16,
            ((*((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint32)
                << 16 as ::core::ffi::c_int
                | *((&raw mut g_ram as *mut uint8).offset(0xaf8 as ::core::ffi::c_int as isize)
                    as *mut uint16) as uint32)
                .wrapping_sub(
                    (*((&raw mut g_ram as *mut uint8).offset(0xb10 as ::core::ffi::c_int as isize)
                        as *mut uint16) as uint32)
                        << 16 as ::core::ffi::c_int
                        | *((&raw mut g_ram as *mut uint8)
                            .offset(0xb12 as ::core::ffi::c_int as isize)
                            as *mut uint16) as uint32,
                )
                .wrapping_add(((1 as ::core::ffi::c_int) << 16 as ::core::ffi::c_int) as uint32),
        );
    } else {
        SetHiLo(
            (&raw mut g_ram as *mut uint8).offset(0xda2 as ::core::ffi::c_int as isize)
                as *mut uint16,
            (&raw mut g_ram as *mut uint8).offset(0xda4 as ::core::ffi::c_int as isize)
                as *mut uint16,
            ((*((&raw mut g_ram as *mut uint8).offset(0xb10 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint32)
                << 16 as ::core::ffi::c_int
                | *((&raw mut g_ram as *mut uint8).offset(0xb12 as ::core::ffi::c_int as isize)
                    as *mut uint16) as uint32)
                .wrapping_sub(
                    (*((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                        as *mut uint16) as uint32)
                        << 16 as ::core::ffi::c_int
                        | *((&raw mut g_ram as *mut uint8)
                            .offset(0xaf8 as ::core::ffi::c_int as isize)
                            as *mut uint16) as uint32,
                )
                .wrapping_add(((1 as ::core::ffi::c_int) << 16 as ::core::ffi::c_int) as uint32),
        );
    };
}
pub unsafe fn Samus_CalcDistanceMoved_Y() {
    if (*((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0xb14 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        SetHiLo(
            (&raw mut g_ram as *mut uint8).offset(0xda6 as ::core::ffi::c_int as isize)
                as *mut uint16,
            (&raw mut g_ram as *mut uint8).offset(0xda8 as ::core::ffi::c_int as isize)
                as *mut uint16,
            ((*((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
                as *mut uint16) as uint32)
                << 16 as ::core::ffi::c_int
                | *((&raw mut g_ram as *mut uint8).offset(0xafc as ::core::ffi::c_int as isize)
                    as *mut uint16) as uint32)
                .wrapping_sub(
                    (*((&raw mut g_ram as *mut uint8).offset(0xb14 as ::core::ffi::c_int as isize)
                        as *mut uint16) as uint32)
                        << 16 as ::core::ffi::c_int
                        | *((&raw mut g_ram as *mut uint8)
                            .offset(0xb16 as ::core::ffi::c_int as isize)
                            as *mut uint16) as uint32,
                )
                .wrapping_add(((1 as ::core::ffi::c_int) << 16 as ::core::ffi::c_int) as uint32),
        );
    } else {
        SetHiLo(
            (&raw mut g_ram as *mut uint8).offset(0xda6 as ::core::ffi::c_int as isize)
                as *mut uint16,
            (&raw mut g_ram as *mut uint8).offset(0xda8 as ::core::ffi::c_int as isize)
                as *mut uint16,
            ((*((&raw mut g_ram as *mut uint8).offset(0xb14 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint32)
                << 16 as ::core::ffi::c_int
                | *((&raw mut g_ram as *mut uint8).offset(0xb16 as ::core::ffi::c_int as isize)
                    as *mut uint16) as uint32)
                .wrapping_sub(
                    (*((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
                        as *mut uint16) as uint32)
                        << 16 as ::core::ffi::c_int
                        | *((&raw mut g_ram as *mut uint8)
                            .offset(0xafc as ::core::ffi::c_int as isize)
                            as *mut uint16) as uint32,
                )
                .wrapping_add(((1 as ::core::ffi::c_int) << 16 as ::core::ffi::c_int) as uint32),
        );
    };
}
static mut kSamus_HandleExtraRunspeedX_Tab0: [uint16; 3] = [
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
];
static mut kSamus_HandleExtraRunspeedX_Tab1: [uint16; 3] = [
    0x1000 as ::core::ffi::c_int as uint16,
    0x400 as ::core::ffi::c_int as uint16,
    0x400 as ::core::ffi::c_int as uint16,
];
static mut kSamus_HandleExtraRunspeedX_Tab2: [uint16; 3] = [
    7 as ::core::ffi::c_int as uint16,
    4 as ::core::ffi::c_int as uint16,
    4 as ::core::ffi::c_int as uint16,
];
static mut kSamus_HandleExtraRunspeedX_Tab3: [uint16; 3] = [
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
];
static mut kSamus_HandleExtraRunspeedX_Tab4: [uint16; 3] = [
    2 as ::core::ffi::c_int as uint16,
    1 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
];
static mut kSamus_HandleExtraRunspeedX_Tab5: [uint16; 3] = [
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
];
pub unsafe fn Samus_HandleExtraRunspeedX() {
    let mut r18: uint16 = 0;
    let mut current_block: u64;
    if *((&raw mut g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0x20 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        r18 = Samus_GetBottom_R18();
        if *((&raw mut g_ram as *mut uint8).offset(0x195e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            if *((&raw mut g_ram as *mut uint8).offset(0x1962 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
                && *((&raw mut g_ram as *mut uint8).offset(0x1962 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    - r18 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    != 0
            {
                current_block = 3594490907146900115;
            } else {
                current_block = 13183875560443969876;
            }
        } else if *((&raw mut g_ram as *mut uint8).offset(0x195e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - r18 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
            && *((&raw mut g_ram as *mut uint8).offset(0x197e as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 4 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
        {
            current_block = 3594490907146900115;
        } else {
            current_block = 13183875560443969876;
        }
    } else {
        current_block = 13183875560443969876;
    }
    match current_block {
        13183875560443969876 => {
            if *(&raw mut g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
                != 1 as ::core::ffi::c_int
                || *((&raw mut g_ram as *mut uint8).offset(0x9b6 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    & *((&raw mut g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                    == 0 as ::core::ffi::c_int
            {
                current_block = 3594490907146900115;
            } else {
                if *((&raw mut g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    & 0x2000 as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int
                {
                    if *((&raw mut g_ram as *mut uint8).offset(0xb3c as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        == 0
                    {
                        *((&raw mut g_ram as *mut uint8)
                            .offset(0xb3c as ::core::ffi::c_int as isize)
                            as *mut uint16) = 1 as uint16;
                        *((&raw mut g_ram as *mut uint8)
                            .offset(0xad0 as ::core::ffi::c_int as isize)
                            as *mut uint16) = 1 as uint16;
                        *((&raw mut g_ram as *mut uint8)
                            .offset(0xace as ::core::ffi::c_int as isize)
                            as *mut uint16) = 0 as uint16;
                        *((&raw mut g_ram as *mut uint8)
                            .offset(0xb3e as ::core::ffi::c_int as isize)
                            as *mut uint16) = *(g_rom.offset(
                            (((0x91b61f as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                                << 15 as ::core::ffi::c_int
                                | 0x91b61f as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                                & 0x3fffff as ::core::ffi::c_int)
                                as isize,
                        ) as *const uint8
                            as *mut uint16)
                            .offset(0 as ::core::ffi::c_int as isize);
                    }
                    if (*((&raw mut g_ram as *mut uint8)
                        .offset(0xb42 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        - kSamus_HandleExtraRunspeedX_Tab2[0 as ::core::ffi::c_int as usize]
                            as ::core::ffi::c_int) as int16
                        as ::core::ffi::c_int
                        >= 0 as ::core::ffi::c_int
                        && (*((&raw mut g_ram as *mut uint8)
                            .offset(0xb44 as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int
                            - kSamus_HandleExtraRunspeedX_Tab3[0 as ::core::ffi::c_int as usize]
                                as ::core::ffi::c_int) as int16
                            as ::core::ffi::c_int
                            >= 0 as ::core::ffi::c_int
                    {
                        *((&raw mut g_ram as *mut uint8)
                            .offset(0xb42 as ::core::ffi::c_int as isize)
                            as *mut uint16) =
                            kSamus_HandleExtraRunspeedX_Tab2[0 as ::core::ffi::c_int as usize];
                        *((&raw mut g_ram as *mut uint8)
                            .offset(0xb44 as ::core::ffi::c_int as isize)
                            as *mut uint16) =
                            kSamus_HandleExtraRunspeedX_Tab3[0 as ::core::ffi::c_int as usize];
                        current_block = 7368539786447134940;
                    } else {
                        current_block = 15768484401365413375;
                    }
                } else {
                    if *((&raw mut g_ram as *mut uint8).offset(0xb3c as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        == 0
                    {
                        *((&raw mut g_ram as *mut uint8)
                            .offset(0xb3c as ::core::ffi::c_int as isize)
                            as *mut uint16) = 1 as uint16;
                        *((&raw mut g_ram as *mut uint8)
                            .offset(0xb3e as ::core::ffi::c_int as isize)
                            as *mut uint16) = 0 as uint16;
                    }
                    if (*((&raw mut g_ram as *mut uint8)
                        .offset(0xb42 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        - kSamus_HandleExtraRunspeedX_Tab4[0 as ::core::ffi::c_int as usize]
                            as ::core::ffi::c_int) as int16
                        as ::core::ffi::c_int
                        >= 0 as ::core::ffi::c_int
                        && (*((&raw mut g_ram as *mut uint8)
                            .offset(0xb44 as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int
                            - kSamus_HandleExtraRunspeedX_Tab5[0 as ::core::ffi::c_int as usize]
                                as ::core::ffi::c_int) as int16
                            as ::core::ffi::c_int
                            >= 0 as ::core::ffi::c_int
                    {
                        *((&raw mut g_ram as *mut uint8)
                            .offset(0xb42 as ::core::ffi::c_int as isize)
                            as *mut uint16) =
                            kSamus_HandleExtraRunspeedX_Tab4[0 as ::core::ffi::c_int as usize];
                        *((&raw mut g_ram as *mut uint8)
                            .offset(0xb44 as ::core::ffi::c_int as isize)
                            as *mut uint16) =
                            kSamus_HandleExtraRunspeedX_Tab5[0 as ::core::ffi::c_int as usize];
                        current_block = 7368539786447134940;
                    } else {
                        current_block = 15768484401365413375;
                    }
                }
                match current_block {
                    7368539786447134940 => {}
                    _ => {
                        AddToHiLo(
                            (&raw mut g_ram as *mut uint8)
                                .offset(0xb42 as ::core::ffi::c_int as isize)
                                as *mut uint16,
                            (&raw mut g_ram as *mut uint8)
                                .offset(0xb44 as ::core::ffi::c_int as isize)
                                as *mut uint16,
                            (kSamus_HandleExtraRunspeedX_Tab0[0 as ::core::ffi::c_int as usize]
                                as uint32)
                                << 16 as ::core::ffi::c_int
                                | kSamus_HandleExtraRunspeedX_Tab1[0 as ::core::ffi::c_int as usize]
                                    as uint32,
                        );
                        current_block = 7368539786447134940;
                    }
                }
            }
        }
        _ => {}
    }
    match current_block {
        3594490907146900115 => {
            if *((&raw mut g_ram as *mut uint8).offset(0xb3c as ::core::ffi::c_int as isize)
                as *mut uint16)
                == 0
            {
                *((&raw mut g_ram as *mut uint8).offset(0xb42 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 0 as uint16;
                *((&raw mut g_ram as *mut uint8).offset(0xb44 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 0 as uint16;
            }
        }
        _ => {}
    }
    if *((&raw mut g_ram as *mut uint8).offset(0xb3e as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0xff00 as ::core::ffi::c_int
        == 1024 as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0xa6e as ::core::ffi::c_int as isize)
            as *mut uint16) = 1 as uint16;
    }
}
pub unsafe fn Samus_MoveRight_NoColl(mut amt: int32) {
    AddToHiLo(
        (&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize) as *mut uint16,
        (&raw mut g_ram as *mut uint8).offset(0xaf8 as ::core::ffi::c_int as isize) as *mut uint16,
        amt as uint32,
    );
    SetHiLo(
        (&raw mut g_ram as *mut uint8).offset(0xdae as ::core::ffi::c_int as isize) as *mut uint16,
        (&raw mut g_ram as *mut uint8).offset(0xdb0 as ::core::ffi::c_int as isize) as *mut uint16,
        amt as uint32,
    );
}
pub unsafe fn Samus_MoveLeft_NoColl(mut amt: int32) {
    AddToHiLo(
        (&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize) as *mut uint16,
        (&raw mut g_ram as *mut uint8).offset(0xaf8 as ::core::ffi::c_int as isize) as *mut uint16,
        -amt as uint32,
    );
    SetHiLo(
        (&raw mut g_ram as *mut uint8).offset(0xdaa as ::core::ffi::c_int as isize) as *mut uint16,
        (&raw mut g_ram as *mut uint8).offset(0xdac as ::core::ffi::c_int as isize) as *mut uint16,
        -amt as uint32,
    );
}
pub unsafe fn Samus_MoveDown_NoColl(mut amt: int32) {
    AddToHiLo(
        (&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize) as *mut uint16,
        (&raw mut g_ram as *mut uint8).offset(0xafc as ::core::ffi::c_int as isize) as *mut uint16,
        amt as uint32,
    );
    SetHiLo(
        (&raw mut g_ram as *mut uint8).offset(0xdb6 as ::core::ffi::c_int as isize) as *mut uint16,
        (&raw mut g_ram as *mut uint8).offset(0xdb8 as ::core::ffi::c_int as isize) as *mut uint16,
        amt as uint32,
    );
}
pub unsafe fn Samus_MoveUp_NoColl(mut amt: int32) {
    AddToHiLo(
        (&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize) as *mut uint16,
        (&raw mut g_ram as *mut uint8).offset(0xafc as ::core::ffi::c_int as isize) as *mut uint16,
        -amt as uint32,
    );
    SetHiLo(
        (&raw mut g_ram as *mut uint8).offset(0xdb2 as ::core::ffi::c_int as isize) as *mut uint16,
        (&raw mut g_ram as *mut uint8).offset(0xdb4 as ::core::ffi::c_int as isize) as *mut uint16,
        -amt as uint32,
    );
}
pub unsafe fn Samus_InitJump() {
    let mut r18: uint16 = 0;
    let mut current_block: u64;
    static mut kSamus_InitJump_Speed_Y: [uint16; 3] = [
        4 as ::core::ffi::c_int as uint16,
        1 as ::core::ffi::c_int as uint16,
        2 as ::core::ffi::c_int as uint16,
    ];
    static mut kSamus_InitJump_Subspeed_Y: [uint16; 3] = [
        0xe000 as ::core::ffi::c_int as uint16,
        0xc000 as ::core::ffi::c_int as uint16,
        0xc000 as ::core::ffi::c_int as uint16,
    ];
    static mut kSamus_InitJump_Tab2: [uint16; 3] = [
        6 as ::core::ffi::c_int as uint16,
        2 as ::core::ffi::c_int as uint16,
        3 as ::core::ffi::c_int as uint16,
    ];
    static mut kSamus_InitJump_Tab3: [uint16; 3] = [
        0 as ::core::ffi::c_int as uint16,
        0x8000 as ::core::ffi::c_int as uint16,
        0x8000 as ::core::ffi::c_int as uint16,
    ];
    let mut v0: uint16 = 0;
    if *((&raw mut g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0x20 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        current_block = 18099320997399164961;
    } else {
        r18 = Samus_GetBottom_R18();
        if *((&raw mut g_ram as *mut uint8).offset(0x195e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            if *((&raw mut g_ram as *mut uint8).offset(0x195e as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - r18 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
                && *((&raw mut g_ram as *mut uint8).offset(0x197e as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    & 4 as ::core::ffi::c_int
                    == 0 as ::core::ffi::c_int
            {
                v0 = 2 as uint16;
                current_block = 1917311967535052937;
            } else {
                current_block = 18099320997399164961;
            }
        } else if *((&raw mut g_ram as *mut uint8).offset(0x1962 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
            || *((&raw mut g_ram as *mut uint8).offset(0x1962 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - r18 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
        {
            current_block = 18099320997399164961;
        } else {
            v0 = 4 as uint16;
            current_block = 1917311967535052937;
        }
    }
    match current_block {
        18099320997399164961 => {
            v0 = 0 as uint16;
        }
        _ => {}
    }
    let mut v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0x100 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize)
            as *mut uint16) = kSamus_InitJump_Tab3[v1 as usize];
        *((&raw mut g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
            as *mut uint16) = kSamus_InitJump_Tab2[v1 as usize];
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize)
            as *mut uint16) = kSamus_InitJump_Subspeed_Y[v1 as usize];
        *((&raw mut g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
            as *mut uint16) = kSamus_InitJump_Speed_Y[v1 as usize];
    }
    if *((&raw mut g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0x2000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        let ref mut fresh125 = *((&raw mut g_ram as *mut uint8)
            .offset(0xb2c as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh125 = (*fresh125 as ::core::ffi::c_int
            + *((&raw mut g_ram as *mut uint8).offset(0xb44 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16;
        let ref mut fresh126 = *((&raw mut g_ram as *mut uint8)
            .offset(0xb2e as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh126 = (*fresh126 as ::core::ffi::c_int
            + (*((&raw mut g_ram as *mut uint8).offset(0xb42 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int)) as uint16;
    }
    *((&raw mut g_ram as *mut uint8).offset(0xa9e as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xaa0 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb36 as ::core::ffi::c_int as isize) as *mut uint16) =
        1 as uint16;
}
pub unsafe fn Samus_InitWallJump() {
    let mut r18: uint16 = 0;
    let mut current_block: u64;
    static mut kSamus_InitWallJump_0: [uint16; 3] = [
        4 as ::core::ffi::c_int as uint16,
        0 as ::core::ffi::c_int as uint16,
        2 as ::core::ffi::c_int as uint16,
    ];
    static mut kSamus_InitWallJump_1: [uint16; 3] = [
        0xa000 as ::core::ffi::c_int as uint16,
        0x4000 as ::core::ffi::c_int as uint16,
        0xa000 as ::core::ffi::c_int as uint16,
    ];
    static mut kSamus_InitWallJump_2: [uint16; 3] = [
        5 as ::core::ffi::c_int as uint16,
        0 as ::core::ffi::c_int as uint16,
        3 as ::core::ffi::c_int as uint16,
    ];
    static mut kSamus_InitWallJump_3: [uint16; 3] = [
        0x8000 as ::core::ffi::c_int as uint16,
        0x8000 as ::core::ffi::c_int as uint16,
        0x8000 as ::core::ffi::c_int as uint16,
    ];
    let mut v0: uint16 = 0;
    if *((&raw mut g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0x20 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        current_block = 16262899787512852811;
    } else {
        r18 = Samus_GetBottom_R18();
        if *((&raw mut g_ram as *mut uint8).offset(0x195e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            if *((&raw mut g_ram as *mut uint8).offset(0x195e as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - r18 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
                && *((&raw mut g_ram as *mut uint8).offset(0x197e as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    & 4 as ::core::ffi::c_int
                    == 0 as ::core::ffi::c_int
            {
                v0 = 2 as uint16;
                current_block = 1917311967535052937;
            } else {
                current_block = 16262899787512852811;
            }
        } else if *((&raw mut g_ram as *mut uint8).offset(0x1962 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
            || *((&raw mut g_ram as *mut uint8).offset(0x1962 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - r18 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
        {
            current_block = 16262899787512852811;
        } else {
            v0 = 4 as uint16;
            current_block = 1917311967535052937;
        }
    }
    match current_block {
        16262899787512852811 => {
            v0 = 0 as uint16;
        }
        _ => {}
    }
    let mut v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0x100 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize)
            as *mut uint16) = kSamus_InitWallJump_3[v1 as usize];
        *((&raw mut g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
            as *mut uint16) = kSamus_InitWallJump_2[v1 as usize];
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize)
            as *mut uint16) = kSamus_InitWallJump_1[v1 as usize];
        *((&raw mut g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
            as *mut uint16) = kSamus_InitWallJump_0[v1 as usize];
    }
    if *((&raw mut g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0x2000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        r18 = (*((&raw mut g_ram as *mut uint8).offset(0xb42 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int) as uint16;
        let ref mut fresh151 = *((&raw mut g_ram as *mut uint8)
            .offset(0xb2c as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh151 = (*fresh151 as ::core::ffi::c_int
            + *((&raw mut g_ram as *mut uint8).offset(0xb44 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16;
        let ref mut fresh152 = *((&raw mut g_ram as *mut uint8)
            .offset(0xb2e as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh152 = (*fresh152 as ::core::ffi::c_int
            + (*((&raw mut g_ram as *mut uint8).offset(0xb42 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int)) as uint16;
    }
    *((&raw mut g_ram as *mut uint8).offset(0xa9e as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xaa0 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb36 as ::core::ffi::c_int as isize) as *mut uint16) =
        1 as uint16;
}
static mut kSamus_SetSpeedForKnockback_Y_Speed: [uint16; 3] = [
    5 as ::core::ffi::c_int as uint16,
    2 as ::core::ffi::c_int as uint16,
    2 as ::core::ffi::c_int as uint16,
];
static mut kSamus_SetSpeedForKnockback_Y_Subspeed: [uint16; 3] = [
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
];
pub unsafe fn Samus_SetSpeedForKnockback_Y() {
    let mut current_block: u64;
    let mut v0: uint16 = 0;
    if *((&raw mut g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0x20 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        let mut r18: uint16 = Samus_GetBottom_R18();
        if *((&raw mut g_ram as *mut uint8).offset(0x195e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            if *((&raw mut g_ram as *mut uint8).offset(0x1962 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
                && *((&raw mut g_ram as *mut uint8).offset(0x1962 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    - r18 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    != 0
            {
                v0 = 4 as uint16;
                current_block = 3276175668257526147;
            } else {
                current_block = 7815301370352969686;
            }
        } else if *((&raw mut g_ram as *mut uint8).offset(0x195e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - r18 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
            && *((&raw mut g_ram as *mut uint8).offset(0x197e as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 4 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
        {
            v0 = 2 as uint16;
            current_block = 3276175668257526147;
        } else {
            current_block = 7815301370352969686;
        }
    } else {
        current_block = 7815301370352969686;
    }
    match current_block {
        7815301370352969686 => {
            v0 = 0 as uint16;
        }
        _ => {}
    }
    let mut v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize) as *mut uint16) =
        kSamus_SetSpeedForKnockback_Y_Subspeed[v1 as usize];
    *((&raw mut g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize) as *mut uint16) =
        kSamus_SetSpeedForKnockback_Y_Speed[v1 as usize];
    *((&raw mut g_ram as *mut uint8).offset(0xa9e as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xaa0 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb36 as ::core::ffi::c_int as isize) as *mut uint16) =
        1 as uint16;
}
pub unsafe fn Samus_InitBombJump() {
    let mut current_block: u64;
    static mut kSamus_InitBombJump_Speed: [uint16; 3] = [
        2 as ::core::ffi::c_int as uint16,
        0 as ::core::ffi::c_int as uint16,
        0 as ::core::ffi::c_int as uint16,
    ];
    static mut kSamus_InitBombJump_Subspeed: [uint16; 3] = [
        0xc000 as ::core::ffi::c_int as uint16,
        0x1000 as ::core::ffi::c_int as uint16,
        0x1000 as ::core::ffi::c_int as uint16,
    ];
    let mut v0: uint16 = 0;
    if *((&raw mut g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0x20 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        let mut r18: uint16 = Samus_GetBottom_R18();
        if *((&raw mut g_ram as *mut uint8).offset(0x195e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            if *((&raw mut g_ram as *mut uint8).offset(0x1962 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
                && *((&raw mut g_ram as *mut uint8).offset(0x1962 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    - r18 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    != 0
            {
                v0 = 4 as uint16;
                current_block = 11812396948646013369;
            } else {
                current_block = 17216689946888361452;
            }
        } else if *((&raw mut g_ram as *mut uint8).offset(0x195e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - r18 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
            && *((&raw mut g_ram as *mut uint8).offset(0x197e as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 4 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
        {
            v0 = 2 as uint16;
            current_block = 11812396948646013369;
        } else {
            current_block = 17216689946888361452;
        }
    } else {
        current_block = 17216689946888361452;
    }
    match current_block {
        17216689946888361452 => {
            v0 = 0 as uint16;
        }
        _ => {}
    }
    let mut v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize) as *mut uint16) =
        kSamus_InitBombJump_Subspeed[v1 as usize];
    *((&raw mut g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize) as *mut uint16) =
        kSamus_InitBombJump_Speed[v1 as usize];
    *((&raw mut g_ram as *mut uint8).offset(0xa9e as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xaa0 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb36 as ::core::ffi::c_int as isize) as *mut uint16) =
        1 as uint16;
}
unsafe fn IsGreaterThanQuirked(
    mut vhi: uint16,
    mut vlo: uint16,
    mut cmphi: uint16,
    mut cmplo: uint16,
) -> bool {
    if (vhi as ::core::ffi::c_int - cmphi as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        if vhi as ::core::ffi::c_int != cmphi as ::core::ffi::c_int
            || (vlo as ::core::ffi::c_int - cmplo as ::core::ffi::c_int) as int16
                as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int
                && vlo as ::core::ffi::c_int != cmplo as ::core::ffi::c_int
        {
            return true_0 != 0;
        }
    }
    return false_0 != 0;
}
pub unsafe fn Samus_CalcBaseSpeed_X(mut k: uint16) -> int32 {
    let mut sste: *mut SamusSpeedTableEntry = get_SamusSpeedTableEntry(k);
    if *((&raw mut g_ram as *mut uint8).offset(0xb4a as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        let mut delta: int32 = (if *(&raw mut g_ram as *mut uint8)
            .offset(0xb4c as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            != 0
        {
            ((Mult8x8(
                *(&raw mut g_ram as *mut uint8).offset(0xb4c as ::core::ffi::c_int as isize),
                (*sste).decel as uint8,
            ) as ::core::ffi::c_int
                >> 8 as ::core::ffi::c_int) as uint32)
                << 16 as ::core::ffi::c_int
                | Mult8x8(
                    *(&raw mut g_ram as *mut uint8).offset(0xb4c as ::core::ffi::c_int as isize),
                    *(&raw mut (*sste).decel_sub as *mut uint8).offset(
                        (::core::mem::size_of::<uint16>() as usize)
                            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                            .wrapping_sub(1 as usize) as isize,
                    ),
                ) as uint32
        } else {
            ((*sste).decel as uint32) << 16 as ::core::ffi::c_int | (*sste).decel_sub as uint32
        }) as int32;
        AddToHiLo(
            (&raw mut g_ram as *mut uint8).offset(0xb46 as ::core::ffi::c_int as isize)
                as *mut uint16,
            (&raw mut g_ram as *mut uint8).offset(0xb48 as ::core::ffi::c_int as isize)
                as *mut uint16,
            -delta as uint32,
        );
        if (*((&raw mut g_ram as *mut uint8).offset(0xb46 as ::core::ffi::c_int as isize)
            as *mut uint16) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        {
            *((&raw mut g_ram as *mut uint8).offset(0xb46 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xb48 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xb4a as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
        }
    } else {
        AddToHiLo(
            (&raw mut g_ram as *mut uint8).offset(0xb46 as ::core::ffi::c_int as isize)
                as *mut uint16,
            (&raw mut g_ram as *mut uint8).offset(0xb48 as ::core::ffi::c_int as isize)
                as *mut uint16,
            ((*sste).accel as uint32) << 16 as ::core::ffi::c_int | (*sste).accel_sub as uint32,
        );
        if IsGreaterThanQuirked(
            *((&raw mut g_ram as *mut uint8).offset(0xb46 as ::core::ffi::c_int as isize)
                as *mut uint16),
            *((&raw mut g_ram as *mut uint8).offset(0xb48 as ::core::ffi::c_int as isize)
                as *mut uint16),
            (*sste).max_speed,
            (*sste).max_speed_sub,
        ) {
            *((&raw mut g_ram as *mut uint8).offset(0xb46 as ::core::ffi::c_int as isize)
                as *mut uint16) = (*sste).max_speed;
            *((&raw mut g_ram as *mut uint8).offset(0xb48 as ::core::ffi::c_int as isize)
                as *mut uint16) = (*sste).max_speed_sub;
        }
    }
    return ((*((&raw mut g_ram as *mut uint8).offset(0xb46 as ::core::ffi::c_int as isize)
        as *mut uint16) as uint32)
        << 16 as ::core::ffi::c_int
        | *((&raw mut g_ram as *mut uint8).offset(0xb48 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint32) as int32;
}
unsafe fn Samus_CalcBaseSpeed_NoDecel_X(mut k: uint16) -> Pair_Bool_Amt {
    let mut sste: *mut SamusSpeedTableEntry = get_SamusSpeedTableEntry(k);
    let mut rv: bool = false_0 != 0;
    if *((&raw mut g_ram as *mut uint8).offset(0xb4a as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        let mut delta: int32 = (if *(&raw mut g_ram as *mut uint8)
            .offset(0xb4c as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            != 0
        {
            ((Mult8x8(
                *(&raw mut g_ram as *mut uint8).offset(0xb4c as ::core::ffi::c_int as isize),
                (*sste).decel as uint8,
            ) as ::core::ffi::c_int
                >> 8 as ::core::ffi::c_int) as uint32)
                << 16 as ::core::ffi::c_int
                | Mult8x8(
                    *(&raw mut g_ram as *mut uint8).offset(0xb4c as ::core::ffi::c_int as isize),
                    *(&raw mut (*sste).decel_sub as *mut uint8).offset(
                        (::core::mem::size_of::<uint16>() as usize)
                            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                            .wrapping_sub(1 as usize) as isize,
                    ),
                ) as uint32
        } else {
            ((*sste).decel as uint32) << 16 as ::core::ffi::c_int | (*sste).decel_sub as uint32
        }) as int32;
        AddToHiLo(
            (&raw mut g_ram as *mut uint8).offset(0xb46 as ::core::ffi::c_int as isize)
                as *mut uint16,
            (&raw mut g_ram as *mut uint8).offset(0xb48 as ::core::ffi::c_int as isize)
                as *mut uint16,
            -delta as uint32,
        );
        if (*((&raw mut g_ram as *mut uint8).offset(0xb46 as ::core::ffi::c_int as isize)
            as *mut uint16) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        {
            *((&raw mut g_ram as *mut uint8).offset(0xb46 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xb48 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xb4a as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
        }
    } else {
        AddToHiLo(
            (&raw mut g_ram as *mut uint8).offset(0xb46 as ::core::ffi::c_int as isize)
                as *mut uint16,
            (&raw mut g_ram as *mut uint8).offset(0xb48 as ::core::ffi::c_int as isize)
                as *mut uint16,
            ((*sste).accel as uint32) << 16 as ::core::ffi::c_int | (*sste).accel_sub as uint32,
        );
        if IsGreaterThanQuirked(
            *((&raw mut g_ram as *mut uint8).offset(0xb46 as ::core::ffi::c_int as isize)
                as *mut uint16),
            *((&raw mut g_ram as *mut uint8).offset(0xb48 as ::core::ffi::c_int as isize)
                as *mut uint16),
            (*sste).max_speed,
            (*sste).max_speed_sub,
        ) {
            *((&raw mut g_ram as *mut uint8).offset(0xb46 as ::core::ffi::c_int as isize)
                as *mut uint16) = (*sste).max_speed;
            *((&raw mut g_ram as *mut uint8).offset(0xb48 as ::core::ffi::c_int as isize)
                as *mut uint16) = (*sste).max_speed_sub;
            rv = true_0 != 0;
        }
    }
    return Pair_Bool_Amt {
        flag: rv,
        amt: ((*((&raw mut g_ram as *mut uint8).offset(0xb46 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint32)
            << 16 as ::core::ffi::c_int
            | *((&raw mut g_ram as *mut uint8).offset(0xb48 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint32) as int32,
    };
}
pub unsafe fn Samus_DetermineSpeedTableEntryPtr_X() -> uint16 {
    if *((&raw mut g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0x20 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        let mut r18: uint16 = Samus_GetBottom_R18();
        if *((&raw mut g_ram as *mut uint8).offset(0x195e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            if *((&raw mut g_ram as *mut uint8).offset(0x1962 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
                && *((&raw mut g_ram as *mut uint8).offset(0x1962 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    - r18 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    != 0
            {
                *((&raw mut g_ram as *mut uint8).offset(0xa6c as ::core::ffi::c_int as isize)
                    as *mut uint16) =
                    addr_kSamusSpeedTable_LavaAcid_X as ::core::ffi::c_int as uint16;
            }
        } else if *((&raw mut g_ram as *mut uint8).offset(0x195e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - r18 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
            && *((&raw mut g_ram as *mut uint8).offset(0x197e as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 4 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
        {
            *((&raw mut g_ram as *mut uint8).offset(0xa6c as ::core::ffi::c_int as isize)
                as *mut uint16) = addr_kSamusSpeedTable_Water_X as ::core::ffi::c_int as uint16;
        }
    }
    return (*((&raw mut g_ram as *mut uint8).offset(0xa6c as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        + 12 as ::core::ffi::c_int
            * *(&raw mut g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Samus_DetermineGrappleSwingSpeed_X() -> uint16 {
    if *((&raw mut g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0x20 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        return addr_stru_909F31 as ::core::ffi::c_int as uint16;
    }
    let mut r18: uint16 = Samus_GetBottom_R18();
    if *((&raw mut g_ram as *mut uint8).offset(0x195e as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        if *((&raw mut g_ram as *mut uint8).offset(0x195e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - r18 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
            && *((&raw mut g_ram as *mut uint8).offset(0x197e as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 4 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
        {
            return addr_stru_909F3D as ::core::ffi::c_int as uint16;
        }
        return addr_stru_909F31 as ::core::ffi::c_int as uint16;
    }
    if *((&raw mut g_ram as *mut uint8).offset(0x1962 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
        || *((&raw mut g_ram as *mut uint8).offset(0x1962 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - r18 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
    {
        return addr_stru_909F31 as ::core::ffi::c_int as uint16;
    }
    return addr_kSamusSpeedTable_Normal_X as ::core::ffi::c_int as uint16;
}
pub unsafe fn Samus_DetermineAccel_Y() {
    let mut r18: uint16 = 0;
    static mut g_word_909EA1: uint16 = 0x1c00 as uint16;
    static mut g_word_909EA3: uint16 = 0x800 as uint16;
    static mut g_word_909EA5: uint16 = 0x900 as uint16;
    static mut g_word_909EA7: uint16 = 0 as uint16;
    static mut g_word_909EA9: uint16 = 0 as uint16;
    static mut g_word_909EAB: uint16 = 0 as uint16;
    if !(*((&raw mut g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x20 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int)
    {
        r18 = Samus_GetBottom_R18();
        if *((&raw mut g_ram as *mut uint8).offset(0x195e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            if *((&raw mut g_ram as *mut uint8).offset(0x195e as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - r18 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
                && *((&raw mut g_ram as *mut uint8).offset(0x197e as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    & 4 as ::core::ffi::c_int
                    == 0 as ::core::ffi::c_int
            {
                *((&raw mut g_ram as *mut uint8).offset(0xb32 as ::core::ffi::c_int as isize)
                    as *mut uint16) = g_word_909EA3;
                *((&raw mut g_ram as *mut uint8).offset(0xb34 as ::core::ffi::c_int as isize)
                    as *mut uint16) = g_word_909EA9;
                return;
            }
        } else if !(*((&raw mut g_ram as *mut uint8).offset(0x1962 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
            || *((&raw mut g_ram as *mut uint8).offset(0x1962 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - r18 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0)
        {
            *((&raw mut g_ram as *mut uint8).offset(0xb32 as ::core::ffi::c_int as isize)
                as *mut uint16) = g_word_909EA5;
            *((&raw mut g_ram as *mut uint8).offset(0xb34 as ::core::ffi::c_int as isize)
                as *mut uint16) = g_word_909EAB;
            return;
        }
    }
    *((&raw mut g_ram as *mut uint8).offset(0xb32 as ::core::ffi::c_int as isize) as *mut uint16) =
        g_word_909EA1;
    *((&raw mut g_ram as *mut uint8).offset(0xb34 as ::core::ffi::c_int as isize) as *mut uint16) =
        g_word_909EA7;
}
pub unsafe fn Samus_GrappleWallJumpCheck(mut amt: int32) -> uint8 {
    *((&raw mut g_ram as *mut uint8).offset(0xe1c as ::core::ffi::c_int as isize) as *mut uint16) =
        -(1 as ::core::ffi::c_int) as uint16;
    if *(&raw mut g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        != 4 as ::core::ffi::c_int
    {
        if *(&raw mut g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            != 8 as ::core::ffi::c_int
        {
            return 0 as uint8;
        }
        *((&raw mut g_ram as *mut uint8).offset(0xb02 as ::core::ffi::c_int as isize)
            as *mut uint16) = 1 as uint16;
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0xb02 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    }
    let mut cres: CheckEnemyColl_Result = Samus_CheckSolidEnemyColl(amt);
    if cres.collision == 0 {
        amt = WallJumpBlockCollDetect(cres.amt);
        return (*((&raw mut g_ram as *mut uint8).offset(0xdd0 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != 0
            && *((&raw mut g_ram as *mut uint8).offset(0x9b4 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int) as ::core::ffi::c_int as uint8;
    }
    if *((&raw mut g_ram as *mut uint8).offset(0x9b4 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        return 0 as uint8;
    }
    *((&raw mut g_ram as *mut uint8).offset(0xe1c as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0x18a6 as ::core::ffi::c_int as isize)
            as *mut uint16);
    return 1 as uint8;
}
pub unsafe fn Samus_WallJumpCheck(mut amt: int32) -> uint8 {
    let mut current_block: u64;
    let mut cres: CheckEnemyColl_Result = CheckSolidEnemyColl_Result {
        collision: 0,
        amt: 0,
    };
    if *(&raw mut g_ram as *mut uint8).offset(0xa27 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        != 3 as ::core::ffi::c_int
        && *(&raw mut g_ram as *mut uint8).offset(0xa27 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            != 20 as ::core::ffi::c_int
    {
        return 0 as uint8;
    }
    if *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        == kPose_81_FaceR_Screwattack as ::core::ffi::c_int
        || *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == kPose_82_FaceL_Screwattack as ::core::ffi::c_int
    {
        if *((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 27 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            current_block = 10931223485667004002;
        } else {
            current_block = 9270328834339962093;
        }
    } else if *((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - 11 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        current_block = 9270328834339962093;
    } else {
        current_block = 10931223485667004002;
    }
    match current_block {
        9270328834339962093 => {
            if *((&raw mut g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & kButton_Left as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            {
                if *((&raw mut g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    & kButton_Right as ::core::ffi::c_int
                    == 0 as ::core::ffi::c_int
                {
                    return 0 as uint8;
                }
                *((&raw mut g_ram as *mut uint8).offset(0xb02 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 0 as uint16;
                cres = Samus_CheckSolidEnemyColl(amt);
                amt = cres.amt;
                if cres.collision == 0 {
                    amt = WallJumpBlockCollDetect(-amt);
                    if *((&raw mut g_ram as *mut uint8).offset(0xdd0 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        == 0
                    {
                        return 0 as uint8;
                    }
                }
            } else {
                *((&raw mut g_ram as *mut uint8).offset(0xb02 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 1 as uint16;
                cres = Samus_CheckSolidEnemyColl(amt);
                amt = cres.amt;
                if cres.collision == 0 {
                    amt = WallJumpBlockCollDetect(amt);
                    if *((&raw mut g_ram as *mut uint8).offset(0xdd0 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        == 0
                    {
                        return 0 as uint8;
                    }
                }
            }
            if *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                == kPose_81_FaceR_Screwattack as ::core::ffi::c_int
                || *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    == kPose_82_FaceL_Screwattack as ::core::ffi::c_int
            {
                *((&raw mut g_ram as *mut uint8).offset(0xa94 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 1 as uint16;
                *((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 26 as uint16;
            } else {
                *((&raw mut g_ram as *mut uint8).offset(0xa94 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 1 as uint16;
                *((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 10 as uint16;
            }
            return 0 as uint8;
        }
        _ => {
            *((&raw mut g_ram as *mut uint8).offset(0xe1c as ::core::ffi::c_int as isize)
                as *mut uint16) = -(1 as ::core::ffi::c_int) as uint16;
            if *((&raw mut g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & kButton_Left as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                *((&raw mut g_ram as *mut uint8).offset(0xb02 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 1 as uint16;
                cres = Samus_CheckSolidEnemyColl(amt);
                amt = cres.amt;
                if cres.collision == 0 {
                    amt = WallJumpBlockCollDetect(amt);
                    if *((&raw mut g_ram as *mut uint8).offset(0xdd0 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        != 0
                        && *((&raw mut g_ram as *mut uint8)
                            .offset(0x9b4 as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int
                            & *((&raw mut g_ram as *mut uint8)
                                .offset(0x8f as ::core::ffi::c_int as isize)
                                as *mut uint16) as ::core::ffi::c_int
                            != 0 as ::core::ffi::c_int
                    {
                        current_block = 17114735350089217247;
                    } else {
                        return 0 as uint8;
                    }
                } else {
                    if *((&raw mut g_ram as *mut uint8).offset(0x9b4 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        & *((&raw mut g_ram as *mut uint8)
                            .offset(0x8f as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int
                        == 0 as ::core::ffi::c_int
                    {
                        return 0 as uint8;
                    }
                    current_block = 11864030878112954706;
                }
            } else {
                if *((&raw mut g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    & kButton_Right as ::core::ffi::c_int
                    == 0 as ::core::ffi::c_int
                {
                    return 0 as uint8;
                }
                *((&raw mut g_ram as *mut uint8).offset(0xb02 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 0 as uint16;
                cres = Samus_CheckSolidEnemyColl(amt);
                amt = cres.amt;
                if cres.collision == 0 {
                    amt = -amt;
                    WallJumpBlockCollDetect(amt);
                    if *((&raw mut g_ram as *mut uint8).offset(0xdd0 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        != 0
                        && *((&raw mut g_ram as *mut uint8)
                            .offset(0x9b4 as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int
                            & *((&raw mut g_ram as *mut uint8)
                                .offset(0x8f as ::core::ffi::c_int as isize)
                                as *mut uint16) as ::core::ffi::c_int
                            != 0 as ::core::ffi::c_int
                    {
                        current_block = 17114735350089217247;
                    } else {
                        return 0 as uint8;
                    }
                } else {
                    if *((&raw mut g_ram as *mut uint8).offset(0x9b4 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        & *((&raw mut g_ram as *mut uint8)
                            .offset(0x8f as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int
                        == 0 as ::core::ffi::c_int
                    {
                        return 0 as uint8;
                    }
                    current_block = 11864030878112954706;
                }
            }
            match current_block {
                11864030878112954706 => {
                    if (amt >> 16 as ::core::ffi::c_int) - 8 as int32 & 0x8000 as int32 != 0 {
                        *((&raw mut g_ram as *mut uint8)
                            .offset(0xdc6 as ::core::ffi::c_int as isize)
                            as *mut uint16) = 5 as uint16;
                        *((&raw mut g_ram as *mut uint8)
                            .offset(0xe1c as ::core::ffi::c_int as isize)
                            as *mut uint16) = *((&raw mut g_ram as *mut uint8)
                            .offset(0x18a6 as ::core::ffi::c_int as isize)
                            as *mut uint16);
                        return 1 as uint8;
                    }
                    return 0 as uint8;
                }
                _ => {
                    if (amt >> 16 as ::core::ffi::c_int) - 8 as int32 & 0x8000 as int32 != 0 {
                        *((&raw mut g_ram as *mut uint8)
                            .offset(0xdc6 as ::core::ffi::c_int as isize)
                            as *mut uint16) = 5 as uint16;
                        return 1 as uint8;
                    }
                    return 0 as uint8;
                }
            }
        }
    };
}
pub unsafe fn Samus_Movement_07_Unused() {
    *((&raw mut g_ram as *mut uint8).offset(0xdc6 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
}
static mut kSamusMovementHandlers: [Option<HandlerFunc>; 28] = [
    Some(Samus_Movement_00_Standing as unsafe fn() -> ()),
    Some(Samus_Movement_01_Running as unsafe fn() -> ()),
    Some(Samus_Movement_02_NormalJumping as unsafe fn() -> ()),
    Some(Samus_Movement_03_SpinJumping as unsafe fn() -> ()),
    Some(Samus_Movement_04_MorphBallOnGround as unsafe fn() -> ()),
    Some(Samus_Movement_05_Crouching as unsafe fn() -> ()),
    Some(Samus_Movement_06_Falling as unsafe fn() -> ()),
    Some(Samus_Movement_07_Unused as unsafe fn() -> ()),
    Some(Samus_Movement_08_MorphBallFalling as unsafe fn() -> ()),
    Some(Samus_Movement_09_Unused as unsafe fn() -> ()),
    Some(Samus_Movement_0A_KnockbackOrCrystalFlashEnding as unsafe fn() -> ()),
    Some(Samus_Movement_0B_Unused as unsafe fn() -> ()),
    Some(Samus_Movement_0C_Unused as unsafe fn() -> ()),
    Some(Samus_Movement_0D_Unused as unsafe fn() -> ()),
    Some(Samus_Movement_0E_TurningAroundOnGround as unsafe fn() -> ()),
    Some(Samus_Movement_0F_CrouchingEtcTransition as unsafe fn() -> ()),
    Some(Samus_Movement_10_Moonwalking as unsafe fn() -> ()),
    Some(Samus_Movement_11_SpringBallOnGround as unsafe fn() -> ()),
    Some(Samus_Movement_12_SpringBallInAir as unsafe fn() -> ()),
    Some(Samus_Movement_13_SpringBallFalling as unsafe fn() -> ()),
    Some(Samus_Movement_14_WallJumping as unsafe fn() -> ()),
    Some(Samus_Movement_15_RanIntoWall as unsafe fn() -> ()),
    Some(Samus_Movement_16_Grappling as unsafe fn() -> ()),
    Some(Samus_Movement_17_TurningAroundJumping as unsafe fn() -> ()),
    Some(Samus_Movement_18_TurningAroundFalling as unsafe fn() -> ()),
    Some(Samus_Movement_19_DamageBoost as unsafe fn() -> ()),
    Some(Samus_Movement_1A_GrabbedByDraygon as unsafe fn() -> ()),
    Some(Samus_Movement_1B_ShinesparkEtc as unsafe fn() -> ()),
];
pub unsafe fn Samus_MovementHandler_Normal() {
    if *((&raw mut g_ram as *mut uint8).offset(0xa78 as ::core::ffi::c_int as isize) as *mut uint16)
        == 0
    {
        kSamusMovementHandlers
            [*(&raw mut g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize) as usize]
            .expect("non-null function pointer")();
        Samus_UpdateSpeedEchoPos();
    }
}
pub unsafe fn Samus_Movement_00_Standing() {
    if *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        != 0
        && *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != kPose_9B_FaceF_VariaGravitySuit as ::core::ffi::c_int
    {
        if (*((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == kPose_01_FaceR_Normal as ::core::ffi::c_int
            || *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                == kPose_02_FaceL_Normal as ::core::ffi::c_int)
            && *((&raw mut g_ram as *mut uint8).offset(0x9b2 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & *((&raw mut g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
        {
            *((&raw mut g_ram as *mut uint8).offset(0xa94 as ::core::ffi::c_int as isize)
                as *mut uint16) = 16 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
        }
        Samus_Move_NoBaseSpeed_X();
        Samus_Move_NoSpeedCalc_Y();
        Samus_CancelSpeedBoost();
        *((&raw mut g_ram as *mut uint8).offset(0xb42 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb44 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb46 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb48 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb4a as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    } else {
        if *((&raw mut g_ram as *mut uint8).offset(0xe18 as ::core::ffi::c_int as isize)
            as *mut uint16)
            != 0
        {
            *((&raw mut g_ram as *mut uint8).offset(0xb02 as ::core::ffi::c_int as isize)
                as *mut uint16) = 2 as uint16;
            Samus_MoveDown_NoSolidColl(
                (1 as ::core::ffi::c_int as int16 as int32) << 16 as ::core::ffi::c_int,
            );
        }
        *((&raw mut g_ram as *mut uint8).offset(0xdc6 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    };
}
pub unsafe fn Samus_Movement_01_Running() {
    static mut kSamusFootstepFrame: [uint8; 10] = [
        0 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
    ];
    Samus_HandleMovement_X();
    Samus_Move_NoSpeedCalc_Y();
    if *((&raw mut g_ram as *mut uint8).offset(0xa94 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        == 1 as ::core::ffi::c_int
    {
        if kSamusFootstepFrame[*((&raw mut g_ram as *mut uint8)
            .offset(0xa96 as ::core::ffi::c_int as isize)
            as *mut uint16) as usize]
            != 0
        {
            Samus_FootstepGraphics();
            if *((&raw mut g_ram as *mut uint8).offset(0x1f51 as ::core::ffi::c_int as isize)
                as *mut uint16)
                == 0
                && *((&raw mut g_ram as *mut uint8).offset(0x179c as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    == 0
                && *((&raw mut g_ram as *mut uint8).offset(0xa68 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    == 0
                && *((&raw mut g_ram as *mut uint8).offset(0xb3e as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    & 0x400 as ::core::ffi::c_int
                    == 0 as ::core::ffi::c_int
            {
                QueueSfx3_Max6(6 as uint16);
            }
        }
    }
}
pub unsafe fn Samus_Movement_02_NormalJumping() {
    Samus_JumpingMovement();
}
pub unsafe fn Samus_Movement_03_SpinJumping() {
    let mut current_block: u64;
    static mut kSamusPhys_JumpMinYVelAir: uint16 = 0x280 as uint16;
    static mut kSamusPhys_JumpMaxYVelAir: uint16 = 0x500 as uint16;
    static mut kSamusPhys_JumpMinYVelWater: uint16 = 0x80 as uint16;
    static mut kSamusPhys_JumpMaxYVelWater: uint16 = 0x500 as uint16;
    let mut r18: uint16 = 0 as uint16;
    if *((&raw mut g_ram as *mut uint8).offset(0xa74 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 4 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        let mut r20: uint16 = Samus_GetTop_R20();
        if *((&raw mut g_ram as *mut uint8).offset(0x195e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            if *((&raw mut g_ram as *mut uint8).offset(0x1962 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
                && *((&raw mut g_ram as *mut uint8).offset(0x1962 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    - r20 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    != 0
            {
                r18 = 1 as uint16;
            }
        } else if *((&raw mut g_ram as *mut uint8).offset(0x195e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - r20 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
            && *((&raw mut g_ram as *mut uint8).offset(0x197e as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 4 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
        {
            r18 = 1 as uint16;
        }
    }
    if r18 == 0 {
        if *((&raw mut g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x200 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            if !(*((&raw mut g_ram as *mut uint8).offset(0xb36 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                != 2 as ::core::ffi::c_int)
            {
                if *((&raw mut g_ram as *mut uint8).offset(0xad2 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    != 0
                {
                    if ((*(((&raw mut g_ram as *mut uint8)
                        .offset(0xb2c as ::core::ffi::c_int as isize)
                        as *mut uint16 as *mut uint8)
                        .offset(1 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        - kSamusPhys_JumpMinYVelWater as ::core::ffi::c_int)
                        as int16 as ::core::ffi::c_int)
                        < 0 as ::core::ffi::c_int
                        || (*(((&raw mut g_ram as *mut uint8)
                            .offset(0xb2c as ::core::ffi::c_int as isize)
                            as *mut uint16 as *mut uint8)
                            .offset(1 as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int
                            - kSamusPhys_JumpMaxYVelWater as ::core::ffi::c_int)
                            as int16 as ::core::ffi::c_int
                            >= 0 as ::core::ffi::c_int
                    {
                        current_block = 12147880666119273379;
                    } else {
                        current_block = 5143058163439228106;
                    }
                } else if ((*(((&raw mut g_ram as *mut uint8)
                    .offset(0xb2c as ::core::ffi::c_int as isize)
                    as *mut uint16 as *mut uint8)
                    .offset(1 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    - kSamusPhys_JumpMinYVelAir as ::core::ffi::c_int)
                    as int16 as ::core::ffi::c_int)
                    < 0 as ::core::ffi::c_int
                    || (*(((&raw mut g_ram as *mut uint8)
                        .offset(0xb2c as ::core::ffi::c_int as isize)
                        as *mut uint16 as *mut uint8)
                        .offset(1 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        - kSamusPhys_JumpMaxYVelAir as ::core::ffi::c_int)
                        as int16 as ::core::ffi::c_int
                        >= 0 as ::core::ffi::c_int
                {
                    current_block = 12147880666119273379;
                } else {
                    current_block = 5143058163439228106;
                }
                match current_block {
                    12147880666119273379 => {}
                    _ => {
                        *((&raw mut g_ram as *mut uint8)
                            .offset(0xdfa as ::core::ffi::c_int as isize)
                            as *mut uint16) =
                            (*((&raw mut g_ram as *mut uint8)
                                .offset(0xdfa as ::core::ffi::c_int as isize)
                                as *mut uint16) as ::core::ffi::c_int
                                & 0xff00 as ::core::ffi::c_int
                                | 1 as ::core::ffi::c_int) as uint16;
                        if *((&raw mut g_ram as *mut uint8)
                            .offset(0x9b4 as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int
                            & *((&raw mut g_ram as *mut uint8)
                                .offset(0x8f as ::core::ffi::c_int as isize)
                                as *mut uint16) as ::core::ffi::c_int
                            != 0 as ::core::ffi::c_int
                        {
                            Samus_InitJump();
                        }
                    }
                }
            }
        }
        if *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == kPose_81_FaceR_Screwattack as ::core::ffi::c_int
            || *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                == kPose_82_FaceL_Screwattack as ::core::ffi::c_int
        {
            *((&raw mut g_ram as *mut uint8).offset(0xa6e as ::core::ffi::c_int as isize)
                as *mut uint16) = 3 as uint16;
        } else if *((&raw mut g_ram as *mut uint8).offset(0xcd0 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 60 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            *((&raw mut g_ram as *mut uint8).offset(0xa6e as ::core::ffi::c_int as isize)
                as *mut uint16) = 4 as uint16;
        }
    } else if *((&raw mut g_ram as *mut uint8).offset(0xa94 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        == 1 as ::core::ffi::c_int
        && *(g_rom.offset(
            (((0x90a514 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x90a514 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint8)
            .offset(
                *((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
                    as *mut uint16) as isize,
            ) as ::core::ffi::c_int
            != 0
    {
        QueueSfx1_Max6(0x2f as uint16);
    }
    Samus_SpinJumpMovement();
}
pub unsafe fn Samus_Movement_04_MorphBallOnGround() {
    if *((&raw mut g_ram as *mut uint8).offset(0xb4a as ::core::ffi::c_int as isize) as *mut uint16)
        == 0
    {
        let mut current_block_10: u64;
        if *(&raw mut g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            == 4 as ::core::ffi::c_int
        {
            if *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                == kPose_41_FaceL_Morphball_Ground as ::core::ffi::c_int
            {
                current_block_10 = 17055623196806638295;
            } else {
                current_block_10 = 7746791466490516765;
            }
        } else if *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == kPose_1D_FaceR_Morphball_Ground as ::core::ffi::c_int
        {
            current_block_10 = 17055623196806638295;
        } else {
            current_block_10 = 7746791466490516765;
        }
        match current_block_10 {
            7746791466490516765 => {}
            _ => {
                Samus_Move_NoBaseSpeed_X();
                if Samus_CheckAndMoveY() as ::core::ffi::c_int & 1 as ::core::ffi::c_int == 0 {
                    Samus_Move_NoSpeedCalc_Y();
                    Samus_CancelSpeedBoost();
                    *((&raw mut g_ram as *mut uint8).offset(0xb42 as ::core::ffi::c_int as isize)
                        as *mut uint16) = 0 as uint16;
                    *((&raw mut g_ram as *mut uint8).offset(0xb44 as ::core::ffi::c_int as isize)
                        as *mut uint16) = 0 as uint16;
                    *((&raw mut g_ram as *mut uint8).offset(0xb46 as ::core::ffi::c_int as isize)
                        as *mut uint16) = 0 as uint16;
                    *((&raw mut g_ram as *mut uint8).offset(0xb48 as ::core::ffi::c_int as isize)
                        as *mut uint16) = 0 as uint16;
                    *((&raw mut g_ram as *mut uint8).offset(0xb4a as ::core::ffi::c_int as isize)
                        as *mut uint16) = 0 as uint16;
                }
                return;
            }
        }
    }
    Samus_HandleMovement_X();
    if Samus_CheckAndMoveY() as ::core::ffi::c_int & 1 as ::core::ffi::c_int == 0 {
        Samus_Move_NoSpeedCalc_Y();
    }
}
pub unsafe fn Samus_Movement_05_Crouching() {
    Samus_Move_NoBaseSpeed_X();
    Samus_Move_NoSpeedCalc_Y();
    *((&raw mut g_ram as *mut uint8).offset(0xb42 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb44 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb46 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb48 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb4a as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
}
pub unsafe fn Samus_Movement_06_Falling() {
    Samus_FallingMovement();
    if (*((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        == kPose_29_FaceR_Fall as ::core::ffi::c_int
        || *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == kPose_2A_FaceL_Fall as ::core::ffi::c_int
        || *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == kPose_67_FaceR_Fall_Gun as ::core::ffi::c_int
        || *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == kPose_68_FaceL_Fall_Gun as ::core::ffi::c_int)
        && *((&raw mut g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 5 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
    {
        if *((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 5 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            *((&raw mut g_ram as *mut uint8).offset(0xa94 as ::core::ffi::c_int as isize)
                as *mut uint16) = 8 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
                as *mut uint16) = 5 as uint16;
        }
    }
}
pub unsafe fn Samus_Movement_08_MorphBallFalling() {
    if *((&raw mut g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & (kButton_Left as ::core::ffi::c_int | kButton_Right as ::core::ffi::c_int)
        == 0 as ::core::ffi::c_int
        && *((&raw mut g_ram as *mut uint8).offset(0xb4a as ::core::ffi::c_int as isize)
            as *mut uint16)
            == 0
    {
        Samus_CancelSpeedBoost();
        *((&raw mut g_ram as *mut uint8).offset(0xb42 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb44 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb46 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb48 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb4a as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    }
    if *((&raw mut g_ram as *mut uint8).offset(0xb20 as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        Samus_MorphedBouncingMovement();
    } else {
        Samus_MorphedFallingMovement();
    };
}
pub unsafe fn Samus_Movement_09_Unused() {}
pub unsafe fn Samus_Movement_0A_KnockbackOrCrystalFlashEnding() {
    *((&raw mut g_ram as *mut uint8).offset(0xdc6 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    Samus_Move_NoSpeedCalc_Y();
}
pub unsafe fn Samus_Movement_0B_Unused() {
    *((&raw mut g_ram as *mut uint8).offset(0xdc6 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
}
pub unsafe fn Samus_Movement_0C_Unused() {
    *((&raw mut g_ram as *mut uint8).offset(0xdc6 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
}
pub unsafe fn Samus_Movement_0D_Unused() {}
pub unsafe fn nullsub_13() {}
static mut kSamusCrouchingEtcFuncs: [Option<HandlerFunc>; 12] = unsafe {
    [
        Some(nullsub_13 as unsafe fn() -> ()),
        Some(nullsub_13 as unsafe fn() -> ()),
        Some(nullsub_13 as unsafe fn() -> ()),
        Some(nullsub_13 as unsafe fn() -> ()),
        Some(SamusCrouchingEtcFunc as unsafe fn() -> ()),
        Some(SamusCrouchingEtcFunc as unsafe fn() -> ()),
        Some(nullsub_13 as unsafe fn() -> ()),
        Some(nullsub_13 as unsafe fn() -> ()),
        Some(nullsub_13 as unsafe fn() -> ()),
        Some(nullsub_13 as unsafe fn() -> ()),
        Some(SamusCrouchingEtcFunc as unsafe fn() -> ()),
        Some(SamusCrouchingEtcFunc as unsafe fn() -> ()),
    ]
};
pub unsafe fn Samus_Movement_0F_CrouchingEtcTransition() {
    let mut current_block: u64;
    if *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - kPose_F1_FaceR_CrouchTrans_AimU as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        if *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - kPose_DB as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            current_block = 3819603838861705400;
        } else {
            kSamusCrouchingEtcFuncs[(*((&raw mut g_ram as *mut uint8)
                .offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - 53 as ::core::ffi::c_int) as usize]
                .expect("non-null function pointer")();
            current_block = 14155750587950065367;
        }
    } else {
        current_block = 14155750587950065367;
    }
    match current_block {
        14155750587950065367 => {
            Samus_Move_NoBaseSpeed_X();
            if Samus_MoveY_Simple_() as ::core::ffi::c_int & 1 as ::core::ffi::c_int == 0 {
                Samus_Move_NoSpeedCalc_Y();
            }
        }
        _ => {}
    }
    if *((&raw mut g_ram as *mut uint8).offset(0xdc6 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        == 1025 as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb36 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb20 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    }
    *((&raw mut g_ram as *mut uint8).offset(0xdc6 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
}
pub unsafe fn SamusCrouchingEtcFunc() {
    *((&raw mut g_ram as *mut uint8).offset(0xa46 as ::core::ffi::c_int as isize) as *mut uint16) =
        3 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xaa4 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
}
pub unsafe fn Samus_Movement_0E_TurningAroundOnGround() {
    Samus_HandleMovement_X();
    Samus_Move_NoSpeedCalc_Y();
    Samus_CancelSpeedBoost();
    *((&raw mut g_ram as *mut uint8).offset(0xb42 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb44 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xdc6 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
}
pub unsafe fn Samus_Movement_10_Moonwalking() {
    Samus_HandleMovement_X();
    Samus_Move_NoSpeedCalc_Y();
}
pub unsafe fn Samus_Movement_11_SpringBallOnGround() {
    if *((&raw mut g_ram as *mut uint8).offset(0xb4a as ::core::ffi::c_int as isize) as *mut uint16)
        == 0
    {
        let mut current_block_10: u64;
        if *(&raw mut g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            == 4 as ::core::ffi::c_int
        {
            if *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                == kPose_7A_FaceL_Springball_Ground as ::core::ffi::c_int
            {
                current_block_10 = 7185981292489567883;
            } else {
                current_block_10 = 7746791466490516765;
            }
        } else if *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == kPose_79_FaceR_Springball_Ground as ::core::ffi::c_int
        {
            current_block_10 = 7185981292489567883;
        } else {
            current_block_10 = 7746791466490516765;
        }
        match current_block_10 {
            7746791466490516765 => {}
            _ => {
                Samus_Move_NoBaseSpeed_X();
                if Samus_CheckAndMoveY() as ::core::ffi::c_int & 1 as ::core::ffi::c_int == 0 {
                    Samus_Move_NoSpeedCalc_Y();
                    Samus_CancelSpeedBoost();
                    *((&raw mut g_ram as *mut uint8).offset(0xb42 as ::core::ffi::c_int as isize)
                        as *mut uint16) = 0 as uint16;
                    *((&raw mut g_ram as *mut uint8).offset(0xb44 as ::core::ffi::c_int as isize)
                        as *mut uint16) = 0 as uint16;
                    *((&raw mut g_ram as *mut uint8).offset(0xb46 as ::core::ffi::c_int as isize)
                        as *mut uint16) = 0 as uint16;
                    *((&raw mut g_ram as *mut uint8).offset(0xb48 as ::core::ffi::c_int as isize)
                        as *mut uint16) = 0 as uint16;
                    *((&raw mut g_ram as *mut uint8).offset(0xb4a as ::core::ffi::c_int as isize)
                        as *mut uint16) = 0 as uint16;
                }
                return;
            }
        }
    }
    Samus_HandleMovement_X();
    if Samus_CheckAndMoveY() as ::core::ffi::c_int & 1 as ::core::ffi::c_int == 0 {
        Samus_Move_NoSpeedCalc_Y();
    }
}
pub unsafe fn Samus_Movement_12_SpringBallInAir() {
    if *((&raw mut g_ram as *mut uint8).offset(0xb20 as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        Samus_MorphedBouncingMovement();
    } else {
        Samus_JumpingMovement();
    };
}
pub unsafe fn Samus_Movement_13_SpringBallFalling() {
    if *((&raw mut g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & (kButton_Left as ::core::ffi::c_int | kButton_Right as ::core::ffi::c_int)
        == 0 as ::core::ffi::c_int
        && *((&raw mut g_ram as *mut uint8).offset(0xb4a as ::core::ffi::c_int as isize)
            as *mut uint16)
            == 0
    {
        Samus_CancelSpeedBoost();
        *((&raw mut g_ram as *mut uint8).offset(0xb42 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb44 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb46 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb48 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb4a as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    }
    if *((&raw mut g_ram as *mut uint8).offset(0xb20 as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        Samus_MorphedBouncingMovement();
    } else {
        Samus_MorphedFallingMovement();
    };
}
pub unsafe fn Samus_Movement_14_WallJumping() {
    if *((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - 23 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        if *((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 3 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
            && *((&raw mut g_ram as *mut uint8).offset(0xcd0 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - 60 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
        {
            *((&raw mut g_ram as *mut uint8).offset(0xa6e as ::core::ffi::c_int as isize)
                as *mut uint16) = 4 as uint16;
        }
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0xa6e as ::core::ffi::c_int as isize)
            as *mut uint16) = 3 as uint16;
    }
    Samus_JumpingMovement();
}
pub unsafe fn Samus_Movement_15_RanIntoWall() {
    Samus_Move_NoBaseSpeed_X();
    Samus_Move_NoSpeedCalc_Y();
    Samus_CancelSpeedBoost();
    *((&raw mut g_ram as *mut uint8).offset(0xb42 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb44 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb46 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb48 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb4a as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
}
pub unsafe fn Samus_Movement_16_Grappling() {
    if *((&raw mut g_ram as *mut uint8).offset(0xdc6 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        != 5 as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0xdc6 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    }
}
pub unsafe fn Samus_Movement_17_TurningAroundJumping() {
    Samus_HandleMovement_X();
    if Samus_CheckAndMoveY() as ::core::ffi::c_int & 1 as ::core::ffi::c_int == 0 {
        Samus_Move_NoSpeedCalc_Y();
    }
    Samus_CancelSpeedBoost();
    *((&raw mut g_ram as *mut uint8).offset(0xb42 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb44 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xdc6 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
}
pub unsafe fn Samus_Movement_18_TurningAroundFalling() {
    Samus_HandleMovement_X();
    if Samus_CheckAndMoveY() as ::core::ffi::c_int & 1 as ::core::ffi::c_int == 0 {
        Samus_Move_NoSpeedCalc_Y();
    }
    Samus_CancelSpeedBoost();
    *((&raw mut g_ram as *mut uint8).offset(0xb42 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb44 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xdc6 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
}
pub unsafe fn Samus_Movement_19_DamageBoost() {
    Samus_JumpingMovement();
}
pub unsafe fn Samus_Movement_1A_GrabbedByDraygon() {
    *((&raw mut g_ram as *mut uint8).offset(0xdc6 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
}
pub unsafe fn Samus_Movement_1B_ShinesparkEtc() {
    *((&raw mut g_ram as *mut uint8).offset(0xdc6 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
}
static mut kShr0x80: [uint8; 8] = [
    0x80 as ::core::ffi::c_int as uint8,
    0x40 as ::core::ffi::c_int as uint8,
    0x20 as ::core::ffi::c_int as uint8,
    0x10 as ::core::ffi::c_int as uint8,
    8 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
];
static mut kShr0xFc00: [uint16; 8] = [
    0xfc00 as ::core::ffi::c_int as uint16,
    0x7e00 as ::core::ffi::c_int as uint16,
    0x3f00 as ::core::ffi::c_int as uint16,
    0x1f80 as ::core::ffi::c_int as uint16,
    0xfc0 as ::core::ffi::c_int as uint16,
    0x7e0 as ::core::ffi::c_int as uint16,
    0x3f0 as ::core::ffi::c_int as uint16,
    0x1f8 as ::core::ffi::c_int as uint16,
];
unsafe fn MarkMapTileAsExplored(mut r18: uint16, mut r24: uint16) {
    let mut v0: uint8 = ((r18 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int) as uint16
        as ::core::ffi::c_int
        >> 8 as ::core::ffi::c_int) as uint8;
    let mut R34: uint16 = (*((&raw mut g_ram as *mut uint8)
        .offset(0x7a1 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        + v0 as ::core::ffi::c_int
        & 0x20 as ::core::ffi::c_int) as uint16;
    let mut r20: uint16 = ((*((&raw mut g_ram as *mut uint8)
        .offset(0x7a1 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        + v0 as ::core::ffi::c_int
        & 0x1f as ::core::ffi::c_int)
        >> 3 as ::core::ffi::c_int) as uint16;
    let mut R22: uint16 = (*((&raw mut g_ram as *mut uint8)
        .offset(0x7a3 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        + ((r24 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int)
        + 1 as ::core::ffi::c_int) as uint16;
    let ref mut fresh13 = *(&raw mut g_ram as *mut uint8)
        .offset(0x7f7 as ::core::ffi::c_int as isize)
        .offset(
            (r20 as ::core::ffi::c_int
                + 4 as ::core::ffi::c_int * (R34 as ::core::ffi::c_int + R22 as ::core::ffi::c_int))
                as uint16 as isize,
        );
    *fresh13 = (*fresh13 as ::core::ffi::c_int
        | kShr0x80[(*((&raw mut g_ram as *mut uint8).offset(0x7a1 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + v0 as ::core::ffi::c_int
            & 7 as ::core::ffi::c_int) as usize] as ::core::ffi::c_int) as uint8;
}
pub unsafe fn DisableMinimapAndMarkBossRoomAsExplored() {
    *((&raw mut g_ram as *mut uint8).offset(0x5f7 as ::core::ffi::c_int as isize) as *mut uint16) =
        1 as uint16;
    let mut v0: uint16 = 0 as uint16;
    loop {
        let mut v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut g_ram as *mut uint8).offset(0xc608 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v1 + 26 as ::core::ffi::c_int) as isize) = 11295 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xc608 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v1 + 58 as ::core::ffi::c_int) as isize) = 11295 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xc608 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v1 + 90 as ::core::ffi::c_int) as isize) = 11295 as uint16;
        v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        if !(((v0 as ::core::ffi::c_int - 10 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
    let mut v2: uint16 = 5 as uint16;
    while *((&raw mut g_ram as *mut uint8).offset(0x179c as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        != (*(g_rom.offset(
            (((0x90a83a as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x90a83a as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut DisableMinimapAndMarkBossRoomAsExploredEnt)
            .offset(v2 as isize))
        .boss_id_ as ::core::ffi::c_int
    {
        v2 = v2.wrapping_sub(1);
        if v2 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            return;
        }
    }
    let mut i: ::core::ffi::c_int = (*(g_rom.offset(
        (((0x90a83a as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x90a83a as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8
        as *mut DisableMinimapAndMarkBossRoomAsExploredEnt)
        .offset(v2 as isize))
    .ptrs as ::core::ffi::c_int;
    loop {
        let mut v4: *const uint16 = RomPtr_90(i as uint16_t) as *const uint16;
        if *v4 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            break;
        }
        MarkMapTileAsExplored(
            *v4.offset(0 as ::core::ffi::c_int as isize),
            *v4.offset(1 as ::core::ffi::c_int as isize),
        );
        i += 4 as ::core::ffi::c_int;
    }
}
pub unsafe fn InitializeMiniMapBroken() {}
pub unsafe fn UpdateMinimap() {
    let mut v4: int16 = 0;
    let mut v10: int16 = 0;
    if *((&raw mut g_ram as *mut uint8).offset(0x5f7 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        != 0
        || *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            >> 4 as ::core::ffi::c_int
            >= *((&raw mut g_ram as *mut uint8).offset(0x7a5 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
        || *((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            >> 4 as ::core::ffi::c_int
            >= *((&raw mut g_ram as *mut uint8).offset(0x7a7 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
    {
        return;
    }
    let mut r46: uint16 = 0 as uint16;
    let mut v0: uint8 = ((*((&raw mut g_ram as *mut uint8)
        .offset(0xaf6 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0xff00 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
        >> 8 as ::core::ffi::c_int) as uint8;
    let mut r34: uint16 = (*((&raw mut g_ram as *mut uint8)
        .offset(0x7a1 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        + v0 as ::core::ffi::c_int
        & 0x20 as ::core::ffi::c_int) as uint16;
    let mut r18: uint16 = (*((&raw mut g_ram as *mut uint8)
        .offset(0x7a1 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        + v0 as ::core::ffi::c_int
        & 0x1f as ::core::ffi::c_int) as uint16;
    let mut v1: uint16 = (*((&raw mut g_ram as *mut uint8)
        .offset(0x7a1 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        + v0 as ::core::ffi::c_int
        & 7 as ::core::ffi::c_int) as uint16;
    let mut r20: uint16 = ((*((&raw mut g_ram as *mut uint8)
        .offset(0x7a1 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        + v0 as ::core::ffi::c_int
        & 0x1f as ::core::ffi::c_int)
        >> 3 as ::core::ffi::c_int) as uint16;
    let mut r22: uint16 = (*((&raw mut g_ram as *mut uint8)
        .offset(0x7a3 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        + ((*((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0xff00 as ::core::ffi::c_int)
            >> 8 as ::core::ffi::c_int)
        + 1 as ::core::ffi::c_int) as uint16;
    let mut v2: uint16 = (r20 as ::core::ffi::c_int
        + 4 as ::core::ffi::c_int * (r34 as ::core::ffi::c_int + r22 as ::core::ffi::c_int))
        as uint16;
    let ref mut fresh98 = *(&raw mut g_ram as *mut uint8)
        .offset(0x7f7 as ::core::ffi::c_int as isize)
        .offset(v2 as isize);
    *fresh98 = (*fresh98 as ::core::ffi::c_int
        | kShr0x80[(*((&raw mut g_ram as *mut uint8).offset(0x7a1 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + v0 as ::core::ffi::c_int
            & 7 as ::core::ffi::c_int) as usize] as ::core::ffi::c_int) as uint8;
    let mut r32: uint16 = v1;
    let mut r30: uint16 = v2;
    let mut v3: uint16 = (v2 as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as uint16;
    v4 = (v1 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as int16;
    if ((v1 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        v4 = (v4 as ::core::ffi::c_int & 7 as ::core::ffi::c_int) as int16;
        v3 = v3.wrapping_sub(1);
        r46 = r46.wrapping_add(1);
    }
    let mut R50: uint16 = v3;
    let mut r52: ::core::ffi::c_int = 2 as ::core::ffi::c_int * v4 as ::core::ffi::c_int;
    let mut v6: ::core::ffi::c_int = r52 >> 1 as ::core::ffi::c_int;
    let mut r24: uint16 = (kShr0xFc00[v6 as usize] as ::core::ffi::c_int
        & swap16(
            *((&raw mut g_ram as *mut uint8)
                .offset(0x7f7 as ::core::ffi::c_int as isize)
                .offset(v3 as isize) as *mut uint8 as *mut uint16),
        ) as ::core::ffi::c_int) as uint16;
    let mut r26: uint16 = (kShr0xFc00[v6 as usize] as ::core::ffi::c_int
        & swap16(
            *((&raw mut g_ram as *mut uint8)
                .offset(0x7f7 as ::core::ffi::c_int as isize)
                .offset((v3 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as isize)
                as *mut uint8 as *mut uint16),
        ) as ::core::ffi::c_int) as uint16;
    let mut r28: uint16 = (kShr0xFc00[v6 as usize] as ::core::ffi::c_int
        & swap16(
            *((&raw mut g_ram as *mut uint8)
                .offset(0x7f7 as ::core::ffi::c_int as isize)
                .offset((v3 as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as isize)
                as *mut uint8 as *mut uint16),
        ) as ::core::ffi::c_int) as uint16;
    let mut r9: *const uint8 = RomPtr_82(
        *(g_rom.offset(
            (((0x829717 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x829717 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(
                *((&raw mut g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize)
                    as *mut uint16) as isize,
            ) as uint16_t,
    );
    let mut r15: *const uint8 = r9;
    r9 = r9.offset(v3 as ::core::ffi::c_int as isize);
    let mut r38: uint16 = swap16(*(r9 as *mut uint16));
    r9 = r9.offset(4 as ::core::ffi::c_int as isize);
    let mut r40: uint16 = swap16(*(r9 as *mut uint16));
    r9 = r9.offset(4 as ::core::ffi::c_int as isize);
    let mut r42: uint16 = swap16(*(r9 as *mut uint16));
    if R50 as ::core::ffi::c_int & 3 as ::core::ffi::c_int == 3 as ::core::ffi::c_int {
        v10 = (if r46 as ::core::ffi::c_int != 0 {
            r52 >> 1 as ::core::ffi::c_int
        } else {
            r32 as ::core::ffi::c_int
        }) as int16;
        if v10 as ::core::ffi::c_int - 6 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
            let mut R48: uint8 = (if r34 as ::core::ffi::c_int != 0 {
                R50 as ::core::ffi::c_int - 124 as ::core::ffi::c_int
            } else {
                R50 as ::core::ffi::c_int + 125 as ::core::ffi::c_int
            }) as uint8;
            let mut v0_0: uint16 = R48 as uint16;
            let mut r44: uint16 = 0 as uint16;
            r9 = r15.offset(R48 as ::core::ffi::c_int as isize);
            *(&raw mut r44 as *mut uint8).offset(0 as ::core::ffi::c_int as isize) =
                *(&raw mut g_ram as *mut uint8)
                    .offset(0x7f7 as ::core::ffi::c_int as isize)
                    .offset(R48 as isize);
            *(&raw mut r44 as *mut uint8).offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) = *r9;
            if r34 as uint8 as ::core::ffi::c_int == 32 as ::core::ffi::c_int {
                *(&raw mut r38 as *mut uint8).offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ) = *(&raw mut r44 as *mut uint8).offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                );
                *(&raw mut r24 as *mut uint8).offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ) = r44 as uint8;
            } else {
                *(&raw mut r38 as *mut uint8).offset(0 as ::core::ffi::c_int as isize) =
                    *(&raw mut r44 as *mut uint8).offset(
                        (::core::mem::size_of::<uint16>() as usize)
                            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                            .wrapping_sub(1 as usize) as isize,
                    );
                *(&raw mut r24 as *mut uint8).offset(0 as ::core::ffi::c_int as isize) =
                    r44 as uint8;
            }
            *(&raw mut r44 as *mut uint8).offset(0 as ::core::ffi::c_int as isize) =
                *(&raw mut g_ram as *mut uint8)
                    .offset(0x7f7 as ::core::ffi::c_int as isize)
                    .offset((v0_0 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as isize);
            r9 = r9.offset(4 as ::core::ffi::c_int as isize);
            *(&raw mut r44 as *mut uint8).offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) = *r9;
            if r34 as uint8 as ::core::ffi::c_int == 32 as ::core::ffi::c_int {
                *(&raw mut r40 as *mut uint8).offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ) = *(&raw mut r44 as *mut uint8).offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                );
                *(&raw mut r26 as *mut uint8).offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ) = r44 as uint8;
            } else {
                *(&raw mut r40 as *mut uint8).offset(0 as ::core::ffi::c_int as isize) =
                    *(&raw mut r44 as *mut uint8).offset(
                        (::core::mem::size_of::<uint16>() as usize)
                            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                            .wrapping_sub(1 as usize) as isize,
                    );
                *(&raw mut r26 as *mut uint8).offset(0 as ::core::ffi::c_int as isize) =
                    r44 as uint8;
            }
            *(&raw mut r44 as *mut uint8).offset(0 as ::core::ffi::c_int as isize) =
                *(&raw mut g_ram as *mut uint8)
                    .offset(0x7f7 as ::core::ffi::c_int as isize)
                    .offset((v0_0 as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as isize);
            r9 = r9.offset(4 as ::core::ffi::c_int as isize);
            *(&raw mut r44 as *mut uint8).offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) = *r9;
            if r34 as uint8 as ::core::ffi::c_int == 32 as ::core::ffi::c_int {
                *(&raw mut r42 as *mut uint8).offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ) = *(&raw mut r44 as *mut uint8).offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                );
                *(&raw mut r28 as *mut uint8).offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ) = r44 as uint8;
            } else {
                *(&raw mut r42 as *mut uint8).offset(0 as ::core::ffi::c_int as isize) =
                    *(&raw mut r44 as *mut uint8).offset(
                        (::core::mem::size_of::<uint16>() as usize)
                            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                            .wrapping_sub(1 as usize) as isize,
                    );
                *(&raw mut r28 as *mut uint8).offset(0 as ::core::ffi::c_int as isize) =
                    r44 as uint8;
            }
        }
    }
    let mut n: ::core::ffi::c_int = r52 >> 1 as ::core::ffi::c_int;
    while n != 0 {
        r24 = (r24 as ::core::ffi::c_int * 2 as ::core::ffi::c_int) as uint16;
        r38 = (r38 as ::core::ffi::c_int * 2 as ::core::ffi::c_int) as uint16;
        r26 = (r26 as ::core::ffi::c_int * 2 as ::core::ffi::c_int) as uint16;
        r40 = (r40 as ::core::ffi::c_int * 2 as ::core::ffi::c_int) as uint16;
        r28 = (r28 as ::core::ffi::c_int * 2 as ::core::ffi::c_int) as uint16;
        r42 = (r42 as ::core::ffi::c_int * 2 as ::core::ffi::c_int) as uint16;
        n -= 1;
    }
    UpdateMinimapInside(r18, r22, r34, r30, r32, r38, r24, r40, r26, r42, r28);
}
pub unsafe fn UpdateMinimapInside(
    mut r18: uint16,
    mut r22: uint16,
    mut r34: uint16,
    mut r30: uint16,
    mut r32: uint16,
    mut r38: uint16,
    mut r24: uint16,
    mut r40: uint16,
    mut r26: uint16,
    mut r42: uint16,
    mut r28: uint16,
) {
    let mut v0: uint16 = 0;
    let mut v1: int16 = 0;
    let mut v5: int16 = 0;
    let mut v7: int16 = 0;
    let mut v8: int16 = 0;
    *(&raw mut v0 as *mut uint8).offset(0 as ::core::ffi::c_int as isize) =
        ((r34 as ::core::ffi::c_int + r22 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int) as uint8;
    *(&raw mut v0 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>() as usize)
            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) = (r34 as ::core::ffi::c_int + r22 as ::core::ffi::c_int) as uint8;
    let mut t: uint16 = (r18 as ::core::ffi::c_int
        + (v0 as ::core::ffi::c_int >> 3 as ::core::ffi::c_int)) as uint16;
    if r34 as ::core::ffi::c_int != 0
        && (t as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int) - 2 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
    {
        v1 = (t as ::core::ffi::c_int - 1026 as ::core::ffi::c_int) as int16;
    } else {
        v1 = (t as ::core::ffi::c_int - 34 as ::core::ffi::c_int) as int16;
    }
    let mut v2: uint16 = (2 as ::core::ffi::c_int * v1 as ::core::ffi::c_int) as uint16;
    let mut r0: *const uint16 = RomPtr(Load24(
        (g_rom.offset(
            (((0x82964a as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x82964a as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut LongPtr)
            .offset(
                *((&raw mut g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize)
                    as *mut uint16) as isize,
            ) as *mut LongPtr,
    ) as uint32_t) as *const uint16;
    let mut r3: *const uint16 = r0.offset(32 as ::core::ffi::c_int as isize);
    let mut r6: *const uint16 = r0.offset(64 as ::core::ffi::c_int as isize);
    let mut n: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
    let mut v3: uint16 = 0 as uint16;
    loop {
        let mut v4: bool = r38 as ::core::ffi::c_int >> 15 as ::core::ffi::c_int != 0;
        r38 = (r38 as ::core::ffi::c_int * 2 as ::core::ffi::c_int) as uint16;
        if !v4 || {
            v5 =
                *r0.offset((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) as int16;
            *((&raw mut g_ram as *mut uint8).offset(0x789 as ::core::ffi::c_int as isize)
                as *mut uint16)
                == 0
        } {
            v5 = 31 as int16;
        }
        let mut v6: ::core::ffi::c_int = v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut g_ram as *mut uint8).offset(0xc608 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v6 + 26 as ::core::ffi::c_int) as isize) = (v5 as ::core::ffi::c_int
            & 0xc3ff as ::core::ffi::c_int
            | 0x2c00 as ::core::ffi::c_int)
            as uint16;
        v4 = r24 as ::core::ffi::c_int >> 15 as ::core::ffi::c_int != 0;
        r24 = (r24 as ::core::ffi::c_int * 2 as ::core::ffi::c_int) as uint16;
        if v4 {
            *((&raw mut g_ram as *mut uint8).offset(0xc608 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v6 + 26 as ::core::ffi::c_int) as isize) =
                (*r0.offset((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int
                    & 0xc3ff as ::core::ffi::c_int
                    | 0x2800 as ::core::ffi::c_int) as uint16;
        }
        v4 = r40 as ::core::ffi::c_int >> 15 as ::core::ffi::c_int != 0;
        r40 = (r40 as ::core::ffi::c_int * 2 as ::core::ffi::c_int) as uint16;
        if !v4 || {
            v7 =
                *r3.offset((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) as int16;
            *((&raw mut g_ram as *mut uint8).offset(0x789 as ::core::ffi::c_int as isize)
                as *mut uint16)
                == 0
        } {
            v7 = 31 as int16;
        }
        *((&raw mut g_ram as *mut uint8).offset(0xc608 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v6 + 58 as ::core::ffi::c_int) as isize) = (v7 as ::core::ffi::c_int
            & 0xc3ff as ::core::ffi::c_int
            | 0x2c00 as ::core::ffi::c_int)
            as uint16;
        v4 = r26 as ::core::ffi::c_int >> 15 as ::core::ffi::c_int != 0;
        r26 = (r26 as ::core::ffi::c_int * 2 as ::core::ffi::c_int) as uint16;
        if v4 {
            *((&raw mut g_ram as *mut uint8).offset(0xc608 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v6 + 58 as ::core::ffi::c_int) as isize) =
                (*r3.offset((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int
                    & 0xc3ff as ::core::ffi::c_int
                    | 0x2800 as ::core::ffi::c_int) as uint16;
            if n == 3 as ::core::ffi::c_int
                && *((&raw mut g_ram as *mut uint8).offset(0xc608 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset((v6 + 58 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int
                    & 0x1ff as ::core::ffi::c_int
                    == 40 as ::core::ffi::c_int
            {
                let ref mut fresh99 = *((&raw mut g_ram as *mut uint8)
                    .offset(0x7f3 as ::core::ffi::c_int as isize)
                    as *mut uint16 as *mut uint8)
                    .offset(r30 as ::core::ffi::c_int as isize);
                *fresh99 = (*fresh99 as ::core::ffi::c_int
                    | kShr0x80[r32 as usize] as ::core::ffi::c_int)
                    as uint8;
            }
        }
        v4 = r42 as ::core::ffi::c_int >> 15 as ::core::ffi::c_int != 0;
        r42 = (r42 as ::core::ffi::c_int * 2 as ::core::ffi::c_int) as uint16;
        if !v4 || {
            v8 =
                *r6.offset((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) as int16;
            *((&raw mut g_ram as *mut uint8).offset(0x789 as ::core::ffi::c_int as isize)
                as *mut uint16)
                == 0
        } {
            v8 = 31 as int16;
        }
        *((&raw mut g_ram as *mut uint8).offset(0xc608 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v6 + 90 as ::core::ffi::c_int) as isize) = (v8 as ::core::ffi::c_int
            & 0xc3ff as ::core::ffi::c_int
            | 0x2c00 as ::core::ffi::c_int)
            as uint16;
        v4 = r28 as ::core::ffi::c_int >> 15 as ::core::ffi::c_int != 0;
        r28 = (r28 as ::core::ffi::c_int * 2 as ::core::ffi::c_int) as uint16;
        if v4 {
            *((&raw mut g_ram as *mut uint8).offset(0xc608 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v6 + 90 as ::core::ffi::c_int) as isize) =
                (*r6.offset((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int
                    & 0xc3ff as ::core::ffi::c_int
                    | 0x2800 as ::core::ffi::c_int) as uint16;
        }
        v3 = (v3 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        v2 = (v2 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        if v2 as ::core::ffi::c_int & 0x3f as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            v2 = (v2 as ::core::ffi::c_int + 1984 as ::core::ffi::c_int) as uint16;
        }
        n -= 1;
        if !(n != 0) {
            break;
        }
    }
    if *(&raw mut g_ram as *mut uint8).offset(0x5b5 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & 8 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        let ref mut fresh100 = *((&raw mut g_ram as *mut uint8)
            .offset(0xc608 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(60 as ::core::ffi::c_int as isize);
        *fresh100 = (*fresh100 as ::core::ffi::c_int | 0x1c00 as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Samus_HandleCooldown() {
    if *((&raw mut g_ram as *mut uint8).offset(0xa78 as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        *((&raw mut g_ram as *mut uint8).offset(0xccc as ::core::ffi::c_int as isize)
            as *mut uint16) = 32 as uint16;
    } else if *((&raw mut g_ram as *mut uint8).offset(0xccc as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0
    {
        if *((&raw mut g_ram as *mut uint8).offset(0xccc as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
            || {
                let ref mut fresh82 = *((&raw mut g_ram as *mut uint8)
                    .offset(0xccc as ::core::ffi::c_int as isize)
                    as *mut uint16);
                *fresh82 = (*fresh82).wrapping_sub(1);
                *((&raw mut g_ram as *mut uint8).offset(0xccc as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int
            }
        {
            *((&raw mut g_ram as *mut uint8).offset(0xccc as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
        }
    }
}
pub unsafe fn Samus_CanFireBeam() -> uint8 {
    if *((&raw mut g_ram as *mut uint8).offset(0xcce as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - 5 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
        || *((&raw mut g_ram as *mut uint8).offset(0xccc as ::core::ffi::c_int as isize)
            as *mut uint16) as uint8 as ::core::ffi::c_int
            != 0
    {
        return 0 as uint8;
    }
    *((&raw mut g_ram as *mut uint8).offset(0xccc as ::core::ffi::c_int as isize) as *mut uint16) =
        1 as uint16;
    let ref mut fresh6 = *((&raw mut g_ram as *mut uint8)
        .offset(0xcce as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh6 = (*fresh6).wrapping_add(1);
    return 1 as uint8;
}
pub unsafe fn Samus_CanFireSuperMissile() -> uint8 {
    if *((&raw mut g_ram as *mut uint8).offset(0x9d2 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        != 2 as ::core::ffi::c_int
    {
        if *((&raw mut g_ram as *mut uint8).offset(0xcce as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 5 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            return 0 as uint8;
        }
    } else if !(*((&raw mut g_ram as *mut uint8).offset(0xcce as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - 4 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0)
    {
        return 0 as uint8;
    }
    if *((&raw mut g_ram as *mut uint8).offset(0xccc as ::core::ffi::c_int as isize) as *mut uint16)
        as uint8
        == 0
    {
        *((&raw mut g_ram as *mut uint8).offset(0xccc as ::core::ffi::c_int as isize)
            as *mut uint16) = 1 as uint16;
        let ref mut fresh7 = *((&raw mut g_ram as *mut uint8)
            .offset(0xcce as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh7 = (*fresh7).wrapping_add(1);
        return 1 as uint8;
    }
    return 0 as uint8;
}
pub unsafe fn UpdateBeamTilesAndPalette() {
    let mut v0: uint16 = (2 as ::core::ffi::c_int
        * (*((&raw mut g_ram as *mut uint8).offset(0x9a6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0xfff as ::core::ffi::c_int)) as uint16;
    let mut v1: uint16 = *((&raw mut g_ram as *mut uint8)
        .offset(0x330 as ::core::ffi::c_int as isize) as *mut uint16);
    (*gVramWriteEntry(
        *((&raw mut g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int,
    ))
    .size = 256 as uint16;
    v1 = (v1 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    (*gVramWriteEntry(v1 as ::core::ffi::c_int)).size = *(g_rom.offset(
        (((0x90c3b1 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x90c3b1 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    v1 = (v1 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    let fresh155 = v1;
    v1 = v1.wrapping_add(1);
    *(&raw mut (*(gVramWriteEntry as unsafe fn(::core::ffi::c_int) -> *mut VramWriteEntry)(
        fresh155 as ::core::ffi::c_int,
    ))
    .size as *mut uint8)
        .offset(0 as ::core::ffi::c_int as isize) = -(102 as ::core::ffi::c_int) as uint8;
    (*gVramWriteEntry(v1 as ::core::ffi::c_int)).size =
        addr_unk_606300 as ::core::ffi::c_int as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize) as *mut uint16) =
        (v1 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    WriteBeamPalette_Y(v0);
}
pub unsafe fn WriteBeamPalette_A(mut a: uint16) {
    WriteBeamPalette_Y(
        (2 as ::core::ffi::c_int * (a as ::core::ffi::c_int & 0xfff as ::core::ffi::c_int))
            as uint16,
    );
}
pub unsafe fn WriteBeamPalette_Y(mut j: uint16) {
    let mut r0: uint16 = *(g_rom.offset(
        (((0x90c3c9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x90c3c9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    let mut v1: uint16 = 0 as uint16;
    let mut v2: uint16 = 0 as uint16;
    loop {
        *((&raw mut g_ram as *mut uint8).offset(0xc000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(
                ((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 224 as ::core::ffi::c_int)
                    as isize,
            ) = *(RomPtr_90((r0 as ::core::ffi::c_int + v1 as ::core::ffi::c_int) as uint16_t)
            as *mut uint16);
        v2 = (v2 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        v1 = (v1 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        if !(((v1 as ::core::ffi::c_int - 32 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
}
pub unsafe fn LoadProjectilePalette(mut a: uint16) {
    let mut r0: uint16 = *(g_rom.offset(
        (((0x90c3c9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x90c3c9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset((a as ::core::ffi::c_int & 0xfff as ::core::ffi::c_int) as isize);
    let mut v1: uint16 = 0 as uint16;
    let mut v2: uint16 = 0 as uint16;
    loop {
        *((&raw mut g_ram as *mut uint8).offset(0xc000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(
                ((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 224 as ::core::ffi::c_int)
                    as isize,
            ) = *(RomPtr_90((r0 as ::core::ffi::c_int + v1 as ::core::ffi::c_int) as uint16_t)
            as *mut uint16);
        v2 = (v2 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        v1 = (v1 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        if !(((v1 as ::core::ffi::c_int - 32 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
}
pub unsafe fn ResetProjectileData() {
    let mut v0: uint16 = 0 as uint16;
    loop {
        let mut v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut g_ram as *mut uint8).offset(0xc90 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb64 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xc04 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xbdc as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xbf0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xbb4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xbc8 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xc2c as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xc40 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xc54 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xcb8 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xc68 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) =
            (0x90b169 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        if !(((v0 as ::core::ffi::c_int - 20 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int)
        {
            break;
        }
    }
    *((&raw mut g_ram as *mut uint8).offset(0xcd2 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xccc as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xcce as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xcee as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    if *((&raw mut g_ram as *mut uint8).offset(0x9ea as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        *((&raw mut g_ram as *mut uint8).offset(0x9d2 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xa04 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    }
    *((&raw mut g_ram as *mut uint8).offset(0xab0 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(0 as ::core::ffi::c_int as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xab0 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(1 as ::core::ffi::c_int as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xab0 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(2 as ::core::ffi::c_int as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xab0 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(3 as ::core::ffi::c_int as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xab8 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(0 as ::core::ffi::c_int as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xab8 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(1 as ::core::ffi::c_int as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xab8 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(2 as ::core::ffi::c_int as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xab8 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(3 as ::core::ffi::c_int as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xac0 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(0 as ::core::ffi::c_int as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xac0 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(1 as ::core::ffi::c_int as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xac0 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(2 as ::core::ffi::c_int as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xac0 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(3 as ::core::ffi::c_int as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xaae as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    if *((&raw mut g_ram as *mut uint8).offset(0xa4a as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        *((&raw mut g_ram as *mut uint8).offset(0xa4a as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        Samus_LoadSuitTargetPalette();
    }
    if *((&raw mut g_ram as *mut uint8).offset(0xa76 as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        SpawnPalfxObject(addr_stru_8DE1F0 as ::core::ffi::c_int as uint16);
    }
}
pub unsafe fn ClearProjectile(mut k: uint16) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut g_ram as *mut uint8).offset(0xb64 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb8c as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xba0 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xc04 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xbdc as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xbf0 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xbb4 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xbc8 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xc2c as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xc40 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xc54 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xcb8 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xc68 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) =
        (0x90b169 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    if (k as ::core::ffi::c_int - 10 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        let ref mut fresh0 = *((&raw mut g_ram as *mut uint8)
            .offset(0xcd2 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh0 = (*fresh0).wrapping_sub(1);
        if *fresh0 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            *((&raw mut g_ram as *mut uint8).offset(0xcd2 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
        }
    } else {
        let ref mut fresh1 = *((&raw mut g_ram as *mut uint8)
            .offset(0xcce as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh1 = (*fresh1).wrapping_sub(1);
        if *fresh1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            *((&raw mut g_ram as *mut uint8).offset(0xcce as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
        }
    };
}
static mut kKillProjectileFuncs: [Option<Func_Y_V>; 10] = unsafe {
    [
        Some(KillProjectileFunc_0 as unsafe fn(uint16) -> ()),
        Some(KillProjectileFunc_1 as unsafe fn(uint16) -> ()),
        Some(KillProjectileFunc_2 as unsafe fn(uint16) -> ()),
        Some(KillProjectileFunc_3 as unsafe fn(uint16) -> ()),
        Some(KillProjectileFunc_4 as unsafe fn(uint16) -> ()),
        Some(KillProjectileFunc_4 as unsafe fn(uint16) -> ()),
        Some(KillProjectileFunc_6 as unsafe fn(uint16) -> ()),
        Some(KillProjectileFunc_7 as unsafe fn(uint16) -> ()),
        Some(KillProjectileFunc_8 as unsafe fn(uint16) -> ()),
        Some(KillProjectileFunc_0 as unsafe fn(uint16) -> ()),
    ]
};
pub unsafe fn KillProjectile(mut k: uint16) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        & 0xf00 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        if (*(((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16 as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int)
            - 3 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            ClearProjectile(k);
            return;
        }
    } else {
        kKillProjectileFuncs[(*((&raw mut g_ram as *mut uint8)
            .offset(0xc04 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int) as usize]
            .expect("non-null function pointer")(k);
    }
    KillProjectileInner(k);
    *((&raw mut g_ram as *mut uint8).offset(0xc68 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        (0x90b169 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn KillProjectileFunc_0(mut j: uint16) {
    let ref mut fresh132 = *((&raw mut g_ram as *mut uint8)
        .offset(0xb78 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    *fresh132 = (*fresh132 as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0xbc8 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int) as uint16;
}
pub unsafe fn KillProjectileFunc_1(mut j: uint16) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh142 = *((&raw mut g_ram as *mut uint8)
        .offset(0xb64 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    *fresh142 = (*fresh142 as ::core::ffi::c_int
        + *((&raw mut g_ram as *mut uint8).offset(0xbb4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int) as uint16;
    let ref mut fresh143 = *((&raw mut g_ram as *mut uint8)
        .offset(0xb78 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    *fresh143 = (*fresh143 as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0xbc8 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int) as uint16;
}
pub unsafe fn KillProjectileFunc_2(mut j: uint16) {
    let ref mut fresh141 = *((&raw mut g_ram as *mut uint8)
        .offset(0xb64 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    *fresh141 = (*fresh141 as ::core::ffi::c_int
        + *((&raw mut g_ram as *mut uint8).offset(0xbb4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int) as uint16;
}
pub unsafe fn KillProjectileFunc_3(mut j: uint16) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh139 = *((&raw mut g_ram as *mut uint8)
        .offset(0xb64 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    *fresh139 = (*fresh139 as ::core::ffi::c_int
        + *((&raw mut g_ram as *mut uint8).offset(0xbb4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int) as uint16;
    let ref mut fresh140 = *((&raw mut g_ram as *mut uint8)
        .offset(0xb78 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    *fresh140 = (*fresh140 as ::core::ffi::c_int
        + *((&raw mut g_ram as *mut uint8).offset(0xbc8 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int) as uint16;
}
pub unsafe fn KillProjectileFunc_4(mut j: uint16) {
    let ref mut fresh138 = *((&raw mut g_ram as *mut uint8)
        .offset(0xb78 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    *fresh138 = (*fresh138 as ::core::ffi::c_int
        + *((&raw mut g_ram as *mut uint8).offset(0xbc8 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int) as uint16;
}
pub unsafe fn KillProjectileFunc_6(mut j: uint16) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh136 = *((&raw mut g_ram as *mut uint8)
        .offset(0xb64 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    *fresh136 = (*fresh136 as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0xbb4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int) as uint16;
    let ref mut fresh137 = *((&raw mut g_ram as *mut uint8)
        .offset(0xb78 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    *fresh137 = (*fresh137 as ::core::ffi::c_int
        + *((&raw mut g_ram as *mut uint8).offset(0xbc8 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int) as uint16;
}
pub unsafe fn KillProjectileFunc_7(mut j: uint16) {
    let ref mut fresh135 = *((&raw mut g_ram as *mut uint8)
        .offset(0xb64 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    *fresh135 = (*fresh135 as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0xbb4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int) as uint16;
}
pub unsafe fn KillProjectileFunc_8(mut j: uint16) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh133 = *((&raw mut g_ram as *mut uint8)
        .offset(0xb64 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    *fresh133 = (*fresh133 as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0xbb4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int) as uint16;
    let ref mut fresh134 = *((&raw mut g_ram as *mut uint8)
        .offset(0xb78 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    *fresh134 = (*fresh134 as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0xbc8 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int) as uint16;
}
pub unsafe fn CallProjPreInstr(mut ea: uint32, mut k: uint16) {
    match ea {
        9482601 => return,
        9481971 => {
            ProjPreInstr_Beam_NoWaveBeam(k);
            return;
        }
        9482058 => {
            ProjPreInstr_Dir0459(k);
            return;
        }
        9482066 => {
            ProjPreInstr_Dir1368(k);
            return;
        }
        9482080 => {
            ProjPreInstr_Dir27(k);
            return;
        }
        9482088 => {
            ProjPreInstr_Missile(k);
            return;
        }
        9482183 => {
            ProjPreInstr_Missile_Func0459(k);
            return;
        }
        9482191 => {
            ProjPreInstr_Missile_Func1368(k);
            return;
        }
        9482205 => {
            ProjPreInstr_Missile_Func27(k);
            return;
        }
        9482213 => {
            ProjPreInstr_SuperMissile(k);
            return;
        }
        9482311 => {
            ProjPreInstr_SuperMissile_Func0459(k);
            return;
        }
        9482322 => {
            ProjPreInstr_SuperMissile_Func1368(k);
            return;
        }
        9482346 => {
            ProjPreInstr_SuperMissile_Func27(k);
            return;
        }
        9482357 => {
            ProjPreInstr_Func1(k);
            return;
        }
        9482393 => {
            ProjPreInstr_Bomb(k);
            return;
        }
        9482414 => {
            ProjPreInstr_PowerBomb(k);
            return;
        }
        9482435 => {
            ProjPreInstr_WavePlasmaEtc(k);
            return;
        }
        9482468 => {
            ProjPreInstr_BeamOrIceWave(k);
            return;
        }
        9482499 => {
            ProjPreInstr_Wave_Shared(k);
            return;
        }
        9482555 => {
            ProjPreInstr_WavePlasmaEtc_0459(k);
            return;
        }
        9482563 => {
            ProjPreInstr_WavePlasmaEtc_1368(k);
            return;
        }
        9482577 => {
            ProjPreInstr_WavePlasmaEtc_27(k);
            return;
        }
        9482585 => {
            ProjPreInstr_HyperBeam(k);
            return;
        }
        9490185 => {
            ProjPreInstr_IceSba(k);
            return;
        }
        9490298 => {
            ProjPreInstr_IceSba2(k);
            return;
        }
        9491666 => {
            ProjPreInstr_SpeedEcho(k);
            return;
        }
        9492371 => {
            ProjPreInstr_PlasmaSba(k);
            return;
        }
        9492449 => {
            ProjPreInstr_PlasmaSbaFunc_0(k);
            return;
        }
        9492474 => {
            ProjPreInstr_PlasmaSbaFunc_1(k);
            return;
        }
        9492499 => {
            ProjPreInstr_PlasmaSbaFunc_2(k);
            return;
        }
        9492727 => {
            ProjPreInstr_SpreadBomb(k);
            return;
        }
        9493000 => {
            ProjPreInstr_WaveSba(k);
            return;
        }
        9493254 => {
            ProjPreInstr_SpazerSba(k);
            return;
        }
        9493660 => {
            ProjPreInstr_EndOfSpazerSba(k);
            return;
        }
        9498579 => {
            ProjPreInstr_UnknownProj8027(k);
            return;
        }
        _ => {
            Unreachable();
        }
    };
}
pub unsafe fn HandleProjectile() {
    *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize) as *mut uint16) =
        18 as uint16;
    let mut i: ::core::ffi::c_int = 18 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        let mut v1: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
        if *((&raw mut g_ram as *mut uint8).offset(0xc40 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize)
            != 0
        {
            CallProjPreInstr(
                (*((&raw mut g_ram as *mut uint8).offset(0xc68 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) as ::core::ffi::c_int
                    | 0x900000 as ::core::ffi::c_int) as uint32,
                i as uint16,
            );
            RunProjectileInstructions();
            i = *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int;
        }
        i -= 2 as ::core::ffi::c_int;
        *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
            as *mut uint16) = i as uint16;
    }
}
static mut kDirToVelMult16_X: [int16; 10] = [
    0 as ::core::ffi::c_int as int16,
    16 as ::core::ffi::c_int as int16,
    16 as ::core::ffi::c_int as int16,
    16 as ::core::ffi::c_int as int16,
    0 as ::core::ffi::c_int as int16,
    0 as ::core::ffi::c_int as int16,
    -(16 as ::core::ffi::c_int) as int16,
    -(16 as ::core::ffi::c_int) as int16,
    -(16 as ::core::ffi::c_int) as int16,
    0 as ::core::ffi::c_int as int16,
];
static mut kDirToVelMult16_Y: [int16; 10] = [
    -(16 as ::core::ffi::c_int) as int16,
    -(16 as ::core::ffi::c_int) as int16,
    0 as ::core::ffi::c_int as int16,
    16 as ::core::ffi::c_int as int16,
    16 as ::core::ffi::c_int as int16,
    16 as ::core::ffi::c_int as int16,
    16 as ::core::ffi::c_int as int16,
    0 as ::core::ffi::c_int as int16,
    -(16 as ::core::ffi::c_int) as int16,
    -(16 as ::core::ffi::c_int) as int16,
];
static mut kProjPreInstr_Beam_Funcs: [Option<Func_Y_V>; 10] = unsafe {
    [
        Some(ProjPreInstr_Dir0459 as unsafe fn(uint16) -> ()),
        Some(ProjPreInstr_Dir1368 as unsafe fn(uint16) -> ()),
        Some(ProjPreInstr_Dir27 as unsafe fn(uint16) -> ()),
        Some(ProjPreInstr_Dir1368 as unsafe fn(uint16) -> ()),
        Some(ProjPreInstr_Dir0459 as unsafe fn(uint16) -> ()),
        Some(ProjPreInstr_Dir0459 as unsafe fn(uint16) -> ()),
        Some(ProjPreInstr_Dir1368 as unsafe fn(uint16) -> ()),
        Some(ProjPreInstr_Dir27 as unsafe fn(uint16) -> ()),
        Some(ProjPreInstr_Dir1368 as unsafe fn(uint16) -> ()),
        Some(ProjPreInstr_Dir0459 as unsafe fn(uint16) -> ()),
    ]
};
pub unsafe fn ProjPreInstr_Beam_NoWaveBeam(mut k: uint16) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut g_ram as *mut uint8).offset(0xc04 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        & 0xf0 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        ClearProjectile(k);
    } else {
        let ref mut fresh72 = *((&raw mut g_ram as *mut uint8)
            .offset(0xc90 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        let fresh73 = *fresh72;
        *fresh72 = (*fresh72).wrapping_sub(1);
        if fresh73 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
            *((&raw mut g_ram as *mut uint8).offset(0xc90 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = 4 as uint16;
            SpawnProjectileTrail(k);
            k = *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
                as *mut uint16);
        }
        let mut v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v4: uint16 = (2 as ::core::ffi::c_int
            * (*((&raw mut g_ram as *mut uint8).offset(0xc04 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v3 as isize) as ::core::ffi::c_int
                & 0xf as ::core::ffi::c_int)) as uint16;
        let mut v5: ::core::ffi::c_int = v4 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let ref mut fresh74 = *((&raw mut g_ram as *mut uint8)
            .offset(0xbdc as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v3 as isize);
        *fresh74 = (*fresh74 as ::core::ffi::c_int
            + kDirToVelMult16_X[v5 as usize] as ::core::ffi::c_int) as uint16;
        let ref mut fresh75 = *((&raw mut g_ram as *mut uint8)
            .offset(0xbf0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v3 as isize);
        *fresh75 = (*fresh75 as ::core::ffi::c_int
            + kDirToVelMult16_Y[v5 as usize] as ::core::ffi::c_int) as uint16;
        kProjPreInstr_Beam_Funcs[v5 as usize].expect("non-null function pointer")(v4);
        DeleteProjectileIfFarOffScreen();
    };
}
pub unsafe fn ProjPreInstr_Dir0459(mut k: uint16) {
    BlockCollNoWaveBeamVert(
        *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
}
pub unsafe fn ProjPreInstr_Dir1368(mut k: uint16) {
    let mut v1: uint16 = *((&raw mut g_ram as *mut uint8)
        .offset(0xdde as ::core::ffi::c_int as isize) as *mut uint16);
    if BlockCollNoWaveBeamHoriz(
        *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
            as *mut uint16),
    ) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        == 0
    {
        BlockCollNoWaveBeamVert(v1);
    }
}
pub unsafe fn ProjPreInstr_Dir27(mut k: uint16) {
    BlockCollNoWaveBeamHoriz(
        *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
}
static mut kProjPreInstr_Missile_Funcs: [Option<Func_Y_V>; 10] = unsafe {
    [
        Some(ProjPreInstr_Missile_Func0459 as unsafe fn(uint16) -> ()),
        Some(ProjPreInstr_Missile_Func1368 as unsafe fn(uint16) -> ()),
        Some(ProjPreInstr_Missile_Func27 as unsafe fn(uint16) -> ()),
        Some(ProjPreInstr_Missile_Func1368 as unsafe fn(uint16) -> ()),
        Some(ProjPreInstr_Missile_Func0459 as unsafe fn(uint16) -> ()),
        Some(ProjPreInstr_Missile_Func0459 as unsafe fn(uint16) -> ()),
        Some(ProjPreInstr_Missile_Func1368 as unsafe fn(uint16) -> ()),
        Some(ProjPreInstr_Missile_Func27 as unsafe fn(uint16) -> ()),
        Some(ProjPreInstr_Missile_Func1368 as unsafe fn(uint16) -> ()),
        Some(ProjPreInstr_Missile_Func0459 as unsafe fn(uint16) -> ()),
    ]
};
pub unsafe fn ProjPreInstr_Missile(mut k: uint16) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut g_ram as *mut uint8).offset(0xc04 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        & 0xf0 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        ClearProjectile(k);
    } else {
        let ref mut fresh68 = *((&raw mut g_ram as *mut uint8)
            .offset(0xc90 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        let fresh69 = *fresh68;
        *fresh68 = (*fresh68).wrapping_sub(1);
        if fresh69 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
            *((&raw mut g_ram as *mut uint8).offset(0xc90 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = 4 as uint16;
            SpawnProjectileTrail(k);
            k = *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
                as *mut uint16);
        }
        let mut v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v4: uint16 = (2 as ::core::ffi::c_int
            * (*((&raw mut g_ram as *mut uint8).offset(0xc04 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v3 as isize) as ::core::ffi::c_int
                & 0xf as ::core::ffi::c_int)) as uint16;
        let mut v5: ::core::ffi::c_int = v4 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let ref mut fresh70 = *((&raw mut g_ram as *mut uint8)
            .offset(0xbdc as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v3 as isize);
        *fresh70 = (*fresh70 as ::core::ffi::c_int
            + kDirToVelMult16_X[v5 as usize] as ::core::ffi::c_int) as uint16;
        let ref mut fresh71 = *((&raw mut g_ram as *mut uint8)
            .offset(0xbf0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v3 as isize);
        *fresh71 = (*fresh71 as ::core::ffi::c_int
            + kDirToVelMult16_Y[v5 as usize] as ::core::ffi::c_int) as uint16;
        Missile_Func1(k);
        kProjPreInstr_Missile_Funcs[(v4 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize]
            .expect("non-null function pointer")(v4);
        DeleteProjectileIfFarOffScreen();
    };
}
pub unsafe fn ProjPreInstr_Missile_Func0459(mut k: uint16) {
    BlockCollMissileVert(
        *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
}
pub unsafe fn ProjPreInstr_Missile_Func1368(mut k: uint16) {
    let mut v1: uint16 = *((&raw mut g_ram as *mut uint8)
        .offset(0xdde as ::core::ffi::c_int as isize) as *mut uint16);
    if BlockCollMissileHoriz(
        *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
            as *mut uint16),
    ) == 0
    {
        BlockCollMissileVert(v1);
    }
}
pub unsafe fn ProjPreInstr_Missile_Func27(mut k: uint16) {
    BlockCollMissileHoriz(
        *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
}
static mut kProjPreInstr_SuperMissile_Funcs: [Option<Func_Y_V>; 10] = unsafe {
    [
        Some(ProjPreInstr_SuperMissile_Func0459 as unsafe fn(uint16) -> ()),
        Some(ProjPreInstr_SuperMissile_Func1368 as unsafe fn(uint16) -> ()),
        Some(ProjPreInstr_SuperMissile_Func27 as unsafe fn(uint16) -> ()),
        Some(ProjPreInstr_SuperMissile_Func1368 as unsafe fn(uint16) -> ()),
        Some(ProjPreInstr_SuperMissile_Func0459 as unsafe fn(uint16) -> ()),
        Some(ProjPreInstr_SuperMissile_Func0459 as unsafe fn(uint16) -> ()),
        Some(ProjPreInstr_SuperMissile_Func1368 as unsafe fn(uint16) -> ()),
        Some(ProjPreInstr_SuperMissile_Func27 as unsafe fn(uint16) -> ()),
        Some(ProjPreInstr_SuperMissile_Func1368 as unsafe fn(uint16) -> ()),
        Some(ProjPreInstr_SuperMissile_Func0459 as unsafe fn(uint16) -> ()),
    ]
};
pub unsafe fn ProjPreInstr_SuperMissile(mut k: uint16) {
    let mut v3: int16 = 0;
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut g_ram as *mut uint8).offset(0xc04 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        & 0xf0 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        ClearProjectile(k);
    } else {
        let ref mut fresh62 = *((&raw mut g_ram as *mut uint8)
            .offset(0xc90 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        let fresh63 = *fresh62;
        *fresh62 = (*fresh62).wrapping_sub(1);
        if fresh63 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
            *((&raw mut g_ram as *mut uint8).offset(0xc90 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = 2 as uint16;
            SpawnProjectileTrail(k);
            k = *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
                as *mut uint16);
        }
        v3 = (*((&raw mut g_ram as *mut uint8).offset(0xc04 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int) as int16;
        Missile_Func1(k);
        kProjPreInstr_SuperMissile_Funcs[v3 as usize].expect("non-null function pointer")(
            v3 as uint16,
        );
        if !(DeleteProjectileIfFarOffScreen() as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0)
        {
            return;
        }
    }
    let mut i: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        if *((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((i >> 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
            & 0xfff as ::core::ffi::c_int
            == 512 as ::core::ffi::c_int
        {
            ClearProjectile(i as uint16);
        }
        i -= 2 as ::core::ffi::c_int;
    }
}
pub unsafe fn ProjPreInstr_SuperMissile_Func0459(mut k: uint16) {
    BlockCollMissileVert(
        *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
    SuperMissileBlockCollDetect_Y();
}
pub unsafe fn ProjPreInstr_SuperMissile_Func1368(mut k: uint16) {
    let mut v1: uint16 = *((&raw mut g_ram as *mut uint8)
        .offset(0xdde as ::core::ffi::c_int as isize) as *mut uint16);
    if BlockCollMissileHoriz(
        *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
            as *mut uint16),
    ) != 0
    {
        SuperMissileBlockCollDetect_X();
    } else {
        SuperMissileBlockCollDetect_X();
        BlockCollMissileVert(v1);
        SuperMissileBlockCollDetect_Y();
    };
}
pub unsafe fn ProjPreInstr_SuperMissile_Func27(mut k: uint16) {
    BlockCollMissileHoriz(
        *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
    SuperMissileBlockCollDetect_X();
}
pub unsafe fn ProjPreInstr_Func1(mut k: uint16) {
    if *((&raw mut g_ram as *mut uint8).offset(0xc04 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        & 0xf0 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        ClearProjectile(k);
        let mut i: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
        while i >= 0 as ::core::ffi::c_int {
            if *((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((i >> 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int
                & 0xfff as ::core::ffi::c_int
                == 512 as ::core::ffi::c_int
            {
                ClearProjectile(i as uint16);
            }
            i -= 2 as ::core::ffi::c_int;
        }
    }
}
pub unsafe fn ProjPreInstr_Bomb(mut k: uint16) {
    if *((&raw mut g_ram as *mut uint8).offset(0xc04 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        & 0xf0 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        ClearProjectile(k);
    } else {
        Bomb_Func2();
        BombOrPowerBomb_Func1(k);
    };
}
pub unsafe fn ProjPreInstr_PowerBomb(mut k: uint16) {
    if *((&raw mut g_ram as *mut uint8).offset(0xc04 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        & 0xf0 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        ClearProjectile(k);
    } else {
        PowerBomb_Func3();
        BombOrPowerBomb_Func1(k);
    };
}
pub unsafe fn ProjPreInstr_WavePlasmaEtc(mut k: uint16) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut g_ram as *mut uint8).offset(0xc04 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        & 0xf0 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        ClearProjectile(k);
    } else {
        let ref mut fresh59 = *((&raw mut g_ram as *mut uint8)
            .offset(0xc90 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        let fresh60 = *fresh59;
        *fresh59 = (*fresh59).wrapping_sub(1);
        if fresh60 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
            *((&raw mut g_ram as *mut uint8).offset(0xc90 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = 4 as uint16;
            SpawnProjectileTrail(k);
            ProjPreInstr_Wave_Shared(
                *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
                    as *mut uint16),
            );
        } else {
            ProjPreInstr_Wave_Shared(k);
        }
    };
}
pub unsafe fn ProjPreInstr_BeamOrIceWave(mut k: uint16) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut g_ram as *mut uint8).offset(0xc04 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        & 0xf0 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        ClearProjectile(k);
    } else {
        let ref mut fresh57 = *((&raw mut g_ram as *mut uint8)
            .offset(0xc90 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        let fresh58 = *fresh57;
        *fresh57 = (*fresh57).wrapping_sub(1);
        if fresh58 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
            *((&raw mut g_ram as *mut uint8).offset(0xc90 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = 3 as uint16;
            SpawnProjectileTrail(k);
            ProjPreInstr_Wave_Shared(
                *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
                    as *mut uint16),
            );
        } else {
            ProjPreInstr_Wave_Shared(k);
        }
    };
}
static mut kProjPreInstr_WavePlasmaEtcFuncs: [Option<Func_Y_V>; 10] = unsafe {
    [
        Some(ProjPreInstr_WavePlasmaEtc_0459 as unsafe fn(uint16) -> ()),
        Some(ProjPreInstr_WavePlasmaEtc_1368 as unsafe fn(uint16) -> ()),
        Some(ProjPreInstr_WavePlasmaEtc_27 as unsafe fn(uint16) -> ()),
        Some(ProjPreInstr_WavePlasmaEtc_1368 as unsafe fn(uint16) -> ()),
        Some(ProjPreInstr_WavePlasmaEtc_0459 as unsafe fn(uint16) -> ()),
        Some(ProjPreInstr_WavePlasmaEtc_0459 as unsafe fn(uint16) -> ()),
        Some(ProjPreInstr_WavePlasmaEtc_1368 as unsafe fn(uint16) -> ()),
        Some(ProjPreInstr_WavePlasmaEtc_27 as unsafe fn(uint16) -> ()),
        Some(ProjPreInstr_WavePlasmaEtc_1368 as unsafe fn(uint16) -> ()),
        Some(ProjPreInstr_WavePlasmaEtc_0459 as unsafe fn(uint16) -> ()),
    ]
};
pub unsafe fn ProjPreInstr_Wave_Shared(mut k: uint16) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v2: uint16 = (2 as ::core::ffi::c_int
        * (*((&raw mut g_ram as *mut uint8).offset(0xc04 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int)) as uint16;
    let mut v3: ::core::ffi::c_int = v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh55 = *((&raw mut g_ram as *mut uint8)
        .offset(0xbdc as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    *fresh55 = (*fresh55 as ::core::ffi::c_int
        + kDirToVelMult16_X[v3 as usize] as ::core::ffi::c_int) as uint16;
    let ref mut fresh56 = *((&raw mut g_ram as *mut uint8)
        .offset(0xbf0 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    *fresh56 = (*fresh56 as ::core::ffi::c_int
        + kDirToVelMult16_Y[v3 as usize] as ::core::ffi::c_int) as uint16;
    kProjPreInstr_WavePlasmaEtcFuncs[v3 as usize].expect("non-null function pointer")(v2);
    DeleteProjectileIfFarOffScreen();
}
pub unsafe fn ProjPreInstr_WavePlasmaEtc_0459(mut k: uint16) {
    BlockCollWaveBeamVert(
        *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
}
pub unsafe fn ProjPreInstr_WavePlasmaEtc_1368(mut k: uint16) {
    let mut v1: uint16 = *((&raw mut g_ram as *mut uint8)
        .offset(0xdde as ::core::ffi::c_int as isize) as *mut uint16);
    if BlockCollWaveBeamHoriz(
        *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
            as *mut uint16),
    ) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        == 0
    {
        BlockCollWaveBeamVert(v1);
    }
}
pub unsafe fn ProjPreInstr_WavePlasmaEtc_27(mut k: uint16) {
    BlockCollWaveBeamHoriz(
        *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
}
pub unsafe fn ProjPreInstr_HyperBeam(mut k: uint16) {
    if *((&raw mut g_ram as *mut uint8).offset(0xc04 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        & 0xf0 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        ClearProjectile(k);
    } else {
        ProjPreInstr_Wave_Shared(k);
    };
}
pub unsafe fn DeleteProjectileIfFarOffScreen() -> uint8 {
    let mut v0: int16 = 0;
    let mut v2: int16 = 0;
    v0 = (*((&raw mut g_ram as *mut uint8).offset(0xb64 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(
            (*((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as isize,
        ) as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16;
    if v0 as ::core::ffi::c_int + 64 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
        if v0 as ::core::ffi::c_int - 320 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0
        {
            v2 = (*((&raw mut g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(
                    (*((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        >> 1 as ::core::ffi::c_int) as isize,
                ) as ::core::ffi::c_int
                - *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as int16;
            if v2 as ::core::ffi::c_int + 64 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == 0
            {
                if v2 as ::core::ffi::c_int - 320 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    != 0
                {
                    return 0 as uint8;
                }
            }
        }
    }
    ClearProjectile(
        *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
    return 1 as uint8;
}
static mut kInitializeProjectileSpeed_XY_Diag: [uint16; 24] = [
    0x400 as ::core::ffi::c_int as uint16,
    0x2ab as ::core::ffi::c_int as uint16,
    0x400 as ::core::ffi::c_int as uint16,
    0x2ab as ::core::ffi::c_int as uint16,
    0x400 as ::core::ffi::c_int as uint16,
    0x2ab as ::core::ffi::c_int as uint16,
    0x400 as ::core::ffi::c_int as uint16,
    0x2ab as ::core::ffi::c_int as uint16,
    0x400 as ::core::ffi::c_int as uint16,
    0x2ab as ::core::ffi::c_int as uint16,
    0x400 as ::core::ffi::c_int as uint16,
    0x2ab as ::core::ffi::c_int as uint16,
    0x400 as ::core::ffi::c_int as uint16,
    0x2ab as ::core::ffi::c_int as uint16,
    0x400 as ::core::ffi::c_int as uint16,
    0x2ab as ::core::ffi::c_int as uint16,
    0x400 as ::core::ffi::c_int as uint16,
    0x2ab as ::core::ffi::c_int as uint16,
    0x400 as ::core::ffi::c_int as uint16,
    0x2ab as ::core::ffi::c_int as uint16,
    0x400 as ::core::ffi::c_int as uint16,
    0x2ab as ::core::ffi::c_int as uint16,
    0x400 as ::core::ffi::c_int as uint16,
    0x2ab as ::core::ffi::c_int as uint16,
];
pub unsafe fn SetInitialProjectileSpeed(mut r20: uint16) {
    let mut v0: ::core::ffi::c_int = r20 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v1: uint16 = (4 as ::core::ffi::c_int
        * (*((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v0 as isize) as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int)) as uint16;
    let mut v2: uint16 = (2 as ::core::ffi::c_int
        * (*((&raw mut g_ram as *mut uint8).offset(0xc04 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v0 as isize) as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int)) as uint16;
    let mut r22: uint16 = 0;
    if v2 == 0
        || v2 as ::core::ffi::c_int == 4 as ::core::ffi::c_int
        || v2 as ::core::ffi::c_int == 8 as ::core::ffi::c_int
        || v2 as ::core::ffi::c_int == 10 as ::core::ffi::c_int
        || v2 as ::core::ffi::c_int == 14 as ::core::ffi::c_int
        || v2 as ::core::ffi::c_int == 18 as ::core::ffi::c_int
    {
        r22 = kInitializeProjectileSpeed_XY_Diag
            [(v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize];
    } else {
        if v2 as ::core::ffi::c_int != 2 as ::core::ffi::c_int
            && v2 as ::core::ffi::c_int != 6 as ::core::ffi::c_int
            && v2 as ::core::ffi::c_int != 12 as ::core::ffi::c_int
            && v2 as ::core::ffi::c_int != 16 as ::core::ffi::c_int
        {
            Unreachable();
        }
        r22 = kInitializeProjectileSpeed_XY_Diag[((v1 as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int)
            + 1 as ::core::ffi::c_int) as usize];
    }
    InitializeProjectileSpeed(r20, r22);
}
pub unsafe fn InitializeProjectileSpeedOfType(mut r20: uint16) {
    InitializeProjectileSpeed(r20, 0 as uint16);
}
pub unsafe fn InitializeProjectileSpeed(mut k: uint16, mut r22: uint16) {
    let mut r18: uint16 = 0;
    let mut kh: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut g_ram as *mut uint8).offset(0xb8c as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(kh as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xba0 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(kh as isize) = 0 as uint16;
    match 2 as ::core::ffi::c_int
        * (*((&raw mut g_ram as *mut uint8).offset(0xc04 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(kh as isize) as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int)
    {
        0 | 18 => {
            if *((&raw mut g_ram as *mut uint8).offset(0xdb2 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint8
                != 0
            {
                r18 = (*(((&raw mut g_ram as *mut uint8)
                    .offset(0xdb0 as ::core::ffi::c_int as isize)
                    as *mut uint16 as *mut uint8)
                    .offset(1 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    >> 2 as ::core::ffi::c_int
                    | 0xc000 as ::core::ffi::c_int) as uint16;
            } else {
                r18 = 0 as uint16;
            }
            *((&raw mut g_ram as *mut uint8).offset(0xbf0 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(kh as isize) =
                (r18 as ::core::ffi::c_int - r22 as ::core::ffi::c_int) as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xbdc as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(kh as isize) = 0 as uint16;
        }
        2 => {
            if *((&raw mut g_ram as *mut uint8).offset(0xdb2 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint8
                != 0
            {
                r18 = (*(((&raw mut g_ram as *mut uint8)
                    .offset(0xdb0 as ::core::ffi::c_int as isize)
                    as *mut uint16 as *mut uint8)
                    .offset(1 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    >> 2 as ::core::ffi::c_int
                    | 0xc000 as ::core::ffi::c_int) as uint16;
            } else {
                r18 = 0 as uint16;
            }
            *((&raw mut g_ram as *mut uint8).offset(0xbf0 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(kh as isize) =
                (r18 as ::core::ffi::c_int - r22 as ::core::ffi::c_int) as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xbdc as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(kh as isize) = (*(((&raw mut g_ram as *mut uint8)
                .offset(0xdac as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(1 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + r22 as ::core::ffi::c_int) as uint16;
        }
        4 => {
            *((&raw mut g_ram as *mut uint8).offset(0xbf0 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(kh as isize) = 0 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xbdc as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(kh as isize) = (*(((&raw mut g_ram as *mut uint8)
                .offset(0xdac as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(1 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + r22 as ::core::ffi::c_int) as uint16;
        }
        6 => {
            *((&raw mut g_ram as *mut uint8).offset(0xbf0 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(kh as isize) = (*(((&raw mut g_ram as *mut uint8)
                .offset(0xdb4 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(1 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + r22 as ::core::ffi::c_int) as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xbdc as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(kh as isize) = (*(((&raw mut g_ram as *mut uint8)
                .offset(0xdac as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(1 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + r22 as ::core::ffi::c_int) as uint16;
        }
        8 | 10 => {
            *((&raw mut g_ram as *mut uint8).offset(0xbf0 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(kh as isize) = (*(((&raw mut g_ram as *mut uint8)
                .offset(0xdb4 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(1 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + r22 as ::core::ffi::c_int) as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xbdc as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(kh as isize) = 0 as uint16;
        }
        12 => {
            *((&raw mut g_ram as *mut uint8).offset(0xbf0 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(kh as isize) = (*(((&raw mut g_ram as *mut uint8)
                .offset(0xdb4 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(1 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + r22 as ::core::ffi::c_int) as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xbdc as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(kh as isize) = (*(((&raw mut g_ram as *mut uint8)
                .offset(0xda8 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(1 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - r22 as ::core::ffi::c_int) as uint16;
        }
        14 => {
            *((&raw mut g_ram as *mut uint8).offset(0xbf0 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(kh as isize) = 0 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xbdc as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(kh as isize) = (*(((&raw mut g_ram as *mut uint8)
                .offset(0xda8 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(1 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - r22 as ::core::ffi::c_int) as uint16;
        }
        16 => {
            if *((&raw mut g_ram as *mut uint8).offset(0xdb2 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint8
                != 0
            {
                r18 = (*(((&raw mut g_ram as *mut uint8)
                    .offset(0xdb0 as ::core::ffi::c_int as isize)
                    as *mut uint16 as *mut uint8)
                    .offset(1 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    >> 2 as ::core::ffi::c_int
                    | 0xc000 as ::core::ffi::c_int) as uint16;
            } else {
                r18 = 0 as uint16;
            }
            *((&raw mut g_ram as *mut uint8).offset(0xbf0 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(kh as isize) =
                (r18 as ::core::ffi::c_int - r22 as ::core::ffi::c_int) as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xbdc as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(kh as isize) = (*(((&raw mut g_ram as *mut uint8)
                .offset(0xda8 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(1 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - r22 as ::core::ffi::c_int) as uint16;
        }
        _ => {
            Unreachable();
        }
    };
}
pub unsafe fn Missile_Func1(mut k: uint16) {
    static mut word_90C301: uint16 = 0x100 as uint16;
    let mut v3: uint16 = 0;
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        & 0xff00 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        if *((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            & 0x200 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            v3 = addr_kSuperMissileAccelerations2 as ::core::ffi::c_int as uint16;
        } else {
            v3 = addr_kSuperMissileAccelerations as ::core::ffi::c_int as uint16;
        }
        let mut v4: *const uint8 = RomPtr_90(
            (v3 as ::core::ffi::c_int
                + 4 as ::core::ffi::c_int
                    * (*((&raw mut g_ram as *mut uint8).offset(0xc04 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset(v1 as isize) as ::core::ffi::c_int
                        & 0xf as ::core::ffi::c_int)) as uint16_t,
        );
        let ref mut fresh64 = *((&raw mut g_ram as *mut uint8)
            .offset(0xbdc as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        *fresh64 =
            (*fresh64 as ::core::ffi::c_int + *(v4 as *mut uint16) as ::core::ffi::c_int) as uint16;
        let ref mut fresh65 = *((&raw mut g_ram as *mut uint8)
            .offset(0xbf0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        *fresh65 = (*fresh65 as ::core::ffi::c_int
            + *(v4.offset(2 as ::core::ffi::c_int as isize) as *mut uint16) as ::core::ffi::c_int)
            as uint16;
    } else {
        let mut v2: uint16 = (word_90C301 as ::core::ffi::c_int
            + *((&raw mut g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = v2;
        if v2 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            InitializeProjectileSpeed(
                k,
                *((&raw mut g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize),
            );
            if *((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                & 0x200 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                Missile_Func2();
            }
        }
    };
}
pub unsafe fn SuperMissileBlockCollDetect_Y() {
    let mut v0: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
        .offset(0xdde as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    if *((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v0 as isize) as ::core::ffi::c_int
        & 0xf00 as ::core::ffi::c_int
        == 512 as ::core::ffi::c_int
        || *((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v0 as isize) as ::core::ffi::c_int
            & 0xf00 as ::core::ffi::c_int
            == 2048 as ::core::ffi::c_int
    {
        let mut v5: uint8 = *((&raw mut g_ram as *mut uint8)
            .offset(0xc7c as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v0 as isize) as uint8;
        if *((&raw mut g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(
                *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int as isize,
            )
            .offset(1 as ::core::ffi::c_int as isize)
            != 0
        {
            let mut v1: uint16 = (abs16(
                *((&raw mut g_ram as *mut uint8).offset(0xbf0 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v0 as isize),
            ) as ::core::ffi::c_int
                & 0xff00 as ::core::ffi::c_int) as uint16;
            if v1 as ::core::ffi::c_int - 2816 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0
            {
                let mut v4: ::core::ffi::c_int =
                    *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        >> 1 as ::core::ffi::c_int;
                if *((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v4 as isize) as ::core::ffi::c_int
                    & 0xf00 as ::core::ffi::c_int
                    != 2048 as ::core::ffi::c_int
                {
                    *((&raw mut g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset((v5 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                        *((&raw mut g_ram as *mut uint8)
                            .offset(0xb78 as ::core::ffi::c_int as isize)
                            as *mut uint16)
                            .offset(v4 as isize);
                    return;
                }
            } else {
                let mut r18: uint16 = ((v1 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int)
                    - 10 as ::core::ffi::c_int) as uint16;
                if *((&raw mut g_ram as *mut uint8).offset(0xbf0 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v0 as isize) as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    == 0 as ::core::ffi::c_int
                {
                    let mut v6: uint16 = *((&raw mut g_ram as *mut uint8)
                        .offset(0xdde as ::core::ffi::c_int as isize)
                        as *mut uint16);
                    *((&raw mut g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset((v5 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                        (*((&raw mut g_ram as *mut uint8)
                            .offset(0xb78 as ::core::ffi::c_int as isize)
                            as *mut uint16)
                            .offset(v0 as isize) as ::core::ffi::c_int
                            - r18 as ::core::ffi::c_int) as uint16;
                    *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
                        as *mut uint16) = v5 as uint16;
                    BlockCollMissileVert(v5 as uint16);
                    *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
                        as *mut uint16) = v6;
                    if *((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset((v6 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_int
                        & 0xf00 as ::core::ffi::c_int
                        == 2048 as ::core::ffi::c_int
                    {
                        ClearProjectile(v5 as uint16);
                    }
                    return;
                }
                let mut v7: uint16 = *((&raw mut g_ram as *mut uint8)
                    .offset(0xdde as ::core::ffi::c_int as isize)
                    as *mut uint16);
                *((&raw mut g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset((v5 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = (r18
                    as ::core::ffi::c_int
                    + *((&raw mut g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset(
                            (*((&raw mut g_ram as *mut uint8)
                                .offset(0xdde as ::core::ffi::c_int as isize)
                                as *mut uint16) as ::core::ffi::c_int
                                >> 1 as ::core::ffi::c_int) as isize,
                        ) as ::core::ffi::c_int)
                    as uint16;
                *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
                    as *mut uint16) = v5 as uint16;
                BlockCollMissileVert(v5 as uint16);
                *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
                    as *mut uint16) = v7;
                if *((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset((v7 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int
                    & 0xf00 as ::core::ffi::c_int
                    != 2048 as ::core::ffi::c_int
                {
                    return;
                }
            }
            ClearProjectile(v5 as uint16);
        }
    }
}
pub unsafe fn Projectile_Func4(mut k: uint16) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize) as *mut uint16
        as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize)
        .offset(1 as ::core::ffi::c_int as isize)
        != 0
    {
        if *((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            & 0xf00 as ::core::ffi::c_int
            == 2048 as ::core::ffi::c_int
        {
            ClearProjectile(
                *((&raw mut g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) as uint8 as uint16,
            );
        }
    }
}
pub unsafe fn SuperMissileBlockCollDetect_X() {
    let mut v0: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
        .offset(0xdde as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    if *((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v0 as isize) as ::core::ffi::c_int
        & 0xf00 as ::core::ffi::c_int
        == 512 as ::core::ffi::c_int
        || *((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v0 as isize) as ::core::ffi::c_int
            & 0xf00 as ::core::ffi::c_int
            == 2048 as ::core::ffi::c_int
    {
        let mut v5: uint8 = *((&raw mut g_ram as *mut uint8)
            .offset(0xc7c as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v0 as isize) as uint8;
        if *((&raw mut g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(
                *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int as isize,
            )
            .offset(1 as ::core::ffi::c_int as isize)
            != 0
        {
            let mut v1: uint16 = (abs16(
                *((&raw mut g_ram as *mut uint8).offset(0xbdc as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v0 as isize),
            ) as ::core::ffi::c_int
                & 0xff00 as ::core::ffi::c_int) as uint16;
            if v1 as ::core::ffi::c_int - 2816 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0
            {
                let mut v4: ::core::ffi::c_int =
                    *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        >> 1 as ::core::ffi::c_int;
                if *((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v4 as isize) as ::core::ffi::c_int
                    & 0xf00 as ::core::ffi::c_int
                    != 2048 as ::core::ffi::c_int
                {
                    *((&raw mut g_ram as *mut uint8).offset(0xb64 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset((v5 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                        *((&raw mut g_ram as *mut uint8)
                            .offset(0xb64 as ::core::ffi::c_int as isize)
                            as *mut uint16)
                            .offset(v4 as isize);
                    return;
                }
            } else {
                let mut r18: uint16 = ((v1 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int)
                    - 10 as ::core::ffi::c_int) as uint16;
                if *((&raw mut g_ram as *mut uint8).offset(0xbdc as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v0 as isize) as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    == 0 as ::core::ffi::c_int
                {
                    let mut v6: uint16 = *((&raw mut g_ram as *mut uint8)
                        .offset(0xdde as ::core::ffi::c_int as isize)
                        as *mut uint16);
                    *((&raw mut g_ram as *mut uint8).offset(0xb64 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset((v5 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                        (*((&raw mut g_ram as *mut uint8)
                            .offset(0xb64 as ::core::ffi::c_int as isize)
                            as *mut uint16)
                            .offset(v0 as isize) as ::core::ffi::c_int
                            - r18 as ::core::ffi::c_int) as uint16;
                    *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
                        as *mut uint16) = v5 as uint16;
                    BlockCollMissileHoriz(v5 as uint16);
                    *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
                        as *mut uint16) = v6;
                    if *((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset((v6 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_int
                        & 0xf00 as ::core::ffi::c_int
                        == 2048 as ::core::ffi::c_int
                    {
                        ClearProjectile(v5 as uint16);
                    }
                    return;
                }
                let mut v7: uint16 = *((&raw mut g_ram as *mut uint8)
                    .offset(0xdde as ::core::ffi::c_int as isize)
                    as *mut uint16);
                *((&raw mut g_ram as *mut uint8).offset(0xb64 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset((v5 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = (r18
                    as ::core::ffi::c_int
                    + *((&raw mut g_ram as *mut uint8).offset(0xb64 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset(
                            (*((&raw mut g_ram as *mut uint8)
                                .offset(0xdde as ::core::ffi::c_int as isize)
                                as *mut uint16) as ::core::ffi::c_int
                                >> 1 as ::core::ffi::c_int) as isize,
                        ) as ::core::ffi::c_int)
                    as uint16;
                *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
                    as *mut uint16) = v5 as uint16;
                BlockCollMissileHoriz(v5 as uint16);
                *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
                    as *mut uint16) = v7;
                if *((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset((v7 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int
                    & 0xf00 as ::core::ffi::c_int
                    != 2048 as ::core::ffi::c_int
                {
                    return;
                }
            }
            ClearProjectile(v5 as uint16);
        }
    }
}
pub unsafe fn ProjInstr_MoveLeftProjectileTrailDown(mut j: uint16) {
    let ref mut fresh85 = *((&raw mut g_ram as *mut uint8)
        .offset(0xd778 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    *fresh85 = (*fresh85).wrapping_add(1);
}
pub unsafe fn ProjInstr_MoveRightProjectileTrailDown(mut j: uint16) {
    let ref mut fresh84 = *((&raw mut g_ram as *mut uint8)
        .offset(0xd79c as ::core::ffi::c_int as isize) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    *fresh84 = (*fresh84).wrapping_add(1);
}
pub unsafe fn ProjInstr_MoveLeftProjectileTrailUp(mut j: uint16) {
    let ref mut fresh83 = *((&raw mut g_ram as *mut uint8)
        .offset(0xd778 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    *fresh83 = (*fresh83).wrapping_sub(1);
}
pub unsafe fn SpawnProjectileTrail(mut k: uint16) {
    let mut v2: int16 = 0;
    let mut v1: uint16 = *((&raw mut g_ram as *mut uint8)
        .offset(0xc18 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    if v1 as ::core::ffi::c_int & 0xf00 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        let mut v3: uint16 = (*(&raw mut v1 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int) as uint16;
        if v3 as ::core::ffi::c_int >= 3 as ::core::ffi::c_int {
            return;
        }
        v2 = (v3 as ::core::ffi::c_int + 31 as ::core::ffi::c_int) as int16;
    } else {
        v2 = (*((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            & 0x3f as ::core::ffi::c_int) as int16;
    }
    let mut v4: uint16 = 34 as uint16;
    while *((&raw mut g_ram as *mut uint8).offset(0xd658 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v4 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        != 0
    {
        v4 = (v4 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
        if v4 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            return;
        }
    }
    let mut v5: ::core::ffi::c_int = v4 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut g_ram as *mut uint8).offset(0xd658 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v5 as isize) = 1 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xd67c as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v5 as isize) = 1 as uint16;
    let mut v6: ::core::ffi::c_int = v2 as ::core::ffi::c_int;
    *((&raw mut g_ram as *mut uint8).offset(0xd6a0 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v5 as isize) = *(g_rom.offset(
        (((0x90b5bb as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x90b5bb as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v6 as isize);
    *((&raw mut g_ram as *mut uint8).offset(0xd6c4 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v5 as isize) = *(g_rom.offset(
        (((0x90b609 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x90b609 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(v6 as isize);
    ProjectileTrail_Func5(
        *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
            as *mut uint16),
        v4,
    );
}
pub unsafe fn CallProjInstr(mut ea: uint32, mut j: uint16) {
    match ea {
        9483557 => {
            ProjInstr_MoveLeftProjectileTrailDown(j);
            return;
        }
        9483655 => {
            ProjInstr_MoveRightProjectileTrailDown(j);
            return;
        }
        9483699 => {
            ProjInstr_MoveLeftProjectileTrailUp(j);
            return;
        }
        _ => {
            Unreachable();
        }
    };
}
pub unsafe fn HandleProjectileTrails() {
    let mut i: ::core::ffi::c_int = 0;
    let mut v10: int16 = 0;
    let mut v11: *mut OamEnt = ::core::ptr::null_mut::<OamEnt>();
    let mut v12: int16 = 0;
    let mut j: uint16 = 0;
    let mut v22: int16 = 0;
    let mut v23: *mut OamEnt = ::core::ptr::null_mut::<OamEnt>();
    let mut v24: int16 = 0;
    let mut k: uint16 = 0;
    let mut v28: int16 = 0;
    let mut v29: *mut OamEnt = ::core::ptr::null_mut::<OamEnt>();
    let mut v30: int16 = 0;
    let mut v33: int16 = 0;
    let mut v34: *mut OamEnt = ::core::ptr::null_mut::<OamEnt>();
    let mut v35: int16 = 0;
    let mut v6: uint16 = 0;
    let mut v18: uint16 = 0;
    let mut v7: ::core::ffi::c_int = 0;
    if *((&raw mut g_ram as *mut uint8).offset(0xa78 as ::core::ffi::c_int as isize) as *mut uint16)
        == 0
    {
        let mut current_block_53: u64;
        let mut v0: uint16 = 34 as uint16;
        loop {
            let mut v2: uint16 = 0;
            let mut v1: ::core::ffi::c_int = 0;
            v1 = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            v2 = *((&raw mut g_ram as *mut uint8).offset(0xd658 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize);
            if v2 != 0 {
                let mut v3: uint16 = (v2 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
                *((&raw mut g_ram as *mut uint8).offset(0xd658 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) = v3;
                if v3 != 0 {
                    current_block_53 = 13826291924415791078;
                } else {
                    i = *((&raw mut g_ram as *mut uint8)
                        .offset(0xd6a0 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset(v1 as isize) as ::core::ffi::c_int;
                    loop {
                        let mut v5: *const uint16 = RomPtr_90(i as uint16_t) as *const uint16;
                        v6 = *v5;
                        if *v5 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                            == 0 as ::core::ffi::c_int
                        {
                            break;
                        }
                        CallProjInstr(
                            (v6 as ::core::ffi::c_int | 0x900000 as ::core::ffi::c_int) as uint32,
                            v0,
                        );
                        i += 2 as ::core::ffi::c_int;
                    }
                    v7 = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
                    *((&raw mut g_ram as *mut uint8)
                        .offset(0xd658 as ::core::ffi::c_int as isize) as *mut uint16)
                        .offset(v7 as isize) = v6;
                    if v6 != 0 {
                        *((&raw mut g_ram as *mut uint8)
                            .offset(0xd6e8 as ::core::ffi::c_int as isize)
                            as *mut uint16)
                            .offset(v7 as isize) = *(RomPtr_90(i as uint16_t) as *mut uint16)
                            .offset(1 as ::core::ffi::c_int as isize);
                        *((&raw mut g_ram as *mut uint8)
                            .offset(0xd6a0 as ::core::ffi::c_int as isize)
                            as *mut uint16)
                            .offset(v7 as isize) = (i + 4 as ::core::ffi::c_int) as uint16;
                        current_block_53 = 13826291924415791078;
                    } else {
                        current_block_53 = 18317007320854588510;
                    }
                }
                match current_block_53 {
                    18317007320854588510 => {}
                    _ => {
                        let mut v8: uint16 = *((&raw mut g_ram as *mut uint8)
                            .offset(0x590 as ::core::ffi::c_int as isize)
                            as *mut uint16);
                        if ((*((&raw mut g_ram as *mut uint8)
                            .offset(0x590 as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int
                            - 512 as ::core::ffi::c_int) as int16
                            as ::core::ffi::c_int)
                            < 0 as ::core::ffi::c_int
                        {
                            let mut v9: ::core::ffi::c_int =
                                v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
                            v10 = (*((&raw mut g_ram as *mut uint8)
                                .offset(0xd730 as ::core::ffi::c_int as isize)
                                as *mut uint16)
                                .offset(v9 as isize)
                                as ::core::ffi::c_int
                                - *((&raw mut g_ram as *mut uint8)
                                    .offset(0x911 as ::core::ffi::c_int as isize)
                                    as *mut uint16)
                                    as ::core::ffi::c_int)
                                as int16;
                            if v10 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int
                                == 0 as ::core::ffi::c_int
                            {
                                v11 = gOamEnt(
                                    *((&raw mut g_ram as *mut uint8)
                                        .offset(0x590 as ::core::ffi::c_int as isize)
                                        as *mut uint16)
                                        as ::core::ffi::c_int,
                                );
                                *(&raw mut (*v11).xcoord as *mut uint16) = v10 as uint16;
                                v12 = (*((&raw mut g_ram as *mut uint8)
                                    .offset(0xd778 as ::core::ffi::c_int as isize)
                                    as *mut uint16)
                                    .offset(v9 as isize)
                                    as ::core::ffi::c_int
                                    - *((&raw mut g_ram as *mut uint8)
                                        .offset(0x915 as ::core::ffi::c_int as isize)
                                        as *mut uint16)
                                        as ::core::ffi::c_int)
                                    as int16;
                                if v12 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int
                                    == 0 as ::core::ffi::c_int
                                {
                                    *(&raw mut (*v11).ycoord as *mut uint16) = v12 as uint16;
                                    *(&raw mut (*v11).charnum as *mut uint16) = *((&raw mut g_ram
                                        as *mut uint8)
                                        .offset(0xd6e8 as ::core::ffi::c_int as isize)
                                        as *mut uint16)
                                        .offset(v9 as isize);
                                    *((&raw mut g_ram as *mut uint8)
                                        .offset(0x590 as ::core::ffi::c_int as isize)
                                        as *mut uint16) = (v8 as ::core::ffi::c_int
                                        + 4 as ::core::ffi::c_int)
                                        as uint16;
                                }
                            }
                        }
                    }
                }
            }
            let mut v13: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            let mut v14: uint16 = *((&raw mut g_ram as *mut uint8)
                .offset(0xd67c as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v13 as isize);
            if v14 != 0 {
                let mut v19: ::core::ffi::c_int = 0;
                let mut current_block_32: u64;
                let mut v15: uint16 =
                    (v14 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
                *((&raw mut g_ram as *mut uint8).offset(0xd67c as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v13 as isize) = v15;
                if v15 != 0 {
                    current_block_32 = 7828949454673616476;
                } else {
                    j = *((&raw mut g_ram as *mut uint8)
                        .offset(0xd6c4 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset(v13 as isize);
                    loop {
                        let mut v17: *const uint16 = RomPtr_90(j as uint16_t) as *const uint16;
                        v18 = *v17;
                        if *v17 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                            == 0 as ::core::ffi::c_int
                        {
                            break;
                        }
                        CallProjInstr(
                            (v18 as ::core::ffi::c_int | 0x900000 as ::core::ffi::c_int) as uint32,
                            v0,
                        );
                        j = (j as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
                    }
                    v19 = 0;
                    v19 = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
                    *((&raw mut g_ram as *mut uint8)
                        .offset(0xd67c as ::core::ffi::c_int as isize) as *mut uint16)
                        .offset(v19 as isize) = v18;
                    if v18 != 0 {
                        *((&raw mut g_ram as *mut uint8)
                            .offset(0xd70c as ::core::ffi::c_int as isize)
                            as *mut uint16)
                            .offset(v19 as isize) = *(RomPtr_90(j as uint16_t) as *mut uint16)
                            .offset(1 as ::core::ffi::c_int as isize);
                        *((&raw mut g_ram as *mut uint8)
                            .offset(0xd6c4 as ::core::ffi::c_int as isize)
                            as *mut uint16)
                            .offset(v19 as isize) =
                            (j as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
                        current_block_32 = 7828949454673616476;
                    } else {
                        current_block_32 = 6174974146017752131;
                    }
                }
                match current_block_32 {
                    7828949454673616476 => {
                        let mut v20: uint16 = *((&raw mut g_ram as *mut uint8)
                            .offset(0x590 as ::core::ffi::c_int as isize)
                            as *mut uint16);
                        if ((*((&raw mut g_ram as *mut uint8)
                            .offset(0x590 as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int
                            - 512 as ::core::ffi::c_int) as int16
                            as ::core::ffi::c_int)
                            < 0 as ::core::ffi::c_int
                        {
                            let mut v21: ::core::ffi::c_int =
                                v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
                            v22 = (*((&raw mut g_ram as *mut uint8)
                                .offset(0xd754 as ::core::ffi::c_int as isize)
                                as *mut uint16)
                                .offset(v21 as isize)
                                as ::core::ffi::c_int
                                - *((&raw mut g_ram as *mut uint8)
                                    .offset(0x911 as ::core::ffi::c_int as isize)
                                    as *mut uint16)
                                    as ::core::ffi::c_int)
                                as int16;
                            if v22 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int
                                == 0 as ::core::ffi::c_int
                            {
                                v23 = gOamEnt(
                                    *((&raw mut g_ram as *mut uint8)
                                        .offset(0x590 as ::core::ffi::c_int as isize)
                                        as *mut uint16)
                                        as ::core::ffi::c_int,
                                );
                                *(&raw mut (*v23).xcoord as *mut uint16) = v22 as uint16;
                                v24 = (*((&raw mut g_ram as *mut uint8)
                                    .offset(0xd79c as ::core::ffi::c_int as isize)
                                    as *mut uint16)
                                    .offset(v21 as isize)
                                    as ::core::ffi::c_int
                                    - *((&raw mut g_ram as *mut uint8)
                                        .offset(0x915 as ::core::ffi::c_int as isize)
                                        as *mut uint16)
                                        as ::core::ffi::c_int)
                                    as int16;
                                if v24 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int
                                    == 0 as ::core::ffi::c_int
                                {
                                    *(&raw mut (*v23).ycoord as *mut uint16) = v24 as uint16;
                                    *(&raw mut (*v23).charnum as *mut uint16) = *((&raw mut g_ram
                                        as *mut uint8)
                                        .offset(0xd70c as ::core::ffi::c_int as isize)
                                        as *mut uint16)
                                        .offset(v21 as isize);
                                    *((&raw mut g_ram as *mut uint8)
                                        .offset(0x590 as ::core::ffi::c_int as isize)
                                        as *mut uint16) = (v20 as ::core::ffi::c_int
                                        + 4 as ::core::ffi::c_int)
                                        as uint16;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            v0 = (v0 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
            if v0 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
                return;
            }
        }
    }
    k = 34 as uint16;
    while k as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        let mut v26: uint16 = *((&raw mut g_ram as *mut uint8)
            .offset(0x590 as ::core::ffi::c_int as isize)
            as *mut uint16);
        if ((*((&raw mut g_ram as *mut uint8).offset(0x590 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 512 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        {
            let mut v27: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            if *((&raw mut g_ram as *mut uint8).offset(0xd658 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v27 as isize)
                != 0
            {
                v28 = (*((&raw mut g_ram as *mut uint8)
                    .offset(0xd730 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v27 as isize) as ::core::ffi::c_int
                    - *((&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int) as int16;
                if v28 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int
                    == 0 as ::core::ffi::c_int
                {
                    v29 = gOamEnt(
                        *((&raw mut g_ram as *mut uint8)
                            .offset(0x590 as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int,
                    );
                    *(&raw mut (*v29).xcoord as *mut uint16) = v28 as uint16;
                    v30 = (*((&raw mut g_ram as *mut uint8)
                        .offset(0xd778 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset(v27 as isize) as ::core::ffi::c_int
                        - *((&raw mut g_ram as *mut uint8)
                            .offset(0x915 as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int)
                        as int16;
                    if v30 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int
                        == 0 as ::core::ffi::c_int
                    {
                        *(&raw mut (*v29).ycoord as *mut uint16) = v30 as uint16;
                        *(&raw mut (*v29).charnum as *mut uint16) =
                            *((&raw mut g_ram as *mut uint8)
                                .offset(0xd6e8 as ::core::ffi::c_int as isize)
                                as *mut uint16)
                                .offset(v27 as isize);
                        *((&raw mut g_ram as *mut uint8)
                            .offset(0x590 as ::core::ffi::c_int as isize)
                            as *mut uint16) =
                            (v26 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
                    }
                }
            }
        }
        let mut v31: uint16 = *((&raw mut g_ram as *mut uint8)
            .offset(0x590 as ::core::ffi::c_int as isize)
            as *mut uint16);
        if ((*((&raw mut g_ram as *mut uint8).offset(0x590 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 512 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        {
            let mut v32: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            if *((&raw mut g_ram as *mut uint8).offset(0xd67c as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v32 as isize)
                != 0
            {
                v33 = (*((&raw mut g_ram as *mut uint8)
                    .offset(0xd754 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v32 as isize) as ::core::ffi::c_int
                    - *((&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int) as int16;
                if v33 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int
                    == 0 as ::core::ffi::c_int
                {
                    v34 = gOamEnt(
                        *((&raw mut g_ram as *mut uint8)
                            .offset(0x590 as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int,
                    );
                    *(&raw mut (*v34).xcoord as *mut uint16) = v33 as uint16;
                    v35 = (*((&raw mut g_ram as *mut uint8)
                        .offset(0xd79c as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset(v32 as isize) as ::core::ffi::c_int
                        - *((&raw mut g_ram as *mut uint8)
                            .offset(0x915 as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int)
                        as int16;
                    if v35 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int
                        == 0 as ::core::ffi::c_int
                    {
                        *(&raw mut (*v34).ycoord as *mut uint16) = v35 as uint16;
                        *(&raw mut (*v34).charnum as *mut uint16) =
                            *((&raw mut g_ram as *mut uint8)
                                .offset(0xd70c as ::core::ffi::c_int as isize)
                                as *mut uint16)
                                .offset(v32 as isize);
                        *((&raw mut g_ram as *mut uint8)
                            .offset(0x590 as ::core::ffi::c_int as isize)
                            as *mut uint16) =
                            (v31 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
                    }
                }
            }
        }
        k = (k as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn HudSelectionHandler_NothingOrPowerBombs() {
    let mut current_block: u64;
    *((&raw mut g_ram as *mut uint8).offset(0xdc2 as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0xcd0 as ::core::ffi::c_int as isize)
            as *mut uint16);
    if *((&raw mut g_ram as *mut uint8).offset(0xa76 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        != 0
        || *((&raw mut g_ram as *mut uint8).offset(0x9a6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x1000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
    {
        if *((&raw mut g_ram as *mut uint8).offset(0x9b2 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & *((&raw mut g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            FireUnchargedBeam();
        }
        return;
    }
    if *((&raw mut g_ram as *mut uint8).offset(0xb5e as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        if *((&raw mut g_ram as *mut uint8).offset(0xcd0 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 60 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            current_block = 14652055052181797410;
        } else {
            current_block = 8147482841674861883;
        }
    } else if *((&raw mut g_ram as *mut uint8).offset(0x9b2 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & *((&raw mut g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        if *((&raw mut g_ram as *mut uint8).offset(0xcd0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            == 0
        {
            return;
        }
        if *((&raw mut g_ram as *mut uint8).offset(0xcd0 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 60 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            current_block = 14652055052181797410;
        } else {
            current_block = 8147482841674861883;
        }
    } else {
        if *((&raw mut g_ram as *mut uint8).offset(0xcd0 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 120 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            let ref mut fresh76 = *((&raw mut g_ram as *mut uint8)
                .offset(0xcd0 as ::core::ffi::c_int as isize)
                as *mut uint16);
            *fresh76 = (*fresh76).wrapping_add(1);
            if *fresh76 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
                ClearFlareAnimationState();
                FireUnchargedBeam();
            }
        } else if FireSba() as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
            *((&raw mut g_ram as *mut uint8).offset(0xcd0 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            ClearFlareAnimationState();
            Samus_LoadSuitPalette();
        }
        return;
    }
    match current_block {
        14652055052181797410 => {
            *((&raw mut g_ram as *mut uint8).offset(0xcd0 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            ClearFlareAnimationState();
            FireUnchargedBeam();
            return;
        }
        _ => {
            *((&raw mut g_ram as *mut uint8).offset(0xcd0 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            ClearFlareAnimationState();
            FireChargedBeam();
            return;
        }
    };
}
static mut kProjectileBombPreInstr: [uint16; 12] = [
    fnProjPreInstr_Beam_NoWaveBeam as uint16,
    fnProjPreInstr_BeamOrIceWave as uint16,
    fnProjPreInstr_Beam_NoWaveBeam as uint16,
    fnProjPreInstr_BeamOrIceWave as uint16,
    fnProjPreInstr_Beam_NoWaveBeam as uint16,
    fnProjPreInstr_WavePlasmaEtc as uint16,
    fnProjPreInstr_Beam_NoWaveBeam as uint16,
    fnProjPreInstr_WavePlasmaEtc as uint16,
    fnProjPreInstr_Beam_NoWaveBeam as uint16,
    fnProjPreInstr_WavePlasmaEtc as uint16,
    fnProjPreInstr_Beam_NoWaveBeam as uint16,
    fnProjPreInstr_WavePlasmaEtc as uint16,
];
pub unsafe fn FireUnchargedBeam() {
    let mut v3: int8 = 0;
    if *((&raw mut g_ram as *mut uint8).offset(0xa76 as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        FireHyperBeam();
        return;
    }
    if Samus_CanFireBeam() as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        let mut v0: uint16 = 0 as uint16;
        while *((&raw mut g_ram as *mut uint8).offset(0xc2c as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            != 0
        {
            v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            if !((v0 as ::core::ffi::c_int - 10 as ::core::ffi::c_int) as int16
                as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int)
            {
                continue;
            }
            v0 = (v0 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
            break;
        }
        if InitProjectilePositionDirection(v0) as ::core::ffi::c_int & 1 as ::core::ffi::c_int == 0
        {
            let mut v5: uint16 = 0;
            let mut v4: ::core::ffi::c_int = 0;
            let mut current_block_32: u64;
            *((&raw mut g_ram as *mut uint8).offset(0x18ac as ::core::ffi::c_int as isize)
                as *mut uint16) = 10 as uint16;
            let mut v1: uint16 = v0;
            let mut v2: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            *((&raw mut g_ram as *mut uint8).offset(0xc90 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v2 as isize) = 4 as uint16;
            v3 = *((&raw mut g_ram as *mut uint8).offset(0x9a6 as ::core::ffi::c_int as isize)
                as *mut uint16) as int8;
            *((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v2 as isize) = (*((&raw mut g_ram as *mut uint8)
                .offset(0x9a6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                | 0x8000 as ::core::ffi::c_int) as uint16;
            QueueSfx1_Max15(
                kUnchargedProjectile_Sfx
                    [(v3 as ::core::ffi::c_int & 0xf as ::core::ffi::c_int) as usize],
            );
            *((&raw mut g_ram as *mut uint8).offset(0xdc0 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            InitializeProjectile(v1);
            if *((&raw mut g_ram as *mut uint8).offset(0x9a6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0x1000 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
                || *((&raw mut g_ram as *mut uint8).offset(0x9b2 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    & *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int
                || *((&raw mut g_ram as *mut uint8).offset(0x9b2 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    & *((&raw mut g_ram as *mut uint8).offset(0xe00 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int
            {
                let mut v6: uint16 = 0;
                v6 = *((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v2 as isize);
                *((&raw mut g_ram as *mut uint8).offset(0xccc as ::core::ffi::c_int as isize)
                    as *mut uint16) = kProjectileCooldown_Uncharged
                    [(v6 as ::core::ffi::c_int & 0x3f as ::core::ffi::c_int) as usize]
                    as uint16;
                if v6 as ::core::ffi::c_int & 1 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                    current_block_32 = 1316823607412241498;
                } else {
                    current_block_32 = 4068382217303356765;
                }
            } else {
                v5 = 0;
                v5 = *((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v2 as isize);
                *((&raw mut g_ram as *mut uint8).offset(0xccc as ::core::ffi::c_int as isize)
                    as *mut uint16) = kBeamAutoFireCooldowns
                    [(v5 as ::core::ffi::c_int & 0x3f as ::core::ffi::c_int) as usize]
                    as uint16;
                if v5 as ::core::ffi::c_int & 1 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                    current_block_32 = 1316823607412241498;
                } else {
                    current_block_32 = 4068382217303356765;
                }
            }
            match current_block_32 {
                4068382217303356765 => {
                    v4 = 0;
                    v4 = v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
                    *((&raw mut g_ram as *mut uint8).offset(0xbdc as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset(v4 as isize) = 0 as uint16;
                    *((&raw mut g_ram as *mut uint8).offset(0xbf0 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset(v4 as isize) = 0 as uint16;
                    *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
                        as *mut uint16) = v1;
                    WaveBeam_CheckColl(v1);
                }
                _ => {
                    *((&raw mut g_ram as *mut uint8).offset(0xbdc as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset(v2 as isize) = 0 as uint16;
                    *((&raw mut g_ram as *mut uint8).offset(0xbf0 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset(v2 as isize) = 0 as uint16;
                    *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
                        as *mut uint16) = v1;
                    CheckBeamCollByDir(v1);
                    v1 = *((&raw mut g_ram as *mut uint8)
                        .offset(0xdde as ::core::ffi::c_int as isize)
                        as *mut uint16);
                    if *((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset(
                            (*((&raw mut g_ram as *mut uint8)
                                .offset(0xdde as ::core::ffi::c_int as isize)
                                as *mut uint16) as ::core::ffi::c_int
                                >> 1 as ::core::ffi::c_int) as isize,
                        ) as ::core::ffi::c_int
                        & 0xf00 as ::core::ffi::c_int
                        != 0 as ::core::ffi::c_int
                    {
                        return;
                    }
                }
            }
            *((&raw mut g_ram as *mut uint8).offset(0xc68 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                kProjectileBombPreInstr[(*((&raw mut g_ram as *mut uint8)
                    .offset(0xc18 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int
                    & 0xf as ::core::ffi::c_int) as usize];
            SetInitialProjectileSpeed(v1);
            return;
        }
    }
    if *((&raw mut g_ram as *mut uint8).offset(0xdc2 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - 16 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        *((&raw mut g_ram as *mut uint8).offset(0xdc0 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        QueueSfx1_Max15(2 as uint16);
    }
}
static mut kFireChargedBeam_Funcs: [uint16; 12] = [
    fnProjPreInstr_Beam_NoWaveBeam as uint16,
    fnProjPreInstr_WavePlasmaEtc as uint16,
    fnProjPreInstr_Beam_NoWaveBeam as uint16,
    fnProjPreInstr_WavePlasmaEtc as uint16,
    fnProjPreInstr_Beam_NoWaveBeam as uint16,
    fnProjPreInstr_WavePlasmaEtc as uint16,
    fnProjPreInstr_Beam_NoWaveBeam as uint16,
    fnProjPreInstr_WavePlasmaEtc as uint16,
    fnProjPreInstr_Beam_NoWaveBeam as uint16,
    fnProjPreInstr_WavePlasmaEtc as uint16,
    fnProjPreInstr_Beam_NoWaveBeam as uint16,
    fnProjPreInstr_WavePlasmaEtc as uint16,
];
pub unsafe fn FireChargedBeam() {
    if Samus_CanFireBeam() as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        let mut v0: uint16 = 0 as uint16;
        while *((&raw mut g_ram as *mut uint8).offset(0xc2c as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            != 0
        {
            v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            if !((v0 as ::core::ffi::c_int - 10 as ::core::ffi::c_int) as int16
                as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int)
            {
                continue;
            }
            v0 = (v0 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
            break;
        }
        let mut r20: uint16 = v0;
        if InitProjectilePositionDirection(r20) as ::core::ffi::c_int & 1 as ::core::ffi::c_int == 0
        {
            let mut current_block_24: u64;
            *((&raw mut g_ram as *mut uint8).offset(0x18ac as ::core::ffi::c_int as isize)
                as *mut uint16) = 10 as uint16;
            let mut v1: uint16 = r20;
            let mut v2: ::core::ffi::c_int = r20 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            *((&raw mut g_ram as *mut uint8).offset(0xc90 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v2 as isize) = 4 as uint16;
            let mut v3: uint16 = (*((&raw mut g_ram as *mut uint8)
                .offset(0x9a6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0x100f as ::core::ffi::c_int
                | 0x8010 as ::core::ffi::c_int) as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v2 as isize) = v3;
            QueueSfx1_Max15(
                kChargedProjectile_Sfx
                    [(v3 as ::core::ffi::c_int & 0xf as ::core::ffi::c_int) as usize],
            );
            *((&raw mut g_ram as *mut uint8).offset(0xdc0 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            InitializeProjectile(v1);
            let mut v5: uint16 = *((&raw mut g_ram as *mut uint8)
                .offset(0xc18 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v2 as isize);
            *((&raw mut g_ram as *mut uint8).offset(0xccc as ::core::ffi::c_int as isize)
                as *mut uint16) = kProjectileCooldown_Uncharged
                [(v5 as ::core::ffi::c_int & 0x3f as ::core::ffi::c_int) as usize]
                as uint16;
            if v5 as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
                let mut v4: ::core::ffi::c_int =
                    v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
                *((&raw mut g_ram as *mut uint8).offset(0xbdc as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v4 as isize) = 0 as uint16;
                *((&raw mut g_ram as *mut uint8).offset(0xbf0 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v4 as isize) = 0 as uint16;
                *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
                    as *mut uint16) = v1;
                WaveBeam_CheckColl(v1);
                current_block_24 = 1109700713171191020;
            } else {
                *((&raw mut g_ram as *mut uint8).offset(0xbdc as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v2 as isize) = 0 as uint16;
                *((&raw mut g_ram as *mut uint8).offset(0xbf0 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v2 as isize) = 0 as uint16;
                *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
                    as *mut uint16) = v1;
                CheckBeamCollByDir(v1);
                v1 = *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
                    as *mut uint16);
                if *((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(
                        (*((&raw mut g_ram as *mut uint8)
                            .offset(0xdde as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int
                            >> 1 as ::core::ffi::c_int) as isize,
                    ) as ::core::ffi::c_int
                    & 0xf00 as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int
                {
                    current_block_24 = 7264328972795780040;
                } else {
                    current_block_24 = 1109700713171191020;
                }
            }
            match current_block_24 {
                1109700713171191020 => {
                    r20 = v1;
                    *((&raw mut g_ram as *mut uint8).offset(0xc68 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                        kFireChargedBeam_Funcs[(*((&raw mut g_ram as *mut uint8)
                            .offset(0xc18 as ::core::ffi::c_int as isize)
                            as *mut uint16)
                            .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_int
                            & 0xf as ::core::ffi::c_int)
                            as usize];
                    SetInitialProjectileSpeed(r20);
                }
                _ => {}
            }
            *((&raw mut g_ram as *mut uint8).offset(0xb18 as ::core::ffi::c_int as isize)
                as *mut uint16) = 4 as uint16;
            return;
        }
    }
    if *((&raw mut g_ram as *mut uint8).offset(0xdc2 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - 16 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        *((&raw mut g_ram as *mut uint8).offset(0xdc0 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        QueueSfx1_Max15(2 as uint16);
    }
}
static mut kProjectileOriginOffsets3_X: [int16; 10] = [
    2 as ::core::ffi::c_int as int16,
    13 as ::core::ffi::c_int as int16,
    11 as ::core::ffi::c_int as int16,
    13 as ::core::ffi::c_int as int16,
    2 as ::core::ffi::c_int as int16,
    -(5 as ::core::ffi::c_int) as int16,
    -(14 as ::core::ffi::c_int) as int16,
    -(11 as ::core::ffi::c_int) as int16,
    -(19 as ::core::ffi::c_int) as int16,
    -(2 as ::core::ffi::c_int) as int16,
];
static mut kProjectileOriginOffsets3_Y: [int16; 10] = [
    -(8 as ::core::ffi::c_int) as int16,
    -(13 as ::core::ffi::c_int) as int16,
    1 as ::core::ffi::c_int as int16,
    4 as ::core::ffi::c_int as int16,
    13 as ::core::ffi::c_int as int16,
    13 as ::core::ffi::c_int as int16,
    4 as ::core::ffi::c_int as int16,
    1 as ::core::ffi::c_int as int16,
    -(19 as ::core::ffi::c_int) as int16,
    -(8 as ::core::ffi::c_int) as int16,
];
static mut kProjectileOriginOffsets4_X: [int16; 10] = [
    2 as ::core::ffi::c_int as int16,
    15 as ::core::ffi::c_int as int16,
    15 as ::core::ffi::c_int as int16,
    13 as ::core::ffi::c_int as int16,
    2 as ::core::ffi::c_int as int16,
    -(5 as ::core::ffi::c_int) as int16,
    -(13 as ::core::ffi::c_int) as int16,
    -(13 as ::core::ffi::c_int) as int16,
    -(15 as ::core::ffi::c_int) as int16,
    -(2 as ::core::ffi::c_int) as int16,
];
static mut kProjectileOriginOffsets4_Y: [int16; 10] = [
    -(8 as ::core::ffi::c_int) as int16,
    -(16 as ::core::ffi::c_int) as int16,
    -(2 as ::core::ffi::c_int) as int16,
    1 as ::core::ffi::c_int as int16,
    13 as ::core::ffi::c_int as int16,
    13 as ::core::ffi::c_int as int16,
    1 as ::core::ffi::c_int as int16,
    -(2 as ::core::ffi::c_int) as int16,
    -(16 as ::core::ffi::c_int) as int16,
    -(8 as ::core::ffi::c_int) as int16,
];
pub unsafe fn InitProjectilePositionDirection(mut r20: uint16) -> uint8 {
    let mut v0: uint16 = *((&raw mut g_ram as *mut uint8)
        .offset(0xa1c as ::core::ffi::c_int as isize) as *mut uint16);
    let mut direction_shots_fired: uint16 = 0;
    if *((&raw mut g_ram as *mut uint8).offset(0xb5e as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        direction_shots_fired = *((&raw mut g_ram as *mut uint8)
            .offset(0xb5e as ::core::ffi::c_int as isize)
            as *mut uint16) as uint8 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb5e as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    } else {
        direction_shots_fired = (*(g_rom.offset(
            (((0x91b629 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x91b629 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut SamusPoseParams)
            .offset(v0 as isize))
        .direction_shots_fired as uint16;
        if direction_shots_fired as ::core::ffi::c_int & 0xf0 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            if direction_shots_fired as ::core::ffi::c_int != 16 as ::core::ffi::c_int || {
                *(&raw mut direction_shots_fired as *mut uint8)
                    .offset(0 as ::core::ffi::c_int as isize) = (*(g_rom.offset(
                    (((0x91b629 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x91b629 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8
                    as *mut SamusPoseParams)
                    .offset(
                        *((&raw mut g_ram as *mut uint8)
                            .offset(0xa24 as ::core::ffi::c_int as isize)
                            as *mut uint16) as isize,
                    ))
                .direction_shots_fired;
                direction_shots_fired as ::core::ffi::c_int & 0xf0 as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int
            } {
                let ref mut fresh5 = *((&raw mut g_ram as *mut uint8)
                    .offset(0xcce as ::core::ffi::c_int as isize)
                    as *mut uint16);
                *fresh5 = (*fresh5).wrapping_sub(1);
                return 1 as uint8;
            }
            direction_shots_fired = direction_shots_fired as uint8 as uint16;
            v0 = *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16);
        }
    }
    let mut v2: ::core::ffi::c_int = r20 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut g_ram as *mut uint8).offset(0xc04 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v2 as isize) = direction_shots_fired;
    let mut r22: uint16 = (*(g_rom.offset(
        (((0x91b629 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x91b629 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut SamusPoseParams)
        .offset(v0 as isize))
    .y_offset_to_gfx as uint16;
    let mut v3: uint16 = (2 as ::core::ffi::c_int
        * (*((&raw mut g_ram as *mut uint8).offset(0xc04 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int)) as uint16;
    if *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        == kPose_75_FaceL_Moonwalk_AimUL as ::core::ffi::c_int
        || *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == kPose_76_FaceR_Moonwalk_AimUR as ::core::ffi::c_int
        || *(&raw mut g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            == 1 as ::core::ffi::c_int
    {
        let mut v6: ::core::ffi::c_int = v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut g_ram as *mut uint8).offset(0xb64 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) = (*((&raw mut g_ram as *mut uint8)
            .offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + kProjectileOriginOffsets4_X[v6 as usize] as ::core::ffi::c_int)
            as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) = (*((&raw mut g_ram as *mut uint8)
            .offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + kProjectileOriginOffsets4_Y[v6 as usize] as ::core::ffi::c_int
            - r22 as ::core::ffi::c_int) as uint16;
        return 0 as uint8;
    } else {
        let mut v4: ::core::ffi::c_int = v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut g_ram as *mut uint8).offset(0xb64 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) = (*((&raw mut g_ram as *mut uint8)
            .offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + kProjectileOriginOffsets3_X[v4 as usize] as ::core::ffi::c_int)
            as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) = (*((&raw mut g_ram as *mut uint8)
            .offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + kProjectileOriginOffsets3_Y[v4 as usize] as ::core::ffi::c_int
            - r22 as ::core::ffi::c_int) as uint16;
        return 0 as uint8;
    };
}
pub unsafe fn HandleChargingBeamGfxAudio() {
    let mut v1: int16 = 0;
    let mut v4: int16 = 0;
    let mut v9: uint16 = 0;
    if *((&raw mut g_ram as *mut uint8).offset(0xa76 as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        if *((&raw mut g_ram as *mut uint8).offset(0xcd0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            != 0
        {
            let mut i: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
            while i >= 0 as ::core::ffi::c_int {
                let mut v7: bool = *(((&raw mut g_ram as *mut uint8)
                    .offset(0xcdc as ::core::ffi::c_int as isize)
                    as *mut uint16 as *mut uint8)
                    .offset(i as isize) as *mut uint16)
                    as ::core::ffi::c_int
                    == 1 as ::core::ffi::c_int;
                let ref mut fresh17 = *(((&raw mut g_ram as *mut uint8)
                    .offset(0xcdc as ::core::ffi::c_int as isize)
                    as *mut uint16 as *mut uint8)
                    .offset(i as isize) as *mut uint16);
                *fresh17 = (*fresh17).wrapping_sub(1);
                let mut v8: bool = *fresh17 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int;
                if v7 as ::core::ffi::c_int != 0 || v8 as ::core::ffi::c_int != 0 {
                    let ref mut fresh20 = *(((&raw mut g_ram as *mut uint8)
                        .offset(0xcd6 as ::core::ffi::c_int as isize)
                        as *mut uint16 as *mut uint8)
                        .offset(i as isize)
                        as *mut uint16);
                    let fresh21 = *fresh20;
                    *fresh20 = (*fresh20).wrapping_sub(1);
                    v7 = fresh21 as ::core::ffi::c_int == 1 as ::core::ffi::c_int;
                    if v7 {
                        if i == 4 as ::core::ffi::c_int {
                            *((&raw mut g_ram as *mut uint8)
                                .offset(0xcd0 as ::core::ffi::c_int as isize)
                                as *mut uint16) = 0 as uint16;
                        }
                    } else {
                        *(((&raw mut g_ram as *mut uint8)
                            .offset(0xcdc as ::core::ffi::c_int as isize)
                            as *mut uint16 as *mut uint8)
                            .offset(i as isize) as *mut uint16) = 3 as uint16;
                    }
                }
                DrawFlareAnimationComponent(i as uint16);
                i -= 2 as ::core::ffi::c_int;
            }
        }
    } else if *((&raw mut g_ram as *mut uint8).offset(0xcd0 as ::core::ffi::c_int as isize)
        as *mut uint16) as int16 as ::core::ffi::c_int
        > 0 as ::core::ffi::c_int
    {
        if *((&raw mut g_ram as *mut uint8).offset(0xcd0 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == 1 as ::core::ffi::c_int
        {
            *((&raw mut g_ram as *mut uint8).offset(0xcd6 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xcd8 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xcda as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xcdc as ::core::ffi::c_int as isize)
                as *mut uint16) = 3 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xcde as ::core::ffi::c_int as isize)
                as *mut uint16) = 5 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xce0 as ::core::ffi::c_int as isize)
                as *mut uint16) = 4 as uint16;
        }
        if *((&raw mut g_ram as *mut uint8).offset(0xcd0 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 15 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            if *((&raw mut g_ram as *mut uint8).offset(0xcd0 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                == 16 as ::core::ffi::c_int
            {
                QueueSfx1_Max9(8 as uint16);
            }
            let mut v0: uint16 = 0 as uint16;
            loop {
                v1 = (*(((&raw mut g_ram as *mut uint8).offset(0xcdc as ::core::ffi::c_int as isize)
                    as *mut uint16 as *mut uint8)
                    .offset(v0 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    - 1 as ::core::ffi::c_int) as int16;
                *(((&raw mut g_ram as *mut uint8).offset(0xcdc as ::core::ffi::c_int as isize)
                    as *mut uint16 as *mut uint8)
                    .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16) = v1 as uint16;
                if (v1 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
                    let mut v2: uint16 =
                        (*(((&raw mut g_ram as *mut uint8)
                            .offset(0xcd6 as ::core::ffi::c_int as isize)
                            as *mut uint16 as *mut uint8)
                            .offset(v0 as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int
                            + 1 as ::core::ffi::c_int) as uint16;
                    *(((&raw mut g_ram as *mut uint8).offset(0xcd6 as ::core::ffi::c_int as isize)
                        as *mut uint16 as *mut uint8)
                        .offset(v0 as ::core::ffi::c_int as isize)
                        as *mut uint16) = v2;
                    let mut v3: uint16 = v2;
                    let mut r0: *const uint8 = RomPtr_90(
                        *(g_rom.offset(
                            (((0x90c481 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                                << 15 as ::core::ffi::c_int
                                | 0x90c481 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                                & 0x3fffff as ::core::ffi::c_int)
                                as isize,
                        ) as *const uint8 as *mut uint16)
                            .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                            as uint16_t,
                    );
                    v4 = *r0.offset(v2 as isize) as int16;
                    if v4 as ::core::ffi::c_int == 255 as ::core::ffi::c_int {
                        *(((&raw mut g_ram as *mut uint8)
                            .offset(0xcd6 as ::core::ffi::c_int as isize)
                            as *mut uint16 as *mut uint8)
                            .offset(v0 as ::core::ffi::c_int as isize)
                            as *mut uint16) = 0 as uint16;
                        v3 = 0 as uint16;
                    } else if v4 as ::core::ffi::c_int == 254 as ::core::ffi::c_int {
                        let mut v5: uint16 =
                            (*(((&raw mut g_ram as *mut uint8)
                                .offset(0xcd6 as ::core::ffi::c_int as isize)
                                as *mut uint16 as *mut uint8)
                                .offset(v0 as ::core::ffi::c_int as isize)
                                as *mut uint16) as ::core::ffi::c_int
                                - *r0.offset(
                                    (v3 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize,
                                ) as ::core::ffi::c_int) as uint16;
                        *(((&raw mut g_ram as *mut uint8)
                            .offset(0xcd6 as ::core::ffi::c_int as isize)
                            as *mut uint16 as *mut uint8)
                            .offset(v0 as ::core::ffi::c_int as isize)
                            as *mut uint16) = v5;
                        v3 = v5;
                    }
                    *(((&raw mut g_ram as *mut uint8).offset(0xcdc as ::core::ffi::c_int as isize)
                        as *mut uint16 as *mut uint8)
                        .offset(v0 as ::core::ffi::c_int as isize)
                        as *mut uint16) = *r0.offset(v3 as isize) as uint16;
                }
                v9 = v0;
                DrawFlareAnimationComponent(v0);
                if *((&raw mut g_ram as *mut uint8).offset(0xcd0 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    - 30 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    != 0
                {
                    break;
                }
                v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
                if !(((v9 as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as int16
                    as ::core::ffi::c_int)
                    < 0 as ::core::ffi::c_int)
                {
                    break;
                }
            }
        }
    }
}
static mut kProjectileOriginOffsets_X: [int16; 13] = [
    2 as ::core::ffi::c_int as int16,
    18 as ::core::ffi::c_int as int16,
    15 as ::core::ffi::c_int as int16,
    17 as ::core::ffi::c_int as int16,
    3 as ::core::ffi::c_int as int16,
    -(4 as ::core::ffi::c_int) as int16,
    -(17 as ::core::ffi::c_int) as int16,
    -(15 as ::core::ffi::c_int) as int16,
    -(18 as ::core::ffi::c_int) as int16,
    -(2 as ::core::ffi::c_int) as int16,
    -(4 as ::core::ffi::c_int) as int16,
    -(4 as ::core::ffi::c_int) as int16,
    -(4 as ::core::ffi::c_int) as int16,
];
static mut kProjectileOriginOffsets_Y: [int16; 13] = [
    -(28 as ::core::ffi::c_int) as int16,
    -(19 as ::core::ffi::c_int) as int16,
    1 as ::core::ffi::c_int as int16,
    6 as ::core::ffi::c_int as int16,
    17 as ::core::ffi::c_int as int16,
    17 as ::core::ffi::c_int as int16,
    6 as ::core::ffi::c_int as int16,
    1 as ::core::ffi::c_int as int16,
    -(20 as ::core::ffi::c_int) as int16,
    -(28 as ::core::ffi::c_int) as int16,
    -(20 as ::core::ffi::c_int) as int16,
    -(2 as ::core::ffi::c_int) as int16,
    8 as ::core::ffi::c_int as int16,
];
static mut kProjectileOriginOffsets2_X: [int16; 10] = [
    2 as ::core::ffi::c_int as int16,
    19 as ::core::ffi::c_int as int16,
    20 as ::core::ffi::c_int as int16,
    18 as ::core::ffi::c_int as int16,
    3 as ::core::ffi::c_int as int16,
    -(4 as ::core::ffi::c_int) as int16,
    -(18 as ::core::ffi::c_int) as int16,
    -(20 as ::core::ffi::c_int) as int16,
    -(19 as ::core::ffi::c_int) as int16,
    -(2 as ::core::ffi::c_int) as int16,
];
static mut kProjectileOriginOffsets2_Y: [int16; 10] = [
    -(32 as ::core::ffi::c_int) as int16,
    -(22 as ::core::ffi::c_int) as int16,
    -(3 as ::core::ffi::c_int) as int16,
    6 as ::core::ffi::c_int as int16,
    25 as ::core::ffi::c_int as int16,
    25 as ::core::ffi::c_int as int16,
    6 as ::core::ffi::c_int as int16,
    -(3 as ::core::ffi::c_int) as int16,
    -(20 as ::core::ffi::c_int) as int16,
    -(32 as ::core::ffi::c_int) as int16,
];
pub unsafe fn DrawFlareAnimationComponent(mut k: uint16) {
    static mut word_93A225: [uint16; 3] = [
        0 as ::core::ffi::c_int as uint16,
        0x1e as ::core::ffi::c_int as uint16,
        0x24 as ::core::ffi::c_int as uint16,
    ];
    static mut word_93A22B: [uint16; 3] = [
        0 as ::core::ffi::c_int as uint16,
        0x2a as ::core::ffi::c_int as uint16,
        0x30 as ::core::ffi::c_int as uint16,
    ];
    let mut v2: int16 = 0;
    let mut v1: uint16 = 0;
    let mut r20: uint16 = 0;
    let mut r18: uint16 = *((&raw mut g_ram as *mut uint8)
        .offset(0xcd6 as ::core::ffi::c_int as isize) as *mut uint16
        as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize) as uint16;
    if *(&raw mut g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == 4 as ::core::ffi::c_int
    {
        v1 = word_93A22B[(k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize];
    } else {
        v1 = word_93A225[(k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize];
    }
    let mut r22: uint16 = (r18 as ::core::ffi::c_int + v1 as ::core::ffi::c_int) as uint16;
    let mut r24: uint16 = (*(g_rom.offset(
        (((0x91b629 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x91b629 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut SamusPoseParams)
        .offset(
            *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) as isize,
        ))
    .y_offset_to_gfx as uint16;
    v2 = (*(g_rom.offset(
        (((0x91b629 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x91b629 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut SamusPoseParams)
        .offset(
            *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) as isize,
        ))
    .direction_shots_fired as int16;
    if v2 as ::core::ffi::c_int != 255 as ::core::ffi::c_int
        && v2 as ::core::ffi::c_int != 16 as ::core::ffi::c_int
    {
        let mut v3: uint16 = (2 as ::core::ffi::c_int
            * (v2 as ::core::ffi::c_int & 0xf as ::core::ffi::c_int))
            as uint16;
        let mut old_x: uint16 = *((&raw mut g_ram as *mut uint8)
            .offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16);
        let mut old_y: uint16 = *((&raw mut g_ram as *mut uint8)
            .offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16);
        if *((&raw mut g_ram as *mut uint8).offset(0x93f as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            Samus_CalcPos_Mode7();
        }
        let mut v4: ::core::ffi::c_int = v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v5: uint16 = 0;
        if *(&raw mut g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            == 1 as ::core::ffi::c_int
        {
            r20 = (*((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + kProjectileOriginOffsets2_X[v4 as usize] as ::core::ffi::c_int
                - *((&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint16;
            v5 = (*((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + kProjectileOriginOffsets2_Y[v4 as usize] as ::core::ffi::c_int
                - r24 as ::core::ffi::c_int
                - *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint16;
        } else {
            r20 = (*((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + kProjectileOriginOffsets_X[v4 as usize] as ::core::ffi::c_int
                - *((&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint16;
            v5 = (*((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + kProjectileOriginOffsets_Y[v4 as usize] as ::core::ffi::c_int
                - r24 as ::core::ffi::c_int
                - *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint16;
        }
        r18 = v5;
        if v5 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            DrawBeamGrappleSpritemap(r22, r20, r18);
        }
        if *((&raw mut g_ram as *mut uint8).offset(0x93f as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            *((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
                as *mut uint16) = old_y;
            *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) = old_x;
        }
    }
}
pub unsafe fn ClearFlareAnimationState() {
    *((&raw mut g_ram as *mut uint8).offset(0xcd6 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xcd8 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xcda as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xcdc as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xcde as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xce0 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
}
pub unsafe fn FireHyperBeam() {
    if Samus_CanFireBeam() as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        let mut v0: uint16 = 0 as uint16;
        while *((&raw mut g_ram as *mut uint8).offset(0xc2c as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            != 0
        {
            v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            if !((v0 as ::core::ffi::c_int - 10 as ::core::ffi::c_int) as int16
                as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int)
            {
                continue;
            }
            v0 = (v0 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
            break;
        }
        let mut r20: uint16 = v0;
        if InitProjectilePositionDirection(r20) as ::core::ffi::c_int & 1 as ::core::ffi::c_int == 0
        {
            *((&raw mut g_ram as *mut uint8).offset(0x18ac as ::core::ffi::c_int as isize)
                as *mut uint16) = 10 as uint16;
            let mut k: uint16 = r20;
            *((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((r20 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                -(28648 as ::core::ffi::c_int) as uint16;
            QueueSfx1_Max15(kChargedProjectile_Sfx[8 as ::core::ffi::c_int as usize]);
            *((&raw mut g_ram as *mut uint8).offset(0xdc0 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            InitializeProjectile(k);
            let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            *((&raw mut g_ram as *mut uint8).offset(0xbdc as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = 0 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xbf0 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = 0 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
                as *mut uint16) = k;
            WaveBeam_CheckColl(k);
            let mut v2: uint16 = *((&raw mut g_ram as *mut uint8)
                .offset(0xdde as ::core::ffi::c_int as isize)
                as *mut uint16);
            let mut v3: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
                .offset(0xdde as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int;
            *((&raw mut g_ram as *mut uint8).offset(0xc2c as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v3 as isize) = 1000 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xc68 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v3 as isize) =
                (0x90b159 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            r20 = v2;
            SetInitialProjectileSpeed(r20);
            *((&raw mut g_ram as *mut uint8).offset(0xccc as ::core::ffi::c_int as isize)
                as *mut uint16) = 21 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xb18 as ::core::ffi::c_int as isize)
                as *mut uint16) = -(32748 as ::core::ffi::c_int) as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xcd6 as ::core::ffi::c_int as isize)
                as *mut uint16) = 29 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xcd8 as ::core::ffi::c_int as isize)
                as *mut uint16) = 5 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xcda as ::core::ffi::c_int as isize)
                as *mut uint16) = 5 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xcdc as ::core::ffi::c_int as isize)
                as *mut uint16) = 3 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xcde as ::core::ffi::c_int as isize)
                as *mut uint16) = 3 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xce0 as ::core::ffi::c_int as isize)
                as *mut uint16) = 3 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xcd0 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0x8000 as uint16;
        }
    }
}
static mut kCheckBeamCollByDir: [Option<Func_V>; 10] = unsafe {
    [
        Some(CheckBeamCollByDir_0459 as unsafe fn() -> ()),
        Some(CheckBeamCollByDir_1368 as unsafe fn() -> ()),
        Some(CheckBeamCollByDir_2 as unsafe fn() -> ()),
        Some(CheckBeamCollByDir_1368 as unsafe fn() -> ()),
        Some(CheckBeamCollByDir_0459 as unsafe fn() -> ()),
        Some(CheckBeamCollByDir_0459 as unsafe fn() -> ()),
        Some(CheckBeamCollByDir_1368 as unsafe fn() -> ()),
        Some(CheckBeamCollByDir_7 as unsafe fn() -> ()),
        Some(CheckBeamCollByDir_1368 as unsafe fn() -> ()),
        Some(CheckBeamCollByDir_0459 as unsafe fn() -> ()),
    ]
};
pub unsafe fn CheckBeamCollByDir(mut k: uint16) {
    kCheckBeamCollByDir[(*((&raw mut g_ram as *mut uint8)
        .offset(0xc04 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int) as usize]
        .expect("non-null function pointer")();
}
pub unsafe fn CheckBeamCollByDir_0459() {
    BlockCollNoWaveBeamVert(
        *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
}
pub unsafe fn CheckBeamCollByDir_1368() {
    let mut v0: uint16 = *((&raw mut g_ram as *mut uint8)
        .offset(0xdde as ::core::ffi::c_int as isize) as *mut uint16);
    if BlockCollNoWaveBeamHoriz(
        *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
            as *mut uint16),
    ) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        == 0
    {
        BlockCollNoWaveBeamVert(v0);
    }
}
pub unsafe fn CheckBeamCollByDir_2() {
    BlockCollNoWaveBeamHoriz(
        *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
}
pub unsafe fn CheckBeamCollByDir_7() {
    let mut v0: uint16 = *((&raw mut g_ram as *mut uint8)
        .offset(0xdde as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0xbdc as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(
            (*((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as isize,
        ) = -(1 as ::core::ffi::c_int) as uint16;
    BlockCollNoWaveBeamHoriz(v0);
}
static mut kWaveBeam_CheckColl_Funcs: [Option<Func_V>; 10] = unsafe {
    [
        Some(WaveBeam_CheckColl_0459 as unsafe fn() -> ()),
        Some(WaveBeam_CheckColl_1368 as unsafe fn() -> ()),
        Some(WaveBeam_CheckColl_2 as unsafe fn() -> ()),
        Some(WaveBeam_CheckColl_1368 as unsafe fn() -> ()),
        Some(WaveBeam_CheckColl_0459 as unsafe fn() -> ()),
        Some(WaveBeam_CheckColl_0459 as unsafe fn() -> ()),
        Some(WaveBeam_CheckColl_1368 as unsafe fn() -> ()),
        Some(WaveBeam_CheckColl_7 as unsafe fn() -> ()),
        Some(WaveBeam_CheckColl_1368 as unsafe fn() -> ()),
        Some(WaveBeam_CheckColl_0459 as unsafe fn() -> ()),
    ]
};
pub unsafe fn WaveBeam_CheckColl(mut k: uint16) {
    kWaveBeam_CheckColl_Funcs[(*((&raw mut g_ram as *mut uint8)
        .offset(0xc04 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int) as usize]
        .expect("non-null function pointer")();
}
pub unsafe fn WaveBeam_CheckColl_0459() {
    BlockCollWaveBeamVert(
        *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
}
pub unsafe fn WaveBeam_CheckColl_1368() {
    let mut v0: uint16 = *((&raw mut g_ram as *mut uint8)
        .offset(0xdde as ::core::ffi::c_int as isize) as *mut uint16);
    if BlockCollWaveBeamHoriz(
        *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
            as *mut uint16),
    ) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        == 0
    {
        BlockCollWaveBeamVert(v0);
    }
}
pub unsafe fn WaveBeam_CheckColl_2() {
    BlockCollWaveBeamHoriz(
        *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
            as *mut uint16),
    );
}
pub unsafe fn WaveBeam_CheckColl_7() {
    let mut v0: uint16 = *((&raw mut g_ram as *mut uint8)
        .offset(0xdde as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0xbdc as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(
            (*((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as isize,
        ) = -(1 as ::core::ffi::c_int) as uint16;
    BlockCollWaveBeamHoriz(v0);
}
pub unsafe fn ProjectileReflection(mut r20: uint16) {
    let mut v0: uint16 = r20;
    let mut v1: ::core::ffi::c_int = r20 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v2: uint16 = *((&raw mut g_ram as *mut uint8)
        .offset(0xc18 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    if v2 as ::core::ffi::c_int & 0x100 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        InitializeProjectile(r20);
        *((&raw mut g_ram as *mut uint8).offset(0xc68 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) =
            (0x90af68 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 240 as uint16;
    } else if v2 as ::core::ffi::c_int & 0x200 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        let mut k: uint16 = r20;
        ClearProjectile(
            *(((&raw mut g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as *mut uint16 as *mut uint8)
                .offset(0 as ::core::ffi::c_int as isize) as uint16,
        );
        InitializeProjectile(k);
        let mut v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut g_ram as *mut uint8).offset(0xc68 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v3 as isize) =
            (0x90afe5 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v3 as isize) = 240 as uint16;
    } else {
        SetInitialProjectileSpeed(r20);
        InitializeProjectile(v0);
        *((&raw mut g_ram as *mut uint8).offset(0xc68 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) =
            kFireChargedBeam_Funcs[(*((&raw mut g_ram as *mut uint8)
                .offset(0xc18 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                & 0xf as ::core::ffi::c_int) as usize];
    };
}
pub unsafe fn HudSelectionHandler_MissilesOrSuperMissiles() {
    let mut current_block: u64;
    if *((&raw mut g_ram as *mut uint8).offset(0x9b2 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
        && *((&raw mut g_ram as *mut uint8).offset(0x9b2 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & *((&raw mut g_ram as *mut uint8).offset(0xe00 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        || Samus_CanFireSuperMissile() as ::core::ffi::c_int & 1 as ::core::ffi::c_int == 0
    {
        return;
    }
    if *((&raw mut g_ram as *mut uint8).offset(0x9d2 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        != 2 as ::core::ffi::c_int
    {
        if *((&raw mut g_ram as *mut uint8).offset(0x9c6 as ::core::ffi::c_int as isize)
            as *mut uint16)
            != 0
        {
            current_block = 7351195479953500246;
        } else {
            current_block = 12512720045134471270;
        }
    } else if *((&raw mut g_ram as *mut uint8).offset(0x9ca as ::core::ffi::c_int as isize)
        as *mut uint16)
        == 0
    {
        current_block = 12512720045134471270;
    } else {
        current_block = 7351195479953500246;
    }
    match current_block {
        7351195479953500246 => {
            let mut v0: uint16 = 0 as uint16;
            loop {
                if !(*((&raw mut g_ram as *mut uint8).offset(0xc2c as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                    != 0)
                {
                    current_block = 13536709405535804910;
                    break;
                }
                v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
                if (v0 as ::core::ffi::c_int - 10 as ::core::ffi::c_int) as int16
                    as ::core::ffi::c_int
                    >= 0 as ::core::ffi::c_int
                {
                    current_block = 12512720045134471270;
                    break;
                }
            }
            match current_block {
                12512720045134471270 => {}
                _ => {
                    let mut r20: uint16 = v0;
                    if InitProjectilePositionDirection(r20) as ::core::ffi::c_int
                        & 1 as ::core::ffi::c_int
                        == 0
                    {
                        *((&raw mut g_ram as *mut uint8)
                            .offset(0x18ac as ::core::ffi::c_int as isize)
                            as *mut uint16) = 20 as uint16;
                        if *((&raw mut g_ram as *mut uint8)
                            .offset(0x9d2 as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int
                            == 2 as ::core::ffi::c_int
                        {
                            let ref mut fresh78 = *((&raw mut g_ram as *mut uint8)
                                .offset(0x9ca as ::core::ffi::c_int as isize)
                                as *mut uint16);
                            *fresh78 = (*fresh78).wrapping_sub(1);
                        } else {
                            let ref mut fresh79 = *((&raw mut g_ram as *mut uint8)
                                .offset(0x9c6 as ::core::ffi::c_int as isize)
                                as *mut uint16);
                            *fresh79 = (*fresh79).wrapping_sub(1);
                        }
                        let mut v1: ::core::ffi::c_int =
                            r20 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
                        *((&raw mut g_ram as *mut uint8)
                            .offset(0xc90 as ::core::ffi::c_int as isize)
                            as *mut uint16)
                            .offset(v1 as isize) = 4 as uint16;
                        let mut v3: uint16 = *((&raw mut g_ram as *mut uint8)
                            .offset(0x9d2 as ::core::ffi::c_int as isize)
                            as *mut uint16);
                        let mut r18: uint16 = swap16(
                            *((&raw mut g_ram as *mut uint8)
                                .offset(0x9d2 as ::core::ffi::c_int as isize)
                                as *mut uint16),
                        );
                        let ref mut fresh80 = *((&raw mut g_ram as *mut uint8)
                            .offset(0xc18 as ::core::ffi::c_int as isize)
                            as *mut uint16)
                            .offset(v1 as isize);
                        *fresh80 = (*fresh80 as ::core::ffi::c_int
                            | (r18 as ::core::ffi::c_int | 0x8000 as ::core::ffi::c_int))
                            as uint16;
                        let mut v4: uint16 = (2 as ::core::ffi::c_int
                            * (v3 as ::core::ffi::c_int & 0xf as ::core::ffi::c_int))
                            as uint16;
                        if *((&raw mut g_ram as *mut uint8)
                            .offset(0x1f51 as ::core::ffi::c_int as isize)
                            as *mut uint16)
                            == 0
                        {
                            QueueSfx1_Max6(
                                kNonBeamProjectile_Sfx[(v4 as ::core::ffi::c_int
                                    >> 1 as ::core::ffi::c_int)
                                    as usize],
                            );
                        }
                        InitializeProjectileSpeedOfType(r20);
                        InitializeProjectile(r20);
                        let mut v7: uint16 = *((&raw mut g_ram as *mut uint8)
                            .offset(0xc18 as ::core::ffi::c_int as isize)
                            as *mut uint16)
                            .offset(v1 as isize);
                        if v7 as ::core::ffi::c_int & 0x200 as ::core::ffi::c_int
                            != 0 as ::core::ffi::c_int
                        {
                            *((&raw mut g_ram as *mut uint8)
                                .offset(0xc68 as ::core::ffi::c_int as isize)
                                as *mut uint16)
                                .offset(v1 as isize) = (0x90afe5 as ::core::ffi::c_int
                                & 0xffff as ::core::ffi::c_int)
                                as uint16;
                        } else {
                            *((&raw mut g_ram as *mut uint8)
                                .offset(0xc68 as ::core::ffi::c_int as isize)
                                as *mut uint16)
                                .offset(v1 as isize) = (0x90af68 as ::core::ffi::c_int
                                & 0xffff as ::core::ffi::c_int)
                                as uint16;
                        }
                        *((&raw mut g_ram as *mut uint8)
                            .offset(0xccc as ::core::ffi::c_int as isize)
                            as *mut uint16) =
                            kNonBeamProjectileCooldowns[(*(&raw mut v7 as *mut uint8).offset(
                                (::core::mem::size_of::<uint16>() as usize)
                                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                                    .wrapping_sub(1 as usize)
                                    as isize,
                            )
                                as ::core::ffi::c_int
                                & 0xf as ::core::ffi::c_int)
                                as usize] as uint16;
                        if *((&raw mut g_ram as *mut uint8)
                            .offset(0xa04 as ::core::ffi::c_int as isize)
                            as *mut uint16)
                            != 0
                        {
                            *((&raw mut g_ram as *mut uint8)
                                .offset(0x9d2 as ::core::ffi::c_int as isize)
                                as *mut uint16) = 0 as uint16;
                            *((&raw mut g_ram as *mut uint8)
                                .offset(0xa04 as ::core::ffi::c_int as isize)
                                as *mut uint16) = 0 as uint16;
                            return;
                        }
                        if *((&raw mut g_ram as *mut uint8)
                            .offset(0x9d2 as ::core::ffi::c_int as isize)
                            as *mut uint16) as ::core::ffi::c_int
                            == 2 as ::core::ffi::c_int
                        {
                            if *((&raw mut g_ram as *mut uint8)
                                .offset(0x9ca as ::core::ffi::c_int as isize)
                                as *mut uint16)
                                != 0
                            {
                                return;
                            }
                        } else if *((&raw mut g_ram as *mut uint8)
                            .offset(0x9c6 as ::core::ffi::c_int as isize)
                            as *mut uint16)
                            != 0
                        {
                            return;
                        }
                        *((&raw mut g_ram as *mut uint8)
                            .offset(0x9d2 as ::core::ffi::c_int as isize)
                            as *mut uint16) = 0 as uint16;
                    }
                    return;
                }
            }
        }
        _ => {}
    }
    let ref mut fresh77 = *((&raw mut g_ram as *mut uint8)
        .offset(0xcce as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh77 = (*fresh77).wrapping_sub(1);
}
pub unsafe fn Missile_Func2() {
    let mut v0: uint16 = 0 as uint16;
    while *((&raw mut g_ram as *mut uint8).offset(0xc2c as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        != 0
    {
        v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        if (v0 as ::core::ffi::c_int - 10 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        {
            return;
        }
    }
    let mut r20: uint16 = v0;
    let mut v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh66 = *((&raw mut g_ram as *mut uint8)
        .offset(0xc18 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    *fresh66 = (*fresh66 as ::core::ffi::c_int | 0x8200 as ::core::ffi::c_int) as uint16;
    let mut v2: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
        .offset(0xdde as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    *((&raw mut g_ram as *mut uint8).offset(0xb64 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) = *((&raw mut g_ram as *mut uint8)
        .offset(0xb64 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize);
    *((&raw mut g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) = *((&raw mut g_ram as *mut uint8)
        .offset(0xb78 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize);
    *((&raw mut g_ram as *mut uint8).offset(0xc04 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) = *((&raw mut g_ram as *mut uint8)
        .offset(0xc04 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v2 as isize);
    InitProjectilePositionDirection(r20);
    InitializeInstrForSuperMissile(v0);
    *((&raw mut g_ram as *mut uint8).offset(0xc68 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) =
        (0x90b075 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(
            (*((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as isize,
        ) = (v0 as ::core::ffi::c_int
        + (*((&raw mut g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(
                (*((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    >> 1 as ::core::ffi::c_int) as isize,
            ) as ::core::ffi::c_int
            & 0xff00 as ::core::ffi::c_int)) as uint16;
    let ref mut fresh67 = *((&raw mut g_ram as *mut uint8)
        .offset(0xcce as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh67 = (*fresh67).wrapping_add(1);
}
pub unsafe fn HudSelectionHandler_MorphBall() {
    if *((&raw mut g_ram as *mut uint8).offset(0x9b2 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & *((&raw mut g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        if *((&raw mut g_ram as *mut uint8).offset(0x9d2 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == 3 as ::core::ffi::c_int
        {
            if *((&raw mut g_ram as *mut uint8).offset(0xcee as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            {
                if HudSelectionHandler_MorphBall_Helper2() as ::core::ffi::c_int
                    & 1 as ::core::ffi::c_int
                    != 0
                {
                    if *((&raw mut g_ram as *mut uint8).offset(0x9ce as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        != 0
                    {
                        let ref mut fresh81 = *((&raw mut g_ram as *mut uint8)
                            .offset(0x9ce as ::core::ffi::c_int as isize)
                            as *mut uint16);
                        *fresh81 = (*fresh81).wrapping_sub(1);
                        if *fresh81 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                            == 0 as ::core::ffi::c_int
                        {
                            *((&raw mut g_ram as *mut uint8)
                                .offset(0xcee as ::core::ffi::c_int as isize)
                                as *mut uint16) = -(1 as ::core::ffi::c_int) as uint16;
                            let mut v2: uint16 = 10 as uint16;
                            while *((&raw mut g_ram as *mut uint8)
                                .offset(0xc18 as ::core::ffi::c_int as isize)
                                as *mut uint16)
                                .offset(
                                    (v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                                )
                                != 0
                            {
                                v2 = (v2 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
                                if !((v2 as ::core::ffi::c_int - 20 as ::core::ffi::c_int) as int16
                                    as ::core::ffi::c_int
                                    >= 0 as ::core::ffi::c_int)
                                {
                                    continue;
                                }
                                v2 = (v2 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
                                break;
                            }
                            let mut r18: uint16 = swap16(
                                *((&raw mut g_ram as *mut uint8)
                                    .offset(0x9d2 as ::core::ffi::c_int as isize)
                                    as *mut uint16),
                            );
                            let mut v4: ::core::ffi::c_int =
                                v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
                            let mut v5: uint16 = (r18 as ::core::ffi::c_int
                                | *((&raw mut g_ram as *mut uint8)
                                    .offset(0xc18 as ::core::ffi::c_int as isize)
                                    as *mut uint16)
                                    .offset(v4 as isize)
                                    as ::core::ffi::c_int)
                                as uint16;
                            *((&raw mut g_ram as *mut uint8)
                                .offset(0xc18 as ::core::ffi::c_int as isize)
                                as *mut uint16)
                                .offset(v4 as isize) = v5;
                            let mut v6: uint8 = *(&raw mut v5 as *mut uint8).offset(
                                (::core::mem::size_of::<uint16>() as usize)
                                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                                    .wrapping_sub(1 as usize)
                                    as isize,
                            );
                            *((&raw mut g_ram as *mut uint8)
                                .offset(0xc04 as ::core::ffi::c_int as isize)
                                as *mut uint16)
                                .offset(v4 as isize) = 0 as uint16;
                            *((&raw mut g_ram as *mut uint8)
                                .offset(0xb64 as ::core::ffi::c_int as isize)
                                as *mut uint16)
                                .offset(v4 as isize) = *((&raw mut g_ram as *mut uint8)
                                .offset(0xaf6 as ::core::ffi::c_int as isize)
                                as *mut uint16);
                            *((&raw mut g_ram as *mut uint8)
                                .offset(0xb78 as ::core::ffi::c_int as isize)
                                as *mut uint16)
                                .offset(v4 as isize) = *((&raw mut g_ram as *mut uint8)
                                .offset(0xafa as ::core::ffi::c_int as isize)
                                as *mut uint16);
                            *((&raw mut g_ram as *mut uint8)
                                .offset(0xc7c as ::core::ffi::c_int as isize)
                                as *mut uint16)
                                .offset(v4 as isize) = 60 as uint16;
                            InitializeInstrForMissile(v2);
                            *((&raw mut g_ram as *mut uint8)
                                .offset(0xc68 as ::core::ffi::c_int as isize)
                                as *mut uint16)
                                .offset(v4 as isize) = (0x90b0ae as ::core::ffi::c_int
                                & 0xffff as ::core::ffi::c_int)
                                as uint16;
                            *((&raw mut g_ram as *mut uint8)
                                .offset(0xccc as ::core::ffi::c_int as isize)
                                as *mut uint16) = kNonBeamProjectileCooldowns
                                [(v6 as ::core::ffi::c_int & 0xf as ::core::ffi::c_int) as usize]
                                as uint16;
                            if *((&raw mut g_ram as *mut uint8)
                                .offset(0xa04 as ::core::ffi::c_int as isize)
                                as *mut uint16)
                                != 0
                            {
                                *((&raw mut g_ram as *mut uint8)
                                    .offset(0x9d2 as ::core::ffi::c_int as isize)
                                    as *mut uint16) = 0 as uint16;
                                *((&raw mut g_ram as *mut uint8)
                                    .offset(0xa04 as ::core::ffi::c_int as isize)
                                    as *mut uint16) = 0 as uint16;
                            } else if *((&raw mut g_ram as *mut uint8)
                                .offset(0x9d2 as ::core::ffi::c_int as isize)
                                as *mut uint16)
                                as ::core::ffi::c_int
                                == 3 as ::core::ffi::c_int
                                && *((&raw mut g_ram as *mut uint8)
                                    .offset(0x9ce as ::core::ffi::c_int as isize)
                                    as *mut uint16)
                                    == 0
                            {
                                *((&raw mut g_ram as *mut uint8)
                                    .offset(0x9d2 as ::core::ffi::c_int as isize)
                                    as *mut uint16) = 0 as uint16;
                            }
                        }
                    }
                }
            }
        } else if HudSelectionHandler_MorphBall_Helper() as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            != 0
        {
            let mut v0: uint16 = 10 as uint16;
            while *((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                != 0
            {
                v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
                if !((v0 as ::core::ffi::c_int - 20 as ::core::ffi::c_int) as int16
                    as ::core::ffi::c_int
                    >= 0 as ::core::ffi::c_int)
                {
                    continue;
                }
                v0 = (v0 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
                break;
            }
            let mut v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            *((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = 1280 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xc04 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = 0 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xb64 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = *((&raw mut g_ram as *mut uint8)
                .offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16);
            *((&raw mut g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = *((&raw mut g_ram as *mut uint8)
                .offset(0xafa as ::core::ffi::c_int as isize)
                as *mut uint16);
            *((&raw mut g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = 60 as uint16;
            InitializeInstrForMissile(v0);
            *((&raw mut g_ram as *mut uint8).offset(0xc68 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) =
                (0x90b099 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xccc as ::core::ffi::c_int as isize)
                as *mut uint16) =
                kNonBeamProjectileCooldowns[5 as ::core::ffi::c_int as usize] as uint16;
        }
    } else if *((&raw mut g_ram as *mut uint8).offset(0xcd0 as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0
    {
        QueueSfx1_Max9(2 as uint16);
        *((&raw mut g_ram as *mut uint8).offset(0xcd0 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        ClearFlareAnimationState();
        Samus_LoadSuitPalette();
    }
}
pub unsafe fn HudSelectionHandler_MorphBall_Helper() -> uint8 {
    if *((&raw mut g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0x1000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        if *((&raw mut g_ram as *mut uint8).offset(0xcd0 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 60 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
            || *((&raw mut g_ram as *mut uint8).offset(0xcd2 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                != 0
        {
            return HudSelectionHandler_MorphBall_Helper2();
        }
        if *((&raw mut g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & kButton_Down as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
            && (*((&raw mut g_ram as *mut uint8).offset(0xcd4 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0xc0 as ::core::ffi::c_int)
                - 192 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
        {
            let ref mut fresh2 = *((&raw mut g_ram as *mut uint8)
                .offset(0xcd4 as ::core::ffi::c_int as isize)
                as *mut uint16);
            *fresh2 = (*fresh2).wrapping_add(1);
        } else {
            BombSpread();
            Samus_LoadSuitPalette();
            QueueSfx1_Max9(2 as uint16);
        }
    }
    return 0 as uint8;
}
pub unsafe fn HudSelectionHandler_MorphBall_Helper2() -> uint8 {
    if *((&raw mut g_ram as *mut uint8).offset(0x9b2 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
        || *((&raw mut g_ram as *mut uint8).offset(0xcd2 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != 0
            && (*((&raw mut g_ram as *mut uint8).offset(0xcd2 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - 5 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
                || *((&raw mut g_ram as *mut uint8).offset(0xccc as ::core::ffi::c_int as isize)
                    as *mut uint16) as uint8 as ::core::ffi::c_int
                    != 0)
    {
        if *((&raw mut g_ram as *mut uint8).offset(0xcd0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            != 0
        {
            QueueSfx1_Max9(2 as uint16);
            *((&raw mut g_ram as *mut uint8).offset(0xcd0 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            ClearFlareAnimationState();
            Samus_LoadSuitPalette();
        }
        return 0 as uint8;
    } else {
        let ref mut fresh3 = *((&raw mut g_ram as *mut uint8)
            .offset(0xccc as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh3 = (*fresh3).wrapping_add(1);
        let ref mut fresh4 = *((&raw mut g_ram as *mut uint8)
            .offset(0xcd2 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh4 = (*fresh4).wrapping_add(1);
        return 1 as uint8;
    };
}
pub unsafe fn Bomb_Func2() {
    let mut v0: uint16 = *((&raw mut g_ram as *mut uint8)
        .offset(0xdde as ::core::ffi::c_int as isize) as *mut uint16);
    let mut v1: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
        .offset(0xdde as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    let mut v2: uint16 = *((&raw mut g_ram as *mut uint8)
        .offset(0xc7c as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    if v2 != 0 {
        let mut v3: uint16 = (v2 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = v3;
        if v3 != 0 {
            if v3 as ::core::ffi::c_int == 15 as ::core::ffi::c_int {
                let ref mut fresh12 = *((&raw mut g_ram as *mut uint8)
                    .offset(0xc40 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize);
                *fresh12 = (*fresh12 as ::core::ffi::c_int + 28 as ::core::ffi::c_int) as uint16;
            }
        } else {
            QueueSfx2_Max6(8 as uint16);
            InitializeBombExplosion(v0);
        }
    }
}
pub unsafe fn PowerBomb_Func3() {
    let mut v0: uint16 = *((&raw mut g_ram as *mut uint8)
        .offset(0xdde as ::core::ffi::c_int as isize) as *mut uint16);
    let mut v1: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
        .offset(0xdde as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    let mut v2: uint16 = *((&raw mut g_ram as *mut uint8)
        .offset(0xc7c as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    if v2 != 0 {
        let mut v3: uint16 = (v2 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = v3;
        if v3 != 0 {
            if v3 as ::core::ffi::c_int == 15 as ::core::ffi::c_int {
                let ref mut fresh61 = *((&raw mut g_ram as *mut uint8)
                    .offset(0xc40 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize);
                *fresh61 = (*fresh61 as ::core::ffi::c_int + 28 as ::core::ffi::c_int) as uint16;
            }
        } else {
            *((&raw mut g_ram as *mut uint8).offset(0xce2 as ::core::ffi::c_int as isize)
                as *mut uint16) = *((&raw mut g_ram as *mut uint8)
                .offset(0xb64 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize);
            *((&raw mut g_ram as *mut uint8).offset(0xce4 as ::core::ffi::c_int as isize)
                as *mut uint16) = *((&raw mut g_ram as *mut uint8)
                .offset(0xb78 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize);
            EnableHdmaObjects();
            SpawnPowerBombExplosion();
            *((&raw mut g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                -(1 as ::core::ffi::c_int) as uint16;
        }
    } else if *((&raw mut g_ram as *mut uint8).offset(0xcee as ::core::ffi::c_int as isize)
        as *mut uint16)
        == 0
    {
        ClearProjectile(
            *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
                as *mut uint16),
        );
    }
}
static mut kRunSwitchedToHudHandler: [Option<Func_U8>; 6] = unsafe {
    [
        Some(SwitchToHudHandler_Nothing as unsafe fn() -> uint8),
        Some(SwitchToHudHandler_Missiles as unsafe fn() -> uint8),
        Some(SwitchToHudHandler_SuperMissiles as unsafe fn() -> uint8),
        Some(SwitchToHudHandler_PowerBombs as unsafe fn() -> uint8),
        Some(SwitchToHudHandler_Grapple as unsafe fn() -> uint8),
        Some(SwitchToHudHandler_Xray as unsafe fn() -> uint8),
    ]
};
pub unsafe fn HandleSwitchingHudSelection() {
    let mut current_block: u64;
    let mut v0: uint16 = 0;
    let mut r22: uint16 = 0 as uint16;
    let mut r18: uint16 = *((&raw mut g_ram as *mut uint8)
        .offset(0x9d2 as ::core::ffi::c_int as isize) as *mut uint16);
    if *((&raw mut g_ram as *mut uint8).offset(0x9b8 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0xa04 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        current_block = 5182151076943748346;
    } else {
        r22 = (*((&raw mut g_ram as *mut uint8).offset(0x9b8 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & *((&raw mut g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int) as ::core::ffi::c_int as uint16;
        if *((&raw mut g_ram as *mut uint8).offset(0x9ba as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            current_block = 16684771485965707775;
        } else {
            v0 = (*((&raw mut g_ram as *mut uint8).offset(0x9d2 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as uint16;
            if *((&raw mut g_ram as *mut uint8).offset(0x9d2 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - 5 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
            {
                current_block = 5182151076943748346;
            } else {
                current_block = 11963831534727063849;
            }
        }
    }
    match current_block {
        5182151076943748346 => {
            v0 = 0 as uint16;
            current_block = 11963831534727063849;
        }
        _ => {}
    }
    match current_block {
        11963831534727063849 => {
            *((&raw mut g_ram as *mut uint8).offset(0x9d2 as ::core::ffi::c_int as isize)
                as *mut uint16) = v0;
            while kRunSwitchedToHudHandler[v0 as usize].expect("non-null function pointer")()
                as ::core::ffi::c_int
                & 1 as ::core::ffi::c_int
                != 0
            {
                v0 = (*((&raw mut g_ram as *mut uint8).offset(0x9d2 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int) as uint16;
                *((&raw mut g_ram as *mut uint8).offset(0x9d2 as ::core::ffi::c_int as isize)
                    as *mut uint16) = v0;
                if v0 as ::core::ffi::c_int - 6 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                    == 0
                {
                    v0 = 0 as uint16;
                    *((&raw mut g_ram as *mut uint8).offset(0x9d2 as ::core::ffi::c_int as isize)
                        as *mut uint16) = 0 as uint16;
                }
            }
            if r22 != 0 {
                *((&raw mut g_ram as *mut uint8).offset(0xa04 as ::core::ffi::c_int as isize)
                    as *mut uint16) = *((&raw mut g_ram as *mut uint8)
                    .offset(0x9d2 as ::core::ffi::c_int as isize)
                    as *mut uint16);
            } else {
                *((&raw mut g_ram as *mut uint8).offset(0xa04 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 0 as uint16;
            }
        }
        _ => {}
    }
    if *((&raw mut g_ram as *mut uint8).offset(0x9d2 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        == r18 as ::core::ffi::c_int
    {
        let mut v1: uint16 = (*((&raw mut g_ram as *mut uint8)
            .offset(0xaaa as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int) as uint16;
        if *((&raw mut g_ram as *mut uint8).offset(0xaaa as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 2 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            v1 = 2 as uint16;
        }
        *((&raw mut g_ram as *mut uint8).offset(0xaaa as ::core::ffi::c_int as isize)
            as *mut uint16) = v1;
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0xaaa as ::core::ffi::c_int as isize)
            as *mut uint16) = 1 as uint16;
    };
}
pub unsafe fn SwitchToHudHandler_Nothing() -> uint8 {
    *((&raw mut g_ram as *mut uint8).offset(0xcd0 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    ClearFlareAnimationState();
    Samus_LoadSuitPalette();
    return 0 as uint8;
}
pub unsafe fn SwitchToHudHandler_Missiles() -> uint8 {
    if *((&raw mut g_ram as *mut uint8).offset(0x9c6 as ::core::ffi::c_int as isize) as *mut uint16)
        == 0
    {
        return 1 as uint8;
    }
    *((&raw mut g_ram as *mut uint8).offset(0xcd0 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    ClearFlareAnimationState();
    Samus_LoadSuitPalette();
    return 0 as uint8;
}
pub unsafe fn SwitchToHudHandler_SuperMissiles() -> uint8 {
    if *((&raw mut g_ram as *mut uint8).offset(0x9ca as ::core::ffi::c_int as isize) as *mut uint16)
        == 0
    {
        return 1 as uint8;
    }
    *((&raw mut g_ram as *mut uint8).offset(0xcd0 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    ClearFlareAnimationState();
    Samus_LoadSuitPalette();
    return 0 as uint8;
}
pub unsafe fn SwitchToHudHandler_PowerBombs() -> uint8 {
    if *((&raw mut g_ram as *mut uint8).offset(0x9ce as ::core::ffi::c_int as isize) as *mut uint16)
        == 0
    {
        return 1 as uint8;
    }
    *((&raw mut g_ram as *mut uint8).offset(0xcd0 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    ClearFlareAnimationState();
    Samus_LoadSuitPalette();
    return 0 as uint8;
}
pub unsafe fn SwitchToHudHandler_Grapple() -> uint8 {
    if *((&raw mut g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0x4000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        return 1 as uint8;
    }
    if *((&raw mut g_ram as *mut uint8).offset(0xd32 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        == 0x9bc4f0 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        Samus_LoadSuitPalette();
        *((&raw mut g_ram as *mut uint8).offset(0xcd0 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        ClearFlareAnimationState();
        *((&raw mut g_ram as *mut uint8).offset(0xd32 as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x9bc4f0 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    return 0 as uint8;
}
pub unsafe fn SwitchToHudHandler_Xray() -> uint8 {
    if *((&raw mut g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        return 1 as uint8;
    }
    *((&raw mut g_ram as *mut uint8).offset(0xcd0 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    ClearFlareAnimationState();
    Samus_LoadSuitPalette();
    return 0 as uint8;
}
pub unsafe fn HandleArmCannonOpenState() {
    if *(&raw mut g_ram as *mut uint8).offset(0xaa7 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        != 0
        || UpdateArmCannonIsOpenFlag() as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
    {
        AdvanceArmCannonFrame();
    }
    *((&raw mut g_ram as *mut uint8).offset(0xaac as ::core::ffi::c_int as isize) as *mut uint16) =
        *RomPtr_90(
            *(g_rom.offset(
                (((0x90c7df as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x90c7df as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(
                    *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                        as *mut uint16) as isize,
                ) as uint16_t,
        )
        .offset(1 as ::core::ffi::c_int as isize) as uint16;
}
pub unsafe fn UpdateArmCannonIsOpenFlag() -> uint8 {
    static mut kFlagShouldArmCannonBeOpen: [int8; 6] = [
        0 as ::core::ffi::c_int as int8,
        1 as ::core::ffi::c_int as int8,
        1 as ::core::ffi::c_int as int8,
        0 as ::core::ffi::c_int as int8,
        1 as ::core::ffi::c_int as int8,
        0 as ::core::ffi::c_int as int8,
    ];
    if *((&raw mut g_ram as *mut uint8).offset(0xaaa as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - 2 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        return 0 as uint8;
    }
    let mut r18: uint16 = kFlagShouldArmCannonBeOpen[*((&raw mut g_ram as *mut uint8)
        .offset(0x9d2 as ::core::ffi::c_int as isize)
        as *mut uint16) as usize] as uint16;
    if *(&raw mut g_ram as *mut uint8).offset(0xaa6 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == r18 as ::core::ffi::c_int
    {
        return 0 as uint8;
    }
    if r18 != 0 {
        *((&raw mut g_ram as *mut uint8).offset(0xaa8 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0xaa8 as ::core::ffi::c_int as isize)
            as *mut uint16) = 4 as uint16;
    }
    *((&raw mut g_ram as *mut uint8).offset(0xaa6 as ::core::ffi::c_int as isize) as *mut uint16) =
        (r18 as ::core::ffi::c_int | 0x100 as ::core::ffi::c_int) as uint16;
    return 1 as uint8;
}
pub unsafe fn AdvanceArmCannonFrame() {
    if *(&raw mut g_ram as *mut uint8).offset(0xaa6 as ::core::ffi::c_int as isize) != 0 {
        if *((&raw mut g_ram as *mut uint8).offset(0xaa8 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 2 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            let ref mut fresh8 = *((&raw mut g_ram as *mut uint8)
                .offset(0xaa8 as ::core::ffi::c_int as isize)
                as *mut uint16);
            *fresh8 = (*fresh8).wrapping_add(1);
            return;
        }
        *((&raw mut g_ram as *mut uint8).offset(0xaa8 as ::core::ffi::c_int as isize)
            as *mut uint16) = 3 as uint16;
    } else {
        if *((&raw mut g_ram as *mut uint8).offset(0xaa8 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != 1 as ::core::ffi::c_int
            && (*((&raw mut g_ram as *mut uint8).offset(0xaa8 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int
        {
            let ref mut fresh9 = *((&raw mut g_ram as *mut uint8)
                .offset(0xaa8 as ::core::ffi::c_int as isize)
                as *mut uint16);
            *fresh9 = (*fresh9).wrapping_sub(1);
            return;
        }
        *((&raw mut g_ram as *mut uint8).offset(0xaa8 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    }
    *((&raw mut g_ram as *mut uint8).offset(0xaa6 as ::core::ffi::c_int as isize) as *mut uint16) =
        *(&raw mut g_ram as *mut uint8).offset(0xaa6 as ::core::ffi::c_int as isize) as uint16;
}
pub unsafe fn Samus_ArmCannon_Draw() {
    static mut kDrawArmCannon_Char: [uint16; 10] = [
        0x281f as ::core::ffi::c_int as uint16,
        0x281f as ::core::ffi::c_int as uint16,
        0x281f as ::core::ffi::c_int as uint16,
        0x681f as ::core::ffi::c_int as uint16,
        0xa81f as ::core::ffi::c_int as uint16,
        0xe81f as ::core::ffi::c_int as uint16,
        0x281f as ::core::ffi::c_int as uint16,
        0x681f as ::core::ffi::c_int as uint16,
        0x681f as ::core::ffi::c_int as uint16,
        0x681f as ::core::ffi::c_int as uint16,
    ];
    let mut v3: uint16 = 0;
    let mut r22: uint16 = 0;
    if *((&raw mut g_ram as *mut uint8).offset(0xaa8 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        != 0
        && (*((&raw mut g_ram as *mut uint8).offset(0x18a8 as ::core::ffi::c_int as isize)
            as *mut uint16)
            == 0
            || *((&raw mut g_ram as *mut uint8).offset(0x5b6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 1 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int)
    {
        let mut v0: uint16 = *(g_rom.offset(
            (((0x90c7df as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x90c7df as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(
                *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                    as *mut uint16) as isize,
            );
        let mut v1: *const uint8 = RomPtr_90(v0 as uint16_t);
        let mut v2: int16 = *v1 as int16;
        if v2 as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            if *((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
                as *mut uint16)
                != 0
            {
                v3 = (2 as ::core::ffi::c_int
                    * (*v1.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        & 0x7f as ::core::ffi::c_int)) as uint16;
            } else {
                v3 = (2 as ::core::ffi::c_int
                    * (*v1 as ::core::ffi::c_int & 0x7f as ::core::ffi::c_int))
                    as uint16;
            }
            r22 = (v0 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
        } else {
            v3 = (2 as ::core::ffi::c_int * v2 as ::core::ffi::c_int) as uint16;
            r22 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        }
        let mut r24: uint16 =
            kDrawArmCannon_Char[(v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize];
        let mut v4: *const uint8 = RomPtr_90(
            (r22 as ::core::ffi::c_int
                + 2 as ::core::ffi::c_int
                    * *((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int) as uint16_t,
        );
        let mut r18: uint16 = *v4.offset(0 as ::core::ffi::c_int as isize) as int8 as uint16;
        let mut r20: uint16 = *v4.offset(1 as ::core::ffi::c_int as isize) as int8 as uint16;
        r22 = (*(g_rom.offset(
            (((0x91b629 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x91b629 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut SamusPoseParams)
            .offset(
                *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                    as *mut uint16) as isize,
            ))
        .y_offset_to_gfx as uint16;
        let mut v7: uint16 = *((&raw mut g_ram as *mut uint8)
            .offset(0x590 as ::core::ffi::c_int as isize)
            as *mut uint16);
        let mut v8: int16 = (r18 as ::core::ffi::c_int
            + *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as int16;
        if v8 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
            if v8 as ::core::ffi::c_int - 256 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0
            {
                let mut v9: *mut OamEnt = gOamEnt(
                    *((&raw mut g_ram as *mut uint8).offset(0x590 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int,
                );
                (*v9).xcoord = v8 as uint8;
                let mut v10: int16 = (r20 as ::core::ffi::c_int
                    + *((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                    - r22 as ::core::ffi::c_int
                    - *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int)
                    as int16;
                if v10 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
                    if v10 as ::core::ffi::c_int - 256 as ::core::ffi::c_int
                        & 0x8000 as ::core::ffi::c_int
                        != 0
                    {
                        (*v9).ycoord = v10 as uint8;
                        *(&raw mut (*v9).charnum as *mut uint16) = r24;
                        *((&raw mut g_ram as *mut uint8)
                            .offset(0x590 as ::core::ffi::c_int as isize)
                            as *mut uint16) =
                            (v7 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
                    }
                }
            }
        }
        let mut v11: *const uint8 = RomPtr_90(
            *(g_rom.offset(
                (((0x90c7df as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x90c7df as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(
                    *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                        as *mut uint16) as isize,
                ) as uint16_t,
        );
        let mut v12: uint8 = *v11;
        if v12 as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            v12 = (if *((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                != 0
            {
                *v11.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            } else {
                *v11.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            }) as uint8;
        }
        let mut v13: uint16 = (*(g_rom.offset(
            (((0x90c7a5 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x90c7a5 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset((v12 as ::core::ffi::c_int & 0x7f as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            + 2 as ::core::ffi::c_int
                * *((&raw mut g_ram as *mut uint8).offset(0xaa8 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint16;
        let mut v14: uint16 = *((&raw mut g_ram as *mut uint8)
            .offset(0x330 as ::core::ffi::c_int as isize)
            as *mut uint16);
        (*gVramWriteEntry(
            *((&raw mut g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int,
        ))
        .size = 32 as uint16;
        v14 = (v14 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        let mut v15: uint16 = *(RomPtr_90(v13 as uint16_t) as *mut uint16);
        (*gVramWriteEntry(v14 as ::core::ffi::c_int)).size = v15;
        v14 = (v14 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        let fresh15 = v14;
        v14 = v14.wrapping_add(1);
        *(&raw mut (*(gVramWriteEntry as unsafe fn(::core::ffi::c_int) -> *mut VramWriteEntry)(
            fresh15 as ::core::ffi::c_int,
        ))
        .size as *mut uint8)
            .offset(0 as ::core::ffi::c_int as isize) = -(102 as ::core::ffi::c_int) as uint8;
        (*gVramWriteEntry(v14 as ::core::ffi::c_int)).size =
            addr_unk_6061F0 as ::core::ffi::c_int as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x330 as ::core::ffi::c_int as isize)
            as *mut uint16) = (v14 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    }
}
unsafe fn Projectile_SinLookup_Inner(mut k: uint16, mut r24: uint16) -> uint16 {
    let mut prod: uint16 = Mult8x8(
        *((&raw const kSinCosTable8bit_Sext as *const int16)
            .offset(64 as ::core::ffi::c_int as isize) as *const int16 as *mut uint8)
            .offset(k as ::core::ffi::c_int as isize),
        r24 as uint8,
    );
    let mut r18: uint16 = (prod as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as uint16;
    prod = Mult8x8(
        *((&raw const kSinCosTable8bit_Sext as *const int16)
            .offset(64 as ::core::ffi::c_int as isize) as *const int16 as *mut uint8)
            .offset(k as ::core::ffi::c_int as isize)
            .offset(1 as ::core::ffi::c_int as isize),
        r24 as uint8,
    );
    return (r18 as ::core::ffi::c_int + prod as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Projectile_SinLookup(mut j: uint16, mut a: uint16) -> Point16U {
    let mut v2: uint16 = 0;
    let mut v4: uint16 = 0;
    let mut r24: uint16 = a;
    let mut r26: uint16 = j;
    if j as ::core::ffi::c_int - 128 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        v2 = Projectile_SinLookup_Inner(
            (2 as ::core::ffi::c_int * j as ::core::ffi::c_int) as uint16,
            r24,
        );
    } else {
        v2 = -(Projectile_SinLookup_Inner(
            (2 as ::core::ffi::c_int
                * (j as ::core::ffi::c_int + 0x80 as ::core::ffi::c_int) as uint8
                    as ::core::ffi::c_int) as uint16,
            r24,
        ) as ::core::ffi::c_int) as uint16;
    }
    let mut v3: int16 = (r26 as ::core::ffi::c_int - 64 as ::core::ffi::c_int) as uint8 as int16;
    if v3 as ::core::ffi::c_int - 128 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        v4 = Projectile_SinLookup_Inner(
            (2 as ::core::ffi::c_int * v3 as ::core::ffi::c_int) as uint16,
            r24,
        );
    } else {
        v4 = -(Projectile_SinLookup_Inner(
            (2 as ::core::ffi::c_int
                * (v3 as ::core::ffi::c_int + 0x80 as ::core::ffi::c_int) as uint8
                    as ::core::ffi::c_int) as uint16,
            r24,
        ) as ::core::ffi::c_int) as uint16;
    }
    return Point16U { x: v2, y: v4 };
}
static mut kCostOfSbaInPowerBombs: [uint16; 12] = [
    0 as ::core::ffi::c_int as uint16,
    1 as ::core::ffi::c_int as uint16,
    1 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    1 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    1 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
];
static mut kFireSbaFuncs: [Option<Func_V_A>; 12] = unsafe {
    [
        Some(FireSba_ClearCarry as unsafe fn() -> uint8),
        Some(FireSba_FireWave as unsafe fn() -> uint8),
        Some(FireSba_FireIce as unsafe fn() -> uint8),
        Some(FireSba_ClearCarry as unsafe fn() -> uint8),
        Some(FireSba_FireSpazer as unsafe fn() -> uint8),
        Some(FireSba_ClearCarry as unsafe fn() -> uint8),
        Some(FireSba_ClearCarry as unsafe fn() -> uint8),
        Some(FireSba_ClearCarry as unsafe fn() -> uint8),
        Some(FireSba_FirePlasma as unsafe fn() -> uint8),
        Some(FireSba_ClearCarry as unsafe fn() -> uint8),
        Some(FireSba_ClearCarry as unsafe fn() -> uint8),
        Some(FireSba_ClearCarry as unsafe fn() -> uint8),
    ]
};
pub unsafe fn FireSba() -> uint8 {
    let mut v2: int16 = 0;
    if *((&raw mut g_ram as *mut uint8).offset(0x9d2 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        != 3 as ::core::ffi::c_int
    {
        return 0 as uint8;
    }
    let mut v1: uint16 = (2 as ::core::ffi::c_int
        * (*((&raw mut g_ram as *mut uint8).offset(0x9a6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int)) as uint16;
    v2 = (*((&raw mut g_ram as *mut uint8).offset(0x9ce as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - kCostOfSbaInPowerBombs[(v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize]
            as ::core::ffi::c_int) as int16;
    if (v2 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
        v2 = 0 as int16;
    }
    *((&raw mut g_ram as *mut uint8).offset(0x9ce as ::core::ffi::c_int as isize) as *mut uint16) =
        v2 as uint16;
    let mut rv: uint8 = kFireSbaFuncs
        [(v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize]
        .expect("non-null function pointer")();
    if *((&raw mut g_ram as *mut uint8).offset(0x9ce as ::core::ffi::c_int as isize) as *mut uint16)
        == 0
    {
        *((&raw mut g_ram as *mut uint8).offset(0x9d2 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xa04 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    }
    return rv;
}
pub unsafe fn FireSba_ClearCarry() -> uint8 {
    return 0 as uint8;
}
pub unsafe fn FireSba_FireWave() -> uint8 {
    static mut kFireSba_FireWave_X: [int16; 4] = [
        128 as ::core::ffi::c_int as int16,
        128 as ::core::ffi::c_int as int16,
        -(128 as ::core::ffi::c_int) as int16,
        -(128 as ::core::ffi::c_int) as int16,
    ];
    static mut kFireSba_FireWave_Y: [int16; 4] = [
        128 as ::core::ffi::c_int as int16,
        -(128 as ::core::ffi::c_int) as int16,
        -(128 as ::core::ffi::c_int) as int16,
        128 as ::core::ffi::c_int as int16,
    ];
    let mut i: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        let mut v1: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
        *((&raw mut g_ram as *mut uint8).offset(0xc90 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 4 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = (*((&raw mut g_ram as *mut uint8)
            .offset(0x9a6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x100f as ::core::ffi::c_int
            | 0x8010 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xc04 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xc68 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) =
            (0x90da08 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xbf0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 600 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xbdc as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb8c as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xba0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb64 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = (kFireSba_FireWave_X[v1 as usize] as ::core::ffi::c_int
            + *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = (kFireSba_FireWave_Y[v1 as usize] as ::core::ffi::c_int
            + *((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16;
        InitializeSbaProjectile(i as uint16);
        i -= 2 as ::core::ffi::c_int;
    }
    *((&raw mut g_ram as *mut uint8).offset(0xcce as ::core::ffi::c_int as isize) as *mut uint16) =
        4 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xccc as ::core::ffi::c_int as isize) as *mut uint16) =
        kProjectileCooldown_Uncharged[(*((&raw mut g_ram as *mut uint8)
            .offset(0xc18 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(0 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            & 0x3f as ::core::ffi::c_int) as usize] as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb60 as ::core::ffi::c_int as isize) as *mut uint16) =
        4 as uint16;
    QueueSfx1_Max6(0x28 as uint16);
    return 1 as uint8;
}
pub unsafe fn FireSba_FireIce() -> uint8 {
    static mut kIcePlasmaSbaProjectileOriginAngles: [uint16; 8] = [
        0 as ::core::ffi::c_int as uint16,
        0x40 as ::core::ffi::c_int as uint16,
        0x80 as ::core::ffi::c_int as uint16,
        0xc0 as ::core::ffi::c_int as uint16,
        0x20 as ::core::ffi::c_int as uint16,
        0x60 as ::core::ffi::c_int as uint16,
        0xa0 as ::core::ffi::c_int as uint16,
        0xe0 as ::core::ffi::c_int as uint16,
    ];
    if *((&raw mut g_ram as *mut uint8).offset(0xc68 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == 0x90cf7a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
        || *((&raw mut g_ram as *mut uint8).offset(0xc68 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 0x90cf09 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        return 0 as uint8;
    }
    let mut i: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        let mut v2: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
        *((&raw mut g_ram as *mut uint8).offset(0xc90 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) = 4 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) = (*((&raw mut g_ram as *mut uint8)
            .offset(0x9a6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x100f as ::core::ffi::c_int
            | 0x8010 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xc04 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xc68 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) =
            (0x90cf09 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) = kIcePlasmaSbaProjectileOriginAngles[v2 as usize];
        *((&raw mut g_ram as *mut uint8).offset(0xbf0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) = 600 as uint16;
        InitializeProjectile(i as uint16);
        i -= 2 as ::core::ffi::c_int;
    }
    *((&raw mut g_ram as *mut uint8).offset(0xcce as ::core::ffi::c_int as isize) as *mut uint16) =
        4 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xccc as ::core::ffi::c_int as isize) as *mut uint16) =
        kProjectileCooldown_Uncharged[(*((&raw mut g_ram as *mut uint8)
            .offset(0xc18 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(0 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            & 0x3f as ::core::ffi::c_int) as usize] as uint16;
    if *(&raw mut g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == 4 as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0xb60 as ::core::ffi::c_int as isize)
            as *mut uint16) = -(4 as ::core::ffi::c_int) as uint16;
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0xb60 as ::core::ffi::c_int as isize)
            as *mut uint16) = 4 as uint16;
    }
    QueueSfx1_Max6(0x23 as uint16);
    return 1 as uint8;
}
pub unsafe fn FireSba_FireSpazer() -> uint8 {
    static mut kFireSpazer_Timer: [int16; 4] = [
        0 as ::core::ffi::c_int as int16,
        0 as ::core::ffi::c_int as int16,
        4 as ::core::ffi::c_int as int16,
        4 as ::core::ffi::c_int as int16,
    ];
    static mut kFireSpazer_Yspeed: [int16; 4] = [
        4 as ::core::ffi::c_int as int16,
        -(4 as ::core::ffi::c_int) as int16,
        4 as ::core::ffi::c_int as int16,
        -(4 as ::core::ffi::c_int) as int16,
    ];
    let mut i: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        let mut v1: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
        *((&raw mut g_ram as *mut uint8).offset(0xc90 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = kFireSpazer_Timer[v1 as usize] as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xc04 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 5 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xc68 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) =
            (0x90db06 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xbdc as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 40 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xbf0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = kFireSpazer_Yspeed[v1 as usize] as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xca4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb8c as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xba0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 0 as uint16;
        if (i - 4 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
            *((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((i >> 1 as ::core::ffi::c_int) as isize) =
                -(32732 as ::core::ffi::c_int) as uint16;
            InitializeShinesparkEchoOrSpazerSba(i as uint16);
        } else {
            *((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((i >> 1 as ::core::ffi::c_int) as isize) =
                (*((&raw mut g_ram as *mut uint8).offset(0x9a6 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    & 0x100f as ::core::ffi::c_int
                    | 0x8010 as ::core::ffi::c_int) as uint16;
            InitializeSbaProjectile(i as uint16);
        }
        i -= 2 as ::core::ffi::c_int;
    }
    *((&raw mut g_ram as *mut uint8).offset(0xcce as ::core::ffi::c_int as isize) as *mut uint16) =
        4 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xccc as ::core::ffi::c_int as isize) as *mut uint16) =
        kProjectileCooldown_Uncharged[(*((&raw mut g_ram as *mut uint8)
            .offset(0xc18 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(0 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            & 0x3f as ::core::ffi::c_int) as usize] as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb60 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    QueueSfx1_Max6(0x25 as uint16);
    return 1 as uint8;
}
pub unsafe fn FireSba_FirePlasma() -> uint8 {
    static mut kIcePlasmaSbaProjectileOriginAngles: [uint16; 8] = [
        0 as ::core::ffi::c_int as uint16,
        0x40 as ::core::ffi::c_int as uint16,
        0x80 as ::core::ffi::c_int as uint16,
        0xc0 as ::core::ffi::c_int as uint16,
        0x20 as ::core::ffi::c_int as uint16,
        0x60 as ::core::ffi::c_int as uint16,
        0xa0 as ::core::ffi::c_int as uint16,
        0xe0 as ::core::ffi::c_int as uint16,
    ];
    if *((&raw mut g_ram as *mut uint8).offset(0xc68 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == 0x90d793 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        return 0 as uint8;
    }
    let mut i: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        let mut v2: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
        *((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) = (*((&raw mut g_ram as *mut uint8)
            .offset(0x9a6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x100f as ::core::ffi::c_int
            | 0x8010 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xc04 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xc68 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) =
            (0x90d793 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) = kIcePlasmaSbaProjectileOriginAngles[v2 as usize];
        *((&raw mut g_ram as *mut uint8).offset(0xbdc as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) = 40 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xbf0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) = 0 as uint16;
        InitializeSbaProjectile(i as uint16);
        i -= 2 as ::core::ffi::c_int;
    }
    *((&raw mut g_ram as *mut uint8).offset(0xcce as ::core::ffi::c_int as isize) as *mut uint16) =
        4 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xccc as ::core::ffi::c_int as isize) as *mut uint16) =
        kProjectileCooldown_Uncharged[(*((&raw mut g_ram as *mut uint8)
            .offset(0xc18 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(0 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            & 0x3f as ::core::ffi::c_int) as usize] as uint16;
    if *(&raw mut g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == 4 as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0xb60 as ::core::ffi::c_int as isize)
            as *mut uint16) = -(4 as ::core::ffi::c_int) as uint16;
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0xb60 as ::core::ffi::c_int as isize)
            as *mut uint16) = 4 as uint16;
    }
    QueueSfx1_Max6(0x27 as uint16);
    return 1 as uint8;
}
pub unsafe fn ProjPreInstr_IceSba(mut k: uint16) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut g_ram as *mut uint8).offset(0xc04 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        & 0xf0 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        QueueSfx1_Max6(0x24 as uint16);
        ClearProjectile(k);
    } else {
        let ref mut fresh49 = *((&raw mut g_ram as *mut uint8)
            .offset(0xc90 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        let fresh50 = *fresh49;
        *fresh49 = (*fresh49).wrapping_sub(1);
        let mut v2: bool = fresh50 as ::core::ffi::c_int == 1 as ::core::ffi::c_int;
        if v2 {
            *((&raw mut g_ram as *mut uint8).offset(0xc90 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = 4 as uint16;
            SpawnProjectileTrail(k);
            k = *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
                as *mut uint16);
        }
        let mut v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut pt: Point16U = Projectile_SinLookup(
            *((&raw mut g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v3 as isize),
            0x20 as uint16,
        );
        *((&raw mut g_ram as *mut uint8).offset(0xb64 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v3 as isize) = (pt.x as ::core::ffi::c_int
            + *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v3 as isize) = (pt.y as ::core::ffi::c_int
            + *((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v3 as isize) =
            (*((&raw mut g_ram as *mut uint8).offset(0xb60 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + *((&raw mut g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v3 as isize) as ::core::ffi::c_int) as uint8 as uint16;
        let ref mut fresh53 = *((&raw mut g_ram as *mut uint8)
            .offset(0xbf0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v3 as isize);
        let fresh54 = *fresh53;
        *fresh53 = (*fresh53).wrapping_sub(1);
        v2 = fresh54 as ::core::ffi::c_int == 1 as ::core::ffi::c_int;
        if v2 {
            *((&raw mut g_ram as *mut uint8).offset(0xc68 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v3 as isize) =
                (0x90cf7a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xbdc as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v3 as isize) = 40 as uint16;
            QueueSfx1_Max6(0x24 as uint16);
        }
        *((&raw mut g_ram as *mut uint8).offset(0xccc as ::core::ffi::c_int as isize)
            as *mut uint16) = 2 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xcd0 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    };
}
pub unsafe fn ProjPreInstr_IceSba2(mut k: uint16) {
    let mut v3: ::core::ffi::c_int = 0;
    let mut pt: Point16U = Point16U { x: 0, y: 0 };
    let mut current_block: u64;
    let mut v5: int16 = 0;
    let mut v6: int16 = 0;
    let mut v7: int16 = 0;
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut g_ram as *mut uint8).offset(0xc04 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        & 0xf0 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        current_block = 3458091977554017730;
    } else {
        let ref mut fresh45 = *((&raw mut g_ram as *mut uint8)
            .offset(0xc90 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        let fresh46 = *fresh45;
        *fresh45 = (*fresh45).wrapping_sub(1);
        if fresh46 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
            *((&raw mut g_ram as *mut uint8).offset(0xc90 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = 4 as uint16;
            SpawnProjectileTrail(k);
            k = *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
                as *mut uint16);
        }
        v3 = 0;
        v3 = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        pt = Projectile_SinLookup(
            *((&raw mut g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v3 as isize),
            *((&raw mut g_ram as *mut uint8).offset(0xbdc as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v3 as isize),
        );
        pt.x = (pt.x as ::core::ffi::c_int
            + *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb64 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v3 as isize) = pt.x;
        v5 = (pt.x as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as int16;
        if v5 as ::core::ffi::c_int + 32 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0
            || v5 as ::core::ffi::c_int - 288 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == 0
            || {
                v6 = (pt.y as ::core::ffi::c_int
                    + *((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int) as int16;
                *((&raw mut g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v3 as isize) = (pt.y as ::core::ffi::c_int
                    + *((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int)
                    as uint16;
                v7 = (v6 as ::core::ffi::c_int
                    - *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int) as int16;
                v7 as ::core::ffi::c_int - 16 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                    != 0
            }
            || v7 as ::core::ffi::c_int - 256 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == 0
        {
            current_block = 3458091977554017730;
        } else {
            *((&raw mut g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v3 as isize) = (*((&raw mut g_ram as *mut uint8)
                .offset(0xb60 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + *((&raw mut g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v3 as isize) as ::core::ffi::c_int)
                as uint8 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xbdc as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v3 as isize) =
                (*((&raw mut g_ram as *mut uint8).offset(0xbdc as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v3 as isize) as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int) as uint8 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xccc as ::core::ffi::c_int as isize)
                as *mut uint16) = 2 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xcd0 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            current_block = 17860125682698302841;
        }
    }
    match current_block {
        3458091977554017730 => {
            ClearProjectile(k);
        }
        _ => {}
    };
}
pub unsafe fn Projectile_Func7_Shinespark() {
    *((&raw mut g_ram as *mut uint8).offset(0xa58 as ::core::ffi::c_int as isize) as *mut uint16) =
        (0x90d068 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb36 as ::core::ffi::c_int as isize) as *mut uint16) =
        1 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb3e as ::core::ffi::c_int as isize) as *mut uint16) =
        1024 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa52 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb42 as ::core::ffi::c_int as isize) as *mut uint16) =
        8 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb44 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb46 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb48 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xdec as ::core::ffi::c_int as isize) as *mut uint16) =
        7 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xdee as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xccc as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xaa2 as ::core::ffi::c_int as isize) as *mut uint16) =
        30 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa68 as ::core::ffi::c_int as isize) as *mut uint16) =
        60 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xacc as ::core::ffi::c_int as isize) as *mut uint16) =
        6 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xace as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa56 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    if *((&raw mut g_ram as *mut uint8).offset(0xcd0 as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        if *((&raw mut g_ram as *mut uint8).offset(0xcd0 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 16 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            QueueSfx1_Max9(2 as uint16);
        }
        *((&raw mut g_ram as *mut uint8).offset(0xcd0 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        ClearFlareAnimationState();
    }
}
pub unsafe fn Samus_MoveHandlerShinesparkWindup() {
    let ref mut fresh124 = *((&raw mut g_ram as *mut uint8)
        .offset(0xaa2 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh124 = (*fresh124).wrapping_sub(1);
    let mut v0: bool =
        *fresh124 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int;
    if *((&raw mut g_ram as *mut uint8).offset(0xaa2 as ::core::ffi::c_int as isize) as *mut uint16)
        == 0
        || v0 as ::core::ffi::c_int != 0
    {
        if *(&raw mut g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            == 4 as ::core::ffi::c_int
        {
            *((&raw mut g_ram as *mut uint8).offset(0xa2a as ::core::ffi::c_int as isize)
                as *mut uint16) = 204 as uint16;
        } else {
            *((&raw mut g_ram as *mut uint8).offset(0xa2a as ::core::ffi::c_int as isize)
                as *mut uint16) = 203 as uint16;
        }
        *((&raw mut g_ram as *mut uint8).offset(0xa58 as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x90d0ab as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xa60 as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x90e90e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xaae as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xac0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(0 as ::core::ffi::c_int as isize) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xac0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(1 as ::core::ffi::c_int as isize) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xab0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(0 as ::core::ffi::c_int as isize) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xab0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(1 as ::core::ffi::c_int as isize) = 0 as uint16;
        QueueSfx3_Max9(0xf as uint16);
    }
}
pub unsafe fn Samus_MoveHandlerVerticalShinespark() {
    *((&raw mut g_ram as *mut uint8).offset(0xa6e as ::core::ffi::c_int as isize) as *mut uint16) =
        2 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa48 as ::core::ffi::c_int as isize) as *mut uint16) =
        8 as uint16;
    Samus_UpdateSpeedEchoPos();
    Samus_ShinesparkMove_Y();
    Samus_EndSuperJump();
    if *((&raw mut g_ram as *mut uint8).offset(0x9c2 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - 30 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
        && {
            let ref mut fresh122 = *((&raw mut g_ram as *mut uint8)
                .offset(0x9c2 as ::core::ffi::c_int as isize)
                as *mut uint16);
            *fresh122 = (*fresh122).wrapping_sub(1);
            *fresh122 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
        }
    {
        *((&raw mut g_ram as *mut uint8).offset(0x9c2 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    }
}
pub unsafe fn Samus_MoveHandler_Shinespark_Diag() {
    *((&raw mut g_ram as *mut uint8).offset(0xa6e as ::core::ffi::c_int as isize) as *mut uint16) =
        2 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa48 as ::core::ffi::c_int as isize) as *mut uint16) =
        8 as uint16;
    Samus_UpdateSpeedEchoPos();
    Samus_ShinesparkMove_X();
    Samus_ShinesparkMove_Y();
    Samus_EndSuperJump();
    if *((&raw mut g_ram as *mut uint8).offset(0x9c2 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - 30 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
        && {
            let ref mut fresh121 = *((&raw mut g_ram as *mut uint8)
                .offset(0x9c2 as ::core::ffi::c_int as isize)
                as *mut uint16);
            *fresh121 = (*fresh121).wrapping_sub(1);
            *fresh121 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
        }
    {
        *((&raw mut g_ram as *mut uint8).offset(0x9c2 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    }
}
pub unsafe fn Samus_MoveHandler_Shinespark_Horiz() {
    *((&raw mut g_ram as *mut uint8).offset(0xa6e as ::core::ffi::c_int as isize) as *mut uint16) =
        2 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa48 as ::core::ffi::c_int as isize) as *mut uint16) =
        8 as uint16;
    Samus_UpdateSpeedEchoPos();
    Samus_ShinesparkMove_X();
    Samus_EndSuperJump();
    if *((&raw mut g_ram as *mut uint8).offset(0x9c2 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - 30 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
        && {
            let ref mut fresh120 = *((&raw mut g_ram as *mut uint8)
                .offset(0x9c2 as ::core::ffi::c_int as isize)
                as *mut uint16);
            *fresh120 = (*fresh120).wrapping_sub(1);
            *fresh120 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
        }
    {
        *((&raw mut g_ram as *mut uint8).offset(0x9c2 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    }
}
unsafe fn Samus_ClampSpeedHi(mut amt: int32, mut val: ::core::ffi::c_int) -> uint32 {
    if (amt >> 16 as ::core::ffi::c_int) - val as int32 & 0x8000 as int32 == 0 {
        amt = (val as int32) << 16 as ::core::ffi::c_int | amt & 0xffff as int32;
    }
    return amt as uint32;
}
pub unsafe fn Samus_ShinesparkMove_X() {
    let mut current_block: u64;
    let mut v4: int16 = 0;
    *((&raw mut g_ram as *mut uint8).offset(0xa68 as ::core::ffi::c_int as isize) as *mut uint16) =
        15 as uint16;
    AddToHiLo(
        (&raw mut g_ram as *mut uint8).offset(0xb42 as ::core::ffi::c_int as isize) as *mut uint16,
        (&raw mut g_ram as *mut uint8).offset(0xb44 as ::core::ffi::c_int as isize) as *mut uint16,
        (*((&raw mut g_ram as *mut uint8).offset(0xb34 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint32)
            << 16 as ::core::ffi::c_int
            | *((&raw mut g_ram as *mut uint8).offset(0xb32 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint32,
    );
    if *((&raw mut g_ram as *mut uint8).offset(0xb42 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - 15 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        SetHiLo(
            (&raw mut g_ram as *mut uint8).offset(0xb42 as ::core::ffi::c_int as isize)
                as *mut uint16,
            (&raw mut g_ram as *mut uint8).offset(0xb44 as ::core::ffi::c_int as isize)
                as *mut uint16,
            ((15 as ::core::ffi::c_int as int16 as ::core::ffi::c_int) << 16 as ::core::ffi::c_int)
                as uint32,
        );
    }
    let mut amt: int32 = 0 as int32;
    if *(&raw mut g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == 4 as ::core::ffi::c_int
    {
        amt = Samus_ClampSpeedHi(
            -Samus_CalcDisplacementMoveLeft(amt),
            15 as ::core::ffi::c_int,
        ) as int32;
        let mut cres: CheckEnemyColl_Result = Samus_CheckSolidEnemyColl(amt);
        amt = cres.amt;
        if cres.collision != 0 {
            *((&raw mut g_ram as *mut uint8).offset(0xdd0 as ::core::ffi::c_int as isize)
                as *mut uint16) = cres.collision;
            current_block = 13493817977953381822;
        } else {
            amt = -amt;
            current_block = 12800627514080957624;
        }
    } else {
        amt = Samus_ClampSpeedHi(
            Samus_CalcDisplacementMoveRight(amt),
            25 as ::core::ffi::c_int,
        ) as int32;
        let mut cres_0: CheckEnemyColl_Result = Samus_CheckSolidEnemyColl(amt);
        amt = cres_0.amt;
        if cres_0.collision != 0 {
            *((&raw mut g_ram as *mut uint8).offset(0xdd0 as ::core::ffi::c_int as isize)
                as *mut uint16) = cres_0.collision;
            current_block = 13493817977953381822;
        } else {
            current_block = 12800627514080957624;
        }
    }
    match current_block {
        12800627514080957624 => {
            Samus_MoveRight_NoSolidColl(amt);
            Samus_AlignYPosSlope();
        }
        _ => {}
    }
    v4 = (*((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0xb10 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16;
    if ((*((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0xb10 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        if v4 as ::core::ffi::c_int + 15 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            *((&raw mut g_ram as *mut uint8).offset(0xb10 as ::core::ffi::c_int as isize)
                as *mut uint16) = (*((&raw mut g_ram as *mut uint8)
                .offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + 15 as ::core::ffi::c_int) as uint16;
        }
    } else if v4 as ::core::ffi::c_int - 16 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        == 0
    {
        *((&raw mut g_ram as *mut uint8).offset(0xb10 as ::core::ffi::c_int as isize)
            as *mut uint16) = (*((&raw mut g_ram as *mut uint8)
            .offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 15 as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Samus_ShinesparkMove_Y() {
    *((&raw mut g_ram as *mut uint8).offset(0xa68 as ::core::ffi::c_int as isize) as *mut uint16) =
        15 as uint16;
    AddToHiLo(
        (&raw mut g_ram as *mut uint8).offset(0xdec as ::core::ffi::c_int as isize) as *mut uint16,
        (&raw mut g_ram as *mut uint8).offset(0xdee as ::core::ffi::c_int as isize) as *mut uint16,
        (*((&raw mut g_ram as *mut uint8).offset(0xb34 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint32)
            << 16 as ::core::ffi::c_int
            | *((&raw mut g_ram as *mut uint8).offset(0xb32 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint32,
    );
    AddToHiLo(
        (&raw mut g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize) as *mut uint16,
        (&raw mut g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize) as *mut uint16,
        (*((&raw mut g_ram as *mut uint8).offset(0xdec as ::core::ffi::c_int as isize)
            as *mut uint16) as uint32)
            << 16 as ::core::ffi::c_int
            | *((&raw mut g_ram as *mut uint8).offset(0xdee as ::core::ffi::c_int as isize)
                as *mut uint16) as uint32,
    );
    let mut amt: int32 = Samus_ClampSpeedHi(
        ((*((&raw mut g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
            as *mut uint16) as uint32)
            << 16 as ::core::ffi::c_int
            | *((&raw mut g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize)
                as *mut uint16) as uint32) as int32,
        14 as ::core::ffi::c_int,
    ) as int32;
    amt = (amt as uint32).wrapping_sub(
        (*((&raw mut g_ram as *mut uint8).offset(0xb5c as ::core::ffi::c_int as isize)
            as *mut uint16) as uint32)
            << 16 as ::core::ffi::c_int
            | *((&raw mut g_ram as *mut uint8).offset(0xb5a as ::core::ffi::c_int as isize)
                as *mut uint16) as uint32,
    ) as int32 as int32;
    *((&raw mut g_ram as *mut uint8).offset(0xb02 as ::core::ffi::c_int as isize) as *mut uint16) =
        2 as uint16;
    let mut cres: CheckEnemyColl_Result =
        Samus_CheckSolidEnemyColl(Samus_ClampSpeedHi(amt, 15 as ::core::ffi::c_int) as int32);
    amt = cres.amt;
    if cres.collision != 0 {
        *((&raw mut g_ram as *mut uint8).offset(0xdd0 as ::core::ffi::c_int as isize)
            as *mut uint16) = cres.collision;
    } else {
        Samus_MoveDown_NoSolidColl(-amt);
    }
    if *((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0xb14 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        + 14 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        *((&raw mut g_ram as *mut uint8).offset(0xb14 as ::core::ffi::c_int as isize)
            as *mut uint16) = (*((&raw mut g_ram as *mut uint8)
            .offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + 14 as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Samus_EndSuperJump() -> uint8 {
    if *((&raw mut g_ram as *mut uint8).offset(0x9c2 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - 30 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
        && *((&raw mut g_ram as *mut uint8).offset(0xdd0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            == 0
    {
        return 0 as uint8;
    }
    if *(&raw mut g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == 4 as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0xac0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(0 as ::core::ffi::c_int as isize) = 32 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xac0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(1 as ::core::ffi::c_int as isize) = 160 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xab0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(2 as ::core::ffi::c_int as isize) = 4 as uint16;
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0xac0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(0 as ::core::ffi::c_int as isize) = 224 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xac0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(1 as ::core::ffi::c_int as isize) = 96 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xab0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(2 as ::core::ffi::c_int as isize) = -(4 as ::core::ffi::c_int) as uint16;
    }
    *((&raw mut g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb42 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb44 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb3e as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb1a as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb36 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa58 as ::core::ffi::c_int as isize) as *mut uint16) =
        (0x90d346 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa5c as ::core::ffi::c_int as isize) as *mut uint16) =
        (0x90ebf3 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xaae as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xab0 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(0 as ::core::ffi::c_int as isize) = *((&raw mut g_ram as *mut uint8)
        .offset(0xaf6 as ::core::ffi::c_int as isize)
        as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0xab0 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(1 as ::core::ffi::c_int as isize) = *((&raw mut g_ram as *mut uint8)
        .offset(0xaf6 as ::core::ffi::c_int as isize)
        as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0xab8 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(0 as ::core::ffi::c_int as isize) = *((&raw mut g_ram as *mut uint8)
        .offset(0xafa as ::core::ffi::c_int as isize)
        as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0xab8 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(1 as ::core::ffi::c_int as isize) = *((&raw mut g_ram as *mut uint8)
        .offset(0xafa as ::core::ffi::c_int as isize)
        as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0xac0 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(2 as ::core::ffi::c_int as isize) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa48 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    QueueSfx1_Max6(0x35 as uint16);
    QueueSfx3_Max6(0x10 as uint16);
    return 1 as uint8;
}
static mut kSamus_MoveHandler_ShinesparkCrash: [Option<Func_V>; 3] = unsafe {
    [
        Some(Samus_MoveHandler_ShinesparkCrash_0 as unsafe fn() -> ()),
        Some(Samus_MoveHandler_ShinesparkCrash_1 as unsafe fn() -> ()),
        Some(Samus_MoveHandler_ShinesparkCrash_2 as unsafe fn() -> ()),
    ]
};
pub unsafe fn Samus_MoveHandler_ShinesparkCrash() {
    *((&raw mut g_ram as *mut uint8).offset(0xa68 as ::core::ffi::c_int as isize) as *mut uint16) =
        15 as uint16;
    kSamus_MoveHandler_ShinesparkCrash[*((&raw mut g_ram as *mut uint8)
        .offset(0xaae as ::core::ffi::c_int as isize)
        as *mut uint16 as *mut uint8)
        .offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) as usize]
        .expect("non-null function pointer")();
    let mut i: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        let mut v1: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
        let mut pt: Point16U = Projectile_SinLookup(
            *((&raw mut g_ram as *mut uint8).offset(0xac0 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize),
            *((&raw mut g_ram as *mut uint8).offset(0xaae as ::core::ffi::c_int as isize)
                as *mut uint16) as uint8 as uint16,
        );
        *((&raw mut g_ram as *mut uint8).offset(0xab0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = (pt.x as ::core::ffi::c_int
            + *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xab8 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = (pt.y as ::core::ffi::c_int
            + *((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
}
pub unsafe fn Samus_MoveHandler_ShinesparkCrash_0() {
    let mut v0: uint16 = (*((&raw mut g_ram as *mut uint8)
        .offset(0xaae as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        + 4 as ::core::ffi::c_int) as uint16;
    if *((&raw mut g_ram as *mut uint8).offset(0xaae as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - 12 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        v0 = (v0 as ::core::ffi::c_int | 0x100 as ::core::ffi::c_int) as uint16;
    }
    *((&raw mut g_ram as *mut uint8).offset(0xaae as ::core::ffi::c_int as isize) as *mut uint16) =
        v0;
}
pub unsafe fn Samus_MoveHandler_ShinesparkCrash_1() {
    *((&raw mut g_ram as *mut uint8).offset(0xac0 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(0 as ::core::ffi::c_int as isize) =
        (*(((&raw mut g_ram as *mut uint8).offset(0xab0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(2 as ::core::ffi::c_int as isize) as *mut uint16 as *mut uint8)
            .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            + *(((&raw mut g_ram as *mut uint8).offset(0xac0 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(0 as ::core::ffi::c_int as isize) as *mut uint16
                as *mut uint8)
                .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int) as uint8
            as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xac0 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(1 as ::core::ffi::c_int as isize) =
        (*(((&raw mut g_ram as *mut uint8).offset(0xab0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(2 as ::core::ffi::c_int as isize) as *mut uint16 as *mut uint8)
            .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            + *(((&raw mut g_ram as *mut uint8).offset(0xac0 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(1 as ::core::ffi::c_int as isize) as *mut uint16
                as *mut uint8)
                .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int) as uint8
            as uint16;
    let ref mut fresh119 = *((&raw mut g_ram as *mut uint8)
        .offset(0xab8 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(2 as ::core::ffi::c_int as isize);
    *fresh119 = (*fresh119 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as uint16;
    if *((&raw mut g_ram as *mut uint8).offset(0xab8 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        - 128 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        *((&raw mut g_ram as *mut uint8).offset(0xaae as ::core::ffi::c_int as isize)
            as *mut uint16) = (*((&raw mut g_ram as *mut uint8)
            .offset(0xaae as ::core::ffi::c_int as isize)
            as *mut uint16) as uint8 as ::core::ffi::c_int
            | 0x200 as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Samus_MoveHandler_ShinesparkCrash_2() {
    let ref mut fresh118 = *((&raw mut g_ram as *mut uint8)
        .offset(0xaae as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh118 = (*fresh118 as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as uint16;
    if *((&raw mut g_ram as *mut uint8).offset(0xaae as ::core::ffi::c_int as isize) as *mut uint16)
        as uint8
        == 0
    {
        *((&raw mut g_ram as *mut uint8).offset(0xa58 as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x90d3f3 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xaa2 as ::core::ffi::c_int as isize)
            as *mut uint16) = 30 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xaae as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xac0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(0 as ::core::ffi::c_int as isize) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xac0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(1 as ::core::ffi::c_int as isize) = 0 as uint16;
    }
}
pub unsafe fn Samus_MoveHandler_ShinesparkCrashEchoCircle() {
    *((&raw mut g_ram as *mut uint8).offset(0xa68 as ::core::ffi::c_int as isize) as *mut uint16) =
        15 as uint16;
    let ref mut fresh117 = *((&raw mut g_ram as *mut uint8)
        .offset(0xaa2 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh117 = (*fresh117).wrapping_sub(1);
    let mut v0: bool =
        *fresh117 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int;
    if *((&raw mut g_ram as *mut uint8).offset(0xaa2 as ::core::ffi::c_int as isize) as *mut uint16)
        == 0
        || v0 as ::core::ffi::c_int != 0
    {
        *((&raw mut g_ram as *mut uint8).offset(0xa58 as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x90d40d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xa5c as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x90eb52 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Samus_MoveHandler_ShinesparkCrashFinish() {
    static mut kShinesparkCrashFinish_Tab0: [uint8; 12] = [
        0 as ::core::ffi::c_int as uint8,
        0x80 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        0x80 as ::core::ffi::c_int as uint8,
        0x40 as ::core::ffi::c_int as uint8,
        0xc0 as ::core::ffi::c_int as uint8,
        0x40 as ::core::ffi::c_int as uint8,
        0xc0 as ::core::ffi::c_int as uint8,
        0xe0 as ::core::ffi::c_int as uint8,
        0x60 as ::core::ffi::c_int as uint8,
        0x20 as ::core::ffi::c_int as uint8,
        0xa0 as ::core::ffi::c_int as uint8,
    ];
    *((&raw mut g_ram as *mut uint8).offset(0xaae as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    if *((&raw mut g_ram as *mut uint8).offset(0xcce as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - 5 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        if *((&raw mut g_ram as *mut uint8).offset(0xcce as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 4 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            let ref mut fresh114 = *((&raw mut g_ram as *mut uint8)
                .offset(0xcce as ::core::ffi::c_int as isize)
                as *mut uint16);
            *fresh114 = (*fresh114).wrapping_add(1);
            *((&raw mut g_ram as *mut uint8).offset(0xac0 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(2 as ::core::ffi::c_int as isize) = 64 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xab0 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(2 as ::core::ffi::c_int as isize) = *((&raw mut g_ram as *mut uint8)
                .offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16);
            *((&raw mut g_ram as *mut uint8).offset(0xab8 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(2 as ::core::ffi::c_int as isize) = *((&raw mut g_ram as *mut uint8)
                .offset(0xafa as ::core::ffi::c_int as isize)
                as *mut uint16);
            *((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(3 as ::core::ffi::c_int as isize) =
                addr_loc_908029 as ::core::ffi::c_int as uint16;
            InitializeShinesparkEchoOrSpazerSba(6 as uint16);
            *((&raw mut g_ram as *mut uint8).offset(0xc68 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(3 as ::core::ffi::c_int as isize) =
                (0x90d4d2 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(3 as ::core::ffi::c_int as isize) =
                kShinesparkCrashFinish_Tab0[(2 as ::core::ffi::c_int
                    * (*((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        - 201 as ::core::ffi::c_int))
                    as uint16 as usize] as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xbdc as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(3 as ::core::ffi::c_int as isize) = 0 as uint16;
        }
        let ref mut fresh115 = *((&raw mut g_ram as *mut uint8)
            .offset(0xcce as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh115 = (*fresh115).wrapping_add(1);
        *((&raw mut g_ram as *mut uint8).offset(0xac0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(3 as ::core::ffi::c_int as isize) = 64 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xab0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(3 as ::core::ffi::c_int as isize) = *((&raw mut g_ram as *mut uint8)
            .offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *((&raw mut g_ram as *mut uint8).offset(0xab8 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(3 as ::core::ffi::c_int as isize) = *((&raw mut g_ram as *mut uint8)
            .offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16);
        *((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(4 as ::core::ffi::c_int as isize) =
            addr_loc_908029 as ::core::ffi::c_int as uint16;
        InitializeShinesparkEchoOrSpazerSba(8 as uint16);
        *((&raw mut g_ram as *mut uint8).offset(0xc68 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(4 as ::core::ffi::c_int as isize) =
            (0x90d4d2 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(4 as ::core::ffi::c_int as isize) =
            kShinesparkCrashFinish_Tab0[((2 as ::core::ffi::c_int
                * (*((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    - 201 as ::core::ffi::c_int)) as uint16
                as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as usize] as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xbdc as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(4 as ::core::ffi::c_int as isize) = 0 as uint16;
    }
    *((&raw mut g_ram as *mut uint8).offset(0xccc as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa68 as ::core::ffi::c_int as isize) as *mut uint16) =
        1 as uint16;
    if *(&raw mut g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == 4 as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0xa2c as ::core::ffi::c_int as isize)
            as *mut uint16) = 2 as uint16;
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0xa2c as ::core::ffi::c_int as isize)
            as *mut uint16) = 1 as uint16;
    }
    *((&raw mut g_ram as *mut uint8).offset(0xa32 as ::core::ffi::c_int as isize) as *mut uint16) =
        2 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xdec as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xdee as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
}
pub unsafe fn ProjPreInstr_SpeedEcho(mut k: uint16) {
    let mut v3: int16 = 0;
    let mut v4: uint16 = 0;
    let mut v5: int16 = 0;
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh44 = *((&raw mut g_ram as *mut uint8)
        .offset(0xbdc as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    *fresh44 = (*fresh44 as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint16;
    let mut pt: Point16U = Projectile_SinLookup(
        *((&raw mut g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize),
        *(((&raw mut g_ram as *mut uint8).offset(0xbdc as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16 as *mut uint8)
            .offset(0 as ::core::ffi::c_int as isize) as uint16,
    );
    let mut v2: uint16 = (pt.x as ::core::ffi::c_int
        + *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as uint16;
    *(((&raw mut g_ram as *mut uint8).offset(0xaae as ::core::ffi::c_int as isize) as *mut uint16
        as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = (pt.x as ::core::ffi::c_int
        + *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int)
        as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb64 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) = v2;
    v3 = (v2 as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16;
    if (v3 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int
        || v3 as ::core::ffi::c_int - 256 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0
        || {
            v4 = (pt.y as ::core::ffi::c_int
                + *((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xab0 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((v1 + 3 as ::core::ffi::c_int) as isize) = (pt.y as ::core::ffi::c_int
                + *((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int)
                as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = v4;
            v5 = (v4 as ::core::ffi::c_int
                - *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as int16;
            (v5 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int
        }
        || v5 as ::core::ffi::c_int - 256 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0
    {
        *((&raw mut g_ram as *mut uint8).offset(0xab8 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v1 + 3 as ::core::ffi::c_int) as isize) = 0 as uint16;
        *(((&raw mut g_ram as *mut uint8).offset(0xaae as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(k as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xab0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((v1 + 3 as ::core::ffi::c_int) as isize) = 0 as uint16;
        ClearProjectile(k);
    }
}
pub unsafe fn Grapple_Func1() {
    let mut v0: bool = false;
    if *((&raw mut g_ram as *mut uint8).offset(0x9b2 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & *((&raw mut g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
        && {
            v0 = ((*((&raw mut g_ram as *mut uint8).offset(0xa9e as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int;
            let ref mut fresh23 = *((&raw mut g_ram as *mut uint8)
                .offset(0xa9e as ::core::ffi::c_int as isize)
                as *mut uint16);
            *fresh23 = (*fresh23).wrapping_sub(1);
            *((&raw mut g_ram as *mut uint8).offset(0xa9e as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                != 0
        }
        && !v0
    {
        if *((&raw mut g_ram as *mut uint8).offset(0xd00 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            let ref mut fresh24 = *((&raw mut g_ram as *mut uint8)
                .offset(0xcfe as ::core::ffi::c_int as isize)
                as *mut uint16);
            *fresh24 = (*fresh24 as ::core::ffi::c_int
                + *((&raw mut g_ram as *mut uint8).offset(0xd00 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint16;
            if *((&raw mut g_ram as *mut uint8).offset(0xcfe as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - 96 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
            {
                *((&raw mut g_ram as *mut uint8).offset(0xd00 as ::core::ffi::c_int as isize)
                    as *mut uint16) = -(16 as ::core::ffi::c_int) as uint16;
            }
        }
        let mut pt: Point16U = Projectile_SinLookup(
            *((&raw mut g_ram as *mut uint8).offset(0xcfa as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ) as uint16,
            *((&raw mut g_ram as *mut uint8).offset(0xcfe as ::core::ffi::c_int as isize)
                as *mut uint16),
        );
        *((&raw mut g_ram as *mut uint8).offset(0xd08 as ::core::ffi::c_int as isize)
            as *mut uint16) = (pt.x as ::core::ffi::c_int
            + *((&raw mut g_ram as *mut uint8).offset(0xd16 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xd0c as ::core::ffi::c_int as isize)
            as *mut uint16) = (pt.y as ::core::ffi::c_int
            + *((&raw mut g_ram as *mut uint8).offset(0xd18 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16;
        let ref mut fresh25 = *((&raw mut g_ram as *mut uint8)
            .offset(0xcfa as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh25 = (*fresh25 as ::core::ffi::c_int + 2048 as ::core::ffi::c_int) as uint16;
        GrappleBeamFunc_BF1B();
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0xd32 as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x9bc856 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn Hdmaobj_CrystalFlash() -> uint8 {
    let mut v1: SamusPose = kPose_00_FaceF_Powersuit;
    if *((&raw mut g_ram as *mut uint8).offset(0x998 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - 40 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        if *((&raw mut g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != *((&raw mut g_ram as *mut uint8).offset(0x9b2 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                | 0x430 as ::core::ffi::c_int
        {
            return 1 as uint8;
        }
    }
    if *((&raw mut g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        != 0
        || *((&raw mut g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != 0
        || *((&raw mut g_ram as *mut uint8).offset(0x9c2 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 51 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        || *((&raw mut g_ram as *mut uint8).offset(0x9d6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != 0
        || *((&raw mut g_ram as *mut uint8).offset(0x9c6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 10 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        || *((&raw mut g_ram as *mut uint8).offset(0x9ca as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 10 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        || *((&raw mut g_ram as *mut uint8).offset(0x9ce as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 10 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
    {
        return 1 as uint8;
    }
    v1 = (if *(&raw mut g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == 4 as ::core::ffi::c_int
    {
        kPose_D4_FaceL_CrystalFlash as ::core::ffi::c_int
    } else {
        kPose_D3_FaceR_CrystalFlash as ::core::ffi::c_int
    }) as SamusPose;
    *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize) as *mut uint16) =
        v1 as uint16;
    SamusFunc_F433();
    Samus_SetAnimationFrameIfPoseChanged();
    if *(&raw mut g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        != 27 as ::core::ffi::c_int
    {
        return 1 as uint8;
    }
    *((&raw mut g_ram as *mut uint8).offset(0xa24 as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0xa20 as ::core::ffi::c_int as isize)
            as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0xa26 as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0xa22 as ::core::ffi::c_int as isize)
            as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0xa20 as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0xa22 as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
            as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0xa58 as ::core::ffi::c_int as isize) as *mut uint16) =
        (0x90d678 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    if *((&raw mut g_ram as *mut uint8).offset(0xa60 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        != 0x90e91d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0xa60 as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x90e90e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    *((&raw mut g_ram as *mut uint8).offset(0xaa2 as ::core::ffi::c_int as isize) as *mut uint16) =
        9 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xdea as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xdec as ::core::ffi::c_int as isize) as *mut uint16) =
        10 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xdee as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xdf0 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xacc as ::core::ffi::c_int as isize) as *mut uint16) =
        7 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xace as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa68 as ::core::ffi::c_int as isize) as *mut uint16) =
        1 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xdf2 as ::core::ffi::c_int as isize) as *mut uint16) =
        1 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x18a8 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x18aa as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa52 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    return 0 as uint8;
}
pub unsafe fn SamusMoveHandler_CrystalFlashStart() {
    let ref mut fresh112 = *((&raw mut g_ram as *mut uint8)
        .offset(0xafa as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh112 = (*fresh112 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
    let ref mut fresh113 = *((&raw mut g_ram as *mut uint8)
        .offset(0xaa2 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh113 = (*fresh113).wrapping_sub(1);
    if *fresh113 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        *((&raw mut g_ram as *mut uint8).offset(0xa94 as ::core::ffi::c_int as isize)
            as *mut uint16) = 3 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
            as *mut uint16) = 6 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xdf0 as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut g_ram as *mut uint8)
            .offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16);
        *((&raw mut g_ram as *mut uint8).offset(0xa58 as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x90d6ce as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x18a8 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x18aa as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        QueueSfx3_Max15(1 as uint16);
        *((&raw mut g_ram as *mut uint8).offset(0xcee as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xce2 as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut g_ram as *mut uint8)
            .offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *((&raw mut g_ram as *mut uint8).offset(0xce4 as ::core::ffi::c_int as isize)
            as *mut uint16) = *((&raw mut g_ram as *mut uint8)
            .offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16);
        EnableHdmaObjects();
        SpawnCrystalFlashHdmaObjs();
    }
}
static mut kSamusMoveHandler_CrystalFlashMainFuncs: [Option<Func_V>; 3] = unsafe {
    [
        Some(SamusMoveHandler_CrystalFlashMain_0 as unsafe fn() -> ()),
        Some(SamusMoveHandler_CrystalFlashMain_1 as unsafe fn() -> ()),
        Some(SamusMoveHandler_CrystalFlashMain_2 as unsafe fn() -> ()),
    ]
};
pub unsafe fn SamusMoveHandler_CrystalFlashMain() {
    kSamusMoveHandler_CrystalFlashMainFuncs[*((&raw mut g_ram as *mut uint8)
        .offset(0xdea as ::core::ffi::c_int as isize)
        as *mut uint16) as usize]
        .expect("non-null function pointer")();
    *((&raw mut g_ram as *mut uint8).offset(0x18a8 as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x18aa as ::core::ffi::c_int as isize)
        as *mut uint16) = 0 as uint16;
}
pub unsafe fn SamusMoveHandler_CrystalFlashMain_0() {
    if *((&raw mut g_ram as *mut uint8).offset(0x5b6 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 7 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        let ref mut fresh109 = *((&raw mut g_ram as *mut uint8)
            .offset(0x9c6 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh109 = (*fresh109).wrapping_sub(1);
        Samus_RestoreHealth(0x32 as uint16);
        let mut v0: bool = ((*((&raw mut g_ram as *mut uint8)
            .offset(0xdec as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 1 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int;
        let ref mut fresh110 = *((&raw mut g_ram as *mut uint8)
            .offset(0xdec as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh110 = (*fresh110).wrapping_sub(1);
        if *fresh110 == 0 || v0 as ::core::ffi::c_int != 0 {
            *((&raw mut g_ram as *mut uint8).offset(0xdec as ::core::ffi::c_int as isize)
                as *mut uint16) = 10 as uint16;
            let ref mut fresh111 = *((&raw mut g_ram as *mut uint8)
                .offset(0xdea as ::core::ffi::c_int as isize)
                as *mut uint16);
            *fresh111 = (*fresh111).wrapping_add(1);
        }
    }
}
pub unsafe fn SamusMoveHandler_CrystalFlashMain_1() {
    if *((&raw mut g_ram as *mut uint8).offset(0x5b6 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 7 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        let ref mut fresh106 = *((&raw mut g_ram as *mut uint8)
            .offset(0x9ca as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh106 = (*fresh106).wrapping_sub(1);
        Samus_RestoreHealth(0x32 as uint16);
        let mut v0: bool = ((*((&raw mut g_ram as *mut uint8)
            .offset(0xdec as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 1 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int;
        let ref mut fresh107 = *((&raw mut g_ram as *mut uint8)
            .offset(0xdec as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh107 = (*fresh107).wrapping_sub(1);
        if *fresh107 == 0 || v0 as ::core::ffi::c_int != 0 {
            *((&raw mut g_ram as *mut uint8).offset(0xdec as ::core::ffi::c_int as isize)
                as *mut uint16) = 10 as uint16;
            let ref mut fresh108 = *((&raw mut g_ram as *mut uint8)
                .offset(0xdea as ::core::ffi::c_int as isize)
                as *mut uint16);
            *fresh108 = (*fresh108).wrapping_add(1);
        }
    }
}
pub unsafe fn SamusMoveHandler_CrystalFlashMain_2() {
    if *((&raw mut g_ram as *mut uint8).offset(0x5b6 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 7 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        let ref mut fresh104 = *((&raw mut g_ram as *mut uint8)
            .offset(0x9ce as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh104 = (*fresh104).wrapping_sub(1);
        Samus_RestoreHealth(0x32 as uint16);
        let mut v0: bool = ((*((&raw mut g_ram as *mut uint8)
            .offset(0xdec as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 1 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int;
        let ref mut fresh105 = *((&raw mut g_ram as *mut uint8)
            .offset(0xdec as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh105 = (*fresh105).wrapping_sub(1);
        if *fresh105 == 0 || v0 as ::core::ffi::c_int != 0 {
            *((&raw mut g_ram as *mut uint8).offset(0xa58 as ::core::ffi::c_int as isize)
                as *mut uint16) =
                (0x90d75b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xa5c as ::core::ffi::c_int as isize)
                as *mut uint16) =
                (0x90eb52 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xa94 as ::core::ffi::c_int as isize)
                as *mut uint16) = 3 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
                as *mut uint16) = 12 as uint16;
        }
    }
}
pub unsafe fn kSamusMoveHandler_CrystalFlashFinish() {
    if *((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        != *((&raw mut g_ram as *mut uint8).offset(0xdf0 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
    {
        let ref mut fresh103 = *((&raw mut g_ram as *mut uint8)
            .offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh103 = (*fresh103).wrapping_add(1);
    }
    if *(&raw mut g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize) == 0 {
        *((&raw mut g_ram as *mut uint8).offset(0xcee as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xa68 as ::core::ffi::c_int as isize)
            as *mut uint16) = -(1 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xa58 as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x90a337 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        if *((&raw mut g_ram as *mut uint8).offset(0xa60 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != 0x90e91d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
        {
            *((&raw mut g_ram as *mut uint8).offset(0xa60 as ::core::ffi::c_int as isize)
                as *mut uint16) =
                (0x90e913 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0x18a8 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0x18aa as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
        }
    }
}
pub unsafe fn ProjPreInstr_PlasmaSba(mut k: uint16) {
    let mut v3: int16 = 0;
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut g_ram as *mut uint8).offset(0xc04 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        & 0xf0 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        ClearProjectile(k);
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0xccc as ::core::ffi::c_int as isize)
            as *mut uint16) = 2 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xcd0 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        let mut pt: Point16U = Projectile_SinLookup(
            *((&raw mut g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize),
            *((&raw mut g_ram as *mut uint8).offset(0xbdc as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize),
        );
        *((&raw mut g_ram as *mut uint8).offset(0xb64 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = (pt.x as ::core::ffi::c_int
            + *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = (pt.y as ::core::ffi::c_int
            + *((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) =
            (*((&raw mut g_ram as *mut uint8).offset(0xb60 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + *((&raw mut g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) as ::core::ffi::c_int) as uint8 as uint16;
        v3 = (2 as ::core::ffi::c_int
            * *((&raw mut g_ram as *mut uint8).offset(0xbf0 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int) as int16;
        if v3 != 0 {
            if v3 as ::core::ffi::c_int == 2 as ::core::ffi::c_int {
                ProjPreInstr_PlasmaSbaFunc_1(k);
            } else {
                if v3 as ::core::ffi::c_int != 4 as ::core::ffi::c_int {
                    Unreachable();
                    loop {}
                }
                ProjPreInstr_PlasmaSbaFunc_2(k);
            }
        } else {
            ProjPreInstr_PlasmaSbaFunc_0(k);
        }
    };
}
pub unsafe fn ProjPreInstr_PlasmaSbaFunc_0(mut j: uint16) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v2: uint16 = (*((&raw mut g_ram as *mut uint8)
        .offset(0xbdc as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        + 4 as ::core::ffi::c_int) as uint8 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xbdc as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) = v2;
    if v2 as ::core::ffi::c_int - 192 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
        *((&raw mut g_ram as *mut uint8).offset(0xbf0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 1 as uint16;
    }
}
pub unsafe fn ProjPreInstr_PlasmaSbaFunc_1(mut j: uint16) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v2: uint16 = (*((&raw mut g_ram as *mut uint8)
        .offset(0xbdc as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        - 4 as ::core::ffi::c_int) as uint8 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xbdc as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) = v2;
    if v2 as ::core::ffi::c_int - 45 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        *((&raw mut g_ram as *mut uint8).offset(0xbf0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 2 as uint16;
    }
}
pub unsafe fn ProjPreInstr_PlasmaSbaFunc_2(mut j: uint16) {
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut g_ram as *mut uint8).offset(0xb64 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        + 32 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
        || *((&raw mut g_ram as *mut uint8).offset(0xb64 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            - 288 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        || *((&raw mut g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            - 16 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        || *((&raw mut g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            - 256 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
    {
        ClearProjectile(j);
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0xbdc as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = (*((&raw mut g_ram as *mut uint8)
            .offset(0xbdc as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            + 4 as ::core::ffi::c_int) as uint8 as uint16;
    };
}
static mut kBombSpread_Tab0: [uint16; 5] = [
    0x78 as ::core::ffi::c_int as uint16,
    0x6e as ::core::ffi::c_int as uint16,
    0x64 as ::core::ffi::c_int as uint16,
    0x6e as ::core::ffi::c_int as uint16,
    0x78 as ::core::ffi::c_int as uint16,
];
static mut kBombSpread_Tab1: [uint16; 5] = [
    0x8100 as ::core::ffi::c_int as uint16,
    0x8080 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0x80 as ::core::ffi::c_int as uint16,
    0x100 as ::core::ffi::c_int as uint16,
];
static mut kBombSpread_Tab2: [uint16; 5] = [
    0 as ::core::ffi::c_int as uint16,
    1 as ::core::ffi::c_int as uint16,
    2 as ::core::ffi::c_int as uint16,
    1 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
];
static mut kBombSpread_Tab3: [uint16; 5] = [
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0x8000 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
];
pub unsafe fn ProjPreInstr_SpreadBomb(mut k: uint16) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut g_ram as *mut uint8).offset(0xc04 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        & 0xf0 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        ClearProjectile(k);
        return;
    }
    Bomb_Func2();
    if *((&raw mut g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize)
        != 0
    {
        AddToHiLo(
            ((&raw mut g_ram as *mut uint8).offset(0xbf0 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as *mut uint16,
            ((&raw mut g_ram as *mut uint8).offset(0xc90 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as *mut uint16,
            (*((&raw mut g_ram as *mut uint8).offset(0xb34 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint32)
                << 16 as ::core::ffi::c_int
                | *((&raw mut g_ram as *mut uint8).offset(0xb32 as ::core::ffi::c_int as isize)
                    as *mut uint16) as uint32,
        );
        AddToHiLo(
            ((&raw mut g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as *mut uint16,
            ((&raw mut g_ram as *mut uint8).offset(0xba0 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as *mut uint16,
            (*((&raw mut g_ram as *mut uint8).offset(0xbf0 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as uint32)
                << 16 as ::core::ffi::c_int
                | *((&raw mut g_ram as *mut uint8).offset(0xc90 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) as uint32,
        );
        if BlockCollSpreadBomb(k) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
            AddToHiLo(
                ((&raw mut g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) as *mut uint16,
                ((&raw mut g_ram as *mut uint8).offset(0xba0 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) as *mut uint16,
                -(((*((&raw mut g_ram as *mut uint8).offset(0xbf0 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) as ::core::ffi::c_int)
                    << 16 as ::core::ffi::c_int
                    | *((&raw mut g_ram as *mut uint8).offset(0xc90 as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset(v1 as isize) as ::core::ffi::c_int) as int32)
                    as uint32,
            );
            if *((&raw mut g_ram as *mut uint8).offset(0xbf0 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                *((&raw mut g_ram as *mut uint8).offset(0xbf0 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) = 0 as uint16;
                *((&raw mut g_ram as *mut uint8).offset(0xbc8 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) = 0 as uint16;
            } else {
                *((&raw mut g_ram as *mut uint8).offset(0xc90 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) =
                    kBombSpread_Tab3[(v1 - 5 as ::core::ffi::c_int) as usize];
                *((&raw mut g_ram as *mut uint8).offset(0xbf0 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) = *((&raw mut g_ram as *mut uint8)
                    .offset(0xca4 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize);
            }
            return;
        }
        let mut t: uint16 = *((&raw mut g_ram as *mut uint8)
            .offset(0xbdc as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        if t as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            AddToHiLo(
                ((&raw mut g_ram as *mut uint8).offset(0xb64 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) as *mut uint16,
                ((&raw mut g_ram as *mut uint8).offset(0xb8c as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) as *mut uint16,
                -(((t as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int) as int16
                    as ::core::ffi::c_int)
                    << 8 as ::core::ffi::c_int) as uint32,
            );
        } else {
            AddToHiLo(
                ((&raw mut g_ram as *mut uint8).offset(0xb64 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) as *mut uint16,
                ((&raw mut g_ram as *mut uint8).offset(0xb8c as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) as *mut uint16,
                ((t as int16 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint32,
            );
        }
    }
    if BlockCollSpreadBomb(k) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        let mut t_0: uint16 = *((&raw mut g_ram as *mut uint8)
            .offset(0xbdc as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        let ref mut fresh43 = *((&raw mut g_ram as *mut uint8)
            .offset(0xbdc as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        *fresh43 = (*fresh43 as ::core::ffi::c_int ^ 0x8000 as ::core::ffi::c_int) as uint16;
        if t_0 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
            AddToHiLo(
                ((&raw mut g_ram as *mut uint8).offset(0xb64 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) as *mut uint16,
                ((&raw mut g_ram as *mut uint8).offset(0xb8c as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) as *mut uint16,
                -((t_0 as int16 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint32,
            );
        } else {
            AddToHiLo(
                ((&raw mut g_ram as *mut uint8).offset(0xb64 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) as *mut uint16,
                ((&raw mut g_ram as *mut uint8).offset(0xb8c as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) as *mut uint16,
                (((t_0 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int) as int16
                    as ::core::ffi::c_int)
                    << 8 as ::core::ffi::c_int) as uint32,
            );
        }
    }
}
pub unsafe fn ProjPreInstr_WaveSba(mut k: uint16) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v2: bool = false;
    let mut v3: bool = false;
    if *((&raw mut g_ram as *mut uint8).offset(0xc04 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        & 0xf0 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
        || {
            v2 = *((&raw mut g_ram as *mut uint8).offset(0xbf0 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                == 1 as ::core::ffi::c_int;
            v3 = ((*((&raw mut g_ram as *mut uint8).offset(0xbf0 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int;
            let ref mut fresh34 = *((&raw mut g_ram as *mut uint8)
                .offset(0xbf0 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize);
            *fresh34 = (*fresh34).wrapping_sub(1);
            v2 as ::core::ffi::c_int != 0
        }
        || v3 as ::core::ffi::c_int != 0
    {
        QueueSfx1_Max6(0x29 as uint16);
        ClearProjectile(k);
        return;
    }
    let mut R34: uint16 = *((&raw mut g_ram as *mut uint8)
        .offset(0xbdc as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    let ref mut fresh37 = *((&raw mut g_ram as *mut uint8)
        .offset(0xc90 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    let fresh38 = *fresh37;
    *fresh37 = (*fresh37).wrapping_sub(1);
    v2 = fresh38 as ::core::ffi::c_int == 1 as ::core::ffi::c_int;
    if v2 {
        *((&raw mut g_ram as *mut uint8).offset(0xc90 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 4 as uint16;
        SpawnProjectileTrail(k);
        k = *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
            as *mut uint16);
    }
    if ((*((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0xb64 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        if *((&raw mut g_ram as *mut uint8).offset(0xbdc as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            + 2047 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            let ref mut fresh39 = *((&raw mut g_ram as *mut uint8)
                .offset(0xbdc as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize);
            *fresh39 = (*fresh39 as ::core::ffi::c_int - 64 as ::core::ffi::c_int) as uint16;
        }
    } else if *((&raw mut g_ram as *mut uint8).offset(0xbdc as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        - 2048 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let ref mut fresh40 = *((&raw mut g_ram as *mut uint8)
            .offset(0xbdc as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        *fresh40 = (*fresh40 as ::core::ffi::c_int + 64 as ::core::ffi::c_int) as uint16;
    }
    AddToHiLo(
        ((&raw mut g_ram as *mut uint8).offset(0xb64 as ::core::ffi::c_int as isize) as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((&raw mut g_ram as *mut uint8).offset(0xb8c as ::core::ffi::c_int as isize) as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((*((&raw mut g_ram as *mut uint8).offset(0xbdc as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as int16 as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint32,
    );
    if ((*((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        if *((&raw mut g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            + 2047 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            let ref mut fresh41 = *((&raw mut g_ram as *mut uint8)
                .offset(0xc7c as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize);
            *fresh41 = (*fresh41 as ::core::ffi::c_int - 64 as ::core::ffi::c_int) as uint16;
        }
    } else if *((&raw mut g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        - 2048 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let ref mut fresh42 = *((&raw mut g_ram as *mut uint8)
            .offset(0xc7c as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        *fresh42 = (*fresh42 as ::core::ffi::c_int + 64 as ::core::ffi::c_int) as uint16;
    }
    AddToHiLo(
        ((&raw mut g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize) as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((&raw mut g_ram as *mut uint8).offset(0xba0 as ::core::ffi::c_int as isize) as *mut uint16)
            .offset(v1 as isize) as *mut uint16,
        ((*((&raw mut g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as int16 as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint32,
    );
    if k as ::core::ffi::c_int == 6 as ::core::ffi::c_int {
        if *((&raw mut g_ram as *mut uint8).offset(0xbdc as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            if R34 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                QueueSfx1_Max6(0x28 as uint16);
            }
        } else if R34 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            QueueSfx1_Max6(0x28 as uint16);
        }
    }
    *((&raw mut g_ram as *mut uint8).offset(0xccc as ::core::ffi::c_int as isize) as *mut uint16) =
        2 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xcd0 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
}
pub unsafe fn BombSpread() {
    if *((&raw mut g_ram as *mut uint8).offset(0xc72 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        != 0x90d8f7 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        let mut v0: uint16 = 10 as uint16;
        loop {
            let mut v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            *((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = -(31488 as ::core::ffi::c_int) as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xc04 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = 0 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xc68 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) =
                (0x90d8f7 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            InitializeInstrForMissile(v0);
            *((&raw mut g_ram as *mut uint8).offset(0xb64 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = *((&raw mut g_ram as *mut uint8)
                .offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16);
            *((&raw mut g_ram as *mut uint8).offset(0xb8c as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = 0 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = *((&raw mut g_ram as *mut uint8)
                .offset(0xafa as ::core::ffi::c_int as isize)
                as *mut uint16);
            *((&raw mut g_ram as *mut uint8).offset(0xba0 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = 0 as uint16;
            let mut v2: ::core::ffi::c_int = (v0 as ::core::ffi::c_int - 10 as ::core::ffi::c_int)
                as uint16 as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int;
            *((&raw mut g_ram as *mut uint8).offset(0xbdc as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = kBombSpread_Tab1[v2 as usize];
            *((&raw mut g_ram as *mut uint8).offset(0xc90 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = kBombSpread_Tab3[v2 as usize];
            let mut v3: uint16 = -((kBombSpread_Tab2[v2 as usize] as ::core::ffi::c_int
                + (*((&raw mut g_ram as *mut uint8).offset(0xcd4 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    >> 6 as ::core::ffi::c_int
                    & 3 as ::core::ffi::c_int)) as int16
                as ::core::ffi::c_int) as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xbf0 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = v3;
            *((&raw mut g_ram as *mut uint8).offset(0xca4 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = v3;
            *((&raw mut g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = kBombSpread_Tab0[v2 as usize];
            v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
            if !(((v0 as ::core::ffi::c_int - 20 as ::core::ffi::c_int) as int16
                as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int)
            {
                break;
            }
        }
        *((&raw mut g_ram as *mut uint8).offset(0xcd2 as ::core::ffi::c_int as isize)
            as *mut uint16) = 5 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xccc as ::core::ffi::c_int as isize)
            as *mut uint16) =
            kNonBeamProjectileCooldowns[(*(((&raw mut g_ram as *mut uint8)
                .offset(0xc18 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(5 as ::core::ffi::c_int as isize)
                as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ) as ::core::ffi::c_int
                & 0xf as ::core::ffi::c_int) as usize] as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xcd4 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xcd0 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    }
}
static mut kProjPreInstr_SpazerSba_FuncsB: [Option<Func_SpazerSba_V>; 3] = unsafe {
    [
        Some(ProjPreInstr_SpazerSba_FuncB_0 as unsafe fn(uint16, uint16) -> ()),
        Some(ProjPreInstr_SpazerSba_FuncB_1 as unsafe fn(uint16, uint16) -> ()),
        Some(ProjPreInstr_SpazerSba_FuncB_2 as unsafe fn(uint16, uint16) -> ()),
    ]
};
static mut kProjPreInstr_SpazerSba_FuncsA: [Option<Func_Y_V>; 4] = unsafe {
    [
        Some(ProjPreInstr_SpazerSba_FuncA_0 as unsafe fn(uint16) -> ()),
        Some(ProjPreInstr_SpazerSba_FuncA_1 as unsafe fn(uint16) -> ()),
        Some(ProjPreInstr_SpazerSba_FuncA_2 as unsafe fn(uint16) -> ()),
        Some(ProjPreInstr_SpazerSba_FuncA_3 as unsafe fn(uint16) -> ()),
    ]
};
pub unsafe fn ProjPreInstr_SpazerSba(mut k: uint16) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut g_ram as *mut uint8).offset(0xc04 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        & 0xf0 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        kProjPreInstr_SpazerSba_FuncsA[v1 as usize].expect("non-null function pointer")(k);
    } else {
        let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let ref mut fresh32 = *((&raw mut g_ram as *mut uint8)
            .offset(0xc90 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize);
        let fresh33 = *fresh32;
        *fresh32 = (*fresh32).wrapping_sub(1);
        if fresh33 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
            *((&raw mut g_ram as *mut uint8).offset(0xc90 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v2 as isize) = 4 as uint16;
            SpawnProjectileTrail(k);
            k = *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
                as *mut uint16);
        }
        let mut v4: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut pt: Point16U = Projectile_SinLookup(
            *((&raw mut g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v4 as isize),
            *((&raw mut g_ram as *mut uint8).offset(0xbdc as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v4 as isize),
        );
        *((&raw mut g_ram as *mut uint8).offset(0xb64 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v4 as isize) = (pt.x as ::core::ffi::c_int
            + *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16;
        kProjPreInstr_SpazerSba_FuncsB[(*((&raw mut g_ram as *mut uint8)
            .offset(0xca4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v4 as isize) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int) as usize]
            .expect("non-null function pointer")(k, pt.y);
        *((&raw mut g_ram as *mut uint8).offset(0xccc as ::core::ffi::c_int as isize)
            as *mut uint16) = 2 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xcd0 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    };
}
pub unsafe fn ProjPreInstr_SpazerSba_FuncA_0(mut k: uint16) {
    ClearProjectile(k);
    ClearProjectile(4 as uint16);
}
pub unsafe fn ProjPreInstr_SpazerSba_FuncA_1(mut k: uint16) {
    ClearProjectile(k);
    ClearProjectile(6 as uint16);
}
pub unsafe fn ProjPreInstr_SpazerSba_FuncA_2(mut k: uint16) {
    ClearProjectile(k);
    ClearProjectile(0 as uint16);
}
pub unsafe fn ProjPreInstr_SpazerSba_FuncA_3(mut k: uint16) {
    ClearProjectile(k);
    ClearProjectile(2 as uint16);
}
pub unsafe fn ProjPreInstr_SpazerSba_FuncB_0(mut j: uint16, mut r22: uint16) {
    static mut kProjPreInstr_SpazerSba_Yspeed: [int16; 4] = [
        2 as ::core::ffi::c_int as int16,
        -(2 as ::core::ffi::c_int) as int16,
        2 as ::core::ffi::c_int as int16,
        -(2 as ::core::ffi::c_int) as int16,
    ];
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) = (r22 as ::core::ffi::c_int
        + *((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as uint16;
    let mut v2: uint16 = (*(((&raw mut g_ram as *mut uint8)
        .offset(0xbf0 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as *mut uint16 as *mut uint8)
        .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        + *(((&raw mut g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as *mut uint16 as *mut uint8)
            .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
        as uint8 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) = v2;
    if v2 as ::core::ffi::c_int == 128 as ::core::ffi::c_int {
        *((&raw mut g_ram as *mut uint8).offset(0xbdc as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 160 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xbf0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = kProjPreInstr_SpazerSba_Yspeed[v1 as usize] as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xc04 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xca4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = 2 as uint16;
    }
}
pub unsafe fn ProjPreInstr_SpazerSba_FuncB_1(mut j: uint16, mut r22: uint16) {
    static mut kSpazerProjectileYSpeed: [int16; 4] = [
        -(2 as ::core::ffi::c_int) as int16,
        2 as ::core::ffi::c_int as int16,
        -(2 as ::core::ffi::c_int) as int16,
        2 as ::core::ffi::c_int as int16,
    ];
    let mut v1: uint16 = (r22 as ::core::ffi::c_int
        + *((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        - 114 as ::core::ffi::c_int) as uint16;
    let mut v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v2 as isize) = v1;
    if v1 as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        - 16 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        FireEndOfSpazerSba(j);
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) =
            (*(((&raw mut g_ram as *mut uint8).offset(0xbf0 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v2 as isize) as *mut uint16 as *mut uint8)
                .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                + *(((&raw mut g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v2 as isize) as *mut uint16 as *mut uint8)
                    .offset(0 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int) as uint8 as uint16;
        let mut v3: uint16 = (*((&raw mut g_ram as *mut uint8)
            .offset(0xbdc as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) as ::core::ffi::c_int
            - 5 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xbdc as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v2 as isize) = v3;
        if v3 == 0 {
            *((&raw mut g_ram as *mut uint8).offset(0xbf0 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v2 as isize) = kSpazerProjectileYSpeed[v2 as usize] as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v2 as isize) =
                (*((&raw mut g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v2 as isize) as ::core::ffi::c_int
                    + 0x80 as ::core::ffi::c_int) as uint8 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xca4 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v2 as isize) = 4 as uint16;
            if j == 0 {
                QueueSfx1_Max6(0x26 as uint16);
            }
        }
    };
}
pub unsafe fn ProjPreInstr_SpazerSba_FuncB_2(mut j: uint16, mut r22: uint16) {
    let mut v2: ::core::ffi::c_int = 0;
    let mut v3: uint16 = 0;
    let mut v1: uint16 = (r22 as ::core::ffi::c_int
        + *((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        - 114 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = v1;
    if v1 as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        - 16 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
        || {
            v2 = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            *((&raw mut g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v2 as isize) =
                (*(((&raw mut g_ram as *mut uint8).offset(0xbf0 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v2 as isize) as *mut uint16 as *mut uint8)
                    .offset(0 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    + *(((&raw mut g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize)
                        as *mut uint16)
                        .offset(v2 as isize) as *mut uint16 as *mut uint8)
                        .offset(0 as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int) as uint8 as uint16;
            v3 = (*((&raw mut g_ram as *mut uint8).offset(0xbdc as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v2 as isize) as ::core::ffi::c_int
                + 5 as ::core::ffi::c_int) as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xbdc as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v2 as isize) = v3;
            v3 as ::core::ffi::c_int - 96 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0
        }
    {
        FireEndOfSpazerSba(j);
    }
}
pub unsafe fn FireEndOfSpazerSba(mut j: uint16) {
    static mut kFireEndOfSpazerSba: [int16; 4] = [
        16 as ::core::ffi::c_int as int16,
        16 as ::core::ffi::c_int as int16,
        -(16 as ::core::ffi::c_int) as int16,
        -(16 as ::core::ffi::c_int) as int16,
    ];
    let mut v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh22 = *((&raw mut g_ram as *mut uint8)
        .offset(0xb64 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    *fresh22 = (*fresh22 as ::core::ffi::c_int
        + kFireEndOfSpazerSba[v1 as usize] as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xc04 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) = 5 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xc90 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) = 4 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xc68 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) =
        (0x90dc9c as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    if ((j as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            -(32732 as ::core::ffi::c_int) as uint16;
        InitializeShinesparkEchoOrSpazerSba(j);
    }
}
pub unsafe fn ProjPreInstr_EndOfSpazerSba(mut k: uint16) {
    let mut v3: ::core::ffi::c_int = 0;
    let mut v4: uint16 = 0;
    let mut current_block: u64;
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut g_ram as *mut uint8).offset(0xc04 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        & 0xf0 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        current_block = 14656271494628884762;
    } else {
        let ref mut fresh30 = *((&raw mut g_ram as *mut uint8)
            .offset(0xc90 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        let fresh31 = *fresh30;
        *fresh30 = (*fresh30).wrapping_sub(1);
        if fresh31 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
            *((&raw mut g_ram as *mut uint8).offset(0xc90 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = 4 as uint16;
            SpawnProjectileTrail(k);
            k = *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
                as *mut uint16);
        }
        v3 = 0;
        v3 = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        v4 = 0;
        v4 = (*((&raw mut g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v3 as isize) as ::core::ffi::c_int
            + 8 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v3 as isize) = v4;
        if v4 as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            - 248 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            current_block = 14656271494628884762;
        } else {
            *((&raw mut g_ram as *mut uint8).offset(0xccc as ::core::ffi::c_int as isize)
                as *mut uint16) = 2 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xcd0 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            current_block = 7651349459974463963;
        }
    }
    match current_block {
        14656271494628884762 => {
            ClearProjectile(k);
        }
        _ => {}
    };
}
static mut kHandleHudSpecificBehaviorAndProjs: [Option<Func_V>; 28] = unsafe {
    [
        Some(HudSelectionHandler_Normal as unsafe fn() -> ()),
        Some(HudSelectionHandler_Normal as unsafe fn() -> ()),
        Some(HudSelectionHandler_Normal as unsafe fn() -> ()),
        Some(HudSelectionHandler_JumpEtc as unsafe fn() -> ()),
        Some(HudSelectionHandler_MorphBall as unsafe fn() -> ()),
        Some(HudSelectionHandler_Normal as unsafe fn() -> ()),
        Some(HudSelectionHandler_Normal as unsafe fn() -> ()),
        Some(HudSelectionHandler_MorphBall as unsafe fn() -> ()),
        Some(HudSelectionHandler_MorphBall as unsafe fn() -> ()),
        Some(HudSelectionHandler_MorphBall as unsafe fn() -> ()),
        Some(HudSelectionHandler_JumpEtc as unsafe fn() -> ()),
        Some(HudSelectionHandler_Grappling as unsafe fn() -> ()),
        Some(HudSelectionHandler_Grappling as unsafe fn() -> ()),
        Some(HudSelectionHandler_JumpEtc as unsafe fn() -> ()),
        Some(HudSelectionHandler_TurningAround as unsafe fn() -> ()),
        Some(HudSelectionHandler_CrouchEtcTrans as unsafe fn() -> ()),
        Some(HudSelectionHandler_Normal as unsafe fn() -> ()),
        Some(HudSelectionHandler_MorphBall as unsafe fn() -> ()),
        Some(HudSelectionHandler_MorphBall as unsafe fn() -> ()),
        Some(HudSelectionHandler_MorphBall as unsafe fn() -> ()),
        Some(HudSelectionHandler_JumpEtc as unsafe fn() -> ()),
        Some(HudSelectionHandler_Normal as unsafe fn() -> ()),
        Some(HudSelectionHandler_Grappling as unsafe fn() -> ()),
        Some(HudSelectionHandler_TurningAround as unsafe fn() -> ()),
        Some(HudSelectionHandler_TurningAround as unsafe fn() -> ()),
        Some(HudSelectionHandler_JumpEtc as unsafe fn() -> ()),
        Some(HudSelectionHandler_GrabbedByDraygon as unsafe fn() -> ()),
        Some(HudSelectionHandler_JumpEtc as unsafe fn() -> ()),
    ]
};
pub unsafe fn Samus_HandleHudSpecificBehaviorAndProjs() {
    if *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        == kPose_00_FaceF_Powersuit as ::core::ffi::c_int
        || *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == kPose_9B_FaceF_VariaGravitySuit as ::core::ffi::c_int
        || {
            Samus_HandleCooldown();
            HandleSwitchingHudSelection();
            kHandleHudSpecificBehaviorAndProjs[*(&raw mut g_ram as *mut uint8)
                .offset(0xa1f as ::core::ffi::c_int as isize)
                as usize]
                .expect("non-null function pointer")();
            *((&raw mut g_ram as *mut uint8).offset(0xa78 as ::core::ffi::c_int as isize)
                as *mut uint16)
                == 0
        }
    {
        HandleProjectile();
    }
}
pub unsafe fn HudSelectionHandler_Normal() {
    static mut kHudSelectionHandler_Normal: [Option<Func_V>; 7] = unsafe {
        [
            Some(HudSelectionHandler_NothingOrPowerBombs as unsafe fn() -> ()),
            Some(HudSelectionHandler_MissilesOrSuperMissiles as unsafe fn() -> ()),
            Some(HudSelectionHandler_MissilesOrSuperMissiles as unsafe fn() -> ()),
            Some(HudSelectionHandler_NothingOrPowerBombs as unsafe fn() -> ()),
            Some(HudSelectionHandler_Grappling as unsafe fn() -> ()),
            Some(HudSelectionHandler_Xray as unsafe fn() -> ()),
            Some(HudSelectionHandler_TurningAround as unsafe fn() -> ()),
        ]
    };
    let mut v0: uint16 = 0;
    if *((&raw mut g_ram as *mut uint8).offset(0xd32 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        == 0x9bc4f0 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        if *((&raw mut g_ram as *mut uint8).offset(0xa78 as ::core::ffi::c_int as isize)
            as *mut uint16)
            != 0
        {
            v0 = 10 as uint16;
        } else {
            v0 = (2 as ::core::ffi::c_int
                * *((&raw mut g_ram as *mut uint8).offset(0x9d2 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint16;
        }
    } else {
        v0 = 8 as uint16;
    }
    kHudSelectionHandler_Normal[(v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize]
        .expect("non-null function pointer")();
}
pub unsafe fn HudSelectionHandler_Grappling() {
    GrappleBeamHandler();
}
pub unsafe fn HudSelectionHandler_TurningAround() {
    if *((&raw mut g_ram as *mut uint8).offset(0xb5e as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        HudSelectionHandler_Normal();
    } else if *((&raw mut g_ram as *mut uint8).offset(0xd32 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        != 0x9bc4f0 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0xd32 as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x9bc856 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn HudSelectionHandler_CrouchEtcTrans() {
    let mut current_block: u64;
    static mut byte_90DDAA: [uint8; 12] = [
        0 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        0 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
    ];
    if *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - kPose_F1_FaceR_CrouchTrans_AimU as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        current_block = 2697389224984845139;
    } else {
        if *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - kPose_DB as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            return;
        }
        if byte_90DDAA[(*((&raw mut g_ram as *mut uint8)
            .offset(0xa1c as ::core::ffi::c_int as isize) as *mut uint16)
            as ::core::ffi::c_int
            - 53 as ::core::ffi::c_int) as uint16 as usize]
            != 0
        {
            if *((&raw mut g_ram as *mut uint8).offset(0xd32 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                != 0x9bc4f0 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
            {
                *((&raw mut g_ram as *mut uint8).offset(0xd32 as ::core::ffi::c_int as isize)
                    as *mut uint16) =
                    (0x9bc856 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
                HudSelectionHandler_Normal();
            }
            current_block = 5720623009719927633;
        } else {
            current_block = 2697389224984845139;
        }
    }
    match current_block {
        2697389224984845139 => {
            HudSelectionHandler_Normal();
        }
        _ => {}
    };
}
pub unsafe fn HudSelectionHandler_JumpEtc() {
    if *((&raw mut g_ram as *mut uint8).offset(0xd32 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        != 0x9bc4f0 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0xd32 as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x9bc856 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        HudSelectionHandler_Normal();
    }
}
pub unsafe fn HudSelectionHandler_Xray() {
    if *((&raw mut g_ram as *mut uint8).offset(0x9b6 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & *((&raw mut g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        XrayRunHandler();
    } else {
        HudSelectionHandler_NothingOrPowerBombs();
    };
}
pub unsafe fn HudSelectionHandler_GrabbedByDraygon() {
    if *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        == kPose_DB as ::core::ffi::c_int | kPose_04_FaceL_AimU as ::core::ffi::c_int
    {
        HudSelectionHandler_MorphBall();
    } else {
        HudSelectionHandler_Normal();
    };
}
static mut kSamusHitInterrupts: [Option<Func_U8>; 28] = unsafe {
    [
        Some(Samus_HitInterrupt_Stand as unsafe fn() -> uint8),
        Some(Samus_HitInterrupt_Stand as unsafe fn() -> uint8),
        Some(Samus_HitInterrupt_Stand as unsafe fn() -> uint8),
        Some(Samus_HitInterrupt_Stand as unsafe fn() -> uint8),
        Some(Samus_HitInterrupt_Ball as unsafe fn() -> uint8),
        Some(Samus_HitInterrupt_Stand as unsafe fn() -> uint8),
        Some(Samus_HitInterrupt_Falling as unsafe fn() -> uint8),
        Some(Samus_HitInterrupt_Unused as unsafe fn() -> uint8),
        Some(Samus_HitInterrupt_Ball as unsafe fn() -> uint8),
        Some(Samus_HitInterrupt_Ball as unsafe fn() -> uint8),
        Some(Samus_HitInterrupt_KnockbackGrapple as unsafe fn() -> uint8),
        Some(Samus_HitInterrupt_Turning as unsafe fn() -> uint8),
        Some(Samus_HitInterrupt_Turning as unsafe fn() -> uint8),
        Some(Samus_HitInterrupt_Stand as unsafe fn() -> uint8),
        Some(Samus_HitInterrupt_Turning as unsafe fn() -> uint8),
        Some(Samus_HitInterrupt_Turning as unsafe fn() -> uint8),
        Some(Samus_HitInterrupt_Stand as unsafe fn() -> uint8),
        Some(Samus_HitInterrupt_Ball as unsafe fn() -> uint8),
        Some(Samus_HitInterrupt_Ball as unsafe fn() -> uint8),
        Some(Samus_HitInterrupt_Ball as unsafe fn() -> uint8),
        Some(Samus_HitInterrupt_Stand as unsafe fn() -> uint8),
        Some(Samus_HitInterrupt_Stand as unsafe fn() -> uint8),
        Some(Samus_HitInterrupt_KnockbackGrapple as unsafe fn() -> uint8),
        Some(Samus_HitInterrupt_Turning as unsafe fn() -> uint8),
        Some(Samus_HitInterrupt_Turning as unsafe fn() -> uint8),
        Some(Samus_HitInterrupt_Turning as unsafe fn() -> uint8),
        Some(Samus_HitInterrupt_Turning as unsafe fn() -> uint8),
        Some(Samus_HitInterrupt_Shinespark as unsafe fn() -> uint8),
    ]
};
pub unsafe fn Samus_HitInterruption() {
    if *((&raw mut g_ram as *mut uint8).offset(0x18aa as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0
    {
        if *((&raw mut g_ram as *mut uint8).offset(0xde0 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 7 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            if *((&raw mut g_ram as *mut uint8).offset(0xa78 as ::core::ffi::c_int as isize)
                as *mut uint16)
                == 0
                && *((&raw mut g_ram as *mut uint8).offset(0xa52 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    == 0
            {
                if kSamusHitInterrupts[*(&raw mut g_ram as *mut uint8)
                    .offset(0xa1f as ::core::ffi::c_int as isize)
                    as usize]
                    .expect("non-null function pointer")() as ::core::ffi::c_int
                    & 1 as ::core::ffi::c_int
                    != 0
                {
                    *((&raw mut g_ram as *mut uint8).offset(0xa30 as ::core::ffi::c_int as isize)
                        as *mut uint16) = 1 as uint16;
                }
            }
        } else {
            *((&raw mut g_ram as *mut uint8).offset(0x18a8 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0x18aa as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
        }
    } else if *((&raw mut g_ram as *mut uint8).offset(0xa52 as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0
    {
        if *(&raw mut g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            == kPose_0A_MoveL_NoAim as ::core::ffi::c_int
        {
            if *((&raw mut g_ram as *mut uint8).offset(0xcd0 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - 16 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
            {
                QueueSfx1_Max6(0x41 as uint16);
            }
            if *(&raw mut g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
                == 4 as ::core::ffi::c_int
            {
                *((&raw mut g_ram as *mut uint8).offset(0xa2c as ::core::ffi::c_int as isize)
                    as *mut uint16) = kPose_2A_FaceL_Fall as ::core::ffi::c_int as uint16;
            } else {
                *((&raw mut g_ram as *mut uint8).offset(0xa2c as ::core::ffi::c_int as isize)
                    as *mut uint16) = kPose_29_FaceR_Fall as ::core::ffi::c_int as uint16;
            }
        } else {
            if *((&raw mut g_ram as *mut uint8).offset(0xa32 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                == 3 as ::core::ffi::c_int
            {
                *((&raw mut g_ram as *mut uint8).offset(0xa32 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 8 as uint16;
                return;
            }
            *((&raw mut g_ram as *mut uint8).offset(0xa2c as ::core::ffi::c_int as isize)
                as *mut uint16) = *((&raw mut g_ram as *mut uint8)
                .offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16);
        }
        *((&raw mut g_ram as *mut uint8).offset(0xa32 as ::core::ffi::c_int as isize)
            as *mut uint16) = 1 as uint16;
    } else if *((&raw mut g_ram as *mut uint8).offset(0xa56 as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0
    {
        SetupBombJump();
    }
}
pub unsafe fn Samus_HitInterrupt_Shinespark() -> uint8 {
    if *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        == kPose_E8_FaceR_Drained_CrouchFalling as ::core::ffi::c_int
        || *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == kPose_E9_FaceL_Drained_CrouchFalling as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0xa94 as ::core::ffi::c_int as isize)
            as *mut uint16) = 17 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
            as *mut uint16) = 26 as uint16;
    }
    *((&raw mut g_ram as *mut uint8).offset(0xa30 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa52 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    return 0 as uint8;
}
pub unsafe fn Samus_HitInterrupt_KnockbackGrapple() -> uint8 {
    *((&raw mut g_ram as *mut uint8).offset(0xa30 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    return 0 as uint8;
}
pub unsafe fn Samus_HitInterrupt_Turning() -> uint8 {
    *((&raw mut g_ram as *mut uint8).offset(0xa30 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa52 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    return 0 as uint8;
}
pub unsafe fn Samus_HitInterrupt_Falling() -> uint8 {
    if *((&raw mut g_ram as *mut uint8).offset(0xa5a as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        != 0x90e41b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        return Samus_HitInterrupt_Stand();
    }
    *((&raw mut g_ram as *mut uint8).offset(0xa30 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa52 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    return 0 as uint8;
}
pub unsafe fn Samus_HitInterrupt_Stand() -> uint8 {
    if *(&raw mut g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == 4 as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0xa2a as ::core::ffi::c_int as isize)
            as *mut uint16) = 84 as uint16;
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0xa2a as ::core::ffi::c_int as isize)
            as *mut uint16) = 83 as uint16;
    }
    return 1 as uint8;
}
pub unsafe fn Samus_HitInterrupt_Ball() -> uint8 {
    *((&raw mut g_ram as *mut uint8).offset(0xa2a as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16);
    return 1 as uint8;
}
pub unsafe fn Samus_HitInterrupt_Unused() -> uint8 {
    if *(&raw mut g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == 4 as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0xa2a as ::core::ffi::c_int as isize)
            as *mut uint16) = 52 as uint16;
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0xa2a as ::core::ffi::c_int as isize)
            as *mut uint16) = 51 as uint16;
    }
    return 1 as uint8;
}
static mut kSamus_MoveHandler_Knockback: [Option<Func_V>; 6] = unsafe {
    [
        Some(Samus_MoveHandler_Knockback_0 as unsafe fn() -> ()),
        Some(Samus_MoveHandler_Knockback_Up as unsafe fn() -> ()),
        Some(Samus_MoveHandler_Knockback_Up as unsafe fn() -> ()),
        Some(Samus_MoveHandler_Knockback_3 as unsafe fn() -> ()),
        Some(Samus_MoveHandler_Knockback_Down as unsafe fn() -> ()),
        Some(Samus_MoveHandler_Knockback_Down as unsafe fn() -> ()),
    ]
};
pub unsafe fn Samus_MoveHandler_Knockback() {
    kSamus_MoveHandler_Knockback[*((&raw mut g_ram as *mut uint8)
        .offset(0xa52 as ::core::ffi::c_int as isize)
        as *mut uint16) as usize]
        .expect("non-null function pointer")();
    *((&raw mut g_ram as *mut uint8).offset(0xdc6 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
}
pub unsafe fn Samus_MoveHandler_Knockback_0() {
    InvalidInterrupt_Crash();
}
pub unsafe fn Samus_MoveHandler_Knockback_Up() {
    Samus_BombJumpRisingXMovement_();
    Samus_MoveY_WithSpeedCalc();
    Samus_ClearMoveVars();
}
pub unsafe fn Samus_MoveHandler_Knockback_3() {
    Samus_MoveY_WithSpeedCalc();
    Samus_ClearMoveVars();
}
pub unsafe fn Samus_MoveHandler_Knockback_Down() {
    Samus_BombJumpRisingXMovement_();
    Samus_Move_NoSpeedCalc_Y();
    Samus_ClearMoveVars();
}
pub unsafe fn Samus_ClearMoveVars() {
    if *((&raw mut g_ram as *mut uint8).offset(0xdd0 as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        *((&raw mut g_ram as *mut uint8).offset(0xb4a as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xdce as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb22 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb1a as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb2a as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb36 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb38 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb46 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb48 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        Samus_AlignBottomWithPrevPose();
    }
}
static mut kSetupBombJump: [Option<Func_U8>; 28] = unsafe {
    [
        Some(SetupBombJump_StandCrouch as unsafe fn() -> uint8),
        Some(SetupBombJump_1 as unsafe fn() -> uint8),
        Some(SetupBombJump_2 as unsafe fn() -> uint8),
        Some(SetupBombJump_2 as unsafe fn() -> uint8),
        Some(SetupBombJump_4 as unsafe fn() -> uint8),
        Some(SetupBombJump_StandCrouch as unsafe fn() -> uint8),
        Some(SetupBombJump_1 as unsafe fn() -> uint8),
        Some(SetupBombJump_4 as unsafe fn() -> uint8),
        Some(SetupBombJump_4 as unsafe fn() -> uint8),
        Some(SetupBombJump_4 as unsafe fn() -> uint8),
        Some(SetupBombJump_4 as unsafe fn() -> uint8),
        Some(SetupBombJump_1 as unsafe fn() -> uint8),
        Some(SetupBombJump_1 as unsafe fn() -> uint8),
        Some(SetupBombJump_1 as unsafe fn() -> uint8),
        Some(SetupBombJump_2 as unsafe fn() -> uint8),
        Some(SetupBombJump_2 as unsafe fn() -> uint8),
        Some(SetupBombJump_1 as unsafe fn() -> uint8),
        Some(SetupBombJump_4 as unsafe fn() -> uint8),
        Some(SetupBombJump_4 as unsafe fn() -> uint8),
        Some(SetupBombJump_4 as unsafe fn() -> uint8),
        Some(SetupBombJump_1 as unsafe fn() -> uint8),
        Some(SetupBombJump_1 as unsafe fn() -> uint8),
        Some(SetupBombJump_1 as unsafe fn() -> uint8),
        Some(SetupBombJump_2 as unsafe fn() -> uint8),
        Some(SetupBombJump_2 as unsafe fn() -> uint8),
        Some(SetupBombJump_2 as unsafe fn() -> uint8),
        Some(SetupBombJump_1A as unsafe fn() -> uint8),
        Some(SetupBombJump_1A as unsafe fn() -> uint8),
    ]
};
pub unsafe fn SetupBombJump() {
    if *((&raw mut g_ram as *mut uint8).offset(0xa56 as ::core::ffi::c_int as isize) as *mut uint16
        as *mut uint8)
        .offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        )
        == 0
    {
        if kSetupBombJump
            [*(&raw mut g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize) as usize]
            .expect("non-null function pointer")() as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            != 0
        {
            *((&raw mut g_ram as *mut uint8).offset(0xa30 as ::core::ffi::c_int as isize)
                as *mut uint16) = 3 as uint16;
        }
    }
}
pub unsafe fn SetupBombJump_StandCrouch() -> uint8 {
    if *((&raw mut g_ram as *mut uint8).offset(0xa78 as ::core::ffi::c_int as isize) as *mut uint16)
        == 0
    {
        return SetupBombJump_1();
    }
    *((&raw mut g_ram as *mut uint8).offset(0xa56 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    return 0 as uint8;
}
pub unsafe fn SetupBombJump_1() -> uint8 {
    if *(&raw mut g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == 4 as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0xa2a as ::core::ffi::c_int as isize)
            as *mut uint16) = kPose_52_FaceL_Jump_NoAim_MoveF as ::core::ffi::c_int as uint16;
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0xa2a as ::core::ffi::c_int as isize)
            as *mut uint16) = kPose_51_FaceR_Jump_NoAim_MoveF as ::core::ffi::c_int as uint16;
    }
    return 1 as uint8;
}
pub unsafe fn SetupBombJump_4() -> uint8 {
    *((&raw mut g_ram as *mut uint8).offset(0xa2a as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16);
    return 1 as uint8;
}
pub unsafe fn SetupBombJump_2() -> uint8 {
    *((&raw mut g_ram as *mut uint8).offset(0xa60 as ::core::ffi::c_int as isize) as *mut uint16) =
        (0x90e913 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    return SetupBombJump_1A();
}
pub unsafe fn SetupBombJump_1A() -> uint8 {
    *((&raw mut g_ram as *mut uint8).offset(0xa56 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    return 0 as uint8;
}
pub unsafe fn Samus_MoveHandler_BombJumpStart() {
    Samus_InitBombJump();
    *((&raw mut g_ram as *mut uint8).offset(0xa58 as ::core::ffi::c_int as isize) as *mut uint16) =
        (0x90e032 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xdc6 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
}
static mut kSamus_MoveHandler_BombJumpMain: [Option<Func_V>; 4] = unsafe {
    [
        Some(Samus_MoveHandler_Knockback_0 as unsafe fn() -> ()),
        Some(Samus_HorizontalBombJump as unsafe fn() -> ()),
        Some(Samus_VerticalBombJump as unsafe fn() -> ()),
        Some(Samus_HorizontalBombJump as unsafe fn() -> ()),
    ]
};
pub unsafe fn Samus_MoveHandler_BombJumpMain() {
    if *((&raw mut g_ram as *mut uint8).offset(0xa56 as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        kSamus_MoveHandler_BombJumpMain[*((&raw mut g_ram as *mut uint8)
            .offset(0xa56 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint8 as usize]
            .expect("non-null function pointer")();
    } else {
        Samus_MoveHandler_BombJumpFunc1();
    };
}
pub unsafe fn Samus_HorizontalBombJump() {
    Samus_BombJumpRisingXMovement_();
    Samus_BombJumpRisingYMovement_();
    if *((&raw mut g_ram as *mut uint8).offset(0xb36 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        == 2 as ::core::ffi::c_int
        || {
            Samus_MoveY_WithSpeedCalc();
            *((&raw mut g_ram as *mut uint8).offset(0xdd0 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                != 0
        }
    {
        Samus_MoveHandler_BombJumpFunc1();
    }
}
pub unsafe fn Samus_VerticalBombJump() {
    Samus_BombJumpRisingYMovement_();
    if *((&raw mut g_ram as *mut uint8).offset(0xb36 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        == 2 as ::core::ffi::c_int
        || {
            Samus_MoveY_WithSpeedCalc();
            *((&raw mut g_ram as *mut uint8).offset(0xdd0 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                != 0
        }
    {
        Samus_MoveHandler_BombJumpFunc1();
    }
}
pub unsafe fn Samus_MoveHandler_BombJumpFunc1() {
    *((&raw mut g_ram as *mut uint8).offset(0xa58 as ::core::ffi::c_int as isize) as *mut uint16) =
        (0x90a337 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    if *((&raw mut g_ram as *mut uint8).offset(0xa60 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        != 0x90e91d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0xa60 as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x90e913 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    *((&raw mut g_ram as *mut uint8).offset(0xa56 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
}
pub unsafe fn CallFrameHandlerGamma(mut ea: uint32) {
    match ea {
        9494683 => {
            Samus_Func1();
            return;
        }
        9494725 => {
            Samus_Func2();
            return;
        }
        9494758 => {
            Samus_Func3();
            return;
        }
        9494804 => {
            DrawTimer_();
            return;
        }
        9494830 => {
            Samus_PushOutOfRidleysWay();
            return;
        }
        9494984 => {
            Samus_Func4();
            return;
        }
        9495201 => {
            Samus_GrabbedByDraygonFrameHandler();
            return;
        }
        9495422 => return,
        9495459 => {
            Samus_Func7();
            return;
        }
        9495579 => {
            Samus_Func9();
            return;
        }
        9496846 => return,
        _ => {
            Unreachable();
        }
    };
}
pub unsafe fn RunFrameHandlerGamma() {
    CallFrameHandlerGamma(
        (*((&raw mut g_ram as *mut uint8).offset(0xa5a as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            | 0x900000 as ::core::ffi::c_int) as uint32,
    );
}
pub unsafe fn Samus_Func1() {
    if *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        == kPose_E9_FaceL_Drained_CrouchFalling as ::core::ffi::c_int
        && *((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 8 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        && *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & kButton_Up as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0xa94 as ::core::ffi::c_int as isize)
            as *mut uint16) = 1 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
            as *mut uint16) = 13 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xa5a as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x90e90e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Samus_Func2() {
    if *((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - 8 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
        && *((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 12 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        && *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & kButton_Up as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0xa94 as ::core::ffi::c_int as isize)
            as *mut uint16) = 1 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
            as *mut uint16) = 18 as uint16;
    }
}
pub unsafe fn Samus_Func3() {
    if ProcessTimer() as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        *((&raw mut g_ram as *mut uint8).offset(0x998 as ::core::ffi::c_int as isize)
            as *mut uint16) = kGameState_35_TimeUp as ::core::ffi::c_int as uint16;
        let mut i: ::core::ffi::c_int = 510 as ::core::ffi::c_int;
        while i >= 0 as ::core::ffi::c_int {
            *((&raw mut g_ram as *mut uint8).offset(0xc200 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset((i >> 1 as ::core::ffi::c_int) as isize) = 0x7fff as uint16;
            i -= 2 as ::core::ffi::c_int;
        }
        *((&raw mut g_ram as *mut uint8).offset(0xa5a as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x90e114 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        DisablePaletteFx();
    }
    if *((&raw mut g_ram as *mut uint8).offset(0x943 as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        DrawTimer();
    }
}
pub unsafe fn DrawTimer_() {
    DrawTimer();
}
pub unsafe fn Samus_SetPushedOutOfCeresRidley() {
    *((&raw mut g_ram as *mut uint8).offset(0xa58 as ::core::ffi::c_int as isize) as *mut uint16) =
        (0x90e90e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa5a as ::core::ffi::c_int as isize) as *mut uint16) =
        (0x90e12e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Samus_PushOutOfRidleysWay() {
    if *(&raw mut g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == 4 as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) = kPose_54_FaceL_Knockback as ::core::ffi::c_int as uint16;
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) = kPose_53_FaceR_Knockback as ::core::ffi::c_int as uint16;
    }
    SamusFunc_F433();
    Samus_SetAnimationFrameIfPoseChanged();
    *((&raw mut g_ram as *mut uint8).offset(0xa24 as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0xa20 as ::core::ffi::c_int as isize)
            as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0xa26 as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0xa22 as ::core::ffi::c_int as isize)
            as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0xa20 as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0xa22 as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
            as *mut uint16);
    let ref mut fresh131 = *((&raw mut g_ram as *mut uint8)
        .offset(0xafa as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh131 = (*fresh131 as ::core::ffi::c_int
        - (21 as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0xb00 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int)) as uint16;
    if *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        - 128 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        *((&raw mut g_ram as *mut uint8).offset(0xa62 as ::core::ffi::c_int as isize)
            as *mut uint16) = 1 as uint16;
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0xa62 as ::core::ffi::c_int as isize)
            as *mut uint16) = 2 as uint16;
    }
    *((&raw mut g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize) as *mut uint16) =
        5 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa56 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa5a as ::core::ffi::c_int as isize) as *mut uint16) =
        (0x90e1c8 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa28 as ::core::ffi::c_int as isize) as *mut uint16) =
        -(1 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa2a as ::core::ffi::c_int as isize) as *mut uint16) =
        -(1 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa2c as ::core::ffi::c_int as isize) as *mut uint16) =
        -(1 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa2e as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa30 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa32 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    ProcessTimer();
    if *((&raw mut g_ram as *mut uint8).offset(0x943 as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        DrawTimer();
    }
}
pub unsafe fn Samus_Func4() {
    static mut off_90E1F7: [Option<Func_V>; 3] = unsafe {
        [
            None,
            Some(Samus_Func5 as unsafe fn() -> ()),
            Some(Samus_Func6 as unsafe fn() -> ()),
        ]
    };
    if *((&raw mut g_ram as *mut uint8).offset(0xa28 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        == kPose_4F_FaceL_Dmgboost as ::core::ffi::c_int
        || *((&raw mut g_ram as *mut uint8).offset(0xa28 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == kPose_50_FaceR_Dmgboost as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0xa28 as ::core::ffi::c_int as isize)
            as *mut uint16) = -(1 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xa2e as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    }
    off_90E1F7[*((&raw mut g_ram as *mut uint8).offset(0xa62 as ::core::ffi::c_int as isize)
        as *mut uint16) as usize]
        .expect("non-null function pointer")();
    *((&raw mut g_ram as *mut uint8).offset(0xdc6 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    ProcessTimer();
    if *((&raw mut g_ram as *mut uint8).offset(0x943 as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        DrawTimer();
    }
}
pub unsafe fn Samus_Func5() {
    Samus_BombJumpFallingXMovement_();
    if *((&raw mut g_ram as *mut uint8).offset(0xdd0 as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        *((&raw mut g_ram as *mut uint8).offset(0xa58 as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x90a337 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xa5a as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x90e0e6 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xa62 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        Samus_ClearMoveVars();
    } else {
        Samus_BombJumpFallingYMovement_();
    };
}
pub unsafe fn Samus_Func6() {
    Samus_BombJumpFallingXMovement_();
    if *((&raw mut g_ram as *mut uint8).offset(0xdd0 as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        *((&raw mut g_ram as *mut uint8).offset(0xa58 as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x90a337 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xa5a as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x90e0e6 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xa62 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        Samus_ClearMoveVars();
    } else {
        Samus_BombJumpFallingYMovement_();
    };
}
pub unsafe fn Samus_GrabbedByDraygonFrameHandler() {
    if *((&raw mut g_ram as *mut uint8).offset(0xd32 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        == 0x9bc77e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0xa28 as ::core::ffi::c_int as isize)
            as *mut uint16) = -(1 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xa2e as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    }
    if *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & (kButton_Up as ::core::ffi::c_int
            | kButton_Down as ::core::ffi::c_int
            | kButton_Left as ::core::ffi::c_int
            | kButton_Right as ::core::ffi::c_int)
        != 0 as ::core::ffi::c_int
        && *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & (kButton_Up as ::core::ffi::c_int
                | kButton_Down as ::core::ffi::c_int
                | kButton_Left as ::core::ffi::c_int
                | kButton_Right as ::core::ffi::c_int)
            != *((&raw mut g_ram as *mut uint8).offset(0xdee as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0xdee as ::core::ffi::c_int as isize)
            as *mut uint16) = (*((&raw mut g_ram as *mut uint8)
            .offset(0x8f as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & (kButton_Up as ::core::ffi::c_int
                | kButton_Down as ::core::ffi::c_int
                | kButton_Left as ::core::ffi::c_int
                | kButton_Right as ::core::ffi::c_int)) as uint16;
        let ref mut fresh130 = *((&raw mut g_ram as *mut uint8)
            .offset(0xdec as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh130 = (*fresh130).wrapping_add(1);
        if (*fresh130 as ::core::ffi::c_int - 60 as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        {
            Samus_ReleaseFromDraygon();
        }
    }
}
pub unsafe fn Samus_SetGrabbedByDraygonPose(mut a: uint16) {
    if a as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) = kPose_EC_FaceR_Draygon_NoMove_NoAim as ::core::ffi::c_int as uint16;
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) = kPose_BA_FaceL_Draygon_NoMove_NoAim as ::core::ffi::c_int as uint16;
    }
    SamusFunc_F433();
    Samus_SetAnimationFrameIfPoseChanged();
    *((&raw mut g_ram as *mut uint8).offset(0xa5a as ::core::ffi::c_int as isize) as *mut uint16) =
        (0x90e2a1 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa58 as ::core::ffi::c_int as isize) as *mut uint16) =
        (0x90e90e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xdec as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xdee as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xdf0 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa24 as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0xa20 as ::core::ffi::c_int as isize)
            as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0xa26 as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0xa22 as ::core::ffi::c_int as isize)
            as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0xa20 as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0xa22 as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
            as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0xa28 as ::core::ffi::c_int as isize) as *mut uint16) =
        -(1 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa2a as ::core::ffi::c_int as isize) as *mut uint16) =
        -(1 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa2c as ::core::ffi::c_int as isize) as *mut uint16) =
        -(1 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa2e as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa30 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa32 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
}
pub unsafe fn Samus_ReleaseFromDraygon_() {
    Samus_ReleaseFromDraygon();
}
pub unsafe fn Samus_ReleaseFromDraygon() {
    if *(&raw mut g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == 4 as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) = kPose_02_FaceL_Normal as ::core::ffi::c_int as uint16;
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) = kPose_01_FaceR_Normal as ::core::ffi::c_int as uint16;
    }
    SamusFunc_F433();
    Samus_SetAnimationFrameIfPoseChanged();
    *((&raw mut g_ram as *mut uint8).offset(0xa58 as ::core::ffi::c_int as isize) as *mut uint16) =
        (0x90a337 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa5a as ::core::ffi::c_int as isize) as *mut uint16) =
        (0x90e90e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa24 as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0xa20 as ::core::ffi::c_int as isize)
            as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0xa26 as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0xa22 as ::core::ffi::c_int as isize)
            as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0xa20 as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0xa22 as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
            as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0xb46 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb48 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa28 as ::core::ffi::c_int as isize) as *mut uint16) =
        -(1 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa2a as ::core::ffi::c_int as isize) as *mut uint16) =
        -(1 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa2c as ::core::ffi::c_int as isize) as *mut uint16) =
        -(1 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa2e as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa30 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa32 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb36 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb20 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb4a as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa64 as ::core::ffi::c_int as isize) as *mut uint16) =
        (*((&raw mut g_ram as *mut uint8).offset(0xa64 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0xfffd as ::core::ffi::c_int
            | 2 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Samus_Func7() {
    Samus_BombJumpFallingXMovement_();
    if *((&raw mut g_ram as *mut uint8).offset(0xdd0 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        != 0
        || {
            Samus_BombJumpFallingYMovement_();
            *((&raw mut g_ram as *mut uint8).offset(0xdd0 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                != 0
        }
    {
        *((&raw mut g_ram as *mut uint8).offset(0xa58 as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x90a337 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xa5a as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x90e90e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xa62 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        Samus_ClearMoveVars();
        *((&raw mut g_ram as *mut uint8).offset(0xa2a as ::core::ffi::c_int as isize)
            as *mut uint16) = 65 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xa30 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    }
}
pub unsafe fn Samus_Func8() {
    *((&raw mut g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa58 as ::core::ffi::c_int as isize) as *mut uint16) =
        (0x90a337 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa5a as ::core::ffi::c_int as isize) as *mut uint16) =
        (0x90e90e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Samus_Func9() {
    if *((&raw mut g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - 5 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        AddToHiLo(
            (&raw mut g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
                as *mut uint16,
            (&raw mut g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize)
                as *mut uint16,
            (*((&raw mut g_ram as *mut uint8).offset(0xb34 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint32)
                << 16 as ::core::ffi::c_int
                | *((&raw mut g_ram as *mut uint8).offset(0xb32 as ::core::ffi::c_int as isize)
                    as *mut uint16) as uint32,
        );
    }
    if (*((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        == kPose_29_FaceR_Fall as ::core::ffi::c_int
        || *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == kPose_2A_FaceL_Fall as ::core::ffi::c_int
        || *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == kPose_67_FaceR_Fall_Gun as ::core::ffi::c_int
        || *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == kPose_68_FaceL_Fall_Gun as ::core::ffi::c_int)
        && *((&raw mut g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 5 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
    {
        *((&raw mut g_ram as *mut uint8).offset(0xa94 as ::core::ffi::c_int as isize)
            as *mut uint16) = 16 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
            as *mut uint16) = 4 as uint16;
    }
}
unsafe fn Samus_ClampSpeed(mut amt: int32) -> int32 {
    let mut r18: uint16 = (amt >> 16 as ::core::ffi::c_int) as uint16;
    let mut r20: uint16 = amt as uint16;
    let mut v0: int16 = r18 as int16;
    if (v0 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
        if v0 as ::core::ffi::c_int + 15 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            r18 = -(15 as ::core::ffi::c_int) as uint16;
        }
    } else if v0 as ::core::ffi::c_int - 16 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        == 0
    {
        r18 = 15 as uint16;
    }
    return (r18 as int32) << 16 as ::core::ffi::c_int | r20 as int32;
}
pub unsafe fn Samus_CalcDisplacementMoveLeft(mut amt: int32) -> int32 {
    amt = Samus_CalcSpeed_X(amt as uint32) as int32;
    *((&raw mut g_ram as *mut uint8).offset(0xb02 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    amt = ((*((&raw mut g_ram as *mut uint8).offset(0xb58 as ::core::ffi::c_int as isize)
        as *mut uint16) as uint32)
        << 16 as ::core::ffi::c_int
        | *((&raw mut g_ram as *mut uint8).offset(0xb56 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint32)
        .wrapping_sub(amt as uint32) as int32;
    return Samus_ClampSpeed(amt);
}
pub unsafe fn Samus_CalcDisplacementMoveRight(mut amt: int32) -> int32 {
    amt = Samus_CalcSpeed_X(amt as uint32) as int32;
    *((&raw mut g_ram as *mut uint8).offset(0xb02 as ::core::ffi::c_int as isize) as *mut uint16) =
        1 as uint16;
    amt = (amt as uint32).wrapping_add(
        (*((&raw mut g_ram as *mut uint8).offset(0xb58 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint32)
            << 16 as ::core::ffi::c_int
            | *((&raw mut g_ram as *mut uint8).offset(0xb56 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint32,
    ) as int32 as int32;
    return Samus_ClampSpeed(amt);
}
pub unsafe fn Samus_CalcSpeed_X(mut amt: uint32) -> uint32 {
    amt = amt.wrapping_add(
        (*((&raw mut g_ram as *mut uint8).offset(0xb42 as ::core::ffi::c_int as isize)
            as *mut uint16) as uint32)
            << 16 as ::core::ffi::c_int
            | *((&raw mut g_ram as *mut uint8).offset(0xb44 as ::core::ffi::c_int as isize)
                as *mut uint16) as uint32,
    );
    amt >>= if *((&raw mut g_ram as *mut uint8).offset(0xa66 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        <= 4 as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0xa66 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
    } else {
        4 as ::core::ffi::c_int
    };
    SetHiLo(
        (&raw mut g_ram as *mut uint8).offset(0xdbc as ::core::ffi::c_int as isize) as *mut uint16,
        (&raw mut g_ram as *mut uint8).offset(0xdbe as ::core::ffi::c_int as isize) as *mut uint16,
        amt,
    );
    return amt;
}
pub unsafe fn Samus_ClearXSpeedIfColl() {
    if *((&raw mut g_ram as *mut uint8).offset(0xdd0 as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        if *((&raw mut g_ram as *mut uint8).offset(0xb02 as ::core::ffi::c_int as isize)
            as *mut uint16)
            != 0
        {
            *((&raw mut g_ram as *mut uint8).offset(0xdce as ::core::ffi::c_int as isize)
                as *mut uint16) = 8 as uint16;
        } else {
            *((&raw mut g_ram as *mut uint8).offset(0xdce as ::core::ffi::c_int as isize)
                as *mut uint16) = 4 as uint16;
        }
        Samus_CancelSpeedBoost();
        *((&raw mut g_ram as *mut uint8).offset(0xb42 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb44 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb46 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb48 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xb4a as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0xdc6 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xdce as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    };
}
pub unsafe fn Samus_MoveUp_SetPoseCalcInput() {
    if *((&raw mut g_ram as *mut uint8).offset(0xdd0 as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        *((&raw mut g_ram as *mut uint8).offset(0xdc6 as ::core::ffi::c_int as isize)
            as *mut uint16) = 4 as uint16;
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0xdc6 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    };
}
static mut kSamus_MoveDown_SetPoseCalcInput_Tab0: [uint8; 28] = [
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    3 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
];
static mut kSamus_MoveDown_SetPoseCalcInput_Tab1: [uint8; 28] = [
    4 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    3 as ::core::ffi::c_int as uint8,
    3 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
];
pub unsafe fn Samus_MoveDown_SetPoseCalcInput() {
    if *((&raw mut g_ram as *mut uint8).offset(0xdd0 as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        *((&raw mut g_ram as *mut uint8).offset(0xdc6 as ::core::ffi::c_int as isize)
            as *mut uint16) = 1 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xdc6 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) = kSamus_MoveDown_SetPoseCalcInput_Tab1[*(&raw mut g_ram as *mut uint8)
            .offset(0xa1f as ::core::ffi::c_int as isize) as usize];
    } else if *((&raw mut g_ram as *mut uint8).offset(0xdc6 as ::core::ffi::c_int as isize)
        as *mut uint16) as uint8 as ::core::ffi::c_int
        != 5 as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0xdc6 as ::core::ffi::c_int as isize)
            as *mut uint16) = 2 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xdc6 as ::core::ffi::c_int as isize)
            as *mut uint16 as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) = kSamus_MoveDown_SetPoseCalcInput_Tab0[*(&raw mut g_ram as *mut uint8)
            .offset(0xa1f as ::core::ffi::c_int as isize) as usize];
    }
}
pub unsafe fn CallFrameHandlerAlfa(mut ea: uint32) {
    match ea {
        9496213 => {
            Samus_FrameHandlerAlfa_Func11();
            return;
        }
        9496265 => {
            Samus_FrameHandlerAlfa_Func12();
            return;
        }
        9496339 => {
            Samus_FrameHandlerAlfa_Func13();
            return;
        }
        9496781 => return,
        _ => {
            Unreachable();
        }
    };
}
pub unsafe fn HandleControllerInputForGamePhysics() {
    CallFrameHandlerAlfa(
        (*((&raw mut g_ram as *mut uint8).offset(0xa42 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            | 0x900000 as ::core::ffi::c_int) as uint32,
    );
}
pub unsafe fn Samus_FrameHandlerAlfa_Func11() {
    *((&raw mut g_ram as *mut uint8).offset(0xa28 as ::core::ffi::c_int as isize) as *mut uint16) =
        -(1 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa2a as ::core::ffi::c_int as isize) as *mut uint16) =
        -(1 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa2c as ::core::ffi::c_int as isize) as *mut uint16) =
        -(1 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa2e as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa30 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa32 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    Samus_SetRadius();
    Samus_CallInputHandler();
    Samus_UpdateSuitPaletteIndex();
    Samus_DetermineAccel_Y();
    BlockInsideDetection();
    Samus_HandleHudSpecificBehaviorAndProjs();
    Samus_Func10();
}
pub unsafe fn Samus_FrameHandlerAlfa_Func12() {
    *((&raw mut g_ram as *mut uint8).offset(0xa28 as ::core::ffi::c_int as isize) as *mut uint16) =
        -(1 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa2a as ::core::ffi::c_int as isize) as *mut uint16) =
        -(1 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa2c as ::core::ffi::c_int as isize) as *mut uint16) =
        -(1 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa2e as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa30 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa32 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa14 as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0xa16 as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0xa90 as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0xdfe as ::core::ffi::c_int as isize)
            as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0xa92 as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0xe00 as ::core::ffi::c_int as isize)
            as *mut uint16);
    Samus_SetRadius();
    Samus_UpdateSuitPaletteIndex();
    Samus_CallInputHandler();
    Samus_DetermineAccel_Y();
    BlockInsideDetection();
    Samus_HandleHudSpecificBehaviorAndProjs();
    Samus_Func10();
}
pub unsafe fn Samus_FrameHandlerAlfa_Func13() {
    HandleProjectile();
    Samus_Func10();
}
pub unsafe fn CallFrameHandlerBeta(mut ea: uint32) {
    match ea {
        9496357 => {
            Samus_FrameHandlerBeta_Func17();
            return;
        }
        9496530 => {
            HandleDemoRecorder_3();
            return;
        }
        9496565 => {
            Samus_FrameHandlerBeta_Func14();
            return;
        }
        9496627 => {
            Samus_Func15();
            return;
        }
        9496682 => {
            Samus_Func16();
            return;
        }
        9496746 => {
            Samus_Func18();
            return;
        }
        9496781 => return,
        9496790 => return,
        9496793 => return,
        9496796 => {
            SetContactDamageIndexAndUpdateMinimap();
            return;
        }
        9496812 => {
            Samus_Func19();
            return;
        }
        9496834 => {
            Samus_LowHealthCheck();
            return;
        }
        _ => {
            Unreachable();
        }
    };
}
pub unsafe fn HandleSamusMovementAndPause() {
    CallFrameHandlerBeta(
        (*((&raw mut g_ram as *mut uint8).offset(0xa44 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            | 0x900000 as ::core::ffi::c_int) as uint32,
    );
}
pub unsafe fn Samus_FrameHandlerBeta_Func17() {
    *((&raw mut g_ram as *mut uint8).offset(0xa6e as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    RunSamusMovementHandler();
    UpdateMinimap();
    RunFrameHandlerGamma();
    Samus_Animate();
    Samus_HitInterruption();
    Samus_HandleTransFromBlockColl();
    Samus_HandleTransitions();
    Samus_HandlePalette();
    Samus_HandlePeriodicDamage();
    Samus_PauseCheck();
    Samus_LowHealthCheck_();
}
pub unsafe fn HandleDemoRecorder_1() {
    if *((&raw mut g_ram as *mut uint8).offset(0x91 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        if *((&raw mut g_ram as *mut uint8).offset(0x9e6 as ::core::ffi::c_int as isize)
            as *mut uint16)
            == 0
            && *((&raw mut g_ram as *mut uint8).offset(0x91 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0x80 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
        {
            DisableEprojs();
            *((&raw mut g_ram as *mut uint8).offset(0xa78 as ::core::ffi::c_int as isize)
                as *mut uint16) = 1 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xa42 as ::core::ffi::c_int as isize)
                as *mut uint16) =
                (0x90e8cd as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xa44 as ::core::ffi::c_int as isize)
                as *mut uint16) =
                (0x90e7d2 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
    } else if *((&raw mut g_ram as *mut uint8).offset(0x9e6 as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0
    {
        *((&raw mut g_ram as *mut uint8).offset(0xa5c as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x90e90e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x9e6 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0x9e6 as ::core::ffi::c_int as isize)
            as *mut uint16) = 1 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xa5c as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x90eb52 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn HandleDemoRecorder_3() {
    if *((&raw mut g_ram as *mut uint8).offset(0x91 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0x80 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0xa42 as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x90e695 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xa44 as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x90e725 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        EnableEprojs();
        *((&raw mut g_ram as *mut uint8).offset(0xa78 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    }
}
pub unsafe fn Samus_FrameHandlerBeta_Func14() {
    *((&raw mut g_ram as *mut uint8).offset(0xa6e as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    RunSamusMovementHandler();
    UpdateMinimap();
    Samus_Animate();
    Samus_HitInterruption();
    Samus_HandleTransFromBlockColl();
    Samus_HandleTransitions();
    Samus_HandlePalette();
    *((&raw mut g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0xa14 as ::core::ffi::c_int as isize)
            as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0xa16 as ::core::ffi::c_int as isize)
            as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0xdfe as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0xa90 as ::core::ffi::c_int as isize)
            as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0xe00 as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0xa92 as ::core::ffi::c_int as isize)
            as *mut uint16);
}
pub unsafe fn Samus_Func15() {
    *((&raw mut g_ram as *mut uint8).offset(0xa6e as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    RunSamusMovementHandler();
    Samus_Animate();
    Samus_HitInterruption();
    Samus_HandleTransFromBlockColl();
    Samus_HandleTransitions();
    Samus_HandlePalette();
    *((&raw mut g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0xa14 as ::core::ffi::c_int as isize)
            as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0xa16 as ::core::ffi::c_int as isize)
            as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0xdfe as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0xa90 as ::core::ffi::c_int as isize)
            as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0xe00 as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0xa92 as ::core::ffi::c_int as isize)
            as *mut uint16);
}
pub unsafe fn Samus_Func16() {
    Samus_SetRadius();
    UpdateMinimap();
    Samus_Animate();
    *((&raw mut g_ram as *mut uint8).offset(0xe18 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb14 as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16);
    if PlaySamusFanfare() as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        if *((&raw mut g_ram as *mut uint8).offset(0xde0 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 7 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
            || *((&raw mut g_ram as *mut uint8).offset(0x8d as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
        {
            *((&raw mut g_ram as *mut uint8).offset(0xde0 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
        }
        *((&raw mut g_ram as *mut uint8).offset(0xa42 as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x90e695 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xa44 as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x90e725 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Samus_Func18() {
    Samus_FrameHandlerBeta_Func17();
    if *((&raw mut g_ram as *mut uint8).offset(0xa5a as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        == 0x90e114 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
        && *((&raw mut g_ram as *mut uint8).offset(0x998 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != kGameState_35_TimeUp as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0x998 as ::core::ffi::c_int as isize)
            as *mut uint16) = kGameState_35_TimeUp as ::core::ffi::c_int as uint16;
    }
}
pub unsafe fn SetContactDamageIndexAndUpdateMinimap() {
    *((&raw mut g_ram as *mut uint8).offset(0xa6e as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    UpdateMinimap();
}
pub unsafe fn Samus_Func19() {
    *((&raw mut g_ram as *mut uint8).offset(0xa6e as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    RunSamusMovementHandler();
    UpdateMinimap();
    Samus_Animate();
}
pub unsafe fn Samus_LowHealthCheck() {
    Samus_LowHealthCheck_();
}
pub unsafe fn CallSamusInputHandler(mut ea: uint32) {
    match ea {
        9496846 => return,
        9496851 => {
            Samus_InputHandler_E913();
            return;
        }
        9496856 => {
            Samus_Func20_();
            return;
        }
        9496861 => {
            Samus_InputHandler_E91D();
            return;
        }
        9496870 => {
            HandleAutoJumpHack();
            return;
        }
        _ => {
            Unreachable();
        }
    };
}
pub unsafe fn Samus_CallInputHandler() {
    CallSamusInputHandler(
        (*((&raw mut g_ram as *mut uint8).offset(0xa60 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            | 0x900000 as ::core::ffi::c_int) as uint32,
    );
}
pub unsafe fn Samus_InputHandler_E913() {
    Samus_InputHandler();
}
pub unsafe fn Samus_Func20_() {
    Samus_Func20();
}
pub unsafe fn Samus_InputHandler_E91D() {
    DemoObjectInputHandler();
    Samus_InputHandler();
}
pub unsafe fn HandleAutoJumpHack() {
    let mut v0: uint16 = *((&raw mut g_ram as *mut uint8)
        .offset(0x8f as ::core::ffi::c_int as isize) as *mut uint16);
    if *((&raw mut g_ram as *mut uint8).offset(0xaf4 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        != 0
        && *((&raw mut g_ram as *mut uint8).offset(0xaf4 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 9 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
    {
        let ref mut fresh26 = *((&raw mut g_ram as *mut uint8)
            .offset(0x8f as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh26 = (*fresh26 as ::core::ffi::c_int
            | *((&raw mut g_ram as *mut uint8).offset(0x9b4 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xaf4 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    }
    Samus_InputHandler();
    *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize) as *mut uint16) =
        v0;
    *((&raw mut g_ram as *mut uint8).offset(0xa60 as ::core::ffi::c_int as isize) as *mut uint16) =
        (0x90e913 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
}
pub unsafe fn CallSamusMovementHandler(mut ea: uint32) {
    match ea {
        9475182 => {
            Samus_MoveHandler_ReleaseFromGrapple();
            return;
        }
        9475275 => {
            Samus_HandleMovement_DrainedCrouching();
            return;
        }
        9478967 => {
            Samus_MovementHandler_Normal();
            return;
        }
        9490536 => {
            Samus_MoveHandlerShinesparkWindup();
            return;
        }
        9490603 => {
            Samus_MoveHandlerVerticalShinespark();
            return;
        }
        9490647 => {
            Samus_MoveHandler_Shinespark_Diag();
            return;
        }
        9490694 => {
            Samus_MoveHandler_Shinespark_Horiz();
            return;
        }
        9491270 => {
            Samus_MoveHandler_ShinesparkCrash();
            return;
        }
        9491331 => {
            Samus_MoveHandler_ShinesparkCrash_0();
            return;
        }
        9491350 => {
            Samus_MoveHandler_ShinesparkCrash_1();
            return;
        }
        9491404 => {
            Samus_MoveHandler_ShinesparkCrash_2();
            return;
        }
        9491443 => {
            Samus_MoveHandler_ShinesparkCrashEchoCircle();
            return;
        }
        9491469 => {
            Samus_MoveHandler_ShinesparkCrashFinish();
            return;
        }
        9492088 => {
            SamusMoveHandler_CrystalFlashStart();
            return;
        }
        9492174 => {
            SamusMoveHandler_CrystalFlashMain();
            return;
        }
        9492315 => {
            kSamusMoveHandler_CrystalFlashFinish();
            return;
        }
        9494328 => {
            Samus_MoveHandler_Knockback();
            return;
        }
        9494352 => {
            Samus_MoveHandler_Knockback_0();
            return;
        }
        9494355 => {
            Samus_MoveHandler_Knockback_Up();
            return;
        }
        9494365 => {
            Samus_MoveHandler_Knockback_3();
            return;
        }
        9494372 => {
            Samus_MoveHandler_Knockback_Down();
            return;
        }
        9494565 => {
            Samus_MoveHandler_BombJumpStart();
            return;
        }
        9494578 => {
            Samus_MoveHandler_BombJumpMain();
            return;
        }
        9494653 => {
            Samus_MoveHandler_BombJumpFunc1();
            return;
        }
        9496846 => return,
        9496911 => {
            SamusMovementType_Xray();
            return;
        }
        9498699 => {
            Samus_Func25_ShineSpark();
            return;
        }
        9498738 => {
            Samus_MoveHandler_F072();
            return;
        }
        _ => {
            Unreachable();
        }
    };
}
pub unsafe fn RunSamusMovementHandler() {
    CallSamusMovementHandler(
        (*((&raw mut g_ram as *mut uint8).offset(0xa58 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            | 0x900000 as ::core::ffi::c_int) as uint32,
    );
}
pub unsafe fn SamusMovementType_Xray() {
    let mut v0: uint16 = 0;
    if *(&raw mut g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        != kMovementType_0E_TurningAroundOnGround as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0xa94 as ::core::ffi::c_int as isize)
            as *mut uint16) = 15 as uint16;
        if *(&raw mut g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            == 4 as ::core::ffi::c_int
        {
            if *((&raw mut g_ram as *mut uint8).offset(0xa82 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - 153 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
            {
                v0 = 4 as uint16;
            } else if *((&raw mut g_ram as *mut uint8).offset(0xa82 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - 178 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
            {
                v0 = 3 as uint16;
            } else if *((&raw mut g_ram as *mut uint8).offset(0xa82 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - 203 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
            {
                v0 = 2 as uint16;
            } else {
                v0 = (*((&raw mut g_ram as *mut uint8).offset(0xa82 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    - 228 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int) as ::core::ffi::c_int
                    as uint16;
            }
        } else if *((&raw mut g_ram as *mut uint8).offset(0xa82 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 25 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            v0 = 0 as uint16;
        } else if *((&raw mut g_ram as *mut uint8).offset(0xa82 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 50 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            v0 = 1 as uint16;
        } else if *((&raw mut g_ram as *mut uint8).offset(0xa82 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 75 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            v0 = 2 as uint16;
        } else if *((&raw mut g_ram as *mut uint8).offset(0xa82 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 100 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            v0 = 3 as uint16;
        } else {
            v0 = 4 as uint16;
        }
        *((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
            as *mut uint16) = v0;
    }
}
pub unsafe fn Samus_HandlePeriodicDamage() {
    if *((&raw mut g_ram as *mut uint8).offset(0xa78 as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        let ref mut fresh127 = *((&raw mut g_ram as *mut uint8)
            .offset(0xa4e as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh127 = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xa50 as ::core::ffi::c_int as isize)
            as *mut uint16) = *fresh127;
        return;
    }
    let mut t: int32 = ((*((&raw mut g_ram as *mut uint8)
        .offset(0xa50 as ::core::ffi::c_int as isize) as *mut uint16)
        as uint32)
        << 16 as ::core::ffi::c_int
        | *((&raw mut g_ram as *mut uint8).offset(0xa4e as ::core::ffi::c_int as isize)
            as *mut uint16) as uint32) as int32;
    if t < 0 as int32 {
        InvalidInterrupt_Crash();
        return;
    }
    if *((&raw mut g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0x20 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        t = t >> 2 as ::core::ffi::c_int & 0xffff00 as int32;
    } else if *((&raw mut g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        t = t >> 1 as ::core::ffi::c_int & 0xffff00 as int32;
    }
    AddToHiLo(
        (&raw mut g_ram as *mut uint8).offset(0x9c2 as ::core::ffi::c_int as isize) as *mut uint16,
        (&raw mut g_ram as *mut uint8).offset(0xa4c as ::core::ffi::c_int as isize) as *mut uint16,
        -t as uint32,
    );
    if (*((&raw mut g_ram as *mut uint8).offset(0x9c2 as ::core::ffi::c_int as isize)
        as *mut uint16) as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        let ref mut fresh128 = *((&raw mut g_ram as *mut uint8)
            .offset(0xa4c as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh128 = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x9c2 as ::core::ffi::c_int as isize)
            as *mut uint16) = *fresh128;
    }
    let ref mut fresh129 = *((&raw mut g_ram as *mut uint8)
        .offset(0xa4e as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh129 = 0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa50 as ::core::ffi::c_int as isize) as *mut uint16) =
        *fresh129;
}
pub unsafe fn Samus_PauseCheck() {
    if *((&raw mut g_ram as *mut uint8).offset(0xcee as ::core::ffi::c_int as isize) as *mut uint16)
        == 0
        && *((&raw mut g_ram as *mut uint8).offset(0xa78 as ::core::ffi::c_int as isize)
            as *mut uint16)
            == 0
        && *((&raw mut g_ram as *mut uint8).offset(0x797 as ::core::ffi::c_int as isize)
            as *mut uint16)
            == 0
        && *((&raw mut g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != 6 as ::core::ffi::c_int
        && *((&raw mut g_ram as *mut uint8).offset(0x998 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == kGameState_8_MainGameplay as ::core::ffi::c_int
        && *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & kButton_Start as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0x723 as ::core::ffi::c_int as isize)
            as *mut uint16) = 1 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x725 as ::core::ffi::c_int as isize)
            as *mut uint16) = 1 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x998 as ::core::ffi::c_int as isize)
            as *mut uint16) = kGameState_12_Pausing as ::core::ffi::c_int as uint16;
    }
}
pub unsafe fn Samus_LowHealthCheck_() {
    if *((&raw mut g_ram as *mut uint8).offset(0x9c2 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - 31 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        if *((&raw mut g_ram as *mut uint8).offset(0xa6a as ::core::ffi::c_int as isize)
            as *mut uint16)
            == 0
        {
            QueueSfx3_Max6(2 as uint16);
            *((&raw mut g_ram as *mut uint8).offset(0xa6a as ::core::ffi::c_int as isize)
                as *mut uint16) = 1 as uint16;
        }
    } else if *((&raw mut g_ram as *mut uint8).offset(0xa6a as ::core::ffi::c_int as isize)
        as *mut uint16)
        != 0
    {
        *((&raw mut g_ram as *mut uint8).offset(0xa6a as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        QueueSfx3_Max6(1 as uint16);
    }
}
pub unsafe fn Samus_LowHealthCheck_0() {
    Samus_LowHealthCheck_();
}
pub unsafe fn Samus_JumpCheck() {
    let mut current_block: u64;
    if *((&raw mut g_ram as *mut uint8).offset(0x9b4 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & *((&raw mut g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
        && *((&raw mut g_ram as *mut uint8).offset(0x9b4 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & *((&raw mut g_ram as *mut uint8).offset(0xdfe as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
    {
        let ref mut fresh14 = *((&raw mut g_ram as *mut uint8)
            .offset(0xaf4 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *fresh14 = (*fresh14).wrapping_add(1);
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0xaf4 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    }
    *((&raw mut g_ram as *mut uint8).offset(0xdfe as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0xe00 as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0x8f as ::core::ffi::c_int as isize) as *mut uint16);
    if (*((&raw mut g_ram as *mut uint8).offset(0x9c2 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - *((&raw mut g_ram as *mut uint8).offset(0xa12 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
        >= 0 as ::core::ffi::c_int
    {
        current_block = 16352754044285948771;
    } else {
        if *((&raw mut g_ram as *mut uint8).offset(0xa48 as ::core::ffi::c_int as isize)
            as *mut uint16)
            == 0
        {
            *((&raw mut g_ram as *mut uint8).offset(0xa48 as ::core::ffi::c_int as isize)
                as *mut uint16) = 1 as uint16;
        }
        if *((&raw mut g_ram as *mut uint8).offset(0xde0 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 7 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            current_block = 16352754044285948771;
        } else {
            *((&raw mut g_ram as *mut uint8).offset(0x9c2 as ::core::ffi::c_int as isize)
                as *mut uint16) = *((&raw mut g_ram as *mut uint8)
                .offset(0xa12 as ::core::ffi::c_int as isize)
                as *mut uint16);
            current_block = 8515828400728868193;
        }
    }
    match current_block {
        16352754044285948771 => {
            *((&raw mut g_ram as *mut uint8).offset(0xa12 as ::core::ffi::c_int as isize)
                as *mut uint16) = *((&raw mut g_ram as *mut uint8)
                .offset(0x9c2 as ::core::ffi::c_int as isize)
                as *mut uint16);
        }
        _ => {}
    };
}
pub unsafe fn Samus_Func10() {
    *((&raw mut g_ram as *mut uint8).offset(0xdaa as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xdac as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xdae as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xdb0 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xdb2 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xdb4 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xdb6 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xdb8 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa9a as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb5e as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    let ref mut fresh27 = *((&raw mut g_ram as *mut uint8)
        .offset(0xdfa as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh27 = ((*fresh27 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa10 as ::core::ffi::c_int as isize) as *mut uint8
        as *mut uint16) = *((&raw mut g_ram as *mut uint8)
        .offset(0xa1e as ::core::ffi::c_int as isize) as *mut uint16);
}
pub unsafe fn DrawSamusAndProjectiles() {
    SamusDrawSprites();
    DrawPlayerExplosions2();
    Samus_JumpCheck();
    Samus_ShootCheck();
}
pub unsafe fn CallSamusDrawHandler(mut ea: uint32) {
    match ea {
        9497426 => {
            SamusDrawHandler_Default();
            return;
        }
        9497478 => {
            sub_90EB86();
            return;
        }
        9496846 | 9497586 => return,
        9497587 => {
            Samus_DrawHandler_EndOfShinespark();
            return;
        }
        9497620 => {
            SamusDisplayHandler_UsingElevator();
            return;
        }
        9497629 => {
            SamusDisplayHandler_SamusReceivedFatal();
            return;
        }
        _ => {
            Unreachable();
        }
    };
}
pub unsafe fn SamusDrawSprites() {
    HandleArmCannonOpenState();
    CallSamusDrawHandler(
        (*((&raw mut g_ram as *mut uint8).offset(0xa5c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            | 0x900000 as ::core::ffi::c_int) as uint32,
    );
}
pub unsafe fn SamusDrawHandler_Default() {
    HandleChargingBeamGfxAudio();
    sub_90EB55();
}
pub unsafe fn sub_90EB55() {
    if *((&raw mut g_ram as *mut uint8).offset(0xaac as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        if *((&raw mut g_ram as *mut uint8).offset(0xaac as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int
            == 2 as ::core::ffi::c_int
        {
            HandleAtmosphericEffects();
            Samus_Draw();
            Samus_ArmCannon_Draw();
        } else {
            HandleAtmosphericEffects();
            Samus_ArmCannon_Draw();
            Samus_Draw();
        }
        Samus_DrawEchoes();
    } else {
        HandleAtmosphericEffects();
        Samus_Draw();
        Samus_DrawEchoes();
    };
}
pub unsafe fn sub_90EB86() {
    if *((&raw mut g_ram as *mut uint8).offset(0xd32 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        + 0x37aa as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        sub_90EB55();
        return;
    }
    if *((&raw mut g_ram as *mut uint8).offset(0xd32 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        == 0x9bc703 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
        || *((&raw mut g_ram as *mut uint8).offset(0xd32 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == 0x9bc759 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        GrappleBeamFunc_BF1B();
    }
    HandleGrappleBeamFlare();
    if *((&raw mut g_ram as *mut uint8).offset(0xaac as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        if *((&raw mut g_ram as *mut uint8).offset(0xaac as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int
            == 2 as ::core::ffi::c_int
        {
            HandleAtmosphericEffects();
            Samus_Draw();
            Samus_ArmCannon_Draw();
            UpdateGrappleBeamTilesAndIncrFlameCtr();
            if *((&raw mut g_ram as *mut uint8).offset(0xcfe as ::core::ffi::c_int as isize)
                as *mut uint16)
                == 0
            {
                return;
            }
        } else {
            HandleAtmosphericEffects();
            Samus_ArmCannon_Draw();
            Samus_Draw();
            UpdateGrappleBeamTilesAndIncrFlameCtr();
            if *((&raw mut g_ram as *mut uint8).offset(0xcfe as ::core::ffi::c_int as isize)
                as *mut uint16)
                == 0
            {
                return;
            }
        }
    } else {
        HandleAtmosphericEffects();
        Samus_Draw();
        UpdateGrappleBeamTilesAndIncrFlameCtr();
        if *((&raw mut g_ram as *mut uint8).offset(0xcfe as ::core::ffi::c_int as isize)
            as *mut uint16)
            == 0
        {
            return;
        }
    }
    HandleGrappleBeamGfx();
}
pub unsafe fn Samus_DrawHandler_EndOfShinespark() {
    Samus_Draw();
    let mut i: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        Samus_DrawShinesparkCrashEchoes(i as uint16);
        i -= 2 as ::core::ffi::c_int;
    }
}
pub unsafe fn Samus_DrawManyEchoes() {
    SamusDrawHandler_Default();
    let mut i: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        Samus_DrawEchoes();
        i -= 2 as ::core::ffi::c_int;
    }
}
pub unsafe fn SamusDisplayHandler_UsingElevator() {
    if *((&raw mut g_ram as *mut uint8).offset(0x5b6 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        SamusDisplayHandler_SamusReceivedFatal();
    }
}
pub unsafe fn SamusDisplayHandler_SamusReceivedFatal() {
    Samus_DrawWhenNotAnimatingOrDying();
}
pub unsafe fn Samus_SetRadius() {
    *((&raw mut g_ram as *mut uint8).offset(0xb00 as ::core::ffi::c_int as isize) as *mut uint16) =
        (*(g_rom.offset(
            (((0x91b629 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x91b629 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut SamusPoseParams)
            .offset(
                *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                    as *mut uint16) as isize,
            ))
        .y_radius as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xafe as ::core::ffi::c_int as isize) as *mut uint16) =
        5 as uint16;
}
pub unsafe fn Samus_GetBottom_R18() -> uint16 {
    return (*((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        + (*(g_rom.offset(
            (((0x91b629 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x91b629 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut SamusPoseParams)
            .offset(
                *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                    as *mut uint16) as isize,
            ))
        .y_radius as ::core::ffi::c_int
        - 1 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Samus_GetTop_R20() -> uint16 {
    return (*((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - (*(g_rom.offset(
            (((0x91b629 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x91b629 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut SamusPoseParams)
            .offset(
                *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                    as *mut uint16) as isize,
            ))
        .y_radius as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Samus_AlignBottomWithPrevPose() {
    let mut r18: uint16 = (*(g_rom.offset(
        (((0x91b629 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x91b629 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut SamusPoseParams)
        .offset(
            *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) as isize,
        ))
    .y_radius as uint16;
    r18 = ((*(g_rom.offset(
        (((0x91b629 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x91b629 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut SamusPoseParams)
        .offset(
            *((&raw mut g_ram as *mut uint8).offset(0xa20 as ::core::ffi::c_int as isize)
                as *mut uint16) as isize,
        ))
    .y_radius as ::core::ffi::c_int
        - r18 as ::core::ffi::c_int) as uint16;
    let ref mut fresh101 = *((&raw mut g_ram as *mut uint8)
        .offset(0xafa as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh101 = (*fresh101 as ::core::ffi::c_int + r18 as ::core::ffi::c_int) as uint16;
    let ref mut fresh102 = *((&raw mut g_ram as *mut uint8)
        .offset(0xb14 as ::core::ffi::c_int as isize) as *mut uint16);
    *fresh102 = (*fresh102 as ::core::ffi::c_int + r18 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Samus_UpdateSuitPaletteIndex() {
    if *((&raw mut g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0x20 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0xa74 as ::core::ffi::c_int as isize)
            as *mut uint16) = 4 as uint16;
    } else if *((&raw mut g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0xa74 as ::core::ffi::c_int as isize)
            as *mut uint16) = 2 as uint16;
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0xa74 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
    };
}
pub unsafe fn MoveSamusWithControlPad() {
    if *((&raw mut g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0x800 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        Samus_MoveUp((-(4 as ::core::ffi::c_int) as int16 as int32) << 16 as ::core::ffi::c_int);
    }
    if *((&raw mut g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0x400 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        Samus_MoveDown((4 as ::core::ffi::c_int as int16 as int32) << 16 as ::core::ffi::c_int);
    }
    if *((&raw mut g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0x200 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        Samus_MoveLeft((-(4 as ::core::ffi::c_int) as int16 as int32) << 16 as ::core::ffi::c_int);
    }
    if *((&raw mut g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0x100 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        Samus_MoveRight((4 as ::core::ffi::c_int as int16 as int32) << 16 as ::core::ffi::c_int);
    }
}
static mut byte_90ED50: [uint8; 28] = [
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
];
pub unsafe fn SwappedAmmoRoutine() {
    if byte_90ED50
        [*(&raw mut g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize) as usize]
        != 0
    {
        *((&raw mut g_ram as *mut uint8).offset(0x9c6 as ::core::ffi::c_int as isize)
            as *mut uint16) = (*((&raw mut g_ram as *mut uint8)
            .offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            >> 4 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x9c8 as ::core::ffi::c_int as isize)
            as *mut uint16) = (*((&raw mut g_ram as *mut uint8)
            .offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            >> 4 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x9ca as ::core::ffi::c_int as isize)
            as *mut uint16) = (*((&raw mut g_ram as *mut uint8)
            .offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            >> 4 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0x9cc as ::core::ffi::c_int as isize)
            as *mut uint16) = (*((&raw mut g_ram as *mut uint8)
            .offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            >> 4 as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn UNUSEDsub_90ED6C() {
    *((&raw mut g_ram as *mut uint8).offset(0x9c6 as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0x9e0 as ::core::ffi::c_int as isize)
            as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0x9c8 as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0x9e0 as ::core::ffi::c_int as isize)
            as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0x9ca as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0x9de as ::core::ffi::c_int as isize)
            as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0x9cc as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0x9de as ::core::ffi::c_int as isize)
            as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0x9ce as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0x9dc as ::core::ffi::c_int as isize)
            as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0x9d0 as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0x9dc as ::core::ffi::c_int as isize)
            as *mut uint16);
}
static mut kSamus_FootstepGraphics: [Option<Func_V>; 8] = unsafe {
    [
        Some(Samus_FootstepGraphics_Crateria as unsafe fn() -> ()),
        Some(Samus_FootstepGraphics_1 as unsafe fn() -> ()),
        Some(Samus_FootstepGraphics_1 as unsafe fn() -> ()),
        Some(Samus_FootstepGraphics_1 as unsafe fn() -> ()),
        Some(Samus_FootstepGraphics_Maridia as unsafe fn() -> ()),
        Some(Samus_FootstepGraphics_1 as unsafe fn() -> ()),
        Some(Samus_FootstepGraphics_1 as unsafe fn() -> ()),
        Some(Samus_FootstepGraphics_1 as unsafe fn() -> ()),
    ]
};
pub unsafe fn Samus_FootstepGraphics() {
    kSamus_FootstepGraphics[*((&raw mut g_ram as *mut uint8)
        .offset(0x79f as ::core::ffi::c_int as isize) as *mut uint16)
        as usize]
        .expect("non-null function pointer")();
}
static mut byte_90EDC9: [uint8; 16] = [
    1 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
];
pub unsafe fn Samus_FootstepGraphics_Crateria() {
    let mut current_block: u64;
    if !(*((&raw mut g_ram as *mut uint8).offset(0x1f51 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        != 0
        || (*((&raw mut g_ram as *mut uint8).offset(0x79d as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 16 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int)
    {
        if byte_90EDC9[*((&raw mut g_ram as *mut uint8).offset(0x79d as ::core::ffi::c_int as isize)
            as *mut uint16) as usize] as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            if byte_90EDC9[*((&raw mut g_ram as *mut uint8)
                .offset(0x79d as ::core::ffi::c_int as isize)
                as *mut uint16) as usize] as ::core::ffi::c_int
                & 2 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                if *((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    - 944 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    == 0
                {
                    current_block = 17315823022392333392;
                } else {
                    current_block = 14302341341873304627;
                }
            } else if byte_90EDC9[*((&raw mut g_ram as *mut uint8)
                .offset(0x79d as ::core::ffi::c_int as isize)
                as *mut uint16) as usize] as ::core::ffi::c_int
                & 4 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                current_block = 17315823022392333392;
            } else {
                current_block = 14302341341873304627;
            }
        } else if *((&raw mut g_ram as *mut uint8).offset(0x196e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != 10 as ::core::ffi::c_int
        {
            current_block = 14302341341873304627;
        } else {
            current_block = 17315823022392333392;
        }
        match current_block {
            14302341341873304627 => {}
            _ => {
                Samus_FootstepGraphics_Maridia();
                return;
            }
        }
    }
    Samus_FootstepGraphics_1();
}
pub unsafe fn Samus_FootstepGraphics_Maridia() {
    let mut r18: uint16 = Samus_GetBottom_R18();
    if *((&raw mut g_ram as *mut uint8).offset(0x195e as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        if *((&raw mut g_ram as *mut uint8).offset(0x1962 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
            && *((&raw mut g_ram as *mut uint8).offset(0x1962 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - r18 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
        {
            return;
        }
    } else if *((&raw mut g_ram as *mut uint8).offset(0x195e as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        - r18 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
        && *((&raw mut g_ram as *mut uint8).offset(0x197e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 4 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
    {
        return;
    }
    if *(&raw mut g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == 4 as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0xadc as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(0 as ::core::ffi::c_int as isize) =
            (*((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - 12 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xadc as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(1 as ::core::ffi::c_int as isize) =
            (*((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int) as uint16;
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0xadc as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(0 as ::core::ffi::c_int as isize) =
            (*((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + 12 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xadc as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(1 as ::core::ffi::c_int as isize) =
            (*((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - 8 as ::core::ffi::c_int) as uint16;
    }
    *((&raw mut g_ram as *mut uint8).offset(0xae4 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(0 as ::core::ffi::c_int as isize) = (*((&raw mut g_ram as *mut uint8)
        .offset(0xafa as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        + 16 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xae4 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(1 as ::core::ffi::c_int as isize) = (*((&raw mut g_ram as *mut uint8)
        .offset(0xafa as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        + 16 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xaec as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(0 as ::core::ffi::c_int as isize) = 256 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xaec as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(1 as ::core::ffi::c_int as isize) = 256 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xad4 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(0 as ::core::ffi::c_int as isize) = -(32766 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xad4 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(1 as ::core::ffi::c_int as isize) = 3 as uint16;
}
pub unsafe fn Samus_FootstepGraphics_1() {
    if *((&raw mut g_ram as *mut uint8).offset(0xb3e as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0xff00 as ::core::ffi::c_int
        == 1024 as ::core::ffi::c_int
    {
        let mut r18: uint16 = Samus_GetBottom_R18();
        if *((&raw mut g_ram as *mut uint8).offset(0x195e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            if *((&raw mut g_ram as *mut uint8).offset(0x1962 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
                && *((&raw mut g_ram as *mut uint8).offset(0x1962 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    - r18 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    != 0
            {
                return;
            }
        } else if *((&raw mut g_ram as *mut uint8).offset(0x195e as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - r18 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
            && *((&raw mut g_ram as *mut uint8).offset(0x197e as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 4 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
        {
            return;
        }
        if *(&raw mut g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            == 4 as ::core::ffi::c_int
        {
            *((&raw mut g_ram as *mut uint8).offset(0xadc as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(0 as ::core::ffi::c_int as isize) =
                (*((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    - 12 as ::core::ffi::c_int) as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xadc as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(1 as ::core::ffi::c_int as isize) =
                (*((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int) as uint16;
        } else {
            *((&raw mut g_ram as *mut uint8).offset(0xadc as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(0 as ::core::ffi::c_int as isize) =
                (*((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    + 12 as ::core::ffi::c_int) as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xadc as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(1 as ::core::ffi::c_int as isize) =
                (*((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    - 8 as ::core::ffi::c_int) as uint16;
        }
        *((&raw mut g_ram as *mut uint8).offset(0xae4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(0 as ::core::ffi::c_int as isize) =
            (*((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + 16 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xae4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(1 as ::core::ffi::c_int as isize) =
            (*((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                + 16 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xaec as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(0 as ::core::ffi::c_int as isize) = 1792 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xaec as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(1 as ::core::ffi::c_int as isize) = 1792 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xad4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(0 as ::core::ffi::c_int as isize) = -(32766 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xad4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(1 as ::core::ffi::c_int as isize) = 3 as uint16;
    }
}
pub unsafe fn Samus_UpdateSpeedEchoPos() {
    if *((&raw mut g_ram as *mut uint8).offset(0xb3e as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0xff00 as ::core::ffi::c_int
        == 1024 as ::core::ffi::c_int
        && *((&raw mut g_ram as *mut uint8).offset(0xaae as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        && *((&raw mut g_ram as *mut uint8).offset(0x9da as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 3 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
    {
        let mut v0: uint16 = *((&raw mut g_ram as *mut uint8)
            .offset(0xaae as ::core::ffi::c_int as isize)
            as *mut uint16);
        let mut v1: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
            .offset(0xaae as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
        *((&raw mut g_ram as *mut uint8).offset(0xab0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = *((&raw mut g_ram as *mut uint8)
            .offset(0xaf6 as ::core::ffi::c_int as isize)
            as *mut uint16);
        *((&raw mut g_ram as *mut uint8).offset(0xab8 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = *((&raw mut g_ram as *mut uint8)
            .offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16);
        let mut v2: uint16 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        if (v2 as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        {
            v2 = 0 as uint16;
        }
        *((&raw mut g_ram as *mut uint8).offset(0xaae as ::core::ffi::c_int as isize)
            as *mut uint16) = v2;
    }
}
pub unsafe fn Samus_PostGrappleCollisionDetect() {
    PostGrappleCollisionDetect_X();
    PostGrappleCollisionDetect_Y();
    if *((&raw mut g_ram as *mut uint8).offset(0xe0a as ::core::ffi::c_int as isize) as *mut uint16)
        == 0
        || *((&raw mut g_ram as *mut uint8).offset(0xe08 as ::core::ffi::c_int as isize)
            as *mut uint16)
            == 0
    {
        if *((&raw mut g_ram as *mut uint8).offset(0xe08 as ::core::ffi::c_int as isize)
            as *mut uint16)
            != 0
        {
            let ref mut fresh153 = *((&raw mut g_ram as *mut uint8)
                .offset(0xafa as ::core::ffi::c_int as isize)
                as *mut uint16);
            *fresh153 = (*fresh153 as ::core::ffi::c_int
                - *((&raw mut g_ram as *mut uint8).offset(0xe08 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as uint16;
            if *((&raw mut g_ram as *mut uint8).offset(0xb00 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - 16 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
            {
                PostGrappleCollisionDetect_Y();
                let ref mut fresh154 = *((&raw mut g_ram as *mut uint8)
                    .offset(0xafa as ::core::ffi::c_int as isize)
                    as *mut uint16);
                *fresh154 = (*fresh154 as ::core::ffi::c_int
                    - *((&raw mut g_ram as *mut uint8).offset(0xe08 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int)
                    as uint16;
            }
        }
    }
}
pub unsafe fn ProjPreInstr_UnknownProj8027(mut k: uint16) {
    static mut kProjPreInstr_UnknownProj8027_X: [int16; 4] = [
        -(4 as ::core::ffi::c_int) as int16,
        -(4 as ::core::ffi::c_int) as int16,
        4 as ::core::ffi::c_int as int16,
        4 as ::core::ffi::c_int as int16,
    ];
    static mut kProjPreInstr_UnknownProj8027_Y: [int16; 4] = [
        4 as ::core::ffi::c_int as int16,
        -(4 as ::core::ffi::c_int) as int16,
        -(4 as ::core::ffi::c_int) as int16,
        4 as ::core::ffi::c_int as int16,
    ];
    static mut kProjPreInstr_UnknownProj8027_X2: [int16; 4] = [
        0x80 as ::core::ffi::c_int as int16,
        0x80 as ::core::ffi::c_int as int16,
        -(0x80 as ::core::ffi::c_int) as int16,
        -(0x80 as ::core::ffi::c_int) as int16,
    ];
    static mut kProjPreInstr_UnknownProj8027_Y2: [int16; 4] = [
        -(0x80 as ::core::ffi::c_int) as int16,
        0x80 as ::core::ffi::c_int as int16,
        0x80 as ::core::ffi::c_int as int16,
        -(0x80 as ::core::ffi::c_int) as int16,
    ];
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh28 = *((&raw mut g_ram as *mut uint8)
        .offset(0xb64 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    *fresh28 = (*fresh28 as ::core::ffi::c_int
        + kProjPreInstr_UnknownProj8027_X[v1 as usize] as ::core::ffi::c_int)
        as uint16;
    let mut v2: uint16 = (kProjPreInstr_UnknownProj8027_Y[v1 as usize] as ::core::ffi::c_int
        + *((&raw mut g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) = v2;
    if v2 as ::core::ffi::c_int
        == *((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
    {
        if *((&raw mut g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            == 1 as ::core::ffi::c_int
        {
            if k as ::core::ffi::c_int == 6 as ::core::ffi::c_int {
                *((&raw mut g_ram as *mut uint8).offset(0xa58 as ::core::ffi::c_int as isize)
                    as *mut uint16) =
                    (0x90f04b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            }
            ClearProjectile(k);
        } else {
            let mut v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            let ref mut fresh29 = *((&raw mut g_ram as *mut uint8)
                .offset(0xc7c as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v3 as isize);
            *fresh29 = (*fresh29).wrapping_add(1);
            *((&raw mut g_ram as *mut uint8).offset(0xa68 as ::core::ffi::c_int as isize)
                as *mut uint16) = 180 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xacc as ::core::ffi::c_int as isize)
                as *mut uint16) = 1 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xace as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xb64 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v3 as isize) =
                (kProjPreInstr_UnknownProj8027_X2[v3 as usize] as ::core::ffi::c_int
                    + *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int) as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v3 as isize) =
                (kProjPreInstr_UnknownProj8027_Y2[v3 as usize] as ::core::ffi::c_int
                    + *((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int) as uint16;
        }
    }
}
pub unsafe fn Samus_Func25_ShineSpark() {
    if *((&raw mut g_ram as *mut uint8).offset(0xa68 as ::core::ffi::c_int as isize) as *mut uint16)
        == 0
    {
        *((&raw mut g_ram as *mut uint8).offset(0xace as ::core::ffi::c_int as isize)
            as *mut uint16) = 6 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xad0 as ::core::ffi::c_int as isize)
            as *mut uint16) = 1 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xa80 as ::core::ffi::c_int as isize)
            as *mut uint16) = 1 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xacc as ::core::ffi::c_int as isize)
            as *mut uint16) = 10 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xa68 as ::core::ffi::c_int as isize)
            as *mut uint16) = 120 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xa58 as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x90f072 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Samus_MoveHandler_F072() {
    if *((&raw mut g_ram as *mut uint8).offset(0xa68 as ::core::ffi::c_int as isize) as *mut uint16)
        == 0
    {
        *((&raw mut g_ram as *mut uint8).offset(0xa42 as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x90e695 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xa58 as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x90a337 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
static mut kSamusCodeHandlers: [Option<Func_U8>; 32] = unsafe {
    [
        Some(SamusCode_00_LockSamus as unsafe fn() -> uint8),
        Some(SamusCode_01_UnlockSamus as unsafe fn() -> uint8),
        Some(SamusCode_02_ReachCeresElevator as unsafe fn() -> uint8),
        Some(SamusCode_03 as unsafe fn() -> uint8),
        Some(SamusCode_04 as unsafe fn() -> uint8),
        Some(SamusCode_05_SetupDrained as unsafe fn() -> uint8),
        Some(SamusCode_06_LockToStation as unsafe fn() -> uint8),
        Some(SamusCode_07_SetupForElevator as unsafe fn() -> uint8),
        Some(SamusCode_08_SetupForCeresStart as unsafe fn() -> uint8),
        Some(SamusCode_08_SetupForZebesStart as unsafe fn() -> uint8),
        Some(SamusCode_0A_ClearDrawHandler as unsafe fn() -> uint8),
        Some(SamusCode_0B_DrawHandlerDefault as unsafe fn() -> uint8),
        Some(SamusCode_0C_UnlockFromMapStation as unsafe fn() -> uint8),
        Some(SamusCode_0D_IsGrappleActive as unsafe fn() -> uint8),
        Some(SamusCode_0E as unsafe fn() -> uint8),
        Some(SamusCode_0F_EnableTimerHandling as unsafe fn() -> uint8),
        Some(SamusCode_10 as unsafe fn() -> uint8),
        Some(SamusCode_11_SetupForDeath as unsafe fn() -> uint8),
        Some(SamusCode_12_SetSuperPaletteFlag1 as unsafe fn() -> uint8),
        Some(SamusCode_12_SetSuperPaletteFlag0 as unsafe fn() -> uint8),
        Some(SamusCode_14 as unsafe fn() -> uint8),
        Some(SamusCode_15_CalledBySuitAcquision as unsafe fn() -> uint8),
        Some(SamusCode_16 as unsafe fn() -> uint8),
        Some(SamusCode_17_DisableRainbowSamusAndStandUp as unsafe fn() -> uint8),
        Some(SamusCode_18 as unsafe fn() -> uint8),
        Some(SamusCode_17_FreezeDrainedSamus as unsafe fn() -> uint8),
        Some(SamusCode_1A as unsafe fn() -> uint8),
        Some(SamusCode_1B_CheckedLockSamus as unsafe fn() -> uint8),
        Some(SamusCode_1C as unsafe fn() -> uint8),
        Some(SamusCode_1D_ClearSoundInDoor as unsafe fn() -> uint8),
        Some(SamusCode_1E as unsafe fn() -> uint8),
        Some(SamusCode_1F as unsafe fn() -> uint8),
    ]
};
pub unsafe fn CallSomeSamusCode(mut a: uint16) -> uint16 {
    let mut code: uint16 = kSamusCodeHandlers
        [(a as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int) as usize]
        .expect("non-null function pointer")() as uint16;
    if code as ::core::ffi::c_int & 1 as ::core::ffi::c_int == 0 {
        return (code as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
    }
    *((&raw mut g_ram as *mut uint8).offset(0xa28 as ::core::ffi::c_int as isize) as *mut uint16) =
        -(1 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa2a as ::core::ffi::c_int as isize) as *mut uint16) =
        -(1 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa2c as ::core::ffi::c_int as isize) as *mut uint16) =
        -(1 as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa2e as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa30 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa32 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    return -(1 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Samus_UpdatePreviousPose() {
    *((&raw mut g_ram as *mut uint8).offset(0xa24 as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0xa20 as ::core::ffi::c_int as isize)
            as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0xa26 as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0xa22 as ::core::ffi::c_int as isize)
            as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0xa20 as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0xa22 as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
            as *mut uint16);
}
pub unsafe fn ClearCarry() -> uint8 {
    return 0 as uint8;
}
pub unsafe fn SamusCode_00_LockSamus() -> uint8 {
    *((&raw mut g_ram as *mut uint8).offset(0xa42 as ::core::ffi::c_int as isize) as *mut uint16) =
        (0x90e713 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa44 as ::core::ffi::c_int as isize) as *mut uint16) =
        (0x90e8dc as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    return 1 as uint8;
}
pub unsafe fn SamusCode_01_UnlockSamus() -> uint8 {
    *((&raw mut g_ram as *mut uint8).offset(0xa42 as ::core::ffi::c_int as isize) as *mut uint16) =
        (0x90e695 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa44 as ::core::ffi::c_int as isize) as *mut uint16) =
        (0x90e725 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    return 1 as uint8;
}
pub unsafe fn SamusCode_02_ReachCeresElevator() -> uint8 {
    if *(&raw mut g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == 4 as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) = kPose_02_FaceL_Normal as ::core::ffi::c_int as uint16;
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) = kPose_01_FaceR_Normal as ::core::ffi::c_int as uint16;
    }
    *((&raw mut g_ram as *mut uint8).offset(0xa9a as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    SamusFunc_F433();
    Samus_SetAnimationFrameIfPoseChanged();
    *((&raw mut g_ram as *mut uint8).offset(0xaa0 as ::core::ffi::c_int as isize) as *mut uint16) =
        60 as uint16;
    return SamusCode_00_LockSamus();
}
pub unsafe fn SamusCode_03() -> uint8 {
    if *((&raw mut g_ram as *mut uint8).offset(0xd32 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        != 0x9bc4f0 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0xd32 as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x9bc8c5 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        return 0 as uint8;
    }
    if *(&raw mut g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        != kMovementType_03_SpinJumping as ::core::ffi::c_int
        && *(&raw mut g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            != kMovementType_14_WallJumping as ::core::ffi::c_int
    {
        return 0 as uint8;
    }
    if *(&raw mut g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == kMovementType_04_MorphBallOnGround as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) = kPose_02_FaceL_Normal as ::core::ffi::c_int as uint16;
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) = kPose_01_FaceR_Normal as ::core::ffi::c_int as uint16;
    }
    *((&raw mut g_ram as *mut uint8).offset(0xa9a as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    SamusFunc_F433();
    Samus_SetAnimationFrameIfPoseChanged();
    Samus_UpdatePreviousPose();
    return 1 as uint8;
}
pub unsafe fn SamusCode_04_06_Common() -> uint8 {
    *((&raw mut g_ram as *mut uint8).offset(0xcd0 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    ClearFlareAnimationState();
    Samus_LoadSuitPalette();
    return 0 as uint8;
}
pub unsafe fn SamusCode_04() -> uint8 {
    *((&raw mut g_ram as *mut uint8).offset(0xb62 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    return SamusCode_04_06_Common();
}
pub unsafe fn SamusCode_06_LockToStation() -> uint8 {
    *((&raw mut g_ram as *mut uint8).offset(0xa42 as ::core::ffi::c_int as isize) as *mut uint16) =
        (0x90e713 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa44 as ::core::ffi::c_int as isize) as *mut uint16) =
        (0x90e8d6 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    if *((&raw mut g_ram as *mut uint8).offset(0xcd0 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - 15 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        QueueSfx1_Max15(2 as uint16);
    }
    return SamusCode_04_06_Common();
}
pub unsafe fn SamusCode_07_SetupForElevator() -> uint8 {
    MakeSamusFaceForward();
    *((&raw mut g_ram as *mut uint8).offset(0xa44 as ::core::ffi::c_int as isize) as *mut uint16) =
        (0x90e8ec as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa58 as ::core::ffi::c_int as isize) as *mut uint16) =
        (0x90a337 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa5c as ::core::ffi::c_int as isize) as *mut uint16) =
        (0x90ec14 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa60 as ::core::ffi::c_int as isize) as *mut uint16) =
        (0x90e913 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa56 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    return 1 as uint8;
}
pub unsafe fn SamusCode_08_SetupForCeresStart() -> uint8 {
    *((&raw mut g_ram as *mut uint8).offset(0xa42 as ::core::ffi::c_int as isize) as *mut uint16) =
        (0x90e8cd as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa44 as ::core::ffi::c_int as isize) as *mut uint16) =
        (0x90e8dc as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize) as *mut uint16) =
        kPose_00_FaceF_Powersuit as ::core::ffi::c_int as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa9a as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    SamusFunc_F433();
    Samus_SetAnimationFrameIfPoseChanged();
    Samus_LoadSuitPalette();
    *((&raw mut g_ram as *mut uint8).offset(0xa5c as ::core::ffi::c_int as isize) as *mut uint16) =
        (0x90eb52 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa20 as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0xa24 as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0xa22 as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
            as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0xa26 as ::core::ffi::c_int as isize) as *mut uint16) =
        *((&raw mut g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
            as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    SpawnEprojWithGfx(
        0 as uint16,
        0 as uint16,
        addr_kEproj_CeresElevatorPad as ::core::ffi::c_int as uint16,
    );
    SpawnEprojWithGfx(
        0 as uint16,
        0 as uint16,
        addr_kEproj_CeresElevatorPlatform as ::core::ffi::c_int as uint16,
    );
    *((&raw mut g_ram as *mut uint8).offset(0x5f7 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    PlayRoomMusicTrackAfterAFrames(0x20 as uint16);
    return 1 as uint8;
}
pub unsafe fn SamusCode_08_SetupForZebesStart() -> uint8 {
    if *((&raw mut g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0x20 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        SpawnPalfxObject(addr_stru_8DE1FC as ::core::ffi::c_int as uint16);
        *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) = kPose_9B_FaceF_VariaGravitySuit as ::core::ffi::c_int as uint16;
    } else if *((&raw mut g_ram as *mut uint8).offset(0x9a2 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        SpawnPalfxObject(addr_stru_8DE1F8 as ::core::ffi::c_int as uint16);
        *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) = kPose_9B_FaceF_VariaGravitySuit as ::core::ffi::c_int as uint16;
    } else {
        SpawnPalfxObject(addr_stru_8DE1F4 as ::core::ffi::c_int as uint16);
        *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) = kPose_00_FaceF_Powersuit as ::core::ffi::c_int as uint16;
    }
    Samus_LoadSuitPalette();
    SamusFunc_F433();
    *((&raw mut g_ram as *mut uint8).offset(0xa94 as ::core::ffi::c_int as isize) as *mut uint16) =
        3 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize) as *mut uint16) =
        2 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xdec as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    return 1 as uint8;
}
pub unsafe fn SamusCode_0A_ClearDrawHandler() -> uint8 {
    *((&raw mut g_ram as *mut uint8).offset(0xa5c as ::core::ffi::c_int as isize) as *mut uint16) =
        (0x90e90e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    return 0 as uint8;
}
pub unsafe fn SamusCode_0B_DrawHandlerDefault() -> uint8 {
    *((&raw mut g_ram as *mut uint8).offset(0xa5c as ::core::ffi::c_int as isize) as *mut uint16) =
        (0x90eb52 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    return SamusCode_01_UnlockSamus();
}
pub unsafe fn SamusCode_0C_UnlockFromMapStation() -> uint8 {
    SamusFunc_E633();
    if *((&raw mut g_ram as *mut uint8).offset(0xa44 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        == 0x90e8d6 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0xa42 as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x90e695 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xa44 as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x90e725 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    return 1 as uint8;
}
pub unsafe fn SamusCode_0D_IsGrappleActive() -> uint8 {
    return (if *((&raw mut g_ram as *mut uint8).offset(0xd32 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        != 0x9bc4f0 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        2 as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    }) as uint8;
}
pub unsafe fn SamusCode_0D_IsGrappleActive_A() -> uint8 {
    return (*((&raw mut g_ram as *mut uint8).offset(0xd32 as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        != 0x9bc4f0 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int)
        as ::core::ffi::c_int as uint8;
}
pub unsafe fn SamusCode_0E() -> uint8 {
    *((&raw mut g_ram as *mut uint8).offset(0xa42 as ::core::ffi::c_int as isize) as *mut uint16) =
        (0x90e695 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa44 as ::core::ffi::c_int as isize) as *mut uint16) =
        (0x90e8aa as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    return 1 as uint8;
}
pub unsafe fn SamusCode_0F_EnableTimerHandling() -> uint8 {
    *((&raw mut g_ram as *mut uint8).offset(0xa5a as ::core::ffi::c_int as isize) as *mut uint16) =
        (0x90e0e6 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    return 0 as uint8;
}
pub unsafe fn SamusCode_10() -> uint8 {
    if *((&raw mut g_ram as *mut uint8).offset(0xa44 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        != 0x90e8d9 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0xa42 as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x90e695 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xa44 as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x90e725 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    return 1 as uint8;
}
pub unsafe fn SamusCode_11_15_Common() -> uint8 {
    *((&raw mut g_ram as *mut uint8).offset(0xa42 as ::core::ffi::c_int as isize) as *mut uint16) =
        (0x90e713 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa44 as ::core::ffi::c_int as isize) as *mut uint16) =
        (0x90e8cd as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa5c as ::core::ffi::c_int as isize) as *mut uint16) =
        (0x90ec1d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    return 1 as uint8;
}
pub unsafe fn SamusCode_11_SetupForDeath() -> uint8 {
    DisablePaletteFx();
    return SamusCode_11_15_Common();
}
pub unsafe fn SamusCode_15_CalledBySuitAcquision() -> uint8 {
    Samus_UpdatePreviousPose();
    *((&raw mut g_ram as *mut uint8).offset(0xcd0 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    ClearFlareAnimationState();
    Samus_LoadSuitPalette();
    return SamusCode_11_15_Common();
}
pub unsafe fn SamusCode_12_SetSuperPaletteFlag1() -> uint8 {
    *((&raw mut g_ram as *mut uint8).offset(0xa4a as ::core::ffi::c_int as isize) as *mut uint16) =
        1 as uint16;
    return 0 as uint8;
}
pub unsafe fn SamusCode_12_SetSuperPaletteFlag0() -> uint8 {
    *((&raw mut g_ram as *mut uint8).offset(0xa4a as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    Samus_LoadSuitPalette();
    return 0 as uint8;
}
pub unsafe fn SamusCode_14() -> uint8 {
    if *((&raw mut g_ram as *mut uint8).offset(0x9c2 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        - 31 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        QueueSfx3_Max6(2 as uint16);
    }
    if SamusCode_0D_IsGrappleActive_A() == 0 {
        if *(&raw mut g_ram as *mut uint8).offset(0xa1e as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            == 3 as ::core::ffi::c_int
        {
            if *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                == kPose_81_FaceR_Screwattack as ::core::ffi::c_int
                || *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    == kPose_82_FaceL_Screwattack as ::core::ffi::c_int
            {
                QueueSfx1_Max6(0x33 as uint16);
            } else if *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                == kPose_1B_FaceR_SpaceJump as ::core::ffi::c_int
                || *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    == kPose_1C_FaceL_SpaceJump as ::core::ffi::c_int
            {
                QueueSfx1_Max6(0x3e as uint16);
            } else {
                QueueSfx1_Max6(0x31 as uint16);
            }
        }
    } else {
        QueueSfx1_Max6(6 as uint16);
    }
    return 0 as uint8;
}
pub unsafe fn SamusCode_05_SetupDrained() -> uint8 {
    *((&raw mut g_ram as *mut uint8).offset(0xa5a as ::core::ffi::c_int as isize) as *mut uint16) =
        (0x90e09b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    return Samus_SetupForBeingDrained();
}
pub unsafe fn Samus_SetupForBeingDrained() -> uint8 {
    *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize) as *mut uint16) =
        kPose_54_FaceL_Knockback as ::core::ffi::c_int as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa9a as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    SamusFunc_F433();
    Samus_SetAnimationFrameIfPoseChanged();
    Samus_UpdatePreviousPose();
    *((&raw mut g_ram as *mut uint8).offset(0xcd0 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    ClearFlareAnimationState();
    Samus_LoadSuitPalette();
    *((&raw mut g_ram as *mut uint8).offset(0xa42 as ::core::ffi::c_int as isize) as *mut uint16) =
        (0x90e713 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa44 as ::core::ffi::c_int as isize) as *mut uint16) =
        (0x90e8d9 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    return 1 as uint8;
}
pub unsafe fn SamusCode_18() -> uint8 {
    *((&raw mut g_ram as *mut uint8).offset(0xa5a as ::core::ffi::c_int as isize) as *mut uint16) =
        (0x90e0c5 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    return Samus_SetupForBeingDrained();
}
pub unsafe fn SamusCode_16() -> uint8 {
    *((&raw mut g_ram as *mut uint8).offset(0xa4a as ::core::ffi::c_int as isize) as *mut uint16) =
        0x8000 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xace as ::core::ffi::c_int as isize) as *mut uint16) =
        1 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xad0 as ::core::ffi::c_int as isize) as *mut uint16) =
        1 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb62 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    return 0 as uint8;
}
pub unsafe fn SamusCode_17_DisableRainbowSamusAndStandUp() -> uint8 {
    *((&raw mut g_ram as *mut uint8).offset(0xa4a as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xace as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xad0 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xb62 as ::core::ffi::c_int as isize) as *mut uint16) =
        0 as uint16;
    Samus_LoadSuitPalette();
    *((&raw mut g_ram as *mut uint8).offset(0xa94 as ::core::ffi::c_int as isize) as *mut uint16) =
        1 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize) as *mut uint16) =
        13 as uint16;
    return 0 as uint8;
}
pub unsafe fn SamusCode_17_FreezeDrainedSamus() -> uint8 {
    *((&raw mut g_ram as *mut uint8).offset(0xa94 as ::core::ffi::c_int as isize) as *mut uint16) =
        1 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize) as *mut uint16) =
        28 as uint16;
    return 1 as uint8;
}
pub unsafe fn SamusCode_1A() -> uint8 {
    *((&raw mut g_ram as *mut uint8).offset(0xa44 as ::core::ffi::c_int as isize) as *mut uint16) =
        (0x90e902 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    return 0 as uint8;
}
pub unsafe fn SamusCode_1B_CheckedLockSamus() -> uint8 {
    if *((&raw mut g_ram as *mut uint8).offset(0xa44 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        == 0x90e8d9 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        return 1 as uint8;
    } else {
        return SamusCode_00_LockSamus();
    };
}
pub unsafe fn SamusCode_1C() -> uint8 {
    let mut current_block_5: u64;
    if *(&raw mut g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == kMovementType_14_WallJumping as ::core::ffi::c_int
    {
        if *((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 23 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            if *((&raw mut g_ram as *mut uint8).offset(0xa96 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - 13 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
            {
                current_block_5 = 15182254099572620017;
            } else {
                current_block_5 = 14206663613128494784;
            }
        } else {
            current_block_5 = 11650488183268122163;
        }
    } else {
        if *(&raw mut g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            != kMovementType_03_SpinJumping as ::core::ffi::c_int
        {
            return 0 as uint8;
        }
        if *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != kPose_81_FaceR_Screwattack as ::core::ffi::c_int
            && *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                != kPose_82_FaceL_Screwattack as ::core::ffi::c_int
        {
            if *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                != kPose_1B_FaceR_SpaceJump as ::core::ffi::c_int
                && *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    != kPose_1C_FaceL_SpaceJump as ::core::ffi::c_int
            {
                current_block_5 = 15182254099572620017;
            } else {
                current_block_5 = 14206663613128494784;
            }
        } else {
            current_block_5 = 11650488183268122163;
        }
    }
    match current_block_5 {
        11650488183268122163 => {}
        15182254099572620017 => {
            QueueSfx1_Max9(0x31 as uint16);
            return 0 as uint8;
        }
        _ => {
            QueueSfx1_Max9(0x3e as uint16);
            return 0 as uint8;
        }
    }
    QueueSfx1_Max9(0x33 as uint16);
    return 0 as uint8;
}
pub unsafe fn SamusCode_1D_ClearSoundInDoor() -> uint8 {
    if *(&raw mut g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == 3 as ::core::ffi::c_int
        || *(&raw mut g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            == 20 as ::core::ffi::c_int
    {
        QueueSfx1_Max15(0x32 as uint16);
        return 0 as uint8;
    } else {
        if *((&raw mut g_ram as *mut uint8).offset(0x9b2 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & *((&raw mut g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            if *((&raw mut g_ram as *mut uint8).offset(0xcd0 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - 16 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
            {
                QueueSfx1_Max15(2 as uint16);
            }
        }
        return 0 as uint8;
    };
}
pub unsafe fn SamusCode_1E() -> uint8 {
    if *((&raw mut g_ram as *mut uint8).offset(0x998 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        == 8 as ::core::ffi::c_int
    {
        if *(&raw mut g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            == 3 as ::core::ffi::c_int
            || *(&raw mut g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
                == 20 as ::core::ffi::c_int
        {
            SamusCode_1C();
            return 0 as uint8;
        }
        if *((&raw mut g_ram as *mut uint8).offset(0xcd0 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 16 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            QueueSfx1_Max9(0x41 as uint16);
        }
    }
    return 0 as uint8;
}
pub unsafe fn SamusCode_1F() -> uint8 {
    if *((&raw mut g_ram as *mut uint8).offset(0xd32 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        != addr_loc_90C4F0 as ::core::ffi::c_int as uint16 as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0xd1e as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xd20 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xd34 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xd36 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xa9e as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xcf8 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xcf6 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xcf4 as ::core::ffi::c_int as isize)
            as *mut uint16) = 0 as uint16;
        LoadProjectilePalette(
            *((&raw mut g_ram as *mut uint8).offset(0x9a6 as ::core::ffi::c_int as isize)
                as *mut uint16),
        );
        *((&raw mut g_ram as *mut uint8).offset(0xd32 as ::core::ffi::c_int as isize)
            as *mut uint16) = -(15120 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xa5c as ::core::ffi::c_int as isize)
            as *mut uint16) =
            (0x90eb52 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
    return 0 as uint8;
}
pub unsafe fn Samus_Func26() -> uint8 {
    if *(&raw mut g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        != kMovementType_03_SpinJumping as ::core::ffi::c_int
        && *(&raw mut g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            != kMovementType_14_WallJumping as ::core::ffi::c_int
        && *((&raw mut g_ram as *mut uint8).offset(0x9b2 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & *((&raw mut g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        && *((&raw mut g_ram as *mut uint8).offset(0xcd0 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 16 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
    {
        QueueSfx1_Max9(0x41 as uint16);
    }
    return 0 as uint8;
}
pub unsafe fn Samus_ShootCheck() {
    let mut current_block: u64;
    if *((&raw mut g_ram as *mut uint8).offset(0xdc0 as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        current_block = 635020727918217222;
    } else {
        if *((&raw mut g_ram as *mut uint8).offset(0xdc0 as ::core::ffi::c_int as isize)
            as *mut uint16)
            != 0
        {
            if *((&raw mut g_ram as *mut uint8).offset(0x9b2 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & *((&raw mut g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                QueueSfx1_Max9(0x41 as uint16);
            }
            *((&raw mut g_ram as *mut uint8).offset(0xdc0 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
        }
        if *((&raw mut g_ram as *mut uint8).offset(0xb40 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != 0
            && *((&raw mut g_ram as *mut uint8).offset(0xb3e as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0x400 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
        {
            *((&raw mut g_ram as *mut uint8).offset(0xb40 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            QueueSfx3_Max15(0x25 as uint16);
        }
        if (*(&raw mut g_ram as *mut uint8).offset(0xa11 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            == 3 as ::core::ffi::c_int
            || *(&raw mut g_ram as *mut uint8).offset(0xa11 as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
                == 20 as ::core::ffi::c_int)
            && *(&raw mut g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
                != kMovementType_03_SpinJumping as ::core::ffi::c_int
            && *(&raw mut g_ram as *mut uint8).offset(0xa1f as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
                != kMovementType_14_WallJumping as ::core::ffi::c_int
        {
            QueueSfx1_Max15(0x32 as uint16);
            if *((&raw mut g_ram as *mut uint8).offset(0xcd0 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - 16 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
                && *((&raw mut g_ram as *mut uint8).offset(0x9b2 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    & *((&raw mut g_ram as *mut uint8).offset(0x8b as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int
            {
                current_block = 635020727918217222;
            } else {
                current_block = 2968425633554183086;
            }
        } else {
            current_block = 2968425633554183086;
        }
    }
    match current_block {
        635020727918217222 => {
            *((&raw mut g_ram as *mut uint8).offset(0xdc0 as ::core::ffi::c_int as isize)
                as *mut uint16) = 1 as uint16;
        }
        _ => {}
    }
    if *((&raw mut g_ram as *mut uint8).offset(0x5d1 as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        if *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == kPose_00_FaceF_Powersuit as ::core::ffi::c_int
            || *((&raw mut g_ram as *mut uint8).offset(0xa1c as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                == kPose_9B_FaceF_VariaGravitySuit as ::core::ffi::c_int
        {
            if *((&raw mut g_ram as *mut uint8).offset(0x8d as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 0x30 as ::core::ffi::c_int
                == 48 as ::core::ffi::c_int
                && *((&raw mut g_ram as *mut uint8).offset(0x91 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    & 0x80 as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int
            {
                *((&raw mut g_ram as *mut uint8).offset(0xde0 as ::core::ffi::c_int as isize)
                    as *mut uint16) = 7 as uint16;
            }
        } else {
            if *((&raw mut g_ram as *mut uint8).offset(0xde0 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - 7 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
            {
                return;
            }
            *((&raw mut g_ram as *mut uint8).offset(0xde0 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
        }
    }
    if CheckEventHappened(0xe as uint16) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
        && *((&raw mut g_ram as *mut uint8).offset(0xa5a as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            == 0x90e114 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
        && *((&raw mut g_ram as *mut uint8).offset(0x998 as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            != kGameState_35_TimeUp as ::core::ffi::c_int
    {
        *((&raw mut g_ram as *mut uint8).offset(0x998 as ::core::ffi::c_int as isize)
            as *mut uint16) = kGameState_35_TimeUp as ::core::ffi::c_int as uint16;
    }
}
